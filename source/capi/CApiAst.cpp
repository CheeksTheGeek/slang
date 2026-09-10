//------------------------------------------------------------------------------
// CApiAst.cpp
// C API: compilation, freezing, symbols, scopes, types, expressions
//
// SPDX-FileCopyrightText: Michael Popoloski
// SPDX-License-Identifier: MIT
//------------------------------------------------------------------------------
#include "CApiInternal.h"

#include <ranges>

#include "slang/ast/ASTContext.h"
#include "slang/ast/ASTVisitor.h"
#include "slang/ast/Constraints.h"
#include "slang/ast/Lookup.h"
#include "slang/ast/Patterns.h"
#include "slang/ast/Statement.h"
#include "slang/ast/TimingControl.h"
#include "slang/ast/expressions/AssertionExpr.h"
#include "slang/ast/expressions/AssignmentExpressions.h"
#include "slang/ast/expressions/CallExpression.h"
#include "slang/ast/expressions/ConversionExpression.h"
#include "slang/ast/expressions/Operator.h"
#include "slang/ast/expressions/OperatorExpressions.h"
#include "slang/ast/expressions/SelectExpressions.h"
#include "slang/ast/statements/ConditionalStatements.h"
#include "slang/ast/statements/LoopStatements.h"
#include "slang/ast/statements/MiscStatements.h"
#include "slang/ast/symbols/BlockSymbols.h"
#include "slang/ast/symbols/ClassSymbols.h"
#include "slang/ast/symbols/CompilationUnitSymbols.h"
#include "slang/ast/symbols/CoverSymbols.h"
#include "slang/ast/symbols/AttributeSymbol.h"
#include "slang/ast/symbols/CheckerSymbols.h"
#include "slang/ast/symbols/InstanceSymbols.h"
#include "slang/ast/symbols/MemberSymbols.h"
#include "slang/ast/symbols/ParameterSymbols.h"
#include "slang/ast/symbols/PortSymbols.h"
#include "slang/ast/symbols/SpecifySymbols.h"
#include "slang/ast/symbols/SubroutineSymbols.h"
#include "slang/ast/symbols/ValueSymbol.h"
#include "slang/ast/symbols/VariableSymbols.h"
#include "slang/ast/types/AllTypes.h"
#include "slang/ast/types/NetType.h"
#include "slang/ast/types/Type.h"

using namespace slang;
using namespace slang::ast;
using namespace slang::capi;

// ---- Helpers ----------------------------------------------------------------

namespace slang::capi {

slang_syntax_tree findTree(slang_compilation comp, const syntax::SyntaxNode& node) {
    const syntax::SyntaxNode* root = &node;
    while (auto p = root->parent.get())
        root = p;
    for (auto tree : comp->trees) {
        if (&tree->tree->root() == root)
            return tree;
    }
    return nullptr;
}

} // namespace slang::capi

template<typename T>
static slang_ast wrapAst(const T& t, slang_compilation comp) {
    if constexpr (std::is_base_of_v<Symbol, T>)
        return slang_ast{&t, comp, (uint32_t)t.kind, SLANG_AST_SYMBOL};
    else if constexpr (std::is_base_of_v<Expression, T>)
        return slang_ast{&t, comp, (uint32_t)t.kind, SLANG_AST_EXPRESSION};
    else if constexpr (std::is_base_of_v<Statement, T>)
        return slang_ast{&t, comp, (uint32_t)t.kind, SLANG_AST_STATEMENT};
    else if constexpr (std::is_base_of_v<TimingControl, T>)
        return slang_ast{&t, comp, (uint32_t)t.kind, SLANG_AST_TIMING_CONTROL};
    else if constexpr (std::is_base_of_v<Constraint, T>)
        return slang_ast{&t, comp, (uint32_t)t.kind, SLANG_AST_CONSTRAINT};
    else if constexpr (std::is_base_of_v<AssertionExpr, T>)
        return slang_ast{&t, comp, (uint32_t)t.kind, SLANG_AST_ASSERTION_EXPR};
    else if constexpr (std::is_base_of_v<BinsSelectExpr, T>)
        return slang_ast{&t, comp, (uint32_t)t.kind, SLANG_AST_BINS_SELECT_EXPR};
    else if constexpr (std::is_base_of_v<Pattern, T>)
        return slang_ast{&t, comp, (uint32_t)t.kind, SLANG_AST_PATTERN};
    else
        static_assert(sizeof(T) == 0, "unknown AST node family");
}

static const Scope* scopeOf(const Symbol& sym) {
    if (sym.kind == SymbolKind::Instance)
        return &sym.as<InstanceSymbol>().body;
    if (sym.kind == SymbolKind::CheckerInstance)
        return &sym.as<CheckerInstanceSymbol>().body;
    return sym.isScope() ? &sym.as<Scope>() : nullptr;
}

static const Type* typeOf(slang_ast node) {
    auto sym = symbolOf(node);
    return sym && sym->isType() ? &sym->as<Type>() : nullptr;
}

static const Statement* stmtOf(slang_ast node) {
    return fromC<Statement>(node, SLANG_AST_STATEMENT);
}

static void ensureRoot(slang_compilation comp) {
    comp->comp->getRoot();
}

// ---- Compilation ------------------------------------------------------------

slang_compilation slang_compilation_create(slang_options options, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, { return new slang_compilation_t(options ? options->toBag() : Bag()); });
    return nullptr;
}

void slang_compilation_destroy(slang_compilation comp) {
    if (!comp)
        return;
    for (auto tree : comp->trees)
        slang_syntax_tree_release(tree);
    delete comp;
}

void slang_compilation_add_tree(slang_compilation comp, slang_syntax_tree tree, slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!comp || !tree) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return;
    }
    if (comp->comp->isFinalized()) {
        setError(err, SLANG_ERR_INVALID_STATE, "compilation is already finalized");
        return;
    }
    if (comp->sm && comp->sm != tree->sm) {
        setError(err, SLANG_ERR_INVALID_ARG,
                 "all trees in a compilation must share one source manager");
        return;
    }
    SLANG_C_GUARD(err, {
        comp->comp->addSyntaxTree(tree->tree);
        comp->trees.push_back(slang_syntax_tree_retain(tree));
        comp->sm = tree->sm;
    });
}

namespace {

// The freeze sweep: visits every reachable AST node, forcing lazy elaboration
// along the way, and optionally constant-folds every expression it meets.
struct FreezeVisitor : public ASTVisitor<FreezeVisitor, VisitFlags::AllGood> {
    slang_freeze_report& report;
    bool prefold;
    const Scope* currentScope = nullptr;

    FreezeVisitor(slang_freeze_report& report, bool prefold) : report(report), prefold(prefold) {}

