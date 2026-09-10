//------------------------------------------------------------------------------
//! @file CApiInternal.h
//! @brief Private definitions shared by the C API implementation files
//
// SPDX-FileCopyrightText: Michael Popoloski
// SPDX-License-Identifier: MIT
//------------------------------------------------------------------------------
#pragma once

#include <atomic>
#include <cstring>
#include <fmt/format.h>
#include <memory>
#include <string>
#include <vector>

#include "slang/ast/Compilation.h"
#include "slang/ast/Expression.h"
#include "slang/c/slang.h"
#include "slang/diagnostics/DiagnosticEngine.h"
#include "slang/diagnostics/Diagnostics.h"
#include "slang/parsing/Lexer.h"
#include "slang/parsing/Parser.h"
#include "slang/parsing/Preprocessor.h"
#include "slang/syntax/SyntaxTree.h"
#include "slang/text/SourceManager.h"
#include "slang/util/Bag.h"

// The handle types declared opaque in slang.h. Each owns exactly the C++
// objects its lifetime rules in the header describe.

struct slang_source_manager_t {
    std::unique_ptr<slang::SourceManager> owned;
    slang::SourceManager& sm;

    slang_source_manager_t() : owned(std::make_unique<slang::SourceManager>()), sm(*owned) {}
    explicit slang_source_manager_t(slang::SourceManager& borrowed) : sm(borrowed) {}
};

struct slang_options_t {
    slang::parsing::PreprocessorOptions pp;
    slang::parsing::LexerOptions lexer;
    slang::parsing::ParserOptions parser;
    slang::ast::CompilationOptions comp;

    slang::Bag toBag() const {
        slang::Bag bag;
        bag.set(pp);
        bag.set(lexer);
        bag.set(parser);
        bag.set(comp);
        return bag;
    }
};

struct slang_syntax_tree_t {
    std::shared_ptr<slang::syntax::SyntaxTree> tree;
    slang_source_manager sm;
    std::atomic<uint32_t> refs{1};
};

struct slang_compilation_t {
    std::unique_ptr<slang::ast::Compilation> comp;
    std::vector<slang_syntax_tree> trees;
    slang_source_manager sm = nullptr;
    bool sealed = false;
    bool elaboratedAll = false;

    explicit slang_compilation_t(const slang::Bag& options) :
        comp(std::make_unique<slang::ast::Compilation>(options)) {}
    explicit slang_compilation_t(std::unique_ptr<slang::ast::Compilation> existing) :
        comp(std::move(existing)) {}
};

struct slang_diagnostics_t {
    std::vector<slang::Diagnostic> diags;
    slang_source_manager sm;
    slang_compilation comp; // nullable
    std::unique_ptr<slang::DiagnosticEngine> engine;

    slang::DiagnosticEngine& getEngine() {
        if (!engine)
            engine = std::make_unique<slang::DiagnosticEngine>(sm->sm);
        return *engine;
    }
};

