//! Caller-defined dataflow analysis: a "definitely assigned" lattice, and the
//! catch_unwind/poison protocol.

use std::collections::BTreeSet;

use sv_lang::dataflow::{DfaEvent, DfaEventKind, Lattice};
use sv_lang::kinds::SymbolKind;
use sv_lang::{Compilation, Session};

/// The set of variable names definitely assigned on every path to a point.
#[derive(Clone, Debug)]
struct Assigned {
    names: BTreeSet<String>,
    unreachable: bool,
}

impl Lattice for Assigned {
    fn top() -> Self {
        Assigned {
            names: BTreeSet::new(),
            unreachable: false,
        }
    }
    fn bottom() -> Self {
        Assigned {
            names: BTreeSet::new(),
            unreachable: true,
        }
    }
    fn join(&mut self, other: &Self) {
        if other.unreachable {
            return;
        }
        if self.unreachable {
            *self = other.clone();
            return;
        }
        self.names.retain(|k| other.names.contains(k)); // intersect
    }
    fn transfer(&mut self, ev: DfaEvent<'_>) {
        if ev.kind == DfaEventKind::Write
            && let Some(sym) = ev.symbol
        {
            self.names.insert(sym.name().to_string());
        }
    }
}

#[test]
fn definitely_assigned() {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source(
        "module m(input logic c);\n\
         \x20 logic x, y, z;\n\
         \x20 always_comb begin\n\
         \x20   if (c) x = 1;\n\
         \x20   y = 2;\n\
         \x20   z = y;\n\
         \x20 end\n\
         endmodule\n",
    )
    .unwrap();
    let mut design = comp.compile().unwrap();
    let dfa = design.eval_session();
    let body = dfa
        .design()
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();

    let exit: Assigned = dfa.run_dataflow(block).unwrap();
    assert!(
        exit.names.contains("y"),
        "y assigned on all paths: {exit:?}"
    );
    assert!(
        exit.names.contains("z"),
        "z assigned on all paths: {exit:?}"
    );
    assert!(!exit.names.contains("x"), "x only on one path: {exit:?}");
}

#[test]
fn loop_and_sequence() {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source(
        "module m;\n\
         \x20 int a, b, total;\n\
         \x20 initial begin\n\
         \x20   a = 1;\n\
         \x20   for (int i = 0; i < 4; i++) total = total + a;\n\
         \x20   b = total;\n\
         \x20 end\n\
         endmodule\n",
    )
    .unwrap();
    let mut design = comp.compile().unwrap();
    let dfa = design.eval_session();
    let body = dfa
        .design()
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let exit: Assigned = dfa.run_dataflow(block).unwrap();
    // `a` (before the loop) and `b` (unconditional write after it) are
    // definitely assigned. `total` is written only inside the loop body, which
    // a conservative "must" analysis treats as possibly not executing, so it is
    // correctly NOT in the definitely-assigned set.
    assert!(exit.names.contains("a"), "{exit:?}");
    assert!(exit.names.contains("b"), "{exit:?}");
    assert!(!exit.names.contains("total"), "{exit:?}");
}

