//! The elaborated (semantic) AST: [`Compilation`], the sealed [`Design`], and
//! the borrowed [`Symbol`] / [`Type`] / [`Expression`] handles over it.
//!
//! # Soundness model
//!
//! Building a design is a `&mut` operation on a [`Compilation`]. Calling
//! [`Compilation::compile`] finalizes and *freezes* it: every scope is fully
//! elaborated (instance caching disabled so no body is skipped), every
//! constant is folded, and the arena is sealed against further allocation.
//! The result is a [`Design`], which is `Send + Sync`: all of its read
//! accessors are genuine reads that never mutate the frozen arena, so any
//! number of threads may traverse it at once.
//!
//! Two operations are *not* pure reads, because slang implements them with a
//! mutation: evaluating a not-yet-folded constant ([`Design::eval`]), and
//! full name lookup ([`Design::lookup`], which records reference-tracking
//! state). Both therefore take `&mut Design` and are unavailable while the
//! design is shared. [`Symbol::find`] (a direct name-table read) and scope
//! iteration stay on `&self`.

use core::marker::PhantomData;
use std::sync::Arc;

use sv_lang_kinds::{ExpressionKind, SymbolKind};
use sv_lang_sys as sys;

use crate::{Diagnostic, Diagnostics, Error, Session, SyntaxTree, ffi, syntax::Node};

/// A compilation in progress: syntax trees are added, then [`compile`](Self::compile)
/// elaborates and freezes them into a [`Design`]. Not `Sync` — building mutates.
pub struct Compilation {
    raw: sys::slang_compilation,
    session: Session,
    // Keeps added trees alive for the compilation's lifetime.
    _trees: Vec<SyntaxTree>,
}

// SAFETY: the raw compilation is owned exclusively (no `Clone`, no shared
// handle) and only touched through `&mut self`, so it may move between threads.
unsafe impl Send for Compilation {}

impl Drop for Compilation {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            // SAFETY: we own the compilation and no Design took it.
            unsafe { sys::slang_compilation_destroy(self.raw) };
        }
    }
}

/// Bits controlling how a compilation is built. Values match slang's
/// `CompilationFlags`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Options {
    flags: u32,
}

impl Options {
    /// Default options.
    ///
    /// # Examples
    /// ```
    /// let opts = sv_lang::Options::new();
    /// # let _ = opts;
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the raw `CompilationFlags` bitmask. `DisableInstanceCaching` is
    /// added automatically by [`Compilation::new_with`] so that freezing is
    /// total; callers do not need to set it.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let opts = sv_lang::Options::new().with_flags(0);
    /// let comp = sv_lang::Compilation::new_with(&session, opts)?;
    /// # let _ = comp;
    /// # Ok(()) }
    /// ```
    pub fn with_flags(mut self, flags: u32) -> Self {
        self.flags = flags;
        self
    }
}

impl Compilation {
    /// Creates an empty compilation.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let comp = sv_lang::Compilation::new(&session)?;
    /// # let _ = comp;
    /// # Ok(()) }
    /// ```
    pub fn new(session: &Session) -> Result<Compilation, Error> {
        Self::new_with(session, Options::new())
    }

    /// Creates an empty compilation with the given options.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::{Compilation, Options, Session};
    /// let session = Session::new();
    /// let comp = Compilation::new_with(&session, Options::new())?;
    /// # let _ = comp;
    /// # Ok(()) }
    /// ```
    pub fn new_with(session: &Session, options: Options) -> Result<Compilation, Error> {
        let mut err = ffi::error();
        // Instance caching must be off for FREEZE_ELABORATE_ALL to be total.
        let flags = options.flags | sys::SLANG_COMP_DISABLE_INSTANCE_CACHING;
        // SAFETY: out-error checked; option handle is transient.
        let raw = unsafe {
            let opts = sys::slang_options_create(&mut err);
            sys::slang_options_set_compilation_flags(opts, flags);
            let comp = sys::slang_compilation_create(opts, &mut err);
            sys::slang_options_destroy(opts);
            comp
        };
        ffi::check(&err)?;
        Ok(Compilation {
            raw,
            session: session.clone(),
            _trees: Vec::new(),
        })
    }

    /// Adds a syntax tree. Fails if the tree came from a different session, or
    /// if the compilation has already been finalized.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// let mut comp = sv_lang::Compilation::new(&session)?;
    /// comp.add(&tree)?;
    /// let design = comp.compile()?;
    /// assert!(design.top_instances().next().is_some());
    /// # Ok(()) }
    /// ```
    pub fn add(&mut self, tree: &SyntaxTree) -> Result<(), Error> {
        let mut err = ffi::error();
        // SAFETY: both handles are valid; out-error checked.
        unsafe { sys::slang_compilation_add_tree(self.raw, tree.raw(), &mut err) };
        ffi::check(&err)?;
        self._trees.push(tree.clone());
        Ok(())
    }

    /// Convenience: parse `text` in the session and add it.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let mut comp = sv_lang::Compilation::new(&session)?;
    /// comp.add_source("module m; endmodule\n")?;
    /// let design = comp.compile()?;
    /// assert_eq!(design.definitions().count(), 1);
    /// # Ok(()) }
    /// ```
    pub fn add_source(&mut self, text: &str) -> Result<(), Error> {
        let tree = self.session.parse(text)?;
        self.add(&tree)
    }

    /// Finalizes, fully elaborates, constant-folds and seals the compilation,
    /// producing a shareable [`Design`].
    ///
    /// Elaboration and folding diagnostics are attached to the design (read
    /// them with [`Design::diagnostics`]); a design is produced even when the
    /// source has errors. An [`Err`] here means the freeze operation itself
    /// failed.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let mut comp = sv_lang::Compilation::new(&session)?;
    /// comp.add_source("module top; endmodule\n")?;
    /// let design = comp.compile()?;
    /// assert_eq!(design.top_instances().next().unwrap().name(), "top");
    /// # Ok(()) }
    /// ```
    pub fn compile(self) -> Result<Design, Error> {
        self.freeze_into(sys::SLANG_FREEZE_ALL)
    }

    /// **TEST-ONLY, DELIBERATELY UNSOUND.** Freezes with `SEAL` only, skipping
    /// the totalization sweep, so the returned design's canonical-type memos are
    /// NOT forced. Sharing it across threads is a data race — this exists solely
    /// to drive the thread-sanitizer negative gate that proves the normal
    /// [`compile`](Self::compile) totalization is what removes that race. It is
    /// never compiled into a normal build.
    #[cfg(feature = "__unsound_test_only")]
    #[doc(hidden)]
    pub fn compile_sealed_only(self) -> Result<Design, Error> {
        self.freeze_into(sys::SLANG_FREEZE_SEAL)
    }

    fn freeze_into(mut self, flags: u32) -> Result<Design, Error> {
        let mut err = ffi::error();
        let mut report = sys::slang_freeze_report::default();
        // SAFETY: we own the compilation; out-error checked.
        unsafe { sys::slang_compilation_freeze(self.raw, flags, &mut report, &mut err) };
        ffi::check(&err)?;

        let raw = self.raw;
        self.raw = core::ptr::null_mut(); // Design takes ownership; suppress our Drop.
        let session = self.session.clone();
        let trees = std::mem::take(&mut self._trees);
        Ok(Design {
            inner: Arc::new(DesignInner {
                raw,
                session,
                _trees: trees,
            }),
            report: FreezeReport::from_raw(report),
        })
    }
}

/// Builds a [`Design`] from an already-frozen raw compilation (used by the
/// driver, which creates and freezes the compilation itself). The `session`
/// keeps the source manager alive; the trees are kept alive by it too.
pub(crate) fn design_from_raw(
    comp: sys::slang_compilation,
    session: Session,
    report: sys::slang_freeze_report,
) -> Design {
    Design {
        inner: Arc::new(DesignInner {
            raw: comp,
            session,
            _trees: Vec::new(),
        }),
        report: FreezeReport::from_raw(report),
    }
}

/// What [`Compilation::compile`] did.
///
/// # Examples
/// ```
/// # fn main() -> Result<(), sv_lang::Error> {
/// let session = sv_lang::Session::new();
/// let mut comp = sv_lang::Compilation::new(&session)?;
/// comp.add_source("module m; localparam int X = 3 * 4; endmodule\n")?;
/// let design = comp.compile()?;
/// let report = design.freeze_report();
/// assert!(report.symbols_elaborated > 0);
/// assert!(report.fully_folded());
/// # Ok(()) }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct FreezeReport {
    /// Symbols visited by the elaboration sweep.
    pub symbols_elaborated: u64,
    /// Expressions visited by the prefold sweep.
    pub expressions_visited: u64,
    /// Expressions that now carry a cached constant value.
    pub expressions_folded: u64,
    /// Expressions the prefold sweep could not attempt or that threw. When
    /// this is non-zero, evaluating one of them later may still allocate; it
    /// does not affect the safety of pure reads.
    pub fold_failures: u64,
    /// Declared/canonical type memos forced by the sweep — the work that makes
    /// concurrent reads of alias and canonical types race-free.
    pub types_canonicalized: u64,
    /// Parameter/specparam constant values forced by the sweep.
    pub params_folded: u64,
}

impl FreezeReport {
    fn from_raw(r: sys::slang_freeze_report) -> Self {
        FreezeReport {
            symbols_elaborated: r.symbols_elaborated,
            expressions_visited: r.expressions_visited,
            expressions_folded: r.expressions_folded,
            fold_failures: r.fold_failures,
            types_canonicalized: r.types_canonicalized,
            params_folded: r.params_folded,
        }
    }

    /// True if every reachable expression was folded or proven non-constant,
    /// i.e. no later read can allocate.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; localparam int X = 3 * 4; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// assert!(design.freeze_report().fully_folded());
    /// # Ok(()) }
    /// ```
    pub fn fully_folded(&self) -> bool {
        self.fold_failures == 0
    }
}

/// A finalized, fully elaborated and sealed design. `Send + Sync`: every read
/// accessor is a genuine read of the frozen arena, so a design may be
/// traversed from any number of threads concurrently. Cloning is a cheap
/// reference-count bump.
#[derive(Clone)]
pub struct Design {
    inner: Arc<DesignInner>,
    report: FreezeReport,
}

impl core::fmt::Debug for Design {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Design")
            .field("freeze", &self.report)
            .finish()
    }
}

struct DesignInner {
    raw: sys::slang_compilation,
    session: Session,
    _trees: Vec<SyntaxTree>,
}

// SAFETY: a frozen compilation is immutable through every `&self` accessor
// this crate exposes (structure reads, cached-constant reads); the two
// allocating operations require `&mut Design`. The source manager is
// internally synchronized. Therefore the design is safe to send and share.
unsafe impl Send for DesignInner {}
// SAFETY: as above — reads never mutate the frozen arena.
unsafe impl Sync for DesignInner {}

impl Drop for DesignInner {
    fn drop(&mut self) {
        // SAFETY: the last Arc owner destroys the compilation; no handle outlives it.
        unsafe { sys::slang_compilation_destroy(self.raw) };
    }
}

impl Design {
    /// What the freeze did.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let report = design.freeze_report();
    /// assert!(report.symbols_elaborated > 0);
    /// # Ok(()) }
    /// ```
    pub fn freeze_report(&self) -> FreezeReport {
        self.report
    }

    /// The session whose source manager the design's trees share.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let mut comp = sv_lang::Compilation::new(&session)?;
    /// comp.add_source("module m; endmodule\n")?;
    /// let design = comp.compile()?;
    /// // The design's session parses more source against the same manager.
    /// let tree = design.session().parse("module n; endmodule\n")?;
    /// assert_eq!(tree.module_names().collect::<Vec<_>>(), ["n"]);
    /// # Ok(()) }
    /// ```
    pub fn session(&self) -> &Session {
        &self.inner.session
    }

    pub(crate) fn raw_compilation(&self) -> sys::slang_compilation {
        self.inner.raw
    }

    fn raw(&self) -> sys::slang_compilation {
        self.inner.raw
    }

