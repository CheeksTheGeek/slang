//! Caller-defined forward dataflow analysis over a procedure.
//!
//! Implement [`Lattice`] for your abstract state — the entry state, how two
//! states merge where control flow rejoins, and how each read/write/call
//! *transfers* the state — and slang drives a procedure's control-flow graph
//! through it, returning the exit state. This is the building block for
//! reaching-definitions, uninitialized-use, constant-propagation, taint, and
//! similar analyses, written entirely in safe Rust.
//!
//! # Soundness of the callback boundary
//!
//! slang calls your `Lattice` methods across the FFI boundary. Every call is
//! wrapped in `std::panic::catch_unwind`: if one of your
//! methods panics, the panic is caught (never unwinding into C++), the state is
//! poisoned, the rest of the run is a no-op, and `run_dataflow` returns an
//! [`Error`]. The analysis is single-threaded per run.

use core::cell::Cell;
use core::ffi::c_void;
use core::marker::PhantomData;
use std::panic::{AssertUnwindSafe, catch_unwind};

use sv_lang_sys as sys;

use crate::{ConstantValue, Error, EvalSession, Expression, Statement, Symbol, ffi};

/// The kind of a [`DfaEvent`].
///
/// ```
/// use sv_lang::dataflow::DfaEventKind;
/// let kind = DfaEventKind::Write;
/// assert!(matches!(kind, DfaEventKind::Write));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DfaEventKind {
    /// A value symbol was read.
    Read,
    /// A value symbol was written (an assignment's left-hand side).
    Write,
    /// A subroutine was called.
    Call,
}

/// A dataflow event delivered to [`Lattice::transfer`] as slang walks a
/// procedure's control flow.
///
/// ```
/// use sv_lang::dataflow::{DfaEvent, DfaEventKind};
/// // events are normally delivered by `run_dataflow`; here is the shape of one.
/// let ev = DfaEvent { kind: DfaEventKind::Call, symbol: None, node: None };
/// assert_eq!(ev.kind, DfaEventKind::Call);
/// ```
#[derive(Clone, Copy)]
pub struct DfaEvent<'d> {
    /// What happened.
    pub kind: DfaEventKind,
    /// The value symbol read or written, or `None` for a call.
    pub symbol: Option<Symbol<'d>>,
    /// The expression node the event occurred at.
    pub node: Option<Expression<'d>>,
}

/// A caller-defined lattice and transfer function for a forward dataflow
/// analysis. The abstract state is `Self`.
///
/// The merge operations are named for the control-flow point they run at, not
/// for a fixed set operation: [`join`](Self::join) runs where branches rejoin,
/// [`meet`](Self::meet) at loop back-edges. A "must" analysis makes both
/// intersect; a "may" analysis makes both union. State mutations in
/// [`transfer`](Self::transfer) are exact for [`DfaEventKind::Write`] events
/// (always visited with a definite state) and best-effort for reads inside
/// compound short-circuit conditions.
///
/// ```
/// use sv_lang::dataflow::{Lattice, DfaEvent, DfaEventKind};
/// // A minimal "may" lattice counting writes seen on any path.
/// #[derive(Clone)]
/// struct Writes(u32);
/// impl Lattice for Writes {
///     fn top() -> Self { Writes(0) }
///     fn join(&mut self, other: &Self) { self.0 = self.0.max(other.0); }
///     fn transfer(&mut self, ev: DfaEvent<'_>) {
///         if ev.kind == DfaEventKind::Write { self.0 += 1; }
///     }
/// }
/// assert_eq!(Writes::top().0, 0);
/// ```
pub trait Lattice: Clone {
    /// The entry (top) state.
    fn top() -> Self;

    /// The unreachable (bottom) state. Defaults to [`top`](Self::top); override
    /// if your lattice distinguishes unreachable from entry.
    fn bottom() -> Self {
        Self::top()
    }

    /// Merges `other` into `self` where branches rejoin.
    fn join(&mut self, other: &Self);

    /// Merges `other` into `self` at a loop back-edge. Defaults to
    /// [`join`](Self::join).
    fn meet(&mut self, other: &Self) {
        self.join(other);
    }

