//------------------------------------------------------------------------------
//! @file slang.h
//! @brief Stable C API for slang, the SystemVerilog compiler frontend
//
// SPDX-FileCopyrightText: Michael Popoloski
// SPDX-License-Identifier: MIT
//------------------------------------------------------------------------------
//
// This header is the ABI-stable surface of slang. It is designed for consumption
// from C and from other languages' foreign-function interfaces (Rust, Go, Zig,
// C#, ...). It deliberately exposes far less than the C++ API, in exchange for a
// surface that does not change shape between slang releases.
//
// Design conventions (these are promises, not suggestions):
//
//  * OWNERS ARE OPAQUE HANDLES, POSITIONS ARE VALUE STRUCTS. Objects that own
//    memory (a source manager, a syntax tree, a compilation, a diagnostic list)
//    are opaque pointers with an explicit destroy/release function. Objects that
//    denote a position *within* an owner (a syntax node, a token, an AST node)
//    are small trivially-copyable structs with no destructor. A position is valid
//    exactly as long as its owner is alive; nothing in this API extends that.
//
//  * OWNERSHIP RULE. If a function returns an object as its return value, that
//    function is a getter and the object's lifetime is tied to the parent object.
//    Objects returned through an out parameter of pointer-to-handle type are
//    owned by the caller, who must free them with the matching function. Do not
//    mix allocators: free every object with the function this header names for
//    it, never with free().
//
//  * ERRORS ARE IN/OUT, NEVER AMBIENT. Every fallible function takes a
//    `slang_error*` as its LAST parameter. If it arrives already failed the
//    function returns immediately without doing anything, so calls can be
//    chained and checked once (the "poison" idiom). There is no thread-local
//    "last error"; a nested call can never clobber the error of an outer one.
//    Negative statuses are warnings: the call succeeded and SLANG_SUCCESS() is
//    true, but something is worth surfacing.
//
//  * NOTHING UNWINDS ACROSS THIS BOUNDARY. Every entry point is noexcept; C++
//    exceptions are converted to error statuses. Callbacks you supply must
//    likewise never unwind into the library (a Rust panic must be caught before
//    returning; C++ must not throw). A callback that needs to abort a traversal
//    returns SLANG_VISIT_BREAK.
//
//  * VERSIONING. SLANG_C_VERSION_MAJOR changes only on an ABI break, which is
//    intended to be never. Enumerator values are append-only. Functions are
//    never removed: if the underlying C++ feature goes away, the function stays
//    and reports SLANG_ERR_UNSUPPORTED. New declarations are gated by
//    SLANG_C_API_AT_LEAST(major, minor) so that a translation unit can target an
//    older surface, and every declaration documents the version it appeared in.
//
//  * KIND ENUMERATIONS ARE DATA, NOT SYMBOLS. slang has ~530 syntax kinds and
//    ~200 AST kinds. They are exposed as small integers plus reflection tables
//    (names, struct membership, member layouts) generated from the same schema
//    that generates slang itself, so coverage is total by construction and a
//    new kind upstream never changes this header.
//
#ifndef SLANG_C_SLANG_H
#define SLANG_C_SLANG_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ------------------------------------------------------------------------- */
/* Versioning                                                                 */
/* ------------------------------------------------------------------------- */

/// The C API version implemented by this header. MAJOR changes only on an ABI
/// break; MINOR increments whenever declarations are added.
#define SLANG_C_VERSION_MAJOR 1
#define SLANG_C_VERSION_MINOR 1

#define SLANG_C_VERSION_ENCODE(major, minor) ((uint32_t)(major) * 10000u + (uint32_t)(minor))
#define SLANG_C_VERSION SLANG_C_VERSION_ENCODE(SLANG_C_VERSION_MAJOR, SLANG_C_VERSION_MINOR)

/// Define before including this header to compile against an older surface.
/// Declarations introduced after the targeted minor version are hidden.
#ifndef SLANG_C_API_TARGET_MINOR
#    define SLANG_C_API_TARGET_MINOR SLANG_C_VERSION_MINOR
#endif

/// True when the targeted API version is at least major.minor.
#define SLANG_C_API_AT_LEAST(major, minor)   \
    (SLANG_C_VERSION_ENCODE(major, minor) <= \
     SLANG_C_VERSION_ENCODE(SLANG_C_VERSION_MAJOR, SLANG_C_API_TARGET_MINOR))

/// Set to 0 to omit declarations that have been deprecated.
#ifndef SLANG_C_API_ALLOW_DEPRECATED
#    define SLANG_C_API_ALLOW_DEPRECATED 1
#endif

/// Set to 1 to opt into the unstable surface: declarations that may change
/// shape or disappear in any release. Only available when targeting the newest
/// version, since they are not part of any versioned promise.
#ifndef SLANG_C_API_ALLOW_UNSTABLE
#    define SLANG_C_API_ALLOW_UNSTABLE 0
#endif
#if SLANG_C_API_ALLOW_UNSTABLE && SLANG_C_API_TARGET_MINOR != SLANG_C_VERSION_MINOR
#    error "the unstable slang C API surface requires targeting the newest API version"
#endif

/* ------------------------------------------------------------------------- */
/* Export                                                                     */
/* ------------------------------------------------------------------------- */

#ifndef SLANG_C_API
#    if defined(SLANG_C_STATIC)
#        define SLANG_C_API
#    elif defined(_WIN32)
#        if defined(SLANG_C_BUILDING)
#            define SLANG_C_API __declspec(dllexport)
#        else
#            define SLANG_C_API __declspec(dllimport)
#        endif
#    elif defined(__GNUC__) && __GNUC__ >= 4
#        define SLANG_C_API __attribute__((visibility("default")))
#    else
#        define SLANG_C_API
#    endif
#endif

/* ------------------------------------------------------------------------- */
/* Status and errors                                                          */
/* ------------------------------------------------------------------------- */

/// Result of a fallible call. Negative values are warnings (the call succeeded),
/// zero is unqualified success, positive values are failures.
typedef enum slang_status {
    /// The result was truncated to fit the caller's buffer.
    SLANG_WARN_TRUNCATED = -2,
    /// The call succeeded but could only partially do what was asked.
    SLANG_WARN_PARTIAL = -1,

    SLANG_OK = 0,

    /// A file could not be read or written.
    SLANG_ERR_IO = 1,
    /// An argument was null, out of range, or otherwise invalid.
    SLANG_ERR_INVALID_ARG = 2,
    /// The object is in the wrong state for this operation (e.g. adding a tree
    /// to a compilation that has already been finalized).
    SLANG_ERR_INVALID_STATE = 3,
    /// The parser gave up on pathologically nested input.
    SLANG_ERR_PARSE_RECURSION = 4,
    /// A syntax tree rewrite was rejected (e.g. two edits to one node).
    SLANG_ERR_REWRITE = 5,
    /// Memory allocation failed.
    SLANG_ERR_OUT_OF_MEMORY = 6,
    /// This build of the library does not support the operation.
    SLANG_ERR_UNSUPPORTED = 7,
    /// The caller and the library disagree about the schema (see
    /// slang_syntax_model_hash) or the C API version.
    SLANG_ERR_ABI_MISMATCH = 8,
    /// A traversal was cancelled by a callback or a cancellation request.
    SLANG_ERR_CANCELLED = 9,
    /// A C++ exception with no more specific mapping was caught.
    SLANG_ERR_INTERNAL = 100,
} slang_status;

#define SLANG_SUCCESS(status) ((status) <= SLANG_OK)
#define SLANG_FAILURE(status) ((status) > SLANG_OK)

/// The in/out error record passed as the last parameter of fallible calls.
/// Initialize with SLANG_ERROR_INIT (or zero-fill). On failure `message` holds a
/// NUL-terminated human-readable explanation, truncated to fit.
typedef struct slang_error {
    slang_status status;
    char message[248];
} slang_error;

#define SLANG_ERROR_INIT {SLANG_OK, {0}}

/// The symbolic name of a status value, e.g. "SLANG_ERR_IO". Never null.
SLANG_C_API const char* slang_status_name(slang_status status);

/* ------------------------------------------------------------------------- */
/* Library information                                                        */
/* ------------------------------------------------------------------------- */

/// The C API version the library was built with, encoded as by
/// SLANG_C_VERSION_ENCODE. Compare against SLANG_C_VERSION at startup; a
/// different major means the header and library are incompatible.
SLANG_C_API uint32_t slang_c_version(void);

/// The slang version string, e.g. "11.0.3+abc1234". Static storage; never null.
SLANG_C_API const char* slang_version_string(void);

/// The SHA-256 (lowercase hex, 64 chars) of the canonical syntax model the
/// library's kind tables were generated from. Consumers with their own copy of
/// the model (e.g. generated typed wrappers) must compare this before using any
/// per-kind member layout information. Static storage; never null.
SLANG_C_API const char* slang_syntax_model_hash(void);

/// As slang_syntax_model_hash, for the diagnostics model.
SLANG_C_API const char* slang_diagnostics_model_hash(void);