    /// The root symbol of the design.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let root = design.root();
    /// assert!(root.parent().is_none());
    /// assert!(root.is_scope());
    /// # Ok(()) }
    /// ```
    pub fn root(&self) -> Symbol<'_> {
        let mut err = ffi::error();
        // SAFETY: the compilation is valid.
        let ast = unsafe { sys::slang_compilation_root(self.raw(), &mut err) };
        Symbol::from_raw(ast)
    }

    /// The top-level module/program instances.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module top; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let names: Vec<_> = design.top_instances().map(|i| i.name().to_string()).collect();
    /// assert_eq!(names, ["top"]);
    /// # Ok(()) }
    /// ```
    pub fn top_instances(&self) -> impl Iterator<Item = Symbol<'_>> {
        // SAFETY: the compilation is valid.
        let count = unsafe { sys::slang_compilation_top_instance_count(self.raw()) };
        (0..count).filter_map(move |i| {
            // SAFETY: the compilation is valid; index is in range.
            let ast = unsafe { sys::slang_compilation_top_instance(self.raw(), i) };
            wrap(ast)
        })
    }

    /// The definitions (modules, interfaces, programs) in a deterministic order.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module a; endmodule\nmodule b; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let names: Vec<_> = design.definitions().map(|d| d.name().to_string()).collect();
    /// assert!(names.contains(&"a".to_string()));
    /// assert!(names.contains(&"b".to_string()));
    /// # Ok(()) }
    /// ```
    pub fn definitions(&self) -> impl Iterator<Item = Symbol<'_>> {
        // SAFETY: the compilation is valid.
        let count = unsafe { sys::slang_compilation_definition_count(self.raw()) };
        (0..count).filter_map(move |i| {
            // SAFETY: the compilation is valid; index is in range.
            let ast = unsafe { sys::slang_compilation_definition(self.raw(), i) };
            wrap(ast)
        })
    }

    /// The packages.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("package pkg; localparam int K = 5; endpackage\nmodule m; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let names: Vec<_> = design.packages().map(|p| p.name().to_string()).collect();
    /// assert!(names.contains(&"pkg".to_string()));
    /// # Ok(()) }
    /// ```
    pub fn packages(&self) -> impl Iterator<Item = Symbol<'_>> {
        // SAFETY: the compilation is valid.
        let count = unsafe { sys::slang_compilation_package_count(self.raw()) };
        (0..count).filter_map(move |i| {
            // SAFETY: the compilation is valid; index is in range.
            let ast = unsafe { sys::slang_compilation_package(self.raw(), i) };
            wrap(ast)
        })
    }

    /// All diagnostics (parse and semantic), owned.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let diags = design.diagnostics();
    /// assert!(!diags.has_errors());
    /// # Ok(()) }
    /// ```
    pub fn diagnostics(&self) -> Diagnostics {
        let mut err = ffi::error();
        // SAFETY: the compilation is valid; the handle is consumed by collect.
        let diags = unsafe { sys::slang_compilation_diagnostics(self.raw(), &mut err) };
        if diags.is_null() {
            return Diagnostics::default();
        }
        crate::collect_diagnostics(self.inner.session.raw(), diags)
    }

    /// Looks up a (possibly dotted) hierarchical name from the design root
    /// using full SystemVerilog lookup rules.
    ///
    /// This records reference-tracking state in the arena, so it takes
    /// `&mut self` and is unavailable while the design is shared. For pure,
    /// shareable navigation use [`Symbol::find`] and the scope iterators.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x; endmodule\n")?;
    /// let mut design = comp.compile()?;
    /// let x = design.lookup("m.x").unwrap();
    /// assert_eq!(x.name(), "x");
    /// # Ok(()) }
    /// ```
    pub fn lookup(&mut self, path: &str) -> Option<Symbol<'_>> {
        let root = self.root().raw;
        let mut err = ffi::error();
        let (p, pl) = ffi::as_ptr_len(path);
        // SAFETY: root is a scope of this compilation; out-error checked.
        let ast = unsafe { sys::slang_scope_lookup(root, p, pl, &mut err) };
        wrap(ast)
    }

    /// Opens an evaluation session: an exclusive borrow of the design through
    /// which not-yet-folded constants may be evaluated. Because evaluating may
    /// allocate into the arena, it requires exclusive (`&mut`) access; the
    /// session holds that borrow so that expression handles obtained through
    /// it can be evaluated without a second mutable borrow.
    ///
    /// ```
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session).unwrap();
    /// # comp.add_source("module m; localparam int X = 3 * 4; endmodule\n").unwrap();
    /// # let mut design = comp.compile().unwrap();
    /// let mut eval = design.eval_session();
    /// let x = eval.design().top_instances().next().unwrap()
    ///     .instance_body().unwrap().find("X").unwrap();
    /// assert_eq!(eval.eval(x.initializer().unwrap()).as_deref(), Some("12"));
    /// ```
    pub fn eval_session(&mut self) -> EvalSession<'_> {
        EvalSession {
            design: self,
            _not_sync: PhantomData,
        }
    }
}

/// An exclusive evaluation session over a [`Design`] (see
/// [`Design::eval_session`]). `Send` (the exclusive borrow may move between
/// threads) but **not** `Sync`: [`eval`](Self::eval) mutates through `&self`,
/// so a shared `&EvalSession` must never be used from two threads at once.
pub struct EvalSession<'d> {
    design: &'d mut Design,
    // Cell<()> is Send but not Sync: it makes EvalSession non-Sync so the
    // interior mutation in `eval(&self)` cannot be shared across threads.
    _not_sync: PhantomData<core::cell::Cell<()>>,
}

impl<'d> EvalSession<'d> {
    /// Read access to the design. Handles obtained through this borrow can be
    /// passed straight to [`eval`](Self::eval).
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module top; endmodule\n")?;
    /// # let mut design = comp.compile()?;
    /// let eval = design.eval_session();
    /// assert_eq!(eval.design().top_instances().next().unwrap().name(), "top");
    /// # Ok(()) }
    /// ```
    pub fn design(&self) -> &Design {
        self.design
    }

    /// Evaluates an expression as a constant, returning its value printed as
    /// SystemVerilog, or `None` if it is not constant. May cache the result
    /// into the design's arena.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; localparam int X = 3 * 4; endmodule\n")?;
    /// # let mut design = comp.compile()?;
    /// let eval = design.eval_session();
    /// let x = eval.design().top_instances().next().unwrap()
    ///     .instance_body().unwrap().find("X").unwrap();
    /// assert_eq!(eval.eval(x.initializer().unwrap()).as_deref(), Some("12"));
    /// # Ok(()) }
    /// ```
    pub fn eval(&self, expr: Expression<'_>) -> Option<String> {
        // A lifetime brand alone cannot prove "same design" (two designs can
        // share a scope), so verify the compilation identity at runtime: the C
        // side unfreezes and allocates into the EXPRESSION's own compilation, so
        // a foreign handle would mutate a design we do not hold exclusively.
        assert!(
            expr.raw.compilation == self.design.raw_compilation(),
            "eval: expression belongs to a different Design than this EvalSession"
        );
        let mut err = ffi::error();
        let mut out = empty_str();
        // SAFETY: the assertion above proves `expr` belongs to the design this
        // session holds exclusively (`&mut Design`, and the session is !Sync),
        // so the allocating eval cannot race; out-error checked.
        let ok = unsafe { sys::slang_expression_eval(expr.raw, &mut out, &mut err) };
        (ok && ffi::check(&err).is_ok()).then(|| ffi::owned_str(out))
    }

    /// Evaluates an expression to a structured [`ConstantValue`](crate::ConstantValue) (see
    /// [`Expression::constant_value`]), or `None` if it is not constant. Like
    /// [`eval`](Self::eval), this may allocate into the design's arena.
    pub fn eval_constant(&self, expr: Expression<'_>) -> Option<crate::ConstantValue> {
        assert!(
            expr.raw.compilation == self.design.raw_compilation(),
            "eval_constant: expression belongs to a different Design than this EvalSession"
        );
        let mut err = ffi::error();
        // SAFETY: as `eval` — the assertion proves exclusive access; out-error checked.
        let raw = unsafe { sys::slang_expression_eval_constant(expr.raw, &mut err) };
        if ffi::check(&err).is_err() {
            if !raw.is_null() {
                // SAFETY: owned handle, free on error.
                unsafe { sys::slang_constant_destroy(raw) };
            }
            return None;
        }
        // SAFETY: `raw` is a valid owned handle (or null); consumed.
        unsafe { crate::ConstantValue::from_raw(raw) }
    }
}

fn empty_str() -> sys::slang_str {
    sys::slang_str {
        data: core::ptr::null(),
        len: 0,
        owner: core::ptr::null_mut(),
    }
}

/// A symbol handle borrowed from a [`Design`]. `Copy` and pointer-sized.
#[derive(Clone, Copy)]
pub struct Symbol<'d> {
    raw: sys::slang_ast,
    _design: PhantomData<&'d Design>,
}

/// A type handle (a symbol that is a type).
#[derive(Clone, Copy)]
pub struct Type<'d> {
    raw: sys::slang_ast,
    _design: PhantomData<&'d Design>,
}

/// An expression handle.
#[derive(Clone, Copy)]
pub struct Expression<'d> {
    raw: sys::slang_ast,
    _design: PhantomData<&'d Design>,
}

/// A borrowed read cursor into a [`Design`] — one of the pointer-sized handle
/// types. `from_raw` is the single place the `{ raw, _design }` shape is built;
/// prefer [`wrap`] (which null-checks) at call sites.
trait Handle<'d>: Copy {
    fn from_raw(raw: sys::slang_ast) -> Self;
}

// Handles are read cursors into the frozen arena of a `Design`, which is itself
// `Send + Sync`, and the `'d` brand keeps a handle from outliving that design.
// Reading through a handle from any thread is a pure read of the frozen arena,
// so a handle is safe to send and share exactly as `&'d Design` is — this is
// what lets `Design::par_visit` hand symbols to a rayon worker pool.
macro_rules! impl_handle {
    ($($t:ident),+ $(,)?) => { $(
        // SAFETY: equivalent to `&'d Design`, which is Send because Design is Sync.
        unsafe impl Send for $t<'_> {}
        // SAFETY: as above — reads never mutate the frozen arena.
        unsafe impl Sync for $t<'_> {}
        impl<'d> Handle<'d> for $t<'d> {
            fn from_raw(raw: sys::slang_ast) -> Self {
                $t {
                    raw,
                    _design: PhantomData,
                }
            }
        }
    )+ };
}
impl_handle!(Symbol, Type, Expression, Statement, SemNode);

/// Wraps a raw ast as a borrowed handle, or `None` if it is null. The single
/// place the null-check-and-wrap pattern lives.
fn wrap<'d, T: Handle<'d>>(raw: sys::slang_ast) -> Option<T> {
    (!raw.ptr.is_null()).then(|| T::from_raw(raw))
}

/// A stable, owned identity for a symbol: its hierarchical path. Unlike a
/// [`Symbol`] handle — a cursor into one [`Design`] that cannot outlive it — a
/// `SymbolId` is a plain owned key (`Hash + Eq + Clone`) that survives the
/// design being dropped and rebuilt, and re-resolves against a design with
/// [`Design::resolve`].
///
/// It is stable across a rebuild of the *same* source, which is what makes it
/// useful to an editor server that recompiles between edits and wants to
/// remember "the symbol the user selected". A symbol with no resolvable
/// hierarchical path (some anonymous or synthesized symbols) cannot be
/// recovered — [`resolve`](Design::resolve) then returns `None`.
///
/// # Examples
/// ```
/// # fn main() -> Result<(), sv_lang::Error> {
/// # let session = sv_lang::Session::new();
/// # let mut comp = sv_lang::Compilation::new(&session)?;
/// # comp.add_source("module m; logic [7:0] x; endmodule\n")?;
/// # let mut design = comp.compile()?;
/// let id = design.lookup("m.x").unwrap().id();
/// // The id is a plain owned value; resolve it back to a live handle.
/// assert_eq!(design.resolve(&id).unwrap().name(), "x");
/// # Ok(()) }
/// ```
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SymbolId(String);

impl SymbolId {
    /// The hierarchical path this id is built from.
    pub fn path(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for SymbolId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Symbol<'_> {
    /// This symbol's stable [`SymbolId`] — its hierarchical path — for use as a
    /// map key or to remember it across a design rebuild (see [`SymbolId`]).
    pub fn id(&self) -> SymbolId {
        SymbolId(self.hierarchical_path())
    }
}

/// Bundles a borrowed handle with an owning reference to its [`Design`], so the
/// handle can be stored past the borrow that produced it. Implemented for
/// [`Symbol`], [`Type`], [`Expression`] and [`Statement`]; the owned
/// counterparts are [`OwnedSymbol`], [`OwnedType`], [`OwnedExpression`] and
/// [`OwnedStatement`].
pub trait IntoOwned {
    /// The owned counterpart.
    type Owned;
    /// Bundles `self` with `design`, which must be the design it came from.
    ///
    /// # Panics
    /// Panics if the handle does not belong to `design`.
    fn into_owned(self, design: &Design) -> Self::Owned;
}

macro_rules! owned_handle {
    ($owned:ident, $handle:ident, $what:literal) => {
        #[doc = concat!(
                    "An owned ", $what, " handle: a [`", stringify!($handle),
                    "`] kept alive together with its [`Design`], so it can be stored and ",
                    "re-borrowed later (e.g. cached by an editor server within one session). ",
                    "Create one with [`IntoOwned::into_owned`]; recover the borrowed handle ",
                    "with [`get`](Self::get). `Send + Sync`."
                )]
        pub struct $owned {
            // Held solely to keep the design's arena alive for `raw`; never read
            // (its `Drop` is the point), so the dead-code lint would fire.
            #[allow(dead_code)]
            inner: Arc<DesignInner>,
            raw: sys::slang_ast,
        }

        impl $owned {
            #[doc = concat!("Re-borrows the ", $what, " against the design kept alive here.")]
            pub fn get(&self) -> $handle<'_> {
                <$handle as Handle>::from_raw(self.raw)
            }
        }

        // SAFETY: `inner` is a `Send + Sync` frozen design and `raw` is a read
        // cursor into its arena, valid for as long as `inner` is held; sharing
        // or sending the pair is exactly as safe as sharing the `Design`.
        unsafe impl Send for $owned {}
        // SAFETY: as above — reads never mutate the frozen arena.
        unsafe impl Sync for $owned {}

        impl core::fmt::Debug for $owned {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($owned)).finish_non_exhaustive()
            }
        }

        impl<'d> IntoOwned for $handle<'d> {
            type Owned = $owned;
            fn into_owned(self, design: &Design) -> $owned {
                assert!(
                    self.raw.compilation == design.raw_compilation(),
                    concat!(
                        stringify!($handle),
                        "::into_owned: handle belongs to a different Design"
                    )
                );
                $owned {
                    inner: Arc::clone(&design.inner),
                    raw: self.raw,
                }
            }
        }
    };
}

owned_handle!(OwnedSymbol, Symbol, "symbol");
owned_handle!(OwnedType, Type, "type");
owned_handle!(OwnedExpression, Expression, "expression");
owned_handle!(OwnedStatement, Statement, "statement");

impl Design {
    /// Re-resolves a [`SymbolId`] to a live [`Symbol`] in this design, or `None`
    /// if nothing at that path exists. Like [`lookup`](Design::lookup) it parses
    /// the path (which allocates), hence `&mut`.
    pub fn resolve(&mut self, id: &SymbolId) -> Option<Symbol<'_>> {
        self.lookup(id.path())
    }
}

/// The kind of definition a `Definition` symbol denotes.
///
/// # Examples
/// ```
/// # fn main() -> Result<(), sv_lang::Error> {
/// use sv_lang::DefinitionKind;
/// # let session = sv_lang::Session::new();
/// # let mut comp = sv_lang::Compilation::new(&session)?;
/// # comp.add_source("interface bus; endinterface\n")?;
/// # let design = comp.compile()?;
/// let def = design.definitions().next().unwrap();
/// assert_eq!(def.definition_kind(), Some(DefinitionKind::Interface));
/// # Ok(()) }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefinitionKind {
    /// A `module`.
    Module,
    /// An `interface`.
    Interface,
    /// A `program`.
    Program,
}

impl<'d> Symbol<'d> {
    pub(crate) fn raw(&self) -> sys::slang_ast {
        self.raw
    }

    fn same_design(self, other: sys::slang_ast) -> bool {
        self.raw.compilation == other.compilation
    }

