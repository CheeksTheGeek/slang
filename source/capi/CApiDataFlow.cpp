//------------------------------------------------------------------------------
// CApiDataFlow.cpp
// C API: caller-defined forward dataflow analysis over a procedure
//
// SPDX-FileCopyrightText: Chaitanya Sharma
// SPDX-License-Identifier: MIT
//------------------------------------------------------------------------------
#include "CApiInternal.h"

#include "slang/analysis/AbstractFlowAnalysis.h"
#include "slang/analysis/AnalysisOptions.h"
#include "slang/ast/EvalContext.h"
#include "slang/ast/Expression.h"
#include "slang/ast/expressions/AssignmentExpressions.h"
#include "slang/ast/expressions/CallExpression.h"
#include "slang/ast/expressions/MiscExpressions.h"
#include "slang/ast/statements/ConditionalStatements.h"
#include "slang/ast/statements/LoopStatements.h"
#include "slang/ast/symbols/BlockSymbols.h"
#include "slang/ast/symbols/MemberSymbols.h"
#include "slang/ast/symbols/SubroutineSymbols.h"
#include "slang/util/TypeTraits.h"

using namespace slang;
using namespace slang::analysis;
using namespace slang::ast;
using namespace slang::capi;

namespace {

// A move-only wrapper around the caller's opaque lattice value, which frees it
// through the lattice's `drop` callback. Move-only matches slang's own
// DataFlowState; the framework copies states only through copyState().
struct RustState {
    void* data = nullptr;
    const slang_dfa_lattice* lat = nullptr;
    void* user = nullptr;

    RustState() = default;
    RustState(void* d, const slang_dfa_lattice* l, void* u) : data(d), lat(l), user(u) {}
    RustState(RustState&& o) noexcept : data(o.data), lat(o.lat), user(o.user) { o.data = nullptr; }
    RustState& operator=(RustState&& o) noexcept {
        if (this != &o) {
            reset();
            data = o.data;
            lat = o.lat;
            user = o.user;
            o.data = nullptr;
        }
        return *this;
    }
    RustState(const RustState&) = delete;
    RustState& operator=(const RustState&) = delete;
    ~RustState() { reset(); }
    void reset() {
        if (data && lat)
            lat->drop(user, data);
        data = nullptr;
    }
};

// Wraps a Statement pointer as a slang_ast in the statement domain (mirrors
// the private wrapAst<T> helper in CApiAst.cpp, which is file-local there;
// CApiAnalysis.cpp keeps its own copy of this too for the same reason).
static slang_ast wrapStmt(const Statement& s, slang_compilation comp) {
    return slang_ast{&s, comp, (uint32_t)s.kind, SLANG_AST_STATEMENT};
}

class RustFlowAnalysis : public AbstractFlowAnalysis<RustFlowAnalysis, RustState> {
public:
    const slang_dfa_lattice& lat;
    void* user;
    slang_compilation comp;

    RustFlowAnalysis(const Symbol& sym, AnalysisOptions opts, const slang_dfa_lattice& lat,
                     void* user, slang_compilation comp) :
        AbstractFlowAnalysis<RustFlowAnalysis, RustState>(sym, opts), lat(lat), user(user),
        comp(comp) {}

    // --- Lattice operations (drive the control-flow graph) ------------------
    RustState topState() { return RustState(lat.top(user), &lat, user); }
    RustState unreachableState() { return RustState(lat.bottom(user), &lat, user); }
    RustState copyState(const RustState& s) {
        return RustState(lat.clone(user, s.data), &lat, user);
    }
    void joinState(RustState& r, const RustState& o) { lat.join(user, r.data, o.data); }
    void meetState(RustState& r, const RustState& o) { lat.meet(user, r.data, o.data); }

    // --- Transfer hooks (run the caller's transfer, then continue the walk) --
    void emit(uint32_t kind, const Symbol* sym, const Expression& node) {
        slang_dfa_event ev{kind, capi::toC(sym, comp), capi::toC(&node, comp)};
        lat.transfer(user, getState().data, &ev);
    }

    void handle(const NamedValueExpression& expr) {
        emit(SLANG_DFA_READ, &expr.symbol, expr);
        visitExpr(expr);
    }

    void handle(const AssignmentExpression& expr) {
        // Walk the assignment first (visits the rhs reads and lhs), then record
        // the write to the lhs's referenced symbol with the now-current state.
        visitExpr(expr);
        if (auto sym = expr.left().getSymbolReference())
            emit(SLANG_DFA_WRITE, sym, expr);
    }

    void handle(const CallExpression& expr) {
        emit(SLANG_DFA_CALL, nullptr, expr);
        visitExpr(expr);
    }

    // --- Statement-begin hooks (observers; the actual traversal is still
    // done by the base class's visitStmt) ------------------------------------
    void handle(const CaseStatement& stmt) {
        if (lat.on_case_begin)
            lat.on_case_begin(user, ctxHandle(), wrapStmt(stmt, comp));
        visitStmt(stmt);
    }