/// Bits reported by slang_build_flags.
typedef enum slang_build_flag {
    /// The library was built with C++ exceptions enabled. When this is clear,
    /// conditions that would normally produce SLANG_ERR_PARSE_RECURSION or
    /// SLANG_ERR_REWRITE abort the process instead.
    SLANG_BUILD_EXCEPTIONS = 1u << 0,
    /// The library was built with internal assertions enabled.
    SLANG_BUILD_ASSERTIONS = 1u << 1,
    /// The library was built with thread support.
    SLANG_BUILD_THREADS = 1u << 2,
} slang_build_flag;

/// Bitmask of slang_build_flag describing how the library was compiled.
SLANG_C_API uint32_t slang_build_flags(void);

/* ------------------------------------------------------------------------- */
/* Strings                                                                    */
/* ------------------------------------------------------------------------- */

/// A string view. `data` is NOT NUL-terminated. When `owner` is null the bytes
/// are borrowed from some slang object and slang_str_free is a no-op; when
/// `owner` is non-null the caller owns the bytes and must call slang_str_free.
typedef struct slang_str {
    const char* data;
    size_t len;
    void* owner;
} slang_str;

/// Frees an owned string. Safe to call on a borrowed or zero-initialized string.
SLANG_C_API void slang_str_free(slang_str str);

/* ------------------------------------------------------------------------- */
/* Source locations                                                           */
/* ------------------------------------------------------------------------- */

/// Identifies a loaded source buffer (a file, or one macro expansion). Zero is
/// the invalid buffer.
typedef uint32_t slang_buffer_id;

/// A location: a buffer plus a byte offset within it. A location whose buffer
/// is zero is "no location".
typedef struct slang_loc {
    slang_buffer_id buffer;
    uint32_t reserved_;
    uint64_t offset;
} slang_loc;

/// A half-open [start, end) range of locations within one buffer.
typedef struct slang_range {
    slang_loc start;
    slang_loc end;
} slang_range;

/* ------------------------------------------------------------------------- */
/* Owners                                                                     */
/* ------------------------------------------------------------------------- */

typedef struct slang_source_manager_t* slang_source_manager;
typedef struct slang_syntax_tree_t* slang_syntax_tree;
typedef struct slang_compilation_t* slang_compilation;
typedef struct slang_diagnostics_t* slang_diagnostics;
typedef struct slang_options_t* slang_options;

/* ------------------------------------------------------------------------- */
/* Positions                                                                  */
/* ------------------------------------------------------------------------- */

/// A syntax node within a syntax tree. Trivially copyable; valid while `tree`
/// is alive. `ptr` is null for "no node" (see slang_node_is_null). `kind` is a
/// syntax kind ordinal (see slang_syntax_kind_name).
typedef struct slang_node {
    const void* ptr;
    slang_syntax_tree tree;
    uint32_t kind;
    uint32_t reserved_;
} slang_node;

/// A token within a syntax tree, identified by its owning node and child index
/// rather than by address, since tokens are stored by value inside their
/// parent. Trivially copyable; valid while `tree` is alive.
typedef struct slang_token {
    const void* owner;
    slang_syntax_tree tree;
    uint32_t index;
    uint16_t kind;
    uint16_t flags;
} slang_token;

/// Bits in slang_token::flags.
typedef enum slang_token_flag {
    /// The token was expected but missing from the source and was synthesized
    /// by the parser during error recovery. Its text is empty.
    SLANG_TOKEN_MISSING = 1u << 0,
} slang_token_flag;

/// The families of elaborated AST nodes. Each family has its own kind
/// enumeration; the pair (domain, kind) identifies a node's concrete type.
typedef enum slang_ast_domain {
    SLANG_AST_SYMBOL = 0,
    SLANG_AST_EXPRESSION = 1,
    SLANG_AST_STATEMENT = 2,
    SLANG_AST_TIMING_CONTROL = 3,
    SLANG_AST_CONSTRAINT = 4,
    SLANG_AST_ASSERTION_EXPR = 5,
    SLANG_AST_BINS_SELECT_EXPR = 6,
    SLANG_AST_PATTERN = 7,
} slang_ast_domain;

/// An elaborated AST node within a compilation. Trivially copyable; valid while
/// `compilation` is alive. Types are symbols (domain SLANG_AST_SYMBOL).
typedef struct slang_ast {
    const void* ptr;
    slang_compilation compilation;
    uint32_t kind;
    uint32_t domain;
} slang_ast;

#ifdef __cplusplus
// The position structs are two pointers plus two 32-bit fields, so their size
// is 24 bytes on a 64-bit target (LP64) and 16 on a 32-bit one (e.g. wasm32).
static_assert(sizeof(slang_node) == 2 * sizeof(void*) + 8, "slang_node ABI");
static_assert(sizeof(slang_token) == 2 * sizeof(void*) + 8, "slang_token ABI");
static_assert(sizeof(slang_ast) == 2 * sizeof(void*) + 8, "slang_ast ABI");
static_assert(sizeof(slang_loc) == 16, "slang_loc ABI");
static_assert(sizeof(slang_error) == 252, "slang_error ABI");
#endif

/* ------------------------------------------------------------------------- */
/* Kind reflection (generated tables)                                         */
/* ------------------------------------------------------------------------- */

/// Number of syntax kinds. Kind 0 is always "Unknown".
SLANG_C_API uint32_t slang_syntax_kind_count(void);

/// The name of a syntax kind, e.g. "ModuleDeclaration". Borrowed, static.
/// Returns an empty string for out-of-range kinds.
SLANG_C_API slang_str slang_syntax_kind_name(uint32_t kind);

/// The generated C++ struct that nodes of this kind are instances of, as a
/// struct ordinal (see slang_syntax_struct_name). Returns UINT32_MAX for kind 0
/// and out-of-range kinds. Many kinds share one struct.
SLANG_C_API uint32_t slang_syntax_kind_struct(uint32_t kind);

/// Number of generated syntax structs.
SLANG_C_API uint32_t slang_syntax_struct_count(void);

/// The name of a syntax struct, e.g. "BinaryExpressionSyntax". Borrowed, static.
SLANG_C_API slang_str slang_syntax_struct_name(uint32_t syntax_struct);

/// Number of declared members of a syntax struct, including inherited ones,
/// in the order they occupy child indices.
SLANG_C_API uint32_t slang_syntax_struct_member_count(uint32_t syntax_struct);

/// The form of a syntax struct member.
typedef enum slang_member_form {
    SLANG_MEMBER_TOKEN = 0,
    SLANG_MEMBER_NODE = 1,
    SLANG_MEMBER_OPTIONAL_NODE = 2,
    SLANG_MEMBER_LIST = 3,
    SLANG_MEMBER_SEPARATED_LIST = 4,
    SLANG_MEMBER_TOKEN_LIST = 5,
} slang_member_form;

/// The name of a member, e.g. "header". Borrowed, static.
SLANG_C_API slang_str slang_syntax_member_name(uint32_t syntax_struct, uint32_t member);

/// The form of a member.
SLANG_C_API slang_member_form slang_syntax_member_form(uint32_t syntax_struct, uint32_t member);

/// Number of token kinds. Kind 0 is always "Unknown".
SLANG_C_API uint32_t slang_token_kind_count(void);

/// The name of a token kind, e.g. "Identifier". Borrowed, static.
SLANG_C_API slang_str slang_token_kind_name(uint32_t kind);

/// The name of an AST kind within a domain, e.g. (SLANG_AST_SYMBOL, 5) ->
/// "Instance". Borrowed, static. Empty for out-of-range values.
SLANG_C_API slang_str slang_ast_kind_name(slang_ast_domain domain, uint32_t kind);

/// Number of kinds in an AST domain.
SLANG_C_API uint32_t slang_ast_kind_count(slang_ast_domain domain);

/* ------------------------------------------------------------------------- */
/* Source manager                                                             */
/* ------------------------------------------------------------------------- */

/// Creates a source manager. Thread-safe for concurrent use once created.
SLANG_C_API slang_source_manager slang_source_manager_create(slang_error* err);

/// Destroys a source manager. Every syntax tree and compilation created from it
/// must already be gone; locations from it become meaningless.
SLANG_C_API void slang_source_manager_destroy(slang_source_manager sm);

/// Adds a directory (or glob pattern) to search for `include files.
SLANG_C_API void slang_source_manager_add_include_dir(slang_source_manager sm, const char* pattern,
                                                      size_t pattern_len, bool system,
                                                      slang_error* err);

/// Registers in-memory text under a path, returning its buffer id. The text is
/// copied. Useful for editors and tests that never touch the filesystem.
SLANG_C_API slang_buffer_id slang_source_manager_assign_text(slang_source_manager sm,
                                                             const char* path, size_t path_len,
                                                             const char* text, size_t text_len,
                                                             slang_error* err);

/// Loads a file from disk, returning its buffer id (0 on failure, with err set).
SLANG_C_API slang_buffer_id slang_source_manager_read_file(slang_source_manager sm,
                                                           const char* path, size_t path_len,
                                                           slang_error* err);

/// The file name (as given when loaded) for a location. Borrowed from `sm`.
SLANG_C_API slang_str slang_source_manager_file_name(slang_source_manager sm, slang_loc loc);

/// The full text of a buffer. Borrowed from `sm`.
SLANG_C_API slang_str slang_source_manager_text(slang_source_manager sm, slang_buffer_id buffer);

/// 1-based line number of a location, resolving `line directives.
SLANG_C_API size_t slang_source_manager_line(slang_source_manager sm, slang_loc loc);