    /// The symbol's semantic kind.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::kinds::SymbolKind;
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let top = design.top_instances().next().unwrap();
    /// assert_eq!(top.kind(), SymbolKind::Instance);
    /// # Ok(()) }
    /// ```
    pub fn kind(&self) -> SymbolKind {
        SymbolKind::from_raw(self.raw.kind as u16).unwrap_or(SymbolKind::Unknown)
    }

    /// The symbol's name (empty for anonymous symbols). Borrowed from the design.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module counter; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// assert_eq!(design.top_instances().next().unwrap().name(), "counter");
    /// # Ok(()) }
    /// ```
    pub fn name(&self) -> &'d str {
        // SAFETY: bytes are borrowed from the frozen design for 'd.
        unsafe { ffi::str_ref(sys::slang_symbol_name(self.raw)) }
    }

    /// The enclosing scope's symbol, or `None` for the root.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::kinds::SymbolKind;
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let x = body.find("x").unwrap();
    /// assert_eq!(x.parent().unwrap().kind(), SymbolKind::InstanceBody);
    /// # Ok(()) }
    /// ```
    pub fn parent(&self) -> Option<Symbol<'d>> {
        // SAFETY: the symbol is valid.
        let p = unsafe { sys::slang_symbol_parent_scope(self.raw) };
        wrap(p)
    }

    /// The next symbol in the same scope, or `None` at the end.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x, y; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let x = body.find("x").unwrap();
    /// assert_eq!(x.next_sibling().unwrap().name(), "y");
    /// # Ok(()) }
    /// ```
    pub fn next_sibling(&self) -> Option<Symbol<'d>> {
        // SAFETY: the symbol is valid.
        let s = unsafe { sys::slang_symbol_next_sibling(self.raw) };
        wrap(s)
    }

    /// True if the symbol is a scope (has members).
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// assert!(body.is_scope());
    /// assert!(!body.find("x").unwrap().is_scope());
    /// # Ok(()) }
    /// ```
    pub fn is_scope(&self) -> bool {
        // SAFETY: the symbol is valid.
        unsafe { sys::slang_symbol_is_scope(self.raw) }
    }

    /// True if the symbol is a type.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; typedef logic [3:0] nib_t; nib_t n; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// assert!(body.find("nib_t").unwrap().is_type());
    /// assert!(!body.find("n").unwrap().is_type());
    /// # Ok(()) }
    /// ```
    pub fn is_type(&self) -> bool {
        // SAFETY: the symbol is valid.
        unsafe { sys::slang_symbol_is_type(self.raw) }
    }

    /// True if the symbol has a value (variable, net, parameter, port, ...).
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// assert!(body.find("x").unwrap().is_value());
    /// # Ok(()) }
    /// ```
    pub fn is_value(&self) -> bool {
        // SAFETY: the symbol is valid.
        unsafe { sys::slang_symbol_is_value(self.raw) }
    }

    /// The symbol's full hierarchical path, e.g. `"top.cpu.alu"`. Owned.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// assert_eq!(body.find("x").unwrap().hierarchical_path(), "m.x");
    /// # Ok(()) }
    /// ```
    pub fn hierarchical_path(&self) -> String {
        let mut err = ffi::error();
        // SAFETY: the symbol is valid; the string is owned.
        unsafe { ffi::owned_str(sys::slang_symbol_hierarchical_path(self.raw, &mut err)) }
    }

    /// The syntax node this symbol was created from, tied to `tree`.
    ///
    /// The node belongs to one of the design's trees; pass a borrow of that
    /// tree to bind the returned node's lifetime. Returns `None` if the
    /// symbol has no syntax or does not belong to `tree`.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::kinds::SyntaxKind;
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; logic [7:0] x; endmodule\n")?;
    /// let mut comp = sv_lang::Compilation::new(&session)?;
    /// comp.add(&tree)?;
    /// let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let node = body.find("x").unwrap().syntax(&tree).unwrap();
    /// assert_eq!(node.kind(), SyntaxKind::Declarator);
    /// # Ok(()) }
    /// ```
    pub fn syntax<'t>(&self, tree: &'t SyntaxTree) -> Option<Node<'t>> {
        // SAFETY: the symbol is valid.
        let node = unsafe { sys::slang_ast_syntax(self.raw) };
        if node.ptr.is_null() || node.tree != tree.raw() {
            return None;
        }
        Some(Node::from_raw_node(node))
    }

    /// Iterates the members of this scope, forcing its elaboration if needed
    /// (a no-op on a frozen design, hence a pure read). Empty if not a scope.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x; logic [7:0] y; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let names: Vec<_> = body.members().map(|m| m.name().to_string()).collect();
    /// assert!(names.contains(&"x".to_string()));
    /// assert!(names.contains(&"y".to_string()));
    /// # Ok(()) }
    /// ```
    pub fn members(&self) -> impl Iterator<Item = Symbol<'d>> {
        let mut err = ffi::error();
        // SAFETY: the symbol is valid; on a frozen design first_member does
        // not allocate.
        let first = unsafe { sys::slang_scope_first_member(self.raw, &mut err) };
        let mut next = wrap::<Symbol>(first);
        core::iter::from_fn(move || {
            let cur = next?;
            next = cur.next_sibling();
            Some(cur)
        })
    }

    /// Visits this symbol and every symbol in its scopes, transitively, in
    /// declaration order. The callback returns a [`Walk`](crate::Walk) to steer
    /// the traversal (descend / skip this subtree / stop). Returns `true` if
    /// the walk completed, `false` if a callback broke out.
    ///
    /// This is a pure read (safe on a shared design): it only follows the
    /// already-elaborated scope structure.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::{Walk, kinds::SymbolKind};
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x, y; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let mut vars = 0;
    /// design.root().visit(|sym| {
    ///     if sym.kind() == SymbolKind::Variable { vars += 1; }
    ///     Walk::Continue
    /// });
    /// assert_eq!(vars, 2);
    /// # Ok(()) }
    /// ```
    pub fn visit(&self, mut f: impl FnMut(Symbol<'d>) -> crate::Walk) -> bool {
        let mut stack = vec![*self];
        while let Some(sym) = stack.pop() {
            match f(sym) {
                crate::Walk::Break => return false,
                crate::Walk::Skip => continue,
                crate::Walk::Continue => {
                    let members: Vec<_> = sym.members().collect();
                    stack.extend(members.into_iter().rev());
                }
            }
        }
        true
    }

    /// The root [`Statement`] of a procedural block (`always`/`initial`/`final`)
    /// or a subroutine (`function`/`task`), or `None` for any other symbol —
    /// the entry point into the elaborated behavioral tree. A pure read.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::kinds::{SymbolKind, StatementKind};
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m(input logic clk, rst, input logic [7:0] d, output logic [7:0] q);\n\
    /// #     always_ff @(posedge clk) if (rst) q <= 0; else q <= d + 1; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let block = body.members().find(|s| s.kind() == SymbolKind::ProceduralBlock).unwrap();
    /// assert_eq!(block.body().unwrap().kind(), StatementKind::Timed);
    /// # Ok(()) }
    /// ```
    pub fn body(&self) -> Option<Statement<'d>> {
        // SAFETY: the symbol is valid; the body was forced by the freeze sweep.
        let ast = unsafe { sys::slang_symbol_body(self.raw) };
        wrap(ast)
    }

    /// Finds a direct member by name (no imports, no upward search). A pure
    /// read, safe on a shared design.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// assert!(body.find("x").is_some());
    /// assert!(body.find("nope").is_none());
    /// # Ok(()) }
    /// ```
    pub fn find(&self, name: &str) -> Option<Symbol<'d>> {
        let mut err = ffi::error();
        let (n, nl) = ffi::as_ptr_len(name);
        // SAFETY: the symbol is valid; find does not allocate.
        let ast = unsafe { sys::slang_scope_find(self.raw, n, nl, &mut err) };
        wrap(ast)
    }

    /// Views this symbol as a [`Type`], if it is one.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; typedef logic [3:0] nib_t; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let nib = body.find("nib_t").unwrap().as_type().unwrap();
    /// assert_eq!(nib.canonical().to_sv_string(), "logic[3:0]");
    /// # Ok(()) }
    /// ```
    pub fn as_type(&self) -> Option<Type<'d>> {
        self.is_type().then(|| Type::from_raw(self.raw))
    }

    /// The declared type of a value symbol.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let t = body.find("x").unwrap().value_type().unwrap();
    /// assert_eq!(t.bit_width(), 8);
    /// # Ok(()) }
    /// ```
    pub fn value_type(&self) -> Option<Type<'d>> {
        if !self.is_value() {
            return None;
        }
        let mut err = ffi::error();
        // SAFETY: the symbol is a value; type is memoized on a frozen design.
        let ast = unsafe { sys::slang_value_type(self.raw, &mut err) };
        wrap(ast)
    }

    /// The initializer expression of a value symbol, if any.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; localparam int X = 3 * 4; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let init = body.find("X").unwrap().initializer().unwrap();
    /// assert_eq!(init.constant().as_deref(), Some("12"));
    /// # Ok(()) }
    /// ```
    pub fn initializer(&self) -> Option<Expression<'d>> {
        if !self.is_value() {
            return None;
        }
        let mut err = ffi::error();
        // SAFETY: the symbol is a value; the initializer is memoized.
        let ast = unsafe { sys::slang_value_initializer(self.raw, &mut err) };
        wrap(ast)
    }

    /// For an `Instance` symbol, its body scope.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::kinds::SymbolKind;
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let top = design.top_instances().next().unwrap();
    /// let body = top.instance_body().unwrap();
    /// assert_eq!(body.kind(), SymbolKind::InstanceBody);
    /// # Ok(()) }
    /// ```
    pub fn instance_body(&self) -> Option<Symbol<'d>> {
        if self.kind() != SymbolKind::Instance {
            return None;
        }
        // SAFETY: the symbol is an instance.
        let ast = unsafe { sys::slang_instance_body(self.raw) };
        wrap(ast)
    }

    /// For an `Instance` symbol, the definition it instantiates.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let top = design.top_instances().next().unwrap();
    /// assert_eq!(top.instance_definition().unwrap().name(), "m");
    /// # Ok(()) }
    /// ```
    pub fn instance_definition(&self) -> Option<Symbol<'d>> {
        if self.kind() != SymbolKind::Instance {
            return None;
        }
        // SAFETY: the symbol is an instance.
        let ast = unsafe { sys::slang_instance_definition(self.raw) };
        wrap(ast)
    }

    /// For an `Instance` symbol, its elaborated parameters (value and type).
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m #(parameter int W = 8); endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let top = design.top_instances().next().unwrap();
    /// let names: Vec<_> = top.parameters().map(|p| p.name().to_string()).collect();
    /// assert!(names.contains(&"W".to_string()));
    /// # Ok(()) }
    /// ```
    pub fn parameters(&self) -> impl Iterator<Item = Symbol<'d>> {
        // SAFETY: the symbol is valid; count is 0 for non-instances.
        let count = unsafe { sys::slang_instance_parameter_count(self.raw) };
        let raw = self.raw;
        (0..count).filter_map(move |i| {
            // SAFETY: the symbol is valid; index is in range.
            let ast = unsafe { sys::slang_instance_parameter(raw, i) };
            wrap(ast)
        })
    }

    /// For a `Parameter`/`TypeParameter` symbol, its value or target type
    /// printed as SystemVerilog. Owned.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; localparam int X = 3 * 4; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// assert_eq!(body.find("X").unwrap().parameter_value().as_deref(), Some("12"));
    /// # Ok(()) }
    /// ```
    pub fn parameter_value(&self) -> Option<String> {
        let mut err = ffi::error();
        // SAFETY: the symbol is valid; the string is owned.
        let s = unsafe { sys::slang_parameter_value(self.raw, &mut err) };
        ffi::check(&err).ok().map(|()| ffi::owned_str(s))
    }

    /// For a `Definition` symbol, whether it is a module, interface or program.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::DefinitionKind;
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let def = design.definitions().next().unwrap();
    /// assert_eq!(def.definition_kind(), Some(DefinitionKind::Module));
    /// # Ok(()) }
    /// ```
    pub fn definition_kind(&self) -> Option<DefinitionKind> {
        if self.kind() != SymbolKind::Definition {
            return None;
        }
        // SAFETY: the symbol is a definition.
        Some(match unsafe { sys::slang_definition_kind_of(self.raw) } {
            sys::SLANG_DEFINITION_INTERFACE => DefinitionKind::Interface,
            sys::SLANG_DEFINITION_PROGRAM => DefinitionKind::Program,
            _ => DefinitionKind::Module,
        })
    }

    /// For an `EnumValue` symbol (from [`Type::enum_members`]), its constant
    /// value printed as SystemVerilog. `None` for any other symbol.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; typedef enum logic [1:0] { A, B, C } e_t; e_t e; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let t = body.find("e").unwrap().value_type().unwrap();
    /// let vals: Vec<_> = t.enum_members().filter_map(|m| m.enum_member_value()).collect();
    /// assert_eq!(vals, ["2'b0", "2'b1", "2'b10"]);
    /// # Ok(()) }
    /// ```
    pub fn enum_member_value(&self) -> Option<String> {
        if self.kind() != SymbolKind::EnumValue {
            return None;
        }
        let mut err = ffi::error();
        // SAFETY: the symbol is an enum value; the value memo is forced pre-seal
        // by the freeze sweep, so this reads it without arena mutation. Owned.
        let s = unsafe { sys::slang_enum_member_value(self.raw, &mut err) };
        ffi::check(&err).ok().map(|()| ffi::owned_str(s))
    }

    /// For a `Field` symbol (from [`Type::fields`]), its bit offset within the
    /// parent struct/union. 0 for a non-field symbol.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m;\n\
    /// #     typedef struct packed { logic [3:0] a; logic [3:0] b; } pair_t; pair_t p; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let t = body.find("p").unwrap().value_type().unwrap();
    /// let fields: Vec<_> = t.fields().collect();
    /// // packed struct: `a` is the high field (offset 4), `b` the low field (offset 0).
    /// assert_eq!(fields[0].field_bit_offset(), 4);
    /// assert_eq!(fields[1].field_bit_offset(), 0);
    /// # Ok(()) }
    /// ```
    pub fn field_bit_offset(&self) -> u64 {
        // SAFETY: the symbol is valid; 0 for a non-field.
        unsafe { sys::slang_field_bit_offset(self.raw) }
    }

    /// For a `Field` symbol (from [`Type::fields`]), its index in declaration
    /// order. 0 for a non-field symbol.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m;\n\
    /// #     typedef struct packed { logic [3:0] a; logic [3:0] b; } pair_t; pair_t p; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let t = body.find("p").unwrap().value_type().unwrap();
    /// let fields: Vec<_> = t.fields().collect();
    /// assert_eq!(fields[0].field_index(), 0);
    /// assert_eq!(fields[1].field_index(), 1);
    /// # Ok(()) }
    /// ```
    pub fn field_index(&self) -> u32 {
        // SAFETY: the symbol is valid; 0 for a non-field.
        unsafe { sys::slang_field_index(self.raw) }
    }
}

