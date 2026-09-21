//------------------------------------------------------------------------------
// CApiAst.cpp
// C API: compilation, freezing, symbols, scopes, types, expressions
//
// SPDX-FileCopyrightText: Chaitanya Sharma
// SPDX-License-Identifier: MIT
//------------------------------------------------------------------------------
#include "CApiInternal.h"

#include <iterator>
#include <ranges>

#include "slang/ast/ASTContext.h"
#include "slang/ast/ASTVisitor.h"
#include "slang/ast/EvalContext.h"
#include "slang/ast/Constraints.h"
#include "slang/ast/Lookup.h"
#include "slang/ast/Patterns.h"
#include "slang/ast/Statement.h"
#include "slang/ast/TimingControl.h"
#include "slang/numeric/Time.h"
#include "slang/ast/expressions/AssertionExpr.h"
#include "slang/ast/expressions/AssignmentExpressions.h"
#include "slang/ast/expressions/CallExpression.h"
#include "slang/ast/expressions/ConversionExpression.h"
#include "slang/ast/expressions/LiteralExpressions.h"
#include "slang/ast/expressions/MiscExpressions.h"
#include "slang/ast/expressions/Operator.h"
#include "slang/ast/expressions/OperatorExpressions.h"
#include "slang/ast/expressions/SelectExpressions.h"
#include "slang/ast/SemanticFacts.h"
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
#include "slang/ast/types/DeclaredType.h"
#include "slang/ast/types/NetType.h"
#include "slang/ast/types/Type.h"
#include "slang/ast/types/TypePrinter.h"
#include "slang/ast/SystemSubroutine.h"
#include "slang/syntax/AllSyntax.h"
#include "slang/syntax/SyntaxKind.h"
#include "slang/text/SourceLocation.h"

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

slang_compilation slang_compilation_create_from_bag(slang_bag bag, uint32_t extra_flags,
                                                    slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        Bag b = bag ? bag->bag : Bag();
        if (extra_flags) {
            auto opts = b.getOrDefault<ast::CompilationOptions>();
            opts.flags |= bitmask<ast::CompilationFlags>(ast::CompilationFlags(extra_flags));
            b.set(opts);
        }
        return new slang_compilation_t(b);
    });
    return nullptr;
}

// Maps a slang_builtin_type ordinal to the compilation's canonical built-in
// Type; null for an unknown ordinal.
static const ast::Type* builtinTypeOf(ast::Compilation& c, uint32_t kind) {
    switch (kind) {
        case SLANG_BUILTIN_TYPE_INT:
            return &c.getIntType();
        case SLANG_BUILTIN_TYPE_LOGIC:
            return &c.getLogicType();
        case SLANG_BUILTIN_TYPE_BIT:
            return &c.getBitType();
        case SLANG_BUILTIN_TYPE_BYTE:
            return &c.getByteType();
        case SLANG_BUILTIN_TYPE_INTEGER:
            return &c.getIntegerType();
        case SLANG_BUILTIN_TYPE_REAL:
            return &c.getRealType();
        case SLANG_BUILTIN_TYPE_SHORTREAL:
            return &c.getShortRealType();
        case SLANG_BUILTIN_TYPE_STRING:
            return &c.getStringType();
        case SLANG_BUILTIN_TYPE_VOID:
            return &c.getVoidType();
        default:
            return nullptr;
    }
}

void slang_compilation_add_nonconstant_system_function(slang_compilation comp, const char* name,
                                                       size_t name_len, uint32_t return_type,
                                                       const uint32_t* arg_types, size_t n_args,
                                                       slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!comp || !name || (n_args && !arg_types)) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return;
    }
    if (comp->comp->isFinalized()) {
        setError(err, SLANG_ERR_INVALID_STATE,
                 "system subroutines must be registered before the compilation is finalized");
        return;
    }
    SLANG_C_GUARD(err, {
        auto& c = *comp->comp;
        const ast::Type* ret = builtinTypeOf(c, return_type);
        if (!ret) {
            setError(err, SLANG_ERR_INVALID_ARG, "unknown return type kind");
            return;
        }
        std::vector<const ast::Type*> args;
        args.reserve(n_args);
        for (size_t i = 0; i < n_args; i++) {
            const ast::Type* a = builtinTypeOf(c, arg_types[i]);
            if (!a) {
                setError(err, SLANG_ERR_INVALID_ARG, "unknown argument type kind");
                return;
            }
            args.push_back(a);
        }
        c.addSystemSubroutine(std::make_shared<ast::NonConstantFunction>(
            std::string(name, name_len), *ret, n_args, args));
    });
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
            // A Specparam's PATHPULSE$ source/dest terminals are resolved
            // and cached together on first read (mutable `pathSource`/
            // `pathDest`, guarded by `pathPulseResolved`) -- force both here
            // so slang_symbol_specparam_path_source/_path_dest are pure
            // reads on the frozen, shared &Design.
            if constexpr (std::is_same_v<T, SpecparamSymbol>) {
                t.getPathSource();
                t.getPathDest();
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
                // Default-value initializer (e.g. `input logic a = 0;`):
                // lazily bound and allocated into the arena on first read
                // (mutable `initializer` field), just like the memos above.
                t.getInitializer();
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
            // A package's `export` declarations are validated and its
            // export maps populated lazily, memoized behind a private
            // mutable bool (ExportData::resolved) -- exactly the shape of
            // memo this sweep exists to force, so that findForImport()
            // becomes a pure read on the frozen design.
            else if constexpr (std::is_same_v<T, PackageSymbol>) {
                t.resolveExports();
                // A name pulled in through `import pkg::*;` only becomes
                // visible to findForImport() once something in this
                // package has actually referenced it unqualified (each
                // such reference lazily populates WildcardImportData::
                // importedSymbols) -- OR once findForImport() itself
                // force-elaborates the whole package to guarantee every
                // reference has been seen, guarded by the same
                // `hasForceElaborated` flag checked below.
                // Compilation::forceElaborate() asserts !isFrozen(), so
                // that force MUST run here, pre-seal: otherwise the first
                // post-freeze findForImport() call on a wildcard-importing
                // package would attempt to mutate the sealed arena (an
                // assertion failure that SLANG_C_ACCESS's catch(...) would
                // silently swallow, exactly the kind of unforced-memo race
                // this sweep exists to rule out).
                if (auto wildcardData = t.getWildcardImportData()) {
                    if (!wildcardData->hasForceElaborated) {
                        wildcardData->hasForceElaborated = true;
                        t.getCompilation().forceElaborate(t);
                    }
                }
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
            // A covergroup body's own `option`/`type_option` setters
            // (declared directly in the covergroup, not inside a coverpoint
            // or cross) aren't reached by the generic scope-member traversal
            // below (CoverageOptionSetter isn't a Symbol) and
            // CovergroupBodySymbol has no visitExprs of its own — force each
            // setter's expression explicitly, exactly like CoverCrossSymbol's
            // and CoverpointSymbol's own options (already forced generically
            // through their visitExprs, invoked via visitDefault below).
            else if constexpr (std::is_same_v<T, CovergroupBodySymbol>) {
                for (auto& opt : t.options)
                    opt.getExpression();
            }
            // A constraint block's bound constraint tree is built (and its
            // expressions bound/allocated into the arena) lazily on first
            // read.
            else if constexpr (std::is_same_v<T, ConstraintBlockSymbol>) {
                t.getConstraints();
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
            // A randsequence production's rule tree (prods, and every
            // embedded expression: ProdItem args, if/case conditions, case
            // labels, weight/randjoin exprs) is built and bound to the arena
            // lazily on first getRules() call (mutable `rules` field) --
            // exactly the memo shape this sweep exists to force. The prod
            // nodes (ProdItem/CodeBlockProd/IfElseProd/CaseProd) sit in
            // custom structs outside the generic Scope-member traversal
            // below, so their embedded expressions are never reached by
            // visitDefault either -- RandSeqProductionSymbol::visitExprs
            // exists for exactly this purpose (same shape as the
            // GenerateBlockArraySymbol/GenerateBlockSymbol branches above).
            else if constexpr (std::is_same_v<T, RandSeqProductionSymbol>) {
                t.getRules();
                t.visitExprs(*this);
            }
            // Checker instance output-port initial expressions (per
            // connection), plus each connection's attribute values. Unlike
            // an attribute attached via Symbol::setAttributes (reached
            // generically through Compilation::getAttributes, which is not
            // exposed as a standalone C accessor and so needs no force of
            // its own yet), a Connection::attributes AttributeSymbol is
            // reachable ONLY through this field — it is never a scope
            // member and never visited by the generic ASTVisitor traversal
            // this sweep runs on, so without this explicit force its
            // getValue() memo stays unresolved and the read-only frozen
            // arena refuses the allocation slang_symbol_attribute_value
            // needs (SOUNDNESS-MEMOS.md).
            else if constexpr (std::is_same_v<T, CheckerInstanceSymbol>) {
                for (auto& conn : t.getPortConnections()) {
                    conn.getOutputInitialExpr();
                    for (auto attr : conn.attributes)
                        attr->getValue();
                }
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
            // An unresolved (definition-not-found) instantiation: its port
            // connection expressions, port names, and the derived
            // mustBeChecker flag are all computed together on first
            // getPortConnections() call (mutable `ports`/`portNames`/
            // `mustBeChecker` fields) -- force that once here so
            // getPortConnections/getPortNames/isChecker are pure reads on
            // the frozen, shared &Design. Neither those connection
            // AssertionExprs nor the (eagerly bound, plain-field)
            // paramExpressions are ever scope members or reached by any
            // visitExprs of this symbol's own -- like the
            // GenerateBlockArraySymbol/RandSeqProductionSymbol branches
            // above, each must be explicitly visited here so its own
            // canonical type gets force-resolved on the frozen design.
            else if constexpr (std::is_same_v<T, UninstantiatedDefSymbol>) {
                for (auto expr : t.paramExpressions) {
                    if (expr)
                        expr->visit(*this);
                }
                for (auto* conn : t.getPortConnections()) {
                    if (conn)
                        conn->visit(*this);
                }
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
            // Loop-generate array: initial/stop/iteration expressions and the
            // loop variable are bound against a private, unregistered scope
            // (`iterScope` in GenerateBlockArraySymbol::fromSyntax) that is
            // never added as a member of any visited Scope, and the array
            // symbol itself has no visitExprs of its own — so unlike a
            // ContinuousAssignSymbol's assignment (reached generically via
            // its own visitExprs), none of these five fields is ever reached
            // by the generic traversal below. Explicitly visiting each one
            // (rather than merely reading the field) forces the same
            // per-node work handle() does for any other visited Symbol/
            // Expression -- declared-type/canonical-type forcing for the
            // loop variable, and canonical-type forcing + prefold + child
            // recursion for the three expressions -- so a later read through
            // the new C accessors is a pure read on the frozen design.
            else if constexpr (std::is_same_v<T, GenerateBlockArraySymbol>) {
                if (t.loopVariable)
                    t.loopVariable->visit(*this);
                if (t.initialExpression)
                    t.initialExpression->visit(*this);
                if (t.stopExpression)
                    t.stopExpression->visit(*this);
                if (t.iterExpression)
                    t.iterExpression->visit(*this);
            }
            // Conditional/case-generate block: the bound if/case condition
            // expression (getConditionExpression(), stored in the same union
            // slot as arrayIndex) and the case-item label expressions are
            // both bound against the *enclosing* scope (the generate
            // construct's own context), not this block's own scope, and
            // GenerateBlockSymbol has no visitExprs of its own -- so, like
            // the loop-array fields above, none of them is ever reached by
            // the generic traversal below without this explicit visit.
            else if constexpr (std::is_same_v<T, GenerateBlockSymbol>) {
                if (auto cond = t.getConditionExpression())
                    cond->visit(*this);
                for (auto expr : t.caseItemExpressions)
                    expr->visit(*this);
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

        // Compilation::getUnsignedIntType() lazily allocates into
        // vectorTypeCache on first call (a mutable, non-const-method memo, not
        // an AST node the visitor below would ever reach); force it here,
        // unconditionally and pre-seal, so slang_compilation_get_unsigned_int_type
        // is a pure read on a frozen, shared &Design. See SOUNDNESS-MEMOS.md.
        comp->comp->getUnsignedIntType();

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

void slang_compilation_unfreeze(slang_compilation comp, slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!comp) {
        setError(err, SLANG_ERR_INVALID_ARG, "null compilation");
        return;
    }
    SLANG_C_GUARD(err, {
        if (comp->sealed) {
            comp->comp->unfreeze();
            comp->sealed = false;
        }
    });
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

uint32_t slang_symbol_root_top_instance_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Root)
            return 0u;
        return (uint32_t)s->as<RootSymbol>().topInstances.size();
    });
}

slang_ast slang_symbol_root_top_instance(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Root)
            return noAst(sym.compilation);
        auto tops = s->as<RootSymbol>().topInstances;
        if (index >= tops.size())
            return noAst(sym.compilation);
        return capi::toC(tops[index], sym.compilation);
    });
}

uint32_t slang_symbol_root_compilation_unit_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Root)
            return 0u;
        return (uint32_t)s->as<RootSymbol>().compilationUnits.size();
    });
}

slang_ast slang_symbol_root_compilation_unit(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Root)
            return noAst(sym.compilation);
        auto units = s->as<RootSymbol>().compilationUnits;
        if (index >= units.size())
            return noAst(sym.compilation);
        return capi::toC(units[index], sym.compilation);
    });
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

slang_ast slang_compilation_get_bit_type(slang_compilation comp) {
    if (!comp)
        return noAst(comp);
    SLANG_C_ACCESS(noAst(comp), { return capi::toC(&comp->comp->getBitType(), comp); });
}

slang_ast slang_compilation_get_byte_type(slang_compilation comp) {
    if (!comp)
        return noAst(comp);
    SLANG_C_ACCESS(noAst(comp), { return capi::toC(&comp->comp->getByteType(), comp); });
}

slang_ast slang_compilation_unit_for_syntax(slang_compilation comp, slang_node syntax) {
    if (!comp)
        return noAst(comp);
    SLANG_C_ACCESS(noAst(comp), {
        auto node = fromC(syntax);
        if (!node || node->kind != syntax::SyntaxKind::CompilationUnit)
            return noAst(comp);
        return capi::toC(comp->comp->getCompilationUnit(node->as<syntax::CompilationUnitSyntax>()),
                         comp);
    });
}

uint32_t slang_compilation_unit_count(slang_compilation comp) {
    if (!comp)
        return 0;
    SLANG_C_ACCESS(0, { return (uint32_t)comp->comp->getCompilationUnits().size(); });
}

slang_ast slang_compilation_unit_at(slang_compilation comp, uint32_t index) {
    if (!comp)
        return noAst(comp);
    SLANG_C_ACCESS(noAst(comp), {
        auto units = comp->comp->getCompilationUnits();
        if (index >= units.size())
            return noAst(comp);
        return capi::toC(units[index], comp);
    });
}

slang_ast slang_compilation_create_script_scope(slang_compilation comp, slang_error* err) {
    if (!checkEntry(err))
        return noAst(comp);
    if (!comp) {
        setError(err, SLANG_ERR_INVALID_ARG, "null compilation");
        return noAst(comp);
    }
    SLANG_C_GUARD(err, {
        // Adds a new arena-allocated compilation unit; lift the seal for the
        // duration (the caller has promised exclusive access), exactly like
        // slang_expression_eval.
        SealLift lift(comp);
        return capi::toC(&comp->comp->createScriptScope(), comp);
    });
    return noAst(comp);
}

void slang_compilation_add_diagnostics(slang_compilation comp, slang_diagnostics diags,
                                       slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!comp || !diags) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return;
    }
    SLANG_C_GUARD(err, {
        SealLift lift(comp);
        Diagnostics tmp;
        for (auto& d : diags->diags)
            tmp.push_back(d);
        comp->comp->addDiagnostics(tmp);
    });
}

// ---- Compilation: DPI exports ------------------------------------------------

bool slang_dpi_export_is_null(slang_dpi_export exp) {
    return exp.ptr == nullptr;
}

uint32_t slang_compilation_dpi_export_count(slang_compilation comp) {
    if (!comp)
        return 0;
    SLANG_C_ACCESS(0, { return (uint32_t)comp->comp->getDPIExports().size(); });
}

slang_dpi_export slang_compilation_dpi_export(slang_compilation comp, uint32_t index) {
    if (!comp)
        return slang_dpi_export{nullptr, comp};
    SLANG_C_ACCESS((slang_dpi_export{nullptr, comp}), {
        auto exports = comp->comp->getDPIExports();
        if (index >= exports.size())
            return slang_dpi_export{nullptr, comp};
        return slang_dpi_export{&exports[index], comp};
    });
}

slang_ast slang_dpi_export_subroutine(slang_dpi_export exp) {
    SLANG_C_ACCESS(noAst(exp.compilation), {
        if (!exp.ptr)
            return noAst(exp.compilation);
        auto& e = *static_cast<const Compilation::DPIExport*>(exp.ptr);
        return capi::toC(e.subroutine, exp.compilation);
    });
}

slang_str slang_dpi_export_c_identifier(slang_dpi_export exp) {
    SLANG_C_ACCESS(borrowed(""), {
        if (!exp.ptr)
            return borrowed("");
        auto& e = *static_cast<const Compilation::DPIExport*>(exp.ptr);
        return borrowed(e.cIdentifier);
    });
}

slang_node slang_dpi_export_syntax(slang_dpi_export exp) {
    SLANG_C_ACCESS(noNode(nullptr), {
        if (!exp.ptr || !exp.compilation)
            return noNode(nullptr);
        auto& e = *static_cast<const Compilation::DPIExport*>(exp.ptr);
        if (!e.syntax)
            return noNode(nullptr);
        auto tree = findTree(exp.compilation, *e.syntax);
        return tree ? capi::toC(e.syntax, tree) : noNode(nullptr);
    });
}

// ---- Compilation: definition lookup ------------------------------------------

slang_definition_lookup_result slang_compilation_try_get_definition(slang_compilation comp,
                                                                     const char* name,
                                                                     size_t name_len,
                                                                     slang_ast scope) {
    auto none = slang_definition_lookup_result{noAst(comp), noAst(comp), nullptr};
    if (!comp || !name)
        return none;
    SLANG_C_ACCESS(none, {
        auto sym = symbolOf(scope);
        auto sc = sym ? scopeOf(*sym) : nullptr;
        if (!sc)
            return none;
        auto result = comp->comp->tryGetDefinition(toView(name, name_len), *sc);
        return slang_definition_lookup_result{
            capi::toC(result.definition, comp),
            capi::toC(result.configRoot, comp),
            (slang_config_rule)result.configRule,
        };
    });
}

// ---- Compilation: built-in types, options, libraries, diagnostics -----------

slang_ast slang_compilation_get_int_type(slang_compilation comp) {
    if (!comp)
        return noAst(comp);
    SLANG_C_ACCESS(noAst(comp), { return capi::toC(&comp->comp->getIntType(), comp); });
}

slang_ast slang_compilation_get_integer_type(slang_compilation comp) {
    if (!comp)
        return noAst(comp);
    SLANG_C_ACCESS(noAst(comp), { return capi::toC(&comp->comp->getIntegerType(), comp); });
}

slang_ast slang_compilation_get_logic_type(slang_compilation comp) {
    if (!comp)
        return noAst(comp);
    SLANG_C_ACCESS(noAst(comp), { return capi::toC(&comp->comp->getLogicType(), comp); });
}

slang_ast slang_compilation_get_real_type(slang_compilation comp) {
    if (!comp)
        return noAst(comp);
    SLANG_C_ACCESS(noAst(comp), { return capi::toC(&comp->comp->getRealType(), comp); });
}

slang_ast slang_compilation_get_short_real_type(slang_compilation comp) {
    if (!comp)
        return noAst(comp);
    SLANG_C_ACCESS(noAst(comp), { return capi::toC(&comp->comp->getShortRealType(), comp); });
}

slang_ast slang_compilation_get_error_type(slang_compilation comp) {
    if (!comp)
        return noAst(comp);
    SLANG_C_ACCESS(noAst(comp), { return capi::toC(&comp->comp->getErrorType(), comp); });
}

slang_ast slang_compilation_get_null_type(slang_compilation comp) {
    if (!comp)
        return noAst(comp);
    SLANG_C_ACCESS(noAst(comp), { return capi::toC(&comp->comp->getNullType(), comp); });
}

slang_ast slang_compilation_get_gate_type(slang_compilation comp, const char* name,
                                          size_t name_len) {
    if (!comp || !name)
        return noAst(comp);
    SLANG_C_ACCESS(noAst(comp),
                   { return capi::toC(comp->comp->getGateType(toView(name, name_len)), comp); });
}

slang_ast slang_compilation_get_package(slang_compilation comp, const char* name,
                                        size_t name_len) {
    if (!comp || !name)
        return noAst(comp);
    SLANG_C_ACCESS(noAst(comp),
                   { return capi::toC(comp->comp->getPackage(toView(name, name_len)), comp); });
}

slang_ast slang_compilation_get_net_type(slang_compilation comp, slang_net_type_kind kind) {
    if (!comp)
        return noAst(comp);
    SLANG_C_ACCESS(noAst(comp), {
        using TK = parsing::TokenKind;
        TK tk = TK::Unknown;
        switch (kind) {
            case SLANG_NET_TYPE_WIRE:
                tk = TK::WireKeyword;
                break;
            case SLANG_NET_TYPE_WAND:
                tk = TK::WAndKeyword;
                break;
            case SLANG_NET_TYPE_WOR:
                tk = TK::WOrKeyword;
                break;
            case SLANG_NET_TYPE_TRI:
                tk = TK::TriKeyword;
                break;
            case SLANG_NET_TYPE_TRIAND:
                tk = TK::TriAndKeyword;
                break;
            case SLANG_NET_TYPE_TRIOR:
                tk = TK::TriOrKeyword;
                break;
            case SLANG_NET_TYPE_TRI0:
                tk = TK::Tri0Keyword;
                break;
            case SLANG_NET_TYPE_TRI1:
                tk = TK::Tri1Keyword;
                break;
            case SLANG_NET_TYPE_TRIREG:
                tk = TK::TriRegKeyword;
                break;
            case SLANG_NET_TYPE_SUPPLY0:
                tk = TK::Supply0Keyword;
                break;
            case SLANG_NET_TYPE_SUPPLY1:
                tk = TK::Supply1Keyword;
                break;
            case SLANG_NET_TYPE_UWIRE:
                tk = TK::UWireKeyword;
                break;
            case SLANG_NET_TYPE_INTERCONNECT:
                tk = TK::InterconnectKeyword;
                break;
            default:
                break;
        }
        return capi::toC(&comp->comp->getNetType(tk), comp);
    });
}

slang_compilation_options slang_compilation_get_options(slang_compilation comp) {
    slang_compilation_options none{};
    if (!comp)
        return none;
    SLANG_C_ACCESS(none, {
        auto& o = comp->comp->getOptions();
        return slang_compilation_options{
            (uint32_t)o.flags.bits(),
            o.maxInstanceDepth,
            o.maxCheckerInstanceDepth,
            o.maxGenerateSteps,
            o.maxConstexprDepth,
            o.maxConstexprSteps,
            o.maxConstexprBacktrace,
            o.maxConstantSize,
            o.maxDefParamSteps,
            o.maxDefParamBlocks,
            o.maxInstanceArray,
            o.maxEnumValues,
            o.maxRecursiveClassSpecialization,
            o.maxUDPCoverageNotes,
            o.errorLimit,
            o.typoCorrectionLimit,
            (uint32_t)o.minTypMax,
            (uint32_t)o.languageVersion,
        };
    });
}

bool slang_compilation_get_default_time_scale(slang_compilation comp, slang_time_scale* out) {
    if (!comp || !out)
        return false;
    SLANG_C_ACCESS(false, {
        auto ts = comp->comp->getDefaultTimeScale();
        if (!ts)
            return false;
        *out = slang_time_scale{
            (uint8_t)ts->base.unit,
            (uint8_t)ts->base.magnitude,
            (uint8_t)ts->precision.unit,
            (uint8_t)ts->precision.magnitude,
        };
        return true;
    });
}

namespace {

TimeScale toCppTimeScale(slang_time_scale ts) {
    return TimeScale(TimeScaleValue((TimeUnit)ts.base_unit, (TimeScaleMagnitude)ts.base_magnitude),
                     TimeScaleValue((TimeUnit)ts.precision_unit,
                                    (TimeScaleMagnitude)ts.precision_magnitude));
}

slang_time_scale_value toC(const TimeScaleValue& v) {
    return slang_time_scale_value{(uint8_t)v.unit, (uint8_t)v.magnitude};
}

} // namespace

slang_time_scale_value slang_time_scale_base(slang_time_scale ts) {
    SLANG_C_ACCESS(slang_time_scale_value{}, { return toC(toCppTimeScale(ts).base); });
}

slang_time_scale_value slang_time_scale_precision(slang_time_scale ts) {
    SLANG_C_ACCESS(slang_time_scale_value{}, { return toC(toCppTimeScale(ts).precision); });
}

double slang_time_scale_apply(slang_time_scale ts, double value, uint8_t unit,
                              bool round_to_precision) {
    SLANG_C_ACCESS(0.0, {
        return toCppTimeScale(ts).apply(value, (TimeUnit)unit, round_to_precision);
    });
}

bool slang_time_scale_from_string(const char* str, size_t str_len, slang_time_scale* out) {
    if (!out)
        return false;
    SLANG_C_ACCESS(false, {
        auto result = TimeScale::fromString(std::string_view(str, str_len));
        if (!result)
            return false;
        *out = slang_time_scale{
            (uint8_t)result->base.unit,
            (uint8_t)result->base.magnitude,
            (uint8_t)result->precision.unit,
            (uint8_t)result->precision.magnitude,
        };
        return true;
    });
}

uint8_t slang_time_scale_value_unit(slang_time_scale_value v) {
    SLANG_C_ACCESS((uint8_t)0, { return v.unit; });
}

uint8_t slang_time_scale_value_magnitude(slang_time_scale_value v) {
    SLANG_C_ACCESS((uint8_t)0, { return v.magnitude; });
}

bool slang_time_scale_value_from_literal(double value, uint8_t unit, slang_time_scale_value* out) {
    if (!out)
        return false;
    SLANG_C_ACCESS(false, {
        auto result = TimeScaleValue::fromLiteral(value, (TimeUnit)unit);
        if (!result)
            return false;
        *out = toC(*result);
        return true;
    });
}

bool slang_time_scale_value_from_string(const char* str, size_t str_len,
                                        slang_time_scale_value* out) {
    if (!out)
        return false;
    SLANG_C_ACCESS(false, {
        auto result = TimeScaleValue::fromString(std::string_view(str, str_len));
        if (!result)
            return false;
        *out = toC(*result);
        return true;
    });
}

uint32_t slang_compilation_top_module_count(slang_compilation comp) {
    if (!comp)
        return 0;
    SLANG_C_ACCESS(0, { return (uint32_t)comp->comp->getOptions().topModules.size(); });
}

slang_str slang_compilation_top_module_at(slang_compilation comp, uint32_t index) {
    if (!comp)
        return borrowed("");
    SLANG_C_ACCESS(borrowed(""), {
        auto& tops = comp->comp->getOptions().topModules;
        if (index >= tops.size())
            return borrowed("");
        // flat_hash_set has no operator[]; the set is fixed at construction
        // (CompilationOptions is never mutated afterward) so advancing an
        // iterator by `index` is stable across repeated calls.
        auto it = tops.begin();
        std::advance(it, index);
        return borrowed(*it);
    });
}

uint32_t slang_compilation_param_override_count(slang_compilation comp) {
    if (!comp)
        return 0;
    SLANG_C_ACCESS(0, { return (uint32_t)comp->comp->getOptions().paramOverrides.size(); });
}

slang_str slang_compilation_param_override_at(slang_compilation comp, uint32_t index) {
    if (!comp)
        return borrowed("");
    SLANG_C_ACCESS(borrowed(""), {
        auto& overrides = comp->comp->getOptions().paramOverrides;
        if (index >= overrides.size())
            return borrowed("");
        return borrowed(overrides[index]);
    });
}

uint32_t slang_compilation_default_liblist_count(slang_compilation comp) {
    if (!comp)
        return 0;
    SLANG_C_ACCESS(0, { return (uint32_t)comp->comp->getOptions().defaultLiblist.size(); });
}

slang_str slang_compilation_default_liblist_at(slang_compilation comp, uint32_t index) {
    if (!comp)
        return borrowed("");
    SLANG_C_ACCESS(borrowed(""), {
        auto& liblist = comp->comp->getOptions().defaultLiblist;
        if (index >= liblist.size())
            return borrowed("");
        return borrowed(liblist[index]);
    });
}

slang_str slang_source_library_name(slang_source_library lib) {
    if (!lib)
        return borrowed("");
    SLANG_C_ACCESS(borrowed(""),
                   { return borrowed(reinterpret_cast<const SourceLibrary*>(lib)->name); });
}

int32_t slang_source_library_priority(slang_source_library lib) {
    if (!lib)
        return 0;
    SLANG_C_ACCESS(
        0, { return (int32_t)reinterpret_cast<const SourceLibrary*>(lib)->priority; });
}

bool slang_source_library_is_default(slang_source_library lib) {
    if (!lib)
        return false;
    SLANG_C_ACCESS(false, { return reinterpret_cast<const SourceLibrary*>(lib)->isDefault; });
}

slang_source_library slang_compilation_get_default_library(slang_compilation comp) {
    if (!comp)
        return nullptr;
    SLANG_C_ACCESS(nullptr, {
        return reinterpret_cast<slang_source_library>(
            const_cast<SourceLibrary*>(&comp->comp->getDefaultLibrary()));
    });
}

slang_source_library slang_compilation_get_source_library(slang_compilation comp,
                                                           const char* name, size_t name_len) {
    if (!comp || !name)
        return nullptr;
    SLANG_C_ACCESS(nullptr, {
        auto lib = comp->comp->getSourceLibrary(toView(name, name_len));
        return reinterpret_cast<slang_source_library>(const_cast<SourceLibrary*>(lib));
    });
}

