//! Raw FFI bindings to slang-c, the stable C API of the
//! [slang](https://sv-lang.com) SystemVerilog compiler frontend.
//!
//! This crate mirrors `include/slang/c/slang.h` declaration for declaration.
//! Everything here is `unsafe` and follows the C header's ownership rules;
//! read that header's preamble before using any of it directly. The safe
//! `sv-lang` crate is the intended consumer.
//!
//! Enumerations are exposed as integer type aliases plus constants, so that
//! values added by a newer library never become undefined behavior here.
#![no_std]
#![allow(
    non_camel_case_types,
    non_upper_case_globals,
    clippy::missing_safety_doc
)]

use core::ffi::{c_char, c_int, c_uint, c_void};

// ---- Versioning -------------------------------------------------------------

/// `SLANG_C_VERSION_MAJOR` the declarations in this crate were generated for.
pub const SLANG_C_VERSION_MAJOR: u32 = 1;
/// `SLANG_C_VERSION_MINOR` the declarations in this crate were generated for.
pub const SLANG_C_VERSION_MINOR: u32 = 0;
/// Encoded as by `SLANG_C_VERSION_ENCODE`.
pub const SLANG_C_VERSION: u32 = SLANG_C_VERSION_MAJOR * 10000 + SLANG_C_VERSION_MINOR;

// ---- Status and errors ------------------------------------------------------

pub type slang_status = c_int;
pub const SLANG_WARN_TRUNCATED: slang_status = -2;
pub const SLANG_WARN_PARTIAL: slang_status = -1;
pub const SLANG_OK: slang_status = 0;
pub const SLANG_ERR_IO: slang_status = 1;
pub const SLANG_ERR_INVALID_ARG: slang_status = 2;
pub const SLANG_ERR_INVALID_STATE: slang_status = 3;
pub const SLANG_ERR_PARSE_RECURSION: slang_status = 4;
pub const SLANG_ERR_REWRITE: slang_status = 5;
pub const SLANG_ERR_OUT_OF_MEMORY: slang_status = 6;
pub const SLANG_ERR_UNSUPPORTED: slang_status = 7;
pub const SLANG_ERR_ABI_MISMATCH: slang_status = 8;
pub const SLANG_ERR_CANCELLED: slang_status = 9;
pub const SLANG_ERR_INTERNAL: slang_status = 100;

/// The in/out error record passed as the last parameter of fallible calls.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct slang_error {
    pub status: slang_status,
    pub message: [c_char; 248],
}

impl slang_error {
    /// Equivalent of `SLANG_ERROR_INIT`.
    pub const INIT: slang_error = slang_error {
        status: SLANG_OK,
        message: [0; 248],
    };
}

impl Default for slang_error {
    fn default() -> Self {
        Self::INIT
    }
}

// ---- Library information ----------------------------------------------------

pub type slang_build_flag = c_uint;
pub const SLANG_BUILD_EXCEPTIONS: slang_build_flag = 1 << 0;
pub const SLANG_BUILD_ASSERTIONS: slang_build_flag = 1 << 1;
pub const SLANG_BUILD_THREADS: slang_build_flag = 1 << 2;

// ---- Strings ----------------------------------------------------------------

/// A string view; owned when `owner` is non-null (free with [`slang_str_free`]).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct slang_str {
    pub data: *const c_char,
    pub len: usize,
    pub owner: *mut c_void,
}

// ---- Source locations -------------------------------------------------------

