//------------------------------------------------------------------------------
// CApiAnalysis.cpp
// C API: semantic analysis (lints and driver tracking)
//
// SPDX-FileCopyrightText: Chaitanya Sharma
// SPDX-License-Identifier: MIT
//------------------------------------------------------------------------------
#include "CApiInternal.h"

#include "slang/analysis/AnalysisManager.h"
#include "slang/analysis/AnalyzedAssertion.h"
#include "slang/analysis/AnalyzedProcedure.h"
#include "slang/analysis/ValueDriver.h"
#include "slang/ast/Scope.h"
#include "slang/ast/Statement.h"
#include "slang/ast/TimingControl.h"
#include "slang/ast/expressions/AssertionExpr.h"
#include "slang/ast/expressions/CallExpression.h"
#include "slang/ast/expressions/MiscExpressions.h"
#include "slang/ast/statements/MiscStatements.h"
#include "slang/ast/symbols/InstanceSymbols.h"
#include "slang/ast/symbols/ValueSymbol.h"
#include "slang/util/ThreadPool.h"

using namespace slang;
using namespace slang::analysis;
using namespace slang::capi;

// slang_analysis_t is defined in CApiInternal.h (shared with CApiDriver.cpp,
// which constructs one from Driver::runAnalysis's result).

static slang_analysis runImpl(slang_compilation comp, uint32_t flags, uint32_t threads,
                              const slang_analysis_listeners* listeners, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    if (!comp) {
        setError(err, SLANG_ERR_INVALID_ARG, "null compilation");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        // Analysis reads a finalized compilation.
        comp->comp->getRoot();

        AnalysisOptions options;
        options.flags = bitmask<AnalysisFlags>(AnalysisFlags(flags));

        std::shared_ptr<ThreadPool> pool;
        if (threads != 1)
            pool = std::make_shared<ThreadPool>(threads); // 0 => hardware concurrency

        auto result = new slang_analysis_t();
        result->comp = comp;
        result->manager = std::make_unique<AnalysisManager>(options, std::move(pool));

        // Each callback is captured by value, so the std::function is
        // self-contained; listeners may run on several worker threads at once.
        if (listeners) {
            slang_compilation c = comp;
            void* user = listeners->user;
            if (auto cb = listeners->on_procedure) {
                result->manager->addListener([cb, user, c](const AnalyzedProcedure& ap) {
                    cb(toC(ap.analyzedSymbol, c), user);
                });
            }
            if (auto cb = listeners->on_scope) {
                result->manager->addListener([cb, user, c](const AnalyzedScope& as) {
                    cb(toC(&as.scope.asSymbol(), c), user);
                });
            }
            if (auto cb = listeners->on_assertion) {
                result->manager->addListener([cb, user, c](const AnalyzedAssertion& aa) {
                    cb(toC(aa.containingSymbol, c), user);
                });
            }
        }
        result->manager->analyze(*comp->comp);
        return result;
    });
    return nullptr;
}

slang_analysis slang_analysis_run(slang_compilation comp, uint32_t flags, uint32_t threads,
                                  slang_error* err) {
    return runImpl(comp, flags, threads, nullptr, err);
}

slang_analysis slang_analysis_run_listening(slang_compilation comp, uint32_t flags,
                                            uint32_t threads,
                                            const slang_analysis_listeners* listeners,
                                            slang_error* err) {
    return runImpl(comp, flags, threads, listeners, err);
}

void slang_analysis_destroy(slang_analysis analysis) {
    delete analysis;
}

slang_diagnostics slang_analysis_diagnostics(slang_analysis analysis, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    if (!analysis) {
        setError(err, SLANG_ERR_INVALID_ARG, "null analysis");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        auto result = new slang_diagnostics_t();
        result->sm = analysis->comp->sm;
        result->comp = analysis->comp;
        auto diags = analysis->manager->getDiagnostics();
        result->diags.assign(diags.begin(), diags.end());
        return result;
    });
    return nullptr;
}

