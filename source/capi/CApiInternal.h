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

#include "slang/analysis/AnalysisManager.h"
#include "slang/ast/Compilation.h"
#include "slang/ast/EvalContext.h"
#include "slang/ast/Expression.h"
#include "slang/ast/LValue.h"
#include "slang/ast/Lookup.h"
#include "slang/c/slang.h"
#include "slang/diagnostics/DiagnosticEngine.h"
#include "slang/diagnostics/Diagnostics.h"
#include "slang/numeric/ConstantValue.h"
#include "slang/parsing/Lexer.h"
#include "slang/parsing/Parser.h"
#include "slang/parsing/Preprocessor.h"
#include "slang/syntax/SyntaxTree.h"
#include "slang/text/SourceManager.h"
#include "slang/util/Bag.h"

namespace slang {
class TextDiagnosticClient;
} // namespace slang

namespace slang::driver {
class SourceLoader;
} // namespace slang::driver

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
    // A shared_ptr (rather than unique_ptr) so this handle can either own its
    // Compilation (the common case) or, via the borrowed-reference
    // constructor below, wrap one owned elsewhere (e.g.
    // slang::ast::ScriptSession::compilation, which is always mutable and
    // must NOT be deleted when this wrapper is destroyed) with a no-op
    // deleter. All existing call sites use `->` / `.get()`-free access, so
    // this is a transparent swap.
    std::shared_ptr<slang::ast::Compilation> comp;
    std::vector<slang_syntax_tree> trees;
    slang_source_manager sm = nullptr;
    bool sealed = false;
    bool elaboratedAll = false;

    explicit slang_compilation_t(const slang::Bag& options) :
        comp(std::make_shared<slang::ast::Compilation>(options)) {}
    explicit slang_compilation_t(std::unique_ptr<slang::ast::Compilation> existing) :
        comp(std::move(existing)) {}
    // Non-owning: wraps a Compilation this handle does not own and must never
    // free (e.g. a ScriptSession's own always-mutable compilation).
    explicit slang_compilation_t(slang::ast::Compilation& borrowed) :
        comp(&borrowed, [](slang::ast::Compilation*) {}) {}
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

// An owned constant value handle (see slang_expression_constant_value /
// slang_expression_eval_constant / slang_script_session_eval). Shared here
// (rather than kept file-local to CApiConstant.cpp) so other translation
// units (e.g. CApiScript.cpp) can construct one directly.
struct slang_constant_t {
    slang::ConstantValue value;
};

// The result of Expression::evalLValue (see slang_expr_eval_lvalue). Owns the
// scratch EvalContext that the LValue's internal ConstantValue* pointers may
// reference (a local materialized for the referenced value symbol), so those
// pointers stay valid for as long as this handle is alive; slang::ast::LValue
// itself is move-only and non-copyable, hence the unique_ptr indirection for
// the EvalContext (which LValue does not own but must outlive).
struct slang_lvalue_t {
    std::unique_ptr<slang::ast::EvalContext> ctx;
    slang::ast::LValue lval;
};

// The scratch buffer behind slang_lookup_result: a mutable slang::ast::
// LookupResult (see slang_lookup_result_create and the slang_lookup_*
// entry points that populate one, e.g. slang_lookup_within_class_randomize),
// plus the compilation handle needed to wrap `result.found` as a slang_ast
// and to copy `result.getDiagnostics()` out into an owned slang_diagnostics.
// `comp` starts null (a fresh, never-populated result has nothing to wrap
// against) and is set by whichever entry point last populated `result`.
struct slang_lookup_result_t {
    slang::ast::LookupResult result;
    slang_compilation comp = nullptr;
};

// The result of running slang::analysis::AnalysisManager over a compilation
// (see slang_analysis_run and slang_driver_run_analysis). Shared here (rather
// than kept file-local to CApiAnalysis.cpp) so CApiDriver.cpp can construct
// one directly from Driver::runAnalysis's result.
struct slang_analysis_t {
    std::unique_ptr<slang::analysis::AnalysisManager> manager;
    slang_compilation comp;
};

// The handle behind slang_driver_text_diag_client: a stable-address wrapper
// around a reference to the driver's own textDiagClient member (a
// std::shared_ptr<TextDiagnosticClient>). Never reallocated for the life of
// the owning slang_driver_t, so returning its address is a pure read.
struct slang_text_diag_client_t {
    std::shared_ptr<slang::TextDiagnosticClient>& client;
};

// The handle behind slang_driver_source_loader: a stable-address wrapper
// around a reference to the driver's own sourceLoader member. Never
// reallocated for the life of the owning slang_driver_t, so returning its
// address is a pure read.
struct slang_source_loader_t {
    slang::driver::SourceLoader& loader;

    // The owning driver's source manager handle, needed to wrap the syntax
    // trees returned by SourceLoader::getLibraryMaps() (see
    // slang_source_loader_library_map_at). Never null once constructed by
    // slang_driver_t; borrowed, outlives this handle.
    slang_source_manager sm = nullptr;

    // A cache of wrapped handles for loader.getLibraryMaps(), grown lazily to
    // match (see syncLibraryMapTrees in CApiDriver.cpp) since the real list
    // only ever grows and its shared_ptr<SyntaxTree> elements need a stable
    // slang_syntax_tree_t wrapper to hand back through the C API. Each entry
    // is created with a single reference owned by this cache and released in
    // the destructor below (a caller that wants one to outlive the loader
    // must retain it first, exactly like every other borrowed tree handle).
    std::vector<slang_syntax_tree> libraryMapTrees;

    // The result of the last SourceLoader::loadSources() call (see
    // slang_source_loader_load_sources / slang_source_loader_loaded_buffer_*).
    // Each SourceBuffer's `data` points into storage owned by the source
    // manager, so it stays valid for the lifetime of this handle without
    // needing to be copied.
    std::vector<slang::SourceBuffer> loadedBuffers;

    ~slang_source_loader_t() {
        for (auto tree : libraryMapTrees)
            slang_syntax_tree_release(tree);
    }
};

namespace slang::capi {

// RAII lift of a compilation's seal for the span of an operation that slang
// implements by mutating the arena — constant-eval caching, name-lookup
// reference tracking, dataflow constant folding. The caller must hold exclusive
// access (the safe wrappers take `&mut`). Re-seals on scope exit exactly when it
// lifted, and never on a compilation that was already unsealed.
struct SealLift {
    slang_compilation comp;
    bool lifted;

    explicit SealLift(slang_compilation comp) : comp(comp), lifted(comp && comp->sealed) {
        if (lifted)
            comp->comp->unfreeze();
    }
    ~SealLift() {
        if (lifted)
            comp->comp->freeze();
    }
    SealLift(const SealLift&) = delete;
    SealLift& operator=(const SealLift&) = delete;
};

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

// The null CST-node cursor into `tree` — the syntax-side analogue of `noAst`.
inline slang_node noNode(slang_syntax_tree tree) {
    return slang_node{nullptr, tree, 0, 0};
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