impl<'d> Type<'d> {
    /// The type as a symbol.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; typedef logic [3:0] nib_t; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let t = body.find("nib_t").unwrap().as_type().unwrap();
    /// assert_eq!(t.as_symbol().name(), "nib_t");
    /// # Ok(()) }
    /// ```
    pub fn as_symbol(&self) -> Symbol<'d> {
        Symbol::from_raw(self.raw)
    }

    /// The canonical type: typedefs and type parameters resolved.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; typedef logic [3:0] nib_t; nib_t n; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let t = body.find("n").unwrap().value_type().unwrap();
    /// assert_eq!(t.to_sv_string(), "m.nib_t");
    /// assert_eq!(t.canonical().to_sv_string(), "logic[3:0]");
    /// # Ok(()) }
    /// ```
    pub fn canonical(&self) -> Type<'d> {
        // SAFETY: the type is valid.
        let ast = unsafe { sys::slang_type_canonical(self.raw) };
        Type::from_raw(ast)
    }

    /// The type printed in SystemVerilog syntax, e.g. `"logic[7:0]"`. Owned.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let t = body.find("x").unwrap().value_type().unwrap();
    /// assert_eq!(t.to_sv_string(), "logic[7:0]");
    /// # Ok(()) }
    /// ```
    pub fn to_sv_string(&self) -> String {
        let mut err = ffi::error();
        // SAFETY: the type is valid; the string is owned.
        unsafe { ffi::owned_str(sys::slang_type_to_string(self.raw, &mut err)) }
    }

    /// Width in bits of an integral type; 0 for non-integral types.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// assert_eq!(body.find("x").unwrap().value_type().unwrap().bit_width(), 8);
    /// # Ok(()) }
    /// ```
    pub fn bit_width(&self) -> u64 {
        // SAFETY: the type is valid.
        unsafe { sys::slang_type_bit_width(self.raw) }
    }

    /// True for a packed/integral type.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x; real r; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// assert!(body.find("x").unwrap().value_type().unwrap().is_integral());
    /// assert!(!body.find("r").unwrap().value_type().unwrap().is_integral());
    /// # Ok(()) }
    /// ```
    pub fn is_integral(&self) -> bool {
        // SAFETY: the type is valid.
        unsafe { sys::slang_type_is_integral(self.raw) }
    }

    /// True for a signed type.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; int si; logic [7:0] u; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// assert!(body.find("si").unwrap().value_type().unwrap().is_signed());
    /// assert!(!body.find("u").unwrap().value_type().unwrap().is_signed());
    /// # Ok(()) }
    /// ```
    pub fn is_signed(&self) -> bool {
        // SAFETY: the type is valid.
        unsafe { sys::slang_type_is_signed(self.raw) }
    }

    /// True for a four-state type (`logic`/`reg`), false for two-state (`bit`).
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x; bit [7:0] b; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// assert!(body.find("x").unwrap().value_type().unwrap().is_four_state());
    /// assert!(!body.find("b").unwrap().value_type().unwrap().is_four_state());
    /// # Ok(()) }
    /// ```
    pub fn is_four_state(&self) -> bool {
        // SAFETY: the type is valid.
        unsafe { sys::slang_type_is_four_state(self.raw) }
    }

    /// True for an unpacked array type.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] mem [0:3]; logic [7:0] x; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// assert!(body.find("mem").unwrap().value_type().unwrap().is_unpacked_array());
    /// assert!(!body.find("x").unwrap().value_type().unwrap().is_unpacked_array());
    /// # Ok(()) }
    /// ```
    pub fn is_unpacked_array(&self) -> bool {
        // SAFETY: the type is valid.
        unsafe { sys::slang_type_is_unpacked_array(self.raw) }
    }

    /// True for a class type.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// assert!(!body.find("x").unwrap().value_type().unwrap().is_class());
    /// # Ok(()) }
    /// ```
    pub fn is_class(&self) -> bool {
        // SAFETY: the type is valid.
        unsafe { sys::slang_type_is_class(self.raw) }
    }

    /// Asserts `other` comes from the same compilation as `self`. A shared `'d`
    /// brand is not proof (two designs can share a scope), so compare the
    /// compilation identity the handles carry; a cross-design comparison is a
    /// usage error that yields a meaningless answer.
    fn assert_same_compilation(&self, other: &Type<'d>) {
        assert!(
            self.raw.compilation == other.raw.compilation,
            "Type comparison across two different Designs"
        );
    }

    /// IEEE 1800 §6.22.1 type equivalence.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x, y; typedef logic [3:0] nib_t; nib_t n; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let xt = body.find("x").unwrap().value_type().unwrap();
    /// let yt = body.find("y").unwrap().value_type().unwrap();
    /// let nt = body.find("n").unwrap().value_type().unwrap();
    /// assert!(xt.is_equivalent(&yt));
    /// assert!(!xt.is_equivalent(&nt));
    /// # Ok(()) }
    /// ```
    pub fn is_equivalent(&self, other: &Type<'d>) -> bool {
        self.assert_same_compilation(other);
        // SAFETY: both are valid and (asserted) from the same compilation.
        unsafe { sys::slang_type_is_equivalent(self.raw, other.raw) }
    }

    /// IEEE 1800 §6.22.2 type matching.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x, y; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let xt = body.find("x").unwrap().value_type().unwrap();
    /// let yt = body.find("y").unwrap().value_type().unwrap();
    /// assert!(xt.is_matching(&yt));
    /// # Ok(()) }
    /// ```
    pub fn is_matching(&self, other: &Type<'d>) -> bool {
        self.assert_same_compilation(other);
        // SAFETY: as `is_equivalent`.
        unsafe { sys::slang_type_is_matching(self.raw, other.raw) }
    }

    /// IEEE 1800 §6.22.3 assignment compatibility (`other` assignable to `self`).
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x; logic [3:0] narrow; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let wide = body.find("x").unwrap().value_type().unwrap();
    /// let narrow = body.find("narrow").unwrap().value_type().unwrap();
    /// // a 4-bit value is assignable to an 8-bit target.
    /// assert!(wide.is_assignment_compatible(&narrow));
    /// # Ok(()) }
    /// ```
    pub fn is_assignment_compatible(&self, other: &Type<'d>) -> bool {
        self.assert_same_compilation(other);
        // SAFETY: as `is_equivalent`.
        unsafe { sys::slang_type_is_assignment_compatible(self.raw, other.raw) }
    }

    /// True if the (canonical) type is an enum.
    pub fn is_enum(&self) -> bool {
        // SAFETY: the type is valid; canonicalized internally.
        unsafe { sys::slang_type_is_enum(self.raw) }
    }

    /// True if the (canonical) type is a packed or unpacked struct.
    pub fn is_struct(&self) -> bool {
        // SAFETY: the type is valid; canonicalized internally.
        unsafe { sys::slang_type_is_struct(self.raw) }
    }

    /// True if the (canonical) type is a packed or unpacked union.
    pub fn is_union(&self) -> bool {
        // SAFETY: the type is valid; canonicalized internally.
        unsafe { sys::slang_type_is_union(self.raw) }
    }

    /// True if the (canonical) type is any packed or unpacked array.
    pub fn is_array(&self) -> bool {
        // SAFETY: the type is valid; canonicalized internally.
        unsafe { sys::slang_type_is_array(self.raw) }
    }

    /// True if the (canonical) type is the `string` type.
    pub fn is_string(&self) -> bool {
        // SAFETY: the type is valid; canonicalized internally.
        unsafe { sys::slang_type_is_string(self.raw) }
    }

    /// The element type of any array type (packed or unpacked), or `None` for a
    /// non-array type.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] mem [4]; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let t = body.find("mem").unwrap().value_type().unwrap();
    /// assert!(t.is_array());
    /// assert_eq!(t.element_type().unwrap().bit_width(), 8);
    /// # Ok(()) }
    /// ```
    pub fn element_type(&self) -> Option<Type<'d>> {
        // SAFETY: the type is valid; a null node for a non-array.
        wrap(unsafe { sys::slang_type_array_element(self.raw) })
    }

    /// The base type of an enum type, or `None` for a non-enum type.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; typedef enum logic [1:0] { A, B } e_t; e_t e; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let t = body.find("e").unwrap().value_type().unwrap();
    /// assert_eq!(t.enum_base().unwrap().bit_width(), 2);
    /// # Ok(()) }
    /// ```
    pub fn enum_base(&self) -> Option<Type<'d>> {
        // SAFETY: the type is valid; a null node for a non-enum.
        wrap(unsafe { sys::slang_type_enum_base(self.raw) })
    }

    /// The members of an enum type, each an `EnumValue` symbol (in declaration
    /// order). Empty for a non-enum type.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; typedef enum logic [1:0] { A, B, C } e_t; e_t e; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let t = body.find("e").unwrap().value_type().unwrap();
    /// let names: Vec<_> = t.enum_members().map(|m| m.name().to_string()).collect();
    /// assert_eq!(names, ["A", "B", "C"]);
    /// # Ok(()) }
    /// ```
    pub fn enum_members(&self) -> impl Iterator<Item = Symbol<'d>> {
        let raw = self.raw;
        // SAFETY: the type is valid; count is 0 for a non-enum.
        let count = unsafe { sys::slang_enum_member_count(raw) };
        (0..count).filter_map(move |i| {
            // SAFETY: the type is valid; index in range.
            let ast = unsafe { sys::slang_enum_member(raw, i) };
            wrap(ast)
        })
    }

    /// The fields of a struct or union type (packed or unpacked), each a
    /// `Field` symbol (in declaration order). Empty for any other type. Read a
    /// field's type with [`Symbol::value_type`], its position with
    /// [`Symbol::field_bit_offset`] / [`Symbol::field_index`].
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m;\n\
    /// #     typedef struct packed { logic [3:0] a; logic [3:0] b; } pair_t; pair_t p; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let t = body.find("p").unwrap().value_type().unwrap();
    /// let names: Vec<_> = t.fields().map(|f| f.name().to_string()).collect();
    /// assert_eq!(names, ["a", "b"]);
    /// # Ok(()) }
    /// ```
    pub fn fields(&self) -> impl Iterator<Item = Symbol<'d>> {
        let raw = self.raw;
        // SAFETY: the type is valid; count is 0 for a non-struct/union.
        let count = unsafe { sys::slang_type_field_count(raw) };
        (0..count).filter_map(move |i| {
            // SAFETY: the type is valid; index in range.
            let ast = unsafe { sys::slang_type_field(raw, i) };
            wrap(ast)
        })
    }

    /// The base class type of a class type (if it derives from one), or `None`.
    pub fn class_base(&self) -> Option<Type<'d>> {
        // SAFETY: the type is valid; a null node for a non-derived class or a
        // non-class type. The baseClass memo is populated by the freeze sweep.
        wrap(unsafe { sys::slang_type_class_base(self.raw) })
    }
}

impl<'d> Expression<'d> {
    /// The expression's semantic kind.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::kinds::ExpressionKind;
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x, y; wire [7:0] s = x & y; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let init = body.find("s").unwrap().initializer().unwrap();
    /// assert_eq!(init.kind(), ExpressionKind::BinaryOp);
    /// # Ok(()) }
    /// ```
    pub fn kind(&self) -> ExpressionKind {
        ExpressionKind::from_raw(self.raw.kind as u16).unwrap_or(ExpressionKind::Invalid)
    }