// Resolves a scope symbol (module body, package, ...) to its analyzed scope.
static const AnalyzedScope* analyzedScopeOf(slang_analysis analysis, slang_ast scope) {
    auto sym = symbolOf(scope);
    if (!analysis || !sym)
        return nullptr;
    const ast::Scope* s = nullptr;
    if (sym->kind == ast::SymbolKind::Instance)
        s = &sym->as<ast::InstanceSymbol>().body;
    else if (sym->isScope())
        s = &sym->as<ast::Scope>();
    if (!s)
        return nullptr;
    return analysis->manager->getAnalyzedScope(*s);
}

uint32_t slang_analysis_scope_procedure_count(slang_analysis analysis, slang_ast scope) {
    SLANG_C_ACCESS(0, {
        auto as = analyzedScopeOf(analysis, scope);
        return as ? (uint32_t)as->procedures.size() : 0;
    });
}

slang_ast slang_analysis_scope_procedure(slang_analysis analysis, slang_ast scope, uint32_t index) {
    SLANG_C_ACCESS(noAst(scope.compilation), {
        auto as = analyzedScopeOf(analysis, scope);
        if (!as || index >= as->procedures.size())
            return noAst(scope.compilation);
        return capi::toC(as->procedures[index].analyzedSymbol.get(), scope.compilation);
    });
}

bool slang_analysis_procedure_has_clock(slang_analysis analysis, slang_ast scope, uint32_t index) {
    SLANG_C_ACCESS(false, {
        auto as = analyzedScopeOf(analysis, scope);
        if (!as || index >= as->procedures.size())
            return false;
        return as->procedures[index].getInferredClock() != nullptr;
    });
}

static std::vector<const ValueDriver*> driversOf(slang_analysis analysis, slang_ast value) {
    auto sym = symbolOf(value);
    if (!analysis || !sym || !sym->isValue())
        return {};
    return analysis->manager->getDrivers(sym->as<ast::ValueSymbol>());
}

uint32_t slang_analysis_driver_count(slang_analysis analysis, slang_ast value) {
    SLANG_C_ACCESS(0, { return (uint32_t)driversOf(analysis, value).size(); });
}

// Maps every DriverKind case explicitly (mirroring toDriverSource below) so
// that a kind this switch doesn't yet know about — there is no such value
// today, but the C API's SLANG_DRIVER_OTHER exists precisely so a future
// addition degrades to an honest "other" instead of silently misreporting as
// procedural.
static slang_driver_kind toDriverKind(DriverKind kind) {
    switch (kind) {
        case DriverKind::Procedural:
            return SLANG_DRIVER_PROCEDURAL;
        case DriverKind::Continuous:
            return SLANG_DRIVER_CONTINUOUS;
    }
    return SLANG_DRIVER_OTHER;
}

// Shared by slang_analysis_driver and slang_analyzed_procedure_driver_at.
static slang_driver_info toDriverInfo(const ValueDriver& driver, slang_compilation comp) {
    uint32_t flags = 0;
    if (driver.isInputPort())
        flags |= SLANG_DRIVER_INPUT_PORT;
    if (driver.isUnidirectionalPort())
        flags |= SLANG_DRIVER_UNIDIRECTIONAL_PORT;
    if (driver.isClockVar())
        flags |= SLANG_DRIVER_CLOCK_VAR;
    return slang_driver_info{
        toDriverKind(driver.kind),
        flags,
        capi::toC(driver.getSourceRange()),
        capi::toC(driver.containingSymbol, comp),
    };
}

bool slang_analysis_driver(slang_analysis analysis, slang_ast value, uint32_t index,
                           slang_driver_info* out) {
    SLANG_C_ACCESS(false, {
    auto drivers = driversOf(analysis, value);
    if (index >= drivers.size())
        return false;
    if (out)
        *out = toDriverInfo(*drivers[index], value.compilation);
    return true;
    });
}

// ---- Analyzed procedures, assertions, and their sub-results -----------------

static slang_analyzed_procedure noProcedure(slang_analysis analysis) {
    return slang_analyzed_procedure{nullptr, analysis};
}
static slang_analyzed_assertion noAssertion(slang_analysis analysis) {
    return slang_analyzed_assertion{nullptr, analysis};
}
static slang_implicit_event_read_set noImplicitSet(slang_analysis analysis) {
    return slang_implicit_event_read_set{nullptr, analysis};
}
static slang_read_range noReadRange(slang_analysis analysis) {
    return slang_read_range{nullptr, analysis};
}
static slang_sensitivity_list noSensitivity(slang_analysis analysis) {
    return slang_sensitivity_list{nullptr, analysis};
}

