//------------------------------------------------------------------------------
// CApiDiagnostics.cpp
// C API: diagnostic lists and rendering
//
// SPDX-FileCopyrightText: Chaitanya Sharma
// SPDX-License-Identifier: MIT
//------------------------------------------------------------------------------
#include "CApiInternal.h"

#include "slang/diagnostics/TextDiagnosticClient.h"

using namespace slang;
using namespace slang::capi;

static slang_severity toC(DiagnosticSeverity severity) {
    switch (severity) {
        case DiagnosticSeverity::Ignored:
            return SLANG_SEVERITY_IGNORED;
        case DiagnosticSeverity::Note:
            return SLANG_SEVERITY_NOTE;
        case DiagnosticSeverity::Warning:
            return SLANG_SEVERITY_WARNING;
        case DiagnosticSeverity::Error:
            return SLANG_SEVERITY_ERROR;
        case DiagnosticSeverity::Fatal:
            return SLANG_SEVERITY_FATAL;
    }
    return SLANG_SEVERITY_IGNORED;
}

static uint32_t packCode(DiagCode code) {
    return ((uint32_t)code.getSubsystem() << 16) | code.getCode();
}

static void fill(slang_diagnostics_t& list, const Diagnostic& d, slang_diag& out) {
    out.code = packCode(d.code);
    out.severity = toC(list.getEngine().getSeverity(d.code, d.location));
    out.location = capi::toC(d.location);
    out.note_count = (uint32_t)d.notes.size();
    out.range_count = (uint32_t)d.ranges.size();
}

static const Diagnostic* diagAt(slang_diagnostics diags, uint32_t index) {
    if (!diags || index >= diags->diags.size())
        return nullptr;
    return &diags->diags[index];
}

void slang_diagnostics_destroy(slang_diagnostics diags) {
    delete diags;
}

uint32_t slang_diagnostics_count(slang_diagnostics diags) {
    SLANG_C_ACCESS(0, { return diags ? (uint32_t)diags->diags.size() : 0; });
}

bool slang_diagnostics_at(slang_diagnostics diags, uint32_t index, slang_diag* out) {
    auto d = diagAt(diags, index);
    if (!d)
        return false;
    // Formatting a diagnostic's severity/location can trip slang assertions on
    // malformed input; keep those from unwinding across the boundary.
    SLANG_C_ACCESS(false, {
        if (out)
            fill(*diags, *d, *out);
        return true;
    });
}

slang_str slang_diagnostics_message(slang_diagnostics diags, uint32_t index, slang_error* err) {
    if (!checkEntry(err))
        return borrowed("");
    auto d = diagAt(diags, index);
    if (!d) {
        setError(err, SLANG_ERR_INVALID_ARG, "diagnostic index out of range");
        return borrowed("");
    }
    SLANG_C_GUARD(err, { return owned(diags->getEngine().formatMessage(*d)); });
    return borrowed("");
}

bool slang_diagnostics_note(slang_diagnostics diags, uint32_t index, uint32_t note,
                            slang_diag* out) {
    auto d = diagAt(diags, index);
    if (!d || note >= d->notes.size())
        return false;
    // fill() formats severity/location and can trip slang assertions on
    // malformed input; keep those from unwinding across the boundary.
    SLANG_C_ACCESS(false, {
        if (out)
            fill(*diags, d->notes[note], *out);
        return true;
    });
}

slang_str slang_diagnostics_note_message(slang_diagnostics diags, uint32_t index, uint32_t note,
                                         slang_error* err) {
    if (!checkEntry(err))
        return borrowed("");
    auto d = diagAt(diags, index);
    if (!d || note >= d->notes.size()) {
        setError(err, SLANG_ERR_INVALID_ARG, "note index out of range");
        return borrowed("");
    }
    SLANG_C_GUARD(err, { return owned(diags->getEngine().formatMessage(d->notes[note])); });
    return borrowed("");
}

bool slang_diagnostics_range(slang_diagnostics diags, uint32_t index, uint32_t range,
                             slang_range* out) {
    auto d = diagAt(diags, index);
    if (!d || range >= d->ranges.size())
        return false;
    SLANG_C_ACCESS(false, {
        if (out)
            *out = capi::toC(d->ranges[range]);
        return true;
    });
}

slang_ast slang_diagnostics_symbol(slang_diagnostics diags, uint32_t index) {
    auto d = diagAt(diags, index);
    if (!d || !diags->comp)
        return noAst(diags ? diags->comp : nullptr);
    SLANG_C_ACCESS(noAst(diags->comp), { return capi::toC(d->symbol, diags->comp); });
}

slang_str slang_diagnostics_render(slang_diagnostics diags, const slang_render_options* options,
                                   slang_error* err) {
    if (!checkEntry(err))
        return borrowed("");
    if (!diags) {
        setError(err, SLANG_ERR_INVALID_ARG, "null diagnostics");
        return borrowed("");
    }
    SLANG_C_GUARD(err, {
        slang_render_options defaults{false, true, true, false};
        auto& opts = options ? *options : defaults;

        DiagnosticEngine engine(diags->sm->sm);
        auto client = std::make_shared<TextDiagnosticClient>();
        client->showColors(opts.colors);
        client->showSourceLine(opts.show_source);
        client->showIncludeStack(opts.show_include_stack);
        client->showAbsPaths(opts.absolute_paths);
        engine.addClient(client);
        for (auto& d : diags->diags)
            engine.issue(d);
        return owned(client->getString());
    });
    return borrowed("");
}