    /// The type of the expression.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x, y; wire [7:0] s = x & y; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let init = body.find("s").unwrap().initializer().unwrap();
    /// assert_eq!(init.expr_type().unwrap().bit_width(), 8);
    /// # Ok(()) }
    /// ```
    pub fn expr_type(&self) -> Option<Type<'d>> {
        // SAFETY: the expression is valid.
        let ast = unsafe { sys::slang_expression_type(self.raw) };
        wrap(ast)
    }

    /// True if the expression is invalid (had errors).
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x, y; wire [7:0] s = x & y; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// assert!(!body.find("s").unwrap().initializer().unwrap().is_bad());
    /// # Ok(()) }
    /// ```
    pub fn is_bad(&self) -> bool {
        // SAFETY: the expression is valid.
        unsafe { sys::slang_expression_is_bad(self.raw) }
    }

    /// The symbol this expression refers to (name references, member accesses),
    /// if any.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x, y; wire [7:0] s = x & y; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let init = body.find("s").unwrap().initializer().unwrap();
    /// let lhs = init.left().unwrap(); // the `x` reference
    /// assert_eq!(lhs.referenced_symbol().unwrap().name(), "x");
    /// # Ok(()) }
    /// ```
    pub fn referenced_symbol(&self) -> Option<Symbol<'d>> {
        // SAFETY: the expression is valid.
        let ast = unsafe { sys::slang_expression_symbol(self.raw) };
        wrap(ast)
    }

    /// The already-folded constant value, printed as SystemVerilog, if this
    /// expression has one cached (see [`FreezeReport`]). This never evaluates,
    /// so it is a pure read safe on a shared design. Owned.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; localparam int X = 3 * 4; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let init = body.find("X").unwrap().initializer().unwrap();
    /// assert_eq!(init.constant().as_deref(), Some("12"));
    /// # Ok(()) }
    /// ```
    pub fn constant(&self) -> Option<String> {
        let mut err = ffi::error();
        let mut out = empty_str();
        // SAFETY: the expression is valid; reading the cached value does not
        // allocate into the arena.
        let ok = unsafe { sys::slang_expression_cached_constant(self.raw, &mut out, &mut err) };
        (ok && ffi::check(&err).is_ok()).then(|| ffi::owned_str(out))
    }

    /// The already-folded constant as a structured [`ConstantValue`](crate::ConstantValue) (integer
    /// with width/signedness/x-z bits, real, string, array, ...), rather than a
    /// printed string. `None` if the expression has no cached constant. A pure
    /// read, safe on a shared design.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; localparam logic [7:0] X = 8'd12; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let v = body.find("X").unwrap().initializer().unwrap().constant_value().unwrap();
    /// assert_eq!(v.as_i64(), Some(12));
    /// assert_eq!(v.as_integer().unwrap().bit_width(), 8);
    /// # Ok(()) }
    /// ```
    pub fn constant_value(&self) -> Option<crate::ConstantValue> {
        let mut err = ffi::error();
        // SAFETY: the expression is valid; out-error checked.
        let raw = unsafe { sys::slang_expression_constant_value(self.raw, &mut err) };
        if ffi::check(&err).is_err() {
            if !raw.is_null() {
                // SAFETY: a non-null handle on error is still owned; free it.
                unsafe { sys::slang_constant_destroy(raw) };
            }
            return None;
        }
        // SAFETY: `raw` is a valid owned handle (or null); consumed.
        unsafe { crate::ConstantValue::from_raw(raw) }
    }

    /// The immediate semantic children of this expression, in slang's own
    /// order, as [`SemNode`]s (mostly sub-expressions; some kinds also carry a
    /// timing control or pattern). A pure read.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x, y; wire [7:0] s = x & y; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let init = body.find("s").unwrap().initializer().unwrap();
    /// assert_eq!(init.children().len(), 2);
    /// # Ok(()) }
    /// ```
    pub fn children(&self) -> Vec<SemNode<'d>> {
        sem_children(self.raw)
    }

    /// The immediate sub-expressions of this expression, in order (e.g. a
    /// `BinaryOp` yields `[left, right]`, a `Call` its arguments). Non-expression
    /// children are skipped; use [`children`](Self::children) for those.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x, y; wire [7:0] s = x & y; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let init = body.find("s").unwrap().initializer().unwrap();
    /// assert_eq!(init.operands().len(), 2);
    /// # Ok(()) }
    /// ```
    pub fn operands(&self) -> Vec<Expression<'d>> {
        sem_children(self.raw)
            .into_iter()
            .filter_map(|n| n.as_expression())
            .collect()
    }

    /// The left operand of a binary expression (`self.operands()[0]`).
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x, y; wire [7:0] s = x & y; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let init = body.find("s").unwrap().initializer().unwrap();
    /// assert_eq!(init.left().unwrap().referenced_symbol().unwrap().name(), "x");
    /// # Ok(()) }
    /// ```
    pub fn left(&self) -> Option<Expression<'d>> {
        self.operands().into_iter().next()
    }

    /// The right operand of a binary expression (`self.operands()[1]`).
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x, y; wire [7:0] s = x & y; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let init = body.find("s").unwrap().initializer().unwrap();
    /// assert_eq!(init.right().unwrap().referenced_symbol().unwrap().name(), "y");
    /// # Ok(()) }
    /// ```
    pub fn right(&self) -> Option<Expression<'d>> {
        self.operands().into_iter().nth(1)
    }

    /// The sole operand of a unary/conversion expression (`self.operands()[0]`).
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x; wire [7:0] inv = ~x; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let inv = body.find("inv").unwrap().initializer().unwrap(); // UnaryOp `~x`
    /// assert_eq!(inv.operand().unwrap().referenced_symbol().unwrap().name(), "x");
    /// # Ok(()) }
    /// ```
    pub fn operand(&self) -> Option<Expression<'d>> {
        self.operands().into_iter().next()
    }

    /// The operator of a `BinaryOp` expression, or `None` if this is not one.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::BinaryOp;
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x, y; wire [7:0] s = x & y; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let init = body.find("s").unwrap().initializer().unwrap();
    /// assert_eq!(init.binary_op(), Some(BinaryOp::BinaryAnd));
    /// # Ok(()) }
    /// ```
    pub fn binary_op(&self) -> Option<BinaryOp> {
        (self.kind() == ExpressionKind::BinaryOp)
            // SAFETY: the expression is valid.
            .then(|| BinaryOp::from_raw(unsafe { sys::slang_expr_binary_op(self.raw) }))
            .flatten()
    }

    /// The operator of a `UnaryOp` expression, or `None` if this is not one.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::UnaryOp;
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x; wire [7:0] inv = ~x; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let inv = body.find("inv").unwrap().initializer().unwrap();
    /// assert_eq!(inv.unary_op(), Some(UnaryOp::BitwiseNot));
    /// # Ok(()) }
    /// ```
    pub fn unary_op(&self) -> Option<UnaryOp> {
        (self.kind() == ExpressionKind::UnaryOp)
            // SAFETY: the expression is valid.
            .then(|| UnaryOp::from_raw(unsafe { sys::slang_expr_unary_op(self.raw) }))
            .flatten()
    }

    /// For an `Assignment` expression, whether it is non-blocking (`<=`).
    /// `None` if this is not an assignment.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::kinds::SymbolKind;
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m(input logic clk, input logic [7:0] d, output logic [7:0] q);\n\
    /// #     always_ff @(posedge clk) q <= d; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let block = body.members().find(|s| s.kind() == SymbolKind::ProceduralBlock).unwrap();
    /// let stmt = block.body().unwrap().statements()[0]; // the `q <= d` statement
    /// let assign = stmt.expressions()[0];
    /// assert_eq!(assign.is_nonblocking(), Some(true));
    /// # Ok(()) }
    /// ```
    pub fn is_nonblocking(&self) -> Option<bool> {
        // SAFETY: the expression is valid.
        (self.kind() == ExpressionKind::Assignment)
            .then(|| unsafe { sys::slang_expr_assignment_is_nonblocking(self.raw) })
    }

    /// For a `Call` expression, the subroutine (`function`/`task`) it invokes,
    /// or `None` for a system call or a non-call.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; function automatic int f(int a); return a + 1; endfunction\n\
    /// #     localparam int C = f(3); endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let call = body.find("C").unwrap().initializer().unwrap(); // `f(3)`
    /// assert_eq!(call.call_subroutine().unwrap().name(), "f");
    /// # Ok(()) }
    /// ```
    pub fn call_subroutine(&self) -> Option<Symbol<'d>> {
        // SAFETY: the expression is valid.
        let ast = unsafe { sys::slang_expr_call_subroutine(self.raw) };
        wrap(ast)
    }

    /// For a `MemberAccess` expression (`s.field`), the accessed member symbol.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m;\n\
    /// #     typedef struct packed { logic [3:0] a; logic [3:0] b; } pair_t;\n\
    /// #     pair_t p; wire [3:0] fld = p.a; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let access = body.find("fld").unwrap().initializer().unwrap(); // `p.a`
    /// assert_eq!(access.member_symbol().unwrap().name(), "a");
    /// # Ok(()) }
    /// ```
    pub fn member_symbol(&self) -> Option<Symbol<'d>> {
        // SAFETY: the expression is valid.
        let ast = unsafe { sys::slang_expr_member_symbol(self.raw) };
        wrap(ast)
    }

    /// The true-value operand of a `ConditionalOp` expression (`c ? t : f`),
    /// or `None` if this is not one.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic c; logic [7:0] a, b;\n\
    /// #     wire [7:0] s = c ? a : b; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let cond = body.find("s").unwrap().initializer().unwrap();
    /// assert_eq!(cond.true_value().unwrap().referenced_symbol().unwrap().name(), "a");
    /// # Ok(()) }
    /// ```
    pub fn true_value(&self) -> Option<Expression<'d>> {
        // SAFETY: the expression is valid; a null node for a non-conditional.
        wrap(unsafe { sys::slang_expr_cond_true(self.raw) })
    }

    /// The false-value operand of a `ConditionalOp` expression (`c ? t : f`),
    /// or `None` if this is not one.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic c; logic [7:0] a, b;\n\
    /// #     wire [7:0] s = c ? a : b; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let cond = body.find("s").unwrap().initializer().unwrap();
    /// assert_eq!(cond.false_value().unwrap().referenced_symbol().unwrap().name(), "b");
    /// # Ok(()) }
    /// ```
    pub fn false_value(&self) -> Option<Expression<'d>> {
        // SAFETY: the expression is valid; a null node for a non-conditional.
        wrap(unsafe { sys::slang_expr_cond_false(self.raw) })
    }

    /// The base value of an `ElementSelect` or `RangeSelect` expression
    /// (`v[...]`), or `None` if this is neither.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] v; logic [2:0] i; wire b = v[i]; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let sel = body.find("b").unwrap().initializer().unwrap();
    /// assert_eq!(sel.select_value().unwrap().referenced_symbol().unwrap().name(), "v");
    /// # Ok(()) }
    /// ```
    pub fn select_value(&self) -> Option<Expression<'d>> {
        // SAFETY: the expression is valid; a null node for a non-select.
        wrap(unsafe { sys::slang_expr_select_value(self.raw) })
    }

    /// The index selector of an `ElementSelect` expression (`v[i]`), or `None`.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] v; logic [2:0] i; wire b = v[i]; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let sel = body.find("b").unwrap().initializer().unwrap();
    /// assert_eq!(sel.selector().unwrap().referenced_symbol().unwrap().name(), "i");
    /// # Ok(()) }
    /// ```
    pub fn selector(&self) -> Option<Expression<'d>> {
        // SAFETY: the expression is valid; a null node for a non-element-select.
        wrap(unsafe { sys::slang_expr_select_selector(self.raw) })
    }

    /// The two range bounds of a `RangeSelect` expression (`v[l:r]`, `v[b+:w]`),
    /// each `None` if this is not a range select.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] v; wire [3:0] n = v[7:4]; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let sel = body.find("n").unwrap().initializer().unwrap();
    /// assert!(sel.range_left().is_some() && sel.range_right().is_some());
    /// # Ok(()) }
    /// ```
    pub fn range_left(&self) -> Option<Expression<'d>> {
        // SAFETY: the expression is valid; a null node for a non-range-select.
        wrap(unsafe { sys::slang_expr_range_left(self.raw) })
    }

    /// The right range bound of a `RangeSelect` expression. See
    /// [`range_left`](Self::range_left).
    pub fn range_right(&self) -> Option<Expression<'d>> {
        // SAFETY: the expression is valid; a null node for a non-range-select.
        wrap(unsafe { sys::slang_expr_range_right(self.raw) })
    }

    /// The `RangeSelectionKind` ordinal of a `RangeSelect` expression
    /// (0 = Simple `[l:r]`, 1 = IndexedUp `[b+:w]`, 2 = IndexedDown `[b-:w]`),
    /// or `None` if this is not a range select.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] v; wire [3:0] n = v[7:4]; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let sel = body.find("n").unwrap().initializer().unwrap();
    /// assert_eq!(sel.range_selection_kind(), Some(0)); // Simple
    /// # Ok(()) }
    /// ```
    pub fn range_selection_kind(&self) -> Option<u32> {
        // SAFETY: the expression is valid.
        (self.kind() == ExpressionKind::RangeSelect)
            .then(|| unsafe { sys::slang_expr_range_selection_kind(self.raw) })
    }

    /// The operand of a `Conversion` expression (a cast or implicit
    /// conversion), or `None` if this is not one.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::kinds::ExpressionKind;
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] a; wire [15:0] w = 16'(a); endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let init = body.find("w").unwrap().initializer().unwrap();
    /// if init.kind() == ExpressionKind::Conversion {
    ///     assert!(init.conversion_operand().is_some());
    /// }
    /// # Ok(()) }
    /// ```
    pub fn conversion_operand(&self) -> Option<Expression<'d>> {
        // SAFETY: the expression is valid; a null node for a non-conversion.
        wrap(unsafe { sys::slang_expr_conversion_operand(self.raw) })
    }

    /// The `ConversionKind` ordinal of a `Conversion` expression, or `None` if
    /// this is not one.
    pub fn conversion_kind(&self) -> Option<u32> {
        // SAFETY: the expression is valid.
        (self.kind() == ExpressionKind::Conversion)
            .then(|| unsafe { sys::slang_expr_conversion_kind(self.raw) })
    }

    /// The count operand of a `Replication` expression (`{n{x}}`), or `None`.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [1:0] x; wire [7:0] r = {4{x}}; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let rep = body.find("r").unwrap().initializer().unwrap();
    /// assert!(rep.replication_count().is_some());
    /// assert!(rep.replication_concat().is_some());
    /// # Ok(()) }
    /// ```
    pub fn replication_count(&self) -> Option<Expression<'d>> {
        // SAFETY: the expression is valid; a null node for a non-replication.
        wrap(unsafe { sys::slang_expr_replication_count(self.raw) })
    }

    /// The concatenation operand of a `Replication` expression (`{n{x}}`), or
    /// `None`. See [`replication_count`](Self::replication_count).
    pub fn replication_concat(&self) -> Option<Expression<'d>> {
        // SAFETY: the expression is valid; a null node for a non-replication.
        wrap(unsafe { sys::slang_expr_replication_concat(self.raw) })
    }
}

/// Collects the immediate semantic children of any AST node.
fn sem_children<'d>(raw: sys::slang_ast) -> Vec<SemNode<'d>> {
    // `slang_ast_sem_children` collects the children once, fills up to `cap`, and
    // returns the true total — so a small guess covers the common case in one FFI
    // call, and a wider node needs a single exact-capacity retry. (Calling
    // `slang_ast_sem_child(i)` in a loop would be O(N^2): each call re-collects.)
    let mut cap = 16usize;
    loop {
        let mut buf = vec![sys::slang_ast::default(); cap];
        // SAFETY: `raw` is a valid node of this design; `buf` has `cap` slots and
        // the callee fills at most `cap`, returning the total child count.
        let total = unsafe { sys::slang_ast_sem_children(raw, buf.as_mut_ptr(), cap as u32) };
        let total = total as usize;
        if total > cap {
            cap = total; // buffer too small; retry once with the exact size
            continue;
        }
        buf.truncate(total);
        return buf.into_iter().filter_map(wrap::<SemNode>).collect();
    }
}

