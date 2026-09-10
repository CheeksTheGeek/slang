//! Fuzz the custom DataFlow/Lattice bridge on arbitrary input: run a trivial
//! lattice over every procedure. The catch_unwind/poison protocol must keep any
//! failure from crossing the FFI boundary — no panic, no abort.
//! Run with `cargo +nightly fuzz run dataflow`.
#![no_main]
use std::collections::BTreeSet;

use libfuzzer_sys::fuzz_target;
use sv_lang::dataflow::{DfaEvent, DfaEventKind, Lattice};
use sv_lang::kinds::SymbolKind;
use sv_lang::{Compilation, Session};

/// The set of symbols written so far — a minimal reaching-writes lattice.
#[derive(Clone, Default, PartialEq)]
struct Written(BTreeSet<String>);

impl Lattice for Written {
    fn top() -> Self {
        Written::default()
    }
    fn join(&mut self, other: &Self) {
        self.0.extend(other.0.iter().cloned());
    }
    fn meet(&mut self, other: &Self) {
        self.0.retain(|k| other.0.contains(k));
    }
    fn transfer(&mut self, event: DfaEvent<'_>) {
        if event.kind == DfaEventKind::Write
            && let Some(s) = event.symbol
        {
            self.0.insert(s.name().to_string());
        }
    }
}

fuzz_target!(|data: &str| {
    let session = Session::new();
    if let Ok(mut comp) = Compilation::new(&session)
        && comp.add_source(data).is_ok()
        && let Ok(mut design) = comp.compile()
    {
        let eval = design.eval_session();
        // The design borrow (`d`) stays open for as long as the collected
        // procedure handles and the run below use it.
        let d = eval.design();
        let procs: Vec<_> = d
            .top_instances()
            .filter_map(|top| top.instance_body())
            .flat_map(|body| body.members().collect::<Vec<_>>())
            .filter(|m| matches!(m.kind(), SymbolKind::ProceduralBlock | SymbolKind::Subroutine))
            .collect();
        for p in procs {
            let _: Result<Written, _> = eval.run_dataflow(p);
        }
    }
});