/// 1-based column number (in bytes) of a location.
SLANG_C_API size_t slang_source_manager_column(slang_source_manager sm, slang_loc loc);

/// True if the location is inside a macro expansion rather than a file.
SLANG_C_API bool slang_source_manager_is_macro_loc(slang_source_manager sm, slang_loc loc);

/// Walks a macro-expansion location back to the location of the macro use in
/// original source text. Identity for file locations.
SLANG_C_API slang_loc slang_source_manager_original_loc(slang_source_manager sm, slang_loc loc);

/* ------------------------------------------------------------------------- */
/* Options                                                                    */
/* ------------------------------------------------------------------------- */

/// Creates an option set with slang's defaults. Options are copied into the
/// objects they are passed to, so the set may be destroyed afterwards.
SLANG_C_API slang_options slang_options_create(slang_error* err);
SLANG_C_API void slang_options_destroy(slang_options options);

/// Language standard versions.
typedef enum slang_language_version {
    SLANG_LANGUAGE_1364_2005 = 0,
    SLANG_LANGUAGE_1800_2017 = 1,
    SLANG_LANGUAGE_1800_2023 = 2,
} slang_language_version;

SLANG_C_API void slang_options_set_language_version(slang_options options,
                                                    slang_language_version version);

/// Defines a preprocessor macro, e.g. name "WIDTH", value "8".
SLANG_C_API void slang_options_define(slang_options options, const char* name, size_t name_len,
                                      const char* value, size_t value_len, slang_error* err);

/// Names a module to treat as a top-level instance (may be called repeatedly).
SLANG_C_API void slang_options_add_top_module(slang_options options, const char* name,
                                              size_t name_len, slang_error* err);

/// Compilation flags. Values match slang's CompilationFlags and are append-only.
typedef enum slang_compilation_flag {
    SLANG_COMP_ALLOW_HIERARCHICAL_CONST = 1u << 0,
    SLANG_COMP_RELAX_ENUM_CONVERSIONS = 1u << 1,
    SLANG_COMP_ALLOW_USE_BEFORE_DECLARE = 1u << 2,
    SLANG_COMP_ALLOW_TOP_LEVEL_IFACE_PORTS = 1u << 3,
    SLANG_COMP_LINT_MODE = 1u << 4,
    SLANG_COMP_IGNORE_UNKNOWN_MODULES = 1u << 5,
    SLANG_COMP_RELAX_STRING_CONVERSIONS = 1u << 6,
    SLANG_COMP_ALLOW_RECURSIVE_IMPLICIT_CALL = 1u << 7,
    SLANG_COMP_ALLOW_BARE_VAL_PARAM_ASSIGNMENT = 1u << 8,
    SLANG_COMP_ALLOW_SELF_DETERMINED_STREAM_CONCAT = 1u << 9,
    SLANG_COMP_ALLOW_MERGING_ANSI_PORTS = 1u << 10,
    /// Elaborate every instance body even when an identical one has already
    /// been elaborated. Required for SLANG_FREEZE_ELABORATE_ALL to be total.
    SLANG_COMP_DISABLE_INSTANCE_CACHING = 1u << 11,
    SLANG_COMP_DISALLOW_REFS_TO_UNKNOWN_INSTANCES = 1u << 12,
    SLANG_COMP_ALLOW_UNNAMED_GENERATE = 1u << 13,
    SLANG_COMP_ALLOW_VIRTUAL_IFACE_WITH_OVERRIDE = 1u << 14,
    SLANG_COMP_ALLOW_ARRAY_CONCAT_ASSIGN_PATTERN = 1u << 15,
    SLANG_COMP_ALLOW_CROSS_AUTO_BIN_MAX = 1u << 16,
    SLANG_COMP_ALLOW_INVALID_TOP = 1u << 17,
    SLANG_COMP_CHECK_UNINSTANTIATED = 1u << 18,
} slang_compilation_flag;

/// Sets the compilation flags (a bitmask of slang_compilation_flag).
SLANG_C_API void slang_options_set_compilation_flags(slang_options options, uint32_t flags);

/* ------------------------------------------------------------------------- */
/* Syntax trees                                                               */
/* ------------------------------------------------------------------------- */

/// Parses source text as a full compilation unit. `name` labels the buffer in
/// diagnostics; `path` is used for `include resolution (both may be empty).
/// Parse errors do NOT fail this call; they are reported through
/// slang_syntax_tree_diagnostics. Failure means the tree could not be built.
SLANG_C_API slang_syntax_tree slang_syntax_tree_from_text(
    slang_source_manager sm, const char* text, size_t text_len, const char* name, size_t name_len,
    const char* path, size_t path_len, slang_options options /* nullable */, slang_error* err);

/// Parses a file from disk as a compilation unit.
SLANG_C_API slang_syntax_tree slang_syntax_tree_from_file(slang_source_manager sm, const char* path,
                                                          size_t path_len,
                                                          slang_options options /* nullable */,
                                                          slang_error* err);

/// Parses an already-loaded buffer (see slang_source_manager_assign_text).
SLANG_C_API slang_syntax_tree slang_syntax_tree_from_buffer(slang_source_manager sm,
                                                            slang_buffer_id buffer,
                                                            slang_options options /* nullable */,
                                                            slang_error* err);

/// Increments the tree's reference count. Trees are shared between a caller and
/// any compilations they are added to; each holder releases independently.
SLANG_C_API slang_syntax_tree slang_syntax_tree_retain(slang_syntax_tree tree);

/// Decrements the reference count, destroying the tree at zero. Every
/// slang_node and slang_token from it becomes invalid at that point.
SLANG_C_API void slang_syntax_tree_release(slang_syntax_tree tree);

/// The root node of the tree.
SLANG_C_API slang_node slang_syntax_tree_root(slang_syntax_tree tree);

/// Diagnostics produced while lexing, preprocessing and parsing this tree.
/// Owned by the caller; free with slang_diagnostics_destroy.
SLANG_C_API slang_diagnostics slang_syntax_tree_diagnostics(slang_syntax_tree tree,
                                                            slang_error* err);

/// The source manager the tree was parsed with (borrowed).
SLANG_C_API slang_source_manager slang_syntax_tree_source_manager(slang_syntax_tree tree);

/// Reconstructs the original source file text from the tree: trivia,
/// directives and skipped text included, macro uses and `include directives
/// left as written rather than expanded. Byte-identical to the input for an
/// unmodified tree. Owned. (slang_node_to_string on the root gives the
/// expanded token stream instead.)
SLANG_C_API slang_str slang_syntax_tree_to_string(slang_syntax_tree tree, slang_error* err);

/* ------------------------------------------------------------------------- */
/* Syntax nodes                                                               */
/* ------------------------------------------------------------------------- */

/// True for the null node returned by accessors when a child is absent.
SLANG_C_API bool slang_node_is_null(slang_node node);

/// The parent node, or the null node for the root.
SLANG_C_API slang_node slang_node_parent(slang_node node);

/// The struct ordinal of this node's concrete type (see slang_syntax_struct_name).
SLANG_C_API uint32_t slang_node_struct(slang_node node);

/// Number of direct children (nodes and tokens, including absent optional slots).
SLANG_C_API uint32_t slang_node_child_count(slang_node node);

/// What occupies a child slot.
typedef enum slang_child_tag {
    /// An optional child that is absent.
    SLANG_CHILD_NONE = 0,
    SLANG_CHILD_NODE = 1,
    SLANG_CHILD_TOKEN = 2,
} slang_child_tag;

/// Fetches child `index`. Exactly one of `*node_out` / `*token_out` is written
/// according to the returned tag; either out pointer may be null. Returns
/// SLANG_CHILD_NONE for an out-of-range index as well as for an absent child.
SLANG_C_API slang_child_tag slang_node_child(slang_node node, uint32_t index, slang_node* node_out,
                                             slang_token* token_out);

/// The contiguous range of child indices occupied by the struct member
/// `member` (an ordinal into slang_syntax_struct_member_count for this node's
/// struct). Scalar members occupy one slot; list members occupy as many slots
/// as they have elements (separated lists count separators). This is computed
/// by the same generated code that implements child indexing, so it is always
/// consistent with slang_node_child. Returns false for an invalid member.
SLANG_C_API bool slang_node_member_span(slang_node node, uint32_t member, uint32_t* start_out,
                                        uint32_t* len_out);

/// The source range covered by the node, excluding leading trivia.
SLANG_C_API slang_range slang_node_range(slang_node node);

/// Prints the node's token stream exactly as the tree holds it: all trivia,
/// directives and skipped text, with macros and includes expanded. This is
/// the text slang_syntax_tree_walk streams for the same subtree. Owned.
SLANG_C_API slang_str slang_node_to_string(slang_node node, slang_error* err);

/// The first / last token in the subtree. The token is "missing" flagged and
/// kind 0 when the subtree has no tokens.
SLANG_C_API slang_token slang_node_first_token(slang_node node);
SLANG_C_API slang_token slang_node_last_token(slang_node node);

/// Structural equivalence of two subtrees (same kinds, same token kinds and
/// value text; trivia ignored).
SLANG_C_API bool slang_node_is_equivalent(slang_node a, slang_node b);

/* ------------------------------------------------------------------------- */
/* Tokens                                                                     */
/* ------------------------------------------------------------------------- */

