//! Safe, ergonomic Rust bindings to [slang](https://sv-lang.com), the fast,
//! standards-compliant SystemVerilog compiler frontend.
//!
//! `sv-lang` gives Rust the whole of slang — not just parsing, but the
//! elaborated semantic model (symbols, types, name resolution, constant
//! evaluation) and lint/driver analysis that until now existed only behind
//! slang's C++ and Python APIs.
//!
//! - **Parse** into a lossless concrete syntax tree with byte-exact round-trip,
//!   and read [`Diagnostics`].
//! - **Elaborate** a [`Compilation`] into a frozen, `Send + Sync` [`Design`] and
//!   read [`Symbol`]s, [`Type`]s, [`Expression`]s, scope iteration, name lookup
//!   and constant values.
//! - **Analyze** the design for unused-code lints and per-signal driver tracking.
//!
//! # Example
//!
//! ```
//! use sv_lang::{AnalysisFlags, Compilation, Session};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let session = Session::new();
//!
//! // Parse: a lossless concrete syntax tree.
//! let tree = session.parse("module counter(input clk); logic [7:0] q; endmodule\n")?;
//! assert_eq!(tree.module_names().collect::<Vec<_>>(), ["counter"]);
//!
//! // Elaborate: a fully-frozen, thread-shareable semantic design.
//! let mut comp = Compilation::new(&session)?;
//! comp.add(&tree)?;
//! let design = comp.compile()?;
//! let counter = design.top_instances().next().unwrap();
//! let q = counter.instance_body().unwrap().find("q").unwrap();
//! assert_eq!(q.value_type().unwrap().to_sv_string(), "logic[7:0]");
//!
//! // Analyze: unused-code lints and driver tracking.
//! let analysis = design.analyze(AnalysisFlags::CHECK_UNUSED, 0)?;
//! let _diagnostics = analysis.diagnostics().items().len();
//! # Ok(())
//! # }
//! ```
#![warn(missing_docs)]

mod ast;
mod constant;
pub mod dataflow;
mod driver;
mod error;
mod ffi;
mod script;
mod syntax;
mod system_subroutine_kinds;

pub use ast::{
    Analysis, AnalysisFlags, AnalysisListener, AnalyzedAssertion, AnalyzedProcedure,
    AnonymousTypeStyle, ArgumentDirection, AssertionInstanceArg, AssertionKind, BinaryOp,
    CallExtraKind, CaseItem, CaseStatementCondition, ChargeStrength, CheckerConnection,
    ClockingSkew, Compilation, CompilationId, CompilationOptions, CompileErrors, Condition,
    ConfigRule, ConstantRange, ConstraintBlockFlags, CoverageBinKind, CoverageOption,
    DefinitionKind, DefinitionLookupResult, Design, DimensionKind, DistItem, DistWeight,
    DistWeightKind, DpiExport, DriveStrength, DriveStrengthPair, DriverKind, DriverSource,
    EdgeKind, ElabSystemTaskKind, EvalSession, EvaluatedDimension, ExpansionHint, ExprCondition,
    Expression, ExternImpl, FloatKind, ForwardTypeRestriction, FreezeReport, GenerateBranchKind,
    ImplicitEventReadSet, IndexSetter, IntegralFlags, IntoOwned, LValue, LookupLocation,
    LookupResult, LookupResultFlags, LookupSelector, LoopDim, MemberSetter, MethodFlags, NetKind,
    NetTypeKind, Options, OwnedExpression, OwnedStatement, OwnedSymbol, OwnedType, Pattern,
    PatternCaseItem, PredefinedIntegerKind, PrimitiveKind, PrimitivePortDirection,
    ProceduralBlockKind, PulseStyleKind, RandCaseItem, RandMode, RandSeqProd, RandSeqProdKind,
    RawDiagnostics, RawDriverFlags, ReadRange, RepeatKind, ScalarKind, SemNode, SensitivityKind,
    SensitivityList, SourceLibrary, SourceLoc, SourceSpan, Statement, StatementBlockKind,
    StatementEvalResult, StreamExpr, Symbol, SymbolId, SystemMethod, SystemTimingCheckArg,
    SystemTimingCheckKind, TimeScale, TimeScaleValue, TimeUnit, TimingPathConnectionKind,
    TimingPathPolarity, TransRange, TransSet, Type, TypePrinter, TypePrintingOptions, TypeSetter,
    UnaryOp, UnconnectedDrive, UnfreezeGuard, UniquePriorityCheck, ValueDriver, ValuePath,
    VariableFlags, VariableLifetime, Visibility, WithClauseMode,
};
pub use constant::{Bit, ConstantValue, Digit, LiteralBase, OwnedSVInt, SVInt};
pub use driver::{
    AnalysisOptions, CommandFileMetadata, DiagEngine, Driver, DriverAnalysis, LoadedSourceBuffer,
    OptionKind, ParseOptions, PreprocessFlags, SourceLoader, SourceOptions, TextDiagClient,
};
pub use error::{Diagnostic, Diagnostics, Error};
pub use script::{ScriptCompilation, ScriptSession};
/// Typed views of every slang syntax node, generated from slang's schema.
pub use syntax::nodes;
/// A generated `syn::visit`-style visitor over the typed syntax tree.
pub use syntax::visitor;
pub use syntax::{
    AstNode, Child, Descendants, Node, SeparatedList, SyntaxList, SyntaxTree, Token, TokenList,
    Walk,
};
pub use system_subroutine_kinds::{KnownSystemName, SubroutineKind};