/// A node in the elaborated *behavioral* tree — a statement, an expression, or
/// another AST node (timing control, constraint, pattern, ...). Obtained from
/// [`Statement::children`], [`Expression::children`], or [`Symbol::body`].
#[derive(Clone, Copy)]
pub struct SemNode<'d> {
    raw: sys::slang_ast,
    _design: PhantomData<&'d Design>,
}

impl<'d> SemNode<'d> {
    /// The node's domain discriminant (`sys::SLANG_AST_*`).
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x, y; wire [7:0] s = x & y; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let kids = body.find("s").unwrap().initializer().unwrap().children();
    /// // both operands are expressions, so they share a domain discriminant.
    /// assert_eq!(kids[0].domain(), kids[1].domain());
    /// # Ok(()) }
    /// ```
    pub fn domain(&self) -> u32 {
        self.raw.domain
    }

    /// The node's kind name (e.g. `"ConditionalStatement"`, `"BinaryExpression"`).
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x, y; wire [7:0] s = x & y; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let kids = body.find("s").unwrap().initializer().unwrap().children();
    /// assert_eq!(kids[0].kind_name(), "NamedValue");
    /// # Ok(()) }
    /// ```
    pub fn kind_name(&self) -> String {
        // SAFETY: the node is valid; kind_name reads the reflection table.
        ffi::owned_str(unsafe { sys::slang_ast_kind_name(self.raw.domain, self.raw.kind) })
    }

    /// This node as an [`Expression`], if it is one.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::kinds::ExpressionKind;
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x, y; wire [7:0] s = x & y; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let kids = body.find("s").unwrap().initializer().unwrap().children();
    /// let e = kids[0].as_expression().unwrap();
    /// assert_eq!(e.kind(), ExpressionKind::NamedValue);
    /// # Ok(()) }
    /// ```
    pub fn as_expression(&self) -> Option<Expression<'d>> {
        (self.raw.domain == sys::SLANG_AST_EXPRESSION).then(|| Expression::from_raw(self.raw))
    }

    /// This node as a [`Statement`], if it is one.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::kinds::SymbolKind;
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m(input logic clk, input logic [7:0] d, output logic [7:0] q);\n\
    /// #     always_ff @(posedge clk) q <= d; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let block = body.members().find(|s| s.kind() == SymbolKind::ProceduralBlock).unwrap();
    /// // the timed statement's children include a sub-statement node.
    /// let children = block.body().unwrap().children();
    /// assert!(children.iter().any(|n| n.as_statement().is_some()));
    /// # Ok(()) }
    /// ```
    pub fn as_statement(&self) -> Option<Statement<'d>> {
        (self.raw.domain == sys::SLANG_AST_STATEMENT).then(|| Statement::from_raw(self.raw))
    }

    /// The node's immediate semantic children.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic [7:0] x, y; wire [7:0] s = x & y; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let init = body.find("s").unwrap().initializer().unwrap();
    /// let node = init.children()[0]; // the `x` operand as a SemNode
    /// assert!(node.children().is_empty());
    /// # Ok(()) }
    /// ```
    pub fn children(&self) -> Vec<SemNode<'d>> {
        sem_children(self.raw)
    }
}

impl core::fmt::Debug for SemNode<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SemNode({})", self.kind_name())
    }
}

/// A statement in the elaborated behavioral tree (the body of an `always`,
/// `initial`, `final`, `function`, or `task`). `Copy` and pointer-sized.
#[derive(Clone, Copy)]
pub struct Statement<'d> {
    raw: sys::slang_ast,
    _design: PhantomData<&'d Design>,
}

impl<'d> Statement<'d> {
    /// The statement's semantic kind.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::kinds::{SymbolKind, StatementKind};
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m(input logic clk, rst, input logic [7:0] d, output logic [7:0] q);\n\
    /// #     always_ff @(posedge clk) if (rst) q <= 0; else q <= d + 1; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let block = body.members().find(|s| s.kind() == SymbolKind::ProceduralBlock).unwrap();
    /// assert_eq!(block.body().unwrap().kind(), StatementKind::Timed);
    /// # Ok(()) }
    /// ```
    pub fn kind(&self) -> sv_lang_kinds::StatementKind {
        sv_lang_kinds::StatementKind::from_raw(self.raw.kind as u16)
            .unwrap_or(sv_lang_kinds::StatementKind::Invalid)
    }

    /// The immediate semantic children (sub-statements, condition/loop
    /// expressions, timing controls), in slang's own order.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::kinds::SymbolKind;
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m(input logic clk, rst, input logic [7:0] d, output logic [7:0] q);\n\
    /// #     always_ff @(posedge clk) if (rst) q <= 0; else q <= d + 1; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let block = body.members().find(|s| s.kind() == SymbolKind::ProceduralBlock).unwrap();
    /// // a timed statement carries its timing control plus the guarded statement.
    /// assert_eq!(block.body().unwrap().children().len(), 2);
    /// # Ok(()) }
    /// ```
    pub fn children(&self) -> Vec<SemNode<'d>> {
        sem_children(self.raw)
    }

    /// The immediate child statements (e.g. the branches of an `if`, a loop
    /// body, the statements of a `begin`/`end` block).
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::kinds::{SymbolKind, StatementKind};
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m(input logic clk, rst, input logic [7:0] d, output logic [7:0] q);\n\
    /// #     always_ff @(posedge clk) if (rst) q <= 0; else q <= d + 1; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let block = body.members().find(|s| s.kind() == SymbolKind::ProceduralBlock).unwrap();
    /// let inner = block.body().unwrap().statements();
    /// assert_eq!(inner.len(), 1);
    /// assert_eq!(inner[0].kind(), StatementKind::Conditional);
    /// # Ok(()) }
    /// ```
    pub fn statements(&self) -> Vec<Statement<'d>> {
        self.children()
            .into_iter()
            .filter_map(|n| n.as_statement())
            .collect()
    }

    /// The immediate child expressions (conditions, the RHS of an assignment,
    /// loop bounds).
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::kinds::SymbolKind;
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m(input logic clk, rst, input logic [7:0] d, output logic [7:0] q);\n\
    /// #     always_ff @(posedge clk) if (rst) q <= 0; else q <= d + 1; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let block = body.members().find(|s| s.kind() == SymbolKind::ProceduralBlock).unwrap();
    /// let conditional = block.body().unwrap().statements()[0];
    /// // the `if` condition (`rst`) is a child expression.
    /// assert!(!conditional.expressions().is_empty());
    /// # Ok(()) }
    /// ```
    pub fn expressions(&self) -> Vec<Expression<'d>> {
        self.children()
            .into_iter()
            .filter_map(|n| n.as_expression())
            .collect()
    }

    /// The then-branch of a `Conditional` (`if`) statement, or `None` if this
    /// is not one.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::kinds::{SymbolKind, StatementKind};
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m(input logic clk, rst, input logic [7:0] d, output logic [7:0] q);\n\
    /// #     always_ff @(posedge clk) if (rst) q <= 0; else q <= d + 1; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let block = body.members().find(|s| s.kind() == SymbolKind::ProceduralBlock).unwrap();
    /// let cond = block.body().unwrap().statements()[0]; // the if/else
    /// assert_eq!(cond.kind(), StatementKind::Conditional);
    /// assert!(cond.then_branch().is_some());
    /// # Ok(()) }
    /// ```
    pub fn then_branch(&self) -> Option<Statement<'d>> {
        // SAFETY: the statement is valid; a null node for a non-conditional.
        let ast = unsafe { sys::slang_stmt_then_branch(self.raw) };
        wrap(ast)
    }

    /// The else-branch of a `Conditional` (`if`) statement, or `None` if there
    /// is no `else` (or this is not a conditional).
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::kinds::SymbolKind;
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m(input logic clk, rst, input logic [7:0] d, output logic [7:0] q);\n\
    /// #     always_ff @(posedge clk) if (rst) q <= 0; else q <= d + 1; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let block = body.members().find(|s| s.kind() == SymbolKind::ProceduralBlock).unwrap();
    /// let cond = block.body().unwrap().statements()[0];
    /// assert!(cond.else_branch().is_some());
    /// # Ok(()) }
    /// ```
    pub fn else_branch(&self) -> Option<Statement<'d>> {
        // SAFETY: the statement is valid; a null node when there is no else.
        let ast = unsafe { sys::slang_stmt_else_branch(self.raw) };
        wrap(ast)
    }

    /// The body of a loop (`for`/`repeat`/`while`/`do-while`/`forever`/
    /// `foreach`) or the guarded statement of a `Timed`/`Wait` statement.
    /// `None` for any other kind.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::kinds::{SymbolKind, StatementKind};
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m(input logic clk); int i;\n\
    /// #     always_ff @(posedge clk) for (i = 0; i < 4; i++) i <= i; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let block = body.members().find(|s| s.kind() == SymbolKind::ProceduralBlock).unwrap();
    /// // the timed statement wraps the for-loop; the for-loop has a body.
    /// let timed = block.body().unwrap();
    /// let for_loop = timed.body().unwrap();
    /// assert_eq!(for_loop.kind(), StatementKind::ForLoop);
    /// assert!(for_loop.body().is_some());
    /// # Ok(()) }
    /// ```
    pub fn body(&self) -> Option<Statement<'d>> {
        // SAFETY: the statement is valid; a null node for a body-less kind.
        let ast = unsafe { sys::slang_stmt_body(self.raw) };
        wrap(ast)
    }

    /// The controlling expression of a statement: the `while`/`do-while`/`wait`
    /// condition, the `repeat` count, or the `for` stop expression. `None` for
    /// any other kind.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::kinds::SymbolKind;
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m(input logic clk); int i;\n\
    /// #     always_ff @(posedge clk) while (i < 4) i <= i + 1; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let block = body.members().find(|s| s.kind() == SymbolKind::ProceduralBlock).unwrap();
    /// let while_loop = block.body().unwrap().body().unwrap();
    /// assert!(while_loop.condition().is_some());
    /// # Ok(()) }
    /// ```
    pub fn condition(&self) -> Option<Expression<'d>> {
        // SAFETY: the statement is valid; a null node for a kind with no
        // controlling expression.
        let ast = unsafe { sys::slang_stmt_cond(self.raw) };
        wrap(ast)
    }

    /// The principal expression of a statement: an `ExpressionStatement`'s
    /// expression, a `case` selector, a `return` value (if any), or a `foreach`
    /// array reference. `None` for any other kind.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::kinds::{SymbolKind, ExpressionKind};
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m(input logic clk, input logic [7:0] d, output logic [7:0] q);\n\
    /// #     always_ff @(posedge clk) q <= d; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let block = body.members().find(|s| s.kind() == SymbolKind::ProceduralBlock).unwrap();
    /// let expr_stmt = block.body().unwrap().statements()[0]; // `q <= d;`
    /// assert_eq!(expr_stmt.expr().unwrap().kind(), ExpressionKind::Assignment);
    /// # Ok(()) }
    /// ```
    pub fn expr(&self) -> Option<Expression<'d>> {
        // SAFETY: the statement is valid; a null node for a kind with no
        // principal expression.
        let ast = unsafe { sys::slang_stmt_expr(self.raw) };
        wrap(ast)
    }

    /// The timing control of a `Timed` statement (an `@(...)`/`#`-delayed
    /// statement), as a [`SemNode`] in the timing-control domain. `None` for any
    /// other kind.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::kinds::{SymbolKind, StatementKind};
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m(input logic clk, input logic [7:0] d, output logic [7:0] q);\n\
    /// #     always_ff @(posedge clk) q <= d; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let block = body.members().find(|s| s.kind() == SymbolKind::ProceduralBlock).unwrap();
    /// let timed = block.body().unwrap();
    /// assert_eq!(timed.kind(), StatementKind::Timed);
    /// assert!(timed.timing().is_some());
    /// # Ok(()) }
    /// ```
    pub fn timing(&self) -> Option<SemNode<'d>> {
        // SAFETY: the statement is valid; a null node for a non-timed kind.
        let ast = unsafe { sys::slang_stmt_timing(self.raw) };
        wrap(ast)
    }
}

impl core::fmt::Debug for Statement<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Statement({:?})", self.kind())
    }
}

/// A SystemVerilog unary operator (see [`Expression::unary_op`]). Ordinals match
/// slang's `UnaryOperator`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
#[allow(missing_docs)]
pub enum UnaryOp {
    Plus,
    Minus,
    BitwiseNot,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseNand,
    BitwiseNor,
    BitwiseXnor,
    LogicalNot,
    Preincrement,
    Predecrement,
    Postincrement,
    Postdecrement,
}

impl UnaryOp {
    /// The operator for a raw slang ordinal, or `None` if unknown.
    ///
    /// # Examples
    /// ```
    /// use sv_lang::UnaryOp;
    /// assert_eq!(UnaryOp::from_raw(2), Some(UnaryOp::BitwiseNot));
    /// assert_eq!(UnaryOp::from_raw(999), None);
    /// ```
    pub fn from_raw(v: u32) -> Option<UnaryOp> {
        (v < 14).then(|| {
            // SAFETY: UnaryOp is #[repr(u32)] with contiguous discriminants
            // 0..14, and v is checked to be in range.
            unsafe { core::mem::transmute::<u32, UnaryOp>(v) }
        })
    }
}

/// A SystemVerilog binary operator (see [`Expression::binary_op`]). Ordinals
/// match slang's `BinaryOperator`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
#[allow(missing_docs)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Mod,
    BinaryAnd,
    BinaryOr,
    BinaryXor,
    BinaryXnor,
    Equality,
    Inequality,
    CaseEquality,
    CaseInequality,
    GreaterThanEqual,
    GreaterThan,
    LessThanEqual,
    LessThan,
    WildcardEquality,
    WildcardInequality,
    LogicalAnd,
    LogicalOr,
    LogicalImplication,
    LogicalEquivalence,
    LogicalShiftLeft,
    LogicalShiftRight,
    ArithmeticShiftLeft,
    ArithmeticShiftRight,
    Power,
}

