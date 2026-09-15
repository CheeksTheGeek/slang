//! [`ScriptSession`]: evaluating snippets of SystemVerilog and keeping state
//! (declared variables and their values, defined types, packages, ...) across
//! calls — the interactive/scripting entry point, as opposed to
//! [`Compilation`](crate::Compilation)'s whole-design elaboration.

use core::marker::PhantomData;

use sv_lang_sys as sys;

use crate::{ConstantValue, Diagnostics, Error, ffi, syntax::Node};

/// A session for evaluating snippets of SystemVerilog source code and
/// maintaining state across multiple `eval` calls (mirrors
/// `slang::ast::ScriptSession`).
///
/// # Examples
/// ```
/// # fn main() -> Result<(), sv_lang::Error> {
/// let mut session = sv_lang::ScriptSession::new()?;
/// session.eval("int x = 10;"); // a declaration: no value
/// let value = session.eval("x + 1").unwrap(); // state persists across calls
/// assert_eq!(value.as_i64(), Some(11));
/// # Ok(()) }
/// ```
pub struct ScriptSession {
    raw: sys::slang_script_session,
}

// SAFETY: the session is only ever touched through `&mut self` (eval*) or
// `&self` (compilation/diagnostics reads); it has no `Clone` and is never
// shared concurrently, so it may move between threads like any owned value.
unsafe impl Send for ScriptSession {}

impl Drop for ScriptSession {
    fn drop(&mut self) {
        // SAFETY: we own this handle exclusively.
        unsafe { sys::slang_script_session_destroy(self.raw) };
    }
}

impl ScriptSession {
    /// Creates a new script session with default options.
    ///
    /// # Panics
    /// Panics only if the library fails to allocate, which in practice means
    /// the process is out of memory.
    ///
    /// # Examples
    /// ```
    /// let session = sv_lang::ScriptSession::new().unwrap();
    /// # let _ = session;
    /// ```
    pub fn new() -> Result<ScriptSession, Error> {
        let mut err = ffi::error();
        // SAFETY: null options selects slang's defaults; out-error checked.
        let raw = unsafe { sys::slang_script_session_create(core::ptr::null_mut(), &mut err) };
        ffi::check(&err)?;
        Ok(ScriptSession { raw })
    }

    fn raw(&self) -> sys::slang_script_session {
        self.raw
    }

    /// Evaluates one snippet of SystemVerilog source in this session's
    /// scope: an expression, statement, or declaration (a variable,
    /// function, task, module, typedef, or package). State (declared
    /// variables and their current values, defined types) persists across
    /// calls on the same session. Returns the snippet's value, or `None` if
    /// it produced none (e.g. a declaration) or could not be evaluated —
    /// check [`diagnostics`](Self::diagnostics) to tell the two apart.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let mut session = sv_lang::ScriptSession::new()?;
    /// assert!(session.eval("int x;").is_none()); // a declaration has no value
    /// assert_eq!(session.eval("x = 5").unwrap().as_i64(), Some(5));
    /// assert_eq!(session.eval("x").unwrap().as_i64(), Some(5)); // state persists
    /// assert!(session.eval("$$$ not valid sv $$$").is_none());
    /// assert!(session.diagnostics().has_errors());
    /// # Ok(()) }
    /// ```
    pub fn eval(&mut self, text: &str) -> Option<ConstantValue> {
        let mut err = ffi::error();
        let (t, tl) = ffi::as_ptr_len(text);
        // SAFETY: the session is valid; the string is valid for the call;
        // out-error checked.
        let raw = unsafe { sys::slang_script_session_eval(self.raw(), t, tl, &mut err) };
        // SAFETY: `raw` is a valid owned handle (or null), consumed.
        unsafe { ConstantValue::from_raw(raw) }
    }

    /// Evaluates a parsed expression syntax node against this session's
    /// scope — an identifier in `expr` resolves against variables this
    /// session has declared (via [`eval`](Self::eval)), regardless of which
    /// [`SyntaxTree`](crate::SyntaxTree) `expr` itself was parsed from.
    /// `None` if `expr` is not an expression node or could not be evaluated.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let mut session = sv_lang::ScriptSession::new()?;
    /// session.eval("int x = 10;");
    ///
    /// // The expression syntax comes from an entirely unrelated tree/session.
    /// let other = sv_lang::Session::new();
    /// let tree = other.parse("module m; initial begin y = x + 5; end endmodule\n")?;
    /// let assign_expr = tree
    ///     .root()
    ///     .descendants()
    ///     .find(|n| n.kind() == sv_lang::kinds::SyntaxKind::AddExpression)
    ///     .unwrap();
    ///
    /// // `x` in `assign_expr` resolves against the *session's* `x`, not `other`'s.
    /// let value = session.eval_expression(assign_expr).unwrap();
    /// assert_eq!(value.as_i64(), Some(15));
    /// # Ok(()) }
    /// ```
    pub fn eval_expression(&mut self, expr: Node<'_>) -> Option<ConstantValue> {
        let mut err = ffi::error();
        // SAFETY: the session is valid; `expr.raw()` is a valid node handle
        // borrowed from a live tree for the duration of this call; out-error
        // checked.
        let raw =
            unsafe { sys::slang_script_session_eval_expression(self.raw(), expr.raw(), &mut err) };
        // SAFETY: `raw` is a valid owned handle (or null), consumed.
        unsafe { ConstantValue::from_raw(raw) }
    }