/// The token's location (start of its text, after leading trivia).
SLANG_C_API slang_loc slang_token_location(slang_token token);

/// The token's source range, excluding trivia.
SLANG_C_API slang_range slang_token_range(slang_token token);

/// The exact source text of the token (no trivia). Borrowed from the tree.
SLANG_C_API slang_str slang_token_raw_text(slang_token token);

/// The token's semantic value text: for string literals the unescaped value,
/// for identifiers the name. Borrowed from the tree.
SLANG_C_API slang_str slang_token_value_text(slang_token token);

/// Number of trivia items (whitespace, comments, directives) preceding the token.
SLANG_C_API uint32_t slang_token_trivia_count(slang_token token);

/// A piece of trivia. `kind` is a trivia kind ordinal (see slang_trivia_kind_name).
/// `text` is borrowed for plain trivia (whitespace, comments, newlines) and
/// OWNED for trivia that wraps structure (preprocessor directives, skipped
/// tokens), where it is the printed form of that structure; always pass it to
/// slang_str_free, which is a no-op for the borrowed case.
typedef struct slang_trivia {
    uint32_t kind;
    uint32_t reserved_;
    slang_str text;
} slang_trivia;

/// Fetches trivia item `index`. Returns false if out of range.
SLANG_C_API bool slang_token_trivia(slang_token token, uint32_t index, slang_trivia* out);

/// The syntax of a directive trivia item (e.g. a `define with its body), or
/// the null node for trivia that carries no structure.
SLANG_C_API slang_node slang_token_trivia_syntax(slang_token token, uint32_t index);

/// Number of trivia kinds, and their names. Borrowed, static.
SLANG_C_API uint32_t slang_trivia_kind_count(void);
SLANG_C_API slang_str slang_trivia_kind_name(uint32_t kind);

/* ------------------------------------------------------------------------- */
/* Syntax traversal                                                           */
/* ------------------------------------------------------------------------- */

/// Result of a visitor callback.
typedef enum slang_visit {
    /// Stop the whole traversal; the traversing call reports SLANG_ERR_CANCELLED.
    SLANG_VISIT_BREAK = 0,
    /// Continue, descending into this node's children.
    SLANG_VISIT_CONTINUE = 1,
    /// Continue, but do not descend into this node's children.
    SLANG_VISIT_SKIP = 2,
} slang_visit;

/// Callback for slang_node_visit. Must not unwind into the library.
typedef slang_visit (*slang_node_visitor)(slang_node node, void* user);

/// Pre-order traversal of the subtree rooted at `node`. The callback is invoked
/// for `node` itself first. Tokens are not visited; walk them via children.
SLANG_C_API void slang_node_visit(slang_node node, slang_node_visitor visitor, void* user,
                                  slang_error* err);

/// A streaming event sink for slang_syntax_tree_walk. Events arrive in source
/// order as a depth-first walk: start_node, then exactly one event per
/// declared MEMBER of the node's struct (see slang_syntax_struct_member_count),
/// then finish_node. A member produces a nested start_node/finish_node pair,
/// a token preceded by its trivia, `absent` for an empty optional slot, or a
/// start_list/finish_list pair enclosing the list's elements (with separator
/// tokens interleaved for separated lists). Because every member produces one
/// event, the k-th child event of a node is member k of its struct, and the
/// concatenated trivia and token text rebuilds the token stream exactly. All
/// callbacks must be non-null and must not unwind into the library.
typedef struct slang_syntax_sink {
    void (*start_node)(void* user, uint32_t kind, uint32_t syntax_struct);
    void (*trivia)(void* user, uint32_t kind, const char* text, size_t len);
    void (*token)(void* user, uint32_t kind, const char* text, size_t len, slang_loc loc,
                  uint16_t flags);
    void (*absent)(void* user);
    void (*start_list)(void* user, slang_member_form form);
    void (*finish_list)(void* user);
    void (*finish_node)(void* user);
} slang_syntax_sink;

/// Streams the whole tree through `sink`. This is the fastest way to mirror a
/// tree into another representation: one call, no per-node round trips.
SLANG_C_API void slang_syntax_tree_walk(slang_syntax_tree tree, const slang_syntax_sink* sink,
                                        void* user, slang_error* err);

/* ------------------------------------------------------------------------- */
/* Diagnostics                                                                */
/* ------------------------------------------------------------------------- */

/// Severity of a reported diagnostic.
typedef enum slang_severity {
    SLANG_SEVERITY_IGNORED = 0,
    SLANG_SEVERITY_NOTE = 1,
    SLANG_SEVERITY_WARNING = 2,
    SLANG_SEVERITY_ERROR = 3,
    SLANG_SEVERITY_FATAL = 4,
} slang_severity;

/// One diagnostic. `code` packs the subsystem in the high 16 bits and the
/// per-subsystem index in the low 16 bits, matching slang's DiagCode; the
/// diagnostics model maps it to a name, default severity and -W option.
typedef struct slang_diag {
    uint32_t code;
    slang_severity severity;
    slang_loc location;
    /// Number of attached notes and highlighted ranges; fetch with
    /// slang_diagnostics_note / slang_diagnostics_range.
    uint32_t note_count;
    uint32_t range_count;
} slang_diag;

/// Frees a diagnostics list.
SLANG_C_API void slang_diagnostics_destroy(slang_diagnostics diags);

/// Number of top-level diagnostics.
SLANG_C_API uint32_t slang_diagnostics_count(slang_diagnostics diags);

/// Fetches diagnostic `index`. Returns false if out of range.
SLANG_C_API bool slang_diagnostics_at(slang_diagnostics diags, uint32_t index, slang_diag* out);

/// The fully formatted message of diagnostic `index`, with arguments
/// substituted (e.g. "unknown module 'foo'"). Owned.
SLANG_C_API slang_str slang_diagnostics_message(slang_diagnostics diags, uint32_t index,
                                                slang_error* err);

/// Fetches note `note` attached to diagnostic `index`. Returns false if out of range.
SLANG_C_API bool slang_diagnostics_note(slang_diagnostics diags, uint32_t index, uint32_t note,
                                        slang_diag* out);

/// The formatted message of a note. Owned.
SLANG_C_API slang_str slang_diagnostics_note_message(slang_diagnostics diags, uint32_t index,
                                                     uint32_t note, slang_error* err);

/// Fetches highlighted source range `range` of diagnostic `index`.
SLANG_C_API bool slang_diagnostics_range(slang_diagnostics diags, uint32_t index, uint32_t range,
                                         slang_range* out);

/// The AST symbol a diagnostic was reported against, if any (null ptr if none
/// or if the list did not come from a compilation).
SLANG_C_API slang_ast slang_diagnostics_symbol(slang_diagnostics diags, uint32_t index);

/// Rendering options for slang_diagnostics_render.
typedef struct slang_render_options {
    /// Emit ANSI color escapes.
    bool colors;
    /// Include the source line and caret/underline for each diagnostic.
    bool show_source;
    /// Include the `include stack and macro expansion notes.
    bool show_include_stack;
    /// Show absolute rather than as-given file paths.
    bool absolute_paths;
} slang_render_options;

/// Renders the whole list as slang's own text diagnostic output, e.g.
///   file.sv:3:5: error: unknown module 'foo'
///       foo f();
///       ^~~
/// `options` may be null for defaults. Owned.
SLANG_C_API slang_str slang_diagnostics_render(slang_diagnostics diags,
                                               const slang_render_options* options /* nullable */,
                                               slang_error* err);

/* ------------------------------------------------------------------------- */
/* Compilation                                                                */
/* ------------------------------------------------------------------------- */

/// Creates an empty compilation. `options` may be null for defaults.
SLANG_C_API slang_compilation slang_compilation_create(slang_options options /* nullable */,
                                                       slang_error* err);

/// Destroys a compilation. Every slang_ast from it becomes invalid. Syntax trees
/// that were added are released (the caller's own retain, if any, remains).
SLANG_C_API void slang_compilation_destroy(slang_compilation comp);

/// Adds a syntax tree. The compilation retains the tree. Fails with
/// SLANG_ERR_INVALID_STATE once the compilation has been finalized.
SLANG_C_API void slang_compilation_add_tree(slang_compilation comp, slang_syntax_tree tree,
                                            slang_error* err);

/// Flags for slang_compilation_freeze.
typedef enum slang_freeze_flag {
    /// Elaborate every scope, including the bodies of instances that are
    /// structurally identical to an already-elaborated one (which slang would
    /// otherwise skip). Without this, some scopes remain lazily elaborated
    /// after freezing and touching them from two threads is a data race.
    /// Requires SLANG_COMP_DISABLE_INSTANCE_CACHING to have been set on the
    /// compilation's options; fails with SLANG_ERR_INVALID_ARG otherwise.
    SLANG_FREEZE_ELABORATE_ALL = 1u << 0,
    /// Constant-fold every expression now, so that later evaluation from the
    /// (logically const) read path finds a cached value and never allocates.
    /// Expressions that cannot be folded are counted in the report.
    SLANG_FREEZE_PREFOLD = 1u << 1,
    /// Seal the compilation: further allocation into it is rejected. Unlike
    /// slang's own internal freeze flag, this is enforced in every build.
    SLANG_FREEZE_SEAL = 1u << 2,

    SLANG_FREEZE_ALL = SLANG_FREEZE_ELABORATE_ALL | SLANG_FREEZE_PREFOLD | SLANG_FREEZE_SEAL,
} slang_freeze_flag;

