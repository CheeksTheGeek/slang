//! `@(clock iff y)` where `clock` is a checker (or sequence/property) event
//! formal: slang unwraps the formal's clocking-event actual and used to drop the
//! use-site `iff`, so the event ran ungated (LRM 16.14.7 / 17.4 say the `iff`
//! gates it). Reported by the sv-sim session (probe gave m=3, LRM gives 2).

use sv_lang::kinds::{SymbolKind, TimingControlKind};
use sv_lang::{Compilation, Session, Symbol, Walk};

const SRC: &str = "\
checker c(event clock = $inferred_clock, logic y);
  logic [3:0] m = 0;
  always_ff @(clock iff y) m <= m + 1;
endchecker
module top;
  logic clk, y, z;
  default clocking @(posedge clk); endclocking
  c c_explicit(posedge clk, y);
  c c_inferred(.y(y));
  c c_both(posedge clk iff z, y);
  c c_list(posedge clk or negedge clk, y);
endmodule
";

/// (instance name, timing-control kind, per-signal-event child kind names)
fn timing_controls(top: Symbol<'_>) -> Vec<(String, TimingControlKind, Vec<Vec<String>>)> {
    let mut out = Vec::new();
    for inst in top
        .members()
        .filter(|s| s.kind() == SymbolKind::CheckerInstance)
    {
        let name = inst.name().to_string();
        inst.visit(|sym| {
            if sym.kind() == SymbolKind::ProceduralBlock
                && let Some(tc) = sym.body().and_then(|b| b.timing_control())
            {
                let events: Vec<_> = if tc.kind() == TimingControlKind::EventList {
                    tc.as_sem_node()
                        .children()
                        .iter()
                        .filter_map(|n| n.as_timing_control())
                        .collect()
                } else {
                    vec![tc]
                };
                let kids = events
                    .iter()
                    .map(|e| {
                        e.as_sem_node()
                            .children()
                            .iter()
                            .map(|k| k.kind_name().to_string())
                            .collect()
                    })
                    .collect();
                out.push((name.clone(), tc.kind(), kids));
            }
            Walk::Continue
        });
    }
    out
}

#[test]
fn checker_event_formal_keeps_use_site_iff() {
    let s = Session::new();
    let mut comp = Compilation::new(&s).unwrap();
    comp.add_source(SRC).unwrap();
    let d = comp.compile().unwrap();
    assert!(!d.diagnostics().has_errors());
    let top = d.top_instances().next().unwrap().instance_body().unwrap();
    let tcs = timing_controls(top);
    let get = |n: &str| {
        tcs.iter()
            .find(|t| t.0 == n)
            .unwrap_or_else(|| panic!("{n}"))
    };

    // Explicit event actual: the signal event now carries the `iff` (expr + iff).
    let (_, k, ev) = get("c_explicit");
    assert_eq!(*k, TimingControlKind::SignalEvent);
    assert_eq!(ev[0].len(), 2, "expr + iff: {ev:?}");

    // $inferred_clock default: same.
    let (_, _, ev) = get("c_inferred");
    assert_eq!(ev[0].len(), 2, "expr + iff: {ev:?}");

    // The actual already has an iff: the two are conjoined (`z && y`).
    let (_, _, ev) = get("c_both");
    assert_eq!(ev[0].len(), 2);
    assert_eq!(ev[0][1], "BinaryOp", "iff is the conjunction: {ev:?}");

    // Event-list actual: the use-site iff gates every event of the list.
    let (_, k, ev) = get("c_list");
    assert_eq!(*k, TimingControlKind::EventList);
    assert_eq!(ev.len(), 2);
    assert!(ev.iter().all(|e| e.len() == 2), "each event gated: {ev:?}");
}