    template<typename T>
    void handle(const T& t) {
        if constexpr (std::is_base_of_v<Symbol, T>) {
            report.symbols_elaborated++;
            // Force every lazily-resolved declared type + initializer so that a
            // later read on a frozen, shared &Design never mutates. getDeclaredType()
            // covers ALL carriers (ValueSymbol, TypeAlias, Subroutine, MethodPrototype,
            // NetType, TypeParameter, AssertionPort, RandSeqProduction, Coverpoint) —
            // the previous ValueSymbol-only path missed the last eight, and left the
            // alias-typed members' canonical memo unforced.
            if (auto dt = t.getDeclaredType()) {
                auto& resolved = dt->getType();
                dt->getInitializer();
                // Force the canonical-type memo (mutable Type::canonical): a no-op
                // for non-aliases, but a TypeAlias resolves it lazily via a
                // non-atomic write that would otherwise race on first concurrent read.
                resolved.getCanonicalType();
                report.types_canonicalized++;
            }
            // A visited Type node (e.g. a TypeAliasType symbol) must have its OWN
            // canonical forced too — types are interned, so every reference through a
            // Type<'d> handle shares this object.
            if constexpr (std::is_base_of_v<Type, T>) {
                t.getCanonicalType();
                report.types_canonicalized++;
            }
            // Parameter/specparam/enum values allocate a ConstantValue from the
            // arena on first read; force them here (pre-seal) so the accessors
            // (slang_parameter_value / slang_enum_member_value) are pure reads on
            // the shared, frozen &Design and never mutate. (SOUNDNESS-MEMOS.md:
            // a new &Design accessor reaching such a memo needs a matching
            // pre-seal force, exactly like these.)
            if constexpr (std::is_same_v<T, ParameterSymbol> ||
                          std::is_same_v<T, SpecparamSymbol> ||
                          std::is_same_v<T, EnumValueSymbol>) {
                t.getValue();
                report.params_folded++;
            }
            // Preemptively force the remaining lazily-resolved memos that no
            // current C accessor reaches but a future one might, so the
            // `Design: Sync` guarantee no longer rests on the accessor surface
            // staying narrow (see SOUNDNESS-MEMOS.md "B-latent"). Running them
            // here, pre-seal, means any accessor that later exposes them reads a
            // memo already resolved on a frozen, shared &Design — never mutates.
            if constexpr (std::is_same_v<T, PortSymbol>) {
                t.getType();
                t.getInternalExpr();
            }
            else if constexpr (std::is_same_v<T, MultiPortSymbol>) {
                t.getType();
            }
            else if constexpr (std::is_same_v<T, NetType>) {
                t.getResolutionFunction();
            }
            else if constexpr (std::is_same_v<T, ClassType>) {
                t.getBaseClass();
                t.getBaseConstructorCall();
                t.getBitstreamWidth();
                t.hasCycles();
            }
            // Coverage memos: covergroups appear in real RTL, so force their
            // lazily-resolved bin/point/cross expressions too (SOUNDNESS-MEMOS.md).
            else if constexpr (std::is_same_v<T, CoverageBinSymbol>) {
                t.getIffExpr();
                t.getNumberOfBinsExpr();
                t.getSetCoverageExpr();
                t.getWithExpr();
                t.getCrossSelectExpr();
                t.getValues();
            }
            else if constexpr (std::is_same_v<T, CoverpointSymbol>) {
                t.getCoverageExpr();
                t.getIffExpr();
            }
            else if constexpr (std::is_same_v<T, CovergroupType>) {
                t.getCoverageEvent();
                t.getBaseGroup();
            }
            else if constexpr (std::is_same_v<T, ClockingBlockSymbol>) {
                t.getEvent();
                t.getDefaultInputSkew();
                t.getDefaultOutputSkew();
            }
            else if constexpr (std::is_same_v<T, CoverCrossSymbol>) {
                t.getIffExpr();
            }
            // Continuous assignments are ubiquitous; force their assignment +
            // delay expressions.
            else if constexpr (std::is_same_v<T, ContinuousAssignSymbol>) {
                t.getAssignment();
                t.getDelay();
            }
            // Elaboration system tasks ($error/$info/... with a condition).
            else if constexpr (std::is_same_v<T, ElabSystemTaskSymbol>) {
                t.getMessage();
                t.getAssertCondition();
            }
            // `alias` statements resolve their net references lazily.
            else if constexpr (std::is_same_v<T, NetAliasSymbol>) {
                t.getNetReferences();
            }
            // specify-block contents: one getter triggers the shared resolve().
            else if constexpr (std::is_same_v<T, TimingPathSymbol>) {
                t.getInputs();
            }
            else if constexpr (std::is_same_v<T, PulseStyleSymbol>) {
                t.getTerminals();
            }
            else if constexpr (std::is_same_v<T, SystemTimingCheckSymbol>) {
                t.getArguments();
            }
            // Checker instance output-port initial expressions (per connection).
            else if constexpr (std::is_same_v<T, CheckerInstanceSymbol>) {
                for (auto& conn : t.getPortConnections())
                    conn.getOutputInitialExpr();
            }
            // Nets carry a lazily-resolved delay control (`wire #2 w;`).
            else if constexpr (std::is_same_v<T, NetSymbol>) {
                t.getDelay();
            }
            // Subroutine formal-argument default values.
            else if constexpr (std::is_same_v<T, FormalArgumentSymbol>) {
                t.getDefaultValue();
            }
            // Module-instance port connections: getPortConnections() resolves the
            // connection list, and each connection's expression is lazy too.
            else if constexpr (std::is_same_v<T, InstanceSymbol>) {
                for (auto* pc : t.getPortConnections()) {
                    if (pc)
                        pc->getExpression();
                }
            }
            // Gate/UDP primitive instances: port expressions + delay.
            else if constexpr (std::is_same_v<T, PrimitiveInstanceSymbol>) {
                t.getPortConnections();
                t.getDelay();
            }
            // Interface ports resolve their connection (+ expression, range) lazily.
            else if constexpr (std::is_same_v<T, InterfacePortSymbol>) {
                t.getConnectionAndExpr();
                t.getDeclaredRange();
            }
            // Attribute constant values.
            else if constexpr (std::is_same_v<T, AttributeSymbol>) {
                t.getValue();
            }
            // A generic (parameterized) class's default specialization.
            else if constexpr (std::is_same_v<T, GenericClassDefSymbol>) {
                if (auto* sc = t.getParentScope())
                    t.getDefaultSpecialization(*sc);
            }
            auto saved = currentScope;
            if (auto scope = scopeOf(t))
                currentScope = scope;
            visitDefault(t);
            currentScope = saved;
        }
        else if constexpr (std::is_base_of_v<Expression, T>) {
            report.expressions_visited++;
            if (prefold)
                fold(t);
            // Defensive: force the canonical of the expression's OWN type so the
            // slang_expression_type() -> type-predicate path is race-free by
            // direct force, not merely by type-interning (a fragile argument).
            // A no-op for non-aliases; never allocates. (type is not_null.)
            t.type->getCanonicalType();
            visitDefault(t);
        }
        else {
            visitDefault(t);
        }
    }

