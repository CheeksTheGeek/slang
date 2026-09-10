//! Elaboration, symbols, types, and the earned `Sync` of a frozen design.

use std::collections::BTreeSet;

use sv_lang::kinds::SymbolKind;
use sv_lang::{Compilation, DefinitionKind, Design, Session};

const DESIGN: &str = "\
package p;
    localparam int W = 8;
endpackage
module leaf #(parameter int N = 4) (input logic clk);
    logic [N-1:0] q = '0;
endmodule
module top;
    import p::*;
    logic [W-1:0] a;
    leaf #(.N(W)) l1(.clk(1'b0));
    leaf #(.N(2)) l2(.clk(1'b0));
endmodule
";

fn compile(src: &str) -> Design {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source(src).unwrap();
    comp.compile().unwrap()
}

#[test]
fn elaborates_and_freezes() {
    let design = compile(DESIGN);
    let report = design.freeze_report();
    assert!(report.symbols_elaborated > 10);
    assert!(report.expressions_folded > 0);
    assert!(
        report.fully_folded(),
        "fold failures: {}",
        report.fold_failures
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    assert_eq!(design.root().kind(), SymbolKind::Root);
    let tops: Vec<_> = design
        .top_instances()
        .map(|s| s.name().to_string())
        .collect();
    assert_eq!(tops, ["top"]);
    let defs: BTreeSet<_> = design.definitions().map(|s| s.name().to_string()).collect();
    assert_eq!(
        defs,
        BTreeSet::from(["top".to_string(), "leaf".to_string()])
    );
    // `p` plus slang's built-in `std` package.
    let pkgs: BTreeSet<_> = design.packages().map(|s| s.name().to_string()).collect();
    assert!(pkgs.contains("p"));
}

#[test]
fn navigates_symbols_and_types() {
    let design = compile(DESIGN);
    let top = design.top_instances().next().unwrap();
    assert_eq!(top.kind(), SymbolKind::Instance);
    assert_eq!(
        top.instance_definition().unwrap().definition_kind(),
        Some(DefinitionKind::Module)
    );

    let body = top.instance_body().unwrap();
    assert!(body.is_scope());

    // `find` is a pure read: locate the two instances and the variable `a`.
    let l1 = body.find("l1").expect("l1");
    let l2 = body.find("l2").expect("l2");
    assert_ne!(l1, l2);
    // Instance caching is disabled, so the two bodies are distinct symbols.
    assert_ne!(l1.instance_body().unwrap(), l2.instance_body().unwrap());

    let a = body.find("a").expect("a");
    assert!(a.is_value());
    let a_ty = a.value_type().unwrap();
    assert_eq!(a_ty.to_sv_string(), "logic[7:0]");
    assert_eq!(a_ty.bit_width(), 8);
    assert!(a_ty.is_integral() && a_ty.is_four_state() && !a_ty.is_signed());

    // The parameter N of l1 is elaborated to 8 (W), of l2 to 2.
    let n1 = l1.parameters().find(|p| p.name() == "N").unwrap();
    assert_eq!(n1.parameter_value().as_deref(), Some("8"));
    let q1 = l1.instance_body().unwrap().find("q").unwrap();
    assert_eq!(q1.value_type().unwrap().bit_width(), 8);
    let q2 = l2.instance_body().unwrap().find("q").unwrap();
    assert_eq!(q2.value_type().unwrap().bit_width(), 2);

    // The package parameter W folded to 8, readable without evaluating.
    let w = design
        .packages()
        .find(|p| p.name() == "p")
        .unwrap()
        .find("W")
        .unwrap();
    assert_eq!(w.initializer().unwrap().constant().as_deref(), Some("8"));
}

#[test]
fn members_iteration_is_a_pure_read() {
    let design = compile(DESIGN);
    let top = design.top_instances().next().unwrap();
    let body = top.instance_body().unwrap();
    let names: Vec<_> = body.members().map(|m| m.name().to_string()).collect();
    for n in ["a", "l1", "l2"] {
        assert!(names.contains(&n.to_string()), "missing {n} in {names:?}");
    }
}

#[test]
fn lookup_and_eval_take_mut() {
    let mut design = compile(DESIGN);
    let l1 = design.lookup("top.l1").expect("hierarchical lookup");
    assert_eq!(l1.name(), "l1");
    let q = design.lookup("top.l1.q").expect("nested lookup");
    assert!(q.is_value());

    // Evaluate a not-necessarily-cached expression via the exclusive session.
    let eval = design.eval_session();
    let w = eval
        .design()
        .packages()
        .find(|p| p.name() == "p")
        .unwrap()
        .find("W")
        .unwrap();
    assert_eq!(eval.eval(w.initializer().unwrap()).as_deref(), Some("8"));
}

#[test]
fn design_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Design>();

    // Concurrently traverse a frozen design from many threads: pure reads only.
    let design = compile(DESIGN);
    let total: usize = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let d = &design;
                scope.spawn(move || count_symbols(d.root()))
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).sum()
    });
    // Every thread saw the same nonzero symbol count.
    let one = count_symbols(design.root());
    assert!(one > 10);
    assert_eq!(total, one * 8);
}