/// Re-export of the kind enumerations.
pub use sv_lang_kinds as kinds;
/// Re-export of the pure-Rust syntax-tree representation, for tools that keep
/// trees after the session is gone (see [`SyntaxTree::mirror`]).
pub use sv_lang_syntax as green;

use std::sync::Arc;

use sv_lang_sys as sys;

/// Reads a static, NUL-terminated string the linked library returns (never
/// freed). SAFETY: `p` must point at such a string.
unsafe fn static_cstr(p: *const core::ffi::c_char) -> &'static str {
    // SAFETY: guaranteed by the caller.
    unsafe { core::ffi::CStr::from_ptr(p) }
        .to_str()
        .unwrap_or("")
}

/// The slang version the linked library reports, e.g. `"11.0.3+abc1234"`.
///
/// # Examples
/// ```
/// let version = sv_lang::slang_version();
/// assert!(!version.is_empty());
/// ```
pub fn slang_version() -> &'static str {
    // SAFETY: the library returns a static NUL-terminated string.
    unsafe { static_cstr(sys::slang_version_string()) }
}

/// The syntax-model hash the linked slang-c reports. It equals
/// [`sv_lang_kinds::SYNTAX_MODEL_HASH`] for a library whose kind tables match
/// these bindings — the equality the startup ABI probe enforces (see the module
/// docs on the `system` feature). Exposed so a `system`-feature build can assert
/// the probe target directly.
///
/// # Examples
/// ```
/// assert_eq!(sv_lang::slang_syntax_model_hash(), sv_lang::kinds::SYNTAX_MODEL_HASH);
/// ```
pub fn slang_syntax_model_hash() -> &'static str {
    // SAFETY: the library returns a static NUL-terminated string.
    unsafe { static_cstr(sys::slang_syntax_model_hash()) }
}

/// Whether the linked slang-c was built with assertions enabled — the
/// seal-enforcement the `Design: Sync` guarantee relies on. Always `true` for
/// the default vendored build; the `system` feature refuses a library where it
/// is `false`.
///
/// # Examples
/// ```
/// assert!(sv_lang::slang_has_assertions());
/// ```
pub fn slang_has_assertions() -> bool {
    // SAFETY: a pure, precondition-free C accessor.
    unsafe { sys::slang_build_flags() & sys::SLANG_BUILD_ASSERTIONS != 0 }
}