static const AnalyzedProcedure* procOf(slang_analyzed_procedure p) {
    return static_cast<const AnalyzedProcedure*>(p.ptr);
}
static const AnalyzedAssertion* assertionOf(slang_analyzed_assertion a) {
    return static_cast<const AnalyzedAssertion*>(a.ptr);
}
static const AnalyzedProcedure::ImplicitEventReadSet* implicitSetOf(
    slang_implicit_event_read_set s) {
    return static_cast<const AnalyzedProcedure::ImplicitEventReadSet*>(s.ptr);
}
static const ReadRange* readRangeOf(slang_read_range r) {
    return static_cast<const ReadRange*>(r.ptr);
}
static const SensitivityList* sensitivityOf(slang_sensitivity_list s) {
    return static_cast<const SensitivityList*>(s.ptr);
}
static slang_value_driver noValueDriver(slang_analysis analysis) {
    return slang_value_driver{nullptr, analysis};
}
static const ValueDriver* valueDriverOf(slang_value_driver d) {
    return static_cast<const ValueDriver*>(d.ptr);
}

// Wraps a Statement/TimingControl/AssertionExpr/Expression pointer as a
// slang_ast in the appropriate domain (mirrors the private wrapAst<T> helper
// in CApiAst.cpp, which is file-local to that translation unit).
static slang_ast wrapStmt(const ast::Statement& s, slang_compilation comp) {
    return slang_ast{&s, comp, (uint32_t)s.kind, SLANG_AST_STATEMENT};
}
static slang_ast wrapTiming(const ast::TimingControl* t, slang_compilation comp) {
    if (!t)
        return noAst(comp, SLANG_AST_TIMING_CONTROL);
    return slang_ast{t, comp, (uint32_t)t->kind, SLANG_AST_TIMING_CONTROL};
}
static slang_ast wrapAssertionExpr(const ast::AssertionExpr& e, slang_compilation comp) {
    return slang_ast{&e, comp, (uint32_t)e.kind, SLANG_AST_ASSERTION_EXPR};
}

slang_analyzed_procedure slang_analysis_scope_procedure_handle(slang_analysis analysis,
                                                                slang_ast scope, uint32_t index) {
    SLANG_C_ACCESS(noProcedure(analysis), {
        auto as = analyzedScopeOf(analysis, scope);
        if (!as || index >= as->procedures.size())
            return noProcedure(analysis);
        return slang_analyzed_procedure{&as->procedures[index], analysis};
    });
}

static std::vector<const AnalyzedAssertion*> assertionsOf(slang_analysis analysis,
                                                          slang_ast containingSymbol) {
    auto sym = symbolOf(containingSymbol);
    if (!analysis || !sym)
        return {};
    return analysis->manager->getAnalyzedAssertions(*sym);
}

uint32_t slang_analysis_assertion_count(slang_analysis analysis, slang_ast containing_symbol) {
    SLANG_C_ACCESS(0, { return (uint32_t)assertionsOf(analysis, containing_symbol).size(); });
}

slang_analyzed_assertion slang_analysis_assertion_at(slang_analysis analysis,
                                                     slang_ast containing_symbol, uint32_t index) {
    SLANG_C_ACCESS(noAssertion(analysis), {
        auto assertions = assertionsOf(analysis, containing_symbol);
        if (index >= assertions.size())
            return noAssertion(analysis);
        return slang_analyzed_assertion{assertions[index], analysis};
    });
}

slang_ast slang_analyzed_assertion_containing_symbol(slang_analyzed_assertion assertion) {
    SLANG_C_ACCESS(noAst(assertion.analysis ? assertion.analysis->comp : nullptr), {
        auto a = assertionOf(assertion);
        if (!a || !assertion.analysis)
            return noAst(nullptr);
        return capi::toC(a->containingSymbol.get(), assertion.analysis->comp);
    });
}

slang_analyzed_procedure slang_analyzed_assertion_procedure(slang_analyzed_assertion assertion) {
    SLANG_C_ACCESS(noProcedure(assertion.analysis), {
        auto a = assertionOf(assertion);
        if (!a || !a->procedure)
            return noProcedure(assertion.analysis);
        return slang_analyzed_procedure{a->procedure, assertion.analysis};
    });
}

