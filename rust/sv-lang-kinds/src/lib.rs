//! Kind enumerations for [slang](https://sv-lang.com), the SystemVerilog compiler frontend.
//!
//! This crate contains only plain Rust enums and tables — no native code, no
//! dependencies, `no_std` — generated from slang's own schema files
//! (`scripts/syntax.txt`, `scripts/tokenkinds.txt`, `scripts/triviakinds.txt`,
//! `scripts/diagnostics.txt`). Discriminants match the C++ enums exactly.
//!
//! It exists as a separate crate so that tooling which only needs to *name*
//! kinds — lint rule tables, editor integrations, configuration parsers — can
//! depend on it without linking the native library, and without being pinned
//! to the linking crate's release cadence.
//!
//! # Example
//!
//! ```
//! use sv_lang_kinds::{SyntaxKind, DiagCode, diag};
//!
//! assert_eq!(SyntaxKind::Unknown.as_raw(), 0);
//! assert_eq!(SyntaxKind::from_raw(0), Some(SyntaxKind::Unknown));
//! assert_eq!(SyntaxKind::from_raw(SyntaxKind::COUNT), None);
//!
//! let code: DiagCode = diag::UnknownModule;
//! assert_eq!(code.name(), "UnknownModule");
//! assert_eq!(DiagCode::from_raw(code.as_raw()), Some(code));
//! ```
#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod generated {
    pub mod ast_kind;
    pub mod diag_code;
    pub mod syntax_kind;
    pub mod token_kind;
}

pub use generated::ast_kind::{
    AST_KINDS_MODEL_HASH, AssertionExprKind, BinsSelectExprKind, ConstraintKind, ExpressionKind,
    PatternKind, StatementKind, SymbolKind, TimingControlKind,
};
pub use generated::diag_code::{
    DIAG_GROUPS, DIAGNOSTICS_MODEL_HASH, DiagCode, DiagGroup, DiagSeverity, DiagSubsystem, diag,
};
pub use generated::syntax_kind::{SLANG_VERSION, SYNTAX_MODEL_HASH, SyntaxKind, SyntaxStruct};
pub use generated::token_kind::{TokenKind, TriviaKind};