/// Owns a slang source manager: the arena that holds loaded source text and
/// resolves locations. Every tree parsed from one session shares its manager,
/// so their locations are comparable.
///
/// A session is cheap to create; make one per independent piece of work.
///
/// # Examples
/// ```
/// # fn main() -> Result<(), sv_lang::Error> {
/// let session = sv_lang::Session::new();
/// let tree = session.parse("module top; endmodule\n")?;
/// assert_eq!(tree.module_names().collect::<Vec<_>>(), ["top"]);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Session {
    inner: Arc<SessionInner>,
}

struct SessionInner {
    sm: sys::slang_source_manager,
    backing: SessionBacking,
}

/// How the source manager behind a [`Session`] is owned.
enum SessionBacking {
    /// The session created and owns the source manager; it is destroyed on drop.
    Owned,
    /// The source manager is owned by something else (a [`Driver`]); this
    /// keepalive keeps that owner alive as long as the session, and the session
    /// does not destroy the manager.
    Borrowed(#[allow(dead_code)] Arc<dyn core::any::Any + Send + Sync>),
}

// SAFETY: slang's SourceManager is internally synchronized (shared_mutex), so
// the handle may be used and dropped from any thread.
unsafe impl Send for SessionInner {}
// SAFETY: as above — the SourceManager guards its own state.
unsafe impl Sync for SessionInner {}

impl Drop for SessionInner {
    fn drop(&mut self) {
        // Only an owned session destroys the manager; a borrowed one leaves it
        // to its real owner (kept alive by the keepalive).
        if matches!(self.backing, SessionBacking::Owned) {
            // SAFETY: no trees from this manager outlive the session (each holds
            // an Arc<SessionInner>), so destroying it here is safe.
            unsafe { sys::slang_source_manager_destroy(self.sm) };
        }
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for Session {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Session")
            .field("source_manager", &self.inner.sm)
            .finish()
    }
}

/// Two sessions are equal iff they share the same underlying source manager
/// (e.g. one obtained via [`Compilation::source_manager`](crate::Compilation::source_manager)
/// pointing back at the session it was built from), not merely equal content.
impl PartialEq for Session {
    fn eq(&self, other: &Self) -> bool {
        self.inner.sm == other.inner.sm
    }
}

impl Eq for Session {}

/// Verifies, once per process, that the linked slang-c library matches what
/// these bindings were generated and reasoned about: built with assertions on
/// (the seal enforcement that upholds the `Design: Sync` guarantee) and with
/// exceptions on (the C API cannot carry slang errors safely otherwise), and
/// carrying the same syntax/diagnostics model hash the kind tables came from.
///
/// For the default in-tree/vendored build this is a tautology; it exists to
/// stop the `system` feature from silently linking a mismatched or
/// assertions-off prebuilt library, which would degrade a soundness abort into
/// silent memory corruption or drift the kind ordinals.
fn verify_abi() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        // SAFETY: pure, always-safe C accessors with no preconditions.
        let flags = unsafe { sys::slang_build_flags() };
        assert!(
            flags & sys::SLANG_BUILD_ASSERTIONS != 0,
            "linked slang-c was built without assertions; sv-lang's Design: Sync guarantee \
             requires an assertions-on build (the post-freeze allocation guard)"
        );
        assert!(
            flags & sys::SLANG_BUILD_EXCEPTIONS != 0,
            "linked slang-c was built with -fno-exceptions; the C API boundary cannot contain \
             slang errors without exceptions"
        );
        // SAFETY: returns a static NUL-terminated string owned by the library.
        let syntax = unsafe { core::ffi::CStr::from_ptr(sys::slang_syntax_model_hash()) };
        assert!(
            syntax.to_bytes() == sv_lang_kinds::SYNTAX_MODEL_HASH.as_bytes(),
            "linked slang-c syntax model hash does not match sv-lang-kinds; the library and the \
             generated kind tables are out of sync (rebuild or fix the `system` library)"
        );
    });
}