slang_ast slang_analyzed_assertion_ast_node(slang_analyzed_assertion assertion) {
    SLANG_C_ACCESS(noAst(assertion.analysis ? assertion.analysis->comp : nullptr), {
        auto a = assertionOf(assertion);
        if (!a || !assertion.analysis)
            return noAst(nullptr);
        auto comp = assertion.analysis->comp;
        return std::visit(
            [comp](auto* node) -> slang_ast {
                using T = std::decay_t<decltype(*node)>;
                if constexpr (std::is_same_v<T, ast::ConcurrentAssertionStatement>)
                    return wrapStmt(*node, comp);
                else
                    return capi::toC(static_cast<const ast::Expression*>(node), comp);
            },
            a->astNode);
    });
}

slang_ast slang_analyzed_assertion_root(slang_analyzed_assertion assertion) {
    SLANG_C_ACCESS(noAst(assertion.analysis ? assertion.analysis->comp : nullptr,
                        SLANG_AST_ASSERTION_EXPR),
                   {
        auto a = assertionOf(assertion);
        if (!a || !assertion.analysis)
            return noAst(nullptr, SLANG_AST_ASSERTION_EXPR);
        return wrapAssertionExpr(a->getRoot(), assertion.analysis->comp);
    });
}

slang_ast slang_analyzed_assertion_semantic_leading_clock(slang_analyzed_assertion assertion) {
    SLANG_C_ACCESS(noAst(assertion.analysis ? assertion.analysis->comp : nullptr,
                        SLANG_AST_TIMING_CONTROL),
                   {
        auto a = assertionOf(assertion);
        if (!a || !assertion.analysis)
            return noAst(nullptr, SLANG_AST_TIMING_CONTROL);
        return wrapTiming(a->getSemanticLeadingClock(), assertion.analysis->comp);
    });
}

slang_ast slang_analyzed_assertion_clock(slang_analyzed_assertion assertion, slang_ast expr) {
    SLANG_C_ACCESS(noAst(assertion.analysis ? assertion.analysis->comp : nullptr,
                        SLANG_AST_TIMING_CONTROL),
                   {
        auto a = assertionOf(assertion);
        auto e = fromC<ast::AssertionExpr>(expr, SLANG_AST_ASSERTION_EXPR);
        if (!a || !e || !assertion.analysis)
            return noAst(nullptr, SLANG_AST_TIMING_CONTROL);
        return wrapTiming(a->getClock(*e), assertion.analysis->comp);
    });
}

slang_ast slang_analyzed_procedure_symbol(slang_analyzed_procedure procedure) {
    SLANG_C_ACCESS(noAst(procedure.analysis ? procedure.analysis->comp : nullptr), {
        auto p = procOf(procedure);
        if (!p || !procedure.analysis)
            return noAst(nullptr);
        return capi::toC(p->analyzedSymbol.get(), procedure.analysis->comp);
    });
}

slang_analyzed_procedure slang_analyzed_procedure_parent(slang_analyzed_procedure procedure) {
    SLANG_C_ACCESS(noProcedure(procedure.analysis), {
        auto p = procOf(procedure);
        if (!p || !p->parentProcedure)
            return noProcedure(procedure.analysis);
        return slang_analyzed_procedure{p->parentProcedure, procedure.analysis};
    });
}

slang_ast slang_analyzed_procedure_inferred_clock(slang_analyzed_procedure procedure) {
    SLANG_C_ACCESS(noAst(procedure.analysis ? procedure.analysis->comp : nullptr,
                        SLANG_AST_TIMING_CONTROL),
                   {
        auto p = procOf(procedure);
        if (!p || !procedure.analysis)
            return noAst(nullptr, SLANG_AST_TIMING_CONTROL);
        return wrapTiming(p->getInferredClock(), procedure.analysis->comp);
    });
}

uint32_t slang_analyzed_procedure_driver_count(slang_analyzed_procedure procedure) {
    SLANG_C_ACCESS(0, {
        auto p = procOf(procedure);
        return p ? (uint32_t)p->getDrivers().size() : 0;
    });
}