    /// Applies a read/write/call event to the state.
    fn transfer(&mut self, event: DfaEvent<'_>);

    /// Called when the analysis begins visiting a case statement, before any
    /// of its branches (`slang::analysis::AbstractFlowAnalysis::
    /// visitStmt(const CaseStatement&)`, exposed to pyslang's PyFlowAnalysis
    /// as its `onCaseBegin` callback). The default implementation is a no-op.
    fn on_case_begin(&mut self, _stmt: Statement<'_>, _ctx: &FlowContext<'_>) {}

    /// Called when the analysis begins visiting a conditional (`if`/`else`)
    /// statement, before its branches (exposed to pyslang's PyFlowAnalysis as
    /// its `onConditionalBegin` callback). The default implementation is a
    /// no-op.
    fn on_conditional_begin(&mut self, _stmt: Statement<'_>, _ctx: &FlowContext<'_>) {}

    /// Called when the analysis begins visiting any loop statement (`for`,
    /// `while`, `do`-`while`, `forever`, `foreach`, `repeat`), before its
    /// body (exposed to pyslang's PyFlowAnalysis as its `onLoopBegin`
    /// callback). The default implementation is a no-op.
    fn on_loop_begin(&mut self, _stmt: Statement<'_>, _ctx: &FlowContext<'_>) {}
}

/// Context handed to [`Lattice`]'s observer hooks (`on_case_begin`,
/// `on_conditional_begin`, `on_loop_begin`): access to analysis-wide state
/// that is not part of the statement being observed.
///
/// Valid only for the duration of the callback that received it — do not let
/// it escape.
pub struct FlowContext<'a> {
    raw: sys::slang_dfa_ctx,
    _marker: PhantomData<&'a ()>,
}

impl FlowContext<'_> {
    /// True if the analysis has recorded an unrecoverable error — an
    /// `InvalidStatement` or `InvalidExpression` was visited (`slang::
    /// analysis::FlowAnalysisBase::bad`, exposed to pyslang as
    /// `PyFlowAnalysis::isBad`).
    ///
    /// ```
    /// use sv_lang::dataflow::{Lattice, DfaEvent, FlowContext};
    /// #[derive(Clone)]
    /// struct S;
    /// impl Lattice for S {
    ///     fn top() -> Self { S }
    ///     fn join(&mut self, _other: &Self) {}
    ///     fn transfer(&mut self, _ev: DfaEvent<'_>) {}
    ///     fn on_loop_begin(&mut self, _stmt: sv_lang::Statement<'_>, ctx: &FlowContext<'_>) {
    ///         let _ = ctx.is_bad(); // queryable from the hook
    ///     }
    /// }
    /// ```
    pub fn is_bad(&self) -> bool {
        // SAFETY: `raw` is valid for the duration of this callback, which is
        // exactly the lifetime of `self`.
        unsafe { sys::slang_dfa_ctx_is_bad(self.raw) }
    }

    /// Evaluates `expr` to a constant value using the analysis's own
    /// evaluation context (`slang::analysis::FlowAnalysisBase::
    /// getEvalContext`, exposed to pyslang as `PyFlowAnalysis::getEvalCtx`)
    /// — the same context slang uses for its own constant folding, e.g. to
    /// determine whether a `for` loop's stop condition is statically known
    /// so it can be unrolled. `None` if `expr` is not constant.
    pub fn eval_constant(&self, expr: Expression<'_>) -> Option<ConstantValue> {
        // SAFETY: `raw` is valid for the duration of this callback.
        let ectx = unsafe { sys::slang_dfa_ctx_eval_context(self.raw) };
        if ectx.is_null() {
            return None;
        }
        let mut err = ffi::error();
        // SAFETY: `ectx` is valid for the duration of this call; `expr`'s AST
        // node is part of the same (frozen) design being analyzed.
        let raw = unsafe { sys::slang_eval_ctx_evaluate(ectx, expr.raw(), &mut err) };
        if ffi::check(&err).is_err() {
            if !raw.is_null() {
                // SAFETY: owned handle, free on error.
                unsafe { sys::slang_constant_destroy(raw) };
            }
            return None;
        }
        // SAFETY: `raw` is a valid owned handle (or null); consumed.
        unsafe { ConstantValue::from_raw(raw) }
    }

    /// The opaque address of the current flow state
    /// (`slang::analysis::AbstractFlowAnalysis::getState`, exposed to
    /// pyslang as `PyFlowAnalysis::getCurrentState`). This is the SAME state
    /// your [`Lattice`] methods already operate on as `&mut Self` — the
    /// address is exposed only for identity (e.g. logging which state
    /// instance a hook fired on); dereferencing it is not supported.
    pub fn current_state_addr(&self) -> usize {
        // SAFETY: `raw` is valid for the duration of this callback.
        unsafe { sys::slang_dfa_ctx_state(self.raw) as usize }
    }
}