impl BinaryOp {
    /// The operator for a raw slang ordinal, or `None` if unknown.
    ///
    /// # Examples
    /// ```
    /// use sv_lang::BinaryOp;
    /// assert_eq!(BinaryOp::from_raw(0), Some(BinaryOp::Add));
    /// assert_eq!(BinaryOp::from_raw(28), None);
    /// ```
    pub fn from_raw(v: u32) -> Option<BinaryOp> {
        (v < 28).then(|| {
            // SAFETY: BinaryOp is #[repr(u32)] with contiguous discriminants
            // 0..28, and v is checked to be in range.
            unsafe { core::mem::transmute::<u32, BinaryOp>(v) }
        })
    }
}

// Debug impls print the kind, which is the useful identity at a glance.
impl core::fmt::Debug for Symbol<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Symbol({}, {:?})", self.kind(), self.name())
    }
}
impl core::fmt::Debug for Type<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Type({})", self.to_sv_string())
    }
}
impl core::fmt::Debug for Expression<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Expression({})", self.kind())
    }
}

// Symbol identity is its address; two-handle comparisons additionally require
// the same design (a cross-design comparison is always unequal).
impl PartialEq for Symbol<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.raw.ptr == other.raw.ptr && self.same_design(other.raw)
    }
}
impl Eq for Symbol<'_> {}

/// Which analysis passes to run, as a set of flags combined with `|`.
///
/// # Examples
/// ```
/// use sv_lang::AnalysisFlags;
/// let flags = AnalysisFlags::CHECK_UNUSED | AnalysisFlags::CHECK_SHADOW;
/// assert!(flags.contains(AnalysisFlags::CHECK_UNUSED));
/// assert!(!flags.contains(AnalysisFlags::ALLOW_MULTI_DRIVEN_LOCALS));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnalysisFlags(u32);

impl AnalysisFlags {
    /// No passes.
    pub const NONE: AnalysisFlags = AnalysisFlags(0);
    /// Report unused code (nets, variables, parameters, imports, ...).
    pub const CHECK_UNUSED: AnalysisFlags = AnalysisFlags(sys::SLANG_ANALYSIS_CHECK_UNUSED);
    /// Treat `unique`/`priority` case statements as full.
    pub const FULL_CASE_UNIQUE_PRIORITY: AnalysisFlags =
        AnalysisFlags(sys::SLANG_ANALYSIS_FULL_CASE_UNIQUE_PRIORITY);
    /// Consider four-state values in full-case analysis.
    pub const FULL_CASE_FOUR_STATE: AnalysisFlags =
        AnalysisFlags(sys::SLANG_ANALYSIS_FULL_CASE_FOUR_STATE);
    /// Allow multiple drivers on local variables.
    pub const ALLOW_MULTI_DRIVEN_LOCALS: AnalysisFlags =
        AnalysisFlags(sys::SLANG_ANALYSIS_ALLOW_MULTI_DRIVEN_LOCALS);
    /// Allow duplicate `initial` drivers.
    pub const ALLOW_DUP_INITIAL_DRIVERS: AnalysisFlags =
        AnalysisFlags(sys::SLANG_ANALYSIS_ALLOW_DUP_INITIAL_DRIVERS);
    /// Warn on variables that shadow an outer declaration.
    pub const CHECK_SHADOW: AnalysisFlags = AnalysisFlags(sys::SLANG_ANALYSIS_CHECK_SHADOW);
    /// Inline reads within functions called from a continuous assignment into
    /// the assignment's implicit sensitivity read-set (like `always_comb`).
    pub const INLINE_CONT_ASSIGN_FUNCTION_READS: AnalysisFlags =
        AnalysisFlags(sys::SLANG_ANALYSIS_INLINE_CONT_ASSIGN_FUNCTION_READS);
    /// `always @*` uses longest-static-prefix ranges for its implicit
    /// sensitivity list (a non-standard extension some simulators support).
    pub const ALWAYS_STAR_USES_LSPS: AnalysisFlags =
        AnalysisFlags(sys::SLANG_ANALYSIS_ALWAYS_STAR_USES_LSPS);
    /// Continuous assignments use longest-static-prefix ranges for their
    /// implicit sensitivity list (like `always_comb`).
    pub const CONT_ASSIGN_USES_LSPS: AnalysisFlags =
        AnalysisFlags(sys::SLANG_ANALYSIS_CONT_ASSIGN_USES_LSPS);

    /// The raw bitmask.
    ///
    /// # Examples
    /// ```
    /// use sv_lang::AnalysisFlags;
    /// assert_eq!(AnalysisFlags::NONE.bits(), 0);
    /// assert!(AnalysisFlags::CHECK_UNUSED.bits() != 0);
    /// ```
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// True if all of `other`'s flags are set.
    ///
    /// # Examples
    /// ```
    /// use sv_lang::AnalysisFlags;
    /// let flags = AnalysisFlags::CHECK_UNUSED | AnalysisFlags::CHECK_SHADOW;
    /// assert!(flags.contains(AnalysisFlags::CHECK_SHADOW));
    /// assert!(AnalysisFlags::NONE.contains(AnalysisFlags::NONE));
    /// ```
    pub const fn contains(self, other: AnalysisFlags) -> bool {
        self.0 & other.0 == other.0
    }
}

impl core::ops::BitOr for AnalysisFlags {
    type Output = AnalysisFlags;
    fn bitor(self, rhs: AnalysisFlags) -> AnalysisFlags {
        AnalysisFlags(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for AnalysisFlags {
    fn bitor_assign(&mut self, rhs: AnalysisFlags) {
        self.0 |= rhs.0;
    }
}

/// How a value is driven.
///
/// # Examples
/// ```
/// use sv_lang::DriverKind;
/// let kind = DriverKind::Continuous;
/// assert!(matches!(kind, DriverKind::Continuous));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriverKind {
    /// A procedural assignment (inside `always`/`initial`/a task/function).
    Procedural,
    /// A continuous assignment or a port/net connection.
    Continuous,
    /// Any other kind of driver.
    Other,
}

/// One driver of a value: an assignment or connection that writes it.
#[derive(Clone, Copy)]
pub struct Driver<'d> {
    info: sys::slang_driver_info,
    _design: PhantomData<&'d Design>,
}

impl<'d> Driver<'d> {
    /// How the value is driven.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::{AnalysisFlags, DriverKind};
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m(input logic a); logic z; assign z = a; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let analysis = design.analyze(AnalysisFlags::NONE, 1)?;
    /// let z = body.find("z").unwrap();
    /// let driver = analysis.drivers(z).next().unwrap();
    /// assert_eq!(driver.kind(), DriverKind::Continuous);
    /// # Ok(()) }
    /// ```
    pub fn kind(&self) -> DriverKind {
        match self.info.kind {
            sys::SLANG_DRIVER_CONTINUOUS => DriverKind::Continuous,
            sys::SLANG_DRIVER_OTHER => DriverKind::Other,
            _ => DriverKind::Procedural,
        }
    }

    /// True if the driver is an input port.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::{AnalysisFlags, kinds::SymbolKind};
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m(input logic a); logic z; assign z = a; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let analysis = design.analyze(AnalysisFlags::NONE, 1)?;
    /// // the input net `a` is driven through the input port.
    /// let net = body.members().find(|s| s.name() == "a" && s.kind() == SymbolKind::Net).unwrap();
    /// let driver = analysis.drivers(net).next().unwrap();
    /// assert!(driver.is_input_port());
    /// # Ok(()) }
    /// ```
    pub fn is_input_port(&self) -> bool {
        self.info.flags & sys::SLANG_DRIVER_INPUT_PORT != 0
    }

    /// True if the driver is a clocking-block variable.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::AnalysisFlags;
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m(input logic a); logic z; assign z = a; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let analysis = design.analyze(AnalysisFlags::NONE, 1)?;
    /// let driver = analysis.drivers(body.find("z").unwrap()).next().unwrap();
    /// assert!(!driver.is_clock_var());
    /// # Ok(()) }
    /// ```
    pub fn is_clock_var(&self) -> bool {
        self.info.flags & sys::SLANG_DRIVER_CLOCK_VAR != 0
    }

    /// The symbol (procedure, continuous-assign, ...) the driver is inside.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::{AnalysisFlags, kinds::SymbolKind};
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m(input logic a); logic z; assign z = a; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let analysis = design.analyze(AnalysisFlags::NONE, 1)?;
    /// let net = body.members().find(|s| s.name() == "a" && s.kind() == SymbolKind::Net).unwrap();
    /// let driver = analysis.drivers(net).next().unwrap();
    /// assert_eq!(driver.containing_symbol().name(), "m");
    /// # Ok(()) }
    /// ```
    pub fn containing_symbol(&self) -> Symbol<'d> {
        Symbol::from_raw(self.info.containing_symbol)
    }
}

impl core::fmt::Debug for Driver<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Driver({:?}, in {:?})",
            self.kind(),
            self.containing_symbol().name()
        )
    }
}

/// Cross-crate wrapper for a symbol node (used by `dataflow`); `None` on null.
pub(crate) fn symbol_opt_from_raw<'d>(ast: sys::slang_ast) -> Option<Symbol<'d>> {
    wrap(ast)
}

/// Cross-crate wrapper for an expression node (used by `dataflow`), guarded on
/// the expression domain: a non-expression `slang_ast` (e.g. a statement node
/// passed as `ev.node`) maps to `None` rather than a mistyped handle.
pub(crate) fn expression_opt_from_raw<'d>(ast: sys::slang_ast) -> Option<Expression<'d>> {
    wrap(ast).filter(|_| ast.domain == sys::SLANG_AST_EXPRESSION)
}

/// One procedure analyzed by slang (an `always`/`initial`/`final` block, a
/// continuous assignment, or a subroutine), as inspected through an
/// [`Analysis`].
#[derive(Clone, Copy)]
pub struct AnalyzedProcedure<'d> {
    symbol: Symbol<'d>,
    analysis: sys::slang_analysis,
    scope: sys::slang_ast,
    index: u32,
}

impl<'d> AnalyzedProcedure<'d> {
    /// The analyzed symbol (e.g. the `ProceduralBlock` or `ContinuousAssign`).
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::{AnalysisFlags, kinds::SymbolKind};
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m(input logic a, output logic y); always_comb y = ~a; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let analysis = design.analyze(AnalysisFlags::NONE, 1)?;
    /// let proc = analysis.procedures(body).next().unwrap();
    /// assert_eq!(proc.symbol().kind(), SymbolKind::ProceduralBlock);
    /// # Ok(()) }
    /// ```
    pub fn symbol(&self) -> Symbol<'d> {
        self.symbol
    }

    /// Whether slang inferred a clock for this procedure.
    ///
    /// Note this is the SVA sense of "inferred clock": slang only performs clock
    /// inference for a procedure that contains at least one **concurrent
    /// assertion**. It is therefore `false` for ordinary clocked logic such as a
    /// plain `always_ff @(posedge clk)` that has no concurrent assertion — this
    /// is not "is this block clocked".
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::AnalysisFlags;
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m(input logic a, output logic y); always_comb y = ~a; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let analysis = design.analyze(AnalysisFlags::NONE, 1)?;
    /// let proc = analysis.procedures(body).next().unwrap();
    /// // No concurrent assertion, so no clock is inferred.
    /// assert!(!proc.has_inferred_clock());
    /// # Ok(()) }
    /// ```
    pub fn has_inferred_clock(&self) -> bool {
        // SAFETY: the analysis and scope are valid; index in range.
        unsafe { sys::slang_analysis_procedure_has_clock(self.analysis, self.scope, self.index) }
    }
}

impl core::fmt::Debug for AnalyzedProcedure<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "AnalyzedProcedure({:?}, clocked={})",
            self.symbol.kind(),
            self.has_inferred_clock()
        )
    }
}

/// The result of running slang's semantic analysis (lints and driver tracking)
/// over a [`Design`]. Borrows the design.
pub struct Analysis<'d> {
    raw: sys::slang_analysis,
    design: &'d Design,
}

// SAFETY: the analysis result is immutable once produced and its accessors are
// reads; it borrows a Send+Sync design.
unsafe impl Send for Analysis<'_> {}
// SAFETY: as above — read-only over an immutable result.
unsafe impl Sync for Analysis<'_> {}

impl Drop for Analysis<'_> {
    fn drop(&mut self) {
        // SAFETY: we own the analysis handle.
        unsafe { sys::slang_analysis_destroy(self.raw) };
    }
}

