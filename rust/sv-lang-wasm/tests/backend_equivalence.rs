//! Cross-backend equivalence: the same SystemVerilog, elaborated once through
//! the native `sv-lang` backend and once through the `sv-lang-wasm` sandbox,
//! must produce identical observable output — module names, the value symbols
//! and their types, and the reaching-writes set of a procedure. This is the
//! guarantee that the wasm backend is a faithful mirror of the native one, not
//! an independently-drifting reimplementation.

use std::collections::BTreeSet;

/// The comparable shape both backends are reduced to.
#[derive(Debug, PartialEq, Eq)]
struct Fingerprint {
    /// Top-instance (module) names, sorted.
    modules: Vec<String>,
    /// `(name, type-string, bit-width)` for every value member of the first top
    /// module, sorted.
    signals: Vec<(String, String, u64)>,
    /// Names written on the exit path of the module's procedural block, sorted.
    writes: Vec<String>,
}

const SRC: &str = "\
module m(input logic clk, input logic rst);
  logic [7:0] q;
  logic [7:0] next;
  always_ff @(posedge clk) begin
    if (rst) q <= 8'd0;
    else     q <= next;
  end
  assign next = q + 8'd1;
endmodule
";

/// Elaborate through the native backend and take the fingerprint.
fn native_fingerprint(src: &str) -> Fingerprint {
    use sv_lang::dataflow::{DfaEvent, DfaEventKind, Lattice};
    use sv_lang::kinds::SymbolKind;
    use sv_lang::{Compilation, Session};

    // The reaching-writes lattice, matching the wasm `WriteSetLattice`:
    // union on join, intersection on meet, record the written symbol.
    #[derive(Clone, Default)]
    struct Writes(BTreeSet<String>);
    impl Lattice for Writes {
        fn top() -> Self {
            Writes::default()
        }
        fn join(&mut self, o: &Self) {
            self.0.extend(o.0.iter().cloned());
        }
        fn meet(&mut self, o: &Self) {
            self.0.retain(|k| o.0.contains(k));
        }
        fn transfer(&mut self, ev: DfaEvent<'_>) {
            if ev.kind == DfaEventKind::Write
                && let Some(s) = ev.symbol
            {
                self.0.insert(s.name().to_string());
            }
        }
    }

    let session = Session::new();
    let tree = session.parse(src).expect("native parse");
    let mut comp = Compilation::new(&session).expect("native compilation");
    comp.add(&tree).expect("native add");
    let mut design = comp.compile().expect("native compile");

    let (modules, signals) = {
        let d = &design;
        let mut modules: Vec<String> = d.top_instances().map(|s| s.name().to_string()).collect();
        modules.sort();
        let body = d.top_instances().next().unwrap().instance_body().unwrap();
        let mut signals: Vec<(String, String, u64)> = body
            .members()
            .filter_map(|m| {
                m.value_type()
                    .map(|t| (m.name().to_string(), t.to_sv_string(), t.bit_width()))
            })
            .collect();
        signals.sort();
        (modules, signals)
    };

    let dfa = design.eval_session();
    let d = dfa.design();
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .expect("procedural block");
    let exit: Writes = dfa.run_dataflow(block).expect("native dataflow");
    let writes: Vec<String> = exit.0.into_iter().collect();

    Fingerprint {
        modules,
        signals,
        writes,
    }
}

/// Elaborate through the wasm backend and take the fingerprint.
fn wasm_fingerprint(src: &str) -> Fingerprint {
    use sv_lang_wasm::Slang;

    let mut slang = Slang::new().expect("load wasm");
    let tree = slang.parse(src).expect("wasm parse");
    let design = slang.compile(&tree).expect("wasm compile");

    let tops = slang.top_instances(&design).expect("wasm tops");
    let mut modules: Vec<String> = tops.iter().map(|&t| slang.name(t).unwrap()).collect();
    modules.sort();

    let body = slang.instance_body(tops[0]).unwrap().expect("body");
    let members = slang.members(body).expect("members");
    let mut signals = Vec::new();
    for &m in &members {
        if let Some(ty) = slang.value_type(m).unwrap() {
            signals.push((
                slang.name(m).unwrap(),
                slang.type_string(ty).unwrap(),
                slang.type_bit_width(ty).unwrap(),
            ));
        }
    }
    signals.sort();

    let proc = members
        .iter()
        .copied()
        .find(|&m| slang.kind_name(m).unwrap().contains("Procedural"))
        .expect("procedural block");
    let mut writes = slang.reaching_writes(&design, proc).expect("wasm dataflow");
    writes.sort();

    Fingerprint {
        modules,
        signals,
        writes,
    }
}

#[test]
fn native_and_wasm_backends_agree() {
    let native = native_fingerprint(SRC);
    let wasm = wasm_fingerprint(SRC);

    // Sanity: the fingerprint is non-trivial (so an all-empty match can't pass).
    assert_eq!(native.modules, vec!["m".to_string()]);
    assert!(
        native.signals.iter().any(|(n, _, w)| n == "q" && *w == 8),
        "expected 8-bit `q`, got {:?}",
        native.signals
    );
    assert!(
        native.writes.contains(&"q".to_string()),
        "expected `q` written, got {:?}",
        native.writes
    );

    assert_eq!(
        native, wasm,
        "native and wasm backends disagree:\n native = {native:#?}\n wasm  = {wasm:#?}"
    );
}