// The state slang carries is a `Box<Option<L>>`: `None` marks a poisoned state
// (a callback panicked), so every pointer slang sees is always valid and
// non-null, and poison simply makes further operations no-ops.
type State<L> = Option<L>;

struct Ctx {
    poisoned: Cell<bool>,
    compilation: sys::slang_compilation,
}

impl Ctx {
    fn poison(&self) {
        self.poisoned.set(true);
    }
}

unsafe fn ctx<'a>(user: *mut c_void) -> &'a Ctx {
    // SAFETY: `user` is the &Ctx passed to slang_dfa_run.
    unsafe { &*(user as *const Ctx) }
}

unsafe extern "C" fn top<L: Lattice>(user: *mut c_void) -> *mut c_void {
    // SAFETY: `user` is the &Ctx passed to slang_dfa_run.
    let c = unsafe { ctx(user) };
    let v = catch_unwind(L::top).unwrap_or_else(|_| {
        c.poison();
        // top panicked; fall back to bottom. If bottom ALSO panics there is no
        // valid state to return and letting the panic unwind across the C frame
        // would be UB, so aborting is the only sound option.
        catch_unwind(L::bottom).unwrap_or_else(|_| std::process::abort())
    });
    Box::into_raw(Box::new(Some(v) as State<L>)) as *mut c_void
}

unsafe extern "C" fn bottom<L: Lattice>(user: *mut c_void) -> *mut c_void {
    // SAFETY: `user` is the &Ctx passed to slang_dfa_run.
    let c = unsafe { ctx(user) };
    let v = catch_unwind(L::bottom).map_err(|_| c.poison()).ok();
    Box::into_raw(Box::new(v as State<L>)) as *mut c_void
}

unsafe extern "C" fn clone_state<L: Lattice>(
    user: *mut c_void,
    state: *const c_void,
) -> *mut c_void {
    // SAFETY: `user` is the &Ctx passed to slang_dfa_run.
    let c = unsafe { ctx(user) };
    // SAFETY: `state` is a Box<State<L>> we created.
    let s = unsafe { &*(state as *const State<L>) };
    let v = s.as_ref().and_then(|l| {
        catch_unwind(AssertUnwindSafe(|| l.clone()))
            .map_err(|_| c.poison())
            .ok()
    });
    Box::into_raw(Box::new(v)) as *mut c_void
}

// Shared body for the join/meet thunks (which differ only by which merge method
// they call): downcast both states and run `f`, poisoning on panic.
unsafe fn merge<L: Lattice>(
    user: *mut c_void,
    into: *mut c_void,
    other: *const c_void,
    f: fn(&mut L, &L),
) {
    // SAFETY: `user` is the &Ctx passed to slang_dfa_run.
    let c = unsafe { ctx(user) };
    // SAFETY: both are Box<State<L>> we created.
    let a = unsafe { &mut *(into as *mut State<L>) };
    // SAFETY: both are Box<State<L>> we created.
    let b = unsafe { &*(other as *const State<L>) };
    if let (Some(a), Some(b)) = (a.as_mut(), b.as_ref())
        && catch_unwind(AssertUnwindSafe(|| f(a, b))).is_err()
    {
        c.poison();
    }
}

unsafe extern "C" fn join<L: Lattice>(user: *mut c_void, into: *mut c_void, other: *const c_void) {
    // SAFETY: forwards the raw pointers from slang_dfa_run unchanged.
    unsafe { merge::<L>(user, into, other, L::join) };
}

unsafe extern "C" fn meet<L: Lattice>(user: *mut c_void, into: *mut c_void, other: *const c_void) {
    // SAFETY: forwards the raw pointers from slang_dfa_run unchanged.
    unsafe { merge::<L>(user, into, other, L::meet) };
}