/// What slang_compilation_freeze did.
typedef struct slang_freeze_report {
    /// Symbols visited by the elaboration sweep.
    uint64_t symbols_elaborated;
    /// Expressions visited by the prefold sweep.
    uint64_t expressions_visited;
    /// Expressions that now carry a cached constant value.
    uint64_t expressions_folded;
    /// Expressions the prefold sweep could not attempt (no enclosing scope) or
    /// whose evaluation threw. Every other expression either has a cached
    /// value or has been proven non-constant, and slang caches nothing further
    /// for those; but a later evaluation of one of THESE may still allocate,
    /// so if this is non-zero the compilation is NOT safe to evaluate
    /// expressions on from multiple threads concurrently (reading structure
    /// remains safe).
    uint64_t fold_failures;
    /// Declared types + canonical-type memos forced by the sweep. Forcing these
    /// is what makes concurrent reads of alias/canonical types race-free.
    uint64_t types_canonicalized;
    /// Parameter/specparam constant values forced by the sweep.
    uint64_t params_folded;
} slang_freeze_report;

/// Finalizes and fully elaborates the compilation, then applies `flags`
/// (a bitmask of slang_freeze_flag). After this call with SLANG_FREEZE_ALL and
/// a zero `fold_failures`, every read-only accessor in this API may be called
/// on the compilation from any number of threads concurrently. Idempotent;
/// `report` may be null.
SLANG_C_API void slang_compilation_freeze(slang_compilation comp, uint32_t flags,
                                          slang_freeze_report* report /* nullable */,
                                          slang_error* err);

/// True once slang_compilation_freeze has been called with SLANG_FREEZE_SEAL.
SLANG_C_API bool slang_compilation_is_sealed(slang_compilation comp);

/// The root symbol of the design. Finalizes the compilation if it has not been.
SLANG_C_API slang_ast slang_compilation_root(slang_compilation comp, slang_error* err);

/// All diagnostics (parse and semantic). Forces full elaboration. Owned.
SLANG_C_API slang_diagnostics slang_compilation_diagnostics(slang_compilation comp,
                                                            slang_error* err);

/// The source manager shared by the compilation's trees (borrowed; null if no
/// trees have been added).
SLANG_C_API slang_source_manager slang_compilation_source_manager(slang_compilation comp);

/// Number of top-level (uninstantiated-by-others) module/program instances.
SLANG_C_API uint32_t slang_compilation_top_instance_count(slang_compilation comp);

/// Fetches top-level instance `index` (a symbol of kind Instance).
SLANG_C_API slang_ast slang_compilation_top_instance(slang_compilation comp, uint32_t index);

/// Number of definitions (modules, interfaces, programs) across all libraries,
/// in a deterministic order.
SLANG_C_API uint32_t slang_compilation_definition_count(slang_compilation comp);
SLANG_C_API slang_ast slang_compilation_definition(slang_compilation comp, uint32_t index);

/// Number of packages, and each package symbol.
SLANG_C_API uint32_t slang_compilation_package_count(slang_compilation comp);
SLANG_C_API slang_ast slang_compilation_package(slang_compilation comp, uint32_t index);

/* ------------------------------------------------------------------------- */
/* AST: generic                                                               */
/* ------------------------------------------------------------------------- */

/// True for the null AST node returned when something is absent.
SLANG_C_API bool slang_ast_is_null(slang_ast node);

/// The source range of an expression, statement, timing control, etc. For
/// symbols this is the range of their declaring syntax when known.
SLANG_C_API slang_range slang_ast_range(slang_ast node);

/// The syntax node this AST node was created from, or the null node.
SLANG_C_API slang_node slang_ast_syntax(slang_ast node);

/// Callback for slang_ast_visit. Must not unwind into the library.
typedef slang_visit (*slang_ast_visitor)(slang_ast node, slang_ast parent, void* user);

/// Pre-order traversal of the AST under `root` across all domains: symbols,
/// their expressions and statements, nested scopes' members, and so on. Only
/// nodes that would be visited by slang's own visitor are visited (uninstantiated
/// bodies are skipped).
SLANG_C_API void slang_ast_visit(slang_ast root, slang_ast_visitor visitor, void* user,
                                 slang_error* err);

/* ------------------------------------------------------------------------- */
/* AST: symbols and scopes                                                    */
/* ------------------------------------------------------------------------- */

/// The symbol's name (may be empty for anonymous symbols). Borrowed from the
/// compilation.
SLANG_C_API slang_str slang_symbol_name(slang_ast symbol);

/// The symbol's declaration location.
SLANG_C_API slang_loc slang_symbol_location(slang_ast symbol);

/// The scope that contains this symbol, or the null node for the root.
SLANG_C_API slang_ast slang_symbol_parent_scope(slang_ast symbol);

/// The next symbol in the same scope, or the null node at the end. Together
/// with slang_scope_first_member this iterates a scope without any per-call
/// allocation.
SLANG_C_API slang_ast slang_symbol_next_sibling(slang_ast symbol);

/// The symbol's full hierarchical path, e.g. "top.cpu.alu". Owned.
SLANG_C_API slang_str slang_symbol_hierarchical_path(slang_ast symbol, slang_error* err);

/// True if the symbol is a scope (module body, package, class, block, ...).
SLANG_C_API bool slang_symbol_is_scope(slang_ast symbol);

/// True if the symbol is a type.
SLANG_C_API bool slang_symbol_is_type(slang_ast symbol);

/// True if the symbol has a value (variable, net, parameter, port, ...).
SLANG_C_API bool slang_symbol_is_value(slang_ast symbol);

/// The first member of a scope, or the null node if empty. This forces the
/// scope's lazy elaboration if it has not happened yet, which mutates the
/// compilation; it is safe to call concurrently only after
/// slang_compilation_freeze with SLANG_FREEZE_ELABORATE_ALL.
SLANG_C_API slang_ast slang_scope_first_member(slang_ast scope, slang_error* err);

/// Finds a direct member by name, without name-lookup rules (no imports, no
/// upward search). Null node if absent. This is a pure read of the scope's
/// name table and is safe from any thread on a frozen compilation.
SLANG_C_API slang_ast slang_scope_find(slang_ast scope, const char* name, size_t name_len,
                                       slang_error* err);

/// Looks up a name from the scope using full SystemVerilog name lookup rules
/// (imports, enclosing scopes, $unit, and hierarchical names with dots). Null
/// node if not found. Lookup diagnostics are discarded.
///
/// slang records reference-tracking state during name lookup (for its
/// unused-declaration lints), so unlike slang_scope_find this is NOT a pure
/// read: on a sealed compilation the seal is lifted for the call and the
/// caller must guarantee exclusive access; concurrent calls of any kind are a
/// data race. Prefer slang_scope_find plus explicit navigation when reading
/// from multiple threads.
SLANG_C_API slang_ast slang_scope_lookup(slang_ast scope, const char* name, size_t name_len,
                                         slang_error* err);

/// The declared type of a value symbol. Null node if the symbol has no type.
SLANG_C_API slang_ast slang_value_type(slang_ast symbol, slang_error* err);

/// The initializer expression of a value symbol, or the null node.
SLANG_C_API slang_ast slang_value_initializer(slang_ast symbol, slang_error* err);

/// For an Instance symbol: its body (an InstanceBody symbol, which is a scope).
SLANG_C_API slang_ast slang_instance_body(slang_ast instance);

/// For an Instance symbol: the Definition it instantiates.
SLANG_C_API slang_ast slang_instance_definition(slang_ast instance);

/// For an Instance symbol: the number of parameters (port and body, value and
/// type) of its body, and each one as a Parameter or TypeParameter symbol.
SLANG_C_API uint32_t slang_instance_parameter_count(slang_ast instance);
SLANG_C_API slang_ast slang_instance_parameter(slang_ast instance, uint32_t index);

/// For a Parameter symbol, its value printed as SystemVerilog; for a
/// TypeParameter symbol, its target type printed as SystemVerilog. Owned.
SLANG_C_API slang_str slang_parameter_value(slang_ast parameter, slang_error* err);

/// Definition kinds, matching slang's DefinitionKind.
typedef enum slang_definition_kind {
    SLANG_DEFINITION_MODULE = 0,
    SLANG_DEFINITION_INTERFACE = 1,
    SLANG_DEFINITION_PROGRAM = 2,
} slang_definition_kind;

/// For a Definition symbol: whether it is a module, interface or program.
SLANG_C_API slang_definition_kind slang_definition_kind_of(slang_ast definition);

/* ------------------------------------------------------------------------- */
/* AST: types                                                                 */
/* ------------------------------------------------------------------------- */

/// The canonical type: typedefs and type parameters resolved.
SLANG_C_API slang_ast slang_type_canonical(slang_ast type);

/// The type printed in SystemVerilog syntax, e.g. "logic[7:0]". Owned.
SLANG_C_API slang_str slang_type_to_string(slang_ast type, slang_error* err);

/// Width in bits of an integral (packed) type; 0 for non-integral types.
SLANG_C_API uint64_t slang_type_bit_width(slang_ast type);

