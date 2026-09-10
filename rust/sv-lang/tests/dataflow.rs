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