bool slang_analyzed_procedure_driver_at(slang_analyzed_procedure procedure, uint32_t index,
                                        slang_driver_info* out) {
    SLANG_C_ACCESS(false, {
        auto p = procOf(procedure);
        if (!p || !procedure.analysis)
            return false;
        auto drivers = p->getDrivers();
        if (index >= drivers.size())
            return false;
        if (out)
            *out = toDriverInfo(*drivers[index], procedure.analysis->comp);
        return true;
    });
}

uint32_t slang_analyzed_procedure_call_expression_count(slang_analyzed_procedure procedure) {
    SLANG_C_ACCESS(0, {
        auto p = procOf(procedure);
        return p ? (uint32_t)p->getCallExpressions().size() : 0;
    });
}

slang_ast slang_analyzed_procedure_call_expression_at(slang_analyzed_procedure procedure,
                                                      uint32_t index) {
    SLANG_C_ACCESS(noAst(procedure.analysis ? procedure.analysis->comp : nullptr), {
        auto p = procOf(procedure);
        if (!p || !procedure.analysis)
            return noAst(nullptr);
        auto calls = p->getCallExpressions();
        if (index >= calls.size())
            return noAst(procedure.analysis->comp);
        return capi::toC(static_cast<const ast::Expression*>(calls[index]), procedure.analysis->comp);
    });
}

uint32_t slang_analyzed_procedure_timing_control_count(slang_analyzed_procedure procedure) {
    SLANG_C_ACCESS(0, {
        auto p = procOf(procedure);
        return p ? (uint32_t)p->getTimingControls().size() : 0;
    });
}

slang_ast slang_analyzed_procedure_timing_control_at(slang_analyzed_procedure procedure,
                                                     uint32_t index) {
    SLANG_C_ACCESS(noAst(procedure.analysis ? procedure.analysis->comp : nullptr,
                        SLANG_AST_STATEMENT),
                   {
        auto p = procOf(procedure);
        if (!p || !procedure.analysis)
            return noAst(nullptr, SLANG_AST_STATEMENT);
        auto controls = p->getTimingControls();
        if (index >= controls.size())
            return noAst(procedure.analysis->comp, SLANG_AST_STATEMENT);
        return wrapStmt(*controls[index], procedure.analysis->comp);
    });
}

uint32_t slang_analyzed_procedure_read_set_count(slang_analyzed_procedure procedure) {
    SLANG_C_ACCESS(0, {
        auto p = procOf(procedure);
        return p ? (uint32_t)p->getReadSet().size() : 0;
    });
}

slang_read_range slang_analyzed_procedure_read_set_at(slang_analyzed_procedure procedure,
                                                      uint32_t index) {
    SLANG_C_ACCESS(noReadRange(procedure.analysis), {
        auto p = procOf(procedure);
        if (!p)
            return noReadRange(procedure.analysis);
        auto reads = p->getReadSet();
        if (index >= reads.size())
            return noReadRange(procedure.analysis);
        return slang_read_range{&reads[index], procedure.analysis};
    });
}

uint32_t slang_analyzed_procedure_implicit_event_read_set_count(slang_analyzed_procedure procedure) {
    SLANG_C_ACCESS(0, {
        auto p = procOf(procedure);
        return p ? (uint32_t)p->getImplicitEventReadSets().size() : 0;
    });
}

slang_implicit_event_read_set slang_analyzed_procedure_implicit_event_read_set_at(
    slang_analyzed_procedure procedure, uint32_t index) {
    SLANG_C_ACCESS(noImplicitSet(procedure.analysis), {
        auto p = procOf(procedure);
        if (!p)
            return noImplicitSet(procedure.analysis);
        auto sets = p->getImplicitEventReadSets();
        if (index >= sets.size())
            return noImplicitSet(procedure.analysis);
        return slang_implicit_event_read_set{&sets[index], procedure.analysis};
    });
}

slang_sensitivity_list slang_analyzed_procedure_sensitivity_list(slang_analyzed_procedure procedure) {
    SLANG_C_ACCESS(noSensitivity(procedure.analysis), {
        auto p = procOf(procedure);
        if (!p)
            return noSensitivity(procedure.analysis);
        return slang_sensitivity_list{&p->getSensitivityList(), procedure.analysis};
    });
}