impl Session {
    /// Creates a new session.
    ///
    /// # Panics
    /// Panics only if the library fails to allocate the source manager, which
    /// in practice means the process is out of memory.
    ///
    /// # Examples
    /// ```
    /// let session = sv_lang::Session::new();
    /// assert!(session.parse("module m; endmodule\n").is_ok());
    /// ```
    pub fn new() -> Self {
        verify_abi();
        let mut err = ffi::error();
        // SAFETY: standard construction; the out-error is checked.
        let sm = unsafe { sys::slang_source_manager_create(&mut err) };
        assert!(
            !sm.is_null(),
            "failed to create slang source manager: {}",
            ffi::message(&err)
        );
        Session {
            inner: Arc::new(SessionInner {
                sm,
                backing: SessionBacking::Owned,
            }),
        }
    }

    /// Wraps a source manager owned by `keepalive` (a [`Driver`]) without taking
    /// ownership of it. The keepalive keeps the real owner alive as long as any
    /// tree, compilation or design derived from this session.
    pub(crate) fn from_borrowed(
        sm: sys::slang_source_manager,
        keepalive: Arc<dyn core::any::Any + Send + Sync>,
    ) -> Session {
        verify_abi();
        Session {
            inner: Arc::new(SessionInner {
                sm,
                backing: SessionBacking::Borrowed(keepalive),
            }),
        }
    }

    /// Parses SystemVerilog source text as a compilation unit.
    ///
    /// Syntax errors do not fail this call; they are available through
    /// [`SyntaxTree::diagnostics`]. An [`Err`] means the tree could not be
    /// built at all.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module counter; endmodule\n")?;
    /// assert!(!tree.diagnostics().has_errors());
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse(&self, text: &str) -> Result<SyntaxTree, Error> {
        self.parse_named(text, "source", "")
    }

    /// As [`parse`](Self::parse), naming the buffer (shown in diagnostics) and
    /// giving it a path (used to resolve `` `include ``).
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse_named("module m; endmodule\n", "cpu.sv", "/work/cpu.sv")?;
    /// assert_eq!(tree.module_names().count(), 1);
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_named(&self, text: &str, name: &str, path: &str) -> Result<SyntaxTree, Error> {
        let mut err = ffi::error();
        let (t, tl) = ffi::as_ptr_len(text);
        let (n, nl) = ffi::as_ptr_len(name);
        let (p, pl) = ffi::as_ptr_len(path);
        // SAFETY: all pointers are valid for the call; the out-error is checked.
        let tree = unsafe {
            sys::slang_syntax_tree_from_text(
                self.inner.sm,
                t,
                tl,
                n,
                nl,
                p,
                pl,
                core::ptr::null_mut(),
                &mut err,
            )
        };
        ffi::check(&err)?;
        Ok(SyntaxTree::from_raw(tree, self.clone()))
    }

    /// Parses a file from disk as a compilation unit.
    ///
    /// # Examples
    /// ```no_run
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse_file("cpu.sv")?;
    /// for name in tree.module_names() {
    ///     println!("{name}");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_file(&self, path: impl AsRef<std::path::Path>) -> Result<SyntaxTree, Error> {
        let path = path.as_ref().to_string_lossy();
        let mut err = ffi::error();
        let (p, pl) = ffi::as_ptr_len(&path);
        // SAFETY: `p`/`pl` describe a valid string; the out-error is checked.
        let tree = unsafe {
            sys::slang_syntax_tree_from_file(self.inner.sm, p, pl, core::ptr::null_mut(), &mut err)
        };
        ffi::check(&err)?;
        Ok(SyntaxTree::from_raw(tree, self.clone()))
    }

    pub(crate) fn raw(&self) -> sys::slang_source_manager {
        self.inner.sm
    }
}