slang_diagnostics slang_compilation_get_parse_diagnostics(slang_compilation comp,
                                                          slang_error* err) {
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
        auto& diags = comp->comp->getParseDiagnostics();
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

slang_diagnostics slang_compilation_get_semantic_diagnostics(slang_compilation comp,
                                                             slang_error* err) {
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
        auto& diags = comp->comp->getSemanticDiagnostics();
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

// ---- Compilation: state, syntax trees, name parsing --------------------------

bool slang_compilation_is_finalized(slang_compilation comp) {
    SLANG_C_ACCESS(false, { return comp && comp->comp->isFinalized(); });
}

bool slang_compilation_is_elaborated(slang_compilation comp) {
    SLANG_C_ACCESS(false, { return comp && comp->comp->isElaborated(); });
}

bool slang_compilation_has_issued_errors(slang_compilation comp) {
    SLANG_C_ACCESS(false, { return comp && comp->comp->hasIssuedErrors(); });
}

bool slang_compilation_has_fatal_errors(slang_compilation comp) {
    SLANG_C_ACCESS(false, { return comp && comp->comp->hasFatalErrors(); });
}

uint32_t slang_compilation_syntax_tree_count(slang_compilation comp) {
    if (!comp)
        return 0;
    SLANG_C_ACCESS(0, { return (uint32_t)comp->trees.size(); });
}

slang_syntax_tree slang_compilation_syntax_tree_at(slang_compilation comp, uint32_t index) {
    if (!comp)
        return nullptr;
    SLANG_C_ACCESS(nullptr, {
        if (index >= comp->trees.size())
            return nullptr;
        return comp->trees[index];
    });
}

slang_node slang_compilation_parse_name(slang_compilation comp, const char* name, size_t name_len,
                                        slang_error* err) {
    if (!checkEntry(err))
        return noNode(nullptr);
    if (!comp || !name) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return noNode(nullptr);
    }
    if (!comp->sm) {
        setError(err, SLANG_ERR_INVALID_STATE, "compilation has no syntax trees");
        return noNode(nullptr);
    }
    // Hand-rolled guard (not SLANG_C_GUARD) so a malformed name maps to
    // SLANG_ERR_INVALID_ARG rather than the generic SLANG_ERR_INTERNAL: slang's
    // parseName throws on a bad name. The catch clauses bind NO exception
    // variable, so this stays valid under -fno-exceptions (where SLANG_CATCH(X)
    // expands to `if (false)` and any bound variable would be undeclared).
    SLANG_TRY {
        SealLift lift(comp);
        return capi::toC(&comp->comp->parseName(toView(name, name_len)), nullptr);
    }
    SLANG_CATCH(const std::bad_alloc&) {
        setError(err, SLANG_ERR_OUT_OF_MEMORY, "out of memory parsing name");
        return noNode(nullptr);
    }
    SLANG_CATCH(const std::exception&) {
        setError(err, SLANG_ERR_INVALID_ARG, "invalid name syntax");
        return noNode(nullptr);
    }
    return noNode(nullptr);
}

slang_node slang_compilation_try_parse_name(slang_compilation comp, const char* name,
                                            size_t name_len, slang_diagnostics* diags_out,
                                            slang_error* err) {
    if (!checkEntry(err))
        return noNode(nullptr);
    if (!comp || !name) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return noNode(nullptr);
    }
    if (!comp->sm) {
        setError(err, SLANG_ERR_INVALID_STATE, "compilation has no syntax trees");
        return noNode(nullptr);
    }
    SLANG_C_GUARD(err, {
        SealLift lift(comp);
        Diagnostics localDiags;
        auto& nameSyntax = comp->comp->tryParseName(toView(name, name_len), localDiags);
        if (diags_out) {
            auto result = new slang_diagnostics_t();
            result->sm = comp->sm;
            result->comp = comp;
            result->diags.assign(localDiags.begin(), localDiags.end());
            *diags_out = result;
        }
        return capi::toC(&nameSyntax, nullptr);
    });
    return noNode(nullptr);
}

// ---- Compilation: more built-in types and system methods ---------------------

slang_ast slang_compilation_get_std_package(slang_compilation comp) {
    if (!comp)
        return noAst(comp);
    SLANG_C_ACCESS(noAst(comp), { return capi::toC(&comp->comp->getStdPackage(), comp); });
}

slang_ast slang_compilation_get_string_type(slang_compilation comp) {
    if (!comp)
        return noAst(comp);
    SLANG_C_ACCESS(noAst(comp), { return capi::toC(&comp->comp->getStringType(), comp); });
}

slang_ast slang_compilation_get_void_type(slang_compilation comp) {
    if (!comp)
        return noAst(comp);
    SLANG_C_ACCESS(noAst(comp), { return capi::toC(&comp->comp->getVoidType(), comp); });
}

slang_ast slang_compilation_get_unbounded_type(slang_compilation comp) {
    if (!comp)
        return noAst(comp);
    SLANG_C_ACCESS(noAst(comp), { return capi::toC(&comp->comp->getUnboundedType(), comp); });
}

slang_ast slang_compilation_get_type_ref_type(slang_compilation comp) {
    if (!comp)
        return noAst(comp);
    SLANG_C_ACCESS(noAst(comp), { return capi::toC(&comp->comp->getTypeRefType(), comp); });
}

slang_ast slang_compilation_get_unsigned_int_type(slang_compilation comp) {
    if (!comp)
        return noAst(comp);
    // getUnsignedIntType() lazily populates Compilation::vectorTypeCache on
    // first call for this exact width/flags combination; slang_compilation_freeze
    // forces it pre-seal (see the FreezeVisitor call site), so by the time a
    // frozen &Design can reach this accessor the cache entry already exists
    // and this is a pure read. See SOUNDNESS-MEMOS.md.
    SLANG_C_ACCESS(noAst(comp), { return capi::toC(&comp->comp->getUnsignedIntType(), comp); });
}

slang_ast slang_compilation_get_wire_net_type(slang_compilation comp) {
    if (!comp)
        return noAst(comp);
    SLANG_C_ACCESS(noAst(comp), { return capi::toC(&comp->comp->getWireNetType(), comp); });
}

slang_str slang_system_subroutine_name(slang_system_subroutine sub) {
    SLANG_C_ACCESS(borrowed(""), {
        if (!sub)
            return borrowed("");
        return borrowed(reinterpret_cast<const SystemSubroutine*>(sub)->name);
    });
}

bool slang_system_subroutine_is_task(slang_system_subroutine sub) {
    SLANG_C_ACCESS(false, {
        return sub &&
               reinterpret_cast<const SystemSubroutine*>(sub)->kind == SubroutineKind::Task;
    });
}

bool slang_system_subroutine_has_output_args(slang_system_subroutine sub) {
    SLANG_C_ACCESS(false, {
        return sub && reinterpret_cast<const SystemSubroutine*>(sub)->hasOutputArgs;
    });
}

uint32_t slang_system_subroutine_kind(slang_system_subroutine sub) {
    SLANG_C_ACCESS(0, {
        if (!sub)
            return 0;
        return static_cast<uint32_t>(reinterpret_cast<const SystemSubroutine*>(sub)->kind);
    });
}

uint32_t slang_system_subroutine_known_name_id(slang_system_subroutine sub) {
    SLANG_C_ACCESS(0, {
        if (!sub)
            return 0;
        return static_cast<uint32_t>(
            reinterpret_cast<const SystemSubroutine*>(sub)->knownNameId);
    });
}

uint32_t slang_system_subroutine_with_clause_mode(slang_system_subroutine sub) {
    SLANG_C_ACCESS(0, {
        if (!sub)
            return 0;
        return static_cast<uint32_t>(
            reinterpret_cast<const SystemSubroutine*>(sub)->withClauseMode);
    });
}

slang_system_subroutine slang_compilation_get_system_method(slang_compilation comp,
                                                             uint32_t type_kind, const char* name,
                                                             size_t name_len) {
    if (!comp || !name)
        return nullptr;
    SLANG_C_ACCESS(nullptr, {
        auto sub = comp->comp->getSystemMethod(static_cast<SymbolKind>(type_kind),
                                               toView(name, name_len));
        return reinterpret_cast<slang_system_subroutine>(const_cast<SystemSubroutine*>(sub));
    });
}

bool slang_system_subroutine_allow_empty_argument(slang_system_subroutine sub,
                                                  uint32_t arg_index) {
    SLANG_C_ACCESS(false, {
        if (!sub)
            return false;
        return reinterpret_cast<const SystemSubroutine*>(sub)->allowEmptyArgument(
            (size_t)arg_index);
    });
}

bool slang_system_subroutine_allow_clocking_argument(slang_system_subroutine sub,
                                                      uint32_t arg_index) {
    SLANG_C_ACCESS(false, {
        if (!sub)
            return false;
        return reinterpret_cast<const SystemSubroutine*>(sub)->allowClockingArgument(
            (size_t)arg_index);
    });
}

namespace {

// Recovers the bound CallExpression + its SystemCallInfo behind `node`, or
// nullptr if it is not a bound system call. Shared by the three
// slang_system_subroutine_{check_arguments,bind_argument,eval} accessors
// below, each of which replays one virtual call against the call's own
// already-elaborated arguments/range/scope.
const CallExpression* systemCallOf(slang_ast node) {
    auto e = exprOf(node);
    if (!e || e->kind != ExpressionKind::Call)
        return nullptr;
    auto& call = e->as<CallExpression>();
    if (!call.isSystemCall())
        return nullptr;
    return &call;
}

// Exposes SystemSubroutine's `protected` replay helpers (kindStr, badArg,
// notConst, noHierarchical, checkArgCount, the static unevaluatedContext) to
// the C API. This adds no data members and overrides no virtuals, so a
// `static_cast` from any `const SystemSubroutine*` to
// `const SystemSubroutinePublicist*` is a same-address reinterpretation:
// calling one of these (non-virtual) members through it runs exactly the
// protected member on the original object. Standard "publicist" idiom for
// reaching a base class's protected members from outside its hierarchy.
class SystemSubroutinePublicist : public SystemSubroutine {
public:
    using SystemSubroutine::badArg;
    using SystemSubroutine::checkArgCount;
    using SystemSubroutine::kindStr;
    using SystemSubroutine::noHierarchical;
    using SystemSubroutine::notConst;
    using SystemSubroutine::unevaluatedContext;
};

const SystemSubroutinePublicist& publicistOf(const SystemSubroutine& sub) {
    return static_cast<const SystemSubroutinePublicist&>(sub);
}

// Reconstructs the `iterOrThis` argument that CallExpression::createSystemCall
// passed to checkArguments/eval at elaboration time, from the SystemCallInfo
// slang stored on the call: the iterator expression for an iterator-method
// call, the call's own first argument (the "this" value) for a randomize
// call, and null otherwise. See CallExpression.cpp's createSystemCall.
const Expression* iterOrThisOf(const CallExpression& call, const CallExpression::SystemCallInfo& info) {
    if (auto iterInfo = std::get_if<CallExpression::IteratorCallInfo>(&info.extraInfo))
        return iterInfo->iterExpr;
    if (std::holds_alternative<CallExpression::RandomizeCallInfo>(info.extraInfo) &&
        !call.arguments().empty()) {
        return call.arguments()[0];
    }
    return nullptr;
}

} // namespace

slang_system_subroutine slang_expr_call_system_subroutine(slang_ast node) {
    SLANG_C_ACCESS(nullptr, {
        auto call = systemCallOf(node);
        if (!call)
            return nullptr;
        auto& info = std::get<CallExpression::SystemCallInfo>(call->subroutine);
        return reinterpret_cast<slang_system_subroutine>(
            const_cast<SystemSubroutine*>(info.subroutine.get()));
    });
}

slang_ast slang_system_subroutine_check_arguments(slang_ast call_node, slang_error* err) {
    if (!checkEntry(err))
        return noAst(call_node.compilation);
    SLANG_C_GUARD(err, {
        auto call = systemCallOf(call_node);
        if (!call) {
            setError(err, SLANG_ERR_INVALID_ARG, "not a bound system call");
            return noAst(call_node.compilation);
        }
        SealLift guard(call_node.compilation);
        auto& info = std::get<CallExpression::SystemCallInfo>(call->subroutine);
        ASTContext context(*info.scope, LookupLocation::max);
        auto& type = info.subroutine->checkArguments(context, call->arguments(),
                                                      call->sourceRange, iterOrThisOf(*call, info));
        return wrapAst(type, call_node.compilation);
    });
    return noAst(call_node.compilation);
}

slang_ast slang_system_subroutine_bind_argument(slang_ast call_node, uint32_t arg_index,
                                                slang_error* err) {
    if (!checkEntry(err))
        return noAst(call_node.compilation, SLANG_AST_EXPRESSION);
    SLANG_C_GUARD(err, {
        auto call = systemCallOf(call_node);
        if (!call) {
            setError(err, SLANG_ERR_INVALID_ARG, "not a bound system call");
            return noAst(call_node.compilation, SLANG_AST_EXPRESSION);
        }
        auto previousArgs = call->arguments();
        if (arg_index >= previousArgs.size()) {
            setError(err, SLANG_ERR_INVALID_ARG, "arg_index out of range");
            return noAst(call_node.compilation, SLANG_AST_EXPRESSION);
        }
        // Recover the *original* argument syntax for this index from the
        // call's own InvocationExpressionSyntax, exactly like
        // createSystemCall does before calling bindArgument the first time.
        auto syn = call->syntax;
        if (!syn || syn->kind != syntax::SyntaxKind::InvocationExpression) {
            setError(err, SLANG_ERR_INVALID_ARG,
                     "call has no InvocationExpressionSyntax (e.g. a with-clause "
                     "iterator/randomize argument, bound through a different path)");
            return noAst(call_node.compilation, SLANG_AST_EXPRESSION);
        }
        auto& invocation = syn->as<syntax::InvocationExpressionSyntax>();
        if (!invocation.arguments || arg_index >= invocation.arguments->parameters.size()) {
            setError(err, SLANG_ERR_INVALID_ARG, "arg_index out of range in call syntax");
            return noAst(call_node.compilation, SLANG_AST_EXPRESSION);
        }
        auto argSyntax = invocation.arguments->parameters[arg_index];
        if (argSyntax->kind != syntax::SyntaxKind::OrderedArgument) {
            setError(err, SLANG_ERR_INVALID_ARG, "argument syntax is not an ordered argument");
            return noAst(call_node.compilation, SLANG_AST_EXPRESSION);
        }

        SealLift guard(call_node.compilation);
        auto& info = std::get<CallExpression::SystemCallInfo>(call->subroutine);
        ASTContext context(*info.scope, LookupLocation::max);
        auto exSyn = context.requireSimpleExpr(*argSyntax->as<syntax::OrderedArgumentSyntax>().expr);
        if (!exSyn) {
            setError(err, SLANG_ERR_INVALID_ARG, "argument syntax is not a simple expression");
            return noAst(call_node.compilation, SLANG_AST_EXPRESSION);
        }

        auto& newArg = info.subroutine->bindArgument(arg_index, context, *exSyn,
                                                      previousArgs.subspan(0, arg_index));
        return wrapAst(newArg, call_node.compilation);
    });
    return noAst(call_node.compilation, SLANG_AST_EXPRESSION);
}

slang_constant slang_system_subroutine_eval(slang_ast call_node, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        auto call = systemCallOf(call_node);
        if (!call) {
            setError(err, SLANG_ERR_INVALID_ARG, "not a bound system call");
            return (slang_constant) nullptr;
        }
        SealLift guard(call_node.compilation);
        auto& info = std::get<CallExpression::SystemCallInfo>(call->subroutine);
        EvalContext evalCtx(ASTContext(*info.scope, LookupLocation::max));
        ConstantValue cv = info.subroutine->eval(evalCtx, call->arguments(), call->sourceRange,
                                                 info);
        if (cv.bad())
            return (slang_constant) nullptr;
        return new slang_constant_t{std::move(cv)};
    });
    return nullptr;
}

slang_str slang_system_subroutine_kind_str(slang_system_subroutine sub) {
    SLANG_C_ACCESS(borrowed(""), {
        if (!sub)
            return borrowed("");
        return borrowed(publicistOf(*reinterpret_cast<const SystemSubroutine*>(sub)).kindStr());
    });
}

slang_ast slang_system_subroutine_bad_arg(slang_ast call_node, uint32_t arg_index,
                                          slang_error* err) {
    if (!checkEntry(err))
        return noAst(call_node.compilation);
    SLANG_C_GUARD(err, {
        auto call = systemCallOf(call_node);
        if (!call) {
            setError(err, SLANG_ERR_INVALID_ARG, "not a bound system call");
            return noAst(call_node.compilation);
        }
        if (arg_index >= call->arguments().size()) {
            setError(err, SLANG_ERR_INVALID_ARG, "arg_index out of range");
            return noAst(call_node.compilation);
        }
        SealLift guard(call_node.compilation);
        auto& info = std::get<CallExpression::SystemCallInfo>(call->subroutine);
        ASTContext context(*info.scope, LookupLocation::max);
        auto& type = publicistOf(*info.subroutine)
                         .badArg(context, *call->arguments()[arg_index]);
        return wrapAst(type, call_node.compilation);
    });
    return noAst(call_node.compilation);
}

bool slang_system_subroutine_check_arg_count(slang_ast call_node, bool is_method, uint32_t min,
                                             uint32_t max, slang_error* err) {
    if (!checkEntry(err))
        return false;
    SLANG_C_GUARD(err, {
        auto call = systemCallOf(call_node);
        if (!call) {
            setError(err, SLANG_ERR_INVALID_ARG, "not a bound system call");
            return false;
        }
        SealLift guard(call_node.compilation);
        auto& info = std::get<CallExpression::SystemCallInfo>(call->subroutine);
        ASTContext context(*info.scope, LookupLocation::max);
        return publicistOf(*info.subroutine)
            .checkArgCount(context, is_method, call->arguments(), call->sourceRange, min, max);
    });
    return false;
}

bool slang_system_subroutine_no_hierarchical(slang_ast call_node, uint32_t arg_index,
                                             slang_error* err) {
    if (!checkEntry(err))
        return false;
    SLANG_C_GUARD(err, {
        auto call = systemCallOf(call_node);
        if (!call) {
            setError(err, SLANG_ERR_INVALID_ARG, "not a bound system call");
            return false;
        }
        if (arg_index >= call->arguments().size()) {
            setError(err, SLANG_ERR_INVALID_ARG, "arg_index out of range");
            return false;
        }
        SealLift guard(call_node.compilation);
        auto& info = std::get<CallExpression::SystemCallInfo>(call->subroutine);
        EvalContext evalCtx(ASTContext(*info.scope, LookupLocation::max));
        return publicistOf(*info.subroutine)
            .noHierarchical(evalCtx, *call->arguments()[arg_index]);
    });
    return false;
}

bool slang_system_subroutine_not_const(slang_ast call_node, slang_error* err) {
    if (!checkEntry(err))
        return false;
    SLANG_C_GUARD(err, {
        auto call = systemCallOf(call_node);
        if (!call) {
            setError(err, SLANG_ERR_INVALID_ARG, "not a bound system call");
            return false;
        }
        SealLift guard(call_node.compilation);
        auto& info = std::get<CallExpression::SystemCallInfo>(call->subroutine);
        EvalContext evalCtx(ASTContext(*info.scope, LookupLocation::max));
        return publicistOf(*info.subroutine).notConst(evalCtx, call->sourceRange);
    });
    return false;
}

bool slang_system_subroutine_unevaluated_context_clears_static_initializer(slang_ast call_node,
                                                                           slang_error* err) {
    if (!checkEntry(err))
        return false;
    SLANG_C_GUARD(err, {
        auto call = systemCallOf(call_node);
        if (!call) {
            setError(err, SLANG_ERR_INVALID_ARG, "not a bound system call");
            return false;
        }
        SealLift guard(call_node.compilation);
        auto& info = std::get<CallExpression::SystemCallInfo>(call->subroutine);
        // A second flag (AssignmentAllowed) that unevaluatedContext must leave
        // untouched, so this proves the transform is targeted rather than
        // wiping every flag.
        ASTContext source(*info.scope, LookupLocation::max,
                          ASTFlags::StaticInitializer | ASTFlags::AssignmentAllowed);
        ASTContext result = SystemSubroutinePublicist::unevaluatedContext(source);
        return source.flags.has(ASTFlags::StaticInitializer) &&
               !result.flags.has(ASTFlags::StaticInitializer) &&
               result.flags.has(ASTFlags::AssignmentAllowed);
    });
    return false;
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

slang_str slang_symbol_lexical_path(slang_ast symbol, slang_error* err) {
    if (!checkEntry(err))
        return borrowed("");
    auto sym = symbolOf(symbol);
    if (!sym) {
        setError(err, SLANG_ERR_INVALID_ARG, "not a symbol");
        return borrowed("");
    }
    // Walks only already-resolved parent-scope pointers and builds a fresh
    // std::string on the caller's own heap (no slang arena allocation), so
    // this is a pure read; still SLANG_C_GUARD-ed like its hierarchical-path
    // sibling in case of an unexpected assertion.
    SLANG_C_GUARD(err, { return owned(sym->getLexicalPath()); });
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

bool slang_symbol_has_declared_type(slang_ast symbol) {
    SLANG_C_ACCESS(false, {
        auto sym = symbolOf(symbol);
        // Same memo every slang_declared_type_* accessor re-derives from
        // `sym` itself; forced for every carrier by the freeze sweep, so
        // this is a pure read.
        return sym && sym->getDeclaredType() != nullptr;
    });
}

slang_ast slang_symbol_declaring_definition(slang_ast symbol) {
    SLANG_C_ACCESS(noAst(symbol.compilation), {
        auto sym = symbolOf(symbol);
        if (!sym)
            return noAst(symbol.compilation);
        auto def = sym->getDeclaringDefinition();
        return def ? toC(def, symbol.compilation) : noAst(symbol.compilation);
    });
}

slang_source_library slang_symbol_source_library(slang_ast symbol) {
    SLANG_C_ACCESS(nullptr, {
        auto sym = symbolOf(symbol);
        auto lib = sym ? sym->getSourceLibrary() : nullptr;
        return reinterpret_cast<slang_source_library>(const_cast<SourceLibrary*>(lib));
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

slang_compilation slang_scope_get_compilation(slang_ast scope) {
    SLANG_C_ACCESS(nullptr, {
        auto sym = symbolOf(scope);
        auto s = sym ? scopeOf(*sym) : nullptr;
        return s ? scope.compilation : nullptr;
    });
}

slang_ast slang_scope_get_compilation_unit(slang_ast scope) {
    SLANG_C_ACCESS(noAst(scope.compilation), {
        auto sym = symbolOf(scope);
        auto s = sym ? scopeOf(*sym) : nullptr;
        if (!s)
            return noAst(scope.compilation);
        return capi::toC(s->getCompilationUnit(), scope.compilation);
    });
}

slang_ast slang_scope_get_containing_instance(slang_ast scope) {
    SLANG_C_ACCESS(noAst(scope.compilation), {
        auto sym = symbolOf(scope);
        auto s = sym ? scopeOf(*sym) : nullptr;
        if (!s)
            return noAst(scope.compilation);
        return capi::toC(s->getContainingInstance(), scope.compilation);
    });
}

slang_ast slang_scope_get_default_net_type(slang_ast scope) {
    SLANG_C_ACCESS(noAst(scope.compilation), {
        auto sym = symbolOf(scope);
        auto s = sym ? scopeOf(*sym) : nullptr;
        if (!s)
            return noAst(scope.compilation);
        return toC(&s->getDefaultNetType(), scope.compilation);
    });
}

bool slang_scope_get_time_scale(slang_ast scope, slang_time_scale* out) {
    SLANG_C_ACCESS(false, {
        auto sym = symbolOf(scope);
        auto s = sym ? scopeOf(*sym) : nullptr;
        if (!s)
            return false;
        auto ts = s->getTimeScale();
        if (!ts)
            return false;
        if (out) {
            *out = slang_time_scale{
                (uint8_t)ts->base.unit,
                (uint8_t)ts->base.magnitude,
                (uint8_t)ts->precision.unit,
                (uint8_t)ts->precision.magnitude,
            };
        }
        return true;
    });
}

bool slang_scope_is_procedural_context(slang_ast scope) {
    SLANG_C_ACCESS(false, {
        auto sym = symbolOf(scope);
        auto s = sym ? scopeOf(*sym) : nullptr;
        return s && s->isProceduralContext();
    });
}

bool slang_scope_is_uninstantiated(slang_ast scope) {
    SLANG_C_ACCESS(false, {
        auto sym = symbolOf(scope);
        auto s = sym ? scopeOf(*sym) : nullptr;
        return s && s->isUninstantiated();
    });
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

slang_ast slang_declared_type_type(slang_ast sym_node) {
    SLANG_C_ACCESS(noAst(sym_node.compilation), {
        auto sym = symbolOf(sym_node);
        auto dt = sym ? sym->getDeclaredType() : nullptr;
        if (!dt)
            return noAst(sym_node.compilation);
        // Forced for every DeclaredType carrier by the freeze sweep
        // (FreezeVisitor's Symbol branch: `if (auto dt = t.getDeclaredType())
        // dt->getType()`), so this is a pure read.
        return toC(&dt->getType(), sym_node.compilation);
    });
}

slang_ast slang_declared_type_initializer(slang_ast sym_node) {
    SLANG_C_ACCESS(noAst(sym_node.compilation, SLANG_AST_EXPRESSION), {
        auto sym = symbolOf(sym_node);
        auto dt = sym ? sym->getDeclaredType() : nullptr;
        // Forced alongside getType() by the same freeze-sweep force.
        return capi::toC(dt ? dt->getInitializer() : nullptr, sym_node.compilation);
    });
}

slang_loc slang_declared_type_initializer_location(slang_ast sym_node) {
    SLANG_C_ACCESS(slang_loc{}, {
        auto sym = symbolOf(sym_node);
        auto dt = sym ? sym->getDeclaredType() : nullptr;
        if (!dt)
            return slang_loc{};
        return capi::toC(dt->getInitializerLocation());
    });
}

slang_node slang_declared_type_initializer_syntax(slang_ast sym_node) {
    SLANG_C_ACCESS(noNode(nullptr), {
        auto sym = symbolOf(sym_node);
        auto dt = sym ? sym->getDeclaredType() : nullptr;
        auto syntax = dt ? dt->getInitializerSyntax() : nullptr;
        if (!syntax || !sym_node.compilation)
            return noNode(nullptr);
        auto tree = findTree(sym_node.compilation, *syntax);
        return tree ? capi::toC(syntax, tree) : noNode(nullptr);
    });
}

slang_node slang_declared_type_type_syntax(slang_ast sym_node) {
    SLANG_C_ACCESS(noNode(nullptr), {
        auto sym = symbolOf(sym_node);
        auto dt = sym ? sym->getDeclaredType() : nullptr;
        auto syntax = dt ? dt->getTypeSyntax() : nullptr;
        if (!syntax || !sym_node.compilation)
            return noNode(nullptr);
        auto tree = findTree(sym_node.compilation, *syntax);
        return tree ? capi::toC(syntax, tree) : noNode(nullptr);
    });
}

bool slang_declared_type_is_evaluating(slang_ast sym_node) {
    SLANG_C_ACCESS(false, {
        auto sym = symbolOf(sym_node);
        auto dt = sym ? sym->getDeclaredType() : nullptr;
        return dt && dt->isEvaluating();
    });
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

bool slang_instance_is_module(slang_ast instance) {
    SLANG_C_ACCESS(false, {
        auto sym = symbolOf(instance);
        if (!sym || sym->kind != SymbolKind::Instance)
            return false;
        return sym->as<InstanceSymbol>().isModule();
    });
}

bool slang_instance_is_interface(slang_ast instance) {
    SLANG_C_ACCESS(false, {
        auto sym = symbolOf(instance);
        if (!sym || sym->kind != SymbolKind::Instance)
            return false;
        return sym->as<InstanceSymbol>().isInterface();
    });
}

namespace {

// Shared bounds-check + connection lookup for the
// slang_instance_port_connection_* family below. The freeze sweep already
// forces getPortConnections() (and every connection's getExpression) for
// every Instance symbol it visits, so this is a pure read on a frozen
// design.
const PortConnection* instancePortConnection(slang_ast instance, uint32_t index) {
    auto sym = symbolOf(instance);
    if (!sym || sym->kind != SymbolKind::Instance)
        return nullptr;
    auto conns = sym->as<InstanceSymbol>().getPortConnections();
    if (index >= conns.size())
        return nullptr;
    return conns[index];
}

} // namespace

uint32_t slang_instance_port_connection_count(slang_ast instance) {
    SLANG_C_ACCESS(0u, {
        auto sym = symbolOf(instance);
        if (!sym || sym->kind != SymbolKind::Instance)
            return 0u;
        return (uint32_t)sym->as<InstanceSymbol>().getPortConnections().size();
    });
}

slang_ast slang_instance_port_connection_port(slang_ast instance, uint32_t index) {
    SLANG_C_ACCESS(noAst(instance.compilation), {
        auto conn = instancePortConnection(instance, index);
        if (!conn)
            return noAst(instance.compilation);
        return capi::toC(&conn->port, instance.compilation);
    });
}

slang_ast slang_instance_port_connection_expression(slang_ast instance, uint32_t index) {
    SLANG_C_ACCESS(noAst(instance.compilation, SLANG_AST_EXPRESSION), {
        auto conn = instancePortConnection(instance, index);
        auto expr = conn ? conn->getExpression() : nullptr;
        return expr ? wrapAst(*expr, instance.compilation)
                    : noAst(instance.compilation, SLANG_AST_EXPRESSION);
    });
}

bool slang_instance_port_connection_is_implicit(slang_ast instance, uint32_t index) {
    SLANG_C_ACCESS(false, {
        auto conn = instancePortConnection(instance, index);
        return conn && conn->isImplicit;
    });
}

bool slang_instance_port_connection_is_wildcard(slang_ast instance, uint32_t index) {
    SLANG_C_ACCESS(false, {
        auto conn = instancePortConnection(instance, index);
        return conn && conn->isWildcard;
    });
}

slang_ast slang_symbol_instance_body_parent_instance(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::InstanceBody)
            return noAst(sym.compilation);
        auto parent = s->as<InstanceBodySymbol>().parentInstance;
        return parent ? capi::toC(parent, sym.compilation) : noAst(sym.compilation);
    });
}

slang_ast slang_symbol_instance_body_definition(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::InstanceBody)
            return noAst(sym.compilation);
        return capi::toC(&s->as<InstanceBodySymbol>().getDefinition(), sym.compilation);
    });
}

uint32_t slang_symbol_instance_body_port_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::InstanceBody)
            return 0u;
        return (uint32_t)s->as<InstanceBodySymbol>().getPortList().size();
    });
}

slang_ast slang_symbol_instance_body_port(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::InstanceBody)
            return noAst(sym.compilation);
        auto ports = s->as<InstanceBodySymbol>().getPortList();
        if (index >= ports.size())
            return noAst(sym.compilation);
        return capi::toC(ports[index], sym.compilation);
    });
}

slang_ast slang_symbol_instance_body_find_port(slang_ast sym, const char* name,
                                                size_t name_len) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::InstanceBody || !name)
            return noAst(sym.compilation);
        auto port = s->as<InstanceBodySymbol>().findPort(toView(name, name_len));
        return port ? capi::toC(port, sym.compilation) : noAst(sym.compilation);
    });
}

bool slang_symbol_instance_body_has_same_type(slang_ast sym, slang_ast other) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        auto o = symbolOf(other);
        if (!s || s->kind != SymbolKind::InstanceBody || !o ||
            o->kind != SymbolKind::InstanceBody) {
            return false;
        }
        return s->as<InstanceBodySymbol>().hasSameType(o->as<InstanceBodySymbol>());
    });
}

uint32_t slang_symbol_instance_array_path_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || !InstanceSymbolBase::isKind(s->kind))
            return 0u;
        return (uint32_t)s->as<InstanceSymbolBase>().arrayPath.size();
    });
}

uint32_t slang_symbol_instance_array_path(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || !InstanceSymbolBase::isKind(s->kind))
            return 0u;
        auto path = s->as<InstanceSymbolBase>().arrayPath;
        if (index >= path.size())
            return 0u;
        return path[index];
    });
}