    void fold(const Expression& expr) {
        if (expr.bad()) {
            report.expressions_folded++;
            return;
        }
        if (!expr.getConstant()) {
            if (!currentScope) {
                report.fold_failures++;
                return;
            }
            SLANG_TRY {
                ASTContext ctx(*currentScope, LookupLocation::max);
                ctx.tryEval(expr);
            }
            SLANG_CATCH(...) {
                report.fold_failures++;
                return;
            }
        }
        if (expr.getConstant())
            report.expressions_folded++;
    }
};

// Visits the AST on behalf of slang_ast_visit, tracking the parent chain.
struct CallbackVisitor : public ASTVisitor<CallbackVisitor, VisitFlags::AllGood> {
    slang_compilation comp;
    slang_ast_visitor visitor;
    void* user;
    slang_ast parent;
    bool stopped = false;

    CallbackVisitor(slang_compilation comp, slang_ast_visitor visitor, void* user) :
        comp(comp), visitor(visitor), user(user), parent(noAst(comp)) {}

    template<typename T>
    void handle(const T& t) {
        if (stopped)
            return;
        slang_ast current = wrapAst(t, comp);
        switch (visitor(current, parent, user)) {
            case SLANG_VISIT_BREAK:
                stopped = true;
                return;
            case SLANG_VISIT_SKIP:
                return;
            case SLANG_VISIT_CONTINUE:
                break;
        }
        auto saved = parent;
        parent = current;
        visitDefault(t);
        parent = saved;
    }
};

} // namespace

void slang_compilation_freeze(slang_compilation comp, uint32_t flags, slang_freeze_report* report,
                              slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!comp) {
        setError(err, SLANG_ERR_INVALID_ARG, "null compilation");
        return;
    }
    if ((flags & SLANG_FREEZE_ELABORATE_ALL) &&
        !comp->comp->hasFlag(CompilationFlags::DisableInstanceCaching)) {
        setError(err, SLANG_ERR_INVALID_ARG,
                 "SLANG_FREEZE_ELABORATE_ALL requires SLANG_COMP_DISABLE_INSTANCE_CACHING");
        return;
    }
    SLANG_C_GUARD(err, {
        slang_freeze_report local{};
        auto& r = report ? *report : local;
        r = slang_freeze_report{};

        bool wasSealed = comp->sealed;
        if (wasSealed)
            comp->comp->unfreeze();

        comp->comp->getAllDiagnostics();

        if (flags & (SLANG_FREEZE_ELABORATE_ALL | SLANG_FREEZE_PREFOLD)) {
            FreezeVisitor visitor(r, (flags & SLANG_FREEZE_PREFOLD) != 0);
            comp->comp->getRoot().visit(visitor);
            if (flags & SLANG_FREEZE_ELABORATE_ALL)
                comp->elaboratedAll = true;
        }

        if ((flags & SLANG_FREEZE_SEAL) || wasSealed) {
            comp->comp->freeze();
            comp->sealed = true;
        }
    });
}

bool slang_compilation_is_sealed(slang_compilation comp) {
    return comp && comp->sealed;
}

slang_ast slang_compilation_root(slang_compilation comp, slang_error* err) {
    if (!checkEntry(err))
        return noAst(comp);
    if (!comp) {
        setError(err, SLANG_ERR_INVALID_ARG, "null compilation");
        return noAst(comp);
    }
    SLANG_C_GUARD(err, { return capi::toC(&comp->comp->getRoot(), comp); });
    return noAst(comp);
}

slang_diagnostics slang_compilation_diagnostics(slang_compilation comp, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    if (!comp) {
        setError(err, SLANG_ERR_INVALID_ARG, "null compilation");
        return nullptr;
    }
    if (!comp->sm) {
        setError(err, SLANG_ERR_INVALID_STATE, "compilation has no syntax trees");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        bool wasSealed = comp->sealed;
        if (wasSealed && !comp->comp->isElaborated())
            comp->comp->unfreeze();
        auto& diags = comp->comp->getAllDiagnostics();
        if (wasSealed)
            comp->comp->freeze();

        auto result = new slang_diagnostics_t();
        result->sm = comp->sm;
        result->comp = comp;
        result->diags.assign(diags.begin(), diags.end());
        return result;
    });
    return nullptr;
}

slang_source_manager slang_compilation_source_manager(slang_compilation comp) {
    return comp ? comp->sm : nullptr;
}

uint32_t slang_compilation_top_instance_count(slang_compilation comp) {
    if (!comp)
        return 0;
    SLANG_C_ACCESS(0, {
        ensureRoot(comp);
        return (uint32_t)comp->comp->getRoot().topInstances.size();
    });
}

slang_ast slang_compilation_top_instance(slang_compilation comp, uint32_t index) {
    if (!comp)
        return noAst(comp);
    SLANG_C_ACCESS(noAst(comp), {
        ensureRoot(comp);
        auto tops = comp->comp->getRoot().topInstances;
        if (index >= tops.size())
            return noAst(comp);
        return capi::toC(tops[index], comp);
    });
}

uint32_t slang_compilation_definition_count(slang_compilation comp) {
    if (!comp)
        return 0;
    SLANG_C_ACCESS(0, { return (uint32_t)comp->comp->getDefinitions().size(); });
}

slang_ast slang_compilation_definition(slang_compilation comp, uint32_t index) {
    if (!comp)
        return noAst(comp);
    SLANG_C_ACCESS(noAst(comp), {
        auto defs = comp->comp->getDefinitions();
        if (index >= defs.size())
            return noAst(comp);
        return capi::toC(defs[index], comp);
    });
}

uint32_t slang_compilation_package_count(slang_compilation comp) {
    if (!comp)
        return 0;
    SLANG_C_ACCESS(0, { return (uint32_t)comp->comp->getPackages().size(); });
}

slang_ast slang_compilation_package(slang_compilation comp, uint32_t index) {
    if (!comp)
        return noAst(comp);
    SLANG_C_ACCESS(noAst(comp), {
        auto pkgs = comp->comp->getPackages();
        if (index >= pkgs.size())
            return noAst(comp);
        return capi::toC(pkgs[index], comp);
    });
}

// ---- AST: generic -----------------------------------------------------------

bool slang_ast_is_null(slang_ast node) {
    return node.ptr == nullptr;
}

