//------------------------------------------------------------------------------
// CApiScript.cpp
// C API: the script session (interactive snippet evaluation)
//
// SPDX-FileCopyrightText: Chaitanya Sharma
// SPDX-License-Identifier: MIT
//------------------------------------------------------------------------------
#include "CApiInternal.h"

#include "slang/ast/ScriptSession.h"
#include "slang/syntax/AllSyntax.h"

using namespace slang;
using namespace slang::capi;

namespace {

const syntax::ExpressionSyntax* exprSyntaxOf(slang_node node) {
    auto n = fromC(node);
    if (!n || !syntax::ExpressionSyntax::isKind(n->kind))
        return nullptr;
    return &n->as<syntax::ExpressionSyntax>();
}

const syntax::StatementSyntax* stmtSyntaxOf(slang_node node) {
    auto n = fromC(node);
    if (!n || !syntax::StatementSyntax::isKind(n->kind))
        return nullptr;
    return &n->as<syntax::StatementSyntax>();
}

} // namespace

// A script session bundles slang::ast::ScriptSession (which owns its own
// always-mutable Compilation, used to hold declared script state) with a
// stable-address wrapper around the process-wide default SourceManager that
// slang::ast::ScriptSession::eval and ::getDiagnostics use internally (see
// slang::syntax::SyntaxTree::fromText / ::getDefaultSourceManager), so
// diagnostics pulled from this session can be rendered.
struct slang_script_session_t {
    std::unique_ptr<ast::ScriptSession> session;
    slang_source_manager_t smHandle;

    explicit slang_script_session_t(Bag options) :
        session(std::make_unique<ast::ScriptSession>(std::move(options))),
        smHandle(syntax::SyntaxTree::getDefaultSourceManager()) {}
};

slang_script_session slang_script_session_create(slang_options options, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        return new slang_script_session_t(options ? options->toBag() : Bag());
    });
    return nullptr;
}

void slang_script_session_destroy(slang_script_session session) {
    delete session;
}

slang_constant slang_script_session_eval(slang_script_session session, const char* text,
                                         size_t text_len, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    if (!session || !text) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        ConstantValue cv = session->session->eval(toView(text, text_len));
        if (cv.bad())
            return (slang_constant) nullptr;
        return new slang_constant_t{std::move(cv)};
    });
    return nullptr;
}

slang_constant slang_script_session_eval_expression(slang_script_session session, slang_node expr,
                                                    slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    if (!session) {
        setError(err, SLANG_ERR_INVALID_ARG, "null session");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        auto syn = exprSyntaxOf(expr);
        if (!syn) {
            setError(err, SLANG_ERR_INVALID_ARG, "node is not an expression");
            return (slang_constant) nullptr;
        }
        ConstantValue cv = session->session->evalExpression(*syn);
        if (cv.bad())
            return (slang_constant) nullptr;
        return new slang_constant_t{std::move(cv)};
    });
    return nullptr;
}

void slang_script_session_eval_statement(slang_script_session session, slang_node stmt,
                                         slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!session) {
        setError(err, SLANG_ERR_INVALID_ARG, "null session");
        return;
    }
    SLANG_C_GUARD(err, {
        auto syn = stmtSyntaxOf(stmt);
        if (!syn) {
            setError(err, SLANG_ERR_INVALID_ARG, "node is not a statement");
            return;
        }
        session->session->evalStatement(*syn);
    });
}

slang_compilation slang_script_session_compilation(slang_script_session session,
                                                    slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    if (!session) {
        setError(err, SLANG_ERR_INVALID_ARG, "null session");
        return nullptr;
    }
    SLANG_C_GUARD(err, { return new slang_compilation_t(session->session->compilation); });
    return nullptr;
}

slang_diagnostics slang_script_session_diagnostics(slang_script_session session,
                                                    slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    if (!session) {
        setError(err, SLANG_ERR_INVALID_ARG, "null session");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        auto result = new slang_diagnostics_t();
        result->sm = &session->smHandle;
        result->comp = nullptr;
        auto diags = session->session->getDiagnostics();
        result->diags.assign(diags.begin(), diags.end());
        return result;
    });
    return nullptr;
}
