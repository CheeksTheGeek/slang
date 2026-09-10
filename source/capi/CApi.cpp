//------------------------------------------------------------------------------
// CApi.cpp
// C API: library information, strings, kind reflection, source manager, options
//
// SPDX-FileCopyrightText: Michael Popoloski
// SPDX-License-Identifier: MIT
//------------------------------------------------------------------------------
#include "CApiInternal.h"

#include "slang/ast/ASTVisitor.h"
#include "slang/ast/Constraints.h"
#include "slang/ast/Patterns.h"
#include "slang/ast/Statement.h"
#include "slang/ast/TimingControl.h"
#include "slang/ast/expressions/AssertionExpr.h"
#include "slang/ast/symbols/CoverSymbols.h"
#include "slang/parsing/TokenKind.h"
#include "slang/syntax/SyntaxKind.h"
#include "slang/util/VersionInfo.h"

using namespace slang;
using namespace slang::capi;

// ---- Status and library information -----------------------------------------

const char* slang_status_name(slang_status status) {
    SLANG_C_ACCESS("", {
    switch (status) {
        case SLANG_WARN_TRUNCATED:
            return "SLANG_WARN_TRUNCATED";
        case SLANG_WARN_PARTIAL:
            return "SLANG_WARN_PARTIAL";
        case SLANG_OK:
            return "SLANG_OK";
        case SLANG_ERR_IO:
            return "SLANG_ERR_IO";
        case SLANG_ERR_INVALID_ARG:
            return "SLANG_ERR_INVALID_ARG";
        case SLANG_ERR_INVALID_STATE:
            return "SLANG_ERR_INVALID_STATE";
        case SLANG_ERR_PARSE_RECURSION:
            return "SLANG_ERR_PARSE_RECURSION";
        case SLANG_ERR_REWRITE:
            return "SLANG_ERR_REWRITE";
        case SLANG_ERR_OUT_OF_MEMORY:
            return "SLANG_ERR_OUT_OF_MEMORY";
        case SLANG_ERR_UNSUPPORTED:
            return "SLANG_ERR_UNSUPPORTED";
        case SLANG_ERR_ABI_MISMATCH:
            return "SLANG_ERR_ABI_MISMATCH";
        case SLANG_ERR_CANCELLED:
            return "SLANG_ERR_CANCELLED";
        case SLANG_ERR_INTERNAL:
            return "SLANG_ERR_INTERNAL";
    }
    return "SLANG_STATUS_UNKNOWN";
    });
}

uint32_t slang_c_version(void) {
    return SLANG_C_VERSION;
}

const char* slang_version_string(void) {
    static const std::string version = VersionInfo::getVersionString();
    return version.c_str();
}

const char* slang_syntax_model_hash(void) {
    return gen::syntaxModelHash;
}

const char* slang_diagnostics_model_hash(void) {
    return gen::diagnosticsModelHash;
}

uint32_t slang_build_flags(void) {
    uint32_t flags = 0;
#if __cpp_exceptions
    flags |= SLANG_BUILD_EXCEPTIONS;
#endif
#if SLANG_ASSERT_ENABLED
    flags |= SLANG_BUILD_ASSERTIONS;
#endif
#if SLANG_USE_THREADS
    flags |= SLANG_BUILD_THREADS;
#endif
    return flags;
}

// ---- Strings ----------------------------------------------------------------

void slang_str_free(slang_str str) {
    delete static_cast<std::string*>(str.owner);
}

// ---- Kind reflection --------------------------------------------------------

uint32_t slang_syntax_kind_count(void) {
    return gen::syntaxKindCount;
}

slang_str slang_syntax_kind_name(uint32_t kind) {
    SLANG_C_ACCESS(borrowed(""), {
        if (kind >= gen::syntaxKindCount)
            return borrowed("");
        return borrowed(toString(syntax::SyntaxKind(kind)));
    });
}

uint32_t slang_syntax_kind_struct(uint32_t kind) {
    SLANG_C_ACCESS(UINT32_MAX, {
        if (kind >= gen::syntaxKindCount)
            return UINT32_MAX;
        return gen::kindToStruct[kind];
    });
}

uint32_t slang_syntax_struct_count(void) {
    return gen::structCount;
}