slang_ast slang_implicit_event_read_set_statement(slang_implicit_event_read_set read_set) {
    SLANG_C_ACCESS(noAst(read_set.analysis ? read_set.analysis->comp : nullptr,
                        SLANG_AST_STATEMENT),
                   {
        auto s = implicitSetOf(read_set);
        if (!s || !read_set.analysis)
            return noAst(nullptr, SLANG_AST_STATEMENT);
        return wrapStmt(*s->statement.get(), read_set.analysis->comp);
    });
}

uint32_t slang_implicit_event_read_set_read_count(slang_implicit_event_read_set read_set) {
    SLANG_C_ACCESS(0, {
        auto s = implicitSetOf(read_set);
        return s ? (uint32_t)s->reads.size() : 0;
    });
}

slang_read_range slang_implicit_event_read_set_read_at(slang_implicit_event_read_set read_set,
                                                       uint32_t index) {
    SLANG_C_ACCESS(noReadRange(read_set.analysis), {
        auto s = implicitSetOf(read_set);
        if (!s || index >= s->reads.size())
            return noReadRange(read_set.analysis);
        return slang_read_range{&s->reads[index], read_set.analysis};
    });
}

slang_ast slang_read_range_symbol(slang_read_range range) {
    SLANG_C_ACCESS(noAst(range.analysis ? range.analysis->comp : nullptr), {
        auto r = readRangeOf(range);
        if (!r || !range.analysis)
            return noAst(nullptr);
        return capi::toC(static_cast<const ast::Symbol*>(r->symbol.get()), range.analysis->comp);
    });
}

bool slang_read_range_bit_range(slang_read_range range, uint64_t* lo, uint64_t* hi) {
    SLANG_C_ACCESS(false, {
        auto r = readRangeOf(range);
        if (!r)
            return false;
        if (lo)
            *lo = r->bitRange.first;
        if (hi)
            *hi = r->bitRange.second;
        return true;
    });
}

slang_sensitivity_kind slang_sensitivity_list_kind(slang_sensitivity_list list) {
    SLANG_C_ACCESS(SLANG_SENSITIVITY_NONE, {
        auto s = sensitivityOf(list);
        if (!s)
            return SLANG_SENSITIVITY_NONE;
        switch (s->kind) {
            case SensitivityList::Kind::None:
                return SLANG_SENSITIVITY_NONE;
            case SensitivityList::Kind::Explicit:
                return SLANG_SENSITIVITY_EXPLICIT;
            case SensitivityList::Kind::Implicit:
                return SLANG_SENSITIVITY_IMPLICIT;
            case SensitivityList::Kind::Dynamic:
                return SLANG_SENSITIVITY_DYNAMIC;
        }
        return SLANG_SENSITIVITY_NONE;
    });
}

slang_ast slang_sensitivity_list_timing_control(slang_sensitivity_list list) {
    SLANG_C_ACCESS(noAst(list.analysis ? list.analysis->comp : nullptr, SLANG_AST_TIMING_CONTROL), {
        auto s = sensitivityOf(list);
        if (!s || !list.analysis)
            return noAst(nullptr, SLANG_AST_TIMING_CONTROL);
        return wrapTiming(s->timingControl, list.analysis->comp);
    });
}

uint32_t slang_sensitivity_list_read_count(slang_sensitivity_list list) {
    SLANG_C_ACCESS(0, {
        auto s = sensitivityOf(list);
        return s ? (uint32_t)s->reads.size() : 0;
    });
}

slang_read_range slang_sensitivity_list_read_at(slang_sensitivity_list list, uint32_t index) {
    SLANG_C_ACCESS(noReadRange(list.analysis), {
        auto s = sensitivityOf(list);
        if (!s || index >= s->reads.size())
            return noReadRange(list.analysis);
        return slang_read_range{&s->reads[index], list.analysis};
    });
}

// ---- Value driver handles -----------------------------------------------

slang_value_driver slang_analysis_driver_handle(slang_analysis analysis, slang_ast value,
                                                uint32_t index) {
    SLANG_C_ACCESS(noValueDriver(analysis), {
        auto drivers = driversOf(analysis, value);
        if (index >= drivers.size())
            return noValueDriver(analysis);
        return slang_value_driver{drivers[index], analysis};
    });
}