pub type slang_buffer_id = u32;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct slang_loc {
    pub buffer: slang_buffer_id,
    pub reserved_: u32,
    pub offset: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct slang_range {
    pub start: slang_loc,
    pub end: slang_loc,
}

// ---- Owners -----------------------------------------------------------------

#[repr(C)]
pub struct slang_source_manager_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct slang_syntax_tree_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct slang_compilation_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct slang_diagnostics_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct slang_options_t {
    _private: [u8; 0],
}

pub type slang_source_manager = *mut slang_source_manager_t;
pub type slang_syntax_tree = *mut slang_syntax_tree_t;
pub type slang_compilation = *mut slang_compilation_t;
pub type slang_diagnostics = *mut slang_diagnostics_t;
pub type slang_options = *mut slang_options_t;

#[repr(C)]
pub struct slang_constant_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct slang_svint_t {
    _private: [u8; 0],
}
pub type slang_constant = *mut slang_constant_t;
pub type slang_svint = *mut slang_svint_t;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct slang_svint_flat {
    pub value: i64,
    pub bit_width: u32,
    pub is_signed: bool,
    pub has_unknown: bool,
}

pub type slang_constant_kind = c_uint;
pub const SLANG_CONSTANT_BAD: slang_constant_kind = 0;
pub const SLANG_CONSTANT_INTEGER: slang_constant_kind = 1;
pub const SLANG_CONSTANT_REAL: slang_constant_kind = 2;
pub const SLANG_CONSTANT_SHORTREAL: slang_constant_kind = 3;
pub const SLANG_CONSTANT_STRING: slang_constant_kind = 4;
pub const SLANG_CONSTANT_NULL: slang_constant_kind = 5;
pub const SLANG_CONSTANT_UNBOUNDED: slang_constant_kind = 6;
pub const SLANG_CONSTANT_UNPACKED: slang_constant_kind = 7;
pub const SLANG_CONSTANT_MAP: slang_constant_kind = 8;
pub const SLANG_CONSTANT_QUEUE: slang_constant_kind = 9;
pub const SLANG_CONSTANT_UNION: slang_constant_kind = 10;

// ---- Positions --------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct slang_node {
    pub ptr: *const c_void,
    pub tree: slang_syntax_tree,
    pub kind: u32,
    pub reserved_: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct slang_token {
    pub owner: *const c_void,
    pub tree: slang_syntax_tree,
    pub index: u32,
    pub kind: u16,
    pub flags: u16,
}

pub type slang_token_flag = c_uint;
pub const SLANG_TOKEN_MISSING: slang_token_flag = 1 << 0;

pub type slang_ast_domain = c_uint;
pub const SLANG_AST_SYMBOL: slang_ast_domain = 0;
pub const SLANG_AST_EXPRESSION: slang_ast_domain = 1;
pub const SLANG_AST_STATEMENT: slang_ast_domain = 2;
pub const SLANG_AST_TIMING_CONTROL: slang_ast_domain = 3;
pub const SLANG_AST_CONSTRAINT: slang_ast_domain = 4;
pub const SLANG_AST_ASSERTION_EXPR: slang_ast_domain = 5;
pub const SLANG_AST_BINS_SELECT_EXPR: slang_ast_domain = 6;
pub const SLANG_AST_PATTERN: slang_ast_domain = 7;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct slang_ast {
    pub ptr: *const c_void,
    pub compilation: slang_compilation,
    pub kind: u32,
    pub domain: u32,
}

// The value-struct ABI. Sizes are also `static_assert`ed on the C side; the
// field offsets below (LP64) are the Rust-side ABI golden — a field reorder
// keeps the size identical but silently corrupts every read across the FFI, so
// pin the offsets, not just the sizes. Guarded to 64-bit native (the only
// target sv-lang-sys links a native slang for; the wasm backend is separate).
#[cfg(target_pointer_width = "64")]
const _: () = {
    use core::mem::{offset_of, size_of};

    assert!(size_of::<slang_node>() == 24);
    assert!(size_of::<slang_token>() == 24);
    assert!(size_of::<slang_ast>() == 24);
    assert!(size_of::<slang_loc>() == 16);
    assert!(size_of::<slang_range>() == 32);
    assert!(size_of::<slang_error>() == 252);
    assert!(size_of::<slang_str>() == 24);
    assert!(size_of::<slang_diag>() == 32);

    assert!(offset_of!(slang_node, ptr) == 0);
    assert!(offset_of!(slang_node, tree) == 8);
    assert!(offset_of!(slang_node, kind) == 16);

    assert!(offset_of!(slang_token, owner) == 0);
    assert!(offset_of!(slang_token, tree) == 8);
    assert!(offset_of!(slang_token, index) == 16);
    assert!(offset_of!(slang_token, kind) == 20);
    assert!(offset_of!(slang_token, flags) == 22);

    assert!(offset_of!(slang_ast, ptr) == 0);
    assert!(offset_of!(slang_ast, compilation) == 8);
    assert!(offset_of!(slang_ast, kind) == 16);
    assert!(offset_of!(slang_ast, domain) == 20);

    assert!(offset_of!(slang_str, data) == 0);
    assert!(offset_of!(slang_str, len) == 8);
    assert!(offset_of!(slang_str, owner) == 16);

    assert!(offset_of!(slang_error, status) == 0);
    assert!(offset_of!(slang_error, message) == 4);

    assert!(offset_of!(slang_loc, buffer) == 0);
    assert!(offset_of!(slang_loc, offset) == 8);

    assert!(offset_of!(slang_diag, code) == 0);
    assert!(offset_of!(slang_diag, location) == 8);
    assert!(offset_of!(slang_diag, note_count) == 24);
    assert!(offset_of!(slang_diag, range_count) == 28);
};

// ---- Kind reflection --------------------------------------------------------

pub type slang_member_form = c_uint;
pub const SLANG_MEMBER_TOKEN: slang_member_form = 0;
pub const SLANG_MEMBER_NODE: slang_member_form = 1;
pub const SLANG_MEMBER_OPTIONAL_NODE: slang_member_form = 2;
pub const SLANG_MEMBER_LIST: slang_member_form = 3;
pub const SLANG_MEMBER_SEPARATED_LIST: slang_member_form = 4;
pub const SLANG_MEMBER_TOKEN_LIST: slang_member_form = 5;

// ---- Options ----------------------------------------------------------------

pub type slang_language_version = c_uint;
pub const SLANG_LANGUAGE_1364_2005: slang_language_version = 0;
pub const SLANG_LANGUAGE_1800_2017: slang_language_version = 1;
pub const SLANG_LANGUAGE_1800_2023: slang_language_version = 2;

pub type slang_compilation_flag = c_uint;
pub const SLANG_COMP_ALLOW_HIERARCHICAL_CONST: slang_compilation_flag = 1 << 0;
pub const SLANG_COMP_RELAX_ENUM_CONVERSIONS: slang_compilation_flag = 1 << 1;
pub const SLANG_COMP_ALLOW_USE_BEFORE_DECLARE: slang_compilation_flag = 1 << 2;
pub const SLANG_COMP_ALLOW_TOP_LEVEL_IFACE_PORTS: slang_compilation_flag = 1 << 3;
pub const SLANG_COMP_LINT_MODE: slang_compilation_flag = 1 << 4;
pub const SLANG_COMP_IGNORE_UNKNOWN_MODULES: slang_compilation_flag = 1 << 5;
pub const SLANG_COMP_RELAX_STRING_CONVERSIONS: slang_compilation_flag = 1 << 6;
pub const SLANG_COMP_ALLOW_RECURSIVE_IMPLICIT_CALL: slang_compilation_flag = 1 << 7;
pub const SLANG_COMP_ALLOW_BARE_VAL_PARAM_ASSIGNMENT: slang_compilation_flag = 1 << 8;
pub const SLANG_COMP_ALLOW_SELF_DETERMINED_STREAM_CONCAT: slang_compilation_flag = 1 << 9;
pub const SLANG_COMP_ALLOW_MERGING_ANSI_PORTS: slang_compilation_flag = 1 << 10;
pub const SLANG_COMP_DISABLE_INSTANCE_CACHING: slang_compilation_flag = 1 << 11;
pub const SLANG_COMP_DISALLOW_REFS_TO_UNKNOWN_INSTANCES: slang_compilation_flag = 1 << 12;
pub const SLANG_COMP_ALLOW_UNNAMED_GENERATE: slang_compilation_flag = 1 << 13;
pub const SLANG_COMP_ALLOW_VIRTUAL_IFACE_WITH_OVERRIDE: slang_compilation_flag = 1 << 14;
pub const SLANG_COMP_ALLOW_ARRAY_CONCAT_ASSIGN_PATTERN: slang_compilation_flag = 1 << 15;
pub const SLANG_COMP_ALLOW_CROSS_AUTO_BIN_MAX: slang_compilation_flag = 1 << 16;
pub const SLANG_COMP_ALLOW_INVALID_TOP: slang_compilation_flag = 1 << 17;
pub const SLANG_COMP_CHECK_UNINSTANTIATED: slang_compilation_flag = 1 << 18;

// ---- Syntax nodes and tokens ------------------------------------------------

pub type slang_child_tag = c_uint;
pub const SLANG_CHILD_NONE: slang_child_tag = 0;
pub const SLANG_CHILD_NODE: slang_child_tag = 1;
pub const SLANG_CHILD_TOKEN: slang_child_tag = 2;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct slang_trivia {
    pub kind: u32,
    pub reserved_: u32,
    pub text: slang_str,
}

// ---- Traversal --------------------------------------------------------------

pub type slang_visit = c_uint;
pub const SLANG_VISIT_BREAK: slang_visit = 0;
pub const SLANG_VISIT_CONTINUE: slang_visit = 1;
pub const SLANG_VISIT_SKIP: slang_visit = 2;

pub type slang_node_visitor =
    Option<unsafe extern "C" fn(node: slang_node, user: *mut c_void) -> slang_visit>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct slang_syntax_sink {
    pub start_node: Option<unsafe extern "C" fn(user: *mut c_void, kind: u32, syntax_struct: u32)>,
    pub trivia:
        Option<unsafe extern "C" fn(user: *mut c_void, kind: u32, text: *const c_char, len: usize)>,
    pub token: Option<
        unsafe extern "C" fn(
            user: *mut c_void,
            kind: u32,
            text: *const c_char,
            len: usize,
            loc: slang_loc,
            flags: u16,
        ),
    >,
    pub absent: Option<unsafe extern "C" fn(user: *mut c_void)>,
    pub start_list: Option<unsafe extern "C" fn(user: *mut c_void, form: slang_member_form)>,
    pub finish_list: Option<unsafe extern "C" fn(user: *mut c_void)>,
    pub finish_node: Option<unsafe extern "C" fn(user: *mut c_void)>,
}

pub type slang_ast_visitor = Option<
    unsafe extern "C" fn(node: slang_ast, parent: slang_ast, user: *mut c_void) -> slang_visit,
>;

// ---- Diagnostics ------------------------------------------------------------

pub type slang_severity = c_uint;
pub const SLANG_SEVERITY_IGNORED: slang_severity = 0;
pub const SLANG_SEVERITY_NOTE: slang_severity = 1;
pub const SLANG_SEVERITY_WARNING: slang_severity = 2;
pub const SLANG_SEVERITY_ERROR: slang_severity = 3;
pub const SLANG_SEVERITY_FATAL: slang_severity = 4;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct slang_diag {
    pub code: u32,
    pub severity: slang_severity,
    pub location: slang_loc,
    pub note_count: u32,
    pub range_count: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct slang_render_options {
    pub colors: bool,
    pub show_source: bool,
    pub show_include_stack: bool,
    pub absolute_paths: bool,
}

// ---- Compilation ------------------------------------------------------------

pub type slang_freeze_flag = c_uint;
pub const SLANG_FREEZE_ELABORATE_ALL: slang_freeze_flag = 1 << 0;
pub const SLANG_FREEZE_PREFOLD: slang_freeze_flag = 1 << 1;
pub const SLANG_FREEZE_SEAL: slang_freeze_flag = 1 << 2;
pub const SLANG_FREEZE_ALL: slang_freeze_flag =
    SLANG_FREEZE_ELABORATE_ALL | SLANG_FREEZE_PREFOLD | SLANG_FREEZE_SEAL;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct slang_freeze_report {
    pub symbols_elaborated: u64,
    pub expressions_visited: u64,
    pub expressions_folded: u64,
    pub fold_failures: u64,
    pub types_canonicalized: u64,
    pub params_folded: u64,
}

pub type slang_definition_kind = c_uint;
pub const SLANG_DEFINITION_MODULE: slang_definition_kind = 0;
pub const SLANG_DEFINITION_INTERFACE: slang_definition_kind = 1;
pub const SLANG_DEFINITION_PROGRAM: slang_definition_kind = 2;

// ---- Analysis ---------------------------------------------------------------

#[repr(C)]
pub struct slang_analysis_t {
    _private: [u8; 0],
}
pub type slang_analysis = *mut slang_analysis_t;

/// Analysis listeners (see `slang_analysis_run_listening`); each may be called
/// concurrently from several worker threads.
pub type slang_procedure_listener =
    Option<unsafe extern "C" fn(procedure: slang_ast, user: *mut c_void)>;
pub type slang_scope_listener = Option<unsafe extern "C" fn(scope: slang_ast, user: *mut c_void)>;
pub type slang_assertion_listener =
    Option<unsafe extern "C" fn(containing_symbol: slang_ast, user: *mut c_void)>;

/// A set of analysis listeners; any callback may be `None`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct slang_analysis_listeners {
    pub on_procedure: slang_procedure_listener,
    pub on_scope: slang_scope_listener,
    pub on_assertion: slang_assertion_listener,
    pub user: *mut c_void,
}

pub type slang_analysis_flag = c_uint;
pub const SLANG_ANALYSIS_CHECK_UNUSED: slang_analysis_flag = 1 << 0;
pub const SLANG_ANALYSIS_FULL_CASE_UNIQUE_PRIORITY: slang_analysis_flag = 1 << 1;
pub const SLANG_ANALYSIS_FULL_CASE_FOUR_STATE: slang_analysis_flag = 1 << 2;
pub const SLANG_ANALYSIS_ALLOW_MULTI_DRIVEN_LOCALS: slang_analysis_flag = 1 << 3;
pub const SLANG_ANALYSIS_ALLOW_DUP_INITIAL_DRIVERS: slang_analysis_flag = 1 << 4;
pub const SLANG_ANALYSIS_CHECK_SHADOW: slang_analysis_flag = 1 << 5;
pub const SLANG_ANALYSIS_INLINE_CONT_ASSIGN_FUNCTION_READS: slang_analysis_flag = 1 << 6;
pub const SLANG_ANALYSIS_ALWAYS_STAR_USES_LSPS: slang_analysis_flag = 1 << 7;
pub const SLANG_ANALYSIS_CONT_ASSIGN_USES_LSPS: slang_analysis_flag = 1 << 8;

pub type slang_driver_kind = c_uint;
pub const SLANG_DRIVER_PROCEDURAL: slang_driver_kind = 0;
pub const SLANG_DRIVER_CONTINUOUS: slang_driver_kind = 1;
pub const SLANG_DRIVER_OTHER: slang_driver_kind = 2;

pub type slang_driver_flag = c_uint;
pub const SLANG_DRIVER_INPUT_PORT: slang_driver_flag = 1 << 0;
pub const SLANG_DRIVER_UNIDIRECTIONAL_PORT: slang_driver_flag = 1 << 1;
pub const SLANG_DRIVER_CLOCK_VAR: slang_driver_flag = 1 << 2;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct slang_driver_info {
    pub kind: slang_driver_kind,
    pub flags: u32,
    pub range: slang_range,
    pub containing_symbol: slang_ast,
}

pub type slang_dfa_event_kind = c_uint;
pub const SLANG_DFA_READ: slang_dfa_event_kind = 0;
pub const SLANG_DFA_WRITE: slang_dfa_event_kind = 1;
pub const SLANG_DFA_CALL: slang_dfa_event_kind = 2;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct slang_dfa_event {
    pub kind: u32,
    pub symbol: slang_ast,
    pub node: slang_ast,
}

#[repr(C)]
pub struct slang_dfa_lattice {
    pub top: Option<unsafe extern "C" fn(user: *mut c_void) -> *mut c_void>,
    pub bottom: Option<unsafe extern "C" fn(user: *mut c_void) -> *mut c_void>,
    pub clone: Option<unsafe extern "C" fn(user: *mut c_void, state: *const c_void) -> *mut c_void>,
    pub join:
        Option<unsafe extern "C" fn(user: *mut c_void, into: *mut c_void, other: *const c_void)>,
    pub meet:
        Option<unsafe extern "C" fn(user: *mut c_void, into: *mut c_void, other: *const c_void)>,
    pub transfer: Option<
        unsafe extern "C" fn(user: *mut c_void, state: *mut c_void, event: *const slang_dfa_event),
    >,
    pub drop: Option<unsafe extern "C" fn(user: *mut c_void, state: *mut c_void)>,
}

// ---- Driver -----------------------------------------------------------------

#[repr(C)]
pub struct slang_driver_t {
    _private: [u8; 0],
}
pub type slang_driver = *mut slang_driver_t;

pub type slang_option_kind = c_uint;
pub const SLANG_OPTION_FLAG: slang_option_kind = 0;
pub const SLANG_OPTION_INT: slang_option_kind = 1;
pub const SLANG_OPTION_STRING: slang_option_kind = 2;

// ---- Functions --------------------------------------------------------------

unsafe extern "C" {
    // Status and library information
    pub fn slang_status_name(status: slang_status) -> *const c_char;
    pub fn slang_c_version() -> u32;
    pub fn slang_version_string() -> *const c_char;
    pub fn slang_syntax_model_hash() -> *const c_char;
    pub fn slang_diagnostics_model_hash() -> *const c_char;
    pub fn slang_build_flags() -> u32;

    // Strings
    pub fn slang_str_free(str: slang_str);

    // Kind reflection
    pub fn slang_syntax_kind_count() -> u32;
    pub fn slang_syntax_kind_name(kind: u32) -> slang_str;
    pub fn slang_syntax_kind_struct(kind: u32) -> u32;
    pub fn slang_syntax_struct_count() -> u32;
    pub fn slang_syntax_struct_name(syntax_struct: u32) -> slang_str;
    pub fn slang_syntax_struct_member_count(syntax_struct: u32) -> u32;
    pub fn slang_syntax_member_name(syntax_struct: u32, member: u32) -> slang_str;
    pub fn slang_syntax_member_form(syntax_struct: u32, member: u32) -> slang_member_form;
    pub fn slang_token_kind_count() -> u32;
    pub fn slang_token_kind_name(kind: u32) -> slang_str;
    pub fn slang_trivia_kind_count() -> u32;
    pub fn slang_trivia_kind_name(kind: u32) -> slang_str;
    pub fn slang_ast_kind_name(domain: slang_ast_domain, kind: u32) -> slang_str;
    pub fn slang_ast_kind_count(domain: slang_ast_domain) -> u32;

    // Source manager
    pub fn slang_source_manager_create(err: *mut slang_error) -> slang_source_manager;
    pub fn slang_source_manager_destroy(sm: slang_source_manager);
    pub fn slang_source_manager_add_include_dir(
        sm: slang_source_manager,
        pattern: *const c_char,
        pattern_len: usize,
        system: bool,
        err: *mut slang_error,
    );
    pub fn slang_source_manager_assign_text(
        sm: slang_source_manager,
        path: *const c_char,
        path_len: usize,
        text: *const c_char,
        text_len: usize,
        err: *mut slang_error,
    ) -> slang_buffer_id;
    pub fn slang_source_manager_read_file(
        sm: slang_source_manager,
        path: *const c_char,
        path_len: usize,
        err: *mut slang_error,
    ) -> slang_buffer_id;
    pub fn slang_source_manager_file_name(sm: slang_source_manager, loc: slang_loc) -> slang_str;
    pub fn slang_source_manager_text(
        sm: slang_source_manager,
        buffer: slang_buffer_id,
    ) -> slang_str;
    pub fn slang_source_manager_line(sm: slang_source_manager, loc: slang_loc) -> usize;
    pub fn slang_source_manager_column(sm: slang_source_manager, loc: slang_loc) -> usize;
    pub fn slang_source_manager_is_macro_loc(sm: slang_source_manager, loc: slang_loc) -> bool;
    pub fn slang_source_manager_original_loc(sm: slang_source_manager, loc: slang_loc)
    -> slang_loc;

    // Options
    pub fn slang_options_create(err: *mut slang_error) -> slang_options;
    pub fn slang_options_destroy(options: slang_options);
    pub fn slang_options_set_language_version(
        options: slang_options,
        version: slang_language_version,
    );
    pub fn slang_options_define(
        options: slang_options,
        name: *const c_char,
        name_len: usize,
        value: *const c_char,
        value_len: usize,
        err: *mut slang_error,
    );
    pub fn slang_options_add_top_module(
        options: slang_options,
        name: *const c_char,
        name_len: usize,
        err: *mut slang_error,
    );
    pub fn slang_options_set_compilation_flags(options: slang_options, flags: u32);

    // Syntax trees
    pub fn slang_syntax_tree_from_text(
        sm: slang_source_manager,
        text: *const c_char,
        text_len: usize,
        name: *const c_char,
        name_len: usize,
        path: *const c_char,
        path_len: usize,
        options: slang_options,
        err: *mut slang_error,
    ) -> slang_syntax_tree;
    pub fn slang_syntax_tree_from_file(
        sm: slang_source_manager,
        path: *const c_char,
        path_len: usize,
        options: slang_options,
        err: *mut slang_error,
    ) -> slang_syntax_tree;
    pub fn slang_syntax_tree_from_buffer(
        sm: slang_source_manager,
        buffer: slang_buffer_id,
        options: slang_options,
        err: *mut slang_error,
    ) -> slang_syntax_tree;
    pub fn slang_syntax_tree_retain(tree: slang_syntax_tree) -> slang_syntax_tree;
    pub fn slang_syntax_tree_release(tree: slang_syntax_tree);
    pub fn slang_syntax_tree_root(tree: slang_syntax_tree) -> slang_node;
    pub fn slang_syntax_tree_diagnostics(
        tree: slang_syntax_tree,
        err: *mut slang_error,
    ) -> slang_diagnostics;
    pub fn slang_syntax_tree_source_manager(tree: slang_syntax_tree) -> slang_source_manager;
    pub fn slang_syntax_tree_to_string(tree: slang_syntax_tree, err: *mut slang_error)
    -> slang_str;

    // Syntax nodes
    pub fn slang_node_is_null(node: slang_node) -> bool;
    pub fn slang_node_parent(node: slang_node) -> slang_node;
    pub fn slang_node_struct(node: slang_node) -> u32;
    pub fn slang_node_child_count(node: slang_node) -> u32;
    pub fn slang_node_child(
        node: slang_node,
        index: u32,
        node_out: *mut slang_node,
        token_out: *mut slang_token,
    ) -> slang_child_tag;
    pub fn slang_node_member_span(
        node: slang_node,
        member: u32,
        start_out: *mut u32,
        len_out: *mut u32,
    ) -> bool;
    pub fn slang_node_range(node: slang_node) -> slang_range;
    pub fn slang_node_to_string(node: slang_node, err: *mut slang_error) -> slang_str;
    pub fn slang_node_first_token(node: slang_node) -> slang_token;
    pub fn slang_node_last_token(node: slang_node) -> slang_token;
    pub fn slang_node_is_equivalent(a: slang_node, b: slang_node) -> bool;

    // Tokens
    pub fn slang_token_location(token: slang_token) -> slang_loc;
    pub fn slang_token_range(token: slang_token) -> slang_range;
    pub fn slang_token_raw_text(token: slang_token) -> slang_str;
    pub fn slang_token_value_text(token: slang_token) -> slang_str;
    pub fn slang_token_trivia_count(token: slang_token) -> u32;
    pub fn slang_token_trivia(token: slang_token, index: u32, out: *mut slang_trivia) -> bool;
    pub fn slang_token_trivia_syntax(token: slang_token, index: u32) -> slang_node;

    // Syntax traversal
    pub fn slang_node_visit(
        node: slang_node,
        visitor: slang_node_visitor,
        user: *mut c_void,
        err: *mut slang_error,
    );
    pub fn slang_syntax_tree_walk(
        tree: slang_syntax_tree,
        sink: *const slang_syntax_sink,
        user: *mut c_void,
        err: *mut slang_error,
    );

    // Diagnostics
    pub fn slang_diagnostics_destroy(diags: slang_diagnostics);
    pub fn slang_diagnostics_count(diags: slang_diagnostics) -> u32;
    pub fn slang_diagnostics_at(diags: slang_diagnostics, index: u32, out: *mut slang_diag)
    -> bool;
    pub fn slang_diagnostics_message(
        diags: slang_diagnostics,
        index: u32,
        err: *mut slang_error,
    ) -> slang_str;
    pub fn slang_diagnostics_note(
        diags: slang_diagnostics,
        index: u32,
        note: u32,
        out: *mut slang_diag,
    ) -> bool;
    pub fn slang_diagnostics_note_message(
        diags: slang_diagnostics,
        index: u32,
        note: u32,
        err: *mut slang_error,
    ) -> slang_str;
    pub fn slang_diagnostics_range(
        diags: slang_diagnostics,
        index: u32,
        range: u32,
        out: *mut slang_range,
    ) -> bool;
    pub fn slang_diagnostics_symbol(diags: slang_diagnostics, index: u32) -> slang_ast;
    pub fn slang_diagnostics_render(
        diags: slang_diagnostics,
        options: *const slang_render_options,
        err: *mut slang_error,
    ) -> slang_str;

    // Compilation
    pub fn slang_compilation_create(
        options: slang_options,
        err: *mut slang_error,
    ) -> slang_compilation;
    pub fn slang_compilation_destroy(comp: slang_compilation);
    pub fn slang_compilation_add_tree(
        comp: slang_compilation,
        tree: slang_syntax_tree,
        err: *mut slang_error,
    );
    pub fn slang_compilation_freeze(
        comp: slang_compilation,
        flags: u32,
        report: *mut slang_freeze_report,
        err: *mut slang_error,
    );
    pub fn slang_compilation_is_sealed(comp: slang_compilation) -> bool;
    pub fn slang_compilation_root(comp: slang_compilation, err: *mut slang_error) -> slang_ast;
    pub fn slang_compilation_diagnostics(
        comp: slang_compilation,
        err: *mut slang_error,
    ) -> slang_diagnostics;
    pub fn slang_compilation_source_manager(comp: slang_compilation) -> slang_source_manager;
    pub fn slang_compilation_top_instance_count(comp: slang_compilation) -> u32;
    pub fn slang_compilation_top_instance(comp: slang_compilation, index: u32) -> slang_ast;
    pub fn slang_compilation_definition_count(comp: slang_compilation) -> u32;
    pub fn slang_compilation_definition(comp: slang_compilation, index: u32) -> slang_ast;
    pub fn slang_compilation_package_count(comp: slang_compilation) -> u32;
    pub fn slang_compilation_package(comp: slang_compilation, index: u32) -> slang_ast;

    // AST: generic
    pub fn slang_ast_is_null(node: slang_ast) -> bool;
    pub fn slang_ast_range(node: slang_ast) -> slang_range;
    pub fn slang_ast_syntax(node: slang_ast) -> slang_node;
    pub fn slang_ast_visit(
        root: slang_ast,
        visitor: slang_ast_visitor,
        user: *mut c_void,
        err: *mut slang_error,
    );

    // AST: symbols and scopes
    pub fn slang_symbol_name(symbol: slang_ast) -> slang_str;
    pub fn slang_symbol_location(symbol: slang_ast) -> slang_loc;
    pub fn slang_symbol_parent_scope(symbol: slang_ast) -> slang_ast;
    pub fn slang_symbol_next_sibling(symbol: slang_ast) -> slang_ast;
    pub fn slang_symbol_hierarchical_path(symbol: slang_ast, err: *mut slang_error) -> slang_str;
    pub fn slang_symbol_is_scope(symbol: slang_ast) -> bool;
    pub fn slang_symbol_is_type(symbol: slang_ast) -> bool;
    pub fn slang_symbol_is_value(symbol: slang_ast) -> bool;
    pub fn slang_scope_first_member(scope: slang_ast, err: *mut slang_error) -> slang_ast;
    pub fn slang_scope_find(
        scope: slang_ast,
        name: *const c_char,
        name_len: usize,
        err: *mut slang_error,
    ) -> slang_ast;
    pub fn slang_scope_lookup(
        scope: slang_ast,
        name: *const c_char,
        name_len: usize,
        err: *mut slang_error,
    ) -> slang_ast;
    pub fn slang_value_type(symbol: slang_ast, err: *mut slang_error) -> slang_ast;
    pub fn slang_value_initializer(symbol: slang_ast, err: *mut slang_error) -> slang_ast;
    pub fn slang_instance_body(instance: slang_ast) -> slang_ast;
    pub fn slang_instance_definition(instance: slang_ast) -> slang_ast;
    pub fn slang_instance_parameter_count(instance: slang_ast) -> u32;
    pub fn slang_instance_parameter(instance: slang_ast, index: u32) -> slang_ast;
    pub fn slang_parameter_value(parameter: slang_ast, err: *mut slang_error) -> slang_str;
    pub fn slang_definition_kind_of(definition: slang_ast) -> slang_definition_kind;

    // AST: types
    pub fn slang_type_canonical(ty: slang_ast) -> slang_ast;
    pub fn slang_type_to_string(ty: slang_ast, err: *mut slang_error) -> slang_str;
    pub fn slang_type_bit_width(ty: slang_ast) -> u64;
    pub fn slang_type_is_integral(ty: slang_ast) -> bool;
    pub fn slang_type_is_signed(ty: slang_ast) -> bool;
    pub fn slang_type_is_four_state(ty: slang_ast) -> bool;
    pub fn slang_type_is_unpacked_array(ty: slang_ast) -> bool;
    pub fn slang_type_is_class(ty: slang_ast) -> bool;
    pub fn slang_type_is_enum(ty: slang_ast) -> bool;
    pub fn slang_type_is_struct(ty: slang_ast) -> bool;
    pub fn slang_type_is_union(ty: slang_ast) -> bool;
    pub fn slang_type_is_array(ty: slang_ast) -> bool;
    pub fn slang_type_is_string(ty: slang_ast) -> bool;
    pub fn slang_type_array_element(ty: slang_ast) -> slang_ast;
    pub fn slang_type_enum_base(ty: slang_ast) -> slang_ast;
    pub fn slang_enum_member_count(ty: slang_ast) -> u32;
    pub fn slang_enum_member(ty: slang_ast, index: u32) -> slang_ast;
    pub fn slang_enum_member_value(member: slang_ast, err: *mut slang_error) -> slang_str;
    pub fn slang_type_field_count(ty: slang_ast) -> u32;
    pub fn slang_type_field(ty: slang_ast, index: u32) -> slang_ast;
    pub fn slang_field_bit_offset(field: slang_ast) -> u64;
    pub fn slang_field_index(field: slang_ast) -> u32;
    pub fn slang_type_class_base(ty: slang_ast) -> slang_ast;
    pub fn slang_type_is_matching(a: slang_ast, b: slang_ast) -> bool;
    pub fn slang_type_is_equivalent(a: slang_ast, b: slang_ast) -> bool;
    pub fn slang_type_is_assignment_compatible(a: slang_ast, b: slang_ast) -> bool;

    // AST: expressions
    pub fn slang_expression_type(expr: slang_ast) -> slang_ast;
    pub fn slang_expression_is_bad(expr: slang_ast) -> bool;
    pub fn slang_expression_symbol(expr: slang_ast) -> slang_ast;
    pub fn slang_expression_cached_constant(
        expr: slang_ast,
        out: *mut slang_str,
        err: *mut slang_error,
    ) -> bool;
    pub fn slang_expression_eval(
        expr: slang_ast,
        out: *mut slang_str,
        err: *mut slang_error,
    ) -> bool;

    // Constant values
    pub fn slang_expression_constant_value(
        expr: slang_ast,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_expression_eval_constant(expr: slang_ast, err: *mut slang_error)
    -> slang_constant;
    pub fn slang_constant_destroy(c: slang_constant);
    pub fn slang_constant_kind_of(c: slang_constant) -> u32;
    pub fn slang_constant_has_unknown(c: slang_constant) -> bool;
    pub fn slang_constant_real(c: slang_constant, out: *mut f64) -> bool;
    pub fn slang_constant_string(c: slang_constant) -> slang_str;
    pub fn slang_constant_size(c: slang_constant) -> u64;
    pub fn slang_constant_element(
        c: slang_constant,
        index: u64,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_constant_to_string(c: slang_constant) -> slang_str;
    pub fn slang_constant_integer(c: slang_constant) -> slang_svint;
    pub fn slang_constant_flat_int(c: slang_constant, out: *mut slang_svint_flat) -> bool;
    pub fn slang_svint_bit_width(v: slang_svint) -> u32;
    pub fn slang_svint_is_signed(v: slang_svint) -> bool;
    pub fn slang_svint_has_unknown(v: slang_svint) -> bool;
    pub fn slang_svint_as_i64(v: slang_svint, out: *mut i64) -> bool;
    pub fn slang_svint_as_u64(v: slang_svint, out: *mut u64) -> bool;
    pub fn slang_svint_get_bit(v: slang_svint, index: u32) -> u8;
    pub fn slang_svint_to_string(v: slang_svint) -> slang_str;

    // Semantic statement/expression tree
    pub fn slang_symbol_body(sym: slang_ast) -> slang_ast;
    pub fn slang_ast_sem_child_count(node: slang_ast) -> u32;
    pub fn slang_ast_sem_child(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_expr_binary_op(node: slang_ast) -> u32;
    pub fn slang_expr_unary_op(node: slang_ast) -> u32;
    pub fn slang_expr_assignment_is_nonblocking(node: slang_ast) -> bool;
    pub fn slang_expr_call_subroutine(node: slang_ast) -> slang_ast;
    pub fn slang_expr_member_symbol(node: slang_ast) -> slang_ast;

    // Typed statement/expression children
    pub fn slang_stmt_then_branch(node: slang_ast) -> slang_ast;
    pub fn slang_stmt_else_branch(node: slang_ast) -> slang_ast;
    pub fn slang_stmt_body(node: slang_ast) -> slang_ast;
    pub fn slang_stmt_cond(node: slang_ast) -> slang_ast;
    pub fn slang_stmt_expr(node: slang_ast) -> slang_ast;
    pub fn slang_stmt_timing(node: slang_ast) -> slang_ast;
    pub fn slang_expr_cond_true(node: slang_ast) -> slang_ast;
    pub fn slang_expr_cond_false(node: slang_ast) -> slang_ast;
    pub fn slang_expr_select_value(node: slang_ast) -> slang_ast;
    pub fn slang_expr_select_selector(node: slang_ast) -> slang_ast;
    pub fn slang_expr_range_left(node: slang_ast) -> slang_ast;
    pub fn slang_expr_range_right(node: slang_ast) -> slang_ast;
    pub fn slang_expr_range_selection_kind(node: slang_ast) -> u32;
    pub fn slang_expr_conversion_operand(node: slang_ast) -> slang_ast;
    pub fn slang_expr_conversion_kind(node: slang_ast) -> u32;
    pub fn slang_expr_replication_count(node: slang_ast) -> slang_ast;
    pub fn slang_expr_replication_concat(node: slang_ast) -> slang_ast;

    // Driver
    pub fn slang_driver_create(err: *mut slang_error) -> slang_driver;
    pub fn slang_driver_destroy(driver: slang_driver);
    pub fn slang_driver_add_option(
        driver: slang_driver,
        names: *const c_char,
        names_len: usize,
        kind: slang_option_kind,
        description: *const c_char,
        description_len: usize,
        value_name: *const c_char,
        value_name_len: usize,
        err: *mut slang_error,
    );
    pub fn slang_driver_parse_args(
        driver: slang_driver,
        argc: c_int,
        argv: *const *const c_char,
        err: *mut slang_error,
    ) -> bool;
    pub fn slang_driver_option_flag(
        driver: slang_driver,
        names: *const c_char,
        names_len: usize,
        out: *mut bool,
    ) -> bool;
    pub fn slang_driver_option_int(
        driver: slang_driver,
        names: *const c_char,
        names_len: usize,
        out: *mut i64,
    ) -> bool;
    pub fn slang_driver_option_string(
        driver: slang_driver,
        names: *const c_char,
        names_len: usize,
        out: *mut slang_str,
    ) -> bool;
    pub fn slang_driver_help_text(
        driver: slang_driver,
        overview: *const c_char,
        overview_len: usize,
        err: *mut slang_error,
    ) -> slang_str;
    pub fn slang_driver_process_options(driver: slang_driver, err: *mut slang_error) -> bool;
    pub fn slang_driver_parse_sources(driver: slang_driver, err: *mut slang_error) -> bool;
    pub fn slang_driver_source_manager(driver: slang_driver) -> slang_source_manager;
    pub fn slang_driver_tree_count(driver: slang_driver) -> u32;
    pub fn slang_driver_tree(driver: slang_driver, index: u32) -> slang_syntax_tree;
    pub fn slang_driver_create_compilation(
        driver: slang_driver,
        extra_flags: u32,
        err: *mut slang_error,
    ) -> slang_compilation;
    pub fn slang_driver_report_compilation(
        driver: slang_driver,
        comp: slang_compilation,
        quiet: bool,
        err: *mut slang_error,
    );
    pub fn slang_driver_report_diagnostics(
        driver: slang_driver,
        quiet: bool,
        err: *mut slang_error,
    ) -> bool;

    // Analysis
    pub fn slang_analysis_run(
        comp: slang_compilation,
        flags: u32,
        threads: u32,
        err: *mut slang_error,
    ) -> slang_analysis;
    pub fn slang_analysis_run_listening(
        comp: slang_compilation,
        flags: u32,
        threads: u32,
        listeners: *const slang_analysis_listeners,
        err: *mut slang_error,
    ) -> slang_analysis;
    pub fn slang_analysis_destroy(analysis: slang_analysis);
    pub fn slang_analysis_diagnostics(
        analysis: slang_analysis,
        err: *mut slang_error,
    ) -> slang_diagnostics;
    pub fn slang_analysis_scope_procedure_count(analysis: slang_analysis, scope: slang_ast) -> u32;
    pub fn slang_analysis_scope_procedure(
        analysis: slang_analysis,
        scope: slang_ast,
        index: u32,
    ) -> slang_ast;
    pub fn slang_analysis_procedure_has_clock(
        analysis: slang_analysis,
        scope: slang_ast,
        index: u32,
    ) -> bool;
    pub fn slang_analysis_driver_count(analysis: slang_analysis, value: slang_ast) -> u32;
    pub fn slang_analysis_driver(
        analysis: slang_analysis,
        value: slang_ast,
        index: u32,
        out: *mut slang_driver_info,
    ) -> bool;

    pub fn slang_dfa_run(
        comp: slang_compilation,
        procedure: slang_ast,
        lattice: *const slang_dfa_lattice,
        user: *mut c_void,
        err: *mut slang_error,
    ) -> *mut c_void;
}