slang_range slang_ast_range(slang_ast node) {
    if (!node.ptr)
        return slang_range{};
    SLANG_C_ACCESS(slang_range{}, {
    switch (node.domain) {
        case SLANG_AST_SYMBOL: {
            auto& sym = *static_cast<const Symbol*>(node.ptr);
            if (auto syntax = sym.getSyntax())
                return capi::toC(syntax->sourceRange());
            return capi::toC(SourceRange(sym.location, sym.location));
        }
        case SLANG_AST_EXPRESSION:
            return capi::toC(static_cast<const Expression*>(node.ptr)->sourceRange);
        case SLANG_AST_STATEMENT:
            return capi::toC(static_cast<const Statement*>(node.ptr)->sourceRange);
        case SLANG_AST_TIMING_CONTROL:
            return capi::toC(static_cast<const TimingControl*>(node.ptr)->sourceRange);
        case SLANG_AST_PATTERN:
            return capi::toC(static_cast<const Pattern*>(node.ptr)->sourceRange);
        case SLANG_AST_CONSTRAINT:
            if (auto s = static_cast<const Constraint*>(node.ptr)->syntax)
                return capi::toC(s->sourceRange());
            return slang_range{};
        case SLANG_AST_ASSERTION_EXPR:
            if (auto s = static_cast<const AssertionExpr*>(node.ptr)->syntax)
                return capi::toC(s->sourceRange());
            return slang_range{};
        case SLANG_AST_BINS_SELECT_EXPR:
            if (auto s = static_cast<const BinsSelectExpr*>(node.ptr)->syntax)
                return capi::toC(s->sourceRange());
            return slang_range{};
    }
    return slang_range{};
    });
}

slang_node slang_ast_syntax(slang_ast node) {
    SLANG_C_ACCESS(noNode(nullptr), {
    const syntax::SyntaxNode* syntax = nullptr;
    if (node.ptr) {
        switch (node.domain) {
            case SLANG_AST_SYMBOL:
                syntax = static_cast<const Symbol*>(node.ptr)->getSyntax();
                break;
            case SLANG_AST_EXPRESSION:
                syntax = static_cast<const Expression*>(node.ptr)->syntax;
                break;
            case SLANG_AST_STATEMENT:
                syntax = static_cast<const Statement*>(node.ptr)->syntax;
                break;
            case SLANG_AST_TIMING_CONTROL:
                syntax = static_cast<const TimingControl*>(node.ptr)->syntax;
                break;
            case SLANG_AST_CONSTRAINT:
                syntax = static_cast<const Constraint*>(node.ptr)->syntax;
                break;
            case SLANG_AST_ASSERTION_EXPR:
                syntax = static_cast<const AssertionExpr*>(node.ptr)->syntax;
                break;
            case SLANG_AST_BINS_SELECT_EXPR:
                syntax = static_cast<const BinsSelectExpr*>(node.ptr)->syntax;
                break;
            case SLANG_AST_PATTERN:
                syntax = static_cast<const Pattern*>(node.ptr)->syntax;
                break;
        }
    }
    if (!syntax || !node.compilation)
        return noNode(nullptr);
    auto tree = findTree(node.compilation, *syntax);
    return tree ? capi::toC(syntax, tree) : noNode(nullptr);
    });
}

void slang_ast_visit(slang_ast root, slang_ast_visitor visitor, void* user, slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!root.ptr || !visitor || !root.compilation) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return;
    }
    SLANG_C_GUARD(err, {
        CallbackVisitor v(root.compilation, visitor, user);
        switch (root.domain) {
            case SLANG_AST_SYMBOL:
                static_cast<const Symbol*>(root.ptr)->visit(v);
                break;
            case SLANG_AST_EXPRESSION:
                static_cast<const Expression*>(root.ptr)->visit(v);
                break;
            case SLANG_AST_STATEMENT:
                static_cast<const Statement*>(root.ptr)->visit(v);
                break;
            case SLANG_AST_TIMING_CONTROL:
                static_cast<const TimingControl*>(root.ptr)->visit(v);
                break;
            case SLANG_AST_CONSTRAINT:
                static_cast<const Constraint*>(root.ptr)->visit(v);
                break;
            case SLANG_AST_ASSERTION_EXPR:
                static_cast<const AssertionExpr*>(root.ptr)->visit(v);
                break;
            case SLANG_AST_BINS_SELECT_EXPR:
                static_cast<const BinsSelectExpr*>(root.ptr)->visit(v);
                break;
            case SLANG_AST_PATTERN:
                static_cast<const Pattern*>(root.ptr)->visit(v);
                break;
            default:
                setError(err, SLANG_ERR_INVALID_ARG, "invalid AST domain");
                return;
        }
        if (v.stopped)
            setError(err, SLANG_ERR_CANCELLED, "traversal cancelled by visitor");
    });
}

// ---- AST: symbols and scopes ------------------------------------------------

slang_str slang_symbol_name(slang_ast symbol) {
    SLANG_C_ACCESS(borrowed(""), {
        auto sym = symbolOf(symbol);
        return sym ? borrowed(sym->name) : borrowed("");
    });
}

slang_loc slang_symbol_location(slang_ast symbol) {
    SLANG_C_ACCESS(slang_loc{}, {
        auto sym = symbolOf(symbol);
        return sym ? capi::toC(sym->location) : slang_loc{};
    });
}

slang_ast slang_symbol_parent_scope(slang_ast symbol) {
    SLANG_C_ACCESS(noAst(symbol.compilation), {
        auto sym = symbolOf(symbol);
        if (!sym || !sym->getParentScope())
            return noAst(symbol.compilation);
        return capi::toC(&sym->getParentScope()->asSymbol(), symbol.compilation);
    });
}

slang_ast slang_symbol_next_sibling(slang_ast symbol) {
    SLANG_C_ACCESS(noAst(symbol.compilation), {
        auto sym = symbolOf(symbol);
        return capi::toC(sym ? sym->getNextSibling() : nullptr, symbol.compilation);
    });
}

slang_str slang_symbol_hierarchical_path(slang_ast symbol, slang_error* err) {
    if (!checkEntry(err))
        return borrowed("");
    auto sym = symbolOf(symbol);
    if (!sym) {
        setError(err, SLANG_ERR_INVALID_ARG, "not a symbol");
        return borrowed("");
    }
    SLANG_C_GUARD(err, { return owned(sym->getHierarchicalPath()); });
    return borrowed("");
}

bool slang_symbol_is_scope(slang_ast symbol) {
    SLANG_C_ACCESS(false, {
        auto sym = symbolOf(symbol);
        return sym && scopeOf(*sym) != nullptr;
    });
}

bool slang_symbol_is_type(slang_ast symbol) {
    SLANG_C_ACCESS(false, {
        auto sym = symbolOf(symbol);
        return sym && sym->isType();
    });
}

bool slang_symbol_is_value(slang_ast symbol) {
    SLANG_C_ACCESS(false, {
        auto sym = symbolOf(symbol);
        return sym && sym->isValue();
    });
}

slang_ast slang_scope_first_member(slang_ast scope, slang_error* err) {
    if (!checkEntry(err))
        return noAst(scope.compilation);
    auto sym = symbolOf(scope);
    auto s = sym ? scopeOf(*sym) : nullptr;
    if (!s) {
        setError(err, SLANG_ERR_INVALID_ARG, "not a scope");
        return noAst(scope.compilation);
    }
    SLANG_C_GUARD(err, {
        auto members = s->members();
        return capi::toC(members.begin() == members.end() ? nullptr : &*members.begin(),
                         scope.compilation);
    });
    return noAst(scope.compilation);
}