slang_str slang_symbol_instance_base_array_name(slang_ast sym) {
    SLANG_C_ACCESS(borrowed(""), {
        auto s = symbolOf(sym);
        if (!s || !InstanceSymbolBase::isKind(s->kind))
            return borrowed("");
        return borrowed(s->as<InstanceSymbolBase>().getArrayName());
    });
}

uint32_t slang_symbol_instance_array_element_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::InstanceArray)
            return 0u;
        return (uint32_t)s->as<InstanceArraySymbol>().elements.size();
    });
}

slang_ast slang_symbol_instance_array_element(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::InstanceArray)
            return noAst(sym.compilation);
        auto elements = s->as<InstanceArraySymbol>().elements;
        if (index >= elements.size())
            return noAst(sym.compilation);
        return capi::toC(elements[index], sym.compilation);
    });
}

slang_constant_range slang_symbol_instance_array_range(slang_ast sym) {
    SLANG_C_ACCESS(slang_constant_range{}, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::InstanceArray)
            return slang_constant_range{};
        auto& r = s->as<InstanceArraySymbol>().range;
        return slang_constant_range{r.left, r.right};
    });
}

slang_str slang_symbol_instance_array_name(slang_ast sym) {
    SLANG_C_ACCESS(borrowed(""), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::InstanceArray)
            return borrowed("");
        return borrowed(s->as<InstanceArraySymbol>().getArrayName());
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

slang_argument_direction slang_symbol_assertion_port_direction(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_ARGUMENT_DIRECTION_NONE, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::AssertionPort)
            return SLANG_ARGUMENT_DIRECTION_NONE;
        auto& dir = s->as<AssertionPortSymbol>().direction;
        return dir ? (slang_argument_direction)(uint32_t)*dir : SLANG_ARGUMENT_DIRECTION_NONE;
    });
}

bool slang_symbol_assertion_port_is_local_var(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::AssertionPort)
            return false;
        return s->as<AssertionPortSymbol>().isLocalVar();
    });
}

slang_ast slang_symbol_checker_instance_body_parent_instance(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::CheckerInstanceBody)
            return noAst(sym.compilation);
        auto parent = s->as<CheckerInstanceBodySymbol>().parentInstance;
        return parent ? capi::toC(parent, sym.compilation) : noAst(sym.compilation);
    });
}

uint32_t slang_symbol_checker_port_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Checker)
            return 0u;
        return (uint32_t)s->as<CheckerSymbol>().ports.size();
    });
}

slang_ast slang_symbol_checker_port(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Checker)
            return noAst(sym.compilation);
        auto ports = s->as<CheckerSymbol>().ports;
        if (index >= ports.size())
            return noAst(sym.compilation);
        return capi::toC(ports[index], sym.compilation);
    });
}

namespace {

// Shared bounds-check + connection lookup for the
// slang_symbol_checker_instance_connection_* family below.
const CheckerInstanceSymbol::Connection* checkerConnection(slang_ast instance, uint32_t index) {
    auto s = symbolOf(instance);
    if (!s || s->kind != SymbolKind::CheckerInstance)
        return nullptr;
    // The freeze sweep already forces getPortConnections() (and every
    // connection's getOutputInitialExpr) for every CheckerInstance symbol
    // it visits, so this is a pure read on a frozen design.
    auto conns = s->as<CheckerInstanceSymbol>().getPortConnections();
    if (index >= conns.size())
        return nullptr;
    return &conns[index];
}

} // namespace

uint32_t slang_symbol_checker_instance_connection_count(slang_ast instance) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(instance);
        if (!s || s->kind != SymbolKind::CheckerInstance)
            return 0u;
        return (uint32_t)s->as<CheckerInstanceSymbol>().getPortConnections().size();
    });
}

slang_ast slang_symbol_checker_instance_connection_actual(slang_ast instance, uint32_t index) {
    SLANG_C_ACCESS(noAst(instance.compilation, SLANG_AST_EXPRESSION), {
        auto conn = checkerConnection(instance, index);
        if (!conn)
            return noAst(instance.compilation, SLANG_AST_EXPRESSION);
        auto comp = instance.compilation;
        return std::visit(
            [comp](auto* ptr) -> slang_ast {
                if (!ptr)
                    return noAst(comp, SLANG_AST_EXPRESSION);
                return wrapAst(*ptr, comp);
            },
            conn->actual);
    });
}

uint32_t slang_symbol_checker_instance_connection_attribute_count(slang_ast instance,
                                                                   uint32_t index) {
    SLANG_C_ACCESS(0u, {
        auto conn = checkerConnection(instance, index);
        return conn ? (uint32_t)conn->attributes.size() : 0u;
    });
}

slang_ast slang_symbol_checker_instance_connection_attribute(slang_ast instance, uint32_t index,
                                                              uint32_t attr_index) {
    SLANG_C_ACCESS(noAst(instance.compilation), {
        auto conn = checkerConnection(instance, index);
        if (!conn || attr_index >= conn->attributes.size())
            return noAst(instance.compilation);
        return capi::toC(conn->attributes[attr_index], instance.compilation);
    });
}

slang_ast slang_symbol_checker_instance_connection_output_initial_expr(slang_ast instance,
                                                                        uint32_t index) {
    SLANG_C_ACCESS(noAst(instance.compilation, SLANG_AST_EXPRESSION), {
        auto conn = checkerConnection(instance, index);
        if (!conn)
            return noAst(instance.compilation, SLANG_AST_EXPRESSION);
        auto expr = conn->getOutputInitialExpr();
        return expr ? wrapAst(*expr, instance.compilation)
                    : noAst(instance.compilation, SLANG_AST_EXPRESSION);
    });
}

slang_visibility slang_symbol_class_property_visibility(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_VISIBILITY_PUBLIC, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ClassProperty)
            return SLANG_VISIBILITY_PUBLIC;
        return (slang_visibility)(uint32_t)s->as<ClassPropertySymbol>().visibility;
    });
}

slang_rand_mode slang_symbol_class_property_rand_mode(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_RAND_MODE_NONE, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ClassProperty)
            return SLANG_RAND_MODE_NONE;
        return (slang_rand_mode)(uint32_t)s->as<ClassPropertySymbol>().randMode;
    });
}

namespace {

slang_clocking_skew noSkew() {
    return slang_clocking_skew{SLANG_EDGE_NONE, slang_ast{nullptr, nullptr, 0,
                                                           SLANG_AST_TIMING_CONTROL}};
}

slang_clocking_skew toC(const ClockingSkew& skew, slang_compilation comp) {
    slang_ast delay = skew.delay ? wrapAst(*skew.delay, comp) : noAst(comp, SLANG_AST_TIMING_CONTROL);
    return slang_clocking_skew{(slang_edge_kind)(uint32_t)skew.edge, delay};
}

} // namespace

bool slang_clocking_skew_has_value(slang_clocking_skew skew) {
    SLANG_C_ACCESS(false, { return skew.edge != SLANG_EDGE_NONE || skew.delay.ptr != nullptr; });
}

slang_argument_direction slang_symbol_clock_var_direction(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_ARGUMENT_DIRECTION_IN, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ClockVar)
            return SLANG_ARGUMENT_DIRECTION_IN;
        return (slang_argument_direction)(uint32_t)s->as<ClockVarSymbol>().direction;
    });
}

slang_clocking_skew slang_symbol_clock_var_input_skew(slang_ast sym) {
    SLANG_C_ACCESS(noSkew(), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ClockVar)
            return noSkew();
        return toC(s->as<ClockVarSymbol>().inputSkew, sym.compilation);
    });
}

slang_clocking_skew slang_symbol_clock_var_output_skew(slang_ast sym) {
    SLANG_C_ACCESS(noSkew(), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ClockVar)
            return noSkew();
        return toC(s->as<ClockVarSymbol>().outputSkew, sym.compilation);
    });
}

slang_clocking_skew slang_symbol_clocking_block_default_input_skew(slang_ast sym) {
    SLANG_C_ACCESS(noSkew(), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ClockingBlock)
            return noSkew();
        // Forced by the freeze sweep; a pure read on a frozen design.
        return toC(s->as<ClockingBlockSymbol>().getDefaultInputSkew(), sym.compilation);
    });
}

slang_clocking_skew slang_symbol_clocking_block_default_output_skew(slang_ast sym) {
    SLANG_C_ACCESS(noSkew(), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ClockingBlock)
            return noSkew();
        // Forced by the freeze sweep; a pure read on a frozen design.
        return toC(s->as<ClockingBlockSymbol>().getDefaultOutputSkew(), sym.compilation);
    });
}

slang_ast slang_symbol_clocking_block_event(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_TIMING_CONTROL), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ClockingBlock)
            return noAst(sym.compilation, SLANG_AST_TIMING_CONTROL);
        // Forced by the freeze sweep; a pure read on a frozen design.
        return wrapAst(s->as<ClockingBlockSymbol>().getEvent(), sym.compilation);
    });
}

slang_ast slang_symbol_continuous_assign_assignment(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ContinuousAssign)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        // Forced by the freeze sweep; a pure read on a frozen design.
        return wrapAst(s->as<ContinuousAssignSymbol>().getAssignment(), sym.compilation);
    });
}

slang_ast slang_symbol_continuous_assign_delay(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_TIMING_CONTROL), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ContinuousAssign)
            return noAst(sym.compilation, SLANG_AST_TIMING_CONTROL);
        // Forced by the freeze sweep; a pure read on a frozen design.
        auto delay = s->as<ContinuousAssignSymbol>().getDelay();
        return delay ? wrapAst(*delay, sym.compilation)
                     : noAst(sym.compilation, SLANG_AST_TIMING_CONTROL);
    });
}

slang_drive_strength_pair slang_symbol_continuous_assign_drive_strength(slang_ast sym) {
    SLANG_C_ACCESS(slang_drive_strength_pair{}, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ContinuousAssign)
            return slang_drive_strength_pair{};
        // Recomputed from syntax on every call; no arena allocation.
        auto [ds0, ds1] = s->as<ContinuousAssignSymbol>().getDriveStrength();
        slang_drive_strength_pair result{};
        if (ds0) {
            result.has_strength0 = true;
            result.strength0 = (slang_drive_strength)(uint32_t)*ds0;
        }
        if (ds1) {
            result.has_strength1 = true;
            result.strength1 = (slang_drive_strength)(uint32_t)*ds1;
        }
        return result;
    });
}

bool slang_symbol_compilation_unit_time_scale(slang_ast sym, slang_time_scale* out) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::CompilationUnit)
            return false;
        auto& ts = s->as<CompilationUnitSymbol>().timeScale;
        if (!ts)
            return false;
        if (out) {
            *out = slang_time_scale{
                (uint8_t)ts->base.unit,
                (uint8_t)ts->base.magnitude,
                (uint8_t)ts->precision.unit,
                (uint8_t)ts->precision.magnitude,
            };
        }
        return true;
    });
}

slang_ast slang_symbol_cover_cross_body_queue_type(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::CoverCrossBody)
            return noAst(sym.compilation);
        auto type = s->as<CoverCrossBodySymbol>().crossQueueType;
        return type ? capi::toC(type, sym.compilation) : noAst(sym.compilation);
    });
}

slang_ast slang_symbol_cover_cross_iff_expr(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::CoverCross)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        // Forced by the freeze sweep; a pure read on a frozen design.
        auto iff = s->as<CoverCrossSymbol>().getIffExpr();
        return iff ? wrapAst(*iff, sym.compilation) : noAst(sym.compilation, SLANG_AST_EXPRESSION);
    });
}

uint32_t slang_symbol_cover_cross_target_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::CoverCross)
            return 0u;
        return (uint32_t)s->as<CoverCrossSymbol>().targets.size();
    });
}

slang_ast slang_symbol_cover_cross_target(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::CoverCross)
            return noAst(sym.compilation);
        auto targets = s->as<CoverCrossSymbol>().targets;
        if (index >= targets.size())
            return noAst(sym.compilation);
        return capi::toC(targets[index], sym.compilation);
    });
}

namespace {

// The `options` span of any symbol kind that can carry `option`/
// `type_option` setters directly in its body (CoverCross, CovergroupBody,
// Coverpoint) — shared by the slang_symbol_cover_cross_option_* family
// below, which despite its CoverCross-derived name works for all three.
// Empty for any other symbol kind.
std::span<const CoverageOptionSetter> coverageOptionsOf(const Symbol* s) {
    if (!s)
        return {};
    switch (s->kind) {
        case SymbolKind::CoverCross:
            return s->as<CoverCrossSymbol>().options;
        case SymbolKind::CovergroupBody:
            return s->as<CovergroupBodySymbol>().options;
        case SymbolKind::Coverpoint:
            return s->as<CoverpointSymbol>().options;
        default:
            return {};
    }
}

// Shared bounds-check + option lookup for the
// slang_symbol_cover_cross_option_* family below.
const CoverageOptionSetter* coverCrossOption(slang_ast sym, uint32_t index) {
    auto options = coverageOptionsOf(symbolOf(sym));
    if (index >= options.size())
        return nullptr;
    return &options[index];
}

} // namespace

uint32_t slang_symbol_cover_cross_option_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, { return (uint32_t)coverageOptionsOf(symbolOf(sym)).size(); });
}

bool slang_symbol_cover_cross_option_is_type_option(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(false, {
        auto opt = coverCrossOption(sym, index);
        return opt && opt->isTypeOption();
    });
}

slang_str slang_symbol_cover_cross_option_name(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(borrowed(""), {
        auto opt = coverCrossOption(sym, index);
        return opt ? borrowed(opt->getName()) : borrowed("");
    });
}

slang_ast slang_symbol_cover_cross_option_expression(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto opt = coverCrossOption(sym, index);
        if (!opt)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        // Forced by the freeze sweep (as part of the owning symbol's own
        // expression walk).
        return wrapAst(opt->getExpression(), sym.compilation);
    });
}

slang_ast slang_symbol_coverpoint_coverage_expr(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Coverpoint)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        // Backed by declaredType.getInitializer(), forced generically by the
        // freeze sweep's getDeclaredType() path — a pure read.
        return wrapAst(s->as<CoverpointSymbol>().getCoverageExpr(), sym.compilation);
    });
}

slang_ast slang_symbol_coverpoint_iff_expr(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Coverpoint)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        // Forced by the freeze sweep.
        auto iff = s->as<CoverpointSymbol>().getIffExpr();
        return iff ? wrapAst(*iff, sym.compilation) : noAst(sym.compilation, SLANG_AST_EXPRESSION);
    });
}

slang_coverage_bin_kind slang_symbol_coverage_bin_kind(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_COVERAGE_BIN_BINS, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::CoverageBin)
            return SLANG_COVERAGE_BIN_BINS;
        switch (s->as<CoverageBinSymbol>().binsKind) {
            case CoverageBinSymbol::IllegalBins:
                return SLANG_COVERAGE_BIN_ILLEGAL_BINS;
            case CoverageBinSymbol::IgnoreBins:
                return SLANG_COVERAGE_BIN_IGNORE_BINS;
            case CoverageBinSymbol::Bins:
            default:
                return SLANG_COVERAGE_BIN_BINS;
        }
    });
}

bool slang_symbol_coverage_bin_is_array(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        return s && s->kind == SymbolKind::CoverageBin && s->as<CoverageBinSymbol>().isArray;
    });
}

bool slang_symbol_coverage_bin_is_wildcard(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        return s && s->kind == SymbolKind::CoverageBin && s->as<CoverageBinSymbol>().isWildcard;
    });
}

bool slang_symbol_coverage_bin_is_default(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        return s && s->kind == SymbolKind::CoverageBin && s->as<CoverageBinSymbol>().isDefault;
    });
}

bool slang_symbol_coverage_bin_is_default_sequence(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        return s && s->kind == SymbolKind::CoverageBin &&
               s->as<CoverageBinSymbol>().isDefaultSequence;
    });
}

slang_ast slang_symbol_coverage_bin_iff_expr(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::CoverageBin)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        // Forced by the freeze sweep.
        auto expr = s->as<CoverageBinSymbol>().getIffExpr();
        return expr ? wrapAst(*expr, sym.compilation) : noAst(sym.compilation, SLANG_AST_EXPRESSION);
    });
}

slang_ast slang_symbol_coverage_bin_number_of_bins_expr(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::CoverageBin)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        // Forced by the freeze sweep.
        auto expr = s->as<CoverageBinSymbol>().getNumberOfBinsExpr();
        return expr ? wrapAst(*expr, sym.compilation) : noAst(sym.compilation, SLANG_AST_EXPRESSION);
    });
}

slang_ast slang_symbol_coverage_bin_set_coverage_expr(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::CoverageBin)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        // Forced by the freeze sweep.
        auto expr = s->as<CoverageBinSymbol>().getSetCoverageExpr();
        return expr ? wrapAst(*expr, sym.compilation) : noAst(sym.compilation, SLANG_AST_EXPRESSION);
    });
}

slang_ast slang_symbol_coverage_bin_with_expr(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::CoverageBin)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        // Forced by the freeze sweep.
        auto expr = s->as<CoverageBinSymbol>().getWithExpr();
        return expr ? wrapAst(*expr, sym.compilation) : noAst(sym.compilation, SLANG_AST_EXPRESSION);
    });
}

slang_ast slang_symbol_coverage_bin_cross_select_expr(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_BINS_SELECT_EXPR), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::CoverageBin)
            return noAst(sym.compilation, SLANG_AST_BINS_SELECT_EXPR);
        // Forced by the freeze sweep.
        auto expr = s->as<CoverageBinSymbol>().getCrossSelectExpr();
        return expr ? wrapAst(*expr, sym.compilation)
                    : noAst(sym.compilation, SLANG_AST_BINS_SELECT_EXPR);
    });
}

uint32_t slang_symbol_coverage_bin_value_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::CoverageBin)
            return 0u;
        // Forced by the freeze sweep.
        return (uint32_t)s->as<CoverageBinSymbol>().getValues().size();
    });
}

slang_ast slang_symbol_coverage_bin_value(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::CoverageBin)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        // Forced by the freeze sweep.
        auto values = s->as<CoverageBinSymbol>().getValues();
        if (index >= values.size())
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(*values[index], sym.compilation);
    });
}

namespace {

// Shared bounds-check + trans-range lookup for the
// slang_symbol_coverage_bin_trans_range_* family below.
const CoverageBinSymbol::TransRangeList* coverageBinTransRange(slang_ast sym, uint32_t setIndex,
                                                                uint32_t rangeIndex) {
    auto s = symbolOf(sym);
    if (!s || s->kind != SymbolKind::CoverageBin)
        return nullptr;
    // Forced by the freeze sweep (as part of the CoverageBin symbol's own
    // expression walk, which also visits every item/repeatFrom/repeatTo
    // expression a set holds).
    auto sets = s->as<CoverageBinSymbol>().getTransList();
    if (setIndex >= sets.size())
        return nullptr;
    auto ranges = sets[setIndex];
    if (rangeIndex >= ranges.size())
        return nullptr;
    return &ranges[rangeIndex];
}

} // namespace

uint32_t slang_symbol_coverage_bin_trans_set_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::CoverageBin)
            return 0u;
        return (uint32_t)s->as<CoverageBinSymbol>().getTransList().size();
    });
}

uint32_t slang_symbol_coverage_bin_trans_range_count(slang_ast sym, uint32_t set_index) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::CoverageBin)
            return 0u;
        auto sets = s->as<CoverageBinSymbol>().getTransList();
        if (set_index >= sets.size())
            return 0u;
        return (uint32_t)sets[set_index].size();
    });
}

uint32_t slang_symbol_coverage_bin_trans_range_item_count(slang_ast sym, uint32_t set_index,
                                                           uint32_t range_index) {
    SLANG_C_ACCESS(0u, {
        auto range = coverageBinTransRange(sym, set_index, range_index);
        return range ? (uint32_t)range->items.size() : 0u;
    });
}

slang_ast slang_symbol_coverage_bin_trans_range_item(slang_ast sym, uint32_t set_index,
                                                      uint32_t range_index, uint32_t item_index) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto range = coverageBinTransRange(sym, set_index, range_index);
        if (!range || item_index >= range->items.size())
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(*range->items[item_index], sym.compilation);
    });
}

slang_repeat_kind slang_symbol_coverage_bin_trans_range_repeat_kind(slang_ast sym,
                                                                     uint32_t set_index,
                                                                     uint32_t range_index) {
    SLANG_C_ACCESS(SLANG_REPEAT_KIND_NONE, {
        auto range = coverageBinTransRange(sym, set_index, range_index);
        return range ? (slang_repeat_kind)(uint32_t)range->repeatKind : SLANG_REPEAT_KIND_NONE;
    });
}

slang_ast slang_symbol_coverage_bin_trans_range_repeat_from(slang_ast sym, uint32_t set_index,
                                                             uint32_t range_index) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto range = coverageBinTransRange(sym, set_index, range_index);
        if (!range || !range->repeatFrom)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(*range->repeatFrom, sym.compilation);
    });
}

slang_ast slang_symbol_coverage_bin_trans_range_repeat_to(slang_ast sym, uint32_t set_index,
                                                           uint32_t range_index) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto range = coverageBinTransRange(sym, set_index, range_index);
        if (!range || !range->repeatTo)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(*range->repeatTo, sym.compilation);
    });
}

bool slang_definition_cell_define(slang_ast definition) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(definition);
        return s && s->kind == SymbolKind::Definition && s->as<DefinitionSymbol>().cellDefine;
    });
}

slang_variable_lifetime slang_definition_default_lifetime(slang_ast definition) {
    SLANG_C_ACCESS(SLANG_VARIABLE_LIFETIME_AUTOMATIC, {
        auto s = symbolOf(definition);
        if (!s || s->kind != SymbolKind::Definition)
            return SLANG_VARIABLE_LIFETIME_AUTOMATIC;
        switch (s->as<DefinitionSymbol>().defaultLifetime) {
            case VariableLifetime::Static:
                return SLANG_VARIABLE_LIFETIME_STATIC;
            case VariableLifetime::Automatic:
            default:
                return SLANG_VARIABLE_LIFETIME_AUTOMATIC;
        }
    });
}

slang_unconnected_drive slang_definition_unconnected_drive(slang_ast definition) {
    SLANG_C_ACCESS(SLANG_UNCONNECTED_DRIVE_NONE, {
        auto s = symbolOf(definition);
        if (!s || s->kind != SymbolKind::Definition)
            return SLANG_UNCONNECTED_DRIVE_NONE;
        switch (s->as<DefinitionSymbol>().unconnectedDrive) {
            case UnconnectedDrive::Pull0:
                return SLANG_UNCONNECTED_DRIVE_PULL0;
            case UnconnectedDrive::Pull1:
                return SLANG_UNCONNECTED_DRIVE_PULL1;
            case UnconnectedDrive::None:
            default:
                return SLANG_UNCONNECTED_DRIVE_NONE;
        }
    });
}

bool slang_definition_time_scale(slang_ast definition, slang_time_scale* out) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(definition);
        if (!s || s->kind != SymbolKind::Definition)
            return false;
        auto& ts = s->as<DefinitionSymbol>().timeScale;
        if (!ts)
            return false;
        if (out) {
            *out = slang_time_scale{
                (uint8_t)ts->base.unit,
                (uint8_t)ts->base.magnitude,
                (uint8_t)ts->precision.unit,
                (uint8_t)ts->precision.magnitude,
            };
        }
        return true;
    });
}

slang_str slang_definition_kind_string(slang_ast definition) {
    SLANG_C_ACCESS(borrowed(""), {
        auto s = symbolOf(definition);
        if (!s || s->kind != SymbolKind::Definition)
            return borrowed("");
        return borrowed(s->as<DefinitionSymbol>().getKindString());
    });
}

slang_str slang_definition_article_kind_string(slang_ast definition) {
    SLANG_C_ACCESS(borrowed(""), {
        auto s = symbolOf(definition);
        if (!s || s->kind != SymbolKind::Definition)
            return borrowed("");
        return borrowed(s->as<DefinitionSymbol>().getArticleKindString());
    });
}

uint64_t slang_definition_instance_count(slang_ast definition) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(definition);
        if (!s || s->kind != SymbolKind::Definition)
            return 0u;
        return (uint64_t)s->as<DefinitionSymbol>().getInstanceCount();
    });
}

slang_elab_system_task_kind slang_symbol_elab_system_task_kind(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_ELAB_SYSTEM_TASK_FATAL, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ElabSystemTask)
            return SLANG_ELAB_SYSTEM_TASK_FATAL;
        switch (s->as<ElabSystemTaskSymbol>().taskKind) {
            case ElabSystemTaskKind::Fatal:
                return SLANG_ELAB_SYSTEM_TASK_FATAL;
            case ElabSystemTaskKind::Error:
                return SLANG_ELAB_SYSTEM_TASK_ERROR;
            case ElabSystemTaskKind::Warning:
                return SLANG_ELAB_SYSTEM_TASK_WARNING;
            case ElabSystemTaskKind::Info:
                return SLANG_ELAB_SYSTEM_TASK_INFO;
            case ElabSystemTaskKind::StaticAssert:
                return SLANG_ELAB_SYSTEM_TASK_STATIC_ASSERT;
            default:
                return SLANG_ELAB_SYSTEM_TASK_FATAL;
        }
    });
}

slang_ast slang_symbol_elab_system_task_assert_condition(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ElabSystemTask)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        // Forced by the freeze sweep.
        auto cond = s->as<ElabSystemTaskSymbol>().getAssertCondition();
        return cond ? wrapAst(*cond, sym.compilation) : noAst(sym.compilation, SLANG_AST_EXPRESSION);
    });
}

bool slang_symbol_elab_system_task_message(slang_ast sym, slang_str* out) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ElabSystemTask)
            return false;
        // Forced by the freeze sweep; the message text lives in the
        // compilation's own arena, so the returned slang_str is a valid
        // borrow for the design's lifetime.
        auto msg = s->as<ElabSystemTaskSymbol>().getMessage();
        if (!msg)
            return false;
        if (out)
            *out = borrowed(*msg);
        return true;
    });
}

slang_str slang_symbol_explicit_import_name(slang_ast sym) {
    SLANG_C_ACCESS(borrowed(""), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ExplicitImport)
            return borrowed("");
        return borrowed(s->as<ExplicitImportSymbol>().importName);
    });
}

slang_str slang_symbol_explicit_import_package_name(slang_ast sym) {
    SLANG_C_ACCESS(borrowed(""), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ExplicitImport)
            return borrowed("");
        return borrowed(s->as<ExplicitImportSymbol>().packageName);
    });
}

slang_ast slang_symbol_explicit_import_package(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ExplicitImport)
            return noAst(sym.compilation);
        // Forced by Compilation::getAllDiagnostics(), which slang_compilation_
        // freeze runs unconditionally before its own sweep (see
        // ast::Elaborator's ExplicitImportSymbol handler) — a pure read.
        auto pkg = s->as<ExplicitImportSymbol>().package();
        return pkg ? capi::toC(pkg, sym.compilation) : noAst(sym.compilation);
    });
}

slang_ast slang_symbol_explicit_import_imported_symbol(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ExplicitImport)
            return noAst(sym.compilation);
        // Forced by Compilation::getAllDiagnostics() (see above) — a pure
        // read.
        auto imported = s->as<ExplicitImportSymbol>().importedSymbol();
        return imported ? capi::toC(imported, sym.compilation) : noAst(sym.compilation);
    });
}

uint32_t slang_symbol_variable_flags(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || !VariableSymbol::isKind(s->kind))
            return 0u;
        return (uint32_t)s->as<VariableSymbol>().flags.bits();
    });
}

slang_variable_lifetime slang_symbol_variable_lifetime(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_VARIABLE_LIFETIME_AUTOMATIC, {
        auto s = symbolOf(sym);
        if (!s || !VariableSymbol::isKind(s->kind))
            return SLANG_VARIABLE_LIFETIME_AUTOMATIC;
        return (slang_variable_lifetime)(uint32_t)s->as<VariableSymbol>().lifetime;
    });
}

slang_str slang_symbol_wildcard_import_package_name(slang_ast sym) {
    SLANG_C_ACCESS(borrowed(""), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::WildcardImport)
            return borrowed("");
        return borrowed(s->as<WildcardImportSymbol>().packageName);
    });
}

slang_ast slang_symbol_wildcard_import_package(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::WildcardImport)
            return noAst(sym.compilation);
        // Forced by Compilation::getAllDiagnostics(), which slang_compilation_
        // freeze runs unconditionally before its own sweep (see
        // ast::Elaborator's WildcardImportSymbol handler) — a pure read.
        auto pkg = s->as<WildcardImportSymbol>().getPackage();
        return pkg ? capi::toC(pkg, sym.compilation) : noAst(sym.compilation);
    });
}

slang_generate_branch_kind slang_symbol_generate_block_branch_kind(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_GENERATE_BRANCH_ILLEGAL_UNCONDITIONAL, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::GenerateBlock)
            return SLANG_GENERATE_BRANCH_ILLEGAL_UNCONDITIONAL;
        return (slang_generate_branch_kind)(uint32_t)s->as<GenerateBlockSymbol>().branchKind;
    });
}

slang_ast slang_symbol_generate_block_condition_expr(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::GenerateBlock)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        // Bound against the construct's enclosing scope, explicitly visited
        // (canonical-type-forced, prefolded) by the freeze sweep's
        // GenerateBlockSymbol branch alongside caseItemExpressions -- see
        // FreezeVisitor in this file -- so this is a pure read.
        auto expr = s->as<GenerateBlockSymbol>().getConditionExpression();
        return expr ? wrapAst(*expr, sym.compilation)
                    : noAst(sym.compilation, SLANG_AST_EXPRESSION);
    });
}

bool slang_symbol_generate_block_is_uninstantiated(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::GenerateBlock)
            return false;
        return s->as<GenerateBlockSymbol>().isUninstantiated;
    });
}

uint32_t slang_symbol_generate_block_case_item_expr_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::GenerateBlock)
            return 0u;
        return (uint32_t)s->as<GenerateBlockSymbol>().caseItemExpressions.size();
    });
}

slang_ast slang_symbol_generate_block_case_item_expr(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::GenerateBlock)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        // Explicitly visited (canonical-type-forced, prefolded) by the freeze
        // sweep's GenerateBlockSymbol branch, so this is a pure read.
        auto exprs = s->as<GenerateBlockSymbol>().caseItemExpressions;
        if (index >= exprs.size())
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(*exprs[index], sym.compilation);
    });
}

uint32_t slang_symbol_generate_block_construct_index(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::GenerateBlock)
            return 0u;
        return s->as<GenerateBlockSymbol>().constructIndex;
    });
}

uint32_t slang_symbol_generate_block_array_entry_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::GenerateBlockArray)
            return 0u;
        return (uint32_t)s->as<GenerateBlockArraySymbol>().entries.size();
    });
}

slang_ast slang_symbol_generate_block_array_entry(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::GenerateBlockArray)
            return noAst(sym.compilation);
        auto entries = s->as<GenerateBlockArraySymbol>().entries;
        if (index >= entries.size())
            return noAst(sym.compilation);
        return toC(entries[index], sym.compilation);
    });
}

uint32_t slang_symbol_generate_block_array_construct_index(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::GenerateBlockArray)
            return 0u;
        return s->as<GenerateBlockArraySymbol>().constructIndex;
    });
}

bool slang_symbol_generate_block_array_valid(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::GenerateBlockArray)
            return false;
        return s->as<GenerateBlockArraySymbol>().valid;
    });
}

