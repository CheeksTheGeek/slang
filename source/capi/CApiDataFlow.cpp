//------------------------------------------------------------------------------
// CApiDataFlow.cpp
// C API: caller-defined forward dataflow analysis over a procedure
//
// SPDX-FileCopyrightText: Michael Popoloski
// SPDX-License-Identifier: MIT
//------------------------------------------------------------------------------
#include "CApiInternal.h"

#include "slang/analysis/AbstractFlowAnalysis.h"
#include "slang/analysis/AnalysisOptions.h"
#include "slang/ast/expressions/AssignmentExpressions.h"
#include "slang/ast/expressions/CallExpression.h"
#include "slang/ast/expressions/MiscExpressions.h"
#include "slang/ast/symbols/BlockSymbols.h"
#include "slang/ast/symbols/MemberSymbols.h"
#include "slang/ast/symbols/SubroutineSymbols.h"

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

    // The opaque exit-state pointer after run() (still owned by this object).
    void* exitStateData() { return getState().data; }
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