    void handle(const ConditionalStatement& stmt) {
        if (lat.on_conditional_begin)
            lat.on_conditional_begin(user, ctxHandle(), wrapStmt(stmt, comp));
        visitStmt(stmt);
    }

    template<typename T>
        requires(IsAnyOf<T, ForLoopStatement, WhileLoopStatement, DoWhileLoopStatement,
                         ForeverLoopStatement, ForeachLoopStatement, RepeatLoopStatement>)
    void handle(const T& stmt) {
        if (lat.on_loop_begin)
            lat.on_loop_begin(user, ctxHandle(), wrapStmt(stmt, comp));
        visitStmt(stmt);
    }

    // The opaque exit-state pointer after run() (still owned by this object).
    void* exitStateData() { return getState().data; }

    // Public forwarders for the slang_dfa_ctx_* accessors: getState() is
    // protected on the base class, but bad/getEvalContext() are already
    // public there.
    void* currentStateData() { return getState().data; }
    slang_dfa_ctx ctxHandle() { return reinterpret_cast<slang_dfa_ctx>(this); }
};

const Statement* bodyOf(const Symbol& sym) {
    switch (sym.kind) {
        case SymbolKind::ProceduralBlock:
            return &sym.as<ProceduralBlockSymbol>().getBody();
        case SymbolKind::Subroutine:
            return &sym.as<SubroutineSymbol>().getBody();
        default:
            return nullptr;
    }
}

} // namespace

void* slang_dfa_run(slang_compilation comp, slang_ast procedure, const slang_dfa_lattice* lattice,
                    void* user, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    auto sym = symbolOf(procedure);
    if (!comp || !sym || !lattice) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        // The flow analysis may constant-fold (e.g. to unroll loops), which
        // allocates into the arena; lift the seal for the run. The caller holds
        // exclusive access (the safe wrapper takes &mut Design).
        SealLift guard(comp);

        AnalysisOptions opts;
        RustFlowAnalysis analysis(*sym, opts, *lattice, user, comp);
        if (sym->kind == SymbolKind::ContinuousAssign) {
            analysis.run(sym->as<ContinuousAssignSymbol>().getAssignment());
        }
        else if (auto body = bodyOf(*sym)) {
            analysis.run(*body);
        }
        else {
            setError(err, SLANG_ERR_INVALID_ARG, "not a procedure");
            return nullptr;
        }
        // The exit state is owned by the analysis object (dropped when it goes
        // out of scope), so hand the caller a fresh copy.
        return lattice->clone(user, analysis.exitStateData());
    });
    return nullptr;
}

// ---- slang_dfa_ctx / slang_eval_ctx accessors -------------------------------
//
// A slang_dfa_ctx is just the live RustFlowAnalysis* for the run currently in
// progress, handed to the caller's on_case_begin/on_conditional_begin/
// on_loop_begin hooks; a slang_eval_ctx is the ast::EvalContext& that analysis
// uses for its own constant folding. Both are valid only for the duration of
// the callback that received them.

static RustFlowAnalysis* ctxOf(slang_dfa_ctx ctx) {
    return reinterpret_cast<RustFlowAnalysis*>(ctx);
}

void* slang_dfa_ctx_state(slang_dfa_ctx ctx) {
    SLANG_C_ACCESS(nullptr, {
        auto a = ctxOf(ctx);
        return a ? a->currentStateData() : nullptr;
    });
}

bool slang_dfa_ctx_is_bad(slang_dfa_ctx ctx) {
    SLANG_C_ACCESS(false, {
        auto a = ctxOf(ctx);
        return a && a->bad;
    });
}

slang_eval_ctx slang_dfa_ctx_eval_context(slang_dfa_ctx ctx) {
    SLANG_C_ACCESS(nullptr, {
        auto a = ctxOf(ctx);
        if (!a)
            return nullptr;
        return reinterpret_cast<slang_eval_ctx>(&a->getEvalContext());
    });
}

slang_constant slang_eval_ctx_evaluate(slang_eval_ctx ectx, slang_ast expr, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        auto e = exprOf(expr);
        if (!e || !ectx)
            return (slang_constant) nullptr;
        // Not a code-execution eval: slang::ast::Expression::eval performs
        // compile-time constant folding of a SystemVerilog expression AST
        // node (e.g. `2+2` -> 4), the same operation slang_expression_eval_constant
        // performs elsewhere in this C API, just reusing the analysis's own
        // EvalContext instead of a fresh one.
        auto& evalCtx = *reinterpret_cast<EvalContext*>(ectx);
        ConstantValue cv = e->eval(evalCtx);
        if (cv.bad())
            return (slang_constant) nullptr;
        return new slang_constant_t{std::move(cv)};
    });
    return nullptr;
}