slang_ast slang_symbol_generate_block_array_initial_expr(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::GenerateBlockArray)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        // Explicitly visited by the freeze sweep's GenerateBlockArraySymbol
        // branch, so this is a pure read.
        auto expr = s->as<GenerateBlockArraySymbol>().initialExpression;
        return expr ? wrapAst(*expr, sym.compilation) : noAst(sym.compilation, SLANG_AST_EXPRESSION);
    });
}

slang_ast slang_symbol_generate_block_array_stop_expr(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::GenerateBlockArray)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        // Explicitly visited by the freeze sweep's GenerateBlockArraySymbol
        // branch, so this is a pure read.
        auto expr = s->as<GenerateBlockArraySymbol>().stopExpression;
        return expr ? wrapAst(*expr, sym.compilation) : noAst(sym.compilation, SLANG_AST_EXPRESSION);
    });
}

slang_ast slang_symbol_generate_block_array_iter_expr(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::GenerateBlockArray)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        // Explicitly visited by the freeze sweep's GenerateBlockArraySymbol
        // branch, so this is a pure read.
        auto expr = s->as<GenerateBlockArraySymbol>().iterExpression;
        return expr ? wrapAst(*expr, sym.compilation) : noAst(sym.compilation, SLANG_AST_EXPRESSION);
    });
}

slang_ast slang_symbol_generate_block_array_loop_variable(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::GenerateBlockArray)
            return noAst(sym.compilation);
        // Explicitly visited by the freeze sweep's GenerateBlockArraySymbol
        // branch (it lives in a private scope the generic traversal never
        // reaches), so this is a pure read.
        auto var = s->as<GenerateBlockArraySymbol>().loopVariable;
        return var ? toC(var, sym.compilation) : noAst(sym.compilation);
    });
}

slang_str slang_symbol_generate_block_external_name(slang_ast sym, slang_error* err) {
    if (!checkEntry(err))
        return borrowed("");
    auto s = symbolOf(sym);
    if (!s || (s->kind != SymbolKind::GenerateBlock && s->kind != SymbolKind::GenerateBlockArray)) {
        setError(err, SLANG_ERR_INVALID_ARG, "not a GenerateBlock or GenerateBlockArray symbol");
        return borrowed("");
    }
    SLANG_C_GUARD(err, {
        if (s->kind == SymbolKind::GenerateBlock)
            return owned(s->as<GenerateBlockSymbol>().getExternalName());
        return owned(s->as<GenerateBlockArraySymbol>().getExternalName());
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

slang_ast slang_expr_symbol_reference(slang_ast expr, bool allow_packed) {
    SLANG_C_ACCESS(noAst(expr.compilation), {
        auto e = exprOf(expr);
        return capi::toC(e ? e->getSymbolReference(allow_packed) : nullptr, expr.compilation);
    });
}

bool slang_expr_has_hierarchical_reference(slang_ast expr) {
    SLANG_C_ACCESS(false, {
        auto e = exprOf(expr);
        return e && e->hasHierarchicalReference();
    });
}

bool slang_expr_is_equivalent_to(slang_ast expr, slang_ast other) {
    SLANG_C_ACCESS(false, {
        auto e = exprOf(expr);
        auto o = exprOf(other);
        return e && o && e->isEquivalentTo(*o);
    });
}

bool slang_expr_is_implicit_string(slang_ast expr) {
    SLANG_C_ACCESS(false, {
        auto e = exprOf(expr);
        return e && e->isImplicitString();
    });
}

bool slang_expr_is_implicitly_assignable_to(slang_ast expr, slang_ast type) {
    SLANG_C_ACCESS(false, {
        auto e = exprOf(expr);
        auto t = typeOf(type);
        if (!e || !t || !expr.compilation)
            return false;
        return e->isImplicitlyAssignableTo(*expr.compilation->comp, *t);
    });
}

bool slang_expr_is_unsized_integer(slang_ast expr) {
    SLANG_C_ACCESS(false, {
        auto e = exprOf(expr);
        return e && e->isUnsizedInteger();
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
        if (!s)
            return noAst(node.compilation, SLANG_AST_TIMING_CONTROL);
        switch (s->kind) {
            case StatementKind::Timed:
                return wrapAst(s->as<TimedStatement>().timing, node.compilation);
            case StatementKind::EventTrigger: {
                auto timing = s->as<EventTriggerStatement>().timing;
                return timing ? wrapAst(*timing, node.compilation)
                              : noAst(node.compilation, SLANG_AST_TIMING_CONTROL);
            }
            default:
                return noAst(node.compilation, SLANG_AST_TIMING_CONTROL);
        }
    });
}

slang_edge_kind slang_timing_control_edge(slang_ast node) {
    // The `edge` field is set at SignalEventControl construction (TimingControl.h)
    // — a pure read, no lazy memo. Returns SLANG_EDGE_NONE for any other kind.
    SLANG_C_ACCESS(SLANG_EDGE_NONE, {
        if (node.domain != SLANG_AST_TIMING_CONTROL || !node.ptr)
            return SLANG_EDGE_NONE;
        auto* tc = static_cast<const TimingControl*>(node.ptr);
        if (tc->kind != TimingControlKind::SignalEvent)
            return SLANG_EDGE_NONE;
        return (slang_edge_kind)tc->as<SignalEventControl>().edge;
    });
}

slang_ast slang_pattern_expr(slang_ast node) {
    // ConstantPattern::expr (Patterns.h) — set at construction, a pure read.
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        if (node.domain != SLANG_AST_PATTERN || !node.ptr)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto* p = static_cast<const Pattern*>(node.ptr);
        if (p->kind == PatternKind::Constant)
            return wrapAst(p->as<ConstantPattern>().expr, node.compilation);
        return noAst(node.compilation, SLANG_AST_EXPRESSION);
    });
}

slang_ast slang_pattern_variable(slang_ast node) {
    // VariablePattern::variable (Patterns.h) — the bound pattern variable symbol.
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_SYMBOL), {
        if (node.domain != SLANG_AST_PATTERN || !node.ptr)
            return noAst(node.compilation, SLANG_AST_SYMBOL);
        auto* p = static_cast<const Pattern*>(node.ptr);
        if (p->kind == PatternKind::Variable)
            return wrapAst(p->as<VariablePattern>().variable, node.compilation);
        return noAst(node.compilation, SLANG_AST_SYMBOL);
    });
}

int32_t slang_assertion_expr_op(slang_ast node) {
    // The Unary/Binary assertion operator (AssertionExpr.h), as its raw enum
    // value; -1 for any other kind. Set at construction, a pure read.
    SLANG_C_ACCESS(-1, {
        if (node.domain != SLANG_AST_ASSERTION_EXPR || !node.ptr)
            return -1;
        auto* a = static_cast<const AssertionExpr*>(node.ptr);
        if (a->kind == AssertionExprKind::Unary)
            return (int32_t)a->as<UnaryAssertionExpr>().op;
        if (a->kind == AssertionExprKind::Binary)
            return (int32_t)a->as<BinaryAssertionExpr>().op;
        return -1;
    });
}

slang_ast slang_constraint_expr(slang_ast node) {
    // ExpressionConstraint::expr (Constraints.h) — the constrained expression.
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        if (node.domain != SLANG_AST_CONSTRAINT || !node.ptr)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto* c = static_cast<const Constraint*>(node.ptr);
        if (c->kind == ConstraintKind::Expression)
            return wrapAst(c->as<ExpressionConstraint>().expr, node.compilation);
        return noAst(node.compilation, SLANG_AST_EXPRESSION);
    });
}

slang_ast slang_constraint_predicate(slang_ast node) {
    // The controlling predicate of an Implication/Conditional constraint
    // (Constraints.h) — an EXPRESSION node, else a null ast.
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        if (node.domain != SLANG_AST_CONSTRAINT || !node.ptr)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto* c = static_cast<const Constraint*>(node.ptr);
        if (c->kind == ConstraintKind::Implication)
            return wrapAst(c->as<ImplicationConstraint>().predicate, node.compilation);
        if (c->kind == ConstraintKind::Conditional)
            return wrapAst(c->as<ConditionalConstraint>().predicate, node.compilation);
        return noAst(node.compilation, SLANG_AST_EXPRESSION);
    });
}

// ---- Block / conditional / case / assertion / event-trigger breadth --------
//
// Every field read below is set directly at Statement construction (see
// Statement.h / ConditionalStatements.h / MiscStatements.h) — none is a lazy
// `mutable` memo, so each accessor is a pure read on a frozen compilation with
// no matching FreezeVisitor force required.

uint32_t slang_stmt_block_kind(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::Block)
            return 0u;
        return (uint32_t)s->as<BlockStatement>().blockKind;
    });
}

slang_ast slang_stmt_block_symbol(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::Block)
            return noAst(node.compilation);
        auto sym = s->as<BlockStatement>().blockSymbol;
        return sym ? toC(sym, node.compilation) : noAst(node.compilation);
    });
}

uint32_t slang_stmt_conditional_check(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::Conditional)
            return 0u;
        return (uint32_t)s->as<ConditionalStatement>().check;
    });
}

uint32_t slang_stmt_conditional_condition_count(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::Conditional)
            return 0u;
        return (uint32_t)s->as<ConditionalStatement>().conditions.size();
    });
}

slang_ast slang_stmt_conditional_condition_expr(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::Conditional)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto& conditions = s->as<ConditionalStatement>().conditions;
        if (index >= conditions.size())
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(*conditions[index].expr, node.compilation);
    });
}

slang_ast slang_stmt_conditional_condition_pattern(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_PATTERN), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::Conditional)
            return noAst(node.compilation, SLANG_AST_PATTERN);
        auto& conditions = s->as<ConditionalStatement>().conditions;
        if (index >= conditions.size())
            return noAst(node.compilation, SLANG_AST_PATTERN);
        auto pattern = conditions[index].pattern;
        return pattern ? wrapAst(*pattern, node.compilation)
                       : noAst(node.compilation, SLANG_AST_PATTERN);
    });
}

uint32_t slang_stmt_case_condition(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::Case)
            return 0u;
        return (uint32_t)s->as<CaseStatement>().condition;
    });
}

uint32_t slang_stmt_case_check(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::Case)
            return 0u;
        return (uint32_t)s->as<CaseStatement>().check;
    });
}

slang_ast slang_stmt_case_default(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_STATEMENT), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::Case)
            return noAst(node.compilation, SLANG_AST_STATEMENT);
        auto def = s->as<CaseStatement>().defaultCase;
        return def ? wrapAst(*def, node.compilation)
                   : noAst(node.compilation, SLANG_AST_STATEMENT);
    });
}

uint32_t slang_stmt_case_item_count(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::Case)
            return 0u;
        return (uint32_t)s->as<CaseStatement>().items.size();
    });
}

slang_ast slang_stmt_case_item_stmt(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_STATEMENT), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::Case)
            return noAst(node.compilation, SLANG_AST_STATEMENT);
        auto& items = s->as<CaseStatement>().items;
        if (index >= items.size())
            return noAst(node.compilation, SLANG_AST_STATEMENT);
        return wrapAst(*items[index].stmt, node.compilation);
    });
}

uint32_t slang_stmt_case_item_expr_count(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(0u, {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::Case)
            return 0u;
        auto& items = s->as<CaseStatement>().items;
        if (index >= items.size())
            return 0u;
        return (uint32_t)items[index].expressions.size();
    });
}

slang_ast slang_stmt_case_item_expr(slang_ast node, uint32_t index, uint32_t j) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::Case)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto& items = s->as<CaseStatement>().items;
        if (index >= items.size())
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto& exprs = items[index].expressions;
        if (j >= exprs.size())
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(*exprs[j], node.compilation);
    });
}

uint32_t slang_stmt_assertion_kind(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::ConcurrentAssertion)
            return 0u;
        return (uint32_t)s->as<ConcurrentAssertionStatement>().assertionKind;
    });
}

slang_ast slang_stmt_assertion_if_true(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_STATEMENT), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::ConcurrentAssertion)
            return noAst(node.compilation, SLANG_AST_STATEMENT);
        auto ifTrue = s->as<ConcurrentAssertionStatement>().ifTrue;
        return ifTrue ? wrapAst(*ifTrue, node.compilation)
                      : noAst(node.compilation, SLANG_AST_STATEMENT);
    });
}

slang_ast slang_stmt_assertion_if_false(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_STATEMENT), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::ConcurrentAssertion)
            return noAst(node.compilation, SLANG_AST_STATEMENT);
        auto ifFalse = s->as<ConcurrentAssertionStatement>().ifFalse;
        return ifFalse ? wrapAst(*ifFalse, node.compilation)
                       : noAst(node.compilation, SLANG_AST_STATEMENT);
    });
}

bool slang_stmt_event_trigger_is_nonblocking(slang_ast node) {
    SLANG_C_ACCESS(false, {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::EventTrigger)
            return false;
        return s->as<EventTriggerStatement>().isNonBlocking;
    });
}

// ---- ForLoop / Foreach / ImmediateAssertion / PatternCase breadth ----------
//
// Every field read below is set directly at Statement construction (see
// LoopStatements.h / MiscStatements.h / ConditionalStatements.h) — none is a
// lazy `mutable` memo, so each accessor is a pure read on a frozen
// compilation with no matching FreezeVisitor force required.

uint32_t slang_stmt_for_loop_initializer_count(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::ForLoop)
            return 0u;
        return (uint32_t)s->as<ForLoopStatement>().initializers.size();
    });
}

slang_ast slang_stmt_for_loop_initializer(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::ForLoop)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto& initializers = s->as<ForLoopStatement>().initializers;
        if (index >= initializers.size())
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(*initializers[index], node.compilation);
    });
}

uint32_t slang_stmt_for_loop_var_count(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::ForLoop)
            return 0u;
        return (uint32_t)s->as<ForLoopStatement>().loopVars.size();
    });
}

slang_ast slang_stmt_for_loop_var(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::ForLoop)
            return noAst(node.compilation);
        auto& loopVars = s->as<ForLoopStatement>().loopVars;
        if (index >= loopVars.size())
            return noAst(node.compilation);
        return wrapAst(*loopVars[index], node.compilation);
    });
}

uint32_t slang_stmt_for_loop_step_count(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::ForLoop)
            return 0u;
        return (uint32_t)s->as<ForLoopStatement>().steps.size();
    });
}

slang_ast slang_stmt_for_loop_step(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::ForLoop)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto& steps = s->as<ForLoopStatement>().steps;
        if (index >= steps.size())
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(*steps[index], node.compilation);
    });
}

uint32_t slang_stmt_foreach_loop_dim_count(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::ForeachLoop)
            return 0u;
        return (uint32_t)s->as<ForeachLoopStatement>().loopDims.size();
    });
}

slang_ast slang_stmt_foreach_loop_dim_var(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::ForeachLoop)
            return noAst(node.compilation);
        auto& dims = s->as<ForeachLoopStatement>().loopDims;
        if (index >= dims.size() || !dims[index].loopVar)
            return noAst(node.compilation);
        return wrapAst(*dims[index].loopVar, node.compilation);
    });
}

slang_constant_range slang_type_fixed_unpacked_array_range(slang_ast type) {
    SLANG_C_ACCESS(slang_constant_range{}, {
        auto t = typeOf(type);
        if (!t)
            return slang_constant_range{};
        auto& ct = t->getCanonicalType();
        if (ct.kind != SymbolKind::FixedSizeUnpackedArrayType)
            return slang_constant_range{};
        auto& r = ct.as<FixedSizeUnpackedArrayType>().range;
        return slang_constant_range{r.left, r.right};
    });
}

uint32_t slang_declared_type_resolved_dimensions(slang_ast sym_node, slang_evaluated_dimension* out,
                                                  uint32_t cap, slang_error* err) {
    if (!checkEntry(err))
        return 0u;
    auto sym = symbolOf(sym_node);
    auto dt = sym ? sym->getDeclaredType() : nullptr;
    if (!dt)
        return 0u;
    SLANG_C_GUARD(err, {
        // getResolvedDimensions() re-binds each dimension's syntax on every
        // call (never cached), allocating fresh Expression nodes into the
        // arena; lift the seal for the call, as slang_expression_eval does.
        SealLift lift(sym_node.compilation);
        auto dims = dt->getResolvedDimensions();
        uint32_t total = (uint32_t)dims.size();
        uint32_t n = total < cap ? total : cap;
        for (uint32_t i = 0; i < n; i++) {
            out[i].kind = (slang_dimension_kind)dims[i].kind;
            out[i].bounds = slang_constant_range{dims[i].range.left, dims[i].range.right};
        }
        return total;
    });
    return 0u;
}

bool slang_stmt_foreach_loop_dim_range(slang_ast node, uint32_t index, slang_constant_range* out) {
    SLANG_C_ACCESS(false, {
        if (!out)
            return false;
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::ForeachLoop)
            return false;
        auto& dims = s->as<ForeachLoopStatement>().loopDims;
        if (index >= dims.size() || !dims[index].range.has_value())
            return false;
        auto& range = *dims[index].range;
        *out = slang_constant_range{range.left, range.right};
        return true;
    });
}

uint32_t slang_stmt_immediate_assertion_kind(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::ImmediateAssertion)
            return 0u;
        return (uint32_t)s->as<ImmediateAssertionStatement>().assertionKind;
    });
}

slang_ast slang_stmt_immediate_assertion_if_true(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_STATEMENT), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::ImmediateAssertion)
            return noAst(node.compilation, SLANG_AST_STATEMENT);
        auto ifTrue = s->as<ImmediateAssertionStatement>().ifTrue;
        return ifTrue ? wrapAst(*ifTrue, node.compilation)
                      : noAst(node.compilation, SLANG_AST_STATEMENT);
    });
}

slang_ast slang_stmt_immediate_assertion_if_false(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_STATEMENT), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::ImmediateAssertion)
            return noAst(node.compilation, SLANG_AST_STATEMENT);
        auto ifFalse = s->as<ImmediateAssertionStatement>().ifFalse;
        return ifFalse ? wrapAst(*ifFalse, node.compilation)
                       : noAst(node.compilation, SLANG_AST_STATEMENT);
    });
}

bool slang_stmt_immediate_assertion_is_deferred(slang_ast node) {
    SLANG_C_ACCESS(false, {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::ImmediateAssertion)
            return false;
        return s->as<ImmediateAssertionStatement>().isDeferred;
    });
}

bool slang_stmt_immediate_assertion_is_final(slang_ast node) {
    SLANG_C_ACCESS(false, {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::ImmediateAssertion)
            return false;
        return s->as<ImmediateAssertionStatement>().isFinal;
    });
}

uint32_t slang_stmt_pattern_case_check(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::PatternCase)
            return 0u;
        return (uint32_t)s->as<PatternCaseStatement>().check;
    });
}

uint32_t slang_stmt_pattern_case_item_count(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::PatternCase)
            return 0u;
        return (uint32_t)s->as<PatternCaseStatement>().items.size();
    });
}

slang_ast slang_stmt_pattern_case_item_pattern(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_PATTERN), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::PatternCase)
            return noAst(node.compilation, SLANG_AST_PATTERN);
        auto& items = s->as<PatternCaseStatement>().items;
        if (index >= items.size())
            return noAst(node.compilation, SLANG_AST_PATTERN);
        return wrapAst(*items[index].pattern, node.compilation);
    });
}

slang_ast slang_stmt_pattern_case_item_filter(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::PatternCase)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto& items = s->as<PatternCaseStatement>().items;
        if (index >= items.size())
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto filter = items[index].filter;
        return filter ? wrapAst(*filter, node.compilation)
                      : noAst(node.compilation, SLANG_AST_EXPRESSION);
    });
}

slang_ast slang_stmt_pattern_case_item_stmt(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_STATEMENT), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::PatternCase)
            return noAst(node.compilation, SLANG_AST_STATEMENT);
        auto& items = s->as<PatternCaseStatement>().items;
        if (index >= items.size())
            return noAst(node.compilation, SLANG_AST_STATEMENT);
        return wrapAst(*items[index].stmt, node.compilation);
    });
}

uint32_t slang_stmt_pattern_case_condition(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::PatternCase)
            return 0u;
        return (uint32_t)s->as<PatternCaseStatement>().condition;
    });
}

slang_ast slang_stmt_pattern_case_default(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_STATEMENT), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::PatternCase)
            return noAst(node.compilation, SLANG_AST_STATEMENT);
        auto def = s->as<PatternCaseStatement>().defaultCase;
        return def ? wrapAst(*def, node.compilation)
                   : noAst(node.compilation, SLANG_AST_STATEMENT);
    });
}

uint32_t slang_stmt_wait_order_event_count(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::WaitOrder)
            return 0u;
        return (uint32_t)s->as<WaitOrderStatement>().events.size();
    });
}

slang_ast slang_stmt_wait_order_event(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::WaitOrder)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto& events = s->as<WaitOrderStatement>().events;
        if (index >= events.size())
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(*events[index], node.compilation);
    });
}

slang_ast slang_stmt_wait_order_if_true(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_STATEMENT), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::WaitOrder)
            return noAst(node.compilation, SLANG_AST_STATEMENT);
        auto ifTrue = s->as<WaitOrderStatement>().ifTrue;
        return ifTrue ? wrapAst(*ifTrue, node.compilation)
                      : noAst(node.compilation, SLANG_AST_STATEMENT);
    });
}

slang_ast slang_stmt_wait_order_if_false(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_STATEMENT), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::WaitOrder)
            return noAst(node.compilation, SLANG_AST_STATEMENT);
        auto ifFalse = s->as<WaitOrderStatement>().ifFalse;
        return ifFalse ? wrapAst(*ifFalse, node.compilation)
                       : noAst(node.compilation, SLANG_AST_STATEMENT);
    });
}

bool slang_stmt_procedural_assign_is_force(slang_ast node) {
    SLANG_C_ACCESS(false, {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::ProceduralAssign)
            return false;
        return s->as<ProceduralAssignStatement>().isForce;
    });
}

bool slang_stmt_procedural_deassign_is_release(slang_ast node) {
    SLANG_C_ACCESS(false, {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::ProceduralDeassign)
            return false;
        return s->as<ProceduralDeassignStatement>().isRelease;
    });
}

uint32_t slang_stmt_randcase_item_count(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::RandCase)
            return 0u;
        return (uint32_t)s->as<RandCaseStatement>().items.size();
    });
}

slang_ast slang_stmt_randcase_item_expr(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::RandCase)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto& items = s->as<RandCaseStatement>().items;
        if (index >= items.size())
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(*items[index].expr, node.compilation);
    });
}

slang_ast slang_stmt_randcase_item_stmt(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_STATEMENT), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::RandCase)
            return noAst(node.compilation, SLANG_AST_STATEMENT);
        auto& items = s->as<RandCaseStatement>().items;
        if (index >= items.size())
            return noAst(node.compilation, SLANG_AST_STATEMENT);
        return wrapAst(*items[index].stmt, node.compilation);
    });
}

slang_ast slang_stmt_randsequence_first_production(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_SYMBOL), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::RandSequence)
            return noAst(node.compilation, SLANG_AST_SYMBOL);
        auto first = s->as<RandSequenceStatement>().firstProduction;
        return first ? wrapAst(*first, node.compilation)
                     : noAst(node.compilation, SLANG_AST_SYMBOL);
    });
}

uint32_t slang_stmt_procedural_checker_instance_count(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::ProceduralChecker)
            return 0u;
        return (uint32_t)s->as<ProceduralCheckerStatement>().instances.size();
    });
}

slang_ast slang_stmt_procedural_checker_instance(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_SYMBOL), {
        auto s = stmtOf(node);
        if (!s || s->kind != StatementKind::ProceduralChecker)
            return noAst(node.compilation, SLANG_AST_SYMBOL);
        auto& instances = s->as<ProceduralCheckerStatement>().instances;
        if (index >= instances.size())
            return noAst(node.compilation, SLANG_AST_SYMBOL);
        return wrapAst(*instances[index], node.compilation);
    });
}

bool slang_stmt_is_bad(slang_ast node) {
    SLANG_C_ACCESS(true, {
        auto s = stmtOf(node);
        return !s || s->bad();
    });
}

uint32_t slang_stmt_eval(slang_ast node, slang_error* err) {
    if (!checkEntry(err))
        return (uint32_t)Statement::EvalResult::Fail;
    auto s = stmtOf(node);
    auto comp = node.compilation;
    if (!s || !comp) {
        setError(err, SLANG_ERR_INVALID_ARG, "not a statement");
        return (uint32_t)Statement::EvalResult::Fail;
    }
    SLANG_C_GUARD(err, {
        // Evaluation may cache/allocate into the compilation's arena (locals
        // created for the scratch frame, constant folding of sub-expressions);
        // lift the seal for the duration of the call, as slang_expression_eval
        // does (the caller has promised exclusive access).
        SealLift lift(comp);
        ASTContext astCtx(comp->comp->getRoot(), LookupLocation::max);
        EvalContext evalCtx(astCtx, EvalFlags::IsScript);
        evalCtx.pushEmptyFrame();
        return (uint32_t)s->eval(evalCtx);
    });
    return (uint32_t)Statement::EvalResult::Fail;
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

SLANG_EXPR_CHILD(slang_expr_inside_left, Inside, InsideExpression, left())
SLANG_EXPR_CHILD(slang_expr_new_array_size, NewArray, NewArrayExpression, sizeExpr())

#undef SLANG_EXPR_CHILD
#undef SLANG_EXPR_ENUM

bool slang_expr_integer_literal_is_declared_unsized(slang_ast node) {
    SLANG_C_ACCESS(false, {
        auto e = exprOf(node);
        return e && e->kind == ExpressionKind::IntegerLiteral &&
               e->as<IntegerLiteral>().isDeclaredUnsized;
    });
}

double slang_expr_real_literal_value(slang_ast node) {
    SLANG_C_ACCESS(0.0, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::RealLiteral)
            return 0.0;
        return e->as<RealLiteral>().getValue();
    });
}

double slang_expr_time_literal_value(slang_ast node) {
    SLANG_C_ACCESS(0.0, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::TimeLiteral)
            return 0.0;
        return e->as<TimeLiteral>().getValue();
    });
}

bool slang_expr_time_literal_scale(slang_ast node, slang_time_scale* out) {
    if (!out)
        return false;
    SLANG_C_ACCESS(false, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::TimeLiteral)
            return false;
        auto ts = e->as<TimeLiteral>().getScale();
        *out = slang_time_scale{
            (uint8_t)ts.base.unit,
            (uint8_t)ts.base.magnitude,
            (uint8_t)ts.precision.unit,
            (uint8_t)ts.precision.magnitude,
        };
        return true;
    });
}

uint8_t slang_expr_unbased_unsized_literal_bit(slang_ast node) {
    SLANG_C_ACCESS((uint8_t)0, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::UnbasedUnsizedIntegerLiteral)
            return (uint8_t)0;
        logic_t bit = e->as<UnbasedUnsizedIntegerLiteral>().getLiteralValue();
        if (!bit.isUnknown())
            return (uint8_t)bit.value; // 0 or 1
        return bit.value == logic_t::x.value ? (uint8_t)2 : (uint8_t)3;
    });
}

slang_constant slang_expr_unbased_unsized_literal_value(slang_ast expr, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    auto e = exprOf(expr);
    if (!e || e->kind != ExpressionKind::UnbasedUnsizedIntegerLiteral) {
        setError(err, SLANG_ERR_INVALID_ARG, "not an unbased-unsized integer literal");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        return new slang_constant_t{
            ConstantValue(e->as<UnbasedUnsizedIntegerLiteral>().getValue())};
    });
    return nullptr;
}

uint32_t slang_expr_inside_range_count(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::Inside)
            return 0u;
        return (uint32_t)e->as<InsideExpression>().rangeList().size();
    });
}

slang_ast slang_expr_inside_range(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::Inside)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto ranges = e->as<InsideExpression>().rangeList();
        if (index >= ranges.size())
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(*ranges[index], node.compilation);
    });
}

slang_ast slang_expr_new_array_init(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::NewArray)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto init = e->as<NewArrayExpression>().initExpr();
        return init ? wrapAst(*init, node.compilation)
                    : noAst(node.compilation, SLANG_AST_EXPRESSION);
    });
}

slang_ast slang_expr_new_class_constructor_call(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::NewClass)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto call = e->as<NewClassExpression>().constructorCall();
        return call ? wrapAst(*call, node.compilation)
                    : noAst(node.compilation, SLANG_AST_EXPRESSION);
    });
}

bool slang_expr_new_class_is_super_class(slang_ast node) {
    SLANG_C_ACCESS(false, {
        auto e = exprOf(node);
        return e && e->kind == ExpressionKind::NewClass &&
               e->as<NewClassExpression>().isSuperClass;
    });
}

uint32_t slang_expr_new_covergroup_argument_count(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::NewCovergroup)
            return 0u;
        return (uint32_t)e->as<NewCovergroupExpression>().arguments.size();
    });
}

slang_ast slang_expr_new_covergroup_argument(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::NewCovergroup)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto& args = e->as<NewCovergroupExpression>().arguments;
        if (index >= args.size())
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(*args[index], node.compilation);
    });
}

slang_ast slang_expr_tagged_union_value(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::TaggedUnion)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto val = e->as<TaggedUnionExpression>().valueExpr;
        return val ? wrapAst(*val, node.compilation) : noAst(node.compilation, SLANG_AST_EXPRESSION);
    });
}

// ---- Expression breadth: Assignment / AssignmentPattern / ArbitrarySymbol /
//      AssertionInstance / Call system-call extras --------------------------
//
// Every field read below (op, timingControl, symbol, body, arguments,
// localVars, isRecursiveProperty, elements, subroutine/extraInfo/scope) is
// set exactly once, directly, at expression construction (see
// AssignmentExpressions.h / MiscExpressions.h / CallExpression.h and their
// .cpp binders) — none is a lazy `mutable` memo, so each accessor here is a
// pure read on a frozen compilation with no matching FreezeVisitor force
// required.

slang_ast slang_expr_arbitrary_symbol(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::ArbitrarySymbol)
            return noAst(node.compilation);
        return toC(e->as<ArbitrarySymbolExpression>().symbol.get(), node.compilation);
    });
}

bool slang_expr_assignment_is_compound(slang_ast node) {
    SLANG_C_ACCESS(false, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::Assignment)
            return false;
        return e->as<AssignmentExpression>().isCompound();
    });
}

bool slang_expr_assignment_is_lvalue_arg(slang_ast node) {
    SLANG_C_ACCESS(false, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::Assignment)
            return false;
        return e->as<AssignmentExpression>().isLValueArg();
    });
}

uint32_t slang_expr_assignment_op(slang_ast node) {
    SLANG_C_ACCESS(0xFFFFFFFFu, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::Assignment)
            return 0xFFFFFFFFu;
        auto& op = e->as<AssignmentExpression>().op;
        return op.has_value() ? (uint32_t)*op : 0xFFFFFFFFu;
    });
}

slang_ast slang_expr_assignment_timing(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_TIMING_CONTROL), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::Assignment)
            return noAst(node.compilation, SLANG_AST_TIMING_CONTROL);
        auto timing = e->as<AssignmentExpression>().timingControl;
        return timing ? wrapAst(*timing, node.compilation)
                      : noAst(node.compilation, SLANG_AST_TIMING_CONTROL);
    });
}