unsafe extern "C" fn transfer<L: Lattice>(
    user: *mut c_void,
    state: *mut c_void,
    event: *const sys::slang_dfa_event,
) {
    // SAFETY: `user` is the &Ctx passed to slang_dfa_run.
    let c = unsafe { ctx(user) };
    // SAFETY: `state` is a Box<State<L>>; `event` is valid for the call.
    let s = unsafe { &mut *(state as *mut State<L>) };
    // SAFETY: `event` is valid for the duration of the call.
    let ev = unsafe { &*event };
    let Some(l) = s.as_mut() else { return };
    let kind = match ev.kind {
        sys::SLANG_DFA_WRITE => DfaEventKind::Write,
        sys::SLANG_DFA_CALL => DfaEventKind::Call,
        _ => DfaEventKind::Read,
    };
    let event = DfaEvent {
        kind,
        symbol: crate::ast::symbol_opt_from_raw(ev.symbol),
        node: crate::ast::expression_opt_from_raw(ev.node),
    };
    if catch_unwind(AssertUnwindSafe(|| l.transfer(event))).is_err() {
        c.poison();
    }
}

// Shared body for the three statement-begin-hook trampolines: read the
// current state through `ctx` (rather than a `state` parameter, since these
// callbacks are not per-event), wrap `stmt`, and dispatch to `f`.
unsafe fn stmt_hook<L: Lattice>(
    user: *mut c_void,
    dfa_ctx: sys::slang_dfa_ctx,
    stmt: sys::slang_ast,
    f: fn(&mut L, crate::Statement<'_>, &FlowContext<'_>),
) {
    // SAFETY: `user` is the &Ctx passed to slang_dfa_run.
    let c = unsafe { ctx(user) };
    // SAFETY: `dfa_ctx` is valid for the duration of this callback.
    let state_ptr = unsafe { sys::slang_dfa_ctx_state(dfa_ctx) };
    if state_ptr.is_null() {
        return;
    }
    // SAFETY: `state_ptr` is a Box<State<L>> we created.
    let s = unsafe { &mut *(state_ptr as *mut State<L>) };
    let Some(l) = s.as_mut() else { return };
    let Some(stmt) = crate::ast::statement_opt_from_raw(stmt) else {
        return;
    };
    let flow_ctx = FlowContext {
        raw: dfa_ctx,
        _marker: PhantomData,
    };
    if catch_unwind(AssertUnwindSafe(|| f(l, stmt, &flow_ctx))).is_err() {
        c.poison();
    }
}

unsafe extern "C" fn on_case_begin<L: Lattice>(
    user: *mut c_void,
    dfa_ctx: sys::slang_dfa_ctx,
    stmt: sys::slang_ast,
) {
    // SAFETY: forwards the raw pointers from slang_dfa_run unchanged.
    unsafe { stmt_hook::<L>(user, dfa_ctx, stmt, L::on_case_begin) };
}

unsafe extern "C" fn on_conditional_begin<L: Lattice>(
    user: *mut c_void,
    dfa_ctx: sys::slang_dfa_ctx,
    stmt: sys::slang_ast,
) {
    // SAFETY: forwards the raw pointers from slang_dfa_run unchanged.
    unsafe { stmt_hook::<L>(user, dfa_ctx, stmt, L::on_conditional_begin) };
}

unsafe extern "C" fn on_loop_begin<L: Lattice>(
    user: *mut c_void,
    dfa_ctx: sys::slang_dfa_ctx,
    stmt: sys::slang_ast,
) {
    // SAFETY: forwards the raw pointers from slang_dfa_run unchanged.
    unsafe { stmt_hook::<L>(user, dfa_ctx, stmt, L::on_loop_begin) };
}

unsafe extern "C" fn drop_state<L: Lattice>(user: *mut c_void, state: *mut c_void) {
    // SAFETY: reclaim the Box<State<L>> we created.
    let boxed = unsafe { Box::from_raw(state as *mut State<L>) };
    // Dropping runs the user lattice's `Drop`, which may panic; a panic must not
    // unwind across this extern "C" boundary (that is UB). Catch it, poison, and
    // swallow — teardown continues, and the run reports the poison as an error.
    if catch_unwind(AssertUnwindSafe(move || drop(boxed))).is_err() {
        // SAFETY: `user` is the &Ctx passed to slang_dfa_run.
        unsafe { ctx(user) }.poison();
    }
}

