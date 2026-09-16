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
pub const SLANG_C_VERSION_MINOR: u32 = 3;
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

/// A statically-known iteration range of array bit/element indices — a plain
/// value, not an AST node. Mirrors slang::ConstantRange; `left`/`right` are
/// not necessarily ascending.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct slang_constant_range {
    pub left: i32,
    pub right: i32,
}

/// The kind of an evaluated array dimension. Mirrors `slang::ast::DimensionKind`.
pub type slang_dimension_kind = c_uint;
pub const SLANG_DIM_UNKNOWN: slang_dimension_kind = 0;
pub const SLANG_DIM_RANGE: slang_dimension_kind = 1;
pub const SLANG_DIM_ABBREVIATED_RANGE: slang_dimension_kind = 2;
pub const SLANG_DIM_DYNAMIC: slang_dimension_kind = 3;
pub const SLANG_DIM_ASSOCIATIVE: slang_dimension_kind = 4;
pub const SLANG_DIM_QUEUE: slang_dimension_kind = 5;
pub const SLANG_DIM_DPI_OPEN_ARRAY: slang_dimension_kind = 6;

/// One resolved array dimension of a symbol's declared type — a plain value,
/// not an AST node. Mirrors `slang::ast::EvaluatedDimension` (kind + range
/// only).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct slang_evaluated_dimension {
    pub kind: slang_dimension_kind,
    pub bounds: slang_constant_range,
}

/// The floating-point kind of a real/shortreal/realtime type. Mirrors
/// `slang::ast::FloatingType::Kind`.
pub type slang_float_kind = c_uint;
pub const SLANG_FLOAT_REAL: slang_float_kind = 0;
pub const SLANG_FLOAT_SHORT_REAL: slang_float_kind = 1;
pub const SLANG_FLOAT_REAL_TIME: slang_float_kind = 2;

/// The kind of a `ScalarType` (`bit`/`logic`/`reg`). Mirrors
/// `slang::ast::ScalarType::Kind`.
pub type slang_scalar_kind = c_uint;
pub const SLANG_SCALAR_BIT: slang_scalar_kind = 0;
pub const SLANG_SCALAR_LOGIC: slang_scalar_kind = 1;
pub const SLANG_SCALAR_REG: slang_scalar_kind = 2;

/// Bits returned by `slang_type_integral_flags`. Values match
/// `slang::ast::IntegralFlags` exactly (a direct bitmask).
pub type slang_integral_flags = c_uint;
pub const SLANG_INTEGRAL_UNSIGNED: slang_integral_flags = 0;
pub const SLANG_INTEGRAL_SIGNED: slang_integral_flags = 1;
pub const SLANG_INTEGRAL_FOUR_STATE: slang_integral_flags = 2;
pub const SLANG_INTEGRAL_REG: slang_integral_flags = 4;

/// The kind restriction a `typedef` forward declaration places on the type it
/// resolves to. Mirrors `slang::ast::ForwardTypeRestriction`.
pub type slang_forward_type_restriction = c_uint;
pub const SLANG_FORWARD_TYPE_NONE: slang_forward_type_restriction = 0;
pub const SLANG_FORWARD_TYPE_ENUM: slang_forward_type_restriction = 1;
pub const SLANG_FORWARD_TYPE_STRUCT: slang_forward_type_restriction = 2;
pub const SLANG_FORWARD_TYPE_UNION: slang_forward_type_restriction = 3;
pub const SLANG_FORWARD_TYPE_CLASS: slang_forward_type_restriction = 4;
pub const SLANG_FORWARD_TYPE_INTERFACE_CLASS: slang_forward_type_restriction = 5;

/// The kind of a predefined integer type (`shortint`/`int`/`longint`/`byte`/
/// `integer`/`time`). Mirrors `slang::ast::PredefinedIntegerType::Kind`.
pub type slang_predefined_integer_kind = c_uint;
pub const SLANG_PREDEFINED_INTEGER_SHORTINT: slang_predefined_integer_kind = 0;
pub const SLANG_PREDEFINED_INTEGER_INT: slang_predefined_integer_kind = 1;
pub const SLANG_PREDEFINED_INTEGER_LONGINT: slang_predefined_integer_kind = 2;
pub const SLANG_PREDEFINED_INTEGER_BYTE: slang_predefined_integer_kind = 3;
pub const SLANG_PREDEFINED_INTEGER_INTEGER: slang_predefined_integer_kind = 4;
pub const SLANG_PREDEFINED_INTEGER_TIME: slang_predefined_integer_kind = 5;

/// The full set of net-type kinds. Unlike `slang_net_type_kind`'s
/// keyword-lookup subset, this also covers the error placeholder (Unknown)
/// and user-defined nettypes. Mirrors `slang::ast::NetType::NetKind`.
pub type slang_net_kind = c_uint;
pub const SLANG_NET_UNKNOWN: slang_net_kind = 0;
pub const SLANG_NET_WIRE: slang_net_kind = 1;
pub const SLANG_NET_WAND: slang_net_kind = 2;
pub const SLANG_NET_WOR: slang_net_kind = 3;
pub const SLANG_NET_TRI: slang_net_kind = 4;
pub const SLANG_NET_TRIAND: slang_net_kind = 5;
pub const SLANG_NET_TRIOR: slang_net_kind = 6;
pub const SLANG_NET_TRI0: slang_net_kind = 7;
pub const SLANG_NET_TRI1: slang_net_kind = 8;
pub const SLANG_NET_TRIREG: slang_net_kind = 9;
pub const SLANG_NET_SUPPLY0: slang_net_kind = 10;
pub const SLANG_NET_SUPPLY1: slang_net_kind = 11;
pub const SLANG_NET_UWIRE: slang_net_kind = 12;
pub const SLANG_NET_INTERCONNECT: slang_net_kind = 13;
pub const SLANG_NET_USER_DEFINED: slang_net_kind = 14;

/// A member visibility modifier. Mirrors `slang::ast::Visibility`.
pub type slang_visibility = c_uint;
pub const SLANG_VISIBILITY_PUBLIC: slang_visibility = 0;
pub const SLANG_VISIBILITY_PROTECTED: slang_visibility = 1;
pub const SLANG_VISIBILITY_LOCAL: slang_visibility = 2;

/// A `rand`/`randc` mode, as declared on a `rand`/`randc` class property.
/// Mirrors `slang::ast::RandMode`.
pub type slang_rand_mode = c_uint;
pub const SLANG_RAND_MODE_NONE: slang_rand_mode = 0;
pub const SLANG_RAND_MODE_RAND: slang_rand_mode = 1;
pub const SLANG_RAND_MODE_RANDC: slang_rand_mode = 2;

/// Bits in `slang_symbol_method_prototype_flags`'s result. Values match
/// `slang::ast::MethodFlags` exactly (a direct bitmask).
pub type slang_method_flag = c_uint;
pub const SLANG_METHOD_NONE: slang_method_flag = 0;
pub const SLANG_METHOD_VIRTUAL: slang_method_flag = 1 << 0;
pub const SLANG_METHOD_PURE: slang_method_flag = 1 << 1;
pub const SLANG_METHOD_STATIC: slang_method_flag = 1 << 2;
pub const SLANG_METHOD_CONSTRUCTOR: slang_method_flag = 1 << 3;
pub const SLANG_METHOD_INTERFACE_EXTERN: slang_method_flag = 1 << 4;
pub const SLANG_METHOD_MODPORT_IMPORT: slang_method_flag = 1 << 5;
pub const SLANG_METHOD_MODPORT_EXPORT: slang_method_flag = 1 << 6;
pub const SLANG_METHOD_DPI_IMPORT: slang_method_flag = 1 << 7;
pub const SLANG_METHOD_DPI_CONTEXT: slang_method_flag = 1 << 8;
pub const SLANG_METHOD_BUILT_IN: slang_method_flag = 1 << 9;
pub const SLANG_METHOD_RANDOMIZE: slang_method_flag = 1 << 10;
pub const SLANG_METHOD_FORK_JOIN: slang_method_flag = 1 << 11;
pub const SLANG_METHOD_DEFAULTED_SUPER_ARG: slang_method_flag = 1 << 12;
pub const SLANG_METHOD_INITIAL: slang_method_flag = 1 << 13;
pub const SLANG_METHOD_EXTENDS: slang_method_flag = 1 << 14;
pub const SLANG_METHOD_FINAL: slang_method_flag = 1 << 15;
pub const SLANG_METHOD_PRE_POST_RANDOMIZE: slang_method_flag = 1 << 16;

#[repr(C)]
/// An opaque `slang::ast::MethodPrototypeSymbol::ExternImpl` node. Borrowed;
/// valid as long as the owning compilation is alive.
pub struct slang_extern_impl_t {
    _private: [u8; 0],
}
pub type slang_extern_impl = *mut slang_extern_impl_t;

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
pub struct slang_lvalue_t {
    _private: [u8; 0],
}
pub type slang_lvalue = *mut slang_lvalue_t;

#[repr(C)]
pub struct slang_type_printer_t {
    _private: [u8; 0],
}
pub type slang_type_printer = *mut slang_type_printer_t;

/// Selects a style for anonymous (unnamed, e.g. an inline unpacked struct)
/// types in printed output. Mirrors
/// `slang::ast::TypePrintingOptions::AnonymousTypeStyle`.
pub type slang_anonymous_type_style = c_uint;
pub const SLANG_ANON_TYPE_SYSTEM_NAME: slang_anonymous_type_style = 0;
pub const SLANG_ANON_TYPE_FRIENDLY_NAME: slang_anonymous_type_style = 1;

/// A plain-old-data mirror of a subset of `slang::ast::TypePrintingOptions`'s
/// public fields. See `slang_type_printer_options` /
/// `slang_type_printer_set_options`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct slang_type_printing_options {
    pub elide_scope_names: bool,
    pub classes_as_links: bool,
    pub enums_as_links: bool,
    pub anonymous_type_style: slang_anonymous_type_style,
    pub has_quote_char: bool,
    pub quote_char: c_char,
    pub print_aka: bool,
    pub skip_scoped_type_names: bool,
    pub skip_type_defs: bool,
    pub full_enum_type: bool,
    pub typedefs_as_links: bool,
    pub print_integral_range: bool,
    pub friendly_member_char_limit: usize,
}

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

/// Bits in `slang_symbol_constraint_block_flags`'s result. Values match
/// `slang::ast::ConstraintBlockFlags` exactly (a direct bitmask).
pub type slang_constraint_block_flags = c_uint;
pub const SLANG_CONSTRAINT_BLOCK_NONE: slang_constraint_block_flags = 0;
pub const SLANG_CONSTRAINT_BLOCK_PURE: slang_constraint_block_flags = 1 << 1;
pub const SLANG_CONSTRAINT_BLOCK_STATIC: slang_constraint_block_flags = 1 << 2;
pub const SLANG_CONSTRAINT_BLOCK_EXTERN: slang_constraint_block_flags = 1 << 3;
pub const SLANG_CONSTRAINT_BLOCK_EXPLICIT_EXTERN: slang_constraint_block_flags = 1 << 4;
pub const SLANG_CONSTRAINT_BLOCK_INITIAL: slang_constraint_block_flags = 1 << 5;
pub const SLANG_CONSTRAINT_BLOCK_EXTENDS: slang_constraint_block_flags = 1 << 6;
pub const SLANG_CONSTRAINT_BLOCK_FINAL: slang_constraint_block_flags = 1 << 7;

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
#[derive(Clone, Copy, Debug, Default)]
pub struct slang_ast {
    pub ptr: *const c_void,
    pub compilation: slang_compilation,
    pub kind: u32,
    pub domain: u32,
}

/// One production element within a RandSeqProductionSymbol rule's prod list.
/// Deliberately a separate type from `slang_ast` (see slang.h).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct slang_randseq_prod {
    pub ptr: *const c_void,
    pub compilation: slang_compilation,
    pub kind: u32,
    pub reserved_: u32,
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
    assert!(size_of::<slang_randseq_prod>() == 24);
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

    assert!(offset_of!(slang_randseq_prod, ptr) == 0);
    assert!(offset_of!(slang_randseq_prod, compilation) == 8);
    assert!(offset_of!(slang_randseq_prod, kind) == 16);
    assert!(offset_of!(slang_randseq_prod, reserved_) == 20);

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

/// The default lifetime (`automatic` or `static`) for variables declared
/// within a definition or subroutine. Mirrors `slang::ast::VariableLifetime`.
pub type slang_variable_lifetime = c_uint;
pub const SLANG_VARIABLE_LIFETIME_AUTOMATIC: slang_variable_lifetime = 0;
pub const SLANG_VARIABLE_LIFETIME_STATIC: slang_variable_lifetime = 1;

/// Bits in `slang_symbol_variable_flags`'s result. Values match
/// `slang::ast::VariableFlags` exactly (a direct bitmask).
pub type slang_variable_flags = c_uint;
pub const SLANG_VARIABLE_FLAG_NONE: slang_variable_flags = 0;
pub const SLANG_VARIABLE_FLAG_CONST: slang_variable_flags = 1 << 0;
pub const SLANG_VARIABLE_FLAG_COMPILER_GENERATED: slang_variable_flags = 1 << 1;
pub const SLANG_VARIABLE_FLAG_IMMUTABLE_COVERAGE_OPTION: slang_variable_flags = 1 << 2;
pub const SLANG_VARIABLE_FLAG_COVERAGE_SAMPLE_FORMAL: slang_variable_flags = 1 << 3;
pub const SLANG_VARIABLE_FLAG_CHECKER_FREE_VARIABLE: slang_variable_flags = 1 << 4;
pub const SLANG_VARIABLE_FLAG_REF_STATIC: slang_variable_flags = 1 << 5;

/// The drive setting applied to an unconnected net within a definition.
/// Mirrors `slang::ast::UnconnectedDrive`.
pub type slang_unconnected_drive = c_uint;
pub const SLANG_UNCONNECTED_DRIVE_NONE: slang_unconnected_drive = 0;
pub const SLANG_UNCONNECTED_DRIVE_PULL0: slang_unconnected_drive = 1;
pub const SLANG_UNCONNECTED_DRIVE_PULL1: slang_unconnected_drive = 2;

/// The kind of elaboration-time system task. Mirrors
/// `slang::ast::ElabSystemTaskKind`.
pub type slang_elab_system_task_kind = c_uint;
pub const SLANG_ELAB_SYSTEM_TASK_FATAL: slang_elab_system_task_kind = 0;
pub const SLANG_ELAB_SYSTEM_TASK_ERROR: slang_elab_system_task_kind = 1;
pub const SLANG_ELAB_SYSTEM_TASK_WARNING: slang_elab_system_task_kind = 2;
pub const SLANG_ELAB_SYSTEM_TASK_INFO: slang_elab_system_task_kind = 3;
pub const SLANG_ELAB_SYSTEM_TASK_STATIC_ASSERT: slang_elab_system_task_kind = 4;

/// An assertion-item / clocking-var argument direction. Mirrors
/// `slang::ast::ArgumentDirection`. `SLANG_ARGUMENT_DIRECTION_NONE` is not
/// one of slang's own enumerators: it encodes an empty
/// `std::optional<ArgumentDirection>`.
pub type slang_argument_direction = c_uint;
pub const SLANG_ARGUMENT_DIRECTION_IN: slang_argument_direction = 0;
pub const SLANG_ARGUMENT_DIRECTION_OUT: slang_argument_direction = 1;
pub const SLANG_ARGUMENT_DIRECTION_INOUT: slang_argument_direction = 2;
pub const SLANG_ARGUMENT_DIRECTION_REF: slang_argument_direction = 3;
pub const SLANG_ARGUMENT_DIRECTION_NONE: slang_argument_direction = 4;