uint32_t slang_value_driver_flags(slang_value_driver driver) {
    SLANG_C_ACCESS(0, {
        auto d = valueDriverOf(driver);
        return d ? (uint32_t)d->flags.bits() : 0u;
    });
}

slang_ast slang_value_driver_symbol(slang_value_driver driver) {
    SLANG_C_ACCESS(noAst(driver.analysis ? driver.analysis->comp : nullptr), {
        auto d = valueDriverOf(driver);
        if (!d || !driver.analysis)
            return noAst(nullptr);
        return capi::toC(static_cast<const ast::Symbol*>(&d->getSymbol()), driver.analysis->comp);
    });
}

bool slang_value_driver_bounds(slang_value_driver driver, uint64_t* lo, uint64_t* hi) {
    SLANG_C_ACCESS(false, {
        auto d = valueDriverOf(driver);
        if (!d)
            return false;
        auto bounds = d->getBounds();
        if (lo)
            *lo = bounds.first;
        if (hi)
            *hi = bounds.second;
        return true;
    });
}

slang_range slang_value_driver_source_range(slang_value_driver driver) {
    SLANG_C_ACCESS(slang_range{}, {
        auto d = valueDriverOf(driver);
        if (!d)
            return slang_range{};
        return capi::toC(d->getSourceRange());
    });
}

bool slang_value_driver_override_range(slang_value_driver driver, slang_range* out) {
    SLANG_C_ACCESS(false, {
        auto d = valueDriverOf(driver);
        if (!d)
            return false;
        auto r = d->getOverrideRange();
        if (!r)
            return false;
        if (out)
            *out = capi::toC(*r);
        return true;
    });
}

bool slang_value_driver_is_unidirectional_port(slang_value_driver driver) {
    SLANG_C_ACCESS(false, {
        auto d = valueDriverOf(driver);
        return d && d->isUnidirectionalPort();
    });
}

bool slang_value_driver_is_in_single_driver_procedure(slang_value_driver driver) {
    SLANG_C_ACCESS(false, {
        auto d = valueDriverOf(driver);
        return d && d->isInSingleDriverProcedure();
    });
}

static slang_driver_source toDriverSource(DriverSource source) {
    switch (source) {
        case DriverSource::Initial:
            return SLANG_DRIVER_SOURCE_INITIAL;
        case DriverSource::Final:
            return SLANG_DRIVER_SOURCE_FINAL;
        case DriverSource::Always:
            return SLANG_DRIVER_SOURCE_ALWAYS;
        case DriverSource::AlwaysComb:
            return SLANG_DRIVER_SOURCE_ALWAYS_COMB;
        case DriverSource::AlwaysLatch:
            return SLANG_DRIVER_SOURCE_ALWAYS_LATCH;
        case DriverSource::AlwaysFF:
            return SLANG_DRIVER_SOURCE_ALWAYS_FF;
        case DriverSource::Subroutine:
            return SLANG_DRIVER_SOURCE_SUBROUTINE;
        case DriverSource::Other:
            break;
    }
    return SLANG_DRIVER_SOURCE_OTHER;
}

slang_driver_source slang_value_driver_source(slang_value_driver driver) {
    SLANG_C_ACCESS(SLANG_DRIVER_SOURCE_OTHER, {
        auto d = valueDriverOf(driver);
        return d ? toDriverSource(d->source) : SLANG_DRIVER_SOURCE_OTHER;
    });
}

static slang_value_path noValuePath(slang_analysis analysis) {
    return slang_value_path{nullptr, analysis};
}
static const ast::ValuePath* valuePathOf(slang_value_path p) {
    return static_cast<const ast::ValuePath*>(p.ptr);
}

slang_value_path slang_value_driver_path(slang_value_driver driver) {
    SLANG_C_ACCESS(noValuePath(driver.analysis), {
        auto d = valueDriverOf(driver);
        if (!d)
            return noValuePath(driver.analysis);
        return slang_value_path{&d->path, driver.analysis};
    });
}

slang_ast slang_value_path_root_symbol(slang_value_path path) {
    SLANG_C_ACCESS(noAst(path.analysis ? path.analysis->comp : nullptr), {
        auto p = valuePathOf(path);
        if (!p || !path.analysis)
            return noAst(nullptr);
        return capi::toC(static_cast<const ast::Symbol*>(p->rootSymbol()), path.analysis->comp);
    });
}