/// Exercises the statement-begin observer hooks (`on_case_begin`,
/// `on_conditional_begin`, `on_loop_begin`) and the [`FlowContext`] they
/// receive (`is_bad`, `eval_constant`) — the flow-analysis-observer surface
/// mirrored from pyslang's `PyFlowAnalysis`.
#[test]
fn statement_begin_hooks_fire_with_correct_kinds_and_context() {
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

    use sv_lang::Statement;
    use sv_lang::dataflow::FlowContext;
    use sv_lang::kinds::StatementKind;

    static CASE_BEGIN: AtomicU32 = AtomicU32::new(0);
    static CONDITIONAL_BEGIN: AtomicU32 = AtomicU32::new(0);
    static LOOP_BEGIN: AtomicU32 = AtomicU32::new(0);
    static SAW_CASE_KIND: AtomicBool = AtomicBool::new(false);
    static SAW_CONDITIONAL_KIND: AtomicBool = AtomicBool::new(false);
    static SAW_REPEAT_KIND: AtomicBool = AtomicBool::new(false);
    static SAW_NOT_BAD: AtomicBool = AtomicBool::new(false);
    static FOLDED_REPEAT_COUNT: AtomicU32 = AtomicU32::new(0);
    static SAW_NONZERO_STATE_ADDR: AtomicBool = AtomicBool::new(false);

    #[derive(Clone)]
    struct Probe;
    impl Lattice for Probe {
        fn top() -> Self {
            Probe
        }
        fn join(&mut self, _other: &Self) {}
        fn transfer(&mut self, _ev: DfaEvent<'_>) {}

        fn on_conditional_begin(&mut self, stmt: Statement<'_>, ctx: &FlowContext<'_>) {
            CONDITIONAL_BEGIN.fetch_add(1, Ordering::SeqCst);
            if stmt.kind() == StatementKind::Conditional {
                SAW_CONDITIONAL_KIND.store(true, Ordering::SeqCst);
            }
            // FlowAnalysisBase::bad is queryable and false on this well-formed
            // design.
            if !ctx.is_bad() {
                SAW_NOT_BAD.store(true, Ordering::SeqCst);
            }
            // AbstractFlowAnalysis::getState -- a real, non-null state address
            // (this is the exact state your Lattice methods are, at this
            // point in the walk, operating on as `&mut Self`).
            if ctx.current_state_addr() != 0 {
                SAW_NONZERO_STATE_ADDR.store(true, Ordering::SeqCst);
            }
        }

        fn on_case_begin(&mut self, stmt: Statement<'_>, _ctx: &FlowContext<'_>) {
            CASE_BEGIN.fetch_add(1, Ordering::SeqCst);
            if stmt.kind() == StatementKind::Case {
                SAW_CASE_KIND.store(true, Ordering::SeqCst);
            }
        }

        fn on_loop_begin(&mut self, stmt: Statement<'_>, ctx: &FlowContext<'_>) {
            LOOP_BEGIN.fetch_add(1, Ordering::SeqCst);
            if stmt.kind() == StatementKind::RepeatLoop {
                SAW_REPEAT_KIND.store(true, Ordering::SeqCst);
            }
            // FlowAnalysisBase::getEvalContext, used here to fold the
            // `repeat` statement's constant count expression the same way
            // the analysis itself would (e.g. to decide whether to unroll).
            if let Some(count) = stmt.condition() {
                if let Some(cv) = ctx.eval_constant(count) {
                    if let Some(n) = cv.as_i64() {
                        FOLDED_REPEAT_COUNT.store(n as u32, Ordering::SeqCst);
                    }
                }
            }
        }
    }

    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source(
        "module m(input logic sel, output int y);\n\
         \x20 always_comb begin\n\
         \x20   if (sel) y = 1; else y = 0;\n\
         \x20   case (sel)\n\
         \x20     1'b0: y = 10;\n\
         \x20     default: y = 20;\n\
         \x20   endcase\n\
         \x20   repeat (3) y = y + 1;\n\
         \x20 end\n\
         endmodule\n",
    )
    .unwrap();
    let mut design = comp.compile().unwrap();
    let dfa = design.eval_session();
    let body = dfa
        .design()
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();

    let _exit: Probe = dfa.run_dataflow(block).unwrap();

    assert_eq!(CONDITIONAL_BEGIN.load(Ordering::SeqCst), 1);
    assert!(SAW_CONDITIONAL_KIND.load(Ordering::SeqCst));
    assert!(SAW_NOT_BAD.load(Ordering::SeqCst));
    assert!(SAW_NONZERO_STATE_ADDR.load(Ordering::SeqCst));

    assert_eq!(CASE_BEGIN.load(Ordering::SeqCst), 1);
    assert!(SAW_CASE_KIND.load(Ordering::SeqCst));

    assert_eq!(LOOP_BEGIN.load(Ordering::SeqCst), 1);
    assert!(SAW_REPEAT_KIND.load(Ordering::SeqCst));
    assert_eq!(FOLDED_REPEAT_COUNT.load(Ordering::SeqCst), 3);
}

/// [`DfaEventKind::Read`] (`onVariableRef`) and [`DfaEventKind::Call`]
/// (`onCallExpression`) are delivered to `transfer` just like `Write` — every
/// other test in this file only ever asserts on `Write`, so this closes the
/// gap by asserting a read of a known variable and at least one call fire.
#[test]
fn read_and_call_events_are_delivered() {
    #[derive(Clone, Default)]
    struct Seen {
        read_names: BTreeSet<String>,
        calls: u32,
    }
    impl Lattice for Seen {
        fn top() -> Self {
            Seen::default()
        }
        fn join(&mut self, other: &Self) {
            self.read_names.extend(other.read_names.iter().cloned());
            self.calls = self.calls.max(other.calls);
        }
        fn transfer(&mut self, ev: DfaEvent<'_>) {
            match ev.kind {
                DfaEventKind::Read => {
                    if let Some(sym) = ev.symbol {
                        self.read_names.insert(sym.name().to_string());
                    }
                }
                DfaEventKind::Call => self.calls += 1,
                DfaEventKind::Write => {}
            }
        }
    }

    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source(
        "module m;\n\
         \x20 function automatic int helper(int v);\n\
         \x20   return v + 1;\n\
         \x20 endfunction\n\
         \x20 int a, b;\n\
         \x20 initial begin\n\
         \x20   a = 1;\n\
         \x20   b = helper(a);\n\
         \x20 end\n\
         endmodule\n",
    )
    .unwrap();
    let mut design = comp.compile().unwrap();
    let dfa = design.eval_session();
    let body = dfa
        .design()
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();

    let exit: Seen = dfa.run_dataflow(block).unwrap();
    assert!(
        exit.read_names.contains("a"),
        "expected a Read event for `a`: {:?}",
        exit.read_names
    );
    assert!(exit.calls >= 1, "expected at least one Call event");
}

/// A lattice whose transfer panics; the run must return an error, not abort.
#[derive(Clone)]
struct Panicky;
impl Lattice for Panicky {
    fn top() -> Self {
        Panicky
    }
    fn join(&mut self, _: &Self) {}
    fn transfer(&mut self, ev: DfaEvent<'_>) {
        if ev.kind == DfaEventKind::Write {
            panic!("boom in transfer");
        }
    }
}

#[test]
fn panicking_lattice_is_poisoned_not_aborted() {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source("module m; int x; initial x = 1; endmodule\n")
        .unwrap();
    let mut design = comp.compile().unwrap();
    let dfa = design.eval_session();
    let body = dfa
        .design()
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    // The panic is caught; run_dataflow returns an error rather than unwinding
    // into C++ and aborting the process.
    let result = dfa.run_dataflow::<Panicky>(block);
    assert!(result.is_err(), "expected a poisoned-run error");
}