namespace {

// Recovers `node` as an AssignmentPatternExpressionBase if its kind is any of
// the three concrete assignment-pattern expressions, else nullptr. The base
// class has no `isKind` of its own (it's abstract), so this replaces the
// usual `e->as<T>()` for the three sibling kinds it covers.
const AssignmentPatternExpressionBase* assignmentPatternOf(const Expression* e) {
    if (!e)
        return nullptr;
    switch (e->kind) {
        case ExpressionKind::SimpleAssignmentPattern:
        case ExpressionKind::StructuredAssignmentPattern:
        case ExpressionKind::ReplicatedAssignmentPattern:
            return static_cast<const AssignmentPatternExpressionBase*>(e);
        default:
            return nullptr;
    }
}

} // namespace

uint32_t slang_expr_pattern_element_count(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto pat = assignmentPatternOf(exprOf(node));
        return pat ? (uint32_t)pat->elements().size() : 0u;
    });
}

slang_ast slang_expr_pattern_element(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto pat = assignmentPatternOf(exprOf(node));
        if (!pat)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto elems = pat->elements();
        if (index >= elems.size())
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(*elems[index], node.compilation);
    });
}

bool slang_expr_assertion_instance_is_recursive(slang_ast node) {
    SLANG_C_ACCESS(false, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::AssertionInstance)
            return false;
        return e->as<AssertionInstanceExpression>().isRecursiveProperty;
    });
}

uint32_t slang_expr_assertion_instance_local_var_count(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::AssertionInstance)
            return 0u;
        return (uint32_t)e->as<AssertionInstanceExpression>().localVars.size();
    });
}

slang_ast slang_expr_assertion_instance_local_var(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::AssertionInstance)
            return noAst(node.compilation);
        auto& vars = e->as<AssertionInstanceExpression>().localVars;
        if (index >= vars.size() || !vars[index])
            return noAst(node.compilation);
        return toC(vars[index], node.compilation);
    });
}

uint32_t slang_expr_assertion_instance_argument_count(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::AssertionInstance)
            return 0u;
        return (uint32_t)e->as<AssertionInstanceExpression>().arguments.size();
    });
}

slang_ast slang_expr_assertion_instance_argument_port(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::AssertionInstance)
            return noAst(node.compilation);
        auto& args = e->as<AssertionInstanceExpression>().arguments;
        if (index >= args.size())
            return noAst(node.compilation);
        auto port = std::get<0>(args[index]);
        return port ? toC(port, node.compilation) : noAst(node.compilation);
    });
}

slang_ast slang_expr_assertion_instance_argument_actual(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::AssertionInstance)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto& args = e->as<AssertionInstanceExpression>().arguments;
        if (index >= args.size())
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto& actual = std::get<1>(args[index]);
        auto comp = node.compilation;
        return std::visit(
            [comp](auto* p) -> slang_ast {
                if (!p)
                    return noAst(comp, SLANG_AST_EXPRESSION);
                return wrapAst(*p, comp);
            },
            actual);
    });
}

slang_ast slang_expr_call_iterator_expr(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto call = systemCallOf(node);
        if (!call)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto& info = std::get<CallExpression::SystemCallInfo>(call->subroutine);
        auto iterInfo = std::get_if<CallExpression::IteratorCallInfo>(&info.extraInfo);
        if (!iterInfo || !iterInfo->iterExpr)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(*iterInfo->iterExpr, node.compilation);
    });
}

slang_ast slang_expr_call_iterator_var(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation), {
        auto call = systemCallOf(node);
        if (!call)
            return noAst(node.compilation);
        auto& info = std::get<CallExpression::SystemCallInfo>(call->subroutine);
        auto iterInfo = std::get_if<CallExpression::IteratorCallInfo>(&info.extraInfo);
        if (!iterInfo || !iterInfo->iterVar)
            return noAst(node.compilation);
        return toC(iterInfo->iterVar, node.compilation);
    });
}

slang_ast slang_expr_call_randomize_inline_constraints(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_CONSTRAINT), {
        auto call = systemCallOf(node);
        if (!call)
            return noAst(node.compilation, SLANG_AST_CONSTRAINT);
        auto& info = std::get<CallExpression::SystemCallInfo>(call->subroutine);
        auto randInfo = std::get_if<CallExpression::RandomizeCallInfo>(&info.extraInfo);
        if (!randInfo || !randInfo->inlineConstraints)
            return noAst(node.compilation, SLANG_AST_CONSTRAINT);
        return wrapAst(*randInfo->inlineConstraints, node.compilation);
    });
}

uint32_t slang_expr_call_extra_info_kind(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto call = systemCallOf(node);
        if (!call)
            return 0u;
        auto& info = std::get<CallExpression::SystemCallInfo>(call->subroutine);
        return (uint32_t)info.extraInfo.index();
    });
}

slang_ast slang_expr_call_system_scope(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation), {
        auto call = systemCallOf(node);
        if (!call)
            return noAst(node.compilation);
        auto& info = std::get<CallExpression::SystemCallInfo>(call->subroutine);
        return toC(&info.scope->asSymbol(), node.compilation);
    });
}

uint32_t slang_expr_call_subroutine_kind(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::Call)
            return 0u;
        return (uint32_t)e->as<CallExpression>().getSubroutineKind();
    });
}

// ---- Call / Conditional / Conversion / CopyClass / Dist expression breadth,
// and Expression-base lvalue evaluation + effective width. ----------------

slang_str slang_expr_call_subroutine_name(slang_ast node) {
    SLANG_C_ACCESS(borrowed(""), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::Call)
            return borrowed("");
        return borrowed(e->as<CallExpression>().getSubroutineName());
    });
}

bool slang_expr_call_is_system_call(slang_ast node) {
    SLANG_C_ACCESS(false, {
        auto e = exprOf(node);
        return e && e->kind == ExpressionKind::Call && e->as<CallExpression>().isSystemCall();
    });
}

slang_ast slang_expr_call_this_class(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::Call)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto thisClass = e->as<CallExpression>().thisClass();
        return thisClass ? wrapAst(*thisClass, node.compilation)
                         : noAst(node.compilation, SLANG_AST_EXPRESSION);
    });
}

uint32_t slang_expr_cond_condition_count(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::ConditionalOp)
            return 0u;
        return (uint32_t)e->as<ConditionalExpression>().conditions.size();
    });
}

slang_ast slang_expr_cond_condition_expr(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::ConditionalOp)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto& conditions = e->as<ConditionalExpression>().conditions;
        if (index >= conditions.size())
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(*conditions[index].expr, node.compilation);
    });
}

slang_ast slang_expr_cond_condition_pattern(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_PATTERN), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::ConditionalOp)
            return noAst(node.compilation, SLANG_AST_PATTERN);
        auto& conditions = e->as<ConditionalExpression>().conditions;
        if (index >= conditions.size())
            return noAst(node.compilation, SLANG_AST_PATTERN);
        auto pattern = conditions[index].pattern;
        return pattern ? wrapAst(*pattern, node.compilation)
                       : noAst(node.compilation, SLANG_AST_PATTERN);
    });
}

bool slang_expr_conversion_is_const_cast(slang_ast node) {
    SLANG_C_ACCESS(false, {
        auto e = exprOf(node);
        return e && e->kind == ExpressionKind::Conversion &&
               e->as<ConversionExpression>().isConstCast;
    });
}

bool slang_expr_conversion_is_implicit(slang_ast node) {
    SLANG_C_ACCESS(false, {
        auto e = exprOf(node);
        return e && e->kind == ExpressionKind::Conversion &&
               e->as<ConversionExpression>().isImplicit();
    });
}

slang_ast slang_expr_copy_class_source(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::CopyClass)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(e->as<CopyClassExpression>().sourceExpr(), node.compilation);
    });
}

slang_ast slang_expr_dist_left(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::Dist)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(e->as<DistExpression>().left(), node.compilation);
    });
}

uint32_t slang_expr_dist_item_count(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::Dist)
            return 0u;
        return (uint32_t)e->as<DistExpression>().items().size();
    });
}

slang_ast slang_expr_dist_item_value(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::Dist)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto items = e->as<DistExpression>().items();
        if (index >= items.size())
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(items[index].value, node.compilation);
    });
}

uint32_t slang_expr_dist_item_weight_kind(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(0xFFFFFFFFu, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::Dist)
            return 0xFFFFFFFFu;
        auto items = e->as<DistExpression>().items();
        if (index >= items.size() || !items[index].weight)
            return 0xFFFFFFFFu;
        return (uint32_t)items[index].weight->kind;
    });
}

slang_ast slang_expr_dist_item_weight_expr(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::Dist)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto items = e->as<DistExpression>().items();
        if (index >= items.size() || !items[index].weight || !items[index].weight->expr)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(*items[index].weight->expr, node.compilation);
    });
}

uint32_t slang_expr_dist_default_weight_kind(slang_ast node) {
    SLANG_C_ACCESS(0xFFFFFFFFu, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::Dist)
            return 0xFFFFFFFFu;
        auto dw = e->as<DistExpression>().defaultWeight();
        return dw ? (uint32_t)dw->kind : 0xFFFFFFFFu;
    });
}

slang_ast slang_expr_dist_default_weight_expr(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::Dist)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto dw = e->as<DistExpression>().defaultWeight();
        if (!dw || !dw->expr)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(*dw->expr, node.compilation);
    });
}

bool slang_expr_effective_width(slang_ast node, uint32_t* out) {
    SLANG_C_ACCESS(false, {
        auto e = exprOf(node);
        if (!e)
            return false;
        auto w = e->getEffectiveWidth();
        if (!w)
            return false;
        if (out)
            *out = (uint32_t)*w;
        return true;
    });
}

slang_lvalue slang_expr_eval_lvalue(slang_ast node, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    auto e = exprOf(node);
    auto comp = node.compilation;
    if (!e || !comp) {
        setError(err, SLANG_ERR_INVALID_ARG, "not an expression");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        // Evaluating an lvalue may materialize a scratch local for the
        // referenced value symbol, and throws a (catchable) internal slang
        // exception for an expression kind that does not implement lvalue
        // evaluation; lift the seal for the duration, exactly like
        // slang_expression_eval / slang_stmt_eval (the caller has promised
        // exclusive access).
        SealLift lift(comp);
        auto ctx = std::make_unique<EvalContext>(
            ASTContext(comp->comp->getRoot(), LookupLocation::max), EvalFlags::IsScript);
        ctx->pushEmptyFrame();
        if (auto sym = e->getSymbolReference(); sym && sym->isValue())
            ctx->createLocal(&sym->as<ValueSymbol>());
        LValue lval = e->evalLValue(*ctx);
        if (lval.bad())
            return (slang_lvalue) nullptr;
        return new slang_lvalue_t{std::move(ctx), std::move(lval)};
    });
    return nullptr;
}

void slang_lvalue_destroy(slang_lvalue lval) {
    delete lval;
}

bool slang_lvalue_is_bad(slang_lvalue lval) {
    SLANG_C_ACCESS(true, { return !lval || lval->lval.bad(); });
}

slang_str slang_lvalue_load(slang_lvalue lval) {
    SLANG_C_ACCESS(borrowed(""), {
        if (!lval || lval->lval.bad())
            return borrowed("");
        return owned(lval->lval.load().toString());
    });
}

bool slang_lvalue_store_int(slang_lvalue lval, int64_t value) {
    SLANG_C_ACCESS(false, {
        if (!lval || lval->lval.bad())
            return false;
        ConstantValue cur = lval->lval.load();
        if (!cur.isInteger())
            return false;
        auto& sv = cur.integer();
        SVInt newVal(sv.getBitWidth(), (uint64_t)value, sv.isSigned());
        lval->lval.store(newVal);
        return true;
    });
}

// ---- Replicated/StructuredAssignmentPattern, StreamingConcatenation, and
// StringLiteral breadth. Every field read below is set once at expression
// construction, so each is a pure read on a frozen compilation with no
// matching FreezeVisitor force required.

slang_ast slang_expr_replicated_pattern_count(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::ReplicatedAssignmentPattern)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(e->as<ReplicatedAssignmentPatternExpression>().count(), node.compilation);
    });
}

uint32_t slang_expr_structured_pattern_member_setter_count(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::StructuredAssignmentPattern)
            return 0u;
        return (uint32_t)e->as<StructuredAssignmentPatternExpression>().memberSetters.size();
    });
}

slang_ast slang_expr_structured_pattern_member_setter_member(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::StructuredAssignmentPattern)
            return noAst(node.compilation);
        auto& setters = e->as<StructuredAssignmentPatternExpression>().memberSetters;
        if (index >= setters.size())
            return noAst(node.compilation);
        return toC(setters[index].member.get(), node.compilation);
    });
}

slang_ast slang_expr_structured_pattern_member_setter_expr(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::StructuredAssignmentPattern)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto& setters = e->as<StructuredAssignmentPatternExpression>().memberSetters;
        if (index >= setters.size())
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(*setters[index].expr, node.compilation);
    });
}

uint32_t slang_expr_structured_pattern_type_setter_count(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::StructuredAssignmentPattern)
            return 0u;
        return (uint32_t)e->as<StructuredAssignmentPatternExpression>().typeSetters.size();
    });
}

slang_ast slang_expr_structured_pattern_type_setter_type(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::StructuredAssignmentPattern)
            return noAst(node.compilation);
        auto& setters = e->as<StructuredAssignmentPatternExpression>().typeSetters;
        if (index >= setters.size())
            return noAst(node.compilation);
        return toC(setters[index].type.get(), node.compilation);
    });
}

slang_ast slang_expr_structured_pattern_type_setter_expr(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::StructuredAssignmentPattern)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto& setters = e->as<StructuredAssignmentPatternExpression>().typeSetters;
        if (index >= setters.size())
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(*setters[index].expr, node.compilation);
    });
}

uint32_t slang_expr_structured_pattern_index_setter_count(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::StructuredAssignmentPattern)
            return 0u;
        return (uint32_t)e->as<StructuredAssignmentPatternExpression>().indexSetters.size();
    });
}

slang_ast slang_expr_structured_pattern_index_setter_index(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::StructuredAssignmentPattern)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto& setters = e->as<StructuredAssignmentPatternExpression>().indexSetters;
        if (index >= setters.size())
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(*setters[index].index, node.compilation);
    });
}

slang_ast slang_expr_structured_pattern_index_setter_expr(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::StructuredAssignmentPattern)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto& setters = e->as<StructuredAssignmentPatternExpression>().indexSetters;
        if (index >= setters.size())
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(*setters[index].expr, node.compilation);
    });
}

slang_ast slang_expr_structured_pattern_default_setter(slang_ast node) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::StructuredAssignmentPattern)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto def = e->as<StructuredAssignmentPatternExpression>().defaultSetter;
        return def ? wrapAst(*def, node.compilation) : noAst(node.compilation, SLANG_AST_EXPRESSION);
    });
}

uint64_t slang_expr_streaming_bitstream_width(slang_ast node) {
    SLANG_C_ACCESS(0ull, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::Streaming)
            return 0ull;
        return e->as<StreamingConcatenationExpression>().getBitstreamWidth();
    });
}

uint64_t slang_expr_streaming_slice_size(slang_ast node) {
    SLANG_C_ACCESS(0ull, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::Streaming)
            return 0ull;
        return e->as<StreamingConcatenationExpression>().getSliceSize();
    });
}

bool slang_expr_streaming_is_fixed_size(slang_ast node) {
    SLANG_C_ACCESS(false, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::Streaming)
            return false;
        return e->as<StreamingConcatenationExpression>().isFixedSize();
    });
}

uint32_t slang_expr_streaming_stream_count(slang_ast node) {
    SLANG_C_ACCESS(0u, {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::Streaming)
            return 0u;
        return (uint32_t)e->as<StreamingConcatenationExpression>().streams().size();
    });
}

slang_ast slang_expr_streaming_stream_operand(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::Streaming)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto streams = e->as<StreamingConcatenationExpression>().streams();
        if (index >= streams.size())
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(*streams[index].operand, node.compilation);
    });
}

slang_ast slang_expr_streaming_stream_with_expr(slang_ast node, uint32_t index) {
    SLANG_C_ACCESS(noAst(node.compilation, SLANG_AST_EXPRESSION), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::Streaming)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        auto streams = e->as<StreamingConcatenationExpression>().streams();
        if (index >= streams.size() || !streams[index].withExpr)
            return noAst(node.compilation, SLANG_AST_EXPRESSION);
        return wrapAst(*streams[index].withExpr, node.compilation);
    });
}

slang_str slang_expr_string_literal_value(slang_ast node) {
    SLANG_C_ACCESS(borrowed(""), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::StringLiteral)
            return borrowed("");
        return borrowed(e->as<StringLiteral>().getValue());
    });
}

slang_str slang_expr_string_literal_raw_value(slang_ast node) {
    SLANG_C_ACCESS(borrowed(""), {
        auto e = exprOf(node);
        if (!e || e->kind != ExpressionKind::StringLiteral)
            return borrowed("");
        return borrowed(e->as<StringLiteral>().getRawValue());
    });
}

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

slang_rand_mode slang_field_rand_mode(slang_ast field) {
    SLANG_C_ACCESS(SLANG_RAND_MODE_NONE, {
        auto sym = symbolOf(field);
        if (!sym || sym->kind != SymbolKind::Field)
            return SLANG_RAND_MODE_NONE;
        return (slang_rand_mode)(uint32_t)sym->as<FieldSymbol>().randMode;
    });
}

slang_rand_mode slang_symbol_rand_mode(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_RAND_MODE_NONE, {
        auto s = symbolOf(sym);
        if (!s)
            return SLANG_RAND_MODE_NONE;
        return (slang_rand_mode)(uint32_t)s->getRandMode();
    });
}

SLANG_TYPE_PRED(slang_type_is_enum, isEnum)
SLANG_TYPE_PRED(slang_type_is_struct, isStruct)
SLANG_TYPE_PRED(slang_type_is_union, isUnion)
SLANG_TYPE_PRED(slang_type_is_array, isArray)
SLANG_TYPE_PRED(slang_type_is_string, isString)
SLANG_TYPE_PRED(slang_type_can_be_string_like, canBeStringLike)
SLANG_TYPE_PRED(slang_type_has_fixed_range, hasFixedRange)
SLANG_TYPE_PRED(slang_type_is_aggregate, isAggregate)
SLANG_TYPE_PRED(slang_type_is_associative_array, isAssociativeArray)
// Unlike every other SLANG_TYPE_PRED here, Type::isAlias() itself checks
// `kind` directly rather than canonicalizing first — see the doc comment on
// slang_type_is_alias in slang.h.
SLANG_TYPE_PRED(slang_type_is_alias, isAlias)

#undef SLANG_TYPE_PRED

slang_scalar_kind slang_type_scalar_kind(slang_ast type) {
    SLANG_C_ACCESS(SLANG_SCALAR_BIT, {
        auto t = typeOf(type);
        if (!t)
            return SLANG_SCALAR_BIT;
        auto& ct = t->getCanonicalType();
        if (ct.kind != SymbolKind::ScalarType)
            return SLANG_SCALAR_BIT;
        return (slang_scalar_kind)ct.as<ScalarType>().scalarKind;
    });
}

slang_ast slang_type_associative_index_type(slang_ast type) {
    SLANG_C_ACCESS(noAst(type.compilation), {
        auto t = typeOf(type);
        if (!t)
            return noAst(type.compilation);
        return toC(t->getAssociativeIndexType(), type.compilation);
    });
}

uint64_t slang_type_bitstream_width(slang_ast type) {
    SLANG_C_ACCESS(0ull, {
        auto t = typeOf(type);
        return t ? t->getBitstreamWidth() : 0ull;
    });
}

// Both class chains are walked via ClassType::getBaseClass, whose memo the
// freeze sweep force-resolves (FreezeVisitor's ClassType branch), so this is
// a pure read.
slang_ast slang_type_common_base(slang_ast a, slang_ast b) {
    SLANG_C_ACCESS(noAst(a.compilation), {
        auto ta = typeOf(a);
        auto tb = typeOf(b);
        if (!ta || !tb)
            return noAst(a.compilation);
        return toC(Type::getCommonBase(*ta, *tb), a.compilation);
    });
}

uint32_t slang_type_integral_flags(slang_ast type) {
    SLANG_C_ACCESS(0u, {
        auto t = typeOf(type);
        return t ? (uint32_t)t->getIntegralFlags().bits() : 0u;
    });
}

uint64_t slang_type_selectable_width(slang_ast type) {
    SLANG_C_ACCESS(0ull, {
        auto t = typeOf(type);
        return t ? t->getSelectableWidth() : 0ull;
    });
}

// implements() walks the base-class chain via ClassType::getBaseClass and
// reads ClassType::getImplementedInterfaces (populated by the same
// ensureElaborated() call as getBaseClass); both memos are force-resolved by
// the freeze sweep, so this is a pure read.
bool slang_type_implements(slang_ast type, slang_ast iface_class) {
    SLANG_C_ACCESS(false, {
        auto t = typeOf(type);
        auto iface = typeOf(iface_class);
        return t && iface && t->implements(*iface);
    });
}

bool slang_type_is_bitstream_castable(slang_ast type, slang_ast rhs) {
    SLANG_C_ACCESS(false, {
        auto t = typeOf(type);
        auto r = typeOf(rhs);
        return t && r && t->isBitstreamCastable(*r);
    });
}

// isBitstreamType() recurses through array element types, unpacked-struct
// field types, and (for classes) property types, every one of which is a
// DeclaredType carrier whose getType() memo the freeze sweep force-resolves
// (FreezeVisitor's Symbol branch), so this is a pure, allocation-free read.
bool slang_type_is_bitstream_type(slang_ast type, bool destination) {
    SLANG_C_ACCESS(false, {
        auto t = typeOf(type);
        return t && t->isBitstreamType(destination);
    });
}

// A boolean type predicate taking one type argument, mirroring
// SLANG_TYPE_PRED above but named to avoid an fn/method spelling mismatch.
#define SLANG_TYPE_PRED2(fn, method)                                                   \
    bool fn(slang_ast type) {                                                          \
        SLANG_C_ACCESS(false, {                                                        \
            auto t = typeOf(type);                                                     \
            return t && t->method();                                                   \
        });                                                                            \
    }

SLANG_TYPE_PRED2(slang_type_is_boolean_convertible, isBooleanConvertible)
SLANG_TYPE_PRED2(slang_type_is_byte_array, isByteArray)
SLANG_TYPE_PRED2(slang_type_is_chandle, isCHandle)
SLANG_TYPE_PRED2(slang_type_is_covergroup, isCovergroup)
SLANG_TYPE_PRED2(slang_type_is_dynamically_sized_array, isDynamicallySizedArray)
SLANG_TYPE_PRED2(slang_type_is_error, isError)
SLANG_TYPE_PRED2(slang_type_is_event, isEvent)
SLANG_TYPE_PRED2(slang_type_is_fixed_size, isFixedSize)
SLANG_TYPE_PRED2(slang_type_is_floating, isFloating)
SLANG_TYPE_PRED2(slang_type_is_handle_type, isHandleType)
SLANG_TYPE_PRED2(slang_type_is_iterable, isIterable)
SLANG_TYPE_PRED2(slang_type_is_null, isNull)
SLANG_TYPE_PRED2(slang_type_is_numeric, isNumeric)
SLANG_TYPE_PRED2(slang_type_is_object_handle_type, isObjectHandleType)
SLANG_TYPE_PRED2(slang_type_is_packed_array, isPackedArray)
SLANG_TYPE_PRED2(slang_type_is_packed_union, isPackedUnion)
SLANG_TYPE_PRED2(slang_type_is_predefined_integer, isPredefinedInteger)
SLANG_TYPE_PRED2(slang_type_is_property_type, isPropertyType)
SLANG_TYPE_PRED2(slang_type_is_queue, isQueue)
SLANG_TYPE_PRED2(slang_type_is_scalar, isScalar)
SLANG_TYPE_PRED2(slang_type_is_sequence_type, isSequenceType)
SLANG_TYPE_PRED2(slang_type_is_simple_bit_vector, isSimpleBitVector)
SLANG_TYPE_PRED2(slang_type_is_simple_type, isSimpleType)
SLANG_TYPE_PRED2(slang_type_is_singular, isSingular)
SLANG_TYPE_PRED2(slang_type_is_tagged_union, isTaggedUnion)
SLANG_TYPE_PRED2(slang_type_is_type_ref_type, isTypeRefType)
SLANG_TYPE_PRED2(slang_type_is_unbounded, isUnbounded)
SLANG_TYPE_PRED2(slang_type_is_unpacked_struct, isUnpackedStruct)
SLANG_TYPE_PRED2(slang_type_is_unpacked_union, isUnpackedUnion)
SLANG_TYPE_PRED2(slang_type_is_untyped_type, isUntypedType)
SLANG_TYPE_PRED2(slang_type_is_valid_for_dpi_arg, isValidForDPIArg)
SLANG_TYPE_PRED2(slang_type_is_valid_for_dpi_return, isValidForDPIReturn)
SLANG_TYPE_PRED2(slang_type_is_valid_for_sequence, isValidForSequence)
SLANG_TYPE_PRED2(slang_type_is_virtual_interface, isVirtualInterface)
SLANG_TYPE_PRED2(slang_type_is_void, isVoid)

#undef SLANG_TYPE_PRED2

// isValidForRand() bottoms out in isIntegral()/isFloating()/isArray() (which
// recurses via getArrayElementType())/isClass()/isUnpackedStruct() — all
// canonical-type/DeclaredType reads the freeze sweep already force-resolves
// — so, like the SLANG_TYPE_PRED2 predicates above, this is a pure,
// allocation-free read.
bool slang_type_is_valid_for_rand(slang_ast type, slang_rand_mode mode,
                                  slang_language_version language_version) {
    SLANG_C_ACCESS(false, {
        auto t = typeOf(type);
        return t && t->isValidForRand(static_cast<RandMode>(mode),
                                      static_cast<LanguageVersion>(language_version));
    });
}

// TypeAliasType::visibility is a plain field set once at construction (never
// lazily computed), so this is a pure, allocation-free read.
slang_visibility slang_type_alias_visibility(slang_ast type) {
    SLANG_C_ACCESS(SLANG_VISIBILITY_PUBLIC, {
        auto t = typeOf(type);
        if (!t || t->kind != SymbolKind::TypeAlias)
            return SLANG_VISIBILITY_PUBLIC;
        return (slang_visibility)t->as<TypeAliasType>().visibility;
    });
}

bool slang_type_is_cast_compatible(slang_ast type, slang_ast rhs) {
    SLANG_C_ACCESS(false, {
        auto t = typeOf(type);
        auto r = typeOf(rhs);
        return t && r && t->isCastCompatible(*r);
    });
}

// isDerivedFrom() walks the base-class chain via ClassType::getBaseClass,
// whose memo the freeze sweep force-resolves (same as slang_type_implements
// above), so this is a pure read.
bool slang_type_is_derived_from(slang_ast type, slang_ast base) {
    SLANG_C_ACCESS(false, {
        auto t = typeOf(type);
        auto b = typeOf(base);
        return t && b && t->isDerivedFrom(*b);
    });
}

// getDefaultValue()/coerceValue() below only ever build a fresh ConstantValue
// on the regular process heap (via GetDefaultVisitor's per-kind
// getDefaultValueImpl(), or ConstantValue::convertTo*) — never the
// compilation's frozen bump arena — so, unlike slang_expression_eval_constant,
// these are pure reads requiring no exclusive access to the design.
slang_constant slang_type_default_value(slang_ast type, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    auto t = typeOf(type);
    if (!t) {
        setError(err, SLANG_ERR_INVALID_ARG, "not a type");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        ConstantValue cv = t->getDefaultValue();
        if (cv.bad())
            return (slang_constant) nullptr;
        return new slang_constant_t{std::move(cv)};
    });
    return nullptr;
}

slang_constant slang_type_coerce_value(slang_ast type, slang_constant value, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    auto t = typeOf(type);
    if (!t || !value) {
        setError(err, SLANG_ERR_INVALID_ARG, "not a type or constant");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        ConstantValue cv = t->coerceValue(value->value);
        if (cv.bad())
            return (slang_constant) nullptr;
        return new slang_constant_t{std::move(cv)};
    });
    return nullptr;
}

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

// A ClassType member accessor: recover the Type, canonicalize, bail to the
// neutral value unless it's a ClassType, then read one member off it. Every
// slang_type_class_* nullable-symbol/expr accessor below shares this shape.
#define SLANG_CLASS_TYPE_MEMBER(fn, expr)                                             \
    slang_ast fn(slang_ast type) {                                                    \
        SLANG_C_ACCESS(noAst(type.compilation), {                                     \
            auto t = typeOf(type);                                                    \
            if (!t)                                                                   \
                return noAst(type.compilation);                                       \
            auto& ct = t->getCanonicalType();                                         \
            if (ct.kind != SymbolKind::ClassType)                                     \
                return noAst(type.compilation);                                       \
            auto& cls = ct.as<ClassType>();                                           \
            auto member = (expr);                                                     \
            return member ? toC(member, type.compilation) : noAst(type.compilation);  \
        });                                                                           \
    }

SLANG_CLASS_TYPE_MEMBER(slang_type_class_generic, cls.genericClass)
// getConstructor()'s only allocating path (synthesizing a default 'new' for
// `extends Base(default)`) is unreachable unless an extends clause exists, in
// which case slang_type_class_base_constructor_call's freeze-sweep force has
// already called getConstructor() and settled it — this is a pure read.
SLANG_CLASS_TYPE_MEMBER(slang_type_class_constructor, cls.getConstructor())
// firstForward is linked by the class's *parent* scope while resolving a
// forward-typedef/class name conflict during that parent's own member
// elaboration — always before this class type is itself reachable by the
// freeze sweep (or by any accessor on a frozen design) — so this is a pure
// read with no separate FreezeVisitor force needed.
SLANG_CLASS_TYPE_MEMBER(slang_type_class_first_forward_decl, cls.getFirstForwardDecl())
SLANG_CLASS_TYPE_MEMBER(slang_type_class_this_var, cls.thisVar)

#undef SLANG_CLASS_TYPE_MEMBER

// getBaseConstructorCall() is pre-forced by the freeze sweep (FreezeVisitor's
// ClassType branch), so reading it here never mutates the frozen arena.
slang_ast slang_type_class_base_constructor_call(slang_ast type) {
    SLANG_C_ACCESS(noAst(type.compilation, SLANG_AST_EXPRESSION), {
        auto t = typeOf(type);
        if (!t)
            return noAst(type.compilation, SLANG_AST_EXPRESSION);
        auto& ct = t->getCanonicalType();
        if (ct.kind != SymbolKind::ClassType)
            return noAst(type.compilation, SLANG_AST_EXPRESSION);
        auto call = ct.as<ClassType>().getBaseConstructorCall();
        return call ? wrapAst(*call, type.compilation)
                    : noAst(type.compilation, SLANG_AST_EXPRESSION);
    });
}

uint32_t slang_type_class_implemented_interface_count(slang_ast type) {
    SLANG_C_ACCESS(0u, {
        auto t = typeOf(type);
        if (!t)
            return 0u;
        auto& ct = t->getCanonicalType();
        if (ct.kind != SymbolKind::ClassType)
            return 0u;
        // getImplementedInterfaces() calls ensureElaborated(), which the
        // class's own scope elaboration (forced transitively by the freeze
        // sweep's t.getBaseClass() call) has already run; a pure read.
        return (uint32_t)ct.as<ClassType>().getImplementedInterfaces().size();
    });
}