impl Design {
    /// Runs the selected analysis passes over the design.
    ///
    /// The design must already be frozen (it always is). `threads` sets the
    /// worker count; 0 lets slang choose, 1 forces single-threaded.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::AnalysisFlags;
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m(input logic a, output logic y); always_comb y = ~a; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let analysis = design.analyze(AnalysisFlags::NONE, 1)?;
    /// assert_eq!(analysis.procedures(body).count(), 1);
    /// # Ok(()) }
    /// ```
    pub fn analyze(&self, flags: AnalysisFlags, threads: u32) -> Result<Analysis<'_>, Error> {
        let mut err = ffi::error();
        // SAFETY: the compilation is valid; out-error checked.
        let raw = unsafe { sys::slang_analysis_run(self.raw(), flags.bits(), threads, &mut err) };
        ffi::check(&err)?;
        Ok(Analysis { raw, design: self })
    }

    /// Runs the selected analysis passes, invoking `listener` with each analyzed
    /// procedure symbol (an `always`/`initial`/`final` block or subroutine),
    /// then returns the completed [`Analysis`]. The ergonomic shorthand for an
    /// [`AnalysisListener`] that only cares about procedures; for scopes and
    /// assertions too, use [`analyze_with_listeners`](Self::analyze_with_listeners).
    ///
    /// The `Symbol` is valid only for that call — the `for<'_>` bound prevents
    /// it from escaping. With `threads != 1` the listener may be called
    /// concurrently, hence `Send + Sync`. A panic is caught (it must not unwind
    /// across slang's C++ frames) and surfaces as [`Error::Internal`].
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use std::sync::Mutex;
    /// use sv_lang::{kinds::SymbolKind, AnalysisFlags};
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic clk, x, y; always_ff @(posedge clk) x <= y; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let procedures = Mutex::new(0usize);
    /// let analysis = design.analyze_with_listener(AnalysisFlags::NONE, 1, |proc| {
    ///     if proc.kind() == SymbolKind::ProceduralBlock {
    ///         *procedures.lock().unwrap() += 1;
    ///     }
    /// })?;
    /// assert_eq!(*procedures.lock().unwrap(), 1);
    /// let _ = analysis.diagnostics();
    /// # Ok(()) }
    /// ```
    pub fn analyze_with_listener<F>(
        &self,
        flags: AnalysisFlags,
        threads: u32,
        listener: F,
    ) -> Result<Analysis<'_>, Error>
    where
        F: Fn(Symbol<'_>) + Send + Sync,
    {
        self.analyze_with_listeners(flags, threads, &ProcedureOnly(listener))
    }

    /// Runs the selected analysis passes, driving an [`AnalysisListener`] whose
    /// `on_procedure` / `on_scope` / `on_assertion` methods fire as those items
    /// are analyzed. The trait's methods default to no-ops, so implement only
    /// the ones you need.
    ///
    /// Each `Symbol` is valid only for that call. With `threads != 1` the
    /// listener may be called concurrently (hence the `Send + Sync` supertrait);
    /// a panic in any method is caught and surfaces as [`Error::Internal`] once
    /// analysis finishes.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use std::sync::atomic::{AtomicUsize, Ordering};
    /// use sv_lang::{AnalysisFlags, AnalysisListener, Symbol};
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic clk, x, y; always_ff @(posedge clk) x <= y; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// #[derive(Default)]
    /// struct Counts { procs: AtomicUsize, scopes: AtomicUsize }
    /// impl AnalysisListener for Counts {
    ///     fn on_procedure(&self, _p: Symbol<'_>) { self.procs.fetch_add(1, Ordering::SeqCst); }
    ///     fn on_scope(&self, _s: Symbol<'_>) { self.scopes.fetch_add(1, Ordering::SeqCst); }
    /// }
    /// let counts = Counts::default();
    /// design.analyze_with_listeners(AnalysisFlags::NONE, 1, &counts)?;
    /// assert!(counts.procs.load(Ordering::SeqCst) >= 1);
    /// # Ok(()) }
    /// ```
    pub fn analyze_with_listeners<L>(
        &self,
        flags: AnalysisFlags,
        threads: u32,
        listener: &L,
    ) -> Result<Analysis<'_>, Error>
    where
        L: AnalysisListener,
    {
        use std::sync::atomic::{AtomicBool, Ordering};

        // Raw pointer to the caller's listener (kept alive on the stack for the
        // synchronous call) plus a poison flag, shared with the C trampolines.
        // `L: Send + Sync` makes concurrent reads from worker threads sound.
        struct Ctx<L> {
            listener: *const L,
            poisoned: AtomicBool,
        }

        // One shared body for all three trampolines: reconstruct the symbol,
        // skip if already poisoned, and contain any panic.
        unsafe fn dispatch<L: AnalysisListener, M>(
            user: *mut core::ffi::c_void,
            raw: sys::slang_ast,
            method: M,
        ) where
            M: Fn(&L, Symbol<'_>),
        {
            // SAFETY: `user` is the `&Ctx<L>` passed below, alive for the call.
            let ctx = unsafe { &*(user as *const Ctx<L>) };
            if ctx.poisoned.load(Ordering::Relaxed) {
                return;
            }
            // SAFETY: `listener` points at the caller's `&L`, still on the stack.
            let listener = unsafe { &*ctx.listener };
            let sym = Symbol::from_raw(raw);
            let call = std::panic::AssertUnwindSafe(|| method(listener, sym));
            if std::panic::catch_unwind(call).is_err() {
                ctx.poisoned.store(true, Ordering::Relaxed);
            }
        }

        unsafe extern "C" fn on_proc<L: AnalysisListener>(
            procedure: sys::slang_ast,
            user: *mut core::ffi::c_void,
        ) {
            // SAFETY: `user` is the `&Ctx<L>` passed to slang_analysis_run_listening.
            unsafe { dispatch::<L, _>(user, procedure, |l, s| l.on_procedure(s)) };
        }
        unsafe extern "C" fn on_scope<L: AnalysisListener>(
            scope: sys::slang_ast,
            user: *mut core::ffi::c_void,
        ) {
            // SAFETY: `user` is the `&Ctx<L>` passed to slang_analysis_run_listening.
            unsafe { dispatch::<L, _>(user, scope, |l, s| l.on_scope(s)) };
        }
        unsafe extern "C" fn on_assert<L: AnalysisListener>(
            containing: sys::slang_ast,
            user: *mut core::ffi::c_void,
        ) {
            // SAFETY: `user` is the `&Ctx<L>` passed to slang_analysis_run_listening.
            unsafe { dispatch::<L, _>(user, containing, |l, s| l.on_assertion(s)) };
        }

        let ctx = Ctx {
            listener: listener as *const L,
            poisoned: AtomicBool::new(false),
        };
        let listeners = sys::slang_analysis_listeners {
            on_procedure: Some(on_proc::<L>),
            on_scope: Some(on_scope::<L>),
            on_assertion: Some(on_assert::<L>),
            user: &ctx as *const Ctx<L> as *mut core::ffi::c_void,
        };

        let mut err = ffi::error();
        // SAFETY: the compilation, `ctx`, and `listener` outlive the synchronous
        // call; the trampolines never unwind; out-error checked.
        let raw = unsafe {
            sys::slang_analysis_run_listening(
                self.raw(),
                flags.bits(),
                threads,
                &listeners,
                &mut err,
            )
        };
        let check = ffi::check(&err);
        if ctx.poisoned.load(Ordering::Relaxed) {
            if !raw.is_null() {
                // SAFETY: we own the just-created analysis handle.
                unsafe { sys::slang_analysis_destroy(raw) };
            }
            return Err(Error::Internal("analysis listener panicked".into()));
        }
        check?;
        Ok(Analysis { raw, design: self })
    }
}

/// Callbacks driven by [`Design::analyze_with_listeners`] as analysis processes
/// each procedure, scope and assertion. All methods default to no-ops, so
/// implement only those you need. `Send + Sync` because multi-threaded analysis
/// may call them concurrently.
pub trait AnalysisListener: Send + Sync {
    /// Each analyzed procedure (`always`/`initial`/`final` block or subroutine);
    /// the argument is the procedure symbol.
    fn on_procedure(&self, _procedure: Symbol<'_>) {}
    /// Each analyzed scope; the argument is the scope's symbol.
    fn on_scope(&self, _scope: Symbol<'_>) {}
    /// Each analyzed assertion; the argument is its containing symbol.
    fn on_assertion(&self, _containing: Symbol<'_>) {}
}

/// Adapts a plain procedure closure to [`AnalysisListener`] for
/// [`Design::analyze_with_listener`].
struct ProcedureOnly<F>(F);

impl<F> AnalysisListener for ProcedureOnly<F>
where
    F: Fn(Symbol<'_>) + Send + Sync,
{
    fn on_procedure(&self, procedure: Symbol<'_>) {
        (self.0)(procedure)
    }
}

impl<'d> Analysis<'d> {
    /// The diagnostics produced by analysis (the lints selected by the flags).
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::AnalysisFlags;
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m(input logic a, output logic y); always_comb y = ~a; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let analysis = design.analyze(AnalysisFlags::CHECK_UNUSED, 1)?;
    /// assert!(!analysis.diagnostics().has_errors());
    /// # Ok(()) }
    /// ```
    pub fn diagnostics(&self) -> Diagnostics {
        let mut err = ffi::error();
        // SAFETY: the analysis is valid; the handle is consumed by collect.
        let diags = unsafe { sys::slang_analysis_diagnostics(self.raw, &mut err) };
        if diags.is_null() {
            return Diagnostics::default();
        }
        crate::collect_diagnostics(self.design.inner.session.raw(), diags)
    }

    /// The procedures (`always`/`initial`/`final` blocks, continuous
    /// assignments, subroutines) that were analyzed in a scope, as their
    /// analyzed symbols. `scope` is a module/instance body, package, etc.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::AnalysisFlags;
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m(input logic a, output logic y); always_comb y = ~a; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let analysis = design.analyze(AnalysisFlags::NONE, 1)?;
    /// assert_eq!(analysis.procedures(body).count(), 1);
    /// # Ok(()) }
    /// ```
    pub fn procedures(
        &self,
        scope: Symbol<'d>,
    ) -> impl Iterator<Item = AnalyzedProcedure<'d>> + '_ {
        // SAFETY: scope belongs to the analyzed design.
        let count = unsafe { sys::slang_analysis_scope_procedure_count(self.raw, scope.raw) };
        (0..count).filter_map(move |i| {
            // SAFETY: index in range.
            let ast = unsafe { sys::slang_analysis_scope_procedure(self.raw, scope.raw, i) };
            (!ast.ptr.is_null()).then_some(AnalyzedProcedure {
                symbol: Symbol::from_raw(ast),
                analysis: self.raw,
                scope: scope.raw,
                index: i,
            })
        })
    }

    /// The drivers of a value symbol (assignments or connections that write it).
    /// Empty if `value` is not a value symbol.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::AnalysisFlags;
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m(input logic a); logic z; assign z = a; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let analysis = design.analyze(AnalysisFlags::NONE, 1)?;
    /// assert_eq!(analysis.drivers(body.find("z").unwrap()).count(), 1);
    /// # Ok(()) }
    /// ```
    pub fn drivers(&self, value: Symbol<'d>) -> impl Iterator<Item = Driver<'d>> + '_ {
        // SAFETY: value belongs to the analyzed design.
        let count = unsafe { sys::slang_analysis_driver_count(self.raw, value.raw) };
        (0..count).filter_map(move |i| {
            let mut info = empty_driver_info();
            // SAFETY: out-param provided; index in range.
            let ok = unsafe { sys::slang_analysis_driver(self.raw, value.raw, i, &mut info) };
            ok.then_some(Driver {
                info,
                _design: PhantomData,
            })
        })
    }
}

fn empty_driver_info() -> sys::slang_driver_info {
    sys::slang_driver_info {
        kind: sys::SLANG_DRIVER_PROCEDURAL,
        flags: 0,
        range: sys::slang_range::default(),
        containing_symbol: sys::slang_ast {
            ptr: core::ptr::null(),
            compilation: core::ptr::null_mut(),
            kind: 0,
            domain: 0,
        },
    }
}

/// A parse-or-elaboration failure carrying the design's diagnostics, for
/// callers that prefer `?` over inspecting a produced design.
///
/// # Examples
/// ```
/// # fn main() -> Result<(), sv_lang::Error> {
/// let session = sv_lang::Session::new();
/// let mut comp = sv_lang::Compilation::new(&session)?;
/// comp.add_source("module m; assign x = missing; endmodule\n")?;
/// let err = comp.compile()?.into_result().unwrap_err();
/// assert!(!err.diagnostics.is_empty());
/// assert!(!err.rendered.is_empty());
/// # Ok(()) }
/// ```
#[derive(Debug, Clone)]
pub struct CompileErrors {
    /// The error and fatal diagnostics.
    pub diagnostics: Vec<Diagnostic>,
    /// slang's rendered text.
    pub rendered: String,
}

impl core::fmt::Display for CompileErrors {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.rendered)
    }
}

impl std::error::Error for CompileErrors {}

impl Design {
    /// Returns the design's error diagnostics as an [`Err`] if it has any, so
    /// that callers wanting "compile or fail" can use `?`. The design is
    /// returned unchanged on success.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let mut comp = sv_lang::Compilation::new(&session)?;
    /// comp.add_source("module m; endmodule\n")?;
    /// // A clean design passes through unchanged.
    /// let design = comp.compile()?.into_result().expect("no errors");
    /// assert!(design.top_instances().next().is_some());
    /// # Ok(()) }
    /// ```
    pub fn into_result(self) -> Result<Design, CompileErrors> {
        let diags = self.diagnostics();
        if diags.has_errors() {
            Err(CompileErrors {
                diagnostics: diags
                    .items()
                    .iter()
                    .filter(|d| d.is_error())
                    .cloned()
                    .collect(),
                rendered: diags.rendered().to_string(),
            })
        } else {
            Ok(self)
        }
    }
}

/// Parallel symbol traversal (feature `rayon`). Sound because a frozen [`Design`]
/// is `Sync` — every accessor reached here is a pure read of the frozen arena —
/// so the scope tree can be walked from every core at once.
#[cfg(feature = "rayon")]
impl Design {
    /// Visits every symbol in the design in parallel, applying `f` to each. The
    /// traversal recurses into scopes with `rayon::join`, so it scales across
    /// the whole thread pool. `f` must be `Sync` (it is shared across threads);
    /// use atomics or a concurrent collector to accumulate results.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use std::sync::atomic::{AtomicUsize, Ordering};
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module a; logic x; endmodule\nmodule b; logic y; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let count = AtomicUsize::new(0);
    /// design.par_visit(|_sym| { count.fetch_add(1, Ordering::Relaxed); });
    /// assert!(count.load(Ordering::Relaxed) > 2);
    /// # Ok(()) }
    /// ```
    pub fn par_visit<F>(&self, f: F)
    where
        F: Fn(Symbol<'_>) + Sync,
    {
        par_walk(self.root(), &f);
    }

    /// As [`par_visit`](Self::par_visit) but rooted at a specific symbol (e.g. a
    /// single top-level instance) rather than the whole design.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use std::sync::atomic::{AtomicUsize, Ordering};
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; logic a, b, c; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let top = design.top_instances().next().unwrap();
    /// let count = AtomicUsize::new(0);
    /// design.par_visit_from(top, |_| { count.fetch_add(1, Ordering::Relaxed); });
    /// assert!(count.load(Ordering::Relaxed) >= 1);
    /// # Ok(()) }
    /// ```
    pub fn par_visit_from<'d, F>(&'d self, sym: Symbol<'d>, f: F)
    where
        F: Fn(Symbol<'d>) + Sync,
    {
        par_walk(sym, &f);
    }
}

#[cfg(feature = "rayon")]
fn par_walk<'d, F>(sym: Symbol<'d>, f: &F)
where
    F: Fn(Symbol<'d>) + Sync,
{
    use rayon::iter::{IntoParallelIterator, ParallelIterator};

    f(sym);
    // Collect members (detaches from the &self borrow; Symbol is Copy) then
    // recurse into each in parallel. Instance bodies are reached through their
    // member set, mirroring the sequential `Symbol::visit`.
    let members: Vec<Symbol<'d>> = sym.members().collect();
    if members.is_empty() {
        return;
    }
    members.into_par_iter().for_each(|m| par_walk(m, f));
}