slang_ast slang_scope_find(slang_ast scope, const char* name, size_t name_len, slang_error* err) {
    if (!checkEntry(err))
        return noAst(scope.compilation);
    auto sym = symbolOf(scope);
    auto s = sym ? scopeOf(*sym) : nullptr;
    if (!s || !name) {
        setError(err, SLANG_ERR_INVALID_ARG, s ? "null name" : "not a scope");
        return noAst(scope.compilation);
    }
    SLANG_C_GUARD(err, { return capi::toC(s->find(toView(name, name_len)), scope.compilation); });
    return noAst(scope.compilation);
}

slang_ast slang_scope_lookup(slang_ast scope, const char* name, size_t name_len, slang_error* err) {
    if (!checkEntry(err))
        return noAst(scope.compilation);
    auto sym = symbolOf(scope);
    auto s = sym ? scopeOf(*sym) : nullptr;
    if (!s || !name) {
        setError(err, SLANG_ERR_INVALID_ARG, s ? "null name" : "not a scope");
        return noAst(scope.compilation);
    }
    SLANG_C_GUARD(err, {
        // The first segment uses full unqualified lookup rules; each further
        // segment is a direct member lookup in the scope the previous one
        // denotes. Scope::lookupName is deliberately not used: it parses the
        // name into a syntax tree allocated in the compilation's arena. Even
        // Lookup::unqualified records reference-tracking state for lint
        // purposes (Compilation::noteReference), so the seal is lifted for
        // the call; the header documents the exclusive-access requirement.
        SealLift lift(scope.compilation);
        auto path = toView(name, name_len);
        auto dot = path.find('.');
        const Symbol* found = Lookup::unqualified(*s, path.substr(0, dot));
        while (found && dot != std::string_view::npos) {
            path = path.substr(dot + 1);
            dot = path.find('.');
            auto next = scopeOf(*found);
            found = next ? next->find(path.substr(0, dot)) : nullptr;
        }
        return capi::toC(found, scope.compilation);
    });
    return noAst(scope.compilation);
}

slang_ast slang_value_type(slang_ast symbol, slang_error* err) {
    if (!checkEntry(err))
        return noAst(symbol.compilation);
    auto sym = symbolOf(symbol);
    if (!sym || !sym->isValue()) {
        setError(err, SLANG_ERR_INVALID_ARG, "not a value symbol");
        return noAst(symbol.compilation);
    }
    SLANG_C_GUARD(err,
                  { return capi::toC(&sym->as<ValueSymbol>().getType(), symbol.compilation); });
    return noAst(symbol.compilation);
}

slang_ast slang_value_initializer(slang_ast symbol, slang_error* err) {
    if (!checkEntry(err))
        return noAst(symbol.compilation, SLANG_AST_EXPRESSION);
    auto sym = symbolOf(symbol);
    if (!sym || !sym->isValue()) {
        setError(err, SLANG_ERR_INVALID_ARG, "not a value symbol");
        return noAst(symbol.compilation, SLANG_AST_EXPRESSION);
    }
    SLANG_C_GUARD(err, {
        return capi::toC(sym->as<ValueSymbol>().getInitializer(), symbol.compilation);
    });
    return noAst(symbol.compilation, SLANG_AST_EXPRESSION);
}

slang_ast slang_instance_body(slang_ast instance) {
    SLANG_C_ACCESS(noAst(instance.compilation), {
        auto sym = symbolOf(instance);
        if (!sym || sym->kind != SymbolKind::Instance)
            return noAst(instance.compilation);
        return capi::toC(&sym->as<InstanceSymbol>().body, instance.compilation);
    });
}

slang_ast slang_instance_definition(slang_ast instance) {
    SLANG_C_ACCESS(noAst(instance.compilation), {
        auto sym = symbolOf(instance);
        if (!sym || sym->kind != SymbolKind::Instance)
            return noAst(instance.compilation);
        return capi::toC(&sym->as<InstanceSymbol>().getDefinition(), instance.compilation);
    });
}

slang_definition_kind slang_definition_kind_of(slang_ast definition) {
    SLANG_C_ACCESS(SLANG_DEFINITION_MODULE, {
    auto sym = symbolOf(definition);
    if (!sym || sym->kind != SymbolKind::Definition)
        return SLANG_DEFINITION_MODULE;
    switch (sym->as<DefinitionSymbol>().definitionKind) {
        case DefinitionKind::Module:
            return SLANG_DEFINITION_MODULE;
        case DefinitionKind::Interface:
            return SLANG_DEFINITION_INTERFACE;
        case DefinitionKind::Program:
            return SLANG_DEFINITION_PROGRAM;
    }
    return SLANG_DEFINITION_MODULE;
    });
}

uint32_t slang_instance_parameter_count(slang_ast instance) {
    SLANG_C_ACCESS(0, {
        auto sym = symbolOf(instance);
        if (!sym || sym->kind != SymbolKind::Instance)
            return 0;
        return (uint32_t)sym->as<InstanceSymbol>().body.getParameters().size();
    });
}

slang_ast slang_instance_parameter(slang_ast instance, uint32_t index) {
    SLANG_C_ACCESS(noAst(instance.compilation), {
        auto sym = symbolOf(instance);
        if (!sym || sym->kind != SymbolKind::Instance)
            return noAst(instance.compilation);
        auto params = sym->as<InstanceSymbol>().body.getParameters();
        if (index >= params.size())
            return noAst(instance.compilation);
        return capi::toC(&params[index]->symbol, instance.compilation);
    });
}

slang_str slang_parameter_value(slang_ast parameter, slang_error* err) {
    if (!checkEntry(err))
        return borrowed("");
    auto sym = symbolOf(parameter);
    if (!sym) {
        setError(err, SLANG_ERR_INVALID_ARG, "not a symbol");
        return borrowed("");
    }
    SLANG_C_GUARD(err, {
        if (sym->kind == SymbolKind::Parameter)
            return owned(sym->as<ParameterSymbol>().getValue().toString());
        if (sym->kind == SymbolKind::TypeParameter)
            return owned(sym->as<TypeParameterSymbol>().targetType.getType().toString());
        setError(err, SLANG_ERR_INVALID_ARG, "not a parameter");
        return borrowed("");
    });
    return borrowed("");
}

// ---- AST: types -------------------------------------------------------------

slang_ast slang_type_canonical(slang_ast type) {
    SLANG_C_ACCESS(noAst(type.compilation), {
        auto t = typeOf(type);
        return t ? capi::toC(&t->getCanonicalType(), type.compilation) : noAst(type.compilation);
    });
}

slang_str slang_type_to_string(slang_ast type, slang_error* err) {
    if (!checkEntry(err))
        return borrowed("");
    auto t = typeOf(type);
    if (!t) {
        setError(err, SLANG_ERR_INVALID_ARG, "not a type");
        return borrowed("");
    }
    SLANG_C_GUARD(err, { return owned(t->toString()); });
    return borrowed("");
}

uint64_t slang_type_bit_width(slang_ast type) {
    SLANG_C_ACCESS(0, {
        auto t = typeOf(type);
        return t ? t->getBitWidth() : 0;
    });
}