fn count_symbols(sym: sv_lang::Symbol<'_>) -> usize {
    1 + sym.members().map(count_symbols).sum::<usize>()
}

#[test]
fn analysis_finds_unused_and_drivers() {
    use sv_lang::{AnalysisFlags, DriverKind};

    let design = compile(
        "\
module m(input logic clk, output logic o);
    logic unused_signal;
    logic driven;
    assign o = driven;
    always_ff @(posedge clk) driven <= 1'b1;
endmodule
",
    );
    let analysis = design.analyze(AnalysisFlags::CHECK_UNUSED, 1).unwrap();

    let diags = analysis.diagnostics();
    assert!(
        diags
            .items()
            .iter()
            .any(|d| d.message.contains("unused_signal")),
        "{diags}"
    );

    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let driven = body.find("driven").unwrap();
    let drivers: Vec<_> = analysis.drivers(driven).collect();
    assert!(!drivers.is_empty());
    assert_eq!(drivers[0].kind(), DriverKind::Procedural);

    let o = body.find("o").unwrap();
    let o_drivers: Vec<_> = analysis.drivers(o).collect();
    assert!(o_drivers.iter().any(|d| d.kind() == DriverKind::Continuous));

    // Analysis borrows a Send+Sync design and is itself Send+Sync.
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<sv_lang::Analysis<'_>>();

    // Inspect the analyzed procedures of the module body: a continuous assign
    // and an always_ff block are both analyzed, each with a valid symbol.
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let procs: Vec<_> = analysis.procedures(body).collect();
    assert!(
        procs.len() >= 2,
        "expected the assign + always_ff, got {}",
        procs.len()
    );
    for p in &procs {
        assert!(!p.symbol().name().is_empty() || p.symbol().kind() != SymbolKind::Unknown);
        // has_inferred_clock is queryable (its exact value depends on slang's
        // clock-inference conditions).
        let _ = p.has_inferred_clock();
    }
}

#[test]
fn into_result_reports_errors() {
    let ok = compile("module m; endmodule\n");
    assert!(ok.into_result().is_ok());

    let bad = compile("module m; foo bar(); endmodule\n");
    let err = bad.into_result().unwrap_err();
    assert!(!err.diagnostics.is_empty());
    assert!(err.rendered.contains("error"));
}