namespace slang::capi {

// Tables emitted by `syntax_gen.py --c-api` and `diagnostic_gen.py --c-api`.
namespace gen {
struct MemberInfo {
    const char* name;
    uint8_t form;
};
struct StructInfo {
    const char* name;
    uint32_t memberBegin;
    uint32_t memberCount;
};
extern const char* const syntaxModelHash;
extern const char* const diagnosticsModelHash;
extern const MemberInfo members[];
extern const StructInfo structs[];
extern const uint32_t structCount;
extern const uint32_t syntaxKindCount;
extern const uint32_t tokenKindCount;
extern const uint32_t triviaKindCount;
extern const uint32_t kindToStruct[];
bool memberSpan(const syntax::SyntaxNode& node, uint32_t member, uint32_t& start, uint32_t& len);
} // namespace gen

// ---- Errors -----------------------------------------------------------------

inline void setError(slang_error* err, slang_status status, std::string_view message) {
    if (!err)
        return;
    err->status = status;
    size_t n = std::min(message.size(), sizeof(err->message) - 1);
    std::memcpy(err->message, message.data(), n);
    err->message[n] = '\0';
}

// Returns true if the call should proceed: the error record (if any) is not
// already in a failed state.
inline bool checkEntry(slang_error* err) {
    return !err || SLANG_SUCCESS(err->status);
}

#if __cpp_exceptions
#    define SLANG_C_GUARD(err, ...)                                                 \
        try {                                                                       \
            __VA_ARGS__                                                             \
        }                                                                           \
        catch (const std::bad_alloc&) {                                             \
            ::slang::capi::setError(err, SLANG_ERR_OUT_OF_MEMORY, "out of memory"); \
        }                                                                           \
        catch (const std::exception& e) {                                           \
            ::slang::capi::setError(err, SLANG_ERR_INTERNAL, e.what());             \
        }                                                                           \
        catch (...) {                                                               \
            ::slang::capi::setError(err, SLANG_ERR_INTERNAL, "unknown exception");  \
        }
#else
#    define SLANG_C_GUARD(err, ...) {__VA_ARGS__}
#endif

// Wraps a value-returning accessor that has no error-out parameter so that a
// slang assertion or exception (which can fire on malformed/partial trees when
// assertions are enabled) cannot unwind across the C boundary; on such a
// failure the accessor returns `dflt`.
#if __cpp_exceptions
#    define SLANG_C_ACCESS(dflt, ...) \
        try {                         \
            __VA_ARGS__               \
        }                             \
        catch (...) {                 \
            return dflt;              \
        }
#else
#    define SLANG_C_ACCESS(dflt, ...) {__VA_ARGS__}
#endif

// ---- Strings ----------------------------------------------------------------

inline slang_str borrowed(std::string_view s) {
    return slang_str{s.data(), s.size(), nullptr};
}

inline slang_str borrowed(const char* s) {
    return slang_str{s, std::strlen(s), nullptr};
}

// Moves a std::string onto the heap and returns it as an owned slang_str.
inline slang_str owned(std::string&& s) {
    auto* p = new std::string(std::move(s));
    return slang_str{p->data(), p->size(), p};
}

inline std::string_view toView(const char* data, size_t len) {
    return data ? std::string_view(data, len) : std::string_view();
}

// ---- Locations --------------------------------------------------------------

inline slang_loc toC(SourceLocation loc) {
    return slang_loc{loc.buffer().getId(), 0, loc.offset()};
}

inline SourceLocation fromC(slang_loc loc) {
    return SourceLocation(BufferID(loc.buffer, ""sv), loc.offset);
}

inline slang_range toC(SourceRange range) {
    return slang_range{toC(range.start()), toC(range.end())};
}

// ---- Positions --------------------------------------------------------------

inline slang_node toC(const syntax::SyntaxNode* node, slang_syntax_tree tree) {
    if (!node)
        return slang_node{nullptr, tree, 0, 0};
    return slang_node{node, tree, (uint32_t)node->kind, 0};
}

inline const syntax::SyntaxNode* fromC(slang_node node) {
    return static_cast<const syntax::SyntaxNode*>(node.ptr);
}

inline slang_token toC(const syntax::SyntaxNode* owner, uint32_t index, parsing::Token token,
                       slang_syntax_tree tree) {
    uint16_t flags = token.isMissing() ? SLANG_TOKEN_MISSING : 0;
    return slang_token{owner, tree, index, (uint16_t)token.kind, flags};
}

inline slang_token noToken(slang_syntax_tree tree) {
    return slang_token{nullptr, tree, 0, 0, SLANG_TOKEN_MISSING};
}

inline parsing::Token fromC(slang_token token) {
    if (!token.owner)
        return parsing::Token();
    return static_cast<const syntax::SyntaxNode*>(token.owner)->childToken(token.index);
}

inline slang_ast toC(const ast::Symbol* sym, slang_compilation comp) {
    if (!sym)
        return slang_ast{nullptr, comp, 0, SLANG_AST_SYMBOL};
    return slang_ast{sym, comp, (uint32_t)sym->kind, SLANG_AST_SYMBOL};
}

inline slang_ast toC(const ast::Expression* expr, slang_compilation comp) {
    if (!expr)
        return slang_ast{nullptr, comp, 0, SLANG_AST_EXPRESSION};
    return slang_ast{expr, comp, (uint32_t)expr->kind, SLANG_AST_EXPRESSION};
}

inline slang_ast noAst(slang_compilation comp, slang_ast_domain domain = SLANG_AST_SYMBOL) {
    return slang_ast{nullptr, comp, 0, (uint32_t)domain};
}

template<typename T>
inline const T* fromC(slang_ast node, slang_ast_domain domain) {
    if (!node.ptr || node.domain != (uint32_t)domain)
        return nullptr;
    return static_cast<const T*>(node.ptr);
}

inline const ast::Symbol* symbolOf(slang_ast node) {
    return fromC<ast::Symbol>(node, SLANG_AST_SYMBOL);
}

inline const ast::Expression* exprOf(slang_ast node) {
    return fromC<ast::Expression>(node, SLANG_AST_EXPRESSION);
}

// Finds the handle for the tree that owns a syntax node, by walking to the
// root and matching it against the trees a compilation holds.
slang_syntax_tree findTree(slang_compilation comp, const syntax::SyntaxNode& node);

} // namespace slang::capi