    /// As [`eval_expression`](Self::eval_expression), but for a parsed
    /// statement syntax node, evaluated against this session's scope for its
    /// side effects only. Does nothing if `stmt` is not a statement node —
    /// check [`diagnostics`](Self::diagnostics) for the resulting error.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let mut session = sv_lang::ScriptSession::new()?;
    /// session.eval("int x;");
    ///
    /// let other = sv_lang::Session::new();
    /// let tree = other.parse("module m; initial begin x = 7; end endmodule\n")?;
    /// let assign_stmt = tree
    ///     .root()
    ///     .descendants()
    ///     .find(|n| n.kind() == sv_lang::kinds::SyntaxKind::ExpressionStatement)
    ///     .unwrap();
    ///
    /// session.eval_statement(assign_stmt);
    /// assert_eq!(session.eval("x").unwrap().as_i64(), Some(7)); // the side effect stuck
    /// # Ok(()) }
    /// ```
    pub fn eval_statement(&mut self, stmt: Node<'_>) {
        let mut err = ffi::error();
        // SAFETY: as `eval_expression` above.
        unsafe { sys::slang_script_session_eval_statement(self.raw(), stmt.raw(), &mut err) };
    }

    /// This session's own compilation: an always-mutable scratch compilation
    /// used to hold declared script state — NOT a frozen
    /// [`Design`](crate::Design). A fresh handle is returned each call,
    /// borrowing the session for its lifetime.
    ///
    /// # Panics
    /// Panics only if the library fails to allocate.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::ScriptSession::new()?;
    /// // A genuine, live Compilation handle: its built-in `bit` type resolves.
    /// assert_eq!(session.compilation().bit_type_name(), "bit");
    /// # Ok(()) }
    /// ```
    pub fn compilation(&self) -> ScriptCompilation<'_> {
        let mut err = ffi::error();
        // SAFETY: the session is valid; out-error checked.
        let raw = unsafe { sys::slang_script_session_compilation(self.raw(), &mut err) };
        assert!(
            !raw.is_null(),
            "slang_script_session_compilation: {}",
            ffi::message(&err)
        );
        ScriptCompilation {
            raw,
            _session: PhantomData,
        }
    }

    /// All diagnostics issued by every `eval*` call on this session so far.
    ///
    /// # Panics
    /// Panics only if the library fails to allocate.
    ///
    /// # Examples
    /// (see [`eval`](Self::eval))
    pub fn diagnostics(&mut self) -> Diagnostics {
        let mut err = ffi::error();
        // SAFETY: the session is valid; out-error checked.
        let raw = unsafe { sys::slang_script_session_diagnostics(self.raw(), &mut err) };
        assert!(
            !raw.is_null(),
            "slang_script_session_diagnostics: {}",
            ffi::message(&err)
        );
        // `raw` is a valid owned handle, consumed by this call. The session
        // internally renders against slang's process-wide default source
        // manager (see slang::syntax::SyntaxTree::getDefaultSourceManager),
        // which we have no handle to here, so per-diagnostic file/line/column
        // detail is not resolved (the rendered summary still is, on the C
        // side, which does hold that source manager).
        crate::collect_diagnostics(core::ptr::null_mut(), raw)
    }
}

/// A [`ScriptSession`]'s own compilation (see
/// [`ScriptSession::compilation`]): an always-mutable scratch compilation
/// used to hold declared script state, distinct from a frozen
/// [`Design`](crate::Design).
pub struct ScriptCompilation<'s> {
    raw: sys::slang_compilation,
    _session: PhantomData<&'s ScriptSession>,
}

impl Drop for ScriptCompilation<'_> {
    fn drop(&mut self) {
        // SAFETY: this handle never owns the underlying Compilation (which
        // belongs to the ScriptSession); destroying it only frees the
        // wrapper handle itself.
        unsafe { sys::slang_compilation_destroy(self.raw) };
    }
}

impl ScriptCompilation<'_> {
    /// The number of top-level (uninstantiated-by-others) module/program
    /// instances — always 0 for a script session, which never elaborates a
    /// design hierarchy the way [`Compilation::compile`](crate::Compilation::compile)
    /// does.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::ScriptSession::new()?;
    /// assert_eq!(session.compilation().top_instance_count(), 0);
    /// # Ok(()) }
    /// ```
    pub fn top_instance_count(&self) -> u32 {
        // SAFETY: `raw` is a valid handle for the life of `self`.
        unsafe { sys::slang_compilation_top_instance_count(self.raw) }
    }

    /// The name of the compilation's built-in 2-state `bit` type (always
    /// `"bit"`) — a pure, allocation-free read proving this is a genuine,
    /// live `Compilation` handle (mirrors `slang::ast::Compilation::getBitType`).
    ///
    /// # Examples
    /// (see [`ScriptSession::compilation`])
    pub fn bit_type_name(&self) -> String {
        // SAFETY: `raw` is a valid handle for the life of `self`.
        let ast = unsafe { sys::slang_compilation_get_bit_type(self.raw) };
        // SAFETY: `slang_symbol_name` is a pure read of the AST node itself,
        // independent of which compilation handle carried it here.
        unsafe { ffi::borrowed_str(sys::slang_symbol_name(ast)) }
    }
}
