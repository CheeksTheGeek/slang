//! Covergroup block events (`@@(begin f or end g)`, LRM 19.3): slang's
//! `BlockEventListControl` has no `visitExprs`, so its events were unreachable
//! through `children()`. Reported by the sv-sim session.

use sv_lang::kinds::{ExpressionKind, TimingControlKind};
use sv_lang::{Compilation, Session};

const SRC: &str = "\
module m;
  bit x;
  task f; endtask
  function void g; endfunction
  covergroup cg @@(begin f or end g or end f);
    coverpoint x;
  endgroup
  covergroup cg_clk @(posedge x);
    coverpoint x;
  endgroup
  cg c1 = new;
  cg_clk c2 = new;
endmodule
";

#[test]
fn block_event_list_exposes_events() {
    let s = Session::new();
    let mut comp = Compilation::new(&s).unwrap();
    comp.add_source(SRC).unwrap();
    let d = comp.compile().unwrap();
    assert!(!d.diagnostics().has_errors());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();

    let ty = body.find("c1").unwrap().value_type().unwrap();
    let tc = ty
        .covergroup_coverage_event()
        .unwrap()
        .as_timing_control()
        .unwrap();
    assert_eq!(tc.kind(), TimingControlKind::BlockEventList);

    let evs = tc.block_events();
    let got: Vec<_> = evs
        .iter()
        .map(|e| {
            let sym = e
                .target
                .symbol_reference(false)
                .unwrap_or_else(|| panic!("target {:?} has no symbol", e.target.kind()));
            (e.is_begin, sym.name().to_string())
        })
        .collect();
    assert_eq!(
        got,
        [
            (true, "f".to_string()),
            (false, "g".to_string()),
            (false, "f".to_string())
        ]
    );

    // The generic walk now sees the same targets.
    let kids = tc.children();
    assert_eq!(kids.len(), 3);
    for (k, e) in kids.iter().zip(&evs) {
        let ke = k.as_expression().unwrap();
        assert_eq!(ke.kind(), e.target.kind());
        assert_ne!(ke.kind(), ExpressionKind::Invalid);
    }

    // Any other kind: empty.
    let ty2 = body.find("c2").unwrap().value_type().unwrap();
    let tc2 = ty2
        .covergroup_coverage_event()
        .unwrap()
        .as_timing_control()
        .unwrap();
    assert_eq!(tc2.kind(), TimingControlKind::SignalEvent);
    assert!(tc2.block_events().is_empty());
}