slang_str slang_syntax_struct_name(uint32_t syntax_struct) {
    SLANG_C_ACCESS(borrowed(""), {
        if (syntax_struct >= gen::structCount)
            return borrowed("");
        return borrowed(gen::structs[syntax_struct].name);
    });
}

uint32_t slang_syntax_struct_member_count(uint32_t syntax_struct) {
    SLANG_C_ACCESS(0, {
        if (syntax_struct >= gen::structCount)
            return 0;
        return gen::structs[syntax_struct].memberCount;
    });
}

static const gen::MemberInfo* memberInfo(uint32_t syntax_struct, uint32_t member) {
    if (syntax_struct >= gen::structCount)
        return nullptr;
    auto& s = gen::structs[syntax_struct];
    if (member >= s.memberCount)
        return nullptr;
    return &gen::members[s.memberBegin + member];
}

slang_str slang_syntax_member_name(uint32_t syntax_struct, uint32_t member) {
    SLANG_C_ACCESS(borrowed(""), {
        auto info = memberInfo(syntax_struct, member);
        return borrowed(info ? info->name : "");
    });
}

slang_member_form slang_syntax_member_form(uint32_t syntax_struct, uint32_t member) {
    SLANG_C_ACCESS(SLANG_MEMBER_TOKEN, {
        auto info = memberInfo(syntax_struct, member);
        return info ? slang_member_form(info->form) : SLANG_MEMBER_TOKEN;
    });
}

uint32_t slang_token_kind_count(void) {
    return gen::tokenKindCount;
}

slang_str slang_token_kind_name(uint32_t kind) {
    SLANG_C_ACCESS(borrowed(""), {
        if (kind >= gen::tokenKindCount)
            return borrowed("");
        return borrowed(toString(parsing::TokenKind(kind)));
    });
}

uint32_t slang_trivia_kind_count(void) {
    return gen::triviaKindCount;
}

slang_str slang_trivia_kind_name(uint32_t kind) {
    SLANG_C_ACCESS(borrowed(""), {
        if (kind >= gen::triviaKindCount)
            return borrowed("");
        return borrowed(toString(parsing::TriviaKind(kind)));
    });
}

#define SLANG_C_AST_KIND_DISPATCH(domain, expr)   \
    switch (domain) {                             \
        case SLANG_AST_SYMBOL:                    \
            return expr(ast::SymbolKind);         \
        case SLANG_AST_EXPRESSION:                \
            return expr(ast::ExpressionKind);     \
        case SLANG_AST_STATEMENT:                 \
            return expr(ast::StatementKind);      \
        case SLANG_AST_TIMING_CONTROL:            \
            return expr(ast::TimingControlKind);  \
        case SLANG_AST_CONSTRAINT:                \
            return expr(ast::ConstraintKind);     \
        case SLANG_AST_ASSERTION_EXPR:            \
            return expr(ast::AssertionExprKind);  \
        case SLANG_AST_BINS_SELECT_EXPR:          \
            return expr(ast::BinsSelectExprKind); \
        case SLANG_AST_PATTERN:                   \
            return expr(ast::PatternKind);        \
    }

slang_str slang_ast_kind_name(slang_ast_domain domain, uint32_t kind) {
#define NAME(T) \
    (kind < T##_traits::values.size() ? borrowed(toString(T##_traits::values[kind])) : borrowed(""))
    SLANG_C_ACCESS(borrowed(""), {
        SLANG_C_AST_KIND_DISPATCH(domain, NAME)
        return borrowed("");
    });
#undef NAME
}

uint32_t slang_ast_kind_count(slang_ast_domain domain) {
#define COUNT(T) (uint32_t)T##_traits::values.size()
    SLANG_C_AST_KIND_DISPATCH(domain, COUNT)
#undef COUNT
    return 0;
}

// ---- Source manager ---------------------------------------------------------

slang_source_manager slang_source_manager_create(slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, { return new slang_source_manager_t(); });
    return nullptr;
}

void slang_source_manager_destroy(slang_source_manager sm) {
    delete sm;
}

void slang_source_manager_add_include_dir(slang_source_manager sm, const char* pattern,
                                          size_t pattern_len, bool system, slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!sm || !pattern) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return;
    }
    SLANG_C_GUARD(err, {
        auto view = toView(pattern, pattern_len);
        auto ec = system ? sm->sm.addSystemDirectories(view) : sm->sm.addUserDirectories(view);
        if (ec)
            setError(err, SLANG_ERR_IO, ec.message());
    });
}