slang_ast slang_type_class_implemented_interface(slang_ast type, uint32_t index) {
    SLANG_C_ACCESS(noAst(type.compilation), {
        auto t = typeOf(type);
        if (!t)
            return noAst(type.compilation);
        auto& ct = t->getCanonicalType();
        if (ct.kind != SymbolKind::ClassType)
            return noAst(type.compilation);
        auto ifaces = ct.as<ClassType>().getImplementedInterfaces();
        if (index >= ifaces.size())
            return noAst(type.compilation);
        return toC(ifaces[index], type.compilation);
    });
}

bool slang_type_class_is_abstract(slang_ast type) {
    SLANG_C_ACCESS(false, {
        auto t = typeOf(type);
        if (!t)
            return false;
        auto& ct = t->getCanonicalType();
        return ct.kind == SymbolKind::ClassType && ct.as<ClassType>().isAbstract;
    });
}

bool slang_type_class_is_final(slang_ast type) {
    SLANG_C_ACCESS(false, {
        auto t = typeOf(type);
        if (!t)
            return false;
        auto& ct = t->getCanonicalType();
        return ct.kind == SymbolKind::ClassType && ct.as<ClassType>().isFinal;
    });
}

bool slang_type_class_is_interface(slang_ast type) {
    SLANG_C_ACCESS(false, {
        auto t = typeOf(type);
        if (!t)
            return false;
        auto& ct = t->getCanonicalType();
        return ct.kind == SymbolKind::ClassType && ct.as<ClassType>().isInterface;
    });
}

uint32_t slang_symbol_constraint_block_flags(slang_ast sym_node) {
    SLANG_C_ACCESS(0u, {
        auto sym = symbolOf(sym_node);
        if (!sym || sym->kind != SymbolKind::ConstraintBlock)
            return 0u;
        return (uint32_t)sym->as<ConstraintBlockSymbol>().flags.bits();
    });
}

slang_ast slang_symbol_constraint_block_this_var(slang_ast sym_node) {
    SLANG_C_ACCESS(noAst(sym_node.compilation), {
        auto sym = symbolOf(sym_node);
        if (!sym || sym->kind != SymbolKind::ConstraintBlock)
            return noAst(sym_node.compilation);
        auto tv = sym->as<ConstraintBlockSymbol>().thisVar;
        return tv ? toC(tv, sym_node.compilation) : noAst(sym_node.compilation);
    });
}

slang_ast slang_symbol_constraint_block_constraints(slang_ast sym_node) {
    SLANG_C_ACCESS(noAst(sym_node.compilation, SLANG_AST_CONSTRAINT), {
        auto sym = symbolOf(sym_node);
        if (!sym || sym->kind != SymbolKind::ConstraintBlock)
            return noAst(sym_node.compilation, SLANG_AST_CONSTRAINT);
        // Pre-forced by the freeze sweep (FreezeVisitor's ConstraintBlockSymbol
        // branch), so this never mutates the frozen arena.
        return wrapAst(sym->as<ConstraintBlockSymbol>().getConstraints(), sym_node.compilation);
    });
}

uint32_t slang_type_covergroup_argument_count(slang_ast type) {
    SLANG_C_ACCESS(0u, {
        auto t = typeOf(type);
        if (!t || t->kind != SymbolKind::CovergroupType)
            return 0u;
        // getArguments() calls ensureElaborated(); covergroup-scope
        // elaboration is forced transitively by the freeze sweep's
        // CovergroupType getCoverageEvent()/getBaseGroup() forces.
        return (uint32_t)t->as<CovergroupType>().getArguments().size();
    });
}

slang_ast slang_type_covergroup_argument(slang_ast type, uint32_t index) {
    SLANG_C_ACCESS(noAst(type.compilation), {
        auto t = typeOf(type);
        if (!t || t->kind != SymbolKind::CovergroupType)
            return noAst(type.compilation);
        auto args = t->as<CovergroupType>().getArguments();
        if (index >= args.size())
            return noAst(type.compilation);
        return toC(args[index], type.compilation);
    });
}

slang_argument_direction slang_symbol_formal_argument_direction(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_ARGUMENT_DIRECTION_IN, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::FormalArgument)
            return SLANG_ARGUMENT_DIRECTION_IN;
        return (slang_argument_direction)(uint32_t)s->as<FormalArgumentSymbol>().direction;
    });
}

slang_ast slang_symbol_formal_argument_default_value(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::FormalArgument)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        // Forced by the freeze sweep (FormalArgumentSymbol::getDefaultValue),
        // so this is a pure read.
        auto expr = s->as<FormalArgumentSymbol>().getDefaultValue();
        return expr ? wrapAst(*expr, sym.compilation) : noAst(sym.compilation, SLANG_AST_EXPRESSION);
    });
}

slang_ast slang_type_covergroup_base_group(slang_ast type) {
    SLANG_C_ACCESS(noAst(type.compilation), {
        auto t = typeOf(type);
        if (!t || t->kind != SymbolKind::CovergroupType)
            return noAst(type.compilation);
        // Pre-forced by the freeze sweep (FreezeVisitor's CovergroupType branch).
        auto base = t->as<CovergroupType>().getBaseGroup();
        return base ? toC(base, type.compilation) : noAst(type.compilation);
    });
}

slang_ast slang_type_covergroup_coverage_event(slang_ast type) {
    SLANG_C_ACCESS(noAst(type.compilation, SLANG_AST_TIMING_CONTROL), {
        auto t = typeOf(type);
        if (!t || t->kind != SymbolKind::CovergroupType)
            return noAst(type.compilation, SLANG_AST_TIMING_CONTROL);
        // Pre-forced by the freeze sweep (FreezeVisitor's CovergroupType branch).
        auto ev = t->as<CovergroupType>().getCoverageEvent();
        return ev ? wrapAst(*ev, type.compilation)
                  : noAst(type.compilation, SLANG_AST_TIMING_CONTROL);
    });
}

bool slang_type_dpi_open_array_is_packed(slang_ast type) {
    SLANG_C_ACCESS(false, {
        auto t = typeOf(type);
        if (!t)
            return false;
        auto& ct = t->getCanonicalType();
        return ct.kind == SymbolKind::DPIOpenArrayType && ct.as<DPIOpenArrayType>().isPacked;
    });
}

int32_t slang_type_enum_system_id(slang_ast type) {
    SLANG_C_ACCESS(0, {
        auto t = typeOf(type);
        if (!t)
            return 0;
        auto& ct = t->getCanonicalType();
        if (ct.kind != SymbolKind::EnumType)
            return 0;
        return (int32_t)ct.as<EnumType>().systemId;
    });
}

slang_float_kind slang_type_floating_kind(slang_ast type) {
    SLANG_C_ACCESS(SLANG_FLOAT_REAL, {
        auto t = typeOf(type);
        if (!t)
            return SLANG_FLOAT_REAL;
        auto& ct = t->getCanonicalType();
        if (ct.kind != SymbolKind::FloatingType)
            return SLANG_FLOAT_REAL;
        return (slang_float_kind)ct.as<FloatingType>().floatKind;
    });
}

slang_forward_type_restriction slang_forwarding_typedef_type_restriction(slang_ast sym_node) {
    SLANG_C_ACCESS(SLANG_FORWARD_TYPE_NONE, {
        auto sym = symbolOf(sym_node);
        if (!sym || sym->kind != SymbolKind::ForwardingTypedef)
            return SLANG_FORWARD_TYPE_NONE;
        return (slang_forward_type_restriction)sym->as<ForwardingTypedefSymbol>().typeRestriction;
    });
}

bool slang_forwarding_typedef_visibility(slang_ast sym_node, slang_visibility* out) {
    SLANG_C_ACCESS(false, {
        if (!out)
            return false;
        auto sym = symbolOf(sym_node);
        if (!sym || sym->kind != SymbolKind::ForwardingTypedef)
            return false;
        auto& vis = sym->as<ForwardingTypedefSymbol>().visibility;
        if (!vis.has_value())
            return false;
        *out = (slang_visibility)*vis;
        return true;
    });
}

bool slang_generic_class_is_interface(slang_ast sym_node) {
    SLANG_C_ACCESS(false, {
        auto sym = symbolOf(sym_node);
        return sym && sym->kind == SymbolKind::GenericClassDef &&
               sym->as<GenericClassDefSymbol>().isInterface;
    });
}

slang_ast slang_generic_class_default_specialization(slang_ast sym_node) {
    SLANG_C_ACCESS(noAst(sym_node.compilation), {
        auto sym = symbolOf(sym_node);
        if (!sym || sym->kind != SymbolKind::GenericClassDef)
            return noAst(sym_node.compilation);
        auto& gc = sym->as<GenericClassDefSymbol>();
        auto scope = gc.getParentScope();
        if (!scope)
            return noAst(sym_node.compilation);
        // Forced by the freeze sweep (FreezeVisitor's GenericClassDefSymbol
        // branch: `t.getDefaultSpecialization(*sc)`), so this is a pure read.
        auto spec = gc.getDefaultSpecialization(*scope);
        return spec ? toC(spec, sym_node.compilation) : noAst(sym_node.compilation);
    });
}

slang_ast slang_generic_class_first_forward_decl(slang_ast sym_node) {
    SLANG_C_ACCESS(noAst(sym_node.compilation), {
        auto sym = symbolOf(sym_node);
        if (!sym || sym->kind != SymbolKind::GenericClassDef)
            return noAst(sym_node.compilation);
        auto fwd = sym->as<GenericClassDefSymbol>().getFirstForwardDecl();
        return fwd ? toC(fwd, sym_node.compilation) : noAst(sym_node.compilation);
    });
}

slang_ast slang_generic_class_invalid_specialization(slang_ast sym_node, slang_error* err) {
    if (!checkEntry(err))
        return noAst(sym_node.compilation);
    auto sym = symbolOf(sym_node);
    if (!sym || sym->kind != SymbolKind::GenericClassDef) {
        setError(err, SLANG_ERR_INVALID_ARG, "not a generic class definition");
        return noAst(sym_node.compilation);
    }
    SLANG_C_GUARD(err, {
        // getInvalidSpecialization() always emplaces a brand-new ClassType
        // into the arena on every call (it takes the forceInvalidParams path
        // in getSpecializationImpl, which skips the specMap/uninstantiatedSpecMap
        // dedup entirely) — never cached, unlike getDefaultSpecialization.
        // Lift the seal for the call; the caller has promised exclusive access.
        SealLift lift(sym_node.compilation);
        auto& spec = sym->as<GenericClassDefSymbol>().getInvalidSpecialization();
        return toC(&spec, sym_node.compilation);
    });
    return noAst(sym_node.compilation);
}

// ---- Type breadth: IntegralType / PackedArrayType / PackedStructType /
//      PackedUnionType / PredefinedIntegerType / QueueType / NetType --------

slang_constant_range slang_type_bit_vector_range(slang_ast type) {
    SLANG_C_ACCESS(slang_constant_range{}, {
        auto t = typeOf(type);
        if (!t || !t->isIntegral())
            return slang_constant_range{};
        auto r = t->getCanonicalType().as<IntegralType>().getBitVectorRange();
        return slang_constant_range{r.left, r.right};
    });
}

bool slang_type_is_declared_reg(slang_ast type) {
    SLANG_C_ACCESS(false, {
        auto t = typeOf(type);
        return t && t->isIntegral() &&
               t->getCanonicalType().as<IntegralType>().isDeclaredReg();
    });
}

slang_constant_range slang_type_packed_array_range(slang_ast type) {
    SLANG_C_ACCESS(slang_constant_range{}, {
        auto t = typeOf(type);
        if (!t)
            return slang_constant_range{};
        auto& ct = t->getCanonicalType();
        if (ct.kind != SymbolKind::PackedArrayType)
            return slang_constant_range{};
        auto& r = ct.as<PackedArrayType>().range;
        return slang_constant_range{r.left, r.right};
    });
}

slang_constant_range slang_type_fixed_range(slang_ast type) {
    SLANG_C_ACCESS(slang_constant_range{}, {
        auto t = typeOf(type);
        if (!t)
            return slang_constant_range{};
        auto r = t->getFixedRange();
        return slang_constant_range{r.left, r.right};
    });
}

int32_t slang_type_packed_struct_system_id(slang_ast type) {
    SLANG_C_ACCESS(0, {
        auto t = typeOf(type);
        if (!t)
            return 0;
        auto& ct = t->getCanonicalType();
        if (ct.kind != SymbolKind::PackedStructType)
            return 0;
        return (int32_t)ct.as<PackedStructType>().systemId;
    });
}

bool slang_type_packed_union_is_soft(slang_ast type) {
    SLANG_C_ACCESS(false, {
        auto t = typeOf(type);
        if (!t)
            return false;
        auto& ct = t->getCanonicalType();
        return ct.kind == SymbolKind::PackedUnionType && ct.as<PackedUnionType>().isSoft;
    });
}

bool slang_type_packed_union_is_tagged(slang_ast type) {
    SLANG_C_ACCESS(false, {
        auto t = typeOf(type);
        if (!t)
            return false;
        auto& ct = t->getCanonicalType();
        return ct.kind == SymbolKind::PackedUnionType && ct.as<PackedUnionType>().isTagged;
    });
}

int32_t slang_type_packed_union_system_id(slang_ast type) {
    SLANG_C_ACCESS(0, {
        auto t = typeOf(type);
        if (!t)
            return 0;
        auto& ct = t->getCanonicalType();
        if (ct.kind != SymbolKind::PackedUnionType)
            return 0;
        return (int32_t)ct.as<PackedUnionType>().systemId;
    });
}

int32_t slang_type_unpacked_struct_system_id(slang_ast type) {
    SLANG_C_ACCESS(0, {
        auto t = typeOf(type);
        if (!t)
            return 0;
        auto& ct = t->getCanonicalType();
        if (ct.kind != SymbolKind::UnpackedStructType)
            return 0;
        return (int32_t)ct.as<UnpackedStructType>().systemId;
    });
}

bool slang_type_unpacked_union_is_tagged(slang_ast type) {
    SLANG_C_ACCESS(false, {
        auto t = typeOf(type);
        if (!t)
            return false;
        auto& ct = t->getCanonicalType();
        return ct.kind == SymbolKind::UnpackedUnionType && ct.as<UnpackedUnionType>().isTagged;
    });
}

int32_t slang_type_unpacked_union_system_id(slang_ast type) {
    SLANG_C_ACCESS(0, {
        auto t = typeOf(type);
        if (!t)
            return 0;
        auto& ct = t->getCanonicalType();
        if (ct.kind != SymbolKind::UnpackedUnionType)
            return 0;
        return (int32_t)ct.as<UnpackedUnionType>().systemId;
    });
}

uint32_t slang_type_packed_union_tag_bits(slang_ast type) {
    SLANG_C_ACCESS(0u, {
        auto t = typeOf(type);
        if (!t)
            return 0u;
        auto& ct = t->getCanonicalType();
        if (ct.kind != SymbolKind::PackedUnionType)
            return 0u;
        return ct.as<PackedUnionType>().tagBits;
    });
}

slang_predefined_integer_kind slang_type_predefined_integer_kind(slang_ast type) {
    SLANG_C_ACCESS(SLANG_PREDEFINED_INTEGER_SHORTINT, {
        auto t = typeOf(type);
        if (!t)
            return SLANG_PREDEFINED_INTEGER_SHORTINT;
        auto& ct = t->getCanonicalType();
        if (ct.kind != SymbolKind::PredefinedIntegerType)
            return SLANG_PREDEFINED_INTEGER_SHORTINT;
        switch (ct.as<PredefinedIntegerType>().integerKind) {
            case PredefinedIntegerType::ShortInt:
                return SLANG_PREDEFINED_INTEGER_SHORTINT;
            case PredefinedIntegerType::Int:
                return SLANG_PREDEFINED_INTEGER_INT;
            case PredefinedIntegerType::LongInt:
                return SLANG_PREDEFINED_INTEGER_LONGINT;
            case PredefinedIntegerType::Byte:
                return SLANG_PREDEFINED_INTEGER_BYTE;
            case PredefinedIntegerType::Integer:
                return SLANG_PREDEFINED_INTEGER_INTEGER;
            case PredefinedIntegerType::Time:
                return SLANG_PREDEFINED_INTEGER_TIME;
        }
        return SLANG_PREDEFINED_INTEGER_SHORTINT;
    });
}

uint32_t slang_type_queue_max_bound(slang_ast type) {
    SLANG_C_ACCESS(0u, {
        auto t = typeOf(type);
        if (!t)
            return 0u;
        auto& ct = t->getCanonicalType();
        if (ct.kind != SymbolKind::QueueType)
            return 0u;
        return ct.as<QueueType>().maxBound;
    });
}

slang_net_kind slang_net_type_net_kind(slang_ast net_type) {
    SLANG_C_ACCESS(SLANG_NET_UNKNOWN, {
        auto sym = symbolOf(net_type);
        if (!sym || sym->kind != SymbolKind::NetType)
            return SLANG_NET_UNKNOWN;
        switch (sym->as<NetType>().netKind) {
            case NetType::Unknown:
                return SLANG_NET_UNKNOWN;
            case NetType::Wire:
                return SLANG_NET_WIRE;
            case NetType::WAnd:
                return SLANG_NET_WAND;
            case NetType::WOr:
                return SLANG_NET_WOR;
            case NetType::Tri:
                return SLANG_NET_TRI;
            case NetType::TriAnd:
                return SLANG_NET_TRIAND;
            case NetType::TriOr:
                return SLANG_NET_TRIOR;
            case NetType::Tri0:
                return SLANG_NET_TRI0;
            case NetType::Tri1:
                return SLANG_NET_TRI1;
            case NetType::TriReg:
                return SLANG_NET_TRIREG;
            case NetType::Supply0:
                return SLANG_NET_SUPPLY0;
            case NetType::Supply1:
                return SLANG_NET_SUPPLY1;
            case NetType::UWire:
                return SLANG_NET_UWIRE;
            case NetType::Interconnect:
                return SLANG_NET_INTERCONNECT;
            case NetType::UserDefined:
                return SLANG_NET_USER_DEFINED;
        }
        return SLANG_NET_UNKNOWN;
    });
}

// Forced by the freeze sweep (FreezeVisitor's NetType branch: t.getResolutionFunction()),
// so this never mutates the frozen arena.
slang_ast slang_net_type_resolution_function(slang_ast net_type) {
    SLANG_C_ACCESS(noAst(net_type.compilation), {
        auto sym = symbolOf(net_type);
        if (!sym || sym->kind != SymbolKind::NetType)
            return noAst(net_type.compilation);
        auto fn = sym->as<NetType>().getResolutionFunction();
        return fn ? toC(fn, net_type.compilation) : noAst(net_type.compilation);
    });
}

bool slang_net_type_is_built_in(slang_ast net_type) {
    SLANG_C_ACCESS(false, {
        auto sym = symbolOf(net_type);
        return sym && sym->kind == SymbolKind::NetType && sym->as<NetType>().isBuiltIn();
    });
}

bool slang_net_type_is_error(slang_ast net_type) {
    SLANG_C_ACCESS(false, {
        auto sym = symbolOf(net_type);
        return sym && sym->kind == SymbolKind::NetType && sym->as<NetType>().isError();
    });
}

slang_ast slang_net_type_get_simulated(slang_ast internal_net, slang_ast external_net,
                                       bool* out_should_warn) {
    SLANG_C_ACCESS(noAst(internal_net.compilation), {
        auto i = symbolOf(internal_net);
        auto e = symbolOf(external_net);
        if (!i || i->kind != SymbolKind::NetType || !e || e->kind != SymbolKind::NetType)
            return noAst(internal_net.compilation);
        bool shouldWarn = false;
        auto& result = NetType::getSimulatedNetType(i->as<NetType>(), e->as<NetType>(),
                                                     shouldWarn);
        if (out_should_warn)
            *out_should_warn = shouldWarn;
        return toC(&result, internal_net.compilation);
    });
}

// ---- AST: type printing -------------------------------------------------------

// The handle behind slang_type_printer: a standalone utility object (its own
// heap-allocated FormatBuffer, never the design's frozen bump arena), so it
// carries no `comp`/lifetime tie to any particular slang_compilation. Kept
// file-local (unlike the shared handles in CApiInternal.h) since only this
// translation unit constructs one.
struct slang_type_printer_t {
    TypePrinter printer;
};

static TypePrintingOptions toOptions(slang_type_printing_options o) {
    TypePrintingOptions opts;
    opts.elideScopeNames = o.elide_scope_names;
    opts.classesAsLinks = o.classes_as_links;
    opts.enumsAsLinks = o.enums_as_links;
    opts.anonymousTypeStyle = o.anonymous_type_style == SLANG_ANON_TYPE_FRIENDLY_NAME
                                  ? TypePrintingOptions::FriendlyName
                                  : TypePrintingOptions::SystemName;
    if (o.has_quote_char)
        opts.quoteChar = o.quote_char;
    opts.printAKA = o.print_aka;
    opts.skipScopedTypeNames = o.skip_scoped_type_names;
    opts.skipTypeDefs = o.skip_type_defs;
    opts.fullEnumType = o.full_enum_type;
    opts.typedefsAsLinks = o.typedefs_as_links;
    opts.printIntegralRange = o.print_integral_range;
    opts.friendlyMemberCharLimit = o.friendly_member_char_limit;
    return opts;
}

static slang_type_printing_options fromOptions(const TypePrintingOptions& o) {
    slang_type_printing_options out{};
    out.elide_scope_names = o.elideScopeNames;
    out.classes_as_links = o.classesAsLinks;
    out.enums_as_links = o.enumsAsLinks;
    out.anonymous_type_style = o.anonymousTypeStyle == TypePrintingOptions::FriendlyName
                                    ? SLANG_ANON_TYPE_FRIENDLY_NAME
                                    : SLANG_ANON_TYPE_SYSTEM_NAME;
    out.has_quote_char = o.quoteChar.has_value();
    out.quote_char = o.quoteChar.has_value() ? *o.quoteChar : '\0';
    out.print_aka = o.printAKA;
    out.skip_scoped_type_names = o.skipScopedTypeNames;
    out.skip_type_defs = o.skipTypeDefs;
    out.full_enum_type = o.fullEnumType;
    out.typedefs_as_links = o.typedefsAsLinks;
    out.print_integral_range = o.printIntegralRange;
    out.friendly_member_char_limit = o.friendlyMemberCharLimit;
    return out;
}

slang_type_printer slang_type_printer_create(slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, { return new slang_type_printer_t(); });
    return nullptr;
}

void slang_type_printer_destroy(slang_type_printer printer) {
    delete printer;
}

// append() only ever touches already freeze-sweep-forced memos (canonical
// types, field/element DeclaredType::getType(), EnumValueSymbol::getValue())
// while writing into the printer's own FormatBuffer — never the design's
// frozen arena — so this is a pure read of `type` on a shared Design.
void slang_type_printer_append(slang_type_printer printer, slang_ast type) {
    if (!printer)
        return;
    SLANG_C_GUARD(nullptr, {
        auto t = typeOf(type);
        if (t)
            printer->printer.append(*t);
    });
}

void slang_type_printer_clear(slang_type_printer printer) {
    if (!printer)
        return;
    SLANG_C_GUARD(nullptr, { printer->printer.clear(); });
}

slang_str slang_type_printer_to_string(slang_type_printer printer, slang_error* err) {
    if (!checkEntry(err))
        return borrowed("");
    if (!printer)
        return borrowed("");
    SLANG_C_GUARD(err, { return owned(printer->printer.toString()); });
    return borrowed("");
}

slang_type_printing_options slang_type_printer_options(slang_type_printer printer) {
    if (!printer)
        return slang_type_printing_options{};
    SLANG_C_ACCESS(slang_type_printing_options{}, { return fromOptions(printer->printer.options); });
}

void slang_type_printer_set_options(slang_type_printer printer,
                                    slang_type_printing_options options) {
    if (!printer)
        return;
    printer->printer.options = toOptions(options);
}

// ---- MethodPrototypeSymbol / ModportClockingSymbol --------------------------

uint32_t slang_symbol_method_prototype_flags(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::MethodPrototype)
            return 0u;
        return (uint32_t)s->as<MethodPrototypeSymbol>().flags.bits();
    });
}

uint32_t slang_symbol_method_prototype_subroutine_kind(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::MethodPrototype)
            return 0u;
        return static_cast<uint32_t>(s->as<MethodPrototypeSymbol>().subroutineKind);
    });
}

slang_visibility slang_symbol_method_prototype_visibility(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_VISIBILITY_PUBLIC, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::MethodPrototype)
            return SLANG_VISIBILITY_PUBLIC;
        return (slang_visibility)(uint32_t)s->as<MethodPrototypeSymbol>().visibility;
    });
}

bool slang_symbol_method_prototype_is_virtual(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::MethodPrototype)
            return false;
        return s->as<MethodPrototypeSymbol>().isVirtual();
    });
}

uint32_t slang_symbol_method_prototype_argument_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::MethodPrototype)
            return 0u;
        return (uint32_t)s->as<MethodPrototypeSymbol>().getArguments().size();
    });
}

slang_ast slang_symbol_method_prototype_argument(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::MethodPrototype)
            return noAst(sym.compilation);
        auto args = s->as<MethodPrototypeSymbol>().getArguments();
        if (index >= args.size())
            return noAst(sym.compilation);
        return toC(args[index], sym.compilation);
    });
}

slang_ast slang_symbol_method_prototype_return_type(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::MethodPrototype)
            return noAst(sym.compilation);
        // Forced by the freeze sweep's generic DeclaredType carrier pass
        // (getDeclaredType()->getType()); a pure read.
        return wrapAst(s->as<MethodPrototypeSymbol>().getReturnType(), sym.compilation);
    });
}

slang_ast slang_symbol_method_prototype_subroutine(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::MethodPrototype)
            return noAst(sym.compilation);
        // Forced by the freeze sweep's generic ASTVisitor traversal
        // (visitDefault's MethodPrototypeSymbol branch calls getSubroutine()
        // whenever this prototype is reached as a scope member -- the only
        // way it is ever reachable at all); a pure read.
        auto sub = s->as<MethodPrototypeSymbol>().getSubroutine();
        return sub ? toC(sub, sym.compilation) : noAst(sym.compilation);
    });
}

slang_ast slang_symbol_method_prototype_override(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::MethodPrototype)
            return noAst(sym.compilation);
        auto ov = s->as<MethodPrototypeSymbol>().getOverride();
        return ov ? toC(ov, sym.compilation) : noAst(sym.compilation);
    });
}

slang_extern_impl slang_symbol_method_prototype_first_extern_impl(slang_ast sym) {
    SLANG_C_ACCESS(nullptr, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::MethodPrototype)
            return nullptr;
        auto impl = s->as<MethodPrototypeSymbol>().getFirstExternImpl();
        return reinterpret_cast<slang_extern_impl>(
            const_cast<MethodPrototypeSymbol::ExternImpl*>(impl));
    });
}

slang_extern_impl slang_extern_impl_next(slang_extern_impl impl) {
    SLANG_C_ACCESS(nullptr, {
        if (!impl)
            return nullptr;
        auto next = reinterpret_cast<const MethodPrototypeSymbol::ExternImpl*>(impl)->getNextImpl();
        return reinterpret_cast<slang_extern_impl>(
            const_cast<MethodPrototypeSymbol::ExternImpl*>(next));
    });
}

slang_ast slang_extern_impl_impl(slang_extern_impl impl, slang_compilation comp) {
    SLANG_C_ACCESS(noAst(comp), {
        if (!impl)
            return noAst(comp);
        auto& e = *reinterpret_cast<const MethodPrototypeSymbol::ExternImpl*>(impl);
        return toC(e.impl.get(), comp);
    });
}

slang_ast slang_symbol_modport_clocking_target(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ModportClocking)
            return noAst(sym.compilation);
        auto target = s->as<ModportClockingSymbol>().target;
        return target ? toC(target, sym.compilation) : noAst(sym.compilation);
    });
}

// ---- InterfacePortSymbol / LetDeclSymbol / Lookup ---------------------------

slang_iface_conn slang_symbol_interface_port_connection(slang_ast sym) {
    slang_iface_conn none{noAst(sym.compilation), noAst(sym.compilation)};
    SLANG_C_ACCESS(none, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::InterfacePort)
            return none;
        // Forced by the freeze sweep's InterfacePortSymbol branch
        // (getConnectionAndExpr(), which getConnection() itself calls), so
        // this is a pure read.
        auto conn = s->as<InterfacePortSymbol>().getConnection();
        return slang_iface_conn{capi::toC(conn.first, sym.compilation),
                                 capi::toC(conn.second, sym.compilation)};
    });
}

slang_iface_conn slang_instance_port_connection_iface_conn(slang_ast instance, uint32_t index) {
    slang_iface_conn none{noAst(instance.compilation), noAst(instance.compilation)};
    SLANG_C_ACCESS(none, {
        auto conn = instancePortConnection(instance, index);
        if (!conn)
            return none;
        // getIfaceConn reads only the plain `connectedSymbol`/`modport`
        // fields (populated at PortConnection construction time, never
        // lazily), so this needs no freeze-sweep force.
        auto ic = conn->getIfaceConn();
        return slang_iface_conn{toC(ic.first, instance.compilation),
                                 toC(ic.second, instance.compilation)};
    });
}

static std::optional<std::span<const ConstantRange>> interfacePortRange(slang_ast sym) {
    auto s = symbolOf(sym);
    if (!s || s->kind != SymbolKind::InterfacePort)
        return std::nullopt;
    // Forced by the freeze sweep alongside the connection, so a pure read.
    return s->as<InterfacePortSymbol>().getDeclaredRange();
}

uint32_t slang_symbol_interface_port_declared_range_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto range = interfacePortRange(sym);
        return range ? (uint32_t)range->size() : 0u;
    });
}

slang_constant_range slang_symbol_interface_port_declared_range_at(slang_ast sym,
                                                                    uint32_t index) {
    SLANG_C_ACCESS(slang_constant_range{}, {
        auto range = interfacePortRange(sym);
        if (!range || index >= range->size())
            return slang_constant_range{};
        auto& r = (*range)[index];
        return slang_constant_range{r.left, r.right};
    });
}

slang_ast slang_symbol_interface_port_interface_def(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::InterfacePort)
            return noAst(sym.compilation);
        return capi::toC(s->as<InterfacePortSymbol>().interfaceDef, sym.compilation);
    });
}

bool slang_symbol_interface_port_is_generic(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        return s && s->kind == SymbolKind::InterfacePort &&
               s->as<InterfacePortSymbol>().isGeneric;
    });
}

bool slang_symbol_interface_port_is_invalid(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        return s && s->kind == SymbolKind::InterfacePort &&
               s->as<InterfacePortSymbol>().isInvalid();
    });
}

slang_str slang_symbol_interface_port_modport(slang_ast sym) {
    SLANG_C_ACCESS(borrowed(""), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::InterfacePort)
            return borrowed("");
        return borrowed(s->as<InterfacePortSymbol>().modport);
    });
}

uint32_t slang_symbol_let_decl_port_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::LetDecl)
            return 0u;
        return (uint32_t)s->as<LetDeclSymbol>().ports.size();
    });
}

