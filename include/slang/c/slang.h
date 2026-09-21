//------------------------------------------------------------------------------
//! @file slang.h
//! @brief Stable C API for slang, the SystemVerilog compiler frontend
//
// SPDX-FileCopyrightText: Chaitanya Sharma
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
#define SLANG_C_VERSION_MINOR 3

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

/// An owned, opaque options bag: slang's type-erased container of the option
/// structs (compilation/lexer/parser/preprocessor options) a driver assembles
/// from its command-line arguments. Produced by slang_driver_create_option_bag,
/// consumed by slang_compilation_create_from_bag, freed with slang_bag_destroy.
typedef struct slang_bag_t* slang_bag;

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

/// One production element within a RandSeqProductionSymbol rule's prod list
/// (slang::ast::RandSeqProductionSymbol::ProdBase and its four concrete
/// variants ProdItem/CodeBlockProd/IfElseProd/CaseProd -- see
/// slang_symbol_randseq_rule_prod). Trivially copyable; valid while
/// `compilation` is alive. `kind` is a slang_randseq_prod_kind ordinal
/// identifying which variant `ptr` points at. A null `ptr` means "no prod"
/// (see slang_randseq_prod_is_null). Deliberately a separate type from
/// slang_ast: these C++ structs are not Symbol/Expression/Statement/... --
/// they are plain arena-allocated nodes private to RandSeqProductionSymbol's
/// rule tree. history: since 1.3.
typedef struct slang_randseq_prod {
    const void* ptr;
    slang_compilation compilation;
    uint32_t kind;
    uint32_t reserved_;
} slang_randseq_prod;

#ifdef __cplusplus
// The position structs are two pointers plus two 32-bit fields, so their size
// is 24 bytes on a 64-bit target (LP64) and 16 on a 32-bit one (e.g. wasm32).
static_assert(sizeof(slang_node) == 2 * sizeof(void*) + 8, "slang_node ABI");
static_assert(sizeof(slang_token) == 2 * sizeof(void*) + 8, "slang_token ABI");
static_assert(sizeof(slang_ast) == 2 * sizeof(void*) + 8, "slang_ast ABI");
static_assert(sizeof(slang_randseq_prod) == 2 * sizeof(void*) + 8,
              "slang_randseq_prod ABI");
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

/// Creates an empty compilation using the options in `bag` (e.g. one assembled
/// by slang_driver_create_option_bag), ORing `extra_flags` (a bitmask of
/// slang_compilation_flag) into the compilation flags — pass
/// SLANG_COMP_DISABLE_INSTANCE_CACHING to make the result totalizable by
/// SLANG_FREEZE_ELABORATE_ALL, or 0 for none. `bag` may be null for defaults.
/// The caller still owns `bag` and must free it separately; add trees as usual.
SLANG_C_API slang_compilation slang_compilation_create_from_bag(slang_bag bag /* nullable */,
                                                                uint32_t extra_flags,
                                                                slang_error* err);

/// A canonical built-in type, for slang_compilation_add_nonconstant_system_function.
typedef enum slang_builtin_type {
    SLANG_BUILTIN_TYPE_INT = 0,
    SLANG_BUILTIN_TYPE_LOGIC = 1,
    SLANG_BUILTIN_TYPE_BIT = 2,
    SLANG_BUILTIN_TYPE_BYTE = 3,
    SLANG_BUILTIN_TYPE_INTEGER = 4,
    SLANG_BUILTIN_TYPE_REAL = 5,
    SLANG_BUILTIN_TYPE_SHORTREAL = 6,
    SLANG_BUILTIN_TYPE_STRING = 7,
    SLANG_BUILTIN_TYPE_VOID = 8,
} slang_builtin_type;

/// Registers a non-constant system function (slang::ast::NonConstantFunction) so
/// the elaborator recognizes `name` (e.g. "$fputc") instead of reporting
/// UnknownSystemName. `return_type` and each of the `n_args` `arg_types` are
/// slang_builtin_type ordinals; all args are required. Must be called BEFORE the
/// compilation is finalized (before freeze / any binding). Errors with
/// SLANG_ERR_INVALID_STATE if the compilation is already finalized.
SLANG_C_API void slang_compilation_add_nonconstant_system_function(
    slang_compilation comp, const char* name, size_t name_len, uint32_t return_type,
    const uint32_t* arg_types, size_t n_args, slang_error* err);

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

/// True once slang_compilation_freeze has been called with SLANG_FREEZE_SEAL,
/// until a matching slang_compilation_unfreeze clears it again.
SLANG_C_API bool slang_compilation_is_sealed(slang_compilation comp);

/// Lifts the seal placed by slang_compilation_freeze(..., SLANG_FREEZE_SEAL, ...),
/// letting the underlying arena accept allocations again; slang_compilation_is_sealed
/// reports false until the compilation is re-sealed. A no-op if the compilation
/// is not currently sealed. This is the low-level primitive slang_compilation's
/// own gated, allocating operations (e.g. constant evaluation, hierarchical
/// lookup) lift and restore internally around themselves; reach for it directly
/// only when building a new such operation of your own. Re-seal by calling
/// slang_compilation_freeze(comp, SLANG_FREEZE_SEAL, NULL, err), which restores
/// the seal without repeating the elaboration/prefold sweep. The caller must
/// hold exclusive access to the compilation for the entire span it is
/// unfrozen: no other read of it may happen concurrently, on this or any
/// other thread, until it is re-sealed. history: since 1.3.
SLANG_C_API void slang_compilation_unfreeze(slang_compilation comp, slang_error* err);

/// The root symbol of the design. Finalizes the compilation if it has not been.
SLANG_C_API slang_ast slang_compilation_root(slang_compilation comp, slang_error* err);

/* slang::ast::RootSymbol accessors below take the root symbol node returned
 * by slang_compilation_root. Each returns the empty/zero default (rather
 * than erroring) if `sym` is not a Root symbol -- see
 * slang_compilation_top_instance_count / slang_compilation_top_instance and
 * slang_compilation_unit_count / slang_compilation_unit_at for the
 * equivalent, more commonly used compilation-level accessors, which these
 * mirror one level down (RootSymbol::topInstances / ::compilationUnits are
 * the same span slang::ast::Compilation::getRoot() exposes). history: since
 * 1.3. */

/// The number of top-level module/interface/program instances in the design
/// rooted at `sym` (slang::ast::RootSymbol::topInstances). 0 if `sym` is not
/// a Root symbol. Set once, at elaboration, before this symbol is ever
/// reachable through a frozen &Design, so this is a pure, allocation-free
/// read. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_root_top_instance_count(slang_ast sym);

/// The `index`'th top-level instance of the design rooted at `sym` (see
/// slang_symbol_root_top_instance_count), as a node of domain
/// SLANG_AST_SYMBOL (SymbolKind::Instance). A null node if `sym` is not a
/// Root symbol or `index` is out of range. A pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API slang_ast slang_symbol_root_top_instance(slang_ast sym, uint32_t index);

/// The number of compilation units (one per syntax tree added to the
/// compilation) contained in the design rooted at `sym`
/// (slang::ast::RootSymbol::compilationUnits). 0 if `sym` is not a Root
/// symbol. Set once, at elaboration, before this symbol is ever reachable
/// through a frozen &Design, so this is a pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API uint32_t slang_symbol_root_compilation_unit_count(slang_ast sym);

/// The `index`'th compilation unit of the design rooted at `sym` (see
/// slang_symbol_root_compilation_unit_count), as a node of domain
/// SLANG_AST_SYMBOL (SymbolKind::CompilationUnit). A null node if `sym` is
/// not a Root symbol or `index` is out of range. A pure, allocation-free
/// read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_root_compilation_unit(slang_ast sym, uint32_t index);

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

/// The compilation's built-in 2-state `bit` type. Set at construction; a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_compilation_get_bit_type(slang_compilation comp);

/// The compilation's built-in 2-state `byte` type. history: since 1.3.
SLANG_C_API slang_ast slang_compilation_get_byte_type(slang_compilation comp);

/// Gets the compilation unit symbol for the given compilation-unit syntax node
/// (the root of a syntax tree already added to this compilation via
/// slang_compilation_add_tree). Null if not found. history: since 1.3.
SLANG_C_API slang_ast slang_compilation_unit_for_syntax(slang_compilation comp, slang_node syntax);

/// Number of compilation units (one per syntax tree) added to this
/// compilation. history: since 1.3.
SLANG_C_API uint32_t slang_compilation_unit_count(slang_compilation comp);

/// Fetches compilation unit `index` (see slang_compilation_unit_count). Null
/// node if out of range. history: since 1.3.
SLANG_C_API slang_ast slang_compilation_unit_at(slang_compilation comp, uint32_t index);

/// Creates a new compilation unit that can be modified dynamically, which is
/// useful in runtime scripting scenarios (mirrors slang's ScriptSession).
/// Succeeds even after the compilation has been finalized, though in that case
/// instantiations added under it will not retroactively become top-level
/// instances. This allocates into the arena, so on a sealed compilation the
/// seal is lifted for the call and the caller must guarantee exclusive access;
/// concurrent calls of any kind (on this compilation) are a data race.
/// history: since 1.3.
SLANG_C_API slang_ast slang_compilation_create_script_scope(slang_compilation comp,
                                                            slang_error* err);

/// Merges externally-produced diagnostics (e.g. from a syntax tree parsed
/// outside this compilation, or another compilation's diagnostics) into this
/// compilation's own diagnostic list. Like slang_compilation_create_script_scope
/// this allocates into the arena, lifting the seal on a sealed compilation for
/// the duration of the call; the caller must guarantee exclusive access.
/// history: since 1.3.
SLANG_C_API void slang_compilation_add_diagnostics(slang_compilation comp, slang_diagnostics diags,
                                                    slang_error* err);

/* ------------------------------------------------------------------------- */
/* Compilation: DPI exports                                                   */
/* ------------------------------------------------------------------------- */

/// A DPI export directive (`export "DPI-C" function/task ...;`) collected
/// during elaboration. Position struct: trivially copyable, valid while the
/// owning compilation is alive. history: since 1.3.
typedef struct slang_dpi_export {
    const void* ptr;
    slang_compilation compilation;
} slang_dpi_export;

/// True for the null DPI export returned when an index is out of range.
/// history: since 1.3.
SLANG_C_API bool slang_dpi_export_is_null(slang_dpi_export exp);

/// Number of DPI export directives collected during elaboration.
/// history: since 1.3.
SLANG_C_API uint32_t slang_compilation_dpi_export_count(slang_compilation comp);

/// Fetches DPI export `index`. The null export (see slang_dpi_export_is_null)
/// if out of range. history: since 1.3.
SLANG_C_API slang_dpi_export slang_compilation_dpi_export(slang_compilation comp, uint32_t index);

/// The exported subroutine symbol. history: since 1.3.
SLANG_C_API slang_ast slang_dpi_export_subroutine(slang_dpi_export exp);

/// The C identifier the subroutine is exported under. Borrowed from the
/// compilation. history: since 1.3.
SLANG_C_API slang_str slang_dpi_export_c_identifier(slang_dpi_export exp);

/// The original `export "DPI-C"` declaration syntax node. history: since 1.3.
SLANG_C_API slang_node slang_dpi_export_syntax(slang_dpi_export exp);

/* ------------------------------------------------------------------------- */
/* Compilation: definition lookup                                            */
/* ------------------------------------------------------------------------- */

/// A rule from a `config` block controlling how a specific cell or instance is
/// resolved (see slang_definition_lookup_result::config_rule). Opaque and
/// borrowed from the compilation; a future release may add field accessors.
/// history: since 1.3.
typedef struct slang_config_rule_t* slang_config_rule;

/// The result of a definition lookup (see slang_compilation_try_get_definition).
/// Position struct: trivially copyable, valid while the owning compilation is
/// alive. history: since 1.3.
typedef struct slang_definition_lookup_result {
    /// The definition that was found, or the null symbol if none was found.
    slang_ast definition;
    /// A config root that applies to this definition and the hierarchy
    /// underneath it, or the null symbol if none.
    slang_ast config_root;
    /// A config rule that applies to instances using this definition, or NULL
    /// if none.
    slang_config_rule config_rule;
} slang_definition_lookup_result;

/// Looks up the definition (module/interface/program) named `name` as visible
/// from `scope` (a scope symbol, e.g. an instance body), taking nested
/// definitions and any `config` block that applies to `scope` into account.
/// Unlike slang's diagnostic-issuing getDefinition overloads (not yet exposed
/// on this stable surface), this never issues a diagnostic when nothing is
/// found. A pure read of already-elaborated state. history: since 1.3.
SLANG_C_API slang_definition_lookup_result slang_compilation_try_get_definition(
    slang_compilation comp, const char* name, size_t name_len, slang_ast scope);

/* ------------------------------------------------------------------------- */
/* Compilation: built-in types, options, libraries, diagnostics              */
/* ------------------------------------------------------------------------- */

/// The compilation's built-in `int` type (a 2-state, signed, 32-bit integer).
/// Set at construction; a pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_compilation_get_int_type(slang_compilation comp);

/// The compilation's built-in `integer` type (a 4-state, signed, 32-bit
/// integer). history: since 1.3.
SLANG_C_API slang_ast slang_compilation_get_integer_type(slang_compilation comp);

/// The compilation's built-in `logic` type (a 4-state, 1-bit value).
/// history: since 1.3.
SLANG_C_API slang_ast slang_compilation_get_logic_type(slang_compilation comp);

/// The compilation's built-in `real` type (a 64-bit floating point value).
/// history: since 1.3.
SLANG_C_API slang_ast slang_compilation_get_real_type(slang_compilation comp);

/// The compilation's built-in `shortreal` type (a 32-bit floating point
/// value). history: since 1.3.
SLANG_C_API slang_ast slang_compilation_get_short_real_type(slang_compilation comp);

/// The compilation's built-in error type, substituted wherever type
/// resolution fails. history: since 1.3.
SLANG_C_API slang_ast slang_compilation_get_error_type(slang_compilation comp);

/// The compilation's built-in `null` (empty queue / class handle literal)
/// type. history: since 1.3.
SLANG_C_API slang_ast slang_compilation_get_null_type(slang_compilation comp);

/// Looks up a built-in gate primitive (`and`, `nand`, `buf`, ...) by name.
/// Null if `name` does not name a built-in gate primitive. A pure hash-map
/// read of a table built at construction. history: since 1.3.
SLANG_C_API slang_ast slang_compilation_get_gate_type(slang_compilation comp, const char* name,
                                                      size_t name_len);

/// Looks up a package by name. Null if no such package has been elaborated.
/// A pure hash-map read. history: since 1.3.
SLANG_C_API slang_ast slang_compilation_get_package(slang_compilation comp, const char* name,
                                                     size_t name_len);

/// The keyword-introduced built-in net types slang_compilation_get_net_type
/// can fetch. Mirrors the subset of slang's NetType::NetKind that is
/// reachable via a keyword lookup (user-defined nettypes are ordinary scope
/// members, found via scope/symbol lookup instead). history: since 1.3.
typedef enum slang_net_type_kind {
    SLANG_NET_TYPE_WIRE = 0,
    SLANG_NET_TYPE_WAND,
    SLANG_NET_TYPE_WOR,
    SLANG_NET_TYPE_TRI,
    SLANG_NET_TYPE_TRIAND,
    SLANG_NET_TYPE_TRIOR,
    SLANG_NET_TYPE_TRI0,
    SLANG_NET_TYPE_TRI1,
    SLANG_NET_TYPE_TRIREG,
    SLANG_NET_TYPE_SUPPLY0,
    SLANG_NET_TYPE_SUPPLY1,
    SLANG_NET_TYPE_UWIRE,
    SLANG_NET_TYPE_INTERCONNECT,
} slang_net_type_kind;

/// The built-in net type for `kind` (a symbol of kind NetType). Every value
/// of slang_net_type_kind resolves to a distinct, always-present built-in
/// net type set up at construction, so this never returns null; a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_compilation_get_net_type(slang_compilation comp,
                                                     slang_net_type_kind kind);

/// A read-only snapshot of a compilation's options (see
/// slang_compilation_get_options). Position struct: trivially copyable and
/// independent of the owning compilation once returned. The variable-length
/// fields of slang's CompilationOptions (topModules, paramOverrides,
/// defaultLiblist) are set at construction time only and are not part of
/// this stable snapshot. history: since 1.3.
typedef struct slang_compilation_options {
    /// Bitmask of slang_compilation_flag.
    uint32_t flags;
    uint32_t max_instance_depth;
    uint32_t max_checker_instance_depth;
    uint32_t max_generate_steps;
    uint32_t max_constexpr_depth;
    uint32_t max_constexpr_steps;
    uint32_t max_constexpr_backtrace;
    uint64_t max_constant_size;
    uint32_t max_defparam_steps;
    uint32_t max_defparam_blocks;
    uint32_t max_instance_array;
    uint32_t max_enum_values;
    uint32_t max_recursive_class_specialization;
    uint32_t max_udp_coverage_notes;
    uint32_t error_limit;
    uint32_t typo_correction_limit;
    /// Raw slang::ast::MinTypMax value: 0 = Min, 1 = Typ, 2 = Max.
    uint32_t min_typ_max;
    /// Raw slang::LanguageVersion value: 0 = 1364-2005, 1 = 1800-2017,
    /// 2 = 1800-2023.
    uint32_t language_version;
} slang_compilation_options;

/// The options this compilation was constructed with. A pure, allocation-free
/// read (the returned struct is a snapshot copy). history: since 1.3.
SLANG_C_API slang_compilation_options slang_compilation_get_options(slang_compilation comp);

/// A time scale: a base and precision unit+magnitude pair (see
/// slang_compilation_get_default_time_scale). Position struct: trivially
/// copyable. history: since 1.3.
typedef struct slang_time_scale {
    /// Raw slang::TimeUnit value for the base: 0=s, 1=ms, 2=us, 3=ns, 4=ps,
    /// 5=fs.
    uint8_t base_unit;
    /// The base magnitude: 1, 10, or 100.
    uint8_t base_magnitude;
    /// Raw slang::TimeUnit value for the precision (same encoding as
    /// base_unit).
    uint8_t precision_unit;
    /// The precision magnitude: 1, 10, or 100.
    uint8_t precision_magnitude;
} slang_time_scale;

/// The default time scale used for design elements that don't specify one
/// explicitly. Returns false (leaving `*out` untouched) if no default time
/// scale was configured. A pure, allocation-free read. history: since 1.3.
SLANG_C_API bool slang_compilation_get_default_time_scale(slang_compilation comp,
                                                           slang_time_scale* out);

/// A single unit+magnitude pair within a slang_time_scale (its base or its
/// precision) — mirrors slang::TimeScaleValue. Position struct: trivially
/// copyable. history: since 1.3.
typedef struct slang_time_scale_value {
    /// Raw slang::TimeUnit value: 0=s, 1=ms, 2=us, 3=ns, 4=ps, 5=fs.
    uint8_t unit;
    /// The magnitude: 1, 10, or 100.
    uint8_t magnitude;
} slang_time_scale_value;

/// The base unit+magnitude of a time scale (the `1ns` of a `` `timescale
/// 1ns/1ps`` directive) — mirrors the slang::TimeScale::base field. A pure
/// function of its argument (no allocation, no handle). history: since 1.3.
SLANG_C_API slang_time_scale_value slang_time_scale_base(slang_time_scale ts);

/// The precision unit+magnitude of a time scale (the `1ps` of a ``
/// `timescale 1ns/1ps`` directive) — mirrors the slang::TimeScale::precision
/// field. A pure function of its argument. history: since 1.3.
SLANG_C_API slang_time_scale_value slang_time_scale_precision(slang_time_scale ts);

/// Scales `value` (given in `unit`) to the number of `ts`'s base-unit ticks
/// it represents, optionally rounded to `ts`'s precision — mirrors
/// slang::TimeScale::apply. `unit` uses the same raw encoding as
/// slang_time_scale::base_unit. A pure function of its arguments. history:
/// since 1.3.
SLANG_C_API double slang_time_scale_apply(slang_time_scale ts, double value, uint8_t unit,
                                          bool round_to_precision);

/// Parses a time scale from SystemVerilog `` `timescale`` syntax (e.g.
/// "1ns/1ps"), writing the result to `*out` and returning true on success —
/// mirrors the static slang::TimeScale::fromString. Returns false (leaving
/// `*out` untouched) if `str` does not parse. A pure function of its
/// arguments (no allocation beyond local parsing state, no handle). history:
/// since 1.3.
SLANG_C_API bool slang_time_scale_from_string(const char* str, size_t str_len,
                                              slang_time_scale* out);

/// The scale unit of a slang_time_scale_value — mirrors the
/// slang::TimeScaleValue::unit field. Same raw encoding as
/// slang_time_scale::base_unit. A pure function of its argument. history:
/// since 1.3.
SLANG_C_API uint8_t slang_time_scale_value_unit(slang_time_scale_value v);

/// The magnitude of a slang_time_scale_value — mirrors the
/// slang::TimeScaleValue::magnitude field (a slang::TimeScaleMagnitude, whose
/// only values are 1, 10, and 100). A pure function of its argument. history:
/// since 1.3.
SLANG_C_API uint8_t slang_time_scale_value_magnitude(slang_time_scale_value v);

/// Constructs a slang_time_scale_value from a numeric literal and unit —
/// mirrors the static slang::TimeScaleValue::fromLiteral. `unit` uses the
/// same raw encoding as slang_time_scale::base_unit. Writes the result to
/// `*out` and returns true on success; returns false (leaving `*out`
/// untouched) if `value` is not exactly 1, 10, or 100 (the only magnitudes a
/// time scale value may have). A pure function of its arguments. history:
/// since 1.3.
SLANG_C_API bool slang_time_scale_value_from_literal(double value, uint8_t unit,
                                                      slang_time_scale_value* out);

/// Parses a single unit+magnitude token (e.g. "10ns", as opposed to the
/// base/precision pair parsed by slang_time_scale_from_string) — mirrors the
/// static slang::TimeScaleValue::fromString. Writes the result to `*out` and
/// returns true on success. Returns false (leaving `*out` untouched) if `str`
/// does not parse. A pure function of its arguments (no allocation beyond
/// local parsing state, no handle). history: since 1.3.
SLANG_C_API bool slang_time_scale_value_from_string(const char* str, size_t str_len,
                                                     slang_time_scale_value* out);

/// The number of explicitly configured top-level module names (see
/// CompilationOptions::topModules) -- non-zero only if the compilation was
/// constructed with an explicit top-module list; if zero, top modules are
/// instead inferred from which modules are unreferenced elsewhere (see
/// slang_compilation_top_instance_count for the modules actually chosen
/// either way). A pure, allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_compilation_top_module_count(slang_compilation comp);

/// Fetches configured top-module name `index` (see
/// slang_compilation_top_module_count). Borrowed. history: since 1.3.
SLANG_C_API slang_str slang_compilation_top_module_at(slang_compilation comp, uint32_t index);

/// The number of parameter override strings (CompilationOptions::paramOverrides,
/// each of the form "name=value") configured for this compilation. A pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_compilation_param_override_count(slang_compilation comp);

/// Fetches parameter override string `index` (see
/// slang_compilation_param_override_count). Borrowed. history: since 1.3.
SLANG_C_API slang_str slang_compilation_param_override_at(slang_compilation comp, uint32_t index);

/// The number of library names in the default liblist search order
/// (CompilationOptions::defaultLiblist) configured for this compilation. A
/// pure, allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_compilation_default_liblist_count(slang_compilation comp);

/// Fetches default-liblist library name `index` (see
/// slang_compilation_default_liblist_count). Borrowed. history: since 1.3.
SLANG_C_API slang_str slang_compilation_default_liblist_at(slang_compilation comp, uint32_t index);

/// A source library (see slang_compilation_get_default_library and
/// slang_compilation_get_source_library). Opaque and borrowed from the
/// owning compilation; a null handle is only ever returned by the
/// name-lookup accessor. history: since 1.3.
typedef struct slang_source_library_t* slang_source_library;

/// The library's name. Borrowed. history: since 1.3.
SLANG_C_API slang_str slang_source_library_name(slang_source_library lib);

/// The library's search priority; lower numbers are higher priority.
/// history: since 1.3.
SLANG_C_API int32_t slang_source_library_priority(slang_source_library lib);

/// True if this is the compilation's default library. history: since 1.3.
SLANG_C_API bool slang_source_library_is_default(slang_source_library lib);

/// The compilation's default source library. Set at construction; never
/// null. A pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_source_library slang_compilation_get_default_library(slang_compilation comp);

/// Looks up a source library by name. Null if no such library is known to
/// this compilation. A pure hash-map read. history: since 1.3.
SLANG_C_API slang_source_library slang_compilation_get_source_library(slang_compilation comp,
                                                                      const char* name,
                                                                      size_t name_len);

/// The diagnostics produced during lexing, preprocessing, and syntax
/// parsing (a subset of slang_compilation_diagnostics). Already forced (and
/// cached) by slang_compilation_freeze, so calling this on a frozen
/// compilation is a pure read of the cached list; calling it before freezing
/// computes and caches it now. Owned. history: since 1.3.
SLANG_C_API slang_diagnostics slang_compilation_get_parse_diagnostics(slang_compilation comp,
                                                                      slang_error* err);

/// The diagnostics produced during semantic analysis: symbol creation, type
/// checking, and name lookup (a subset of slang_compilation_diagnostics).
/// Already forced (and cached) by slang_compilation_freeze; see
/// slang_compilation_get_parse_diagnostics. Owned. history: since 1.3.
SLANG_C_API slang_diagnostics slang_compilation_get_semantic_diagnostics(slang_compilation comp,
                                                                         slang_error* err);

/* ------------------------------------------------------------------------- */
/* Compilation: state, syntax trees, name parsing                            */
/* ------------------------------------------------------------------------- */

/// Indicates whether the design has been compiled (via slang_compilation_root
/// or slang_compilation_diagnostics) and can no longer accept new syntax
/// trees. A pure, allocation-free read. history: since 1.3.
SLANG_C_API bool slang_compilation_is_finalized(slang_compilation comp);

/// Indicates whether the design has been elaborated such that the AST is
/// fully resolved and all symbols have been created. This is distinct from
/// slang_compilation_is_finalized, which only means syntax trees have been
/// added; it becomes true once slang_compilation_diagnostics (or
/// slang_compilation_get_semantic_diagnostics) has been called. A pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API bool slang_compilation_is_elaborated(slang_compilation comp);

/// True if any errors have been issued on any scope within this compilation.
/// A pure, allocation-free read. history: since 1.3.
SLANG_C_API bool slang_compilation_has_issued_errors(slang_compilation comp);

/// True if there are any fatal errors reported in the compilation, or if the
/// configured error limit was hit and elaboration stopped early because of
/// it. A pure, allocation-free read. history: since 1.3.
SLANG_C_API bool slang_compilation_has_fatal_errors(slang_compilation comp);

/// Number of syntax trees added to this compilation via
/// slang_compilation_add_tree. history: since 1.3.
SLANG_C_API uint32_t slang_compilation_syntax_tree_count(slang_compilation comp);

/// Fetches syntax tree `index` (see slang_compilation_syntax_tree_count).
/// Borrowed; retain it (slang_syntax_tree_retain) to keep it beyond the
/// compilation. Null if out of range. history: since 1.3.
SLANG_C_API slang_syntax_tree slang_compilation_syntax_tree_at(slang_compilation comp,
                                                                uint32_t index);

/// Parses `name` as a hierarchical/scoped name — mostly for testing and API
/// convenience; normal compilation never does this. Requires a compilation
/// that already has a source manager (i.e. at least one syntax tree has been
/// added). On a malformed name this reports SLANG_ERR_INVALID_ARG via `err`
/// (mirroring slang's std::runtime_error), leaving the returned node null.
/// The returned node has a null `tree` field (it does not belong to any
/// registered slang_syntax_tree, so it is a detached cursor: structural
/// navigation, source text, and children/tokens all still work, but
/// slang_syntax_tree_* lookups on it do not apply); it stays valid as long as
/// the compilation is alive. This allocates into the arena, so it requires
/// exclusive access to the compilation, exactly like
/// slang_compilation_create_script_scope. history: since 1.3.
SLANG_C_API slang_node slang_compilation_parse_name(slang_compilation comp, const char* name,
                                                    size_t name_len, slang_error* err);

/// Parses `name` as a hierarchical/scoped name, like slang_compilation_parse_name,
/// but never fails: diagnostics from a malformed name are collected into
/// `*diags_out` instead (owned; destroy with slang_diagnostics_destroy).
/// `diags_out` may be null to discard them. Requires a compilation that
/// already has a source manager (i.e. at least one syntax tree has been
/// added). This allocates into the arena, so it requires exclusive access to
/// the compilation, exactly like slang_compilation_create_script_scope.
/// history: since 1.3.
SLANG_C_API slang_node slang_compilation_try_parse_name(slang_compilation comp, const char* name,
                                                        size_t name_len,
                                                        slang_diagnostics* diags_out /* nullable */,
                                                        slang_error* err);

/* ------------------------------------------------------------------------- */
/* Compilation: more built-in types and system methods                       */
/* ------------------------------------------------------------------------- */

/// The compilation's built-in 'std' package. Set at construction; never
/// null. A pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_compilation_get_std_package(slang_compilation comp);

/// The compilation's built-in `string` type. history: since 1.3.
SLANG_C_API slang_ast slang_compilation_get_string_type(slang_compilation comp);

/// The compilation's built-in `void` type (the return type of a task, or a
/// function declared to return nothing). history: since 1.3.
SLANG_C_API slang_ast slang_compilation_get_void_type(slang_compilation comp);

/// The compilation's built-in unbounded ('$') type, used for queue/array
/// sizing expressions like `q[$]` or `x[i:$]`. history: since 1.3.
SLANG_C_API slang_ast slang_compilation_get_unbounded_type(slang_compilation comp);

/// The compilation's built-in `type()` reference type (the type of a
/// `type(expr)` construct passed as a value, e.g. to `$cast`). history:
/// since 1.3.
SLANG_C_API slang_ast slang_compilation_get_type_ref_type(slang_compilation comp);

/// The compilation's built-in `int` (2-state, signed, 32-bit) type, indexed
/// by an internal cache keyed on width/flags. Not set at construction: the
/// first call across the whole process to request this particular
/// width/flags combination allocates and caches it, which
/// slang_compilation_freeze forces pre-seal — so after freezing this is a
/// pure, allocation-free read, exactly like the other built-in type
/// accessors. history: since 1.3.
SLANG_C_API slang_ast slang_compilation_get_unsigned_int_type(slang_compilation comp);

/// The compilation's built-in `wire` net type. Set at construction; never
/// null. A pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_compilation_get_wire_net_type(slang_compilation comp);

/// A built-in or user-registered system task/function/method handler (see
/// slang_compilation_get_system_method). Opaque and borrowed — its lifetime
/// either matches the whole process (the built-ins) or the owning
/// compilation (one registered via a not-yet-exposed
/// Compilation::addSystemMethod); either way it outlives any compilation
/// that can observe it. A future release may add further field accessors.
/// history: since 1.3.
typedef struct slang_system_subroutine_t* slang_system_subroutine;

/// The subroutine's name, including the leading `$` for a built-in system
/// task/function (e.g. "$cast"), or the plain method name for a type method
/// (e.g. "push_back"). Borrowed. history: since 1.3.
SLANG_C_API slang_str slang_system_subroutine_name(slang_system_subroutine sub);

/// True if `sub` is a task (as opposed to a function). history: since 1.3.
SLANG_C_API bool slang_system_subroutine_is_task(slang_system_subroutine sub);

/// Looks up a system method for the given type-symbol kind (a raw
/// slang::ast::SymbolKind value, e.g. the kind of a QueueType or
/// DynamicArrayType symbol) and name (e.g. "push_back", "size", "num"). Null
/// if no such method is registered. A pure hash-map read. history: since 1.3.
SLANG_C_API slang_system_subroutine slang_compilation_get_system_method(slang_compilation comp,
                                                                        uint32_t type_kind,
                                                                        const char* name,
                                                                        size_t name_len);

/// True if `sub` allows an empty argument (e.g. the middle slot of
/// `$display(a, , c)`) at the given zero-based `arg_index` (see
/// slang::ast::SystemSubroutine::allowEmptyArgument). A pure virtual call:
/// every built-in either returns a fixed answer or one derived only from
/// `arg_index`, but a not-yet-exposed user-registered subroutine could in
/// principle do more, so this is not promised allocation-free. history:
/// since 1.3.
SLANG_C_API bool slang_system_subroutine_allow_empty_argument(slang_system_subroutine sub,
                                                               uint32_t arg_index);

/// True if `sub` allows a clocking event (e.g. `@(posedge clk)`) to be passed
/// as the argument at the given zero-based `arg_index` (see
/// slang::ast::SystemSubroutine::allowClockingArgument). See
/// slang_system_subroutine_allow_empty_argument for why this is not promised
/// allocation-free. history: since 1.3.
SLANG_C_API bool slang_system_subroutine_allow_clocking_argument(slang_system_subroutine sub,
                                                                  uint32_t arg_index);

/// The system subroutine invoked by a Call expression, or a null handle for
/// a user-defined-subroutine call or a non-call node (the counterpart to
/// slang_expr_call_subroutine, which is null exactly when this one is not).
/// history: since 1.3.
SLANG_C_API slang_system_subroutine slang_expr_call_system_subroutine(slang_ast node);

/// Re-runs slang::ast::SystemSubroutine::checkArguments for the system call
/// `call` (a bound Call expression whose subroutine is a system
/// subroutine — see slang_expr_call_system_subroutine), against the exact
/// arguments, source range, and call context slang itself elaborated the
/// call with, and returns the resulting type. Since checkArguments can
/// allocate a diagnostic-bearing error type (or, for a handful of built-ins,
/// a fresh type node) into the compilation's arena, this requires exclusive
/// access the same way slang_expression_eval_constant does. Fails (via
/// `err`) if `call` is not a bound system call. history: since 1.3.
SLANG_C_API slang_ast slang_system_subroutine_check_arguments(slang_ast call, slang_error* err);

/// Re-runs slang::ast::SystemSubroutine::bindArgument for one argument of the
/// system call `call` (see slang_system_subroutine_check_arguments),
/// re-binding the *original* argument syntax at zero-based `arg_index`
/// against the subroutine's own binding logic (e.g. SimpleSystemSubroutine's
/// argument-type checks), with every earlier argument passed as
/// `previousArgs` exactly as slang bound them. Returns the freshly bound
/// expression, newly allocated into the compilation's arena (requires
/// exclusive access, as with slang_system_subroutine_check_arguments). Fails
/// (via `err`) if `call` is not a bound system call, `arg_index` is out of
/// range, or the original syntax at that index is not present (e.g. a
/// `with`-clause iterator/randomize argument, which is bound through a
/// different path than an ordinary ordered argument). history: since 1.3.
SLANG_C_API slang_ast slang_system_subroutine_bind_argument(slang_ast call, uint32_t arg_index,
                                                             slang_error* err);

/// True if `sub` has output (or ref / inout) arguments (see
/// slang::ast::SystemSubroutine::hasOutputArgs). A pure, allocation-free field
/// read. history: since 1.3.
SLANG_C_API bool slang_system_subroutine_has_output_args(slang_system_subroutine sub);

/// The subroutine's kind, as a raw slang::ast::SubroutineKind value (0 =
/// Function, 1 = Task; see slang_system_subroutine_is_task for the common
/// case as a bool). A pure, allocation-free field read. history: since 1.3.
SLANG_C_API uint32_t slang_system_subroutine_kind(slang_system_subroutine sub);

/// The subroutine's slang::parsing::KnownSystemName enumerator, as a raw
/// value (KnownSystemName::Unknown == 0 for a not-built-in / user-registered
/// subroutine). A pure, allocation-free field read. history: since 1.3.
SLANG_C_API uint32_t slang_system_subroutine_known_name_id(slang_system_subroutine sub);

/// The subroutine's `with`-clause mode, as a raw
/// slang::ast::SystemSubroutine::WithClauseMode value (0 = None, 1 =
/// Iterator, 2 = Randomize). A pure, allocation-free field read. history:
/// since 1.3.
SLANG_C_API uint32_t slang_system_subroutine_with_clause_mode(slang_system_subroutine sub);

/// Re-runs the protected slang::ast::SystemSubroutine::kindStr helper for
/// `sub`: "task" or "function" depending on slang_system_subroutine_kind.
/// Borrowed (points at process-lifetime static storage). A pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_str slang_system_subroutine_kind_str(slang_system_subroutine sub);

/// Re-runs the protected slang::ast::SystemSubroutine::badArg helper for the
/// system `call`'s argument at zero-based `arg_index` (a bound Call
/// expression whose subroutine is a system subroutine — see
/// slang_expr_call_system_subroutine), reporting a BadSystemSubroutineArg
/// diagnostic against that argument (visible afterwards through
/// slang_compilation_has_issued_errors) and returning the compilation's error
/// type, exactly as slang itself does when a built-in rejects one of its own
/// arguments. Since reporting the diagnostic allocates into the compilation's
/// arena, this requires exclusive access the same way
/// slang_system_subroutine_check_arguments does. Fails (via `err`) if `call`
/// is not a bound system call or `arg_index` is out of range. history: since
/// 1.3.
SLANG_C_API slang_ast slang_system_subroutine_bad_arg(slang_ast call, uint32_t arg_index,
                                                       slang_error* err);

/// Re-runs the protected slang::ast::SystemSubroutine::checkArgCount helper
/// for the system `call` (see slang_system_subroutine_bad_arg), checking
/// whether its already-bound argument count (adjusted by `is_method`, exactly
/// as slang::ast::SimpleSystemSubroutine does) falls within [`min`, `max`],
/// reporting a TooFewArguments/TooManyArguments diagnostic and returning
/// false if not. Since a failing check reports a diagnostic into the
/// compilation's arena, this requires exclusive access the same way
/// slang_system_subroutine_check_arguments does. Fails (via `err`) if `call`
/// is not a bound system call. history: since 1.3.
SLANG_C_API bool slang_system_subroutine_check_arg_count(slang_ast call, bool is_method,
                                                          uint32_t min, uint32_t max,
                                                          slang_error* err);

/// Re-runs the protected slang::ast::SystemSubroutine::noHierarchical helper
/// for the system `call`'s argument at zero-based `arg_index` (see
/// slang_system_subroutine_bad_arg), returning false (and reporting a
/// SysFuncHierarchicalNotAllowed diagnostic) if that argument contains a
/// hierarchical reference, true otherwise. The diagnostic is reported
/// through a scratch EvalContext built just for this replay, so — unlike
/// slang_system_subroutine_bad_arg and slang_system_subroutine_check_arg_count,
/// which report through the call's own ASTContext directly into the
/// compilation — it is never flushed and is not visible through
/// slang_compilation_has_issued_errors; only the returned bool is. Since
/// building that scratch context still requires unfreezing the compilation,
/// this requires exclusive access the same way
/// slang_system_subroutine_check_arguments does. Fails (via `err`) if `call`
/// is not a bound system call or `arg_index` is out of range. history: since
/// 1.3.
SLANG_C_API bool slang_system_subroutine_no_hierarchical(slang_ast call, uint32_t arg_index,
                                                          slang_error* err);

/// Re-runs the protected slang::ast::SystemSubroutine::notConst helper for
/// the system `call` (see slang_system_subroutine_bad_arg): unconditionally
/// reports a SysFuncNotConst diagnostic against the call's own source range
/// and returns false, exactly as a slang::ast::NonConstantFunction's eval
/// does. As with slang_system_subroutine_no_hierarchical, the diagnostic is
/// reported through a scratch EvalContext that is never flushed, so it is
/// not visible through slang_compilation_has_issued_errors; only the false
/// return value is. Since building that scratch context still requires
/// unfreezing the compilation, this requires exclusive access the
/// same way slang_system_subroutine_check_arguments does. Fails (via `err`)
/// if `call` is not a bound system call. history: since 1.3.
SLANG_C_API bool slang_system_subroutine_not_const(slang_ast call, slang_error* err);

/// Re-runs the protected static slang::ast::SystemSubroutine::unevaluatedContext
/// helper against a synthetic ASTContext built over the system `call`'s own
/// scope, and reports whether the transformation it performs (clearing the
/// StaticInitializer flag while leaving every other flag untouched) actually
/// took place. This exercises the helper's logic directly rather than
/// exposing an ASTContext value, since ASTContext is not otherwise part of
/// the C API. Since building the synthetic contexts requires exclusive
/// access to compare against the call's own frozen state, this requires
/// exclusive access the same way slang_system_subroutine_check_arguments
/// does. Fails (via `err`) if `call` is not a bound system call. history:
/// since 1.3.
SLANG_C_API bool slang_system_subroutine_unevaluated_context_clears_static_initializer(
    slang_ast call, slang_error* err);

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

/// The symbol's lexical path, walking up to the compilation unit and joining
/// each parent's name with "." (or "::" between a package/class/covergroup
/// and its member), e.g. "pkg::C::f". Unlike slang_symbol_hierarchical_path
/// this does not walk through instance bodies, so it reflects the textual
/// nesting a `resolve` or `$typename`-style reference would use, not the
/// instantiated hierarchy. Owned. slang::ast::Symbol::getLexicalPath walks
/// only already-resolved parent-scope pointers and builds a fresh
/// std::string on the caller's own heap (no slang arena allocation), so this
/// is a pure read. history: since 1.3.
SLANG_C_API slang_str slang_symbol_lexical_path(slang_ast symbol, slang_error* err);

/// True if the symbol is a scope (module body, package, class, block, ...).
SLANG_C_API bool slang_symbol_is_scope(slang_ast symbol);

/// True if the symbol is a type.
SLANG_C_API bool slang_symbol_is_type(slang_ast symbol);

/// True if the symbol has a value (variable, net, parameter, port, ...).
SLANG_C_API bool slang_symbol_is_value(slang_ast symbol);

/// True if the symbol carries a slang::ast::DeclaredType (see
/// slang_declared_type_type and the other slang_declared_type_* accessors,
/// which all take the carrier symbol itself rather than a separate handle).
/// Mirrors `Symbol::getDeclaredType() != nullptr`. The underlying memo (when
/// present) is the same one the freeze sweep force-resolves for every
/// DeclaredType carrier, so this is a pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API bool slang_symbol_has_declared_type(slang_ast symbol);

/// The nearest enclosing definition (module/interface/program) this symbol
/// is declared within, as a node of domain SLANG_AST_SYMBOL
/// (SymbolKind::Definition). A null node if the symbol isn't declared inside
/// any definition (e.g. it lives in a package or in $unit).
/// slang::ast::Symbol::getDeclaringDefinition walks already-resolved parent
/// scope pointers up to the nearest InstanceBody and reads its (constant,
/// set-at-construction) Definition reference -- a pure, allocation-free
/// read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_declaring_definition(slang_ast symbol);

/// The source library that contains the symbol, as a slang_source_library
/// handle (see slang_source_library_name and friends). The null handle if
/// the symbol has no enclosing CompilationUnit/Definition/Instance (e.g. the
/// root symbol itself). slang::ast::Symbol::getSourceLibrary walks
/// already-resolved parent scope pointers and reads a `sourceLibrary` field
/// set once at construction -- a pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API slang_source_library slang_symbol_source_library(slang_ast symbol);

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

/// The compilation that owns `scope` (slang::ast::Scope::getCompilation), as
/// the same slang_compilation handle every slang_ast node already carries in
/// its `compilation` field. The null handle if `scope` is not a scope. A
/// direct field read -- a pure, allocation-free read that returns a borrowed
/// handle (do not free it). history: since 1.3.
SLANG_C_API slang_compilation slang_scope_get_compilation(slang_ast scope);

/// The compilation unit that contains `scope`, if any
/// (slang::ast::Scope::getCompilationUnit), as a node of domain
/// SLANG_AST_SYMBOL (SymbolKind::CompilationUnit). A null node if `scope` is
/// not a scope, or the scope is not nested under a compilation unit (e.g. it
/// is the $root scope itself, or belongs to a script session). Walks the
/// parent-scope chain with no caching and no allocation, so this is a pure
/// read. history: since 1.3.
SLANG_C_API slang_ast slang_scope_get_compilation_unit(slang_ast scope);

/// The instance body that contains `scope`, if any
/// (slang::ast::Scope::getContainingInstance), as a node of domain
/// SLANG_AST_SYMBOL (SymbolKind::InstanceBody). A null node if `scope` is
/// not a scope, or no enclosing instance exists (e.g. a package, or a
/// checker not instantiated inside a module). Walks the parent-scope chain
/// with no caching and no allocation, so this is a pure read.
/// history: since 1.3.
SLANG_C_API slang_ast slang_scope_get_containing_instance(slang_ast scope);

/// The default net type for implicit nets in `scope`
/// (slang::ast::Scope::getDefaultNetType), as a node of domain
/// SLANG_AST_SYMBOL (SymbolKind::NetType). Never null for an actual scope --
/// slang always resolves to a concrete net type, falling back to the
/// compilation's default `wire` if nothing in the enclosing chain overrides
/// it. A null node if `scope` is not a scope. Walks the parent-scope chain
/// with no caching and no allocation, so this is a pure read.
/// history: since 1.3.
SLANG_C_API slang_ast slang_scope_get_default_net_type(slang_ast scope);

/// The time scale for delay values expressed within `scope`
/// (slang::ast::Scope::getTimeScale). Writes the result to `*out` and
/// returns true if resolved -- like the C++ method, this falls back to the
/// compilation's default time scale when nothing in the enclosing chain sets
/// one explicitly, so it returns true for any actual scope in a compilation
/// that has a default time scale (see slang_compilation_get_default_time_scale).
/// Returns false (leaving `*out` unmodified) if `scope` is not a scope, or no
/// time scale could be resolved at all. Walks the parent-scope chain with no
/// caching and no allocation, so this is a pure read. history: since 1.3.
SLANG_C_API bool slang_scope_get_time_scale(slang_ast scope, slang_time_scale* out);

/// True if `scope` represents a procedural context -- a procedural block, or
/// a task/function scope (slang::ast::Scope::isProceduralContext). False if
/// `scope` is not a scope, or is a scope with no procedural context (e.g. a
/// module, package, or generate block). A direct symbol-kind check -- a
/// pure, allocation-free read. history: since 1.3.
SLANG_C_API bool slang_scope_is_procedural_context(slang_ast scope);

/// True if `scope` is in an uninstantiated context -- e.g. inside a module
/// that is never instantiated in the design, or when the compilation runs in
/// lint mode (slang::ast::Scope::isUninstantiated). False if `scope` is not
/// a scope. Walks the parent-scope chain with no caching and no allocation,
/// so this is a pure read. history: since 1.3.
SLANG_C_API bool slang_scope_is_uninstantiated(slang_ast scope);

/// The declared type of a value symbol. Null node if the symbol has no type.
SLANG_C_API slang_ast slang_value_type(slang_ast symbol, slang_error* err);

/// The initializer expression of a value symbol, or the null node.
SLANG_C_API slang_ast slang_value_initializer(slang_ast symbol, slang_error* err);

/* slang::ast::DeclaredType is the general glue between a symbol and its type
 * that slang_value_type/slang_value_initializer above expose only for
 * ValueSymbol; the accessors below work for every carrier (ValueSymbol,
 * TypeAliasType, SubroutineSymbol, MethodPrototypeSymbol, NetType,
 * TypeParameterSymbol, AssertionPortSymbol, RandSeqProductionSymbol,
 * CoverpointSymbol), via Symbol::getDeclaredType(). history: since 1.3. */

/// The resolved type of a symbol's declared type, as a node of domain
/// SLANG_AST_SYMBOL; a null node for a symbol with no declared type. The
/// underlying memo is forced by the freeze sweep for every carrier, so this
/// is a pure read. Mirrors slang::ast::DeclaredType::getType.
/// history: since 1.3.
SLANG_C_API slang_ast slang_declared_type_type(slang_ast sym);

/// The resolved initializer expression of a symbol's declared type; a null
/// node if there is none, or the symbol has no declared type. Forced by the
/// freeze sweep, so this is a pure read. Mirrors
/// slang::ast::DeclaredType::getInitializer. history: since 1.3.
SLANG_C_API slang_ast slang_declared_type_initializer(slang_ast sym);

/// The source location to use when reporting diagnostics about a symbol's
/// declared-type initializer (set alongside the initializer syntax, before
/// resolution); a zeroed location if the symbol has no declared type or no
/// initializer syntax was ever set. A pure, allocation-free read. Mirrors
/// slang::ast::DeclaredType::getInitializerLocation. history: since 1.3.
SLANG_C_API slang_loc slang_declared_type_initializer_location(slang_ast sym);

/// The initializer expression syntax previously set on a symbol's declared
/// type, as a syntax node; the null node if none was set (or the symbol has
/// no declared type). A pure, allocation-free read. Mirrors
/// slang::ast::DeclaredType::getInitializerSyntax. history: since 1.3.
SLANG_C_API slang_node slang_declared_type_initializer_syntax(slang_ast sym);

/// The type syntax set directly on a symbol's declared type (NOT following a
/// link to another declared type — see slang::ast::DeclaredType::getTypeSyntax
/// vs getResolvedTypeSyntax), as a syntax node; the null node if the symbol
/// has no declared type, or its type links to another declared type instead
/// of carrying its own syntax. A pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API slang_node slang_declared_type_type_syntax(slang_ast sym);

/// True if a symbol's declared type is in the process of resolving (used
/// internally by slang to detect self-referential type cycles); false once
/// resolution has completed, or if the symbol has no declared type. On a
/// frozen design type resolution is always already complete, so this is
/// always false there; still a pure, allocation-free read. Mirrors
/// slang::ast::DeclaredType::isEvaluating. history: since 1.3.
SLANG_C_API bool slang_declared_type_is_evaluating(slang_ast sym);

/// For an Instance symbol: its body (an InstanceBody symbol, which is a scope).
SLANG_C_API slang_ast slang_instance_body(slang_ast instance);

/// For an Instance symbol: the Definition it instantiates.
SLANG_C_API slang_ast slang_instance_definition(slang_ast instance);

/// For an Instance symbol: the number of parameters (port and body, value and
/// type) of its body, and each one as a Parameter or TypeParameter symbol.
SLANG_C_API uint32_t slang_instance_parameter_count(slang_ast instance);
SLANG_C_API slang_ast slang_instance_parameter(slang_ast instance, uint32_t index);

/// For an Instance symbol: true if the definition it instantiates is a
/// `module`. False for any other symbol kind, and for an Instance of an
/// `interface` or `program` definition. Mirrors
/// slang::ast::InstanceSymbol::isModule. A pure, allocation-free read (a
/// direct comparison against the already-forced DefinitionSymbol::
/// definitionKind reached through getDefinition()). history: since 1.3.
SLANG_C_API bool slang_instance_is_module(slang_ast instance);

/// For an Instance symbol: true if the definition it instantiates is an
/// `interface`. False for any other symbol kind, and for an Instance of a
/// `module` or `program` definition. Mirrors
/// slang::ast::InstanceSymbol::isInterface. A pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API bool slang_instance_is_interface(slang_ast instance);

/* An Instance symbol's resolved port connections
 * (slang::ast::InstanceSymbol::getPortConnections / PortConnection). Unlike
 * a CheckerInstance's Connection (slang_symbol_checker_instance_connection_*
 * above), a regular PortConnection always carries exactly one bound
 * expression (or none, for an unconnected/default port). The connection list
 * is lazily resolved (getPortConnections) but the freeze sweep forces it --
 * and every connection's getExpression() -- for every Instance symbol
 * reached, so all of the accessors below are pure reads on a frozen design.
 * history: since 1.3. */

/// The number of resolved port connections of an Instance symbol. 0 for a
/// non-Instance symbol.
SLANG_C_API uint32_t slang_instance_port_connection_count(slang_ast instance);

/// The i'th connection's port symbol (a Port, MultiPort, or InterfacePort
/// symbol of the instance's body), as a node of domain SLANG_AST_SYMBOL. A
/// null node if `index` is out of range or `instance` is not an Instance
/// symbol.
SLANG_C_API slang_ast slang_instance_port_connection_port(slang_ast instance, uint32_t index);

/// The i'th connection's bound expression, as a node of domain
/// SLANG_AST_EXPRESSION; a null node if the connection has no expression
/// (an unconnected port, an interface port connection, or a default value
/// with no syntax of its own), or if `index` is out of range or `instance`
/// is not an Instance symbol.
SLANG_C_API slang_ast slang_instance_port_connection_expression(slang_ast instance,
                                                                 uint32_t index);

/// True if the i'th connection was left implicit (`.name` shorthand, or
/// `.*`). False if `index` is out of range or `instance` is not an Instance
/// symbol. Mirrors slang::ast::PortConnection::isImplicit.
SLANG_C_API bool slang_instance_port_connection_is_implicit(slang_ast instance, uint32_t index);

/// True if the i'th connection was produced by a `.*` wildcard. False if
/// `index` is out of range or `instance` is not an Instance symbol. Mirrors
/// slang::ast::PortConnection::isWildcard.
SLANG_C_API bool slang_instance_port_connection_is_wildcard(slang_ast instance, uint32_t index);

/// For an InstanceBody symbol: the InstanceSymbol that owns it, as a node of
/// domain SLANG_AST_SYMBOL; a null node if it has none (e.g. a body created
/// for a virtual-interface placeholder or a default-instantiated definition
/// with no enclosing Instance), or if `sym` is not an InstanceBody symbol. A
/// direct field read (slang::ast::InstanceBodySymbol::parentInstance) -- a
/// pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_instance_body_parent_instance(slang_ast sym);

/// For an InstanceBody symbol: the Definition it was elaborated from, as a
/// node of domain SLANG_AST_SYMBOL; a null node if `sym` is not an
/// InstanceBody symbol. A direct field read
/// (slang::ast::InstanceBodySymbol::getDefinition) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_instance_body_definition(slang_ast sym);

/// For an InstanceBody symbol: its port list (Port, MultiPort, and
/// InterfacePort symbols, in declaration order), and each one as a node of
/// domain SLANG_AST_SYMBOL. 0 / a null node if `sym` is not an InstanceBody
/// symbol or `index` is out of range. The port list is populated by
/// ensureElaborated(), which the freeze sweep's generic scope-member
/// traversal forces for every reached Scope (including every InstanceBody),
/// so this is a pure read. Mirrors
/// slang::ast::InstanceBodySymbol::getPortList. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_instance_body_port_count(slang_ast sym);
SLANG_C_API slang_ast slang_symbol_instance_body_port(slang_ast sym, uint32_t index);

/// For an InstanceBody symbol: the port with the given name (searched among
/// its port list, see slang_symbol_instance_body_port_count), as a node of
/// domain SLANG_AST_SYMBOL; a null node if there is no such port, `name` is
/// empty, or `sym` is not an InstanceBody symbol. Like the port list itself,
/// this reads only the already-elaborated port list (forced by the freeze
/// sweep) -- a pure read, not a fresh name lookup. Mirrors
/// slang::ast::InstanceBodySymbol::findPort. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_instance_body_find_port(slang_ast sym, const char* name,
                                                            size_t name_len);

/// True if two InstanceBody symbols were elaborated from the same
/// Definition with equivalent parameter values (and so share identical
/// member layout) -- i.e. they could share a single canonical elaborated
/// body. False if either symbol is not an InstanceBody symbol. Mirrors
/// slang::ast::InstanceBodySymbol::hasSameType. A pure, allocation-free
/// read. history: since 1.3.
SLANG_C_API bool slang_symbol_instance_body_has_same_type(slang_ast sym, slang_ast other);

/// For an Instance, PrimitiveInstance, or CheckerInstance symbol: the path
/// of zero-based indices locating it within any enclosing instance array(s)
/// (outermost array first), and the i'th entry of that path. 0 / an
/// unspecified value if `sym` is not one of those kinds or `index` is out of
/// range -- always check slang_symbol_instance_array_path_count first. A
/// direct field read (slang::ast::InstanceSymbolBase::arrayPath) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_instance_array_path_count(slang_ast sym);
SLANG_C_API uint32_t slang_symbol_instance_array_path(slang_ast sym, uint32_t index);

/// For an Instance, PrimitiveInstance, or CheckerInstance symbol: if it is
/// part of an instance array, the name of the outermost enclosing array;
/// otherwise the instance's own name. Empty if `sym` is not one of those
/// kinds. A pure read over already-resolved parent-scope links (walks
/// upward through InstanceArraySymbol parents) -- never allocates; returns a
/// borrowed view into the frozen arena. Mirrors
/// slang::ast::InstanceSymbolBase::getArrayName. history: since 1.3.
SLANG_C_API slang_str slang_symbol_instance_base_array_name(slang_ast sym);

/// For an InstanceArray symbol: the number of instance elements it contains,
/// and each one as a node of domain SLANG_AST_SYMBOL (an Instance,
/// PrimitiveInstance, CheckerInstance, or nested InstanceArray symbol). 0 /
/// a null node if `sym` is not an InstanceArray symbol or `index` is out of
/// range. A direct field read (slang::ast::InstanceArraySymbol::elements) --
/// a pure, allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_instance_array_element_count(slang_ast sym);
SLANG_C_API slang_ast slang_symbol_instance_array_element(slang_ast sym, uint32_t index);

/// For an InstanceArray symbol: if this array is itself an element of an
/// outer (multidimensional) instance array, the outermost array's name;
/// otherwise this array's own name. Empty if `sym` is not an InstanceArray
/// symbol. A pure read over already-resolved parent-scope links; never
/// allocates; returns a borrowed view into the frozen arena. Mirrors
/// slang::ast::InstanceArraySymbol::getArrayName. history: since 1.3.
SLANG_C_API slang_str slang_symbol_instance_array_name(slang_ast sym);

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

/* An assertion-item / clocking-var argument direction (`input`, `output`,
 * `inout`, `ref`), shared by AssertionPortSymbol, ClockVarSymbol and
 * FormalArgumentSymbol. Mirrors slang::ast::ArgumentDirection.
 * SLANG_ARGUMENT_DIRECTION_NONE is not one of slang's own enumerators: it is
 * this API's encoding of an empty std::optional<ArgumentDirection> (see
 * slang_symbol_assertion_port_direction). history: since 1.3. */
typedef enum slang_argument_direction {
    SLANG_ARGUMENT_DIRECTION_IN = 0,
    SLANG_ARGUMENT_DIRECTION_OUT = 1,
    SLANG_ARGUMENT_DIRECTION_INOUT = 2,
    SLANG_ARGUMENT_DIRECTION_REF = 3,
    SLANG_ARGUMENT_DIRECTION_NONE = 4,
} slang_argument_direction;

/// For an AssertionPort symbol (a formal port of a sequence, property, let,
/// or checker declaration): its direction if it is a local variable port
/// (`local [input|output|...]`), or SLANG_ARGUMENT_DIRECTION_NONE if it is
/// not (see slang_symbol_assertion_port_is_local_var). A direct field read
/// (slang::ast::AssertionPortSymbol::direction), populated at declaration —
/// a pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_argument_direction slang_symbol_assertion_port_direction(slang_ast sym);

/// For an AssertionPort symbol: true if it is a local variable port (`local
/// ...`) rather than a plain formal port. Mirrors
/// slang::ast::AssertionPortSymbol::isLocalVar (`direction.has_value()`). A
/// pure, allocation-free read. history: since 1.3.
SLANG_C_API bool slang_symbol_assertion_port_is_local_var(slang_ast sym);

/// For a CheckerInstanceBody symbol: the CheckerInstance symbol this is the
/// body of, as a node of domain SLANG_AST_SYMBOL; a null node if there is
/// none (or `sym` is not a CheckerInstanceBody). A direct field read
/// (slang::ast::CheckerInstanceBodySymbol::parentInstance) — a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_checker_instance_body_parent_instance(slang_ast sym);

/// For a Checker symbol (a `checker` declaration): the number of formal
/// ports, and each one as an AssertionPort symbol (node of domain
/// SLANG_AST_SYMBOL). A direct field read (slang::ast::CheckerSymbol::ports)
/// — a pure, allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_checker_port_count(slang_ast sym);
SLANG_C_API slang_ast slang_symbol_checker_port(slang_ast sym, uint32_t index);

/* A CheckerInstance symbol's resolved port connections
 * (slang::ast::CheckerInstanceSymbol::getPortConnections /
 * CheckerInstanceSymbol::Connection). The connection list is lazily resolved
 * (getPortConnections) but the freeze sweep forces it — and every
 * connection's getOutputInitialExpr — for every CheckerInstance symbol
 * reached, so all of the accessors below are pure reads on a frozen design.
 * history: since 1.3. */

/// The number of resolved port connections of a CheckerInstance symbol. 0
/// for a non-CheckerInstance symbol.
SLANG_C_API uint32_t slang_symbol_checker_instance_connection_count(slang_ast instance);

/// The i'th connection's actual argument, as a node of the domain that
/// matches its kind (SLANG_AST_EXPRESSION for a plain formal argument,
/// SLANG_AST_ASSERTION_EXPR for a sequence/property/let actual,
/// SLANG_AST_TIMING_CONTROL for a clocking-event actual). A null ast (domain
/// SLANG_AST_EXPRESSION) if `index` is out of range, `instance` is not a
/// CheckerInstance symbol, or the connection has no resolved actual. Mirrors
/// slang::ast::CheckerInstanceSymbol::Connection::actual. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_checker_instance_connection_actual(slang_ast instance,
                                                                       uint32_t index);

/// The number of attribute instances attached directly to the i'th port
/// connection (e.g. `(* foo = 1 *) .i(a)`). 0 if `index` is out of range or
/// `instance` is not a CheckerInstance symbol.
SLANG_C_API uint32_t slang_symbol_checker_instance_connection_attribute_count(slang_ast instance,
                                                                               uint32_t index);

/// The attr_index'th attribute of the i'th port connection, as a node of
/// domain SLANG_AST_SYMBOL (an Attribute symbol). A null node if either
/// index is out of range. Mirrors
/// slang::ast::CheckerInstanceSymbol::Connection::attributes.
/// history: since 1.3.
SLANG_C_API slang_ast slang_symbol_checker_instance_connection_attribute(slang_ast instance,
                                                                          uint32_t index,
                                                                          uint32_t attr_index);

/// The i'th connection's output-port initial-value expression (the `= expr`
/// of an output checker formal, evaluated in the instantiation's context),
/// as a node of domain SLANG_AST_EXPRESSION. A null node if `index` is out
/// of range, `instance` is not a CheckerInstance symbol, the connection's
/// formal has no default, or the formal is not an output port. Mirrors
/// slang::ast::CheckerInstanceSymbol::Connection::getOutputInitialExpr.
/// history: since 1.3.
SLANG_C_API slang_ast slang_symbol_checker_instance_connection_output_initial_expr(
    slang_ast instance, uint32_t index);

/// An edge kind for a clocking skew specification. Mirrors
/// slang::ast::EdgeKind. history: since 1.3.
typedef enum slang_edge_kind {
    SLANG_EDGE_NONE = 0,
    SLANG_EDGE_POSEDGE = 1,
    SLANG_EDGE_NEGEDGE = 2,
    SLANG_EDGE_BOTHEDGES = 3,
} slang_edge_kind;

/// A clocking-block input/output skew specification: an edge (posedge /
/// negedge / edge / unspecified) plus an optional delay, e.g. the `posedge
/// #3` of `default input posedge #3;` or the `#1step` of `input #1step x;`.
/// `delay` is a node of domain SLANG_AST_TIMING_CONTROL, null if none was
/// specified. Mirrors slang::ast::ClockingSkew. history: since 1.3.
typedef struct slang_clocking_skew {
    slang_edge_kind edge;
    slang_ast delay;
} slang_clocking_skew;

/// True if `skew` carries any explicit skew information (a non-default edge
/// or a delay) — mirrors slang::ast::ClockingSkew::hasValue. Returns false for
/// a default-constructed (zeroed) skew. A pure function of its argument (no
/// allocation, no handle). history: since 1.3.
SLANG_C_API bool slang_clocking_skew_has_value(slang_clocking_skew skew);

/// For a ClockVar symbol (a clocking-block signal, e.g. the `x` of `input
/// #1step output #1step x;`): its direction (always has a value, unlike
/// slang_symbol_assertion_port_direction). A direct field read
/// (slang::ast::ClockVarSymbol::direction) — a pure, allocation-free read.
/// Returns SLANG_ARGUMENT_DIRECTION_IN for a non-ClockVar symbol.
/// history: since 1.3.
SLANG_C_API slang_argument_direction slang_symbol_clock_var_direction(slang_ast sym);

/// For a ClockVar symbol: its input skew (the part of its declaration that
/// applies when the signal is sampled as an input), or a zeroed skew
/// (SLANG_EDGE_NONE, no delay) for a non-ClockVar symbol. A direct field
/// read (slang::ast::ClockVarSymbol::inputSkew) — a pure, allocation-free
/// read. history: since 1.3.
SLANG_C_API slang_clocking_skew slang_symbol_clock_var_input_skew(slang_ast sym);

/// For a ClockVar symbol: its output skew, analogous to
/// slang_symbol_clock_var_input_skew. Mirrors
/// slang::ast::ClockVarSymbol::outputSkew. history: since 1.3.
SLANG_C_API slang_clocking_skew slang_symbol_clock_var_output_skew(slang_ast sym);

/// For a ClockingBlock symbol: the `default input` skew declared in its body
/// (e.g. `default input posedge #3;`), or a zeroed skew (SLANG_EDGE_NONE, no
/// delay) if none was declared, or `sym` is not a ClockingBlock symbol. The
/// underlying memo is forced by the freeze sweep, so this is a pure read.
/// Mirrors slang::ast::ClockingBlockSymbol::getDefaultInputSkew.
/// history: since 1.3.
SLANG_C_API slang_clocking_skew slang_symbol_clocking_block_default_input_skew(slang_ast sym);

/// For a ClockingBlock symbol: the `default output` skew, analogous to
/// slang_symbol_clocking_block_default_input_skew. Mirrors
/// slang::ast::ClockingBlockSymbol::getDefaultOutputSkew.
/// history: since 1.3.
SLANG_C_API slang_clocking_skew slang_symbol_clocking_block_default_output_skew(slang_ast sym);

/// For a ClockingBlock symbol: the clocking event (e.g. the `@clk` of
/// `clocking cb @clk;`), as a node of domain SLANG_AST_TIMING_CONTROL. A null
/// node if `sym` is not a ClockingBlock symbol. The underlying memo is forced
/// by the freeze sweep, so this is a pure read. Mirrors
/// slang::ast::ClockingBlockSymbol::getEvent. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_clocking_block_event(slang_ast sym);

/// For a ContinuousAssign symbol (an `assign lhs = rhs;` statement): its
/// bound assignment expression, as a node of domain SLANG_AST_EXPRESSION. A
/// null node if `sym` is not a ContinuousAssign symbol. The underlying memo
/// is forced by the freeze sweep, so this is a pure read. Mirrors
/// slang::ast::ContinuousAssignSymbol::getAssignment. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_continuous_assign_assignment(slang_ast sym);

/// For a ContinuousAssign symbol: its delay control (the `#2` of `assign #2
/// y = a;`), as a node of domain SLANG_AST_TIMING_CONTROL. A null node if
/// `sym` is not a ContinuousAssign symbol, or it has no delay. The underlying
/// memo is forced by the freeze sweep, so this is a pure read. Mirrors
/// slang::ast::ContinuousAssignSymbol::getDelay. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_continuous_assign_delay(slang_ast sym);

/// A net/gate drive strength level (`supply0`/`strong1`/`pull0`/`weak1`/
/// `highz0`, independent of which value it drives). Mirrors
/// slang::ast::DriveStrength. history: since 1.3.
typedef enum slang_drive_strength {
    SLANG_DRIVE_STRENGTH_SUPPLY = 0,
    SLANG_DRIVE_STRENGTH_STRONG = 1,
    SLANG_DRIVE_STRENGTH_PULL = 2,
    SLANG_DRIVE_STRENGTH_WEAK = 3,
    SLANG_DRIVE_STRENGTH_HIGHZ = 4,
} slang_drive_strength;

/// A pair of optional drive strengths: the strength used when the net is
/// driven to 0 (strength0) and to 1 (strength1), e.g. the `(strong0,
/// pull1)` of `assign (strong0, pull1) y = a;`. Mirrors
/// std::pair<std::optional<slang::ast::DriveStrength>,
/// std::optional<slang::ast::DriveStrength>>. Position struct: trivially
/// copyable. history: since 1.3.
typedef struct slang_drive_strength_pair {
    bool has_strength0;
    slang_drive_strength strength0;
    bool has_strength1;
    slang_drive_strength strength1;
} slang_drive_strength_pair;

/// For a ContinuousAssign symbol: its explicit drive strength, if any (e.g.
/// the `(strong0, pull1)` of `assign (strong0, pull1) y = a;`). Both
/// `has_strength0`/`has_strength1` are false if `sym` is not a
/// ContinuousAssign symbol, or it has no strength specification. Recomputed
/// from syntax on every call (no arena allocation) — a pure, allocation-free
/// read. Mirrors slang::ast::ContinuousAssignSymbol::getDriveStrength.
/// history: since 1.3.
SLANG_C_API slang_drive_strength_pair slang_symbol_continuous_assign_drive_strength(
    slang_ast sym);

/// For a CompilationUnit symbol (the root scope of one compilation unit):
/// the time scale in effect for declarations placed directly at its own
/// ($unit) scope — set by an explicit `timeunit`/`timeprecision`
/// declaration there, or else defaulted at construction to the
/// compilation's configured default time scale (see
/// slang_compilation_get_default_time_scale). Note this is distinct from a
/// module's own effective time scale, which instead comes from its nearest
/// enclosing `` `timescale`` directive (tracked per-definition, not on this
/// symbol). Writes the result to `*out` and returns true if present; returns
/// false (leaving `*out` untouched) if `sym` is not a CompilationUnit
/// symbol, or neither source applies. A direct field read
/// (slang::ast::CompilationUnitSymbol::timeScale) — a pure, allocation-free
/// read. history: since 1.3.
SLANG_C_API bool slang_symbol_compilation_unit_time_scale(slang_ast sym, slang_time_scale* out);

/// For a CoverCrossBody symbol (the hidden scope holding a cover cross's own
/// members): the synthesized queue type of its cross-coverage values (the
/// type of an implicit `cross.name` iteration), as a node of domain
/// SLANG_AST_SYMBOL (a Type). A null node if `sym` is not a CoverCrossBody
/// symbol. A direct field read (slang::ast::CoverCrossBodySymbol::
/// crossQueueType), set once when the body is constructed — a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_cover_cross_body_queue_type(slang_ast sym);

/// For a CoverCross symbol (a `cross` declaration inside a covergroup): its
/// `iff` guard expression, if any, as a node of domain SLANG_AST_EXPRESSION.
/// A null node if `sym` is not a CoverCross symbol, or it has no `iff`
/// clause. The underlying memo is forced by the freeze sweep, so this is a
/// pure read. Mirrors slang::ast::CoverCrossSymbol::getIffExpr.
/// history: since 1.3.
SLANG_C_API slang_ast slang_symbol_cover_cross_iff_expr(slang_ast sym);

/// For a CoverCross symbol: the number of coverpoints it crosses, and each
/// one as a Coverpoint symbol (node of domain SLANG_AST_SYMBOL). 0 / a null
/// node if `sym` is not a CoverCross symbol or `index` is out of range. A
/// direct field read (slang::ast::CoverCrossSymbol::targets) — a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_cover_cross_target_count(slang_ast sym);
SLANG_C_API slang_ast slang_symbol_cover_cross_target(slang_ast sym, uint32_t index);

/// The number of `option`/`type_option` setters declared directly in the
/// body of a coverage-option-setter owner (e.g. the `option.weight = 2;` of
/// a cross, or the `option.per_instance = 1;` declared directly in a
/// covergroup or coverpoint body). `sym` may be a CoverCross, CovergroupBody,
/// or Coverpoint symbol — see slang_symbol_cover_cross_option_is_type_option.
/// 0 if `sym` is none of those kinds. A direct field read
/// (slang::ast::CoverCrossSymbol::options /
/// slang::ast::CovergroupBodySymbol::options /
/// slang::ast::CoverpointSymbol::options) — a pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API uint32_t slang_symbol_cover_cross_option_count(slang_ast sym);

/// True if the `index`'th option setter of a coverage-option-setter owner
/// sets `type_option.*` (a covergroup-type-wide option) rather than a plain
/// per-instance `option.*`. `sym` may be a CoverCross, CovergroupBody, or
/// Coverpoint symbol — every kind that can carry `option`/`type_option`
/// setters directly in its body — each exposing its own `options` span
/// through this same accessor family (named after the first owner it was
/// added for). False if `sym` is none of those kinds, or `index` is out of
/// range. Recomputed from syntax on every call — a pure, allocation-free
/// read. Mirrors slang::ast::CoverageOptionSetter::isTypeOption.
/// history: since 1.3.
SLANG_C_API bool slang_symbol_cover_cross_option_is_type_option(slang_ast sym, uint32_t index);

/// The option name being set by the `index`'th option setter of a
/// coverage-option-setter owner (e.g. "weight" for `option.weight = 2;`).
/// See slang_symbol_cover_cross_option_is_type_option for which symbol kinds
/// `sym` may be. Empty if `sym` is none of those kinds, `index` is out of
/// range, or the setter's left-hand side doesn't parse as
/// `option.name`/`type_option.name`. Borrowed. Recomputed from syntax on
/// every call — a pure, allocation-free read. Mirrors
/// slang::ast::CoverageOptionSetter::getName. history: since 1.3.
SLANG_C_API slang_str slang_symbol_cover_cross_option_name(slang_ast sym, uint32_t index);

/// The bound right-hand-side expression of the `index`'th option setter of a
/// coverage-option-setter owner, as a node of domain SLANG_AST_EXPRESSION.
/// See slang_symbol_cover_cross_option_is_type_option for which symbol kinds
/// `sym` may be. A null node if `sym` is none of those kinds or `index` is
/// out of range. The underlying memo is forced by the freeze sweep (as part
/// of the owning symbol's own expression walk), so this is a pure read.
/// Mirrors slang::ast::CoverageOptionSetter::getExpression.
/// history: since 1.3.
SLANG_C_API slang_ast slang_symbol_cover_cross_option_expression(slang_ast sym, uint32_t index);

/// For a Coverpoint symbol (a `coverpoint` declaration inside a covergroup):
/// its sampled coverage expression (e.g. the `a` of `cp: coverpoint a;`), as
/// a node of domain SLANG_AST_EXPRESSION. A null node if `sym` is not a
/// Coverpoint symbol. Backed by the symbol's declared-type initializer,
/// which — like every declared-type carrier (see slang_declared_type_type
/// above) — is forced by the freeze sweep, so this is a pure read. Mirrors
/// slang::ast::CoverpointSymbol::getCoverageExpr. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_coverpoint_coverage_expr(slang_ast sym);

/// For a Coverpoint symbol: its `iff` guard expression (e.g. the `en` of
/// `cp: coverpoint a iff (en);`), as a node of domain SLANG_AST_EXPRESSION. A
/// null node if `sym` is not a Coverpoint symbol or has no `iff` clause. The
/// underlying memo is forced by the freeze sweep, so this is a pure read.
/// Mirrors slang::ast::CoverpointSymbol::getIffExpr. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_coverpoint_iff_expr(slang_ast sym);

/// A coverage bin's kind (`bins`, `illegal_bins`, or `ignore_bins`). Mirrors
/// slang::ast::CoverageBinSymbol::BinKind. history: since 1.3.
typedef enum slang_coverage_bin_kind {
    SLANG_COVERAGE_BIN_BINS = 0,
    SLANG_COVERAGE_BIN_ILLEGAL_BINS = 1,
    SLANG_COVERAGE_BIN_IGNORE_BINS = 2,
} slang_coverage_bin_kind;

/// For a CoverageBin symbol (a `bins`/`illegal_bins`/`ignore_bins`
/// declaration inside a coverpoint, or a `bins` selection inside a cross):
/// its kind. SLANG_COVERAGE_BIN_BINS if `sym` is not a CoverageBin symbol. A
/// direct field read (slang::ast::CoverageBinSymbol::binsKind) — a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_coverage_bin_kind slang_symbol_coverage_bin_kind(slang_ast sym);

/// For a CoverageBin symbol: true if it was declared with `[...]` array
/// syntax (e.g. `bins b[]` / `bins b[4]`), regardless of whether an explicit
/// size expression was given (see slang_symbol_coverage_bin_number_of_bins_
/// expr). False if `sym` is not a CoverageBin symbol. A direct field read
/// (slang::ast::CoverageBinSymbol::isArray) — a pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API bool slang_symbol_coverage_bin_is_array(slang_ast sym);

/// For a CoverageBin symbol: true if declared with the `wildcard` qualifier
/// (`wildcard bins b = {...};`). False if `sym` is not a CoverageBin symbol.
/// A direct field read (slang::ast::CoverageBinSymbol::isWildcard) — a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API bool slang_symbol_coverage_bin_is_wildcard(slang_ast sym);

/// For a CoverageBin symbol: true if it's the coverpoint's catch-all
/// `default` bin (`bins b = default;`) — also true for a `default sequence`
/// bin (see slang_symbol_coverage_bin_is_default_sequence). False if `sym`
/// is not a CoverageBin symbol. A direct field read
/// (slang::ast::CoverageBinSymbol::isDefault) — a pure, allocation-free
/// read. history: since 1.3.
SLANG_C_API bool slang_symbol_coverage_bin_is_default(slang_ast sym);

/// For a CoverageBin symbol: true if it's specifically a `default sequence`
/// bin (`bins b = default sequence;`), as opposed to a plain `default` bin.
/// False if `sym` is not a CoverageBin symbol. A direct field read
/// (slang::ast::CoverageBinSymbol::isDefaultSequence) — a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API bool slang_symbol_coverage_bin_is_default_sequence(slang_ast sym);

/// For a CoverageBin symbol: its `iff` guard expression, if any (e.g. the
/// `iff (en)` of `bins b = {1} iff (en);`), as a node of domain
/// SLANG_AST_EXPRESSION. A null node if `sym` is not a CoverageBin symbol, or
/// it has no `iff` clause. The underlying memo is forced by the freeze
/// sweep, so this is a pure read. Mirrors
/// slang::ast::CoverageBinSymbol::getIffExpr. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_coverage_bin_iff_expr(slang_ast sym);

/// For a CoverageBin symbol: the explicit `[...]` bin-count expression of an
/// array bin (e.g. the `4` of `bins b[4] = {...}`), as a node of domain
/// SLANG_AST_EXPRESSION. A null node if `sym` is not a CoverageBin symbol, or
/// it has no explicit count (including a non-array bin, or an array bin
/// declared without one, e.g. `bins b[] = {...}`). The underlying memo is
/// forced by the freeze sweep, so this is a pure read. Mirrors
/// slang::ast::CoverageBinSymbol::getNumberOfBinsExpr. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_coverage_bin_number_of_bins_expr(slang_ast sym);

/// For a CoverageBin symbol initialized from a single expression that
/// denotes a whole coverage set rather than a value/range list (e.g. the
/// `q` of `bins b = q;` where `q` is an array-typed expression): that bound
/// expression, as a node of domain SLANG_AST_EXPRESSION. A null node if
/// `sym` is not a CoverageBin symbol, or it was initialized some other way
/// (a value/range list, a transition list, `default`, or a cross's `bins`
/// selection). The underlying memo is forced by the freeze sweep, so this is
/// a pure read. Mirrors slang::ast::CoverageBinSymbol::getSetCoverageExpr.
/// history: since 1.3.
SLANG_C_API slang_ast slang_symbol_coverage_bin_set_coverage_expr(slang_ast sym);

/// For a CoverageBin symbol: its `with (...)` filter expression, if any
/// (e.g. the `item > 12` of `bins b[2] = {[12:15]} with (item > 12);`,
/// evaluated once per candidate value with `item` bound to it), as a node of
/// domain SLANG_AST_EXPRESSION. A null node if `sym` is not a CoverageBin
/// symbol, or it has no `with` clause. The underlying memo is forced by the
/// freeze sweep, so this is a pure read. Mirrors
/// slang::ast::CoverageBinSymbol::getWithExpr. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_coverage_bin_with_expr(slang_ast sym);

/// For a CoverageBin symbol created from a cross's `bins` selection (e.g.
/// the `binsof(cp.hi)` of `bins sel = binsof(cp.hi);` inside a `cross`
/// body): its bound selection expression, as a node of domain
/// SLANG_AST_BINS_SELECT_EXPR. A null node if `sym` is not a CoverageBin
/// symbol, or it wasn't declared from a `BinsSelection` (an ordinary
/// coverpoint bin has none). The underlying memo is forced by the freeze
/// sweep, so this is a pure read. Mirrors
/// slang::ast::CoverageBinSymbol::getCrossSelectExpr. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_coverage_bin_cross_select_expr(slang_ast sym);

/// For a CoverageBin symbol initialized from a value or range list (e.g. the
/// `1, [3:5]` of `bins b = {1, [3:5]};`): the number of bound value/range
/// expressions. 0 if `sym` is not a CoverageBin symbol, or it was
/// initialized some other way. A direct-then-forced memo
/// (slang::ast::CoverageBinSymbol::getValues), forced by the freeze sweep —
/// a pure read. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_coverage_bin_value_count(slang_ast sym);

/// The `index`'th value/range expression of a CoverageBin symbol's
/// value/range-list initializer (see slang_symbol_coverage_bin_value_count),
/// as a node of domain SLANG_AST_EXPRESSION. A null node if `sym` is not a
/// CoverageBin symbol, or `index` is out of range. The underlying memo is
/// forced by the freeze sweep, so this is a pure read. Mirrors
/// slang::ast::CoverageBinSymbol::getValues. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_coverage_bin_value(slang_ast sym, uint32_t index);

/// How a `TransRangeList` repeats within a `bins` trans-set item, e.g. the
/// `[* 2]`/`[-> 2]`/`[= 2]` suffix of a transition range
/// (`1 => 2[*2] => 3`). Mirrors slang::ast::CoverageBinSymbol::
/// TransRangeList::RepeatKind. history: since 1.3.
typedef enum slang_repeat_kind {
    SLANG_REPEAT_KIND_NONE = 0,
    SLANG_REPEAT_KIND_CONSECUTIVE = 1,
    SLANG_REPEAT_KIND_NONCONSECUTIVE = 2,
    SLANG_REPEAT_KIND_GOTO = 3,
} slang_repeat_kind;

/* A CoverageBin symbol's transition list (`bins b = (1,2 => 3), (4=>5);`) —
 * slang::ast::CoverageBinSymbol::getTransList(), a span of "sets" (one per
 * comma-separated alternative), each a span of TransRangeList (one per
 * `=>`-separated range in that alternative). The freeze sweep forces
 * getTransList() (as part of the CoverageBin symbol's own expression walk,
 * which also visits every item/repeatFrom/repeatTo expression it holds), so
 * every accessor below is a pure read on a frozen design. Indexing is
 * two-level: `set_index` selects the comma-separated alternative, then
 * `range_index` selects the `=>`-separated range within it. history: since
 * 1.3. */

/// The number of trans-sets (comma-separated alternatives) of a CoverageBin
/// symbol's transition list. 0 if `sym` is not a CoverageBin symbol or has no
/// transition list.
SLANG_C_API uint32_t slang_symbol_coverage_bin_trans_set_count(slang_ast sym);

/// The number of `=>`-separated TransRangeList entries in trans-set
/// `set_index` of a CoverageBin symbol's transition list. 0 if `sym` is not a
/// CoverageBin symbol or `set_index` is out of range.
SLANG_C_API uint32_t slang_symbol_coverage_bin_trans_range_count(slang_ast sym,
                                                                  uint32_t set_index);

/// The number of value expressions in the `range_index`'th TransRangeList of
/// trans-set `set_index` (e.g. 2 for the `1,2` of `1,2 => 3`). 0 if either
/// index is out of range. Mirrors
/// slang::ast::CoverageBinSymbol::TransRangeList::items (the count).
SLANG_C_API uint32_t slang_symbol_coverage_bin_trans_range_item_count(slang_ast sym,
                                                                       uint32_t set_index,
                                                                       uint32_t range_index);

/// The `item_index`'th value expression of the `range_index`'th
/// TransRangeList of trans-set `set_index`, as a node of domain
/// SLANG_AST_EXPRESSION. A null node if any index is out of range. Mirrors
/// slang::ast::CoverageBinSymbol::TransRangeList::items (the element).
SLANG_C_API slang_ast slang_symbol_coverage_bin_trans_range_item(slang_ast sym,
                                                                  uint32_t set_index,
                                                                  uint32_t range_index,
                                                                  uint32_t item_index);

/// The repeat kind of the `range_index`'th TransRangeList of trans-set
/// `set_index` (SLANG_REPEAT_KIND_NONE if it has no `[...]` repeat suffix, or
/// either index is out of range). Mirrors
/// slang::ast::CoverageBinSymbol::TransRangeList::repeatKind.
SLANG_C_API slang_repeat_kind slang_symbol_coverage_bin_trans_range_repeat_kind(
    slang_ast sym, uint32_t set_index, uint32_t range_index);

/// The count (`[* n]`) or `from` bound (`[* from:to]`) of a repeat on the
/// `range_index`'th TransRangeList of trans-set `set_index`, as a node of
/// domain SLANG_AST_EXPRESSION. A null node if either index is out of
/// range, or the repeat has no `[...]` suffix at all. Mirrors
/// slang::ast::CoverageBinSymbol::TransRangeList::repeatFrom.
SLANG_C_API slang_ast slang_symbol_coverage_bin_trans_range_repeat_from(slang_ast sym,
                                                                        uint32_t set_index,
                                                                        uint32_t range_index);

/// The `to` bound of a `[* from:to]`/`[-> from:to]`/`[= from:to]`-style
/// range repeat on the `range_index`'th TransRangeList of trans-set
/// `set_index`, as a node of domain SLANG_AST_EXPRESSION. A null node if
/// either index is out of range, the repeat has no `[...]` suffix, or it is
/// a single fixed count (`[* n]`) rather than a `from:to` range. Mirrors
/// slang::ast::CoverageBinSymbol::TransRangeList::repeatTo.
SLANG_C_API slang_ast slang_symbol_coverage_bin_trans_range_repeat_to(slang_ast sym,
                                                                      uint32_t set_index,
                                                                      uint32_t range_index);

/* A Definition symbol (`module`/`interface`/`program` before any instance is
 * elaborated) — see slang_definition_kind_of above for its kind. Every field
 * below (cellDefine, defaultLifetime, timeScale, unconnectedDrive) is a plain
 * value populated once when the definition is parsed, and getKindString /
 * getArticleKindString / getInstanceCount are pure functions of already-set
 * state — none of it is a lazily-resolved memo, so every accessor here is a
 * pure, allocation-free read regardless of freeze. history: since 1.3. */

/// The default lifetime (`automatic` or `static`) for variables declared
/// within a definition or subroutine. Mirrors slang::ast::VariableLifetime.
typedef enum slang_variable_lifetime {
    SLANG_VARIABLE_LIFETIME_AUTOMATIC = 0,
    SLANG_VARIABLE_LIFETIME_STATIC = 1,
} slang_variable_lifetime;

/// The drive setting applied to an unconnected net within a definition.
/// Mirrors slang::ast::UnconnectedDrive.
typedef enum slang_unconnected_drive {
    SLANG_UNCONNECTED_DRIVE_NONE = 0,
    SLANG_UNCONNECTED_DRIVE_PULL0 = 1,
    SLANG_UNCONNECTED_DRIVE_PULL1 = 2,
} slang_unconnected_drive;

/// For a Definition symbol: whether it was declared with a `` `celldefine ``
/// directive in effect. False if `definition` is not a Definition symbol.
/// Mirrors the slang::ast::DefinitionSymbol::cellDefine field.
SLANG_C_API bool slang_definition_cell_define(slang_ast definition);

/// For a Definition symbol: the default lifetime (`automatic` or `static`)
/// for variables it declares. SLANG_VARIABLE_LIFETIME_AUTOMATIC if
/// `definition` is not a Definition symbol. Mirrors
/// slang::ast::DefinitionSymbol::defaultLifetime.
SLANG_C_API slang_variable_lifetime slang_definition_default_lifetime(slang_ast definition);

/// For a Definition symbol: the drive setting used for unconnected nets
/// within it. SLANG_UNCONNECTED_DRIVE_NONE if `definition` is not a
/// Definition symbol. Mirrors slang::ast::DefinitionSymbol::unconnectedDrive.
SLANG_C_API slang_unconnected_drive slang_definition_unconnected_drive(slang_ast definition);

/// For a Definition symbol: its timescale, written into `*out` and returning
/// true if one was explicitly specified for it (via a `` `timescale ``
/// directive or a per-definition override); false (leaving `*out` untouched)
/// if none was specified or `definition` is not a Definition symbol. Mirrors
/// slang::ast::DefinitionSymbol::timeScale (a std::optional<TimeScale>).
SLANG_C_API bool slang_definition_time_scale(slang_ast definition, slang_time_scale* out);

/// A string description of a Definition symbol's kind: "module", "interface",
/// or "program". Empty if `definition` is not a Definition symbol. Borrowed
/// (points at static storage). Mirrors slang::ast::DefinitionSymbol::
/// getKindString.
SLANG_C_API slang_str slang_definition_kind_string(slang_ast definition);

/// Like slang_definition_kind_string, but with an indefinite article: "a
/// module", "an interface", "a program". Empty if `definition` is not a
/// Definition symbol. Borrowed. Mirrors
/// slang::ast::DefinitionSymbol::getArticleKindString.
SLANG_C_API slang_str slang_definition_article_kind_string(slang_ast definition);

/// For a Definition symbol: the number of times it has been instantiated so
/// far in the visited design (as counted by slang::ast::InstanceSymbol
/// elaboration). 0 if `definition` is not a Definition symbol. This reflects
/// the state of the design as of the end of elaboration (a frozen design's
/// elaboration is already complete, so this is a pure read of a stable
/// count). Mirrors slang::ast::DefinitionSymbol::getInstanceCount.
SLANG_C_API uint64_t slang_definition_instance_count(slang_ast definition);

/* An ElabSystemTaskSymbol represents an elaboration-time system task
 * ($fatal/$error/$warning/$info/$static_assert). Its message and assert
 * condition are lazily resolved on first read but forced by the freeze
 * sweep (SOUNDNESS-MEMOS.md: `ElabSystemTaskSymbol::{message,
 * assertCondition}`), so both accessors below are pure reads on a frozen
 * design. history: since 1.3. */

/// The kind of elaboration-time system task ($fatal/$error/$warning/$info/
/// $static_assert). Mirrors slang::ast::ElabSystemTaskKind.
typedef enum slang_elab_system_task_kind {
    SLANG_ELAB_SYSTEM_TASK_FATAL = 0,
    SLANG_ELAB_SYSTEM_TASK_ERROR = 1,
    SLANG_ELAB_SYSTEM_TASK_WARNING = 2,
    SLANG_ELAB_SYSTEM_TASK_INFO = 3,
    SLANG_ELAB_SYSTEM_TASK_STATIC_ASSERT = 4,
} slang_elab_system_task_kind;

/// For an ElabSystemTask symbol: which system task it is. Defaults to
/// SLANG_ELAB_SYSTEM_TASK_FATAL if `sym` is not an ElabSystemTask symbol (use
/// slang_symbol_kind_of to distinguish that case). Mirrors
/// slang::ast::ElabSystemTaskSymbol::taskKind.
SLANG_C_API slang_elab_system_task_kind slang_symbol_elab_system_task_kind(slang_ast sym);

/// For an ElabSystemTask symbol: the condition expression of a
/// `$static_assert` (or the assertion-like condition of the others, when
/// present), as a node of domain SLANG_AST_EXPRESSION. A null node if `sym`
/// is not an ElabSystemTask symbol or the task has no condition (e.g. a bare
/// `$fatal;`). Mirrors slang::ast::ElabSystemTaskSymbol::getAssertCondition.
SLANG_C_API slang_ast slang_symbol_elab_system_task_assert_condition(slang_ast sym);

/// For an ElabSystemTask symbol: its formatted message string (the result of
/// evaluating and concatenating its arguments), written into `*out` and
/// returning true if the task carries a message; false (leaving `*out`
/// untouched) if it has none, or `sym` is not an ElabSystemTask symbol.
/// Mirrors slang::ast::ElabSystemTaskSymbol::getMessage (a
/// std::optional<std::string_view>). Owned string data lives on the design;
/// the returned slang_str is borrowed and valid for the design's lifetime.
SLANG_C_API bool slang_symbol_elab_system_task_message(slang_ast sym, slang_str* out);

/* An ExplicitImportSymbol represents a single-name `import pkg::name;` item.
 * Its resolved package/imported-symbol are lazily computed on first read
 * (ExplicitImportSymbol::{package_, import, initialized}), but every
 * ExplicitImportSymbol reachable from the design is already visited by the
 * `Compilation::getAllDiagnostics()` diagnostic pass that `slang_compilation_
 * freeze` runs unconditionally before its own sweep — see
 * `ast::Elaborator`'s ExplicitImportSymbol handler, which calls
 * `importedSymbol()` — so both accessors below are pure reads on a frozen
 * design. history: since 1.3. */

/// For an ExplicitImport symbol (`import pkg::name;`): the imported name
/// (`name`), as declared — empty if this is a wildcard-style single-name
/// import (`import pkg::name;` always has one, but `sym` may not be an
/// ExplicitImport symbol at all, in which case this is also empty). Mirrors
/// the slang::ast::ExplicitImportSymbol::importName field. Borrowed.
SLANG_C_API slang_str slang_symbol_explicit_import_name(slang_ast sym);

/// For an ExplicitImport symbol: the name of the package it imports from
/// (`pkg` in `import pkg::name;`), as written -- empty if `sym` is not an
/// ExplicitImport symbol. A direct field read
/// (slang::ast::ExplicitImportSymbol::packageName), populated at
/// construction -- a pure, allocation-free read (unlike
/// slang_symbol_explicit_import_package below, this never needs the lazy
/// package-resolution memo). Borrowed. history: since 1.3.
SLANG_C_API slang_str slang_symbol_explicit_import_package_name(slang_ast sym);

/// For an ExplicitImport symbol: the package it imports from, as a node of
/// domain SLANG_AST_SYMBOL. A null node if `sym` is not an ExplicitImport
/// symbol or the package name failed to resolve. Mirrors
/// slang::ast::ExplicitImportSymbol::package.
SLANG_C_API slang_ast slang_symbol_explicit_import_package(slang_ast sym);

/// For an ExplicitImport symbol: the symbol it imports (a member of the
/// imported package), as a node of domain SLANG_AST_SYMBOL. A null node if
/// `sym` is not an ExplicitImport symbol or the imported name failed to
/// resolve. Mirrors slang::ast::ExplicitImportSymbol::importedSymbol.
SLANG_C_API slang_ast slang_symbol_explicit_import_imported_symbol(slang_ast sym);

/* A VariableSymbol represents a variable declaration -- the base of Variable,
 * FormalArgument, Field, ClassProperty, Iterator, PatternVar, ClockVar and
 * LocalAssertionVar symbols. Both fields below are plain data populated once
 * at construction (slang::ast::VariableSymbol::lifetime / flags), so both
 * accessors are pure, allocation-free reads regardless of freeze.
 * history: since 1.3. */

/// Bits returned by slang_symbol_variable_flags. Values match
/// slang::ast::VariableFlags exactly (a direct bitmask). history: since 1.3.
typedef enum slang_variable_flags {
    SLANG_VARIABLE_FLAG_NONE = 0,
    SLANG_VARIABLE_FLAG_CONST = 1u << 0,
    SLANG_VARIABLE_FLAG_COMPILER_GENERATED = 1u << 1,
    SLANG_VARIABLE_FLAG_IMMUTABLE_COVERAGE_OPTION = 1u << 2,
    SLANG_VARIABLE_FLAG_COVERAGE_SAMPLE_FORMAL = 1u << 3,
    SLANG_VARIABLE_FLAG_CHECKER_FREE_VARIABLE = 1u << 4,
    SLANG_VARIABLE_FLAG_REF_STATIC = 1u << 5,
} slang_variable_flags;

/// The flags of a Variable-family symbol (see slang_variable_flags), as a
/// raw bitmask; 0 if `sym` is not a VariableSymbol (or one of its derived
/// kinds -- Variable, FormalArgument, Field, ClassProperty, Iterator,
/// PatternVar, ClockVar, LocalAssertionVar). Mirrors the
/// slang::ast::VariableSymbol::flags field. A pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API uint32_t slang_symbol_variable_flags(slang_ast sym);

/// The lifetime (`automatic` or `static`) of a Variable-family symbol (see
/// slang_symbol_variable_flags above for which kinds qualify).
/// SLANG_VARIABLE_LIFETIME_AUTOMATIC if `sym` is not a VariableSymbol.
/// Mirrors the slang::ast::VariableSymbol::lifetime field. A pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_variable_lifetime slang_symbol_variable_lifetime(slang_ast sym);

/* A WildcardImportSymbol represents a `import pkg::*;` item. Its resolved
 * package is lazily computed on first read (WildcardImportSymbol::package),
 * but every WildcardImportSymbol reachable from the design is already
 * visited by the `Compilation::getAllDiagnostics()` diagnostic pass that
 * `slang_compilation_freeze` runs unconditionally before its own sweep --
 * see `ast::Elaborator`'s WildcardImportSymbol handler, which calls
 * `getPackage()` -- so slang_symbol_wildcard_import_package below is a pure
 * read on a frozen design. history: since 1.3. */

/// For a WildcardImport symbol: the name of the package it imports from
/// (`pkg` in `import pkg::*;`), as written -- empty if `sym` is not a
/// WildcardImport symbol. A direct field read
/// (slang::ast::WildcardImportSymbol::packageName), populated at
/// construction -- a pure, allocation-free read (unlike
/// slang_symbol_wildcard_import_package below, this never needs the lazy
/// package-resolution memo). Borrowed. history: since 1.3.
SLANG_C_API slang_str slang_symbol_wildcard_import_package_name(slang_ast sym);

/// For a WildcardImport symbol: the package it imports from, as a node of
/// domain SLANG_AST_SYMBOL. A null node if `sym` is not a WildcardImport
/// symbol or the package name failed to resolve. Mirrors
/// slang::ast::WildcardImportSymbol::getPackage.
SLANG_C_API slang_ast slang_symbol_wildcard_import_package(slang_ast sym);

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

/// If this class type was specialized from a generic (parameterized) class,
/// the GenericClassDefSymbol it was specialized from, as a node of domain
/// SLANG_AST_SYMBOL; a null node otherwise (including for a non-class type).
/// Set once at specialization time, so a pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API slang_ast slang_type_class_generic(slang_ast type);

/// If this class type has a base class with a constructor, the expression
/// used to invoke it — an explicit `super.new(...)` call found in this
/// class's own constructor body, or a call slang synthesizes for it from an
/// `extends Base(args)` / `extends Base(default)` clause; a null node if
/// there is no base-class constructor call (including for a non-class
/// type). The underlying memo is forced by the freeze sweep, so this is a
/// pure read. history: since 1.3.
SLANG_C_API slang_ast slang_type_class_base_constructor_call(slang_ast type);

/// This class type's constructor: an explicit `new` method, or one
/// synthesized for an `extends Base(default)` clause with no explicit `new`;
/// a null node if it has neither (including for a non-class type). A pure
/// read: any synthesis this can trigger is already forced, whenever an
/// extends clause is present, by slang_type_class_base_constructor_call's
/// freeze-sweep force, and the synthesis path is unreachable without one.
/// history: since 1.3.
SLANG_C_API slang_ast slang_type_class_constructor(slang_ast type);

/// The first forward `typedef class` declaration that named this class type,
/// as a node of domain SLANG_AST_SYMBOL; a null node if it was never forward
/// declared (including for a non-class type). The link is made when the
/// class's own *parent* scope resolves the name conflict between the forward
/// typedef and the class while elaborating its members — always before this
/// class type is itself reachable — so this is a pure read. history: since 1.3.
SLANG_C_API slang_ast slang_type_class_first_forward_decl(slang_ast type);

/// The interface classes this class type implements, flattened across the
/// full inheritance hierarchy (if this class is itself an interface class,
/// the interface classes it extends instead). Count is 0 for a non-class
/// type; slang_type_class_implemented_interface returns a null node when out
/// of range. The underlying memo is populated by class-scope elaboration,
/// which the freeze sweep forces, so this is a pure read. history: since 1.3.
SLANG_C_API uint32_t slang_type_class_implemented_interface_count(slang_ast type);
SLANG_C_API slang_ast slang_type_class_implemented_interface(slang_ast type, uint32_t index);

/// True if this class type was declared `virtual` (abstract, requiring an
/// implementation in a derived class); false otherwise, including for a
/// non-class type. A pure, allocation-free read. history: since 1.3.
SLANG_C_API bool slang_type_class_is_abstract(slang_ast type);

/// True if this class type was declared `:final` (an 1800-2023 extension
/// that forbids extending it); false otherwise, including for a non-class
/// type. A pure, allocation-free read. history: since 1.3.
SLANG_C_API bool slang_type_class_is_final(slang_ast type);

/// True if this class type is an `interface class`; false otherwise,
/// including for a non-class type. A pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API bool slang_type_class_is_interface(slang_ast type);

/// The implicit `this` variable of a class type — a compiler-generated
/// Variable symbol usable by non-static class property initializers — as a
/// node of domain SLANG_AST_SYMBOL; a null node for a non-class type. Set
/// once at class construction, so a pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API slang_ast slang_type_class_this_var(slang_ast type);

/// Bits returned by slang_constraint_block_flags. Values match
/// slang::ast::ConstraintBlockFlags exactly (a direct bitmask).
/// history: since 1.3.
typedef enum slang_constraint_block_flags {
    SLANG_CONSTRAINT_BLOCK_NONE = 0,
    SLANG_CONSTRAINT_BLOCK_PURE = 1u << 1,
    SLANG_CONSTRAINT_BLOCK_STATIC = 1u << 2,
    SLANG_CONSTRAINT_BLOCK_EXTERN = 1u << 3,
    SLANG_CONSTRAINT_BLOCK_EXPLICIT_EXTERN = 1u << 4,
    SLANG_CONSTRAINT_BLOCK_INITIAL = 1u << 5,
    SLANG_CONSTRAINT_BLOCK_EXTENDS = 1u << 6,
    SLANG_CONSTRAINT_BLOCK_FINAL = 1u << 7,
} slang_constraint_block_flags;

/// The flags of a ConstraintBlock symbol (see slang_constraint_block_flags),
/// as a raw bitmask; 0 for a non-constraint-block symbol. A pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_constraint_block_flags(slang_ast sym);

/// The implicit `this` variable of a non-static ConstraintBlock symbol, as a
/// node of domain SLANG_AST_SYMBOL; a null node for a static constraint
/// block, or a non-constraint-block symbol. Set once at construction, so a
/// pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_constraint_block_this_var(slang_ast sym);

/// The bound constraint tree of a ConstraintBlock symbol, as a node of domain
/// SLANG_AST_CONSTRAINT; a null node for a non-constraint-block symbol. The
/// underlying memo is forced by the freeze sweep, so this is a pure read.
/// history: since 1.3.
SLANG_C_API slang_ast slang_symbol_constraint_block_constraints(slang_ast sym);

/// The formal arguments of a covergroup type, each a node of domain
/// SLANG_AST_SYMBOL (kind FormalArgument), in declaration order. Count is 0
/// for a non-covergroup type; slang_type_covergroup_argument returns a null
/// node when out of range. The underlying memo is populated by
/// covergroup-scope elaboration, which the freeze sweep forces, so this is a
/// pure read. history: since 1.3.
SLANG_C_API uint32_t slang_type_covergroup_argument_count(slang_ast type);
SLANG_C_API slang_ast slang_type_covergroup_argument(slang_ast type, uint32_t index);

/// For a FormalArgument symbol (a subroutine/covergroup formal argument):
/// its declared direction (`input` by default, or `output`/`inout`/`ref`).
/// A direct field read (slang::ast::FormalArgumentSymbol::direction) -- a
/// pure, allocation-free read. Returns SLANG_ARGUMENT_DIRECTION_IN for a
/// non-FormalArgument symbol. history: since 1.3.
SLANG_C_API slang_argument_direction slang_symbol_formal_argument_direction(slang_ast sym);

/// For a FormalArgument symbol: its default value expression (used when the
/// caller omits this argument), as a node of domain SLANG_AST_EXPRESSION; a
/// null node if it has none, or `sym` is not a FormalArgument symbol. The
/// underlying memo (FormalArgumentSymbol::defaultVal) is forced by the
/// freeze sweep, so this is a pure read. Mirrors
/// slang::ast::FormalArgumentSymbol::getDefaultValue. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_formal_argument_default_value(slang_ast sym);

/// Which branch of a conditional (`if`/`case`) or loop generate construct
/// produced a given GenerateBlock symbol. Mirrors
/// slang::ast::GenerateBranchKind. history: since 1.3.
typedef enum slang_generate_branch_kind {
    SLANG_GENERATE_BRANCH_IF_TRUE = 0,
    SLANG_GENERATE_BRANCH_IF_FALSE = 1,
    SLANG_GENERATE_BRANCH_CASE_ITEM = 2,
    SLANG_GENERATE_BRANCH_CASE_DEFAULT = 3,
    SLANG_GENERATE_BRANCH_LOOP_ITERATION = 4,
    SLANG_GENERATE_BRANCH_ILLEGAL_UNCONDITIONAL = 5,
} slang_generate_branch_kind;

/// For a GenerateBlock symbol (one instantiated block of an `if`/`case`/loop
/// generate construct): which branch of the originating construct produced
/// it. A direct field read (slang::ast::GenerateBlockSymbol::branchKind) --
/// a pure, allocation-free read. Returns
/// SLANG_GENERATE_BRANCH_ILLEGAL_UNCONDITIONAL for a non-GenerateBlock
/// symbol. history: since 1.3.
SLANG_C_API slang_generate_branch_kind slang_symbol_generate_block_branch_kind(slang_ast sym);

/// For a GenerateBlock symbol: its bound if/case condition expression (the
/// `cond` of `if (cond)` or the selector of a `case` generate-item), as a
/// node of domain SLANG_AST_EXPRESSION; a null node if this block was not
/// produced by a conditional branch (branch_kind is
/// SLANG_GENERATE_BRANCH_LOOP_ITERATION or
/// SLANG_GENERATE_BRANCH_ILLEGAL_UNCONDITIONAL) or `sym` is not a
/// GenerateBlock symbol. Like caseItemExpressions, this is bound against the
/// construct's enclosing scope and is explicitly visited (canonical-type-
/// forced, prefolded) by the freeze sweep's GenerateBlockSymbol branch, so
/// this is a pure read. Mirrors
/// slang::ast::GenerateBlockSymbol::getConditionExpression.
/// history: since 1.3.
SLANG_C_API slang_ast slang_symbol_generate_block_condition_expr(slang_ast sym);

/// For a GenerateBlock symbol: true if the generate construct that produced
/// it was never actually instantiated in the design (e.g. the untaken branch
/// of an `if`/`case` generate, kept around only so name-lookup rules inside
/// it can still be checked). False for any other symbol kind. A direct field
/// read (slang::ast::GenerateBlockSymbol::isUninstantiated) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API bool slang_symbol_generate_block_is_uninstantiated(slang_ast sym);

/// For a GenerateBlock symbol: the number of case-item label expressions
/// bound for it (only nonempty for a `case` generate block whose branch_kind
/// is SLANG_GENERATE_BRANCH_CASE_ITEM), and each one as a node of domain
/// SLANG_AST_EXPRESSION. 0 / a null node if `sym` is not a GenerateBlock
/// symbol or `index` is out of range. A direct field read
/// (slang::ast::GenerateBlockSymbol::caseItemExpressions), explicitly
/// visited (canonical-type-forced, prefolded) by the freeze sweep since
/// these are bound against the construct's enclosing scope rather than this
/// block's own -- see FreezeVisitor in CApiAst.cpp -- so this is a pure
/// read. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_generate_block_case_item_expr_count(slang_ast sym);
SLANG_C_API slang_ast slang_symbol_generate_block_case_item_expr(slang_ast sym, uint32_t index);

/// For a GenerateBlock symbol: the constructIndex assigned to it by its
/// originating generate construct (its position among that construct's
/// blocks, used e.g. to name unlabeled blocks). A direct field read
/// (slang::ast::GenerateBlockSymbol::constructIndex) -- a pure,
/// allocation-free read. Returns 0 for a non-GenerateBlock symbol.
/// history: since 1.3.
SLANG_C_API uint32_t slang_symbol_generate_block_construct_index(slang_ast sym);

/// For a GenerateBlock symbol: the loop-iteration index that produced it (a
/// structured constant), if it was produced by a loop-generate construct.
/// See slang_symbol_generate_block_array_index, declared alongside
/// slang_constant later in this file.

/// For a GenerateBlockArray symbol (an array of blocks produced by a
/// loop-generate construct): the number of instantiated block entries, and
/// each one as a node of domain SLANG_AST_SYMBOL (kind GenerateBlock). 0 / a
/// null node if `sym` is not a GenerateBlockArray symbol or `index` is out
/// of range. A direct field read (slang::ast::GenerateBlockArraySymbol::
/// entries) -- a pure, allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_generate_block_array_entry_count(slang_ast sym);
SLANG_C_API slang_ast slang_symbol_generate_block_array_entry(slang_ast sym, uint32_t index);

/// For a GenerateBlockArray symbol: the constructIndex assigned to it by its
/// enclosing scope (analogous to slang_symbol_generate_block_construct_index).
/// A direct field read (slang::ast::GenerateBlockArraySymbol::
/// constructIndex) -- a pure, allocation-free read. Returns 0 for a
/// non-GenerateBlockArray symbol. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_generate_block_array_construct_index(slang_ast sym);

/// For a GenerateBlockArray symbol: true if the loop-generate construct that
/// produced it completed successfully (its stop expression evaluated to a
/// well-defined boolean on every iteration, without exceeding the
/// compilation's max-generate-steps limit or looping forever). False for a
/// non-GenerateBlockArray symbol. A direct field read
/// (slang::ast::GenerateBlockArraySymbol::valid) -- a pure, allocation-free
/// read. history: since 1.3.
SLANG_C_API bool slang_symbol_generate_block_array_valid(slang_ast sym);

/// For a GenerateBlockArray symbol: the bound initial-value expression of
/// its loop variable (the `i = 0` of `for (genvar i = 0; ...; ...)`), as a
/// node of domain SLANG_AST_EXPRESSION; a null node if `sym` is not a
/// GenerateBlockArray symbol (every GenerateBlockArray has one once its
/// genvar identifier resolves). A direct field read
/// (slang::ast::GenerateBlockArraySymbol::initialExpression), explicitly
/// visited (canonical-type-forced, prefolded) by the freeze sweep since it
/// is bound against the construct's enclosing scope -- see FreezeVisitor in
/// CApiAst.cpp -- so this is a pure read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_generate_block_array_initial_expr(slang_ast sym);

/// For a GenerateBlockArray symbol: the bound stop-condition expression of
/// its loop-generate construct (the `i < N` of `for (...; i < N; ...)`), as
/// a node of domain SLANG_AST_EXPRESSION; a null node if `sym` is not a
/// GenerateBlockArray symbol. A direct field read
/// (slang::ast::GenerateBlockArraySymbol::stopExpression), explicitly
/// visited by the freeze sweep exactly like the initial expression above,
/// so this is a pure read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_generate_block_array_stop_expr(slang_ast sym);

/// For a GenerateBlockArray symbol: the bound iteration expression of its
/// loop-generate construct (the `i++` of `for (...; ...; i++)`), as a node
/// of domain SLANG_AST_EXPRESSION; a null node if `sym` is not a
/// GenerateBlockArray symbol. A direct field read
/// (slang::ast::GenerateBlockArraySymbol::iterExpression), explicitly
/// visited by the freeze sweep exactly like the initial expression above,
/// so this is a pure read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_generate_block_array_iter_expr(slang_ast sym);

/// For a GenerateBlockArray symbol: the loop variable used by its bound
/// stop and iteration expressions (a compiler-generated local shadowing the
/// loop's genvar), as a node of domain SLANG_AST_SYMBOL; a null node if
/// `sym` is not a GenerateBlockArray symbol. A direct field read
/// (slang::ast::GenerateBlockArraySymbol::loopVariable), explicitly visited
/// by the freeze sweep since it lives in a private scope never reached by
/// the generic traversal -- see FreezeVisitor in CApiAst.cpp -- so this is
/// a pure read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_generate_block_array_loop_variable(slang_ast sym);

/// For a GenerateBlock or GenerateBlockArray symbol: its external name --
/// the declared name if it has one, or else a synthesized `genblk<N>` name
/// (where N is derived from its constructIndex among unnamed siblings).
/// Empty for any other symbol kind. Recomputed on every call (mirrors
/// slang::ast::GenerateBlockSymbol::getExternalName /
/// slang::ast::GenerateBlockArraySymbol::getExternalName) -- a pure read
/// that allocates only the returned owned string, never the frozen arena.
/// Owned. history: since 1.3.
SLANG_C_API slang_str slang_symbol_generate_block_external_name(slang_ast sym, slang_error* err);

/// If this covergroup type was declared with `covergroup extends base_cg`
/// (an 1800-2023 extension), the base covergroup's type; a null node
/// otherwise (including for a non-covergroup type). Forced by the freeze
/// sweep, so this is a pure read. history: since 1.3.
SLANG_C_API slang_ast slang_type_covergroup_base_group(slang_ast type);

/// The sampling event of a covergroup type (e.g. the `@(posedge clk)` of
/// `covergroup cg @(posedge clk);`), as a node of domain
/// SLANG_AST_TIMING_CONTROL; a null node if the covergroup has no sampling
/// event of its own (including for a non-covergroup type). Forced by the
/// freeze sweep, so this is a pure read. history: since 1.3.
SLANG_C_API slang_ast slang_type_covergroup_coverage_event(slang_ast type);

/// True if a DPI open-array type (a formal argument of a `DPI-C` import with
/// an unsized dimension, e.g. `logic a[]`) is the packed form (`logic[]`)
/// rather than the unpacked form; false for the unpacked form, and false for
/// any other type. A pure, allocation-free read. history: since 1.3.
SLANG_C_API bool slang_type_dpi_open_array_is_packed(slang_ast type);

/// The system-generated ID of an enum type; 0 for any other type. Assigned
/// once at construction (slang::ast::EnumType::systemId), so a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API int32_t slang_type_enum_system_id(slang_ast type);

/// The floating-point kind of a real/shortreal/realtime type. Mirrors
/// slang::ast::FloatingType::Kind. history: since 1.3.
typedef enum slang_float_kind {
    SLANG_FLOAT_REAL = 0,
    SLANG_FLOAT_SHORT_REAL = 1,
    SLANG_FLOAT_REAL_TIME = 2,
} slang_float_kind;

/// The floating-point kind of a real/shortreal/realtime type;
/// SLANG_FLOAT_REAL for any other type. A pure, allocation-free read.
/// Mirrors slang::ast::FloatingType::floatKind. history: since 1.3.
SLANG_C_API slang_float_kind slang_type_floating_kind(slang_ast type);

/// The kind restriction a `typedef` forward declaration places on the type it
/// resolves to (e.g. `typedef enum e;` restricts to enum). Mirrors
/// slang::ast::ForwardTypeRestriction. history: since 1.3.
typedef enum slang_forward_type_restriction {
    SLANG_FORWARD_TYPE_NONE = 0,
    SLANG_FORWARD_TYPE_ENUM = 1,
    SLANG_FORWARD_TYPE_STRUCT = 2,
    SLANG_FORWARD_TYPE_UNION = 3,
    SLANG_FORWARD_TYPE_CLASS = 4,
    SLANG_FORWARD_TYPE_INTERFACE_CLASS = 5,
} slang_forward_type_restriction;

/// A member visibility modifier. Mirrors slang::ast::Visibility.
/// history: since 1.3.
typedef enum slang_visibility {
    SLANG_VISIBILITY_PUBLIC = 0,
    SLANG_VISIBILITY_PROTECTED = 1,
    SLANG_VISIBILITY_LOCAL = 2,
} slang_visibility;

/// The type-kind restriction of a ForwardingTypedef symbol (see
/// slang_forward_type_restriction); SLANG_FORWARD_TYPE_NONE for any other
/// symbol. Set once at construction, so a pure, allocation-free read.
/// Mirrors slang::ast::ForwardingTypedefSymbol::typeRestriction.
/// history: since 1.3.
SLANG_C_API slang_forward_type_restriction slang_forwarding_typedef_type_restriction(
    slang_ast sym);

/// The visibility modifier of a ForwardingTypedef symbol, if it declared one
/// (see slang_visibility). Returns false (leaving `*out` untouched) if the
/// symbol declared none, or is not a ForwardingTypedef symbol. A pure,
/// allocation-free read. Mirrors
/// slang::ast::ForwardingTypedefSymbol::visibility. history: since 1.3.
SLANG_C_API bool slang_forwarding_typedef_visibility(slang_ast sym, slang_visibility* out);

/// True if a GenericClassDef symbol was declared as an `interface class`;
/// false otherwise, including for any other symbol. A pure, allocation-free
/// read. Mirrors slang::ast::GenericClassDefSymbol::isInterface.
/// history: since 1.3.
SLANG_C_API bool slang_generic_class_is_interface(slang_ast sym);

/// The default specialization of a GenericClassDef symbol — the class type
/// obtained when every parameter uses its default value — as a node of
/// domain SLANG_AST_SYMBOL; a null node if some parameter has no default (or
/// for any other symbol). Forced by the freeze sweep, so this is a pure
/// read. Mirrors slang::ast::GenericClassDefSymbol::getDefaultSpecialization.
/// history: since 1.3.
SLANG_C_API slang_ast slang_generic_class_default_specialization(slang_ast sym);

/// The first forward `typedef class` declaration that named a GenericClassDef
/// symbol, as a node of domain SLANG_AST_SYMBOL; a null node if it was never
/// forward declared (including for any other symbol). Linked by the class's
/// own parent scope while resolving the forward-typedef/class name conflict
/// during that parent's member elaboration — always before this symbol is
/// itself reachable — so this is a pure read. Mirrors
/// slang::ast::GenericClassDefSymbol::getFirstForwardDecl. history: since 1.3.
SLANG_C_API slang_ast slang_generic_class_first_forward_decl(slang_ast sym);

/// Forces and returns a specialization of a GenericClassDef symbol with every
/// parameter set to an invalid placeholder value, letting callers inspect
/// members that don't depend on parameter values; a null node for any other
/// symbol. UNLIKE slang_generic_class_default_specialization, a fresh,
/// uncached specialization is (re)computed on EVERY call, so this allocates
/// into the arena and requires exclusive access — callers must serialize
/// this with any concurrent read of the same design (mirrors
/// slang_expression_eval's contract). Mirrors
/// slang::ast::GenericClassDefSymbol::getInvalidSpecialization.
/// history: since 1.3.
SLANG_C_API slang_ast slang_generic_class_invalid_specialization(slang_ast sym, slang_error* err);

/// True if an integral type was declared using the `reg` keyword (looking
/// through any packed-array dimensions to the underlying scalar type); false
/// for every other type, including for a non-integral type. A pure,
/// allocation-free read (the canonical-type memo it walks through is forced
/// by the freeze sweep for every reachable type). Mirrors
/// slang::ast::IntegralType::isDeclaredReg. history: since 1.3.
SLANG_C_API bool slang_type_is_declared_reg(slang_ast type);

/// The system-generated ID of a packed struct type; 0 for any other type.
/// Assigned once at construction, so a pure, allocation-free read.
/// Canonicalizes first. Mirrors slang::ast::PackedStructType::systemId.
/// history: since 1.3.
SLANG_C_API int32_t slang_type_packed_struct_system_id(slang_ast type);

/// True if a packed union type is declared `soft`; false for a non-soft
/// packed union, and for any other type. A pure, allocation-free read.
/// Canonicalizes first. Mirrors slang::ast::PackedUnionType::isSoft.
/// history: since 1.3.
SLANG_C_API bool slang_type_packed_union_is_soft(slang_ast type);

/// True if a packed union type is declared `tagged`; false for a non-tagged
/// packed union, and for any other type. A pure, allocation-free read.
/// Canonicalizes first. Mirrors slang::ast::PackedUnionType::isTagged.
/// history: since 1.3.
SLANG_C_API bool slang_type_packed_union_is_tagged(slang_ast type);

/// The system-generated ID of a packed union type; 0 for any other type.
/// Assigned once at construction, so a pure, allocation-free read.
/// Canonicalizes first. Mirrors slang::ast::PackedUnionType::systemId.
/// history: since 1.3.
SLANG_C_API int32_t slang_type_packed_union_system_id(slang_ast type);

/// The number of bits reserved for the tag of a `tagged` packed union type;
/// 0 for a non-tagged packed union, and for any other type. A pure,
/// allocation-free read. Canonicalizes first. Mirrors
/// slang::ast::PackedUnionType::tagBits. history: since 1.3.
SLANG_C_API uint32_t slang_type_packed_union_tag_bits(slang_ast type);

/// The system-generated ID of an unpacked struct type; 0 for any other type.
/// Assigned once at construction, so a pure, allocation-free read.
/// Canonicalizes first. Mirrors slang::ast::UnpackedStructType::systemId.
/// history: since 1.3.
SLANG_C_API int32_t slang_type_unpacked_struct_system_id(slang_ast type);

/// True if an unpacked union type is declared `tagged`; false for a
/// non-tagged unpacked union, and for any other type. A pure,
/// allocation-free read. Canonicalizes first. Mirrors
/// slang::ast::UnpackedUnionType::isTagged. history: since 1.3.
SLANG_C_API bool slang_type_unpacked_union_is_tagged(slang_ast type);

/// The system-generated ID of an unpacked union type; 0 for any other type.
/// Assigned once at construction, so a pure, allocation-free read.
/// Canonicalizes first. Mirrors slang::ast::UnpackedUnionType::systemId.
/// history: since 1.3.
SLANG_C_API int32_t slang_type_unpacked_union_system_id(slang_ast type);

/// The kind of a predefined integer type (`shortint`/`int`/`longint`/`byte`/
/// `integer`/`time`). Mirrors slang::ast::PredefinedIntegerType::Kind.
/// history: since 1.3.
typedef enum slang_predefined_integer_kind {
    SLANG_PREDEFINED_INTEGER_SHORTINT = 0,
    SLANG_PREDEFINED_INTEGER_INT = 1,
    SLANG_PREDEFINED_INTEGER_LONGINT = 2,
    SLANG_PREDEFINED_INTEGER_BYTE = 3,
    SLANG_PREDEFINED_INTEGER_INTEGER = 4,
    SLANG_PREDEFINED_INTEGER_TIME = 5,
} slang_predefined_integer_kind;

/// The kind of a predefined integer type (see slang_predefined_integer_kind);
/// SLANG_PREDEFINED_INTEGER_SHORTINT for any other type (including a
/// non-predefined-integer type — check slang_type_is_integral /
/// slang_type_bit_width first to disambiguate). A pure, allocation-free read.
/// Canonicalizes first. Mirrors slang::ast::PredefinedIntegerType::integerKind.
/// history: since 1.3.
SLANG_C_API slang_predefined_integer_kind slang_type_predefined_integer_kind(slang_ast type);

/// The maximum number of elements allowed in a queue type (e.g. 4 in
/// `int q[$:4]`, 0 for an unbounded `int q[$]`); 0 for any other type.
/// Canonicalizes first. A pure, allocation-free read. Mirrors
/// slang::ast::QueueType::maxBound. history: since 1.3.
SLANG_C_API uint32_t slang_type_queue_max_bound(slang_ast type);

/// The full set of net-type kinds (see slang::ast::NetType::NetKind), unlike
/// slang_net_type_kind's keyword-lookup subset: this also covers the error
/// placeholder (Unknown) and user-defined nettypes. history: since 1.3.
typedef enum slang_net_kind {
    SLANG_NET_UNKNOWN = 0,
    SLANG_NET_WIRE = 1,
    SLANG_NET_WAND = 2,
    SLANG_NET_WOR = 3,
    SLANG_NET_TRI = 4,
    SLANG_NET_TRIAND = 5,
    SLANG_NET_TRIOR = 6,
    SLANG_NET_TRI0 = 7,
    SLANG_NET_TRI1 = 8,
    SLANG_NET_TRIREG = 9,
    SLANG_NET_SUPPLY0 = 10,
    SLANG_NET_SUPPLY1 = 11,
    SLANG_NET_UWIRE = 12,
    SLANG_NET_INTERCONNECT = 13,
    SLANG_NET_USER_DEFINED = 14,
} slang_net_kind;

/// The specific net kind of a NetType symbol (see slang_net_kind);
/// SLANG_NET_UNKNOWN for any other symbol. Set once at construction, so a
/// pure, allocation-free read. Mirrors slang::ast::NetType::netKind.
/// history: since 1.3.
SLANG_C_API slang_net_kind slang_net_type_net_kind(slang_ast net_type);

/// The custom resolution function of a NetType symbol (declared with a
/// `nettype T name with func;` clause), as a node of domain SLANG_AST_SYMBOL
/// (kind Subroutine); a null node if it has none (including for any other
/// symbol). The underlying memo is forced by the freeze sweep, so this is a
/// pure read. Mirrors slang::ast::NetType::getResolutionFunction.
/// history: since 1.3.
SLANG_C_API slang_ast slang_net_type_resolution_function(slang_ast net_type);

/// True if a NetType symbol is one of the built-in kinds (i.e. not
/// user-defined); false for a user-defined nettype, and for any other
/// symbol. A pure, allocation-free read. Mirrors
/// slang::ast::NetType::isBuiltIn. history: since 1.3.
SLANG_C_API bool slang_net_type_is_built_in(slang_ast net_type);

/// True if a NetType symbol is the error placeholder nettype (netKind ==
/// Unknown); false otherwise, including for any other symbol. A pure,
/// allocation-free read. Mirrors slang::ast::NetType::isError.
/// history: since 1.3.
SLANG_C_API bool slang_net_type_is_error(slang_ast net_type);

/// The net type that results from resolving a port connection between an
/// `internal` net type (inside a module) and an `external` one (at the
/// instantiation site), per IEEE 1800 §23.3.3.2; also reports through
/// `*out_should_warn` whether simulators should warn about the resolution
/// (left untouched, and a null node returned, if either argument is not a
/// NetType symbol). A pure, allocation-free read. Mirrors
/// slang::ast::NetType::getSimulatedNetType. history: since 1.3.
SLANG_C_API slang_ast slang_net_type_get_simulated(slang_ast internal_net, slang_ast external_net,
                                                    bool* out_should_warn);

/// Type relations from IEEE 1800 §6.22.
SLANG_C_API bool slang_type_is_matching(slang_ast a, slang_ast b);
SLANG_C_API bool slang_type_is_equivalent(slang_ast a, slang_ast b);
SLANG_C_API bool slang_type_is_assignment_compatible(slang_ast a, slang_ast b);

/// The kind of a ScalarType (`bit`/`logic`/`reg`). Mirrors
/// slang::ast::ScalarType::Kind. history: since 1.3.
typedef enum slang_scalar_kind {
    SLANG_SCALAR_BIT = 0,
    SLANG_SCALAR_LOGIC = 1,
    SLANG_SCALAR_REG = 2,
} slang_scalar_kind;

/// The scalar kind of a ScalarType symbol (see slang_scalar_kind);
/// SLANG_SCALAR_BIT for any other type (check slang_type_is_integral plus the
/// canonical kind first to disambiguate a genuine `bit` from a non-scalar
/// type). Canonicalizes first. Set once at construction, so a pure,
/// allocation-free read. Mirrors slang::ast::ScalarType::scalarKind.
/// history: since 1.3.
SLANG_C_API slang_scalar_kind slang_type_scalar_kind(slang_ast type);

/// True if this type's value can be treated as string-like: the string type
/// itself, plus byte arrays and every integral type; false for any other
/// type. Canonicalizes first. A pure, allocation-free read. Mirrors
/// slang::ast::Type::canBeStringLike. history: since 1.3.
SLANG_C_API bool slang_type_can_be_string_like(slang_ast type);

/// For an associative array type with an explicit (non-wildcard) index type,
/// that index type; a null node for a wildcard-indexed (`[*]`) associative
/// array, or for any other type. Canonicalizes first. A pure,
/// allocation-free read. Mirrors slang::ast::Type::getAssociativeIndexType.
/// history: since 1.3.
SLANG_C_API slang_ast slang_type_associative_index_type(slang_ast type);

/// `$bits` of the type: the number of bits produced/consumed by a bitstream
/// (streaming) cast. 0 if the type has no statically known bitstream size
/// (e.g. a dynamic array, associative array, or queue). Canonicalizes first.
/// For a class type this reads ClassType::getBitstreamWidth, whose memo the
/// freeze sweep force-resolves, so this remains a pure read. Mirrors
/// slang::ast::Type::getBitstreamWidth. history: since 1.3.
SLANG_C_API uint64_t slang_type_bitstream_width(slang_ast type);

/// If `a` and `b` are both class types with a common base class somewhere in
/// their inheritance chains, that common base type; a null node otherwise
/// (including when either is not a class type). Canonicalizes both first.
/// Walks each chain via ClassType::getBaseClass, whose memo the freeze sweep
/// force-resolves, so this is a pure read. Mirrors
/// slang::ast::Type::getCommonBase. history: since 1.3.
SLANG_C_API slang_ast slang_type_common_base(slang_ast a, slang_ast b);

/// Bits returned by slang_type_integral_flags. Values match
/// slang::ast::IntegralFlags exactly (a direct bitmask). history: since 1.3.
typedef enum slang_integral_flags {
    SLANG_INTEGRAL_UNSIGNED = 0,
    SLANG_INTEGRAL_SIGNED = 1,
    SLANG_INTEGRAL_FOUR_STATE = 2,
    SLANG_INTEGRAL_REG = 4,
} slang_integral_flags;

/// The combination of integral-type traits for this type (see
/// slang_integral_flags); all-zero (SLANG_INTEGRAL_UNSIGNED) for a
/// non-integral type. Canonicalizes first. A pure, allocation-free read.
/// Mirrors slang::ast::Type::getIntegralFlags. history: since 1.3.
SLANG_C_API uint32_t slang_type_integral_flags(slang_ast type);

/// The "selectable" width of the type: the size used to determine whether
/// static-portion assignments to it overlap with each other. Dynamically
/// sized types report 1. Canonicalizes first. A pure, allocation-free read.
/// Mirrors slang::ast::Type::getSelectableWidth. history: since 1.3.
SLANG_C_API uint64_t slang_type_selectable_width(slang_ast type);

/// True if this type has a statically fixed size range — a packed array or
/// fixed-size unpacked array, or any integral type (whose range is its
/// bitwidth); false otherwise. Canonicalizes first. A pure, allocation-free
/// read. Mirrors slang::ast::Type::hasFixedRange. history: since 1.3.
SLANG_C_API bool slang_type_has_fixed_range(slang_ast type);

/// True if `type` is a class type that implements the interface class
/// `iface_class` (directly, or via a base class), or is itself an interface
/// class that extends it; false otherwise (including when either is not a
/// class type). Walks the base-class chain via ClassType::getBaseClass and
/// reads ClassType::getImplementedInterfaces, both of whose underlying memos
/// the freeze sweep force-resolves, so this is a pure read. Mirrors
/// slang::ast::Type::implements. history: since 1.3.
SLANG_C_API bool slang_type_implements(slang_ast type, slang_ast iface_class);

/// True if this is an aggregate type — an unpacked struct, unpacked union, or
/// any fixed/dynamic/associative array or queue; false for any "singular"
/// type (including packed aggregates and scalars). Canonicalizes first. A
/// pure, allocation-free read. Mirrors slang::ast::Type::isAggregate.
/// history: since 1.3.
SLANG_C_API bool slang_type_is_aggregate(slang_ast type);

/// True if this type node is itself a type alias (a `typedef`) — unlike
/// every other slang_type_is_* predicate, this does NOT canonicalize first:
/// it reports true only for the alias node itself, not for a reference that
/// merely resolves to one. False for any other type or non-type node. A
/// pure, allocation-free read. Mirrors slang::ast::Type::isAlias.
/// history: since 1.3.
SLANG_C_API bool slang_type_is_alias(slang_ast type);

/// True if this is an associative array type; false otherwise. Canonicalizes
/// first. A pure, allocation-free read. Mirrors
/// slang::ast::Type::isAssociativeArray. history: since 1.3.
SLANG_C_API bool slang_type_is_associative_array(slang_ast type);

/// True if `rhs` can be bit-stream cast to `type` — i.e. a `type'(rhs_expr)`
/// streaming/bitstream cast between the two would be legal. Canonicalizes
/// both first. A pure, allocation-free read. Mirrors
/// slang::ast::Type::isBitstreamCastable. history: since 1.3.
SLANG_C_API bool slang_type_is_bitstream_castable(slang_ast type, slang_ast rhs);

/// True if this type can be packed into a stream of bits — an integral type,
/// a string, an unpacked array/struct whose elements/fields all satisfy this,
/// or a non-interface, non-cyclic class whose properties all satisfy this.
/// If `destination` is true, this is checked in the context of the
/// destination side of a bitstream cast, which disallows associative arrays
/// and any class. Canonicalizes first; recurses through element/field/
/// property types, all of whose underlying `getType()` memos the freeze
/// sweep force-resolves for every DeclaredType carrier, so this is a pure,
/// allocation-free read. Mirrors slang::ast::Type::isBitstreamType. history:
/// since 1.3.
SLANG_C_API bool slang_type_is_bitstream_type(slang_ast type, bool destination);

/// True if this type is convertible to a boolean predicate for use in a
/// conditional expression — any numeric type, or a null/chandle/string/
/// event/class/covergroup/virtual-interface type. Canonicalizes first. A
/// pure, allocation-free read. Mirrors slang::ast::Type::isBooleanConvertible.
/// history: since 1.3.
SLANG_C_API bool slang_type_is_boolean_convertible(slang_ast type);

/// True if this is an unpacked array of `byte`, the shape various
/// string-related language rules check for to interpret such an argument as
/// a string. Canonicalizes first. A pure, allocation-free read. Mirrors
/// slang::ast::Type::isByteArray. history: since 1.3.
SLANG_C_API bool slang_type_is_byte_array(slang_ast type);

/// True if this is a C-handle type. Canonicalizes first. A pure,
/// allocation-free read. Mirrors slang::ast::Type::isCHandle. history:
/// since 1.3.
SLANG_C_API bool slang_type_is_chandle(slang_ast type);

/// True if `rhs` is "cast compatible" to `type` — implicitly or explicitly
/// convertible to it (assignment compatible, or an enum/string/integral
/// special case per IEEE 1800 §6.22.4); the reverse is not necessarily true.
/// Canonicalizes both first. A pure, allocation-free read. Mirrors
/// slang::ast::Type::isCastCompatible. history: since 1.3.
SLANG_C_API bool slang_type_is_cast_compatible(slang_ast type, slang_ast rhs);

/// True if this is a covergroup type. Canonicalizes first. A pure,
/// allocation-free read. Mirrors slang::ast::Type::isCovergroup. history:
/// since 1.3.
SLANG_C_API bool slang_type_is_covergroup(slang_ast type);

/// True if `type` is a class type that derives from the class type `base`
/// (directly, or via a base class), or if `type`'s canonical type is the
/// error type (permissively treated as derived from anything, to avoid
/// knock-on errors); false otherwise, including when `base` is not a class
/// type. Canonicalizes both first; walks the base-class chain via
/// `ClassType::getBaseClass`, whose memo the freeze sweep force-resolves, so
/// this is a pure read. Mirrors slang::ast::Type::isDerivedFrom. history:
/// since 1.3.
SLANG_C_API bool slang_type_is_derived_from(slang_ast type, slang_ast base);

/// True if this is a dynamic array, associative array, or queue type; false
/// otherwise (including for a fixed-size unpacked array). Canonicalizes
/// first. A pure, allocation-free read. Mirrors
/// slang::ast::Type::isDynamicallySizedArray. history: since 1.3.
SLANG_C_API bool slang_type_is_dynamically_sized_array(slang_ast type);

/// True if this is the error type, reported for a type that failed to
/// resolve. Canonicalizes first. A pure, allocation-free read. Mirrors
/// slang::ast::Type::isError. history: since 1.3.
SLANG_C_API bool slang_type_is_error(slang_ast type);

/// True if this is an event type. Canonicalizes first. A pure,
/// allocation-free read. Mirrors slang::ast::Type::isEvent. history:
/// since 1.3.
SLANG_C_API bool slang_type_is_event(slang_ast type);

/// True if this type has a fixed bitstream size, as opposed to a
/// dynamically sized type like a dynamic array, associative array, queue, or
/// string — an integral or floating type; a fixed-size unpacked array,
/// struct, or union whose elements/fields all satisfy this; or a class whose
/// bitstream width is nonzero. Canonicalizes first; recurses through
/// element/field types and (for classes) reads `ClassType::getBitstreamWidth`,
/// all pre-forced by the freeze sweep, so this is a pure, allocation-free
/// read. Mirrors slang::ast::Type::isFixedSize. history: since 1.3.
SLANG_C_API bool slang_type_is_fixed_size(slang_ast type);

/// True if this is a floating point type (`real`, `shortreal`, or
/// `realtime`). Canonicalizes first. A pure, allocation-free read. Mirrors
/// slang::ast::Type::isFloating. history: since 1.3.
SLANG_C_API bool slang_type_is_floating(slang_ast type);

/// True if this is a type that acts like a handle — a class, event,
/// chandle, virtual interface, or the null type. Canonicalizes first. A
/// pure, allocation-free read. Mirrors slang::ast::Type::isHandleType.
/// history: since 1.3.
SLANG_C_API bool slang_type_is_handle_type(slang_ast type);

/// True if this type is considered iterable — any type with a fixed range,
/// any array, or a string, excluding plain scalar (`bit`/`logic`/`reg`)
/// types. Canonicalizes first. A pure, allocation-free read. Mirrors
/// slang::ast::Type::isIterable. history: since 1.3.
SLANG_C_API bool slang_type_is_iterable(slang_ast type);

/// True if this is the null type. Canonicalizes first. A pure,
/// allocation-free read. Mirrors slang::ast::Type::isNull. history:
/// since 1.3.
SLANG_C_API bool slang_type_is_null(slang_ast type);

/// True if this is a numeric type — any integral or floating type.
/// Canonicalizes first. A pure, allocation-free read. Mirrors
/// slang::ast::Type::isNumeric. history: since 1.3.
SLANG_C_API bool slang_type_is_numeric(slang_ast type);

/// True if this is a type that is a handle to some object that contains
/// accessible members — a class, covergroup, or virtual interface type
/// (unlike slang_type_is_handle_type, this excludes chandle, event, and the
/// null type). Canonicalizes first. A pure, allocation-free read. Mirrors
/// slang::ast::Type::isObjectHandleType. history: since 1.3.
SLANG_C_API bool slang_type_is_object_handle_type(slang_ast type);

/// True if this is a packed array type. Canonicalizes first. A pure,
/// allocation-free read. Mirrors slang::ast::Type::isPackedArray. history:
/// since 1.3.
SLANG_C_API bool slang_type_is_packed_array(slang_ast type);

/// True if this is a packed union type. Canonicalizes first. A pure,
/// allocation-free read. Mirrors slang::ast::Type::isPackedUnion. history:
/// since 1.3.
SLANG_C_API bool slang_type_is_packed_union(slang_ast type);

/// True if this is a predefined integer type (`int`, `shortint`, `longint`,
/// `byte`, `integer`, or `time`). Canonicalizes first. A pure,
/// allocation-free read. Mirrors slang::ast::Type::isPredefinedInteger.
/// history: since 1.3.
SLANG_C_API bool slang_type_is_predefined_integer(slang_ast type);

/// True if this is the property type (used for assertion property
/// expressions). Canonicalizes first. A pure, allocation-free read. Mirrors
/// slang::ast::Type::isPropertyType. history: since 1.3.
SLANG_C_API bool slang_type_is_property_type(slang_ast type);

/// True if this is a queue type. Canonicalizes first. A pure,
/// allocation-free read. Mirrors slang::ast::Type::isQueue. history:
/// since 1.3.
SLANG_C_API bool slang_type_is_queue(slang_ast type);

/// True if this is a scalar integral type (`bit`, `logic`, or `reg`).
/// Canonicalizes first. A pure, allocation-free read. Mirrors
/// slang::ast::Type::isScalar. history: since 1.3.
SLANG_C_API bool slang_type_is_scalar(slang_ast type);

/// True if this is the sequence type (used for assertion sequence
/// expressions). Canonicalizes first. A pure, allocation-free read. Mirrors
/// slang::ast::Type::isSequenceType. history: since 1.3.
SLANG_C_API bool slang_type_is_sequence_type(slang_ast type);

/// True if this is a simple bit vector type — a predefined integer type, a
/// scalar type, or a packed array of a scalar element type. Canonicalizes
/// first. A pure, allocation-free read. Mirrors
/// slang::ast::Type::isSimpleBitVector. history: since 1.3.
SLANG_C_API bool slang_type_is_simple_bit_vector(slang_ast type);

/// True if this type is considered a "simple type" — a built-in integer, a
/// floating type, a string, a class, or a type alias. Unlike most
/// slang_type_is_* predicates, this checks the type node's own `kind`
/// directly rather than canonicalizing first (mirroring
/// slang::ast::Type::isSimpleType exactly). A pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API bool slang_type_is_simple_type(slang_ast type);

/// True if this is a "singular" type — the opposite of an aggregate type,
/// i.e. every type except unpacked structs, unpacked unions, and arrays.
/// Canonicalizes first. A pure, allocation-free read. Mirrors
/// slang::ast::Type::isSingular. history: since 1.3.
SLANG_C_API bool slang_type_is_singular(slang_ast type);

/// True if this is a tagged union, packed or unpacked. Canonicalizes first.
/// A pure, allocation-free read. Mirrors slang::ast::Type::isTaggedUnion.
/// history: since 1.3.
SLANG_C_API bool slang_type_is_tagged_union(slang_ast type);

/// True if this is the type reference type (the type of a `type(...)`
/// expression). Canonicalizes first. A pure, allocation-free read. Mirrors
/// slang::ast::Type::isTypeRefType. history: since 1.3.
SLANG_C_API bool slang_type_is_type_ref_type(slang_ast type);

/// True if this is the unbounded type (the type of the `$` token used in a
/// queue or range). Canonicalizes first. A pure, allocation-free read.
/// Mirrors slang::ast::Type::isUnbounded. history: since 1.3.
SLANG_C_API bool slang_type_is_unbounded(slang_ast type);

/// True if this is an unpacked structure type. Canonicalizes first. A pure,
/// allocation-free read. Mirrors slang::ast::Type::isUnpackedStruct.
/// history: since 1.3.
SLANG_C_API bool slang_type_is_unpacked_struct(slang_ast type);

/// True if this is an unpacked union type. Canonicalizes first. A pure,
/// allocation-free read. Mirrors slang::ast::Type::isUnpackedUnion. history:
/// since 1.3.
SLANG_C_API bool slang_type_is_unpacked_union(slang_ast type);

/// True if this is the untyped type — the type given to a sequence/property/
/// `let` formal port that declares no type at all (e.g. the `x` in
/// `sequence s(x); ... endsequence`). Canonicalizes first. A pure,
/// allocation-free read. Mirrors slang::ast::Type::isUntypedType. history:
/// since 1.3.
SLANG_C_API bool slang_type_is_untyped_type(slang_ast type);

/// True if this type is valid for use as a DPI import/export function
/// argument. Canonicalizes first (and recurses into array element / unpacked
/// struct field types, whose DeclaredType memos the freeze sweep
/// force-resolves), so this is a pure, allocation-free read. Mirrors
/// slang::ast::Type::isValidForDPIArg. history: since 1.3.
SLANG_C_API bool slang_type_is_valid_for_dpi_arg(slang_ast type);

/// True if this type is valid for use as a DPI import/export function return
/// value. Canonicalizes first. A pure, allocation-free read. Mirrors
/// slang::ast::Type::isValidForDPIReturn. history: since 1.3.
SLANG_C_API bool slang_type_is_valid_for_dpi_return(slang_ast type);

/// A `rand`/`randc` mode, as declared on a `rand`/`randc` class property.
/// Mirrors slang::ast::RandMode. history: since 1.3.
typedef enum slang_rand_mode {
    SLANG_RAND_MODE_NONE = 0,
    SLANG_RAND_MODE_RAND = 1,
    SLANG_RAND_MODE_RANDC = 2,
} slang_rand_mode;

/// True if this type is valid for use as a random variable declared with the
/// given mode, under the given language version (see slang_language_version
/// — floating-point `rand` members are only legal since 1800-2023).
/// Canonicalizes first and recurses into array element types (whose
/// DeclaredType memo the freeze sweep force-resolves), so this is a pure,
/// allocation-free read. Mirrors slang::ast::Type::isValidForRand. history:
/// since 1.3.
SLANG_C_API bool slang_type_is_valid_for_rand(slang_ast type, slang_rand_mode mode,
                                              slang_language_version language_version);

/// For a ClassProperty symbol (a class member variable): its declared
/// visibility (`public` by default, or `protected`/`local`). A direct field
/// read (slang::ast::ClassPropertySymbol::visibility) — a pure,
/// allocation-free read. Returns SLANG_VISIBILITY_PUBLIC for a
/// non-ClassProperty symbol. history: since 1.3.
SLANG_C_API slang_visibility slang_symbol_class_property_visibility(slang_ast sym);

/// For a ClassProperty symbol: its `rand`/`randc` mode
/// (SLANG_RAND_MODE_NONE if neither). A direct field read
/// (slang::ast::ClassPropertySymbol::randMode) — a pure, allocation-free
/// read. Returns SLANG_RAND_MODE_NONE for a non-ClassProperty symbol.
/// history: since 1.3.
SLANG_C_API slang_rand_mode slang_symbol_class_property_rand_mode(slang_ast sym);

/// For a Field symbol (a struct/union member): its `rand`/`randc` mode, if it
/// was declared inside a `rand`/`randc`-qualified struct/union
/// (SLANG_RAND_MODE_NONE otherwise). A direct field read
/// (slang::ast::FieldSymbol::randMode) -- a pure, allocation-free read.
/// Returns SLANG_RAND_MODE_NONE for a non-Field symbol. history: since 1.3.
SLANG_C_API slang_rand_mode slang_field_rand_mode(slang_ast field);

/// The symbol's `rand`/`randc` mode -- SLANG_RAND_MODE_NONE unless the
/// symbol is a ClassProperty or Field with a `rand`/`randc` qualifier.
/// slang::ast::Symbol::getRandMode reads the same field as
/// slang_symbol_class_property_rand_mode / slang_field_rand_mode,
/// generalized over symbol kind via a switch on `sym->kind` -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_rand_mode slang_symbol_rand_mode(slang_ast sym);

/// True if this type is valid for use in a sequence/property expression
/// (must be cast-compatible with an integral type). Canonicalizes first. A
/// pure, allocation-free read. Mirrors slang::ast::Type::isValidForSequence.
/// history: since 1.3.
SLANG_C_API bool slang_type_is_valid_for_sequence(slang_ast type);

/// True if this is a virtual interface type. Canonicalizes first. A pure,
/// allocation-free read. Mirrors slang::ast::Type::isVirtualInterface.
/// history: since 1.3.
SLANG_C_API bool slang_type_is_virtual_interface(slang_ast type);

/// True if this is the Void type. Canonicalizes first. A pure,
/// allocation-free read. Mirrors slang::ast::Type::isVoid. history: since
/// 1.3.
SLANG_C_API bool slang_type_is_void(slang_ast type);

/// The visibility modifier of a `TypeAliasType` (a `typedef`), e.g.
/// `local`/`protected` on a class-scoped typedef (see slang_visibility).
/// Returns SLANG_VISIBILITY_PUBLIC (slang's own field default) for any node
/// that is not a type alias. Set once at construction, so a pure,
/// allocation-free read. Mirrors slang::ast::TypeAliasType::visibility.
/// history: since 1.3.
SLANG_C_API slang_visibility slang_type_alias_visibility(slang_ast type);

/* ------------------------------------------------------------------------- */
/* AST: type printing                                                        */
/* ------------------------------------------------------------------------- */

/// A utility object that renders a Type to a string in SystemVerilog syntax,
/// accumulating into its own internal buffer across calls to
/// slang_type_printer_append. Independent of any Design's frozen arena (it
/// allocates only its own buffer on the regular process heap), so it may be
/// created, used, and destroyed freely while a Design is shared read-only
/// across threads. Owned; free with slang_type_printer_destroy. Mirrors
/// slang::ast::TypePrinter. history: since 1.3.
typedef struct slang_type_printer_t* slang_type_printer;

/// Selects a style for anonymous (unnamed, e.g. an inline unpacked struct)
/// types in printed output. Mirrors
/// slang::ast::TypePrintingOptions::AnonymousTypeStyle. history: since 1.3.
typedef enum slang_anonymous_type_style {
    /// Print the compiler-internal system ID name (the default).
    SLANG_ANON_TYPE_SYSTEM_NAME = 0,
    /// Print a synthesized, more human-friendly name.
    SLANG_ANON_TYPE_FRIENDLY_NAME = 1,
} slang_anonymous_type_style;

/// The options that control a slang_type_printer's output — a plain-old-data
/// mirror of a subset of slang::ast::TypePrintingOptions's public fields.
/// Read with slang_type_printer_options, write with
/// slang_type_printer_set_options. Default-constructed (via
/// slang_type_printer_create) with every bool false,
/// friendly_member_char_limit 60, has_quote_char false, and
/// SLANG_ANON_TYPE_SYSTEM_NAME, matching slang's own defaults.
/// history: since 1.3.
typedef struct slang_type_printing_options {
    /// Elide the names of scopes containing the types. Mirrors
    /// TypePrintingOptions::elideScopeNames.
    bool elide_scope_names;
    /// Print classes and covergroups as links instead of their expanded type
    /// details. Mirrors TypePrintingOptions::classesAsLinks.
    bool classes_as_links;
    /// Print enums as links instead of their expanded type details. Mirrors
    /// TypePrintingOptions::enumsAsLinks.
    bool enums_as_links;
    /// Selects a style for anonymous types. Mirrors
    /// TypePrintingOptions::anonymousTypeStyle.
    slang_anonymous_type_style anonymous_type_style;
    /// True if quote_char holds a value to wrap around printed type names;
    /// false if type names are printed unquoted. Mirrors whether
    /// TypePrintingOptions::quoteChar is engaged. history: since 1.3.
    bool has_quote_char;
    /// The quote character to wrap around printed type names, valid only
    /// when has_quote_char is true. Mirrors
    /// TypePrintingOptions::quoteChar's value. history: since 1.3.
    char quote_char;
    /// Print an 'aka' note when unwrapping typedefs. Mirrors
    /// TypePrintingOptions::printAKA. history: since 1.3.
    bool print_aka;
    /// Skip over scoped type names completely. Mirrors
    /// TypePrintingOptions::skipScopedTypeNames. history: since 1.3.
    bool skip_scoped_type_names;
    /// Skip expanding typedefs. Mirrors TypePrintingOptions::skipTypeDefs.
    /// history: since 1.3.
    bool skip_type_defs;
    /// Include the enum's base type. Mirrors
    /// TypePrintingOptions::fullEnumType. history: since 1.3.
    bool full_enum_type;
    /// Print typedefs as links instead of their expanded type details.
    /// Mirrors TypePrintingOptions::typedefsAsLinks. history: since 1.3.
    bool typedefs_as_links;
    /// Print the constant range of integral types for packed non-array
    /// objects. Mirrors TypePrintingOptions::printIntegralRange.
    /// history: since 1.3.
    bool print_integral_range;
    /// A limit on the size of a friendly-named struct/union member list,
    /// beyond which the output will be abbreviated. Mirrors
    /// TypePrintingOptions::friendlyMemberCharLimit. history: since 1.3.
    size_t friendly_member_char_limit;
} slang_type_printing_options;

/// Creates a new type printer with slang's default options (see
/// slang_type_printing_options). history: since 1.3.
SLANG_C_API slang_type_printer slang_type_printer_create(slang_error* err);

/// Frees a type printer created by slang_type_printer_create.
/// history: since 1.3.
SLANG_C_API void slang_type_printer_destroy(slang_type_printer printer);

/// Appends the given type to the printer's internal string buffer, rendered
/// in SystemVerilog syntax. A no-op if `type` is not a type node. A pure
/// read of `type` (mutates only the printer's own buffer, never the design's
/// frozen arena). Mirrors slang::ast::TypePrinter::append. history: since
/// 1.3.
SLANG_C_API void slang_type_printer_append(slang_type_printer printer, slang_ast type);

/// Clears the printer's internal string buffer. Mirrors
/// slang::ast::TypePrinter::clear. history: since 1.3.
SLANG_C_API void slang_type_printer_clear(slang_type_printer printer);

/// Returns the printer's accumulated string buffer as a copy. Owned. Mirrors
/// slang::ast::TypePrinter::toString. history: since 1.3.
SLANG_C_API slang_str slang_type_printer_to_string(slang_type_printer printer, slang_error* err);

/// The printer's current options (see slang_type_printing_options). Mirrors
/// reading slang::ast::TypePrinter::options. history: since 1.3.
SLANG_C_API slang_type_printing_options slang_type_printer_options(slang_type_printer printer);

/// Replaces the printer's options wholesale (see
/// slang_type_printing_options). Mirrors writing
/// slang::ast::TypePrinter::options. history: since 1.3.
SLANG_C_API void slang_type_printer_set_options(slang_type_printer printer,
                                                slang_type_printing_options options);

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

/// The symbol referenced by this expression, if it is (or reduces to) a
/// direct reference to one — mirrors slang::ast::Expression::getSymbolReference.
/// Unlike slang_expression_symbol (which always behaves as if `allow_packed`
/// were true), this also lets a select or member access into a *packed* type
/// be excluded: when `allow_packed` is false, such a select/access returns
/// the null node instead of the underlying symbol, so only unpacked
/// aggregates (arrays, structs, unions, object handles) count as an
/// addressable whole symbol. Null node if this expression has no direct
/// symbol reference. A pure read. history: since 1.3.
SLANG_C_API slang_ast slang_expr_symbol_reference(slang_ast expr, bool allow_packed);

/// True if any subexpression of this expression is a hierarchical reference
/// (a name resolved via `scope.member`-style hierarchical lookup, as opposed
/// to ordinary lexical scoping) — mirrors
/// slang::ast::Expression::hasHierarchicalReference. A pure read.
/// history: since 1.3.
SLANG_C_API bool slang_expr_has_hierarchical_reference(slang_ast expr);

/// True if `expr` is structurally equivalent to `other` (same shape and
/// constants once implicit type conversions are looked through) — mirrors
/// slang::ast::Expression::isEquivalentTo. A pure read. history: since 1.3.
SLANG_C_API bool slang_expr_is_equivalent_to(slang_ast expr, slang_ast other);

/// True if `expr` is implicitly treated as a string in a string context — a
/// string literal, or an integral expression built up out of string
/// literals via unary/binary/conditional operators, concatenation,
/// replication, or a value range — mirrors
/// slang::ast::Expression::isImplicitString. A pure read. history: since 1.3.
SLANG_C_API bool slang_expr_is_implicit_string(slang_ast expr);

/// True if `expr` could be implicitly assigned to a value of `type` (IEEE
/// 1800 §6.22.3 assignment compatibility, plus the string/enum/relaxed-
/// conversion special cases) — mirrors
/// slang::ast::Expression::isImplicitlyAssignableTo. `type` must be a type
/// node (see e.g. slang_expression_type). A pure read: no lookup,
/// evaluation, or arena allocation, despite taking a Compilation reference
/// internally. history: since 1.3.
SLANG_C_API bool slang_expr_is_implicitly_assignable_to(slang_ast expr, slang_ast type);

/// True if `expr` is represented by an unsized integer value — an integer
/// literal written without an explicit size (e.g. the `4` in `x + 4`), or an
/// unbased unsized literal (`'1`, `'z`) — mirrors
/// slang::ast::Expression::isUnsizedInteger. A pure read. history: since 1.3.
SLANG_C_API bool slang_expr_is_unsized_integer(slang_ast expr);

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

/// For an Attribute symbol (`(* name = expr *)`): its bound, constant-folded
/// value (same representation as slang_expression_constant_value). NULL on
/// error (including a non-Attribute symbol, or a value that failed to
/// fold). The underlying memo is forced by the freeze sweep, so this is a
/// pure read on a frozen design; the caller owns the returned handle
/// (slang_constant_destroy). Mirrors slang::ast::AttributeSymbol::getValue.
/// history: since 1.3.
SLANG_C_API slang_constant slang_symbol_attribute_value(slang_ast sym, slang_error* err);

/// The value of an IntegerLiteral expression as a structured constant (same
/// representation as slang_expression_constant_value) — mirrors
/// slang::ast::IntegerLiteral::getValue. Unlike the cached constant of a
/// general expression, this is always available: an integer literal's value
/// is set once at construction, not lazily folded, so this never evaluates
/// and is a pure read. Fails (via `err`) if `expr` is not an IntegerLiteral.
/// history: since 1.3.
SLANG_C_API slang_constant slang_expr_integer_literal_value(slang_ast expr, slang_error* err);

/// For a GenerateBlock symbol: the loop-iteration index that produced it, as
/// a structured constant (same representation as
/// slang_expression_constant_value), if it was produced by a loop-generate
/// construct (branch_kind SLANG_GENERATE_BRANCH_LOOP_ITERATION). NULL
/// (`err` unset) if it was not, or `sym` is not a GenerateBlock symbol.
/// Mirrors slang::ast::GenerateBlockSymbol::getArrayIndex. The underlying
/// value is the already-folded value of the implicit localparam of the same
/// name as the loop's genvar (ParameterSymbol::getValue(), forced pre-seal
/// by the freeze sweep), so this is a pure read; the caller owns the
/// returned handle (slang_constant_destroy). history: since 1.3.
SLANG_C_API slang_constant slang_symbol_generate_block_array_index(slang_ast sym, slang_error* err);

/// Evaluates an expression now and returns its structured constant value, or
/// NULL if not constant. MAY allocate (the seal is lifted); caller must hold
/// exclusive access, as with slang_expression_eval.
SLANG_C_API slang_constant slang_expression_eval_constant(slang_ast expr, slang_error* err);

/// Re-runs slang::ast::SystemSubroutine::eval for the system call `call`
/// (a bound Call expression whose subroutine is a system subroutine — see
/// slang_expr_call_system_subroutine and slang_system_subroutine_check_arguments),
/// evaluating it in a fresh constant-evaluation context against its own
/// already-bound arguments and source range. Returns the resulting constant
/// value, or NULL if evaluation failed (e.g. the subroutine is not allowed in
/// a constant context) — this is the same pattern as
/// slang_expression_eval_constant, applied to the specific subroutine's own
/// eval override rather than the general expression-eval dispatch. Requires
/// exclusive access, since evaluation can allocate. Fails (via `err`) if
/// `call` is not a bound system call. history: since 1.3.
SLANG_C_API slang_constant slang_system_subroutine_eval(slang_ast call, slang_error* err);

/// The default value for an uninitialized variable of this type (e.g. all
/// zero bits for a four-state integral type), as a structured constant (same
/// representation as slang_expression_constant_value); NULL if `type` is not
/// a type node. Canonicalizes first. Builds a fresh ConstantValue on the
/// regular process heap on every call (never the compilation's frozen
/// arena), so — unlike slang_expression_eval_constant — this is a pure read
/// requiring no exclusive access. Mirrors slang::ast::Type::getDefaultValue.
/// history: since 1.3.
SLANG_C_API slang_constant slang_type_default_value(slang_ast type, slang_error* err);

/// Coerces `value` into a constant appropriate for `type` — e.g. resizing and
/// re-signing an integral value, or converting a numeric value to a
/// (short)real or string. Returns NULL for a type this cannot coerce into
/// (any type other than integral, floating, or string), or if `type`/`value`
/// is invalid. As with slang_type_default_value, this only allocates on the
/// regular process heap, so it is a pure read requiring no exclusive access.
/// Mirrors slang::ast::Type::coerceValue. history: since 1.3.
SLANG_C_API slang_constant slang_type_coerce_value(slang_ast type, slang_constant value,
                                                    slang_error* err);

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

/// True if the constant is empty: a zero-length string, or a zero-element
/// unpacked array/map/queue; false for every other kind (including a NULL
/// constant). Mirrors slang::ConstantValue::empty. history: since 1.3.
SLANG_C_API bool slang_constant_empty(slang_constant c);

/// True if the constant is a container kind (unpacked array/map/queue, i.e.
/// slang_constant_kind_of returns SLANG_CONSTANT_UNPACKED/_MAP/_QUEUE);
/// false otherwise (including a NULL constant). Mirrors
/// slang::ConstantValue::isContainer. history: since 1.3.
SLANG_C_API bool slang_constant_is_container(slang_constant c);

/// True if the constant is "truthy" under SystemVerilog's condition-to-bool
/// conversion: a nonzero/non-x/z integer, a nonzero real, an unbounded ('$')
/// placeholder, or a nonempty string; false otherwise (including a NULL
/// constant). Mirrors slang::ConstantValue::isTrue. history: since 1.3.
SLANG_C_API bool slang_constant_is_true(slang_constant c);

/// True if the constant is "falsy": a known-zero integer, a zero real, a
/// null handle, or an empty string; false otherwise (including a NULL
/// constant — a NULL constant is neither true nor false). Mirrors
/// slang::ConstantValue::isFalse. history: since 1.3.
SLANG_C_API bool slang_constant_is_false(slang_constant c);

/// The size of this value in bits when flattened to a bitstream: the bit
/// width for an integer, `8 * length` for a string, or the recursive sum of
/// element widths for an unpacked array/map/queue/union; 0 for every other
/// kind (including a NULL constant). Mirrors
/// slang::ConstantValue::getBitstreamWidth. A pure read (no allocation).
/// history: since 1.3.
SLANG_C_API uint64_t slang_constant_bitstream_width(slang_constant c);

/// A slice `[upper:lower]` of this constant, with an implicit "bad" fill for
/// any out-of-range unpacked-array element (mirrors
/// slang::ConstantValue::getSlice with a default-constructed, i.e. bad,
/// `defaultValue`). Valid for an integer (a bit-range slice), an unpacked
/// array (an element range; out-of-range indices come back as
/// SLANG_CONSTANT_BAD elements), a queue (an element range; out-of-range
/// indices are simply dropped), or a string (a single-character select,
/// which requires `upper == lower`). Returns a new owned constant, or NULL
/// if `c` is none of those kinds. history: since 1.3.
SLANG_C_API slang_constant slang_constant_get_slice(slang_constant c, int32_t upper, int32_t lower,
                                                     slang_error* err);

/// This constant converted to a `real`, or NULL if it is not real, shortreal,
/// or an integer. Mirrors slang::ConstantValue::convertToReal. New owned
/// constant of kind SLANG_CONSTANT_REAL on success. history: since 1.3.
SLANG_C_API slang_constant slang_constant_convert_to_real(slang_constant c, slang_error* err);

/// This constant converted to a `shortreal`, or NULL if it is not real,
/// shortreal, or an integer. Mirrors slang::ConstantValue::convertToShortReal.
/// New owned constant of kind SLANG_CONSTANT_SHORTREAL on success. history:
/// since 1.3.
SLANG_C_API slang_constant slang_constant_convert_to_short_real(slang_constant c,
                                                                 slang_error* err);

/// This constant converted to a string per IEEE 1800-2017 §6.16 (each 8-bit
/// chunk of an integer, MSB-first, becomes a character, with all-zero chunks
/// dropped), or NULL if it is neither a string nor an integer. Mirrors
/// slang::ConstantValue::convertToStr. New owned constant of kind
/// SLANG_CONSTANT_STRING on success. history: since 1.3.
SLANG_C_API slang_constant slang_constant_convert_to_str(slang_constant c, slang_error* err);

/// This constant converted to a fixed-size unpacked array of `size` 8-bit
/// (`is_signed`) byte constants — an integer or string is first converted to
/// a string (as slang_constant_convert_to_str) and then packed one character
/// per byte, zero-padded or truncated to `size`; an unpacked array is passed
/// through unchanged; `size` 0 uses the natural (string) length. NULL if `c`
/// is none of those kinds. Mirrors slang::ConstantValue::convertToByteArray.
/// New owned constant of kind SLANG_CONSTANT_UNPACKED on success. history:
/// since 1.3.
SLANG_C_API slang_constant slang_constant_convert_to_byte_array(slang_constant c, uint32_t size,
                                                                 bool is_signed, slang_error* err);

/// This constant converted to a queue of 8-bit (`is_signed`) byte constants
/// — an integer or string is first converted to a string (as
/// slang_constant_convert_to_str) and then packed one character per element;
/// a queue is passed through unchanged. NULL if `c` is none of those kinds.
/// Mirrors slang::ConstantValue::convertToByteQueue. New owned constant of
/// kind SLANG_CONSTANT_QUEUE on success. history: since 1.3.
SLANG_C_API slang_constant slang_constant_convert_to_byte_queue(slang_constant c, bool is_signed,
                                                                 slang_error* err);

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

/* Bit-counting queries. All ignore x/z unless noted otherwise, are pure
 * reads (no allocation), and return 0 for a NULL handle. history: since
 * 1.3. */

/// The number of leading (MSB-side) 1 bits. Does not treat unknown bits
/// specially — mirrors slang::SVInt::countLeadingOnes.
SLANG_C_API uint32_t slang_svint_count_leading_ones(slang_svint v);

/// The number of leading (MSB-side) 0 bits. Does not treat unknown bits
/// specially — mirrors slang::SVInt::countLeadingZeros.
SLANG_C_API uint32_t slang_svint_count_leading_zeros(slang_svint v);

/// The number of leading (MSB-side) unknown (x or z) bits — mirrors
/// slang::SVInt::countLeadingUnknowns.
SLANG_C_API uint32_t slang_svint_count_leading_unknowns(slang_svint v);

/// The number of leading (MSB-side) z bits — mirrors
/// slang::SVInt::countLeadingZs.
SLANG_C_API uint32_t slang_svint_count_leading_zs(slang_svint v);

/// The total number of 1 bits — mirrors slang::SVInt::countOnes.
SLANG_C_API uint32_t slang_svint_count_ones(slang_svint v);

/// The total number of 0 bits — mirrors slang::SVInt::countZeros.
SLANG_C_API uint32_t slang_svint_count_zeros(slang_svint v);

/// The total number of x bits — mirrors slang::SVInt::countXs.
SLANG_C_API uint32_t slang_svint_count_xs(slang_svint v);

/// The total number of z bits — mirrors slang::SVInt::countZs.
SLANG_C_API uint32_t slang_svint_count_zs(slang_svint v);

/// The number of "active bits": the bit width minus the number of leading
/// zeros, i.e. the minimum width needed to hold the value ignoring sign and
/// unknown bits — mirrors slang::SVInt::getActiveBits.
SLANG_C_API uint32_t slang_svint_active_bits(slang_svint v);

/// The minimum number of bits needed to represent this value: `getActiveBits()`
/// (plus one for a positive signed value whose top active bit would otherwise
/// be read as the sign bit, and collapsing to 1 for an all-zero value) —
/// mirrors slang::SVInt::getMinRepresentedBits. A pure read; 0 for a NULL
/// handle. history: since 1.3.
SLANG_C_API uint32_t slang_svint_min_represented_bits(slang_svint v);

/// True if the low-order bit is 0 — mirrors slang::SVInt::isEven. A pure
/// read; false for a NULL handle. history: since 1.3.
SLANG_C_API bool slang_svint_is_even(slang_svint v);

/// True if the low-order bit is 1 — mirrors slang::SVInt::isOdd. A pure
/// read; false for a NULL handle. history: since 1.3.
SLANG_C_API bool slang_svint_is_odd(slang_svint v);

/// True if the most-significant bit is set — mirrors slang::SVInt::isNegative
/// (this does not consult the signedness flag; it is a raw top-bit test). A
/// pure read; false for a NULL handle. history: since 1.3.
SLANG_C_API bool slang_svint_is_negative(slang_svint v);

/// True if every bit from `msb` (inclusive) up to the top bit equals the sign
/// bit (bit `bit_width - 1`) — mirrors slang::SVInt::isSignExtendedFrom. A
/// pure read; false for a NULL handle. history: since 1.3.
SLANG_C_API bool slang_svint_is_sign_extended_from(slang_svint v, uint32_t msb);

/// The bitwise AND-reduction of every bit, as a logic_t (0, 1, 2=x, or 3=z;
/// same 2-bit encoding as slang_svint_get_bit) — mirrors
/// slang::SVInt::reductionAnd. A pure read; 0 for a NULL handle. history:
/// since 1.3.
SLANG_C_API uint8_t slang_svint_reduction_and(slang_svint v);

/// The bitwise OR-reduction of every bit, as a logic_t (2-bit encoding) —
/// mirrors slang::SVInt::reductionOr. A pure read; 0 for a NULL handle.
/// history: since 1.3.
SLANG_C_API uint8_t slang_svint_reduction_or(slang_svint v);

/// The bitwise XOR-reduction of every bit (parity), as a logic_t (2-bit
/// encoding) — mirrors slang::SVInt::reductionXor. A pure read; 0 for a NULL
/// handle. history: since 1.3.
SLANG_C_API uint8_t slang_svint_reduction_xor(slang_svint v);

/// Four-state logical implication `lhs -> rhs` (`!lhs || rhs` under
/// reduction-OR truthiness), as a logic_t (2-bit encoding) — mirrors the
/// static slang::SVInt::logicalImpl. A pure read; 0 if either handle is
/// NULL. history: since 1.3.
SLANG_C_API uint8_t slang_svint_logical_impl(slang_svint lhs, slang_svint rhs);

/// Four-state logical equivalence (`logicalImpl(lhs,rhs) && logicalImpl(rhs,lhs)`),
/// as a logic_t (2-bit encoding) — mirrors the static
/// slang::SVInt::logicalEquiv. A pure read; 0 if either handle is NULL.
/// history: since 1.3.
SLANG_C_API uint8_t slang_svint_logical_equiv(slang_svint lhs, slang_svint rhs);

/// A single four-state logic bit — mirrors slang::logic_t. Unlike the
/// compact 2-bit encoding used by slang_svint_reduction_and and friends, this
/// carries slang::logic_t's own raw byte: 0 or 1 for a known bit, the
/// sentinel 0x80 (X_VALUE) for unknown 'x', or 0x40 (Z_VALUE) for
/// high-impedance 'z' — the same encoding as a slang_svint_digit. Position
/// struct: trivially copyable. history: since 1.3.
typedef struct slang_logic_value {
    /// The raw value byte; see the type-level doc comment for its encoding.
    uint8_t value;
} slang_logic_value;

/// The raw value byte of a slang_logic_value — mirrors the
/// slang::logic_t::value field. A pure function of its argument. history:
/// since 1.3.
SLANG_C_API uint8_t slang_logic_value_value(slang_logic_value v);

/// The four-state unknown 'x' constant — mirrors the static slang::logic_t::x.
/// A pure function with no arguments. history: since 1.3.
SLANG_C_API slang_logic_value slang_logic_value_x(void);

/// The four-state high-impedance 'z' constant — mirrors the static
/// slang::logic_t::z. A pure function with no arguments. history: since 1.3.
SLANG_C_API slang_logic_value slang_logic_value_z(void);

/// True if `v` is x or z — mirrors slang::logic_t::isUnknown. A pure function
/// of its argument. history: since 1.3.
SLANG_C_API bool slang_logic_value_is_unknown(slang_logic_value v);

/// Four-state bitwise AND (0 & anything = 0; 1 & 1 = 1; otherwise x) —
/// mirrors slang::logic_t::operator&. A pure function of its arguments.
/// history: since 1.3.
SLANG_C_API slang_logic_value slang_logic_value_and(slang_logic_value lhs, slang_logic_value rhs);

/// Four-state bitwise OR (1 | anything = 1; 0 | 0 = 0; otherwise x) —
/// mirrors slang::logic_t::operator|. A pure function of its arguments.
/// history: since 1.3.
SLANG_C_API slang_logic_value slang_logic_value_or(slang_logic_value lhs, slang_logic_value rhs);

/// Four-state bitwise XOR (x if either side is unknown) — mirrors
/// slang::logic_t::operator^. A pure function of its arguments. history:
/// since 1.3.
SLANG_C_API slang_logic_value slang_logic_value_xor(slang_logic_value lhs, slang_logic_value rhs);

/// Four-state bitwise NOT (slang::logic_t defines `operator~` as logical
/// negation, the same as `operator!`: x stays x, 0 and 1 invert) — mirrors
/// slang::logic_t::operator~. A pure function of its argument. history:
/// since 1.3.
SLANG_C_API slang_logic_value slang_logic_value_not(slang_logic_value v);

/// `v` raised to the power `rhs`, per IEEE 1800 `**` semantics, as a new
/// owned constant wrapping an SVInt of the same bit width as `v` (may be
/// heap-backed if that width is >64 bits) — mirrors slang::SVInt::pow.
/// Allocates; NULL (with `err` set) if `v` or `rhs` is NULL or the operation
/// throws. Free the result with slang_constant_destroy. history: since 1.3.
SLANG_C_API slang_constant slang_svint_pow(slang_svint v, slang_svint rhs, slang_error* err);

/// `v` concatenated with itself `times` times, as a new owned constant
/// wrapping an SVInt of width `v`'s bit width times `times` — mirrors
/// slang::SVInt::replicate (`times` is read via SVInt::as<uint32_t>, so it
/// must be a known, non-negative, 32-bit-representable value). Allocates;
/// NULL (with `err` set) if `v` or `times` is NULL, `times` does not fit
/// requirement above, or the operation throws. Free the result with
/// slang_constant_destroy. history: since 1.3.
SLANG_C_API slang_constant slang_svint_replicate(slang_svint v, slang_svint times,
                                                  slang_error* err);

/// `v` resized to `bits`: truncated if smaller, sign/zero-extended (per `v`'s
/// signedness) if larger, unchanged if equal — as a new owned constant
/// wrapping the resulting SVInt. Mirrors slang::SVInt::resize. Allocates;
/// NULL (with `err` set) if `v` is NULL, `bits` is 0, or the operation
/// throws. Free the result with slang_constant_destroy. history: since 1.3.
SLANG_C_API slang_constant slang_svint_resize(slang_svint v, uint32_t bits, slang_error* err);

/// `v` with its bits reversed (bit 0 and the top bit swap, and so on), as a
/// new owned constant wrapping an SVInt of the same width as `v` — mirrors
/// slang::SVInt::reverse. Allocates; NULL (with `err` set) if `v` is NULL or
/// the operation throws. Free the result with slang_constant_destroy.
/// history: since 1.3.
SLANG_C_API slang_constant slang_svint_reverse(slang_svint v, slang_error* err);

/// `v` sign-extended to `bits`, as a new owned constant wrapping the
/// resulting SVInt. Mirrors slang::SVInt::sext. Allocates; NULL (with `err`
/// set) if `v` is NULL, `bits` is not strictly greater than `v`'s bit width,
/// or the operation throws. Free the result with slang_constant_destroy.
/// history: since 1.3.
SLANG_C_API slang_constant slang_svint_sext(slang_svint v, uint32_t bits, slang_error* err);

/// `v` zero-extended to `bits`, as a new owned constant wrapping the
/// resulting SVInt. Mirrors slang::SVInt::zext. Allocates; NULL (with `err`
/// set) if `v` is NULL, `bits` is not strictly greater than `v`'s bit width,
/// or the operation throws. Free the result with slang_constant_destroy.
/// history: since 1.3.
SLANG_C_API slang_constant slang_svint_zext(slang_svint v, uint32_t bits, slang_error* err);

/// `v` extended to `bits`: sign-extended (mirrors slang::SVInt::sext) if
/// `is_signed`, zero-extended (mirrors slang::SVInt::zext) otherwise — as a
/// new owned constant wrapping the resulting SVInt. Mirrors
/// slang::SVInt::extend. Allocates; NULL (with `err` set) if `v` is NULL,
/// `bits` is not strictly greater than `v`'s bit width, or the operation
/// throws. Free the result with slang_constant_destroy. history: since 1.3.
SLANG_C_API slang_constant slang_svint_extend(slang_svint v, uint32_t bits, bool is_signed,
                                               slang_error* err);

/// `v` truncated to its low `bits` bits, as a new owned constant wrapping
/// the resulting SVInt — mirrors slang::SVInt::trunc. Allocates; NULL (with
/// `err` set) if `v` is NULL, `bits` is 0, `bits` exceeds `v`'s bit width, or
/// the operation throws. Free the result with slang_constant_destroy.
/// history: since 1.3.
SLANG_C_API slang_constant slang_svint_trunc(slang_svint v, uint32_t bits, slang_error* err);

/// A subrange `[msb:lsb]` of `v`'s bits, as a new owned constant of width
/// `msb - lsb + 1` — mirrors slang::SVInt::slice directly (unlike
/// slang_constant_get_slice, which dispatches on the constant's kind and
/// only forwards to slang::SVInt::slice for an integer, this always treats
/// `v` as the integer it already is). Any out-of-range index is filled with
/// x (slang's own out-of-bounds handling), so this only fails on
/// `msb < lsb`. Allocates; NULL (with `err` set) if `v` is NULL, `msb < lsb`,
/// or the operation throws. Free the result with slang_constant_destroy.
/// history: since 1.3.
SLANG_C_API slang_constant slang_svint_slice(slang_svint v, int32_t msb, int32_t lsb,
                                              slang_error* err);

/// Bitwise XNOR of `lhs` and `rhs` (the narrower operand is extended to
/// match the wider one's width and signedness, exactly like the `&`/`|`/`^`
/// operators below), as a new owned constant — mirrors slang::SVInt::xnor.
/// Allocates; NULL (with `err` set) if either handle is NULL or the
/// operation throws. Free the result with slang_constant_destroy. history:
/// since 1.3.
SLANG_C_API slang_constant slang_svint_xnor(slang_svint lhs, slang_svint rhs, slang_error* err);

/// Bitwise AND of `lhs` and `rhs` (the narrower operand is extended to
/// match), as a new owned constant — mirrors slang::SVInt::operator&.
/// Allocates; NULL (with `err` set) if either handle is NULL or the
/// operation throws. Free the result with slang_constant_destroy. history:
/// since 1.3.
SLANG_C_API slang_constant slang_svint_and(slang_svint lhs, slang_svint rhs, slang_error* err);

/// Bitwise OR of `lhs` and `rhs` (the narrower operand is extended to
/// match), as a new owned constant — mirrors slang::SVInt::operator|.
/// Allocates; NULL (with `err` set) if either handle is NULL or the
/// operation throws. Free the result with slang_constant_destroy. history:
/// since 1.3.
SLANG_C_API slang_constant slang_svint_or(slang_svint lhs, slang_svint rhs, slang_error* err);

/// Bitwise XOR of `lhs` and `rhs` (the narrower operand is extended to
/// match), as a new owned constant — mirrors slang::SVInt::operator^.
/// Allocates; NULL (with `err` set) if either handle is NULL or the
/// operation throws. Free the result with slang_constant_destroy. history:
/// since 1.3.
SLANG_C_API slang_constant slang_svint_xor(slang_svint lhs, slang_svint rhs, slang_error* err);

/// Bitwise NOT (one's complement) of `v`, as a new owned constant of the
/// same width — mirrors slang::SVInt::operator~ (unary). Allocates; NULL
/// (with `err` set) if `v` is NULL or the operation throws. Free the result
/// with slang_constant_destroy. history: since 1.3.
SLANG_C_API slang_constant slang_svint_not(slang_svint v, slang_error* err);

/// In-place bitwise AND-assignment `v &= rhs` — mirrors
/// slang::SVInt::operator&=. Mutates `v`'s bits (and, if `rhs` carries
/// unknown bits `v` didn't already have, `v`'s own unknown-tracking storage)
/// without changing `v`'s bit width — unlike the C++ operator, this never
/// auto-extends the narrower operand: `v` and `rhs` must already share the
/// same bit width, or the call fails (`v` left unmodified). Never touches
/// the compilation's frozen arena, only `v`'s own privately-owned storage —
/// safe on any constant the caller holds exclusively (e.g. one produced by
/// another slang_constant_*/slang_svint_* factory). Returns `v` again on
/// success (for chaining), or NULL (with `err` set) if either handle is NULL
/// or the widths differ. history: since 1.3.
SLANG_C_API slang_svint slang_svint_iand(slang_svint v, slang_svint rhs, slang_error* err);

/// In-place bitwise OR-assignment `v |= rhs` — mirrors
/// slang::SVInt::operator|=. Same in-place, no-width-change contract as
/// slang_svint_iand. history: since 1.3.
SLANG_C_API slang_svint slang_svint_ior(slang_svint v, slang_svint rhs, slang_error* err);

/// In-place bitwise XOR-assignment `v ^= rhs` — mirrors
/// slang::SVInt::operator^=. Same in-place, no-width-change contract as
/// slang_svint_iand. history: since 1.3.
SLANG_C_API slang_svint slang_svint_ixor(slang_svint v, slang_svint rhs, slang_error* err);

/// Replaces the bit range `[msb:lsb]` of `v` with `value`, in place — mirrors
/// slang::SVInt::set. `v`'s bit width is unchanged (only `msb - lsb + 1`
/// bits of it are overwritten). Fails (via `err`, leaving `v` unmodified) if
/// `v` or `value` is NULL, `msb < lsb`, or `value`'s bit width is not
/// exactly `msb - lsb + 1`. history: since 1.3.
SLANG_C_API void slang_svint_set(slang_svint v, int32_t msb, int32_t lsb, slang_svint value,
                                  slang_error* err);

/// Sets every bit of `v` to 1, in place (bit width and signedness unchanged)
/// — mirrors slang::SVInt::setAllOnes. A no-op if `v` is NULL. history:
/// since 1.3.
SLANG_C_API void slang_svint_set_all_ones(slang_svint v);

/// Sets every bit of `v` to 0, in place — mirrors slang::SVInt::setAllZeros.
/// A no-op if `v` is NULL. history: since 1.3.
SLANG_C_API void slang_svint_set_all_zeros(slang_svint v);

/// Sets every bit of `v` to x, in place — mirrors slang::SVInt::setAllX. A
/// no-op if `v` is NULL. history: since 1.3.
SLANG_C_API void slang_svint_set_all_x(slang_svint v);

/// Sets every bit of `v` to z, in place — mirrors slang::SVInt::setAllZ. A
/// no-op if `v` is NULL. history: since 1.3.
SLANG_C_API void slang_svint_set_all_z(slang_svint v);

/// Reinterprets `v` as signed or unsigned in place, without changing its
/// bits or width — mirrors slang::SVInt::setSigned. A no-op if `v` is NULL.
/// history: since 1.3.
SLANG_C_API void slang_svint_set_signed(slang_svint v, bool is_signed);

/// Removes all unknown (x/z) bits from `v` in place, converting them to 0 —
/// mirrors slang::SVInt::flattenUnknowns. `v`'s bit width is unchanged. A
/// no-op if `v` is NULL. history: since 1.3.
SLANG_C_API void slang_svint_flatten_unknowns(slang_svint v);

/// Resizes `v` in place to the minimum number of bits that can represent its
/// current value without changing it (see slang::SVInt::getMinRepresentedBits)
/// — mirrors slang::SVInt::shrinkToFit. May reallocate `v`'s internal
/// storage. A no-op if `v` is NULL. history: since 1.3.
SLANG_C_API void slang_svint_shrink_to_fit(slang_svint v);

/// If bit `msb` of `v` is set, duplicates it into every bit in
/// `[bit_width-1:msb]`, in place — mirrors slang::SVInt::signExtendFrom.
/// `v`'s bit width is unchanged. Fails (via `err`) if `v` is NULL or `msb`
/// is not strictly less than `v`'s bit width minus one. history: since 1.3.
SLANG_C_API void slang_svint_sign_extend_from(slang_svint v, uint32_t msb, slang_error* err);

/// Creates a new owned constant wrapping an all-X SVInt of the given bit
/// width — mirrors the static slang::SVInt::createFillX. Allocates; NULL
/// (with `err` set) if `bits` is 0. history: since 1.3.
SLANG_C_API slang_constant slang_svint_create_fill_x(uint32_t bits, bool is_signed,
                                                      slang_error* err);

/// Creates a new owned constant wrapping an all-Z SVInt of the given bit
/// width — mirrors the static slang::SVInt::createFillZ. Allocates; NULL
/// (with `err` set) if `bits` is 0. history: since 1.3.
SLANG_C_API slang_constant slang_svint_create_fill_z(uint32_t bits, bool is_signed,
                                                      slang_error* err);

/// One digit for slang_svint_from_digits: a value in `[0, radix)` for the
/// digit's base, or the sentinel 0x80 (matching slang::logic_t's own X
/// encoding) for an unknown 'x' digit, or 0x40 (matching slang::logic_t's Z
/// encoding) for a high-impedance 'z' digit. Only base 2/8/16 numbers may mix
/// per-digit x/z with normal digits; a base-10 number with any_unknown set
/// must have exactly one digit, which must be all-x or all-z (see
/// slang::SVInt::fromDigits).
typedef uint8_t slang_svint_digit;

/// Constructs a new owned constant wrapping an SVInt built from an array of
/// digits in the given base (e.g. the digits of a sized literal like
/// `12'hFF`) — mirrors the static slang::SVInt::fromDigits. `base` is a raw
/// slang::LiteralBase ordinal: 0=binary, 1=octal, 2=decimal, 3=hex. If the
/// value doesn't fit in `bits`, it is truncated from the left (matching the
/// SystemVerilog literal-truncation rule). Allocates; NULL (with `err` set)
/// if `v` is NULL, `bits` is 0, `digit_count` is 0, `base` is out of range,
/// or a digit is too large for `base`. history: since 1.3.
SLANG_C_API slang_constant slang_svint_from_digits(uint32_t bits, uint8_t base, bool is_signed,
                                                    bool any_unknown,
                                                    const slang_svint_digit* digits,
                                                    uint32_t digit_count, slang_error* err);

/// Constructs a new owned constant wrapping an SVInt converted from a double
/// (rounding to nearest, ties away from zero, when `round` is true; truncated
/// toward zero otherwise) — mirrors the static slang::SVInt::fromDouble.
/// Allocates; NULL (with `err` set) if `bits` is 0. history: since 1.3.
SLANG_C_API slang_constant slang_svint_from_double(uint32_t bits, double value, bool is_signed,
                                                    bool round, slang_error* err);

/// Constructs a new owned constant wrapping an SVInt converted from a float
/// — mirrors the static slang::SVInt::fromFloat. Same rounding and error
/// contract as slang_svint_from_double. history: since 1.3.
SLANG_C_API slang_constant slang_svint_from_float(uint32_t bits, float value, bool is_signed,
                                                   bool round, slang_error* err);

/// Concatenates `count` operands (msb-first, i.e. `operands[0]` becomes the
/// most-significant bits of the result) into a new owned constant wrapping
/// their bitwise concatenation — mirrors the static slang::SVInt::concat.
/// `count` of 0 yields a 1-bit zero. Allocates; NULL (with `err` set) if any
/// of the first `count` entries of `operands` is NULL. history: since 1.3.
SLANG_C_API slang_constant slang_svint_concat(const slang_svint* operands, uint32_t count,
                                              slang_error* err);

/// Evaluates `condition ? lhs : rhs` over SVInts (four-state: the result is
/// bitwise-unknown wherever `condition` is unknown and `lhs`/`rhs` disagree
/// there), as a new owned constant — mirrors the static
/// slang::SVInt::conditional. `lhs` and `rhs` are extended to their common
/// width first (matching the C++ static function). Allocates; NULL (with
/// `err` set) if `condition`, `lhs`, or `rhs` is NULL. history: since 1.3.
SLANG_C_API slang_constant slang_svint_conditional(slang_svint condition, slang_svint lhs,
                                                    slang_svint rhs, slang_error* err);

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

/// Bulk-fills up to `cap` immediate semantic children of `node` into `out`,
/// returning the total child count (which may exceed `cap`). Collects the
/// children once, so enumerating an N-child node is O(N) rather than the
/// O(N^2) of calling slang_ast_sem_child in a loop. history: since 1.2.
SLANG_C_API uint32_t slang_ast_sem_children(slang_ast node, slang_ast* out, uint32_t cap);

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

/// The timing control of a Timed statement (an `@(...)`/`#`-delayed
/// statement), or the optional delaying timing control of an EventTrigger
/// statement (`->`/`->>`). Null ast (domain TIMING_CONTROL) if there is none
/// (an EventTrigger with no delay) or `node` is neither kind.
SLANG_C_API slang_ast slang_stmt_timing(slang_ast node);

/// The edge of a signal-event timing control (`node` a TIMING_CONTROL of kind
/// SignalEvent) — SLANG_EDGE_POSEDGE/NEGEDGE/BOTHEDGES, or SLANG_EDGE_NONE for a
/// level-sensitive event and for any other node/kind. history: since 1.3.
SLANG_C_API slang_edge_kind slang_timing_control_edge(slang_ast node);

/// The constant expression of a ConstantPattern (`node` a PATTERN of kind
/// Constant) — an EXPRESSION node, or a null ast for any other pattern kind.
/// history: since 1.3.
SLANG_C_API slang_ast slang_pattern_expr(slang_ast node);

/// The bound variable symbol of a VariablePattern (`node` a PATTERN of kind
/// Variable) — a SYMBOL node, or a null ast for any other pattern kind.
/// history: since 1.3.
SLANG_C_API slang_ast slang_pattern_variable(slang_ast node);

/// The operator of a Unary/Binary assertion expression (`node` an
/// ASSERTION_EXPR), as its raw slang UnaryAssertionOperator/BinaryAssertionOperator
/// enum value; -1 for any other kind. history: since 1.3.
SLANG_C_API int32_t slang_assertion_expr_op(slang_ast node);

/// The constrained expression of an ExpressionConstraint (`node` a CONSTRAINT of
/// kind Expression) — an EXPRESSION node, else a null ast. history: since 1.3.
SLANG_C_API slang_ast slang_constraint_expr(slang_ast node);

/// The controlling predicate of an Implication/Conditional constraint (`node` a
/// CONSTRAINT) — an EXPRESSION node, else a null ast. history: since 1.3.
SLANG_C_API slang_ast slang_constraint_predicate(slang_ast node);

/* Block / conditional / case / concurrent-assertion / event-trigger statement
 * breadth. All are pure reads on a frozen compilation: every field they read
 * is set directly at Statement construction, never lazily computed. history:
 * since 1.3. */

/// The kind of a Block statement (`begin/end` vs `fork`/`join`/`join_any`/
/// `join_none`), as its StatementBlockKind enum ordinal. 0 (Sequential) for a
/// non-block node.
SLANG_C_API uint32_t slang_stmt_block_kind(slang_ast node);

/// The StatementBlockSymbol associated with a Block statement, or a null ast
/// (domain SYMBOL) if it has none or `node` is not a block.
SLANG_C_API slang_ast slang_stmt_block_symbol(slang_ast node);

/// The unique/priority check (UniquePriorityCheck enum ordinal: None/Unique/
/// Unique0/Priority) applied to a Conditional statement's condition list. 0
/// (None) for a non-conditional node.
SLANG_C_API uint32_t slang_stmt_conditional_check(slang_ast node);

/// The number of conditions of a Conditional statement (more than one only for
/// a pattern-matching `if` with `&&&`-chained conditions). 0 for a
/// non-conditional node.
SLANG_C_API uint32_t slang_stmt_conditional_condition_count(slang_ast node);

/// The controlling expression of the i'th condition of a Conditional
/// statement. Null ast (domain EXPRESSION) if `index` is out of range or
/// `node` is not a conditional.
SLANG_C_API slang_ast slang_stmt_conditional_condition_expr(slang_ast node, uint32_t index);

/// The optional pattern of the i'th condition of a Conditional statement
/// (e.g. `if (v matches p)`). Null ast (domain PATTERN) if that condition has
/// no pattern, `index` is out of range, or `node` is not a conditional.
SLANG_C_API slang_ast slang_stmt_conditional_condition_pattern(slang_ast node, uint32_t index);

/// The kind of case condition evaluated by a Case statement
/// (CaseStatementCondition enum ordinal: Normal/WildcardXOrZ/WildcardJustZ/
/// Inside). 0 (Normal) for a non-case node.
SLANG_C_API uint32_t slang_stmt_case_condition(slang_ast node);

/// The unique/priority check applied to a Case statement's condition, as its
/// UniquePriorityCheck enum ordinal. 0 (None) for a non-case node.
SLANG_C_API uint32_t slang_stmt_case_check(slang_ast node);

/// The default-case body of a Case statement, or a null ast (domain
/// STATEMENT) if there is none or `node` is not a case.
SLANG_C_API slang_ast slang_stmt_case_default(slang_ast node);

/// The number of item groups of a Case statement. 0 for a non-case node.
SLANG_C_API uint32_t slang_stmt_case_item_count(slang_ast node);

/// The body statement of the i'th item group of a Case statement. Null ast
/// (domain STATEMENT) if `index` is out of range or `node` is not a case.
SLANG_C_API slang_ast slang_stmt_case_item_stmt(slang_ast node, uint32_t index);

/// The number of matching expressions of the i'th item group of a Case
/// statement. 0 if `index` is out of range or `node` is not a case.
SLANG_C_API uint32_t slang_stmt_case_item_expr_count(slang_ast node, uint32_t index);

/// The j'th matching expression of the i'th item group of a Case statement.
/// Null ast (domain EXPRESSION) if either index is out of range or `node` is
/// not a case.
SLANG_C_API slang_ast slang_stmt_case_item_expr(slang_ast node, uint32_t index, uint32_t j);

/// The kind of a ConcurrentAssertion statement (`assert`/`assume`/
/// `cover property`/`cover sequence`/`restrict`/`expect`), as its
/// AssertionKind enum ordinal. 0 (Assert) for a non-assertion node.
SLANG_C_API uint32_t slang_stmt_assertion_kind(slang_ast node);

/// The pass-action statement of a ConcurrentAssertion statement, or a null ast
/// (domain STATEMENT) if there is none or `node` is not an assertion.
SLANG_C_API slang_ast slang_stmt_assertion_if_true(slang_ast node);

/// The fail-action statement of a ConcurrentAssertion statement, or a null ast
/// (domain STATEMENT) if there is none or `node` is not an assertion.
SLANG_C_API slang_ast slang_stmt_assertion_if_false(slang_ast node);

/// True if an EventTrigger statement (`->`/`->>`) is non-blocking (`->>`);
/// false for a blocking trigger or a non-event-trigger node.
SLANG_C_API bool slang_stmt_event_trigger_is_nonblocking(slang_ast node);

/// A statically-known iteration range of array bit/element indices (e.g. a
/// `foreach` loop dimension's bounds) — a plain value, not an AST node.
/// Mirrors slang::ConstantRange; `left`/`right` are not necessarily ascending
/// (a descending range has `left > right`). history: since 1.3.
typedef struct slang_constant_range {
    int32_t left;
    int32_t right;
} slang_constant_range;

/* slang::ConstantRange operations. All take/return the plain value struct
 * above by value — there is no AST node or handle involved, so these are
 * pure functions of their arguments with no failure mode of their own (a
 * malformed `select` passed to subrange, which slang's own API documents as
 * a precondition violation, is still caught by SLANG_C_ACCESS's default).
 * history: since 1.3. */

/// The left (declared-first) bound (see slang_constant_range).
SLANG_C_API int32_t slang_constant_range_left(slang_constant_range r);

/// The right (declared-second) bound (see slang_constant_range).
SLANG_C_API int32_t slang_constant_range_right(slang_constant_range r);

/// The width of the range (upper - lower + 1), regardless of the order in
/// which the bounds are specified. Mirrors slang::ConstantRange::width.
SLANG_C_API uint32_t slang_constant_range_width(slang_constant_range r);

/// The lower of left/right. Mirrors slang::ConstantRange::lower.
SLANG_C_API int32_t slang_constant_range_lower(slang_constant_range r);

/// The upper of left/right. Mirrors slang::ConstantRange::upper.
SLANG_C_API int32_t slang_constant_range_upper(slang_constant_range r);

/// True if the range is "descending" (msb >= lsb, e.g. `[7:0]`); false for
/// an "ascending" range (e.g. `[0:7]`). Mirrors
/// slang::ConstantRange::isDescending.
SLANG_C_API bool slang_constant_range_is_descending(slang_constant_range r);

/// The range with its bit ordering reversed (left/right swapped). Mirrors
/// slang::ConstantRange::reverse.
SLANG_C_API slang_constant_range slang_constant_range_reverse(slang_constant_range r);

/// Selects `select` as a subrange of `r`, correctly handling both forms of
/// range ordering. `select` must not be wider than `r`. Mirrors
/// slang::ConstantRange::subrange.
SLANG_C_API slang_constant_range slang_constant_range_subrange(slang_constant_range r,
                                                                slang_constant_range select);

/// True if `index` falls within `r`. Mirrors
/// slang::ConstantRange::containsPoint.
SLANG_C_API bool slang_constant_range_contains_point(slang_constant_range r, int32_t index);

/// True if `r` and `other` overlap (including one being wholly contained in
/// the other). Mirrors slang::ConstantRange::overlaps.
SLANG_C_API bool slang_constant_range_overlaps(slang_constant_range r, slang_constant_range other);

/// Translates `index` to be relative to `r`. For example, if `r` is `[7:2]`
/// and `index` is 3, the result is 1; if `r` is `[2:7]` and `index` is 3, the
/// result is 4. Mirrors slang::ConstantRange::translateIndex.
SLANG_C_API int32_t slang_constant_range_translate_index(slang_constant_range r, int32_t index);

/// Builds a range from a left value indexed up or down by a width, as the
/// SystemVerilog `+:`/`-:` range-select operators do: `l` is the base index,
/// `width` the element count, `descending` the target range's declared bit
/// order, and `indexed_up` selects `+:` (true) vs `-:` (false). Writes the
/// result to `*out` and returns true, or returns false (leaving `*out`
/// untouched) on 32-bit overflow of the computed bound. Mirrors
/// slang::ConstantRange::getIndexedRange.
SLANG_C_API bool slang_constant_range_get_indexed_range(int32_t l, int32_t width, bool descending,
                                                         bool indexed_up,
                                                         slang_constant_range* out);

/// The range of a fixed-size unpacked array type's single dimension (e.g.
/// `[3:0]` in `logic [7:0] mem [3:0]`'s outer unpacked dimension); zeroed for
/// any other type. A pure, allocation-free read. Mirrors
/// slang::ast::FixedSizeUnpackedArrayType::range. history: since 1.3.
SLANG_C_API slang_constant_range slang_type_fixed_unpacked_array_range(slang_ast type);

/// For an InstanceArray symbol: its declared index range (e.g. `[3:0]` of
/// `m[3:0]`), as a slang_constant_range. An all-zero range if `sym` is not
/// an InstanceArray symbol. A direct field read
/// (slang::ast::InstanceArraySymbol::range) -- a pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API slang_constant_range slang_symbol_instance_array_range(slang_ast sym);

/// If this is an integral type (scalar, predefined integer, packed array,
/// packed struct/union, or enum), the address range of its bits (e.g.
/// `[7:0]`) — a packed array reports its own declared range, every other
/// integral kind reports `[bitWidth-1:0]`; a zeroed range for any
/// non-integral type. A pure, allocation-free read. Mirrors
/// slang::ast::IntegralType::getBitVectorRange. history: since 1.3.
SLANG_C_API slang_constant_range slang_type_bit_vector_range(slang_ast type);

/// The declared range of a packed array type's single dimension (e.g. `[7:0]`
/// in `logic [7:0] a`); a zeroed range for any other type. Canonicalizes
/// first. A pure, allocation-free read. Mirrors
/// slang::ast::PackedArrayType::range. history: since 1.3.
SLANG_C_API slang_constant_range slang_type_packed_array_range(slang_ast type);

/// The fixed range of the type (see slang_type_has_fixed_range) — the union
/// of what slang_type_bit_vector_range and slang_type_fixed_unpacked_array_range
/// each report for their own type kind, dispatched here on the type's own
/// canonical kind; a zeroed range for a type with no fixed range. A pure,
/// allocation-free read. Mirrors slang::ast::Type::getFixedRange.
/// history: since 1.3.
SLANG_C_API slang_constant_range slang_type_fixed_range(slang_ast type);

/// The kind of an evaluated array dimension. Mirrors
/// slang::ast::DimensionKind. history: since 1.3.
typedef enum slang_dimension_kind {
    SLANG_DIM_UNKNOWN = 0,
    SLANG_DIM_RANGE = 1,
    SLANG_DIM_ABBREVIATED_RANGE = 2,
    SLANG_DIM_DYNAMIC = 3,
    SLANG_DIM_ASSOCIATIVE = 4,
    SLANG_DIM_QUEUE = 5,
    SLANG_DIM_DPI_OPEN_ARRAY = 6,
} slang_dimension_kind;

/// One resolved array dimension of a symbol's declared type (see
/// slang_declared_type_resolved_dimensions) — a plain value, not an AST
/// node. `bounds` is meaningful for SLANG_DIM_RANGE /
/// SLANG_DIM_ABBREVIATED_RANGE only (zeroed otherwise). Mirrors
/// slang::ast::EvaluatedDimension (kind + range only; the associative-type,
/// queue-max-size and left/right-expression fields are not exposed here).
/// history: since 1.3.
typedef struct slang_evaluated_dimension {
    slang_dimension_kind kind;
    slang_constant_range bounds;
} slang_evaluated_dimension;

/// The resolved packed-then-unpacked dimensions of a symbol's declared type,
/// in declaration order. Fills up to `cap` entries of `out` and returns the
/// true total (as with slang_ast_sem_children: a small guess covers common
/// cases in one call, and a wider result needs one exact-capacity retry). 0
/// if the symbol has no declared type.
///
/// UNLIKE every other slang_declared_type_* accessor, this is NOT cached by
/// slang and NOT forced by the freeze sweep: it re-binds each dimension's
/// syntax on EVERY call, allocating fresh Expression nodes into the arena,
/// so it requires exclusive access and must be serialized with any
/// concurrent read of the same design (mirrors slang_expression_eval's
/// contract). Mirrors slang::ast::DeclaredType::getResolvedDimensions.
/// history: since 1.3.
SLANG_C_API uint32_t slang_declared_type_resolved_dimensions(slang_ast sym,
                                                              slang_evaluated_dimension* out,
                                                              uint32_t cap, slang_error* err);

/* ForLoop / Foreach / ImmediateAssertion / PatternCase statement breadth. All
 * are pure reads on a frozen compilation: every field they read is set
 * directly at Statement construction, never lazily computed.
 * history: since 1.3. */

/// The number of variable initializer expressions of a ForLoop statement
/// (mutually exclusive with slang_stmt_for_loop_var_count — a for loop has
/// either initializer expressions or declared loop variables, never both). 0
/// for a non-for-loop node.
SLANG_C_API uint32_t slang_stmt_for_loop_initializer_count(slang_ast node);

/// The i'th variable initializer expression of a ForLoop statement. Null ast
/// (domain EXPRESSION) if `index` is out of range or `node` is not a for
/// loop.
SLANG_C_API slang_ast slang_stmt_for_loop_initializer(slang_ast node, uint32_t index);

/// The number of loop variables declared by a ForLoop statement (e.g.
/// `for (int i = 0; ...)`; mutually exclusive with
/// slang_stmt_for_loop_initializer_count). 0 for a non-for-loop node.
SLANG_C_API uint32_t slang_stmt_for_loop_var_count(slang_ast node);

/// The i'th loop variable symbol declared by a ForLoop statement. Null ast
/// (domain SYMBOL) if `index` is out of range or `node` is not a for loop.
SLANG_C_API slang_ast slang_stmt_for_loop_var(slang_ast node, uint32_t index);

/// The number of per-iteration step expressions of a ForLoop statement. 0 for
/// a non-for-loop node.
SLANG_C_API uint32_t slang_stmt_for_loop_step_count(slang_ast node);

/// The i'th per-iteration step expression of a ForLoop statement. Null ast
/// (domain EXPRESSION) if `index` is out of range or `node` is not a for
/// loop.
SLANG_C_API slang_ast slang_stmt_for_loop_step(slang_ast node, uint32_t index);

/// The number of iterated dimensions of a Foreach loop statement. 0 for a
/// non-foreach node.
SLANG_C_API uint32_t slang_stmt_foreach_loop_dim_count(slang_ast node);

/// The loop-variable symbol (an IteratorSymbol) of the i'th dimension of a
/// Foreach loop statement, or a null ast (domain SYMBOL) if that dimension is
/// skipped (e.g. `foreach (a[,j])`), `index` is out of range, or `node` is
/// not a foreach loop.
SLANG_C_API slang_ast slang_stmt_foreach_loop_dim_var(slang_ast node, uint32_t index);

/// The statically-known range of the i'th dimension of a Foreach loop
/// statement, written to `*out`. Returns false (and leaves `*out` untouched)
/// if that dimension is dynamically sized, `index` is out of range, or
/// `node` is not a foreach loop.
SLANG_C_API bool slang_stmt_foreach_loop_dim_range(slang_ast node, uint32_t index,
                                                   slang_constant_range* out);

/// The kind of an ImmediateAssertion statement (`assert`/`assume`/`cover`),
/// as its AssertionKind enum ordinal. 0 (Assert) for a non-immediate-
/// assertion node.
SLANG_C_API uint32_t slang_stmt_immediate_assertion_kind(slang_ast node);

/// The pass-action statement of an ImmediateAssertion statement, or a null
/// ast (domain STATEMENT) if there is none or `node` is not an immediate
/// assertion.
SLANG_C_API slang_ast slang_stmt_immediate_assertion_if_true(slang_ast node);

/// The fail-action statement of an ImmediateAssertion statement, or a null
/// ast (domain STATEMENT) if there is none or `node` is not an immediate
/// assertion.
SLANG_C_API slang_ast slang_stmt_immediate_assertion_if_false(slang_ast node);

/// True if an ImmediateAssertion statement is a "deferred" immediate
/// assertion (`assert #0(...)`/`assert final(...)`, as opposed to a plain
/// immediate `assert(...)`); false otherwise or for a non-immediate-assertion
/// node.
SLANG_C_API bool slang_stmt_immediate_assertion_is_deferred(slang_ast node);

/// True if a deferred ImmediateAssertion statement is declared `final`
/// (`assert final(...)` rather than `assert #0(...)`); false for a
/// non-final/non-deferred assertion or a non-immediate-assertion node.
SLANG_C_API bool slang_stmt_immediate_assertion_is_final(slang_ast node);

/// The unique/priority check applied to a PatternCase statement's condition,
/// as its UniquePriorityCheck enum ordinal. 0 (None) for a non-pattern-case
/// node.
SLANG_C_API uint32_t slang_stmt_pattern_case_check(slang_ast node);

/// The number of item groups of a PatternCase statement. 0 for a
/// non-pattern-case node.
SLANG_C_API uint32_t slang_stmt_pattern_case_item_count(slang_ast node);

/// The matching pattern of the i'th item group of a PatternCase statement.
/// Null ast (domain PATTERN) if `index` is out of range or `node` is not a
/// pattern case.
SLANG_C_API slang_ast slang_stmt_pattern_case_item_pattern(slang_ast node, uint32_t index);

/// The optional filter expression of the i'th item group of a PatternCase
/// statement. Null ast (domain EXPRESSION) if there is none, `index` is out
/// of range, or `node` is not a pattern case.
SLANG_C_API slang_ast slang_stmt_pattern_case_item_filter(slang_ast node, uint32_t index);

/// The body statement of the i'th item group of a PatternCase statement.
/// Null ast (domain STATEMENT) if `index` is out of range or `node` is not a
/// pattern case.
SLANG_C_API slang_ast slang_stmt_pattern_case_item_stmt(slang_ast node, uint32_t index);

/// The kind of case condition evaluated by a PatternCase statement (Normal/
/// WildcardXOrZ/WildcardJustZ/Inside), as its CaseStatementCondition enum
/// ordinal. 0 (Normal) for a non-pattern-case node. history: since 1.3.
SLANG_C_API uint32_t slang_stmt_pattern_case_condition(slang_ast node);

/// The default-case body of a PatternCase statement, or a null ast (domain
/// STATEMENT) if there is none or `node` is not a pattern case.
/// history: since 1.3.
SLANG_C_API slang_ast slang_stmt_pattern_case_default(slang_ast node);

/* WaitOrder / ProceduralAssign / ProceduralDeassign / RandCase /
 * RandSequence / ProceduralChecker statement breadth, plus the two members
 * on the Statement base itself (bad/eval). All the accessors below are pure
 * reads on a frozen compilation except slang_stmt_eval, which evaluates (see
 * its own doc). history: since 1.3. */

/// The number of ordered event expressions of a `wait_order` (WaitOrder)
/// statement. 0 for a non-wait-order node.
SLANG_C_API uint32_t slang_stmt_wait_order_event_count(slang_ast node);

/// The i'th event expression of a WaitOrder statement. Null ast (domain
/// EXPRESSION) if `index` is out of range or `node` is not a wait-order
/// statement.
SLANG_C_API slang_ast slang_stmt_wait_order_event(slang_ast node, uint32_t index);

/// The statement to run if every event of a WaitOrder statement triggered in
/// order, or a null ast (domain STATEMENT) if there is none or `node` is not
/// a wait-order statement.
SLANG_C_API slang_ast slang_stmt_wait_order_if_true(slang_ast node);

/// The statement to run if any event of a WaitOrder statement did not
/// trigger in order, or a null ast (domain STATEMENT) if there is none or
/// `node` is not a wait-order statement.
SLANG_C_API slang_ast slang_stmt_wait_order_if_false(slang_ast node);

/// True if a ProceduralAssign statement is a `force` statement, false if it
/// is a plain procedural `assign` statement (or `node` is not a
/// procedural-assign statement).
SLANG_C_API bool slang_stmt_procedural_assign_is_force(slang_ast node);

/// True if a ProceduralDeassign statement is a `release` statement, false if
/// it is a plain `deassign` statement (or `node` is not a
/// procedural-deassign statement).
SLANG_C_API bool slang_stmt_procedural_deassign_is_release(slang_ast node);

/// The number of items of a `randcase` (RandCase) statement. 0 for a
/// non-randcase node.
SLANG_C_API uint32_t slang_stmt_randcase_item_count(slang_ast node);

/// The matching-weight expression of the i'th item of a RandCase statement.
/// Null ast (domain EXPRESSION) if `index` is out of range or `node` is not
/// a randcase statement.
SLANG_C_API slang_ast slang_stmt_randcase_item_expr(slang_ast node, uint32_t index);

/// The body statement of the i'th item of a RandCase statement. Null ast
/// (domain STATEMENT) if `index` is out of range or `node` is not a randcase
/// statement.
SLANG_C_API slang_ast slang_stmt_randcase_item_stmt(slang_ast node, uint32_t index);

/// The first production symbol (a RandSeqProductionSymbol) of a
/// `randsequence` (RandSequence) statement, or a null ast (domain SYMBOL) if
/// the sequence is empty or `node` is not a randsequence statement.
SLANG_C_API slang_ast slang_stmt_randsequence_first_production(slang_ast node);

/// The number of checker instances of a procedural checker-instantiation
/// (ProceduralChecker) statement. 0 for a non-procedural-checker node.
SLANG_C_API uint32_t slang_stmt_procedural_checker_instance_count(slang_ast node);

/// The i'th checker instance symbol of a ProceduralChecker statement. Null
/// ast (domain SYMBOL) if `index` is out of range or `node` is not a
/// procedural-checker statement.
SLANG_C_API slang_ast slang_stmt_procedural_checker_instance(slang_ast node, uint32_t index);

/// True if a statement node is invalid (had errors) — mirrors
/// slang::ast::Statement::bad(). A pure, allocation-free read.
SLANG_C_API bool slang_stmt_is_bad(slang_ast node);

/// Possible outcomes of evaluating a statement (see slang_stmt_eval).
/// Ordinals match slang's Statement::EvalResult.
typedef enum slang_stmt_eval_result {
    SLANG_STMT_EVAL_FAIL = 0,
    SLANG_STMT_EVAL_SUCCESS = 1,
    SLANG_STMT_EVAL_RETURN = 2,
    SLANG_STMT_EVAL_BREAK = 3,
    SLANG_STMT_EVAL_CONTINUE = 4,
    SLANG_STMT_EVAL_DISABLE = 5,
} slang_stmt_eval_result;

/// Evaluates a statement now for its side effects (assignments, jumps out of
/// loops/functions/blocks), mirroring slang::ast::Statement::eval. Runs
/// under a fresh, scratch EvalContext built just for this call, in "script"
/// evaluation mode with a single empty top-level variable frame (see
/// slang::ast::EvalContext::pushEmptyFrame — the same setup
/// slang::ast::ScriptSession uses to run a standalone statement): any local
/// variable the statement declares or writes lives only in that scratch
/// frame and is discarded when the call returns, so this never mutates the
/// design's own storage, and a variable read that was not itself created in
/// this same call (e.g. one belonging to an enclosing scope this statement
/// was originally elaborated in) evaluates as not-constant, yielding
/// SLANG_STMT_EVAL_FAIL. MAY allocate into the compilation's arena (the seal
/// is lifted for the duration of the call, as with slang_expression_eval)
/// and the caller must therefore guarantee exclusive access to the
/// compilation; concurrent calls of any kind (on this or any other node) are
/// a data race. Fails (via `err`, returning SLANG_STMT_EVAL_FAIL) if `node`
/// is not a statement.
SLANG_C_API uint32_t slang_stmt_eval(slang_ast node, slang_error* err);

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

/// For an IntegerLiteral expression, whether the original source token was
/// written without an explicit size (e.g. the `4` in `4'd4` is sized while a
/// bare `4` is not); false for a non-IntegerLiteral node.
SLANG_C_API bool slang_expr_integer_literal_is_declared_unsized(slang_ast node);

/// The value of a RealLiteral expression as a double; 0.0 for a
/// non-RealLiteral node.
SLANG_C_API double slang_expr_real_literal_value(slang_ast node);

/// The value of a TimeLiteral expression (e.g. the `1.5` of `1.5ns`) as a
/// double, in the units of slang_expr_time_literal_scale; 0.0 for a
/// non-TimeLiteral node. A pure, allocation-free read — mirrors
/// slang::ast::TimeLiteral::getValue. history: since 1.3.
SLANG_C_API double slang_expr_time_literal_value(slang_ast node);

/// The time scale in effect for a TimeLiteral expression's context (see
/// slang_time_scale) — mirrors slang::ast::TimeLiteral::getScale. Returns
/// false (leaving `*out` untouched) for a non-TimeLiteral node or a null
/// `out`. A pure, allocation-free read. history: since 1.3.
SLANG_C_API bool slang_expr_time_literal_scale(slang_ast node, slang_time_scale* out);

/// The raw single-bit value of an UnbasedUnsizedIntegerLiteral expression
/// (e.g. the `1` of `'1`, before it is filled out to the expression's type),
/// as a four-state bit: 0, 1, 2 (X), or 3 (Z) — same encoding as
/// slang_svint_get_bit. Mirrors
/// slang::ast::UnbasedUnsizedIntegerLiteral::getLiteralValue. 0 for a
/// non-UnbasedUnsizedIntegerLiteral node. A pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API uint8_t slang_expr_unbased_unsized_literal_bit(slang_ast node);

/// The value of an UnbasedUnsizedIntegerLiteral expression sized to the type
/// of the expression (e.g. `'1` in an 8-bit context yields `8'hff`), as a
/// structured constant (same representation as
/// slang_expression_constant_value) — mirrors
/// slang::ast::UnbasedUnsizedIntegerLiteral::getValue. This is always
/// available (the fill is computed from the already-set literal bit and
/// type, not lazily folded), so this never evaluates and is a pure read.
/// Fails (via `err`) if `expr` is not an UnbasedUnsizedIntegerLiteral.
/// history: since 1.3.
SLANG_C_API slang_constant slang_expr_unbased_unsized_literal_value(slang_ast expr,
                                                                     slang_error* err);

/// The left-hand operand of an Inside expression (`expr inside {...}`); the
/// null node for a non-Inside node.
SLANG_C_API slang_ast slang_expr_inside_left(slang_ast node);

/// The set membership ranges of an Inside expression (`expr inside {a, b:c}`).
/// Count is 0 for a non-Inside node; slang_expr_inside_range returns the
/// null node when out of range.
SLANG_C_API uint32_t slang_expr_inside_range_count(slang_ast node);
SLANG_C_API slang_ast slang_expr_inside_range(slang_ast node, uint32_t index);

/// The size expression of a NewArray expression (`new[n]`, i.e. the `n`); the
/// null node for a non-NewArray node.
SLANG_C_API slang_ast slang_expr_new_array_size(slang_ast node);

/// The optional initializer expression of a NewArray expression
/// (`new[n](init)`, i.e. `init`); the null node if it has none, or this is
/// not a NewArray node.
SLANG_C_API slang_ast slang_expr_new_array_init(slang_ast node);

/// The constructor-call expression of a NewClass expression (`new(args)`) —
/// a bound Call expression invoking the class's constructor. The null node
/// if it has none (no explicit constructor and no arguments), or this is not
/// a NewClass node.
SLANG_C_API slang_ast slang_expr_new_class_constructor_call(slang_ast node);

/// True if a NewClass expression invokes a superclass's constructor
/// (`super.new(...)`); false for a non-NewClass node.
SLANG_C_API bool slang_expr_new_class_is_super_class(slang_ast node);

/// The arguments passed to a NewCovergroup expression
/// (`new covergroup_type(args)`). Count is 0 for a non-NewCovergroup node;
/// slang_expr_new_covergroup_argument returns the null node when out of
/// range.
SLANG_C_API uint32_t slang_expr_new_covergroup_argument_count(slang_ast node);
SLANG_C_API slang_ast slang_expr_new_covergroup_argument(slang_ast node, uint32_t index);

/// The value-setting expression of a TaggedUnion expression (`tag{value}`,
/// i.e. `value`). Null ast if the member being set is a void member (no
/// value expression), or `node` is not a TaggedUnion expression.
SLANG_C_API slang_ast slang_expr_tagged_union_value(slang_ast node);

/* Assignment, AssignmentPattern, ArbitrarySymbol, AssertionInstance, and the
 * system-call breadth of Call expressions. Every accessor below is a pure
 * read of fields set once at expression construction (no lazily-cached
 * memo is involved), so each is safe on a frozen, shared compilation with no
 * matching FreezeVisitor force. history: since 1.3. */

/// The symbol referenced by an ArbitrarySymbol expression (e.g. the module
/// name argument to `$printtimescale`). Null ast for a non-ArbitrarySymbol
/// node.
SLANG_C_API slang_ast slang_expr_arbitrary_symbol(slang_ast node);

/// True if an Assignment expression is a compound assignment (`+=`, `&=`,
/// ...); false for a simple assignment or a non-assignment node.
SLANG_C_API bool slang_expr_assignment_is_compound(slang_ast node);

/// True if an Assignment expression was implied by its lhs being the target
/// of an lvalue argument or port connection (no explicit assignment operator
/// or rhs); false otherwise, including for a non-assignment node.
SLANG_C_API bool slang_expr_assignment_is_lvalue_arg(slang_ast node);

/// The BinaryOperator of a compound Assignment expression's implied operator
/// (e.g. `Add` for `+=`), as its enum ordinal. Returns UINT32_MAX (the same
/// "no value" sentinel used elsewhere in this header, e.g.
/// slang_syntax_struct_name) for a simple assignment, a non-assignment node,
/// or an invalid node.
SLANG_C_API uint32_t slang_expr_assignment_op(slang_ast node);

/// The timing control of an Assignment expression (e.g. the `#5` in
/// `x = #5 y;`), as a node of domain SLANG_AST_TIMING_CONTROL. Null ast if
/// the assignment has none, or for a non-assignment node.
SLANG_C_API slang_ast slang_expr_assignment_timing(slang_ast node);

/// The number of elements in a SimpleAssignmentPattern, StructuredAssignmentPattern,
/// or ReplicatedAssignmentPattern expression (`'{...}`); 0 for any other node.
SLANG_C_API uint32_t slang_expr_pattern_element_count(slang_ast node);

/// The i'th element of an assignment-pattern expression (see
/// slang_expr_pattern_element_count). Null ast if out of range or `node` is
/// not one of those three expression kinds.
SLANG_C_API slang_ast slang_expr_pattern_element(slang_ast node, uint32_t index);

/// True if an AssertionInstance expression is a recursive property
/// instantiation; false otherwise, including for a non-AssertionInstance node.
SLANG_C_API bool slang_expr_assertion_instance_is_recursive(slang_ast node);

/// The number of local variables materialized in the body of an
/// AssertionInstance expression's assertion item; 0 for any other node.
SLANG_C_API uint32_t slang_expr_assertion_instance_local_var_count(slang_ast node);

/// The i'th local variable of an AssertionInstance expression (see
/// slang_expr_assertion_instance_local_var_count), as a node of domain
/// SLANG_AST_SYMBOL. Null ast if out of range.
SLANG_C_API slang_ast slang_expr_assertion_instance_local_var(slang_ast node, uint32_t index);

/// The number of arguments to an AssertionInstance expression's assertion
/// item; 0 for any other node.
SLANG_C_API uint32_t slang_expr_assertion_instance_argument_count(slang_ast node);

/// The formal port symbol of the i'th argument of an AssertionInstance
/// expression (see slang_expr_assertion_instance_argument_count), as a node
/// of domain SLANG_AST_SYMBOL. Null ast if out of range.
SLANG_C_API slang_ast slang_expr_assertion_instance_argument_port(slang_ast node, uint32_t index);

/// The actual argument bound to the i'th argument of an AssertionInstance
/// expression. slang binds each actual as an expression, a sequence/property
/// (assertion) expression, or a clocking event, so the returned node's own
/// `domain` field (SLANG_AST_EXPRESSION, SLANG_AST_ASSERTION_EXPR, or
/// SLANG_AST_TIMING_CONTROL) tells the caller which; a null ast means out of
/// range.
SLANG_C_API slang_ast slang_expr_assertion_instance_argument_actual(slang_ast node,
                                                                     uint32_t index);

/// For a Call expression's iterator-method system call (e.g. `find_first`
/// with a `with` clause), the iterator expression specified with the call
/// (e.g. the `item > 0` in `q.find_first(item) with (item > 0)`). Null ast if
/// this is not such a call.
SLANG_C_API slang_ast slang_expr_call_iterator_expr(slang_ast node);

/// For a Call expression's iterator-method system call, the implicit
/// iterator variable (e.g. `item` above), as a node of domain SLANG_AST_SYMBOL.
/// Null ast if this is not such a call.
SLANG_C_API slang_ast slang_expr_call_iterator_var(slang_ast node);

/// For a Call expression's `randomize` system call, the inline constraints
/// specified with the call (e.g. the `x > 0` in `obj.randomize() with
/// { x > 0; }`), as a node of domain SLANG_AST_CONSTRAINT. Null ast if this
/// is not such a call, or the call has no inline constraints.
SLANG_C_API slang_ast slang_expr_call_randomize_inline_constraints(slang_ast node);

/// The kind of extra info attached to a Call expression's system-call info:
/// 0 if none, 1 if it is iterator-method info (see slang_expr_call_iterator_expr
/// / slang_expr_call_iterator_var), 2 if it is randomize-method info (see
/// slang_expr_call_randomize_inline_constraints). 0 for a user-subroutine call
/// or a non-call node.
SLANG_C_API uint32_t slang_expr_call_extra_info_kind(slang_ast node);

/// The scope in which a Call expression's system call occurs, as a node of
/// domain SLANG_AST_SYMBOL (the scope's own owning symbol). Null ast for a
/// user-subroutine call or a non-call node.
SLANG_C_API slang_ast slang_expr_call_system_scope(slang_ast node);

/// The SubroutineKind of a Call expression (0 = Function, 1 = Task), as its
/// enum ordinal. 0 for a non-call node (indistinguishable from a genuine
/// Function call; check slang_expression_is_bad / the node's own kind first
/// if that distinction matters).
SLANG_C_API uint32_t slang_expr_call_subroutine_kind(slang_ast node);

/* Call/Conditional/Conversion/CopyClass/Dist expression breadth, and the
 * Expression base-class lvalue-evaluation and effective-width accessors.
 * Every field read below (other than slang_expr_eval_lvalue) is set once at
 * expression construction, so each is a pure read on a frozen compilation
 * with no matching FreezeVisitor force. history: since 1.3. */

/// The name of the subroutine a Call expression invokes (the user function/
/// task name, or the system task/function name, e.g. "$clog2"). Borrowed;
/// empty for a non-call node.
SLANG_C_API slang_str slang_expr_call_subroutine_name(slang_ast node);

/// True if a Call expression is a system call (as opposed to a call to a
/// user-defined function/task); false for a non-call node.
SLANG_C_API bool slang_expr_call_is_system_call(slang_ast node);

/// For a Call expression that is a class method call, the expression for the
/// implicit `this` (the object the method is called on). Null ast for a
/// static/non-method call, or a non-call node.
SLANG_C_API slang_ast slang_expr_call_this_class(slang_ast node);

/// The number of conditions controlling a ConditionalOp expression
/// (`c ? t : f`); more than one only for a pattern-matching conditional with
/// `&&&`-chained conditions. 0 for a non-conditional node.
SLANG_C_API uint32_t slang_expr_cond_condition_count(slang_ast node);

/// The i'th condition expression of a ConditionalOp expression (see
/// slang_expr_cond_condition_count). Null ast if out of range.
SLANG_C_API slang_ast slang_expr_cond_condition_expr(slang_ast node, uint32_t index);

/// The optional pattern-match pattern of the i'th condition of a
/// ConditionalOp expression (see slang_expr_cond_condition_count), as a node
/// of domain SLANG_AST_PATTERN. Null ast if that condition has no pattern, or
/// out of range.
SLANG_C_API slang_ast slang_expr_cond_condition_pattern(slang_ast node, uint32_t index);

/// True if a Conversion expression is a `const'()` const-cast; false
/// otherwise, including for a non-conversion node.
SLANG_C_API bool slang_expr_conversion_is_const_cast(slang_ast node);

/// True if a Conversion expression was implicitly inserted by the compiler
/// (as opposed to an explicit cast written in the source); false otherwise,
/// including for a non-conversion node.
SLANG_C_API bool slang_expr_conversion_is_implicit(slang_ast node);

/// The source operand of a CopyClass expression (`new that_obj`). Null ast
/// for a non-CopyClass node.
SLANG_C_API slang_ast slang_expr_copy_class_source(slang_ast node);

/// The left-hand operand of a Dist expression (`expr dist {...}`). Null ast
/// for a non-Dist node.
SLANG_C_API slang_ast slang_expr_dist_left(slang_ast node);

/// The number of value/weight items in a Dist expression's `dist {...}` list.
/// 0 for a non-Dist node.
SLANG_C_API uint32_t slang_expr_dist_item_count(slang_ast node);

/// The value (or range) expression of the i'th item of a Dist expression (see
/// slang_expr_dist_item_count). Null ast if out of range.
SLANG_C_API slang_ast slang_expr_dist_item_value(slang_ast node, uint32_t index);

/// The DistWeight::Kind of the i'th item's weight (0 = PerValue `:=`,
/// 1 = PerRange `:/`), or 0xFFFFFFFF if that item has no explicit weight, is
/// out of range, or `node` is not a Dist expression.
SLANG_C_API uint32_t slang_expr_dist_item_weight_kind(slang_ast node, uint32_t index);

/// The weight expression of the i'th item of a Dist expression, or a null ast
/// if that item has no explicit weight (see slang_expr_dist_item_weight_kind).
SLANG_C_API slang_ast slang_expr_dist_item_weight_expr(slang_ast node, uint32_t index);

/// The DistWeight::Kind of a Dist expression's default weight (applied to any
/// value not covered by an explicit item), or 0xFFFFFFFF if it has none, or
/// `node` is not a Dist expression.
SLANG_C_API uint32_t slang_expr_dist_default_weight_kind(slang_ast node);

/// The expression of a Dist expression's default weight, or a null ast if it
/// has none (see slang_expr_dist_default_weight_kind).
SLANG_C_API slang_ast slang_expr_dist_default_weight_expr(slang_ast node);

/// The effective width (in bits) an expression would have if the types of
/// all known constants within it were declared with only the bits necessary
/// to represent them (see Expression::getEffectiveWidth). Writes the width
/// to `*out` and returns true on success; returns false (leaving `*out`
/// unwritten) if `node` is not an expression or the computation could not
/// determine a width (e.g. an erroneous sub-expression).
SLANG_C_API bool slang_expr_effective_width(slang_ast node, uint32_t* out);

/// An owned compile-time lvalue produced by slang_expr_eval_lvalue (mirrors
/// slang::ast::LValue, which internally may be a plain storage location or a
/// std::vector-backed tree of concatenated lvalues). Owned by the caller;
/// free with slang_lvalue_destroy. history: since 1.3.
typedef struct slang_lvalue_t* slang_lvalue;

/// Evaluates `expr` as a compile-time lvalue (mirrors
/// Expression::evalLValue). Builds a fresh, scratch EvalContext for the call
/// (in "script" evaluation mode, the same setup slang_stmt_eval uses) and, if
/// `expr` resolves to a referenced value symbol (see
/// slang_expression_symbol), materializes a local for it seeded with its
/// type's default value -- so the returned lvalue is scratch storage
/// independent of the design's own state, discarded when the handle is
/// destroyed. Returns NULL if `expr` does not represent an lvalue (e.g. a
/// non-assignable expression kind -- this raises a catchable internal
/// exception in slang, reported the same way as any other malformed input),
/// or if it references no value symbol the evaluation could resolve. MAY
/// allocate into the compilation's arena (the seal is lifted for the
/// duration of the call, as with slang_expression_eval) and the caller must
/// therefore guarantee exclusive access to the compilation. history: since
/// 1.3.
SLANG_C_API slang_lvalue slang_expr_eval_lvalue(slang_ast node, slang_error* err);

/// Frees an lvalue handle returned by slang_expr_eval_lvalue. history: since
/// 1.3.
SLANG_C_API void slang_lvalue_destroy(slang_lvalue lval);

/// True if an lvalue handle is invalid (e.g. slang_expr_eval_lvalue could not
/// resolve a storage location), or `lval` is NULL.
SLANG_C_API bool slang_lvalue_is_bad(slang_lvalue lval);

/// Loads the current value of an lvalue and returns it printed as
/// SystemVerilog (see slang::ast::LValue::load). Owned; an empty string if
/// the lvalue is bad.
SLANG_C_API slang_str slang_lvalue_load(slang_lvalue lval);

/// Stores an integer into an lvalue (see slang::ast::LValue::store),
/// reencoded at the lvalue's own current bit width and signedness. Returns
/// false, storing nothing, if the lvalue is bad or does not currently hold
/// an integer.
SLANG_C_API bool slang_lvalue_store_int(slang_lvalue lval, int64_t value);

/* Replicated/StructuredAssignmentPattern breadth, StreamingConcatenation, and
 * StringLiteral. Every field read below is set once at expression
 * construction (no lazily-cached memo is involved), so each is a pure read
 * on a frozen, shared compilation with no matching FreezeVisitor force.
 * history: since 1.3. */

/// The replication-count expression of a ReplicatedAssignmentPattern
/// expression (`'{n{...}}`, i.e. the `n`). Null ast for any other node.
SLANG_C_API slang_ast slang_expr_replicated_pattern_count(slang_ast node);

/// The number of member setters (`member: value`) in a
/// StructuredAssignmentPattern expression (`'{...}`); 0 for any other node.
SLANG_C_API uint32_t slang_expr_structured_pattern_member_setter_count(slang_ast node);

/// The member symbol set by the i'th member setter (see
/// slang_expr_structured_pattern_member_setter_count), as a node of domain
/// SLANG_AST_SYMBOL. Null ast if out of range.
SLANG_C_API slang_ast slang_expr_structured_pattern_member_setter_member(slang_ast node,
                                                                         uint32_t index);

/// The value expression of the i'th member setter (see
/// slang_expr_structured_pattern_member_setter_count). Null ast if out of
/// range.
SLANG_C_API slang_ast slang_expr_structured_pattern_member_setter_expr(slang_ast node,
                                                                       uint32_t index);

/// The number of type setters (`type_name: value`) in a
/// StructuredAssignmentPattern expression; 0 for any other node.
SLANG_C_API uint32_t slang_expr_structured_pattern_type_setter_count(slang_ast node);

/// The type matched by the i'th type setter (see
/// slang_expr_structured_pattern_type_setter_count), as a node of domain
/// SLANG_AST_SYMBOL (every type is also a symbol). Null ast if out of range.
SLANG_C_API slang_ast slang_expr_structured_pattern_type_setter_type(slang_ast node,
                                                                     uint32_t index);

/// The value expression of the i'th type setter (see
/// slang_expr_structured_pattern_type_setter_count). Null ast if out of
/// range.
SLANG_C_API slang_ast slang_expr_structured_pattern_type_setter_expr(slang_ast node,
                                                                     uint32_t index);

/// The number of index setters (`[n]: value`) in a
/// StructuredAssignmentPattern expression; 0 for any other node.
SLANG_C_API uint32_t slang_expr_structured_pattern_index_setter_count(slang_ast node);

/// The array-index expression of the i'th index setter (see
/// slang_expr_structured_pattern_index_setter_count). Null ast if out of
/// range.
SLANG_C_API slang_ast slang_expr_structured_pattern_index_setter_index(slang_ast node,
                                                                       uint32_t index);

/// The value expression of the i'th index setter (see
/// slang_expr_structured_pattern_index_setter_count). Null ast if out of
/// range.
SLANG_C_API slang_ast slang_expr_structured_pattern_index_setter_expr(slang_ast node,
                                                                       uint32_t index);

/// The default setter expression of a StructuredAssignmentPattern expression
/// (`'{default: value, ...}`, i.e. `value`) — applied to any element that
/// doesn't match a more specific member/type/index setter. Null ast if this
/// pattern has no default setter, or `node` is not a StructuredAssignmentPattern.
SLANG_C_API slang_ast slang_expr_structured_pattern_default_setter(slang_ast node);

/// The bitstream width of a StreamingConcatenation expression (`{<<{...}}`,
/// `{>>{...}}`); 0 for any other node.
SLANG_C_API uint64_t slang_expr_streaming_bitstream_width(slang_ast node);

/// The slice size of a StreamingConcatenation expression: 0 for a
/// left-to-right concatenation, otherwise the size (in bits) of the blocks
/// to slice and reorder for a right-to-left concatenation. 0 for any other
/// node (indistinguishable from a genuine left-to-right streaming
/// concatenation; check the node's own kind first if that distinction
/// matters).
SLANG_C_API uint64_t slang_expr_streaming_slice_size(slang_ast node);

/// True if a StreamingConcatenation expression has a fixed size (as opposed
/// to involving a dynamically sized element, e.g. a queue or dynamic array
/// stream operand); false for any other node.
SLANG_C_API bool slang_expr_streaming_is_fixed_size(slang_ast node);

/// The number of stream expressions in a StreamingConcatenation expression's
/// operand list; 0 for any other node.
SLANG_C_API uint32_t slang_expr_streaming_stream_count(slang_ast node);

/// The operand expression of the i'th stream of a StreamingConcatenation
/// expression (see slang_expr_streaming_stream_count). Null ast if out of
/// range.
SLANG_C_API slang_ast slang_expr_streaming_stream_operand(slang_ast node, uint32_t index);

/// The optional `with` clause expression of the i'th stream of a
/// StreamingConcatenation expression (the range selector in
/// `stream with [a:b]`), see slang_expr_streaming_stream_count. Null ast if
/// out of range, or the stream has no `with` clause.
SLANG_C_API slang_ast slang_expr_streaming_stream_with_expr(slang_ast node, uint32_t index);

/// The value of a StringLiteral expression (the processed text between the
/// quotes, with escapes resolved). Borrowed; empty for a non-StringLiteral
/// node.
SLANG_C_API slang_str slang_expr_string_literal_value(slang_ast node);

/// The raw, unprocessed text of a StringLiteral expression's source token
/// (escapes not resolved; the enclosing double quotes ARE included, e.g.
/// `"hi\n"` for the source `"hi\n"`). Borrowed; empty for a non-StringLiteral
/// node.
SLANG_C_API slang_str slang_expr_string_literal_raw_value(slang_ast node);

/// The value of a StringLiteral expression interpreted as an integer
/// constant (SystemVerilog packs a string literal's bytes into an unsigned
/// packed vector when it's used in an integer context), as a structured
/// constant (same representation as slang_expression_constant_value). Unlike
/// the cached constant of a general expression, this is always available: a
/// string literal's integer value is computed once at construction, not
/// lazily folded, so this never evaluates and is a pure read. Fails (via
/// `err`) if `node` is not a StringLiteral.
SLANG_C_API slang_constant slang_expr_string_literal_int_value(slang_ast node, slang_error* err);

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

/// Creates a driver WITHOUT slang's standard arguments (--std, --top, -D, -I,
/// -W..., etc.) registered — see slang_driver_add_standard_args to add them
/// explicitly afterward. slang_driver_create is the common case and adds them
/// automatically during construction; this variant exists for callers that
/// want to assemble a fully custom argument set (via slang_driver_add_option)
/// from scratch. history: since 1.3.
SLANG_C_API slang_driver slang_driver_create_bare(slang_error* err);

/// Destroys a driver, its source manager, its trees, and any compilation it
/// created; positions obtained from any of them become invalid.
SLANG_C_API void slang_driver_destroy(slang_driver driver);

/// Adds slang's standard set of command-line arguments (--std, --top, -D, -I,
/// -W... warning flags, diagnostics formatting, etc.) to this driver's command
/// line parser, matching slang::driver::Driver::addStandardArgs. Drivers
/// created with slang_driver_create already have them registered, so this is
/// then a no-op; drivers created with slang_driver_create_bare do not, so
/// callers using that constructor call this once (before parsing) to opt in.
/// history: since 1.3.
SLANG_C_API void slang_driver_add_standard_args(slang_driver driver, slang_error* err);

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

/// Options controlling how a command line is tokenized/parsed (see
/// slang_driver_parse_args_with_options). Create with
/// slang_parse_options_create; every option defaults to false, matching what
/// slang_driver_parse_args uses. history: since 1.3.
typedef struct slang_parse_options_t* slang_parse_options;

/// Creates a parse-options set with every option false (slang's defaults).
SLANG_C_API slang_parse_options slang_parse_options_create(slang_error* err);
SLANG_C_API void slang_parse_options_destroy(slang_parse_options options);

/// If set, comments (`#`/`//` line comments, `/* */` block comments) are
/// parsed and ignored rather than treated as arguments. history: since 1.3.
SLANG_C_API void slang_parse_options_set_support_comments(slang_parse_options options, bool value);
SLANG_C_API bool slang_parse_options_support_comments(slang_parse_options options);

/// If set, the first argument is NOT treated as the program name (by default,
/// as in `argv`, it is). history: since 1.3.
SLANG_C_API void slang_parse_options_set_ignore_program_name(slang_parse_options options,
                                                              bool value);
SLANG_C_API bool slang_parse_options_ignore_program_name(slang_parse_options options);

/// If set, environment variables found in argument text are expanded (forms
/// `$VAR`, `$(VAR)`, `${VAR}`). history: since 1.3.
SLANG_C_API void slang_parse_options_set_expand_env_vars(slang_parse_options options, bool value);
SLANG_C_API bool slang_parse_options_expand_env_vars(slang_parse_options options);

/// If set, giving the same (non-list) option more than once does not raise a
/// "duplicate argument" error — the later value wins. history: since 1.3.
SLANG_C_API void slang_parse_options_set_ignore_duplicates(slang_parse_options options, bool value);
SLANG_C_API bool slang_parse_options_ignore_duplicates(slang_parse_options options);

/// As slang_driver_parse_args, but with explicit parse options; `options` may
/// be null for the all-false defaults slang_driver_parse_args uses.
/// history: since 1.3.
SLANG_C_API bool slang_driver_parse_args_with_options(slang_driver driver, int argc,
                                                      const char* const* argv,
                                                      slang_parse_options options,
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

/// Assembles the driver's command-line-derived options into an owned option bag
/// (slang::Driver::createOptionBag). For most uses prefer
/// slang_driver_create_compilation, which applies the same options AND wires up
/// the driver's source libraries, library maps, and user-defined subroutines;
/// this exposes the raw bag for advanced composition (building a compilation by
/// hand via slang_compilation_create_from_bag). Free with slang_bag_destroy.
SLANG_C_API slang_bag slang_driver_create_option_bag(slang_driver driver, slang_error* err);

/// Destroys an option bag from slang_driver_create_option_bag.
SLANG_C_API void slang_bag_destroy(slang_bag bag);

/// Reports the compilation's diagnostics through the driver's configured
/// diagnostic output (colors, `--diag-*` options, error limits).
SLANG_C_API void slang_driver_report_compilation(slang_driver driver, slang_compilation comp,
                                                 bool quiet, slang_error* err);

/// Prints the summary line ("Build succeeded: ...") and returns whether the
/// run had no errors.
SLANG_C_API bool slang_driver_report_diagnostics(slang_driver driver, bool quiet, slang_error* err);

/// The version of the SystemVerilog language this driver will use (see
/// slang::driver::Driver::languageVersion). Raw slang::LanguageVersion value:
/// 0 = 1364-2005, 1 = 1800-2017, 2 = 1800-2023 (same encoding as
/// slang_compilation_options::language_version). Reflects the --std option
/// once slang_driver_process_options has run; before that it is the default
/// (1800-2017). A pure, allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_driver_language_version(slang_driver driver);

/// Processes command file(s) matching `pattern` for more options, like
/// passing "-f pattern" on the command line (see
/// slang::driver::Driver::processCommandFiles). If make_relative is true,
/// paths within the file are resolved relative to the file itself rather than
/// the current working directory; if separate_unit is true, the file is
/// treated as a separate compilation-unit listing whose options apply only to
/// that unit rather than the whole compilation. Returns false (having already
/// printed why) if the pattern matched no files, a file could not be read, or
/// options within it were rejected. history: since 1.3.
SLANG_C_API bool slang_driver_process_command_files(slang_driver driver, const char* pattern,
                                                    size_t pattern_len, bool make_relative,
                                                    bool separate_unit, slang_error* err);

/// Metadata collected about a single command file processed by
/// slang_driver_process_command_files (see slang_driver_command_file_metadata_at
/// and slang::driver::Driver::CommandFileMetadata). Borrowed; a snapshot that
/// is rebuilt (invalidating previously returned handles) every time
/// slang_driver_process_command_files runs again on the same driver.
/// history: since 1.3.
typedef struct slang_command_file_metadata_t* slang_command_file_metadata;

/// The number of command files slang_driver_process_command_files has
/// processed so far on this driver; every processed command file has an
/// entry, even if it contributed no metadata. A pure, allocation-free read of
/// the cached snapshot. history: since 1.3.
SLANG_C_API uint32_t slang_driver_command_file_metadata_count(slang_driver driver);

/// Fetches command-file metadata entry `index` (see
/// slang_driver_command_file_metadata_count). Null if out of range.
/// history: since 1.3.
SLANG_C_API slang_command_file_metadata slang_driver_command_file_metadata_at(slang_driver driver,
                                                                              uint32_t index);

/// The canonical path of the command file this metadata describes. Borrowed.
/// history: since 1.3.
SLANG_C_API slang_str slang_command_file_metadata_path(slang_command_file_metadata meta);

/// The number of raw -D defines this command file contributed (see
/// slang::driver::Driver::CommandFileMetadata::defines). A pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_command_file_metadata_define_count(slang_command_file_metadata meta);

/// Fetches define string `index` (see
/// slang_command_file_metadata_define_count), in the raw "-D NAME[=VALUE]"
/// form the command file spelled it. Borrowed. history: since 1.3.
SLANG_C_API slang_str slang_command_file_metadata_define_at(slang_command_file_metadata meta,
                                                             uint32_t index);

/// Writes any dependency files requested via the driver's --depfile-style
/// options (a no-op if none were configured); see
/// slang::driver::Driver::optionallyWriteDepFiles. Errors (e.g. a file that
/// could not be written) are printed through the driver, not raised here.
/// history: since 1.3.
SLANG_C_API void slang_driver_optionally_write_dep_files(slang_driver driver, slang_error* err);

/// A handle to a driver's diagnostic engine (see slang_driver_diag_engine),
/// used internally to classify, filter, and format diagnostics as they are
/// issued. Borrowed; valid for the driver's lifetime. history: since 1.3.
typedef struct slang_diag_engine_t* slang_diag_engine;

/// The driver's diagnostic engine (slang::driver::Driver::diagEngine). A
/// pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_diag_engine slang_driver_diag_engine(slang_driver driver);

/// The number of error-severity diagnostics issued through this engine so
/// far. A pure, allocation-free read. history: since 1.3.
SLANG_C_API int32_t slang_diag_engine_num_errors(slang_diag_engine engine);

/// The number of warning-severity diagnostics issued through this engine so
/// far. A pure, allocation-free read. history: since 1.3.
SLANG_C_API int32_t slang_diag_engine_num_warnings(slang_diag_engine engine);

/// A snapshot of the analysis options a driver's configured flags would
/// produce (see slang_driver_get_analysis_options and
/// slang::analysis::AnalysisOptions). Position struct: trivially copyable.
/// history: since 1.3.
typedef struct slang_analysis_options {
    /// Bitmask of slang_analysis_flag.
    uint32_t flags;
    uint32_t max_case_analysis_steps;
    uint32_t max_loop_analysis_steps;
} slang_analysis_options;

/// The analysis options this driver's configured options would produce for
/// slang_analysis_run (see slang::driver::Driver::getAnalysisOptions). A
/// pure, allocation-free read (the returned struct is a snapshot copy).
/// history: since 1.3.
SLANG_C_API slang_analysis_options slang_driver_get_analysis_options(slang_driver driver);

/// Flags controlling slang_driver_run_preprocessor's output. Values match
/// slang::driver::PreprocessOutputFlags and are append-only. history: since 1.3.
typedef enum slang_preprocess_output_flag {
    /// Include comments in the output (otherwise stripped).
    SLANG_PREPROCESS_INCLUDE_COMMENTS = 1u << 0,
    /// Include preprocessor directives in the output (otherwise stripped).
    SLANG_PREPROCESS_INCLUDE_DIRECTIVES = 1u << 1,
    /// Obfuscate identifiers by replacing them with randomized alphanumeric
    /// strings.
    SLANG_PREPROCESS_OBFUSCATE_IDS = 1u << 2,
    /// With SLANG_PREPROCESS_OBFUSCATE_IDS, use a fixed randomization seed so
    /// obfuscated names are reproducible across runs (used for testing).
    SLANG_PREPROCESS_USE_FIXED_OBFUSCATION_SEED = 1u << 3,
    /// Include source line information in the output.
    SLANG_PREPROCESS_INCLUDE_SOURCE_INFO = 1u << 4,
} slang_preprocess_output_flag;

/// Runs the preprocessor on all of the driver's loaded source buffers and
/// prints the result to stdout (see slang::driver::Driver::runPreprocessor).
/// `flags` is a bitmask of slang_preprocess_output_flag. Returns false (having
/// already printed the diagnostics) if a preprocessing error occurred.
/// history: since 1.3.
SLANG_C_API bool slang_driver_run_preprocessor(slang_driver driver, uint32_t flags,
                                               slang_error* err);

/// Prints all macros defined while preprocessing the driver's loaded source
/// buffers to stdout, one per line. If `group_by_file` is true, macros are
/// grouped and labelled by the file that defined them (see
/// slang::driver::Driver::reportMacros). history: since 1.3.
SLANG_C_API void slang_driver_report_macros(slang_driver driver, bool group_by_file,
                                            slang_error* err);

/// Reports (through this driver's diagnostic engine) all diagnostics found
/// while parsing every syntax tree loaded via slang_driver_parse_sources, plus
/// any library maps loaded along the way (see
/// slang::driver::Driver::reportParseDiags). Returns true if none of them were
/// errors. history: since 1.3.
SLANG_C_API bool slang_driver_report_parse_diags(slang_driver driver, slang_error* err);

/// Runs a full compile-report-analyze-report cycle exactly as the `slang` CLI
/// does: creates a compilation from the driver's loaded sources and options,
/// reports its diagnostics, runs analysis, and reports the final summary (see
/// slang::driver::Driver::runFullCompilation). If `quiet` is true,
/// non-essential output is suppressed. Returns true if the run had no errors.
/// history: since 1.3.
SLANG_C_API bool slang_driver_run_full_compilation(slang_driver driver, bool quiet,
                                                   slang_error* err);

/// Sets whether the driver's error/warning messages (slang_driver_print_error-
/// style output) and its text diagnostic client use terminal color codes (see
/// slang::driver::Driver::setTerminalColorsEnabled). history: since 1.3.
SLANG_C_API void slang_driver_set_terminal_colors_enabled(slang_driver driver, bool enable,
                                                          slang_error* err);

/// A handle to a driver's source loader (see slang_driver_source_loader),
/// which handles loading and parsing groups of source files: file-pattern
/// globs, library maps, search directories, and separately-compiled units.
/// Borrowed; valid for the driver's lifetime. history: since 1.3.
typedef struct slang_source_loader_t* slang_source_loader;

/// The driver's source loader (slang::driver::Driver::sourceLoader). A pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_source_loader slang_driver_source_loader(slang_driver driver);

/// Adds files to be loaded, specified via the given glob `pattern` (see
/// slang::driver::SourceLoader::addFiles). Errors (e.g. no file matched) are
/// recorded and surfaced the next time sources are parsed, not raised here.
/// history: since 1.3.
SLANG_C_API void slang_source_loader_add_files(slang_source_loader loader, const char* pattern,
                                               size_t pattern_len, slang_error* err);

/// Adds library files to be loaded under the library named `library_name`,
/// specified via the given glob `pattern` (see
/// slang::driver::SourceLoader::addLibraryFiles). Library files are only
/// considered "used" if referenced from the main source; their modules are
/// not automatically instantiated. history: since 1.3.
SLANG_C_API void slang_source_loader_add_library_files(slang_source_loader loader,
                                                        const char* library_name,
                                                        size_t library_name_len,
                                                        const char* pattern, size_t pattern_len,
                                                        slang_error* err);

/// Loads and parses library map files matching `pattern`, creating the
/// libraries they declare and queuing the files they reference for loading
/// (see slang::driver::SourceLoader::addLibraryMaps). `base_path` resolves
/// relative paths within the map; `options` (nullable) supplies the parse
/// options used to parse the map file itself. history: since 1.3.
SLANG_C_API void slang_source_loader_add_library_maps(slang_source_loader loader,
                                                       const char* pattern, size_t pattern_len,
                                                       const char* base_path, size_t base_path_len,
                                                       slang_options options /* nullable */,
                                                       slang_error* err);

/// Adds directories in which to search for library module files, specified
/// via the given glob `pattern` (see
/// slang::driver::SourceLoader::addSearchDirectories). A search for a library
/// module occurs when there are instantiations found for unknown modules (or
/// interfaces or programs); the given directories are searched for files
/// named after the missing module plus any registered search extensions (see
/// slang_source_loader_add_search_extension). history: since 1.3.
SLANG_C_API void slang_source_loader_add_search_directories(slang_source_loader loader,
                                                             const char* pattern,
                                                             size_t pattern_len, slang_error* err);

/// Adds a file extension (without the leading `.`) used when searching for
/// library module files in the directories registered via
/// slang_source_loader_add_search_directories (see
/// slang::driver::SourceLoader::addSearchExtension). The extensions ".v" and
/// ".sv" are always included automatically. history: since 1.3.
SLANG_C_API void slang_source_loader_add_search_extension(slang_source_loader loader,
                                                           const char* extension,
                                                           size_t extension_len, slang_error* err);

/// Adds a group of files as a separately compiled compilation unit (see
/// slang::driver::SourceLoader::addSeparateUnit). Unlike files added via
/// slang_source_loader_add_files, every file matching one of the
/// `file_patterns` globs is guaranteed to be grouped into a single
/// compilation unit and preprocessed with the given `include_paths` and
/// `defines` (each `NAME` or `NAME=VALUE`, without a leading `-D`). If
/// `library_name` is non-empty the unit is included in the library of that
/// name; otherwise it is included in the default library as a non-library
/// unit. `warning_options` are `-W`-style strings (without the leading `-W`)
/// applied to diagnostics from this unit. Every string array argument may be
/// null when its matching count is 0. history: since 1.3.
SLANG_C_API void slang_source_loader_add_separate_unit(
    slang_source_loader loader, const char* const* file_patterns, size_t file_patterns_count,
    const char* const* include_paths, size_t include_paths_count, const char* const* defines,
    size_t defines_count, const char* library_name, size_t library_name_len,
    const char* const* warning_options, size_t warning_options_count, slang_error* err);

/// True if there is at least one source file queued to load (see
/// slang::driver::SourceLoader::hasFiles). A pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API bool slang_source_loader_has_files(slang_source_loader loader);

/// The number of errors recorded while loading files so far (e.g. a glob
/// pattern that matched nothing) — see slang_source_loader_error_at and
/// slang::driver::SourceLoader::getErrors. A pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API uint32_t slang_source_loader_error_count(slang_source_loader loader);

/// The text of the `index`'th loading error (see
/// slang_source_loader_error_count). Borrowed; empty if out of range.
/// history: since 1.3.
SLANG_C_API slang_str slang_source_loader_error_at(slang_source_loader loader, uint32_t index);

/// The number of library map syntax trees loaded and parsed so far via
/// slang_source_loader_add_library_maps (see
/// slang::driver::SourceLoader::getLibraryMaps). A pure, allocation-free
/// read. history: since 1.3.
SLANG_C_API uint32_t slang_source_loader_library_map_count(slang_source_loader loader);

/// The `index`'th library map syntax tree (see
/// slang_source_loader_library_map_count). Borrowed; retain it (see
/// slang_syntax_tree_retain) to keep it beyond the loader's lifetime. Null if
/// out of range. history: since 1.3.
SLANG_C_API slang_syntax_tree slang_source_loader_library_map_at(slang_source_loader loader,
                                                                  uint32_t index);

/// Loads (but does not parse) every source that has been added to the loader
/// so far, replacing any buffers loaded by a previous call (see
/// slang::driver::SourceLoader::loadSources). Returns the number of buffers
/// loaded; read them back with slang_source_loader_loaded_buffer_id and
/// slang_source_loader_loaded_buffer_text. history: since 1.3.
SLANG_C_API uint32_t slang_source_loader_load_sources(slang_source_loader loader,
                                                       slang_error* err);

/// The buffer ID of the `index`'th buffer loaded by the last
/// slang_source_loader_load_sources call. Zero (the invalid buffer ID) if out
/// of range. history: since 1.3.
SLANG_C_API slang_buffer_id slang_source_loader_loaded_buffer_id(slang_source_loader loader,
                                                                  uint32_t index);

/// The source text of the `index`'th buffer loaded by the last
/// slang_source_loader_load_sources call (see
/// slang_source_loader_loaded_buffer_id). Borrowed from the driver's source
/// manager; empty if out of range. history: since 1.3.
SLANG_C_API slang_str slang_source_loader_loaded_buffer_text(slang_source_loader loader,
                                                              uint32_t index);

/// A handle to an owned slang::driver::SourceOptions value: the options that
/// control how slang_source_loader_load_sources's counterpart
/// loadAndParseSources divides files into compilation units and threads.
/// Create with slang_source_options_create and free with
/// slang_source_options_destroy. history: since 1.3.
typedef struct slang_source_options_t* slang_source_options;

/// Creates a default-constructed source options value (numThreads unset,
/// singleUnit/onlyLint/librariesInheritMacros all false). history: since 1.3.
SLANG_C_API slang_source_options slang_source_options_create(slang_error* err);

/// Destroys a source options value. history: since 1.3.
SLANG_C_API void slang_source_options_destroy(slang_source_options options);

/// Sets the number of threads to use for loading and parsing
/// (slang::driver::SourceOptions::numThreads). Pass `has_value` false to
/// clear it back to "unset" (slang picks a default). history: since 1.3.
SLANG_C_API void slang_source_options_set_num_threads(slang_source_options options,
                                                       bool has_value, uint32_t value);

/// Reads slang::driver::SourceOptions::numThreads: returns true and writes
/// `*out` (if non-null) when a thread count has been set, false (leaving
/// `*out` untouched) when it is unset. history: since 1.3.
SLANG_C_API bool slang_source_options_num_threads(slang_source_options options, uint32_t* out);

/// Sets slang::driver::SourceOptions::singleUnit: if true, all source files
/// are treated as one compilation unit (their text is merged). history:
/// since 1.3.
SLANG_C_API void slang_source_options_set_single_unit(slang_source_options options, bool value);

/// Reads slang::driver::SourceOptions::singleUnit. history: since 1.3.
SLANG_C_API bool slang_source_options_single_unit(slang_source_options options);

/// Sets slang::driver::SourceOptions::onlyLint: if true, only lint the code
/// rather than elaborating a full hierarchy. history: since 1.3.
SLANG_C_API void slang_source_options_set_only_lint(slang_source_options options, bool value);

/// Reads slang::driver::SourceOptions::onlyLint. history: since 1.3.
SLANG_C_API bool slang_source_options_only_lint(slang_source_options options);

/// Sets slang::driver::SourceOptions::librariesInheritMacros: if true,
/// library files inherit macro definitions from primary source files.
/// history: since 1.3.
SLANG_C_API void slang_source_options_set_libraries_inherit_macros(slang_source_options options,
                                                                    bool value);

/// Reads slang::driver::SourceOptions::librariesInheritMacros. history: since
/// 1.3.
SLANG_C_API bool slang_source_options_libraries_inherit_macros(slang_source_options options);

/// A handle to a driver's text diagnostic client (see
/// slang_driver_text_diag_client), which formats diagnostics issued through
/// the driver's diagnostic engine as human-readable text. Borrowed; valid for
/// the driver's lifetime. history: since 1.3.
typedef struct slang_text_diag_client_t* slang_text_diag_client;

/// The driver's text diagnostic client (slang::driver::Driver::textDiagClient),
/// registered with the driver's diagnostic engine at construction. Note that
/// slang::driver::Driver always constructs this as a StderrDiagnosticClient
/// (a TextDiagnosticClient subclass): it streams each diagnostic straight to
/// the process's stderr and clears its own text buffer immediately
/// afterward, so slang_text_diag_client_get_string reads back empty right
/// after a report call even though real diagnostic text was printed. A pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_text_diag_client slang_driver_text_diag_client(slang_driver driver);

/// The client's currently accumulated formatted-text buffer (see
/// slang_driver_text_diag_client for why this driver's client clears itself
/// right after every report). Owned. history: since 1.3.
SLANG_C_API slang_str slang_text_diag_client_get_string(slang_text_diag_client client,
                                                        slang_error* err);

/// True if the client's formatted-text buffer is empty. history: since 1.3.
SLANG_C_API bool slang_text_diag_client_empty(slang_text_diag_client client);

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

/// Runs analysis over `comp` using the analysis options and thread pool
/// configured on `driver` (see slang::driver::Driver::runAnalysis and
/// slang_driver_get_analysis_options), reporting the resulting diagnostics
/// through the driver's diagnostic engine. This forces `comp`'s diagnostics
/// (as slang::driver::Driver::runAnalysis does internally) and transiently
/// freezes/unfreezes it around the analysis pass, leaving it in the same
/// sealed/unsealed state it was in before the call. Unlike slang_analysis_run,
/// in the driver's lint-only mode (--lint-only) this returns an analysis
/// manager that was never run (matching the driver's own behavior). MAY
/// allocate; caller must hold exclusive access to `comp`. history: since 1.3.
SLANG_C_API slang_analysis slang_driver_run_analysis(slang_driver driver, slang_compilation comp,
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

/// A handle to a single driver of a value (slang::analysis::ValueDriver): an
/// assignment or connection that writes it. Unlike slang_driver_info (a
/// flattened snapshot returned by value/index), this is a live handle that
/// exposes the raw underlying fields. Trivially copyable; `ptr` is null for
/// "none"; valid while `analysis` is alive. history: since 1.3.
typedef struct slang_value_driver {
    const void* ptr;
    slang_analysis analysis;
} slang_value_driver;

#ifdef __cplusplus
static_assert(sizeof(slang_value_driver) == 2 * sizeof(void*), "slang_value_driver ABI");
#endif

/// The raw bits of this driver (slang::analysis::ValueDriver::flags, of type
/// slang::analysis::DriverFlags). Unlike slang_driver_info::flags (which
/// exposes only 3 curated, derived bits), this is the literal underlying
/// bitmask — see slang_driver_flags_raw for its bit layout. A pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_value_driver_flags(slang_value_driver driver);

/// Bits in slang_value_driver_flags's result. Values match
/// slang::analysis::DriverFlags exactly (a direct bitmask, not curated).
typedef enum slang_driver_flags_raw {
    SLANG_DRIVER_FLAG_INPUT_PORT = 1u << 0,
    SLANG_DRIVER_FLAG_OUTPUT_PORT = 1u << 1,
    SLANG_DRIVER_FLAG_CLOCK_VAR = 1u << 2,
    SLANG_DRIVER_FLAG_INITIALIZER = 1u << 3,
    SLANG_DRIVER_FLAG_FROM_SIDE_EFFECT = 1u << 4,
    SLANG_DRIVER_FLAG_HAS_OVERRIDE_RANGE = 1u << 5,
    SLANG_DRIVER_FLAG_VIA_INDIRECT_PORT = 1u << 6,
} slang_driver_flags_raw;

/// The symbol assigned to by this driver (slang::analysis::ValueDriver::
/// getSymbol). Null (ptr-less slang_ast) if `driver` is invalid. A pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_value_driver_symbol(slang_value_driver driver);

/// The bit range assigned to by this driver (slang::analysis::ValueDriver::
/// getBounds), written to `*lo`/`*hi` (either may be null to ignore). Returns
/// false (leaving `*lo`/`*hi` untouched) if `driver` is invalid. A pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API bool slang_value_driver_bounds(slang_value_driver driver, uint64_t* lo, uint64_t* hi);

/// The source range describing this driver as written in the source code
/// (slang::analysis::ValueDriver::getSourceRange). A zeroed slang_range if
/// `driver` is invalid. A pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_range slang_value_driver_source_range(slang_value_driver driver);

/// An optional extra source range indicating the driver actually came from
/// some other, indirected location, such as a modport port expansion
/// (slang::analysis::ValueDriver::getOverrideRange). Written to `*out` if
/// non-null. Returns false (leaving `*out` untouched) if there is no override
/// range or `driver` is invalid. A pure, allocation-free read. history: since
/// 1.3.
SLANG_C_API bool slang_value_driver_override_range(slang_value_driver driver, slang_range* out);

/// True if this driver is for a unidirectional port, i.e. an input or output
/// port (as opposed to inout or ref) (slang::analysis::ValueDriver::
/// isUnidirectionalPort). False if `driver` is invalid. A pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API bool slang_value_driver_is_unidirectional_port(slang_value_driver driver);

/// True if this driver lives inside a single-driver procedure, such as
/// `always_comb`, `always_latch`, or `always_ff`
/// (slang::analysis::ValueDriver::isInSingleDriverProcedure). False if
/// `driver` is invalid. A pure, allocation-free read. history: since 1.3.
SLANG_C_API bool slang_value_driver_is_in_single_driver_procedure(slang_value_driver driver);

/// The kind of construct a driver originated from
/// (slang::analysis::ValueDriver::source): a procedural block kind,
/// a subroutine, or something else.
typedef enum slang_driver_source {
    SLANG_DRIVER_SOURCE_INITIAL = 0,
    SLANG_DRIVER_SOURCE_FINAL = 1,
    SLANG_DRIVER_SOURCE_ALWAYS = 2,
    SLANG_DRIVER_SOURCE_ALWAYS_COMB = 3,
    SLANG_DRIVER_SOURCE_ALWAYS_LATCH = 4,
    SLANG_DRIVER_SOURCE_ALWAYS_FF = 5,
    SLANG_DRIVER_SOURCE_SUBROUTINE = 6,
    SLANG_DRIVER_SOURCE_OTHER = 7,
} slang_driver_source;

/// The source of this driver (slang::analysis::ValueDriver::source): the kind
/// of procedural block, subroutine, or other construct it came from.
/// SLANG_DRIVER_SOURCE_OTHER if `driver` is invalid. A pure, allocation-free
/// read. history: since 1.3.
SLANG_C_API slang_driver_source slang_value_driver_source(slang_value_driver driver);

/// A handle to the value path driven by a slang_value_driver (slang::ast::
/// ValuePath): the target value and sub-path (field accesses, element
/// selects, ...) being assigned. Trivially copyable; `ptr` is null for
/// "none"; valid while `analysis` is alive. history: since 1.3.
typedef struct slang_value_path {
    const void* ptr;
    slang_analysis analysis;
} slang_value_path;

#ifdef __cplusplus
static_assert(sizeof(slang_value_path) == 2 * sizeof(void*), "slang_value_path ABI");
#endif

/// The path driven by this driver (slang::analysis::ValueDriver::path). A
/// null-ptr'd handle if `driver` is invalid. A pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API slang_value_path slang_value_driver_path(slang_value_driver driver);

/// The value symbol at the root of a value path (slang::ast::ValuePath::
/// rootSymbol). Null (ptr-less slang_ast) if `path` is invalid or has no
/// root. A pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_value_path_root_symbol(slang_value_path path);

/// Fetches a handle to driver `index` of a value symbol (see
/// slang_analysis_driver_count, which returns the same driver's flattened
/// slang_driver_info via slang_analysis_driver). The `ptr` field of the
/// result is null if `index` is out of range. A pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API slang_value_driver slang_analysis_driver_handle(slang_analysis analysis,
                                                             slang_ast value, uint32_t index);

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
/* Analyzed procedures, assertions, and their sub-results                     */
/* ------------------------------------------------------------------------- */

/// A handle to an analyzed procedure (slang::analysis::AnalyzedProcedure): the
/// per-procedure analysis results (drivers, subroutine calls, timing controls,
/// read sets, inferred clock, effective sensitivity list) computed for one
/// `always`/`initial`/`final` block, continuous assignment, or subroutine.
/// Trivially copyable; `ptr` is null for "none"; valid while `analysis` is
/// alive. history: since 1.3.
typedef struct slang_analyzed_procedure {
    const void* ptr;
    slang_analysis analysis;
} slang_analyzed_procedure;

/// A handle to an analyzed concurrent assertion or procedural checker
/// instantiation (slang::analysis::AnalyzedAssertion). Trivially copyable;
/// `ptr` is null for "none"; valid while `analysis` is alive. history: since
/// 1.3.
typedef struct slang_analyzed_assertion {
    const void* ptr;
    slang_analysis analysis;
} slang_analyzed_assertion;

/// A handle to one @* timing region's read set within an analyzed procedure
/// (slang::analysis::AnalyzedProcedure::ImplicitEventReadSet). Trivially
/// copyable; `ptr` is null for "none"; valid while `analysis` is alive.
/// history: since 1.3.
typedef struct slang_implicit_event_read_set {
    const void* ptr;
    slang_analysis analysis;
} slang_implicit_event_read_set;

/// A handle to one (symbol, bit-range) entry of an analyzed procedure's read
/// set (slang::analysis::ReadRange). Trivially copyable; `ptr` is null for
/// "none"; valid while `analysis` is alive. history: since 1.3.
typedef struct slang_read_range {
    const void* ptr;
    slang_analysis analysis;
} slang_read_range;

/// A handle to an analyzed procedure's effective sensitivity list
/// (slang::analysis::SensitivityList). Trivially copyable; `ptr` is null for
/// "none"; valid while `analysis` is alive. history: since 1.3.
typedef struct slang_sensitivity_list {
    const void* ptr;
    slang_analysis analysis;
} slang_sensitivity_list;

#ifdef __cplusplus
static_assert(sizeof(slang_analyzed_procedure) == 2 * sizeof(void*), "slang_analyzed_procedure ABI");
static_assert(sizeof(slang_analyzed_assertion) == 2 * sizeof(void*), "slang_analyzed_assertion ABI");
static_assert(sizeof(slang_implicit_event_read_set) == 2 * sizeof(void*),
             "slang_implicit_event_read_set ABI");
static_assert(sizeof(slang_read_range) == 2 * sizeof(void*), "slang_read_range ABI");
static_assert(sizeof(slang_sensitivity_list) == 2 * sizeof(void*), "slang_sensitivity_list ABI");
#endif

/// The kind of a procedure's effective sensitivity list. Values match
/// slang::analysis::SensitivityList::Kind. history: since 1.3.
typedef enum slang_sensitivity_kind {
    /// No event-based sensitivity (e.g. `initial`/`final`, or no timing control).
    SLANG_SENSITIVITY_NONE = 0,
    /// Explicit sensitivity list specified in source (e.g. `always @(posedge clk)`).
    SLANG_SENSITIVITY_EXPLICIT = 1,
    /// Implicit sensitivity derived from signals read (e.g. `always_comb`, `always @*`).
    SLANG_SENSITIVITY_IMPLICIT = 2,
    /// Multiple/non-leading event controls; no single static sensitivity.
    SLANG_SENSITIVITY_DYNAMIC = 3,
} slang_sensitivity_kind;

/// Gets a handle to procedure `index` of an analyzed scope (see
/// slang_analysis_scope_procedure_count and slang_analysis_scope_procedure,
/// which return the same procedure's analyzed *symbol*). The `ptr` field of
/// the result is null if `index` is out of range. A pure, allocation-free
/// read. history: since 1.3.
SLANG_C_API slang_analyzed_procedure slang_analysis_scope_procedure_handle(slang_analysis analysis,
                                                                           slang_ast scope,
                                                                           uint32_t index);

/// The number of analyzed assertions (concurrent assertions and procedural
/// checker instantiations) found within `containing_symbol` (see
/// slang::analysis::AnalysisManager::getAnalyzedAssertions). Zero if none were
/// analyzed there. history: since 1.3.
SLANG_C_API uint32_t slang_analysis_assertion_count(slang_analysis analysis,
                                                    slang_ast containing_symbol);

/// Gets a handle to assertion `index` within `containing_symbol` (see
/// slang_analysis_assertion_count). The `ptr` field of the result is null if
/// `index` is out of range. history: since 1.3.
SLANG_C_API slang_analyzed_assertion slang_analysis_assertion_at(slang_analysis analysis,
                                                                 slang_ast containing_symbol,
                                                                 uint32_t index);

/// The symbol that contains this assertion (slang::analysis::AnalyzedAssertion::
/// containingSymbol). A pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_analyzed_assertion_containing_symbol(slang_analyzed_assertion assertion);

/// The procedure that contains this assertion, if any (slang::analysis::
/// AnalyzedAssertion::procedure). The `ptr` field of the result is null if the
/// assertion has no containing procedure (e.g. a module-level property
/// instantiation). A pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_analyzed_procedure slang_analyzed_assertion_procedure(
    slang_analyzed_assertion assertion);

/// The AST node that describes this assertion (slang::analysis::
/// AnalyzedAssertion::astNode): either a concurrent assertion statement
/// (domain SLANG_AST_STATEMENT) or an assertion instance expression bound
/// standalone, e.g. from a procedural checker (domain SLANG_AST_EXPRESSION). A
/// pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_analyzed_assertion_ast_node(slang_analyzed_assertion assertion);

/// The root of this assertion's expression tree (slang::analysis::
/// AnalyzedAssertion::getRoot), as a node of domain SLANG_AST_ASSERTION_EXPR.
/// A pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_analyzed_assertion_root(slang_analyzed_assertion assertion);

/// The semantic leading clock of this assertion (slang::analysis::
/// AnalyzedAssertion::getSemanticLeadingClock), as a node of domain
/// SLANG_AST_TIMING_CONTROL. Null (ptr-less slang_ast) if the assertion has no
/// leading clock. A pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_analyzed_assertion_semantic_leading_clock(
    slang_analyzed_assertion assertion);

/// The clock that applies to `expr` (a node of domain SLANG_AST_ASSERTION_EXPR
/// belonging to this assertion's tree), per slang::analysis::AnalyzedAssertion::
/// getClock. Null (ptr-less slang_ast) if `expr` is multi-clocked (its
/// subexpressions must be examined individually) or not part of this
/// assertion. A pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_analyzed_assertion_clock(slang_analyzed_assertion assertion,
                                                     slang_ast expr);

/// The symbol that was analyzed (slang::analysis::AnalyzedProcedure::
/// analyzedSymbol): the `always`/`initial`/`final` block, continuous
/// assignment, or subroutine itself. A pure, allocation-free read. history:
/// since 1.3.
SLANG_C_API slang_ast slang_analyzed_procedure_symbol(slang_analyzed_procedure procedure);

/// The procedure that contains this one, if any (slang::analysis::
/// AnalyzedProcedure::parentProcedure). Only ever non-null for procedural
/// checker instances (a checker instantiated inside another procedure). The
/// `ptr` field of the result is null if there is no parent procedure. A
/// pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_analyzed_procedure slang_analyzed_procedure_parent(
    slang_analyzed_procedure procedure);

/// The inferred clocking block for this procedure, if any (slang::analysis::
/// AnalyzedProcedure::getInferredClock), as a node of domain
/// SLANG_AST_TIMING_CONTROL. Clock inference is only performed for procedures
/// containing at least one concurrent assertion. A pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API slang_ast slang_analyzed_procedure_inferred_clock(slang_analyzed_procedure procedure);

/// The number of drivers recorded directly on this procedure (slang::analysis::
/// AnalyzedProcedure::getDrivers). A pure, allocation-free read. history:
/// since 1.3.
SLANG_C_API uint32_t slang_analyzed_procedure_driver_count(slang_analyzed_procedure procedure);

/// Fetches driver `index` of this procedure (see
/// slang_analyzed_procedure_driver_count). Returns false if out of range.
/// history: since 1.3.
SLANG_C_API bool slang_analyzed_procedure_driver_at(slang_analyzed_procedure procedure,
                                                    uint32_t index, slang_driver_info* out);

/// The number of subroutine call expressions found in this procedure
/// (slang::analysis::AnalyzedProcedure::getCallExpressions). A pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_analyzed_procedure_call_expression_count(
    slang_analyzed_procedure procedure);

/// Call expression `index` of this procedure (see
/// slang_analyzed_procedure_call_expression_count), as a node of domain
/// SLANG_AST_EXPRESSION. Null (ptr-less slang_ast) if out of range. history:
/// since 1.3.
SLANG_C_API slang_ast slang_analyzed_procedure_call_expression_at(slang_analyzed_procedure procedure,
                                                                  uint32_t index);

/// The number of timing control statements found directly in this procedure
/// (slang::analysis::AnalyzedProcedure::getTimingControls). A pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_analyzed_procedure_timing_control_count(
    slang_analyzed_procedure procedure);

/// Timing control statement `index` of this procedure (see
/// slang_analyzed_procedure_timing_control_count), as a node of domain
/// SLANG_AST_STATEMENT. Null (ptr-less slang_ast) if out of range. history:
/// since 1.3.
SLANG_C_API slang_ast slang_analyzed_procedure_timing_control_at(slang_analyzed_procedure procedure,
                                                                 uint32_t index);

/// The number of (symbol, bit-range) entries read anywhere in this procedure
/// (slang::analysis::AnalyzedProcedure::getReadSet). A pure, allocation-free
/// read. history: since 1.3.
SLANG_C_API uint32_t slang_analyzed_procedure_read_set_count(slang_analyzed_procedure procedure);

/// Read-set entry `index` of this procedure (see
/// slang_analyzed_procedure_read_set_count). The `ptr` field of the result is
/// null if out of range. history: since 1.3.
SLANG_C_API slang_read_range slang_analyzed_procedure_read_set_at(slang_analyzed_procedure procedure,
                                                                  uint32_t index);

/// The number of per-@*-region read sets in this procedure (slang::analysis::
/// AnalyzedProcedure::getImplicitEventReadSets). A pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API uint32_t slang_analyzed_procedure_implicit_event_read_set_count(
    slang_analyzed_procedure procedure);

/// Implicit-event read set `index` of this procedure (see
/// slang_analyzed_procedure_implicit_event_read_set_count). The `ptr` field of
/// the result is null if out of range. history: since 1.3.
SLANG_C_API slang_implicit_event_read_set slang_analyzed_procedure_implicit_event_read_set_at(
    slang_analyzed_procedure procedure, uint32_t index);

/// This procedure's effective sensitivity list (slang::analysis::
/// AnalyzedProcedure::getSensitivityList). Never null-ptr for a valid
/// `procedure`. A pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_sensitivity_list slang_analyzed_procedure_sensitivity_list(
    slang_analyzed_procedure procedure);

/// The @*-timed statement this read set belongs to (slang::analysis::
/// AnalyzedProcedure::ImplicitEventReadSet::statement), as a node of domain
/// SLANG_AST_STATEMENT. A pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_implicit_event_read_set_statement(
    slang_implicit_event_read_set read_set);

/// The number of (symbol, bit-range) entries read in this @* region. A pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_implicit_event_read_set_read_count(
    slang_implicit_event_read_set read_set);

/// Read entry `index` of this @* region (see
/// slang_implicit_event_read_set_read_count). The `ptr` field of the result is
/// null if out of range. history: since 1.3.
SLANG_C_API slang_read_range slang_implicit_event_read_set_read_at(
    slang_implicit_event_read_set read_set, uint32_t index);

/// The symbol being read (slang::analysis::ReadRange::symbol). A pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_read_range_symbol(slang_read_range range);

/// The bit range being read (slang::analysis::ReadRange::bitRange), written to
/// `*lo`/`*hi` (either may be null to ignore). Returns false (leaving
/// `*lo`/`*hi` untouched) if `range` is invalid. A pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API bool slang_read_range_bit_range(slang_read_range range, uint64_t* lo, uint64_t* hi);

/// The kind of a sensitivity list (slang::analysis::SensitivityList::kind). A
/// pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_sensitivity_kind slang_sensitivity_list_kind(slang_sensitivity_list list);

/// For SLANG_SENSITIVITY_EXPLICIT: the timing control containing the explicit
/// sensitivity (slang::analysis::SensitivityList::timingControl), as a node of
/// domain SLANG_AST_TIMING_CONTROL. Null (ptr-less slang_ast) otherwise. A
/// pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_sensitivity_list_timing_control(slang_sensitivity_list list);

/// The number of (symbol, bit-range) entries forming this sensitivity list. A
/// pure, allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_sensitivity_list_read_count(slang_sensitivity_list list);

/// Read entry `index` of this sensitivity list (see
/// slang_sensitivity_list_read_count). The `ptr` field of the result is null
/// if out of range. history: since 1.3.
SLANG_C_API slang_read_range slang_sensitivity_list_read_at(slang_sensitivity_list list,
                                                            uint32_t index);

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

/// An opaque handle to the running analysis's own context, passed to
/// slang_dfa_lattice::on_case_begin / on_conditional_begin / on_loop_begin.
/// Valid only for the duration of the callback that received it — do not
/// retain it. Mirrors what pyslang's PyFlowAnalysis exposes to Python
/// callbacks as methods on the analysis object itself (getCurrentState,
/// isBad, getEvalCtx). history: since 1.3.
typedef struct slang_dfa_ctx_t* slang_dfa_ctx;

/// An opaque handle to an evaluation context (slang::ast::EvalContext) used
/// during a custom dataflow analysis for the analysis's own constant folding
/// (slang::analysis::FlowAnalysisBase::getEvalContext). Valid only for the
/// duration of the callback that produced it. history: since 1.3.
typedef struct slang_eval_ctx_t* slang_eval_ctx;

/// The current flow state at this point in the walk (slang::analysis::
/// AbstractFlowAnalysis::getState, exposed to pyslang as PyFlowAnalysis::
/// getCurrentState). This is the SAME opaque pointer that
/// slang_dfa_lattice::transfer receives as `state` for events occurring
/// here; provided on `ctx` too so hooks that are not per-event (on_case_begin
/// and friends) can still read or mutate it. Null if `ctx` is invalid.
/// history: since 1.3.
SLANG_C_API void* slang_dfa_ctx_state(slang_dfa_ctx ctx);

/// True if the analysis has recorded an unrecoverable error, such as an
/// InvalidStatement or InvalidExpression having been visited (slang::
/// analysis::FlowAnalysisBase::bad, exposed to pyslang as PyFlowAnalysis::
/// isBad). False if `ctx` is invalid. A pure, allocation-free read. history:
/// since 1.3.
SLANG_C_API bool slang_dfa_ctx_is_bad(slang_dfa_ctx ctx);

/// The evaluation context this analysis uses for its own constant folding —
/// e.g. to determine whether a for-loop's stop condition is statically known
/// so it can be unrolled (slang::analysis::FlowAnalysisBase::
/// getEvalContext, exposed to pyslang as PyFlowAnalysis::getEvalCtx). Use
/// slang_eval_ctx_evaluate to fold an expression through it. Null (ptr-less)
/// if `ctx` is invalid. history: since 1.3.
SLANG_C_API slang_eval_ctx slang_dfa_ctx_eval_context(slang_dfa_ctx ctx);

/// Evaluates `expr` (an AST node of domain SLANG_AST_EXPRESSION) to a
/// constant value using `ectx` (slang::ast::Expression::eval). Returns null
/// on evaluation failure or invalid input. MAY allocate (constant evaluation
/// can allocate into the arena, exactly like the analysis's own internal
/// folding); caller must free a non-null result with slang_constant_destroy.
/// history: since 1.3.
SLANG_C_API slang_constant slang_eval_ctx_evaluate(slang_eval_ctx ectx, slang_ast expr,
                                                   slang_error* err);

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

    /// Optional (may be null). Called when the analysis begins visiting a
    /// case statement, before visiting any of its branches (slang::
    /// analysis::AbstractFlowAnalysis::visitStmt(const CaseStatement&),
    /// exposed to pyslang as PyFlowAnalysis's onCaseBegin callback). `stmt`
    /// has domain SLANG_AST_STATEMENT.
    void (*on_case_begin)(void* user, slang_dfa_ctx ctx, slang_ast stmt);
    /// Optional (may be null). Called when the analysis begins visiting a
    /// conditional (if/else) statement, before visiting its branches
    /// (slang::analysis::AbstractFlowAnalysis::visitStmt(const
    /// ConditionalStatement&), exposed to pyslang as PyFlowAnalysis's
    /// onConditionalBegin callback). `stmt` has domain SLANG_AST_STATEMENT.
    void (*on_conditional_begin)(void* user, slang_dfa_ctx ctx, slang_ast stmt);
    /// Optional (may be null). Called when the analysis begins visiting any
    /// loop statement (for/while/do-while/forever/foreach/repeat), before
    /// its body (exposed to pyslang as PyFlowAnalysis's onLoopBegin
    /// callback). `stmt` has domain SLANG_AST_STATEMENT.
    void (*on_loop_begin)(void* user, slang_dfa_ctx ctx, slang_ast stmt);
} slang_dfa_lattice;

/// Runs the caller's forward dataflow analysis over a procedure symbol (an
/// `always`/`initial`/`final` block, a subroutine, or a continuous assign).
/// Returns the exit state (a fresh copy the caller owns and must free with
/// `lattice->drop`), or NULL on error or if `procedure` is not a procedure.
SLANG_C_API void* slang_dfa_run(slang_compilation comp, slang_ast procedure,
                                const slang_dfa_lattice* lattice, void* user, slang_error* err);

/* ------------------------------------------------------------------------- */
/* Script session                                                             */
/* ------------------------------------------------------------------------- */

/// A session for evaluating snippets of SystemVerilog and keeping state
/// (declared variables, packages, ...) across calls (mirrors
/// slang::ast::ScriptSession). Owned; free with slang_script_session_destroy.
/// history: since 1.3.
typedef struct slang_script_session_t* slang_script_session;

/// Creates a new script session. `options` (nullable) supplies the parse and
/// compilation options applied to every snippet evaluated in it.
/// history: since 1.3.
SLANG_C_API slang_script_session slang_script_session_create(slang_options options /* nullable */,
                                                              slang_error* err);

/// Destroys a script session, its internal compilation, and any syntax trees
/// it parsed.
SLANG_C_API void slang_script_session_destroy(slang_script_session session);

/// Evaluates one snippet of SystemVerilog source in this session's scope: an
/// expression, statement, or declaration (a variable, function, task, module,
/// typedef, or package), maintaining state (declared variables and their
/// values, defined types, ...) across calls (see
/// slang::ast::ScriptSession::eval). Returns the snippet's constant value, or
/// NULL if it produced none (e.g. a declaration) or could not be evaluated.
/// Allocates into the session's own compilation/arena. history: since 1.3.
SLANG_C_API slang_constant slang_script_session_eval(slang_script_session session,
                                                     const char* text, size_t text_len,
                                                     slang_error* err);

/// Evaluates a parsed expression syntax node (e.g. from a tree parsed via
/// slang_syntax_tree_from_text, or any other live slang_node whose kind is an
/// expression) against this session's scope — so an identifier in `expr`
/// resolves against variables this session has declared, regardless of which
/// tree `expr` itself came from (see
/// slang::ast::ScriptSession::evalExpression). Returns NULL if `expr` is not
/// an expression node or could not be evaluated. history: since 1.3.
SLANG_C_API slang_constant slang_script_session_eval_expression(slang_script_session session,
                                                                 slang_node expr,
                                                                 slang_error* err);

/// As slang_script_session_eval_expression, but for a parsed statement syntax
/// node, evaluated for its side effects only (see
/// slang::ast::ScriptSession::evalStatement). Fails (via `err`) if `stmt` is
/// not a statement node. history: since 1.3.
SLANG_C_API void slang_script_session_eval_statement(slang_script_session session,
                                                      slang_node stmt, slang_error* err);

/// This session's own compilation (slang::ast::ScriptSession::compilation) —
/// a never-sealed scratch compilation, still open to further changes, used to
/// hold declared script state; NOT a frozen Design. Returns a fresh handle
/// borrowing it (the underlying compilation is destroyed with the session,
/// not with this handle); free the returned handle with
/// slang_compilation_destroy when done with it. history: since 1.3.
SLANG_C_API slang_compilation slang_script_session_compilation(slang_script_session session,
                                                                slang_error* err);

/// All diagnostics issued by every slang_script_session_eval* call on this
/// session so far (see slang::ast::ScriptSession::getDiagnostics). Owned by
/// the caller; free with slang_diagnostics_destroy. history: since 1.3.
SLANG_C_API slang_diagnostics slang_script_session_diagnostics(slang_script_session session,
                                                                slang_error* err);

/* ------------------------------------------------------------------------- */
/* MethodPrototypeSymbol / ModportClockingSymbol / InterfacePortSymbol /     */
/* LetDeclSymbol / Lookup                                                    */
/* ------------------------------------------------------------------------- */

/* A MethodPrototypeSymbol represents a class/interface method's prototype (a
 * `pure virtual`/`extern` declaration, or a modport's imported/exported
 * task/function signature) -- slang::ast::MethodPrototypeSymbol. Read
 * through slang_ast (SLANG_AST_SYMBOL, SymbolKind::MethodPrototype); every
 * accessor below returns its stated default for any other symbol kind.
 * history: since 1.3. */

/// Bits returned by slang_symbol_method_prototype_flags. Values match
/// slang::ast::MethodFlags exactly (a direct bitmask). history: since 1.3.
typedef enum slang_method_flag {
    SLANG_METHOD_NONE = 0,
    SLANG_METHOD_VIRTUAL = 1u << 0,
    SLANG_METHOD_PURE = 1u << 1,
    SLANG_METHOD_STATIC = 1u << 2,
    SLANG_METHOD_CONSTRUCTOR = 1u << 3,
    SLANG_METHOD_INTERFACE_EXTERN = 1u << 4,
    SLANG_METHOD_MODPORT_IMPORT = 1u << 5,
    SLANG_METHOD_MODPORT_EXPORT = 1u << 6,
    SLANG_METHOD_DPI_IMPORT = 1u << 7,
    SLANG_METHOD_DPI_CONTEXT = 1u << 8,
    SLANG_METHOD_BUILT_IN = 1u << 9,
    SLANG_METHOD_RANDOMIZE = 1u << 10,
    SLANG_METHOD_FORK_JOIN = 1u << 11,
    SLANG_METHOD_DEFAULTED_SUPER_ARG = 1u << 12,
    SLANG_METHOD_INITIAL = 1u << 13,
    SLANG_METHOD_EXTENDS = 1u << 14,
    SLANG_METHOD_FINAL = 1u << 15,
    SLANG_METHOD_PRE_POST_RANDOMIZE = 1u << 16,
} slang_method_flag;

/// `sym`'s flags (see slang_method_flag), as a raw bitmask -- slang::ast::
/// MethodPrototypeSymbol::flags. Set once at construction, so a pure,
/// allocation-free read. 0 (SLANG_METHOD_NONE) for a non-MethodPrototype
/// symbol. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_method_prototype_flags(slang_ast sym);

/// `sym`'s subroutine kind (0 = Function, 1 = Task), as a raw slang::ast::
/// SubroutineKind value -- slang::ast::MethodPrototypeSymbol::
/// subroutineKind. Set once at construction, so a pure, allocation-free
/// read. Also 0 (indistinguishable from an actual Function) for a
/// non-MethodPrototype symbol -- check the symbol's own kind first.
/// history: since 1.3.
SLANG_C_API uint32_t slang_symbol_method_prototype_subroutine_kind(slang_ast sym);

/// `sym`'s declared visibility (public unless marked `local`/`protected`) --
/// slang::ast::MethodPrototypeSymbol::visibility. Set once at construction,
/// so a pure, allocation-free read. SLANG_VISIBILITY_PUBLIC for a
/// non-MethodPrototype symbol. history: since 1.3.
SLANG_C_API slang_visibility slang_symbol_method_prototype_visibility(slang_ast sym);

/// True if `sym` is virtual: explicitly `virtual`/`extends`, or it overrides
/// a base-class method (see slang_symbol_method_prototype_override) --
/// slang::ast::MethodPrototypeSymbol::isVirtual. False for a
/// non-MethodPrototype symbol. A pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API bool slang_symbol_method_prototype_is_virtual(slang_ast sym);

/// The number of formal arguments declared on `sym`'s prototype --
/// slang::ast::MethodPrototypeSymbol::getArguments. 0 for a
/// non-MethodPrototype symbol. The argument list is built once, during
/// elaboration, before this symbol is ever reachable through a frozen
/// &Design, so this is a pure, allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_method_prototype_argument_count(slang_ast sym);

/// The formal argument at `index` (see
/// slang_symbol_method_prototype_argument_count), as a node of domain
/// SLANG_AST_SYMBOL (SymbolKind::FormalArgument). A null node if `sym` is
/// not a MethodPrototype symbol or `index` is out of range.
/// history: since 1.3.
SLANG_C_API slang_ast slang_symbol_method_prototype_argument(slang_ast sym, uint32_t index);

/// `sym`'s resolved return type, as a node of domain SLANG_AST_SYMBOL --
/// slang::ast::MethodPrototypeSymbol::getReturnType (declaredReturnType.
/// getType()). This is the same memo slang_declared_type_type reaches
/// generically (MethodPrototypeSymbol is one of the DeclaredType carriers
/// the freeze sweep forces for every symbol), so this is a pure read. A
/// null node for a non-MethodPrototype symbol. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_method_prototype_return_type(slang_ast sym);

/// The concrete subroutine this prototype resolves to -- an out-of-block
/// `extern`/pure-virtual implementation, or a synthesized stub for an
/// unimplemented pure method -- as a node of domain SLANG_AST_SYMBOL
/// (SymbolKind::Subroutine). A null node if `sym` is not a MethodPrototype
/// symbol, or resolution failed (a diagnostic was issued instead of a
/// stub). Mirrors slang::ast::MethodPrototypeSymbol::getSubroutine. The
/// underlying memo is reached (and forced) by the freeze sweep's generic
/// scope-member traversal whenever this prototype is itself a reachable
/// scope member -- the only way a C accessor can ever observe it in the
/// first place -- so this is a pure read on a frozen design.
/// history: since 1.3.
SLANG_C_API slang_ast slang_symbol_method_prototype_subroutine(slang_ast sym);

/// The symbol `sym` overrides, set via slang::ast::MethodPrototypeSymbol::
/// setOverride during class-hierarchy resolution -- a plain field read
/// (slang::ast::MethodPrototypeSymbol::getOverride). A null node if `sym`
/// is not a MethodPrototype symbol, or it overrides nothing. A pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_method_prototype_override(slang_ast sym);

/// A single `extern` implementation registered against a
/// MethodPrototypeSymbol (slang::ast::MethodPrototypeSymbol::ExternImpl) --
/// one node of the singly-linked list
/// slang_symbol_method_prototype_first_extern_impl /
/// slang_extern_impl_next walks (built when an InterfaceExtern method is
/// implemented by one or more modules). Opaque and borrowed; valid as long
/// as the owning compilation is alive. history: since 1.3.
typedef struct slang_extern_impl_t* slang_extern_impl;

/// The first `extern` implementation registered against `sym` (see
/// slang_extern_impl_next to walk the rest), or a null handle if `sym` is
/// not a MethodPrototype symbol or none has been registered. Mirrors
/// slang::ast::MethodPrototypeSymbol::getFirstExternImpl. A pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_extern_impl slang_symbol_method_prototype_first_extern_impl(slang_ast sym);

/// The next `extern` implementation after `impl` in its list (see
/// slang_symbol_method_prototype_first_extern_impl), or a null handle at the
/// end of the list, or if `impl` itself is null. Mirrors slang::ast::
/// MethodPrototypeSymbol::ExternImpl::getNextImpl. A pure, allocation-free
/// read. history: since 1.3.
SLANG_C_API slang_extern_impl slang_extern_impl_next(slang_extern_impl impl);

/// The SubroutineSymbol `impl` wraps (never null for a non-null `impl`), as
/// a node of domain SLANG_AST_SYMBOL -- slang::ast::MethodPrototypeSymbol::
/// ExternImpl::impl. `comp` supplies the compilation to wrap the result
/// against -- pass the compilation of the slang_ast `impl` was reached
/// through (e.g. the owning MethodPrototypeSymbol's own `.compilation`). A
/// null node if `impl` is null. A pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API slang_ast slang_extern_impl_impl(slang_extern_impl impl, slang_compilation comp);

/* A ModportClockingSymbol represents a clocking block exposed through an
 * interface modport (`modport m(clocking cb);`) -- slang::ast::
 * ModportClockingSymbol. history: since 1.3. */

/// The clocking block `sym` exposes -- slang::ast::ModportClockingSymbol::
/// target -- as a node of domain SLANG_AST_SYMBOL. A null node if `sym` is
/// not a ModportClocking symbol. Set once at construction (a plain field),
/// so a pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_modport_clocking_target(slang_ast sym);

/// The interface instance (and, if applicable, modport) a module/program's
/// InterfacePort symbol is connected to -- a plain value pair, not an AST
/// node. Either half may be a null slang_ast (no connection resolved, or no
/// modport restriction). Mirrors slang::ast::InterfacePortSymbol::IfaceConn
/// (std::pair<const Symbol*, const ModportSymbol*>). history: since 1.3.
typedef struct slang_iface_conn {
    slang_ast instance;
    slang_ast modport;
} slang_iface_conn;

/// The interface instance (and modport, if restricted) that an InterfacePort
/// symbol connects to. Both halves are null (ptr-less) slang_ast values if
/// `sym` is not an InterfacePort symbol. The connection is lazily resolved
/// by slang but forced for every InterfacePort symbol by the freeze sweep
/// (FreezeVisitor's InterfacePortSymbol branch), so this is a pure read on a
/// frozen design. Mirrors slang::ast::InterfacePortSymbol::getConnection.
/// history: since 1.3.
SLANG_C_API slang_iface_conn slang_symbol_interface_port_connection(slang_ast sym);

/// The `index`'th resolved port connection of an Instance symbol
/// (slang_instance_port_connection_count /
/// slang_instance_port_connection_port), as an interface connection: the
/// interface instance (and modport, if restricted) it binds to. Both halves
/// are null (ptr-less) slang_ast values if the connection is not to an
/// InterfacePort (i.e. its port is a plain Port or MultiPort symbol), or if
/// `index` is out of range or `instance` is not an Instance symbol. Reads
/// only plain, non-lazy fields set when the connection was built (slang::
/// ast::PortConnection::getIfaceConn), so this is a pure, allocation-free
/// read -- no freeze-sweep force needed. history: since 1.3.
SLANG_C_API slang_iface_conn slang_instance_port_connection_iface_conn(slang_ast instance,
                                                                        uint32_t index);

/// The number of dimensions of an InterfacePort symbol's declared array
/// range (e.g. 1 for `.bus[3:0]`). 0 if `sym` is not an InterfacePort
/// symbol, has no array dimensions, or the dimensions failed to evaluate.
/// Forced (alongside the connection) by the freeze sweep, so a pure read.
/// Mirrors slang::ast::InterfacePortSymbol::getDeclaredRange (the span's
/// size). history: since 1.3.
SLANG_C_API uint32_t slang_symbol_interface_port_declared_range_count(slang_ast sym);

/// The `index`'th dimension of an InterfacePort symbol's declared array
/// range (see slang_symbol_interface_port_declared_range_count). A
/// zero-valued range if `index` is out of range. history: since 1.3.
SLANG_C_API slang_constant_range slang_symbol_interface_port_declared_range_at(slang_ast sym,
                                                                                uint32_t index);

/// The Definition symbol for an InterfacePort symbol's interface, as a node
/// of domain SLANG_AST_SYMBOL. A null node for a generic interface port (see
/// slang_symbol_interface_port_is_generic) or if `sym` is not an
/// InterfacePort symbol. A direct field read
/// (slang::ast::InterfacePortSymbol::interfaceDef) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_interface_port_interface_def(slang_ast sym);

/// True if an InterfacePort symbol is a generic interface port (`interface
/// bus;` with no named interface, accepting a connection to any interface
/// type); false if `sym` is not an InterfacePort symbol. A direct field read
/// (slang::ast::InterfacePortSymbol::isGeneric). history: since 1.3.
SLANG_C_API bool slang_symbol_interface_port_is_generic(slang_ast sym);

/// True if an InterfacePort symbol failed to resolve to either a named
/// interface definition or a generic interface (both interfaceDef and
/// isGeneric are unset); false (including for a non-InterfacePort symbol).
/// Mirrors slang::ast::InterfacePortSymbol::isInvalid. history: since 1.3.
SLANG_C_API bool slang_symbol_interface_port_is_invalid(slang_ast sym);

/// The modport name restricting an InterfacePort symbol's accessible
/// signals (e.g. "master" for `.bus(intf.master)`), or empty if the port has
/// no modport restriction or `sym` is not an InterfacePort symbol. Borrowed
/// (points into the source text). A direct field read
/// (slang::ast::InterfacePortSymbol::modport). history: since 1.3.
SLANG_C_API slang_str slang_symbol_interface_port_modport(slang_ast sym);

/// The number of formal argument ports of a LetDeclSymbol (`let` construct).
/// 0 if `sym` is not a LetDecl symbol. A direct field read
/// (slang::ast::LetDeclSymbol::ports) -- a pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API uint32_t slang_symbol_let_decl_port_count(slang_ast sym);

/// The `index`'th formal argument port of a LetDeclSymbol, as a node of
/// domain SLANG_AST_SYMBOL (an AssertionPort symbol). A null node if `sym`
/// is not a LetDecl symbol or `index` is out of range. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_let_decl_port(slang_ast sym, uint32_t index);

/* slang::ast::Lookup: centralized name-resolution and accessibility logic.
 * The pure predicates (getVisibility/isVisibleFrom/isAccessibleFrom) read
 * only already-elaborated Symbol fields. Every other function here performs
 * a fresh name lookup or accessibility diagnostic pass, which records
 * reference-tracking state and/or issues diagnostics into the compilation's
 * arena -- exactly like slang_scope_lookup, so each internally lifts the
 * seal for its own duration; callers must not invoke these on a design
 * shared across threads (mirrors the `&mut Design` requirement documented
 * on the safe Rust wrapper). history: since 1.3. */

/// If `symbol` is a class member, its declared accessibility
/// (public/protected/local); SLANG_VISIBILITY_PUBLIC for anything else,
/// including a non-symbol `symbol`. A pure, allocation-free read. Mirrors
/// slang::ast::Lookup::getVisibility.
SLANG_C_API slang_visibility slang_lookup_get_visibility(slang_ast symbol);

/// True if `symbol` is visible from `scope` (a class member's public/
/// protected/local accessibility, per SystemVerilog visibility rules); true
/// for any non-class-member symbol. False if `symbol` or `scope` is
/// invalid. A pure, allocation-free read (issues no diagnostics). Mirrors
/// slang::ast::Lookup::isVisibleFrom.
SLANG_C_API bool slang_lookup_is_visible_from(slang_ast symbol, slang_ast scope);

/// True if instance member `target` is accessible from `source_scope` (a
/// symbol, typically a class type or an instance context) -- i.e. they share
/// the same parent scope, or `source_scope` is (or derives from) the class
/// that owns `target`. Does not consider visibility modifiers (see
/// slang_lookup_is_visible_from for those). False if either argument is
/// invalid. A pure, allocation-free read. Mirrors
/// slang::ast::Lookup::isAccessibleFrom.
SLANG_C_API bool slang_lookup_is_accessible_from(slang_ast target, slang_ast source_scope);

/// Like slang_lookup_is_visible_from, but built on the real
/// slang::ast::Lookup::ensureVisible: constructs a fresh ASTContext rooted
/// at `context_scope` (LookupLocation::max, no randomize/assertion details)
/// and issues no diagnostic (called with no source range). Returns the same
/// boolean slang_lookup_is_visible_from would for this pair, through the
/// diagnostic-issuing entry point rather than the direct predicate -- kept
/// as a distinct binding because slang exposes them as separate API
/// surfaces with separate contracts (ensureVisible is what expression
/// binding actually calls). False if `symbol` or `context_scope` is
/// invalid. Allocates the diagnostic report (even when empty) into the
/// design's arena, so requires exclusive access. history: since 1.3.
SLANG_C_API bool slang_lookup_ensure_visible(slang_ast symbol, slang_ast context_scope,
                                              slang_error* err);

/// Checks whether `symbol` (an instance class member) is accessible for
/// non-static use from `context_scope`, per slang::ast::Lookup::
/// ensureAccessible: false when `context_scope` is a static-only context
/// (e.g. outside any instance of the owning class) or a different, unrelated
/// class than the one that declares `symbol`; true for anything that isn't a
/// class instance member. Issues no diagnostic (called with no source
/// range). False if `symbol` or `context_scope` is invalid. Allocates like
/// slang_lookup_ensure_visible. history: since 1.3.
SLANG_C_API bool slang_lookup_ensure_accessible(slang_ast symbol, slang_ast context_scope,
                                                 slang_error* err);

/// Looks up `name` (parsed fresh, per slang::ast::Compilation::
/// tryParseName) starting in `context_scope`, following the same full
/// SystemVerilog name-resolution rules as slang_scope_lookup but through
/// slang::ast::Lookup::name directly (e.g. supports a scope-resolution
/// `::`-qualified class-scoped name, unlike slang_scope_lookup's plain
/// dot-walk). Returns the found symbol as a node of domain SLANG_AST_SYMBOL,
/// or a null node if nothing was found or either argument is invalid.
/// Allocates (parses a fresh syntax subtree and records reference-tracking
/// state), so requires exclusive access. history: since 1.3.
SLANG_C_API slang_ast slang_lookup_name(slang_ast context_scope, const char* name,
                                         size_t name_len, slang_error* err);

/// Resolves `name` to a class type, per slang::ast::Lookup::findClass:
/// looks it up as a type name starting in `context_scope`, and returns it
/// only if it names a class (any error -- not found, not a class -- is
/// silently swallowed here, matching how findClass reports through the
/// context's diagnostics rather than a return code). Returns the class as a
/// node of domain SLANG_AST_SYMBOL (slang::ast::ClassType, itself a Type and
/// so a Symbol), or a null node on failure or invalid arguments. Allocates
/// like slang_lookup_name. history: since 1.3.
SLANG_C_API slang_ast slang_lookup_find_class(slang_ast context_scope, const char* name,
                                               size_t name_len, slang_error* err);

/// Searches the local variables materialized in the body of an
/// AssertionInstance expression (slang_expr_assertion_instance_local_var)
/// for one named `name`, per slang::ast::Lookup::findAssertionLocalVar:
/// builds a fresh AssertionInstanceDetails from `assertion_inst`'s
/// already-elaborated slang::ast::AssertionInstanceExpression::localVars,
/// then calls the real slang::ast::Lookup::findAssertionLocalVar against
/// it. `context_scope` supplies the ASTContext's scope (used only if `name`
/// requires further member-selection, which a plain identifier never does).
/// Writes the found symbol to `*out_symbol` (a null node if not found) and
/// returns whether a match was found. False (leaving `*out_symbol`
/// untouched) if `assertion_inst` is not an AssertionInstance expression,
/// `context_scope` is invalid, or no local variable named `name` exists.
/// Allocates like slang_lookup_name. history: since 1.3.
SLANG_C_API bool slang_lookup_find_assertion_local_var(slang_ast assertion_inst,
                                                        slang_ast context_scope,
                                                        const char* name, size_t name_len,
                                                        slang_ast* out_symbol, slang_error* err);

/// Searches the linked list of temporary variables headed by `temp_var` (a
/// TempVarSymbol -- e.g. an IteratorSymbol created for an array method's
/// `with` clause, see slang_expr_call_iterator_var) for one named `name`,
/// per slang::ast::Lookup::findTempVar. `context_scope` supplies the
/// ASTContext's scope (used only for further member-selection, which a
/// plain identifier never needs). Writes the found symbol to `*out_symbol`
/// (a null node if not found) and returns whether a match was found. False
/// (leaving `*out_symbol` untouched) if `temp_var` is not a TempVarSymbol or
/// `context_scope` is invalid. Allocates like slang_lookup_name. history:
/// since 1.3.
SLANG_C_API bool slang_lookup_find_temp_var(slang_ast temp_var, slang_ast context_scope,
                                             const char* name, size_t name_len,
                                             slang_ast* out_symbol, slang_error* err);

/* slang::ast::LookupLocation: an ordering of symbols within a scope's member
 * list, used to decide whether a found symbol is visible from a given point
 * (declared-before-use). A plain, trivially copyable value -- never anything
 * to free. history: since 1.3. */

/// A position within a scope's member list (slang::ast::LookupLocation).
/// `scope` is a node of domain SLANG_AST_SYMBOL naming the scope (via
/// Scope::asSymbol()); a null `scope` node is the "no particular scope"
/// sentinel used by slang_lookup_location_min / slang_lookup_location_max,
/// which compare before/after every real location regardless of `index`.
/// Trivially copyable. history: since 1.3.
typedef struct slang_lookup_location {
    slang_ast scope;
    uint32_t index;
} slang_lookup_location;

/// A location placed just before `symbol` in its parent scope's member list,
/// per slang::ast::LookupLocation::before -- what a lookup that must ignore
/// declarations after a given point (e.g. a variable initializer) compares
/// against. A location with a null scope node and index 0 if `symbol` is
/// invalid or has no parent scope. A pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API slang_lookup_location slang_lookup_location_before(slang_ast symbol);

/// A location placed just after `symbol` in its parent scope's member list,
/// per slang::ast::LookupLocation::after. A location with a null scope node
/// and index 0 if `symbol` is invalid or has no parent scope. A pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_lookup_location slang_lookup_location_after(slang_ast symbol);

/// The location that compares after every other location in the same scope
/// (slang::ast::LookupLocation::max) -- what a normal (declaration-order-
/// insensitive) name lookup uses. `comp` gives the returned value's (null)
/// scope node somewhere to belong; it plays no role in the comparison
/// itself. A pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_lookup_location slang_lookup_location_max(slang_compilation comp);

/// The location that compares before every other location in the same scope
/// (slang::ast::LookupLocation::min). See slang_lookup_location_max for
/// `comp`. A pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_lookup_location slang_lookup_location_min(slang_compilation comp);

/// The scope half of `loc`, as a node of domain SLANG_AST_SYMBOL (the
/// scope's owning symbol) -- slang::ast::LookupLocation::getScope, via
/// Scope::asSymbol(). A null node for a location with no scope (e.g.
/// slang_lookup_location_min/_max). A pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API slang_ast slang_lookup_location_get_scope(slang_lookup_location loc);

/// The member-list index half of `loc` -- slang::ast::LookupLocation::
/// getIndex. A pure, allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_lookup_location_get_index(slang_lookup_location loc);

/* slang::ast::LookupResult: the mutable scratch structure a name-lookup
 * operation fills in (slang::ast::Lookup::name, ::withinClassRandomize, and
 * friends). Modeled as an owned handle -- construct one with
 * slang_lookup_result_create, hand it to a lookup entry point, read it back,
 * then free it -- the same shape as a stack-local LookupResult in C++.
 * history: since 1.3. */

typedef struct slang_lookup_result_t* slang_lookup_result;

/// Creates an empty lookup-result scratch buffer (slang::ast::LookupResult,
/// default-constructed). Owned by the caller; free with
/// slang_lookup_result_destroy. history: since 1.3.
SLANG_C_API slang_lookup_result slang_lookup_result_create(void);

/// Destroys a lookup-result created by slang_lookup_result_create. A no-op
/// on null. history: since 1.3.
SLANG_C_API void slang_lookup_result_destroy(slang_lookup_result result);

/// Performs a lookup within a class randomize() scope, per
/// slang::ast::Lookup::withinClassRandomize: resolves `name` (a plain
/// identifier, `this[.super].name`, or `super.name`) against `class_type`'s
/// members first, falling back to nothing (the caller is expected to then
/// perform a normal lookup in `context_scope`) if it starts with `local::`
/// or isn't found. `class_type` must be a class type (its scope is used as
/// slang::ast::ASTContext::RandomizeDetails::classType); `this_var` is the
/// class-handle symbol for a dotted-handle randomize call, or a null node
/// for a bare/static randomize; `context_scope` supplies the ASTContext's
/// own scope (used for `this`/`super` resolution and to warn when `name`
/// also names an unrelated local variable). `result` is cleared, then
/// populated with whatever was found (see slang_lookup_result_found and
/// friends) -- pass a fresh or previously-cleared handle. Returns whether a
/// symbol was found (mirroring the underlying function's return value; the
/// diagnostics collected along the way, including on a false return, are
/// still readable through `result`). False, leaving `result` cleared, if
/// `class_type`, `this_var` (when non-null), `context_scope`, or `result` is
/// invalid, or `name` is null. Allocates (parses a fresh syntax subtree and
/// records reference-tracking state), so requires exclusive access.
/// history: since 1.3.
SLANG_C_API bool slang_lookup_within_class_randomize(slang_ast class_type, slang_ast this_var,
                                                      slang_ast context_scope, const char* name,
                                                      size_t name_len, slang_lookup_result result,
                                                      slang_error* err);

/// The symbol found by the lookup that populated `result` (slang::ast::
/// LookupResult::found), as a node of domain SLANG_AST_SYMBOL. A null node
/// if nothing was found, `result` was never populated or was cleared, or
/// `result` is invalid. A pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_lookup_result_found(slang_lookup_result result);

/// Bits returned by slang_lookup_result_flags. Values match
/// slang::ast::LookupResultFlags exactly (a direct bitmask).
/// history: since 1.3.
typedef enum slang_lookup_result_flag {
    SLANG_LOOKUP_RESULT_NONE = 0,
    SLANG_LOOKUP_RESULT_WAS_IMPORTED = 1u << 0,
    SLANG_LOOKUP_RESULT_IS_HIERARCHICAL = 1u << 1,
    SLANG_LOOKUP_RESULT_SUPPRESS_UNDECLARED = 1u << 2,
    SLANG_LOOKUP_RESULT_FROM_TYPE_PARAM = 1u << 3,
    SLANG_LOOKUP_RESULT_FROM_FORWARD_TYPEDEF = 1u << 4,
    SLANG_LOOKUP_RESULT_IFACE_PORT = 1u << 5,
} slang_lookup_result_flag;

/// `result`'s flags (see slang_lookup_result_flags), as a raw bitmask --
/// slang::ast::LookupResult::flags. 0 (SLANG_LOOKUP_RESULT_NONE) for an
/// invalid or never-populated `result`. A pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API uint32_t slang_lookup_result_flags(slang_lookup_result result);

/// True if an error occurred during the lookup that populated `result`,
/// per slang::ast::LookupResult::hasError: either nothing was found despite
/// an explicit import being expected, or any collected diagnostic (see
/// slang_lookup_result_diagnostics) is itself an error. False for an
/// invalid `result`. A pure, allocation-free read. history: since 1.3.
SLANG_C_API bool slang_lookup_result_has_error(slang_lookup_result result);

/// The system subroutine the lookup that populated `result` found, if the
/// name resolved to one -- slang::ast::LookupResult::systemSubroutine. In
/// that case slang_lookup_result_found returns the null symbol (the two
/// fields are mutually exclusive). A null handle if nothing was found, the
/// found symbol is a regular Symbol instead, or `result` is invalid or
/// never populated. A pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_system_subroutine slang_lookup_result_system_subroutine(
    slang_lookup_result result);

/// The number of scope levels the lookup that populated `result` walked
/// upward through the hierarchy before descending back down to the found
/// symbol -- slang::ast::LookupResult::upwardCount. 0 for a
/// non-hierarchical lookup, one that never needed to go upward, or an
/// invalid `result`. A pure, allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_lookup_result_upward_count(slang_lookup_result result);

/// Resets `result` to the same empty state slang_lookup_result_create
/// produces, per slang::ast::LookupResult::clear. Returns false, doing
/// nothing, if `result` is invalid. A pure, allocation-free write (frees no
/// memory the caller can observe; releases the scratch buffer's own
/// contents). history: since 1.3.
SLANG_C_API bool slang_lookup_result_clear(slang_lookup_result result);

/// Issues a diagnostic (slang::diagnostics::UnexpectedSelection, an error)
/// if `result` has any pending selectors (see
/// slang_lookup_result_selector_count), per slang::ast::LookupResult::
/// errorIfSelectors -- used by a caller that wants to reject `name.member`-
/// style selections it cannot itself apply. Builds a fresh ASTContext
/// rooted at `context_scope` (LookupLocation::max) and issues the
/// diagnostic into that scope's compilation (NOT into `result`'s own
/// diagnostic list). Because slang_compilation_diagnostics /
/// slang_compilation_get_semantic_diagnostics cache their result on first
/// call -- already forced once, pre-seal, by the slang_compilation_freeze
/// that produced this (frozen) compilation -- a diagnostic issued here is
/// never visible through either; slang_compilation_has_issued_errors, which
/// reads the compilation's live error counter instead, is the only way to
/// observe it. A no-op if `result` has no selectors. Returns false, doing
/// nothing, if `result` or `context_scope` is invalid. Allocates the
/// diagnostic report into the design's arena, so requires exclusive access.
/// history: since 1.3.
SLANG_C_API bool slang_lookup_result_error_if_selectors(slang_lookup_result result,
                                                         slang_ast context_scope,
                                                         slang_error* err);

/// Reports every diagnostic collected while populating `result` (see
/// slang_lookup_result_diagnostics) to `context_scope`'s compilation, per
/// slang::ast::LookupResult::reportDiags -- used by a caller (e.g. a
/// randomize() `with` clause binder) that wants slang's own diagnostic
/// engine to see them, rather than silently dropping or re-formatting them
/// itself. Builds a fresh ASTContext rooted at `context_scope`
/// (LookupLocation::max), the same way slang_lookup_result_error_if_selectors
/// does; `result`'s own diagnostic list (slang_lookup_result_diagnostics) is
/// left unchanged. Because slang_compilation_diagnostics /
/// slang_compilation_get_semantic_diagnostics cache their result on first
/// call, a diagnostic reported here after that first call is never visible
/// through either; slang_compilation_has_issued_errors, which reads the
/// compilation's live error counter instead, is the only way to observe it
/// (mirrors slang_lookup_result_error_if_selectors' own note). A no-op,
/// returning true, if `result` has no diagnostics. Returns false, doing
/// nothing, if `result` or `context_scope` is invalid. Allocates the
/// diagnostic reports into the design's arena, so requires exclusive
/// access. history: since 1.3.
SLANG_C_API bool slang_lookup_result_report_diags(slang_lookup_result result,
                                                   slang_ast context_scope, slang_error* err);

/// The diagnostics collected while populating `result` (slang::ast::
/// LookupResult::getDiagnostics), as a fresh owned list -- e.g. the
/// RandomizeConstraintShadow warning slang_lookup_within_class_randomize can
/// leave behind. Distinct from slang_lookup_result_error_if_selectors'
/// UnexpectedSelection diagnostic, which goes to the compilation instead.
/// Empty for an invalid `result`. Owned by the caller; free with
/// slang_diagnostics_destroy. A pure, allocation-free read (the returned
/// handle owns a copy; no state is created inside `result` or its design).
/// history: since 1.3.
SLANG_C_API slang_diagnostics slang_lookup_result_diagnostics(slang_lookup_result result);

/// The number of selectors queued on `result` -- entries of slang::ast::
/// LookupResult::selectors, recorded when a dotted name resolved partway
/// through to a value and the remaining `.member`/`[index]` components were
/// deferred for the caller to apply itself. 0 for an invalid `result`. A
/// pure, allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_lookup_result_selector_count(slang_lookup_result result);

/// True if the selector at `index` is a dotted member selection (slang::
/// ast::LookupResult::MemberSelector) -- readable through
/// slang_lookup_result_selector_name/_dot_location/_name_range -- as
/// opposed to an indexed element selection (a syntax node this API does not
/// yet expose). False if `result` is invalid or `index` is out of range.
/// A pure, allocation-free read. history: since 1.3.
SLANG_C_API bool slang_lookup_result_selector_is_member(slang_lookup_result result,
                                                         uint32_t index);

/// The member name of the MemberSelector at `index` (slang::ast::
/// LookupResult::MemberSelector::name). An empty string if `result` is
/// invalid, `index` is out of range, or that selector is not a
/// MemberSelector (see slang_lookup_result_selector_is_member). A pure,
/// allocation-free read (borrowed; valid as long as `result` is not
/// destroyed or cleared). history: since 1.3.
SLANG_C_API slang_str slang_lookup_result_selector_name(slang_lookup_result result,
                                                         uint32_t index);

/// The source location of the `.` that led to the MemberSelector at `index`
/// (slang::ast::LookupResult::MemberSelector::dotLocation). A zeroed
/// (no-location) slang_loc under the same conditions as
/// slang_lookup_result_selector_name's empty string. A pure, allocation-free
/// read. history: since 1.3.
SLANG_C_API slang_loc slang_lookup_result_selector_dot_location(slang_lookup_result result,
                                                                 uint32_t index);

/// The source range of the member name of the MemberSelector at `index`
/// (slang::ast::LookupResult::MemberSelector::nameRange). A zeroed range
/// under the same conditions as slang_lookup_result_selector_name's empty
/// string. A pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_range slang_lookup_result_selector_name_range(slang_lookup_result result,
                                                                 uint32_t index);

/* ------------------------------------------------------------------------- */
/* ModportPortSymbol / ModportSymbol / MultiPortSymbol / NetAliasSymbol /    */
/* NetSymbol / PackageSymbol                                                 */
/* ------------------------------------------------------------------------- */

/* A ModportPortSymbol represents a single port specifier in a modport
 * declaration (e.g. `input req` in `modport m(input req);`) --
 * slang::ast::ModportPortSymbol. Read through slang_ast (SLANG_AST_SYMBOL,
 * SymbolKind::ModportPort); every accessor below returns its stated default
 * for any other symbol kind. history: since 1.3. */

/// For a ModportPort symbol: the direction of data flowing across the port
/// (`input`/`output`/`inout`/`ref`). SLANG_ARGUMENT_DIRECTION_IN for any
/// other symbol kind. A direct field read
/// (slang::ast::ModportPortSymbol::direction), populated at declaration --
/// a pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_argument_direction slang_symbol_modport_port_direction(slang_ast sym);

/// For a ModportPort symbol: its explicit connection expression, if any
/// (e.g. the `req.next` of `modport m(.req(req.next));`), as a node of
/// domain SLANG_AST_EXPRESSION. A null node if `sym` is not a ModportPort
/// symbol, or the port has no explicit connection -- an implicit port
/// (e.g. plain `input req`) instead connects directly to a like-named
/// internal symbol (see slang_symbol_modport_port_internal_symbol). A
/// direct field read (slang::ast::ModportPortSymbol::explicitConnection)
/// -- a pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_modport_port_explicit_connection(slang_ast sym);

/// For a ModportPort symbol: the instance-internal symbol it connects to,
/// if any (e.g. the like-named `req` net/variable for plain `input req`).
/// A null node (domain SLANG_AST_SYMBOL) if `sym` is not a ModportPort
/// symbol, or the port has no direct internal connection (an
/// explicitly-connected port -- see
/// slang_symbol_modport_port_explicit_connection -- leaves this null). A
/// direct field read (slang::ast::ModportPortSymbol::internalSymbol) -- a
/// pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_modport_port_internal_symbol(slang_ast sym);

/// For a Modport symbol (an interface `modport m(...);` declaration): true
/// if it declares at least one `export` item. False if `sym` is not a
/// Modport symbol. A direct field read (slang::ast::ModportSymbol::
/// hasExports) -- a pure, allocation-free read. history: since 1.3.
SLANG_C_API bool slang_symbol_modport_has_exports(slang_ast sym);

/// For a MultiPort symbol (a port that externally appears as a single
/// connection but internally fans out to multiple names, e.g. `.p({a, b})`
/// in a port list): the most restrictive aggregated direction of data flow
/// across its constituent ports (see slang_symbol_multi_port_port_count /
/// slang_symbol_multi_port_port to inspect each one's own direction).
/// SLANG_ARGUMENT_DIRECTION_IN for any other symbol kind. A direct field
/// read (slang::ast::MultiPortSymbol::direction) -- a pure, allocation-free
/// read. history: since 1.3.
SLANG_C_API slang_argument_direction slang_symbol_multi_port_direction(slang_ast sym);

/// For a MultiPort symbol: always a null node (domain SLANG_AST_EXPRESSION)
/// -- multi-ports never have initializers
/// (slang::ast::MultiPortSymbol::getInitializer is a fixed placeholder
/// returning nullptr, kept only for parity with the single-port PortSymbol
/// interface so generic code can treat both uniformly). A pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_multi_port_initializer(slang_ast sym);

/// For a MultiPort symbol: its externally-visible type, as a node of
/// domain SLANG_AST_SYMBOL; a null node if `sym` is not a MultiPort
/// symbol. The underlying memo is forced by the freeze sweep, so this is a
/// pure read. Mirrors slang::ast::MultiPortSymbol::getType.
/// history: since 1.3.
SLANG_C_API slang_ast slang_symbol_multi_port_type(slang_ast sym);

/// For a MultiPort symbol: always false -- multi-ports are never null
/// ports (slang::ast::MultiPortSymbol::isNullPort is a fixed field kept
/// only for parity with the single-port PortSymbol interface). False for
/// any other symbol kind too. A pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API bool slang_symbol_multi_port_is_null_port(slang_ast sym);

/// The number of constituent single-ports that a MultiPort symbol fans out
/// to internally (e.g. 2 for `.p({a, b})`). 0 if `sym` is not a MultiPort
/// symbol. A direct field read (the span size of
/// slang::ast::MultiPortSymbol::ports) -- a pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API uint32_t slang_symbol_multi_port_port_count(slang_ast sym);

/// The `index`'th constituent Port symbol of a MultiPort symbol (in
/// declaration order, e.g. `a` then `b` for `.p({a, b})`), as a node of
/// domain SLANG_AST_SYMBOL. A null node if `sym` is not a MultiPort
/// symbol, or `index` is out of range (see
/// slang_symbol_multi_port_port_count). A direct field read (an element of
/// slang::ast::MultiPortSymbol::ports) -- a pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API slang_ast slang_symbol_multi_port_port(slang_ast sym, uint32_t index);

/// The number of net-reference expressions a NetAlias symbol (an `alias
/// lhs = rhs;` declaration) resolves to (typically two, one per side of
/// the alias, but a chained `alias a = b = c;` yields more). 0 if `sym` is
/// not a NetAlias symbol. The underlying memo is forced by the freeze
/// sweep, so this is a pure read. Mirrors the span size of
/// slang::ast::NetAliasSymbol::getNetReferences. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_net_alias_reference_count(slang_ast sym);

/// The `index`'th net-reference expression of a NetAlias symbol, as a node
/// of domain SLANG_AST_EXPRESSION. A null node if `sym` is not a NetAlias
/// symbol, or `index` is out of range (see
/// slang_symbol_net_alias_reference_count). Forced by the freeze sweep, so
/// this is a pure read. Mirrors an element of
/// slang::ast::NetAliasSymbol::getNetReferences. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_net_alias_reference(slang_ast sym, uint32_t index);

/* A NetSymbol represents a single net declaration (`wire`, `tri`, a
 * user-defined nettype, ...) -- slang::ast::NetSymbol. Read through
 * slang_ast (SLANG_AST_SYMBOL, SymbolKind::Net); every accessor below
 * returns its stated default for any other symbol kind. history: since
 * 1.3. */

/// The vectored/scalared expansion hint on a net declaration (`vectored`/
/// `scalared` before the net's range, e.g. `wire vectored [7:0] w;`), which
/// only affects how bit-selects of the net are treated for simulation
/// purposes. Mirrors the nested slang::ast::NetSymbol::ExpansionHint enum.
/// history: since 1.3.
typedef enum slang_expansion_hint {
    SLANG_EXPANSION_HINT_NONE = 0,
    SLANG_EXPANSION_HINT_VECTORED = 1,
    SLANG_EXPANSION_HINT_SCALARED = 2,
} slang_expansion_hint;

/// For a Net symbol: its expansion hint (see slang_expansion_hint).
/// SLANG_EXPANSION_HINT_NONE for any other symbol kind. A direct field
/// read (slang::ast::NetSymbol::expansionHint) -- a pure, allocation-free
/// read. history: since 1.3.
SLANG_C_API slang_expansion_hint slang_symbol_net_expansion_hint(slang_ast sym);

/// A `trireg` net's charge strength level (`small`/`medium`/`large`).
/// Mirrors slang::ast::ChargeStrength. SLANG_CHARGE_STRENGTH_NONE is not
/// one of slang's own enumerators: it is this API's encoding of an empty
/// std::optional<ChargeStrength> (see slang_symbol_net_charge_strength).
/// history: since 1.3.
typedef enum slang_charge_strength {
    SLANG_CHARGE_STRENGTH_SMALL = 0,
    SLANG_CHARGE_STRENGTH_MEDIUM = 1,
    SLANG_CHARGE_STRENGTH_LARGE = 2,
    SLANG_CHARGE_STRENGTH_NONE = 3,
} slang_charge_strength;

/// For a Net symbol: its explicit charge strength, if any (e.g. `small`
/// for `trireg small w;`; only meaningful for a `trireg` net).
/// SLANG_CHARGE_STRENGTH_NONE if `sym` is not a Net symbol, or it has no
/// charge strength specification. Recomputed from syntax on every call (no
/// arena allocation) -- a pure, allocation-free read. Mirrors
/// slang::ast::NetSymbol::getChargeStrength. history: since 1.3.
SLANG_C_API slang_charge_strength slang_symbol_net_charge_strength(slang_ast sym);

/// For a Net symbol: its delay control (the `#2` of `wire #2 w = a;`), as
/// a node of domain SLANG_AST_TIMING_CONTROL. A null node if `sym` is not
/// a Net symbol, or it has no delay. The underlying memo is forced by the
/// freeze sweep, so this is a pure read. Mirrors
/// slang::ast::NetSymbol::getDelay. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_net_delay(slang_ast sym);

/// For a Net symbol: its explicit drive strength, if any (e.g. the
/// `(strong0, pull1)` of `wire (strong0, pull1) w = a;`). Both
/// `has_strength0`/`has_strength1` are false if `sym` is not a Net symbol,
/// or it has no strength specification. Recomputed from syntax on every
/// call (no arena allocation) -- a pure, allocation-free read. Mirrors
/// slang::ast::NetSymbol::getDriveStrength. history: since 1.3.
SLANG_C_API slang_drive_strength_pair slang_symbol_net_drive_strength(slang_ast sym);

/// For a Net symbol: true if it was implicitly declared (e.g. the bare `w`
/// on the left of `assign w = a;` with no preceding `wire w;`, under
/// default-nettype rules). False if `sym` is not a Net symbol. A direct
/// field read (slang::ast::NetSymbol::isImplicit) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API bool slang_symbol_net_is_implicit(slang_ast sym);

/// For a Package symbol (a `package p; ... endpackage` construct): the
/// default lifetime (`automatic` or `static`) for variables it declares.
/// SLANG_VARIABLE_LIFETIME_AUTOMATIC if `sym` is not a Package symbol. A
/// direct field read (slang::ast::PackageSymbol::defaultLifetime) -- a
/// pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_variable_lifetime slang_symbol_package_default_lifetime(slang_ast sym);

/// Looks up `name` in a Package symbol, following slang::ast::PackageSymbol::
/// findForImport: first checks the package's own directly-declared members
/// (including its own explicit `import`s), then -- for a name only visible
/// through that package's `export` declarations -- validates those
/// declarations (idempotent; memoized the first time any lookup needs them)
/// and returns the exported symbol. Returns the found symbol as a node of
/// domain SLANG_AST_SYMBOL, or a null node if nothing was found, `sym` is not
/// a Package symbol, or `name` is empty. The export-resolution memo (slang::
/// ast::PackageSymbol's private ExportData::resolved) is forced for every
/// Package symbol by the freeze sweep, so despite mirroring a lazy getter
/// this is a pure read on a frozen design. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_package_find_for_import(slang_ast sym, const char* name,
                                                             size_t name_len);

/// True if a Package symbol has an `export *::*;` declaration (re-exporting
/// every package it imports from). False if `sym` is not a Package symbol. A
/// direct field read (slang::ast::PackageSymbol::hasExportAll), populated at
/// declaration -- a pure, allocation-free read. history: since 1.3.
SLANG_C_API bool slang_symbol_package_has_export_all(slang_ast sym);

/// A Package symbol's own explicit timescale (via a `` `timescale `` directive
/// preceding it, or a per-element override), written to `*out` and returning
/// true if it has one. False (leaving `*out` untouched) if `sym` is not a
/// Package symbol, or it has no explicit timescale. A direct field read
/// (slang::ast::PackageSymbol::timeScale) -- a pure, allocation-free read.
/// Mirrors slang_symbol_compilation_unit_time_scale / slang_definition_time_scale.
/// history: since 1.3.
SLANG_C_API bool slang_symbol_package_time_scale(slang_ast sym, slang_time_scale* out);

/* ------------------------------------------------------------------------- */
/* ParameterSymbolBase / ParameterSymbol                                     */
/* ------------------------------------------------------------------------- */

/// True if `sym` (a Parameter or TypeParameter symbol) is a `localparam`.
/// False for any other symbol kind, or a non-local parameter. A direct field
/// read (slang::ast::ParameterSymbolBase::isLocalParam) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API bool slang_symbol_parameter_is_local_param(slang_ast sym);

/// True if `sym` (a Parameter or TypeParameter symbol) was declared in a
/// module/interface/program/checker's parameter port list (`#(...)`). False
/// for any other symbol kind, or a parameter declared in the body. A direct
/// field read (slang::ast::ParameterSymbolBase::isPortParam) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API bool slang_symbol_parameter_is_port_param(slang_ast sym);

/// True if `sym` (a Parameter or TypeParameter symbol) was declared in the
/// body of its containing construct rather than its parameter port list --
/// the complement of slang_symbol_parameter_is_port_param. False for any
/// other symbol kind. Mirrors slang::ast::ParameterSymbolBase::isBodyParam
/// (`!isPortParam()`) -- a pure, allocation-free read. history: since 1.3.
SLANG_C_API bool slang_symbol_parameter_is_body_param(slang_ast sym);

/// True if a Parameter symbol's value was overridden from its default (by a
/// `#(...)` instantiation override, a `defparam`, or a config rule). False if
/// `sym` is not a Parameter symbol, or it kept its declared default. Reads a
/// flag on the underlying DeclaredType (slang::ast::ParameterSymbol::
/// isOverridden -> DeclaredTypeFlags::InitializerOverridden) resolved by the
/// same getType() the freeze sweep already forces for every symbol, so this
/// is a pure read. history: since 1.3.
SLANG_C_API bool slang_symbol_parameter_is_overridden(slang_ast sym);

/// True if a TypeParameter symbol's type was overridden from its default
/// (by a `#(...)` instantiation override or a config rule). False if `sym`
/// is not a TypeParameter symbol, or it kept its declared default type.
/// Reads a flag on the underlying DeclaredType
/// (slang::ast::TypeParameterSymbol::isOverridden ->
/// DeclaredTypeFlags::TypeOverridden) resolved by the same getType() the
/// freeze sweep already forces for every symbol with a declared type
/// (TypeParameterSymbol is one of its carriers), so this is a pure read.
/// history: since 1.3.
SLANG_C_API bool slang_symbol_type_parameter_is_overridden(slang_ast sym);

/* ------------------------------------------------------------------------- */
/* PortSymbol                                                                 */
/* ------------------------------------------------------------------------- */

/// The direction of data flowing across a Port symbol (`input`/`output`/
/// `inout`/`ref`). SLANG_ARGUMENT_DIRECTION_INOUT if `sym` is not a Port
/// symbol (matching slang::ast::PortSymbol::direction's own default). A
/// direct field read -- a pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_argument_direction slang_symbol_port_direction(slang_ast sym);

/// The source location where a Port symbol's external name is declared (the
/// ANSI port's own name, or the non-ANSI `.name` in the module header's port
/// list). A zero-valued (buffer-less) location if `sym` is not a Port
/// symbol. A direct field read (slang::ast::PortSymbol::externalLoc) -- a
/// pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_loc slang_symbol_port_external_loc(slang_ast sym);

/// A Port symbol's default-value initializer expression (e.g. the `= 0` of
/// an ANSI port `input logic a = 0`), as a node of domain
/// SLANG_AST_EXPRESSION. A null node if `sym` is not a Port symbol, or it has
/// no initializer. The underlying memo is forced by the freeze sweep, so this
/// is a pure read. Mirrors slang::ast::PortSymbol::getInitializer. history:
/// since 1.3.
SLANG_C_API slang_ast slang_symbol_port_initializer(slang_ast sym);

/// The expression, bound in the instance body's own scope, that a Port
/// symbol connects internally to (e.g. the reference to its internal net or
/// variable, or a concatenation for a multi-bit port built from several
/// internal signals). A null node if `sym` is not a Port symbol, or it has no
/// internal connection (a null port). The underlying memo is forced by the
/// freeze sweep, so this is a pure read. Mirrors slang::ast::PortSymbol::
/// getInternalExpr. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_port_internal_expr(slang_ast sym);

/// A Port symbol's resolved type, as a node of domain SLANG_AST_SYMBOL. A
/// null node if `sym` is not a Port symbol. The underlying memo is forced by
/// the freeze sweep, so this is a pure read. Mirrors slang::ast::PortSymbol::
/// getType. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_port_type(slang_ast sym);

/// The instance-internal symbol a Port symbol connects to (its internal net
/// or variable), as a node of domain SLANG_AST_SYMBOL. A null node if `sym`
/// is not a Port symbol, or it is a null port with nothing internal to
/// connect to. A direct field read (slang::ast::PortSymbol::internalSymbol)
/// -- a pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_port_internal_symbol(slang_ast sym);

/// True if a Port symbol was declared using ANSI port-list syntax (e.g.
/// `module m(input logic a);`), and false if it was declared using non-ANSI
/// syntax (a bare name in the port list plus a separate `input logic a;`
/// declaration in the body) -- or if `sym` is not a Port symbol. A direct
/// field read (slang::ast::PortSymbol::isAnsiPort) -- a pure, allocation-free
/// read. history: since 1.3.
SLANG_C_API bool slang_symbol_port_is_ansi_port(slang_ast sym);

/// True if a Port symbol's connection is (or resolves to) a net -- its
/// internal expression is a net-typed lvalue (a net reference, or a
/// concatenation of only net references), or, if it has no internal
/// expression, its internal symbol is itself a Net. False if `sym` is not a
/// Port symbol. Reads the same internal-expression memo already forced by
/// the freeze sweep (see slang_symbol_port_internal_expr), so this is a
/// pure read. Mirrors slang::ast::PortSymbol::isNetPort. history: since 1.3.
SLANG_C_API bool slang_symbol_port_is_net_port(slang_ast sym);

/// True if a Port symbol is a null port -- an empty, dot-less position in a
/// port list (e.g. the missing slot of `module m(a, , c);`) that does not
/// connect to anything internal to the instance -- and false if `sym` is
/// not a Port symbol. A direct field read
/// (slang::ast::PortSymbol::isNullPort) -- a pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API bool slang_symbol_port_is_null_port(slang_ast sym);

/* ------------------------------------------------------------------------- */
/* PrimitiveInstanceSymbol / PrimitivePortSymbol / PrimitiveSymbol           */
/* ------------------------------------------------------------------------- */

/// A user-defined primitive (UDP) port's declared direction (`input`,
/// `output`, an `output reg` of a sequential UDP, or the shared `inout`
/// terminal of a switch-level primitive). Mirrors
/// slang::ast::PrimitivePortDirection. history: since 1.3.
typedef enum slang_primitive_port_direction {
    SLANG_PRIMITIVE_PORT_DIRECTION_IN = 0,
    SLANG_PRIMITIVE_PORT_DIRECTION_OUT = 1,
    SLANG_PRIMITIVE_PORT_DIRECTION_OUT_REG = 2,
    SLANG_PRIMITIVE_PORT_DIRECTION_INOUT = 3,
} slang_primitive_port_direction;

/// For a PrimitivePort symbol (a port declaration inside a
/// `primitive`/`endprimitive` block): its declared direction.
/// SLANG_PRIMITIVE_PORT_DIRECTION_IN if `sym` is not a PrimitivePort symbol.
/// A direct field read (slang::ast::PrimitivePortSymbol::direction) -- a
/// pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_primitive_port_direction slang_symbol_primitive_port_direction(slang_ast sym);

/// The kind of gate primitive a Primitive symbol represents: a
/// user-defined primitive (`primitive`/`endprimitive`), a built-in gate with
/// a hard-coded, fixed-arity truth table (e.g. `not`, `bufif0`), a built-in
/// gate that accepts a variable number of inputs (`and`, `or`, `nand`, ...),
/// one that accepts a variable number of outputs (`buf`, `not`), or a
/// bidirectional switch (`tran`, `rtran`, `tranif0`, ...). Mirrors
/// slang::ast::PrimitiveSymbol::PrimitiveKind. history: since 1.3.
typedef enum slang_primitive_kind {
    SLANG_PRIMITIVE_KIND_USER_DEFINED = 0,
    SLANG_PRIMITIVE_KIND_FIXED = 1,
    SLANG_PRIMITIVE_KIND_N_INPUT = 2,
    SLANG_PRIMITIVE_KIND_N_OUTPUT = 3,
    SLANG_PRIMITIVE_KIND_BI_DI_SWITCH = 4,
} slang_primitive_kind;

/// For a Primitive symbol (a `primitive`/`endprimitive` declaration, or one
/// of the built-in gate primitives such as `and`/`not`/`bufif0`): which kind
/// of primitive it is. SLANG_PRIMITIVE_KIND_USER_DEFINED if `sym` is not a
/// Primitive symbol. A direct field read
/// (slang::ast::PrimitiveSymbol::primitiveKind) -- a pure, allocation-free
/// read. history: since 1.3.
SLANG_C_API slang_primitive_kind slang_symbol_primitive_kind(slang_ast sym);

/// True if a Primitive symbol is sequential -- it carries internal state
/// between evaluations, e.g. a UDP with a `reg` output, or a built-in latch
/// like `sr` -- and false if it is purely combinational, or if `sym` is not
/// a Primitive symbol. A direct field read
/// (slang::ast::PrimitiveSymbol::isSequential) -- a pure, allocation-free
/// read. history: since 1.3.
SLANG_C_API bool slang_symbol_primitive_is_sequential(slang_ast sym);

/// A Primitive symbol's initial-value expression for its (sequential-only)
/// state, already evaluated to a constant (e.g. the `initial out = 1'b0;`
/// of a sequential UDP body), as an owned slang_constant handle -- release
/// it with slang_constant_release. Null (with `*err` left at
/// SLANG_SUCCESS) if `sym` is not a Primitive symbol, or it has no
/// initial-value expression. A direct field read
/// (slang::ast::PrimitiveSymbol::initVal), already a resolved
/// ConstantValue* set once at construction time -- a pure, allocation-free
/// read of the frozen arena (the returned handle's own storage is a
/// separate, caller-owned C-heap allocation). history: since 1.3.
SLANG_C_API slang_constant slang_symbol_primitive_init_val(slang_ast sym, slang_error* err);

/// The number of ports a Primitive symbol declares, and each one as a
/// PrimitivePort symbol (node of domain SLANG_AST_SYMBOL), in declaration
/// order (output port(s) first, per UDP syntax rules). 0 / a null node if
/// `sym` is not a Primitive symbol or `index` is out of range. A direct
/// field read (slang::ast::PrimitiveSymbol::ports) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_primitive_port_count(slang_ast sym);
SLANG_C_API slang_ast slang_symbol_primitive_port(slang_ast sym, uint32_t index);

/// The number of rows in a Primitive symbol's truth table (its `table` ...
/// `endtable` body) -- see slang_symbol_primitive_table_entry_inputs /
/// _output / _state below for each row's fields. 0 if `sym` is not a
/// Primitive symbol. A direct field read
/// (slang::ast::PrimitiveSymbol::table) -- a pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API uint32_t slang_symbol_primitive_table_count(slang_ast sym);

/// The `index`'th row's input-level pattern of a Primitive symbol's truth
/// table -- one character per input port (`0`/`1`/`x`/`?`/`b`, ...), plus a
/// parenthesized two-character edge (e.g. `(01)`) in place of an
/// edge-sensitive input's single character. An empty string if `sym` is not
/// a Primitive symbol or `index` is out of range. Borrowed, valid for the
/// life of the owning Design/Compilation. A direct field read
/// (slang::ast::PrimitiveSymbol::TableEntry::inputs) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_str slang_symbol_primitive_table_entry_inputs(slang_ast sym, uint32_t index);

/// The `index`'th row's output-level symbol of a Primitive symbol's truth
/// table (e.g. "1", "0", "x", or "-" for "no change" in a sequential UDP),
/// as a single-character string. An empty string if `sym` is not a
/// Primitive symbol or `index` is out of range. Borrowed, valid for the
/// life of the owning Design/Compilation -- points directly at the row's
/// own `output` field, which is stable for as long as the frozen arena that
/// owns the table is. A direct field read
/// (slang::ast::PrimitiveSymbol::TableEntry::output) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_str slang_symbol_primitive_table_entry_output(slang_ast sym, uint32_t index);

/// The `index`'th row's current-state symbol of a Primitive symbol's truth
/// table (the sequential UDP's own prior-output column, e.g. "0", "1", or
/// "?"), as a single-character string. An empty string if `sym` is not a
/// Primitive symbol, `index` is out of range, or the row has no state
/// column (a combinational UDP's table rows have none). Borrowed, valid for
/// the life of the owning Design/Compilation -- points directly at the
/// row's own `state` field, stable for as long as the frozen arena that
/// owns the table is. A direct field read
/// (slang::ast::PrimitiveSymbol::TableEntry::state) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_str slang_symbol_primitive_table_entry_state(slang_ast sym, uint32_t index);

/// For a PrimitiveInstance symbol (an instantiation of a gate/UDP
/// primitive): the number of expressions bound to its port connections, and
/// each one as a node of domain SLANG_AST_EXPRESSION, in port-list order.
/// 0 / a null node if `sym` is not a PrimitiveInstance symbol or `index` is
/// out of range. The underlying memo is forced by the freeze sweep
/// (FreezeVisitor's PrimitiveInstanceSymbol branch), so this is a pure
/// read. Mirrors slang::ast::PrimitiveInstanceSymbol::getPortConnections.
/// history: since 1.3.
SLANG_C_API uint32_t slang_symbol_primitive_instance_port_connection_count(slang_ast sym);
SLANG_C_API slang_ast slang_symbol_primitive_instance_port_connection(slang_ast sym,
                                                                      uint32_t index);

/// For a PrimitiveInstance symbol: its explicit delay control (e.g. the
/// `#2` of `and #2 g1(y, a, b);`), as a node of domain
/// SLANG_AST_TIMING_CONTROL. A null node if `sym` is not a PrimitiveInstance
/// symbol, or it has no delay. The underlying memo is forced by the freeze
/// sweep (FreezeVisitor's PrimitiveInstanceSymbol branch), so this is a
/// pure read. Mirrors slang::ast::PrimitiveInstanceSymbol::getDelay.
/// history: since 1.3.
SLANG_C_API slang_ast slang_symbol_primitive_instance_delay(slang_ast sym);

/// For a PrimitiveInstance symbol: its explicit drive strength, if any
/// (e.g. the `(strong0, pull1)` of `and (strong0, pull1) g1(y, a, b);`).
/// Both `has_strength0`/`has_strength1` are false if `sym` is not a
/// PrimitiveInstance symbol, or it has no strength specification.
/// Recomputed from syntax on every call (no arena allocation) -- a pure,
/// allocation-free read. Mirrors
/// slang::ast::PrimitiveInstanceSymbol::getDriveStrength. history: since
/// 1.3.
SLANG_C_API slang_drive_strength_pair slang_symbol_primitive_instance_drive_strength(
    slang_ast sym);

/* ------------------------------------------------------------------------- */
/* ProceduralBlockSymbol                                                      */
/* ------------------------------------------------------------------------- */

/// The number of nested statement-block scopes a ProceduralBlock symbol's
/// body directly introduces (e.g. one per `begin : name ... end` or
/// `fork ... join` with its own declarations), and each one as a
/// StatementBlock symbol (node of domain SLANG_AST_SYMBOL). 0 / a null node
/// if `sym` is not a ProceduralBlock symbol or `index` is out of range. A
/// direct field read (slang::ast::ProceduralBlockSymbol::getBlocks) -- a
/// pure, allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_procedural_block_block_count(slang_ast sym);
SLANG_C_API slang_ast slang_symbol_procedural_block_block(slang_ast sym, uint32_t index);

/// The kind of procedural block a ProceduralBlock symbol is. Mirrors
/// slang::ast::ProceduralBlockKind. history: since 1.3.
typedef enum slang_procedural_block_kind {
    SLANG_PROCEDURAL_BLOCK_KIND_INITIAL = 0,
    SLANG_PROCEDURAL_BLOCK_KIND_FINAL = 1,
    SLANG_PROCEDURAL_BLOCK_KIND_ALWAYS = 2,
    SLANG_PROCEDURAL_BLOCK_KIND_ALWAYS_COMB = 3,
    SLANG_PROCEDURAL_BLOCK_KIND_ALWAYS_LATCH = 4,
    SLANG_PROCEDURAL_BLOCK_KIND_ALWAYS_FF = 5,
} slang_procedural_block_kind;

/// For a ProceduralBlock symbol: which kind of procedural block it is (e.g.
/// `initial`, `always_ff`). SLANG_PROCEDURAL_BLOCK_KIND_INITIAL if `sym` is
/// not a ProceduralBlock symbol. A direct field read
/// (slang::ast::ProceduralBlockSymbol::procedureKind) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_procedural_block_kind slang_symbol_procedural_block_procedure_kind(
    slang_ast sym);

/// True if a ProceduralBlock symbol is a "single driver" block -- an
/// `always_comb`, `always_latch`, or `always_ff` block, each restricted
/// (and analyzed) as the sole driver of every variable it assigns -- and
/// false for a plain `always`/`initial`/`final` block, or if `sym` is not a
/// ProceduralBlock symbol. A direct field read
/// (slang::ast::ProceduralBlockSymbol::procedureKind) -- a pure,
/// allocation-free read. Mirrors
/// slang::ast::ProceduralBlockSymbol::isSingleDriverBlock. history: since
/// 1.3.
SLANG_C_API bool slang_symbol_procedural_block_is_single_driver_block(slang_ast sym);

/* ------------------------------------------------------------------------- */
/* PropertySymbol                                                             */
/* ------------------------------------------------------------------------- */

/// The number of formal argument ports of a Property symbol (a `property`
/// assertion declaration). 0 if `sym` is not a Property symbol. A direct
/// field read (slang::ast::PropertySymbol::ports) -- a pure, allocation-free
/// read. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_property_port_count(slang_ast sym);

/// The `index`'th formal argument port of a Property symbol, as a node of
/// domain SLANG_AST_SYMBOL (an AssertionPort symbol). A null node if `sym`
/// is not a Property symbol or `index` is out of range. A direct field read
/// (an element of slang::ast::PropertySymbol::ports) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_property_port(slang_ast sym, uint32_t index);

/* ------------------------------------------------------------------------- */
/* TimingPathSymbol                                                           */
/* ------------------------------------------------------------------------- */

/// A specify-block timing path's connection kind: full (`*>`, every input
/// bit paths to every output bit) or parallel (`=>`, input/output must have
/// matching widths, paired bit-for-bit). Mirrors
/// slang::ast::TimingPathSymbol::ConnectionKind. history: since 1.3.
typedef enum slang_timing_path_connection_kind {
    SLANG_TIMING_PATH_CONNECTION_KIND_FULL = 0,
    SLANG_TIMING_PATH_CONNECTION_KIND_PARALLEL = 1,
} slang_timing_path_connection_kind;

/// A specify-block timing path's polarity: unknown (unspecified), positive
/// (`+`), or negative (`-`). Mirrors slang::ast::TimingPathSymbol::Polarity
/// -- used for both the overall path polarity and (separately) the
/// edge-sensitive-path polarity. history: since 1.3.
typedef enum slang_timing_path_polarity {
    SLANG_TIMING_PATH_POLARITY_UNKNOWN = 0,
    SLANG_TIMING_PATH_POLARITY_POSITIVE = 1,
    SLANG_TIMING_PATH_POLARITY_NEGATIVE = 2,
} slang_timing_path_polarity;

/// A TimingPath symbol's connection kind (`*>` vs `=>`).
/// SLANG_TIMING_PATH_CONNECTION_KIND_FULL if `sym` is not a TimingPath
/// symbol. A direct field read (slang::ast::TimingPathSymbol::
/// connectionKind), set once at construction from the path operator token
/// -- a pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_timing_path_connection_kind slang_symbol_timing_path_connection_kind(
    slang_ast sym);

/// A TimingPath symbol's overall polarity (the `+`/`-` on the path
/// operator itself, e.g. `(a *> b) = 1` vs `(a +*> b) = 1`).
/// SLANG_TIMING_PATH_POLARITY_UNKNOWN if `sym` is not a TimingPath symbol.
/// A direct field read (slang::ast::TimingPathSymbol::polarity), set once
/// at construction -- a pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_timing_path_polarity slang_symbol_timing_path_polarity(slang_ast sym);

/// A TimingPath symbol's edge-sensitive-path polarity (the `+`/`-` in an
/// edge-sensitive path suffix, e.g. the `+` of `(posedge a => (b +: c))
/// = 1`). SLANG_TIMING_PATH_POLARITY_UNKNOWN if `sym` is not a TimingPath
/// symbol, or its path has no edge-sensitive suffix. A direct field read
/// (slang::ast::TimingPathSymbol::edgePolarity), set once at construction
/// -- a pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_timing_path_polarity slang_symbol_timing_path_edge_polarity(slang_ast sym);

/// A TimingPath symbol's edge identifier (the `posedge`/`negedge`/`edge`
/// prefix on its source terminal, e.g. the `posedge` of `(posedge a => b)
/// = 1`). SLANG_EDGE_NONE if `sym` is not a TimingPath symbol, or its path
/// has no edge identifier. A direct field read
/// (slang::ast::TimingPathSymbol::edgeIdentifier), set once at construction
/// -- a pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_edge_kind slang_symbol_timing_path_edge_identifier(slang_ast sym);

/// True if a TimingPath symbol is state-dependent -- declared with `if
/// (cond)` or `ifnone`, as opposed to an unconditional path. False if `sym`
/// is not a TimingPath symbol, or its path is unconditional. A direct field
/// read (slang::ast::TimingPathSymbol::isStateDependent), set once at
/// construction (by the ConditionalPathDeclaration/IfNonePathDeclaration
/// fromSyntax overloads) -- a pure, allocation-free read. history: since
/// 1.3.
SLANG_C_API bool slang_symbol_timing_path_is_state_dependent(slang_ast sym);

/// The `if (cond)` condition expression of a state-dependent TimingPath
/// symbol, as a node of domain SLANG_AST_EXPRESSION. A null node if `sym`
/// is not a TimingPath symbol, its path is unconditional, or it is an
/// `ifnone` path (which has no condition expression of its own).
/// slang::ast::TimingPathSymbol::getConditionExpr lazily resolves the whole
/// path (source/dest terminals, condition, delays) together on first call
/// (a lazily-cached `conditionExpr` field et al., guarded by `isResolved`); the freeze
/// sweep forces that resolution pre-seal (FreezeVisitor's TimingPathSymbol
/// branch: `t.getInputs()`, which shares the same resolve()), so this is a
/// pure read on the frozen design. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_timing_path_condition_expr(slang_ast sym);

/// The edge-sensitive source expression of a TimingPath symbol (the
/// parenthesized destination-side expression of an edge-sensitive path,
/// e.g. the `c` of `(posedge a => (b +: c)) = 1`). A null node if `sym` is
/// not a TimingPath symbol, or its path has no edge-sensitive suffix.
/// Resolved (and forced by the freeze sweep) the same way as
/// slang_symbol_timing_path_condition_expr -- a pure read on the frozen
/// design. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_timing_path_edge_source_expr(slang_ast sym);

/// The number of input terminal expressions of a TimingPath symbol (the
/// left-hand side of its path operator, e.g. 1 for `(a *> b) = 1`). 0 if
/// `sym` is not a TimingPath symbol. Resolved (and forced by the freeze
/// sweep) the same way as slang_symbol_timing_path_condition_expr -- a
/// pure read on the frozen design. Mirrors the span size of
/// slang::ast::TimingPathSymbol::getInputs. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_timing_path_input_count(slang_ast sym);

/// The `index`'th input terminal expression of a TimingPath symbol (see
/// slang_symbol_timing_path_input_count), as a node of domain
/// SLANG_AST_EXPRESSION. A null node if `sym` is not a TimingPath symbol,
/// or `index` is out of range. Forced the same way as
/// slang_symbol_timing_path_input_count, so this is a pure read on the
/// frozen design. Mirrors an element of
/// slang::ast::TimingPathSymbol::getInputs. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_timing_path_input(slang_ast sym, uint32_t index);

/// The number of output terminal expressions of a TimingPath symbol (the
/// right-hand side of its path operator, e.g. 1 for `(a *> b) = 1`). 0 if
/// `sym` is not a TimingPath symbol. Forced the same way as
/// slang_symbol_timing_path_input_count, so this is a pure read on the
/// frozen design. Mirrors the span size of
/// slang::ast::TimingPathSymbol::getOutputs. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_timing_path_output_count(slang_ast sym);

/// The `index`'th output terminal expression of a TimingPath symbol (see
/// slang_symbol_timing_path_output_count), as a node of domain
/// SLANG_AST_EXPRESSION. A null node if `sym` is not a TimingPath symbol,
/// or `index` is out of range. Forced the same way as
/// slang_symbol_timing_path_input_count, so this is a pure read on the
/// frozen design. Mirrors an element of
/// slang::ast::TimingPathSymbol::getOutputs. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_timing_path_output(slang_ast sym, uint32_t index);

/// The number of delay-value expressions of a TimingPath symbol (the
/// right-hand side of its `=`, e.g. 1 for `(a *> b) = 1` or 3 for `(a *> b)
/// = (1, 2, 3)`). 0 if `sym` is not a TimingPath symbol. Forced the same
/// way as slang_symbol_timing_path_input_count, so this is a pure read on
/// the frozen design. Mirrors the span size of
/// slang::ast::TimingPathSymbol::getDelays. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_timing_path_delay_count(slang_ast sym);

/// The `index`'th delay-value expression of a TimingPath symbol (see
/// slang_symbol_timing_path_delay_count), as a node of domain
/// SLANG_AST_EXPRESSION. A null node if `sym` is not a TimingPath symbol,
/// or `index` is out of range. Forced the same way as
/// slang_symbol_timing_path_input_count, so this is a pure read on the
/// frozen design. Mirrors an element of
/// slang::ast::TimingPathSymbol::getDelays. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_timing_path_delay(slang_ast sym, uint32_t index);

/* ------------------------------------------------------------------------- */
/* PulseStyleSymbol                                                           */
/* ------------------------------------------------------------------------- */

/// The kind of a PulseStyle symbol (a `pulsestyle_onevent`/
/// `pulsestyle_ondetect`/`showcancelled`/`noshowcancelled` specify-block
/// declaration). Mirrors slang::ast::PulseStyleKind. history: since 1.3.
typedef enum slang_pulse_style_kind {
    SLANG_PULSE_STYLE_KIND_ON_EVENT = 0,
    SLANG_PULSE_STYLE_KIND_ON_DETECT = 1,
    SLANG_PULSE_STYLE_KIND_SHOW_CANCELLED = 2,
    SLANG_PULSE_STYLE_KIND_NO_SHOW_CANCELLED = 3,
} slang_pulse_style_kind;

/// For a PulseStyle symbol: which of the four pulse-style declarations it
/// is. SLANG_PULSE_STYLE_KIND_ON_EVENT if `sym` is not a PulseStyle symbol.
/// A direct field read (slang::ast::PulseStyleSymbol::pulseStyleKind) -- a
/// pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_pulse_style_kind slang_symbol_pulse_style_kind(slang_ast sym);

/// The number of terminal expressions a PulseStyle symbol applies to (the
/// output/inout ports of the module it appears in, e.g. 2 for `pulsestyle_
/// onevent a, b;`). 0 if `sym` is not a PulseStyle symbol. The underlying
/// memo is forced by the freeze sweep, so this is a pure read. Mirrors the
/// span size of slang::ast::PulseStyleSymbol::getTerminals. history: since
/// 1.3.
SLANG_C_API uint32_t slang_symbol_pulse_style_terminal_count(slang_ast sym);

/// The `index`'th terminal expression of a PulseStyle symbol, as a node of
/// domain SLANG_AST_EXPRESSION. A null node if `sym` is not a PulseStyle
/// symbol, or `index` is out of range (see
/// slang_symbol_pulse_style_terminal_count). Forced by the freeze sweep, so
/// this is a pure read. Mirrors an element of
/// slang::ast::PulseStyleSymbol::getTerminals. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_pulse_style_terminal(slang_ast sym, uint32_t index);

/* ------------------------------------------------------------------------- */
/* SystemTimingCheckSymbol                                                    */
/* ------------------------------------------------------------------------- */

/// The kind of a `$setup`/`$hold`/`$setuphold`/`$recovery`/`$removal`/
/// `$recrem`/`$skew`/`$timeskew`/`$fullskew`/`$period`/`$width`/`$nochange`
/// system timing check. Mirrors slang::ast::SystemTimingCheckKind.
/// history: since 1.3.
typedef enum slang_system_timing_check_kind {
    SLANG_SYSTEM_TIMING_CHECK_KIND_UNKNOWN = 0,
    SLANG_SYSTEM_TIMING_CHECK_KIND_SETUP = 1,
    SLANG_SYSTEM_TIMING_CHECK_KIND_HOLD = 2,
    SLANG_SYSTEM_TIMING_CHECK_KIND_SETUP_HOLD = 3,
    SLANG_SYSTEM_TIMING_CHECK_KIND_RECOVERY = 4,
    SLANG_SYSTEM_TIMING_CHECK_KIND_REMOVAL = 5,
    SLANG_SYSTEM_TIMING_CHECK_KIND_REC_REM = 6,
    SLANG_SYSTEM_TIMING_CHECK_KIND_SKEW = 7,
    SLANG_SYSTEM_TIMING_CHECK_KIND_TIME_SKEW = 8,
    SLANG_SYSTEM_TIMING_CHECK_KIND_FULL_SKEW = 9,
    SLANG_SYSTEM_TIMING_CHECK_KIND_PERIOD = 10,
    SLANG_SYSTEM_TIMING_CHECK_KIND_WIDTH = 11,
    SLANG_SYSTEM_TIMING_CHECK_KIND_NO_CHANGE = 12,
} slang_system_timing_check_kind;

/// For a SystemTimingCheck symbol (a `$setup(...)`-style specify-block
/// system timing check): which check it is. SLANG_SYSTEM_TIMING_CHECK_KIND_
/// UNKNOWN if `sym` is not a SystemTimingCheck symbol. A direct field read
/// (slang::ast::SystemTimingCheckSymbol::timingCheckKind) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_system_timing_check_kind slang_symbol_system_timing_check_kind(slang_ast sym);

/// The number of arguments of a SystemTimingCheck symbol, including any
/// elided (empty) trailing/optional ones -- e.g. 4 for `$setup(a, posedge b,
/// 10, notifier)`. 0 if `sym` is not a SystemTimingCheck symbol.
/// slang::ast::SystemTimingCheckSymbol::getArguments() lazily binds and
/// caches every argument expression on first call (a once-computed, cached
/// `args` span); the freeze sweep force-resolves it for every
/// SystemTimingCheck symbol, so this is a pure read on a frozen design.
/// history: since 1.3.
SLANG_C_API uint32_t slang_symbol_system_timing_check_argument_count(slang_ast sym);

/// The `index`'th argument's main expression (slang::ast::
/// SystemTimingCheckSymbol::Arg::expr) -- the signal/event, limit value, or
/// notifier reference, depending on the argument's position -- as a node of
/// domain SLANG_AST_EXPRESSION. A null node if `sym` is not a
/// SystemTimingCheck symbol, `index` is out of range (see
/// slang_symbol_system_timing_check_argument_count), or the argument was
/// elided. Forced the same way as slang_symbol_system_timing_check_argument_
/// count, so this is a pure read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_system_timing_check_argument_expr(slang_ast sym,
                                                                     uint32_t index);

/// The `index`'th argument's `&&&`-qualified condition expression (slang::
/// ast::SystemTimingCheckSymbol::Arg::condition), as a node of domain
/// SLANG_AST_EXPRESSION. A null node if `sym` is not a SystemTimingCheck
/// symbol, `index` is out of range, or the argument has no condition (only
/// an event argument -- e.g. `posedge clk &&& en` -- ever carries one).
/// Forced the same way as slang_symbol_system_timing_check_argument_count,
/// so this is a pure read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_system_timing_check_argument_condition(slang_ast sym,
                                                                          uint32_t index);

/// The `index`'th argument's sampling edge (slang::ast::
/// SystemTimingCheckSymbol::Arg::edge), e.g. SLANG_EDGE_POSEDGE for `posedge
/// clk`. SLANG_EDGE_NONE if `sym` is not a SystemTimingCheck symbol, `index`
/// is out of range, or the argument declares no edge (not an event argument,
/// or an unqualified event like plain `clk`). Forced the same way as
/// slang_symbol_system_timing_check_argument_count, so this is a pure read.
/// history: since 1.3.
SLANG_C_API slang_edge_kind slang_symbol_system_timing_check_argument_edge(slang_ast sym,
                                                                           uint32_t index);

/// The number of edge descriptors on the `index`'th argument (slang::ast::
/// SystemTimingCheckSymbol::Arg::edgeDescriptors), e.g. 2 for `edge [01,
/// z1] clk`. 0 if `sym` is not a SystemTimingCheck symbol, `index` is out of
/// range, or the argument has no edge-descriptor list. Forced the same way
/// as slang_symbol_system_timing_check_argument_count, so this is a pure
/// read. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_system_timing_check_argument_edge_descriptor_count(
    slang_ast sym, uint32_t arg_index);

/// The `desc_index`'th edge descriptor of the `index`'th argument (see
/// slang_symbol_system_timing_check_argument_edge_descriptor_count), as a
/// 2-character string (e.g. "01", "z1") -- slang::ast::
/// SystemTimingCheckSymbol::EdgeDescriptor is a `std::array<char, 2>`.
/// Empty if `sym` is not a SystemTimingCheck symbol, or either index is out
/// of range. Borrowed from the compilation's arena (the span the freeze
/// sweep force-resolves alongside the rest of this argument). history:
/// since 1.3.
SLANG_C_API slang_str slang_symbol_system_timing_check_argument_edge_descriptor(
    slang_ast sym, uint32_t arg_index, uint32_t desc_index);

/* ------------------------------------------------------------------------- */
/* RandSeqProductionSymbol                                                    */
/* ------------------------------------------------------------------------- */

/// The kind of a randsequence production's rule-tree "prod" node. Mirrors
/// the nested slang::ast::RandSeqProductionSymbol::ProdKind enum.
/// history: since 1.3.
typedef enum slang_randseq_prod_kind {
    /// A plain production reference (`prodName(args)`); read as a ProdItem
    /// via slang_randseq_prod_item_target / slang_randseq_prod_item_arg*.
    SLANG_RANDSEQ_PROD_KIND_ITEM = 0,
    /// An inline `{ ... }` statement block; read via
    /// slang_randseq_prod_code_block_block.
    SLANG_RANDSEQ_PROD_KIND_CODE_BLOCK = 1,
    /// An `if (expr) item [else item]`; read via slang_randseq_prod_if_else_*.
    SLANG_RANDSEQ_PROD_KIND_IF_ELSE = 2,
    /// A `repeat (expr) item`; read via slang_randseq_prod_repeat_*.
    SLANG_RANDSEQ_PROD_KIND_REPEAT = 3,
    /// A `case (expr) ... endcase`; read via slang_randseq_prod_case_*.
    SLANG_RANDSEQ_PROD_KIND_CASE = 4,
} slang_randseq_prod_kind;

/// True for the null randseq-prod value returned when something is absent
/// (e.g. an out-of-range index, or a wrong-kind accessor). history: since
/// 1.3.
SLANG_C_API bool slang_randseq_prod_is_null(slang_randseq_prod prod);

/// The number of rules (`|`-separated alternatives) a RandSeqProduction
/// symbol (a `randsequence` production) has. 0 if `sym` is not a
/// RandSeqProduction symbol. The underlying rule tree is built and cached
/// lazily on first read (slang::ast::RandSeqProductionSymbol::getRules);
/// forced by the freeze sweep, so this is a pure read. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_randseq_rule_count(slang_ast sym);

/// The number of prod elements in the `rule_index`'th rule of a
/// RandSeqProduction symbol's rule tree (slang::ast::RandSeqProductionSymbol
/// ::Rule::prods). 0 if `sym` is not a RandSeqProduction symbol, or
/// `rule_index` is out of range. Forced by the freeze sweep, so this is a
/// pure read. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_randseq_rule_prod_count(slang_ast sym, uint32_t rule_index);

/// The `prod_index`'th prod element of the `rule_index`'th rule of a
/// RandSeqProduction symbol's rule tree, as a slang_randseq_prod (see
/// slang_randseq_prod_is_null for the null-prod / out-of-range case).
/// Forced by the freeze sweep, so this is a pure read. Mirrors an element of
/// slang::ast::RandSeqProductionSymbol::Rule::prods. history: since 1.3.
SLANG_C_API slang_randseq_prod slang_symbol_randseq_rule_prod(slang_ast sym, uint32_t rule_index,
                                                               uint32_t prod_index);

/// The kind of a randseq prod node (see slang_randseq_prod_kind).
/// SLANG_RANDSEQ_PROD_KIND_ITEM for a null prod. A direct field read
/// (slang::ast::RandSeqProductionSymbol::ProdBase::kind) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_randseq_prod_kind slang_randseq_prod_get_kind(slang_randseq_prod prod);

/// For an Item-kind randseq prod (a ProdItem): the production it invokes,
/// as a node of domain SLANG_AST_SYMBOL (a RandSeqProduction symbol). A null
/// node if `prod` is not Item-kind. A direct field read
/// (slang::ast::RandSeqProductionSymbol::ProdItem::target) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_randseq_prod_item_target(slang_randseq_prod prod);

/// The number of arguments passed at an Item-kind randseq prod's call site
/// (a ProdItem, e.g. 1 for `add("foo")`). 0 if `prod` is not Item-kind. A
/// direct field read (slang::ast::RandSeqProductionSymbol::ProdItem::args)
/// -- a pure, allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_randseq_prod_item_arg_count(slang_randseq_prod prod);

/// The `index`'th argument expression at an Item-kind randseq prod's call
/// site, as a node of domain SLANG_AST_EXPRESSION. A null node if `prod` is
/// not Item-kind, or `index` is out of range (see
/// slang_randseq_prod_item_arg_count). A direct field read (an element of
/// slang::ast::RandSeqProductionSymbol::ProdItem::args) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_randseq_prod_item_arg(slang_randseq_prod prod, uint32_t index);

/// For a CodeBlock-kind randseq prod (a CodeBlockProd, an inline `{ ... }`):
/// its statement block, as a node of domain SLANG_AST_SYMBOL (a
/// StatementBlock symbol). A null node if `prod` is not CodeBlock-kind. A
/// direct field read
/// (slang::ast::RandSeqProductionSymbol::CodeBlockProd::block) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_randseq_prod_code_block_block(slang_randseq_prod prod);

/// For an IfElse-kind randseq prod (an IfElseProd): its condition
/// expression, as a node of domain SLANG_AST_EXPRESSION. A null node if
/// `prod` is not IfElse-kind. A direct field read
/// (slang::ast::RandSeqProductionSymbol::IfElseProd::expr) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_randseq_prod_if_else_expr(slang_randseq_prod prod);

/// For an IfElse-kind randseq prod: its "if true" production item, as an
/// Item-kind slang_randseq_prod (never null when `prod` is itself
/// IfElse-kind). A null prod (see slang_randseq_prod_is_null) if `prod` is
/// not IfElse-kind. A direct field read
/// (slang::ast::RandSeqProductionSymbol::IfElseProd::ifItem) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_randseq_prod slang_randseq_prod_if_else_if_item(slang_randseq_prod prod);

/// True if an IfElse-kind randseq prod has an `else` clause. False if
/// `prod` is not IfElse-kind, or it has no `else`. A direct field read
/// (slang::ast::RandSeqProductionSymbol::IfElseProd::elseItem.has_value())
/// -- a pure, allocation-free read. history: since 1.3.
SLANG_C_API bool slang_randseq_prod_if_else_has_else_item(slang_randseq_prod prod);

/// For an IfElse-kind randseq prod: its `else` production item, as an
/// Item-kind slang_randseq_prod. A null prod (see
/// slang_randseq_prod_is_null) if `prod` is not IfElse-kind, or it has no
/// `else` clause (see slang_randseq_prod_if_else_has_else_item). A direct
/// field read
/// (slang::ast::RandSeqProductionSymbol::IfElseProd::elseItem) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_randseq_prod slang_randseq_prod_if_else_else_item(slang_randseq_prod prod);

/// For a Repeat-kind randseq prod (a RepeatProd, `repeat (expr) item`): its
/// repeat-count expression, as a node of domain SLANG_AST_EXPRESSION. A null
/// node if `prod` is not Repeat-kind. A direct field read
/// (slang::ast::RandSeqProductionSymbol::RepeatProd::expr) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_randseq_prod_repeat_expr(slang_randseq_prod prod);

/// For a Repeat-kind randseq prod: the production item it repeats, as an
/// Item-kind slang_randseq_prod (never null when `prod` is itself
/// Repeat-kind). A null prod (see slang_randseq_prod_is_null) if `prod` is
/// not Repeat-kind. A direct field read
/// (slang::ast::RandSeqProductionSymbol::RepeatProd::item) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_randseq_prod slang_randseq_prod_repeat_item(slang_randseq_prod prod);

/// For a Case-kind randseq prod (a CaseProd): its case-selector expression,
/// as a node of domain SLANG_AST_EXPRESSION. A null node if `prod` is not
/// Case-kind. A direct field read
/// (slang::ast::RandSeqProductionSymbol::CaseProd::expr) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_randseq_prod_case_expr(slang_randseq_prod prod);

/// The number of non-default case items of a Case-kind randseq prod
/// (slang::ast::RandSeqProductionSymbol::CaseProd::items). 0 if `prod` is
/// not Case-kind. A direct field read -- a pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API uint32_t slang_randseq_prod_case_item_count(slang_randseq_prod prod);

/// The number of label expressions of the `item_index`'th case item of a
/// Case-kind randseq prod (e.g. 2 for `1, 2: push;`). 0 if `prod` is not
/// Case-kind, or `item_index` is out of range. A direct field read
/// (slang::ast::RandSeqProductionSymbol::CaseItem::expressions) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_randseq_prod_case_item_expression_count(slang_randseq_prod prod,
                                                                    uint32_t item_index);

/// The `expr_index`'th label expression of the `item_index`'th case item of
/// a Case-kind randseq prod, as a node of domain SLANG_AST_EXPRESSION. A
/// null node if `prod` is not Case-kind, or either index is out of range.
/// A direct field read (an element of
/// slang::ast::RandSeqProductionSymbol::CaseItem::expressions) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_randseq_prod_case_item_expression(slang_randseq_prod prod,
                                                               uint32_t item_index,
                                                               uint32_t expr_index);

/// The production item of the `item_index`'th case item of a Case-kind
/// randseq prod, as an Item-kind slang_randseq_prod. A null prod (see
/// slang_randseq_prod_is_null) if `prod` is not Case-kind, or `item_index`
/// is out of range. A direct field read
/// (slang::ast::RandSeqProductionSymbol::CaseItem::item) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_randseq_prod slang_randseq_prod_case_item_item(slang_randseq_prod prod,
                                                                  uint32_t item_index);

/// True if a Case-kind randseq prod has a `default` item. False if `prod`
/// is not Case-kind, or it has no `default`. A direct field read
/// (slang::ast::RandSeqProductionSymbol::CaseProd::defaultItem.has_value())
/// -- a pure, allocation-free read. history: since 1.3.
SLANG_C_API bool slang_randseq_prod_case_has_default_item(slang_randseq_prod prod);

/// The `default` production item of a Case-kind randseq prod, as an
/// Item-kind slang_randseq_prod. A null prod (see
/// slang_randseq_prod_is_null) if `prod` is not Case-kind, or it has no
/// `default` item (see slang_randseq_prod_case_has_default_item). A direct
/// field read
/// (slang::ast::RandSeqProductionSymbol::CaseProd::defaultItem) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_randseq_prod slang_randseq_prod_case_default_item(slang_randseq_prod prod);

/* slang::ast::RandSeqProductionSymbol::Rule accessors: each takes the same
 * (sym, rule_index) pair as slang_symbol_randseq_rule_prod_count /
 * slang_symbol_randseq_rule_prod above, addressing the `rule_index`'th rule
 * of RandSeqProduction symbol `sym`'s rule tree. history: since 1.3. */

/// `ruleBlock`, the implicit statement block a randsequence rule's local
/// rule variables and inline code are attached to, as a node of domain
/// SLANG_AST_SYMBOL (a StatementBlock symbol). Never null for an in-range
/// rule (slang::ast::RandSeqProductionSymbol::Rule::ruleBlock is a
/// not_null). A null node if `sym` is not a RandSeqProduction symbol or
/// `rule_index` is out of range. Forced by the freeze sweep (part of
/// getRules()), so this is a pure read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_randseq_rule_block(slang_ast sym, uint32_t rule_index);

/// A rule's optional `:=` weight expression, as a node of domain
/// SLANG_AST_EXPRESSION. A null node if the rule has no weight expression,
/// `sym` is not a RandSeqProduction symbol, or `rule_index` is out of range.
/// Mirrors slang::ast::RandSeqProductionSymbol::Rule::weightExpr. Forced by
/// the freeze sweep, so this is a pure read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_randseq_rule_weight_expr(slang_ast sym, uint32_t rule_index);

/// True if a rule's prod list is a `rand join` (optionally `rand join
/// (expr)`) group rather than a plain sequence. False if `sym` is not a
/// RandSeqProduction symbol or `rule_index` is out of range. Mirrors
/// slang::ast::RandSeqProductionSymbol::Rule::isRandJoin. Forced by the
/// freeze sweep, so this is a pure read. history: since 1.3.
SLANG_C_API bool slang_symbol_randseq_rule_is_rand_join(slang_ast sym, uint32_t rule_index);

/// A `rand join`-kind rule's optional join-weight expression (the `expr` in
/// `rand join (expr)`), as a node of domain SLANG_AST_EXPRESSION. A null
/// node if the rule has no such expression (including when
/// slang_symbol_randseq_rule_is_rand_join is false), `sym` is not a
/// RandSeqProduction symbol, or `rule_index` is out of range. Mirrors
/// slang::ast::RandSeqProductionSymbol::Rule::randJoinExpr. Forced by the
/// freeze sweep, so this is a pure read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_randseq_rule_rand_join_expr(slang_ast sym,
                                                                uint32_t rule_index);

/// True if a rule has a trailing inline `{ ... }` code block
/// (slang::ast::RandSeqProductionSymbol::Rule::codeBlock.has_value()). False
/// if `sym` is not a RandSeqProduction symbol or `rule_index` is out of
/// range. Forced by the freeze sweep, so this is a pure read.
/// history: since 1.3.
SLANG_C_API bool slang_symbol_randseq_rule_has_code_block(slang_ast sym, uint32_t rule_index);

/// A rule's trailing inline `{ ... }` code block, as a CodeBlock-kind
/// slang_randseq_prod (read its statement block via
/// slang_randseq_prod_code_block_block). A null prod (see
/// slang_randseq_prod_is_null) if the rule has no code block (see
/// slang_symbol_randseq_rule_has_code_block), `sym` is not a
/// RandSeqProduction symbol, or `rule_index` is out of range. Mirrors
/// slang::ast::RandSeqProductionSymbol::Rule::codeBlock. Forced by the
/// freeze sweep, so this is a pure read. history: since 1.3.
SLANG_C_API slang_randseq_prod slang_symbol_randseq_rule_code_block(slang_ast sym,
                                                                     uint32_t rule_index);

/// The number of formal arguments a RandSeqProduction symbol (a
/// `randsequence` production) declares, e.g. 2 for
/// `production p(int a, string b);`. 0 if `sym` is not a RandSeqProduction
/// symbol. slang::ast::RandSeqProductionSymbol::arguments is set once during
/// elaboration, before this symbol is ever reachable through a frozen
/// &Design, so this is a pure, allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_randseq_argument_count(slang_ast sym);

/// The `index`'th formal argument of a RandSeqProduction symbol (see
/// slang_symbol_randseq_argument_count), as a node of domain SLANG_AST_SYMBOL
/// (SymbolKind::FormalArgument). A null node if `sym` is not a
/// RandSeqProduction symbol or `index` is out of range. A pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_randseq_argument(slang_ast sym, uint32_t index);

/// The resolved return type of a RandSeqProduction symbol
/// (slang::ast::RandSeqProductionSymbol::getReturnType, `void` for a
/// production with no declared return type), as a node of domain
/// SLANG_AST_SYMBOL. A null node if `sym` is not a RandSeqProduction symbol.
/// The underlying slang::ast::DeclaredType memo (declaredReturnType) is
/// forced by the freeze sweep's generic per-symbol getDeclaredType() pass
/// (RandSeqProductionSymbol is one of its carriers), so this is a pure read.
/// Equivalent to slang_declared_type_type on the same symbol, but typed to
/// this symbol kind. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_randseq_return_type(slang_ast sym);

/* ------------------------------------------------------------------------- */
/* SequenceSymbol                                                            */
/* ------------------------------------------------------------------------- */

/// The number of formal argument ports of a Sequence symbol (a `sequence`
/// assertion declaration). 0 if `sym` is not a Sequence symbol. A direct
/// field read (slang::ast::SequenceSymbol::ports) -- a pure, allocation-free
/// read. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_sequence_port_count(slang_ast sym);

/// The `index`'th formal argument port of a Sequence symbol, as a node of
/// domain SLANG_AST_SYMBOL (an AssertionPort symbol). A null node if `sym`
/// is not a Sequence symbol or `index` is out of range. A direct field read
/// (an element of slang::ast::SequenceSymbol::ports) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_sequence_port(slang_ast sym, uint32_t index);

/* ------------------------------------------------------------------------- */
/* SpecparamSymbol                                                           */
/* ------------------------------------------------------------------------- */

/// True if a Specparam symbol is a `PATHPULSE$...`-named specparam (slang
/// treats these specially, as specify-block pulse-control declarations,
/// rather than an ordinary specparam value). False if `sym` is not a
/// Specparam symbol. A direct field read
/// (slang::ast::SpecparamSymbol::isPathPulse) -- a pure, allocation-free
/// read. history: since 1.3.
SLANG_C_API bool slang_symbol_specparam_is_path_pulse(slang_ast sym);

/// The source terminal of a `PATHPULSE$source$dest` specparam
/// (slang::ast::SpecparamSymbol::getPathSource), as a node of domain
/// SLANG_AST_SYMBOL (a value symbol). A null node if `sym` is not a
/// Specparam symbol, is not a path-pulse specparam (see
/// slang_symbol_specparam_is_path_pulse), or its source terminal name failed
/// to resolve. The source/dest pair is lazily resolved together and cached
/// on first read (the private `pathSource`/`pathDest` fields); the freeze
/// sweep forces both for every Specparam symbol, so this is a pure read on
/// a frozen design. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_specparam_path_source(slang_ast sym);

/// The destination terminal of a `PATHPULSE$source$dest` specparam
/// (slang::ast::SpecparamSymbol::getPathDest), as a node of domain
/// SLANG_AST_SYMBOL (a value symbol). A null node if `sym` is not a
/// Specparam symbol, is not a path-pulse specparam (see
/// slang_symbol_specparam_is_path_pulse), or its destination terminal name
/// failed to resolve. Forced by the freeze sweep alongside
/// slang_symbol_specparam_path_source, so this is a pure read on a frozen
/// design. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_specparam_path_dest(slang_ast sym);

/* ------------------------------------------------------------------------- */
/* StatementBlockSymbol                                                      */
/* ------------------------------------------------------------------------- */

/// The kind of a StatementBlock symbol (`begin/end` vs `fork`/`join`/
/// `join_any`/`join_none`), as its StatementBlockKind enum ordinal (compare
/// slang_stmt_block_kind, which reads the same enum off a Block *statement*
/// rather than the symbol it declares). 0 (Sequential) if `sym` is not a
/// StatementBlock symbol. A direct field read
/// (slang::ast::StatementBlockSymbol::blockKind) -- a pure, allocation-free
/// read. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_statement_block_kind(slang_ast sym);

/// The default lifetime (`automatic` or `static`) for variables declared
/// directly within a StatementBlock symbol that don't specify their own.
/// SLANG_VARIABLE_LIFETIME_AUTOMATIC if `sym` is not a StatementBlock
/// symbol. A direct field read
/// (slang::ast::StatementBlockSymbol::defaultLifetime) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_variable_lifetime slang_symbol_statement_block_default_lifetime(slang_ast sym);

/* ------------------------------------------------------------------------- */
/* SubroutineSymbol                                                          */
/* ------------------------------------------------------------------------- */

/// The default lifetime (`automatic` or `static`) for local variables of a
/// Subroutine symbol (a `function`/`task` declaration) that don't specify
/// their own. SLANG_VARIABLE_LIFETIME_AUTOMATIC if `sym` is not a
/// Subroutine symbol. A direct field read
/// (slang::ast::SubroutineSymbol::defaultLifetime) -- a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_variable_lifetime slang_symbol_subroutine_default_lifetime(slang_ast sym);

/// A Subroutine symbol's flags (see slang_method_flag). 0
/// (SLANG_METHOD_NONE) if `sym` is not a Subroutine symbol. A direct field
/// read (slang::ast::SubroutineSymbol::flags) -- a pure, allocation-free
/// read. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_subroutine_flags(slang_ast sym);

/// The number of formal arguments of a Subroutine symbol (a `function`/
/// `task` declaration, or a DPI import). 0 if `sym` is not a Subroutine
/// symbol. slang::ast::SubroutineSymbol::getArguments() forces the
/// subroutine's own scope to elaborate on first call (a lazily-populated
/// `arguments` span); the freeze sweep's generic per-symbol traversal
/// already elaborates every visited scope, including a Subroutine symbol's
/// own (it is itself a Scope), so this is a pure read on a frozen design.
/// history: since 1.3.
SLANG_C_API uint32_t slang_symbol_subroutine_argument_count(slang_ast sym);

/// The `index`'th formal argument of a Subroutine symbol (see
/// slang_symbol_subroutine_argument_count), as a node of domain
/// SLANG_AST_SYMBOL (SymbolKind::FormalArgument). A null node if `sym` is
/// not a Subroutine symbol or `index` is out of range. Forced the same way
/// as slang_symbol_subroutine_argument_count, so this is a pure read on a
/// frozen design. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_subroutine_argument(slang_ast sym, uint32_t index);

/// The method a Subroutine symbol overrides (resolved via `virtual`/class
/// inheritance during elaboration), as a node of domain SLANG_AST_SYMBOL
/// (another Subroutine symbol). A null node if `sym` is not a Subroutine
/// symbol, or it overrides nothing. slang::ast::SubroutineSymbol::
/// getOverride reads a pointer set once during class-member elaboration
/// (setOverride), before this symbol is ever reachable through a frozen
/// &Design, so this is a pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_subroutine_override(slang_ast sym);

/// The class-method prototype a Subroutine symbol implements (an `extern`
/// method body's declaration, or the pure-virtual/`extern` prototype the
/// subroutine was constructed from), as a node of domain SLANG_AST_SYMBOL (a
/// MethodPrototype symbol). A null node if `sym` is not a Subroutine symbol,
/// or it has no associated prototype (an ordinary in-body method).
/// slang::ast::SubroutineSymbol::getPrototype reads a pointer set once
/// during elaboration, before this symbol is ever reachable through a
/// frozen &Design, so this is a pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API slang_ast slang_symbol_subroutine_prototype(slang_ast sym);

/// The resolved return type of a Subroutine symbol
/// (slang::ast::SubroutineSymbol::getReturnType, `void` for a task or a
/// function with no declared return type), as a node of domain
/// SLANG_AST_SYMBOL. A null node if `sym` is not a Subroutine symbol. The
/// underlying slang::ast::DeclaredType memo (declaredReturnType) is forced
/// by the freeze sweep's generic per-symbol getDeclaredType() pass
/// (SubroutineSymbol is one of its carriers), so this is a pure read.
/// history: since 1.3.
SLANG_C_API slang_ast slang_symbol_subroutine_return_type(slang_ast sym);

/// A Subroutine symbol's kind -- SLANG_SUBROUTINE_FUNCTION or
/// SLANG_SUBROUTINE_TASK (see slang_expr_call_subroutine_kind for the raw
/// values). SLANG_SUBROUTINE_FUNCTION (0, indistinguishable from an actual
/// function) if `sym` is not a Subroutine symbol. A direct field read
/// (slang::ast::SubroutineSymbol::subroutineKind), set once at construction
/// -- a pure, allocation-free read. history: since 1.3.
SLANG_C_API uint32_t slang_symbol_subroutine_kind(slang_ast sym);

/// True if a Subroutine symbol is a virtual class method: declared
/// `virtual`, an `extends`-overriding external implementation, or it
/// overrides another method (slang::ast::SubroutineSymbol::isVirtual --
/// `flags.has(Virtual | Extends) || overrides != nullptr`). False if `sym`
/// is not a Subroutine symbol. `overrides` is a pointer set once during
/// class-member elaboration (see slang_symbol_subroutine_override), before
/// this symbol is ever reachable through a frozen &Design, so this is a
/// pure, allocation-free read. history: since 1.3.
SLANG_C_API bool slang_symbol_subroutine_is_virtual(slang_ast sym);

/// The variable that holds a Subroutine symbol's return value while its body
/// executes (slang::ast::SubroutineSymbol::returnValVar), as a node of
/// domain SLANG_AST_SYMBOL (a Variable symbol). A null node if `sym` is not
/// a Subroutine symbol, or it is a task (which has no return value). A
/// pointer set once at construction, before this symbol is ever reachable
/// through a frozen &Design, so this is a pure, allocation-free read.
/// history: since 1.3.
SLANG_C_API slang_ast slang_symbol_subroutine_return_val_var(slang_ast sym);

/// The implicit `this` variable of a Subroutine symbol that is a
/// (non-static) class method (slang::ast::SubroutineSymbol::thisVar), as a
/// node of domain SLANG_AST_SYMBOL (a Variable symbol). A null node if
/// `sym` is not a Subroutine symbol, or it isn't a class method (a
/// free-standing function/task, DPI import, or `static` class method). A
/// pointer set once at construction (addThisVar), before this symbol is
/// ever reachable through a frozen &Design, so this is a pure,
/// allocation-free read. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_subroutine_this_var(slang_ast sym);

/* ------------------------------------------------------------------------- */
/* UninstantiatedDefSymbol                                                    */
/* ------------------------------------------------------------------------- */

/// The definition name an UninstantiatedDef symbol refers to -- the module/
/// interface/program/checker name from its instantiation syntax that could
/// not be resolved to an actual definition (e.g. `foo bar(.a(x));` when no
/// `foo` definition exists). An empty string if `sym` is not an
/// UninstantiatedDef symbol. A direct field read
/// (slang::ast::UninstantiatedDefSymbol::definitionName), set once at
/// construction -- a pure, allocation-free read. history: since 1.3.
SLANG_C_API slang_str slang_symbol_uninstantiated_def_definition_name(slang_ast sym);

/// The number of parameter expressions passed to an UninstantiatedDef
/// symbol's `#(...)` parameter list. 0 if `sym` is not an UninstantiatedDef
/// symbol. A direct field read
/// (slang::ast::UninstantiatedDefSymbol::paramExpressions), bound eagerly
/// at construction -- a pure read (the freeze sweep's UninstantiatedDefSymbol
/// branch explicitly visits each one, since none is otherwise reached by
/// the generic traversal, so every element's own canonical type is also
/// already resolved). history: since 1.3.
SLANG_C_API uint32_t slang_symbol_uninstantiated_def_param_expression_count(slang_ast sym);

/// The `index`'th parameter expression of an UninstantiatedDef symbol (see
/// slang_symbol_uninstantiated_def_param_expression_count), as a node of
/// domain SLANG_AST_EXPRESSION. These are self-determined expressions --
/// not necessarily correctly typed, since (with no resolved definition) the
/// destination parameter's type can't be known. A null node if `sym` is not
/// an UninstantiatedDef symbol or `index` is out of range. Forced the same
/// way as slang_symbol_uninstantiated_def_param_expression_count, so this
/// is a pure read. Mirrors an element of
/// slang::ast::UninstantiatedDefSymbol::paramExpressions. history: since
/// 1.3.
SLANG_C_API slang_ast slang_symbol_uninstantiated_def_param_expression(slang_ast sym,
                                                                       uint32_t index);

/// The number of port connections of an UninstantiatedDef symbol's
/// instantiation. 0 if `sym` is not an UninstantiatedDef symbol.
/// slang::ast::UninstantiatedDefSymbol::getPortConnections() binds the
/// connection list (and derives isChecker()) together on first call
/// (lazily-cached `ports`/`portNames`/`mustBeChecker` fields); the freeze sweep
/// forces that pre-seal (FreezeVisitor's UninstantiatedDefSymbol branch),
/// so this is a pure read on the frozen design. Mirrors the span size of
/// slang::ast::UninstantiatedDefSymbol::getPortConnections. history: since
/// 1.3.
SLANG_C_API uint32_t slang_symbol_uninstantiated_def_port_connection_count(slang_ast sym);

/// The `index`'th port connection expression of an UninstantiatedDef
/// symbol (see slang_symbol_uninstantiated_def_port_connection_count), as
/// a node of domain SLANG_AST_EXPRESSION. Each connection is bound as a
/// slang::ast::AssertionExpr (since, with no resolved definition, slang
/// can't yet tell an ordinary expression port from a checker formal); this
/// accessor unwraps the common case (a plain expression, wrapped as a
/// SimpleAssertionExpr) and returns null for the rarer checker-only shapes
/// (a sequence/property actual, or an empty `()` connection) -- check
/// slang_symbol_uninstantiated_def_is_checker() first if that distinction
/// matters. A null node if `sym` is not an UninstantiatedDef symbol,
/// `index` is out of range, or the connection isn't a plain expression.
/// Forced the same way as
/// slang_symbol_uninstantiated_def_port_connection_count, so this is a
/// pure read on the frozen design. history: since 1.3.
SLANG_C_API slang_ast slang_symbol_uninstantiated_def_port_connection(slang_ast sym,
                                                                      uint32_t index);

/// The name of the `index`'th port connection of an UninstantiatedDef
/// symbol (see slang_symbol_uninstantiated_def_port_connection_count) --
/// empty if it used ordered (positional) connection syntax rather than a
/// named `.name(...)` connection, or if `sym` is not an UninstantiatedDef
/// symbol or `index` is out of range. Forced the same way as
/// slang_symbol_uninstantiated_def_port_connection_count, so this is a
/// pure read. Mirrors an element of
/// slang::ast::UninstantiatedDefSymbol::getPortNames. history: since 1.3.
SLANG_C_API slang_str slang_symbol_uninstantiated_def_port_name(slang_ast sym, uint32_t index);

/// True if an UninstantiatedDef symbol must be a checker instance, based on
/// the syntax used to instantiate it (a connection using sequence/property
/// actual-argument syntax, a repetition, or an empty `()` connection --
/// none of which is legal for an ordinary module/interface/program port).
/// False if `sym` is not an UninstantiatedDef symbol, or nothing about its
/// connections forces that conclusion (it may still turn out to be a
/// checker once/if the definition is found). Forced the same way as
/// slang_symbol_uninstantiated_def_port_connection_count, so this is a
/// pure read. Mirrors slang::ast::UninstantiatedDefSymbol::isChecker.
/// history: since 1.3.
SLANG_C_API bool slang_symbol_uninstantiated_def_is_checker(slang_ast sym);

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