SLANG_C_API bool slang_type_is_integral(slang_ast type);
SLANG_C_API bool slang_type_is_signed(slang_ast type);
SLANG_C_API bool slang_type_is_four_state(slang_ast type);
SLANG_C_API bool slang_type_is_unpacked_array(slang_ast type);
SLANG_C_API bool slang_type_is_class(slang_ast type);

/// Structural classification (each canonicalizes first, so typedefs are seen
/// through). is_array covers every packed and unpacked array kind.
SLANG_C_API bool slang_type_is_enum(slang_ast type);
SLANG_C_API bool slang_type_is_struct(slang_ast type);
SLANG_C_API bool slang_type_is_union(slang_ast type);
SLANG_C_API bool slang_type_is_array(slang_ast type);
SLANG_C_API bool slang_type_is_string(slang_ast type);

/// For any array type (packed or unpacked), its element type; a null node
/// otherwise. Canonicalizes first.
SLANG_C_API slang_ast slang_type_array_element(slang_ast type);

/// For an enum type, its underlying base type; a null node otherwise.
SLANG_C_API slang_ast slang_type_enum_base(slang_ast type);

/// The members of an enum type, each an EnumValue symbol. Count is 0 for a
/// non-enum type; slang_enum_member returns a null node when out of range.
SLANG_C_API uint32_t slang_enum_member_count(slang_ast type);
SLANG_C_API slang_ast slang_enum_member(slang_ast type, uint32_t index);

/// For an EnumValue symbol, its constant value printed as SystemVerilog. Owned.
SLANG_C_API slang_str slang_enum_member_value(slang_ast member, slang_error* err);

/// The fields of a struct or union type (packed or unpacked), each a Field
/// symbol. Count is 0 for any other type; slang_type_field returns a null node
/// when out of range. Read a field's type with slang_value_type.
SLANG_C_API uint32_t slang_type_field_count(slang_ast type);
SLANG_C_API slang_ast slang_type_field(slang_ast type, uint32_t index);

/// For a Field symbol, its bit offset within the parent struct/union, and its
/// index in declaration order. Both 0 for a non-field symbol.
SLANG_C_API uint64_t slang_field_bit_offset(slang_ast field);
SLANG_C_API uint32_t slang_field_index(slang_ast field);

/// For a class type, its base class type (if it derives from one); a null node
/// otherwise.
SLANG_C_API slang_ast slang_type_class_base(slang_ast type);

/// Type relations from IEEE 1800 §6.22.
SLANG_C_API bool slang_type_is_matching(slang_ast a, slang_ast b);
SLANG_C_API bool slang_type_is_equivalent(slang_ast a, slang_ast b);
SLANG_C_API bool slang_type_is_assignment_compatible(slang_ast a, slang_ast b);

/* ------------------------------------------------------------------------- */
/* AST: expressions and statements                                            */
/* ------------------------------------------------------------------------- */

/// The type of an expression.
SLANG_C_API slang_ast slang_expression_type(slang_ast expr);

/// True if the expression is invalid (had errors).
SLANG_C_API bool slang_expression_is_bad(slang_ast expr);

/// The symbol an expression refers to (for name references and member
/// accesses), or the null node.
SLANG_C_API slang_ast slang_expression_symbol(slang_ast expr);

/// For an expression that has already been constant-folded (see
/// SLANG_FREEZE_PREFOLD), its value printed as SystemVerilog, e.g. "32'd8" or
/// "'{1, 2}". Returns false if no cached value exists. This never evaluates,
/// so it is safe from any thread on a frozen compilation. Owned.
SLANG_C_API bool slang_expression_cached_constant(slang_ast expr, slang_str* out, slang_error* err);

/// Evaluates an expression as a constant now, returning it printed as
/// SystemVerilog. This MAY allocate into the compilation (the seal is lifted
/// for the duration of the call) and the caller must therefore guarantee
/// exclusive access to the compilation; concurrent calls of any kind are a
/// data race. Returns false if the expression is not constant. Owned.
SLANG_C_API bool slang_expression_eval(slang_ast expr, slang_str* out, slang_error* err);

/* ------------------------------------------------------------------------- */
/* Constant values                                                            */
/* ------------------------------------------------------------------------- */

/* A folded/evaluated constant, as structured data rather than a string. An
 * owned handle (free with slang_constant_destroy); an SVInt or element handle
 * borrows from its parent constant and is valid while the parent lives. */
typedef struct slang_constant_t* slang_constant;
typedef struct slang_svint_t* slang_svint; /* borrows from a slang_constant */

typedef enum slang_constant_kind {
    SLANG_CONSTANT_BAD = 0,
    SLANG_CONSTANT_INTEGER = 1,
    SLANG_CONSTANT_REAL = 2,
    SLANG_CONSTANT_SHORTREAL = 3,
    SLANG_CONSTANT_STRING = 4,
    SLANG_CONSTANT_NULL = 5,
    SLANG_CONSTANT_UNBOUNDED = 6,
    SLANG_CONSTANT_UNPACKED = 7, /* fixed/dynamic array */
    SLANG_CONSTANT_MAP = 8,
    SLANG_CONSTANT_QUEUE = 9,
    SLANG_CONSTANT_UNION = 10,
} slang_constant_kind;

/// A flat fast-path view of a single-word integer constant (see
/// slang_constant_flat_int). Only valid when that returns true.
typedef struct slang_svint_flat {
    int64_t value;
    uint32_t bit_width;
    bool is_signed;
    bool has_unknown;
} slang_svint_flat;

/// The already-folded constant value of an expression (see SLANG_FREEZE_PREFOLD),
/// as structured data, or NULL if it has none. A pure read (never evaluates).
SLANG_C_API slang_constant slang_expression_constant_value(slang_ast expr, slang_error* err);

/// Evaluates an expression now and returns its structured constant value, or
/// NULL if not constant. MAY allocate (the seal is lifted); caller must hold
/// exclusive access, as with slang_expression_eval.
SLANG_C_API slang_constant slang_expression_eval_constant(slang_ast expr, slang_error* err);

/// Frees a constant produced by the two functions above (and by
/// slang_constant_element).
SLANG_C_API void slang_constant_destroy(slang_constant c);

/// The constant's kind (see slang_constant_kind).
SLANG_C_API uint32_t slang_constant_kind_of(slang_constant c);

/// True if the constant contains any unknown (x/z) bits.
SLANG_C_API bool slang_constant_has_unknown(slang_constant c);

/// For a real/shortreal constant, its value; false otherwise.
SLANG_C_API bool slang_constant_real(slang_constant c, double* out);

/// For a string constant, its text (borrowed empty otherwise).
SLANG_C_API slang_str slang_constant_string(slang_constant c);

/// For an unpacked array/queue constant, its element count (0 otherwise).
SLANG_C_API uint64_t slang_constant_size(slang_constant c);

/// The i'th element of an array/queue constant, as a new owned constant (NULL
/// if out of range).
SLANG_C_API slang_constant slang_constant_element(slang_constant c, uint64_t index,
                                                  slang_error* err);

/// The constant printed as SystemVerilog (parity with the string API). Owned.
SLANG_C_API slang_str slang_constant_to_string(slang_constant c);

/// For an integer constant, a borrowed SVInt handle (NULL otherwise).
SLANG_C_API slang_svint slang_constant_integer(slang_constant c);

/// Fast path: fills a flat view for a single-word (<=64-bit, no x/z) integer
/// constant and returns true; false for wide/unknown/non-integer constants
/// (use the slang_svint_* accessors then).
SLANG_C_API bool slang_constant_flat_int(slang_constant c, slang_svint_flat* out);

/// The bit width of an SVInt.
SLANG_C_API uint32_t slang_svint_bit_width(slang_svint v);
/// Whether the SVInt is signed.
SLANG_C_API bool slang_svint_is_signed(slang_svint v);
/// Whether the SVInt has any unknown (x/z) bits.
SLANG_C_API bool slang_svint_has_unknown(slang_svint v);
/// The SVInt as an i64, or false if it has unknown bits or does not fit.
SLANG_C_API bool slang_svint_as_i64(slang_svint v, int64_t* out);
/// The SVInt as a u64, or false if it has unknown bits or does not fit.
SLANG_C_API bool slang_svint_as_u64(slang_svint v, uint64_t* out);
/// The i'th bit: 0, 1, 2 (x), or 3 (z). Out-of-range returns 0.
SLANG_C_API uint8_t slang_svint_get_bit(slang_svint v, uint32_t index);
/// The SVInt printed as SystemVerilog (e.g. "8'hff"). Owned.
SLANG_C_API slang_str slang_svint_to_string(slang_svint v);

/* ------------------------------------------------------------------------- */
/* Semantic statement / expression tree                                       */
/* ------------------------------------------------------------------------- */

/* The elaborated behavioral tree: procedural blocks and subroutines have a
 * root Statement; statements and expressions have child statements/expressions.
 * Walk generically with slang_ast_sem_child_count/slang_ast_sem_child (children
 * come back in slang's own visitation order, e.g. a BinaryOp yields {left,
 * right}), read the kind with slang_ast_kind_name, and read operators with the
 * typed accessors below. All are pure reads on a frozen compilation. */

/// The root Statement of a procedural block (`always`/`initial`/`final`) or a
/// subroutine (`function`/`task`) symbol. Returns a null ast (domain STATEMENT)
/// for any other symbol.
SLANG_C_API slang_ast slang_symbol_body(slang_ast sym);