/// Which branch of a conditional or loop generate construct produced a
/// given GenerateBlock symbol. Mirrors `slang::ast::GenerateBranchKind`.
pub type slang_generate_branch_kind = c_uint;
pub const SLANG_GENERATE_BRANCH_IF_TRUE: slang_generate_branch_kind = 0;
pub const SLANG_GENERATE_BRANCH_IF_FALSE: slang_generate_branch_kind = 1;
pub const SLANG_GENERATE_BRANCH_CASE_ITEM: slang_generate_branch_kind = 2;
pub const SLANG_GENERATE_BRANCH_CASE_DEFAULT: slang_generate_branch_kind = 3;
pub const SLANG_GENERATE_BRANCH_LOOP_ITERATION: slang_generate_branch_kind = 4;
pub const SLANG_GENERATE_BRANCH_ILLEGAL_UNCONDITIONAL: slang_generate_branch_kind = 5;

/// An edge kind for a clocking skew specification. Mirrors
/// `slang::ast::EdgeKind`.
pub type slang_edge_kind = c_uint;
pub const SLANG_EDGE_NONE: slang_edge_kind = 0;
pub const SLANG_EDGE_POSEDGE: slang_edge_kind = 1;
pub const SLANG_EDGE_NEGEDGE: slang_edge_kind = 2;
pub const SLANG_EDGE_BOTHEDGES: slang_edge_kind = 3;

/// A clocking-block input/output skew specification: an edge plus an
/// optional delay (`delay` is a node of domain `SLANG_AST_TIMING_CONTROL`,
/// null if none was specified). Mirrors `slang::ast::ClockingSkew`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct slang_clocking_skew {
    pub edge: slang_edge_kind,
    pub delay: slang_ast,
}

/// A net/gate drive strength level, independent of which value it drives.
/// Mirrors `slang::ast::DriveStrength`.
pub type slang_drive_strength = c_uint;
pub const SLANG_DRIVE_STRENGTH_SUPPLY: slang_drive_strength = 0;
pub const SLANG_DRIVE_STRENGTH_STRONG: slang_drive_strength = 1;
pub const SLANG_DRIVE_STRENGTH_PULL: slang_drive_strength = 2;
pub const SLANG_DRIVE_STRENGTH_WEAK: slang_drive_strength = 3;
pub const SLANG_DRIVE_STRENGTH_HIGHZ: slang_drive_strength = 4;

/// A pair of optional drive strengths (strength used driving 0 / driving 1).
/// Mirrors `std::pair<std::optional<slang::ast::DriveStrength>,
/// std::optional<slang::ast::DriveStrength>>`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct slang_drive_strength_pair {
    pub has_strength0: bool,
    pub strength0: slang_drive_strength,
    pub has_strength1: bool,
    pub strength1: slang_drive_strength,
}

/// A user-defined primitive (UDP) port's declared direction. Mirrors
/// `slang::ast::PrimitivePortDirection`.
pub type slang_primitive_port_direction = c_uint;
pub const SLANG_PRIMITIVE_PORT_DIRECTION_IN: slang_primitive_port_direction = 0;
pub const SLANG_PRIMITIVE_PORT_DIRECTION_OUT: slang_primitive_port_direction = 1;
pub const SLANG_PRIMITIVE_PORT_DIRECTION_OUT_REG: slang_primitive_port_direction = 2;
pub const SLANG_PRIMITIVE_PORT_DIRECTION_INOUT: slang_primitive_port_direction = 3;

/// The kind of gate primitive a Primitive symbol represents. Mirrors the
/// nested `slang::ast::PrimitiveSymbol::PrimitiveKind` enum.
pub type slang_primitive_kind = c_uint;
pub const SLANG_PRIMITIVE_KIND_USER_DEFINED: slang_primitive_kind = 0;
pub const SLANG_PRIMITIVE_KIND_FIXED: slang_primitive_kind = 1;
pub const SLANG_PRIMITIVE_KIND_N_INPUT: slang_primitive_kind = 2;
pub const SLANG_PRIMITIVE_KIND_N_OUTPUT: slang_primitive_kind = 3;
pub const SLANG_PRIMITIVE_KIND_BI_DI_SWITCH: slang_primitive_kind = 4;

/// The vectored/scalared expansion hint on a net declaration. Mirrors the
/// nested `slang::ast::NetSymbol::ExpansionHint` enum.
pub type slang_expansion_hint = c_uint;
pub const SLANG_EXPANSION_HINT_NONE: slang_expansion_hint = 0;
pub const SLANG_EXPANSION_HINT_VECTORED: slang_expansion_hint = 1;
pub const SLANG_EXPANSION_HINT_SCALARED: slang_expansion_hint = 2;

/// A `trireg` net's charge strength level. Mirrors
/// `slang::ast::ChargeStrength`. `SLANG_CHARGE_STRENGTH_NONE` is not one of
/// slang's own enumerators: it encodes an empty
/// `std::optional<ChargeStrength>`.
pub type slang_charge_strength = c_uint;
pub const SLANG_CHARGE_STRENGTH_SMALL: slang_charge_strength = 0;
pub const SLANG_CHARGE_STRENGTH_MEDIUM: slang_charge_strength = 1;
pub const SLANG_CHARGE_STRENGTH_LARGE: slang_charge_strength = 2;
pub const SLANG_CHARGE_STRENGTH_NONE: slang_charge_strength = 3;

/// How a `TransRangeList` repeats within a `bins` trans-set item. Mirrors
/// `slang::ast::CoverageBinSymbol::TransRangeList::RepeatKind`.
pub type slang_repeat_kind = c_uint;
pub const SLANG_REPEAT_KIND_NONE: slang_repeat_kind = 0;
pub const SLANG_REPEAT_KIND_CONSECUTIVE: slang_repeat_kind = 1;
pub const SLANG_REPEAT_KIND_NONCONSECUTIVE: slang_repeat_kind = 2;
pub const SLANG_REPEAT_KIND_GOTO: slang_repeat_kind = 3;

/// A coverage bin's kind (`bins`, `illegal_bins`, or `ignore_bins`). Mirrors
/// `slang::ast::CoverageBinSymbol::BinKind`.
pub type slang_coverage_bin_kind = c_uint;
pub const SLANG_COVERAGE_BIN_BINS: slang_coverage_bin_kind = 0;
pub const SLANG_COVERAGE_BIN_ILLEGAL_BINS: slang_coverage_bin_kind = 1;
pub const SLANG_COVERAGE_BIN_IGNORE_BINS: slang_coverage_bin_kind = 2;

// ---- Compilation: DPI exports ------------------------------------------------

/// A DPI export directive. Position struct: valid while `compilation` is alive.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct slang_dpi_export {
    pub ptr: *const c_void,
    pub compilation: slang_compilation,
}

// ---- Compilation: definition lookup ------------------------------------------

#[repr(C)]
pub struct slang_config_rule_t {
    _private: [u8; 0],
}
pub type slang_config_rule = *mut slang_config_rule_t;

/// The result of a definition lookup. Position struct: valid while the owning
/// compilation is alive.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct slang_definition_lookup_result {
    pub definition: slang_ast,
    pub config_root: slang_ast,
    pub config_rule: slang_config_rule,
}

// ---- Lookup: LookupLocation / LookupResult -----------------------------------

/// A position within a scope's member list (`slang::ast::LookupLocation`).
/// Trivially copyable value struct.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct slang_lookup_location {
    pub scope: slang_ast,
    pub index: u32,
}

#[repr(C)]
pub struct slang_lookup_result_t {
    _private: [u8; 0],
}
/// Owned by the caller; free with `slang_lookup_result_destroy`.
pub type slang_lookup_result = *mut slang_lookup_result_t;

/// Bits in `slang_lookup_result_flags`'s result. Values match
/// `slang::ast::LookupResultFlags` exactly (a direct bitmask).
pub type slang_lookup_result_flag = c_uint;
pub const SLANG_LOOKUP_RESULT_NONE: slang_lookup_result_flag = 0;
pub const SLANG_LOOKUP_RESULT_WAS_IMPORTED: slang_lookup_result_flag = 1 << 0;
pub const SLANG_LOOKUP_RESULT_IS_HIERARCHICAL: slang_lookup_result_flag = 1 << 1;
pub const SLANG_LOOKUP_RESULT_SUPPRESS_UNDECLARED: slang_lookup_result_flag = 1 << 2;
pub const SLANG_LOOKUP_RESULT_FROM_TYPE_PARAM: slang_lookup_result_flag = 1 << 3;
pub const SLANG_LOOKUP_RESULT_FROM_FORWARD_TYPEDEF: slang_lookup_result_flag = 1 << 4;
pub const SLANG_LOOKUP_RESULT_IFACE_PORT: slang_lookup_result_flag = 1 << 5;

// ---- Compilation: built-in types, options, libraries, diagnostics -----------

pub type slang_net_type_kind = c_uint;
pub const SLANG_NET_TYPE_WIRE: slang_net_type_kind = 0;
pub const SLANG_NET_TYPE_WAND: slang_net_type_kind = 1;
pub const SLANG_NET_TYPE_WOR: slang_net_type_kind = 2;
pub const SLANG_NET_TYPE_TRI: slang_net_type_kind = 3;
pub const SLANG_NET_TYPE_TRIAND: slang_net_type_kind = 4;
pub const SLANG_NET_TYPE_TRIOR: slang_net_type_kind = 5;
pub const SLANG_NET_TYPE_TRI0: slang_net_type_kind = 6;
pub const SLANG_NET_TYPE_TRI1: slang_net_type_kind = 7;
pub const SLANG_NET_TYPE_TRIREG: slang_net_type_kind = 8;
pub const SLANG_NET_TYPE_SUPPLY0: slang_net_type_kind = 9;
pub const SLANG_NET_TYPE_SUPPLY1: slang_net_type_kind = 10;
pub const SLANG_NET_TYPE_UWIRE: slang_net_type_kind = 11;
pub const SLANG_NET_TYPE_INTERCONNECT: slang_net_type_kind = 12;

/// A read-only snapshot of a compilation's options. Position struct:
/// trivially copyable, independent of the owning compilation.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct slang_compilation_options {
    pub flags: u32,
    pub max_instance_depth: u32,
    pub max_checker_instance_depth: u32,
    pub max_generate_steps: u32,
    pub max_constexpr_depth: u32,
    pub max_constexpr_steps: u32,
    pub max_constexpr_backtrace: u32,
    pub max_constant_size: u64,
    pub max_defparam_steps: u32,
    pub max_defparam_blocks: u32,
    pub max_instance_array: u32,
    pub max_enum_values: u32,
    pub max_recursive_class_specialization: u32,
    pub max_udp_coverage_notes: u32,
    pub error_limit: u32,
    pub typo_correction_limit: u32,
    pub min_typ_max: u32,
    pub language_version: u32,
}

/// A time scale: a base and precision unit+magnitude pair. Position struct.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct slang_time_scale {
    pub base_unit: u8,
    pub base_magnitude: u8,
    pub precision_unit: u8,
    pub precision_magnitude: u8,
}

/// A single unit+magnitude pair within a `slang_time_scale`. Position struct.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct slang_time_scale_value {
    pub unit: u8,
    pub magnitude: u8,
}

/// A single four-state logic bit (`slang::logic_t`'s own raw byte encoding:
/// 0/1 for a known bit, 0x80 for unknown 'x', 0x40 for high-impedance 'z').
/// Position struct.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct slang_logic_value {
    pub value: u8,
}

#[repr(C)]
pub struct slang_source_library_t {
    _private: [u8; 0],
}
/// A source library, borrowed from the owning compilation.
pub type slang_source_library = *mut slang_source_library_t;

#[repr(C)]
pub struct slang_system_subroutine_t {
    _private: [u8; 0],
}
/// A built-in or user-registered system task/function/method handler,
/// borrowed (outlives any compilation that can observe it).
pub type slang_system_subroutine = *mut slang_system_subroutine_t;

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

/// A handle to a single driver of a value (`slang::analysis::ValueDriver`).
/// Trivially copyable; `ptr` is null for "none".
#[repr(C)]
#[derive(Clone, Copy)]
pub struct slang_value_driver {
    pub ptr: *const c_void,
    pub analysis: slang_analysis,
}

/// Bits in `slang_value_driver_flags`'s result. Values match
/// `slang::analysis::DriverFlags` exactly (a direct bitmask, not curated).
pub type slang_driver_flags_raw = c_uint;
pub const SLANG_DRIVER_FLAG_INPUT_PORT: slang_driver_flags_raw = 1 << 0;
pub const SLANG_DRIVER_FLAG_OUTPUT_PORT: slang_driver_flags_raw = 1 << 1;
pub const SLANG_DRIVER_FLAG_CLOCK_VAR: slang_driver_flags_raw = 1 << 2;
pub const SLANG_DRIVER_FLAG_INITIALIZER: slang_driver_flags_raw = 1 << 3;
pub const SLANG_DRIVER_FLAG_FROM_SIDE_EFFECT: slang_driver_flags_raw = 1 << 4;
pub const SLANG_DRIVER_FLAG_HAS_OVERRIDE_RANGE: slang_driver_flags_raw = 1 << 5;
pub const SLANG_DRIVER_FLAG_VIA_INDIRECT_PORT: slang_driver_flags_raw = 1 << 6;

/// The kind of construct a driver originated from (`slang::analysis::
/// ValueDriver::source`).
pub type slang_driver_source = c_uint;
pub const SLANG_DRIVER_SOURCE_INITIAL: slang_driver_source = 0;
pub const SLANG_DRIVER_SOURCE_FINAL: slang_driver_source = 1;
pub const SLANG_DRIVER_SOURCE_ALWAYS: slang_driver_source = 2;
pub const SLANG_DRIVER_SOURCE_ALWAYS_COMB: slang_driver_source = 3;
pub const SLANG_DRIVER_SOURCE_ALWAYS_LATCH: slang_driver_source = 4;
pub const SLANG_DRIVER_SOURCE_ALWAYS_FF: slang_driver_source = 5;
pub const SLANG_DRIVER_SOURCE_SUBROUTINE: slang_driver_source = 6;
pub const SLANG_DRIVER_SOURCE_OTHER: slang_driver_source = 7;

/// A handle to the value path driven by a `slang_value_driver`
/// (`slang::ast::ValuePath`). Trivially copyable; `ptr` is null for "none".
#[repr(C)]
#[derive(Clone, Copy)]
pub struct slang_value_path {
    pub ptr: *const c_void,
    pub analysis: slang_analysis,
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

/// Opaque handle to a running dataflow analysis's own context, passed to
/// `slang_dfa_lattice`'s `on_case_begin` / `on_conditional_begin` /
/// `on_loop_begin` hooks. Valid only for the duration of the callback.
#[repr(C)]
pub struct slang_dfa_ctx_t {
    _private: [u8; 0],
}
pub type slang_dfa_ctx = *mut slang_dfa_ctx_t;

/// Opaque handle to an evaluation context (`slang::ast::EvalContext`) used
/// during a custom dataflow analysis. Valid only for the duration of the
/// callback that produced it.
#[repr(C)]
pub struct slang_eval_ctx_t {
    _private: [u8; 0],
}
pub type slang_eval_ctx = *mut slang_eval_ctx_t;

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
    pub on_case_begin:
        Option<unsafe extern "C" fn(user: *mut c_void, ctx: slang_dfa_ctx, stmt: slang_ast)>,
    pub on_conditional_begin:
        Option<unsafe extern "C" fn(user: *mut c_void, ctx: slang_dfa_ctx, stmt: slang_ast)>,
    pub on_loop_begin:
        Option<unsafe extern "C" fn(user: *mut c_void, ctx: slang_dfa_ctx, stmt: slang_ast)>,
}

/// A handle to an analyzed procedure (`slang::analysis::AnalyzedProcedure`).
/// Trivially copyable; `ptr` is null for "none".
#[repr(C)]
#[derive(Clone, Copy)]
pub struct slang_analyzed_procedure {
    pub ptr: *const c_void,
    pub analysis: slang_analysis,
}

/// A handle to an analyzed assertion (`slang::analysis::AnalyzedAssertion`).
/// Trivially copyable; `ptr` is null for "none".
#[repr(C)]
#[derive(Clone, Copy)]
pub struct slang_analyzed_assertion {
    pub ptr: *const c_void,
    pub analysis: slang_analysis,
}