#[test]
fn canonical_types_are_race_free_after_freeze() {
    // A typedef's canonical-type memo is filled lazily by a non-atomic write.
    // The freeze sweep must force it; if it did not, first-touching a
    // canonical-routing accessor on a shared &Design from several threads would
    // be a data race (see the TSan-negative gate). Here we assert the concurrent
    // first-touch neither aborts nor deadlocks and every thread agrees.
    let src = "\
module m;
    typedef logic [7:0] byte_t;
    typedef byte_t word_t;
    byte_t a;
    word_t b;
endmodule
";
    let design = compile(src);

    let answers: Vec<Vec<bool>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let d = &design;
                scope.spawn(move || {
                    let mut out = Vec::new();
                    for top in d.top_instances() {
                        let Some(body) = top.instance_body() else {
                            continue;
                        };
                        for m in body.members() {
                            if let Some(t) = m.value_type() {
                                // Both route through getCanonicalType(); the freeze
                                // must have made these pure reads.
                                out.push(t.is_class());
                                out.push(t.canonical().is_integral());
                            }
                        }
                    }
                    out
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    assert!(!answers[0].is_empty(), "no alias-typed members were seen");
    assert!(
        answers.iter().all(|a| *a == answers[0]),
        "threads disagreed on canonical-type reads"
    );
    // The freeze report should record that canonicalization work happened.
    assert!(design.freeze_report().types_canonicalized > 0);
}

#[test]
fn analysis_flags_cover_slangs_full_set() {
    use sv_lang::AnalysisFlags;

    // Every analysis flag slang defines (bits 0..=8), exposed as a Rust const.
    let all = [
        AnalysisFlags::CHECK_UNUSED,
        AnalysisFlags::FULL_CASE_UNIQUE_PRIORITY,
        AnalysisFlags::FULL_CASE_FOUR_STATE,
        AnalysisFlags::ALLOW_MULTI_DRIVEN_LOCALS,
        AnalysisFlags::ALLOW_DUP_INITIAL_DRIVERS,
        AnalysisFlags::CHECK_SHADOW,
        AnalysisFlags::INLINE_CONT_ASSIGN_FUNCTION_READS,
        AnalysisFlags::ALWAYS_STAR_USES_LSPS,
        AnalysisFlags::CONT_ASSIGN_USES_LSPS,
    ];

    // Each is a distinct single bit, and together they tile slang's full mask
    // (`ContAssignUsesLSPs` == 1 << 8, so the union is 0x1FF).
    let mut union = AnalysisFlags::NONE;
    for (i, f) in all.iter().enumerate() {
        assert_eq!(f.bits().count_ones(), 1, "flag {i} is not a single bit");
        assert_eq!(f.bits(), 1 << i, "flag {i} is out of order");
        assert!(!union.contains(*f), "flag {i} duplicates an earlier bit");
        union |= *f;
    }
    assert_eq!(
        union.bits(),
        0x1FF,
        "the flag set does not tile slang's mask"
    );
}

#[test]
fn analysis_honors_a_non_default_flag() {
    use sv_lang::AnalysisFlags;

    // `initial` normally may not co-drive an always_comb signal; the flag lifts
    // that, so the multi-driver diagnostic must appear only without the flag.
    let src = "\
module m;
    logic x;
    always_comb x = 1'b0;
    initial x = 1'b1;
endmodule
";
    let design = compile(src);
    let strict = design.analyze(AnalysisFlags::NONE, 1).unwrap();
    let strict_diags = strict.diagnostics();
    let permissive = design
        .analyze(AnalysisFlags::ALLOW_DUP_INITIAL_DRIVERS, 1)
        .unwrap();
    let permissive_diags = permissive.diagnostics();

    assert!(
        strict_diags.items().len() > permissive_diags.items().len(),
        "the flag should suppress at least one driver diagnostic: strict={strict_diags}, permissive={permissive_diags}"
    );
}

#[test]
fn freeze_forces_port_class_and_nettype_memos() {
    // Exercises the FreezeVisitor's preemptive forcing of the (otherwise
    // B-latent) port / multiport / class / user-net-type / coverage memos:
    // freeze visits each of these constructs and forces its lazy getter, so this
    // must compile (freeze) without crashing, and the design stays readable.
    let design = compile(
        "\
nettype real myreal;
class C;
    int x;
    function int get();
        return x;
    endfunction
endclass
module m(input logic clk, input logic [3:0] a, output logic y);
    myreal w;
    assign y = |a;
    covergroup cg @(posedge clk);
        cp_a: coverpoint a {
            bins lo = {0};
            bins hi = {[1:15]};
        }
    endgroup
    cg cov = new();
endmodule
",
    );
    // The seal held and reads still work through the narrow accessor surface.
    let m = design
        .top_instances()
        .find(|s| s.name() == "m")
        .expect("module m is a top instance");
    let body = m.instance_body().unwrap();
    assert!(body.find("a").is_some(), "port a is visible");
    assert!(body.find("w").is_some(), "net w is visible");
    // The freeze report shows the sweep did real work.
    assert!(design.freeze_report().symbols_elaborated > 0);
}