// A boolean type predicate: recover the Type, forward one `Type::isX()` query,
// and report `false` for a null/non-type node. Every `slang_type_is_*` accessor
// shares this body, so each is one self-documenting line.
#define SLANG_TYPE_PRED(fn, method)                                                    \
    bool fn(slang_ast type) {                                                          \
        SLANG_C_ACCESS(false, {                                                        \
            auto t = typeOf(type);                                                     \
            return t && t->method();                                                   \
        });                                                                            \
    }

SLANG_TYPE_PRED(slang_type_is_integral, isIntegral)
SLANG_TYPE_PRED(slang_type_is_signed, isSigned)
SLANG_TYPE_PRED(slang_type_is_four_state, isFourState)
SLANG_TYPE_PRED(slang_type_is_unpacked_array, isUnpackedArray)
SLANG_TYPE_PRED(slang_type_is_class, isClass)

bool slang_type_is_matching(slang_ast a, slang_ast b) {
    SLANG_C_ACCESS(false, {
        auto ta = typeOf(a);
        auto tb = typeOf(b);
        return ta && tb && ta->isMatching(*tb);
    });
}

bool slang_type_is_equivalent(slang_ast a, slang_ast b) {
    SLANG_C_ACCESS(false, {
        auto ta = typeOf(a);
        auto tb = typeOf(b);
        return ta && tb && ta->isEquivalent(*tb);
    });
}

bool slang_type_is_assignment_compatible(slang_ast a, slang_ast b) {
    SLANG_C_ACCESS(false, {
        auto ta = typeOf(a);
        auto tb = typeOf(b);
        return ta && tb && ta->isAssignmentCompatible(*tb);
    });
}

// ---- AST: expressions -------------------------------------------------------

slang_ast slang_expression_type(slang_ast expr) {
    SLANG_C_ACCESS(noAst(expr.compilation), {
        auto e = exprOf(expr);
        return e ? capi::toC(e->type.get(), expr.compilation) : noAst(expr.compilation);
    });
}

bool slang_expression_is_bad(slang_ast expr) {
    SLANG_C_ACCESS(true, {
        auto e = exprOf(expr);
        return !e || e->bad();
    });
}

slang_ast slang_expression_symbol(slang_ast expr) {
    SLANG_C_ACCESS(noAst(expr.compilation), {
        auto e = exprOf(expr);
        return capi::toC(e ? e->getSymbolReference() : nullptr, expr.compilation);
    });
}

bool slang_expression_cached_constant(slang_ast expr, slang_str* out, slang_error* err) {
    if (!checkEntry(err))
        return false;
    auto e = exprOf(expr);
    if (!e) {
        setError(err, SLANG_ERR_INVALID_ARG, "not an expression");
        return false;
    }
    auto cv = e->getConstant();
    if (!cv || cv->bad())
        return false;
    SLANG_C_GUARD(err, {
        if (out)
            *out = owned(cv->toString());
        return true;
    });
    return false;
}

bool slang_expression_eval(slang_ast expr, slang_str* out, slang_error* err) {
    if (!checkEntry(err))
        return false;
    auto e = exprOf(expr);
    auto comp = expr.compilation;
    if (!e || !comp) {
        setError(err, SLANG_ERR_INVALID_ARG, "not an expression");
        return false;
    }
    SLANG_C_GUARD(err, {
        // Evaluation may cache into the compilation's arena; lift the seal for
        // the duration of the call (the caller has promised exclusive access).
        SealLift lift(comp);
        ASTContext ctx(comp->comp->getRoot(), LookupLocation::max);
        ConstantValue cv = ctx.tryEval(*e);
        if (cv.bad())
            return false;
        if (out)
            *out = owned(cv.toString());
        return true;
    });
    return false;
}

// ---- Semantic statement/expression tree -------------------------------------

namespace {

// Collects the immediate AST children (sub-statements and sub-expressions) of a
// node into a flat list, in slang's own visitation order. Rooted through the
// node's own visit(): the root is seen at depth 0 (where we descend exactly one
// level via visitDefault) and each direct child at depth 1 (where we record it
// without recursing further).
struct SemChildCollector : ASTVisitor<SemChildCollector, VisitFlags::AllGood> {
    slang_compilation comp;
    std::vector<slang_ast>& out;
    int depth = 0;

    SemChildCollector(slang_compilation comp, std::vector<slang_ast>& out) :
        comp(comp), out(out) {}

    template<typename T>
    void handle(const T& t) {
        if (depth == 0) {
            depth++;
            visitDefault(t);
            depth--;
        }
        else {
            out.push_back(wrapAst(t, comp));
        }
    }
};

std::vector<slang_ast> collectSemChildren(slang_ast node) {
    std::vector<slang_ast> out;
    if (!node.ptr)
        return out;
    SemChildCollector c(node.compilation, out);
    switch ((slang_ast_domain)node.domain) {
        case SLANG_AST_SYMBOL:
            static_cast<const Symbol*>(node.ptr)->visit(c);
            break;
        case SLANG_AST_EXPRESSION:
            static_cast<const Expression*>(node.ptr)->visit(c);
            break;
        case SLANG_AST_STATEMENT:
            static_cast<const Statement*>(node.ptr)->visit(c);
            break;
        case SLANG_AST_TIMING_CONTROL:
            static_cast<const TimingControl*>(node.ptr)->visit(c);
            break;
        case SLANG_AST_CONSTRAINT:
            static_cast<const Constraint*>(node.ptr)->visit(c);
            break;
        case SLANG_AST_ASSERTION_EXPR:
            static_cast<const AssertionExpr*>(node.ptr)->visit(c);
            break;
        case SLANG_AST_PATTERN:
            static_cast<const Pattern*>(node.ptr)->visit(c);
            break;
        case SLANG_AST_BINS_SELECT_EXPR:
            static_cast<const BinsSelectExpr*>(node.ptr)->visit(c);
            break;
    }
    return out;
}

} // namespace

slang_ast slang_symbol_body(slang_ast sym_node) {
    SLANG_C_ACCESS(noAst(sym_node.compilation, SLANG_AST_STATEMENT), {
        auto sym = symbolOf(sym_node);
        if (!sym)
            return noAst(sym_node.compilation, SLANG_AST_STATEMENT);
        const Statement* body = nullptr;
        if (sym->kind == SymbolKind::ProceduralBlock)
            body = &sym->as<ProceduralBlockSymbol>().getBody();
        else if (sym->kind == SymbolKind::Subroutine)
            body = &sym->as<SubroutineSymbol>().getBody();
        if (!body)
            return noAst(sym_node.compilation, SLANG_AST_STATEMENT);
        return wrapAst(*body, sym_node.compilation);
    });
}

uint32_t slang_ast_sem_child_count(slang_ast node) {
    SLANG_C_ACCESS(0u, { return (uint32_t)collectSemChildren(node).size(); });
}