slang_buffer_id slang_source_manager_assign_text(slang_source_manager sm, const char* path,
                                                 size_t path_len, const char* text, size_t text_len,
                                                 slang_error* err) {
    if (!checkEntry(err))
        return 0;
    if (!sm || !text) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return 0;
    }
    SLANG_C_GUARD(err, {
        auto buffer = sm->sm.assignText(toView(path, path_len), toView(text, text_len));
        return buffer.id.getId();
    });
    return 0;
}

slang_buffer_id slang_source_manager_read_file(slang_source_manager sm, const char* path,
                                               size_t path_len, slang_error* err) {
    if (!checkEntry(err))
        return 0;
    if (!sm || !path) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return 0;
    }
    SLANG_C_GUARD(err, {
        auto result = sm->sm.readSource(std::string(toView(path, path_len)));
        if (!result) {
            setError(err, SLANG_ERR_IO,
                     fmt::format("{}: {}", toView(path, path_len), result.error().message()));
            return 0;
        }
        return result->id.getId();
    });
    return 0;
}

slang_str slang_source_manager_file_name(slang_source_manager sm, slang_loc loc) {
    if (!sm)
        return borrowed("");
    SLANG_C_ACCESS(borrowed(""), { return borrowed(sm->sm.getFileName(fromC(loc))); });
}

slang_str slang_source_manager_text(slang_source_manager sm, slang_buffer_id buffer) {
    if (!sm || !buffer)
        return borrowed("");
    SLANG_C_ACCESS(borrowed(""),
                   { return borrowed(sm->sm.getSourceText(BufferID(buffer, ""sv))); });
}

size_t slang_source_manager_line(slang_source_manager sm, slang_loc loc) {
    SLANG_C_ACCESS(0, { return sm ? sm->sm.getLineNumber(fromC(loc)) : 0; });
}

size_t slang_source_manager_column(slang_source_manager sm, slang_loc loc) {
    SLANG_C_ACCESS(0, { return sm ? sm->sm.getColumnNumber(fromC(loc)) : 0; });
}

bool slang_source_manager_is_macro_loc(slang_source_manager sm, slang_loc loc) {
    SLANG_C_ACCESS(false, { return sm && loc.buffer && sm->sm.isMacroLoc(fromC(loc)); });
}

slang_loc slang_source_manager_original_loc(slang_source_manager sm, slang_loc loc) {
    if (!sm || !loc.buffer)
        return loc;
    SLANG_C_ACCESS(loc, { return toC(sm->sm.getFullyOriginalLoc(fromC(loc))); });
}

// ---- Options ----------------------------------------------------------------

slang_options slang_options_create(slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, { return new slang_options_t(); });
    return nullptr;
}

void slang_options_destroy(slang_options options) {
    delete options;
}

void slang_options_set_language_version(slang_options options, slang_language_version version) {
    if (!options)
        return;
    auto lv = LanguageVersion::Default;
    switch (version) {
        case SLANG_LANGUAGE_1364_2005:
            lv = LanguageVersion::v1364_2005;
            break;
        case SLANG_LANGUAGE_1800_2017:
            lv = LanguageVersion::v1800_2017;
            break;
        case SLANG_LANGUAGE_1800_2023:
            lv = LanguageVersion::v1800_2023;
            break;
    }
    options->pp.languageVersion = lv;
    options->lexer.languageVersion = lv;
    options->parser.languageVersion = lv;
    options->comp.languageVersion = lv;
}

void slang_options_define(slang_options options, const char* name, size_t name_len,
                          const char* value, size_t value_len, slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!options || !name) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return;
    }
    SLANG_C_GUARD(err, {
        std::string def(toView(name, name_len));
        if (value) {
            def += '=';
            def += toView(value, value_len);
        }
        options->pp.predefines.push_back(std::move(def));
    });
}

void slang_options_add_top_module(slang_options options, const char* name, size_t name_len,
                                  slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!options || !name) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return;
    }
    SLANG_C_GUARD(err, { options->comp.topModules.emplace(toView(name, name_len)); });
}

void slang_options_set_compilation_flags(slang_options options, uint32_t flags) {
    if (options)
        options->comp.flags = bitmask<ast::CompilationFlags>(ast::CompilationFlags(flags));
}