/// The number of immediate semantic children (sub-statements and
/// sub-expressions) of an AST node, in visitation order.
SLANG_C_API uint32_t slang_ast_sem_child_count(slang_ast node);

/// The i'th immediate semantic child of an AST node (see
/// slang_ast_sem_child_count). Returns a null ast if out of range.
SLANG_C_API slang_ast slang_ast_sem_child(slang_ast node, uint32_t index);

/// The BinaryOperator of a BinaryOp expression, as its enum ordinal (0 for a
/// non-binary node). Ordinals follow slang's BinaryOperator enum.
SLANG_C_API uint32_t slang_expr_binary_op(slang_ast node);

/// The UnaryOperator of a UnaryOp expression, as its enum ordinal (0 for a
/// non-unary node). Ordinals follow slang's UnaryOperator enum.
SLANG_C_API uint32_t slang_expr_unary_op(slang_ast node);

/// True if an Assignment expression is a non-blocking assignment (`<=`); false
/// for a blocking assignment or a non-assignment node.
SLANG_C_API bool slang_expr_assignment_is_nonblocking(slang_ast node);

/// The subroutine symbol called by a Call expression, or a null ast for a
/// system call or a non-call node.
SLANG_C_API slang_ast slang_expr_call_subroutine(slang_ast node);

/// The member symbol accessed by a MemberAccess expression (e.g. `s.field`), or
/// a null ast for a non-member-access node.
SLANG_C_API slang_ast slang_expr_member_symbol(slang_ast node);

/* Typed by-name accessors for statement and expression children. Each returns
 * the named child of the node when it is of the matching kind, and a null ast
 * (of the appropriate domain) otherwise. They are conveniences over the generic
 * slang_ast_sem_child enumeration; all are pure reads on a frozen compilation. */

/// The then-branch (`ifTrue`) statement of a Conditional (`if`) statement.
SLANG_C_API slang_ast slang_stmt_then_branch(slang_ast node);

/// The else-branch (`ifFalse`) statement of a Conditional statement, or a null
/// ast if there is no `else`.
SLANG_C_API slang_ast slang_stmt_else_branch(slang_ast node);

/// The body statement of a loop (`for`/`repeat`/`while`/`do-while`/`forever`/
/// `foreach`) or the guarded statement of a Timed or Wait statement.
SLANG_C_API slang_ast slang_stmt_body(slang_ast node);

/// The controlling expression of a statement: the `while`/`do-while`/`wait`
/// condition, the `repeat` count, or the `for` stop expression.
SLANG_C_API slang_ast slang_stmt_cond(slang_ast node);

/// The principal expression of a statement: an ExpressionStatement's
/// expression, a `case` selector, a `return` value (if any), or a `foreach`
/// array reference.
SLANG_C_API slang_ast slang_stmt_expr(slang_ast node);

/// The timing control of a Timed statement (an `@(...)`/`#`-delayed statement).
SLANG_C_API slang_ast slang_stmt_timing(slang_ast node);

/// The true-/false-value operands of a ConditionalOp expression (`c ? t : f`).
SLANG_C_API slang_ast slang_expr_cond_true(slang_ast node);
SLANG_C_API slang_ast slang_expr_cond_false(slang_ast node);

/// The base value of an ElementSelect or RangeSelect expression (`v[...]`).
SLANG_C_API slang_ast slang_expr_select_value(slang_ast node);

/// The index selector of an ElementSelect expression (`v[i]`).
SLANG_C_API slang_ast slang_expr_select_selector(slang_ast node);

/// The two range bounds of a RangeSelect expression (`v[l:r]`, `v[b+:w]`, ...).
SLANG_C_API slang_ast slang_expr_range_left(slang_ast node);
SLANG_C_API slang_ast slang_expr_range_right(slang_ast node);

/// The RangeSelectionKind of a RangeSelect expression as an enum ordinal
/// (Simple/IndexedUp/IndexedDown); 0 for a non-range-select node.
SLANG_C_API uint32_t slang_expr_range_selection_kind(slang_ast node);

/// The operand of a Conversion expression (a cast or implicit conversion).
SLANG_C_API slang_ast slang_expr_conversion_operand(slang_ast node);

/// The ConversionKind of a Conversion expression as an enum ordinal; 0 for a
/// non-conversion node.
SLANG_C_API uint32_t slang_expr_conversion_kind(slang_ast node);

/// The count and concatenation operands of a Replication expression (`{n{x}}`).
SLANG_C_API slang_ast slang_expr_replication_count(slang_ast node);
SLANG_C_API slang_ast slang_expr_replication_concat(slang_ast node);

/* ------------------------------------------------------------------------- */
/* Driver                                                                     */
/* ------------------------------------------------------------------------- */

/// A driver bundles slang's standard command line (file lists, `-I`, `-D`,
/// `--top`, `-W...`, diagnostics formatting, ...) with the source manager,
/// syntax trees and compilation it produces, exactly as the `slang` binary
/// does. Tools that want to accept the same flags as slang use this instead of
/// assembling options by hand.
typedef struct slang_driver_t* slang_driver;

/// Creates a driver with slang's standard arguments registered.
SLANG_C_API slang_driver slang_driver_create(slang_error* err);

/// Destroys a driver, its source manager, its trees, and any compilation it
/// created; positions obtained from any of them become invalid.
SLANG_C_API void slang_driver_destroy(slang_driver driver);

/// The value type of a custom command-line option.
typedef enum slang_option_kind {
    /// A boolean switch (`--flag` / `--no-flag`).
    SLANG_OPTION_FLAG = 0,
    SLANG_OPTION_INT = 1,
    SLANG_OPTION_STRING = 2,
} slang_option_kind;

/// Registers a custom option. `names` is a comma-separated list of spellings,
/// e.g. "-h,--help"; the same string is used to read the value back.
/// `value_name` (e.g. "<depth>") is shown in help text and may be empty.
SLANG_C_API void slang_driver_add_option(slang_driver driver, const char* names, size_t names_len,
                                         slang_option_kind kind, const char* description,
                                         size_t description_len, const char* value_name,
                                         size_t value_name_len, slang_error* err);

/// Parses a command line (argv[0] is the program name). Returns false if the
/// arguments were invalid; the driver has already printed why.
SLANG_C_API bool slang_driver_parse_args(slang_driver driver, int argc, const char* const* argv,
                                         slang_error* err);

/// Reads back a custom option. Each returns false if the option was not given
/// on the command line (`*out` is untouched). String values are borrowed from
/// the driver.
SLANG_C_API bool slang_driver_option_flag(slang_driver driver, const char* names, size_t names_len,
                                          bool* out);
SLANG_C_API bool slang_driver_option_int(slang_driver driver, const char* names, size_t names_len,
                                         int64_t* out);
SLANG_C_API bool slang_driver_option_string(slang_driver driver, const char* names,
                                            size_t names_len, slang_str* out);

/// The formatted help text for all registered options. Owned.
SLANG_C_API slang_str slang_driver_help_text(slang_driver driver, const char* overview,
                                             size_t overview_len, slang_error* err);

/// Applies the parsed options (include paths, defines, ...). Returns false on
/// an error, which the driver has already reported.
SLANG_C_API bool slang_driver_process_options(slang_driver driver, slang_error* err);

/// Loads and parses every source file named on the command line. Returns
/// false if any file had errors (they are reported later).
SLANG_C_API bool slang_driver_parse_sources(slang_driver driver, slang_error* err);

/// The driver's source manager (borrowed; lives as long as the driver).
SLANG_C_API slang_source_manager slang_driver_source_manager(slang_driver driver);

/// The parsed syntax trees (borrowed; retain one to keep it beyond the driver).
SLANG_C_API uint32_t slang_driver_tree_count(slang_driver driver);
SLANG_C_API slang_syntax_tree slang_driver_tree(slang_driver driver, uint32_t index);

/// Creates a compilation from the parsed trees with the options given on the
/// command line, ORing `extra_flags` (a bitmask of slang_compilation_flag)
/// into the compilation flags — pass SLANG_COMP_DISABLE_INSTANCE_CACHING to
/// make the result totalizable by SLANG_FREEZE_ELABORATE_ALL, or 0 for none.
/// Owned by the caller, but it borrows the driver's source manager and so must
/// be destroyed before the driver.
SLANG_C_API slang_compilation slang_driver_create_compilation(slang_driver driver,
                                                              uint32_t extra_flags,
                                                              slang_error* err);

/// Reports the compilation's diagnostics through the driver's configured
/// diagnostic output (colors, `--diag-*` options, error limits).
SLANG_C_API void slang_driver_report_compilation(slang_driver driver, slang_compilation comp,
                                                 bool quiet, slang_error* err);

/// Prints the summary line ("Build succeeded: ...") and returns whether the
/// run had no errors.
SLANG_C_API bool slang_driver_report_diagnostics(slang_driver driver, bool quiet, slang_error* err);

/* ------------------------------------------------------------------------- */
/* Analysis                                                                   */
/* ------------------------------------------------------------------------- */

/// The result of running slang's semantic analysis passes (unused-code,
/// multi-driver, case, and related lints, plus driver tracking) over a
/// compilation. Opaque; free with slang_analysis_destroy.
typedef struct slang_analysis_t* slang_analysis;