slang_ast slang_ast_sem_child(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation), {
        auto children = collectSemChildren(node);
        if (index >= children.size())
            return noAst(node.compilation);
        return children[index];
    });
}

uint32_t slang_ast_sem_children(slang_ast node, slang_ast* out, uint32_t cap) {
    SLANG_C_ACCESS(0u, {
        auto children = collectSemChildren(node);
        uint32_t n = children.size() < cap ? (uint32_t)children.size() : cap;
        for (uint32_t i = 0; i < n; i++)
            out[i] = children[i];
        return (uint32_t)children.size();
    });
}

// Every typed expression accessor shares one body: recover the Expression,
// bail to the neutral value unless it is a specific ExpressionKind, then read
// one child or attribute off the concrete subtype. These two macros capture
// that body so each accessor is a single self-documenting line; the trailing
// `member` is spliced after `.`, so pass a field (`op`) or a getter (`left()`).
#define SLANG_EXPR_CHILD(fn, KIND, T, member)                                          \
    slang_ast fn(slang_ast node) {                                                     \
        SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {                \
            auto e = exprOf(node);                                                     \
            if (!e || e->kind != ExpressionKind::KIND)                                 \
                return noAst(node.compilation, SLANG_AST_EXPRESSION);                  \
            return wrapAst(e->as<T>().member, node.compilation);                       \
        });                                                                            \
    }
#define SLANG_EXPR_ENUM(fn, KIND, T, member)                                           \
    uint32_t fn(slang_ast node) {                                                      \
        SLANG_C_ACCESS(0u, {                                                           \
            auto e = exprOf(node);                                                     \
            if (!e || e->kind != ExpressionKind::KIND)                                 \
                return 0u;                                                             \
            return (uint32_t)e->as<T>().member;                                        \
        });                                                                            \
    }

SLANG_EXPR_ENUM(slang_expr_binary_op, BinaryOp, BinaryExpression, op)
SLANG_EXPR_ENUM(slang_expr_unary_op, UnaryOp, UnaryExpression, op)

bool slang_expr_assignment_is_nonblocking(slang_ast node) {
    SLANG_C_ACCESS(false, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::Assignment)
            return false;
        return e->as<AssignmentExpression>().isNonBlocking();
    });
}

slang_ast slang_expr_call_subroutine(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::Call)
            return noAst(node.compilation);
        auto& call = e->as<CallExpression>();
        if (call.isSystemCall())
            return noAst(node.compilation);
        return toC(std::get<const SubroutineSymbol*>(call.subroutine), node.compilation);
    });
}

slang_ast slang_expr_member_symbol(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::MemberAccess)
            return noAst(node.compilation);
        return toC(&e->as<MemberAccessExpression>().member, node.compilation);
    });
}

// ---- Typed statement children -----------------------------------------------

slang_ast slang_stmt_then_branch(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_STATEMENT), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::Conditional)
            return noAst(node.compilation, SLANG_AST_STATEMENT);
        return wrapAst(s->as<ConditionalStatement>().ifTrue, node.compilation);
    });
}

slang_ast slang_stmt_else_branch(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_STATEMENT), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::Conditional)
            return noAst(node.compilation, SLANG_AST_STATEMENT);
        auto ifFalse = s->as<ConditionalStatement>().ifFalse;
        return ifFalse ? wrapAst(*ifFalse, node.compilation)
                       : noAst(node.compilation, SLANG_AST_STATEMENT);
    });
}

slang_ast slang_stmt_body(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_STATEMENT), {
        auto s = stmtOf(node);
        if (!s)
            return noAst(node.compilation, SLANG_AST_STATEMENT);
        switch (s->kind) {
            case StatementKind::ForLoop:
                return wrapAst(s->as<ForLoopStatement>().body, node.compilation);
            case StatementKind::RepeatLoop:
                return wrapAst(s->as<RepeatLoopStatement>().body, node.compilation);
            case StatementKind::WhileLoop:
                return wrapAst(s->as<WhileLoopStatement>().body, node.compilation);
            case StatementKind::DoWhileLoop:
                return wrapAst(s->as<DoWhileLoopStatement>().body, node.compilation);
            case StatementKind::ForeverLoop:
                return wrapAst(s->as<ForeverLoopStatement>().body, node.compilation);
            case StatementKind::ForeachLoop:
                return wrapAst(s->as<ForeachLoopStatement>().body, node.compilation);
            case StatementKind::Timed:
                return wrapAst(s->as<TimedStatement>().stmt, node.compilation);
            case StatementKind::Wait:
                return wrapAst(s->as<WaitStatement>().stmt, node.compilation);
            default:
                return noAst(node.compilation, SLANG_AST_STATEMENT);
        }
    });
}

slang_ast slang_stmt_cond(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto s = stmtOf(node);
        if (!s)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        switch (s->kind) {
            case StatementKind::WhileLoop:
                return wrapAst(s->as<WhileLoopStatement>().cond, node.compilation);
            case StatementKind::DoWhileLoop:
                return wrapAst(s->as<DoWhileLoopStatement>().cond, node.compilation);
            case StatementKind::Wait:
                return wrapAst(s->as<WaitStatement>().cond, node.compilation);
            case StatementKind::RepeatLoop:
                return wrapAst(s->as<RepeatLoopStatement>().count, node.compilation);
            case StatementKind::ForLoop: {
                auto stop = s->as<ForLoopStatement>().stopExpr;
                return stop ? wrapAst(*stop, node.compilation)
                            : noAst(node.compilation, SLANG_AST_EXPRESSION);
            }
            default:
                return noAst(node.compilation, SLANG_AST_EXPRESSION);
        }
    });
}

slang_ast slang_stmt_expr(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto s = stmtOf(node);
        if (!s)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        switch (s->kind) {
            case StatementKind::ExpressionStatement:
                return wrapAst(s->as<ExpressionStatement>().expr, node.compilation);
            case StatementKind::Case:
                return wrapAst(s->as<CaseStatement>().expr, node.compilation);
            case StatementKind::ForeachLoop:
                return wrapAst(s->as<ForeachLoopStatement>().arrayRef, node.compilation);
            case StatementKind::Return: {
                auto expr = s->as<ReturnStatement>().expr;
                return expr ? wrapAst(*expr, node.compilation)
                            : noAst(node.compilation, SLANG_AST_EXPRESSION);
            }
            default:
                return noAst(node.compilation, SLANG_AST_EXPRESSION);
        }
    });
}

slang_ast slang_stmt_timing(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_TIMING_CONTROL), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::Timed)
            return noAst(node.compilation, SLANG_AST_TIMING_CONTROL);
        return wrapAst(s->as<TimedStatement>().timing, node.compilation);
    });
}

// ---- Typed expression children ----------------------------------------------

SLANG_EXPR_CHILD(slang_expr_cond_true, ConditionalOp, ConditionalExpression, left())
SLANG_EXPR_CHILD(slang_expr_cond_false, ConditionalOp, ConditionalExpression, right())