/// Renders a location's file/line/column from a session's source manager.
pub(crate) fn location_of(
    sm: sys::slang_source_manager,
    loc: sys::slang_loc,
) -> (String, usize, usize) {
    if loc.buffer == 0 {
        return (String::new(), 0, 0);
    }
    // SAFETY: `sm` is live and `loc` came from a tree parsed with it.
    unsafe {
        let file = ffi::borrowed_str(sys::slang_source_manager_file_name(sm, loc));
        let line = sys::slang_source_manager_line(sm, loc);
        let column = sys::slang_source_manager_column(sm, loc);
        (file, line, column)
    }
}

/// Collects an owned [`Diagnostics`] from a C diagnostics handle, consuming
/// (destroying) it.
pub(crate) fn collect_diagnostics(
    sm: sys::slang_source_manager,
    diags: sys::slang_diagnostics,
) -> Diagnostics {
    use std::collections::HashMap;
    use std::sync::Arc;

    let mut items = Vec::new();
    let mut err = ffi::error();
    // Intern each buffer's source text once and share the Arc across the
    // diagnostics that point into it.
    let mut sources: HashMap<u32, Option<Arc<str>>> = HashMap::new();
    // SAFETY: `diags` is a valid handle we own; freed at the end.
    unsafe {
        let count = sys::slang_diagnostics_count(diags);
        for i in 0..count {
            let mut d = sys::slang_diag::default();
            if !sys::slang_diagnostics_at(diags, i, &mut d) {
                continue;
            }
            let message = ffi::owned_str(sys::slang_diagnostics_message(diags, i, &mut err));
            let (file, line, column) = location_of(sm, d.location);

            // Primary byte span: the first highlighted range if present, else a
            // one-byte point at the location.
            let (byte_offset, byte_len) = if d.range_count > 0 {
                let mut range = sys::slang_range::default();
                if sys::slang_diagnostics_range(diags, i, 0, &mut range) {
                    let start = range.start.offset as usize;
                    let end = range.end.offset as usize;
                    (start, end.saturating_sub(start).max(1))
                } else {
                    (d.location.offset as usize, 1)
                }
            } else {
                (d.location.offset as usize, 1)
            };

            let source = if d.location.buffer != 0 && !sm.is_null() {
                sources
                    .entry(d.location.buffer)
                    .or_insert_with(|| {
                        let text = ffi::borrowed_str(sys::slang_source_manager_text(
                            sm,
                            d.location.buffer,
                        ));
                        (!text.is_empty()).then(|| Arc::<str>::from(text.as_str()))
                    })
                    .clone()
            } else {
                None
            };

            items.push(Diagnostic {
                code: kinds::DiagCode::from_raw(d.code)
                    .unwrap_or(kinds::DiagCode::new(kinds::DiagSubsystem::General, 0)),
                severity: severity_from_raw(d.severity),
                message,
                line,
                column,
                file,
                byte_offset,
                byte_len,
                source,
            });
        }
        let rendered = ffi::owned_str(sys::slang_diagnostics_render(
            diags,
            core::ptr::null(),
            &mut err,
        ));
        sys::slang_diagnostics_destroy(diags);
        Diagnostics { items, rendered }
    }
}

fn severity_from_raw(raw: sys::slang_severity) -> kinds::DiagSeverity {
    match raw {
        sys::SLANG_SEVERITY_NOTE => kinds::DiagSeverity::Note,
        sys::SLANG_SEVERITY_WARNING => kinds::DiagSeverity::Warning,
        sys::SLANG_SEVERITY_ERROR => kinds::DiagSeverity::Error,
        sys::SLANG_SEVERITY_FATAL => kinds::DiagSeverity::Fatal,
        _ => kinds::DiagSeverity::Ignored,
    }
}