impl<'d> EvalSession<'d> {
    /// Runs a caller-defined forward dataflow analysis over a procedure symbol
    /// (an `always`/`initial`/`final` block, a subroutine, or a continuous
    /// assign) and returns the exit state.
    ///
    /// This lives on the exclusive [`EvalSession`] because the flow analysis
    /// may constant-fold into the arena. Returns an [`Error`] if the procedure
    /// is not analyzable or if one of the lattice's methods panicked.
    ///
    /// ```
    /// # use sv_lang::{Session, Compilation, kinds::SymbolKind};
    /// # use sv_lang::dataflow::{Lattice, DfaEvent, DfaEventKind};
    /// use std::collections::BTreeSet;
    ///
    /// // "Definitely assigned": the set of variable names assigned on all paths.
    /// #[derive(Clone)]
    /// struct Assigned(BTreeSet<String>, bool /* universe (unreachable) */);
    /// impl Lattice for Assigned {
    ///     fn top() -> Self { Assigned(BTreeSet::new(), false) }
    ///     fn bottom() -> Self { Assigned(BTreeSet::new(), true) }
    ///     fn join(&mut self, other: &Self) {
    ///         if other.1 { return; }
    ///         if self.1 { *self = other.clone(); return; }
    ///         self.0.retain(|k| other.0.contains(k)); // intersect
    ///     }
    ///     fn transfer(&mut self, ev: DfaEvent<'_>) {
    ///         if ev.kind == DfaEventKind::Write {
    ///             if let Some(s) = ev.symbol { self.0.insert(s.name().to_string()); }
    ///         }
    ///     }
    /// }
    ///
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = Session::new();
    /// let mut comp = Compilation::new(&session)?;
    /// comp.add_source("module m(input logic c); logic x, y; \
    ///     always_comb begin if (c) x = 1; y = 2; end endmodule\n")?;
    /// let mut design = comp.compile()?;
    /// let dfa = design.eval_session();
    /// let body = dfa.design().top_instances().next().unwrap().instance_body().unwrap();
    /// let block = body.members().find(|s| s.kind() == SymbolKind::ProceduralBlock).unwrap();
    /// let exit: Assigned = dfa.run_dataflow(block)?;
    /// assert!(exit.0.contains("y"));   // assigned on all paths
    /// assert!(!exit.0.contains("x"));  // only on one path
    /// # Ok(())
    /// # }
    /// ```
    pub fn run_dataflow<L: Lattice>(&self, procedure: Symbol<'_>) -> Result<L, Error> {
        // The C side lifts the seal on THIS design and lazily elaborates the
        // procedure's body; a foreign procedure would mutate a design we do not
        // hold exclusively. A lifetime brand cannot prove same-design (two
        // designs can share a scope), so check the compilation identity here.
        assert!(
            procedure.raw().compilation == self.design().raw_compilation(),
            "run_dataflow: procedure belongs to a different Design than this session"
        );
        let c = Ctx {
            poisoned: Cell::new(false),
            compilation: self.design().raw_compilation(),
        };
        let lattice = sys::slang_dfa_lattice {
            top: Some(top::<L>),
            bottom: Some(bottom::<L>),
            clone: Some(clone_state::<L>),
            join: Some(join::<L>),
            meet: Some(meet::<L>),
            transfer: Some(transfer::<L>),
            drop: Some(drop_state::<L>),
            on_case_begin: Some(on_case_begin::<L>),
            on_conditional_begin: Some(on_conditional_begin::<L>),
            on_loop_begin: Some(on_loop_begin::<L>),
        };
        let mut err = ffi::error();
        // SAFETY: the compilation and procedure are valid; the lattice callbacks
        // do not unwind (each is wrapped in catch_unwind); `c` outlives the call.
        let raw = unsafe {
            sys::slang_dfa_run(
                c.compilation,
                procedure.raw(),
                &lattice,
                &c as *const Ctx as *mut c_void,
                &mut err,
            )
        };
        ffi::check(&err)?;
        if raw.is_null() {
            return Err(Error::Internal("dataflow produced no state".into()));
        }
        // SAFETY: `raw` is the Box<State<L>> our `clone` callback returned.
        let exit = unsafe { *Box::from_raw(raw as *mut State<L>) };
        if c.poisoned.get() {
            return Err(Error::Internal(
                "a dataflow lattice callback panicked".into(),
            ));
        }
        exit.ok_or_else(|| Error::Internal("dataflow produced no state".into()))
    }
}