slang_ast slang_expr_select_value(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto e = exprOf(node);
        if (!e)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        if (e->kind == ExpressionKind::ElementSelect)
            return wrapAst(e->as<ElementSelectExpression>().value(), node.compilation);
        if (e->kind == ExpressionKind::RangeSelect)
            return wrapAst(e->as<RangeSelectExpression>().value(), node.compilation);
        return noAst(node.compilation, SLANG_AST_EXPRESSION);
    });
}

SLANG_EXPR_CHILD(slang_expr_select_selector, ElementSelect, ElementSelectExpression, selector())
SLANG_EXPR_CHILD(slang_expr_range_left, RangeSelect, RangeSelectExpression, left())
SLANG_EXPR_CHILD(slang_expr_range_right, RangeSelect, RangeSelectExpression, right())
SLANG_EXPR_CHILD(slang_expr_conversion_operand, Conversion, ConversionExpression, operand())
SLANG_EXPR_CHILD(slang_expr_replication_count, Replication, ReplicationExpression, count())
SLANG_EXPR_CHILD(slang_expr_replication_concat, Replication, ReplicationExpression, concat())

SLANG_EXPR_ENUM(slang_expr_conversion_kind, Conversion, ConversionExpression, conversionKind)
SLANG_EXPR_ENUM(slang_expr_range_selection_kind, RangeSelect, RangeSelectExpression,
                getSelectionKind())

#undef SLANG_EXPR_CHILD
#undef SLANG_EXPR_ENUM

// ---- Type breadth -----------------------------------------------------------

// The scope of a struct/union type (packed or unpacked), whose members include
// the FieldSymbols; null for any other type.
static const Scope* structScope(const Type* t) {
    if (!t)
        return nullptr;
    auto& ct = t->getCanonicalType();
    switch (ct.kind) {
        case SymbolKind::PackedStructType:
        case SymbolKind::UnpackedStructType:
        case SymbolKind::PackedUnionType:
        case SymbolKind::UnpackedUnionType:
            return scopeOf(ct);
        default:
            return nullptr;
    }
}

slang_ast slang_type_array_element(slang_ast type) {
    SLANG_C_ACCESS(noAst(type.compilation), {
        auto t = typeOf(type);
        if (!t)
            return noAst(type.compilation);
        auto elem = t->getArrayElementType();
        return elem ? toC(elem, type.compilation) : noAst(type.compilation);
    });
}

slang_ast slang_type_enum_base(slang_ast type) {
    SLANG_C_ACCESS(noAst(type.compilation), {
        auto t = typeOf(type);
        if (!t)
            return noAst(type.compilation);
        auto& ct = t->getCanonicalType();
        if (ct.kind != SymbolKind::EnumType)
            return noAst(type.compilation);
        return toC(&ct.as<EnumType>().baseType, type.compilation);
    });
}

uint32_t slang_enum_member_count(slang_ast type) {
    SLANG_C_ACCESS(0u, {
        auto t = typeOf(type);
        if (!t)
            return 0u;
        auto& ct = t->getCanonicalType();
        if (ct.kind != SymbolKind::EnumType)
            return 0u;
        return (uint32_t)std::ranges::distance(ct.as<EnumType>().values());
    });
}

slang_ast slang_enum_member(slang_ast type, uint32_t index) {
    SLANG_C_ACCESS(noAst(type.compilation), {
        auto t = typeOf(type);
        if (!t)
            return noAst(type.compilation);
        auto& ct = t->getCanonicalType();
        if (ct.kind != SymbolKind::EnumType)
            return noAst(type.compilation);
        uint32_t i = 0;
        for (auto& value : ct.as<EnumType>().values()) {
            if (i++ == index)
                return toC(&value, type.compilation);
        }
        return noAst(type.compilation);
    });
}

slang_str slang_enum_member_value(slang_ast member, slang_error* err) {
    if (!checkEntry(err))
        return borrowed("");
    auto sym = symbolOf(member);
    if (!sym || sym->kind != SymbolKind::EnumValue) {
        setError(err, SLANG_ERR_INVALID_ARG, "not an enum value");
        return borrowed("");
    }
    // getValue() reads a memo forced pre-seal by FreezeVisitor, so this never
    // allocates from the arena; toString() allocates only an owned std::string.
    SLANG_C_GUARD(err, { return owned(sym->as<EnumValueSymbol>().getValue().toString()); });
    return borrowed("");
}

uint32_t slang_type_field_count(slang_ast type) {
    SLANG_C_ACCESS(0u, {
        auto s = structScope(typeOf(type));
        if (!s)
            return 0u;
        return (uint32_t)std::ranges::count_if(
            s->members(), [](const Symbol& m) { return m.kind == SymbolKind::Field; });
    });
}

slang_ast slang_type_field(slang_ast type, uint32_t index) {
    SLANG_C_ACCESS(noAst(type.compilation), {
        auto s = structScope(typeOf(type));
        if (!s)
            return noAst(type.compilation);
        uint32_t i = 0;
        for (auto& member : s->members()) {
            if (member.kind != SymbolKind::Field)
                continue;
            if (i++ == index)
                return toC(&member, type.compilation);
        }
        return noAst(type.compilation);
    });
}

uint64_t slang_field_bit_offset(slang_ast field) {
    SLANG_C_ACCESS((uint64_t)0, {
        auto sym = symbolOf(field);
        if (!sym || sym->kind != SymbolKind::Field)
            return (uint64_t)0;
        return sym->as<FieldSymbol>().bitOffset;
    });
}

uint32_t slang_field_index(slang_ast field) {
    SLANG_C_ACCESS(0u, {
        auto sym = symbolOf(field);
        if (!sym || sym->kind != SymbolKind::Field)
            return 0u;
        return sym->as<FieldSymbol>().fieldIndex;
    });
}

SLANG_TYPE_PRED(slang_type_is_enum, isEnum)
SLANG_TYPE_PRED(slang_type_is_struct, isStruct)
SLANG_TYPE_PRED(slang_type_is_union, isUnion)
SLANG_TYPE_PRED(slang_type_is_array, isArray)
SLANG_TYPE_PRED(slang_type_is_string, isString)

#undef SLANG_TYPE_PRED

slang_ast slang_type_class_base(slang_ast type) {
    SLANG_C_ACCESS(noAst(type.compilation), {
        auto t = typeOf(type);
        if (!t)
            return noAst(type.compilation);
        auto& ct = t->getCanonicalType();
        if (ct.kind != SymbolKind::ClassType)
            return noAst(type.compilation);
        // getBaseClass() calls ensureElaborated(); every reachable scope (this
        // class type included) is already elaborated by the freeze sweep, so
        // the baseClass memo is populated pre-seal and this is a pure read.
        auto base = ct.as<ClassType>().getBaseClass();
        return base ? toC(base, type.compilation) : noAst(type.compilation);
    });
}

// ---- Unstable ---------------------------------------------------------------

#if SLANG_C_API_ALLOW_UNSTABLE
const void* slang_unstable_native_ptr(slang_ast node) {
    return node.ptr;
}
#endif