/// A handle to one `@*` region's read set within an analyzed procedure
/// (`slang::analysis::AnalyzedProcedure::ImplicitEventReadSet`). Trivially
/// copyable; `ptr` is null for "none".
#[repr(C)]
#[derive(Clone, Copy)]
pub struct slang_implicit_event_read_set {
    pub ptr: *const c_void,
    pub analysis: slang_analysis,
}

/// A handle to one (symbol, bit-range) entry of an analyzed procedure's read
/// set (`slang::analysis::ReadRange`). Trivially copyable; `ptr` is null for
/// "none".
#[repr(C)]
#[derive(Clone, Copy)]
pub struct slang_read_range {
    pub ptr: *const c_void,
    pub analysis: slang_analysis,
}

/// A handle to an analyzed procedure's effective sensitivity list
/// (`slang::analysis::SensitivityList`). Trivially copyable; `ptr` is null for
/// "none".
#[repr(C)]
#[derive(Clone, Copy)]
pub struct slang_sensitivity_list {
    pub ptr: *const c_void,
    pub analysis: slang_analysis,
}

pub type slang_sensitivity_kind = c_uint;
pub const SLANG_SENSITIVITY_NONE: slang_sensitivity_kind = 0;
pub const SLANG_SENSITIVITY_EXPLICIT: slang_sensitivity_kind = 1;
pub const SLANG_SENSITIVITY_IMPLICIT: slang_sensitivity_kind = 2;
pub const SLANG_SENSITIVITY_DYNAMIC: slang_sensitivity_kind = 3;

// ---- Driver -----------------------------------------------------------------

#[repr(C)]
pub struct slang_driver_t {
    _private: [u8; 0],
}
pub type slang_driver = *mut slang_driver_t;

#[repr(C)]
pub struct slang_bag_t {
    _private: [u8; 0],
}
pub type slang_bag = *mut slang_bag_t;

pub type slang_option_kind = c_uint;
pub const SLANG_OPTION_FLAG: slang_option_kind = 0;
pub const SLANG_OPTION_INT: slang_option_kind = 1;
pub const SLANG_OPTION_STRING: slang_option_kind = 2;

#[repr(C)]
pub struct slang_parse_options_t {
    _private: [u8; 0],
}
pub type slang_parse_options = *mut slang_parse_options_t;

#[repr(C)]
pub struct slang_command_file_metadata_t {
    _private: [u8; 0],
}
/// Metadata for one command file processed by `slang_driver_process_command_files`.
/// Borrowed; a snapshot rebuilt (invalidating previous handles) on the next
/// such call on the same driver.
pub type slang_command_file_metadata = *mut slang_command_file_metadata_t;

#[repr(C)]
pub struct slang_diag_engine_t {
    _private: [u8; 0],
}
/// A driver's diagnostic engine. Borrowed; valid for the driver's lifetime.
pub type slang_diag_engine = *mut slang_diag_engine_t;

/// A snapshot of the analysis options a driver's configured flags would
/// produce. Position struct: trivially copyable.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct slang_analysis_options {
    pub flags: u32,
    pub max_case_analysis_steps: u32,
    pub max_loop_analysis_steps: u32,
}

/// Bitmask flags controlling `slang_driver_run_preprocessor`'s output.
pub type slang_preprocess_output_flag = c_uint;
pub const SLANG_PREPROCESS_INCLUDE_COMMENTS: slang_preprocess_output_flag = 1 << 0;
pub const SLANG_PREPROCESS_INCLUDE_DIRECTIVES: slang_preprocess_output_flag = 1 << 1;
pub const SLANG_PREPROCESS_OBFUSCATE_IDS: slang_preprocess_output_flag = 1 << 2;
pub const SLANG_PREPROCESS_USE_FIXED_OBFUSCATION_SEED: slang_preprocess_output_flag = 1 << 3;
pub const SLANG_PREPROCESS_INCLUDE_SOURCE_INFO: slang_preprocess_output_flag = 1 << 4;

#[repr(C)]
pub struct slang_source_loader_t {
    _private: [u8; 0],
}
/// A driver's source loader. Borrowed; valid for the driver's lifetime.
pub type slang_source_loader = *mut slang_source_loader_t;

#[repr(C)]
pub struct slang_source_options_t {
    _private: [u8; 0],
}
/// An owned `slang::driver::SourceOptions` value. Free with
/// `slang_source_options_destroy`.
pub type slang_source_options = *mut slang_source_options_t;

#[repr(C)]
pub struct slang_text_diag_client_t {
    _private: [u8; 0],
}
/// A driver's text diagnostic client. Borrowed; valid for the driver's lifetime.
pub type slang_text_diag_client = *mut slang_text_diag_client_t;

#[repr(C)]
pub struct slang_script_session_t {
    _private: [u8; 0],
}
/// A session for evaluating snippets of SystemVerilog and keeping state
/// across calls. Owned; free with `slang_script_session_destroy`.
pub type slang_script_session = *mut slang_script_session_t;

/// The interface instance (and, if applicable, modport) an InterfacePort
/// symbol is connected to (`slang::ast::InterfacePortSymbol::IfaceConn`).
/// Either half may be a null (ptr-less) `slang_ast`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct slang_iface_conn {
    pub instance: slang_ast,
    pub modport: slang_ast,
}

/// Mirrors `slang::ast::ProceduralBlockKind`.
pub type slang_procedural_block_kind = c_uint;
pub const SLANG_PROCEDURAL_BLOCK_KIND_INITIAL: slang_procedural_block_kind = 0;
pub const SLANG_PROCEDURAL_BLOCK_KIND_FINAL: slang_procedural_block_kind = 1;
pub const SLANG_PROCEDURAL_BLOCK_KIND_ALWAYS: slang_procedural_block_kind = 2;
pub const SLANG_PROCEDURAL_BLOCK_KIND_ALWAYS_COMB: slang_procedural_block_kind = 3;
pub const SLANG_PROCEDURAL_BLOCK_KIND_ALWAYS_LATCH: slang_procedural_block_kind = 4;
pub const SLANG_PROCEDURAL_BLOCK_KIND_ALWAYS_FF: slang_procedural_block_kind = 5;

/// Mirrors `slang::ast::TimingPathSymbol::ConnectionKind`.
pub type slang_timing_path_connection_kind = c_uint;
pub const SLANG_TIMING_PATH_CONNECTION_KIND_FULL: slang_timing_path_connection_kind = 0;
pub const SLANG_TIMING_PATH_CONNECTION_KIND_PARALLEL: slang_timing_path_connection_kind = 1;

/// Mirrors `slang::ast::TimingPathSymbol::Polarity`.
pub type slang_timing_path_polarity = c_uint;
pub const SLANG_TIMING_PATH_POLARITY_UNKNOWN: slang_timing_path_polarity = 0;
pub const SLANG_TIMING_PATH_POLARITY_POSITIVE: slang_timing_path_polarity = 1;
pub const SLANG_TIMING_PATH_POLARITY_NEGATIVE: slang_timing_path_polarity = 2;

/// Mirrors `slang::ast::PulseStyleKind`.
pub type slang_pulse_style_kind = c_uint;
pub const SLANG_PULSE_STYLE_KIND_ON_EVENT: slang_pulse_style_kind = 0;
pub const SLANG_PULSE_STYLE_KIND_ON_DETECT: slang_pulse_style_kind = 1;
pub const SLANG_PULSE_STYLE_KIND_SHOW_CANCELLED: slang_pulse_style_kind = 2;
pub const SLANG_PULSE_STYLE_KIND_NO_SHOW_CANCELLED: slang_pulse_style_kind = 3;

/// Mirrors `slang::ast::SystemTimingCheckKind`.
pub type slang_system_timing_check_kind = c_uint;
pub const SLANG_SYSTEM_TIMING_CHECK_KIND_UNKNOWN: slang_system_timing_check_kind = 0;
pub const SLANG_SYSTEM_TIMING_CHECK_KIND_SETUP: slang_system_timing_check_kind = 1;
pub const SLANG_SYSTEM_TIMING_CHECK_KIND_HOLD: slang_system_timing_check_kind = 2;
pub const SLANG_SYSTEM_TIMING_CHECK_KIND_SETUP_HOLD: slang_system_timing_check_kind = 3;
pub const SLANG_SYSTEM_TIMING_CHECK_KIND_RECOVERY: slang_system_timing_check_kind = 4;
pub const SLANG_SYSTEM_TIMING_CHECK_KIND_REMOVAL: slang_system_timing_check_kind = 5;
pub const SLANG_SYSTEM_TIMING_CHECK_KIND_REC_REM: slang_system_timing_check_kind = 6;
pub const SLANG_SYSTEM_TIMING_CHECK_KIND_SKEW: slang_system_timing_check_kind = 7;
pub const SLANG_SYSTEM_TIMING_CHECK_KIND_TIME_SKEW: slang_system_timing_check_kind = 8;
pub const SLANG_SYSTEM_TIMING_CHECK_KIND_FULL_SKEW: slang_system_timing_check_kind = 9;
pub const SLANG_SYSTEM_TIMING_CHECK_KIND_PERIOD: slang_system_timing_check_kind = 10;
pub const SLANG_SYSTEM_TIMING_CHECK_KIND_WIDTH: slang_system_timing_check_kind = 11;
pub const SLANG_SYSTEM_TIMING_CHECK_KIND_NO_CHANGE: slang_system_timing_check_kind = 12;