slang_ast slang_symbol_let_decl_port(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::LetDecl)
            return noAst(sym.compilation);
        auto ports = s->as<LetDeclSymbol>().ports;
        if (index >= ports.size())
            return noAst(sym.compilation);
        return capi::toC(ports[index], sym.compilation);
    });
}

slang_visibility slang_lookup_get_visibility(slang_ast symbol) {
    SLANG_C_ACCESS(SLANG_VISIBILITY_PUBLIC, {
        auto s = symbolOf(symbol);
        if (!s)
            return SLANG_VISIBILITY_PUBLIC;
        return (slang_visibility)(uint32_t)Lookup::getVisibility(*s);
    });
}

bool slang_lookup_is_visible_from(slang_ast symbol, slang_ast scope) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(symbol);
        auto ssym = symbolOf(scope);
        auto sc = ssym ? scopeOf(*ssym) : nullptr;
        if (!s || !sc)
            return false;
        return Lookup::isVisibleFrom(*s, *sc);
    });
}

bool slang_lookup_is_accessible_from(slang_ast target, slang_ast source_scope) {
    SLANG_C_ACCESS(false, {
        auto t = symbolOf(target);
        auto s = symbolOf(source_scope);
        if (!t || !s || !t->getParentScope())
            return false;
        return Lookup::isAccessibleFrom(*t, *s);
    });
}

bool slang_lookup_ensure_visible(slang_ast symbol, slang_ast context_scope, slang_error* err) {
    if (!checkEntry(err))
        return false;
    auto s = symbolOf(symbol);
    auto csym = symbolOf(context_scope);
    auto scope = csym ? scopeOf(*csym) : nullptr;
    if (!s || !scope) {
        setError(err, SLANG_ERR_INVALID_ARG, s ? "not a scope" : "not a symbol");
        return false;
    }
    SLANG_C_GUARD(err, {
        SealLift lift(symbol.compilation);
        ASTContext context(*scope, LookupLocation::max);
        return Lookup::ensureVisible(*s, context, std::nullopt);
    });
    return false;
}

bool slang_lookup_ensure_accessible(slang_ast symbol, slang_ast context_scope, slang_error* err) {
    if (!checkEntry(err))
        return false;
    auto s = symbolOf(symbol);
    auto csym = symbolOf(context_scope);
    auto scope = csym ? scopeOf(*csym) : nullptr;
    if (!s || !scope) {
        setError(err, SLANG_ERR_INVALID_ARG, s ? "not a scope" : "not a symbol");
        return false;
    }
    SLANG_C_GUARD(err, {
        SealLift lift(symbol.compilation);
        ASTContext context(*scope, LookupLocation::max);
        return Lookup::ensureAccessible(*s, context, std::nullopt);
    });
    return false;
}

slang_ast slang_lookup_name(slang_ast context_scope, const char* name, size_t name_len,
                            slang_error* err) {
    if (!checkEntry(err))
        return noAst(context_scope.compilation);
    auto csym = symbolOf(context_scope);
    auto scope = csym ? scopeOf(*csym) : nullptr;
    if (!scope || !name) {
        setError(err, SLANG_ERR_INVALID_ARG, scope ? "null name" : "not a scope");
        return noAst(context_scope.compilation);
    }
    SLANG_C_GUARD(err, {
        SealLift lift(context_scope.compilation);
        Diagnostics localDiags;
        auto& nameSyntax =
            context_scope.compilation->comp->tryParseName(toView(name, name_len), localDiags);
        ASTContext context(*scope, LookupLocation::max);
        LookupResult result;
        Lookup::name(nameSyntax, context, LookupFlags::None, result);
        return capi::toC(result.found, context_scope.compilation);
    });
    return noAst(context_scope.compilation);
}

slang_ast slang_lookup_find_class(slang_ast context_scope, const char* name, size_t name_len,
                                  slang_error* err) {
    if (!checkEntry(err))
        return noAst(context_scope.compilation);
    auto csym = symbolOf(context_scope);
    auto scope = csym ? scopeOf(*csym) : nullptr;
    if (!scope || !name) {
        setError(err, SLANG_ERR_INVALID_ARG, scope ? "null name" : "not a scope");
        return noAst(context_scope.compilation);
    }
    SLANG_C_GUARD(err, {
        SealLift lift(context_scope.compilation);
        Diagnostics localDiags;
        auto& nameSyntax =
            context_scope.compilation->comp->tryParseName(toView(name, name_len), localDiags);
        ASTContext context(*scope, LookupLocation::max);
        auto classType = Lookup::findClass(nameSyntax, context);
        return capi::toC(classType, context_scope.compilation);
    });
    return noAst(context_scope.compilation);
}

bool slang_lookup_find_assertion_local_var(slang_ast assertion_inst, slang_ast context_scope,
                                           const char* name, size_t name_len,
                                           slang_ast* out_symbol, slang_error* err) {
    if (!checkEntry(err))
        return false;
    auto expr = exprOf(assertion_inst);
    auto csym = symbolOf(context_scope);
    auto scope = csym ? scopeOf(*csym) : nullptr;
    if (!expr || expr->kind != ExpressionKind::AssertionInstance || !scope || !name) {
        setError(err, SLANG_ERR_INVALID_ARG,
                 !expr || expr->kind != ExpressionKind::AssertionInstance
                     ? "not an AssertionInstance expression"
                 : !scope ? "not a scope"
                          : "null name");
        return false;
    }
    SLANG_C_GUARD(err, {
        SealLift lift(assertion_inst.compilation);
        auto& aie = expr->as<AssertionInstanceExpression>();

        // AssertionInstanceDetails is normally an ephemeral part of the
        // ASTContext chain built while binding the body of a sequence/
        // property instance, and is never stored on the frozen AST. Its
        // `localVars` map, though, is exactly the already-elaborated
        // (and freeze-sweep-forced, as part of the AssertionInstance
        // expression's own visitExprs) `localVars` span on the expression
        // node itself -- so rebuilding just that map here and calling the
        // real Lookup::findAssertionLocalVar against it is a faithful,
        // non-reinvented use of the real function, not a reimplementation
        // of its matching logic.
        AssertionInstanceDetails details;
        details.symbol = &aie.symbol;
        for (auto lv : aie.localVars)
            details.localVars[lv->name] = lv;

        ASTContext context(*scope, LookupLocation::max);
        context.assertionInstance = &details;

        Diagnostics localDiags;
        auto& nameSyntax =
            assertion_inst.compilation->comp->tryParseName(toView(name, name_len), localDiags);

        LookupResult result;
        bool found = Lookup::findAssertionLocalVar(context, nameSyntax, result);
        if (out_symbol)
            *out_symbol = capi::toC(result.found, assertion_inst.compilation);
        return found;
    });
    return false;
}

bool slang_lookup_find_temp_var(slang_ast temp_var, slang_ast context_scope, const char* name,
                                size_t name_len, slang_ast* out_symbol, slang_error* err) {
    if (!checkEntry(err))
        return false;
    auto tv = symbolOf(temp_var);
    auto csym = symbolOf(context_scope);
    auto scope = csym ? scopeOf(*csym) : nullptr;
    if (!tv || !TempVarSymbol::isKind(tv->kind) || !scope || !name) {
        setError(err, SLANG_ERR_INVALID_ARG,
                 !tv || !TempVarSymbol::isKind(tv->kind) ? "not a TempVarSymbol"
                 : !scope                                ? "not a scope"
                                                          : "null name");
        return false;
    }
    SLANG_C_GUARD(err, {
        SealLift lift(temp_var.compilation);
        Diagnostics localDiags;
        auto& nameSyntax =
            temp_var.compilation->comp->tryParseName(toView(name, name_len), localDiags);

        LookupResult result;
        bool found = Lookup::findTempVar(*scope, tv->as<TempVarSymbol>(), nameSyntax, result);
        if (out_symbol)
            *out_symbol = capi::toC(result.found, temp_var.compilation);
        return found;
    });
    return false;
}

// ---- LookupLocation -----------------------------------------------------

namespace {

// Converts a real slang::ast::LookupLocation into its C value-struct form,
// wrapping its (possibly null) scope against `comp`.
slang_lookup_location toC(LookupLocation loc, slang_compilation comp) {
    const Symbol* scopeSym = loc.getScope() ? &loc.getScope()->asSymbol() : nullptr;
    return slang_lookup_location{capi::toC(scopeSym, comp), (uint32_t)loc.getIndex()};
}

} // namespace

slang_lookup_location slang_lookup_location_before(slang_ast symbol) {
    auto dflt = slang_lookup_location{noAst(symbol.compilation), 0};
    SLANG_C_ACCESS(dflt, {
        auto sym = symbolOf(symbol);
        if (!sym)
            return dflt;
        return toC(LookupLocation::before(*sym), symbol.compilation);
    });
}

slang_lookup_location slang_lookup_location_after(slang_ast symbol) {
    auto dflt = slang_lookup_location{noAst(symbol.compilation), 0};
    SLANG_C_ACCESS(dflt, {
        auto sym = symbolOf(symbol);
        if (!sym)
            return dflt;
        return toC(LookupLocation::after(*sym), symbol.compilation);
    });
}

slang_lookup_location slang_lookup_location_max(slang_compilation comp) {
    auto dflt = slang_lookup_location{noAst(comp), 0};
    SLANG_C_ACCESS(dflt, { return toC(LookupLocation::max, comp); });
}

slang_lookup_location slang_lookup_location_min(slang_compilation comp) {
    auto dflt = slang_lookup_location{noAst(comp), 0};
    SLANG_C_ACCESS(dflt, { return toC(LookupLocation::min, comp); });
}

slang_ast slang_lookup_location_get_scope(slang_lookup_location loc) {
    SLANG_C_ACCESS(noAst(loc.scope.compilation), { return loc.scope; });
}

uint32_t slang_lookup_location_get_index(slang_lookup_location loc) {
    SLANG_C_ACCESS(0u, { return loc.index; });
}

// ---- LookupResult -------------------------------------------------------

slang_lookup_result slang_lookup_result_create(void) {
    SLANG_C_ACCESS(nullptr, { return new slang_lookup_result_t(); });
}

void slang_lookup_result_destroy(slang_lookup_result result) {
    delete result;
}

bool slang_lookup_within_class_randomize(slang_ast class_type, slang_ast this_var,
                                         slang_ast context_scope, const char* name,
                                         size_t name_len, slang_lookup_result result,
                                         slang_error* err) {
    if (!checkEntry(err))
        return false;
    auto classSym = symbolOf(class_type);
    auto classScope = classSym ? scopeOf(*classSym) : nullptr;
    auto ctxSym = symbolOf(context_scope);
    auto scope = ctxSym ? scopeOf(*ctxSym) : nullptr;
    if (!classScope || !scope || !name || !result) {
        setError(err, SLANG_ERR_INVALID_ARG,
                 !classScope ? "class_type is not a scope"
                 : !scope    ? "context_scope is not a scope"
                 : !result   ? "null result"
                             : "null name");
        return false;
    }
    SLANG_C_GUARD(err, {
        SealLift lift(class_type.compilation);
        Diagnostics localDiags;
        auto& nameSyntax =
            class_type.compilation->comp->tryParseName(toView(name, name_len), localDiags);

        ASTContext::RandomizeDetails details;
        details.classType = classScope;
        details.thisVar = symbolOf(this_var);

        ASTContext context(*scope, LookupLocation::max);
        context.randomizeDetails = &details;

        result->result.clear();
        result->comp = class_type.compilation;
        return Lookup::withinClassRandomize(context, nameSyntax, LookupFlags::None,
                                            result->result);
    });
    return false;
}

slang_ast slang_lookup_result_found(slang_lookup_result result) {
    SLANG_C_ACCESS(noAst(result ? result->comp : nullptr), {
        if (!result)
            return noAst(nullptr);
        return capi::toC(result->result.found, result->comp);
    });
}

uint32_t slang_lookup_result_flags(slang_lookup_result result) {
    SLANG_C_ACCESS(0u, {
        if (!result)
            return 0u;
        return (uint32_t)result->result.flags.bits();
    });
}

bool slang_lookup_result_has_error(slang_lookup_result result) {
    SLANG_C_ACCESS(false, {
        if (!result)
            return false;
        return result->result.hasError();
    });
}

slang_system_subroutine slang_lookup_result_system_subroutine(slang_lookup_result result) {
    SLANG_C_ACCESS(nullptr, {
        if (!result)
            return nullptr;
        return reinterpret_cast<slang_system_subroutine>(
            const_cast<SystemSubroutine*>(result->result.systemSubroutine));
    });
}

uint32_t slang_lookup_result_upward_count(slang_lookup_result result) {
    SLANG_C_ACCESS(0u, {
        if (!result)
            return 0u;
        return result->result.upwardCount;
    });
}

bool slang_lookup_result_clear(slang_lookup_result result) {
    if (!result)
        return false;
    SLANG_C_GUARD(nullptr, {
        result->result.clear();
        return true;
    });
    return false;
}

bool slang_lookup_result_error_if_selectors(slang_lookup_result result, slang_ast context_scope,
                                            slang_error* err) {
    if (!checkEntry(err))
        return false;
    auto ctxSym = symbolOf(context_scope);
    auto scope = ctxSym ? scopeOf(*ctxSym) : nullptr;
    if (!result || !scope) {
        setError(err, SLANG_ERR_INVALID_ARG, !result ? "null result" : "context_scope is not a scope");
        return false;
    }
    SLANG_C_GUARD(err, {
        SealLift lift(context_scope.compilation);
        ASTContext context(*scope, LookupLocation::max);
        result->result.errorIfSelectors(context);
        return true;
    });
    return false;
}

bool slang_lookup_result_report_diags(slang_lookup_result result, slang_ast context_scope,
                                      slang_error* err) {
    if (!checkEntry(err))
        return false;
    auto ctxSym = symbolOf(context_scope);
    auto scope = ctxSym ? scopeOf(*ctxSym) : nullptr;
    if (!result || !scope) {
        setError(err, SLANG_ERR_INVALID_ARG, !result ? "null result" : "context_scope is not a scope");
        return false;
    }
    SLANG_C_GUARD(err, {
        SealLift lift(context_scope.compilation);
        ASTContext context(*scope, LookupLocation::max);
        result->result.reportDiags(context);
        return true;
    });
    return false;
}

slang_diagnostics slang_lookup_result_diagnostics(slang_lookup_result result) {
    SLANG_C_ACCESS(nullptr, {
        auto out = new slang_diagnostics_t();
        if (!result)
            return out;
        out->sm = result->comp ? result->comp->sm : nullptr;
        out->comp = result->comp;
        auto& diags = result->result.getDiagnostics();
        out->diags.assign(diags.begin(), diags.end());
        return out;
    });
}

uint32_t slang_lookup_result_selector_count(slang_lookup_result result) {
    SLANG_C_ACCESS(0u, {
        if (!result)
            return 0u;
        return (uint32_t)result->result.selectors.size();
    });
}

bool slang_lookup_result_selector_is_member(slang_lookup_result result, uint32_t index) {
    SLANG_C_ACCESS(false, {
        if (!result || index >= result->result.selectors.size())
            return false;
        return result->result.selectors[index].index() == 1;
    });
}

slang_str slang_lookup_result_selector_name(slang_lookup_result result, uint32_t index) {
    SLANG_C_ACCESS(borrowed(""), {
        if (!result || index >= result->result.selectors.size())
            return borrowed("");
        auto& sel = result->result.selectors[index];
        if (sel.index() != 1)
            return borrowed("");
        return borrowed(std::get<1>(sel).name);
    });
}

slang_loc slang_lookup_result_selector_dot_location(slang_lookup_result result, uint32_t index) {
    SLANG_C_ACCESS(slang_loc{}, {
        if (!result || index >= result->result.selectors.size())
            return slang_loc{};
        auto& sel = result->result.selectors[index];
        if (sel.index() != 1)
            return slang_loc{};
        return capi::toC(std::get<1>(sel).dotLocation);
    });
}

slang_range slang_lookup_result_selector_name_range(slang_lookup_result result, uint32_t index) {
    SLANG_C_ACCESS(slang_range{}, {
        if (!result || index >= result->result.selectors.size())
            return slang_range{};
        auto& sel = result->result.selectors[index];
        if (sel.index() != 1)
            return slang_range{};
        return capi::toC(std::get<1>(sel).nameRange);
    });
}

slang_argument_direction slang_symbol_modport_port_direction(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_ARGUMENT_DIRECTION_IN, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ModportPort)
            return SLANG_ARGUMENT_DIRECTION_IN;
        return (slang_argument_direction)(uint32_t)s->as<ModportPortSymbol>().direction;
    });
}

slang_ast slang_symbol_modport_port_explicit_connection(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ModportPort)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        auto expr = s->as<ModportPortSymbol>().explicitConnection;
        return expr ? toC(expr, sym.compilation) : noAst(sym.compilation, SLANG_AST_EXPRESSION);
    });
}

slang_ast slang_symbol_modport_port_internal_symbol(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ModportPort)
            return noAst(sym.compilation);
        auto internal = s->as<ModportPortSymbol>().internalSymbol;
        return internal ? toC(internal, sym.compilation) : noAst(sym.compilation);
    });
}

bool slang_symbol_modport_has_exports(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Modport)
            return false;
        return s->as<ModportSymbol>().hasExports;
    });
}

slang_argument_direction slang_symbol_multi_port_direction(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_ARGUMENT_DIRECTION_IN, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::MultiPort)
            return SLANG_ARGUMENT_DIRECTION_IN;
        return (slang_argument_direction)(uint32_t)s->as<MultiPortSymbol>().direction;
    });
}

slang_ast slang_symbol_multi_port_initializer(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::MultiPort)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        // MultiPortSymbol::getInitializer is a fixed placeholder that always
        // returns nullptr (multi-ports never have initializers), kept only
        // for parity with the single-port PortSymbol interface.
        return noAst(sym.compilation, SLANG_AST_EXPRESSION);
    });
}

slang_ast slang_symbol_multi_port_type(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::MultiPort)
            return noAst(sym.compilation);
        // Forced by the freeze sweep (FreezeVisitor's MultiPortSymbol
        // branch: `t.getType()`), so this is a pure read.
        return toC(&s->as<MultiPortSymbol>().getType(), sym.compilation);
    });
}

bool slang_symbol_multi_port_is_null_port(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::MultiPort)
            return false;
        return s->as<MultiPortSymbol>().isNullPort;
    });
}

uint32_t slang_symbol_multi_port_port_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::MultiPort)
            return 0u;
        return (uint32_t)s->as<MultiPortSymbol>().ports.size();
    });
}

slang_ast slang_symbol_multi_port_port(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::MultiPort)
            return noAst(sym.compilation);
        auto ports = s->as<MultiPortSymbol>().ports;
        if (index >= ports.size())
            return noAst(sym.compilation);
        return toC(ports[index], sym.compilation);
    });
}

uint32_t slang_symbol_net_alias_reference_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::NetAlias)
            return 0u;
        // Forced by the freeze sweep (FreezeVisitor's NetAliasSymbol
        // branch), so this is a pure read.
        return (uint32_t)s->as<NetAliasSymbol>().getNetReferences().size();
    });
}

slang_ast slang_symbol_net_alias_reference(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::NetAlias)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        // Forced by the freeze sweep; a pure read.
        auto refs = s->as<NetAliasSymbol>().getNetReferences();
        if (index >= refs.size())
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        return toC(refs[index], sym.compilation);
    });
}

slang_expansion_hint slang_symbol_net_expansion_hint(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_EXPANSION_HINT_NONE, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Net)
            return SLANG_EXPANSION_HINT_NONE;
        return (slang_expansion_hint)(uint32_t)s->as<NetSymbol>().expansionHint;
    });
}

slang_charge_strength slang_symbol_net_charge_strength(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_CHARGE_STRENGTH_NONE, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Net)
            return SLANG_CHARGE_STRENGTH_NONE;
        // Recomputed from syntax on every call; no arena allocation.
        auto cs = s->as<NetSymbol>().getChargeStrength();
        return cs ? (slang_charge_strength)(uint32_t)*cs : SLANG_CHARGE_STRENGTH_NONE;
    });
}

slang_ast slang_symbol_net_delay(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_TIMING_CONTROL), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Net)
            return noAst(sym.compilation, SLANG_AST_TIMING_CONTROL);
        // Forced by the freeze sweep (FreezeVisitor's NetSymbol branch:
        // `t.getDelay()`), so this is a pure read.
        auto delay = s->as<NetSymbol>().getDelay();
        return delay ? wrapAst(*delay, sym.compilation)
                     : noAst(sym.compilation, SLANG_AST_TIMING_CONTROL);
    });
}

slang_drive_strength_pair slang_symbol_net_drive_strength(slang_ast sym) {
    SLANG_C_ACCESS(slang_drive_strength_pair{}, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Net)
            return slang_drive_strength_pair{};
        // Recomputed from syntax on every call; no arena allocation.
        auto [ds0, ds1] = s->as<NetSymbol>().getDriveStrength();
        slang_drive_strength_pair result{};
        if (ds0) {
            result.has_strength0 = true;
            result.strength0 = (slang_drive_strength)(uint32_t)*ds0;
        }
        if (ds1) {
            result.has_strength1 = true;
            result.strength1 = (slang_drive_strength)(uint32_t)*ds1;
        }
        return result;
    });
}

bool slang_symbol_net_is_implicit(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Net)
            return false;
        return s->as<NetSymbol>().isImplicit;
    });
}

slang_variable_lifetime slang_symbol_package_default_lifetime(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_VARIABLE_LIFETIME_AUTOMATIC, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Package)
            return SLANG_VARIABLE_LIFETIME_AUTOMATIC;
        return (slang_variable_lifetime)(uint32_t)s->as<PackageSymbol>().defaultLifetime;
    });
}

slang_ast slang_symbol_package_find_for_import(slang_ast sym, const char* name, size_t name_len) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Package || !name)
            return noAst(sym.compilation);
        // The export-resolution memo (PackageSymbol's private
        // ExportData::resolved) is forced for every Package symbol by the
        // freeze sweep (FreezeVisitor's PackageSymbol branch:
        // `t.resolveExports()`), so this is a pure read: findForImport
        // itself only walks already-elaborated name maps.
        auto found = s->as<PackageSymbol>().findForImport(toView(name, name_len));
        return toC(found, sym.compilation);
    });
}

bool slang_symbol_package_has_export_all(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Package)
            return false;
        return s->as<PackageSymbol>().hasExportAll;
    });
}

bool slang_symbol_package_time_scale(slang_ast sym, slang_time_scale* out) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Package)
            return false;
        auto& ts = s->as<PackageSymbol>().timeScale;
        if (!ts)
            return false;
        if (out) {
            *out = slang_time_scale{
                (uint8_t)ts->base.unit,
                (uint8_t)ts->base.magnitude,
                (uint8_t)ts->precision.unit,
                (uint8_t)ts->precision.magnitude,
            };
        }
        return true;
    });
}

// ---- ParameterSymbolBase / ParameterSymbol -----------------------------------

namespace {

const ParameterSymbolBase* parameterBaseOf(const Symbol* s) {
    if (!s)
        return nullptr;
    if (s->kind == SymbolKind::Parameter)
        return &s->as<ParameterSymbol>();
    if (s->kind == SymbolKind::TypeParameter)
        return &s->as<TypeParameterSymbol>();
    return nullptr;
}

} // namespace

bool slang_symbol_parameter_is_local_param(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto base = parameterBaseOf(symbolOf(sym));
        return base && base->isLocalParam();
    });
}

bool slang_symbol_parameter_is_port_param(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto base = parameterBaseOf(symbolOf(sym));
        return base && base->isPortParam();
    });
}

bool slang_symbol_parameter_is_body_param(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto base = parameterBaseOf(symbolOf(sym));
        return base && base->isBodyParam();
    });
}

bool slang_symbol_parameter_is_overridden(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Parameter)
            return false;
        // Reads a DeclaredTypeFlags bit resolved by the same getType() the
        // freeze sweep already forces generically for every symbol with a
        // declared type -- a pure read.
        return s->as<ParameterSymbol>().isOverridden();
    });
}

bool slang_symbol_type_parameter_is_overridden(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::TypeParameter)
            return false;
        // Reads a DeclaredTypeFlags bit resolved by the same getType() the
        // freeze sweep already forces generically for every symbol with a
        // declared type -- a pure read.
        return s->as<TypeParameterSymbol>().isOverridden();
    });
}

// ---- PortSymbol ---------------------------------------------------------------

slang_argument_direction slang_symbol_port_direction(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_ARGUMENT_DIRECTION_INOUT, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Port)
            return SLANG_ARGUMENT_DIRECTION_INOUT;
        return (slang_argument_direction)(uint32_t)s->as<PortSymbol>().direction;
    });
}

slang_loc slang_symbol_port_external_loc(slang_ast sym) {
    SLANG_C_ACCESS(slang_loc{}, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Port)
            return slang_loc{};
        return toC(s->as<PortSymbol>().externalLoc);
    });
}

slang_ast slang_symbol_port_initializer(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Port)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        // Forced by the freeze sweep (FreezeVisitor's PortSymbol branch:
        // `t.getInitializer()`), so this is a pure read.
        auto init = s->as<PortSymbol>().getInitializer();
        return init ? wrapAst(*init, sym.compilation) : noAst(sym.compilation, SLANG_AST_EXPRESSION);
    });
}

slang_ast slang_symbol_port_internal_expr(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Port)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        // Forced by the freeze sweep (FreezeVisitor's PortSymbol branch:
        // `t.getInternalExpr()`), so this is a pure read.
        auto expr = s->as<PortSymbol>().getInternalExpr();
        return expr ? wrapAst(*expr, sym.compilation) : noAst(sym.compilation, SLANG_AST_EXPRESSION);
    });
}

slang_ast slang_symbol_port_type(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Port)
            return noAst(sym.compilation);
        // Forced by the freeze sweep (FreezeVisitor's PortSymbol branch:
        // `t.getType()`), so this is a pure read.
        return toC(&s->as<PortSymbol>().getType(), sym.compilation);
    });
}

slang_ast slang_symbol_port_internal_symbol(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Port)
            return noAst(sym.compilation);
        return toC(s->as<PortSymbol>().internalSymbol, sym.compilation);
    });
}

bool slang_symbol_port_is_ansi_port(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Port)
            return false;
        return s->as<PortSymbol>().isAnsiPort;
    });
}

bool slang_symbol_port_is_net_port(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Port)
            return false;
        // isNetPort() only reads the internalExpr memo already forced by
        // the freeze sweep (FreezeVisitor's PortSymbol branch), so this is
        // a pure read.
        return s->as<PortSymbol>().isNetPort();
    });
}

bool slang_symbol_port_is_null_port(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Port)
            return false;
        return s->as<PortSymbol>().isNullPort;
    });
}

// ---- PrimitiveInstanceSymbol / PrimitivePortSymbol / PrimitiveSymbol --------

slang_primitive_port_direction slang_symbol_primitive_port_direction(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_PRIMITIVE_PORT_DIRECTION_IN, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::PrimitivePort)
            return SLANG_PRIMITIVE_PORT_DIRECTION_IN;
        return (slang_primitive_port_direction)(uint32_t)s->as<PrimitivePortSymbol>().direction;
    });
}

slang_primitive_kind slang_symbol_primitive_kind(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_PRIMITIVE_KIND_USER_DEFINED, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Primitive)
            return SLANG_PRIMITIVE_KIND_USER_DEFINED;
        return (slang_primitive_kind)(uint32_t)s->as<PrimitiveSymbol>().primitiveKind;
    });
}

bool slang_symbol_primitive_is_sequential(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Primitive)
            return false;
        return s->as<PrimitiveSymbol>().isSequential;
    });
}

uint32_t slang_symbol_primitive_port_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Primitive)
            return 0u;
        return (uint32_t)s->as<PrimitiveSymbol>().ports.size();
    });
}

slang_ast slang_symbol_primitive_port(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Primitive)
            return noAst(sym.compilation);
        auto ports = s->as<PrimitiveSymbol>().ports;
        if (index >= ports.size())
            return noAst(sym.compilation);
        return toC(ports[index], sym.compilation);
    });
}

namespace {

// Shared bounds-check + row lookup for the
// slang_symbol_primitive_table_entry_* family below.
const PrimitiveSymbol::TableEntry* primitiveTableEntry(slang_ast sym, uint32_t index) {
    auto s = symbolOf(sym);
    if (!s || s->kind != SymbolKind::Primitive)
        return nullptr;
    auto table = s->as<PrimitiveSymbol>().table;
    if (index >= table.size())
        return nullptr;
    return &table[index];
}

} // namespace

uint32_t slang_symbol_primitive_table_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Primitive)
            return 0u;
        return (uint32_t)s->as<PrimitiveSymbol>().table.size();
    });
}

slang_str slang_symbol_primitive_table_entry_inputs(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(borrowed(""), {
        auto row = primitiveTableEntry(sym, index);
        return row ? borrowed(row->inputs) : borrowed("");
    });
}

slang_str slang_symbol_primitive_table_entry_output(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(borrowed(""), {
        auto row = primitiveTableEntry(sym, index);
        if (!row || !row->output)
            return borrowed("");
        // Points directly at the row's own `output` field, stable for as
        // long as the frozen arena that owns the table span is.
        return borrowed(std::string_view(&row->output, 1));
    });
}

slang_str slang_symbol_primitive_table_entry_state(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(borrowed(""), {
        auto row = primitiveTableEntry(sym, index);
        if (!row || !row->state)
            return borrowed("");
        // Points directly at the row's own `state` field, stable for as
        // long as the frozen arena that owns the table span is.
        return borrowed(std::string_view(&row->state, 1));
    });
}

uint32_t slang_symbol_primitive_instance_port_connection_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::PrimitiveInstance)
            return 0u;
        // Forced by the freeze sweep (FreezeVisitor's PrimitiveInstanceSymbol
        // branch: `t.getPortConnections()`), so this is a pure read.
        return (uint32_t)s->as<PrimitiveInstanceSymbol>().getPortConnections().size();
    });
}

slang_ast slang_symbol_primitive_instance_port_connection(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::PrimitiveInstance)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        auto conns = s->as<PrimitiveInstanceSymbol>().getPortConnections();
        if (index >= conns.size() || !conns[index])
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        return toC(conns[index], sym.compilation);
    });
}

slang_ast slang_symbol_primitive_instance_delay(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_TIMING_CONTROL), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::PrimitiveInstance)
            return noAst(sym.compilation, SLANG_AST_TIMING_CONTROL);
        // Forced by the freeze sweep (FreezeVisitor's PrimitiveInstanceSymbol
        // branch: `t.getDelay()`), so this is a pure read.
        auto delay = s->as<PrimitiveInstanceSymbol>().getDelay();
        return delay ? wrapAst(*delay, sym.compilation)
                     : noAst(sym.compilation, SLANG_AST_TIMING_CONTROL);
    });
}

slang_drive_strength_pair slang_symbol_primitive_instance_drive_strength(slang_ast sym) {
    SLANG_C_ACCESS(slang_drive_strength_pair{}, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::PrimitiveInstance)
            return slang_drive_strength_pair{};
        // Recomputed from syntax on every call; no arena allocation.
        auto [ds0, ds1] = s->as<PrimitiveInstanceSymbol>().getDriveStrength();
        slang_drive_strength_pair result{};
        if (ds0) {
            result.has_strength0 = true;
            result.strength0 = (slang_drive_strength)(uint32_t)*ds0;
        }
        if (ds1) {
            result.has_strength1 = true;
            result.strength1 = (slang_drive_strength)(uint32_t)*ds1;
        }
        return result;
    });
}