/// Analysis flags. Values match slang's AnalysisFlags and are append-only.
typedef enum slang_analysis_flag {
    /// Report unused code (nets, variables, parameters, imports, ...).
    SLANG_ANALYSIS_CHECK_UNUSED = 1u << 0,
    SLANG_ANALYSIS_FULL_CASE_UNIQUE_PRIORITY = 1u << 1,
    SLANG_ANALYSIS_FULL_CASE_FOUR_STATE = 1u << 2,
    SLANG_ANALYSIS_ALLOW_MULTI_DRIVEN_LOCALS = 1u << 3,
    SLANG_ANALYSIS_ALLOW_DUP_INITIAL_DRIVERS = 1u << 4,
    SLANG_ANALYSIS_CHECK_SHADOW = 1u << 5,
    SLANG_ANALYSIS_INLINE_CONT_ASSIGN_FUNCTION_READS = 1u << 6,
    SLANG_ANALYSIS_ALWAYS_STAR_USES_LSPS = 1u << 7,
    SLANG_ANALYSIS_CONT_ASSIGN_USES_LSPS = 1u << 8,
} slang_analysis_flag;

/// Runs analysis over a compilation (which is finalized first). `flags` is a
/// bitmask of slang_analysis_flag; `threads` is the worker-thread count (0
/// lets slang choose). The compilation must outlive the returned handle.
SLANG_C_API slang_analysis slang_analysis_run(slang_compilation comp, uint32_t flags,
                                              uint32_t threads, slang_error* err);

/// Listeners invoked during analysis, once per analyzed procedure, scope or
/// assertion. The `slang_ast` is the relevant symbol: the procedure
/// (`always`/`initial`/`final` block or subroutine), the scope's symbol, or the
/// assertion's containing symbol. With more than one worker thread they may be
/// called concurrently, so they must be thread-safe, and must not throw or let
/// an exception escape. history: since 1.1.
typedef void (*slang_procedure_listener)(slang_ast procedure, void* user);
typedef void (*slang_scope_listener)(slang_ast scope, void* user);
typedef void (*slang_assertion_listener)(slang_ast containing_symbol, void* user);

/// A set of analysis listeners; any callback may be NULL (not registered). All
/// share the one `user` pointer. history: since 1.1.
typedef struct slang_analysis_listeners {
    slang_procedure_listener on_procedure;
    slang_scope_listener on_scope;
    slang_assertion_listener on_assertion;
    void* user;
} slang_analysis_listeners;

/// Like slang_analysis_run, but registers `listeners` before analyzing, so each
/// analyzed procedure/scope/assertion is reported as it is processed. A NULL
/// `listeners`, or one whose callbacks are all NULL, behaves exactly like
/// slang_analysis_run. history: since 1.1.
SLANG_C_API slang_analysis slang_analysis_run_listening(slang_compilation comp, uint32_t flags,
                                                        uint32_t threads,
                                                        const slang_analysis_listeners* listeners,
                                                        slang_error* err);

/// Destroys an analysis result.
SLANG_C_API void slang_analysis_destroy(slang_analysis analysis);

/// The diagnostics produced by analysis (the lints selected by the flags).
/// Owned by the caller; free with slang_diagnostics_destroy.
SLANG_C_API slang_diagnostics slang_analysis_diagnostics(slang_analysis analysis, slang_error* err);

/// How a value is driven.
typedef enum slang_driver_kind {
    SLANG_DRIVER_PROCEDURAL = 0,
    SLANG_DRIVER_CONTINUOUS = 1,
    SLANG_DRIVER_OTHER = 2,
} slang_driver_kind;

/// One driver of a value: an assignment or connection that writes it.
typedef struct slang_driver_info {
    slang_driver_kind kind;
    /// Bit 0: the driver is an input port. Bit 1: a unidirectional port.
    /// Bit 2: a clocking-block variable.
    uint32_t flags;
    /// The source range of the driving expression.
    slang_range range;
    /// The symbol (procedure, continuous-assign, ...) the driver is inside.
    slang_ast containing_symbol;
} slang_driver_info;

/// Bits in slang_driver_info::flags.
typedef enum slang_driver_flag {
    SLANG_DRIVER_INPUT_PORT = 1u << 0,
    SLANG_DRIVER_UNIDIRECTIONAL_PORT = 1u << 1,
    SLANG_DRIVER_CLOCK_VAR = 1u << 2,
} slang_driver_flag;

/// The number of procedures (`always`/`initial`/`final` blocks, continuous
/// assignments, subroutines) that were analyzed in a scope symbol (a module
/// instance body, package, ...). Zero if `scope` is not an analyzed scope.
SLANG_C_API uint32_t slang_analysis_scope_procedure_count(slang_analysis analysis, slang_ast scope);

/// The analyzed symbol of procedure `index` in a scope (e.g. a ProceduralBlock
/// or ContinuousAssign symbol). Null if out of range.
SLANG_C_API slang_ast slang_analysis_scope_procedure(slang_analysis analysis, slang_ast scope,
                                                     uint32_t index);

/// True if procedure `index` in `scope` has an inferred clock (it is a clocked
/// procedure). `index` out of range yields false.
SLANG_C_API bool slang_analysis_procedure_has_clock(slang_analysis analysis, slang_ast scope,
                                                    uint32_t index);

/// The number of drivers of a value symbol (an assignment or connection that
/// writes it). Zero if `value` is not a value symbol.
SLANG_C_API uint32_t slang_analysis_driver_count(slang_analysis analysis, slang_ast value);

/// Fetches driver `index` of a value symbol. Returns false if out of range.
SLANG_C_API bool slang_analysis_driver(slang_analysis analysis, slang_ast value, uint32_t index,
                                       slang_driver_info* out);

/* ------------------------------------------------------------------------- */
/* Custom dataflow analysis                                                   */
/* ------------------------------------------------------------------------- */

/// A dataflow event delivered to a lattice's transfer function as slang walks
/// a procedure's control flow.
typedef struct slang_dfa_event {
    /// See slang_dfa_event_kind.
    uint32_t kind;
    /// The value symbol read or written (SLANG_DFA_READ / SLANG_DFA_WRITE), or
    /// the null AST for a call.
    slang_ast symbol;
    /// The expression node the event occurred at.
    slang_ast node;
} slang_dfa_event;

/// The kind of a slang_dfa_event.
typedef enum slang_dfa_event_kind {
    /// A value symbol was read.
    SLANG_DFA_READ = 0,
    /// A value symbol was written (an assignment's left-hand side).
    SLANG_DFA_WRITE = 1,
    /// A subroutine was called.
    SLANG_DFA_CALL = 2,
} slang_dfa_event_kind;

/// A caller-defined lattice and transfer function for a forward dataflow
/// analysis. slang drives a procedure's control-flow graph and calls these to
/// create, copy, merge and evolve the abstract state, which is an opaque
/// caller-owned pointer (`void*`). All callbacks receive the `user` pointer
/// passed to slang_dfa_run, and none may unwind into the library.
///
/// The lattice's merge operations are named for the control-flow point they run
/// at, not for a fixed set-theoretic operation: `join` runs where branches
/// rejoin, `meet` at loop back-edges. A "must" analysis makes both intersect; a
/// "may" analysis makes both union. `state` mutations in `transfer` are exact
/// for SLANG_DFA_WRITE events (always visited with a definite state) and
/// best-effort for reads inside compound short-circuit conditions.
typedef struct slang_dfa_lattice {
    /// Creates the entry/top state.
    void* (*top)(void* user);
    /// Creates the unreachable/bottom state.
    void* (*bottom)(void* user);
    /// Deep-copies a state.
    void* (*clone)(void* user, const void* state);
    /// Merges `other` into `into` where branches rejoin.
    void (*join)(void* user, void* into, const void* other);
    /// Merges `other` into `into` at a loop back-edge.
    void (*meet)(void* user, void* into, const void* other);
    /// Applies `event` to `state`.
    void (*transfer)(void* user, void* state, const slang_dfa_event* event);
    /// Frees a state.
    void (*drop)(void* user, void* state);
} slang_dfa_lattice;

/// Runs the caller's forward dataflow analysis over a procedure symbol (an
/// `always`/`initial`/`final` block, a subroutine, or a continuous assign).
/// Returns the exit state (a fresh copy the caller owns and must free with
/// `lattice->drop`), or NULL on error or if `procedure` is not a procedure.
SLANG_C_API void* slang_dfa_run(slang_compilation comp, slang_ast procedure,
                                const slang_dfa_lattice* lattice, void* user, slang_error* err);

/* ------------------------------------------------------------------------- */
/* Unstable surface                                                           */
/* ------------------------------------------------------------------------- */

#if SLANG_C_API_ALLOW_UNSTABLE
/// The underlying C++ object pointer (slang::syntax::SyntaxNode* or
/// slang::ast::Symbol* etc.). For consumers that link the C++ library too and
/// want to escape to it. Not part of any stability promise.
SLANG_C_API const void* slang_unstable_native_ptr(slang_ast node);
SLANG_C_API const void* slang_unstable_native_node_ptr(slang_node node);
#endif

#ifdef __cplusplus
} // extern "C"
#endif

#endif // SLANG_C_SLANG_H
