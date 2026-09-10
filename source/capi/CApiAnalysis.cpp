//------------------------------------------------------------------------------
// CApiAnalysis.cpp
// C API: semantic analysis (lints and driver tracking)
//
// SPDX-FileCopyrightText: Michael Popoloski
// SPDX-License-Identifier: MIT
//------------------------------------------------------------------------------
#include "CApiInternal.h"

#include "slang/analysis/AnalysisManager.h"
#include "slang/analysis/AnalyzedAssertion.h"
#include "slang/analysis/AnalyzedProcedure.h"
#include "slang/analysis/ValueDriver.h"
#include "slang/ast/Scope.h"
#include "slang/ast/symbols/InstanceSymbols.h"
#include "slang/ast/symbols/ValueSymbol.h"
#include "slang/util/ThreadPool.h"

using namespace slang;
using namespace slang::analysis;
using namespace slang::capi;

struct slang_analysis_t {
    std::unique_ptr<AnalysisManager> manager;
    slang_compilation comp;
};

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

bool slang_analysis_driver(slang_analysis analysis, slang_ast value, uint32_t index,
                           slang_driver_info* out) {
    SLANG_C_ACCESS(false, {
    auto drivers = driversOf(analysis, value);
    if (index >= drivers.size())
        return false;
    if (out) {
        auto driver = drivers[index];
        uint32_t flags = 0;
        if (driver->isInputPort())
            flags |= SLANG_DRIVER_INPUT_PORT;
        if (driver->isUnidirectionalPort())
            flags |= SLANG_DRIVER_UNIDIRECTIONAL_PORT;
        if (driver->isClockVar())
            flags |= SLANG_DRIVER_CLOCK_VAR;
        *out = slang_driver_info{
            driver->kind == DriverKind::Continuous ? SLANG_DRIVER_CONTINUOUS
                                                   : SLANG_DRIVER_PROCEDURAL,
            flags,
            capi::toC(driver->getSourceRange()),
            capi::toC(driver->containingSymbol, value.compilation),
        };
    }
    return true;
    });
}