// ---- ProceduralBlockSymbol ---------------------------------------------------

uint32_t slang_symbol_procedural_block_block_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ProceduralBlock)
            return 0u;
        return (uint32_t)s->as<ProceduralBlockSymbol>().getBlocks().size();
    });
}

slang_ast slang_symbol_procedural_block_block(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ProceduralBlock)
            return noAst(sym.compilation);
        auto blocks = s->as<ProceduralBlockSymbol>().getBlocks();
        if (index >= blocks.size())
            return noAst(sym.compilation);
        return toC(blocks[index], sym.compilation);
    });
}

bool slang_symbol_procedural_block_is_single_driver_block(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ProceduralBlock)
            return false;
        return s->as<ProceduralBlockSymbol>().isSingleDriverBlock();
    });
}

slang_procedural_block_kind slang_symbol_procedural_block_procedure_kind(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_PROCEDURAL_BLOCK_KIND_INITIAL, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::ProceduralBlock)
            return SLANG_PROCEDURAL_BLOCK_KIND_INITIAL;
        return (slang_procedural_block_kind)(
            uint32_t)s->as<ProceduralBlockSymbol>().procedureKind;
    });
}

// ---- PropertySymbol -----------------------------------------------------------

uint32_t slang_symbol_property_port_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Property)
            return 0u;
        return (uint32_t)s->as<PropertySymbol>().ports.size();
    });
}

slang_ast slang_symbol_property_port(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Property)
            return noAst(sym.compilation);
        auto ports = s->as<PropertySymbol>().ports;
        if (index >= ports.size())
            return noAst(sym.compilation);
        return toC(ports[index], sym.compilation);
    });
}

// ---- TimingPathSymbol ----------------------------------------------------------

slang_timing_path_connection_kind slang_symbol_timing_path_connection_kind(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_TIMING_PATH_CONNECTION_KIND_FULL, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::TimingPath)
            return SLANG_TIMING_PATH_CONNECTION_KIND_FULL;
        return (slang_timing_path_connection_kind)(uint32_t)
            s->as<TimingPathSymbol>().connectionKind;
    });
}

slang_timing_path_polarity slang_symbol_timing_path_polarity(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_TIMING_PATH_POLARITY_UNKNOWN, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::TimingPath)
            return SLANG_TIMING_PATH_POLARITY_UNKNOWN;
        return (slang_timing_path_polarity)(uint32_t)s->as<TimingPathSymbol>().polarity;
    });
}

slang_timing_path_polarity slang_symbol_timing_path_edge_polarity(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_TIMING_PATH_POLARITY_UNKNOWN, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::TimingPath)
            return SLANG_TIMING_PATH_POLARITY_UNKNOWN;
        return (slang_timing_path_polarity)(uint32_t)s->as<TimingPathSymbol>().edgePolarity;
    });
}

slang_edge_kind slang_symbol_timing_path_edge_identifier(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_EDGE_NONE, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::TimingPath)
            return SLANG_EDGE_NONE;
        return (slang_edge_kind)(uint32_t)s->as<TimingPathSymbol>().edgeIdentifier;
    });
}

bool slang_symbol_timing_path_is_state_dependent(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::TimingPath)
            return false;
        return s->as<TimingPathSymbol>().isStateDependent;
    });
}

slang_ast slang_symbol_timing_path_condition_expr(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::TimingPath)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        // Forced by the freeze sweep (FreezeVisitor's TimingPathSymbol
        // branch: `t.getInputs()`, which shares the same resolve()).
        return toC(s->as<TimingPathSymbol>().getConditionExpr(), sym.compilation);
    });
}

slang_ast slang_symbol_timing_path_edge_source_expr(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::TimingPath)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        // Forced by the freeze sweep, same as above.
        return toC(s->as<TimingPathSymbol>().getEdgeSourceExpr(), sym.compilation);
    });
}

uint32_t slang_symbol_timing_path_input_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::TimingPath)
            return 0u;
        // Forced by the freeze sweep, same as above.
        return (uint32_t)s->as<TimingPathSymbol>().getInputs().size();
    });
}

slang_ast slang_symbol_timing_path_input(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::TimingPath)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        auto inputs = s->as<TimingPathSymbol>().getInputs();
        if (index >= inputs.size())
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        return toC(inputs[index], sym.compilation);
    });
}

uint32_t slang_symbol_timing_path_output_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::TimingPath)
            return 0u;
        // Forced by the freeze sweep, same as above.
        return (uint32_t)s->as<TimingPathSymbol>().getOutputs().size();
    });
}

slang_ast slang_symbol_timing_path_output(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::TimingPath)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        auto outputs = s->as<TimingPathSymbol>().getOutputs();
        if (index >= outputs.size())
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        return toC(outputs[index], sym.compilation);
    });
}

uint32_t slang_symbol_timing_path_delay_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::TimingPath)
            return 0u;
        // Forced by the freeze sweep, same as above.
        return (uint32_t)s->as<TimingPathSymbol>().getDelays().size();
    });
}

slang_ast slang_symbol_timing_path_delay(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::TimingPath)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        auto delays = s->as<TimingPathSymbol>().getDelays();
        if (index >= delays.size())
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        return toC(delays[index], sym.compilation);
    });
}

// ---- PulseStyleSymbol ----------------------------------------------------------

slang_pulse_style_kind slang_symbol_pulse_style_kind(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_PULSE_STYLE_KIND_ON_EVENT, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::PulseStyle)
            return SLANG_PULSE_STYLE_KIND_ON_EVENT;
        return (slang_pulse_style_kind)(uint32_t)s->as<PulseStyleSymbol>().pulseStyleKind;
    });
}

uint32_t slang_symbol_pulse_style_terminal_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::PulseStyle)
            return 0u;
        // Forced by the freeze sweep (FreezeVisitor's PulseStyleSymbol
        // branch: `t.getTerminals()`), so this is a pure read.
        return (uint32_t)s->as<PulseStyleSymbol>().getTerminals().size();
    });
}

slang_ast slang_symbol_pulse_style_terminal(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::PulseStyle)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        auto terminals = s->as<PulseStyleSymbol>().getTerminals();
        if (index >= terminals.size() || !terminals[index])
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        return toC(terminals[index], sym.compilation);
    });
}

// ---- SystemTimingCheckSymbol -----------------------------------------------

slang_system_timing_check_kind slang_symbol_system_timing_check_kind(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_SYSTEM_TIMING_CHECK_KIND_UNKNOWN, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::SystemTimingCheck)
            return SLANG_SYSTEM_TIMING_CHECK_KIND_UNKNOWN;
        return (slang_system_timing_check_kind)(uint32_t)
            s->as<SystemTimingCheckSymbol>().timingCheckKind;
    });
}

uint32_t slang_symbol_system_timing_check_argument_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::SystemTimingCheck)
            return 0u;
        // Forced by the freeze sweep (FreezeVisitor's SystemTimingCheckSymbol
        // branch: `t.getArguments()`), so this is a pure read.
        return (uint32_t)s->as<SystemTimingCheckSymbol>().getArguments().size();
    });
}

slang_ast slang_symbol_system_timing_check_argument_expr(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::SystemTimingCheck)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        auto args = s->as<SystemTimingCheckSymbol>().getArguments();
        if (index >= args.size() || !args[index].expr)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        return toC(args[index].expr, sym.compilation);
    });
}

slang_ast slang_symbol_system_timing_check_argument_condition(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::SystemTimingCheck)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        auto args = s->as<SystemTimingCheckSymbol>().getArguments();
        if (index >= args.size() || !args[index].condition)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        return toC(args[index].condition, sym.compilation);
    });
}

slang_edge_kind slang_symbol_system_timing_check_argument_edge(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(SLANG_EDGE_NONE, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::SystemTimingCheck)
            return SLANG_EDGE_NONE;
        auto args = s->as<SystemTimingCheckSymbol>().getArguments();
        if (index >= args.size())
            return SLANG_EDGE_NONE;
        return (slang_edge_kind)(uint32_t)args[index].edge;
    });
}

uint32_t slang_symbol_system_timing_check_argument_edge_descriptor_count(slang_ast sym,
                                                                         uint32_t arg_index) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::SystemTimingCheck)
            return 0u;
        auto args = s->as<SystemTimingCheckSymbol>().getArguments();
        if (arg_index >= args.size())
            return 0u;
        return (uint32_t)args[arg_index].edgeDescriptors.size();
    });
}

slang_str slang_symbol_system_timing_check_argument_edge_descriptor(slang_ast sym,
                                                                     uint32_t arg_index,
                                                                     uint32_t desc_index) {
    SLANG_C_ACCESS(borrowed(""), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::SystemTimingCheck)
            return borrowed("");
        auto args = s->as<SystemTimingCheckSymbol>().getArguments();
        if (arg_index >= args.size())
            return borrowed("");
        auto descs = args[arg_index].edgeDescriptors;
        if (desc_index >= descs.size())
            return borrowed("");
        auto& desc = descs[desc_index];
        return borrowed(std::string_view(desc.data(), desc.size()));
    });
}

// ---- RandSeqProductionSymbol ---------------------------------------------------

namespace {

using ProdBase = RandSeqProductionSymbol::ProdBase;
using ProdKind = RandSeqProductionSymbol::ProdKind;
using ProdItem = RandSeqProductionSymbol::ProdItem;
using CodeBlockProd = RandSeqProductionSymbol::CodeBlockProd;
using IfElseProd = RandSeqProductionSymbol::IfElseProd;
using RepeatProd = RandSeqProductionSymbol::RepeatProd;
using CaseProd = RandSeqProductionSymbol::CaseProd;
using CaseItem = RandSeqProductionSymbol::CaseItem;
using Rule = RandSeqProductionSymbol::Rule;

slang_randseq_prod noProd(slang_compilation comp) {
    return slang_randseq_prod{nullptr, comp, 0, 0};
}

slang_randseq_prod toC(const ProdBase* prod, slang_compilation comp) {
    if (!prod)
        return noProd(comp);
    return slang_randseq_prod{prod, comp, (uint32_t)prod->kind, 0};
}

slang_randseq_prod toC(const ProdItem& item, slang_compilation comp) {
    return toC(static_cast<const ProdBase*>(&item), comp);
}

const ProdBase* randSeqProdOf(slang_randseq_prod prod) {
    return static_cast<const ProdBase*>(prod.ptr);
}

// Bounds-checked (rule_index, prod_index) -> ProdBase* lookup, shared by
// slang_symbol_randseq_rule_prod_count and slang_symbol_randseq_rule_prod.
std::span<const ProdBase* const> randSeqRuleProds(slang_ast sym, uint32_t ruleIndex) {
    auto s = symbolOf(sym);
    if (!s || s->kind != SymbolKind::RandSeqProduction)
        return {};
    // Forced by the freeze sweep (FreezeVisitor's RandSeqProductionSymbol
    // branch: `t.getRules()`), so this is a pure read.
    auto rules = s->as<RandSeqProductionSymbol>().getRules();
    if (ruleIndex >= rules.size())
        return {};
    return rules[ruleIndex].prods;
}

// Bounds-checked rule_index -> Rule* lookup, shared by every
// slang_symbol_randseq_rule_* field accessor below.
const Rule* randSeqRule(slang_ast sym, uint32_t ruleIndex) {
    auto s = symbolOf(sym);
    if (!s || s->kind != SymbolKind::RandSeqProduction)
        return nullptr;
    // Forced by the freeze sweep (FreezeVisitor's RandSeqProductionSymbol
    // branch: `t.getRules()`), so this is a pure read.
    auto rules = s->as<RandSeqProductionSymbol>().getRules();
    if (ruleIndex >= rules.size())
        return nullptr;
    return &rules[ruleIndex];
}

// Bounds-checked kind-gated cast, shared by every Item-kind randseq-prod
// accessor below.
const ProdItem* asItem(slang_randseq_prod prod) {
    auto base = randSeqProdOf(prod);
    if (!base || base->kind != ProdKind::Item)
        return nullptr;
    return static_cast<const ProdItem*>(base);
}

const CodeBlockProd* asCodeBlock(slang_randseq_prod prod) {
    auto base = randSeqProdOf(prod);
    if (!base || base->kind != ProdKind::CodeBlock)
        return nullptr;
    return static_cast<const CodeBlockProd*>(base);
}

const IfElseProd* asIfElse(slang_randseq_prod prod) {
    auto base = randSeqProdOf(prod);
    if (!base || base->kind != ProdKind::IfElse)
        return nullptr;
    return static_cast<const IfElseProd*>(base);
}

const CaseProd* asCase(slang_randseq_prod prod) {
    auto base = randSeqProdOf(prod);
    if (!base || base->kind != ProdKind::Case)
        return nullptr;
    return static_cast<const CaseProd*>(base);
}

const RepeatProd* asRepeat(slang_randseq_prod prod) {
    auto base = randSeqProdOf(prod);
    if (!base || base->kind != ProdKind::Repeat)
        return nullptr;
    return static_cast<const RepeatProd*>(base);
}

} // namespace

bool slang_randseq_prod_is_null(slang_randseq_prod prod) {
    return prod.ptr == nullptr;
}

uint32_t slang_symbol_randseq_rule_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::RandSeqProduction)
            return 0u;
        // Forced by the freeze sweep, so this is a pure read.
        return (uint32_t)s->as<RandSeqProductionSymbol>().getRules().size();
    });
}

uint32_t slang_symbol_randseq_rule_prod_count(slang_ast sym, uint32_t rule_index) {
    SLANG_C_ACCESS(0u, { return (uint32_t)randSeqRuleProds(sym, rule_index).size(); });
}

slang_randseq_prod slang_symbol_randseq_rule_prod(slang_ast sym, uint32_t rule_index,
                                                   uint32_t prod_index) {
    SLANG_C_ACCESS(noProd(sym.compilation), {
        auto prods = randSeqRuleProds(sym, rule_index);
        if (prod_index >= prods.size())
            return noProd(sym.compilation);
        return toC(prods[prod_index], sym.compilation);
    });
}

slang_randseq_prod_kind slang_randseq_prod_get_kind(slang_randseq_prod prod) {
    SLANG_C_ACCESS(SLANG_RANDSEQ_PROD_KIND_ITEM, {
        auto base = randSeqProdOf(prod);
        if (!base)
            return SLANG_RANDSEQ_PROD_KIND_ITEM;
        return (slang_randseq_prod_kind)(uint32_t)base->kind;
    });
}

slang_ast slang_randseq_prod_item_target(slang_randseq_prod prod) {
    SLANG_C_ACCESS(noAst(prod.compilation), {
        auto item = asItem(prod);
        if (!item || !item->target)
            return noAst(prod.compilation);
        return toC(item->target, prod.compilation);
    });
}

uint32_t slang_randseq_prod_item_arg_count(slang_randseq_prod prod) {
    SLANG_C_ACCESS(0u, {
        auto item = asItem(prod);
        return item ? (uint32_t)item->args.size() : 0u;
    });
}

slang_ast slang_randseq_prod_item_arg(slang_randseq_prod prod, uint32_t index) {
    SLANG_C_ACCESS(noAst(prod.compilation, SLANG_AST_EXPRESSION), {
        auto item = asItem(prod);
        if (!item || index >= item->args.size() || !item->args[index])
            return noAst(prod.compilation, SLANG_AST_EXPRESSION);
        return toC(item->args[index], prod.compilation);
    });
}

slang_ast slang_randseq_prod_code_block_block(slang_randseq_prod prod) {
    SLANG_C_ACCESS(noAst(prod.compilation), {
        auto cb = asCodeBlock(prod);
        if (!cb)
            return noAst(prod.compilation);
        return toC(cb->block.get(), prod.compilation);
    });
}

slang_ast slang_randseq_prod_if_else_expr(slang_randseq_prod prod) {
    SLANG_C_ACCESS(noAst(prod.compilation, SLANG_AST_EXPRESSION), {
        auto ie = asIfElse(prod);
        if (!ie)
            return noAst(prod.compilation, SLANG_AST_EXPRESSION);
        return toC(ie->expr.get(), prod.compilation);
    });
}

slang_randseq_prod slang_randseq_prod_if_else_if_item(slang_randseq_prod prod) {
    SLANG_C_ACCESS(noProd(prod.compilation), {
        auto ie = asIfElse(prod);
        if (!ie)
            return noProd(prod.compilation);
        return toC(ie->ifItem, prod.compilation);
    });
}

bool slang_randseq_prod_if_else_has_else_item(slang_randseq_prod prod) {
    SLANG_C_ACCESS(false, {
        auto ie = asIfElse(prod);
        return ie && ie->elseItem.has_value();
    });
}

slang_randseq_prod slang_randseq_prod_if_else_else_item(slang_randseq_prod prod) {
    SLANG_C_ACCESS(noProd(prod.compilation), {
        auto ie = asIfElse(prod);
        if (!ie || !ie->elseItem)
            return noProd(prod.compilation);
        return toC(*ie->elseItem, prod.compilation);
    });
}

slang_ast slang_randseq_prod_repeat_expr(slang_randseq_prod prod) {
    SLANG_C_ACCESS(noAst(prod.compilation, SLANG_AST_EXPRESSION), {
        auto rp = asRepeat(prod);
        if (!rp)
            return noAst(prod.compilation, SLANG_AST_EXPRESSION);
        return toC(rp->expr.get(), prod.compilation);
    });
}

slang_randseq_prod slang_randseq_prod_repeat_item(slang_randseq_prod prod) {
    SLANG_C_ACCESS(noProd(prod.compilation), {
        auto rp = asRepeat(prod);
        if (!rp)
            return noProd(prod.compilation);
        return toC(rp->item, prod.compilation);
    });
}

slang_ast slang_randseq_prod_case_expr(slang_randseq_prod prod) {
    SLANG_C_ACCESS(noAst(prod.compilation, SLANG_AST_EXPRESSION), {
        auto cp = asCase(prod);
        if (!cp)
            return noAst(prod.compilation, SLANG_AST_EXPRESSION);
        return toC(cp->expr.get(), prod.compilation);
    });
}

uint32_t slang_randseq_prod_case_item_count(slang_randseq_prod prod) {
    SLANG_C_ACCESS(0u, {
        auto cp = asCase(prod);
        return cp ? (uint32_t)cp->items.size() : 0u;
    });
}

uint32_t slang_randseq_prod_case_item_expression_count(slang_randseq_prod prod,
                                                        uint32_t item_index) {
    SLANG_C_ACCESS(0u, {
        auto cp = asCase(prod);
        if (!cp || item_index >= cp->items.size())
            return 0u;
        return (uint32_t)cp->items[item_index].expressions.size();
    });
}

slang_ast slang_randseq_prod_case_item_expression(slang_randseq_prod prod, uint32_t item_index,
                                                  uint32_t expr_index) {
    SLANG_C_ACCESS(noAst(prod.compilation, SLANG_AST_EXPRESSION), {
        auto cp = asCase(prod);
        if (!cp || item_index >= cp->items.size())
            return noAst(prod.compilation, SLANG_AST_EXPRESSION);
        auto exprs = cp->items[item_index].expressions;
        if (expr_index >= exprs.size() || !exprs[expr_index])
            return noAst(prod.compilation, SLANG_AST_EXPRESSION);
        return toC(exprs[expr_index], prod.compilation);
    });
}

slang_randseq_prod slang_randseq_prod_case_item_item(slang_randseq_prod prod,
                                                     uint32_t item_index) {
    SLANG_C_ACCESS(noProd(prod.compilation), {
        auto cp = asCase(prod);
        if (!cp || item_index >= cp->items.size())
            return noProd(prod.compilation);
        return toC(cp->items[item_index].item, prod.compilation);
    });
}

bool slang_randseq_prod_case_has_default_item(slang_randseq_prod prod) {
    SLANG_C_ACCESS(false, {
        auto cp = asCase(prod);
        return cp && cp->defaultItem.has_value();
    });
}

slang_randseq_prod slang_randseq_prod_case_default_item(slang_randseq_prod prod) {
    SLANG_C_ACCESS(noProd(prod.compilation), {
        auto cp = asCase(prod);
        if (!cp || !cp->defaultItem)
            return noProd(prod.compilation);
        return toC(*cp->defaultItem, prod.compilation);
    });
}

slang_ast slang_symbol_randseq_rule_block(slang_ast sym, uint32_t rule_index) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto rule = randSeqRule(sym, rule_index);
        if (!rule)
            return noAst(sym.compilation);
        return toC(rule->ruleBlock.get(), sym.compilation);
    });
}

slang_ast slang_symbol_randseq_rule_weight_expr(slang_ast sym, uint32_t rule_index) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto rule = randSeqRule(sym, rule_index);
        if (!rule)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        return toC(rule->weightExpr, sym.compilation);
    });
}

bool slang_symbol_randseq_rule_is_rand_join(slang_ast sym, uint32_t rule_index) {
    SLANG_C_ACCESS(false, {
        auto rule = randSeqRule(sym, rule_index);
        return rule && rule->isRandJoin;
    });
}

slang_ast slang_symbol_randseq_rule_rand_join_expr(slang_ast sym, uint32_t rule_index) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto rule = randSeqRule(sym, rule_index);
        if (!rule)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        return toC(rule->randJoinExpr, sym.compilation);
    });
}

bool slang_symbol_randseq_rule_has_code_block(slang_ast sym, uint32_t rule_index) {
    SLANG_C_ACCESS(false, {
        auto rule = randSeqRule(sym, rule_index);
        return rule && rule->codeBlock.has_value();
    });
}

slang_randseq_prod slang_symbol_randseq_rule_code_block(slang_ast sym, uint32_t rule_index) {
    SLANG_C_ACCESS(noProd(sym.compilation), {
        auto rule = randSeqRule(sym, rule_index);
        if (!rule || !rule->codeBlock)
            return noProd(sym.compilation);
        return toC(static_cast<const ProdBase*>(&*rule->codeBlock), sym.compilation);
    });
}

uint32_t slang_symbol_randseq_argument_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::RandSeqProduction)
            return 0u;
        return (uint32_t)s->as<RandSeqProductionSymbol>().arguments.size();
    });
}

slang_ast slang_symbol_randseq_argument(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::RandSeqProduction)
            return noAst(sym.compilation);
        auto args = s->as<RandSeqProductionSymbol>().arguments;
        if (index >= args.size())
            return noAst(sym.compilation);
        return toC(args[index], sym.compilation);
    });
}

slang_ast slang_symbol_randseq_return_type(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::RandSeqProduction)
            return noAst(sym.compilation);
        // Forced by the freeze sweep's generic per-symbol getDeclaredType()
        // pass (Symbol::getDeclaredType's RandSeqProduction case returns
        // &declaredReturnType), so this is a pure read.
        return toC(&s->as<RandSeqProductionSymbol>().getReturnType(), sym.compilation);
    });
}

// ---- SequenceSymbol -----------------------------------------------------------

uint32_t slang_symbol_sequence_port_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Sequence)
            return 0u;
        return (uint32_t)s->as<SequenceSymbol>().ports.size();
    });
}

slang_ast slang_symbol_sequence_port(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Sequence)
            return noAst(sym.compilation);
        auto ports = s->as<SequenceSymbol>().ports;
        if (index >= ports.size())
            return noAst(sym.compilation);
        return toC(ports[index], sym.compilation);
    });
}

// ---- SpecparamSymbol ----------------------------------------------------------

bool slang_symbol_specparam_is_path_pulse(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        return s && s->kind == SymbolKind::Specparam && s->as<SpecparamSymbol>().isPathPulse;
    });
}

slang_ast slang_symbol_specparam_path_source(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Specparam)
            return noAst(sym.compilation);
        // Forced by the freeze sweep (SpecparamSymbol branch: getPathSource()
        // + getPathDest()), so this is a pure read on a frozen design.
        auto src = s->as<SpecparamSymbol>().getPathSource();
        return src ? toC(src, sym.compilation) : noAst(sym.compilation);
    });
}

slang_ast slang_symbol_specparam_path_dest(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Specparam)
            return noAst(sym.compilation);
        auto dst = s->as<SpecparamSymbol>().getPathDest();
        return dst ? toC(dst, sym.compilation) : noAst(sym.compilation);
    });
}

// ---- StatementBlockSymbol -------------------------------------------------------

uint32_t slang_symbol_statement_block_kind(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::StatementBlock)
            return 0u;
        return (uint32_t)s->as<StatementBlockSymbol>().blockKind;
    });
}

slang_variable_lifetime slang_symbol_statement_block_default_lifetime(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_VARIABLE_LIFETIME_AUTOMATIC, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::StatementBlock)
            return SLANG_VARIABLE_LIFETIME_AUTOMATIC;
        return s->as<StatementBlockSymbol>().defaultLifetime == VariableLifetime::Static
                   ? SLANG_VARIABLE_LIFETIME_STATIC
                   : SLANG_VARIABLE_LIFETIME_AUTOMATIC;
    });
}

// ---- SubroutineSymbol -----------------------------------------------------------

slang_variable_lifetime slang_symbol_subroutine_default_lifetime(slang_ast sym) {
    SLANG_C_ACCESS(SLANG_VARIABLE_LIFETIME_AUTOMATIC, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Subroutine)
            return SLANG_VARIABLE_LIFETIME_AUTOMATIC;
        return s->as<SubroutineSymbol>().defaultLifetime == VariableLifetime::Static
                   ? SLANG_VARIABLE_LIFETIME_STATIC
                   : SLANG_VARIABLE_LIFETIME_AUTOMATIC;
    });
}

uint32_t slang_symbol_subroutine_flags(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Subroutine)
            return 0u;
        return (uint32_t)s->as<SubroutineSymbol>().flags.bits();
    });
}

uint32_t slang_symbol_subroutine_argument_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Subroutine)
            return 0u;
        // getArguments() forces this Subroutine's own scope to elaborate;
        // the freeze sweep's generic Scope-member traversal already did so
        // (a SubroutineSymbol is itself a Scope), so this is a pure read.
        return (uint32_t)s->as<SubroutineSymbol>().getArguments().size();
    });
}

slang_ast slang_symbol_subroutine_argument(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Subroutine)
            return noAst(sym.compilation);
        auto args = s->as<SubroutineSymbol>().getArguments();
        if (index >= args.size())
            return noAst(sym.compilation);
        return toC(args[index], sym.compilation);
    });
}

slang_ast slang_symbol_subroutine_override(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Subroutine)
            return noAst(sym.compilation);
        auto ov = s->as<SubroutineSymbol>().getOverride();
        return ov ? toC(ov, sym.compilation) : noAst(sym.compilation);
    });
}

slang_ast slang_symbol_subroutine_prototype(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Subroutine)
            return noAst(sym.compilation);
        auto proto = s->as<SubroutineSymbol>().getPrototype();
        return proto ? toC(proto, sym.compilation) : noAst(sym.compilation);
    });
}

slang_ast slang_symbol_subroutine_return_type(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Subroutine)
            return noAst(sym.compilation);
        // Forced by the freeze sweep's generic per-symbol getDeclaredType()
        // pass (Symbol::getDeclaredType's Subroutine case returns
        // &declaredReturnType), so this is a pure read.
        return toC(&s->as<SubroutineSymbol>().getReturnType(), sym.compilation);
    });
}

uint32_t slang_symbol_subroutine_kind(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Subroutine)
            return 0u;
        return static_cast<uint32_t>(s->as<SubroutineSymbol>().subroutineKind);
    });
}

bool slang_symbol_subroutine_is_virtual(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Subroutine)
            return false;
        return s->as<SubroutineSymbol>().isVirtual();
    });
}

slang_ast slang_symbol_subroutine_return_val_var(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Subroutine)
            return noAst(sym.compilation);
        auto rv = s->as<SubroutineSymbol>().returnValVar;
        return rv ? toC(rv, sym.compilation) : noAst(sym.compilation);
    });
}

slang_ast slang_symbol_subroutine_this_var(slang_ast sym) {
    SLANG_C_ACCESS(noAst(sym.compilation), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Subroutine)
            return noAst(sym.compilation);
        auto tv = s->as<SubroutineSymbol>().thisVar;
        return tv ? toC(tv, sym.compilation) : noAst(sym.compilation);
    });
}

// ---- UninstantiatedDefSymbol ----------------------------------------------------

slang_str slang_symbol_uninstantiated_def_definition_name(slang_ast sym) {
    SLANG_C_ACCESS(borrowed(""), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::UninstantiatedDef)
            return borrowed("");
        return borrowed(s->as<UninstantiatedDefSymbol>().definitionName);
    });
}

uint32_t slang_symbol_uninstantiated_def_param_expression_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::UninstantiatedDef)
            return 0u;
        return (uint32_t)s->as<UninstantiatedDefSymbol>().paramExpressions.size();
    });
}

slang_ast slang_symbol_uninstantiated_def_param_expression(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::UninstantiatedDef)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        auto params = s->as<UninstantiatedDefSymbol>().paramExpressions;
        if (index >= params.size())
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        return toC(params[index], sym.compilation);
    });
}

uint32_t slang_symbol_uninstantiated_def_port_connection_count(slang_ast sym) {
    SLANG_C_ACCESS(0u, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::UninstantiatedDef)
            return 0u;
        // Forced by the freeze sweep (FreezeVisitor's UninstantiatedDefSymbol
        // branch: `t.getPortConnections()`).
        return (uint32_t)s->as<UninstantiatedDefSymbol>().getPortConnections().size();
    });
}

slang_ast slang_symbol_uninstantiated_def_port_connection(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(noAst(sym.compilation, SLANG_AST_EXPRESSION), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::UninstantiatedDef)
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        auto conns = s->as<UninstantiatedDefSymbol>().getPortConnections();
        // Only the common case (a plain expression, wrapped as a
        // SimpleAssertionExpr) unwraps to an Expression; the rarer
        // checker-only connection shapes (sequence/property actuals, an
        // empty `()`) have no single Expression to return.
        if (index >= conns.size() || !conns[index] ||
            conns[index]->kind != AssertionExprKind::Simple) {
            return noAst(sym.compilation, SLANG_AST_EXPRESSION);
        }
        return toC(&conns[index]->as<SimpleAssertionExpr>().expr, sym.compilation);
    });
}

slang_str slang_symbol_uninstantiated_def_port_name(slang_ast sym, uint32_t index) {
    SLANG_C_ACCESS(borrowed(""), {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::UninstantiatedDef)
            return borrowed("");
        // Forced the same way as slang_symbol_uninstantiated_def_port_connection_count.
        auto names = s->as<UninstantiatedDefSymbol>().getPortNames();
        if (index >= names.size())
            return borrowed("");
        return borrowed(names[index]);
    });
}

bool slang_symbol_uninstantiated_def_is_checker(slang_ast sym) {
    SLANG_C_ACCESS(false, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::UninstantiatedDef)
            return false;
        // Forced the same way as slang_symbol_uninstantiated_def_port_connection_count.
        return s->as<UninstantiatedDefSymbol>().isChecker();
    });
}

// ---- Unstable ---------------------------------------------------------------

#if SLANG_C_API_ALLOW_UNSTABLE
const void* slang_unstable_native_ptr(slang_ast node) {
    return node.ptr;
}
#endif