/// Mirrors the nested `slang::ast::RandSeqProductionSymbol::ProdKind` enum.
pub type slang_randseq_prod_kind = c_uint;
pub const SLANG_RANDSEQ_PROD_KIND_ITEM: slang_randseq_prod_kind = 0;
pub const SLANG_RANDSEQ_PROD_KIND_CODE_BLOCK: slang_randseq_prod_kind = 1;
pub const SLANG_RANDSEQ_PROD_KIND_IF_ELSE: slang_randseq_prod_kind = 2;
pub const SLANG_RANDSEQ_PROD_KIND_REPEAT: slang_randseq_prod_kind = 3;
pub const SLANG_RANDSEQ_PROD_KIND_CASE: slang_randseq_prod_kind = 4;

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
    pub fn slang_compilation_create_from_bag(
        bag: slang_bag,
        extra_flags: u32,
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
    pub fn slang_compilation_unfreeze(comp: slang_compilation, err: *mut slang_error);
    pub fn slang_compilation_root(comp: slang_compilation, err: *mut slang_error) -> slang_ast;
    pub fn slang_symbol_root_top_instance_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_root_top_instance(sym: slang_ast, index: u32) -> slang_ast;
    pub fn slang_symbol_root_compilation_unit_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_root_compilation_unit(sym: slang_ast, index: u32) -> slang_ast;
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
    pub fn slang_compilation_get_bit_type(comp: slang_compilation) -> slang_ast;
    pub fn slang_compilation_get_byte_type(comp: slang_compilation) -> slang_ast;
    pub fn slang_compilation_unit_for_syntax(
        comp: slang_compilation,
        syntax: slang_node,
    ) -> slang_ast;
    pub fn slang_compilation_unit_count(comp: slang_compilation) -> u32;
    pub fn slang_compilation_unit_at(comp: slang_compilation, index: u32) -> slang_ast;
    pub fn slang_compilation_create_script_scope(
        comp: slang_compilation,
        err: *mut slang_error,
    ) -> slang_ast;
    pub fn slang_compilation_add_diagnostics(
        comp: slang_compilation,
        diags: slang_diagnostics,
        err: *mut slang_error,
    );

    // Compilation: DPI exports
    pub fn slang_dpi_export_is_null(exp: slang_dpi_export) -> bool;
    pub fn slang_compilation_dpi_export_count(comp: slang_compilation) -> u32;
    pub fn slang_compilation_dpi_export(comp: slang_compilation, index: u32) -> slang_dpi_export;
    pub fn slang_dpi_export_subroutine(exp: slang_dpi_export) -> slang_ast;
    pub fn slang_dpi_export_c_identifier(exp: slang_dpi_export) -> slang_str;
    pub fn slang_dpi_export_syntax(exp: slang_dpi_export) -> slang_node;

    // Compilation: definition lookup
    pub fn slang_compilation_try_get_definition(
        comp: slang_compilation,
        name: *const c_char,
        name_len: usize,
        scope: slang_ast,
    ) -> slang_definition_lookup_result;

    // Compilation: built-in types, options, libraries, diagnostics
    pub fn slang_compilation_get_int_type(comp: slang_compilation) -> slang_ast;
    pub fn slang_compilation_get_integer_type(comp: slang_compilation) -> slang_ast;
    pub fn slang_compilation_get_logic_type(comp: slang_compilation) -> slang_ast;
    pub fn slang_compilation_get_real_type(comp: slang_compilation) -> slang_ast;
    pub fn slang_compilation_get_short_real_type(comp: slang_compilation) -> slang_ast;
    pub fn slang_compilation_get_error_type(comp: slang_compilation) -> slang_ast;
    pub fn slang_compilation_get_null_type(comp: slang_compilation) -> slang_ast;
    pub fn slang_compilation_get_gate_type(
        comp: slang_compilation,
        name: *const c_char,
        name_len: usize,
    ) -> slang_ast;
    pub fn slang_compilation_get_package(
        comp: slang_compilation,
        name: *const c_char,
        name_len: usize,
    ) -> slang_ast;
    pub fn slang_compilation_get_net_type(
        comp: slang_compilation,
        kind: slang_net_type_kind,
    ) -> slang_ast;
    pub fn slang_compilation_get_options(comp: slang_compilation) -> slang_compilation_options;
    pub fn slang_compilation_get_default_time_scale(
        comp: slang_compilation,
        out: *mut slang_time_scale,
    ) -> bool;
    pub fn slang_time_scale_base(ts: slang_time_scale) -> slang_time_scale_value;
    pub fn slang_time_scale_precision(ts: slang_time_scale) -> slang_time_scale_value;
    pub fn slang_time_scale_apply(
        ts: slang_time_scale,
        value: f64,
        unit: u8,
        round_to_precision: bool,
    ) -> f64;
    pub fn slang_time_scale_from_string(
        str_: *const c_char,
        str_len: usize,
        out: *mut slang_time_scale,
    ) -> bool;
    pub fn slang_time_scale_value_unit(v: slang_time_scale_value) -> u8;
    pub fn slang_time_scale_value_magnitude(v: slang_time_scale_value) -> u8;
    pub fn slang_time_scale_value_from_literal(
        value: f64,
        unit: u8,
        out: *mut slang_time_scale_value,
    ) -> bool;
    pub fn slang_time_scale_value_from_string(
        str_: *const c_char,
        str_len: usize,
        out: *mut slang_time_scale_value,
    ) -> bool;
    pub fn slang_compilation_top_module_count(comp: slang_compilation) -> u32;
    pub fn slang_compilation_top_module_at(comp: slang_compilation, index: u32) -> slang_str;
    pub fn slang_compilation_param_override_count(comp: slang_compilation) -> u32;
    pub fn slang_compilation_param_override_at(comp: slang_compilation, index: u32) -> slang_str;
    pub fn slang_compilation_default_liblist_count(comp: slang_compilation) -> u32;
    pub fn slang_compilation_default_liblist_at(comp: slang_compilation, index: u32) -> slang_str;
    pub fn slang_source_library_name(lib: slang_source_library) -> slang_str;
    pub fn slang_source_library_priority(lib: slang_source_library) -> i32;
    pub fn slang_source_library_is_default(lib: slang_source_library) -> bool;
    pub fn slang_compilation_get_default_library(comp: slang_compilation) -> slang_source_library;
    pub fn slang_compilation_get_source_library(
        comp: slang_compilation,
        name: *const c_char,
        name_len: usize,
    ) -> slang_source_library;
    pub fn slang_compilation_get_parse_diagnostics(
        comp: slang_compilation,
        err: *mut slang_error,
    ) -> slang_diagnostics;
    pub fn slang_compilation_get_semantic_diagnostics(
        comp: slang_compilation,
        err: *mut slang_error,
    ) -> slang_diagnostics;

    // Compilation: state, syntax trees, name parsing
    pub fn slang_compilation_is_finalized(comp: slang_compilation) -> bool;
    pub fn slang_compilation_is_elaborated(comp: slang_compilation) -> bool;
    pub fn slang_compilation_has_issued_errors(comp: slang_compilation) -> bool;
    pub fn slang_compilation_has_fatal_errors(comp: slang_compilation) -> bool;
    pub fn slang_compilation_syntax_tree_count(comp: slang_compilation) -> u32;
    pub fn slang_compilation_syntax_tree_at(
        comp: slang_compilation,
        index: u32,
    ) -> slang_syntax_tree;
    pub fn slang_compilation_parse_name(
        comp: slang_compilation,
        name: *const c_char,
        name_len: usize,
        err: *mut slang_error,
    ) -> slang_node;
    pub fn slang_compilation_try_parse_name(
        comp: slang_compilation,
        name: *const c_char,
        name_len: usize,
        diags_out: *mut slang_diagnostics,
        err: *mut slang_error,
    ) -> slang_node;

    // Compilation: more built-in types and system methods
    pub fn slang_compilation_get_std_package(comp: slang_compilation) -> slang_ast;
    pub fn slang_compilation_get_string_type(comp: slang_compilation) -> slang_ast;
    pub fn slang_compilation_get_void_type(comp: slang_compilation) -> slang_ast;
    pub fn slang_compilation_get_unbounded_type(comp: slang_compilation) -> slang_ast;
    pub fn slang_compilation_get_type_ref_type(comp: slang_compilation) -> slang_ast;
    pub fn slang_compilation_get_unsigned_int_type(comp: slang_compilation) -> slang_ast;
    pub fn slang_compilation_get_wire_net_type(comp: slang_compilation) -> slang_ast;
    pub fn slang_system_subroutine_name(sub: slang_system_subroutine) -> slang_str;
    pub fn slang_system_subroutine_is_task(sub: slang_system_subroutine) -> bool;
    pub fn slang_compilation_get_system_method(
        comp: slang_compilation,
        type_kind: u32,
        name: *const c_char,
        name_len: usize,
    ) -> slang_system_subroutine;
    pub fn slang_system_subroutine_allow_empty_argument(
        sub: slang_system_subroutine,
        arg_index: u32,
    ) -> bool;
    pub fn slang_system_subroutine_allow_clocking_argument(
        sub: slang_system_subroutine,
        arg_index: u32,
    ) -> bool;
    pub fn slang_expr_call_system_subroutine(node: slang_ast) -> slang_system_subroutine;
    pub fn slang_system_subroutine_check_arguments(
        call: slang_ast,
        err: *mut slang_error,
    ) -> slang_ast;
    pub fn slang_system_subroutine_bind_argument(
        call: slang_ast,
        arg_index: u32,
        err: *mut slang_error,
    ) -> slang_ast;
    pub fn slang_system_subroutine_eval(call: slang_ast, err: *mut slang_error) -> slang_constant;
    pub fn slang_system_subroutine_has_output_args(sub: slang_system_subroutine) -> bool;
    pub fn slang_system_subroutine_kind(sub: slang_system_subroutine) -> u32;
    pub fn slang_system_subroutine_known_name_id(sub: slang_system_subroutine) -> u32;
    pub fn slang_system_subroutine_with_clause_mode(sub: slang_system_subroutine) -> u32;
    pub fn slang_system_subroutine_kind_str(sub: slang_system_subroutine) -> slang_str;
    pub fn slang_system_subroutine_bad_arg(
        call: slang_ast,
        arg_index: u32,
        err: *mut slang_error,
    ) -> slang_ast;
    pub fn slang_system_subroutine_check_arg_count(
        call: slang_ast,
        is_method: bool,
        min: u32,
        max: u32,
        err: *mut slang_error,
    ) -> bool;
    pub fn slang_system_subroutine_no_hierarchical(
        call: slang_ast,
        arg_index: u32,
        err: *mut slang_error,
    ) -> bool;
    pub fn slang_system_subroutine_not_const(call: slang_ast, err: *mut slang_error) -> bool;
    pub fn slang_system_subroutine_unevaluated_context_clears_static_initializer(
        call: slang_ast,
        err: *mut slang_error,
    ) -> bool;

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
    pub fn slang_symbol_lexical_path(symbol: slang_ast, err: *mut slang_error) -> slang_str;
    pub fn slang_symbol_is_scope(symbol: slang_ast) -> bool;
    pub fn slang_symbol_is_type(symbol: slang_ast) -> bool;
    pub fn slang_symbol_is_value(symbol: slang_ast) -> bool;
    pub fn slang_symbol_has_declared_type(symbol: slang_ast) -> bool;
    pub fn slang_symbol_declaring_definition(symbol: slang_ast) -> slang_ast;
    pub fn slang_symbol_source_library(symbol: slang_ast) -> slang_source_library;
    pub fn slang_symbol_rand_mode(sym: slang_ast) -> slang_rand_mode;
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
    pub fn slang_scope_get_compilation(scope: slang_ast) -> slang_compilation;
    pub fn slang_scope_get_compilation_unit(scope: slang_ast) -> slang_ast;
    pub fn slang_scope_get_containing_instance(scope: slang_ast) -> slang_ast;
    pub fn slang_scope_get_default_net_type(scope: slang_ast) -> slang_ast;
    pub fn slang_scope_get_time_scale(scope: slang_ast, out: *mut slang_time_scale) -> bool;
    pub fn slang_scope_is_procedural_context(scope: slang_ast) -> bool;
    pub fn slang_scope_is_uninstantiated(scope: slang_ast) -> bool;
    pub fn slang_value_type(symbol: slang_ast, err: *mut slang_error) -> slang_ast;
    pub fn slang_value_initializer(symbol: slang_ast, err: *mut slang_error) -> slang_ast;
    pub fn slang_declared_type_type(sym: slang_ast) -> slang_ast;
    pub fn slang_declared_type_initializer(sym: slang_ast) -> slang_ast;
    pub fn slang_declared_type_initializer_location(sym: slang_ast) -> slang_loc;
    pub fn slang_declared_type_initializer_syntax(sym: slang_ast) -> slang_node;
    pub fn slang_declared_type_type_syntax(sym: slang_ast) -> slang_node;
    pub fn slang_declared_type_is_evaluating(sym: slang_ast) -> bool;
    pub fn slang_declared_type_resolved_dimensions(
        sym: slang_ast,
        out: *mut slang_evaluated_dimension,
        cap: u32,
        err: *mut slang_error,
    ) -> u32;
    pub fn slang_instance_body(instance: slang_ast) -> slang_ast;
    pub fn slang_instance_definition(instance: slang_ast) -> slang_ast;
    pub fn slang_instance_parameter_count(instance: slang_ast) -> u32;
    pub fn slang_instance_parameter(instance: slang_ast, index: u32) -> slang_ast;
    pub fn slang_instance_is_module(instance: slang_ast) -> bool;
    pub fn slang_instance_is_interface(instance: slang_ast) -> bool;
    pub fn slang_instance_port_connection_count(instance: slang_ast) -> u32;
    pub fn slang_instance_port_connection_port(instance: slang_ast, index: u32) -> slang_ast;
    pub fn slang_instance_port_connection_expression(instance: slang_ast, index: u32) -> slang_ast;
    pub fn slang_instance_port_connection_is_implicit(instance: slang_ast, index: u32) -> bool;
    pub fn slang_instance_port_connection_is_wildcard(instance: slang_ast, index: u32) -> bool;
    pub fn slang_symbol_instance_body_parent_instance(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_instance_body_definition(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_instance_body_port_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_instance_body_port(sym: slang_ast, index: u32) -> slang_ast;
    pub fn slang_symbol_instance_body_find_port(
        sym: slang_ast,
        name: *const c_char,
        name_len: usize,
    ) -> slang_ast;
    pub fn slang_symbol_instance_body_has_same_type(sym: slang_ast, other: slang_ast) -> bool;
    pub fn slang_symbol_instance_array_path_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_instance_array_path(sym: slang_ast, index: u32) -> u32;
    pub fn slang_symbol_instance_base_array_name(sym: slang_ast) -> slang_str;
    pub fn slang_symbol_instance_array_element_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_instance_array_element(sym: slang_ast, index: u32) -> slang_ast;
    pub fn slang_symbol_instance_array_range(sym: slang_ast) -> slang_constant_range;
    pub fn slang_symbol_instance_array_name(sym: slang_ast) -> slang_str;
    pub fn slang_parameter_value(parameter: slang_ast, err: *mut slang_error) -> slang_str;
    pub fn slang_definition_kind_of(definition: slang_ast) -> slang_definition_kind;
    pub fn slang_definition_cell_define(definition: slang_ast) -> bool;
    pub fn slang_definition_default_lifetime(definition: slang_ast) -> slang_variable_lifetime;
    pub fn slang_definition_unconnected_drive(definition: slang_ast) -> slang_unconnected_drive;
    pub fn slang_definition_time_scale(definition: slang_ast, out: *mut slang_time_scale) -> bool;
    pub fn slang_definition_kind_string(definition: slang_ast) -> slang_str;
    pub fn slang_definition_article_kind_string(definition: slang_ast) -> slang_str;
    pub fn slang_definition_instance_count(definition: slang_ast) -> u64;

    pub fn slang_symbol_assertion_port_direction(sym: slang_ast) -> slang_argument_direction;
    pub fn slang_symbol_assertion_port_is_local_var(sym: slang_ast) -> bool;
    pub fn slang_symbol_attribute_value(sym: slang_ast, err: *mut slang_error) -> slang_constant;
    pub fn slang_symbol_checker_instance_body_parent_instance(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_checker_port_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_checker_port(sym: slang_ast, index: u32) -> slang_ast;
    pub fn slang_symbol_checker_instance_connection_count(instance: slang_ast) -> u32;
    pub fn slang_symbol_checker_instance_connection_actual(
        instance: slang_ast,
        index: u32,
    ) -> slang_ast;
    pub fn slang_symbol_checker_instance_connection_attribute_count(
        instance: slang_ast,
        index: u32,
    ) -> u32;
    pub fn slang_symbol_checker_instance_connection_attribute(
        instance: slang_ast,
        index: u32,
        attr_index: u32,
    ) -> slang_ast;
    pub fn slang_symbol_checker_instance_connection_output_initial_expr(
        instance: slang_ast,
        index: u32,
    ) -> slang_ast;
    pub fn slang_symbol_class_property_visibility(sym: slang_ast) -> slang_visibility;
    pub fn slang_symbol_class_property_rand_mode(sym: slang_ast) -> slang_rand_mode;
    pub fn slang_clocking_skew_has_value(skew: slang_clocking_skew) -> bool;
    pub fn slang_symbol_clock_var_direction(sym: slang_ast) -> slang_argument_direction;
    pub fn slang_symbol_clock_var_input_skew(sym: slang_ast) -> slang_clocking_skew;
    pub fn slang_symbol_clock_var_output_skew(sym: slang_ast) -> slang_clocking_skew;
    pub fn slang_symbol_clocking_block_default_input_skew(sym: slang_ast) -> slang_clocking_skew;
    pub fn slang_symbol_clocking_block_default_output_skew(sym: slang_ast) -> slang_clocking_skew;
    pub fn slang_symbol_clocking_block_event(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_continuous_assign_assignment(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_continuous_assign_delay(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_continuous_assign_drive_strength(
        sym: slang_ast,
    ) -> slang_drive_strength_pair;
    pub fn slang_symbol_compilation_unit_time_scale(
        sym: slang_ast,
        out: *mut slang_time_scale,
    ) -> bool;
    pub fn slang_symbol_cover_cross_body_queue_type(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_cover_cross_iff_expr(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_cover_cross_target_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_cover_cross_target(sym: slang_ast, index: u32) -> slang_ast;
    pub fn slang_symbol_cover_cross_option_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_cover_cross_option_is_type_option(sym: slang_ast, index: u32) -> bool;
    pub fn slang_symbol_cover_cross_option_name(sym: slang_ast, index: u32) -> slang_str;
    pub fn slang_symbol_cover_cross_option_expression(sym: slang_ast, index: u32) -> slang_ast;
    pub fn slang_symbol_coverpoint_coverage_expr(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_coverpoint_iff_expr(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_coverage_bin_kind(sym: slang_ast) -> slang_coverage_bin_kind;
    pub fn slang_symbol_coverage_bin_is_array(sym: slang_ast) -> bool;
    pub fn slang_symbol_coverage_bin_is_wildcard(sym: slang_ast) -> bool;
    pub fn slang_symbol_coverage_bin_is_default(sym: slang_ast) -> bool;
    pub fn slang_symbol_coverage_bin_is_default_sequence(sym: slang_ast) -> bool;
    pub fn slang_symbol_coverage_bin_iff_expr(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_coverage_bin_number_of_bins_expr(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_coverage_bin_set_coverage_expr(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_coverage_bin_with_expr(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_coverage_bin_cross_select_expr(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_coverage_bin_value_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_coverage_bin_value(sym: slang_ast, index: u32) -> slang_ast;
    pub fn slang_symbol_coverage_bin_trans_set_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_coverage_bin_trans_range_count(sym: slang_ast, set_index: u32) -> u32;
    pub fn slang_symbol_coverage_bin_trans_range_item_count(
        sym: slang_ast,
        set_index: u32,
        range_index: u32,
    ) -> u32;
    pub fn slang_symbol_coverage_bin_trans_range_item(
        sym: slang_ast,
        set_index: u32,
        range_index: u32,
        item_index: u32,
    ) -> slang_ast;
    pub fn slang_symbol_coverage_bin_trans_range_repeat_kind(
        sym: slang_ast,
        set_index: u32,
        range_index: u32,
    ) -> slang_repeat_kind;
    pub fn slang_symbol_coverage_bin_trans_range_repeat_from(
        sym: slang_ast,
        set_index: u32,
        range_index: u32,
    ) -> slang_ast;
    pub fn slang_symbol_coverage_bin_trans_range_repeat_to(
        sym: slang_ast,
        set_index: u32,
        range_index: u32,
    ) -> slang_ast;

    pub fn slang_symbol_elab_system_task_kind(sym: slang_ast) -> slang_elab_system_task_kind;
    pub fn slang_symbol_elab_system_task_assert_condition(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_elab_system_task_message(sym: slang_ast, out: *mut slang_str) -> bool;

    pub fn slang_symbol_explicit_import_name(sym: slang_ast) -> slang_str;
    pub fn slang_symbol_explicit_import_package_name(sym: slang_ast) -> slang_str;
    pub fn slang_symbol_explicit_import_package(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_explicit_import_imported_symbol(sym: slang_ast) -> slang_ast;

    pub fn slang_symbol_variable_flags(sym: slang_ast) -> u32;
    pub fn slang_symbol_variable_lifetime(sym: slang_ast) -> slang_variable_lifetime;

    pub fn slang_symbol_wildcard_import_package_name(sym: slang_ast) -> slang_str;
    pub fn slang_symbol_wildcard_import_package(sym: slang_ast) -> slang_ast;

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
    pub fn slang_field_rand_mode(field: slang_ast) -> slang_rand_mode;
    pub fn slang_type_class_base(ty: slang_ast) -> slang_ast;
    pub fn slang_type_class_generic(ty: slang_ast) -> slang_ast;
    pub fn slang_type_class_base_constructor_call(ty: slang_ast) -> slang_ast;
    pub fn slang_type_class_constructor(ty: slang_ast) -> slang_ast;
    pub fn slang_type_class_first_forward_decl(ty: slang_ast) -> slang_ast;
    pub fn slang_type_class_implemented_interface_count(ty: slang_ast) -> u32;
    pub fn slang_type_class_implemented_interface(ty: slang_ast, index: u32) -> slang_ast;
    pub fn slang_type_class_is_abstract(ty: slang_ast) -> bool;
    pub fn slang_type_class_is_final(ty: slang_ast) -> bool;
    pub fn slang_type_class_is_interface(ty: slang_ast) -> bool;
    pub fn slang_type_class_this_var(ty: slang_ast) -> slang_ast;
    pub fn slang_symbol_constraint_block_flags(sym: slang_ast) -> u32;
    pub fn slang_symbol_constraint_block_this_var(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_constraint_block_constraints(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_formal_argument_direction(sym: slang_ast) -> slang_argument_direction;
    pub fn slang_symbol_formal_argument_default_value(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_generate_block_branch_kind(sym: slang_ast) -> slang_generate_branch_kind;
    pub fn slang_symbol_generate_block_condition_expr(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_generate_block_is_uninstantiated(sym: slang_ast) -> bool;
    pub fn slang_symbol_generate_block_case_item_expr_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_generate_block_case_item_expr(sym: slang_ast, index: u32) -> slang_ast;
    pub fn slang_symbol_generate_block_construct_index(sym: slang_ast) -> u32;
    pub fn slang_symbol_generate_block_array_index(
        sym: slang_ast,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_symbol_generate_block_array_entry_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_generate_block_array_entry(sym: slang_ast, index: u32) -> slang_ast;
    pub fn slang_symbol_generate_block_array_construct_index(sym: slang_ast) -> u32;
    pub fn slang_symbol_generate_block_array_valid(sym: slang_ast) -> bool;
    pub fn slang_symbol_generate_block_array_initial_expr(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_generate_block_array_stop_expr(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_generate_block_array_iter_expr(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_generate_block_array_loop_variable(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_generate_block_external_name(
        sym: slang_ast,
        err: *mut slang_error,
    ) -> slang_str;
    pub fn slang_type_covergroup_argument_count(ty: slang_ast) -> u32;
    pub fn slang_type_covergroup_argument(ty: slang_ast, index: u32) -> slang_ast;
    pub fn slang_type_covergroup_base_group(ty: slang_ast) -> slang_ast;
    pub fn slang_type_covergroup_coverage_event(ty: slang_ast) -> slang_ast;
    pub fn slang_type_dpi_open_array_is_packed(ty: slang_ast) -> bool;
    pub fn slang_type_enum_system_id(ty: slang_ast) -> i32;
    pub fn slang_type_floating_kind(ty: slang_ast) -> slang_float_kind;
    pub fn slang_type_fixed_unpacked_array_range(ty: slang_ast) -> slang_constant_range;
    pub fn slang_forwarding_typedef_type_restriction(
        sym: slang_ast,
    ) -> slang_forward_type_restriction;
    pub fn slang_forwarding_typedef_visibility(sym: slang_ast, out: *mut slang_visibility) -> bool;
    pub fn slang_generic_class_is_interface(sym: slang_ast) -> bool;
    pub fn slang_generic_class_default_specialization(sym: slang_ast) -> slang_ast;
    pub fn slang_generic_class_first_forward_decl(sym: slang_ast) -> slang_ast;
    pub fn slang_generic_class_invalid_specialization(
        sym: slang_ast,
        err: *mut slang_error,
    ) -> slang_ast;
    pub fn slang_type_is_matching(a: slang_ast, b: slang_ast) -> bool;
    pub fn slang_type_is_equivalent(a: slang_ast, b: slang_ast) -> bool;
    pub fn slang_type_is_assignment_compatible(a: slang_ast, b: slang_ast) -> bool;
    pub fn slang_type_scalar_kind(ty: slang_ast) -> slang_scalar_kind;
    pub fn slang_type_can_be_string_like(ty: slang_ast) -> bool;
    pub fn slang_type_associative_index_type(ty: slang_ast) -> slang_ast;
    pub fn slang_type_bitstream_width(ty: slang_ast) -> u64;
    pub fn slang_type_common_base(a: slang_ast, b: slang_ast) -> slang_ast;
    pub fn slang_type_integral_flags(ty: slang_ast) -> u32;
    pub fn slang_type_selectable_width(ty: slang_ast) -> u64;
    pub fn slang_type_has_fixed_range(ty: slang_ast) -> bool;
    pub fn slang_type_implements(ty: slang_ast, iface_class: slang_ast) -> bool;
    pub fn slang_type_is_aggregate(ty: slang_ast) -> bool;
    pub fn slang_type_is_alias(ty: slang_ast) -> bool;
    pub fn slang_type_is_associative_array(ty: slang_ast) -> bool;
    pub fn slang_type_is_bitstream_castable(ty: slang_ast, rhs: slang_ast) -> bool;
    pub fn slang_type_is_bitstream_type(ty: slang_ast, destination: bool) -> bool;
    pub fn slang_type_is_boolean_convertible(ty: slang_ast) -> bool;
    pub fn slang_type_is_byte_array(ty: slang_ast) -> bool;
    pub fn slang_type_is_chandle(ty: slang_ast) -> bool;
    pub fn slang_type_is_cast_compatible(ty: slang_ast, rhs: slang_ast) -> bool;
    pub fn slang_type_is_covergroup(ty: slang_ast) -> bool;
    pub fn slang_type_is_derived_from(ty: slang_ast, base: slang_ast) -> bool;
    pub fn slang_type_is_dynamically_sized_array(ty: slang_ast) -> bool;
    pub fn slang_type_is_error(ty: slang_ast) -> bool;
    pub fn slang_type_is_event(ty: slang_ast) -> bool;
    pub fn slang_type_is_fixed_size(ty: slang_ast) -> bool;
    pub fn slang_type_is_floating(ty: slang_ast) -> bool;
    pub fn slang_type_is_handle_type(ty: slang_ast) -> bool;
    pub fn slang_type_is_iterable(ty: slang_ast) -> bool;
    pub fn slang_type_is_null(ty: slang_ast) -> bool;
    pub fn slang_type_is_numeric(ty: slang_ast) -> bool;
    pub fn slang_type_is_object_handle_type(ty: slang_ast) -> bool;
    pub fn slang_type_is_packed_array(ty: slang_ast) -> bool;
    pub fn slang_type_is_packed_union(ty: slang_ast) -> bool;
    pub fn slang_type_is_predefined_integer(ty: slang_ast) -> bool;
    pub fn slang_type_is_property_type(ty: slang_ast) -> bool;
    pub fn slang_type_is_queue(ty: slang_ast) -> bool;
    pub fn slang_type_is_scalar(ty: slang_ast) -> bool;
    pub fn slang_type_is_sequence_type(ty: slang_ast) -> bool;
    pub fn slang_type_is_simple_bit_vector(ty: slang_ast) -> bool;
    pub fn slang_type_is_simple_type(ty: slang_ast) -> bool;
    pub fn slang_type_is_singular(ty: slang_ast) -> bool;
    pub fn slang_type_is_tagged_union(ty: slang_ast) -> bool;
    pub fn slang_type_is_type_ref_type(ty: slang_ast) -> bool;
    pub fn slang_type_is_unbounded(ty: slang_ast) -> bool;
    pub fn slang_type_is_unpacked_struct(ty: slang_ast) -> bool;
    pub fn slang_type_is_unpacked_union(ty: slang_ast) -> bool;
    pub fn slang_type_is_untyped_type(ty: slang_ast) -> bool;
    pub fn slang_type_is_valid_for_dpi_arg(ty: slang_ast) -> bool;
    pub fn slang_type_is_valid_for_dpi_return(ty: slang_ast) -> bool;
    pub fn slang_type_is_valid_for_rand(
        ty: slang_ast,
        mode: slang_rand_mode,
        language_version: slang_language_version,
    ) -> bool;
    pub fn slang_type_is_valid_for_sequence(ty: slang_ast) -> bool;
    pub fn slang_type_is_virtual_interface(ty: slang_ast) -> bool;
    pub fn slang_type_is_void(ty: slang_ast) -> bool;
    pub fn slang_type_alias_visibility(ty: slang_ast) -> slang_visibility;

    // AST: type printing
    pub fn slang_type_printer_create(err: *mut slang_error) -> slang_type_printer;
    pub fn slang_type_printer_destroy(printer: slang_type_printer);
    pub fn slang_type_printer_append(printer: slang_type_printer, ty: slang_ast);
    pub fn slang_type_printer_clear(printer: slang_type_printer);
    pub fn slang_type_printer_to_string(
        printer: slang_type_printer,
        err: *mut slang_error,
    ) -> slang_str;
    pub fn slang_type_printer_options(printer: slang_type_printer) -> slang_type_printing_options;
    pub fn slang_type_printer_set_options(
        printer: slang_type_printer,
        options: slang_type_printing_options,
    );
    pub fn slang_type_fixed_range(ty: slang_ast) -> slang_constant_range;
    pub fn slang_type_bit_vector_range(ty: slang_ast) -> slang_constant_range;
    pub fn slang_type_is_declared_reg(ty: slang_ast) -> bool;
    pub fn slang_type_packed_array_range(ty: slang_ast) -> slang_constant_range;
    pub fn slang_type_packed_struct_system_id(ty: slang_ast) -> i32;
    pub fn slang_type_packed_union_is_soft(ty: slang_ast) -> bool;
    pub fn slang_type_packed_union_is_tagged(ty: slang_ast) -> bool;
    pub fn slang_type_packed_union_system_id(ty: slang_ast) -> i32;
    pub fn slang_type_packed_union_tag_bits(ty: slang_ast) -> u32;
    pub fn slang_type_unpacked_struct_system_id(ty: slang_ast) -> i32;
    pub fn slang_type_unpacked_union_is_tagged(ty: slang_ast) -> bool;
    pub fn slang_type_unpacked_union_system_id(ty: slang_ast) -> i32;
    pub fn slang_type_predefined_integer_kind(ty: slang_ast) -> slang_predefined_integer_kind;
    pub fn slang_type_queue_max_bound(ty: slang_ast) -> u32;
    pub fn slang_net_type_net_kind(net_type: slang_ast) -> slang_net_kind;
    pub fn slang_net_type_resolution_function(net_type: slang_ast) -> slang_ast;
    pub fn slang_net_type_is_built_in(net_type: slang_ast) -> bool;
    pub fn slang_net_type_is_error(net_type: slang_ast) -> bool;
    pub fn slang_net_type_get_simulated(
        internal_net: slang_ast,
        external_net: slang_ast,
        out_should_warn: *mut bool,
    ) -> slang_ast;

    // AST: expressions
    pub fn slang_expression_type(expr: slang_ast) -> slang_ast;
    pub fn slang_expression_is_bad(expr: slang_ast) -> bool;
    pub fn slang_expression_symbol(expr: slang_ast) -> slang_ast;
    pub fn slang_expr_symbol_reference(expr: slang_ast, allow_packed: bool) -> slang_ast;
    pub fn slang_expr_has_hierarchical_reference(expr: slang_ast) -> bool;
    pub fn slang_expr_is_equivalent_to(expr: slang_ast, other: slang_ast) -> bool;
    pub fn slang_expr_is_implicit_string(expr: slang_ast) -> bool;
    pub fn slang_expr_is_implicitly_assignable_to(expr: slang_ast, ty: slang_ast) -> bool;
    pub fn slang_expr_is_unsized_integer(expr: slang_ast) -> bool;
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
    pub fn slang_expr_integer_literal_value(
        expr: slang_ast,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_expr_unbased_unsized_literal_value(
        expr: slang_ast,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_type_default_value(ty: slang_ast, err: *mut slang_error) -> slang_constant;
    pub fn slang_type_coerce_value(
        ty: slang_ast,
        value: slang_constant,
        err: *mut slang_error,
    ) -> slang_constant;
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
    pub fn slang_constant_empty(c: slang_constant) -> bool;
    pub fn slang_constant_is_container(c: slang_constant) -> bool;
    pub fn slang_constant_is_true(c: slang_constant) -> bool;
    pub fn slang_constant_is_false(c: slang_constant) -> bool;
    pub fn slang_constant_bitstream_width(c: slang_constant) -> u64;
    pub fn slang_constant_get_slice(
        c: slang_constant,
        upper: i32,
        lower: i32,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_constant_convert_to_real(
        c: slang_constant,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_constant_convert_to_short_real(
        c: slang_constant,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_constant_convert_to_str(
        c: slang_constant,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_constant_convert_to_byte_array(
        c: slang_constant,
        size: u32,
        is_signed: bool,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_constant_convert_to_byte_queue(
        c: slang_constant,
        is_signed: bool,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_svint_bit_width(v: slang_svint) -> u32;
    pub fn slang_svint_is_signed(v: slang_svint) -> bool;
    pub fn slang_svint_has_unknown(v: slang_svint) -> bool;
    pub fn slang_svint_as_i64(v: slang_svint, out: *mut i64) -> bool;
    pub fn slang_svint_as_u64(v: slang_svint, out: *mut u64) -> bool;
    pub fn slang_svint_get_bit(v: slang_svint, index: u32) -> u8;
    pub fn slang_svint_to_string(v: slang_svint) -> slang_str;
    pub fn slang_svint_count_leading_ones(v: slang_svint) -> u32;
    pub fn slang_svint_count_leading_zeros(v: slang_svint) -> u32;
    pub fn slang_svint_count_leading_unknowns(v: slang_svint) -> u32;
    pub fn slang_svint_count_leading_zs(v: slang_svint) -> u32;
    pub fn slang_svint_count_ones(v: slang_svint) -> u32;
    pub fn slang_svint_count_zeros(v: slang_svint) -> u32;
    pub fn slang_svint_count_xs(v: slang_svint) -> u32;
    pub fn slang_svint_count_zs(v: slang_svint) -> u32;
    pub fn slang_svint_active_bits(v: slang_svint) -> u32;
    pub fn slang_svint_min_represented_bits(v: slang_svint) -> u32;
    pub fn slang_svint_is_even(v: slang_svint) -> bool;
    pub fn slang_svint_is_odd(v: slang_svint) -> bool;
    pub fn slang_svint_is_negative(v: slang_svint) -> bool;
    pub fn slang_svint_is_sign_extended_from(v: slang_svint, msb: u32) -> bool;
    pub fn slang_svint_reduction_and(v: slang_svint) -> u8;
    pub fn slang_svint_reduction_or(v: slang_svint) -> u8;
    pub fn slang_svint_reduction_xor(v: slang_svint) -> u8;
    pub fn slang_svint_logical_impl(lhs: slang_svint, rhs: slang_svint) -> u8;
    pub fn slang_svint_logical_equiv(lhs: slang_svint, rhs: slang_svint) -> u8;
    pub fn slang_logic_value_value(v: slang_logic_value) -> u8;
    pub fn slang_logic_value_x() -> slang_logic_value;
    pub fn slang_logic_value_z() -> slang_logic_value;
    pub fn slang_logic_value_is_unknown(v: slang_logic_value) -> bool;
    pub fn slang_logic_value_and(
        lhs: slang_logic_value,
        rhs: slang_logic_value,
    ) -> slang_logic_value;
    pub fn slang_logic_value_or(
        lhs: slang_logic_value,
        rhs: slang_logic_value,
    ) -> slang_logic_value;
    pub fn slang_logic_value_xor(
        lhs: slang_logic_value,
        rhs: slang_logic_value,
    ) -> slang_logic_value;
    pub fn slang_logic_value_not(v: slang_logic_value) -> slang_logic_value;
    pub fn slang_svint_pow(
        v: slang_svint,
        rhs: slang_svint,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_svint_replicate(
        v: slang_svint,
        times: slang_svint,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_svint_resize(v: slang_svint, bits: u32, err: *mut slang_error) -> slang_constant;
    pub fn slang_svint_reverse(v: slang_svint, err: *mut slang_error) -> slang_constant;
    pub fn slang_svint_sext(v: slang_svint, bits: u32, err: *mut slang_error) -> slang_constant;
    pub fn slang_svint_zext(v: slang_svint, bits: u32, err: *mut slang_error) -> slang_constant;
    pub fn slang_svint_extend(
        v: slang_svint,
        bits: u32,
        is_signed: bool,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_svint_trunc(v: slang_svint, bits: u32, err: *mut slang_error) -> slang_constant;
    pub fn slang_svint_slice(
        v: slang_svint,
        msb: i32,
        lsb: i32,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_svint_xnor(
        lhs: slang_svint,
        rhs: slang_svint,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_svint_and(
        lhs: slang_svint,
        rhs: slang_svint,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_svint_or(
        lhs: slang_svint,
        rhs: slang_svint,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_svint_xor(
        lhs: slang_svint,
        rhs: slang_svint,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_svint_not(v: slang_svint, err: *mut slang_error) -> slang_constant;
    pub fn slang_svint_iand(v: slang_svint, rhs: slang_svint, err: *mut slang_error)
    -> slang_svint;
    pub fn slang_svint_ior(v: slang_svint, rhs: slang_svint, err: *mut slang_error) -> slang_svint;
    pub fn slang_svint_ixor(v: slang_svint, rhs: slang_svint, err: *mut slang_error)
    -> slang_svint;
    pub fn slang_svint_set(
        v: slang_svint,
        msb: i32,
        lsb: i32,
        value: slang_svint,
        err: *mut slang_error,
    );
    pub fn slang_svint_set_all_ones(v: slang_svint);
    pub fn slang_svint_set_all_zeros(v: slang_svint);
    pub fn slang_svint_set_all_x(v: slang_svint);
    pub fn slang_svint_set_all_z(v: slang_svint);
    pub fn slang_svint_set_signed(v: slang_svint, is_signed: bool);
    pub fn slang_svint_flatten_unknowns(v: slang_svint);
    pub fn slang_svint_shrink_to_fit(v: slang_svint);
    pub fn slang_svint_sign_extend_from(v: slang_svint, msb: u32, err: *mut slang_error);
    pub fn slang_svint_create_fill_x(
        bits: u32,
        is_signed: bool,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_svint_create_fill_z(
        bits: u32,
        is_signed: bool,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_svint_from_digits(
        bits: u32,
        base: u8,
        is_signed: bool,
        any_unknown: bool,
        digits: *const u8,
        digit_count: u32,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_svint_from_double(
        bits: u32,
        value: f64,
        is_signed: bool,
        round: bool,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_svint_from_float(
        bits: u32,
        value: f32,
        is_signed: bool,
        round: bool,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_svint_concat(
        operands: *const slang_svint,
        count: u32,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_svint_conditional(
        condition: slang_svint,
        lhs: slang_svint,
        rhs: slang_svint,
        err: *mut slang_error,
    ) -> slang_constant;

    // Semantic statement/expression tree
    pub fn slang_symbol_body(sym: slang_ast) -> slang_ast;
    pub fn slang_ast_sem_child_count(node: slang_ast) -> u32;
    pub fn slang_ast_sem_child(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_ast_sem_children(node: slang_ast, out: *mut slang_ast, cap: u32) -> u32;
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
    pub fn slang_timing_control_edge(node: slang_ast) -> slang_edge_kind;
    pub fn slang_pattern_expr(node: slang_ast) -> slang_ast;
    pub fn slang_pattern_variable(node: slang_ast) -> slang_ast;
    pub fn slang_assertion_expr_op(node: slang_ast) -> i32;

    // Block / conditional / case / concurrent-assertion / event-trigger breadth
    pub fn slang_stmt_block_kind(node: slang_ast) -> u32;
    pub fn slang_stmt_block_symbol(node: slang_ast) -> slang_ast;
    pub fn slang_stmt_conditional_check(node: slang_ast) -> u32;
    pub fn slang_stmt_conditional_condition_count(node: slang_ast) -> u32;
    pub fn slang_stmt_conditional_condition_expr(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_stmt_conditional_condition_pattern(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_stmt_case_condition(node: slang_ast) -> u32;
    pub fn slang_stmt_case_check(node: slang_ast) -> u32;
    pub fn slang_stmt_case_default(node: slang_ast) -> slang_ast;
    pub fn slang_stmt_case_item_count(node: slang_ast) -> u32;
    pub fn slang_stmt_case_item_stmt(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_stmt_case_item_expr_count(node: slang_ast, index: u32) -> u32;
    pub fn slang_stmt_case_item_expr(node: slang_ast, index: u32, j: u32) -> slang_ast;
    pub fn slang_stmt_assertion_kind(node: slang_ast) -> u32;
    pub fn slang_stmt_assertion_if_true(node: slang_ast) -> slang_ast;
    pub fn slang_stmt_assertion_if_false(node: slang_ast) -> slang_ast;
    pub fn slang_stmt_event_trigger_is_nonblocking(node: slang_ast) -> bool;

    // ForLoop / Foreach / ImmediateAssertion / PatternCase breadth
    pub fn slang_stmt_for_loop_initializer_count(node: slang_ast) -> u32;
    pub fn slang_stmt_for_loop_initializer(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_stmt_for_loop_var_count(node: slang_ast) -> u32;
    pub fn slang_stmt_for_loop_var(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_stmt_for_loop_step_count(node: slang_ast) -> u32;
    pub fn slang_stmt_for_loop_step(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_stmt_foreach_loop_dim_count(node: slang_ast) -> u32;
    pub fn slang_stmt_foreach_loop_dim_var(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_stmt_foreach_loop_dim_range(
        node: slang_ast,
        index: u32,
        out: *mut slang_constant_range,
    ) -> bool;

    // slang::ConstantRange operations (plain value struct, no handle)
    pub fn slang_constant_range_left(r: slang_constant_range) -> i32;
    pub fn slang_constant_range_right(r: slang_constant_range) -> i32;
    pub fn slang_constant_range_width(r: slang_constant_range) -> u32;
    pub fn slang_constant_range_lower(r: slang_constant_range) -> i32;
    pub fn slang_constant_range_upper(r: slang_constant_range) -> i32;
    pub fn slang_constant_range_is_descending(r: slang_constant_range) -> bool;
    pub fn slang_constant_range_reverse(r: slang_constant_range) -> slang_constant_range;
    pub fn slang_constant_range_subrange(
        r: slang_constant_range,
        select: slang_constant_range,
    ) -> slang_constant_range;
    pub fn slang_constant_range_contains_point(r: slang_constant_range, index: i32) -> bool;
    pub fn slang_constant_range_overlaps(
        r: slang_constant_range,
        other: slang_constant_range,
    ) -> bool;
    pub fn slang_constant_range_translate_index(r: slang_constant_range, index: i32) -> i32;
    pub fn slang_constant_range_get_indexed_range(
        l: i32,
        width: i32,
        descending: bool,
        indexed_up: bool,
        out: *mut slang_constant_range,
    ) -> bool;
    pub fn slang_stmt_immediate_assertion_kind(node: slang_ast) -> u32;
    pub fn slang_stmt_immediate_assertion_if_true(node: slang_ast) -> slang_ast;
    pub fn slang_stmt_immediate_assertion_if_false(node: slang_ast) -> slang_ast;
    pub fn slang_stmt_immediate_assertion_is_deferred(node: slang_ast) -> bool;
    pub fn slang_stmt_immediate_assertion_is_final(node: slang_ast) -> bool;
    pub fn slang_stmt_pattern_case_check(node: slang_ast) -> u32;
    pub fn slang_stmt_pattern_case_item_count(node: slang_ast) -> u32;
    pub fn slang_stmt_pattern_case_item_pattern(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_stmt_pattern_case_item_filter(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_stmt_pattern_case_item_stmt(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_stmt_pattern_case_condition(node: slang_ast) -> u32;
    pub fn slang_stmt_pattern_case_default(node: slang_ast) -> slang_ast;
    pub fn slang_stmt_wait_order_event_count(node: slang_ast) -> u32;
    pub fn slang_stmt_wait_order_event(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_stmt_wait_order_if_true(node: slang_ast) -> slang_ast;
    pub fn slang_stmt_wait_order_if_false(node: slang_ast) -> slang_ast;
    pub fn slang_stmt_procedural_assign_is_force(node: slang_ast) -> bool;
    pub fn slang_stmt_procedural_deassign_is_release(node: slang_ast) -> bool;
    pub fn slang_stmt_randcase_item_count(node: slang_ast) -> u32;
    pub fn slang_stmt_randcase_item_expr(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_stmt_randcase_item_stmt(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_stmt_randsequence_first_production(node: slang_ast) -> slang_ast;
    pub fn slang_stmt_procedural_checker_instance_count(node: slang_ast) -> u32;
    pub fn slang_stmt_procedural_checker_instance(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_stmt_is_bad(node: slang_ast) -> bool;
    pub fn slang_stmt_eval(node: slang_ast, err: *mut slang_error) -> u32;

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
    pub fn slang_expr_integer_literal_is_declared_unsized(node: slang_ast) -> bool;
    pub fn slang_expr_real_literal_value(node: slang_ast) -> f64;
    pub fn slang_expr_time_literal_value(node: slang_ast) -> f64;
    pub fn slang_expr_time_literal_scale(node: slang_ast, out: *mut slang_time_scale) -> bool;
    pub fn slang_expr_unbased_unsized_literal_bit(node: slang_ast) -> u8;
    pub fn slang_expr_inside_left(node: slang_ast) -> slang_ast;
    pub fn slang_expr_inside_range_count(node: slang_ast) -> u32;
    pub fn slang_expr_inside_range(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_expr_new_array_size(node: slang_ast) -> slang_ast;
    pub fn slang_expr_new_array_init(node: slang_ast) -> slang_ast;
    pub fn slang_expr_new_class_constructor_call(node: slang_ast) -> slang_ast;
    pub fn slang_expr_new_class_is_super_class(node: slang_ast) -> bool;
    pub fn slang_expr_new_covergroup_argument_count(node: slang_ast) -> u32;
    pub fn slang_expr_new_covergroup_argument(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_expr_tagged_union_value(node: slang_ast) -> slang_ast;

    pub fn slang_expr_arbitrary_symbol(node: slang_ast) -> slang_ast;
    pub fn slang_expr_assignment_is_compound(node: slang_ast) -> bool;
    pub fn slang_expr_assignment_is_lvalue_arg(node: slang_ast) -> bool;
    pub fn slang_expr_assignment_op(node: slang_ast) -> u32;
    pub fn slang_expr_assignment_timing(node: slang_ast) -> slang_ast;
    pub fn slang_expr_pattern_element_count(node: slang_ast) -> u32;
    pub fn slang_expr_pattern_element(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_expr_assertion_instance_is_recursive(node: slang_ast) -> bool;
    pub fn slang_expr_assertion_instance_local_var_count(node: slang_ast) -> u32;
    pub fn slang_expr_assertion_instance_local_var(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_expr_assertion_instance_argument_count(node: slang_ast) -> u32;
    pub fn slang_expr_assertion_instance_argument_port(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_expr_assertion_instance_argument_actual(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_expr_call_iterator_expr(node: slang_ast) -> slang_ast;
    pub fn slang_expr_call_iterator_var(node: slang_ast) -> slang_ast;
    pub fn slang_expr_call_randomize_inline_constraints(node: slang_ast) -> slang_ast;
    pub fn slang_expr_call_extra_info_kind(node: slang_ast) -> u32;
    pub fn slang_expr_call_system_scope(node: slang_ast) -> slang_ast;
    pub fn slang_expr_call_subroutine_kind(node: slang_ast) -> u32;

    // Call/Conditional/Conversion/CopyClass/Dist expression breadth, and the
    // Expression base-class lvalue-evaluation and effective-width accessors.
    pub fn slang_expr_call_subroutine_name(node: slang_ast) -> slang_str;
    pub fn slang_expr_call_is_system_call(node: slang_ast) -> bool;
    pub fn slang_expr_call_this_class(node: slang_ast) -> slang_ast;
    pub fn slang_expr_cond_condition_count(node: slang_ast) -> u32;
    pub fn slang_expr_cond_condition_expr(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_expr_cond_condition_pattern(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_expr_conversion_is_const_cast(node: slang_ast) -> bool;
    pub fn slang_expr_conversion_is_implicit(node: slang_ast) -> bool;
    pub fn slang_expr_copy_class_source(node: slang_ast) -> slang_ast;
    pub fn slang_expr_dist_left(node: slang_ast) -> slang_ast;
    pub fn slang_expr_dist_item_count(node: slang_ast) -> u32;
    pub fn slang_expr_dist_item_value(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_expr_dist_item_weight_kind(node: slang_ast, index: u32) -> u32;
    pub fn slang_expr_dist_item_weight_expr(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_expr_dist_default_weight_kind(node: slang_ast) -> u32;
    pub fn slang_expr_dist_default_weight_expr(node: slang_ast) -> slang_ast;
    pub fn slang_expr_effective_width(node: slang_ast, out: *mut u32) -> bool;
    pub fn slang_expr_eval_lvalue(node: slang_ast, err: *mut slang_error) -> slang_lvalue;
    pub fn slang_lvalue_destroy(lval: slang_lvalue);
    pub fn slang_lvalue_is_bad(lval: slang_lvalue) -> bool;
    pub fn slang_lvalue_load(lval: slang_lvalue) -> slang_str;
    pub fn slang_lvalue_store_int(lval: slang_lvalue, value: i64) -> bool;

    // Replicated/StructuredAssignmentPattern breadth, StreamingConcatenation,
    // and StringLiteral.
    pub fn slang_expr_replicated_pattern_count(node: slang_ast) -> slang_ast;
    pub fn slang_expr_structured_pattern_member_setter_count(node: slang_ast) -> u32;
    pub fn slang_expr_structured_pattern_member_setter_member(
        node: slang_ast,
        index: u32,
    ) -> slang_ast;
    pub fn slang_expr_structured_pattern_member_setter_expr(
        node: slang_ast,
        index: u32,
    ) -> slang_ast;
    pub fn slang_expr_structured_pattern_type_setter_count(node: slang_ast) -> u32;
    pub fn slang_expr_structured_pattern_type_setter_type(node: slang_ast, index: u32)
    -> slang_ast;
    pub fn slang_expr_structured_pattern_type_setter_expr(node: slang_ast, index: u32)
    -> slang_ast;
    pub fn slang_expr_structured_pattern_index_setter_count(node: slang_ast) -> u32;
    pub fn slang_expr_structured_pattern_index_setter_index(
        node: slang_ast,
        index: u32,
    ) -> slang_ast;
    pub fn slang_expr_structured_pattern_index_setter_expr(
        node: slang_ast,
        index: u32,
    ) -> slang_ast;
    pub fn slang_expr_structured_pattern_default_setter(node: slang_ast) -> slang_ast;
    pub fn slang_expr_streaming_bitstream_width(node: slang_ast) -> u64;
    pub fn slang_expr_streaming_slice_size(node: slang_ast) -> u64;
    pub fn slang_expr_streaming_is_fixed_size(node: slang_ast) -> bool;
    pub fn slang_expr_streaming_stream_count(node: slang_ast) -> u32;
    pub fn slang_expr_streaming_stream_operand(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_expr_streaming_stream_with_expr(node: slang_ast, index: u32) -> slang_ast;
    pub fn slang_expr_string_literal_value(node: slang_ast) -> slang_str;
    pub fn slang_expr_string_literal_raw_value(node: slang_ast) -> slang_str;
    pub fn slang_expr_string_literal_int_value(
        node: slang_ast,
        err: *mut slang_error,
    ) -> slang_constant;

    // Driver
    pub fn slang_driver_create(err: *mut slang_error) -> slang_driver;
    pub fn slang_driver_create_bare(err: *mut slang_error) -> slang_driver;
    pub fn slang_driver_destroy(driver: slang_driver);
    pub fn slang_driver_add_standard_args(driver: slang_driver, err: *mut slang_error);
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
    pub fn slang_parse_options_create(err: *mut slang_error) -> slang_parse_options;
    pub fn slang_parse_options_destroy(options: slang_parse_options);
    pub fn slang_parse_options_set_support_comments(options: slang_parse_options, value: bool);
    pub fn slang_parse_options_support_comments(options: slang_parse_options) -> bool;
    pub fn slang_parse_options_set_ignore_program_name(options: slang_parse_options, value: bool);
    pub fn slang_parse_options_ignore_program_name(options: slang_parse_options) -> bool;
    pub fn slang_parse_options_set_expand_env_vars(options: slang_parse_options, value: bool);
    pub fn slang_parse_options_expand_env_vars(options: slang_parse_options) -> bool;
    pub fn slang_parse_options_set_ignore_duplicates(options: slang_parse_options, value: bool);
    pub fn slang_parse_options_ignore_duplicates(options: slang_parse_options) -> bool;
    pub fn slang_driver_parse_args_with_options(
        driver: slang_driver,
        argc: c_int,
        argv: *const *const c_char,
        options: slang_parse_options,
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
    pub fn slang_driver_create_option_bag(driver: slang_driver, err: *mut slang_error)
    -> slang_bag;
    pub fn slang_bag_destroy(bag: slang_bag);
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
    pub fn slang_driver_language_version(driver: slang_driver) -> u32;
    pub fn slang_driver_process_command_files(
        driver: slang_driver,
        pattern: *const c_char,
        pattern_len: usize,
        make_relative: bool,
        separate_unit: bool,
        err: *mut slang_error,
    ) -> bool;
    pub fn slang_driver_command_file_metadata_count(driver: slang_driver) -> u32;
    pub fn slang_driver_command_file_metadata_at(
        driver: slang_driver,
        index: u32,
    ) -> slang_command_file_metadata;
    pub fn slang_command_file_metadata_path(meta: slang_command_file_metadata) -> slang_str;
    pub fn slang_command_file_metadata_define_count(meta: slang_command_file_metadata) -> u32;
    pub fn slang_command_file_metadata_define_at(
        meta: slang_command_file_metadata,
        index: u32,
    ) -> slang_str;
    pub fn slang_driver_optionally_write_dep_files(driver: slang_driver, err: *mut slang_error);
    pub fn slang_driver_diag_engine(driver: slang_driver) -> slang_diag_engine;
    pub fn slang_diag_engine_num_errors(engine: slang_diag_engine) -> i32;
    pub fn slang_diag_engine_num_warnings(engine: slang_diag_engine) -> i32;
    pub fn slang_driver_get_analysis_options(driver: slang_driver) -> slang_analysis_options;
    pub fn slang_driver_run_preprocessor(
        driver: slang_driver,
        flags: u32,
        err: *mut slang_error,
    ) -> bool;
    pub fn slang_driver_report_macros(
        driver: slang_driver,
        group_by_file: bool,
        err: *mut slang_error,
    );
    pub fn slang_driver_report_parse_diags(driver: slang_driver, err: *mut slang_error) -> bool;
    pub fn slang_driver_run_full_compilation(
        driver: slang_driver,
        quiet: bool,
        err: *mut slang_error,
    ) -> bool;
    pub fn slang_driver_set_terminal_colors_enabled(
        driver: slang_driver,
        enable: bool,
        err: *mut slang_error,
    );
    pub fn slang_driver_source_loader(driver: slang_driver) -> slang_source_loader;
    pub fn slang_source_loader_add_files(
        loader: slang_source_loader,
        pattern: *const c_char,
        pattern_len: usize,
        err: *mut slang_error,
    );
    pub fn slang_source_loader_add_library_files(
        loader: slang_source_loader,
        library_name: *const c_char,
        library_name_len: usize,
        pattern: *const c_char,
        pattern_len: usize,
        err: *mut slang_error,
    );
    pub fn slang_source_loader_add_library_maps(
        loader: slang_source_loader,
        pattern: *const c_char,
        pattern_len: usize,
        base_path: *const c_char,
        base_path_len: usize,
        options: slang_options,
        err: *mut slang_error,
    );
    pub fn slang_source_loader_add_search_directories(
        loader: slang_source_loader,
        pattern: *const c_char,
        pattern_len: usize,
        err: *mut slang_error,
    );
    pub fn slang_source_loader_add_search_extension(
        loader: slang_source_loader,
        extension: *const c_char,
        extension_len: usize,
        err: *mut slang_error,
    );
    pub fn slang_source_loader_add_separate_unit(
        loader: slang_source_loader,
        file_patterns: *const *const c_char,
        file_patterns_count: usize,
        include_paths: *const *const c_char,
        include_paths_count: usize,
        defines: *const *const c_char,
        defines_count: usize,
        library_name: *const c_char,
        library_name_len: usize,
        warning_options: *const *const c_char,
        warning_options_count: usize,
        err: *mut slang_error,
    );
    pub fn slang_source_loader_has_files(loader: slang_source_loader) -> bool;
    pub fn slang_source_loader_error_count(loader: slang_source_loader) -> u32;
    pub fn slang_source_loader_error_at(loader: slang_source_loader, index: u32) -> slang_str;
    pub fn slang_source_loader_library_map_count(loader: slang_source_loader) -> u32;
    pub fn slang_source_loader_library_map_at(
        loader: slang_source_loader,
        index: u32,
    ) -> slang_syntax_tree;
    pub fn slang_source_loader_load_sources(
        loader: slang_source_loader,
        err: *mut slang_error,
    ) -> u32;
    pub fn slang_source_loader_loaded_buffer_id(
        loader: slang_source_loader,
        index: u32,
    ) -> slang_buffer_id;
    pub fn slang_source_loader_loaded_buffer_text(
        loader: slang_source_loader,
        index: u32,
    ) -> slang_str;
    pub fn slang_source_options_create(err: *mut slang_error) -> slang_source_options;
    pub fn slang_source_options_destroy(options: slang_source_options);
    pub fn slang_source_options_set_num_threads(
        options: slang_source_options,
        has_value: bool,
        value: u32,
    );
    pub fn slang_source_options_num_threads(options: slang_source_options, out: *mut u32) -> bool;
    pub fn slang_source_options_set_single_unit(options: slang_source_options, value: bool);
    pub fn slang_source_options_single_unit(options: slang_source_options) -> bool;
    pub fn slang_source_options_set_only_lint(options: slang_source_options, value: bool);
    pub fn slang_source_options_only_lint(options: slang_source_options) -> bool;
    pub fn slang_source_options_set_libraries_inherit_macros(
        options: slang_source_options,
        value: bool,
    );
    pub fn slang_source_options_libraries_inherit_macros(options: slang_source_options) -> bool;
    pub fn slang_driver_text_diag_client(driver: slang_driver) -> slang_text_diag_client;
    pub fn slang_text_diag_client_get_string(
        client: slang_text_diag_client,
        err: *mut slang_error,
    ) -> slang_str;
    pub fn slang_text_diag_client_empty(client: slang_text_diag_client) -> bool;

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
    pub fn slang_driver_run_analysis(
        driver: slang_driver,
        comp: slang_compilation,
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
    pub fn slang_analysis_driver_handle(
        analysis: slang_analysis,
        value: slang_ast,
        index: u32,
    ) -> slang_value_driver;
    pub fn slang_value_driver_flags(driver: slang_value_driver) -> u32;
    pub fn slang_value_driver_symbol(driver: slang_value_driver) -> slang_ast;
    pub fn slang_value_driver_bounds(
        driver: slang_value_driver,
        lo: *mut u64,
        hi: *mut u64,
    ) -> bool;
    pub fn slang_value_driver_source_range(driver: slang_value_driver) -> slang_range;
    pub fn slang_value_driver_is_unidirectional_port(driver: slang_value_driver) -> bool;
    pub fn slang_value_driver_is_in_single_driver_procedure(driver: slang_value_driver) -> bool;
    pub fn slang_value_driver_source(driver: slang_value_driver) -> slang_driver_source;
    pub fn slang_value_driver_path(driver: slang_value_driver) -> slang_value_path;
    pub fn slang_value_path_root_symbol(path: slang_value_path) -> slang_ast;
    pub fn slang_value_driver_override_range(
        driver: slang_value_driver,
        out: *mut slang_range,
    ) -> bool;

    // Analyzed procedures, assertions, and their sub-results
    pub fn slang_analysis_scope_procedure_handle(
        analysis: slang_analysis,
        scope: slang_ast,
        index: u32,
    ) -> slang_analyzed_procedure;
    pub fn slang_analysis_assertion_count(
        analysis: slang_analysis,
        containing_symbol: slang_ast,
    ) -> u32;
    pub fn slang_analysis_assertion_at(
        analysis: slang_analysis,
        containing_symbol: slang_ast,
        index: u32,
    ) -> slang_analyzed_assertion;
    pub fn slang_analyzed_assertion_containing_symbol(
        assertion: slang_analyzed_assertion,
    ) -> slang_ast;
    pub fn slang_analyzed_assertion_procedure(
        assertion: slang_analyzed_assertion,
    ) -> slang_analyzed_procedure;
    pub fn slang_analyzed_assertion_ast_node(assertion: slang_analyzed_assertion) -> slang_ast;
    pub fn slang_analyzed_assertion_root(assertion: slang_analyzed_assertion) -> slang_ast;
    pub fn slang_analyzed_assertion_semantic_leading_clock(
        assertion: slang_analyzed_assertion,
    ) -> slang_ast;
    pub fn slang_analyzed_assertion_clock(
        assertion: slang_analyzed_assertion,
        expr: slang_ast,
    ) -> slang_ast;
    pub fn slang_analyzed_procedure_symbol(procedure: slang_analyzed_procedure) -> slang_ast;
    pub fn slang_analyzed_procedure_parent(
        procedure: slang_analyzed_procedure,
    ) -> slang_analyzed_procedure;
    pub fn slang_analyzed_procedure_inferred_clock(
        procedure: slang_analyzed_procedure,
    ) -> slang_ast;
    pub fn slang_analyzed_procedure_driver_count(procedure: slang_analyzed_procedure) -> u32;
    pub fn slang_analyzed_procedure_driver_at(
        procedure: slang_analyzed_procedure,
        index: u32,
        out: *mut slang_driver_info,
    ) -> bool;
    pub fn slang_analyzed_procedure_call_expression_count(
        procedure: slang_analyzed_procedure,
    ) -> u32;
    pub fn slang_analyzed_procedure_call_expression_at(
        procedure: slang_analyzed_procedure,
        index: u32,
    ) -> slang_ast;
    pub fn slang_analyzed_procedure_timing_control_count(
        procedure: slang_analyzed_procedure,
    ) -> u32;
    pub fn slang_analyzed_procedure_timing_control_at(
        procedure: slang_analyzed_procedure,
        index: u32,
    ) -> slang_ast;
    pub fn slang_analyzed_procedure_read_set_count(procedure: slang_analyzed_procedure) -> u32;
    pub fn slang_analyzed_procedure_read_set_at(
        procedure: slang_analyzed_procedure,
        index: u32,
    ) -> slang_read_range;
    pub fn slang_analyzed_procedure_implicit_event_read_set_count(
        procedure: slang_analyzed_procedure,
    ) -> u32;
    pub fn slang_analyzed_procedure_implicit_event_read_set_at(
        procedure: slang_analyzed_procedure,
        index: u32,
    ) -> slang_implicit_event_read_set;
    pub fn slang_analyzed_procedure_sensitivity_list(
        procedure: slang_analyzed_procedure,
    ) -> slang_sensitivity_list;
    pub fn slang_implicit_event_read_set_statement(
        read_set: slang_implicit_event_read_set,
    ) -> slang_ast;
    pub fn slang_implicit_event_read_set_read_count(read_set: slang_implicit_event_read_set)
    -> u32;
    pub fn slang_implicit_event_read_set_read_at(
        read_set: slang_implicit_event_read_set,
        index: u32,
    ) -> slang_read_range;
    pub fn slang_read_range_symbol(range: slang_read_range) -> slang_ast;
    pub fn slang_read_range_bit_range(range: slang_read_range, lo: *mut u64, hi: *mut u64) -> bool;
    pub fn slang_sensitivity_list_kind(list: slang_sensitivity_list) -> slang_sensitivity_kind;
    pub fn slang_sensitivity_list_timing_control(list: slang_sensitivity_list) -> slang_ast;
    pub fn slang_sensitivity_list_read_count(list: slang_sensitivity_list) -> u32;
    pub fn slang_sensitivity_list_read_at(
        list: slang_sensitivity_list,
        index: u32,
    ) -> slang_read_range;

    pub fn slang_dfa_run(
        comp: slang_compilation,
        procedure: slang_ast,
        lattice: *const slang_dfa_lattice,
        user: *mut c_void,
        err: *mut slang_error,
    ) -> *mut c_void;
    pub fn slang_dfa_ctx_state(ctx: slang_dfa_ctx) -> *mut c_void;
    pub fn slang_dfa_ctx_is_bad(ctx: slang_dfa_ctx) -> bool;
    pub fn slang_dfa_ctx_eval_context(ctx: slang_dfa_ctx) -> slang_eval_ctx;
    pub fn slang_eval_ctx_evaluate(
        ectx: slang_eval_ctx,
        expr: slang_ast,
        err: *mut slang_error,
    ) -> slang_constant;

    // Script session
    pub fn slang_script_session_create(
        options: slang_options,
        err: *mut slang_error,
    ) -> slang_script_session;
    pub fn slang_script_session_destroy(session: slang_script_session);
    pub fn slang_script_session_eval(
        session: slang_script_session,
        text: *const c_char,
        text_len: usize,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_script_session_eval_expression(
        session: slang_script_session,
        expr: slang_node,
        err: *mut slang_error,
    ) -> slang_constant;
    pub fn slang_script_session_eval_statement(
        session: slang_script_session,
        stmt: slang_node,
        err: *mut slang_error,
    );
    pub fn slang_script_session_compilation(
        session: slang_script_session,
        err: *mut slang_error,
    ) -> slang_compilation;
    pub fn slang_script_session_diagnostics(
        session: slang_script_session,
        err: *mut slang_error,
    ) -> slang_diagnostics;

    // MethodPrototypeSymbol / ModportClockingSymbol
    pub fn slang_symbol_method_prototype_flags(sym: slang_ast) -> u32;
    pub fn slang_symbol_method_prototype_subroutine_kind(sym: slang_ast) -> u32;
    pub fn slang_symbol_method_prototype_visibility(sym: slang_ast) -> slang_visibility;
    pub fn slang_symbol_method_prototype_is_virtual(sym: slang_ast) -> bool;
    pub fn slang_symbol_method_prototype_argument_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_method_prototype_argument(sym: slang_ast, index: u32) -> slang_ast;
    pub fn slang_symbol_method_prototype_return_type(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_method_prototype_subroutine(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_method_prototype_override(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_method_prototype_first_extern_impl(sym: slang_ast) -> slang_extern_impl;
    pub fn slang_extern_impl_next(impl_: slang_extern_impl) -> slang_extern_impl;
    pub fn slang_extern_impl_impl(impl_: slang_extern_impl, comp: slang_compilation) -> slang_ast;
    pub fn slang_symbol_modport_clocking_target(sym: slang_ast) -> slang_ast;

    // InterfacePortSymbol / LetDeclSymbol / Lookup
    pub fn slang_symbol_interface_port_connection(sym: slang_ast) -> slang_iface_conn;
    pub fn slang_instance_port_connection_iface_conn(
        instance: slang_ast,
        index: u32,
    ) -> slang_iface_conn;
    pub fn slang_symbol_interface_port_declared_range_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_interface_port_declared_range_at(
        sym: slang_ast,
        index: u32,
    ) -> slang_constant_range;
    pub fn slang_symbol_interface_port_interface_def(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_interface_port_is_generic(sym: slang_ast) -> bool;
    pub fn slang_symbol_interface_port_is_invalid(sym: slang_ast) -> bool;
    pub fn slang_symbol_interface_port_modport(sym: slang_ast) -> slang_str;
    pub fn slang_symbol_let_decl_port_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_let_decl_port(sym: slang_ast, index: u32) -> slang_ast;

    pub fn slang_lookup_get_visibility(symbol: slang_ast) -> slang_visibility;
    pub fn slang_lookup_is_visible_from(symbol: slang_ast, scope: slang_ast) -> bool;
    pub fn slang_lookup_is_accessible_from(target: slang_ast, source_scope: slang_ast) -> bool;
    pub fn slang_lookup_ensure_visible(
        symbol: slang_ast,
        context_scope: slang_ast,
        err: *mut slang_error,
    ) -> bool;
    pub fn slang_lookup_ensure_accessible(
        symbol: slang_ast,
        context_scope: slang_ast,
        err: *mut slang_error,
    ) -> bool;
    pub fn slang_lookup_name(
        context_scope: slang_ast,
        name: *const c_char,
        name_len: usize,
        err: *mut slang_error,
    ) -> slang_ast;
    pub fn slang_lookup_find_class(
        context_scope: slang_ast,
        name: *const c_char,
        name_len: usize,
        err: *mut slang_error,
    ) -> slang_ast;
    pub fn slang_lookup_find_assertion_local_var(
        assertion_inst: slang_ast,
        context_scope: slang_ast,
        name: *const c_char,
        name_len: usize,
        out_symbol: *mut slang_ast,
        err: *mut slang_error,
    ) -> bool;
    pub fn slang_lookup_find_temp_var(
        temp_var: slang_ast,
        context_scope: slang_ast,
        name: *const c_char,
        name_len: usize,
        out_symbol: *mut slang_ast,
        err: *mut slang_error,
    ) -> bool;

    // LookupLocation
    pub fn slang_lookup_location_before(symbol: slang_ast) -> slang_lookup_location;
    pub fn slang_lookup_location_after(symbol: slang_ast) -> slang_lookup_location;
    pub fn slang_lookup_location_max(comp: slang_compilation) -> slang_lookup_location;
    pub fn slang_lookup_location_min(comp: slang_compilation) -> slang_lookup_location;
    pub fn slang_lookup_location_get_scope(loc: slang_lookup_location) -> slang_ast;
    pub fn slang_lookup_location_get_index(loc: slang_lookup_location) -> u32;

    // LookupResult
    pub fn slang_lookup_result_create() -> slang_lookup_result;
    pub fn slang_lookup_result_destroy(result: slang_lookup_result);
    pub fn slang_lookup_within_class_randomize(
        class_type: slang_ast,
        this_var: slang_ast,
        context_scope: slang_ast,
        name: *const c_char,
        name_len: usize,
        result: slang_lookup_result,
        err: *mut slang_error,
    ) -> bool;
    pub fn slang_lookup_result_found(result: slang_lookup_result) -> slang_ast;
    pub fn slang_lookup_result_flags(result: slang_lookup_result) -> u32;
    pub fn slang_lookup_result_has_error(result: slang_lookup_result) -> bool;
    pub fn slang_lookup_result_system_subroutine(
        result: slang_lookup_result,
    ) -> slang_system_subroutine;
    pub fn slang_lookup_result_upward_count(result: slang_lookup_result) -> u32;
    pub fn slang_lookup_result_clear(result: slang_lookup_result) -> bool;
    pub fn slang_lookup_result_error_if_selectors(
        result: slang_lookup_result,
        context_scope: slang_ast,
        err: *mut slang_error,
    ) -> bool;
    pub fn slang_lookup_result_report_diags(
        result: slang_lookup_result,
        context_scope: slang_ast,
        err: *mut slang_error,
    ) -> bool;
    pub fn slang_lookup_result_diagnostics(result: slang_lookup_result) -> slang_diagnostics;
    pub fn slang_lookup_result_selector_count(result: slang_lookup_result) -> u32;
    pub fn slang_lookup_result_selector_is_member(result: slang_lookup_result, index: u32) -> bool;
    pub fn slang_lookup_result_selector_name(result: slang_lookup_result, index: u32) -> slang_str;
    pub fn slang_lookup_result_selector_dot_location(
        result: slang_lookup_result,
        index: u32,
    ) -> slang_loc;
    pub fn slang_lookup_result_selector_name_range(
        result: slang_lookup_result,
        index: u32,
    ) -> slang_range;

    pub fn slang_symbol_modport_port_direction(sym: slang_ast) -> slang_argument_direction;
    pub fn slang_symbol_modport_port_explicit_connection(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_modport_port_internal_symbol(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_modport_has_exports(sym: slang_ast) -> bool;

    pub fn slang_symbol_multi_port_direction(sym: slang_ast) -> slang_argument_direction;
    pub fn slang_symbol_multi_port_initializer(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_multi_port_type(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_multi_port_is_null_port(sym: slang_ast) -> bool;
    pub fn slang_symbol_multi_port_port_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_multi_port_port(sym: slang_ast, index: u32) -> slang_ast;

    pub fn slang_symbol_net_alias_reference_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_net_alias_reference(sym: slang_ast, index: u32) -> slang_ast;

    pub fn slang_symbol_net_expansion_hint(sym: slang_ast) -> slang_expansion_hint;
    pub fn slang_symbol_net_charge_strength(sym: slang_ast) -> slang_charge_strength;
    pub fn slang_symbol_net_delay(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_net_drive_strength(sym: slang_ast) -> slang_drive_strength_pair;
    pub fn slang_symbol_net_is_implicit(sym: slang_ast) -> bool;

    pub fn slang_symbol_package_default_lifetime(sym: slang_ast) -> slang_variable_lifetime;
    pub fn slang_symbol_package_find_for_import(
        sym: slang_ast,
        name: *const c_char,
        name_len: usize,
    ) -> slang_ast;
    pub fn slang_symbol_package_has_export_all(sym: slang_ast) -> bool;
    pub fn slang_symbol_package_time_scale(sym: slang_ast, out: *mut slang_time_scale) -> bool;

    // ParameterSymbolBase / ParameterSymbol
    pub fn slang_symbol_parameter_is_local_param(sym: slang_ast) -> bool;
    pub fn slang_symbol_parameter_is_port_param(sym: slang_ast) -> bool;
    pub fn slang_symbol_parameter_is_body_param(sym: slang_ast) -> bool;
    pub fn slang_symbol_parameter_is_overridden(sym: slang_ast) -> bool;
    pub fn slang_symbol_type_parameter_is_overridden(sym: slang_ast) -> bool;

    // PortSymbol
    pub fn slang_symbol_port_direction(sym: slang_ast) -> slang_argument_direction;
    pub fn slang_symbol_port_external_loc(sym: slang_ast) -> slang_loc;
    pub fn slang_symbol_port_initializer(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_port_internal_expr(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_port_type(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_port_internal_symbol(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_port_is_ansi_port(sym: slang_ast) -> bool;
    pub fn slang_symbol_port_is_net_port(sym: slang_ast) -> bool;
    pub fn slang_symbol_port_is_null_port(sym: slang_ast) -> bool;

    // PrimitiveInstanceSymbol / PrimitivePortSymbol / PrimitiveSymbol
    pub fn slang_symbol_primitive_port_direction(sym: slang_ast) -> slang_primitive_port_direction;
    pub fn slang_symbol_primitive_kind(sym: slang_ast) -> slang_primitive_kind;
    pub fn slang_symbol_primitive_is_sequential(sym: slang_ast) -> bool;
    pub fn slang_symbol_primitive_init_val(sym: slang_ast, err: *mut slang_error)
    -> slang_constant;
    pub fn slang_symbol_primitive_port_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_primitive_port(sym: slang_ast, index: u32) -> slang_ast;
    pub fn slang_symbol_primitive_table_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_primitive_table_entry_inputs(sym: slang_ast, index: u32) -> slang_str;
    pub fn slang_symbol_primitive_table_entry_output(sym: slang_ast, index: u32) -> slang_str;
    pub fn slang_symbol_primitive_table_entry_state(sym: slang_ast, index: u32) -> slang_str;
    pub fn slang_symbol_primitive_instance_port_connection_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_primitive_instance_port_connection(sym: slang_ast, index: u32)
    -> slang_ast;
    pub fn slang_symbol_primitive_instance_delay(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_primitive_instance_drive_strength(
        sym: slang_ast,
    ) -> slang_drive_strength_pair;

    // ProceduralBlockSymbol
    pub fn slang_symbol_procedural_block_block_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_procedural_block_block(sym: slang_ast, index: u32) -> slang_ast;
    pub fn slang_symbol_procedural_block_procedure_kind(
        sym: slang_ast,
    ) -> slang_procedural_block_kind;
    pub fn slang_symbol_procedural_block_is_single_driver_block(sym: slang_ast) -> bool;

    // PropertySymbol
    pub fn slang_symbol_property_port_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_property_port(sym: slang_ast, index: u32) -> slang_ast;

    // TimingPathSymbol
    pub fn slang_symbol_timing_path_connection_kind(
        sym: slang_ast,
    ) -> slang_timing_path_connection_kind;
    pub fn slang_symbol_timing_path_polarity(sym: slang_ast) -> slang_timing_path_polarity;
    pub fn slang_symbol_timing_path_edge_polarity(sym: slang_ast) -> slang_timing_path_polarity;
    pub fn slang_symbol_timing_path_edge_identifier(sym: slang_ast) -> slang_edge_kind;
    pub fn slang_symbol_timing_path_is_state_dependent(sym: slang_ast) -> bool;
    pub fn slang_symbol_timing_path_condition_expr(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_timing_path_edge_source_expr(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_timing_path_input_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_timing_path_input(sym: slang_ast, index: u32) -> slang_ast;
    pub fn slang_symbol_timing_path_output_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_timing_path_output(sym: slang_ast, index: u32) -> slang_ast;
    pub fn slang_symbol_timing_path_delay_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_timing_path_delay(sym: slang_ast, index: u32) -> slang_ast;

    // PulseStyleSymbol
    pub fn slang_symbol_pulse_style_kind(sym: slang_ast) -> slang_pulse_style_kind;
    pub fn slang_symbol_pulse_style_terminal_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_pulse_style_terminal(sym: slang_ast, index: u32) -> slang_ast;

    // RandSeqProductionSymbol
    pub fn slang_randseq_prod_is_null(prod: slang_randseq_prod) -> bool;
    pub fn slang_symbol_randseq_rule_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_randseq_rule_prod_count(sym: slang_ast, rule_index: u32) -> u32;
    pub fn slang_symbol_randseq_rule_prod(
        sym: slang_ast,
        rule_index: u32,
        prod_index: u32,
    ) -> slang_randseq_prod;
    pub fn slang_randseq_prod_get_kind(prod: slang_randseq_prod) -> slang_randseq_prod_kind;
    pub fn slang_randseq_prod_item_target(prod: slang_randseq_prod) -> slang_ast;
    pub fn slang_randseq_prod_item_arg_count(prod: slang_randseq_prod) -> u32;
    pub fn slang_randseq_prod_item_arg(prod: slang_randseq_prod, index: u32) -> slang_ast;
    pub fn slang_randseq_prod_code_block_block(prod: slang_randseq_prod) -> slang_ast;
    pub fn slang_randseq_prod_if_else_expr(prod: slang_randseq_prod) -> slang_ast;
    pub fn slang_randseq_prod_if_else_if_item(prod: slang_randseq_prod) -> slang_randseq_prod;
    pub fn slang_randseq_prod_if_else_has_else_item(prod: slang_randseq_prod) -> bool;
    pub fn slang_randseq_prod_if_else_else_item(prod: slang_randseq_prod) -> slang_randseq_prod;
    pub fn slang_randseq_prod_repeat_expr(prod: slang_randseq_prod) -> slang_ast;
    pub fn slang_randseq_prod_repeat_item(prod: slang_randseq_prod) -> slang_randseq_prod;
    pub fn slang_randseq_prod_case_expr(prod: slang_randseq_prod) -> slang_ast;
    pub fn slang_randseq_prod_case_item_count(prod: slang_randseq_prod) -> u32;
    pub fn slang_randseq_prod_case_item_expression_count(
        prod: slang_randseq_prod,
        item_index: u32,
    ) -> u32;
    pub fn slang_randseq_prod_case_item_expression(
        prod: slang_randseq_prod,
        item_index: u32,
        expr_index: u32,
    ) -> slang_ast;
    pub fn slang_randseq_prod_case_item_item(
        prod: slang_randseq_prod,
        item_index: u32,
    ) -> slang_randseq_prod;
    pub fn slang_randseq_prod_case_has_default_item(prod: slang_randseq_prod) -> bool;
    pub fn slang_randseq_prod_case_default_item(prod: slang_randseq_prod) -> slang_randseq_prod;
    pub fn slang_symbol_randseq_rule_block(sym: slang_ast, rule_index: u32) -> slang_ast;
    pub fn slang_symbol_randseq_rule_weight_expr(sym: slang_ast, rule_index: u32) -> slang_ast;
    pub fn slang_symbol_randseq_rule_is_rand_join(sym: slang_ast, rule_index: u32) -> bool;
    pub fn slang_symbol_randseq_rule_rand_join_expr(sym: slang_ast, rule_index: u32) -> slang_ast;
    pub fn slang_symbol_randseq_rule_has_code_block(sym: slang_ast, rule_index: u32) -> bool;
    pub fn slang_symbol_randseq_rule_code_block(
        sym: slang_ast,
        rule_index: u32,
    ) -> slang_randseq_prod;
    pub fn slang_symbol_randseq_argument_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_randseq_argument(sym: slang_ast, index: u32) -> slang_ast;
    pub fn slang_symbol_randseq_return_type(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_sequence_port_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_sequence_port(sym: slang_ast, index: u32) -> slang_ast;
    pub fn slang_symbol_specparam_is_path_pulse(sym: slang_ast) -> bool;
    pub fn slang_symbol_specparam_path_source(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_specparam_path_dest(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_statement_block_kind(sym: slang_ast) -> u32;
    pub fn slang_symbol_statement_block_default_lifetime(sym: slang_ast)
    -> slang_variable_lifetime;
    pub fn slang_symbol_subroutine_default_lifetime(sym: slang_ast) -> slang_variable_lifetime;
    pub fn slang_symbol_subroutine_flags(sym: slang_ast) -> u32;
    pub fn slang_symbol_subroutine_argument_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_subroutine_argument(sym: slang_ast, index: u32) -> slang_ast;
    pub fn slang_symbol_subroutine_override(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_subroutine_prototype(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_subroutine_return_type(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_subroutine_kind(sym: slang_ast) -> u32;
    pub fn slang_symbol_subroutine_is_virtual(sym: slang_ast) -> bool;
    pub fn slang_symbol_subroutine_return_val_var(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_subroutine_this_var(sym: slang_ast) -> slang_ast;
    pub fn slang_symbol_system_timing_check_kind(sym: slang_ast) -> slang_system_timing_check_kind;
    pub fn slang_symbol_system_timing_check_argument_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_system_timing_check_argument_expr(sym: slang_ast, index: u32) -> slang_ast;
    pub fn slang_symbol_system_timing_check_argument_condition(
        sym: slang_ast,
        index: u32,
    ) -> slang_ast;
    pub fn slang_symbol_system_timing_check_argument_edge(
        sym: slang_ast,
        index: u32,
    ) -> slang_edge_kind;
    pub fn slang_symbol_system_timing_check_argument_edge_descriptor_count(
        sym: slang_ast,
        arg_index: u32,
    ) -> u32;
    pub fn slang_symbol_system_timing_check_argument_edge_descriptor(
        sym: slang_ast,
        arg_index: u32,
        desc_index: u32,
    ) -> slang_str;

    // UninstantiatedDefSymbol
    pub fn slang_symbol_uninstantiated_def_definition_name(sym: slang_ast) -> slang_str;
    pub fn slang_symbol_uninstantiated_def_param_expression_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_uninstantiated_def_param_expression(
        sym: slang_ast,
        index: u32,
    ) -> slang_ast;
    pub fn slang_symbol_uninstantiated_def_port_connection_count(sym: slang_ast) -> u32;
    pub fn slang_symbol_uninstantiated_def_port_connection(sym: slang_ast, index: u32)
    -> slang_ast;
    pub fn slang_symbol_uninstantiated_def_port_name(sym: slang_ast, index: u32) -> slang_str;
    pub fn slang_symbol_uninstantiated_def_is_checker(sym: slang_ast) -> bool;
}
