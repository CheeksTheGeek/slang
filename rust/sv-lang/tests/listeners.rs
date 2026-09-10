//! `Design::analyze_with_listener` — streaming each analyzed procedure to a
//! caller callback, with panic containment and thread safety.

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use sv_lang::kinds::SymbolKind;
use sv_lang::{AnalysisFlags, AnalysisListener, Compilation, Design, Error, Session, Symbol};

fn design(src: &str) -> Design {
    let session = Session::new();
    let mut comp = Compilation::new(&session).expect("compilation");
    comp.add_source(src).expect("add source");
    comp.compile().expect("elaborate")
}

const THREE_PROCS: &str = "\
module m;
    logic clk, a, b, c;
    always_ff @(posedge clk) a <= b;
    always_comb c = a;
    initial b = 1'b0;
endmodule
";

#[test]
fn listener_sees_each_procedure() {
    let d = design(THREE_PROCS);
    let kinds = Mutex::new(Vec::<SymbolKind>::new());

    let analysis = d
        .analyze_with_listener(AnalysisFlags::NONE, 1, |proc| {
            kinds.lock().unwrap().push(proc.kind());
        })
        .expect("analysis with listener");
    // The analysis itself is still usable afterward.
    assert!(!analysis.diagnostics().has_errors());

    let kinds = kinds.into_inner().unwrap();
    // always_ff, always_comb, initial — three procedural blocks.
    let procs = kinds
        .iter()
        .filter(|&&k| k == SymbolKind::ProceduralBlock)
        .count();
    assert_eq!(procs, 3, "expected 3 procedures, saw {kinds:?}");
}

#[test]
fn panicking_listener_is_reported_not_aborted() {
    let d = design("module m; logic clk, x, y; always_ff @(posedge clk) x <= y; endmodule\n");
    let result = d.analyze_with_listener(AnalysisFlags::NONE, 1, |_proc| {
        panic!("listener boom");
    });
    assert!(
        matches!(result, Err(Error::Internal(_))),
        "a panicking listener should surface as Err(Internal), not abort"
    );
}

#[test]
fn listener_runs_multithreaded() {
    let d = design(THREE_PROCS);
    let count = AtomicUsize::new(0);
    // threads = 0 => slang picks hardware concurrency; the listener must be
    // safe to call from several threads at once (it is `Send + Sync`).
    d.analyze_with_listener(AnalysisFlags::NONE, 0, |proc| {
        if proc.kind() == SymbolKind::ProceduralBlock {
            count.fetch_add(1, Ordering::SeqCst);
        }
    })
    .expect("multithreaded analysis");
    assert_eq!(count.load(Ordering::SeqCst), 3);
}

#[derive(Default)]
struct Counts {
    procs: AtomicUsize,
    scopes: AtomicUsize,
    assertions: AtomicUsize,
}

impl AnalysisListener for Counts {
    fn on_procedure(&self, _p: Symbol<'_>) {
        self.procs.fetch_add(1, Ordering::SeqCst);
    }
    fn on_scope(&self, _s: Symbol<'_>) {
        self.scopes.fetch_add(1, Ordering::SeqCst);
    }
    fn on_assertion(&self, _c: Symbol<'_>) {
        self.assertions.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn trait_listener_sees_procedures_and_scopes() {
    let d = design(THREE_PROCS);
    let counts = Counts::default();
    d.analyze_with_listeners(AnalysisFlags::NONE, 1, &counts)
        .expect("analysis with trait listener");
    assert_eq!(counts.procs.load(Ordering::SeqCst), 3);
    assert!(
        counts.scopes.load(Ordering::SeqCst) >= 1,
        "expected at least one analyzed scope"
    );
}

#[test]
fn trait_listener_sees_assertions() {
    // A concurrent assertion produces an analyzed assertion.
    let d = design(
        "module m;\n  logic clk, a, b;\n  assert property (@(posedge clk) a |-> b);\nendmodule\n",
    );
    let counts = Counts::default();
    d.analyze_with_listeners(AnalysisFlags::NONE, 1, &counts)
        .expect("analysis with trait listener");
    assert!(
        counts.assertions.load(Ordering::SeqCst) >= 1,
        "expected at least one analyzed assertion"
    );
}
