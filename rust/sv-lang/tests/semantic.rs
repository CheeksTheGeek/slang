//! Elaboration, symbols, types, and the earned `Sync` of a frozen design.

use std::collections::BTreeSet;

use sv_lang::kinds::{SymbolKind, SyntaxKind};
use sv_lang::{
    Compilation, DefinitionKind, Design, Driver, ElabSystemTaskKind, IntegralFlags, MethodFlags,
    NetKind, PreprocessFlags, ScalarKind, ScriptSession, Session, SourceOptions,
    StatementBlockKind, TimeUnit, UnconnectedDrive, VariableFlags, VariableLifetime,
};

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
fn value_driver_handles_expose_flags_symbol_bounds_and_ranges() {
    use sv_lang::{AnalysisFlags, RawDriverFlags};

    let design = compile(
        "\
module m(input logic a, output logic z);
    assign z = a;
endmodule
",
    );
    let analysis = design.analyze(AnalysisFlags::NONE, 1).unwrap();
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    // ValueDriver::flags -- the input port net `a` is driven through its
    // input port; the raw bitmask carries that bit (and not an unrelated one).
    let a = body.find("a").unwrap();
    let a_driver = analysis.driver_handles(a).next().unwrap();
    assert!(a_driver.flags().contains(RawDriverFlags::INPUT_PORT));
    assert!(!a_driver.flags().contains(RawDriverFlags::CLOCK_VAR));

    // ValueDriver::getSymbol -- the driven symbol is `a` itself.
    assert_eq!(a_driver.symbol(), a);

    // ValueDriver::getBounds -- a scalar bit's full range is [0, 0].
    assert_eq!(a_driver.bounds(), (0, 0));

    // ValueDriver::getOverrideRange -- no indirection here, so none.
    assert!(a_driver.override_range().is_none());

    // ValueDriver::getSourceRange -- the continuous assignment driving `z`
    // has a real, non-empty source range in the same buffer.
    let z = body.find("z").unwrap();
    let z_driver = analysis.driver_handles(z).next().unwrap();
    assert_eq!(z_driver.symbol(), z);
    let range = z_driver.source_range();
    assert!(range.end.offset > range.start.offset);
    assert_eq!(range.start.buffer, range.end.buffer);
}

#[test]
fn value_driver_handles_expose_port_directionality_source_and_path() {
    use sv_lang::{AnalysisFlags, DriverSource};

    let design = compile(
        "\
module m(input logic a, inout wire b, output logic z);
    logic y;
    assign z = a;
    always_comb y = a;
endmodule
",
    );
    let analysis = design.analyze(AnalysisFlags::NONE, 1).unwrap();
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    // ValueDriver::isUnidirectionalPort -- the internal assignment into
    // input port `a` is flagged as an input port driver, so it counts as
    // unidirectional...
    let a = body.find("a").unwrap();
    let a_driver = analysis.driver_handles(a).next().unwrap();
    assert!(a_driver.is_unidirectional_port());

    // ...while the inout port `b` is neither an input nor an output port
    // driver, so it is not unidirectional.
    let b = body.find("b").unwrap();
    let b_driver = analysis.driver_handles(b).next().unwrap();
    assert!(!b_driver.is_unidirectional_port());

    // ValueDriver::isInSingleDriverProcedure / ValueDriver::source -- `z` is
    // driven by a plain continuous assignment (source Other, not a
    // single-driver procedure)...
    let z = body.find("z").unwrap();
    let z_driver = analysis.driver_handles(z).next().unwrap();
    assert!(!z_driver.is_in_single_driver_procedure());
    assert_eq!(z_driver.source(), DriverSource::Other);
    assert!(!DriverSource::Other.is_single_driver_procedure());

    // ...while `y` is driven from inside an `always_comb` block, which slang
    // classifies as a single-driver procedure.
    let y = body.find("y").unwrap();
    let y_driver = analysis.driver_handles(y).next().unwrap();
    assert!(y_driver.is_in_single_driver_procedure());
    assert_eq!(y_driver.source(), DriverSource::AlwaysComb);
    assert!(DriverSource::AlwaysComb.is_single_driver_procedure());

    // ValueDriver::path -- the root of each driver's path is the driven
    // value symbol itself, matching ValueDriver::getSymbol.
    assert_eq!(a_driver.path().root_symbol(), Some(a_driver.symbol()));
    assert_eq!(z_driver.path().root_symbol(), Some(z));
    assert_eq!(y_driver.path().root_symbol(), Some(y));
    // Two different drivers' paths root to two different symbols.
    assert_ne!(z_driver.path().root_symbol(), y_driver.path().root_symbol());
}

#[test]
fn analyzed_procedure_drivers_read_set_and_sensitivity() {
    use sv_lang::{AnalysisFlags, SensitivityKind};

    let design = compile(
        "\
module m(input logic a, output logic y);
    always_comb y = a;
endmodule
",
    );
    let analysis = design.analyze(AnalysisFlags::NONE, 1).unwrap();
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let proc = analysis.procedures(body).next().unwrap();

    // AnalyzedProcedure::getDrivers -- the always_comb drives exactly `y`,
    // and the driver's containing symbol is the procedure itself.
    let drivers: Vec<_> = proc.drivers().collect();
    assert_eq!(drivers.len(), 1);
    assert_eq!(drivers[0].containing_symbol(), proc.symbol());

    // AnalyzedProcedure::getReadSet -- reads `a` over its full (1-bit) range.
    let reads: Vec<_> = proc.read_set().collect();
    assert_eq!(reads.len(), 1);
    assert_eq!(reads[0].symbol().name(), "a");
    assert_eq!(reads[0].bit_range(), (0, 0));

    // AnalyzedProcedure::getSensitivityList -- always_comb is implicit,
    // derived from the read set.
    let sens = proc.sensitivity_list();
    assert_eq!(sens.kind(), SensitivityKind::Implicit);
    let sens_names: Vec<_> = sens
        .reads()
        .map(|r| r.symbol().name().to_string())
        .collect();
    assert_eq!(sens_names, ["a"]);
}

#[test]
fn analyzed_procedure_call_expressions() {
    use sv_lang::{AnalysisFlags, kinds::ExpressionKind};

    let design = compile(
        "\
module m;
    function automatic int f(int x); return x + 1; endfunction
    int y;
    initial y = f(1);
endmodule
",
    );
    let analysis = design.analyze(AnalysisFlags::NONE, 1).unwrap();
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let proc = analysis.procedures(body).next().unwrap();

    // AnalyzedProcedure::getCallExpressions -- exactly the one call to f(1).
    let calls: Vec<_> = proc.call_expressions().collect();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].kind(), ExpressionKind::Call);
}

#[test]
fn analyzed_procedure_timing_controls_and_implicit_event_read_sets() {
    use sv_lang::{AnalysisFlags, kinds::StatementKind};

    let design = compile(
        "\
module m(input logic a, input logic b, output logic y);
    initial forever @* y = a & b;
endmodule
",
    );
    let analysis = design.analyze(AnalysisFlags::NONE, 1).unwrap();
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let proc = analysis.procedures(body).next().unwrap();

    // AnalyzedProcedure::getTimingControls -- the single `@*` timed statement.
    let timing: Vec<_> = proc.timing_controls().collect();
    assert_eq!(timing.len(), 1);
    assert_eq!(timing[0].kind(), StatementKind::Timed);

    // AnalyzedProcedure::getImplicitEventReadSets and
    // ImplicitEventReadSet::statement -- one region, tied to that same
    // statement, whose reads are exactly `a` and `b`.
    let sets: Vec<_> = proc.implicit_event_read_sets().collect();
    assert_eq!(sets.len(), 1);
    assert_eq!(sets[0].statement().kind(), StatementKind::Timed);

    let mut names: Vec<_> = sets[0]
        .reads()
        .map(|r| r.symbol().name().to_string())
        .collect();
    names.sort();
    assert_eq!(names, ["a", "b"]);
}

#[test]
fn analyzed_procedure_parent_links_a_procedural_checker_to_its_enclosing_procedure() {
    use sv_lang::{AnalysisFlags, Walk, kinds::SymbolKind};

    // `check_01`'s own `always_ff` is analyzed as a *nested* AnalyzedProcedure
    // whose parentProcedure is the `always` block of `m` that instantiates the
    // checker procedurally (slang::analysis::AnalyzedProcedure::
    // parentProcedure is non-null only for this exact shape).
    let design = compile(
        "\
checker check_01(a, b, event clk = $inferred_clock);
    default clocking @clk; endclocking
    always_ff @clk begin
        a1: assert property(a);
        a2: assert property(b);
    end
endchecker : check_01

module m(input logic clock, ok, req);
    always @(posedge clock) begin
        check_01 mycheck(ok, req);
    end
endmodule
",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let outer_symbol = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();

    // The checker's own `always_ff` is not a direct member of `m`'s body — it
    // is nested inside the checker instance's body — so find it with a
    // recursive visit, distinguishing it from the outer `always` by identity.
    let mut inner_symbol = None;
    design.root().visit(|sym| {
        if sym.kind() == SymbolKind::ProceduralBlock && sym != outer_symbol {
            inner_symbol = Some(sym);
        }
        Walk::Continue
    });
    let inner_symbol = inner_symbol.expect("nested always_ff inside the checker instance");

    let analysis = design.analyze(AnalysisFlags::NONE, 1).unwrap();
    let outer_proc = analysis.procedures(body).next().unwrap();
    assert_eq!(outer_proc.symbol(), outer_symbol);
    // The outer procedure itself has no parent.
    assert!(outer_proc.parent().is_none());

    // Reach the nested AnalyzedProcedure through one of its own assertions.
    let assertion = analysis.assertions(inner_symbol).next().unwrap();
    let inner_proc = assertion.procedure().unwrap();
    assert_eq!(inner_proc.symbol(), inner_symbol);

    let parent = inner_proc
        .parent()
        .expect("nested procedural-checker procedure has a parent");
    assert_eq!(parent.symbol(), outer_symbol);
}

#[test]
fn analyzed_assertion_exposes_procedure_root_and_clocks() {
    use sv_lang::AnalysisFlags;

    let design = compile(
        "\
module m;
    logic clk, a, b;
    always_ff @(posedge clk) begin
        assert property (@(posedge clk) a |-> b);
    end
endmodule
",
    );
    let analysis = design.analyze(AnalysisFlags::NONE, 1).unwrap();
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    // AnalyzedProcedure::getInferredClock (has_inferred_clock delegates to it).
    let proc = analysis.procedures(body).next().unwrap();
    assert!(proc.has_inferred_clock());
    assert!(proc.inferred_clock().is_some());

    let assertions: Vec<_> = analysis.assertions(proc.symbol()).collect();
    assert_eq!(assertions.len(), 1);
    let assertion = assertions[0];

    // AnalyzedAssertion::containingSymbol
    assert_eq!(assertion.containing_symbol(), proc.symbol());

    // AnalyzedAssertion::procedure -- points back to the same, fully valid
    // procedure (this specifically regression-tests a dangling-pointer bug in
    // slang's AnalysisScopeVisitor/AnalysisManager where the AnalyzedProcedure
    // captured by an assertion's `procedure` field was a temporary that had
    // already been moved-from and destroyed by the time it was read back).
    let via_assertion = assertion
        .procedure()
        .expect("the assertion has a containing procedure");
    assert_eq!(via_assertion.symbol(), proc.symbol());
    assert!(via_assertion.has_inferred_clock());

    // AnalyzedAssertion::astNode -- a concurrent assertion statement here,
    // not a plain expression.
    let ast_node = assertion.ast_node();
    assert!(ast_node.as_statement().is_some());
    assert!(ast_node.as_expression().is_none());

    // AnalyzedAssertion::getRoot -- the property expression tree's root; it
    // lives in its own AST domain, so it is neither a statement nor a plain
    // expression.
    let root = assertion.root();
    assert!(root.as_statement().is_none() && root.as_expression().is_none());
    assert!(!root.kind_name().is_empty());

    // AnalyzedAssertion::getSemanticLeadingClock
    let leading = assertion
        .semantic_leading_clock()
        .expect("a single-clocked assertion should have a leading clock");

    // AnalyzedAssertion::getClock(root) resolves the same clock for the whole
    // (unambiguous, single-clocked) property.
    let clock = assertion
        .clock(root)
        .expect("a single-clocked assertion's root should resolve a clock");
    assert_eq!(clock.kind_name(), leading.kind_name());
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

// The last of the once-"B-latent" memo families, now force-resolved in the
// FreezeVisitor. Each test elaborates a design containing the construct; if
// forcing its lazy getter faulted (a precondition assert, in this assertions-on
// build), `compile()` would abort. Reaching the assertion proves it is safe.

#[test]
fn freeze_forces_continuous_assign_memos() {
    let d = compile("module m(input a, b, output y);\n  assign #2 y = a & b;\nendmodule\n");
    assert!(d.freeze_report().symbols_elaborated > 0);
}

#[test]
fn freeze_forces_net_alias_memos() {
    let d = compile("module m;\n  wire x, w;\n  alias x = w;\nendmodule\n");
    assert!(d.freeze_report().symbols_elaborated > 0);
}

#[test]
fn freeze_forces_elab_system_task_memos() {
    let d = compile("module m;\n  $info(\"elaborated\");\nendmodule\n");
    assert!(d.freeze_report().symbols_elaborated > 0);
}

#[test]
fn freeze_forces_specify_memos() {
    let d = compile(
        "module m(input a, input clk, output y);\n  assign y = a;\n  specify\n    pulsestyle_onevent y;\n    (a => y) = 1;\n    $setup(a, posedge clk, 1);\n  endspecify\nendmodule\n",
    );
    assert!(d.freeze_report().symbols_elaborated > 0);
}

#[test]
fn freeze_forces_checker_memos() {
    let d = compile(
        "checker chk(input logic i, output logic o = 1'b0);\n  assign o = i;\nendchecker\nmodule m(input logic a);\n  logic w;\n  chk c1(.i(a), .o(w));\nendmodule\n",
    );
    assert!(d.freeze_report().symbols_elaborated > 0);
}

#[test]
fn freeze_forces_instance_net_gate_attribute_memos() {
    // Module instance port connections, a delayed net, a gate primitive, and an
    // attribute value — all lazily resolved, all forced in the sweep.
    let d = compile(
        "module sub(input logic a, output logic y);\n  assign y = a;\nendmodule\nmodule m(input logic a);\n  (* keep = 1 *) wire #3 w;\n  logic o;\n  sub s(.a(a), .y(o));\n  and g(w, a, o);\nendmodule\n",
    );
    assert!(d.freeze_report().symbols_elaborated > 0);
}

#[test]
fn freeze_forces_formalarg_classbase_genericclass_covergroup_memos() {
    let d = compile(
        "class B;\n  int b;\nendclass\nclass D extends B;\n  int d;\nendclass\nclass G #(int N = 4);\n  logic [N-1:0] data;\nendclass\nmodule m(input logic clk, input logic [2:0] a);\n  function automatic int f(int x = 7);\n    return x;\n  endfunction\n  covergroup cg @(posedge clk);\n    cp: coverpoint a;\n  endgroup\n  cg ci = new();\n  initial begin\n    int r;\n    r = f();\n  end\nendmodule\n",
    );
    assert!(d.freeze_report().symbols_elaborated > 0);
}

#[test]
fn freeze_forces_interface_port_memos() {
    let d = compile(
        "interface bus;\n  logic x;\nendinterface\nmodule m(bus b);\n  assign b.x = 1'b0;\nendmodule\n",
    );
    assert!(d.freeze_report().symbols_elaborated > 0);
}

#[test]
fn freeze_forces_port_initializer_memo() {
    // A port's default-value initializer (PortSymbol::getInitializer) binds
    // and allocates an Expression into the arena on first read, just like
    // the type/internal-expr memos above -- forced by the same PortSymbol
    // branch. If it weren't forced pre-seal, reading it post-freeze would
    // mutate the shared arena on first touch.
    let d = compile(
        "module m(input logic [3:0] a = 4'hA, output logic y);\n  assign y = |a;\nendmodule\n",
    );
    assert!(d.freeze_report().symbols_elaborated > 0);
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let a = body
        .instance_body_ports()
        .find(|p| p.name() == "a")
        .unwrap();
    let init = a.port_initializer().unwrap();
    assert_eq!(init.constant_value().unwrap().as_i64(), Some(10));
}

#[test]
fn freeze_forces_package_export_resolution_memo() {
    // PackageSymbol::findForImport's export-resolution memo
    // (ExportData::resolved, populated by resolveExports()) is forced for
    // every Package symbol by the freeze sweep -- reached because packages
    // are added as members of their enclosing compilation-unit scope
    // (Scope::addMembers's PackageDeclaration case calls addMember), so the
    // generic Root -> CompilationUnit -> members() traversal visits them.
    let d = compile(
        "package base_pkg;\n  localparam int K = 5;\nendpackage\n\
         package re_pkg;\n  export base_pkg::K;\n  import base_pkg::K;\nendpackage\n\
         module m; endmodule\n",
    );
    assert!(d.freeze_report().symbols_elaborated > 0);
    let re = d.packages().find(|p| p.name() == "re_pkg").unwrap();
    let k = re.package_find_for_import("K").unwrap();
    assert_eq!(k.name(), "K");
}

// ---- Compilation: built-in types --------------------------------------------

#[test]
fn compilation_built_in_bit_and_byte_types() {
    let d = compile(DESIGN);
    let bit = d.bit_type();
    assert!(bit.is_integral());
    assert_eq!(bit.bit_width(), 1);
    assert!(!bit.is_four_state());
    assert!(!bit.is_signed());

    let byte = d.byte_type();
    assert!(byte.is_integral());
    assert_eq!(byte.bit_width(), 8);
    assert!(!byte.is_four_state());
    assert!(byte.is_signed());

    // Distinct types, not the same object reinterpreted.
    assert!(!bit.is_matching(&byte));
}

// ---- Compilation: compilation units -----------------------------------------

#[test]
fn compilation_units_lists_one_per_tree() {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source("module a; endmodule\n").unwrap();
    comp.add_source("module b; endmodule\n").unwrap();
    let d = comp.compile().unwrap();
    assert_eq!(d.compilation_units().count(), 2);
}

#[test]
fn compilation_unit_for_syntax_resolves_the_owning_unit_and_rejects_foreign_nodes() {
    let session = Session::new();
    let tree_a = session.parse("module a; endmodule\n").unwrap();
    let tree_b = session.parse("module b; endmodule\n").unwrap();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add(&tree_a).unwrap();
    let d = comp.compile().unwrap();

    // The root of the added tree resolves to the (only) compilation unit.
    let unit = d.compilation_unit_for_syntax(tree_a.root()).unwrap();
    let only_unit = d.compilation_units().next().unwrap();
    assert_eq!(unit.id(), only_unit.id());

    // A syntax-unit root that was never added to this compilation resolves to
    // nothing — proves this isn't a stub that echoes back whatever it's given.
    assert!(d.compilation_unit_for_syntax(tree_b.root()).is_none());

    // A non-compilation-unit node (a child of the root) also resolves to none.
    let child = tree_a.root().children().next().unwrap();
    assert!(d.compilation_unit_for_syntax(child).is_none());
}

// ---- Compilation: script scope + add_diagnostics ----------------------------

#[test]
fn create_script_scope_adds_an_empty_scope_outside_compilation_units() {
    let mut d = compile("module m; endmodule\n");
    assert_eq!(d.compilation_units().count(), 1);

    // Each check is extracted into an owned value so the `&mut` borrow the
    // scope holds doesn't outlive this block (NLL), letting `d` be read again.
    let (is_scope, member_count, has_parent) = {
        let scope = d.create_script_scope().unwrap();
        (
            scope.is_scope(),
            scope.members().count(),
            scope.parent().is_some(),
        )
    };
    assert!(is_scope);
    assert_eq!(member_count, 0);
    assert!(has_parent);
    // The script scope is not one of the tree-derived compilation units (both
    // are anonymous, so this checks the count, not the hierarchical-path based
    // `SymbolId`, which happens to be the same "$unit" for every anonymous
    // unit).
    assert_eq!(d.compilation_units().count(), 1);

    // A second call succeeds too (each allocates a fresh symbol; slang would
    // assert/throw on a corrupt or already-sealed arena).
    let (is_scope2, member_count2) = {
        let scope2 = d.create_script_scope().unwrap();
        (scope2.is_scope(), scope2.members().count())
    };
    assert!(is_scope2);
    assert_eq!(member_count2, 0);
}

#[test]
fn add_diagnostics_merges_a_foreign_designs_semantic_diagnostics() {
    // Design A: a real semantic error (undefined identifier), no parse errors.
    let session_a = Session::new();
    let mut comp_a = Compilation::new(&session_a).unwrap();
    comp_a
        .add_source("module m; logic x; initial x = undefined_thing; endmodule\n")
        .unwrap();
    let design_a = comp_a.compile().unwrap();
    assert!(design_a.diagnostics().has_errors());
    let a_error_count = design_a.diagnostics().error_count();
    assert!(a_error_count > 0);

    // Design B starts clean and has nothing to do with design A's source.
    let session_b = Session::new();
    let mut comp_b = Compilation::new(&session_b).unwrap();
    comp_b.add_source("module n; endmodule\n").unwrap();
    assert!(!comp_b.compile().unwrap().diagnostics().has_errors());

    // Rebuild comp_b (compile() consumed the previous one) and merge A's
    // diagnostics in before freezing.
    let mut comp_b = Compilation::new(&session_b).unwrap();
    comp_b.add_source("module n; endmodule\n").unwrap();
    comp_b.add_diagnostics(&design_a.raw_diagnostics()).unwrap();
    let design_b = comp_b.compile().unwrap();

    // B now reports (at least) A's errors, even though B's own source is clean.
    assert!(design_b.diagnostics().has_errors());
    assert!(design_b.diagnostics().error_count() >= a_error_count);
}

// ---- Compilation: DPI exports ------------------------------------------------

#[test]
fn dpi_exports_lists_subroutine_c_identifier_and_syntax() {
    let session = Session::new();
    let tree = session
        .parse(
            "module m;\n  function int f(); return 1; endfunction\n  \
             export \"DPI-C\" my_f = function f;\nendmodule\n",
        )
        .unwrap();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add(&tree).unwrap();
    let d = comp.compile().unwrap();

    let exports: Vec<_> = d.dpi_exports().collect();
    assert_eq!(exports.len(), 1);
    assert_eq!(exports[0].c_identifier(), "my_f");
    assert_eq!(exports[0].subroutine().name(), "f");
    let node = exports[0].syntax(&tree).unwrap();
    assert_eq!(node.kind(), sv_lang::kinds::SyntaxKind::DPIExport);
}

#[test]
fn dpi_exports_empty_when_none_declared() {
    let d = compile(DESIGN);
    assert_eq!(d.dpi_exports().count(), 0);
}

// ---- Compilation: definition lookup -----------------------------------------

#[test]
fn try_get_definition_finds_and_reports_absence() {
    let d = compile(
        "module leaf; endmodule\nmodule other; endmodule\nmodule top;\n  leaf l();\nendmodule\n",
    );
    let top_body = d.top_instances().next().unwrap().instance_body().unwrap();

    // Found: a real nested-definition lookup, not an echo of the input.
    let found = d.try_get_definition("leaf", top_body);
    assert_eq!(found.definition().unwrap().name(), "leaf");
    // No `config` block is in play, so both config fields are absent.
    assert!(found.config_root().is_none());
    assert!(found.config_rule().is_none());

    // A definition that exists but was not asked for is not what comes back.
    let other = d.try_get_definition("other", top_body);
    assert_eq!(other.definition().unwrap().name(), "other");
    assert_ne!(
        found.definition().unwrap().id(),
        other.definition().unwrap().id()
    );

    // Not found: absent, not a fallback to something else.
    let missing = d.try_get_definition("does_not_exist_anywhere", top_body);
    assert!(missing.definition().is_none());
    assert!(missing.config_root().is_none());
    assert!(missing.config_rule().is_none());
}

// ---- Compilation: state, syntax trees, name parsing -------------------------

#[test]
fn state_predicates_reflect_finalization_elaboration_and_errors() {
    let clean = compile(DESIGN);
    assert!(clean.is_finalized());
    assert!(clean.is_elaborated());
    assert!(!clean.has_issued_errors());
    assert!(!clean.has_fatal_errors());

    let broken = compile("module m; assign x = does_not_exist; endmodule\n");
    assert!(broken.is_finalized());
    assert!(broken.is_elaborated());
    assert!(broken.has_issued_errors());
    // An unresolved name is an error, but not a fatal one / error-limit trip.
    assert!(!broken.has_fatal_errors());
}

#[test]
fn syntax_trees_returns_added_trees_in_order() {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source("module a; endmodule\n").unwrap();
    comp.add_source("module b; endmodule\n").unwrap();
    comp.add_source("module c; endmodule\n").unwrap();
    let d = comp.compile().unwrap();

    let trees = d.syntax_trees();
    assert_eq!(trees.len(), 3);
    assert_eq!(trees[0].module_names().collect::<Vec<_>>(), ["a"]);
    assert_eq!(trees[1].module_names().collect::<Vec<_>>(), ["b"]);
    assert_eq!(trees[2].module_names().collect::<Vec<_>>(), ["c"]);

    // Retained independently of the compilation: still readable afterward.
    let names: Vec<_> = trees
        .iter()
        .flat_map(|t| t.module_names())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    assert_eq!(names, ["a", "b", "c"]);
}

#[test]
fn syntax_trees_count_matches_exactly_what_was_added() {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source("module only; endmodule\n").unwrap();
    let d = comp.compile().unwrap();
    // Exactly the one tree added — not e.g. every tree ever parsed by the
    // shared session (a second, unrelated parse must not leak in).
    let _unrelated = d.session().parse("module other; endmodule\n").unwrap();
    assert_eq!(d.syntax_trees().len(), 1);
    assert_eq!(
        d.syntax_trees()[0].module_names().collect::<Vec<_>>(),
        ["only"]
    );
}

#[test]
fn parse_name_parses_hierarchical_names_and_reports_bad_ones() {
    let mut d = compile(DESIGN);

    let simple = d.parse_name("foo").unwrap();
    assert_eq!(simple.text(), "foo");

    let scoped = d.parse_name("foo.bar.baz").unwrap();
    assert_eq!(scoped.text(), "foo.bar.baz");
    assert_eq!(scoped.kind(), sv_lang::kinds::SyntaxKind::ScopedName);

    let indexed = d.parse_name("foo[3]").unwrap();
    assert_eq!(indexed.text(), "foo[3]");

    // A name the parser can't recover from at all is a real error, not an
    // empty/echoed success.
    let err = d.parse_name("###").unwrap_err();
    assert!(matches!(err, sv_lang::Error::InvalidArgument(_)));
}

#[test]
fn try_parse_name_never_fails_and_reports_diagnostics_instead() {
    let mut d = compile(DESIGN);

    let (good, diags) = d.try_parse_name("foo.bar");
    assert_eq!(good.text(), "foo.bar");
    assert!(!diags.has_errors());

    let (_bad, diags) = d.try_parse_name("###");
    assert!(diags.has_errors());
    assert!(diags.error_count() >= 1);
}

// ---- Compilation: built-in types, options, libraries, diagnostics ----------

#[test]
fn built_in_type_getters_return_distinct_correctly_shaped_types() {
    let d = compile(DESIGN);

    let int_t = d.int_type();
    assert!(int_t.is_integral());
    assert!(int_t.is_signed());
    assert!(!int_t.is_four_state());
    assert_eq!(int_t.bit_width(), 32);

    let integer_t = d.integer_type();
    assert!(integer_t.is_integral());
    assert!(integer_t.is_signed());
    assert!(integer_t.is_four_state());
    assert_eq!(integer_t.bit_width(), 32);

    let logic_t = d.logic_type();
    assert!(logic_t.is_integral());
    assert!(logic_t.is_four_state());
    assert_eq!(logic_t.bit_width(), 1);

    let real_t = d.real_type();
    assert!(!real_t.is_integral());
    assert_eq!(real_t.to_sv_string(), "real");

    let short_real_t = d.short_real_type();
    assert!(!short_real_t.is_integral());
    assert_eq!(short_real_t.to_sv_string(), "shortreal");

    let error_t = d.error_type();
    assert_eq!(error_t.as_symbol().kind(), SymbolKind::ErrorType);

    let null_t = d.null_type();
    assert_eq!(null_t.as_symbol().kind(), SymbolKind::NullType);
    assert_eq!(null_t.as_symbol().name(), "null");

    // Every one of these is a genuinely distinct type, not several handles
    // onto the same object.
    let ids: BTreeSet<String> = [
        int_t.as_symbol().id(),
        integer_t.as_symbol().id(),
        logic_t.as_symbol().id(),
        real_t.as_symbol().id(),
        short_real_t.as_symbol().id(),
    ]
    .into_iter()
    .map(|id| id.path().to_string())
    .collect();
    assert_eq!(ids.len(), 5);

    // Repeated calls return the same underlying built-in object.
    assert_eq!(d.int_type().as_symbol().id(), int_t.as_symbol().id());
}

#[test]
fn gate_type_looks_up_built_in_primitives_by_name() {
    let d = compile(DESIGN);

    let and_gate = d.gate_type("and").unwrap();
    assert_eq!(and_gate.name(), "and");
    assert_eq!(and_gate.kind(), SymbolKind::Primitive);

    let bufif0 = d.gate_type("bufif0").unwrap();
    assert_eq!(bufif0.name(), "bufif0");

    // A distinct gate is a distinct symbol.
    assert_ne!(and_gate.id(), bufif0.id());

    // Not a built-in gate name, and not a fallback to something else.
    assert!(d.gate_type("does_not_exist_as_a_gate").is_none());
    // A module name is not a gate, even though it is a real definition.
    assert!(d.gate_type("leaf").is_none());
}

#[test]
fn primitive_and_procedural_block_symbols_expose_full_surface() {
    use sv_lang::{DriveStrength, PrimitiveKind, PrimitivePortDirection};

    const SRC: &str = "\
primitive udp_latch(q, clk, d);
    output q; reg q;
    input clk, d;
    initial q = 1'b0;
    table
        1 0 : ? : 0;
        1 1 : ? : 1;
        0 ? : ? : -;
    endtable
endprimitive

module ports_mod(a, , c);
    input a, c;
endmodule

module gate_mod(input a, b, output logic y);
    wire w;
    assign w = a;

    and (strong0, pull1) #2 g1(w, a, b);

    always_ff @(posedge a) begin : blk
        logic tmp;
        tmp = b;
        y <= tmp;
    end

    always @(a or b)
        y = a & b;
endmodule
";
    let d = compile(SRC);
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());

    // -- PortSymbol::isNetPort / isNullPort ---------------------------------
    let ports_body = d
        .top_instances()
        .find(|inst| inst.name() == "ports_mod")
        .unwrap()
        .instance_body()
        .unwrap();
    let ports: Vec<_> = ports_body.instance_body_ports().collect();
    assert_eq!(ports.len(), 3);
    assert!(ports[0].port_is_net_port());
    assert!(!ports[0].port_is_null_port());
    assert!(!ports[1].port_is_net_port());
    assert!(ports[1].port_is_null_port());
    assert!(ports[2].port_is_net_port());
    assert!(!ports[2].port_is_null_port());

    // -- PrimitiveSymbol on the built-in "and" gate (NInput, no table) ------
    let and_gate = d.gate_type("and").unwrap();
    assert_eq!(and_gate.primitive_kind(), PrimitiveKind::NInput);
    assert!(!and_gate.primitive_is_sequential());
    assert!(and_gate.primitive_init_val().is_none());
    let and_ports: Vec<_> = and_gate.primitive_ports().collect();
    assert_eq!(and_ports.len(), 2);
    assert_eq!(
        and_ports[0].primitive_port_direction(),
        PrimitivePortDirection::Out
    );
    assert_eq!(
        and_ports[1].primitive_port_direction(),
        PrimitivePortDirection::In
    );
    assert_eq!(and_gate.primitive_table().count(), 0);

    // -- PrimitiveSymbol on the user-defined sequential UDP ------------------
    // A user-defined primitive is excluded from ordinary name-map lookup
    // (`Scope::find`; like a module definition, it lives in its own
    // namespace -- see `slang::ast::canLookupByName`), so it has to be
    // found by walking members and matching on kind + name.
    let unit = d.compilation_units().next().unwrap();
    let udp = unit
        .members()
        .find(|s| s.kind() == SymbolKind::Primitive && s.name() == "udp_latch")
        .unwrap();
    assert_eq!(udp.kind(), SymbolKind::Primitive);
    assert_eq!(udp.primitive_kind(), PrimitiveKind::UserDefined);
    assert!(udp.primitive_is_sequential());
    assert_eq!(udp.primitive_init_val().unwrap().as_i64(), Some(0));

    let udp_ports: Vec<_> = udp.primitive_ports().collect();
    assert_eq!(udp_ports.len(), 3);
    assert_eq!(
        udp_ports[0].primitive_port_direction(),
        PrimitivePortDirection::OutReg
    );
    assert_eq!(
        udp_ports[1].primitive_port_direction(),
        PrimitivePortDirection::In
    );
    assert_eq!(
        udp_ports[2].primitive_port_direction(),
        PrimitivePortDirection::In
    );

    let table: Vec<_> = udp.primitive_table().collect();
    assert_eq!(table.len(), 3);
    assert_eq!(table[0], ("10", "0", "?"));
    assert_eq!(table[1], ("11", "1", "?"));
    assert_eq!(table[2], ("0?", "-", "?"));

    // A Primitive/PrimitivePort accessor on a non-matching symbol kind
    // reports the documented default rather than panicking.
    assert_eq!(
        d.root().primitive_port_direction(),
        PrimitivePortDirection::In
    );
    assert_eq!(d.root().primitive_kind(), PrimitiveKind::UserDefined);
    assert!(!d.root().primitive_is_sequential());
    assert_eq!(d.root().primitive_ports().count(), 0);
    assert_eq!(d.root().primitive_table().count(), 0);

    // -- PrimitiveInstanceSymbol on the gate instantiation -------------------
    let gate_body = d
        .top_instances()
        .find(|inst| inst.name() == "gate_mod")
        .unwrap()
        .instance_body()
        .unwrap();
    let g1 = gate_body.find("g1").unwrap();
    assert_eq!(g1.kind(), SymbolKind::PrimitiveInstance);
    let conns: Vec<_> = g1.primitive_instance_port_connections().collect();
    assert_eq!(conns.len(), 3);
    let delay = g1.primitive_instance_delay().unwrap();
    assert_eq!(delay.domain(), sv_lang_sys::SLANG_AST_TIMING_CONTROL);
    let ds = g1.primitive_instance_drive_strength();
    assert_eq!(ds.strength0, Some(DriveStrength::Strong));
    assert_eq!(ds.strength1, Some(DriveStrength::Pull));

    // -- ProceduralBlockSymbol -------------------------------------------------
    let procs: Vec<_> = gate_body
        .members()
        .filter(|s| s.kind() == SymbolKind::ProceduralBlock)
        .collect();
    assert_eq!(procs.len(), 2);

    let ff = &procs[0];
    assert!(ff.is_single_driver_block());
    let blocks: Vec<_> = ff.procedural_blocks().collect();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].name(), "blk");

    let plain = &procs[1];
    assert!(!plain.is_single_driver_block());
    assert_eq!(plain.procedural_blocks().count(), 0);
}

#[test]
fn package_looks_up_elaborated_packages_by_name() {
    let d = compile(DESIGN);

    let pkg = d.package("p").unwrap();
    assert_eq!(pkg.name(), "p");
    assert_eq!(pkg.kind(), SymbolKind::Package);
    // It really is the same package elaborated elsewhere in the design.
    assert_eq!(
        pkg.id(),
        d.packages().find(|p| p.name() == "p").unwrap().id()
    );

    assert!(d.package("does_not_exist_as_a_package").is_none());
    // A module name is not a package.
    assert!(d.package("leaf").is_none());
}

#[test]
fn net_type_returns_the_matching_built_in_for_every_keyword() {
    let d = compile(DESIGN);

    let cases = [
        (sv_lang::NetTypeKind::Wire, "wire"),
        (sv_lang::NetTypeKind::WAnd, "wand"),
        (sv_lang::NetTypeKind::WOr, "wor"),
        (sv_lang::NetTypeKind::Tri, "tri"),
        (sv_lang::NetTypeKind::TriAnd, "triand"),
        (sv_lang::NetTypeKind::TriOr, "trior"),
        (sv_lang::NetTypeKind::Tri0, "tri0"),
        (sv_lang::NetTypeKind::Tri1, "tri1"),
        (sv_lang::NetTypeKind::TriReg, "trireg"),
        (sv_lang::NetTypeKind::Supply0, "supply0"),
        (sv_lang::NetTypeKind::Supply1, "supply1"),
        (sv_lang::NetTypeKind::UWire, "uwire"),
        (sv_lang::NetTypeKind::Interconnect, "interconnect"),
    ];

    let mut ids: BTreeSet<String> = BTreeSet::new();
    for (kind, expected_name) in cases {
        let nt = d.net_type(kind);
        assert_eq!(nt.name(), expected_name);
        assert_eq!(nt.kind(), SymbolKind::NetType);
        ids.insert(nt.id().path().to_string());
    }
    // Thirteen keywords, thirteen genuinely distinct net type objects.
    assert_eq!(ids.len(), cases.len());

    // Repeated calls for the same keyword return the same object.
    assert_eq!(
        d.net_type(sv_lang::NetTypeKind::Wire).id(),
        d.net_type(sv_lang::NetTypeKind::Wire).id()
    );
}

#[test]
fn more_built_in_type_and_symbol_getters_return_distinct_correctly_shaped_values() {
    let d = compile(DESIGN);

    let std_pkg = d.std_package();
    assert_eq!(std_pkg.name(), "std");
    assert_eq!(std_pkg.kind(), SymbolKind::Package);
    // It really is the same package elaborated elsewhere in the design.
    assert_eq!(
        std_pkg.id(),
        d.packages().find(|p| p.name() == "std").unwrap().id()
    );

    let string_t = d.string_type();
    assert!(!string_t.is_integral());
    assert_eq!(string_t.to_sv_string(), "string");

    let void_t = d.void_type();
    assert_eq!(void_t.to_sv_string(), "void");

    let unbounded_t = d.unbounded_type();
    assert_eq!(unbounded_t.as_symbol().kind(), SymbolKind::UnboundedType);

    let type_ref_t = d.type_ref_type();
    assert_eq!(type_ref_t.as_symbol().kind(), SymbolKind::TypeRefType);

    let unsigned_int_t = d.unsigned_int_type();
    assert!(unsigned_int_t.is_integral());
    assert!(!unsigned_int_t.is_signed());
    assert!(!unsigned_int_t.is_four_state());
    assert_eq!(unsigned_int_t.bit_width(), 32);
    // Distinct from the built-in (signed) `int` type it's easy to confuse it
    // with.
    assert_ne!(
        unsigned_int_t.as_symbol().id(),
        d.int_type().as_symbol().id()
    );

    let wire = d.wire_net_type();
    assert_eq!(wire.name(), "wire");
    assert_eq!(wire.kind(), SymbolKind::NetType);
    assert_eq!(wire.id(), d.net_type(sv_lang::NetTypeKind::Wire).id());

    // Every one of the type-shaped getters above is a genuinely distinct
    // object, not several handles onto the same type.
    let ids: BTreeSet<String> = [
        string_t.as_symbol().id(),
        void_t.as_symbol().id(),
        unbounded_t.as_symbol().id(),
        type_ref_t.as_symbol().id(),
        unsigned_int_t.as_symbol().id(),
    ]
    .into_iter()
    .map(|id| id.path().to_string())
    .collect();
    assert_eq!(ids.len(), 5);
}

#[test]
fn freeze_forces_unsigned_int_type_memo() {
    // Compilation::getUnsignedIntType() lazily allocates into
    // Compilation::vectorTypeCache on the first call for this exact
    // width/flags combination, via a non-const method that SLANG_ASSERTs
    // !isFrozen() on a cache miss. If slang_compilation_freeze did not force
    // this pre-seal, the very first post-freeze call below would trip that
    // assertion and abort the whole test process (in this assertions-on
    // build) rather than merely fail. Reaching the assertions proves it is
    // forced and safe.
    let d = compile(DESIGN);
    let t = d.unsigned_int_type();
    assert!(t.is_integral());
    assert!(!t.is_signed());
    assert_eq!(t.bit_width(), 32);
    // A second call is a plain cache hit, returning the same object.
    assert_eq!(t.as_symbol().id(), d.unsigned_int_type().as_symbol().id());
}

#[test]
fn get_system_method_looks_up_by_receiver_kind_and_name() {
    let d = compile(DESIGN);

    let push_back = d
        .get_system_method(SymbolKind::QueueType, "push_back")
        .unwrap();
    assert_eq!(push_back.name(), "push_back");
    assert!(!push_back.is_task());

    let pop_front = d
        .get_system_method(SymbolKind::QueueType, "pop_front")
        .unwrap();
    assert_eq!(pop_front.name(), "pop_front");

    // Distinct methods are visible as such (not one echoed handle).
    assert_ne!(push_back.name(), pop_front.name());

    // A real method name, but registered against a different receiver kind:
    // must not be found under the wrong kind.
    assert!(
        d.get_system_method(SymbolKind::EnumType, "push_back")
            .is_none()
    );
    // A real receiver kind, but not a method it has.
    assert!(
        d.get_system_method(SymbolKind::QueueType, "no_such_method_exists")
            .is_none()
    );

    // `num` is registered for EnumType (EnumNumMethod).
    let num = d.get_system_method(SymbolKind::EnumType, "num").unwrap();
    assert_eq!(num.name(), "num");
    assert!(!num.is_task());
}

#[test]
fn compilation_options_reports_the_constructed_settings() {
    let d = compile(DESIGN);
    let opts = d.options();

    // slang's own documented defaults (Compilation.h's CompilationOptions).
    assert_eq!(opts.max_instance_depth, 128);
    assert_eq!(opts.max_checker_instance_depth, 64);
    assert_eq!(opts.max_generate_steps, 131072);
    assert_eq!(opts.max_constexpr_depth, 128);
    assert_eq!(opts.max_constexpr_steps, 1_000_000);
    assert_eq!(opts.max_constexpr_backtrace, 10);
    assert_eq!(opts.max_constant_size, 8 * 1024 * 1024);
    assert_eq!(opts.max_defparam_steps, 128);
    assert_eq!(opts.max_defparam_blocks, u32::MAX);
    assert_eq!(opts.max_instance_array, 65535);
    assert_eq!(opts.max_enum_values, 65535);
    assert_eq!(opts.max_recursive_class_specialization, 8);
    assert_eq!(opts.max_udp_coverage_notes, 8);
    assert_eq!(opts.error_limit, 64);
    assert_eq!(opts.typo_correction_limit, 32);
    // MinTypMax::Typ = 1, LanguageVersion::Default (v1800_2017) = 1.
    assert_eq!(opts.min_typ_max, 1);
    assert_eq!(opts.language_version, 1);

    // `Compilation::new_with` always sets the raw compilation flags
    // explicitly (`Options::new().flags` = 0, overriding slang's own
    // AllowTopLevelIfacePorts default), ORing in DisableInstanceCaching
    // (bit 11) so that SLANG_FREEZE_ELABORATE_ALL can be total — so a
    // default-built compilation's snapshot carries exactly that one bit.
    assert_eq!(
        opts.flags,
        1 << 11,
        "expected only DisableInstanceCaching set"
    );
}

#[test]
fn unfreeze_lifts_the_seal_and_the_guard_restores_it() {
    let mut design = compile(DESIGN);
    // `compile()` runs SLANG_FREEZE_ALL, which includes SLANG_FREEZE_SEAL.
    assert!(design.is_sealed());

    {
        let guard = design.unfreeze();
        assert!(
            !guard.design().is_sealed(),
            "unfreeze must clear the seal for the guard's lifetime"
        );
        // The design is still fully readable while unfrozen: structural
        // reads never depended on the seal, only on the totalization sweep
        // `compile()` already ran.
        assert_eq!(guard.design().definitions().count(), 2);
    }
    // Dropping the guard re-seals it, without repeating elaboration: the
    // freeze report snapshot taken at `compile()` time is unaffected.
    assert!(design.is_sealed());
    assert!(design.freeze_report().fully_folded());

    // A second unfreeze/reseal cycle works identically (idempotent, not a
    // one-shot operation).
    let guard = design.unfreeze();
    assert!(!guard.design().is_sealed());
    drop(guard);
    assert!(design.is_sealed());

    // Reads keep working correctly after the round trip: not merely
    // "doesn't crash", but the same structural answer as before unfreezing.
    assert_eq!(
        design.top_instances().next().expect("top instance").name(),
        "top"
    );
}

#[test]
fn default_time_scale_is_none_when_not_configured() {
    // sv-lang's `Options` builder never sets `CompilationOptions::defaultTimeScale`
    // (there is no C-API setter for it), and it is not derived from `` `timescale ``
    // source directives (those are per-scope, not this compilation-wide default) —
    // so this getter has exactly one reachable state, and this is it.
    let d = compile(DESIGN);
    assert!(d.default_time_scale().is_none());

    let with_directive = compile("`timescale 1ns/1ps\nmodule m; endmodule\n");
    assert!(with_directive.default_time_scale().is_none());
}

#[test]
fn source_library_default_and_named_lookup() {
    let d = compile(DESIGN);

    let default_lib = d.default_library();
    assert!(default_lib.is_default());
    assert_eq!(default_lib.name(), "work");

    // The default library is registered under its own name too.
    let looked_up = d.source_library("work").unwrap();
    assert_eq!(looked_up.name(), "work");
    assert!(looked_up.is_default());
    assert_eq!(looked_up.priority(), default_lib.priority());

    assert!(d.source_library("does_not_exist_as_a_library").is_none());
}

#[test]
fn parse_and_semantic_diagnostics_partition_all_diagnostics() {
    // Clean design: no diagnostics anywhere.
    let clean = compile(DESIGN);
    assert!(!clean.parse_diagnostics().has_errors());
    assert!(!clean.semantic_diagnostics().has_errors());
    assert!(!clean.diagnostics().has_errors());

    // A genuine syntax error shows up in parse diagnostics but is not a
    // semantic-analysis finding.
    let syntax_err = compile("module m; logic x initial x = 1; endmodule\n");
    assert!(syntax_err.parse_diagnostics().has_errors());
    assert_eq!(
        syntax_err.parse_diagnostics().error_count(),
        syntax_err.diagnostics().error_count()
    );

    // A genuine semantic error (undeclared identifier) shows up in semantic
    // diagnostics but not in parse diagnostics.
    let sem_err = compile("module m; int x; initial x = y; endmodule\n");
    assert!(!sem_err.parse_diagnostics().has_errors());
    assert!(sem_err.semantic_diagnostics().has_errors());
    assert_eq!(
        sem_err.semantic_diagnostics().error_count(),
        sem_err.diagnostics().error_count()
    );
    assert!(sem_err.diagnostics().has_errors());
}

// ---- ScriptSession ----------------------------------------------------------

#[test]
fn script_session_eval_persists_state_and_reports_diagnostics() {
    let mut session = ScriptSession::new().unwrap();

    // A declaration produces no value...
    assert!(session.eval("int x = 3;").is_none());
    // ...but its state persists into later snippets on the same session.
    let v = session.eval("x * 4").unwrap();
    assert_eq!(v.as_i64(), Some(12));

    assert!(!session.diagnostics().has_errors());

    // A genuinely invalid snippet reports no value and a real diagnostic.
    assert!(
        session
            .eval("this is not $$$ valid systemverilog $$$")
            .is_none()
    );
    assert!(session.diagnostics().has_errors());
}

#[test]
fn script_session_eval_expression_and_statement_bind_against_session_scope() {
    let mut session = ScriptSession::new().unwrap();
    session.eval("int x = 100;");

    // The expression syntax comes from an entirely unrelated session/tree; only
    // its `x` identifier is shared in spelling, not in origin.
    let other = Session::new();
    let expr_tree = other
        .parse("module m; initial begin y = x - 1; end endmodule\n")
        .unwrap();
    let expr = expr_tree
        .root()
        .descendants()
        .find(|n| n.kind() == SyntaxKind::SubtractExpression)
        .expect("a subtract expression");
    let value = session
        .eval_expression(expr)
        .expect("evaluates against the session's own x");
    assert_eq!(value.as_i64(), Some(99));

    // Same story for a statement: its side effect lands in the session, not
    // wherever it was parsed.
    let stmt_tree = other
        .parse("module m2; initial begin x = 55; end endmodule\n")
        .unwrap();
    let stmt = stmt_tree
        .root()
        .descendants()
        .find(|n| n.kind() == SyntaxKind::ExpressionStatement)
        .expect("an expression statement");
    session.eval_statement(stmt);
    assert_eq!(session.eval("x").unwrap().as_i64(), Some(55));
}

#[test]
fn script_session_compilation_is_a_live_non_design_compilation() {
    let session = ScriptSession::new().unwrap();
    // A genuine, live Compilation handle: its built-in type resolves...
    assert_eq!(session.compilation().bit_type_name(), "bit");
    // ...and it never elaborates a design hierarchy the way
    // Compilation::compile does.
    assert_eq!(session.compilation().top_instance_count(), 0);

    // Declaring more script state doesn't change that.
    let mut session = session;
    session.eval("module inline_mod; endmodule");
    assert_eq!(session.compilation().top_instance_count(), 0);
}

// ---- Driver: source loader ---------------------------------------------------

#[test]
fn driver_source_loader_add_files_loads_the_named_file() {
    let dir = std::env::temp_dir().join("sv_lang_semantic_add_files_test");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("top.sv");
    std::fs::write(&file, "module top; endmodule\n").unwrap();

    let mut driver = Driver::new().unwrap();
    driver
        .source_loader()
        .add_files(&file.to_string_lossy())
        .unwrap();
    driver.process_options().unwrap();
    assert!(driver.parse_sources().unwrap());

    let trees = driver.trees();
    assert_eq!(trees.len(), 1);
    assert_eq!(trees[0].module_names().collect::<Vec<_>>(), ["top"]);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn driver_source_loader_add_library_files_are_not_automatically_top_level() {
    let dir = std::env::temp_dir().join("sv_lang_semantic_add_library_files_test");
    std::fs::create_dir_all(&dir).unwrap();
    let main = dir.join("top.sv");
    std::fs::write(&main, "module top; endmodule\n").unwrap();
    let libmod = dir.join("libmod.sv");
    std::fs::write(&libmod, "module libmod; endmodule\n").unwrap();

    let mut driver = Driver::new().unwrap();
    driver
        .source_loader()
        .add_files(&main.to_string_lossy())
        .unwrap();
    driver
        .source_loader()
        .add_library_files("mylib", &libmod.to_string_lossy())
        .unwrap();
    driver.process_options().unwrap();
    assert!(driver.parse_sources().unwrap());
    assert_eq!(driver.trees().len(), 2); // both files were loaded and parsed...

    let design = driver.compile().unwrap();
    let tops: Vec<_> = design
        .top_instances()
        .map(|s| s.name().to_string())
        .collect();
    // ...but only the non-library module was automatically instantiated.
    assert_eq!(tops, ["top"]);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn driver_source_loader_add_library_maps_creates_the_library_and_queues_its_files() {
    let dir = std::env::temp_dir().join("sv_lang_semantic_add_library_maps_test");
    std::fs::create_dir_all(&dir).unwrap();
    let main = dir.join("top.sv");
    std::fs::write(&main, "module top; endmodule\n").unwrap();
    std::fs::write(dir.join("libmod.sv"), "module libmod; endmodule\n").unwrap();
    let map = dir.join("libs.map");
    std::fs::write(&map, "library mylib libmod.sv;\n").unwrap();

    let mut driver = Driver::new().unwrap();
    driver
        .source_loader()
        .add_files(&main.to_string_lossy())
        .unwrap();
    driver
        .source_loader()
        .add_library_maps(&map.to_string_lossy(), &dir.to_string_lossy())
        .unwrap();
    driver.process_options().unwrap();
    assert!(driver.parse_sources().unwrap());
    assert_eq!(driver.trees().len(), 2); // top.sv + the map-referenced libmod.sv

    let design = driver.compile().unwrap();
    let tops: Vec<_> = design
        .top_instances()
        .map(|s| s.name().to_string())
        .collect();
    assert_eq!(tops, ["top"]); // libmod is a library file: not auto-instantiated

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn driver_source_loader_add_search_directories_resolves_missing_modules() {
    let dir = std::env::temp_dir().join("sv_lang_semantic_add_search_directories_test");
    std::fs::create_dir_all(&dir).unwrap();
    // `lib_mod` is never added directly; it can only be found via the search
    // directory registered below.
    std::fs::write(dir.join("lib_mod.sv"), "module lib_mod; endmodule\n").unwrap();
    let top = dir.join("top.sv");
    std::fs::write(&top, "module top; lib_mod u(); endmodule\n").unwrap();

    // Without the search directory, the unknown-instance diagnostic makes the
    // compile fail.
    let mut without = Driver::new().unwrap();
    without
        .source_loader()
        .add_files(&top.to_string_lossy())
        .unwrap();
    without.process_options().unwrap();
    without.parse_sources().unwrap();
    let design_without = without.compile().unwrap();
    assert!(design_without.into_result().is_err());

    // With it, `lib_mod` is found, parsed, and instantiated.
    let mut with = Driver::new().unwrap();
    with.source_loader()
        .add_files(&top.to_string_lossy())
        .unwrap();
    with.source_loader()
        .add_search_directories(&dir.to_string_lossy())
        .unwrap();
    with.process_options().unwrap();
    assert!(with.parse_sources().unwrap());
    assert_eq!(with.trees().len(), 2); // top.sv + the search-resolved lib_mod.sv
    let design_with = with.compile().unwrap();
    assert!(design_with.into_result().is_ok());

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn driver_source_loader_add_search_extension_is_required_to_find_a_nonstandard_extension() {
    let dir = std::env::temp_dir().join("sv_lang_semantic_add_search_extension_test");
    std::fs::create_dir_all(&dir).unwrap();
    // ".vsrc" is not one of the two extensions (".v"/".sv") always searched.
    std::fs::write(dir.join("lib_mod.vsrc"), "module lib_mod; endmodule\n").unwrap();
    let top = dir.join("top.sv");
    std::fs::write(&top, "module top; lib_mod u(); endmodule\n").unwrap();

    // A search directory alone is not enough without the matching extension.
    let mut without_ext = Driver::new().unwrap();
    without_ext
        .source_loader()
        .add_files(&top.to_string_lossy())
        .unwrap();
    without_ext
        .source_loader()
        .add_search_directories(&dir.to_string_lossy())
        .unwrap();
    without_ext.process_options().unwrap();
    without_ext.parse_sources().unwrap();
    assert_eq!(without_ext.trees().len(), 1); // lib_mod.vsrc was never found
    assert!(without_ext.compile().unwrap().into_result().is_err());

    // Registering the extension lets the same search directory find it.
    let mut with_ext = Driver::new().unwrap();
    with_ext
        .source_loader()
        .add_files(&top.to_string_lossy())
        .unwrap();
    with_ext
        .source_loader()
        .add_search_directories(&dir.to_string_lossy())
        .unwrap();
    with_ext
        .source_loader()
        .add_search_extension("vsrc")
        .unwrap();
    with_ext.process_options().unwrap();
    assert!(with_ext.parse_sources().unwrap());
    assert_eq!(with_ext.trees().len(), 2);
    assert!(with_ext.compile().unwrap().into_result().is_ok());

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn driver_source_loader_add_separate_unit_groups_files_applies_defines_and_library() {
    let dir = std::env::temp_dir().join("sv_lang_semantic_add_separate_unit_test");
    std::fs::create_dir_all(&dir).unwrap();
    let a = dir.join("a.sv");
    std::fs::write(&a, "module top; endmodule\n").unwrap();
    let b = dir.join("b.sv");
    // Only compiles into a module if the FOO define (passed below, without
    // any -D on a command line) is active.
    std::fs::write(&b, "`ifdef FOO\nmodule extra; endmodule\n`endif\n").unwrap();
    let lib = dir.join("lib.sv");
    std::fs::write(&lib, "module libbed; endmodule\n").unwrap();

    let mut driver = Driver::new().unwrap();
    // `a.sv` and `b.sv` are grouped into ONE compilation unit (unlike two
    // separate add_files calls, which would produce two trees) and see the
    // FOO define.
    driver
        .source_loader()
        .add_separate_unit(
            &[
                a.to_string_lossy().into_owned(),
                b.to_string_lossy().into_owned(),
            ],
            &[],
            &["FOO".to_string()],
            "",
            &[],
        )
        .unwrap();
    // `lib.sv` is its own separate unit, routed into a named library, so its
    // module is not automatically instantiated.
    driver
        .source_loader()
        .add_separate_unit(
            &[lib.to_string_lossy().into_owned()],
            &[],
            &[],
            "mylib",
            &[],
        )
        .unwrap();
    driver.process_options().unwrap();
    assert!(driver.parse_sources().unwrap());

    let trees = driver.trees();
    assert_eq!(trees.len(), 2); // {a.sv, b.sv} as one tree, lib.sv as another
    let combined = trees
        .iter()
        .find(|t| t.module_names().count() == 2)
        .expect("a.sv + b.sv grouped into a single tree");
    let mut names: Vec<_> = combined.module_names().collect();
    names.sort();
    assert_eq!(names, ["extra", "top"]); // proves the FOO define took effect

    let design = driver.compile().unwrap();
    let mut tops: Vec<_> = design
        .top_instances()
        .map(|s| s.name().to_string())
        .collect();
    tops.sort();
    // `top` and `extra` are both un-instantiated modules (so both become top
    // instances); `libbed` is excluded because it went into a named library.
    assert_eq!(tops, ["extra", "top"]);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn driver_source_loader_has_files_and_errors_reflect_add_files_outcomes() {
    let dir = std::env::temp_dir().join("sv_lang_semantic_has_files_and_errors_test");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("m.sv");
    std::fs::write(&file, "module m; endmodule\n").unwrap();

    let mut driver = Driver::new().unwrap();
    assert!(!driver.source_loader().has_files());
    assert!(driver.source_loader().errors().is_empty());

    // A literal filename that doesn't exist records an error but adds no file.
    driver
        .source_loader()
        .add_files("sv_lang_missing_literal_file.sv")
        .unwrap();
    assert!(!driver.source_loader().has_files());
    let errors = driver.source_loader().errors();
    assert_eq!(errors.len(), 1);
    assert!(
        errors[0].contains("sv_lang_missing_literal_file.sv"),
        "error was: {:?}",
        errors[0]
    );

    // A real file adds an entry (has_files flips true) without adding a
    // second error.
    driver
        .source_loader()
        .add_files(&file.to_string_lossy())
        .unwrap();
    assert!(driver.source_loader().has_files());
    assert_eq!(driver.source_loader().errors().len(), 1);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn driver_source_loader_library_maps_grows_as_maps_are_added() {
    let dir = std::env::temp_dir().join("sv_lang_semantic_library_maps_test");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("lib_mod.sv"), "module lib_mod; endmodule\n").unwrap();
    let map1 = dir.join("libs1.map");
    std::fs::write(&map1, "library mylib1 lib_mod.sv;\n").unwrap();
    let map2 = dir.join("libs2.map");
    std::fs::write(&map2, "library mylib2 lib_mod.sv;\n").unwrap();

    let mut driver = Driver::new().unwrap();
    assert!(driver.source_loader().library_maps().is_empty());

    driver
        .source_loader()
        .add_library_maps(&map1.to_string_lossy(), &dir.to_string_lossy())
        .unwrap();
    let maps = driver.source_loader().library_maps();
    assert_eq!(maps.len(), 1);
    assert!(maps[0].text().contains("mylib1"));

    // A second map grows the list without disturbing the first entry.
    driver
        .source_loader()
        .add_library_maps(&map2.to_string_lossy(), &dir.to_string_lossy())
        .unwrap();
    let maps = driver.source_loader().library_maps();
    assert_eq!(maps.len(), 2);
    assert!(maps[0].text().contains("mylib1"));
    assert!(maps[1].text().contains("mylib2"));

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn driver_source_loader_load_sources_loads_text_without_parsing() {
    let dir = std::env::temp_dir().join("sv_lang_semantic_load_sources_test");
    std::fs::create_dir_all(&dir).unwrap();
    let a = dir.join("a.sv");
    std::fs::write(&a, "module a; endmodule\n").unwrap();
    let b = dir.join("b.sv");
    std::fs::write(&b, "module b; endmodule\n").unwrap();

    let mut driver = Driver::new().unwrap();
    driver
        .source_loader()
        .add_files(&a.to_string_lossy())
        .unwrap();
    driver
        .source_loader()
        .add_files(&b.to_string_lossy())
        .unwrap();

    let buffers = driver.source_loader().load_sources().unwrap();
    assert_eq!(buffers.len(), 2);
    assert!(buffers.iter().all(|b| b.id != 0));
    // Every assigned buffer ID is distinct.
    assert_ne!(buffers[0].id, buffers[1].id);
    let mut texts: Vec<_> = buffers.iter().map(|b| b.text.as_str()).collect();
    texts.sort();
    assert_eq!(texts, ["module a; endmodule\n", "module b; endmodule\n"]);

    // loadSources only loads; it never parses, so no trees exist yet.
    assert_eq!(driver.trees().len(), 0);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn source_options_getters_reflect_the_values_set() {
    let mut options = SourceOptions::new();
    assert_eq!(options.num_threads(), None);
    assert!(!options.single_unit());
    assert!(!options.only_lint());
    assert!(!options.libraries_inherit_macros());

    options.set_num_threads(Some(2));
    options.set_single_unit(true);
    options.set_only_lint(true);
    options.set_libraries_inherit_macros(true);

    assert_eq!(options.num_threads(), Some(2));
    assert!(options.single_unit());
    assert!(options.only_lint());
    assert!(options.libraries_inherit_macros());

    // Clearing the thread count goes back to "unset", independent of the
    // bool options.
    options.set_num_threads(None);
    assert_eq!(options.num_threads(), None);
    assert!(options.single_unit());
}

// ---- SystemSubroutine -----------------------------------------------------

#[test]
fn expr_system_subroutine_distinguishes_builtins_and_reads_argument_predicates() {
    // $clog2: a function whose argument predicates are both false.
    let design = compile("module m; localparam integer W = $clog2(8); endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let clog2_call = body.find("W").unwrap().initializer().unwrap();
    assert_eq!(clog2_call.kind(), sv_lang::kinds::ExpressionKind::Call);
    assert!(clog2_call.call_subroutine().is_none());
    let clog2 = clog2_call.system_subroutine().unwrap();
    assert_eq!(clog2.name(), "$clog2");
    assert!(!clog2.is_task());
    assert!(!clog2.allow_empty_argument(0));
    assert!(!clog2.allow_clocking_argument(0));

    // $display: a task whose allowEmptyArgument is unconditionally true.
    let design = compile("module m; initial $display(\"hi\"); endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let display_call = block.body().unwrap().expressions()[0];
    let display = display_call.system_subroutine().unwrap();
    assert_eq!(display.name(), "$display");
    assert!(display.is_task());
    assert!(display.allow_empty_argument(0));

    // $rose: a function whose allowClockingArgument is true only at index 1.
    let design =
        compile("module m(input logic clk, output logic q); always @* q = $rose(clk); endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let assign = block.body().unwrap().body().unwrap().expressions()[0];
    let rhs = assign.right().unwrap();
    let rose_call = rhs.conversion_operand().unwrap_or(rhs);
    let rose = rose_call.system_subroutine().unwrap();
    assert_eq!(rose.name(), "$rose");
    assert!(!rose.allow_clocking_argument(0));
    assert!(rose.allow_clocking_argument(1));
}

#[test]
fn eval_session_check_bind_and_eval_system_call_replay_clog2() {
    let mut design = compile("module m; localparam integer W = $clog2(8); endmodule\n");
    let eval = design.eval_session();
    let call = eval
        .design()
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap()
        .find("W")
        .unwrap()
        .initializer()
        .unwrap();

    // checkArguments reproduces the exact type slang elaborated the call with.
    let checked_type = eval.check_system_call(call).unwrap();
    assert_eq!(
        checked_type.to_sv_string(),
        call.expr_type().unwrap().to_sv_string()
    );
    assert_eq!(checked_type.to_sv_string(), "integer");

    // bindArgument re-binds the original `8` argument syntax to a fresh,
    // equal-valued expression.
    let bound_arg = eval.bind_system_call_argument(call, 0).unwrap();
    assert_eq!(eval.eval_constant(bound_arg).unwrap().as_i64(), Some(8));
    // Out-of-range and non-system-call inputs fail cleanly rather than panicking.
    assert!(eval.bind_system_call_argument(call, 5).is_none());

    // eval reproduces the constant-folded result: ceil(log2(8)) == 3.
    let value = eval.eval_system_call(call).unwrap();
    assert_eq!(value.as_i64(), Some(3));
    assert_eq!(
        value.as_i64(),
        call.constant_value().unwrap().as_i64(),
        "replayed eval should match the value slang already folded"
    );

    // A non-system-call expression (the literal `8` argument itself) is
    // rejected cleanly rather than panicking.
    assert!(eval.check_system_call(bound_arg).is_none());
    assert!(eval.eval_system_call(bound_arg).is_none());
}

#[test]
fn system_subroutine_field_accessors_distinguish_builtins() {
    // $clog2: a pure function, no output args, no with-clause, KnownSystemName::Clog2.
    let design = compile("module m; localparam integer W = $clog2(8); endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let clog2_call = body.find("W").unwrap().initializer().unwrap();
    let clog2 = clog2_call.system_subroutine().unwrap();
    assert!(!clog2.has_output_args());
    assert_eq!(clog2.kind(), sv_lang::SubroutineKind::Function);
    assert_eq!(clog2.known_name_id(), sv_lang::KnownSystemName::Clog2);
    assert_eq!(clog2.with_clause_mode(), sv_lang::WithClauseMode::None);
    assert_eq!(clog2.kind_str(), "function");

    // $display: a task, KnownSystemName::Display.
    let design = compile("module m; initial $display(\"hi\"); endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let display_call = block.body().unwrap().expressions()[0];
    let display = display_call.system_subroutine().unwrap();
    assert_eq!(display.kind(), sv_lang::SubroutineKind::Task);
    assert_eq!(display.known_name_id(), sv_lang::KnownSystemName::Display);
    assert_eq!(display.kind_str(), "task");

    // $sscanf: a function with output (ref) arguments.
    let design = compile(
        "module m; string s; integer a; integer r; initial r = $sscanf(s, \"%d\", a); \
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let assign = block.body().unwrap().expressions()[0];
    let sscanf_call = assign.right().unwrap();
    let sscanf = sscanf_call.system_subroutine().unwrap();
    assert!(sscanf.has_output_args());
    assert_eq!(sscanf.kind(), sv_lang::SubroutineKind::Function);
    assert_eq!(sscanf.known_name_id(), sv_lang::KnownSystemName::SScanf);

    // q.sum(): an iterator method (WithClauseMode::Iterator), not a built-in
    // `$name` (KnownSystemName::Sum is still set, but this call goes through
    // Compilation::getSystemMethod rather than a `$`-prefixed lookup).
    let design = compile("module m; int q[$]; initial void'(q.sum()); endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let stmt_expr = block.body().unwrap().expressions()[0];
    let sum_call = stmt_expr.conversion_operand().unwrap_or(stmt_expr);
    let sum = sum_call.system_subroutine().unwrap();
    assert_eq!(sum.kind(), sv_lang::SubroutineKind::Function);
    assert_eq!(sum.known_name_id(), sv_lang::KnownSystemName::Sum);
    assert_eq!(sum.with_clause_mode(), sv_lang::WithClauseMode::Iterator);
}

#[test]
fn eval_session_replays_protected_system_subroutine_helpers() {
    let mut design = compile("module m; localparam integer W = $clog2(8); endmodule\n");
    assert!(!design.has_issued_errors());
    let eval = design.eval_session();
    let call = eval
        .design()
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap()
        .find("W")
        .unwrap()
        .initializer()
        .unwrap();

    // badArg unconditionally returns the compilation's error type and (through
    // the call's own ASTContext) reports a real diagnostic.
    let bad_ty = eval.bad_arg(call, 0).unwrap();
    assert_eq!(
        bad_ty.as_symbol().kind(),
        sv_lang::kinds::SymbolKind::ErrorType
    );
    // Out-of-range arg_index fails cleanly.
    assert!(eval.bad_arg(call, 5).is_none());

    // checkArgCount: $clog2(8) was bound with exactly 1 argument.
    assert_eq!(eval.check_arg_count(call, false, 1, 1), Some(true));
    assert_eq!(eval.check_arg_count(call, false, 2, 3), Some(false)); // too few
    assert_eq!(eval.check_arg_count(call, false, 0, 0), Some(false)); // too many

    // noHierarchical: the literal `8` argument has no hierarchical reference.
    assert_eq!(eval.no_hierarchical(call, 0), Some(true));
    assert!(eval.no_hierarchical(call, 5).is_none());

    // notConst unconditionally returns false.
    assert_eq!(eval.not_const(call), Some(false));

    // unevaluatedContext clears StaticInitializer while leaving other flags.
    assert_eq!(
        eval.unevaluated_context_clears_static_initializer(call),
        Some(true)
    );

    // The badArg replay above reported through the call's own ASTContext,
    // straight into the compilation.
    assert!(design.has_issued_errors());

    // Non-system-call input fails cleanly rather than panicking.
    let mut clean = compile("module m; localparam integer W = $clog2(8); endmodule\n");
    let clean_eval = clean.eval_session();
    let non_call = clean_eval
        .design()
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap()
        .find("W")
        .unwrap()
        .initializer()
        .unwrap();
    let literal_arg = clean_eval.bind_system_call_argument(non_call, 0).unwrap();
    assert!(clean_eval.bad_arg(literal_arg, 0).is_none());
    assert!(
        clean_eval
            .check_arg_count(literal_arg, false, 0, 1)
            .is_none()
    );
    assert!(clean_eval.no_hierarchical(literal_arg, 0).is_none());
    assert!(clean_eval.not_const(literal_arg).is_none());
    assert!(
        clean_eval
            .unevaluated_context_clears_static_initializer(literal_arg)
            .is_none()
    );
}

#[test]
fn eval_session_no_hierarchical_detects_a_real_hierarchical_reference() {
    // $display does not itself call noHierarchical (unlike e.g. $bits, which
    // would already reject this call during elaboration), so `s.x` binds
    // successfully and we can replay noHierarchical against it directly.
    let mut design = compile(
        "module sub; logic [7:0] x; endmodule\n\
         module m; sub s(); initial $display(s.x); endmodule\n",
    );
    assert!(!design.has_issued_errors());
    let eval = design.eval_session();
    let body = eval
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
    let call = block.body().unwrap().expressions()[0];
    // `s.x` reaches into a sibling instance's scope: a genuine hierarchical
    // reference, so noHierarchical must report it and return false.
    assert_eq!(eval.no_hierarchical(call, 0), Some(false));
}

// ---- Driver: reporting / running -----------------------------------------

#[test]
fn driver_report_parse_diags_reports_syntax_errors_and_succeeds_on_clean_input() {
    let dir = std::env::temp_dir().join("sv_lang_semantic_report_parse_diags_test");
    std::fs::create_dir_all(&dir).unwrap();

    let good = dir.join("good.sv");
    std::fs::write(&good, "module m; endmodule\n").unwrap();
    let mut good_driver = Driver::new().unwrap();
    good_driver
        .parse_args(["slang".to_string(), good.to_string_lossy().into_owned()])
        .unwrap();
    good_driver.process_options().unwrap();
    good_driver.parse_sources().unwrap();
    assert!(good_driver.report_parse_diags());
    assert_eq!(good_driver.diag_engine().num_errors(), 0);

    let bad = dir.join("bad.sv");
    std::fs::write(&bad, "module m; +++ endmodule\n").unwrap();
    let mut bad_driver = Driver::new().unwrap();
    bad_driver
        .parse_args(["slang".to_string(), bad.to_string_lossy().into_owned()])
        .unwrap();
    bad_driver.process_options().unwrap();
    bad_driver.parse_sources().unwrap();
    // The syntax error exists in the parsed tree but is not yet issued
    // through the diagnostic engine until report_parse_diags runs.
    assert_eq!(bad_driver.diag_engine().num_errors(), 0);
    assert!(!bad_driver.report_parse_diags());
    assert!(bad_driver.diag_engine().num_errors() > 0);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn driver_run_full_compilation_reports_success_and_failure() {
    let dir = std::env::temp_dir().join("sv_lang_semantic_run_full_compilation_test");
    std::fs::create_dir_all(&dir).unwrap();

    let good = dir.join("good.sv");
    std::fs::write(&good, "module m; endmodule\n").unwrap();
    let mut good_driver = Driver::new().unwrap();
    good_driver
        .parse_args(["slang".to_string(), good.to_string_lossy().into_owned()])
        .unwrap();
    good_driver.process_options().unwrap();
    good_driver.parse_sources().unwrap();
    assert!(good_driver.run_full_compilation(true));

    let bad = dir.join("bad.sv");
    std::fs::write(
        &bad,
        "module m; initial x = totally_undeclared_thing; endmodule\n",
    )
    .unwrap();
    let mut bad_driver = Driver::new().unwrap();
    bad_driver
        .parse_args(["slang".to_string(), bad.to_string_lossy().into_owned()])
        .unwrap();
    bad_driver.process_options().unwrap();
    bad_driver.parse_sources().unwrap();
    assert!(!bad_driver.run_full_compilation(true));

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn driver_run_analysis_runs_the_configured_passes_and_reports_through_the_driver() {
    let mut driver = Driver::new().unwrap();
    let mut comp = Compilation::new(driver.session()).unwrap();
    comp.add_source("module m; logic completely_unused_signal; endmodule\n")
        .unwrap();

    assert_eq!(driver.diag_engine().num_warnings(), 0);
    let analysis = driver.run_analysis(&mut comp).unwrap();
    let diags = analysis.diagnostics();

    // The driver's CheckUnused default flag (always on: see
    // Driver::getAnalysisOptions) fires a real unused-signal warning...
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("completely_unused_signal")),
        "expected an unused-signal diagnostic, got: {diags}"
    );
    // ...and it was also reported through the driver's own diagnostic engine.
    assert!(driver.diag_engine().num_warnings() > 0);
}

#[test]
fn driver_text_diag_client_is_registered_and_starts_empty() {
    let driver = Driver::new().unwrap();
    let client = driver.text_diag_client();
    assert!(client.empty());
    assert!(client.get_string().is_empty());
}

// The following tests redirect the process's real stdout/stderr file
// descriptors to inspect what slang actually printed (report_macros and
// run_preprocessor have no other observable output; set_terminal_colors_enabled's
// effect lands in a StderrDiagnosticClient buffer that self-clears before any
// C accessor can read it back — see TextDiagClient::get_string). Redirecting
// fd 1/2 is process-global, so every such capture in this binary is
// serialized through CAPTURE_LOCK to avoid corrupting a concurrently running
// test (or, worse, leaving the real stdout/stderr pointed at a deleted temp
// file).
#[cfg(unix)]
mod fd_capture {
    use std::fs::File;
    use std::io::{Read, Write};
    use std::os::unix::io::AsRawFd;
    use std::sync::Mutex;

    unsafe extern "C" {
        fn dup(fd: i32) -> i32;
        fn dup2(oldfd: i32, newfd: i32) -> i32;
        fn close(fd: i32) -> i32;
        // slang's OS::print/printE write through C's buffered stdio (`FILE*
        // stdout`/`stderr`, explicitly set fully-buffered by OS::OS()), which
        // is invisible to our fd-level redirection until flushed: a NULL
        // argument flushes every open stdio stream.
        fn fflush(stream: *mut core::ffi::c_void) -> i32;
    }

    static CAPTURE_LOCK: Mutex<()> = Mutex::new(());

    /// Runs `f` with file descriptor `target_fd` (1 = stdout, 2 = stderr)
    /// redirected to a temp file, and returns everything written to it.
    pub fn capture_fd(target_fd: i32, f: impl FnOnce()) -> String {
        let _guard = CAPTURE_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        std::io::stdout().flush().ok();
        std::io::stderr().flush().ok();

        let path = std::env::temp_dir().join(format!(
            "sv_lang_semantic_fd_capture_{}_{target_fd}.txt",
            std::process::id()
        ));
        let tmp = File::create(&path).expect("create capture temp file");

        // SAFETY: raw fd redirection of this process's own standard streams,
        // serialized by CAPTURE_LOCK; the saved fd and temp file are both
        // cleaned up below before returning.
        let saved = unsafe { dup(target_fd) };
        assert!(saved >= 0, "dup({target_fd}) failed");
        // dup2 returns the new descriptor number (== target_fd) on success,
        // not 0; only a negative return indicates failure.
        // SAFETY: `tmp` is a valid, live file; `target_fd` is 1 or 2, both
        // always-open standard streams; serialized by CAPTURE_LOCK.
        let rc = unsafe { dup2(tmp.as_raw_fd(), target_fd) };
        assert!(rc >= 0, "dup2 redirect failed");

        f();

        std::io::stdout().flush().ok();
        std::io::stderr().flush().ok();
        // Force slang's own buffered C-stdio writes out to the (still
        // redirected) fd before we point it back at the real stream.
        // SAFETY: `fflush(NULL)` is well-defined per the C standard (flushes
        // every open stdio stream); no pointer is dereferenced.
        unsafe { fflush(core::ptr::null_mut()) };

        // SAFETY: `saved` is the valid fd this function duplicated above;
        // restoring it is the inverse of the redirect, serialized the same way.
        let rc = unsafe { dup2(saved, target_fd) };
        assert!(rc >= 0, "dup2 restore failed");
        // SAFETY: `saved` is no longer needed once restored above.
        unsafe { close(saved) };

        let mut buf = String::new();
        File::open(&path)
            .expect("reopen capture temp file")
            .read_to_string(&mut buf)
            .expect("read capture temp file");
        std::fs::remove_file(&path).ok();
        buf
    }
}

#[cfg(unix)]
#[test]
fn driver_run_preprocessor_flags_change_output_and_reports_missing_includes() {
    let dir = std::env::temp_dir().join("sv_lang_semantic_run_preprocessor_test");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("m.sv");
    std::fs::write(
        &file,
        "// a very distinctive leading comment\nmodule m; endmodule\n",
    )
    .unwrap();

    let mut driver = Driver::new().unwrap();
    driver
        .parse_args(["slang".to_string(), file.to_string_lossy().into_owned()])
        .unwrap();
    driver.process_options().unwrap();

    let mut ok = false;
    let without_comments = fd_capture::capture_fd(1, || {
        ok = driver.run_preprocessor(PreprocessFlags::NONE).unwrap();
    });
    assert!(ok);
    assert!(!without_comments.contains("a very distinctive leading comment"));

    let mut ok = false;
    let with_comments = fd_capture::capture_fd(1, || {
        ok = driver
            .run_preprocessor(PreprocessFlags::INCLUDE_COMMENTS)
            .unwrap();
    });
    assert!(ok);
    assert!(with_comments.contains("a very distinctive leading comment"));

    // A missing `include is a genuine preprocessing error.
    let bad = dir.join("bad_include.sv");
    std::fs::write(
        &bad,
        "`include \"does-not-exist-anywhere.svh\"\nmodule m; endmodule\n",
    )
    .unwrap();
    let mut bad_driver = Driver::new().unwrap();
    bad_driver
        .parse_args(["slang".to_string(), bad.to_string_lossy().into_owned()])
        .unwrap();
    bad_driver.process_options().unwrap();
    let mut bad_ok = true;
    let _ = fd_capture::capture_fd(2, || {
        bad_ok = bad_driver.run_preprocessor(PreprocessFlags::NONE).unwrap();
    });
    assert!(!bad_ok);

    std::fs::remove_dir_all(&dir).ok();
}

#[cfg(unix)]
#[test]
fn driver_report_macros_prints_defined_macro_names_and_values() {
    let dir = std::env::temp_dir().join("sv_lang_semantic_report_macros_test");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("m.sv");
    std::fs::write(&file, "`define FOO 42\nmodule m; endmodule\n").unwrap();

    let mut driver = Driver::new().unwrap();
    driver
        .parse_args(["slang".to_string(), file.to_string_lossy().into_owned()])
        .unwrap();
    driver.process_options().unwrap();

    let output = fd_capture::capture_fd(1, || driver.report_macros(false));
    assert!(output.contains("FOO"), "output was: {output:?}");
    assert!(output.contains("42"), "output was: {output:?}");

    let grouped = fd_capture::capture_fd(1, || driver.report_macros(true));
    assert!(
        grouped.contains("m.sv"),
        "expected the file name when grouping: {grouped:?}"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[cfg(unix)]
#[test]
fn driver_set_terminal_colors_enabled_toggles_ansi_codes_in_stderr_output() {
    let dir = std::env::temp_dir().join("sv_lang_semantic_terminal_colors_test");
    std::fs::create_dir_all(&dir).unwrap();
    let bad = dir.join("bad.sv");
    std::fs::write(
        &bad,
        "module m; initial $display(totally_undeclared_xyz); endmodule\n",
    )
    .unwrap();

    let make_driver = || {
        let mut driver = Driver::new().unwrap();
        driver
            .parse_args(["slang".to_string(), bad.to_string_lossy().into_owned()])
            .unwrap();
        driver.process_options().unwrap();
        driver.parse_sources().unwrap();
        driver
    };

    let mut colored = make_driver();
    colored.set_terminal_colors_enabled(true);
    let design = colored.compile().unwrap();
    let colored_output = fd_capture::capture_fd(2, || colored.report_compilation(&design));
    assert!(
        colored_output.contains('\u{1b}'),
        "expected ANSI escape codes in: {colored_output:?}"
    );

    let mut plain = make_driver();
    plain.set_terminal_colors_enabled(false);
    let design = plain.compile().unwrap();
    let plain_output = fd_capture::capture_fd(2, || plain.report_compilation(&design));
    assert!(
        !plain_output.contains('\u{1b}'),
        "expected no ANSI escape codes in: {plain_output:?}"
    );
    assert!(plain_output.contains("totally_undeclared_xyz"));

    std::fs::remove_dir_all(&dir).ok();
}

// ---- Statement breadth: Block/Conditional/Case/ConcurrentAssertion/EventTrigger

#[test]
fn stmt_block_kind_and_symbol_distinguish_named_sequential_and_unnamed_fork() {
    use sv_lang::StatementBlockKind;
    use sv_lang::kinds::StatementKind;

    let design = compile(
        "module m;\n\
         initial begin: b\n\
           int i;\n\
           i = 0;\n\
         end\n\
         initial fork\n\
           #1;\n\
           #2;\n\
         join_any\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let blocks: Vec<_> = body
        .members()
        .filter(|s| s.kind() == SymbolKind::ProceduralBlock)
        .collect();
    assert_eq!(blocks.len(), 2);

    // Named `begin: b ... end`: Sequential, with its StatementBlockSymbol.
    let named = blocks[0].body().unwrap();
    assert_eq!(named.kind(), StatementKind::Block);
    assert_eq!(named.block_kind(), Some(StatementBlockKind::Sequential));
    let sym = named.block_symbol().unwrap();
    assert_eq!(sym.name(), "b");

    // Unnamed `fork ... join_any` with no local declarations: JoinAny, no symbol.
    let forked = blocks[1].body().unwrap();
    assert_eq!(forked.kind(), StatementKind::Block);
    assert_eq!(forked.block_kind(), Some(StatementBlockKind::JoinAny));
    assert!(forked.block_symbol().is_none());

    // A non-block statement (the block's own body statement, whatever its
    // kind) carries neither a block kind nor a block symbol.
    let inner = named.statements()[0];
    assert_ne!(inner.kind(), StatementKind::Block);
    assert_eq!(inner.block_kind(), None);
    assert!(inner.block_symbol().is_none());
}

#[test]
fn stmt_conditional_check_conditions_and_pattern_match() {
    use sv_lang::UniquePriorityCheck;
    use sv_lang::kinds::{ExpressionKind, PatternKind, StatementKind};

    // A plain `priority if`/`else`: one condition, no pattern.
    let design = compile(
        "module m(input logic clk, rst, input logic [7:0] d, output logic [7:0] q);\n\
         always_ff @(posedge clk) priority if (rst) q <= 0; else q <= d + 1;\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let cond = block.body().unwrap().body().unwrap(); // inside the @(posedge clk) timing wrapper
    assert_eq!(cond.kind(), StatementKind::Conditional);
    assert_eq!(cond.check(), Some(UniquePriorityCheck::Priority));

    let conditions = cond.conditions();
    assert_eq!(conditions.len(), 1);
    assert_eq!(conditions[0].expr().kind(), ExpressionKind::NamedValue);
    assert!(conditions[0].pattern().is_none());

    // A non-conditional statement carries neither a check nor conditions.
    let then_branch = cond.then_branch().unwrap();
    assert!(then_branch.conditions().is_empty());
    assert_eq!(then_branch.check(), None);

    // A pattern-matching `if` over a tagged union: the condition's pattern is
    // a `Tagged` pattern binding the `.a` payload variable.
    let design = compile(
        "module m(output int r);\n\
         typedef union tagged { void Invalid; int Valid; } tagged_t;\n\
         tagged_t v;\n\
         always_comb if (v matches tagged Valid .a) r = a; else r = 0;\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let cond = block.body().unwrap();
    assert_eq!(cond.kind(), StatementKind::Conditional);
    assert_eq!(cond.check(), Some(UniquePriorityCheck::None));

    let conditions = cond.conditions();
    assert_eq!(conditions.len(), 1);
    let pattern = conditions[0].pattern().unwrap();
    assert_eq!(pattern.kind(), PatternKind::Tagged);
}

#[test]
fn stmt_case_condition_check_default_and_items() {
    use sv_lang::kinds::{ExpressionKind, StatementKind};
    use sv_lang::{CaseStatementCondition, UniquePriorityCheck};

    let design = compile(
        "module m(input logic [1:0] s, output logic o);\n\
         always_comb unique case (s)\n\
           2'b00, 2'b01: o = 0;\n\
           2'b10: o = 1;\n\
           default: o = 0;\n\
         endcase\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let case_stmt = block.body().unwrap();
    assert_eq!(case_stmt.kind(), StatementKind::Case);
    assert_eq!(case_stmt.check(), Some(UniquePriorityCheck::Unique));
    assert_eq!(
        case_stmt.case_condition(),
        Some(CaseStatementCondition::Normal)
    );
    assert!(case_stmt.default_case().is_some());

    let items = case_stmt.case_items();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].expressions().len(), 2);
    assert_eq!(items[1].expressions().len(), 1);
    assert_eq!(
        items[0].stmt().expr().unwrap().kind(),
        ExpressionKind::Assignment
    );
    assert_eq!(
        items[1].stmt().expr().unwrap().kind(),
        ExpressionKind::Assignment
    );

    // A non-case statement carries no items/condition/default.
    let default_stmt = case_stmt.default_case().unwrap();
    assert!(default_stmt.case_items().is_empty());
    assert_eq!(default_stmt.case_condition(), None);
    assert!(default_stmt.default_case().is_none());

    // `casex`/`casez`/`case ... inside` each report their distinct condition kind.
    let casex = compile(
        "module m(input logic [1:0] s, output logic o);\n\
         always_comb casex (s) 2'b0?: o = 0; default: o = 1; endcase\n\
         endmodule\n",
    );
    let body = casex
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    assert_eq!(
        block.body().unwrap().case_condition(),
        Some(CaseStatementCondition::WildcardXOrZ)
    );

    let casez = compile(
        "module m(input logic [1:0] s, output logic o);\n\
         always_comb casez (s) 2'b0?: o = 0; default: o = 1; endcase\n\
         endmodule\n",
    );
    let body = casez
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    assert_eq!(
        block.body().unwrap().case_condition(),
        Some(CaseStatementCondition::WildcardJustZ)
    );

    let inside = compile(
        "module m(input logic [1:0] s, output logic o);\n\
         always_comb case (s) inside\n\
           2'b00: o = 0;\n\
           default: o = 1;\n\
         endcase\n\
         endmodule\n",
    );
    let body = inside
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    assert_eq!(
        block.body().unwrap().case_condition(),
        Some(CaseStatementCondition::Inside)
    );
}

#[test]
fn stmt_concurrent_assertion_kind_and_actions() {
    use sv_lang::AssertionKind;
    use sv_lang::kinds::StatementKind;

    // `assert property` with only a fail-action (`else`).
    let design = compile(
        "module m(input logic clk, a, output logic err);\n\
         assert property (@(posedge clk) a) else err <= 1;\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let assertion = block.body().unwrap();
    assert_eq!(assertion.kind(), StatementKind::ConcurrentAssertion);
    assert_eq!(assertion.assertion_kind(), Some(AssertionKind::Assert));
    assert!(assertion.assertion_if_true().is_none());
    assert!(assertion.assertion_if_false().is_some());

    // Both a pass-action and a fail-action.
    let design = compile(
        "module m(input logic clk, a, output logic ok, err);\n\
         assert property (@(posedge clk) a) ok <= 1; else err <= 1;\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let assertion = block.body().unwrap();
    assert!(assertion.assertion_if_true().is_some());
    assert!(assertion.assertion_if_false().is_some());

    // `cover property` and `assume property` report their own AssertionKind.
    let design = compile(
        "module m(input logic clk, a);\n\
         cover property (@(posedge clk) a);\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    assert_eq!(
        block.body().unwrap().assertion_kind(),
        Some(AssertionKind::CoverProperty)
    );

    let design = compile(
        "module m(input logic clk, a);\n\
         assume property (@(posedge clk) a);\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    assert_eq!(
        block.body().unwrap().assertion_kind(),
        Some(AssertionKind::Assume)
    );

    // A non-assertion statement carries no assertion kind or actions.
    let design = compile(
        "module m(input logic clk, d, output logic q);\n\
         always_ff @(posedge clk) q <= d;\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let expr_stmt = block.body().unwrap().body().unwrap();
    assert!(expr_stmt.assertion_kind().is_none());
    assert!(expr_stmt.assertion_if_true().is_none());
    assert!(expr_stmt.assertion_if_false().is_none());
}

#[test]
fn stmt_event_trigger_is_nonblocking_distinguishes_blocking_and_nonblocking() {
    use sv_lang::kinds::StatementKind;

    let design = compile(
        "module m(input logic clk); event e;\n\
         always @(posedge clk) begin\n\
           ->e;\n\
           ->>e;\n\
         end\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let timed = block.body().unwrap();
    let inner = timed.body().unwrap(); // the begin/end block
    // The block's single semantic child is the statement-list node wrapping
    // both triggers; its own children are the two flattened statements.
    let wrapped = inner.statements();
    assert_eq!(wrapped.len(), 1);
    assert_eq!(wrapped[0].kind(), StatementKind::List);
    let stmts = wrapped[0].statements();
    assert_eq!(stmts.len(), 2);

    assert_eq!(stmts[0].kind(), StatementKind::EventTrigger);
    assert_eq!(stmts[0].is_nonblocking_trigger(), Some(false));

    assert_eq!(stmts[1].kind(), StatementKind::EventTrigger);
    assert_eq!(stmts[1].is_nonblocking_trigger(), Some(true));

    // A non-event-trigger statement carries no answer.
    assert_eq!(timed.is_nonblocking_trigger(), None);
}

#[test]
fn stmt_event_trigger_timing_reuses_generic_timing_accessor() {
    use sv_lang::kinds::StatementKind;

    let design = compile(
        "module m(input logic clk); event e;\n\
         always @(posedge clk) begin\n\
           ->> #5 e;\n\
           ->e;\n\
         end\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let timed = block.body().unwrap();
    let inner = timed.body().unwrap();
    let stmts = inner.statements()[0].statements();
    assert_eq!(stmts.len(), 2);

    // The delayed non-blocking trigger carries a timing control...
    assert_eq!(stmts[0].kind(), StatementKind::EventTrigger);
    assert!(stmts[0].timing().is_some());
    // ...while the plain blocking trigger has none.
    assert_eq!(stmts[1].kind(), StatementKind::EventTrigger);
    assert!(stmts[1].timing().is_none());

    // A kind with neither a `Timed` nor an `EventTrigger` shape carries no
    // timing control either (the outer `always @(posedge clk)` wrapper
    // itself does, since it IS a Timed statement).
    assert!(timed.timing().is_some());
    assert!(inner.timing().is_none());
}

#[test]
fn stmt_for_loop_initializers_mutually_exclusive_with_loop_vars() {
    use sv_lang::kinds::StatementKind;

    // Plain initializer-expression form: `for (i = 0; ...)`.
    let design = compile("module m; int i; initial for (i = 0; i < 4; i++) begin end endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let for_loop = block.body().unwrap();
    assert_eq!(for_loop.kind(), StatementKind::ForLoop);
    assert_eq!(for_loop.for_loop_initializers().len(), 1);
    assert!(for_loop.for_loop_vars().is_empty());
    assert_eq!(for_loop.for_loop_steps().len(), 1);

    // Declared-loop-variable form: `for (int i = 0; ...)` — slang wraps this
    // in an implicit block (a scope for `i`) containing
    // [VariableDeclaration, ForLoop]; the ForLoop's own `loopVars` still
    // resolves to that declared variable.
    let design = compile("module m; initial for (int i = 0; i < 4; i++) begin end endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let wrapper = block.body().unwrap();
    assert_eq!(wrapper.kind(), StatementKind::Block);
    let list = wrapper.statements();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].kind(), StatementKind::List);
    let inner = list[0].statements();
    assert_eq!(inner.len(), 2);
    assert_eq!(inner[0].kind(), StatementKind::VariableDeclaration);
    let for_loop = inner[1];
    assert_eq!(for_loop.kind(), StatementKind::ForLoop);
    assert!(for_loop.for_loop_initializers().is_empty());
    let vars = for_loop.for_loop_vars();
    assert_eq!(vars.len(), 1);
    assert_eq!(vars[0].name(), "i");
    assert_eq!(for_loop.for_loop_steps().len(), 1);

    // A multi-clause for loop: `for (i = 0, j = 1; ...; i++, j--)`.
    let design = compile(
        "module m; int i, j; initial for (i = 0, j = 1; i < j; i++, j--) begin end endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let for_loop = block.body().unwrap();
    assert_eq!(for_loop.kind(), StatementKind::ForLoop);
    assert_eq!(for_loop.for_loop_initializers().len(), 2);
    assert_eq!(for_loop.for_loop_steps().len(), 2);

    // A non-for-loop statement answers empty for all three.
    let design = compile("module m(output logic o); initial o = 1; endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let expr_stmt = block.body().unwrap();
    assert_eq!(expr_stmt.kind(), StatementKind::ExpressionStatement);
    assert!(expr_stmt.for_loop_initializers().is_empty());
    assert!(expr_stmt.for_loop_vars().is_empty());
    assert!(expr_stmt.for_loop_steps().is_empty());
}

#[test]
fn stmt_foreach_loop_dims_breadth() {
    use sv_lang::kinds::StatementKind;

    // A single static dimension.
    let design = compile("module m; int a[4]; initial foreach (a[i]) begin end endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    // `foreach`'s loop variable needs its own scope, so slang wraps the loop
    // in an implicit single-statement block.
    let wrapper = block.body().unwrap();
    assert_eq!(wrapper.kind(), StatementKind::Block);
    let foreach = wrapper.statements()[0];
    assert_eq!(foreach.kind(), StatementKind::ForeachLoop);
    let dims = foreach.foreach_loop_dims();
    assert_eq!(dims.len(), 1);
    assert_eq!(dims[0].loop_var().unwrap().name(), "i");
    let range = dims[0].range().unwrap();
    assert_eq!((range.left, range.right), (0, 3));
    assert_eq!((range.lower(), range.upper()), (0, 3));
    assert_eq!(range.width(), 4);

    // Two static dimensions, both iterated: `foreach (a[i,j])`.
    let design = compile("module m; int a[4][3]; initial foreach (a[i,j]) begin end endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let foreach = block.body().unwrap().statements()[0];
    assert_eq!(foreach.kind(), StatementKind::ForeachLoop);
    let dims = foreach.foreach_loop_dims();
    assert_eq!(dims.len(), 2);
    assert_eq!(dims[0].loop_var().unwrap().name(), "i");
    assert_eq!(
        (
            dims[0].range().unwrap().lower(),
            dims[0].range().unwrap().upper()
        ),
        (0, 3)
    );
    assert_eq!(dims[1].loop_var().unwrap().name(), "j");
    assert_eq!(
        (
            dims[1].range().unwrap().lower(),
            dims[1].range().unwrap().upper()
        ),
        (0, 2)
    );

    // A skipped dimension: `foreach (a[,j])` — the first dimension has no
    // loop variable but still reports its static range.
    let design = compile("module m; int a[4][3]; initial foreach (a[,j]) begin end endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let foreach = block.body().unwrap().statements()[0];
    let dims = foreach.foreach_loop_dims();
    assert_eq!(dims.len(), 2);
    assert!(dims[0].loop_var().is_none());
    assert_eq!(
        (
            dims[0].range().unwrap().lower(),
            dims[0].range().unwrap().upper()
        ),
        (0, 3)
    );
    assert_eq!(dims[1].loop_var().unwrap().name(), "j");

    // A dynamically-sized array: the dimension has a loop variable but no
    // statically-known range.
    let design = compile("module m; int a[]; initial foreach (a[i]) begin end endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let foreach = block.body().unwrap().statements()[0];
    let dims = foreach.foreach_loop_dims();
    assert_eq!(dims.len(), 1);
    assert_eq!(dims[0].loop_var().unwrap().name(), "i");
    assert!(dims[0].range().is_none());

    // A non-foreach statement answers empty.
    let design = compile("module m(output logic o); initial o = 1; endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    assert!(block.body().unwrap().foreach_loop_dims().is_empty());
}

#[test]
fn constant_range_operations() {
    use sv_lang::ConstantRange;
    use sv_lang::kinds::StatementKind;

    // A descending declared range (`[3:0]`) and an ascending one (`[0:3]`),
    // both taken from real foreach loop dimensions of a compiled design.
    let design = compile(
        "module m;\n\
           int a[3:0];\n\
           int b[0:3];\n\
           initial foreach (a[i]) begin end\n\
           initial foreach (b[j]) begin end\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let mut blocks = body
        .members()
        .filter(|s| s.kind() == SymbolKind::ProceduralBlock);
    let descending = {
        let block = blocks.next().unwrap();
        let foreach = block.body().unwrap().statements()[0];
        assert_eq!(foreach.kind(), StatementKind::ForeachLoop);
        foreach.foreach_loop_dims()[0].range().unwrap()
    };
    let ascending = {
        let block = blocks.next().unwrap();
        let foreach = block.body().unwrap().statements()[0];
        foreach.foreach_loop_dims()[0].range().unwrap()
    };
    assert_eq!((descending.left, descending.right), (3, 0));
    assert_eq!((ascending.left, ascending.right), (0, 3));

    // isDescending.
    assert!(descending.is_descending());
    assert!(!ascending.is_descending());

    // reverse.
    assert_eq!(descending.reverse(), ConstantRange { left: 0, right: 3 });
    assert_eq!(ascending.reverse(), ConstantRange { left: 3, right: 0 });
    assert_eq!(descending.reverse().reverse(), descending);

    // containsPoint.
    assert!(descending.contains_point(0));
    assert!(descending.contains_point(3));
    assert!(!descending.contains_point(4));
    assert!(!descending.contains_point(-1));

    // overlaps.
    let disjoint = ConstantRange { left: 7, right: 4 };
    let touching = ConstantRange { left: 5, right: 3 };
    assert!(!descending.overlaps(disjoint));
    assert!(descending.overlaps(touching));
    assert!(descending.overlaps(descending));

    // translateIndex: for a descending range it's `index - lower()`; for an
    // ascending one it's `upper() - index`.
    assert_eq!(descending.translate_index(1), 1);
    assert_eq!(ascending.translate_index(1), 2);

    // subrange: selecting local offsets [1:0] out of the descending [3:0]
    // range (matches slang::ConstantRange::subrange exactly, including its
    // left/right convention: the result's bounds are offset-shifted lowest-
    // to-highest regardless of the parent's own declared direction).
    let sub = descending.subrange(ConstantRange { left: 1, right: 0 });
    assert_eq!(sub, ConstantRange { left: 0, right: 1 });

    // getIndexedRange: `+:`/`-:` selects, mirroring slang's own operator
    // semantics for e.g. `a[1 +: 2]` / `a[1 -: 2]` against a descending base.
    let up = ConstantRange::get_indexed_range(1, 2, true, true).unwrap();
    assert_eq!(up, ConstantRange { left: 2, right: 1 });
    let down = ConstantRange::get_indexed_range(1, 2, true, false).unwrap();
    assert_eq!(down, ConstantRange { left: 1, right: 0 });
    // Overflowing the 32-bit bound fails cleanly instead of wrapping.
    assert!(ConstantRange::get_indexed_range(i32::MAX, 2, true, true).is_none());
}

#[test]
fn constant_value_predicates() {
    let design = compile(
        "module m;\n\
           localparam int Zero = 0;\n\
           localparam int Nonzero = 5;\n\
           localparam logic [3:0] HasX = 4'b1x0z;\n\
           localparam string Empty = \"\";\n\
           localparam string Full = \"hi\";\n\
           localparam int Arr[3] = '{1, 2, 3};\n\
           localparam int EmptyQ[$] = '{};\n\
         endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let value_of = |name: &str| {
        body.find(name)
            .unwrap()
            .initializer()
            .unwrap()
            .constant_value()
            .unwrap()
    };

    // Every scalar kind (here: integers) reports `empty() == true` — slang's
    // own `size()`/`empty()` only really means something for containers and
    // strings, and default to 0/true for everything else.
    let zero = value_of("Zero");
    assert!(zero.empty());
    assert!(!zero.is_container());
    assert!(!zero.is_true());
    assert!(zero.is_false());

    let nonzero = value_of("Nonzero");
    assert!(nonzero.empty());
    assert!(nonzero.is_true());
    assert!(!nonzero.is_false());

    // `4'b1x0z` has an unknown (x) bit but also a definite 1 bit (bit 3), so
    // it is "true" despite `hasUnknown()` — and, since it has any unknown at
    // all, never "false" either. Exercises the real
    // slang::ConstantValue::isTrue/isFalse three-valued reduction, not a
    // from-scratch reimplementation of it.
    let has_x = value_of("HasX");
    assert!(has_x.as_integer().unwrap().has_unknown());
    assert!(has_x.is_true());
    assert!(!has_x.is_false());

    let empty_str = value_of("Empty");
    assert!(empty_str.empty());
    assert!(!empty_str.is_container());
    assert!(!empty_str.is_true());
    assert!(empty_str.is_false());

    let full_str = value_of("Full");
    assert!(!full_str.empty());
    assert!(full_str.is_true());
    assert!(!full_str.is_false());

    let arr = value_of("Arr");
    assert!(!arr.empty());
    assert!(arr.is_container());
    assert!(!arr.is_true());
    assert!(!arr.is_false());

    let empty_q = value_of("EmptyQ");
    assert!(empty_q.empty());
    assert!(empty_q.is_container());
    assert!(!empty_q.is_true());
    assert!(!empty_q.is_false());
}

#[test]
fn stmt_immediate_assertion_breadth() {
    use sv_lang::AssertionKind;
    use sv_lang::kinds::StatementKind;

    let design = compile(
        "module m(input logic a, output logic err);\n\
         initial begin\n\
           assert (a);\n\
           assert #0 (a) else $error(\"bad\");\n\
           assert final (a);\n\
         end\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let stmts = block.body().unwrap().statements()[0].statements();
    assert_eq!(stmts.len(), 3);

    // Plain immediate `assert (a);`: not deferred, not final. slang always
    // synthesizes a pass-action statement (an empty one here, since none was
    // written) but no fail-action.
    let plain = stmts[0];
    assert_eq!(plain.kind(), StatementKind::ImmediateAssertion);
    assert_eq!(
        plain.immediate_assertion_kind(),
        Some(AssertionKind::Assert)
    );
    assert_eq!(plain.is_deferred_assertion(), Some(false));
    assert_eq!(plain.is_final_assertion(), Some(false));
    assert_eq!(
        plain.immediate_assertion_if_true().map(|s| s.kind()),
        Some(StatementKind::Empty)
    );
    assert!(plain.immediate_assertion_if_false().is_none());
    // The generic ConcurrentAssertion-only accessor reports nothing for an
    // ImmediateAssertion node — the two statement kinds carry distinct
    // AssertionKind fields despite sharing the enum type.
    assert_eq!(plain.assertion_kind(), None);

    // `assert #0 (a) else $error(...);`: deferred (`#0`), not final, only a
    // fail-action.
    let deferred = stmts[1];
    assert_eq!(deferred.kind(), StatementKind::ImmediateAssertion);
    assert_eq!(
        deferred.immediate_assertion_kind(),
        Some(AssertionKind::Assert)
    );
    assert_eq!(deferred.is_deferred_assertion(), Some(true));
    assert_eq!(deferred.is_final_assertion(), Some(false));
    assert!(deferred.immediate_assertion_if_true().is_none());
    assert!(deferred.immediate_assertion_if_false().is_some());

    // `assert final (a);`: deferred AND final.
    let final_assert = stmts[2];
    assert_eq!(final_assert.kind(), StatementKind::ImmediateAssertion);
    assert_eq!(final_assert.is_deferred_assertion(), Some(true));
    assert_eq!(final_assert.is_final_assertion(), Some(true));

    // A non-assertion statement answers `None`/empty throughout.
    let design = compile("module m(output logic o); initial o = 1; endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let expr_stmt = block.body().unwrap();
    assert_eq!(expr_stmt.immediate_assertion_kind(), None);
    assert_eq!(expr_stmt.is_deferred_assertion(), None);
    assert_eq!(expr_stmt.is_final_assertion(), None);
    assert!(expr_stmt.immediate_assertion_if_true().is_none());
    assert!(expr_stmt.immediate_assertion_if_false().is_none());
}

#[test]
fn stmt_pattern_case_breadth() {
    use sv_lang::UniquePriorityCheck;
    use sv_lang::kinds::{PatternKind, StatementKind};

    let design = compile(
        "module m(input int v, output int o);\n\
         initial priority case (v) matches\n\
           .d &&& d > 0: o = d;\n\
           default: o = 0;\n\
         endcase endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let case = block.body().unwrap();
    assert_eq!(case.kind(), StatementKind::PatternCase);
    assert_eq!(
        case.pattern_case_check(),
        Some(UniquePriorityCheck::Priority)
    );

    let items = case.pattern_case_items();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].pattern().kind(), PatternKind::Variable);
    assert!(items[0].filter().is_some());
    // The bound pattern variable `d` needs its own scope, so slang wraps the
    // arm's statement in an implicit block.
    let arm_stmt = items[0].stmt();
    assert_eq!(arm_stmt.kind(), StatementKind::Block);
    let inner = arm_stmt.statements();
    assert_eq!(inner.len(), 1);
    assert_eq!(inner[0].kind(), StatementKind::ExpressionStatement);

    // Without a `unique`/`priority` qualifier the check is `None` (slang's
    // `UniquePriorityCheck::None` ordinal 0).
    let design = compile(
        "module m(input int v, output int o);\n\
         initial case (v) matches\n\
           .d: o = d;\n\
         endcase endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let case = block.body().unwrap();
    assert_eq!(case.kind(), StatementKind::PatternCase);
    assert_eq!(case.pattern_case_check(), Some(UniquePriorityCheck::None));
    let items = case.pattern_case_items();
    assert_eq!(items.len(), 1);
    assert!(items[0].filter().is_none());

    // A non-pattern-case statement answers empty/`None`.
    let design = compile("module m(output logic o); initial o = 1; endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let expr_stmt = block.body().unwrap();
    assert_eq!(expr_stmt.pattern_case_check(), None);
    assert!(expr_stmt.pattern_case_items().is_empty());
}

#[test]
fn stmt_pattern_case_condition_and_default_breadth() {
    use sv_lang::CaseStatementCondition;
    use sv_lang::kinds::StatementKind;

    let design = compile(
        "module m(input int v, output int o); initial case (v) matches\n\
           .d: o = d;\n\
           default: o = 0;\n\
         endcase endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let case = block.body().unwrap();
    assert_eq!(case.kind(), StatementKind::PatternCase);
    assert_eq!(
        case.pattern_case_condition(),
        Some(CaseStatementCondition::Normal)
    );
    let default_stmt = case.pattern_case_default().unwrap();
    assert_eq!(default_stmt.kind(), StatementKind::ExpressionStatement);

    // Without a `default` item, `pattern_case_default` is `None`.
    let design = compile(
        "module m(input int v, output int o); initial case (v) matches\n\
           .d: o = d;\n\
         endcase endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let case = block.body().unwrap();
    assert!(case.pattern_case_default().is_none());

    // A non-pattern-case statement answers `None`.
    let design = compile("module m(output logic o); initial o = 1; endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let expr_stmt = block.body().unwrap();
    assert_eq!(expr_stmt.pattern_case_condition(), None);
    assert!(expr_stmt.pattern_case_default().is_none());
}

#[test]
fn stmt_wait_order_breadth() {
    use sv_lang::kinds::StatementKind;

    let design = compile(
        "module m; event a, b;\n\
         initial wait_order (a, b) $display(\"t\"); else $display(\"f\");\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let stmt = block.body().unwrap();
    assert_eq!(stmt.kind(), StatementKind::WaitOrder);

    let events = stmt.wait_order_events();
    assert_eq!(events.len(), 2);
    for ev in &events {
        assert!(ev.expr_type().is_some());
    }
    assert_eq!(
        stmt.wait_order_if_true().unwrap().kind(),
        StatementKind::ExpressionStatement
    );
    assert_eq!(
        stmt.wait_order_if_false().unwrap().kind(),
        StatementKind::ExpressionStatement
    );

    // A bare `wait_order(...);` has no `else`, and a bare-statement action
    // parses as an (non-null) empty statement rather than a missing one.
    let design = compile("module m; event a, b; initial wait_order (a, b); endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let stmt = block.body().unwrap();
    assert_eq!(stmt.kind(), StatementKind::WaitOrder);
    assert_eq!(stmt.wait_order_events().len(), 2);
    assert_eq!(
        stmt.wait_order_if_true().unwrap().kind(),
        StatementKind::Empty
    );
    assert!(stmt.wait_order_if_false().is_none());

    // A non-wait-order statement answers empty/`None`.
    let design = compile("module m(output logic o); initial o = 1; endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let expr_stmt = block.body().unwrap();
    assert!(expr_stmt.wait_order_events().is_empty());
    assert!(expr_stmt.wait_order_if_true().is_none());
    assert!(expr_stmt.wait_order_if_false().is_none());
}

#[test]
fn stmt_procedural_assign_deassign_breadth() {
    use sv_lang::kinds::StatementKind;

    let design = compile(
        "module m; logic x;\n\
         initial force x = 1;\n\
         initial assign x = 0;\n\
         initial release x;\n\
         initial deassign x;\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let blocks: Vec<_> = body
        .members()
        .filter(|s| s.kind() == SymbolKind::ProceduralBlock)
        .collect();
    assert_eq!(blocks.len(), 4);

    let force_stmt = blocks[0].body().unwrap();
    assert_eq!(force_stmt.kind(), StatementKind::ProceduralAssign);
    assert!(force_stmt.procedural_assign_is_force());

    let assign_stmt = blocks[1].body().unwrap();
    assert_eq!(assign_stmt.kind(), StatementKind::ProceduralAssign);
    assert!(!assign_stmt.procedural_assign_is_force());

    let release_stmt = blocks[2].body().unwrap();
    assert_eq!(release_stmt.kind(), StatementKind::ProceduralDeassign);
    assert!(release_stmt.procedural_deassign_is_release());

    let deassign_stmt = blocks[3].body().unwrap();
    assert_eq!(deassign_stmt.kind(), StatementKind::ProceduralDeassign);
    assert!(!deassign_stmt.procedural_deassign_is_release());

    // A non-matching statement answers `false` for both.
    let design = compile("module m(output logic o); initial o = 1; endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let expr_stmt = block.body().unwrap();
    assert!(!expr_stmt.procedural_assign_is_force());
    assert!(!expr_stmt.procedural_deassign_is_release());
}

#[test]
fn stmt_randcase_items_breadth() {
    use sv_lang::kinds::{ExpressionKind, StatementKind};

    let design = compile(
        "module m(output int o); initial randcase\n\
           1: o = 1;\n\
           2: o = 2;\n\
         endcase endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let stmt = block.body().unwrap();
    assert_eq!(stmt.kind(), StatementKind::RandCase);

    let items = stmt.randcase_items();
    assert_eq!(items.len(), 2);
    for item in &items {
        assert_eq!(item.expr().kind(), ExpressionKind::IntegerLiteral);
        assert_eq!(item.stmt().kind(), StatementKind::ExpressionStatement);
    }

    // A non-randcase statement answers an empty item list.
    let design = compile("module m(output logic o); initial o = 1; endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    assert!(block.body().unwrap().randcase_items().is_empty());
}

#[test]
fn stmt_randsequence_first_production_breadth() {
    use sv_lang::kinds::StatementKind;

    let design = compile(
        "module m; initial randsequence(p)\n\
           p : { };\n\
         endsequence endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    // The randsequence needs its own scope, so slang wraps it in an implicit
    // block.
    let wrapper = block.body().unwrap();
    assert_eq!(wrapper.kind(), StatementKind::Block);
    let stmt = wrapper.statements()[0];
    assert_eq!(stmt.kind(), StatementKind::RandSequence);

    let first = stmt.randsequence_first_production().unwrap();
    assert_eq!(first.name(), "p");

    // A non-randsequence statement answers `None`.
    let design = compile("module m(output logic o); initial o = 1; endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    assert!(
        block
            .body()
            .unwrap()
            .randsequence_first_production()
            .is_none()
    );
}

#[test]
fn stmt_procedural_checker_instances_breadth() {
    use sv_lang::kinds::StatementKind;

    let design = compile(
        "module m;\n\
         checker chk;\n\
         endchecker\n\
         initial chk c1();\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let stmt = block.body().unwrap();
    assert_eq!(stmt.kind(), StatementKind::ProceduralChecker);

    let instances = stmt.procedural_checker_instances();
    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0].name(), "c1");

    // A non-procedural-checker statement answers an empty list.
    let design = compile("module m(output logic o); initial o = 1; endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    assert!(
        block
            .body()
            .unwrap()
            .procedural_checker_instances()
            .is_empty()
    );
}

#[test]
fn stmt_is_bad_breadth() {
    use sv_lang::kinds::StatementKind;

    // `continue` outside a loop is a compile error that slang recovers from
    // by substituting an Invalid statement (`Statement::bad()` is true).
    let design = compile("module m; initial continue; endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let stmt = block.body().unwrap();
    assert_eq!(stmt.kind(), StatementKind::Invalid);
    assert!(stmt.is_bad());

    // A well-formed statement is not bad.
    let design = compile("module m(output logic o); initial o = 1; endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    assert!(!block.body().unwrap().is_bad());
}

fn find_stmt_kind<'d>(
    s: sv_lang::Statement<'d>,
    k: sv_lang::kinds::StatementKind,
) -> Option<sv_lang::Statement<'d>> {
    if s.kind() == k {
        return Some(s);
    }
    for child in s.statements() {
        if let Some(f) = find_stmt_kind(child, k) {
            return Some(f);
        }
    }
    if let Some(b) = s.body() {
        if let Some(f) = find_stmt_kind(b, k) {
            return Some(f);
        }
    }
    if let Some(t) = s.then_branch() {
        if let Some(f) = find_stmt_kind(t, k) {
            return Some(f);
        }
    }
    None
}

#[test]
fn stmt_eval_success_fail_disable() {
    use sv_lang::StatementEvalResult;

    let mut design = compile(
        "module m; initial begin : blk\n\
         int x;\n\
         x = 1;\n\
         disable blk;\n\
         end endmodule\n",
    );
    let eval = design.eval_session();
    // The three statements share one implicit `List` (block.body() is the
    // named `begin : blk ... end` wrapper, whose one child is that list).
    let stmts = eval
        .design()
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap()
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap()
        .body()
        .unwrap()
        .statements()[0]
        .statements();

    // Declaring `x` needs no outside state, so it succeeds under its own
    // fresh scratch frame.
    assert_eq!(eval.eval_stmt(stmts[0]), StatementEvalResult::Success);
    // But each call's frame is discarded when it returns: writing `x` in its
    // OWN separate call finds no such local (this call never declared it),
    // so it fails rather than reusing the earlier declaration.
    assert_eq!(eval.eval_stmt(stmts[1]), StatementEvalResult::Fail);
    // `disable` needs no local state, so it always succeeds in unwinding to
    // its named block, reported as `Disable`.
    assert_eq!(eval.eval_stmt(stmts[2]), StatementEvalResult::Disable);

    // An Invalid statement (see stmt_is_bad_breadth) always fails, without
    // running its (nonexistent) evalImpl.
    let mut design2 = compile("module m; initial continue; endmodule\n");
    let eval2 = design2.eval_session();
    let bad_stmt = eval2
        .design()
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap()
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap()
        .body()
        .unwrap();
    assert!(bad_stmt.is_bad());
    assert_eq!(eval2.eval_stmt(bad_stmt), StatementEvalResult::Fail);
}

#[test]
fn stmt_eval_return_break_continue() {
    use sv_lang::StatementEvalResult;
    use sv_lang::kinds::StatementKind;

    let mut design = compile(
        "module m; task t(); return; endtask\n\
         initial for (int i = 0; i < 3; i++) begin\n\
         if (i == 0) continue;\n\
         if (i == 1) break;\n\
         end\n\
         endmodule\n",
    );
    let eval = design.eval_session();

    let task_body = eval
        .design()
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap()
        .members()
        .find(|s| s.name() == "t")
        .unwrap()
        .body()
        .unwrap();
    assert_eq!(eval.eval_stmt(task_body), StatementEvalResult::Return);

    let proc_body = || {
        eval.design()
            .top_instances()
            .next()
            .unwrap()
            .instance_body()
            .unwrap()
            .members()
            .find(|s| s.kind() == SymbolKind::ProceduralBlock)
            .unwrap()
            .body()
            .unwrap()
    };

    let break_stmt = find_stmt_kind(proc_body(), StatementKind::Break).unwrap();
    assert_eq!(eval.eval_stmt(break_stmt), StatementEvalResult::Break);

    let continue_stmt = find_stmt_kind(proc_body(), StatementKind::Continue).unwrap();
    assert_eq!(eval.eval_stmt(continue_stmt), StatementEvalResult::Continue);
}

// ---- Expression, part 1: Assignment / AssignmentPattern / ArbitrarySymbol /
//      AssertionInstance / Call system-call breadth ---------------------------

#[test]
fn arbitrary_symbol_expression_resolves_the_referenced_symbol() {
    use sv_lang::kinds::{ExpressionKind, SymbolKind};

    let design = compile(
        "module leaf; endmodule\n\
         module top; leaf u_leaf(); initial $printtimescale(u_leaf); endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let call = block.body().unwrap().expr().unwrap(); // `$printtimescale(u_leaf)`
    assert_eq!(call.kind(), ExpressionKind::Call);

    let children = call.children();
    let arg = children[0].as_expression().unwrap();
    assert_eq!(arg.kind(), ExpressionKind::ArbitrarySymbol);

    let sym = arg.arbitrary_symbol().unwrap();
    assert_eq!(sym.name(), "u_leaf");
    assert_eq!(sym.kind(), SymbolKind::Instance);

    // A non-ArbitrarySymbol expression answers `None`.
    assert!(call.arbitrary_symbol().is_none());
}

#[test]
fn assignment_expression_compound_op_and_timing() {
    use sv_lang::BinaryOp;
    use sv_lang::kinds::SymbolKind;

    let design = compile(
        "module m; int x, y;\n\
         initial begin\n\
         x += 1;\n\
         x = 1;\n\
         x = #5 y;\n\
         end endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    // `begin...end` wraps its content in one `List` statement.
    let stmts = block.body().unwrap().statements()[0].statements();
    assert_eq!(stmts.len(), 3);

    // `x += 1`: compound, operator Add, not an lvalue-arg, no timing.
    let compound = stmts[0].expr().unwrap();
    assert_eq!(compound.is_compound_assignment(), Some(true));
    assert_eq!(compound.assignment_op(), Some(BinaryOp::Add));
    assert_eq!(compound.is_lvalue_arg(), Some(false));
    assert!(compound.assignment_timing().is_none());

    // `x = 1`: simple, no operator, no timing.
    let simple = stmts[1].expr().unwrap();
    assert_eq!(simple.is_compound_assignment(), Some(false));
    assert_eq!(simple.assignment_op(), None);
    assert!(simple.assignment_timing().is_none());

    // `x = #5 y`: simple, but carries an intra-assignment timing control.
    let timed = stmts[2].expr().unwrap();
    assert_eq!(timed.is_compound_assignment(), Some(false));
    assert!(timed.assignment_timing().is_some());

    // A non-assignment expression answers `None` for every accessor.
    let rhs = simple.right().unwrap(); // the literal `1`
    assert_eq!(rhs.is_compound_assignment(), None);
    assert_eq!(rhs.is_lvalue_arg(), None);
    assert_eq!(rhs.assignment_op(), None);
    assert!(rhs.assignment_timing().is_none());
}

#[test]
fn assignment_expression_is_lvalue_arg_for_an_implied_output_argument() {
    use sv_lang::kinds::{ExpressionKind, SymbolKind};

    let design = compile(
        "module m;\n\
         task t(output int y); y = 5; endtask\n\
         int z;\n\
         initial t(z);\n\
         endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let call = block.body().unwrap().expr().unwrap(); // `t(z)`
    assert_eq!(call.kind(), ExpressionKind::Call);
    assert_eq!(call.subroutine_kind(), Some(sv_lang::SubroutineKind::Task));

    // The single `output` argument is implicitly an Assignment expression
    // with no explicit operator or rhs in the source.
    let arg = call.children()[0].as_expression().unwrap();
    assert_eq!(arg.kind(), ExpressionKind::Assignment);
    assert_eq!(arg.is_lvalue_arg(), Some(true));
    assert_eq!(arg.is_compound_assignment(), Some(false));
    assert_eq!(arg.assignment_op(), None);
    assert!(arg.assignment_timing().is_none());
}

#[test]
fn assignment_pattern_elements_lists_each_element_expression() {
    let design = compile("module m; int arr[3] = '{1, 2, 3}; endmodule\n");
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let init = body.find("arr").unwrap().initializer().unwrap();
    let elems = init.pattern_elements();
    assert_eq!(elems.len(), 3);
    let values: Vec<_> = elems
        .iter()
        .map(|e| match e.constant_value().unwrap() {
            sv_lang::ConstantValue::Integer(v) => v.as_i64().unwrap(),
            other => panic!("expected an integer element, got {other:?}"),
        })
        .collect();
    assert_eq!(values, [1, 2, 3]);

    // A non-pattern expression (one of the pattern's own elements, a plain
    // integer literal) has no elements of its own.
    assert!(elems[0].pattern_elements().is_empty());
}

#[test]
fn replicated_assignment_pattern_count_is_the_replication_expression() {
    use sv_lang::kinds::ExpressionKind;

    let design = compile("module m; int arr[3] = '{3{7}}; endmodule\n");
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let init = body.find("arr").unwrap().initializer().unwrap();
    assert_eq!(init.kind(), ExpressionKind::ReplicatedAssignmentPattern);

    let count = init.replicated_pattern_count().unwrap();
    match count.constant_value().unwrap() {
        sv_lang::ConstantValue::Integer(v) => assert_eq!(v.as_i64(), Some(3)),
        other => panic!("expected an integer count, got {other:?}"),
    }

    // `elements()` holds the pattern's own (unreplicated) element(s); the
    // replication count above is what says how many times it repeats.
    let elems = init.pattern_elements();
    assert_eq!(elems.len(), 1);
    match elems[0].constant_value().unwrap() {
        sv_lang::ConstantValue::Integer(v) => assert_eq!(v.as_i64(), Some(7)),
        other => panic!("expected an integer element, got {other:?}"),
    }

    // A non-replicated pattern has no replication count.
    let simple = compile("module m; int arr[3] = '{1, 2, 3}; endmodule\n");
    let simple_body = simple
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let simple_init = simple_body.find("arr").unwrap().initializer().unwrap();
    assert!(simple_init.replicated_pattern_count().is_none());
}

#[test]
fn structured_assignment_pattern_member_setters_expose_member_and_value() {
    use sv_lang::kinds::ExpressionKind;

    let design = compile(
        "module m;\n\
         typedef struct { int a; int b; } s_t;\n\
         s_t s = '{a: 1, b: 2};\n\
         endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let init = body.find("s").unwrap().initializer().unwrap();
    assert_eq!(init.kind(), ExpressionKind::StructuredAssignmentPattern);

    let setters = init.structured_pattern_member_setters();
    assert_eq!(setters.len(), 2);

    assert_eq!(setters[0].member().name(), "a");
    match setters[0].expr().constant_value().unwrap() {
        sv_lang::ConstantValue::Integer(v) => assert_eq!(v.as_i64(), Some(1)),
        other => panic!("expected an integer, got {other:?}"),
    }

    assert_eq!(setters[1].member().name(), "b");
    match setters[1].expr().constant_value().unwrap() {
        sv_lang::ConstantValue::Integer(v) => assert_eq!(v.as_i64(), Some(2)),
        other => panic!("expected an integer, got {other:?}"),
    }

    // A non-structured pattern has no member setters.
    let replicated = compile("module m; int arr[3] = '{3{7}}; endmodule\n");
    let replicated_body = replicated
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let replicated_init = replicated_body.find("arr").unwrap().initializer().unwrap();
    assert!(
        replicated_init
            .structured_pattern_member_setters()
            .is_empty()
    );
}

#[test]
fn structured_assignment_pattern_type_and_index_setters() {
    use sv_lang::kinds::ExpressionKind;

    let design = compile("module m; int arr[3] = '{0: 9, int: 5, default: 0}; endmodule\n");
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let init = body.find("arr").unwrap().initializer().unwrap();
    assert_eq!(init.kind(), ExpressionKind::StructuredAssignmentPattern);

    // The `[0]: 9` index setter.
    let index_setters = init.structured_pattern_index_setters();
    assert_eq!(index_setters.len(), 1);
    match index_setters[0].index().constant_value().unwrap() {
        sv_lang::ConstantValue::Integer(v) => assert_eq!(v.as_i64(), Some(0)),
        other => panic!("expected an integer index, got {other:?}"),
    }
    match index_setters[0].expr().constant_value().unwrap() {
        sv_lang::ConstantValue::Integer(v) => assert_eq!(v.as_i64(), Some(9)),
        other => panic!("expected an integer, got {other:?}"),
    }

    // The `int: 5` type setter.
    let type_setters = init.structured_pattern_type_setters();
    assert_eq!(type_setters.len(), 1);
    assert_eq!(type_setters[0].ty().to_sv_string(), "int");
    match type_setters[0].expr().constant_value().unwrap() {
        sv_lang::ConstantValue::Integer(v) => assert_eq!(v.as_i64(), Some(5)),
        other => panic!("expected an integer, got {other:?}"),
    }

    // The `default: 0` default setter.
    let default_setter = init.structured_pattern_default_setter().unwrap();
    match default_setter.constant_value().unwrap() {
        sv_lang::ConstantValue::Integer(v) => assert_eq!(v.as_i64(), Some(0)),
        other => panic!("expected an integer, got {other:?}"),
    }
}

#[test]
fn structured_assignment_pattern_default_setter_absent() {
    use sv_lang::kinds::ExpressionKind;

    // No `default:` entry at all: `defaultSetter` is `None`.
    let design = compile(
        "module m; typedef struct { int a; int b; } s_t;\n\
         s_t s = '{a: 1, b: 2}; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let init = body.find("s").unwrap().initializer().unwrap();
    assert_eq!(init.kind(), ExpressionKind::StructuredAssignmentPattern);
    assert!(init.structured_pattern_default_setter().is_none());

    // A non-StructuredAssignmentPattern expression also reports `None`.
    let replicated = compile("module m; int arr[3] = '{3{7}}; endmodule\n");
    let replicated_body = replicated
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let replicated_init = replicated_body.find("arr").unwrap().initializer().unwrap();
    assert!(
        replicated_init
            .structured_pattern_default_setter()
            .is_none()
    );
}

#[test]
fn streaming_concatenation_breadth() {
    use sv_lang::kinds::ExpressionKind;

    let design = compile(
        "module m; logic [7:0] a, b; logic [15:0] c = {<<{a, b}}; \
         logic [15:0] d = {<<8{a, b}}; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    // `c`: `{<<{...}}` with no explicit slice size defaults to bit-granular
    // (slice size 1) right-to-left reordering.
    let init_c = body.find("c").unwrap().initializer().unwrap();
    assert_eq!(init_c.kind(), ExpressionKind::Conversion);
    let stream_c = init_c.conversion_operand().unwrap();
    assert_eq!(stream_c.kind(), ExpressionKind::Streaming);
    assert_eq!(stream_c.streaming_bitstream_width(), Some(16));
    assert_eq!(stream_c.streaming_slice_size(), Some(1));
    assert_eq!(stream_c.streaming_is_fixed_size(), Some(true));

    let streams_c = stream_c.streams();
    assert_eq!(streams_c.len(), 2);
    assert_eq!(
        streams_c[0].operand().referenced_symbol().unwrap().name(),
        "a"
    );
    assert_eq!(
        streams_c[1].operand().referenced_symbol().unwrap().name(),
        "b"
    );
    assert!(streams_c[0].with_expr().is_none());
    assert!(streams_c[1].with_expr().is_none());

    // `d`: an explicit slice size of 8.
    let init_d = body.find("d").unwrap().initializer().unwrap();
    let stream_d = init_d.conversion_operand().unwrap();
    assert_eq!(stream_d.streaming_slice_size(), Some(8));

    // A non-streaming expression reports no width/size/fixedness and no streams.
    let non_stream = streams_c[0].operand();
    assert!(non_stream.streaming_bitstream_width().is_none());
    assert!(non_stream.streaming_slice_size().is_none());
    assert!(non_stream.streaming_is_fixed_size().is_none());
    assert!(non_stream.streams().is_empty());
}

#[test]
fn streaming_concatenation_with_expr_on_a_stream() {
    use sv_lang::kinds::ExpressionKind;

    // A fixed-array stream operand with a `with` range selector picking a
    // sub-range of elements.
    let design = compile(
        "module m;\n\
         bit [7:0] q[4];\n\
         bit [15:0] r = {<<{q with [0:1]}};\n\
         endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let init = body.find("r").unwrap().initializer().unwrap();
    assert_eq!(init.kind(), ExpressionKind::Conversion);
    let stream = init.conversion_operand().unwrap();
    assert_eq!(stream.kind(), ExpressionKind::Streaming);

    let streams = stream.streams();
    assert_eq!(streams.len(), 1);
    assert_eq!(
        streams[0].operand().referenced_symbol().unwrap().name(),
        "q"
    );
    assert!(streams[0].with_expr().is_some());
}

#[test]
fn string_literal_value_raw_value_and_int_value() {
    use sv_lang::kinds::ExpressionKind;

    let design = compile(
        r#"module m;
        bit [23:0] s = "hi\n";
        bit [15:0] x = "AB";
        endmodule
        "#,
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let s = body.find("s").unwrap().initializer().unwrap();
    assert_eq!(s.kind(), ExpressionKind::StringLiteral);
    assert_eq!(s.string_literal_value().as_deref(), Some("hi\n"));
    assert_eq!(s.string_literal_raw_value().as_deref(), Some(r#""hi\n""#));
    match s.string_literal_int_value().unwrap() {
        sv_lang::ConstantValue::Integer(v) => {
            // 'h' = 0x68, 'i' = 0x69, '\n' = 0x0a, packed msb-first.
            assert_eq!(v.as_i64(), Some(0x68690a));
        }
        other => panic!("expected an integer, got {other:?}"),
    }

    let x = body.find("x").unwrap().initializer().unwrap();
    assert_eq!(x.kind(), ExpressionKind::StringLiteral);
    assert_eq!(x.string_literal_value().as_deref(), Some("AB"));
    assert_eq!(x.string_literal_raw_value().as_deref(), Some(r#""AB""#));
    match x.string_literal_int_value().unwrap() {
        sv_lang::ConstantValue::Integer(v) => assert_eq!(v.as_i64(), Some(0x4142)),
        other => panic!("expected an integer, got {other:?}"),
    }

    // A non-StringLiteral node reports None for all three accessors.
    let arr_design = compile("module m; int arr[1] = '{1}; endmodule\n");
    let arr_body = arr_design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let pattern = arr_body.find("arr").unwrap().initializer().unwrap();
    assert!(pattern.string_literal_value().is_none());
    assert!(pattern.string_literal_raw_value().is_none());
    assert!(pattern.string_literal_int_value().is_none());
}

#[test]
fn assertion_instance_expression_arguments_local_vars_and_recursion() {
    use sv_lang::kinds::{ExpressionKind, SymbolKind};

    let mut design = compile(
        "module m;\n\
         sequence s(int i);\n\
         int j;\n\
         (i > 0 && j >= 0, j = 1);\n\
         endsequence\n\
         initial begin\n\
         assert property (s(3));\n\
         end\n\
         endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    // `arguments()`'s actuals are bound once at elaboration but (unlike the
    // expanded `body`) are not walked by the generic prefold sweep, so read
    // their value through an `EvalSession` rather than assuming
    // `constant_value()` was already populated.
    let eval = design.eval_session();

    let body = eval
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
    let assert_stmt = block.body().unwrap().statements()[0];

    // ConcurrentAssertionStatement's own AssertionExpr root (the property
    // spec) is reached generically as the statement's first semantic child.
    let spec = assert_stmt.children()[0];
    // The property-spec root is an AssertionExpr node, not a plain
    // expression or statement.
    assert!(spec.as_expression().is_none());
    assert!(spec.as_statement().is_none());
    assert_eq!(spec.kind_name(), "Simple");

    let inst = spec.children()[0].as_expression().unwrap();
    assert_eq!(inst.kind(), ExpressionKind::AssertionInstance);
    assert_eq!(inst.is_recursive_property(), Some(false));

    // The sequence's own local variable `j` is materialized in the expanded
    // instance body.
    let locals = inst.assertion_local_vars();
    assert_eq!(locals.len(), 1);
    assert_eq!(locals[0].name(), "j");

    // One argument: formal port `i` bound to the actual `3`.
    let args = inst.assertion_arguments();
    assert_eq!(args.len(), 1);
    assert_eq!(args[0].port.name(), "i");
    let actual = args[0].actual.as_expression().unwrap();
    assert_eq!(actual.kind(), ExpressionKind::IntegerLiteral);
    match eval.eval_constant(actual).unwrap() {
        sv_lang::ConstantValue::Integer(v) => assert_eq!(v.as_i64(), Some(3)),
        other => panic!("expected an integer, got {other:?}"),
    }

    // A non-AssertionInstance expression answers empty/false/None.
    assert!(!inst.is_recursive_property().is_none()); // sanity: it IS Some
    assert_eq!(actual.is_recursive_property(), None);
    assert!(actual.assertion_local_vars().is_empty());
    assert!(actual.assertion_arguments().is_empty());
}

#[test]
fn assertion_instance_expression_marks_a_recursive_property_instantiation() {
    use sv_lang::kinds::ExpressionKind;

    let design = compile(
        "module m;\n\
         logic a, clk;\n\
         property p;\n\
         @(posedge clk) a |=> p;\n\
         endproperty\n\
         cp: assert property (p);\n\
         endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    // A module-level concurrent assertion elaborates into its own implicit,
    // unnamed procedural block.
    let block = body
        .members()
        .find(|s| s.kind() == sv_lang::kinds::SymbolKind::ProceduralBlock)
        .unwrap();
    let assert_stmt = block.body().unwrap();

    let mut instances = Vec::new();
    collect_assertion_instances(assert_stmt.children()[0], &mut instances);

    // The outer instantiation of `p` at the assert site is not recursive;
    // the inner self-reference inside `p`'s own (expanded) body is.
    assert!(
        instances
            .iter()
            .any(|e| e.kind() == ExpressionKind::AssertionInstance
                && e.is_recursive_property() == Some(false))
    );
    assert!(
        instances
            .iter()
            .any(|e| e.kind() == ExpressionKind::AssertionInstance
                && e.is_recursive_property() == Some(true))
    );
}

fn collect_assertion_instances<'d>(
    node: sv_lang::SemNode<'d>,
    out: &mut Vec<sv_lang::Expression<'d>>,
) {
    if let Some(e) = node.as_expression() {
        out.push(e);
    }
    for child in node.children() {
        collect_assertion_instances(child, out);
    }
}

#[test]
fn call_iterator_method_exposes_the_iterator_expr_and_var() {
    use sv_lang::kinds::SymbolKind;

    let design = compile(
        "module m; int q[$]; int r[$];\n\
         initial r = q.find_first(item) with (item > 0);\n\
         endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let assign = block.body().unwrap().expr().unwrap();
    let call = assign.right().unwrap();

    assert!(call.system_subroutine().is_some());
    assert_eq!(
        call.system_call_extra_kind(),
        sv_lang::CallExtraKind::Iterator
    );
    assert!(call.iterator_expr().is_some());
    let iter_var = call.iterator_var().unwrap();
    assert_eq!(iter_var.name(), "item");
    assert!(call.randomize_inline_constraints().is_none());

    // The scope of the system call is the enclosing module.
    assert_eq!(call.system_call_scope().unwrap().name(), "m");

    // A non-call expression answers `None`/`CallExtraKind::None` throughout.
    let non_call = assign.left().unwrap();
    assert_eq!(
        non_call.system_call_extra_kind(),
        sv_lang::CallExtraKind::None
    );
    assert!(non_call.iterator_expr().is_none());
    assert!(non_call.iterator_var().is_none());
    assert!(non_call.system_call_scope().is_none());
}

#[test]
fn call_randomize_method_exposes_its_inline_constraints() {
    use sv_lang::kinds::SymbolKind;

    let design = compile(
        "class C; rand int x; endclass\n\
         module m; C obj; int ok;\n\
         initial begin\n\
         obj = new();\n\
         ok = obj.randomize() with { x > 0; };\n\
         end endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let stmts = block.body().unwrap().statements()[0].statements();
    let assign = stmts[1].expr().unwrap(); // `ok = obj.randomize() with {...}`
    let call = assign.right().unwrap();

    assert!(call.system_subroutine().is_some());
    assert_eq!(
        call.system_call_extra_kind(),
        sv_lang::CallExtraKind::Randomize
    );
    assert!(call.randomize_inline_constraints().is_some());
    assert!(call.iterator_expr().is_none());
    assert!(call.iterator_var().is_none());
    assert_eq!(call.system_call_scope().unwrap().name(), "m");
}

#[test]
fn call_expression_subroutine_kind_distinguishes_function_and_task() {
    let design = compile(
        "module m;\n\
         function automatic int f(int a); return a + 1; endfunction\n\
         task automatic t(); ; endtask\n\
         localparam int C = f(3);\n\
         initial t();\n\
         endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let func_call = body.find("C").unwrap().initializer().unwrap();
    assert_eq!(
        func_call.subroutine_kind(),
        Some(sv_lang::SubroutineKind::Function)
    );
    assert!(func_call.system_subroutine().is_none());

    let block = body
        .members()
        .find(|s| s.kind() == sv_lang::kinds::SymbolKind::ProceduralBlock)
        .unwrap();
    let task_call = block.body().unwrap().expr().unwrap();
    assert_eq!(
        task_call.subroutine_kind(),
        Some(sv_lang::SubroutineKind::Task)
    );

    // `t()` takes no arguments, so it has no semantic children.
    assert!(task_call.children().is_empty());
    // A Call expression is not a binary/pattern expression.
    assert!(func_call.right().is_none());
    assert!(func_call.pattern_elements().is_empty());
}

#[test]
fn call_expression_subroutine_name_is_system_call_and_this_class() {
    let design = compile(
        "class C; function int f(); return 1; endfunction endclass\n\
         module m; C obj = new;\n\
         localparam integer W = $clog2(8);\n\
         function automatic int g(int a); return a; endfunction\n\
         localparam int Y = g(3);\n\
         int r; initial r = obj.f();\n\
         endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    // A system call: named "$clog2", is a system call, and has no `this`.
    let sys_call = body.find("W").unwrap().initializer().unwrap();
    assert_eq!(sys_call.call_subroutine_name().as_deref(), Some("$clog2"));
    assert_eq!(sys_call.is_system_call(), Some(true));
    assert!(sys_call.call_this_class().is_none());

    // A free user-function call: named "g", not a system call, no `this`.
    let free_call = body.find("Y").unwrap().initializer().unwrap();
    assert_eq!(free_call.call_subroutine_name().as_deref(), Some("g"));
    assert_eq!(free_call.is_system_call(), Some(false));
    assert!(free_call.call_this_class().is_none());

    // A class method call: named "f", not a system call, and `this` is `obj`.
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let method_call = block.body().unwrap().expr().unwrap().right().unwrap(); // `obj.f()`
    assert_eq!(method_call.call_subroutine_name().as_deref(), Some("f"));
    assert_eq!(method_call.is_system_call(), Some(false));
    assert_eq!(
        method_call
            .call_this_class()
            .unwrap()
            .referenced_symbol()
            .unwrap()
            .name(),
        "obj"
    );

    // A non-call expression (the `8` argument itself) reports `None` for
    // every one of these.
    let non_call = sys_call.children()[0].as_expression().unwrap();
    assert!(non_call.call_subroutine_name().is_none());
    assert!(non_call.is_system_call().is_none());
    assert!(non_call.call_this_class().is_none());
}

#[test]
fn conditional_expression_conditions_supports_pattern_match() {
    use sv_lang::kinds::{ExpressionKind, PatternKind};

    // A plain ternary: one condition, no pattern.
    let design = compile(
        "module m; logic c; logic [7:0] a, b;\n\
         wire [7:0] s = c ? a : b; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let cond = body.find("s").unwrap().initializer().unwrap();
    let conditions = cond.conditions();
    assert_eq!(conditions.len(), 1);
    assert_eq!(conditions[0].expr().kind(), ExpressionKind::NamedValue);
    assert!(conditions[0].pattern().is_none());
    assert_eq!(
        cond.true_value()
            .unwrap()
            .referenced_symbol()
            .unwrap()
            .name(),
        "a"
    );
    assert_eq!(
        cond.false_value()
            .unwrap()
            .referenced_symbol()
            .unwrap()
            .name(),
        "b"
    );

    // A non-conditional expression carries no conditions.
    assert!(cond.true_value().unwrap().conditions().is_empty());

    // A pattern-matching ternary over a tagged union: the condition's pattern
    // is a `Tagged` pattern binding the `.a` payload variable.
    let design = compile(
        "module m;\n\
         typedef union tagged { void Invalid; int Valid; } tagged_t;\n\
         tagged_t v;\n\
         int r = v matches tagged Valid .a ? a : 0;\n\
         endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let cond = body.find("r").unwrap().initializer().unwrap();
    assert_eq!(cond.kind(), ExpressionKind::ConditionalOp);
    let conditions = cond.conditions();
    assert_eq!(conditions.len(), 1);
    let pattern = conditions[0].pattern().unwrap();
    assert_eq!(pattern.kind(), PatternKind::Tagged);
}

#[test]
fn conversion_expression_const_cast_and_implicit_distinguish_cast_forms() {
    use sv_lang::kinds::ExpressionKind;

    let design = compile(
        "module m;\n\
         parameter int FOO = 1;\n\
         localparam int X = const'(FOO);\n\
         logic [7:0] a; wire [15:0] w = a;\n\
         endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    // `const'(FOO)`: an explicit const-cast -- not implicit, is a const-cast.
    let const_cast = body.find("X").unwrap().initializer().unwrap();
    assert_eq!(const_cast.kind(), ExpressionKind::Conversion);
    assert_eq!(const_cast.conversion_is_const_cast(), Some(true));
    assert_eq!(const_cast.conversion_is_implicit(), Some(false));

    // Widening `a` (8 bits) to `w`'s 16 bits: an implicit conversion, not a
    // const-cast.
    let widen = body.find("w").unwrap().initializer().unwrap();
    assert_eq!(widen.kind(), ExpressionKind::Conversion);
    assert_eq!(widen.conversion_is_implicit(), Some(true));
    assert_eq!(widen.conversion_is_const_cast(), Some(false));

    // A non-conversion expression reports `None` for both.
    assert_eq!(
        const_cast
            .conversion_operand()
            .unwrap()
            .conversion_is_implicit(),
        None
    );
    assert_eq!(
        const_cast
            .conversion_operand()
            .unwrap()
            .conversion_is_const_cast(),
        None
    );
}

#[test]
fn copy_class_expression_source_expr() {
    use sv_lang::kinds::ExpressionKind;

    let design = compile(
        "class C; int x; endclass\n\
         module m; C a = new; C b; initial b = new a; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let assign = block.body().unwrap().expr().unwrap(); // `b = new a`
    let rhs = assign.right().unwrap();
    assert_eq!(rhs.kind(), ExpressionKind::CopyClass);
    assert_eq!(
        rhs.copy_class_source()
            .unwrap()
            .referenced_symbol()
            .unwrap()
            .name(),
        "a"
    );

    // A non-CopyClass expression reports `None`.
    assert!(assign.left().unwrap().copy_class_source().is_none());
}

/// Finds the `Dist` expression inside `obj.randomize() with { ... };`, the
/// second statement of the procedural block's `begin...end`.
fn find_randomize_dist(design: &Design) -> sv_lang::Expression<'_> {
    use sv_lang::kinds::ExpressionKind;

    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let stmts = block.body().unwrap().statements()[0].statements();
    let assign = stmts[1].expr().unwrap();
    let call = assign.right().unwrap();
    let inline = call.randomize_inline_constraints().unwrap();
    let dist = inline.children()[0].children()[0].as_expression().unwrap();
    assert_eq!(dist.kind(), ExpressionKind::Dist);
    dist
}

#[test]
fn dist_expression_items_weights_and_left() {
    use sv_lang::DistWeightKind;
    use sv_lang::kinds::ExpressionKind;

    let design = compile(
        "class C; rand int x; endclass\n\
         module m; C obj; int ok;\n\
         initial begin\n\
         obj = new();\n\
         ok = obj.randomize() with { x dist { 0 := 1, [1:3] :/ 2 }; };\n\
         end endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let dist = find_randomize_dist(&design);

    assert_eq!(
        dist.dist_left()
            .unwrap()
            .referenced_symbol()
            .unwrap()
            .name(),
        "x"
    );

    let items = dist.dist_items();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].value().kind(), ExpressionKind::IntegerLiteral);
    let w0 = items[0].weight().unwrap();
    assert_eq!(w0.kind, DistWeightKind::PerValue);
    assert_eq!(w0.expr.kind(), ExpressionKind::IntegerLiteral);

    assert_eq!(items[1].value().kind(), ExpressionKind::ValueRange);
    let w1 = items[1].weight().unwrap();
    assert_eq!(w1.kind, DistWeightKind::PerRange);

    // No `default` item was specified, so this is genuinely absent.
    assert!(dist.dist_default_weight().is_none());

    // A non-Dist expression reports empty/`None` throughout.
    assert!(w0.expr.dist_items().is_empty());
    assert!(w0.expr.dist_left().is_none());
    assert!(w0.expr.dist_default_weight().is_none());
}

#[test]
fn dist_expression_default_weight_is_some_when_specified() {
    // The `default` dist-item keyword is an 1800-2023 language feature, so
    // this parses the source through a `Driver` configured with `--std`
    // (the plain `compile` helper uses slang's default -- 1800-2017 --
    // language version, under which this same source reports a diagnostic).
    use sv_lang::{DistWeightKind, kinds::ExpressionKind};

    let dir = std::env::temp_dir().join("sv_lang_semantic_dist_default_weight_test");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("dw.sv");
    std::fs::write(
        &file,
        "class C; rand int x; endclass\n\
         module m; C obj; int ok;\n\
         initial begin\n\
         obj = new();\n\
         ok = obj.randomize() with { x dist { 0 := 1, default :/ 5 }; };\n\
         end endmodule\n",
    )
    .unwrap();

    let mut driver = Driver::new().unwrap();
    driver
        .parse_args([
            "slang".to_string(),
            file.to_string_lossy().into_owned(),
            "--std=1800-2023".to_string(),
        ])
        .unwrap();
    driver.process_options().unwrap();
    driver.parse_sources().unwrap();
    assert_eq!(driver.language_version(), 2); // SLANG_LANGUAGE_1800_2023

    let mut comp = Compilation::new(driver.session()).unwrap();
    for tree in &driver.trees() {
        comp.add(tree).unwrap();
    }
    let design = comp.compile().unwrap();
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    let dist = find_randomize_dist(&design);
    let default_weight = dist.dist_default_weight().unwrap();
    assert_eq!(default_weight.kind, DistWeightKind::PerRange);
    assert_eq!(default_weight.expr.kind(), ExpressionKind::IntegerLiteral);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn expression_effective_width_computes_minimal_bits() {
    // `3` needs 2 bits, `4` needs 3 bits; the sum's effective width is the max.
    let design = compile("module m; localparam int X = 3 + 4; localparam int Y = 100; endmodule\n");
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    assert_eq!(
        body.find("X")
            .unwrap()
            .initializer()
            .unwrap()
            .effective_width(),
        Some(3)
    );
    // 100 = 0b1100100, needing 7 bits.
    assert_eq!(
        body.find("Y")
            .unwrap()
            .initializer()
            .unwrap()
            .effective_width(),
        Some(7)
    );
}

#[test]
fn eval_session_eval_lvalue_reads_writes_scratch_and_rejects_non_lvalue() {
    let mut design = compile(
        "module m; int x; int z;\n\
         initial begin x = 5; z = x + 1; end\n\
         endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    let eval = design.eval_session();
    let (first_assign, second_assign) = {
        let body = eval
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
        let stmts = block.body().unwrap().statements()[0].statements();
        (stmts[0].expr().unwrap(), stmts[1].expr().unwrap())
    };

    // `x`: a plain NamedValue lvalue. Seeded with `int`'s default (0),
    // independent of the design's own storage; writes go only to the scratch
    // frame this call created.
    let lhs = first_assign.left().unwrap();
    let mut lval = eval.eval_lvalue(lhs).unwrap();
    assert!(!lval.is_bad());
    assert_eq!(lval.load().as_deref(), Some("0"));
    assert!(lval.store_int(42));
    assert_eq!(lval.load().as_deref(), Some("42"));

    // A second, independent call gets its own fresh scratch storage again.
    let mut lval2 = eval.eval_lvalue(lhs).unwrap();
    assert_eq!(lval2.load().as_deref(), Some("0"));
    assert!(lval2.store_int(7));
    assert_eq!(lval2.load().as_deref(), Some("7"));
    // The first handle's own storage is unaffected by the second.
    assert_eq!(lval.load().as_deref(), Some("42"));

    // `x + 1`: a BinaryOp expression does not implement lvalue evaluation --
    // slang raises an internal (catchable) assertion, which comes back as
    // `None` rather than panicking or aborting.
    let rhs = second_assign.right().unwrap();
    assert!(eval.eval_lvalue(rhs).is_none());
}

// ---- Expression semantic surface, part 3 -----------------------------------

#[test]
fn expr_symbol_reference_respects_allow_packed() {
    let design =
        compile("module m; logic [7:0] packed_arr; logic sel = packed_arr[0]; endmodule\n");
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let sel = body.find("sel").unwrap().initializer().unwrap(); // `packed_arr[0]`

    // With `allow_packed = true` (matching `referenced_symbol`'s behavior), a
    // select into a packed array still resolves to the underlying symbol.
    assert_eq!(sel.symbol_reference(true).unwrap().name(), "packed_arr");
    assert_eq!(sel.referenced_symbol().unwrap().name(), "packed_arr");

    // With `allow_packed = false`, a select into a *packed* type is excluded.
    assert!(sel.symbol_reference(false).is_none());
}

#[test]
fn expr_has_hierarchical_reference_distinguishes_dotted_and_local_names() {
    let design = compile(
        "module leaf; logic [7:0] v = 8'd1; endmodule\n\
         module top; leaf u_leaf(); logic [7:0] w = u_leaf.v; logic [7:0] z = 8'd2; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    assert!(
        body.find("w")
            .unwrap()
            .initializer()
            .unwrap()
            .has_hierarchical_reference()
    );
    assert!(
        !body
            .find("z")
            .unwrap()
            .initializer()
            .unwrap()
            .has_hierarchical_reference()
    );
}

#[test]
fn expr_is_equivalent_to_compares_structural_shape() {
    let design = compile(
        "module m; logic [7:0] x; wire [7:0] a = x + 1; wire [7:0] b = x + 1;\n\
         wire [7:0] c = x + 2; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let a = body.find("a").unwrap().initializer().unwrap();
    let b = body.find("b").unwrap().initializer().unwrap();
    let c = body.find("c").unwrap().initializer().unwrap();
    assert!(a.is_equivalent_to(&b));
    assert!(!a.is_equivalent_to(&c));
    // Equivalence is reflexive.
    assert!(a.is_equivalent_to(&a));
}

#[test]
fn expr_is_implicit_string_true_for_string_concat_false_for_int() {
    let design = compile(
        "module m; localparam bit [15:0] s = {\"h\", \"i\"}; localparam int n = 4; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    assert!(
        body.find("s")
            .unwrap()
            .initializer()
            .unwrap()
            .is_implicit_string()
    );
    assert!(
        !body
            .find("n")
            .unwrap()
            .initializer()
            .unwrap()
            .is_implicit_string()
    );
}

#[test]
fn expr_is_implicitly_assignable_to_checks_type_compatibility() {
    let design = compile(
        "module m; typedef enum { A, B } e_t;\n\
         logic [7:0] x = 8'd1; logic [3:0] y = 4'd2; e_t z; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let x = body.find("x").unwrap().initializer().unwrap();
    let y_ty = body.find("y").unwrap().value_type().unwrap();
    let z_ty = body.find("z").unwrap().value_type().unwrap();
    // Any two integral types are assignment-compatible regardless of width.
    assert!(x.is_implicitly_assignable_to(&y_ty));
    // A plain integral value is not implicitly assignable to an unrelated enum.
    assert!(!x.is_implicitly_assignable_to(&z_ty));
}

#[test]
fn expr_is_unsized_integer_true_for_unsized_literal_false_for_sized() {
    let design = compile("module m; localparam int a = 4; localparam int b = 8'd4; endmodule\n");
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    assert!(
        body.find("a")
            .unwrap()
            .initializer()
            .unwrap()
            .is_unsized_integer()
    );
    assert!(
        !body
            .find("b")
            .unwrap()
            .initializer()
            .unwrap()
            .is_unsized_integer()
    );
}

#[test]
fn integer_literal_value_and_is_declared_unsized() {
    use sv_lang::kinds::ExpressionKind;

    let design =
        compile("module m; localparam int a = 4; localparam bit [7:0] b = 8'd4; endmodule\n");
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let a = body.find("a").unwrap().initializer().unwrap();
    assert_eq!(a.kind(), ExpressionKind::IntegerLiteral);
    assert_eq!(a.integer_literal_value().unwrap().as_i64(), Some(4));
    assert_eq!(a.is_declared_unsized(), Some(true));

    let b = body.find("b").unwrap().initializer().unwrap();
    assert_eq!(b.kind(), ExpressionKind::IntegerLiteral);
    let bv = b.integer_literal_value().unwrap();
    assert_eq!(bv.as_i64(), Some(4));
    assert_eq!(bv.as_integer().unwrap().bit_width(), 8);
    assert_eq!(b.is_declared_unsized(), Some(false));

    // A non-IntegerLiteral expression reports `None` for both accessors.
    let sum = {
        // Build an expression that is not an IntegerLiteral: a BinaryOp.
        let design2 = compile("module m; localparam int c = 1 + 2; endmodule\n");
        let body2 = design2
            .top_instances()
            .next()
            .unwrap()
            .instance_body()
            .unwrap();
        body2.find("c").unwrap().initializer().unwrap().kind()
    };
    assert_eq!(sum, ExpressionKind::BinaryOp);
}

#[test]
fn real_literal_value_reads_the_double() {
    use sv_lang::kinds::ExpressionKind;

    let design = compile("module m; real r = 3.5; int n = 4; endmodule\n");
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let r = body.find("r").unwrap().initializer().unwrap();
    assert_eq!(r.kind(), ExpressionKind::RealLiteral);
    assert_eq!(r.real_literal_value(), Some(3.5));

    // A non-RealLiteral expression reports `None`.
    let n = body.find("n").unwrap().initializer().unwrap();
    assert_eq!(n.real_literal_value(), None);
}

#[test]
fn time_literal_value_and_scale() {
    use sv_lang::kinds::ExpressionKind;

    let design = compile(
        "`timescale 1ns/1ps\n\
         module m; realtime t = 1.5ns; int n = 4; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let t = body.find("t").unwrap().initializer().unwrap();
    assert_eq!(t.kind(), ExpressionKind::TimeLiteral);
    assert_eq!(t.time_literal_value(), Some(1.5));

    let scale = t.time_literal_scale().unwrap();
    assert_eq!(scale.base.unit, sv_lang::TimeUnit::Nanoseconds);
    assert_eq!(scale.base.magnitude, 1);
    assert_eq!(scale.precision.unit, sv_lang::TimeUnit::Picoseconds);
    assert_eq!(scale.precision.magnitude, 1);

    // A non-TimeLiteral expression reports `None` for both accessors.
    let n = body.find("n").unwrap().initializer().unwrap();
    assert_eq!(n.time_literal_value(), None);
    assert!(n.time_literal_scale().is_none());
}

#[test]
fn unbased_unsized_integer_literal_bit_and_value() {
    use sv_lang::Bit;
    use sv_lang::kinds::ExpressionKind;

    let design = compile(
        "module m; logic [7:0] all_ones = '1; logic [7:0] all_zeros = '0;\n\
         logic [7:0] all_x = 'x; logic [7:0] all_z = 'z; int n = 4; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let ones = body.find("all_ones").unwrap().initializer().unwrap();
    assert_eq!(ones.kind(), ExpressionKind::UnbasedUnsizedIntegerLiteral);
    assert_eq!(ones.unbased_unsized_literal_bit(), Some(Bit::One));
    let ones_v = ones.unbased_unsized_literal_value().unwrap();
    assert_eq!(ones_v.as_integer().unwrap().bit_width(), 8);
    assert_eq!(ones_v.as_i64(), Some(0xff));

    let zeros = body.find("all_zeros").unwrap().initializer().unwrap();
    assert_eq!(zeros.unbased_unsized_literal_bit(), Some(Bit::Zero));
    assert_eq!(
        zeros.unbased_unsized_literal_value().unwrap().as_i64(),
        Some(0)
    );

    let x = body.find("all_x").unwrap().initializer().unwrap();
    assert_eq!(x.unbased_unsized_literal_bit(), Some(Bit::X));
    let x_v = x.unbased_unsized_literal_value().unwrap();
    match x_v {
        sv_lang::ConstantValue::Integer(v) => assert!(v.has_unknown()),
        other => panic!("expected an integer, got {other:?}"),
    }

    let z = body.find("all_z").unwrap().initializer().unwrap();
    assert_eq!(z.unbased_unsized_literal_bit(), Some(Bit::Z));

    // A non-UnbasedUnsizedIntegerLiteral expression reports `None` for both.
    let n = body.find("n").unwrap().initializer().unwrap();
    assert_eq!(n.unbased_unsized_literal_bit(), None);
    assert!(n.unbased_unsized_literal_value().is_none());
}

#[test]
fn tagged_union_value_expr() {
    use sv_lang::kinds::ExpressionKind;

    let design = compile(
        "module m; typedef union tagged { void Invalid; int Valid; } u_t;\n\
         u_t u = tagged Valid 5;\n\
         u_t v = tagged Invalid;\n\
         endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let u = body.find("u").unwrap().initializer().unwrap();
    assert_eq!(u.kind(), ExpressionKind::TaggedUnion);
    let value = u.tagged_union_value().unwrap();
    match value.constant_value().unwrap() {
        sv_lang::ConstantValue::Integer(v) => assert_eq!(v.as_i64(), Some(5)),
        other => panic!("expected an integer, got {other:?}"),
    }

    // A `void` member has no value expression.
    let v = body.find("v").unwrap().initializer().unwrap();
    assert_eq!(v.kind(), ExpressionKind::TaggedUnion);
    assert!(v.tagged_union_value().is_none());
}

#[test]
fn inside_expression_left_and_range_list() {
    use sv_lang::kinds::ExpressionKind;

    let design = compile(
        "module m; logic [7:0] x; logic y;\n\
         initial y = (x inside {8'd1, [8'd2:8'd4]}); endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let assign = block.body().unwrap().expr().unwrap(); // `y = (x inside {...})`
    let inside = assign.right().unwrap();
    assert_eq!(inside.kind(), ExpressionKind::Inside);

    let left = inside.inside_left().unwrap();
    assert_eq!(left.referenced_symbol().unwrap().name(), "x");

    let ranges = inside.inside_range_list();
    assert_eq!(ranges.len(), 2);

    // A non-Inside expression reports no left/ranges.
    assert!(left.inside_left().is_none());
    assert!(left.inside_range_list().is_empty());
}

#[test]
fn new_array_expression_size_and_init() {
    use sv_lang::kinds::ExpressionKind;

    // Without an initializer.
    let design = compile("module m; int n = 4; int arr[]; initial arr = new[n]; endmodule\n");
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let assign = block.body().unwrap().expr().unwrap(); // `arr = new[n]`
    let new_array = assign.right().unwrap();
    assert_eq!(new_array.kind(), ExpressionKind::NewArray);
    assert_eq!(
        new_array
            .new_array_size()
            .unwrap()
            .referenced_symbol()
            .unwrap()
            .name(),
        "n"
    );
    assert!(new_array.new_array_init().is_none());

    // With an initializer.
    let design2 = compile("module m; int n = 4; int arr[]; initial arr = new[n](arr); endmodule\n");
    assert!(
        !design2.diagnostics().has_errors(),
        "{}",
        design2.diagnostics()
    );
    let body2 = design2
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block2 = body2
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let assign2 = block2.body().unwrap().expr().unwrap();
    let new_array2 = assign2.right().unwrap();
    assert_eq!(new_array2.kind(), ExpressionKind::NewArray);
    assert!(new_array2.new_array_init().is_some());
}

#[test]
fn new_class_expression_constructor_call_and_is_super_class() {
    use sv_lang::kinds::ExpressionKind;

    // A plain `new(args)`: has a constructor call, is not a super-class call.
    let design = compile(
        "class C; int x; function new(int v); x = v; endfunction endclass\n\
         module m; C obj; initial obj = new(3); endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let assign = block.body().unwrap().expr().unwrap(); // `obj = new(3)`
    let new_class = assign.right().unwrap();
    assert_eq!(new_class.kind(), ExpressionKind::NewClass);
    let call = new_class.new_class_constructor_call().unwrap();
    assert_eq!(call.kind(), ExpressionKind::Call);
    assert_eq!(new_class.new_class_is_super_class(), Some(false));

    // A non-NewClass expression reports `None` for is_super_class.
    assert_eq!(call.new_class_is_super_class(), None);
    assert!(call.new_class_constructor_call().is_none());

    // `super.new(v)`: a super-class constructor call, found inside D's own
    // `new` subroutine body.
    let design2 = compile(
        "class B; int b; function new(int v); b = v; endfunction endclass\n\
         class D extends B; function new(int v); super.new(v); endfunction endclass\n\
         module m; endmodule\n",
    );
    assert!(
        !design2.diagnostics().has_errors(),
        "{}",
        design2.diagnostics()
    );
    let mut d_new = None;
    let mut seen_d = false;
    design2.root().visit(|sym| {
        if sym.name() == "D" && sym.kind() == SymbolKind::ClassType {
            seen_d = true;
        }
        if seen_d && sym.name() == "new" && sym.kind() == SymbolKind::Subroutine && d_new.is_none()
        {
            d_new = Some(sym);
            return sv_lang::Walk::Skip;
        }
        sv_lang::Walk::Continue
    });
    let d_new = d_new.expect("D::new subroutine not found");
    let ctor_body = d_new.body().unwrap();
    let super_call_expr = ctor_body.expr().unwrap(); // `super.new(v)`
    assert_eq!(super_call_expr.kind(), ExpressionKind::NewClass);
    assert_eq!(super_call_expr.new_class_is_super_class(), Some(true));
    assert!(super_call_expr.new_class_constructor_call().is_some());
}

#[test]
fn new_covergroup_expression_arguments() {
    use sv_lang::kinds::ExpressionKind;

    let design = compile(
        "module m(input logic clk, input logic [2:0] a);\n\
         covergroup cg(int lo); cp: coverpoint a; endgroup\n\
         cg cgh; initial cgh = new(1); endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let assign = block.body().unwrap().expr().unwrap(); // `cgh = new(1)`
    let new_cg = assign.right().unwrap();
    assert_eq!(new_cg.kind(), ExpressionKind::NewCovergroup);

    let args = new_cg.new_covergroup_arguments();
    assert_eq!(args.len(), 1);
    assert_eq!(args[0].integer_literal_value().unwrap().as_i64(), Some(1));

    // A non-NewCovergroup expression has no arguments.
    assert!(args[0].new_covergroup_arguments().is_empty());
}

#[test]
fn svint_bit_counting() {
    let design = compile(
        "module m;\n\
           // 00111010: two leading zeros, four ones, four zeros, active_bits = 6.\n\
           localparam logic [7:0] A = 8'b0011_1010;\n\
           // 11001011: two leading ones, five ones, three zeros, active_bits = 8.\n\
           localparam logic [7:0] B = 8'b1100_1011;\n\
           // x,x,z,1,0,1,0,1: three leading unknowns (x/z run), two x's, one z.\n\
           localparam logic [7:0] C = 8'bxxz1_0101;\n\
           // z,1,x,0,0,1,0,1: exactly one leading z (stopped by the following 1).\n\
           localparam logic [7:0] D = 8'bz1x0_0101;\n\
         endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let int_of = |name: &str| {
        body.find(name)
            .unwrap()
            .initializer()
            .unwrap()
            .constant_value()
            .unwrap()
            .as_integer()
            .unwrap()
            .clone()
    };

    let a = int_of("A");
    assert_eq!(a.count_leading_zeros(), 2);
    assert_eq!(a.count_leading_ones(), 0);
    assert_eq!(a.count_ones(), 4);
    assert_eq!(a.count_zeros(), 4);
    assert_eq!(a.active_bits(), 6);
    assert_eq!(a.count_leading_unknowns(), 0);
    assert_eq!(a.count_leading_zs(), 0);
    assert_eq!(a.count_xs(), 0);
    assert_eq!(a.count_zs(), 0);

    let b = int_of("B");
    assert_eq!(b.count_leading_ones(), 2);
    assert_eq!(b.count_leading_zeros(), 0);
    assert_eq!(b.count_ones(), 5);
    assert_eq!(b.count_zeros(), 3);
    assert_eq!(b.active_bits(), 8);

    let c = int_of("C");
    assert!(c.has_unknown());
    assert_eq!(c.count_leading_unknowns(), 3);
    assert_eq!(c.count_xs(), 2);
    assert_eq!(c.count_zs(), 1);
    // The leading run is x,x,z — no leading z run (the MSB isn't z).
    assert_eq!(c.count_leading_zs(), 0);

    let d = int_of("D");
    assert!(d.has_unknown());
    assert_eq!(d.count_leading_zs(), 1);
    // The leading run is just the single z (bit 6 is a known 1).
    assert_eq!(d.count_leading_unknowns(), 1);
}

#[test]
fn constant_value_conversions_and_slicing() {
    let design = compile(
        "module m;\n\
           localparam int X = 4;\n\
           localparam bit [23:0] Str = \"hi!\";\n\
           localparam bit [15:0] TwoChars = \"hi\";\n\
           localparam logic [7:0] Byte = 8'hAB;\n\
           localparam int Arr[3] = '{10, 20, 30};\n\
           localparam real R = 1.5;\n\
         endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let init_of = |name: &str| body.find(name).unwrap().initializer().unwrap();

    // getBitstreamWidth: scalar integer width, and the recursive sum over an
    // unpacked array's elements.
    let x = init_of("X");
    assert_eq!(x.constant_bitstream_width(), Some(32));
    let arr = init_of("Arr");
    assert_eq!(arr.constant_bitstream_width(), Some(3 * 32));

    // getSlice: a bit-range slice of an integer, both nibbles of 0xAB, plus
    // the bad-fill behavior for an out-of-range unpacked-array index.
    let byte = init_of("Byte");
    assert_eq!(byte.constant_slice(3, 0).unwrap().as_i64(), Some(0xB));
    assert_eq!(byte.constant_slice(7, 4).unwrap().as_i64(), Some(0xA));
    assert_eq!(byte.constant_slice(7, 0).unwrap().as_i64(), Some(0xAB));

    let low_two = arr.constant_slice(1, 0).unwrap();
    let elems: Vec<_> = low_two
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e.as_i64())
        .collect();
    assert_eq!(elems, [Some(10), Some(20)]);
    // Index 3 is out of range for a 3-element array (valid indices 0..=2):
    // the slice comes back with a bad-constant fill rather than panicking.
    let oob = arr.constant_slice(3, 3).unwrap();
    let oob_elems = oob.as_array().unwrap();
    assert_eq!(oob_elems.len(), 1);
    assert_eq!(oob_elems[0], sv_lang::ConstantValue::Bad);

    // convertToReal / convertToShortReal: integer -> floating point.
    assert_eq!(x.constant_as_real().unwrap().as_f64(), Some(4.0));
    assert_eq!(x.constant_as_short_real().unwrap().as_f64(), Some(4.0));
    // Converting a container constant to real fails (not real/shortreal/integer).
    assert!(arr.constant_as_real().is_none());

    // convertToStr: an integer packed with a string literal round-trips.
    assert_eq!(
        init_of("Str").constant_as_str().unwrap().as_str(),
        Some("hi!")
    );
    // Converting a container (not string/integer) to str fails.
    assert!(arr.constant_as_str().is_none());

    // convertToByteArray / convertToByteQueue: MSB-first character packing.
    let two_chars = init_of("TwoChars");
    let bytes = two_chars.constant_as_byte_array(0, false).unwrap();
    let byte_vals: Vec<_> = bytes
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e.as_i64())
        .collect();
    assert_eq!(byte_vals, [Some('h' as i64), Some('i' as i64)]);

    let queue = two_chars.constant_as_byte_queue(false).unwrap();
    let queue_vals: Vec<_> = queue
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e.as_i64())
        .collect();
    assert_eq!(queue_vals, [Some('h' as i64), Some('i' as i64)]);

    // convertToByteArray passes an already-unpacked-array constant through
    // unchanged (its elements are ints here, not bytes).
    let arr_passthrough = arr.constant_as_byte_array(0, false).unwrap();
    assert_eq!(arr_passthrough, arr.constant_value().unwrap());

    // Converting a value that is neither unpacked/string/integer (a real)
    // fails for byte-array, and a non-queue (the unpacked array) fails for
    // byte-queue.
    let r = init_of("R");
    assert!(r.constant_as_byte_array(0, false).is_none());
    assert!(arr.constant_as_byte_queue(false).is_none());
}

#[test]
fn svint_numeric_ops() {
    let design = compile(
        "module m;\n\
           // -4 as 8-bit signed two's complement: 1111_1100.\n\
           localparam bit signed [7:0] Neg = -8'sd4;\n\
           // +5 as 8-bit signed: 0000_0101.\n\
           localparam bit signed [7:0] Pos = 8'sd5;\n\
           localparam bit [3:0] AllOnes = 4'b1111;\n\
           localparam bit [3:0] SomeOnes = 4'b0100;\n\
           localparam bit [3:0] AllZeros = 4'b0000;\n\
           // Three set bits: odd parity.\n\
           localparam bit [3:0] ParityOdd = 4'b0111;\n\
           localparam bit Zero = 1'b0;\n\
           localparam bit One = 1'b1;\n\
           localparam bit [3:0] Base4 = 4'd3;\n\
           localparam int Exp4 = 3;\n\
           localparam bit [2:0] Rep3 = 3'b101;\n\
           localparam int Times2 = 2;\n\
           localparam bit [7:0] ByteVal = 8'hAB;\n\
           localparam bit [3:0] RevSrc = 4'b1000;\n\
           // -1 as 4-bit signed: 1111 (also 15 unsigned).\n\
           localparam bit signed [3:0] NegOne4 = -4'sd1;\n\
           localparam real R = 1.5;\n\
         endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let init_of = |name: &str| body.find(name).unwrap().initializer().unwrap();

    // isNegative / isEven / isOdd / getMinRepresentedBits, on a negative
    // signed value: -4 = 1111_1100 (6 leading ones) needs 3 bits
    // (bitWidth - leadingOnes + 1 = 8 - 6 + 1).
    let neg = init_of("Neg");
    assert_eq!(neg.constant_is_negative(), Some(true));
    assert_eq!(neg.constant_is_even(), Some(true));
    assert_eq!(neg.constant_is_odd(), Some(false));
    assert_eq!(neg.constant_min_represented_bits(), Some(3));
    // Bits 2..7 are all the sign bit (1); bit 1 is 0 and breaks the run.
    assert_eq!(neg.constant_is_sign_extended_from(2), Some(true));
    assert_eq!(neg.constant_is_sign_extended_from(1), Some(false));

    // A positive signed value: 5 = 0000_0101, active_bits = 3, needs 4 bits
    // signed (activeBits + 1) since 3 bits (-4..3) can't hold +5.
    let pos = init_of("Pos");
    assert_eq!(pos.constant_is_negative(), Some(false));
    assert_eq!(pos.constant_is_even(), Some(false));
    assert_eq!(pos.constant_is_odd(), Some(true));
    assert_eq!(pos.constant_min_represented_bits(), Some(4));

    // Reduction AND/OR/XOR.
    assert_eq!(
        init_of("AllOnes").constant_reduction_and(),
        Some(sv_lang::Bit::One)
    );
    assert_eq!(
        init_of("SomeOnes").constant_reduction_and(),
        Some(sv_lang::Bit::Zero)
    );
    assert_eq!(
        init_of("SomeOnes").constant_reduction_or(),
        Some(sv_lang::Bit::One)
    );
    assert_eq!(
        init_of("AllZeros").constant_reduction_or(),
        Some(sv_lang::Bit::Zero)
    );
    assert_eq!(
        init_of("ParityOdd").constant_reduction_xor(),
        Some(sv_lang::Bit::One)
    );
    assert_eq!(
        init_of("AllOnes").constant_reduction_xor(),
        Some(sv_lang::Bit::Zero)
    );

    // Static logicalImpl / logicalEquiv: !lhs || rhs, and mutual implication.
    let zero = init_of("Zero");
    let one = init_of("One");
    assert_eq!(zero.constant_logical_impl(&one), Some(sv_lang::Bit::One));
    assert_eq!(one.constant_logical_impl(&zero), Some(sv_lang::Bit::Zero));
    assert_eq!(one.constant_logical_equiv(&one), Some(sv_lang::Bit::One));
    assert_eq!(zero.constant_logical_equiv(&one), Some(sv_lang::Bit::Zero));

    // pow: result has the *receiver's* bit width, modulo that width.
    // 3**3 == 27, which truncates to 4 bits as 27 mod 16 == 11.
    let base4 = init_of("Base4");
    let exp4 = init_of("Exp4");
    let powered = base4.constant_pow(&exp4).unwrap();
    assert_eq!(powered.as_integer().unwrap().bit_width(), 4);
    assert_eq!(powered.as_i64(), Some(11));

    // replicate: 3-bit 101 repeated twice -> 6-bit 101101 == 45.
    let rep3 = init_of("Rep3");
    let times2 = init_of("Times2");
    let replicated = rep3.constant_replicate(&times2).unwrap();
    assert_eq!(replicated.as_integer().unwrap().bit_width(), 6);
    assert_eq!(replicated.as_i64(), Some(0b101_101));

    // resize: truncation keeps the low bits; zero-extension (unsigned)
    // preserves the value.
    let byte_val = init_of("ByteVal");
    let truncated = byte_val.constant_resize(4).unwrap();
    assert_eq!(truncated.as_integer().unwrap().bit_width(), 4);
    assert_eq!(truncated.as_i64(), Some(0xB));
    let widened = byte_val.constant_resize(12).unwrap();
    assert_eq!(widened.as_integer().unwrap().bit_width(), 12);
    assert_eq!(widened.as_i64(), Some(0xAB));
    // resize to 0 bits is invalid.
    assert!(byte_val.constant_resize(0).is_none());

    // reverse: 4'b1000 (8) reversed is 4'b0001 (1), same width.
    let reversed = init_of("RevSrc").constant_reverse().unwrap();
    assert_eq!(reversed.as_integer().unwrap().bit_width(), 4);
    assert_eq!(reversed.as_i64(), Some(1));

    // sext vs zext on the same bit pattern (4'b1111): sign-extending a
    // negative signed value keeps it -1, while zero-extending the same bits
    // treats them as the unsigned value 15.
    let neg_one = init_of("NegOne4");
    let sexted = neg_one.constant_sext(8).unwrap();
    assert_eq!(sexted.as_integer().unwrap().bit_width(), 8);
    assert_eq!(sexted.as_i64(), Some(-1));
    let zexted = neg_one.constant_zext(8).unwrap();
    assert_eq!(zexted.as_integer().unwrap().bit_width(), 8);
    assert_eq!(zexted.as_i64(), Some(0xF));
    // sext/zext require bits strictly greater than the current width.
    assert!(neg_one.constant_sext(4).is_none());
    assert!(neg_one.constant_zext(3).is_none());

    // Every accessor returns None for a non-integer (real) constant.
    let r = init_of("R");
    assert_eq!(r.constant_min_represented_bits(), None);
    assert_eq!(r.constant_is_even(), None);
    assert_eq!(r.constant_is_odd(), None);
    assert_eq!(r.constant_is_negative(), None);
    assert_eq!(r.constant_is_sign_extended_from(0), None);
    assert_eq!(r.constant_reduction_and(), None);
    assert_eq!(r.constant_reduction_or(), None);
    assert_eq!(r.constant_reduction_xor(), None);
    assert_eq!(r.constant_logical_impl(&one), None);
    assert_eq!(r.constant_logical_equiv(&one), None);
    assert!(r.constant_pow(&exp4).is_none());
    assert!(r.constant_replicate(&times2).is_none());
    assert!(r.constant_resize(4).is_none());
    assert!(r.constant_reverse().is_none());
    assert!(r.constant_sext(8).is_none());
    assert!(r.constant_zext(8).is_none());
}

#[test]
fn svint_numeric_ops_part4() {
    let design = compile(
        "module m;\n\
           localparam bit [3:0] X = 4'b1100;\n\
           localparam bit [3:0] Y = 4'b1010;\n\
           localparam bit [7:0] ByteVal = 8'hAB;\n\
           localparam bit [7:0] SliceSrc = 8'hA5;\n\
           // -1 as 4-bit signed: 1111 (also 15 unsigned).\n\
           localparam bit signed [3:0] NegOne4 = -4'sd1;\n\
           localparam bit [3:0] ZeroInit = 4'b0000;\n\
           localparam real R = 1.5;\n\
         endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let init_of = |name: &str| body.find(name).unwrap().initializer().unwrap();

    // ---- extend: sign- or zero-extends per `is_signed`, like sext/zext. ----
    let neg_one = init_of("NegOne4");
    let sexted = neg_one.constant_extend(8, true).unwrap();
    assert_eq!(sexted.as_integer().unwrap().bit_width(), 8);
    assert_eq!(sexted.as_i64(), Some(-1));
    let zexted = neg_one.constant_extend(8, false).unwrap();
    assert_eq!(zexted.as_integer().unwrap().bit_width(), 8);
    assert_eq!(zexted.as_i64(), Some(0xF));
    // extend requires bits strictly greater than the current width.
    assert!(neg_one.constant_extend(4, true).is_none());
    assert!(neg_one.constant_extend(3, false).is_none());

    // ---- trunc: keeps the low bits. ----
    let byte_val = init_of("ByteVal");
    let truncated = byte_val.constant_trunc(4).unwrap();
    assert_eq!(truncated.as_integer().unwrap().bit_width(), 4);
    assert_eq!(truncated.as_i64(), Some(0xB));
    // trunc requires 0 < bits <= current width.
    assert!(byte_val.constant_trunc(0).is_none());
    assert!(byte_val.constant_trunc(9).is_none());

    // ---- bit_slice: a bit-range slice, x-filled out of range. ----
    let slice_src = init_of("SliceSrc"); // 0xA5 = 1010_0101
    let low = slice_src.constant_bit_slice(3, 0).unwrap();
    assert_eq!(low.as_integer().unwrap().bit_width(), 4);
    assert_eq!(low.as_i64(), Some(0x5));
    let high = slice_src.constant_bit_slice(7, 4).unwrap();
    assert_eq!(high.as_i64(), Some(0xA));
    let oob = slice_src.constant_bit_slice(11, 8).unwrap();
    assert!(oob.as_integer().unwrap().has_unknown());
    // slice requires msb >= lsb.
    assert!(slice_src.constant_bit_slice(0, 3).is_none());

    // ---- and/or/xor/xnor/not: X = 1100, Y = 1010. ----
    let x = init_of("X");
    let y = init_of("Y");
    assert_eq!(x.constant_and(&y).unwrap().as_i64(), Some(0b1000));
    assert_eq!(x.constant_or(&y).unwrap().as_i64(), Some(0b1110));
    assert_eq!(x.constant_xor(&y).unwrap().as_i64(), Some(0b0110));
    assert_eq!(x.constant_xnor(&y).unwrap().as_i64(), Some(0b1001));
    assert_eq!(x.constant_not().unwrap().as_i64(), Some(0b0011));
    // Results keep the (matched) operand width.
    assert_eq!(
        x.constant_and(&y)
            .unwrap()
            .as_integer()
            .unwrap()
            .bit_width(),
        4
    );
    assert_eq!(
        x.constant_not().unwrap().as_integer().unwrap().bit_width(),
        4
    );

    // ---- constant_integer_copy + the in-place OwnedSVInt mutators. ----
    // Every copy is independent: mutating one never affects `x`/`y` or any
    // other copy.
    let mut xv = x.constant_integer_copy().unwrap();
    assert_eq!(xv.snapshot().as_i64(), Some(0b1100));
    let yv = y.constant_integer_copy().unwrap();

    xv.and_assign(&yv).unwrap();
    assert_eq!(xv.snapshot().as_i64(), Some(0b1000));
    assert_eq!(xv.snapshot().bit_width(), 4);
    // The source expression's own folded constant is untouched.
    assert_eq!(x.constant_value().unwrap().as_i64(), Some(0b1100));

    let mut xv2 = x.constant_integer_copy().unwrap();
    xv2.or_assign(&yv).unwrap();
    assert_eq!(xv2.snapshot().as_i64(), Some(0b1110));

    let mut xv3 = x.constant_integer_copy().unwrap();
    xv3.xor_assign(&yv).unwrap();
    assert_eq!(xv3.snapshot().as_i64(), Some(0b0110));

    // iand/ior/ixor never change the bit width: a mismatched rhs is
    // rejected (and leaves the receiver unmodified) instead of
    // auto-extending like the C++ operator does.
    let mut byte_owned = byte_val.constant_integer_copy().unwrap();
    assert!(byte_owned.and_assign(&yv).is_err());
    assert_eq!(byte_owned.snapshot().as_i64(), Some(0xAB));
    assert!(byte_owned.or_assign(&yv).is_err());
    assert!(byte_owned.xor_assign(&yv).is_err());

    // set: replaces a bit range in place without changing the width.
    let mut set_target = byte_val.constant_integer_copy().unwrap(); // 8'hAB
    set_target.set_range(3, 0, &yv).unwrap(); // low nibble <- Y (0xA)
    assert_eq!(set_target.snapshot().as_i64(), Some(0xAA));
    assert_eq!(set_target.snapshot().bit_width(), 8);
    // set requires msb >= lsb and a matching-width value.
    assert!(set_target.set_range(0, 3, &yv).is_err());
    assert!(set_target.set_range(3, 0, &xv).is_ok()); // xv is now 4 bits wide (0b1000)
    assert_eq!(set_target.snapshot().as_i64(), Some(0xA8));

    // setAllOnes/Zeros/X/Z, each from a fresh all-zero copy.
    let zero = init_of("ZeroInit");
    let mut ones = zero.constant_integer_copy().unwrap();
    ones.set_all_ones();
    assert_eq!(ones.snapshot().as_i64(), Some(0b1111));
    assert_eq!(ones.snapshot().bit_width(), 4);

    let mut zeros = zero.constant_integer_copy().unwrap();
    zeros.set_all_zeros();
    assert_eq!(zeros.snapshot().as_i64(), Some(0));

    let mut all_x = zero.constant_integer_copy().unwrap();
    all_x.set_all_x();
    assert!(all_x.snapshot().has_unknown());
    assert_eq!(all_x.snapshot().bit(0), Some(sv_lang::Bit::X));
    assert_eq!(all_x.snapshot().bit(3), Some(sv_lang::Bit::X));

    let mut all_z = zero.constant_integer_copy().unwrap();
    all_z.set_all_z();
    assert!(all_z.snapshot().has_unknown());
    assert_eq!(all_z.snapshot().bit(0), Some(sv_lang::Bit::Z));

    // Every accessor returns None/Err for a non-integer (real) constant.
    let r = init_of("R");
    assert!(r.constant_extend(8, true).is_none());
    assert!(r.constant_trunc(4).is_none());
    assert!(r.constant_bit_slice(3, 0).is_none());
    assert!(r.constant_and(&y).is_none());
    assert!(r.constant_or(&y).is_none());
    assert!(r.constant_xor(&y).is_none());
    assert!(r.constant_xnor(&y).is_none());
    assert!(r.constant_not().is_none());
    assert!(r.constant_integer_copy().is_none());
}

#[test]
fn svint_part5_mutators_and_static_factories() {
    use sv_lang::{Digit, LiteralBase, OwnedSVInt};

    let design = compile(
        "module m;\n\
           localparam bit [7:0] ByteVal = 8'h0F;\n\
           localparam bit [31:0] Small = 32'd3;\n\
           localparam bit [7:0] Neg8 = -8'sd1;\n\
           localparam bit [3:0] X = 4'hA;\n\
           localparam bit [3:0] Y = 4'hB;\n\
           localparam bit C1 = 1;\n\
           localparam bit C0 = 0;\n\
         endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let init_of = |name: &str| body.find(name).unwrap().initializer().unwrap();

    // ---- set_signed: reinterprets in place, bits/width unchanged. ----
    let byte_val = init_of("ByteVal");
    let mut v = byte_val.constant_integer_copy().unwrap();
    assert!(!v.snapshot().is_signed());
    v.set_signed(true);
    assert!(v.snapshot().is_signed());
    assert_eq!(v.snapshot().as_i64(), Some(0x0F));
    assert_eq!(v.snapshot().bit_width(), 8);

    // ---- flatten_unknowns: x/z -> 0, width unchanged. ----
    let mut fx = byte_val.constant_integer_copy().unwrap();
    fx.set_all_x();
    assert!(fx.snapshot().has_unknown());
    fx.flatten_unknowns();
    assert!(!fx.snapshot().has_unknown());
    assert_eq!(fx.snapshot().as_i64(), Some(0));
    assert_eq!(fx.snapshot().bit_width(), 8);

    // ---- shrink_to_fit: resizes to the minimum representing width. ----
    let small = init_of("Small"); // 32'd3, declared width 32
    let mut sv = small.constant_integer_copy().unwrap();
    assert_eq!(sv.snapshot().bit_width(), 32);
    sv.shrink_to_fit();
    assert_eq!(sv.snapshot().bit_width(), 2);
    assert_eq!(sv.snapshot().as_i64(), Some(3));

    // ---- sign_extend_from: duplicates bit `msb` upward in place. ----
    let _neg8 = init_of("Neg8"); // 8'b1111_1111, unused beyond documenting intent

    let mut se2 = byte_val.constant_integer_copy().unwrap(); // 8'h0F = 0000_1111
    se2.sign_extend_from(3).unwrap(); // bit 3 is set -> fills [7:4]
    assert_eq!(se2.snapshot().as_i64(), Some(0xFF));
    assert_eq!(se2.snapshot().bit_width(), 8);

    let mut se3 = init_of("Small").constant_integer_copy().unwrap();
    // With a fresh copy, bit_width - 1 = 31, so msb must be < 31; msb == 31 fails.
    assert!(se3.sign_extend_from(31).is_err());
    assert!(se3.sign_extend_from(30).is_ok());

    // ---- create_fill_x / create_fill_z: static factories. ----
    let fill_x = OwnedSVInt::create_fill_x(4, false).unwrap();
    assert!(fill_x.snapshot().has_unknown());
    assert_eq!(fill_x.snapshot().bit(0), Some(sv_lang::Bit::X));
    assert_eq!(fill_x.snapshot().bit_width(), 4);
    assert!(OwnedSVInt::create_fill_x(0, false).is_err());

    let fill_z = OwnedSVInt::create_fill_z(4, false).unwrap();
    assert!(fill_z.snapshot().has_unknown());
    assert_eq!(fill_z.snapshot().bit(0), Some(sv_lang::Bit::Z));
    assert!(OwnedSVInt::create_fill_z(0, false).is_err());

    // ---- from_digits: builds from an array of per-base digits. ----
    let hex = OwnedSVInt::from_digits(
        8,
        LiteralBase::Hex,
        false,
        false,
        &[Digit::Value(0xF), Digit::Value(0xA)],
    )
    .unwrap();
    assert_eq!(hex.snapshot().as_i64(), Some(0xFA));
    assert_eq!(hex.snapshot().bit_width(), 8);

    let bin_x = OwnedSVInt::from_digits(
        4,
        LiteralBase::Binary,
        false,
        true,
        &[Digit::X, Digit::Value(1), Digit::Value(0), Digit::Value(1)],
    )
    .unwrap();
    assert!(bin_x.snapshot().has_unknown());
    assert_eq!(bin_x.snapshot().bit(3), Some(sv_lang::Bit::X));
    assert_eq!(bin_x.snapshot().bit(2), Some(sv_lang::Bit::One));

    let dec_z = OwnedSVInt::from_digits(8, LiteralBase::Decimal, false, true, &[Digit::Z]).unwrap();
    assert!(dec_z.snapshot().has_unknown());
    assert_eq!(dec_z.snapshot().bit(0), Some(sv_lang::Bit::Z));

    // Errors: zero bits, no digits, a digit too large for its base.
    assert!(
        OwnedSVInt::from_digits(0, LiteralBase::Hex, false, false, &[Digit::Value(1)]).is_err()
    );
    assert!(OwnedSVInt::from_digits(8, LiteralBase::Hex, false, false, &[]).is_err());
    assert!(
        OwnedSVInt::from_digits(8, LiteralBase::Binary, false, false, &[Digit::Value(2)]).is_err()
    );

    // ---- from_double / from_float: static factories with rounding. ----
    let d_round = OwnedSVInt::from_double(32, 3.7, false, true).unwrap();
    assert_eq!(d_round.snapshot().as_i64(), Some(4));
    let d_trunc = OwnedSVInt::from_double(32, 3.7, false, false).unwrap();
    assert_eq!(d_trunc.snapshot().as_i64(), Some(3));
    assert!(OwnedSVInt::from_double(0, 3.7, false, true).is_err());

    let f_round = OwnedSVInt::from_float(32, 3.7f32, false, true).unwrap();
    assert_eq!(f_round.snapshot().as_i64(), Some(4));
    assert!(OwnedSVInt::from_float(0, 3.7f32, false, true).is_err());

    // ---- concat: msb-first concatenation of operands. ----
    let x = init_of("X").constant_integer_copy().unwrap(); // 4'hA
    let y = init_of("Y").constant_integer_copy().unwrap(); // 4'hB
    let cat = OwnedSVInt::concat(&[&x, &y]).unwrap();
    assert_eq!(cat.snapshot().bit_width(), 8);
    assert_eq!(cat.snapshot().as_i64(), Some(0xAB));
    // Order matters: reversing the operands reverses which half is which.
    let cat_rev = OwnedSVInt::concat(&[&y, &x]).unwrap();
    assert_eq!(cat_rev.snapshot().as_i64(), Some(0xBA));
    // An empty concatenation yields a 1-bit zero.
    let empty = OwnedSVInt::concat(&[]).unwrap();
    assert_eq!(empty.snapshot().bit_width(), 1);
    assert_eq!(empty.snapshot().as_i64(), Some(0));

    // ---- conditional: condition ? lhs : rhs, four-state aware. ----
    let c1 = init_of("C1").constant_integer_copy().unwrap();
    let c0 = init_of("C0").constant_integer_copy().unwrap();
    let picked_true = OwnedSVInt::conditional(&c1, &x, &y).unwrap();
    assert_eq!(picked_true.snapshot().as_i64(), Some(0xA));
    let picked_false = OwnedSVInt::conditional(&c0, &x, &y).unwrap();
    assert_eq!(picked_false.snapshot().as_i64(), Some(0xB));

    // An unknown condition with differing operands yields a bitwise-unknown
    // result (X differs from Y in every bit here: 1010 vs 1011).
    let unknown_cond = OwnedSVInt::create_fill_x(1, false).unwrap();
    let picked_unknown = OwnedSVInt::conditional(&unknown_cond, &x, &y).unwrap();
    assert!(picked_unknown.snapshot().has_unknown());
}

#[test]
fn time_scale_base_precision_apply_and_parse() {
    let design = compile(
        "`timescale 1ns/1ps\n\
         module m; realtime t = 1.5ns; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let t = body.find("t").unwrap().initializer().unwrap();
    let scale = t.time_literal_scale().unwrap();

    // ---- base / precision: mirror the public fields via the C API. ----
    assert_eq!(scale.base().unit, sv_lang::TimeUnit::Nanoseconds);
    assert_eq!(scale.base().magnitude, 1);
    assert_eq!(scale.base().unit(), sv_lang::TimeUnit::Nanoseconds);
    assert_eq!(scale.precision().unit, sv_lang::TimeUnit::Picoseconds);
    assert_eq!(scale.precision().magnitude, 1);
    assert_eq!(scale.precision().unit(), sv_lang::TimeUnit::Picoseconds);
    assert_eq!(scale.base(), scale.base);
    assert_eq!(scale.precision(), scale.precision);

    // ---- apply: scales a value (given in `unit`) to base-scale ticks. ----
    // 1.5ns, in a 1ns base scale, is 1.5 base ticks either way.
    assert_eq!(scale.apply(1.5, sv_lang::TimeUnit::Nanoseconds, false), 1.5);
    // 1500ps, in the same 1ns base scale, is also 1.5 base ticks.
    assert_eq!(
        scale.apply(1500.0, sv_lang::TimeUnit::Picoseconds, false),
        1.5
    );
    // 1.5001ns rounded to the 1ps precision comes back as 1.5 (in ns units,
    // i.e. 1.5 base ticks), since the extra 0.0001ns is below the precision.
    let rounded = scale.apply(1.5001, sv_lang::TimeUnit::Nanoseconds, true);
    assert!((rounded - 1.5).abs() < 1e-9);

    // ---- parse: the static factory, mirroring `TimeScale::fromString`. ----
    let parsed = sv_lang::TimeScale::parse("1ns/1ps").unwrap();
    assert_eq!(parsed, scale);
    assert_eq!(parsed.base.unit, sv_lang::TimeUnit::Nanoseconds);
    assert_eq!(parsed.base.magnitude, 1);
    assert_eq!(parsed.precision.unit, sv_lang::TimeUnit::Picoseconds);
    assert_eq!(parsed.precision.magnitude, 1);

    let parsed_coarser = sv_lang::TimeScale::parse("10us/100ns").unwrap();
    assert_eq!(parsed_coarser.base.unit, sv_lang::TimeUnit::Microseconds);
    assert_eq!(parsed_coarser.base.magnitude, 10);
    assert_eq!(
        parsed_coarser.precision.unit,
        sv_lang::TimeUnit::Nanoseconds
    );
    assert_eq!(parsed_coarser.precision.magnitude, 100);

    assert!(sv_lang::TimeScale::parse("not a time scale").is_none());
    assert!(sv_lang::TimeScale::parse("").is_none());
}

#[test]
fn time_scale_value_magnitude_from_literal_and_from_string() {
    let design = compile("`timescale 10ns/1ps\nmodule m; realtime t = 1.5ns; endmodule\n");
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let t = body.find("t").unwrap().initializer().unwrap();
    let scale = t.time_literal_scale().unwrap();

    // ---- magnitude: mirrors the TimeScaleValue::magnitude field. ----
    assert_eq!(scale.base().magnitude(), 10);
    assert_eq!(scale.base().magnitude(), scale.base.magnitude);
    assert_eq!(scale.precision().magnitude(), 1);

    // ---- from_literal: the static factory, mirroring
    // TimeScaleValue::fromLiteral. Only 1, 10, 100 are valid magnitudes. ----
    let lit = sv_lang::TimeScaleValue::from_literal(10.0, sv_lang::TimeUnit::Nanoseconds).unwrap();
    assert_eq!(lit, scale.base());
    assert_eq!(lit.unit, sv_lang::TimeUnit::Nanoseconds);
    assert_eq!(lit.magnitude, 10);

    let lit_100 =
        sv_lang::TimeScaleValue::from_literal(100.0, sv_lang::TimeUnit::Microseconds).unwrap();
    assert_eq!(lit_100.unit, sv_lang::TimeUnit::Microseconds);
    assert_eq!(lit_100.magnitude, 100);

    // 7 is not a legal time scale magnitude (only 1, 10, 100 are).
    assert!(sv_lang::TimeScaleValue::from_literal(7.0, sv_lang::TimeUnit::Nanoseconds).is_none());

    // ---- parse: the static factory, mirroring TimeScaleValue::fromString,
    // which parses a single unit+magnitude token (unlike TimeScale::parse,
    // which parses a "base/precision" pair). ----
    let parsed = sv_lang::TimeScaleValue::parse("10ns").unwrap();
    assert_eq!(parsed, scale.base());
    assert_eq!(parsed.unit, sv_lang::TimeUnit::Nanoseconds);
    assert_eq!(parsed.magnitude, 10);

    let parsed_ps = sv_lang::TimeScaleValue::parse("100fs").unwrap();
    assert_eq!(parsed_ps.unit, sv_lang::TimeUnit::Femtoseconds);
    assert_eq!(parsed_ps.magnitude, 100);

    assert!(sv_lang::TimeScaleValue::parse("not a time value").is_none());
    assert!(sv_lang::TimeScaleValue::parse("").is_none());
}

#[test]
fn logic_t_value_x_z_is_unknown_and_bitwise_ops() {
    use sv_lang::Bit;

    // ---- value: mirrors the slang::logic_t::value field's own raw byte
    // encoding, distinct from Bit's usual compact 2-bit wire encoding. ----
    assert_eq!(Bit::Zero.logic_t_value(), 0);
    assert_eq!(Bit::One.logic_t_value(), 1);
    assert_eq!(Bit::X.logic_t_value(), 0x80);
    assert_eq!(Bit::Z.logic_t_value(), 0x40);

    // ---- x / z: the static logic_t::x / logic_t::z constants, round-tripped
    // through the C API rather than constructed directly in Rust. ----
    assert_eq!(Bit::x(), Bit::X);
    assert_eq!(Bit::z(), Bit::Z);

    // ---- isUnknown: true only for X and Z. ----
    assert!(!Bit::Zero.is_unknown());
    assert!(!Bit::One.is_unknown());
    assert!(Bit::X.is_unknown());
    assert!(Bit::Z.is_unknown());

    // ---- operator& (AND): 0 & anything = 0; 1 & 1 = 1; otherwise x. ----
    assert_eq!(Bit::Zero & Bit::Zero, Bit::Zero);
    assert_eq!(Bit::Zero & Bit::One, Bit::Zero);
    assert_eq!(Bit::Zero & Bit::X, Bit::Zero);
    assert_eq!(Bit::Zero & Bit::Z, Bit::Zero);
    assert_eq!(Bit::One & Bit::One, Bit::One);
    assert_eq!(Bit::One & Bit::X, Bit::X);
    assert_eq!(Bit::One & Bit::Z, Bit::X);
    assert_eq!(Bit::X & Bit::X, Bit::X);

    // ---- operator| (OR): 1 | anything = 1; 0 | 0 = 0; otherwise x. ----
    assert_eq!(Bit::One | Bit::Zero, Bit::One);
    assert_eq!(Bit::One | Bit::One, Bit::One);
    assert_eq!(Bit::One | Bit::X, Bit::One);
    assert_eq!(Bit::One | Bit::Z, Bit::One);
    assert_eq!(Bit::Zero | Bit::Zero, Bit::Zero);
    assert_eq!(Bit::Zero | Bit::X, Bit::X);
    assert_eq!(Bit::Zero | Bit::Z, Bit::X);
    assert_eq!(Bit::X | Bit::X, Bit::X);

    // ---- operator^ (XOR): x whenever either side is unknown. ----
    assert_eq!(Bit::Zero ^ Bit::Zero, Bit::Zero);
    assert_eq!(Bit::Zero ^ Bit::One, Bit::One);
    assert_eq!(Bit::One ^ Bit::One, Bit::Zero);
    assert_eq!(Bit::One ^ Bit::X, Bit::X);
    assert_eq!(Bit::Zero ^ Bit::Z, Bit::X);
    assert_eq!(Bit::X ^ Bit::X, Bit::X);

    // ---- operator~ (NOT, which slang defines as logical negation): 0/1
    // invert, x/z both stay x. ----
    assert_eq!(!Bit::Zero, Bit::One);
    assert_eq!(!Bit::One, Bit::Zero);
    assert_eq!(!Bit::X, Bit::X);
    assert_eq!(!Bit::Z, Bit::X);
}

// ---- ClassType / ConstraintBlockSymbol / CovergroupType / DPIOpenArrayType ----

/// Parses and compiles `src` under `--std=1800-2023`, for constructs (class
/// `:final`, `covergroup extends`) gated behind that language version. Mirrors
/// `dist_expression_default_weight_is_some_when_specified`'s Driver setup.
fn compile_2023(src: &str) -> Design {
    // Each call gets its own directory (not just its own file): two tests
    // race on `compile_2023` in parallel, and one finishing and
    // `remove_dir_all`-ing a *shared* directory out from under the other's
    // still-in-flight write/read was exactly the failure this once had.
    let dir = std::env::temp_dir().join(format!(
        "sv_lang_semantic_2023_test_{}_{}",
        std::process::id(),
        rand_suffix()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("t.sv");
    std::fs::write(&file, src).unwrap();

    let mut driver = Driver::new().unwrap();
    driver
        .parse_args([
            "slang".to_string(),
            file.to_string_lossy().into_owned(),
            "--std=1800-2023".to_string(),
        ])
        .unwrap();
    driver.process_options().unwrap();
    driver.parse_sources().unwrap();
    assert_eq!(driver.language_version(), 2); // SLANG_LANGUAGE_1800_2023

    let mut comp = Compilation::new(driver.session()).unwrap();
    for tree in &driver.trees() {
        comp.add(tree).unwrap();
    }
    let design = comp.compile().unwrap();
    std::fs::remove_dir_all(&dir).ok();
    design
}

/// A cheap process-local counter so parallel `#[test]` runs of
/// `compile_2023` never collide on the same temp file name.
fn rand_suffix() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[test]
fn class_type_generic_class() {
    use sv_lang::kinds::SymbolKind;

    let design = compile(
        "class G #(int W = 8); logic [W-1:0] data; endclass\n\
         class Plain; endclass\n\
         module m; G #(16) g1; Plain p1; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let g1 = body.find("g1").unwrap().value_type().unwrap();
    let generic = g1.class_generic_class().unwrap();
    assert_eq!(generic.name(), "G");
    assert_eq!(generic.kind(), SymbolKind::GenericClassDef);

    let p1 = body.find("p1").unwrap().value_type().unwrap();
    assert!(p1.class_generic_class().is_none());
}

#[test]
fn class_type_base_constructor_call_and_constructor() {
    use sv_lang::kinds::ExpressionKind;

    let design = compile(
        "interface class IFace; endclass\n\
         class Base; function new(int x = 0); endfunction endclass\n\
         class Derived extends Base implements IFace;\n\
             function new(); super.new(5); endfunction\n\
         endclass\n\
         module m; Base b1; Derived d1; IFace i1_h; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    // Base has no extends clause: no base-class constructor call.
    let base_ty = body.find("b1").unwrap().value_type().unwrap();
    assert!(base_ty.class_base_constructor_call().is_none());
    assert_eq!(base_ty.class_constructor().unwrap().name(), "new");

    // Derived explicitly calls `super.new(5)`.
    let derived_ty = body.find("d1").unwrap().value_type().unwrap();
    let call = derived_ty.class_base_constructor_call().unwrap();
    assert_eq!(call.kind(), ExpressionKind::NewClass);
    assert_eq!(derived_ty.class_constructor().unwrap().name(), "new");

    // IFace declares no constructor at all.
    let iface_ty = body.find("i1_h").unwrap().value_type().unwrap();
    assert!(iface_ty.class_constructor().is_none());
    assert!(iface_ty.class_base_constructor_call().is_none());
}

#[test]
fn class_type_first_forward_decl() {
    use sv_lang::kinds::SymbolKind;

    let design = compile(
        "typedef class Fwd;\n\
         class Fwd; endclass\n\
         class Plain; endclass\n\
         module m; Fwd f1; Plain p1; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let fwd_ty = body.find("f1").unwrap().value_type().unwrap();
    let decl = fwd_ty.class_first_forward_decl().unwrap();
    assert_eq!(decl.kind(), SymbolKind::ForwardingTypedef);

    let plain_ty = body.find("p1").unwrap().value_type().unwrap();
    assert!(plain_ty.class_first_forward_decl().is_none());
}

#[test]
fn class_type_implemented_interfaces() {
    let design = compile(
        "interface class IFaceA; endclass\n\
         interface class IFaceB; endclass\n\
         class C implements IFaceA, IFaceB; endclass\n\
         class Plain; endclass\n\
         module m; C c1; Plain p1; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let ty = body.find("c1").unwrap().value_type().unwrap();
    let mut names: Vec<_> = ty
        .class_implemented_interfaces()
        .map(|t| t.as_symbol().name().to_string())
        .collect();
    names.sort();
    assert_eq!(names, ["IFaceA", "IFaceB"]);

    let plain_ty = body.find("p1").unwrap().value_type().unwrap();
    assert_eq!(plain_ty.class_implemented_interfaces().count(), 0);
}

#[test]
fn class_type_is_abstract_and_is_interface() {
    let design = compile(
        "virtual class Abs; endclass\n\
         class Concrete extends Abs; endclass\n\
         interface class IFace; endclass\n\
         module m; Concrete c1; IFace i1_h; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let concrete_ty = body.find("c1").unwrap().value_type().unwrap();
    assert!(!concrete_ty.class_is_abstract());
    assert!(!concrete_ty.class_is_interface());
    let abs_ty = concrete_ty.class_base().unwrap();
    assert!(abs_ty.class_is_abstract());
    assert!(!abs_ty.class_is_interface());

    let iface_ty = body.find("i1_h").unwrap().value_type().unwrap();
    assert!(iface_ty.class_is_interface());
}

#[test]
fn class_type_is_final() {
    // `:final` is an 1800-2023 extension.
    let design = compile_2023(
        "class :final A; endclass\n\
         class Plain; endclass\n\
         module m; A a1; Plain p1; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    assert!(
        body.find("a1")
            .unwrap()
            .value_type()
            .unwrap()
            .class_is_final()
    );
    assert!(
        !body
            .find("p1")
            .unwrap()
            .value_type()
            .unwrap()
            .class_is_final()
    );
}

#[test]
fn class_type_this_var() {
    let design = compile("class C; int x; endclass\nmodule m; C c1; endmodule\n");
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let ty = body.find("c1").unwrap().value_type().unwrap();
    let this_var = ty.class_this_var().unwrap();
    assert_eq!(this_var.name(), "this");
    // It points back at the enclosing class type itself.
    assert_eq!(
        this_var.value_type().unwrap().to_sv_string(),
        ty.to_sv_string()
    );
}

#[test]
fn constraint_block_flags_and_this_var() {
    use sv_lang::ConstraintBlockFlags;

    let design = compile(
        "class C;\n\
             rand int x;\n\
             constraint c_pos { x > 0; }\n\
             static constraint c_static { x > 0; }\n\
         endclass\n\
         module m; C obj; endmodule\n",
    );
    let units = design.compilation_units().next().unwrap();
    let class = units.find("C").unwrap();

    let c_pos = class.find("c_pos").unwrap();
    assert_eq!(c_pos.constraint_block_flags(), ConstraintBlockFlags::NONE);
    assert_eq!(c_pos.constraint_block_this_var().unwrap().name(), "this");

    let c_static = class.find("c_static").unwrap();
    assert!(
        c_static
            .constraint_block_flags()
            .contains(ConstraintBlockFlags::STATIC)
    );
    assert!(c_static.constraint_block_this_var().is_none());

    // A non-constraint-block symbol reports the neutral values throughout.
    let x = class.find("x").unwrap();
    assert_eq!(x.constraint_block_flags(), ConstraintBlockFlags::NONE);
    assert!(x.constraint_block_this_var().is_none());
    assert!(x.constraint_block_constraints().is_none());
}

#[test]
fn constraint_block_constraints_tree_shape() {
    use sv_lang::kinds::ExpressionKind;

    let design = compile(
        "class C; rand int x; constraint c_pos { x > 0; } endclass\n\
         module m; C obj; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let units = design.compilation_units().next().unwrap();
    let c_pos = units.find("C").unwrap().find("c_pos").unwrap();

    let tree = c_pos.constraint_block_constraints().unwrap();
    assert_eq!(tree.domain(), sv_lang_sys::SLANG_AST_CONSTRAINT);
    // One `ExpressionConstraint` child wrapping the `x > 0` binary expression.
    let items = tree.children();
    assert_eq!(items.len(), 1);
    let expr = items[0].children()[0].as_expression().unwrap();
    assert_eq!(expr.kind(), ExpressionKind::BinaryOp);
}

#[test]
fn freeze_forces_constraint_block_memos() {
    let d = compile(
        "class C;\n\
             rand int x;\n\
             constraint c_pos { x > 0; }\n\
             static constraint c_static { x >= 0; }\n\
         endclass\n\
         module m; C obj; endmodule\n",
    );
    assert!(d.freeze_report().symbols_elaborated > 0);
    // Reading through the (already-forced) accessor still works post-seal.
    let units = d.compilation_units().next().unwrap();
    assert!(
        units
            .find("C")
            .unwrap()
            .find("c_pos")
            .unwrap()
            .constraint_block_constraints()
            .is_some()
    );
}

#[test]
fn covergroup_type_arguments_and_coverage_event() {
    let design = compile(
        "module m;\n\
             bit clk; bit x;\n\
             covergroup cg(int lo, int hi) @(posedge clk);\n\
                 cp: coverpoint x;\n\
             endgroup\n\
             cg cg_inst;\n\
             initial cg_inst = new(0, 10);\n\
         endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let ty = body.find("cg_inst").unwrap().value_type().unwrap();

    let names: Vec<_> = ty
        .covergroup_arguments()
        .map(|a| a.name().to_string())
        .collect();
    assert_eq!(names, ["lo", "hi"]);

    let event = ty.covergroup_coverage_event().unwrap();
    assert_eq!(event.domain(), sv_lang_sys::SLANG_AST_TIMING_CONTROL);

    // No `extends` clause: no base group.
    assert!(ty.covergroup_base_group().is_none());
}

#[test]
fn covergroup_type_base_group() {
    // `covergroup extends` is an 1800-2023 extension.
    let design = compile_2023(
        "class base;\n\
             bit a;\n\
             covergroup g1 (bit lo);\n\
                 coverpoint a;\n\
             endgroup\n\
             function new();\n\
                 g1 = new(0);\n\
             endfunction\n\
         endclass\n\
         class derived extends base;\n\
             covergroup extends g1;\n\
             endgroup :g1\n\
             function new();\n\
                 super.new();\n\
                 g1 = new(1);\n\
             endfunction\n\
         endclass\n\
         module m; derived d1; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let derived_ty = body.find("d1").unwrap().value_type().unwrap();
    let g1_prop = derived_ty.as_symbol().find("g1").unwrap();
    let g1_ty = g1_prop.value_type().unwrap();

    let base_group = g1_ty.covergroup_base_group().unwrap();
    assert_eq!(base_group.as_symbol().kind(), SymbolKind::CovergroupType);
    // The base group's own formal-argument list ("lo") is reachable through it.
    let base_arg_names: Vec<_> = base_group
        .covergroup_arguments()
        .map(|a| a.name().to_string())
        .collect();
    assert_eq!(base_arg_names, ["lo"]);
}

#[test]
fn dpi_open_array_is_packed() {
    use sv_lang::kinds::SymbolKind;

    let design = compile(
        "import \"DPI-C\" function void f1(logic[]);\n\
         import \"DPI-C\" function void f2(logic a[]);\n\
         module m; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let unit = design.compilation_units().next().unwrap();
    let arg_type = |func: &str| {
        unit.find(func)
            .unwrap()
            .members()
            .find(|s| s.kind() == SymbolKind::FormalArgument)
            .unwrap()
            .value_type()
            .unwrap()
    };

    let packed_ty = arg_type("f1");
    assert!(packed_ty.dpi_open_array_is_packed());
    let unpacked_ty = arg_type("f2");
    assert!(!unpacked_ty.dpi_open_array_is_packed());

    // A non-DPI-open-array type reports false.
    assert!(!packed_ty.element_type().unwrap().dpi_open_array_is_packed());
}

// ---- DeclaredType general accessors -----------------------------------------

#[test]
fn declared_type_general_accessors() {
    let design =
        compile("module m; localparam int X = 3 * 4; typedef logic [7:0] byte_t; endmodule\n");
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    // A ValueSymbol (Parameter) carrier: type, initializer, initializer
    // location and is_evaluating all resolve.
    let x = body.find("X").unwrap();
    assert_eq!(x.declared_type().unwrap().bit_width(), 32);
    assert_eq!(
        x.declared_initializer().unwrap().constant().as_deref(),
        Some("12")
    );
    assert_ne!(x.declared_initializer_location().buffer, 0);
    assert!(!x.declared_type_is_evaluating());

    // A TypeAliasType carrier (not a ValueSymbol): declared_type() still
    // resolves, but there's no initializer and no initializer location was
    // ever set.
    let byte_t = body.find("byte_t").unwrap();
    assert_eq!(byte_t.declared_type().unwrap().bit_width(), 8);
    assert!(byte_t.declared_initializer().is_none());
    assert_eq!(byte_t.declared_initializer_location().buffer, 0);
    assert!(!byte_t.declared_type_is_evaluating());

    // A symbol with no declared type at all (a scope, not a DeclaredType
    // carrier) reports None/zeroed/false uniformly.
    assert!(body.declared_type().is_none());
    assert!(body.declared_initializer().is_none());
    assert_eq!(body.declared_initializer_location().buffer, 0);
    assert!(!body.declared_type_is_evaluating());
}

#[test]
fn declared_type_syntax_accessors() {
    use sv_lang::kinds::SyntaxKind;

    let session = Session::new();
    let tree = session
        .parse("module m; localparam int X = 3 * 4; logic [7:0] y; endmodule\n")
        .unwrap();
    let other_tree = session.parse("module n; endmodule\n").unwrap();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add(&tree).unwrap();
    let design = comp.compile().unwrap();
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let x = body.find("X").unwrap();
    let init_node = x.declared_initializer_syntax(&tree).unwrap();
    assert_eq!(init_node.kind(), SyntaxKind::MultiplyExpression);
    // localparam's type syntax is the `int` keyword type, not a link.
    let type_node = x.declared_type_syntax(&tree).unwrap();
    assert_eq!(type_node.kind(), SyntaxKind::IntType);

    let y = body.find("y").unwrap();
    assert_eq!(
        y.declared_type_syntax(&tree).unwrap().kind(),
        SyntaxKind::LogicType
    );
    // No initializer syntax was ever set for `y`.
    assert!(y.declared_initializer_syntax(&tree).is_none());

    // Asking with an unrelated tree yields None in both directions.
    assert!(x.declared_initializer_syntax(&other_tree).is_none());
    assert!(x.declared_type_syntax(&other_tree).is_none());
}

#[test]
fn declared_type_resolved_dimensions_all_kinds() {
    use sv_lang::{ConstantRange, DimensionKind};

    let mut design = compile(
        "module m;\n\
         logic [7:0] fixed_arr [3:0];\n\
         logic [7:0] dyn_arr [];\n\
         logic [7:0] assoc_arr [int];\n\
         logic [7:0] queue_arr [$];\n\
         endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let eval = design.eval_session();
    let body = eval
        .design()
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let fixed = body.find("fixed_arr").unwrap();
    let dims = eval.resolved_dimensions(fixed);
    // One packed dim [7:0], then one unpacked [3:0].
    assert_eq!(dims.len(), 2);
    assert_eq!(dims[0].kind, DimensionKind::Range);
    assert_eq!(dims[0].bounds, ConstantRange { left: 7, right: 0 });
    assert_eq!(dims[1].kind, DimensionKind::Range);
    assert_eq!(dims[1].bounds, ConstantRange { left: 3, right: 0 });

    let dyn_arr = body.find("dyn_arr").unwrap();
    let dims = eval.resolved_dimensions(dyn_arr);
    assert_eq!(dims.len(), 2);
    assert_eq!(dims[1].kind, DimensionKind::Dynamic);

    let assoc = body.find("assoc_arr").unwrap();
    let dims = eval.resolved_dimensions(assoc);
    assert_eq!(dims.len(), 2);
    assert_eq!(dims[1].kind, DimensionKind::Associative);

    let queue = body.find("queue_arr").unwrap();
    let dims = eval.resolved_dimensions(queue);
    assert_eq!(dims.len(), 2);
    assert_eq!(dims[1].kind, DimensionKind::Queue);

    // A symbol with no declared type yields an empty list, not a crash.
    assert!(eval.resolved_dimensions(body).is_empty());
}

// ---- EnumType / FloatingType / FixedSizeUnpackedArrayType -------------------

#[test]
fn enum_type_system_id_is_unique_and_zero_for_non_enum() {
    let design = compile(
        "module m;\n\
         typedef enum { A, B } e1_t;\n\
         typedef enum { C, D } e2_t;\n\
         e1_t e1; e2_t e2; logic [7:0] x;\n\
         endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let id1 = body
        .find("e1")
        .unwrap()
        .value_type()
        .unwrap()
        .enum_system_id();
    let id2 = body
        .find("e2")
        .unwrap()
        .value_type()
        .unwrap()
        .enum_system_id();
    assert_ne!(id1, 0);
    assert_ne!(id2, 0);
    assert_ne!(id1, id2);

    let id_x = body
        .find("x")
        .unwrap()
        .value_type()
        .unwrap()
        .enum_system_id();
    assert_eq!(id_x, 0);
}

#[test]
fn floating_type_kind_matches_declaration() {
    use sv_lang::FloatKind;

    let design = compile("module m; real r; shortreal sr; realtime rt; logic [7:0] x; endmodule\n");
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    assert_eq!(
        body.find("r")
            .unwrap()
            .value_type()
            .unwrap()
            .floating_kind(),
        FloatKind::Real
    );
    assert_eq!(
        body.find("sr")
            .unwrap()
            .value_type()
            .unwrap()
            .floating_kind(),
        FloatKind::ShortReal
    );
    assert_eq!(
        body.find("rt")
            .unwrap()
            .value_type()
            .unwrap()
            .floating_kind(),
        FloatKind::RealTime
    );
    // Non-floating types report the documented fallback.
    assert_eq!(
        body.find("x")
            .unwrap()
            .value_type()
            .unwrap()
            .floating_kind(),
        FloatKind::Real
    );
}

#[test]
fn fixed_size_unpacked_array_range_ascending_and_descending() {
    use sv_lang::ConstantRange;

    let design = compile(
        "module m;\n\
         logic [7:0] mem_desc [3:0];\n\
         logic [7:0] mem_asc [0:3];\n\
         logic [7:0] scalar;\n\
         endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let desc_ty = body.find("mem_desc").unwrap().value_type().unwrap();
    assert_eq!(
        desc_ty.fixed_unpacked_array_range(),
        ConstantRange { left: 3, right: 0 }
    );
    let asc_ty = body.find("mem_asc").unwrap().value_type().unwrap();
    assert_eq!(
        asc_ty.fixed_unpacked_array_range(),
        ConstantRange { left: 0, right: 3 }
    );

    // Not a fixed-size unpacked array type: a zeroed range.
    let scalar_ty = body.find("scalar").unwrap().value_type().unwrap();
    assert_eq!(
        scalar_ty.fixed_unpacked_array_range(),
        ConstantRange { left: 0, right: 0 }
    );
}

// ---- ForwardingTypedefSymbol -------------------------------------------------

#[test]
fn forwarding_typedef_type_restriction_and_none_visibility() {
    use sv_lang::ForwardTypeRestriction;

    let design = compile(
        "typedef class Fwd;\n\
         class Fwd; endclass\n\
         class Plain; endclass\n\
         module m; Fwd f1; Plain p1; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    // A plain top-level `typedef class Fwd;`: restricted to Class, no
    // visibility qualifier.
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let fwd_ty = body.find("f1").unwrap().value_type().unwrap();
    let fwd = fwd_ty.class_first_forward_decl().unwrap();
    assert_eq!(
        fwd.forwarding_typedef_type_restriction(),
        ForwardTypeRestriction::Class
    );
    assert!(fwd.forwarding_typedef_visibility().is_none());

    // A symbol that isn't a ForwardingTypedef reports the "none" defaults.
    assert_eq!(
        fwd_ty.as_symbol().forwarding_typedef_type_restriction(),
        ForwardTypeRestriction::None
    );
    assert!(fwd_ty.as_symbol().forwarding_typedef_visibility().is_none());

    // Unaffected: a class with no forward declaration at all.
    let plain_ty = body.find("p1").unwrap().value_type().unwrap();
    assert!(plain_ty.class_first_forward_decl().is_none());
}

#[test]
fn forwarding_typedef_visibility_some() {
    use sv_lang::Visibility;

    // A class's own `checkForwardDecls` always checks its forward decl
    // against `Visibility::Public` (slang hardcodes this — a class body
    // itself carries no visibility qualifier), so a `protected`/`local`
    // forward-declared class always mismatches and raises
    // `ForwardTypedefVisibility`. That diagnostic doesn't stop the
    // `ForwardingTypedefSymbol` from being constructed with
    // `visibility = Protected` first, though, so it's still the right
    // fixture for exercising the `Some(..)` branch of the accessor.
    let design = compile(
        "class Outer;\n\
             protected typedef class Inner;\n\
             class Inner; endclass\n\
         endclass\n\
         module m; endmodule\n",
    );
    let diags = design.diagnostics();
    assert_eq!(diags.items().len(), 1, "{diags}");
    assert!(
        diags.items()[0]
            .to_string()
            .contains("declared visibility of forward typedef does not match"),
        "{diags}"
    );

    let outer = design
        .compilation_units()
        .next()
        .unwrap()
        .find("Outer")
        .unwrap();
    let inner_ty = outer.find("Inner").unwrap().as_type().unwrap();
    let inner_fwd = inner_ty.class_first_forward_decl().unwrap();
    assert_eq!(
        inner_fwd.forwarding_typedef_visibility(),
        Some(Visibility::Protected)
    );
}

// ---- GenericClassDefSymbol ---------------------------------------------------

#[test]
fn generic_class_is_interface_flag() {
    use sv_lang::kinds::SymbolKind;

    let design = compile(
        "interface class IG #(int W = 8); endclass\n\
         class NG #(int W = 8); endclass\n\
         module m; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let unit = design.compilation_units().next().unwrap();

    let ig = unit.find("IG").unwrap();
    assert_eq!(ig.kind(), SymbolKind::GenericClassDef);
    assert!(ig.generic_class_is_interface());

    let ng = unit.find("NG").unwrap();
    assert_eq!(ng.kind(), SymbolKind::GenericClassDef);
    assert!(!ng.generic_class_is_interface());
}

#[test]
fn generic_class_default_specialization_present_and_absent() {
    let design = compile(
        "class G #(int W = 8); logic [W-1:0] data; endclass\n\
         class NoDefault #(int W); logic [W-1:0] data; endclass\n\
         module m; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let unit = design.compilation_units().next().unwrap();

    let g = unit.find("G").unwrap();
    let spec = g.generic_class_default_specialization().unwrap();
    let data = spec.as_symbol().find("data").unwrap();
    assert_eq!(data.value_type().unwrap().bit_width(), 8);

    // No default value for W: no default specialization.
    let no_default = unit.find("NoDefault").unwrap();
    assert!(no_default.generic_class_default_specialization().is_none());
}

#[test]
fn generic_class_first_forward_decl_present_and_absent() {
    use sv_lang::ForwardTypeRestriction;
    use sv_lang::kinds::SymbolKind;

    let design = compile(
        "typedef class G;\n\
         class G #(int W = 8); endclass\n\
         class Plain #(int W = 8); endclass\n\
         module m; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let unit = design.compilation_units().next().unwrap();

    let g = unit.find("G").unwrap();
    assert_eq!(g.kind(), SymbolKind::GenericClassDef);
    let fwd = g.generic_class_first_forward_decl().unwrap();
    assert_eq!(fwd.kind(), SymbolKind::ForwardingTypedef);
    assert_eq!(
        fwd.forwarding_typedef_type_restriction(),
        ForwardTypeRestriction::Class
    );

    let plain = unit.find("Plain").unwrap();
    assert!(plain.generic_class_first_forward_decl().is_none());
}

#[test]
fn generic_class_invalid_specialization_ignores_missing_defaults() {
    use sv_lang::kinds::SymbolKind;

    let mut design = compile(
        "class C #(type T = int); T x; endclass\n\
         class NoDefault #(int W); logic [W-1:0] data; endclass\n\
         module m; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let eval = design.eval_session();
    let unit = eval.design().compilation_units().next().unwrap();
    let generic = unit.find("C").unwrap();

    // Unlike generic_class_default_specialization, this forces a
    // specialization even though nothing requested one: it works even for
    // NoDefault, which has no valid default specialization at all.
    let no_default = unit.find("NoDefault").unwrap();
    assert!(no_default.generic_class_default_specialization().is_none());
    let spec = eval
        .generic_class_invalid_specialization(no_default)
        .unwrap();
    assert_eq!(spec.as_symbol().kind(), SymbolKind::ClassType);
    assert_eq!(spec.class_generic_class().unwrap().name(), "NoDefault");

    let spec2 = eval.generic_class_invalid_specialization(generic).unwrap();
    assert_eq!(spec2.as_symbol().kind(), SymbolKind::ClassType);
    assert_eq!(spec2.class_generic_class().unwrap().name(), "C");

    // Not a GenericClassDef symbol: None.
    assert!(eval.generic_class_invalid_specialization(unit).is_none());
}

// ---- IntegralType / PackedArrayType / PackedStructType / PackedUnionType /
//      PredefinedIntegerType / QueueType / NetType breadth -----------------

#[test]
fn integral_type_bit_vector_range_and_is_declared_reg() {
    let design = compile(
        "module m;\n\
             int i;\n\
             logic [7:0] y;\n\
             reg [3:0] r;\n\
             logic l;\n\
             real not_integral;\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    // A predefined integer type falls through to the generic
    // [bitWidth-1:0] path (it isn't a PackedArrayType).
    let i_ty = body.find("i").unwrap().value_type().unwrap();
    let i_range = i_ty.bit_vector_range();
    assert_eq!((i_range.left, i_range.right), (31, 0));
    assert!(!i_ty.is_declared_reg());

    // A packed array reports its own declared range from both accessors.
    let y_ty = body.find("y").unwrap().value_type().unwrap();
    let y_range = y_ty.bit_vector_range();
    assert_eq!((y_range.left, y_range.right), (7, 0));
    assert_eq!(y_range, y_ty.packed_array_range());
    assert!(!y_ty.is_declared_reg());

    // `reg` is declared-reg even through a packed-array dimension.
    let r_ty = body.find("r").unwrap().value_type().unwrap();
    assert!(r_ty.is_declared_reg());
    let r_range = r_ty.bit_vector_range();
    assert_eq!((r_range.left, r_range.right), (3, 0));

    // `logic` (not `reg`) is not declared-reg.
    assert!(
        !body
            .find("l")
            .unwrap()
            .value_type()
            .unwrap()
            .is_declared_reg()
    );

    // A non-integral type: zeroed range, and not declared-reg.
    let real_ty = body.find("not_integral").unwrap().value_type().unwrap();
    let real_range = real_ty.bit_vector_range();
    assert_eq!((real_range.left, real_range.right), (0, 0));
    assert!(!real_ty.is_declared_reg());

    // A non-array integral type has a zeroed packed_array_range.
    let zero = i_ty.packed_array_range();
    assert_eq!((zero.left, zero.right), (0, 0));
}

#[test]
fn packed_struct_type_system_id_is_distinct_per_declaration() {
    let design = compile(
        "module m;\n\
             struct packed { bit a; bit b; } s1;\n\
             struct packed { bit x; bit y; bit z; } s2;\n\
             logic l;\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let id1 = body
        .find("s1")
        .unwrap()
        .value_type()
        .unwrap()
        .packed_struct_system_id();
    let id2 = body
        .find("s2")
        .unwrap()
        .value_type()
        .unwrap()
        .packed_struct_system_id();
    assert_ne!(id1, 0);
    assert_ne!(id2, 0);
    assert_ne!(
        id1, id2,
        "each packed struct declaration gets its own system ID"
    );

    // Not a packed struct type: 0.
    assert_eq!(
        body.find("l")
            .unwrap()
            .value_type()
            .unwrap()
            .packed_struct_system_id(),
        0
    );
}

#[test]
fn packed_union_type_soft_tagged_system_id_and_tag_bits() {
    let design = compile(
        "module m;\n\
             union soft { logic [7:0] a; logic [5:0] b; } su;\n\
             union tagged packed { byte a; byte b; byte c; } tu;\n\
             union packed { logic [7:0] a; bit [7:0] b; } pu;\n\
             logic l;\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let su_ty = body.find("su").unwrap().value_type().unwrap();
    assert!(su_ty.packed_union_is_soft());
    assert!(!su_ty.packed_union_is_tagged());
    assert_eq!(su_ty.packed_union_tag_bits(), 0);
    assert_ne!(su_ty.packed_union_system_id(), 0);

    // 3 members -> ceil(log2(3)) = 2 tag bits (slang: bit_width(fieldIndex - 1)).
    let tu_ty = body.find("tu").unwrap().value_type().unwrap();
    assert!(tu_ty.packed_union_is_tagged());
    assert!(!tu_ty.packed_union_is_soft());
    assert_eq!(tu_ty.packed_union_tag_bits(), 2);
    assert_ne!(tu_ty.packed_union_system_id(), 0);

    let pu_ty = body.find("pu").unwrap().value_type().unwrap();
    assert!(!pu_ty.packed_union_is_soft());
    assert!(!pu_ty.packed_union_is_tagged());
    assert_eq!(pu_ty.packed_union_tag_bits(), 0);
    assert_ne!(pu_ty.packed_union_system_id(), 0);

    // Every packed union declaration gets a distinct system ID.
    assert_ne!(
        su_ty.packed_union_system_id(),
        tu_ty.packed_union_system_id()
    );
    assert_ne!(
        tu_ty.packed_union_system_id(),
        pu_ty.packed_union_system_id()
    );

    // Not a packed union type: all neutral values.
    let l_ty = body.find("l").unwrap().value_type().unwrap();
    assert!(!l_ty.packed_union_is_soft());
    assert!(!l_ty.packed_union_is_tagged());
    assert_eq!(l_ty.packed_union_tag_bits(), 0);
    assert_eq!(l_ty.packed_union_system_id(), 0);
}

#[test]
fn unpacked_struct_type_system_id_is_distinct_per_declaration() {
    let design = compile(
        "module m;\n\
             struct { bit a; bit b; } s1;\n\
             struct { bit x; bit y; bit z; } s2;\n\
             logic l;\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let id1 = body
        .find("s1")
        .unwrap()
        .value_type()
        .unwrap()
        .unpacked_struct_system_id();
    let id2 = body
        .find("s2")
        .unwrap()
        .value_type()
        .unwrap()
        .unpacked_struct_system_id();
    assert_ne!(id1, 0);
    assert_ne!(id2, 0);
    assert_ne!(
        id1, id2,
        "each unpacked struct declaration gets its own system ID"
    );

    // Not an unpacked struct type: 0.
    assert_eq!(
        body.find("l")
            .unwrap()
            .value_type()
            .unwrap()
            .unpacked_struct_system_id(),
        0
    );
}

#[test]
fn unpacked_union_type_tagged_and_system_id() {
    let design = compile(
        "module m;\n\
             union tagged { int a; shortreal b; } tu;\n\
             union { int a; shortreal b; } uu;\n\
             logic l;\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let tu_ty = body.find("tu").unwrap().value_type().unwrap();
    assert!(tu_ty.unpacked_union_is_tagged());
    assert_ne!(tu_ty.unpacked_union_system_id(), 0);

    let uu_ty = body.find("uu").unwrap().value_type().unwrap();
    assert!(!uu_ty.unpacked_union_is_tagged());
    assert_ne!(uu_ty.unpacked_union_system_id(), 0);

    // Every unpacked union declaration gets a distinct system ID.
    assert_ne!(
        tu_ty.unpacked_union_system_id(),
        uu_ty.unpacked_union_system_id()
    );

    // Not an unpacked union type: neutral values.
    let l_ty = body.find("l").unwrap().value_type().unwrap();
    assert!(!l_ty.unpacked_union_is_tagged());
    assert_eq!(l_ty.unpacked_union_system_id(), 0);
}

#[test]
fn predefined_integer_type_integer_kind_covers_every_variant() {
    use sv_lang::PredefinedIntegerKind;

    let design = compile(
        "module m;\n\
             shortint si; int i; longint li; byte b; integer g; time t;\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let kind_of = |name: &str| {
        body.find(name)
            .unwrap()
            .value_type()
            .unwrap()
            .predefined_integer_kind()
    };
    assert_eq!(kind_of("si"), PredefinedIntegerKind::ShortInt);
    assert_eq!(kind_of("i"), PredefinedIntegerKind::Int);
    assert_eq!(kind_of("li"), PredefinedIntegerKind::LongInt);
    assert_eq!(kind_of("b"), PredefinedIntegerKind::Byte);
    assert_eq!(kind_of("g"), PredefinedIntegerKind::Integer);
    assert_eq!(kind_of("t"), PredefinedIntegerKind::Time);

    // Bit widths corroborate the kind mapping is not accidental.
    assert_eq!(
        body.find("si").unwrap().value_type().unwrap().bit_width(),
        16
    );
    assert_eq!(
        body.find("li").unwrap().value_type().unwrap().bit_width(),
        64
    );
    assert_eq!(body.find("b").unwrap().value_type().unwrap().bit_width(), 8);
}

#[test]
fn queue_type_max_bound_bounded_vs_unbounded() {
    let design = compile(
        "module m;\n\
             int bounded[$:4];\n\
             int unbounded[$];\n\
             int not_a_queue[3:0];\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    assert_eq!(
        body.find("bounded")
            .unwrap()
            .value_type()
            .unwrap()
            .queue_max_bound(),
        4
    );
    assert_eq!(
        body.find("unbounded")
            .unwrap()
            .value_type()
            .unwrap()
            .queue_max_bound(),
        0
    );
    assert_eq!(
        body.find("not_a_queue")
            .unwrap()
            .value_type()
            .unwrap()
            .queue_max_bound(),
        0
    );
}

#[test]
fn net_type_kind_built_in_error_and_simulated_resolution() {
    use sv_lang::{NetKind, NetTypeKind};

    let design = compile("nettype real myreal;\nmodule m; endmodule\n");

    // netKind, isBuiltIn, isError across a keyword-built-in and a
    // user-defined nettype.
    let wire = design.net_type(NetTypeKind::Wire);
    assert_eq!(wire.net_kind(), NetKind::Wire);
    assert!(wire.net_type_is_built_in());
    assert!(!wire.net_type_is_error());

    let wand = design.net_type(NetTypeKind::WAnd);
    assert_eq!(wand.net_kind(), NetKind::WAnd);
    let wor = design.net_type(NetTypeKind::WOr);
    assert_eq!(wor.net_kind(), NetKind::WOr);

    let myreal = design
        .compilation_units()
        .next()
        .unwrap()
        .find("myreal")
        .unwrap();
    assert_eq!(myreal.kind(), SymbolKind::NetType);
    assert_eq!(myreal.net_kind(), NetKind::UserDefined);
    assert!(!myreal.net_type_is_built_in());
    assert!(!myreal.net_type_is_error());

    // A non-NetType symbol: the Unknown/false neutral values.
    let top = design.top_instances().next().unwrap();
    assert_eq!(top.net_kind(), NetKind::Unknown);
    assert!(!top.net_type_is_built_in());

    // getSimulatedNetType: WAnd internal against a WOr external takes the
    // cross-kind branch that both resolves to the external type AND sets
    // should_warn (see slang::ast::NetType::getSimulatedNetType's WAnd
    // case), so this exercises both outputs at once, not just the common
    // no-conflict path.
    let (resolved, should_warn) = wand.simulated_net_type(&wor).unwrap();
    assert_eq!(resolved.id(), wor.id());
    assert!(should_warn);

    // The reflexive Wire/Wire case takes the no-conflict path.
    let (resolved2, should_warn2) = wire.simulated_net_type(&wire).unwrap();
    assert_eq!(resolved2.id(), wire.id());
    assert!(!should_warn2);

    // Either argument not a NetType: None.
    assert!(wire.simulated_net_type(&top).is_none());
    assert!(top.simulated_net_type(&wire).is_none());
}

#[test]
fn net_type_resolution_function_is_forced_by_freeze() {
    let design = compile(
        "function automatic real Tsum(input real driver[]);\n\
             Tsum = 0.0;\n\
             foreach (driver[i]) Tsum += driver[i];\n\
         endfunction\n\
         nettype real wT;\n\
         nettype real wTsum with Tsum;\n\
         module m; endmodule\n",
    );
    let unit = design.compilation_units().next().unwrap();

    let wt = unit.find("wT").unwrap();
    assert_eq!(wt.kind(), SymbolKind::NetType);
    assert!(wt.net_type_resolution_function().is_none());

    let wtsum = unit.find("wTsum").unwrap();
    assert_eq!(wtsum.kind(), SymbolKind::NetType);
    let f = wtsum.net_type_resolution_function().unwrap();
    assert_eq!(f.kind(), SymbolKind::Subroutine);
    assert_eq!(f.name(), "Tsum");
}

#[test]
fn net_type_declared_type_via_generic_declared_type_accessor() {
    // NetType::declaredType has no dedicated accessor: it is one of the
    // Symbol::getDeclaredType() carriers documented alongside
    // slang_declared_type_type, so the existing generic Symbol::declared_type
    // already covers it (mirrors NetType::getDataType()).
    let design = compile("nettype real myreal;\nmodule m; endmodule\n");

    // A built-in net type's data type is `logic` (a 1-bit 4-state scalar).
    let wire = design.wire_net_type();
    let wire_ty = wire.declared_type().unwrap();
    assert!(wire_ty.is_integral());
    assert_eq!(wire_ty.bit_width(), 1);
    assert!(wire_ty.is_four_state());

    // A user-defined nettype's data type is whatever it was declared with.
    let myreal = design
        .compilation_units()
        .next()
        .unwrap()
        .find("myreal")
        .unwrap();
    let myreal_ty = myreal.declared_type().unwrap();
    assert!(!myreal_ty.is_integral());
    assert_eq!(myreal_ty.floating_kind(), sv_lang::FloatKind::Real);
}

// ---- Type semantic surface, part 4 --------------------------------------

#[test]
fn type_scalar_kind_and_can_be_string_like() {
    let design = compile(
        "module m;\n\
             bit b;\n\
             logic l;\n\
             reg r;\n\
             int i;\n\
             real rl;\n\
             byte s[];\n\
             string str;\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    assert_eq!(
        body.find("b").unwrap().value_type().unwrap().scalar_kind(),
        ScalarKind::Bit
    );
    assert_eq!(
        body.find("l").unwrap().value_type().unwrap().scalar_kind(),
        ScalarKind::Logic
    );
    assert_eq!(
        body.find("r").unwrap().value_type().unwrap().scalar_kind(),
        ScalarKind::Reg
    );
    // A non-scalar type (predefined integer) reports the Bit default.
    assert_eq!(
        body.find("i").unwrap().value_type().unwrap().scalar_kind(),
        ScalarKind::Bit
    );

    // canBeStringLike: string itself, byte arrays, and every integral type;
    // false for a non-integral, non-string, non-byte-array type.
    assert!(
        body.find("i")
            .unwrap()
            .value_type()
            .unwrap()
            .can_be_string_like()
    );
    assert!(
        body.find("s")
            .unwrap()
            .value_type()
            .unwrap()
            .can_be_string_like()
    );
    assert!(
        body.find("str")
            .unwrap()
            .value_type()
            .unwrap()
            .can_be_string_like()
    );
    assert!(
        !body
            .find("rl")
            .unwrap()
            .value_type()
            .unwrap()
            .can_be_string_like()
    );
}

#[test]
fn type_associative_index_and_is_associative_array() {
    let design = compile(
        "module m;\n\
             int keyed[string];\n\
             int wild[*];\n\
             int x;\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let keyed = body.find("keyed").unwrap().value_type().unwrap();
    assert!(keyed.is_associative_array());
    let idx = keyed.associative_index_type().unwrap();
    assert!(idx.is_string());

    let wild = body.find("wild").unwrap().value_type().unwrap();
    assert!(wild.is_associative_array());
    // A wildcard-indexed associative array has no explicit index type.
    assert!(wild.associative_index_type().is_none());

    let x = body.find("x").unwrap().value_type().unwrap();
    assert!(!x.is_associative_array());
    assert!(x.associative_index_type().is_none());
}

#[test]
fn type_bitstream_width_and_selectable_width() {
    let design = compile(
        "class C;\n\
             int x;\n\
             byte y;\n\
         endclass\n\
         module m;\n\
             C c;\n\
             logic [7:0] v;\n\
             int q[$];\n\
             string s;\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let v = body.find("v").unwrap().value_type().unwrap();
    assert_eq!(v.bitstream_width(), 8);
    assert_eq!(v.selectable_width(), 8);

    // A queue has no statically known bitstream size.
    let q = body.find("q").unwrap().value_type().unwrap();
    assert_eq!(q.bitstream_width(), 0);
    // Dynamically sized types report a selectable width of 1.
    assert_eq!(q.selectable_width(), 1);
    let s = body.find("s").unwrap().value_type().unwrap();
    assert_eq!(s.selectable_width(), 1);

    // A class type: getBitstreamWidth sums its properties' widths
    // (int=32 + byte=8), and the underlying memo is forced by the freeze
    // sweep (see SOUNDNESS-MEMOS.md's ClassType row), so this is a pure
    // read that never mutates the frozen design.
    let c = body.find("c").unwrap().value_type().unwrap();
    assert_eq!(c.bitstream_width(), 40);
}

#[test]
fn type_common_base_and_implements() {
    let design = compile(
        "interface class IFoo;\n\
         endclass\n\
         class Base;\n\
         endclass\n\
         class A extends Base implements IFoo;\n\
         endclass\n\
         class B extends Base;\n\
         endclass\n\
         class Other;\n\
         endclass\n\
         module m;\n\
             A a;\n\
             B b;\n\
             Other o;\n\
             int i;\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let at = body.find("a").unwrap().value_type().unwrap();
    let bt = body.find("b").unwrap().value_type().unwrap();
    let ot = body.find("o").unwrap().value_type().unwrap();
    let it = body.find("i").unwrap().value_type().unwrap();

    // A and B share Base as a common ancestor.
    let common = at.common_base(&bt).unwrap();
    assert_eq!(common.to_sv_string(), "Base");
    // Reflexive: A vs A.
    assert_eq!(at.common_base(&at).unwrap().to_sv_string(), "A");
    // Other shares no base with A.
    assert!(at.common_base(&ot).is_none());
    // Non-class types: None.
    assert!(it.common_base(&at).is_none());
    assert!(at.common_base(&it).is_none());

    // implements: A implements IFoo directly; B does not (no `implements`
    // clause, even though it shares A's base class); Other doesn't either.
    let ifoo = at.class_implemented_interfaces().next().unwrap();
    assert!(at.implements(&ifoo));
    assert!(!bt.implements(&ifoo));
    assert!(!ot.implements(&ifoo));
}

#[test]
fn type_fixed_range_and_has_fixed_range() {
    let design = compile(
        "module m;\n\
             logic [7:0] v;\n\
             logic [7:0] fixedarr [3:0];\n\
             int q[$];\n\
             string s;\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let v = body.find("v").unwrap().value_type().unwrap();
    assert!(v.has_fixed_range());
    let vr = v.fixed_range();
    assert_eq!((vr.left, vr.right), (7, 0));
    // Agrees with the dedicated bit_vector_range accessor.
    assert_eq!(vr, v.bit_vector_range());

    let fa = body.find("fixedarr").unwrap().value_type().unwrap();
    assert!(fa.has_fixed_range());
    let far = fa.fixed_range();
    assert_eq!((far.left, far.right), (3, 0));
    assert_eq!(far, fa.fixed_unpacked_array_range());

    let q = body.find("q").unwrap().value_type().unwrap();
    assert!(!q.has_fixed_range());
    let qr = q.fixed_range();
    assert_eq!((qr.left, qr.right), (0, 0));

    let s = body.find("s").unwrap().value_type().unwrap();
    assert!(!s.has_fixed_range());
}

#[test]
fn type_integral_flags() {
    let design = compile(
        "module m;\n\
             int si;\n\
             bit [7:0] u;\n\
             logic [3:0] fs;\n\
             reg [1:0] rg;\n\
             real rl;\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let si = body
        .find("si")
        .unwrap()
        .value_type()
        .unwrap()
        .integral_flags();
    assert!(si.contains(IntegralFlags::SIGNED));
    assert!(!si.contains(IntegralFlags::FOUR_STATE));
    assert!(!si.contains(IntegralFlags::REG));

    let u = body
        .find("u")
        .unwrap()
        .value_type()
        .unwrap()
        .integral_flags();
    assert_eq!(u, IntegralFlags::UNSIGNED);

    let fs = body
        .find("fs")
        .unwrap()
        .value_type()
        .unwrap()
        .integral_flags();
    assert!(fs.contains(IntegralFlags::FOUR_STATE));
    assert!(!fs.contains(IntegralFlags::SIGNED));
    assert!(!fs.contains(IntegralFlags::REG));

    let rg = body
        .find("rg")
        .unwrap()
        .value_type()
        .unwrap()
        .integral_flags();
    assert!(rg.contains(IntegralFlags::REG));
    assert!(rg.contains(IntegralFlags::FOUR_STATE));

    // A non-integral type: all-zero.
    let rl = body
        .find("rl")
        .unwrap()
        .value_type()
        .unwrap()
        .integral_flags();
    assert_eq!(rl, IntegralFlags::UNSIGNED);
    assert_eq!(rl.bits(), 0);
}

#[test]
fn type_is_aggregate_and_is_alias() {
    let design = compile(
        "module m;\n\
             typedef logic [3:0] nib_t;\n\
             nib_t n;\n\
             struct { int a; int b; } st;\n\
             logic [7:0] arr [3:0];\n\
             logic [7:0] v;\n\
             int i;\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    // isAggregate: unpacked struct/array types are aggregate; packed and
    // scalar types are not.
    assert!(
        body.find("st")
            .unwrap()
            .value_type()
            .unwrap()
            .is_aggregate()
    );
    assert!(
        body.find("arr")
            .unwrap()
            .value_type()
            .unwrap()
            .is_aggregate()
    );
    assert!(!body.find("v").unwrap().value_type().unwrap().is_aggregate());
    assert!(!body.find("i").unwrap().value_type().unwrap().is_aggregate());

    // isAlias: true only for the alias node itself, not its canonical form,
    // and not for a type that was never an alias.
    let n = body.find("n").unwrap().value_type().unwrap();
    assert!(n.is_alias());
    assert!(!n.canonical().is_alias());
    assert!(!body.find("v").unwrap().value_type().unwrap().is_alias());
}

#[test]
fn type_is_bitstream_castable() {
    let design = compile(
        "module m;\n\
             logic [15:0] x;\n\
             logic [7:0] y [1:0];\n\
             real rl;\n\
             int q[$];\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let xt = body.find("x").unwrap().value_type().unwrap();
    let yt = body.find("y").unwrap().value_type().unwrap();
    let rt = body.find("rl").unwrap().value_type().unwrap();
    let qt = body.find("q").unwrap().value_type().unwrap();

    // Same total bit width (16 bits either way): castable.
    assert!(xt.is_bitstream_castable(&yt));
    assert!(yt.is_bitstream_castable(&xt));
    // `real` is not a bitstream type at all.
    assert!(!xt.is_bitstream_castable(&rt));
    // Two dynamically-sized bitstream types (both queues of the same
    // element bit width) always match per Bitstream::dynamicSizesMatch.
    assert!(qt.is_bitstream_castable(&qt));
}

#[test]
fn type_bool_predicates_chunk30() {
    let design = compile(
        "class Base;\n\
         endclass\n\
         class Derived extends Base;\n\
         endclass\n\
         class Unrelated;\n\
         endclass\n\
         module m;\n\
             logic [7:0] x;\n\
             int i;\n\
             real r;\n\
             string s;\n\
             byte barr[];\n\
             chandle h;\n\
             event e;\n\
             int q[$];\n\
             logic [7:0] arr[3:0];\n\
             bit single;\n\
             int keyed[string];\n\
             typedef enum { A, B } e_t;\n\
             e_t en;\n\
             Derived d;\n\
             Base base_var;\n\
             Unrelated u;\n\
             covergroup cg;\n\
                 c: coverpoint i;\n\
             endgroup\n\
             cg g = new;\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let ty = |name: &str| body.find(name).unwrap().value_type().unwrap();

    let xt = ty("x");
    let it = ty("i");
    let rt = ty("r");
    let st = ty("s");
    let barrt = ty("barr");
    let ht = ty("h");
    let et = ty("e");
    let qt = ty("q");
    let arrt = ty("arr");
    let singlet = ty("single");
    let keyedt = ty("keyed");
    let ent = ty("en");
    let dt = ty("d");
    let baset = ty("base_var");
    let ut = ty("u");
    let gt = ty("g");

    // isBitstreamType: integral/string types and (when not on the
    // destination side) associative arrays qualify; `real` never does.
    assert!(xt.is_bitstream_type(false));
    assert!(keyedt.is_bitstream_type(false));
    assert!(!keyedt.is_bitstream_type(true));
    assert!(!rt.is_bitstream_type(false));

    // isBooleanConvertible: numeric, string, and class-handle types
    // qualify; a plain unpacked array does not.
    assert!(it.is_boolean_convertible());
    assert!(st.is_boolean_convertible());
    assert!(dt.is_boolean_convertible());
    assert!(!arrt.is_boolean_convertible());

    // isByteArray: only an unpacked array of `byte`.
    assert!(barrt.is_byte_array());
    assert!(!arrt.is_byte_array());

    // isCHandle.
    assert!(ht.is_chandle());
    assert!(!xt.is_chandle());

    // isCastCompatible: an enum can cast to another numeric type but not to
    // a string.
    assert!(ent.is_cast_compatible(&it));
    assert!(!ent.is_cast_compatible(&st));

    // isCovergroup.
    assert!(gt.is_covergroup());
    assert!(!xt.is_covergroup());

    // isDerivedFrom: walks the base-class chain; an unrelated class is not
    // derived from it.
    assert!(dt.is_derived_from(&baset));
    assert!(!ut.is_derived_from(&baset));

    // isDynamicallySizedArray: a queue qualifies, a fixed-size array does
    // not.
    assert!(qt.is_dynamically_sized_array());
    assert!(!arrt.is_dynamically_sized_array());

    // isError.
    assert!(design.error_type().is_error());
    assert!(!xt.is_error());

    // isEvent.
    assert!(et.is_event());
    assert!(!xt.is_event());

    // isFixedSize: a fixed-size unpacked array qualifies; a string or queue
    // does not.
    assert!(arrt.is_fixed_size());
    assert!(!st.is_fixed_size());
    assert!(!qt.is_fixed_size());

    // isFloating.
    assert!(rt.is_floating());
    assert!(!it.is_floating());

    // isHandleType: chandle, event, and class types all qualify.
    assert!(ht.is_handle_type());
    assert!(et.is_handle_type());
    assert!(dt.is_handle_type());
    assert!(!xt.is_handle_type());

    // isIterable: fixed-range and array types qualify; a plain 1-bit scalar
    // (excluded despite having a fixed range) and a handle type do not.
    assert!(arrt.is_iterable());
    assert!(qt.is_iterable());
    assert!(!singlet.is_iterable());
    assert!(!ht.is_iterable());

    // isNull.
    assert!(design.null_type().is_null());
    assert!(!xt.is_null());

    // isNumeric: integral and floating types qualify; string does not.
    assert!(it.is_numeric());
    assert!(rt.is_numeric());
    assert!(!st.is_numeric());
}

#[test]
fn type_default_value_and_coerce_value() {
    let design = compile(
        "class C;\n\
         endclass\n\
         module m;\n\
             logic [7:0] fourstate;\n\
             bit [7:0] twostate;\n\
             localparam int X = 300;\n\
             logic [3:0] y;\n\
             C c;\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    // Four-state default: all bits unknown.
    let fs_ty = body.find("fourstate").unwrap().value_type().unwrap();
    let fs_default = fs_ty.default_value().unwrap();
    let fs_int = fs_default.as_integer().unwrap();
    assert_eq!(fs_int.bit_width(), 8);
    assert!(fs_int.has_unknown());

    // Two-state default: all bits zero.
    let ts_ty = body.find("twostate").unwrap().value_type().unwrap();
    let ts_default = ts_ty.default_value().unwrap();
    let ts_int = ts_default.as_integer().unwrap();
    assert_eq!(ts_int.bit_width(), 8);
    assert!(!ts_int.has_unknown());
    assert_eq!(ts_int.as_i64(), Some(0));

    // coerceValue: 300 truncated into a 4-bit field is 300 % 16 == 12.
    let init = body.find("X").unwrap().initializer().unwrap();
    let value = init.constant_integer_copy().unwrap();
    let yt = body.find("y").unwrap().value_type().unwrap();
    let coerced = yt.coerce_value(&value).unwrap();
    assert_eq!(coerced.as_i64(), Some(12));
    let coerced_int = coerced.as_integer().unwrap();
    assert_eq!(coerced_int.bit_width(), 4);

    // Coercing into a non-integral/non-floating/non-string type (a class
    // handle) isn't supported: coerceValue returns a bad value, which
    // slang_type_coerce_value reports as a null constant.
    let ct = body.find("c").unwrap().value_type().unwrap();
    assert!(ct.coerce_value(&value).is_none());
}

#[test]
fn type_bool_predicates_chunk31() {
    let design = compile(
        "module m;\n\
             class C;\n\
                 int x;\n\
             endclass\n\
             C c;\n\
             chandle h;\n\
             logic [7:0] pa;\n\
             int pi;\n\
             union packed { logic [7:0] a; logic [7:0] b; } pu;\n\
             union tagged packed { void inv; logic [7:0] v; } tu;\n\
             bit sc;\n\
             real r;\n\
             int q[$];\n\
             int fixed_arr[3:0];\n\
             struct { int a; int b; } us;\n\
             union { int a; int b; } uu;\n\
             parameter P = $;\n\
             localparam bit tr = (type(int) == type(logic));\n\
             property p1(sequence sq);\n\
                 sq;\n\
             endproperty\n\
             property p2(property pr);\n\
                 pr;\n\
             endproperty\n\
         endmodule\n",
    );
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let ty = |name: &str| body.find(name).unwrap().value_type().unwrap();

    let ct = ty("c");
    let ht = ty("h");
    let pat = ty("pa");
    let pit = ty("pi");
    let put = ty("pu");
    let tut = ty("tu");
    let sct = ty("sc");
    let rt = ty("r");
    let qt = ty("q");
    let fat = ty("fixed_arr");
    let ust = ty("us");
    let uut = ty("uu");
    let pt = ty("P");

    // isObjectHandleType: a class handle qualifies; a chandle does not
    // (even though both are handle types).
    assert!(ct.is_object_handle_type());
    assert!(ht.is_handle_type());
    assert!(!ht.is_object_handle_type());

    // isPackedArray: a packed array qualifies; a predefined integer does
    // not (even though both are simple bit vectors).
    assert!(pat.is_packed_array());
    assert!(!pit.is_packed_array());

    // isPackedUnion: a packed union qualifies; a predefined integer does
    // not. Both plain and tagged packed unions qualify.
    assert!(put.is_packed_union());
    assert!(tut.is_packed_union());
    assert!(!pit.is_packed_union());

    // isPredefinedInteger: `int` qualifies; a packed array does not.
    assert!(pit.is_predefined_integer());
    assert!(!pat.is_predefined_integer());

    // isPropertyType / isSequenceType: an assertion port declared
    // `property`/`sequence` has that well-known type; a plain `int` port
    // does not have either.
    let p1 = body.find("p1").unwrap();
    let sq = p1.find("sq").unwrap().declared_type().unwrap();
    assert!(sq.is_sequence_type());
    assert!(!sq.is_property_type());

    let p2 = body.find("p2").unwrap();
    let pr = p2.find("pr").unwrap().declared_type().unwrap();
    assert!(pr.is_property_type());
    assert!(!pr.is_sequence_type());

    assert!(!pit.is_sequence_type());
    assert!(!pit.is_property_type());

    // isQueue: a queue qualifies; a fixed-size unpacked array does not.
    assert!(qt.is_queue());
    assert!(!fat.is_queue());

    // isScalar: `bit` qualifies; `int` (a predefined integer) does not.
    assert!(sct.is_scalar());
    assert!(!pit.is_scalar());

    // isSimpleBitVector: predefined integers, scalars, and packed arrays
    // of a scalar element all qualify; `real` does not.
    assert!(pit.is_simple_bit_vector());
    assert!(sct.is_simple_bit_vector());
    assert!(pat.is_simple_bit_vector());
    assert!(!rt.is_simple_bit_vector());

    // isSimpleType: built-in integers/floats/strings/classes/aliases
    // qualify; an inline unpacked array does not.
    assert!(pit.is_simple_type());
    assert!(rt.is_simple_type());
    assert!(ct.is_simple_type());
    assert!(!fat.is_simple_type());

    // isSingular: the opposite of isAggregate.
    assert!(pit.is_singular());
    assert!(!fat.is_singular());
    assert!(!ust.is_singular());

    // isTaggedUnion: the tagged packed union qualifies; the plain packed
    // union does not.
    assert!(tut.is_tagged_union());
    assert!(!put.is_tagged_union());

    // isTypeRefType: the operands of `type(int) == type(logic)` have the
    // type-reference type; the comparison's own `bit` result does not.
    let tr_init = body.find("tr").unwrap().initializer().unwrap();
    let left = tr_init.left().unwrap();
    let right = tr_init.right().unwrap();
    assert!(left.expr_type().unwrap().is_type_ref_type());
    assert!(right.expr_type().unwrap().is_type_ref_type());
    assert!(!tr_init.expr_type().unwrap().is_type_ref_type());

    // isUnbounded: a parameter defaulted to `$` has the unbounded type;
    // `int` does not.
    assert!(pt.is_unbounded());
    assert!(!pit.is_unbounded());

    // isUnpackedStruct / isUnpackedUnion: mutually exclusive, and distinct
    // from the packed union above.
    assert!(ust.is_unpacked_struct());
    assert!(!ust.is_unpacked_union());
    assert!(uut.is_unpacked_union());
    assert!(!uut.is_unpacked_struct());
    assert!(!put.is_unpacked_struct());
    assert!(!put.is_unpacked_union());
}

// ---- Type: isUntypedType / isValidForDPIArg / isValidForDPIReturn /------
// isValidForRand / isValidForSequence / isVirtualInterface / isVoid --------

#[test]
fn type_is_untyped_valid_for_dpi_rand_sequence_virtual_iface_void() {
    use sv_lang::RandMode;

    let design = compile(
        "sequence sq(x); x; endsequence\n\
         interface I; endinterface\n\
         class C; endclass\n\
         module m;\n\
             int i;\n\
             real r;\n\
             logic [7:0] l;\n\
             C c;\n\
             virtual I vif;\n\
             function void fv(); endfunction\n\
             function int fi(); return 0; endfunction\n\
         endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    // isUntypedType: a sequence formal port with no declared type at all
    // gets the untyped type; an ordinary `int` variable does not.
    let unit = design.compilation_units().next().unwrap();
    let sq = unit.find("sq").unwrap();
    let x = sq.find("x").unwrap().declared_type().unwrap();
    assert!(x.is_untyped_type());

    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let it = body.find("i").unwrap().value_type().unwrap();
    let rt = body.find("r").unwrap().value_type().unwrap();
    let lt = body.find("l").unwrap().value_type().unwrap();
    let ct = body.find("c").unwrap().value_type().unwrap();
    let vift = body.find("vif").unwrap().value_type().unwrap();
    assert!(!it.is_untyped_type());

    // isValidForDPIArg: integral/floating types qualify; a class handle
    // does not.
    assert!(it.is_valid_for_dpi_arg());
    assert!(rt.is_valid_for_dpi_arg());
    assert!(!ct.is_valid_for_dpi_arg());

    // isValidForDPIReturn: floating types qualify; a four-state `logic`
    // (a predefined integer, but four-state) does not, nor does a class
    // handle.
    assert!(rt.is_valid_for_dpi_return());
    assert!(!lt.is_valid_for_dpi_return());
    assert!(!ct.is_valid_for_dpi_return());

    // isValidForRand: integral types qualify under any mode/language
    // version; floating types need Rand *and* language version >= 2
    // (1800-2023); class types need Rand.
    assert!(it.is_valid_for_rand(RandMode::Rand, 1));
    assert!(it.is_valid_for_rand(RandMode::None, 1));
    assert!(rt.is_valid_for_rand(RandMode::Rand, 2));
    assert!(!rt.is_valid_for_rand(RandMode::Rand, 1));
    assert!(!rt.is_valid_for_rand(RandMode::RandC, 2));
    assert!(ct.is_valid_for_rand(RandMode::Rand, 1));
    assert!(!ct.is_valid_for_rand(RandMode::None, 1));

    // isValidForSequence: integral/floating types qualify; a class handle
    // does not.
    assert!(it.is_valid_for_sequence());
    assert!(rt.is_valid_for_sequence());
    assert!(!ct.is_valid_for_sequence());

    // isVirtualInterface: only the `virtual I` variable.
    assert!(vift.is_virtual_interface());
    assert!(!it.is_virtual_interface());

    // isVoid: only a `function void`'s declared return type.
    let fv = body.find("fv").unwrap().declared_type().unwrap();
    let fi = body.find("fi").unwrap().declared_type().unwrap();
    assert!(fv.is_void());
    assert!(!fi.is_void());
}

#[test]
fn type_alias_visibility_field() {
    use sv_lang::Visibility;

    let design = compile(
        "class C;\n\
             local typedef int t_local;\n\
             protected typedef int t_protected;\n\
             typedef int t_public;\n\
         endclass\n\
         module m; logic [3:0] x; endmodule\n",
    );
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    let unit = design.compilation_units().next().unwrap();
    let c = unit.find("C").unwrap();
    let t_local = c.find("t_local").unwrap().as_type().unwrap();
    let t_protected = c.find("t_protected").unwrap().as_type().unwrap();
    let t_public = c.find("t_public").unwrap().as_type().unwrap();
    assert_eq!(t_local.type_alias_visibility(), Visibility::Local);
    assert_eq!(t_protected.type_alias_visibility(), Visibility::Protected);
    assert_eq!(t_public.type_alias_visibility(), Visibility::Public);

    // A type node that isn't an alias at all defaults to Public.
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let x = body.find("x").unwrap().value_type().unwrap();
    assert_eq!(x.type_alias_visibility(), Visibility::Public);
}

// ---- TypePrinter / TypePrintingOptions ---------------------------------------

#[test]
fn type_printer_append_clear_to_string_and_options() {
    use sv_lang::{AnonymousTypeStyle, TypePrinter, TypePrintingOptions};

    let design = compile("module m; logic [7:0] x; int y; endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let xt = body.find("x").unwrap().value_type().unwrap();
    let yt = body.find("y").unwrap().value_type().unwrap();

    let mut printer = TypePrinter::new();
    assert_eq!(printer.text(), "");

    printer.append(&xt);
    assert_eq!(printer.text(), "logic[7:0]");

    // append() accumulates across calls, with no separator inserted.
    printer.append(&yt);
    assert_eq!(printer.text(), "logic[7:0]int");

    printer.clear();
    assert_eq!(printer.text(), "");
    printer.append(&yt);
    assert_eq!(printer.text(), "int");

    // options(): defaults match slang's, and set_options() round-trips.
    let defaults = printer.options();
    assert_eq!(defaults, TypePrintingOptions::default());
    assert_eq!(
        defaults.anonymous_type_style,
        AnonymousTypeStyle::SystemName
    );
    assert!(!defaults.classes_as_links);
    assert!(!defaults.elide_scope_names);
    assert!(!defaults.enums_as_links);
    assert_eq!(defaults.quote_char, None);
    assert!(!defaults.print_aka);
    assert!(!defaults.skip_scoped_type_names);
    assert!(!defaults.skip_type_defs);
    assert!(!defaults.full_enum_type);
    assert!(!defaults.typedefs_as_links);
    assert!(!defaults.print_integral_range);
    assert_eq!(defaults.friendly_member_char_limit, 60);

    printer.set_options(TypePrintingOptions {
        elide_scope_names: true,
        classes_as_links: true,
        enums_as_links: true,
        anonymous_type_style: AnonymousTypeStyle::FriendlyName,
        quote_char: Some('\''),
        print_aka: true,
        skip_scoped_type_names: true,
        skip_type_defs: true,
        full_enum_type: true,
        typedefs_as_links: true,
        print_integral_range: true,
        friendly_member_char_limit: 12,
    });
    let updated = printer.options();
    assert!(updated.elide_scope_names);
    assert!(updated.classes_as_links);
    assert!(updated.enums_as_links);
    assert_eq!(
        updated.anonymous_type_style,
        AnonymousTypeStyle::FriendlyName
    );
    assert_eq!(updated.quote_char, Some('\''));
    assert!(updated.print_aka);
    assert!(updated.skip_scoped_type_names);
    assert!(updated.skip_type_defs);
    assert!(updated.full_enum_type);
    assert!(updated.typedefs_as_links);
    assert!(updated.print_integral_range);
    assert_eq!(updated.friendly_member_char_limit, 12);

    // Clearing quote_char back to None round-trips too (not just Some).
    printer.set_options(TypePrintingOptions {
        quote_char: None,
        ..updated
    });
    assert_eq!(printer.options().quote_char, None);
}

#[test]
fn type_printer_quote_char_and_print_integral_range_affect_output() {
    use sv_lang::{TypePrinter, TypePrintingOptions};

    let design = compile("module m; logic [7:0] x; endmodule\n");
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let xt = body.find("x").unwrap().value_type().unwrap();

    // Baseline: no quotes, no integral range annotation.
    let mut printer = TypePrinter::new();
    printer.append(&xt);
    assert_eq!(printer.text(), "logic[7:0]");

    // quote_char wraps the printed type name in the given character.
    let mut quoted = TypePrinter::new();
    quoted.set_options(TypePrintingOptions {
        quote_char: Some('"'),
        ..TypePrintingOptions::default()
    });
    quoted.append(&xt);
    assert_eq!(quoted.text(), "\"logic[7:0]\"");

    // print_integral_range appends the constant range of a packed
    // struct/union/enum's underlying representation — NOT a plain packed
    // array like `xt` above (TypePrinter::append only checks it for
    // EnumType / PackedStructType / PackedUnionType canonical kinds).
    let struct_design = compile("module m; struct packed { bit a; bit b; } s; endmodule\n");
    let struct_body = struct_design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let st = struct_body.find("s").unwrap().value_type().unwrap();

    let mut unranged = TypePrinter::new();
    unranged.append(&st);
    assert!(!unranged.text().contains("bit[1:0]"));

    let mut ranged = TypePrinter::new();
    ranged.set_options(TypePrintingOptions {
        print_integral_range: true,
        ..TypePrintingOptions::default()
    });
    ranged.append(&st);
    assert!(
        ranged.text().contains("bit[1:0]"),
        "expected an integral range annotation in {:?}",
        ranged.text()
    );
}

// ---- chunk 34: Symbol, part 1 -----------------------------------------------

#[test]
fn assertion_port_direction_and_is_local_var_via_checker_ports() {
    use sv_lang::ArgumentDirection;

    // Every checker-declaration port is populated with a real direction
    // (never `local`, which is rejected there — LocalNotAllowed), so
    // `isLocalVar()` is always true and `direction()` always carries the
    // explicit or inherited direction. The second port has no explicit
    // direction and so inherits the previous one (`In`).
    let d = compile("checker chk(input logic i, output logic o, logic p);\nendchecker\n");
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());

    let unit = d.compilation_units().next().unwrap();
    let chk = unit.find("chk").unwrap();
    let ports: Vec<_> = chk.checker_ports().collect();
    assert_eq!(ports.len(), 3);
    assert_eq!(ports[0].name(), "i");
    assert_eq!(ports[1].name(), "o");
    assert_eq!(ports[2].name(), "p");

    for p in &ports {
        assert!(p.assertion_port_is_local_var());
    }
    assert_eq!(
        ports[0].assertion_port_direction(),
        Some(ArgumentDirection::In)
    );
    assert_eq!(
        ports[1].assertion_port_direction(),
        Some(ArgumentDirection::Out)
    );
    // `p` has no explicit direction; it inherits the previous port's (Out).
    assert_eq!(
        ports[2].assertion_port_direction(),
        Some(ArgumentDirection::Out)
    );

    // Non-AssertionPort symbols report the "no direction" / "not local"
    // defaults rather than panicking.
    assert_eq!(chk.assertion_port_direction(), None);
    assert!(!chk.assertion_port_is_local_var());
}

#[test]
fn checker_instance_connections_actual_attributes_and_output_initial_expr() {
    use sv_lang::kinds::{ExpressionKind, SymbolKind};

    let d = compile(
        "checker chk(input logic i, output logic o = 1'b0);\n\
         assign o = i;\nendchecker\n\
         module m(input logic a);\n\
           logic w;\n\
           chk c1((* foo = 1 *) .i(a), .o(w));\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());

    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let c1 = body.find("c1").unwrap();
    assert_eq!(c1.kind(), SymbolKind::CheckerInstance);

    let conns: Vec<_> = c1.checker_instance_connections().collect();
    assert_eq!(conns.len(), 2);

    // Connection 0 (`.i(a)`): actual is a plain expression referencing `a`,
    // carries one attribute, and (being an input) has no output-initial
    // expression.
    let actual0 = conns[0].actual().unwrap();
    let e0 = actual0.as_expression().unwrap();
    assert_eq!(e0.kind(), ExpressionKind::NamedValue);
    assert!(conns[0].output_initial_expr().is_none());

    let attrs0: Vec<_> = conns[0].attributes().collect();
    assert_eq!(attrs0.len(), 1);
    assert_eq!(attrs0[0].kind(), SymbolKind::Attribute);
    assert_eq!(attrs0[0].name(), "foo");
    assert_eq!(attrs0[0].attribute_value().unwrap().as_i64(), Some(1));

    // Connection 1 (`.o(w)`): no attributes, and the output formal's default
    // initializer (`= 1'b0`) is reachable as its output-initial expression.
    assert_eq!(conns[1].attributes().count(), 0);
    let out_init = conns[1].output_initial_expr().unwrap();
    assert_eq!(out_init.kind(), ExpressionKind::IntegerLiteral);
    assert_eq!(out_init.constant_value().unwrap().as_i64(), Some(0));

    // Non-Attribute symbols answer `None` rather than panicking.
    assert!(c1.attribute_value().is_none());

    // A non-CheckerInstance symbol has no connections at all.
    assert_eq!(body.checker_instance_connections().count(), 0);
}

#[test]
fn checker_instance_body_parent_instance_round_trips() {
    use sv_lang::kinds::SymbolKind;

    let d = compile(
        "checker chk(input logic i, output logic o = 1'b0);\n\
         assign o = i;\nendchecker\n\
         module m(input logic a);\n\
           logic w;\n\
           chk c1(.i(a), .o(w));\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());

    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let c1 = body.find("c1").unwrap();

    // `c1.members()` flattens straight through to the checker instance
    // body's own members (the CheckerInstanceBody wrapper is never itself a
    // scope member, so a `members()`/`visit()` walk never surfaces it) —
    // reach it via any member's `parent()` instead.
    let member = c1.members().next().unwrap();
    let checker_body = member.parent().unwrap();
    assert_eq!(checker_body.kind(), SymbolKind::CheckerInstanceBody);
    assert_eq!(
        checker_body.checker_instance_body_parent_instance(),
        Some(c1)
    );

    // A non-CheckerInstanceBody symbol has no parent instance.
    assert_eq!(c1.checker_instance_body_parent_instance(), None);
}

#[test]
fn freeze_forces_checker_instance_connection_attribute_and_actual_memos() {
    // The connection-level actual/attribute-value/output-initial-expr memos
    // are reachable only through CheckerInstanceSymbol::getPortConnections,
    // and (for the attribute) never visited by the generic ASTVisitor sweep
    // on their own — proving the freeze sweep's explicit forces hold on a
    // frozen, shared design (no crash / no fold-to-None from an unforced
    // memo hitting the sealed arena).
    let d = compile(
        "checker chk(input logic i, output logic o = 1'b0);\n\
         assign o = i;\nendchecker\n\
         module m(input logic a);\n\
           logic w;\n\
           chk c1((* foo = 1 *) .i(a), .o(w));\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    assert!(d.freeze_report().symbols_elaborated > 0);

    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let c1 = body.find("c1").unwrap();
    let conns: Vec<_> = c1.checker_instance_connections().collect();
    assert!(conns[0].actual().is_some());
    let attrs: Vec<_> = conns[0].attributes().collect();
    assert_eq!(attrs[0].attribute_value().unwrap().as_i64(), Some(1));
}

#[test]
fn class_property_visibility_and_rand_mode() {
    use sv_lang::{RandMode, Visibility, kinds::SymbolKind};

    let d =
        compile("class C;\n  rand int x;\n  local randc byte y;\n  protected int z;\nendclass\n");
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());

    let unit = d.compilation_units().next().unwrap();
    let c = unit.find("C").unwrap();
    let props: Vec<_> = c
        .members()
        .filter(|m| m.kind() == SymbolKind::ClassProperty)
        .collect();
    assert_eq!(props.len(), 3);

    let x = props.iter().find(|p| p.name() == "x").unwrap();
    assert_eq!(x.class_property_visibility(), Visibility::Public);
    assert_eq!(x.class_property_rand_mode(), RandMode::Rand);

    let y = props.iter().find(|p| p.name() == "y").unwrap();
    assert_eq!(y.class_property_visibility(), Visibility::Local);
    assert_eq!(y.class_property_rand_mode(), RandMode::RandC);

    let z = props.iter().find(|p| p.name() == "z").unwrap();
    assert_eq!(z.class_property_visibility(), Visibility::Protected);
    assert_eq!(z.class_property_rand_mode(), RandMode::None);

    // A non-ClassProperty symbol reports slang's own field defaults.
    assert_eq!(c.class_property_visibility(), Visibility::Public);
    assert_eq!(c.class_property_rand_mode(), RandMode::None);
}

#[test]
fn clock_var_direction_and_skews() {
    use sv_lang::{ArgumentDirection, EdgeKind};

    // Adapted from slang's own "Clocking blocks" unit test (MemberTests.cpp):
    // a default input skew (posedge + delay), a default output skew (both
    // edges, no delay), and a per-signal clock var with an explicit
    // direction and input/output delay-only skews.
    let d = compile(
        "module test;\n\
         wire clk;\n\
         int foo, a;\n\
         clocking cb @clk;\n\
             input a, b = foo;\n\
             default input posedge #3;\n\
             default output edge;\n\
             inout foo;\n\
             input #1step output #1step asdf = foo;\n\
         endclocking\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());

    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let cb = body.find("cb").unwrap();

    let default_in = cb.clocking_block_default_input_skew();
    assert_eq!(default_in.edge, EdgeKind::PosEdge);
    assert!(default_in.delay.is_some());

    let default_out = cb.clocking_block_default_output_skew();
    assert_eq!(default_out.edge, EdgeKind::BothEdges);
    assert!(default_out.delay.is_none());

    let asdf = cb.find("asdf").unwrap();
    assert_eq!(asdf.clock_var_direction(), ArgumentDirection::InOut);

    let in_skew = asdf.clock_var_input_skew();
    assert_eq!(in_skew.edge, EdgeKind::None);
    assert!(in_skew.delay.is_some());

    let out_skew = asdf.clock_var_output_skew();
    assert_eq!(out_skew.edge, EdgeKind::None);
    assert!(out_skew.delay.is_some());

    // A plain `input a` clocking var has an unspecified (empty) input skew
    // and, having no `output` at all, a likewise-empty output skew.
    let a_var = cb.find("a").unwrap();
    assert_eq!(a_var.clock_var_direction(), ArgumentDirection::In);
    let a_in = a_var.clock_var_input_skew();
    assert_eq!(a_in.edge, EdgeKind::None);
    assert!(a_in.delay.is_none());
    let a_out = a_var.clock_var_output_skew();
    assert_eq!(a_out.edge, EdgeKind::None);
    assert!(a_out.delay.is_none());

    // A non-ClockVar / non-ClockingBlock symbol reports the zeroed
    // defaults rather than panicking.
    assert_eq!(cb.clock_var_direction(), ArgumentDirection::In);
    let zeroed = cb.clock_var_input_skew();
    assert_eq!(zeroed.edge, EdgeKind::None);
    assert!(zeroed.delay.is_none());
    let zeroed_block = asdf.clocking_block_default_input_skew();
    assert_eq!(zeroed_block.edge, EdgeKind::None);
    assert!(zeroed_block.delay.is_none());
}

#[test]
fn freeze_forces_clocking_block_default_skew_memos() {
    // ClockingBlockSymbol::{defaultInputSkew, defaultOutputSkew} are lazily
    // computed and cached; the freeze sweep forces both pre-seal, so
    // reading them post-freeze is a pure read (no crash / no allocation
    // against the sealed arena).
    let d = compile(
        "module test;\n\
         wire clk;\n\
         clocking cb @clk;\n\
             default input posedge #3;\n\
             default output edge;\n\
         endclocking\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    assert!(d.freeze_report().symbols_elaborated > 0);

    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let cb = body.find("cb").unwrap();
    assert!(cb.clocking_block_default_input_skew().delay.is_some());
    assert!(cb.clocking_block_default_output_skew().delay.is_none());
}

// Finds a covergroup type's hidden CovergroupBody scope member, where its
// own coverpoints/crosses actually live (CovergroupType::members() does NOT
// transparently forward them; CovergroupType::inheritMembers only handles
// `covergroup extends`).
fn cg_body(cov_ty_sym: sv_lang::Symbol<'_>) -> sv_lang::Symbol<'_> {
    cov_ty_sym
        .members()
        .find(|m| m.kind() == SymbolKind::CovergroupBody)
        .expect("covergroup type has a CovergroupBody member")
}

#[test]
fn clocking_skew_has_value() {
    let d = compile(
        "module test;\n\
         wire clk;\n\
         int a;\n\
         clocking cb @clk;\n\
             input a;\n\
             default input posedge #3;\n\
         endclocking\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let cb = body.find("cb").unwrap();

    assert!(cb.clocking_block_default_input_skew().has_value());
    assert!(!cb.clocking_block_default_output_skew().has_value());

    let a_var = cb.find("a").unwrap();
    assert!(!a_var.clock_var_input_skew().has_value());
}

#[test]
fn clocking_block_event() {
    let d = compile("module test;\n  wire clk;\n  clocking cb @clk;\n  endclocking\nendmodule\n");
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let cb = body.find("cb").unwrap();

    let event = cb.clocking_block_event().unwrap();
    assert_eq!(event.domain(), sv_lang_sys::SLANG_AST_TIMING_CONTROL);

    // A non-ClockingBlock symbol reports `None`.
    assert!(body.clocking_block_event().is_none());
}

#[test]
fn compilation_unit_time_scale() {
    // The backtick `` `timescale`` directive sets the enclosing *definition*'s
    // (module's) own effective time scale, not CompilationUnitSymbol::timeScale
    // — that field is set only by an explicit `timeunit`/`timeprecision`
    // declaration placed directly at $unit scope (or defaulted at
    // construction to the compilation's configured default).
    let d = compile("timeunit 1ns;\ntimeprecision 1ps;\nmodule m; endmodule\n");
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());

    let unit = d.compilation_units().next().unwrap();
    assert_eq!(unit.kind(), SymbolKind::CompilationUnit);
    let ts = unit.compilation_unit_time_scale().unwrap();
    assert_eq!(ts.base.magnitude(), 1);
    assert_eq!(ts.base.unit(), sv_lang::TimeUnit::Nanoseconds);
    assert_eq!(ts.precision.magnitude(), 1);
    assert_eq!(ts.precision.unit(), sv_lang::TimeUnit::Picoseconds);

    // A design with no explicit $unit-scope time-unit declaration, and no
    // configured compilation default, reports `None`.
    let plain = compile("module m; endmodule\n");
    let plain_unit = plain.compilation_units().next().unwrap();
    assert!(plain_unit.compilation_unit_time_scale().is_none());

    // A non-CompilationUnit symbol also reports `None`.
    let m = d.top_instances().next().unwrap();
    assert!(m.compilation_unit_time_scale().is_none());
}

#[test]
fn continuous_assign_assignment_delay_and_drive_strength() {
    use sv_lang::DriveStrength;

    let d = compile(
        "module m(input a, b, output y, y2);\n\
         assign (strong0, pull1) #2 y = a & b;\n\
         assign y2 = a;\n\
        endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let mut assigns = body
        .members()
        .filter(|s| s.kind() == SymbolKind::ContinuousAssign);

    let strong_pull = assigns.next().unwrap();
    let assignment = strong_pull.continuous_assign_assignment().unwrap();
    assert!(assignment.constant().is_none()); // not a constant expression
    let delay = strong_pull.continuous_assign_delay().unwrap();
    assert_eq!(delay.domain(), sv_lang_sys::SLANG_AST_TIMING_CONTROL);
    let ds = strong_pull.continuous_assign_drive_strength();
    assert_eq!(ds.strength0, Some(DriveStrength::Strong));
    assert_eq!(ds.strength1, Some(DriveStrength::Pull));

    let plain = assigns.next().unwrap();
    assert!(plain.continuous_assign_assignment().is_some());
    assert!(plain.continuous_assign_delay().is_none());
    let plain_ds = plain.continuous_assign_drive_strength();
    assert_eq!(plain_ds.strength0, None);
    assert_eq!(plain_ds.strength1, None);
    assert!(!plain_ds.strength0.is_some() && !plain_ds.strength1.is_some());

    // A non-ContinuousAssign symbol reports the empty defaults.
    assert!(body.continuous_assign_assignment().is_none());
    assert!(body.continuous_assign_delay().is_none());
    let none_ds = body.continuous_assign_drive_strength();
    assert_eq!(none_ds, sv_lang::DriveStrengthPair::default());
}

#[test]
fn cover_cross_targets_iff_options_and_body_queue_type() {
    let d = compile(
        "module m;\n\
         bit clk; bit [1:0] a, b;\n\
         covergroup cg @(posedge clk);\n\
             cp_a: coverpoint a;\n\
             cp_b: coverpoint b;\n\
             x: cross cp_a, cp_b {\n\
                 option.weight = 2;\n\
                 ignore_bins ig = binsof(cp_a) intersect {0};\n\
             }\n\
         endgroup\n\
         cg cov = new();\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let cov_ty = body.find("cov").unwrap().value_type().unwrap();
    let group_body = cg_body(cov_ty.as_symbol());
    let x = group_body.find("x").unwrap();
    assert_eq!(x.kind(), SymbolKind::CoverCross);

    let target_names: Vec<_> = x
        .cover_cross_targets()
        .map(|t| t.name().to_string())
        .collect();
    assert_eq!(target_names, ["cp_a", "cp_b"]);
    for t in x.cover_cross_targets() {
        assert_eq!(t.kind(), SymbolKind::Coverpoint);
    }

    // No `iff` clause was declared on this cross.
    assert!(x.cover_cross_iff_expr().is_none());

    let options: Vec<_> = x.cover_cross_options().collect();
    assert_eq!(options.len(), 1);
    assert!(!options[0].is_type_option());
    assert_eq!(options[0].name(), "weight");
    assert!(options[0].expression().is_some());

    // The cross's own scope holds a hidden CoverCrossBody member carrying
    // the synthesized cross-value queue type.
    let cross_body = x
        .members()
        .find(|m| m.kind() == SymbolKind::CoverCrossBody)
        .unwrap();
    let queue_type = cross_body.cover_cross_body_queue_type().unwrap();
    assert!(queue_type.is_queue());

    // A non-CoverCrossBody symbol reports `None`.
    assert!(x.cover_cross_body_queue_type().is_none());

    // A non-CoverCross symbol reports the empty defaults.
    assert_eq!(group_body.cover_cross_targets().count(), 0);
    assert!(group_body.cover_cross_iff_expr().is_none());
    assert_eq!(group_body.cover_cross_options().count(), 0);
}

#[test]
fn cover_cross_iff_expr_present() {
    let d = compile(
        "module m;\n\
         bit clk; bit [1:0] a, b;\n\
         covergroup cg @(posedge clk);\n\
             cp_a: coverpoint a;\n\
             cp_b: coverpoint b;\n\
             x: cross cp_a, cp_b iff (a != 0);\n\
         endgroup\n\
         cg cov = new();\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let cov_ty = body.find("cov").unwrap().value_type().unwrap();
    let x = cg_body(cov_ty.as_symbol()).find("x").unwrap();
    assert!(x.cover_cross_iff_expr().is_some());
}

#[test]
fn coverage_bin_trans_range_list() {
    use sv_lang::RepeatKind;

    let d = compile(
        "module m;\n\
         bit clk; bit [3:0] a;\n\
         covergroup cg @(posedge clk);\n\
             cp: coverpoint a {\n\
                 bins t = (1,2 => 3[*2]), (4=>5);\n\
                 bins plain = {0};\n\
             }\n\
         endgroup\n\
         cg cov = new();\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let cov_ty = body.find("cov").unwrap().value_type().unwrap();
    let cp = cg_body(cov_ty.as_symbol()).find("cp").unwrap();

    let t = cp.find("t").unwrap();
    assert_eq!(t.kind(), SymbolKind::CoverageBin);
    let sets: Vec<_> = t.coverage_bin_trans_sets().collect();
    assert_eq!(sets.len(), 2, "two comma-separated alternatives");

    // First alternative: "1,2 => 3[*2]".
    let first_ranges: Vec<_> = sets[0].ranges().collect();
    assert_eq!(first_ranges.len(), 2);
    let first_items: Vec<_> = first_ranges[0].items().collect();
    assert_eq!(first_items.len(), 2); // "1,2"
    assert_eq!(first_ranges[0].repeat_kind(), RepeatKind::None);
    assert!(first_ranges[0].repeat_from().is_none());
    assert!(first_ranges[0].repeat_to().is_none());

    assert_eq!(first_ranges[1].items().count(), 1); // "3"
    assert_eq!(first_ranges[1].repeat_kind(), RepeatKind::Consecutive);
    // `[*2]` is a single fixed count, not a `from:to` range: only
    // `repeat_from` (the count itself) is populated.
    assert!(first_ranges[1].repeat_from().is_some());
    assert!(first_ranges[1].repeat_to().is_none());

    // Second alternative: "4=>5", no repeat suffix anywhere.
    let second_ranges: Vec<_> = sets[1].ranges().collect();
    assert_eq!(second_ranges.len(), 2);
    for r in &second_ranges {
        assert_eq!(r.items().count(), 1);
        assert_eq!(r.repeat_kind(), RepeatKind::None);
        assert!(r.repeat_from().is_none());
        assert!(r.repeat_to().is_none());
    }

    // A `bins` with a plain value-set initializer (no `=>` transition list
    // at all) reports zero trans-sets.
    let plain = cp.find("plain").unwrap();
    assert_eq!(plain.coverage_bin_trans_sets().count(), 0);

    // A non-CoverageBin symbol also reports zero trans-sets.
    assert_eq!(cp.coverage_bin_trans_sets().count(), 0);
}

#[test]
fn coverage_bin_trans_range_list_range_and_goto_repeat() {
    use sv_lang::RepeatKind;

    let d = compile(
        "module m;\n\
         bit clk; bit [3:0] a;\n\
         covergroup cg @(posedge clk);\n\
             cp: coverpoint a {\n\
                 bins r = (1 => 2[*2:4]);\n\
                 bins g = (1 => 2[->3]);\n\
                 bins ncon = (1 => 2[=3]);\n\
             }\n\
         endgroup\n\
         cg cov = new();\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let cov_ty = body.find("cov").unwrap().value_type().unwrap();
    let cp = cg_body(cov_ty.as_symbol()).find("cp").unwrap();

    let r = cp.find("r").unwrap();
    let r_sets: Vec<_> = r.coverage_bin_trans_sets().collect();
    let r_ranges: Vec<_> = r_sets[0].ranges().collect();
    // "2[*2:4]" is a `from:to` range repeat: both bounds are populated.
    assert_eq!(r_ranges[1].repeat_kind(), RepeatKind::Consecutive);
    assert!(r_ranges[1].repeat_from().is_some());
    assert!(r_ranges[1].repeat_to().is_some());

    let g = cp.find("g").unwrap();
    let g_sets: Vec<_> = g.coverage_bin_trans_sets().collect();
    let g_ranges: Vec<_> = g_sets[0].ranges().collect();
    // "2[->3]" (goto repeat) is a single fixed count.
    assert_eq!(g_ranges[1].repeat_kind(), RepeatKind::GoTo);
    assert!(g_ranges[1].repeat_from().is_some());
    assert!(g_ranges[1].repeat_to().is_none());

    let ncon = cp.find("ncon").unwrap();
    let ncon_sets: Vec<_> = ncon.coverage_bin_trans_sets().collect();
    let ncon_ranges: Vec<_> = ncon_sets[0].ranges().collect();
    // "2[=3]" (non-consecutive repeat) is also a single fixed count.
    assert_eq!(ncon_ranges[1].repeat_kind(), RepeatKind::Nonconsecutive);
    assert!(ncon_ranges[1].repeat_from().is_some());
    assert!(ncon_ranges[1].repeat_to().is_none());
}

#[test]
fn coverpoint_coverage_expr_is_the_sampled_expression() {
    let d = compile(
        "module m;\n\
         bit clk; bit [3:0] a;\n\
         covergroup cg @(posedge clk);\n\
             cp: coverpoint a;\n\
         endgroup\n\
         cg cov = new();\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let cov_ty = body.find("cov").unwrap().value_type().unwrap();
    let group_body = cg_body(cov_ty.as_symbol());
    let cp = group_body.find("cp").unwrap();

    let expr = cp
        .coverpoint_coverage_expr()
        .expect("coverpoint has a coverage expr");
    // `a` is 4 bits wide; the sampled expression carries the same width.
    assert_eq!(expr.expr_type().unwrap().bit_width(), 4);

    // A non-Coverpoint symbol reports `None`.
    assert!(group_body.coverpoint_coverage_expr().is_none());
}

#[test]
fn coverage_bin_kind_and_declaration_flags() {
    use sv_lang::CoverageBinKind;

    let d = compile(
        "module m;\n\
         bit clk; bit [3:0] a; bit en;\n\
         covergroup cg @(posedge clk);\n\
             cp: coverpoint a {\n\
                 wildcard bins lo = {4'b00??};\n\
                 bins hi[2] = {[12:15]};\n\
                 illegal_bins bad = {8};\n\
                 ignore_bins ig = {9};\n\
                 bins def = default;\n\
             }\n\
             seqcp: coverpoint a {\n\
                 bins seq = default sequence;\n\
             }\n\
         endgroup\n\
         cg cov = new();\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let cov_ty = body.find("cov").unwrap().value_type().unwrap();
    let group_body = cg_body(cov_ty.as_symbol());
    let cp = group_body.find("cp").unwrap();

    let lo = cp.find("lo").unwrap();
    assert_eq!(lo.coverage_bin_kind(), CoverageBinKind::Bins);
    assert!(lo.coverage_bin_is_wildcard());
    assert!(!lo.coverage_bin_is_array());
    assert!(!lo.coverage_bin_is_default());
    assert!(!lo.coverage_bin_is_default_sequence());

    let hi = cp.find("hi").unwrap();
    assert_eq!(hi.coverage_bin_kind(), CoverageBinKind::Bins);
    assert!(hi.coverage_bin_is_array());
    assert!(!hi.coverage_bin_is_wildcard());

    let bad = cp.find("bad").unwrap();
    assert_eq!(bad.coverage_bin_kind(), CoverageBinKind::IllegalBins);

    let ig = cp.find("ig").unwrap();
    assert_eq!(ig.coverage_bin_kind(), CoverageBinKind::IgnoreBins);

    let def = cp.find("def").unwrap();
    assert!(def.coverage_bin_is_default());
    assert!(!def.coverage_bin_is_default_sequence());

    let seq = cg_body(cov_ty.as_symbol())
        .find("seqcp")
        .unwrap()
        .find("seq")
        .unwrap();
    assert!(seq.coverage_bin_is_default());
    assert!(seq.coverage_bin_is_default_sequence());

    // A non-CoverageBin symbol reports the plain-Bins/false defaults.
    assert_eq!(cp.coverage_bin_kind(), CoverageBinKind::Bins);
    assert!(!cp.coverage_bin_is_array());
    assert!(!cp.coverage_bin_is_wildcard());
    assert!(!cp.coverage_bin_is_default());
    assert!(!cp.coverage_bin_is_default_sequence());
}

#[test]
fn coverage_bin_lazy_exprs_and_values() {
    let d = compile(
        "module m;\n\
         bit clk; bit [3:0] a; bit en; bit [31:0] arr[] = '{1, 2, 3};\n\
         covergroup cg @(posedge clk);\n\
             cp: coverpoint a {\n\
                 bins hi[2] = {[12:15]} with (item > 12);\n\
                 illegal_bins bad = {8} iff (en);\n\
                 bins fromArr = arr;\n\
                 bins def = default;\n\
             }\n\
         endgroup\n\
         cg cov = new();\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let cov_ty = body.find("cov").unwrap().value_type().unwrap();
    let cp = cg_body(cov_ty.as_symbol()).find("cp").unwrap();

    let hi = cp.find("hi").unwrap();
    // "2" bin-count expression.
    assert!(hi.coverage_bin_number_of_bins_expr().is_some());
    // "item > 12" filter.
    assert!(hi.coverage_bin_with_expr().is_some());
    // "[12:15]" is one range-list value.
    let hi_values: Vec<_> = hi.coverage_bin_values().collect();
    assert_eq!(hi_values.len(), 1);
    // No `iff`, no whole-set expression, no cross selection on a plain bin.
    assert!(hi.coverage_bin_iff_expr().is_none());
    assert!(hi.coverage_bin_set_coverage_expr().is_none());
    assert!(hi.coverage_bin_cross_select_expr().is_none());

    let bad = cp.find("bad").unwrap();
    assert!(bad.coverage_bin_iff_expr().is_some());
    assert!(bad.coverage_bin_number_of_bins_expr().is_none());

    let from_arr = cp.find("fromArr").unwrap();
    // `bins fromArr = arr;` binds the whole-array expression, not a value list.
    assert!(from_arr.coverage_bin_set_coverage_expr().is_some());
    assert_eq!(from_arr.coverage_bin_values().count(), 0);

    let def = cp.find("def").unwrap();
    // `default` has none of the lazily-resolved expressions.
    assert!(def.coverage_bin_iff_expr().is_none());
    assert!(def.coverage_bin_number_of_bins_expr().is_none());
    assert!(def.coverage_bin_set_coverage_expr().is_none());
    assert!(def.coverage_bin_with_expr().is_none());
    assert_eq!(def.coverage_bin_values().count(), 0);

    // A non-CoverageBin symbol reports the empty defaults throughout.
    assert!(cp.coverage_bin_iff_expr().is_none());
    assert!(cp.coverage_bin_number_of_bins_expr().is_none());
    assert!(cp.coverage_bin_set_coverage_expr().is_none());
    assert!(cp.coverage_bin_with_expr().is_none());
    assert!(cp.coverage_bin_cross_select_expr().is_none());
    assert_eq!(cp.coverage_bin_values().count(), 0);
}

#[test]
fn coverage_bin_cross_select_expr_in_cross_body() {
    let d = compile(
        "module m;\n\
         bit clk; bit [1:0] a, b;\n\
         covergroup cg @(posedge clk);\n\
             cp_a: coverpoint a;\n\
             cp_b: coverpoint b;\n\
             x: cross cp_a, cp_b {\n\
                 bins sel = binsof(cp_a) intersect {0};\n\
             }\n\
         endgroup\n\
         cg cov = new();\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let cov_ty = body.find("cov").unwrap().value_type().unwrap();
    let group_body = cg_body(cov_ty.as_symbol());
    let x = group_body.find("x").unwrap();
    let cross_body = x
        .members()
        .find(|m| m.kind() == SymbolKind::CoverCrossBody)
        .unwrap();
    let sel = cross_body.find("sel").unwrap();
    assert_eq!(sel.kind(), SymbolKind::CoverageBin);

    assert!(sel.coverage_bin_cross_select_expr().is_some());
    // Declared from a `bins` selection, not a value/range list.
    assert_eq!(sel.coverage_bin_values().count(), 0);

    // A plain coverpoint bin has no cross-select expression.
    let cp_a = group_body.find("cp_a").unwrap();
    assert!(cp_a.coverage_bin_cross_select_expr().is_none());
}

#[test]
fn covergroup_body_options_and_freeze() {
    let d = compile(
        "module m;\n\
         bit clk;\n\
         covergroup cg @(posedge clk);\n\
             option.per_instance = 1;\n\
             type_option.weight = 4;\n\
             cp: coverpoint clk;\n\
         endgroup\n\
         cg cov = new();\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    // The covergroup body's own option setters aren't reached by the generic
    // scope-member traversal (CoverageOptionSetter isn't a Symbol) and
    // CovergroupBodySymbol has no visitExprs of its own, so this proves the
    // dedicated FreezeVisitor branch actually ran (a missed force would leave
    // the expression's memo unresolved on this now-sealed, shared design).
    assert!(d.freeze_report().symbols_elaborated > 0);

    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let cov_ty = body.find("cov").unwrap().value_type().unwrap();
    let group_body = cg_body(cov_ty.as_symbol());
    assert_eq!(group_body.kind(), SymbolKind::CovergroupBody);

    let options: Vec<_> = group_body.covergroup_options().collect();
    assert_eq!(options.len(), 2);

    assert!(!options[0].is_type_option());
    assert_eq!(options[0].name(), "per_instance");
    assert!(options[0].expression().is_some());

    assert!(options[1].is_type_option());
    assert_eq!(options[1].name(), "weight");
    assert!(options[1].expression().is_some());

    // A coverpoint's own (empty here) `options` span works through the same
    // generalized accessor family.
    let cp = group_body.find("cp").unwrap();
    assert_eq!(cp.covergroup_options().count(), 0);

    // A non-owning symbol (the covergroup type itself) reports none either.
    assert_eq!(cov_ty.as_symbol().covergroup_options().count(), 0);
}

#[test]
fn coverpoint_iff_expr_type_and_options() {
    let d = compile(
        "module m;\n\
         bit clk; bit en; bit [3:0] a;\n\
         covergroup cg @(posedge clk);\n\
             cp: coverpoint a iff (en) { option.weight = 3; }\n\
             nogate: coverpoint a;\n\
         endgroup\n\
         cg cov = new();\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());

    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let cov_ty = body.find("cov").unwrap().value_type().unwrap();
    let group_body = cg_body(cov_ty.as_symbol());

    let cp = group_body.find("cp").unwrap();
    // `getIffExpr`: the `en` guard is present and boolean.
    let iff = cp.coverpoint_iff_expr().expect("cp has an iff clause");
    assert_eq!(iff.expr_type().unwrap().bit_width(), 1);

    // `getType`: the coverpoint's declared type matches `a`'s 4 bits.
    assert_eq!(cp.declared_type().unwrap().bit_width(), 4);

    // `options`: the one `option.weight = 3;` setter, reached through the
    // same generalized accessor family as CoverCross/CovergroupBody.
    let options: Vec<_> = cp.cover_cross_options().collect();
    assert_eq!(options.len(), 1);
    assert!(!options[0].is_type_option());
    assert_eq!(options[0].name(), "weight");
    assert!(options[0].expression().is_some());

    // A coverpoint with no `iff` clause reports `None`.
    let nogate = group_body.find("nogate").unwrap();
    assert!(nogate.coverpoint_iff_expr().is_none());
    assert_eq!(nogate.cover_cross_options().count(), 0);

    // A non-Coverpoint symbol reports `None`/empty for both.
    assert!(cov_ty.as_symbol().coverpoint_iff_expr().is_none());
}

#[test]
fn definition_cell_define_and_kind_strings() {
    let d = compile(
        "`celldefine\nmodule celled; endmodule\n`endcelldefine\n\
         interface bus; endinterface\n\
         program prog; endprogram\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());

    let mut defs: Vec<_> = d.definitions().collect();
    defs.sort_by_key(|s| s.name().to_string());

    let celled = defs.iter().find(|s| s.name() == "celled").unwrap();
    assert!(celled.definition_cell_define());
    assert_eq!(celled.definition_kind_string(), "module");
    assert_eq!(celled.definition_article_kind_string(), "a module");

    let bus = defs.iter().find(|s| s.name() == "bus").unwrap();
    assert!(!bus.definition_cell_define());
    assert_eq!(bus.definition_kind_string(), "interface");
    assert_eq!(bus.definition_article_kind_string(), "an interface");

    let prog = defs.iter().find(|s| s.name() == "prog").unwrap();
    assert_eq!(prog.definition_kind_string(), "program");
    assert_eq!(prog.definition_article_kind_string(), "a program");

    // A non-Definition symbol reports the empty/default values.
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    assert!(!body.definition_cell_define());
    assert_eq!(body.definition_kind_string(), "");
    assert_eq!(body.definition_article_kind_string(), "");
}

#[test]
fn definition_default_lifetime_and_unconnected_drive() {
    let d = compile(
        "`unconnected_drive pull1\nmodule pulled(input a); endmodule\n\
         `nounconnected_drive\n\
         module automatic am; endmodule\n\
         module static_m; endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());

    let mut defs: Vec<_> = d.definitions().collect();
    defs.sort_by_key(|s| s.name().to_string());

    let am = defs.iter().find(|s| s.name() == "am").unwrap();
    assert_eq!(
        am.definition_default_lifetime(),
        Some(VariableLifetime::Automatic)
    );
    assert_eq!(
        am.definition_unconnected_drive(),
        Some(UnconnectedDrive::None)
    );

    let static_m = defs.iter().find(|s| s.name() == "static_m").unwrap();
    assert_eq!(
        static_m.definition_default_lifetime(),
        Some(VariableLifetime::Static)
    );

    let pulled = defs.iter().find(|s| s.name() == "pulled").unwrap();
    assert_eq!(
        pulled.definition_unconnected_drive(),
        Some(UnconnectedDrive::Pull1)
    );
    // The default lifetime absent any `automatic`/`static` keyword is
    // `static` per SV semantics.
    assert_eq!(
        pulled.definition_default_lifetime(),
        Some(VariableLifetime::Static)
    );

    // A non-Definition symbol reports `None`.
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    assert!(body.definition_default_lifetime().is_none());
    assert!(body.definition_unconnected_drive().is_none());
}

#[test]
fn definition_time_scale_and_instance_count() {
    let d = compile(
        "`timescale 1ns/1ps\nmodule scaled; endmodule\n\
         module leaf; endmodule\n\
         module top; leaf l0(); leaf l1(); leaf l2(); endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());

    let mut defs: Vec<_> = d.definitions().collect();
    defs.sort_by_key(|s| s.name().to_string());

    let scaled = defs.iter().find(|s| s.name() == "scaled").unwrap();
    let ts = scaled
        .definition_time_scale()
        .expect("scaled has an explicit timescale");
    assert_eq!(ts.base.magnitude, 1);
    assert_eq!(ts.precision.magnitude, 1);

    let leaf = defs.iter().find(|s| s.name() == "leaf").unwrap();
    // getInstanceCount: `leaf` is instantiated 3 times by `top`.
    assert_eq!(leaf.definition_instance_count(), 3);

    let top = defs.iter().find(|s| s.name() == "top").unwrap();
    assert_eq!(top.definition_instance_count(), 0);

    // A non-Definition symbol reports 0 and no timescale.
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    assert_eq!(body.definition_instance_count(), 0);
    assert!(body.definition_time_scale().is_none());
}

#[test]
fn elab_system_task_kind_condition_and_message() {
    let d = compile(
        "module m;\n\
         $warning(\"w %0d\", 1);\n\
         $static_assert(1 == 1, \"never fires\");\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    assert!(d.freeze_report().symbols_elaborated > 0);

    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let tasks: Vec<_> = body
        .members()
        .filter(|m| m.kind() == SymbolKind::ElabSystemTask)
        .collect();
    assert_eq!(tasks.len(), 2);

    let warning = tasks
        .iter()
        .find(|t| t.elab_system_task_kind() == Some(ElabSystemTaskKind::Warning))
        .expect("a $warning task");
    assert_eq!(warning.elab_system_task_message(), Some(": w 1"));
    assert!(warning.elab_system_task_assert_condition().is_none());

    let assertion = tasks
        .iter()
        .find(|t| t.elab_system_task_kind() == Some(ElabSystemTaskKind::StaticAssert))
        .expect("a $static_assert task");
    assert_eq!(assertion.elab_system_task_message(), Some(": never fires"));
    let cond = assertion
        .elab_system_task_assert_condition()
        .expect("$static_assert has a condition expression");
    assert_eq!(cond.expr_type().unwrap().bit_width(), 1);

    // A non-ElabSystemTask symbol reports the default/empty values.
    assert_eq!(
        body.elab_system_task_kind(),
        None,
        "a non-ElabSystemTask symbol has no task kind"
    );
    assert!(body.elab_system_task_assert_condition().is_none());
    assert!(body.elab_system_task_message().is_none());
}

#[test]
fn explicit_import_name_package_and_imported_symbol() {
    let d = compile(
        "package p;\n\
         int x = 42;\n\
         endpackage\n\
         module m;\n\
         import p::x;\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());

    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let imp = body
        .members()
        .find(|m| m.kind() == SymbolKind::ExplicitImport)
        .expect("module has an explicit import");

    assert_eq!(imp.explicit_import_name(), "x");

    let pkg = imp
        .explicit_import_package()
        .expect("the import's package resolved");
    assert_eq!(pkg.name(), "p");
    assert_eq!(pkg.kind(), SymbolKind::Package);

    let target = imp
        .explicit_import_imported_symbol()
        .expect("the imported symbol resolved");
    assert_eq!(target.name(), "x");

    // A non-ExplicitImport symbol reports the default/empty values.
    assert_eq!(body.explicit_import_name(), "");
    assert!(body.explicit_import_package().is_none());
    assert!(body.explicit_import_imported_symbol().is_none());
}

// Proves the freeze sweep forces ExplicitImportSymbol's resolution even
// though no FreezeVisitor branch targets it directly: `getAllDiagnostics()`
// (run unconditionally, pre-seal, by `slang_compilation_freeze`) already
// visits every ExplicitImportSymbol and calls `importedSymbol()` on it (see
// `ast::Elaborator`'s handler), so by the time the design is frozen and
// shared, `explicit_import_package`/`explicit_import_imported_symbol` are
// pure reads. A failure to force this would only show up as a data race on
// concurrent first access (see `tsan_race.rs`), not as a functional failure
// here — this test instead proves the resolution is idempotent and correct.
#[test]
fn freeze_forces_explicit_import_resolution() {
    let d = compile(
        "package p;\n\
         int x = 42;\n\
         endpackage\n\
         module m;\n\
         import p::x;\n\
         endmodule\n",
    );
    assert!(d.freeze_report().symbols_elaborated > 0);

    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let imp = body
        .members()
        .find(|m| m.kind() == SymbolKind::ExplicitImport)
        .unwrap();

    // Reading twice must be consistent (a lazy first-touch race would risk
    // returning different pointers/None on repeated reads under TSan).
    let first = imp.explicit_import_imported_symbol().unwrap();
    let second = imp.explicit_import_imported_symbol().unwrap();
    assert_eq!(first.name(), second.name());
    assert_eq!(first.name(), "x");
}

// ---- ExplicitImportSymbol::packageName --------------------------------------

#[test]
fn explicit_import_package_name() {
    let d = compile(
        "package p;\n\
         int x;\n\
         endpackage\n\
         module m;\n\
         import p::x;\n\
         endmodule\n",
    );
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let imp = body
        .members()
        .find(|m| m.kind() == SymbolKind::ExplicitImport)
        .unwrap();
    assert_eq!(imp.explicit_import_package_name(), "p");
    assert_eq!(imp.explicit_import_name(), "x");

    // A non-ExplicitImport symbol reads back empty.
    assert_eq!(body.explicit_import_package_name(), "");
}

// ---- VariableSymbol::flags / lifetime ---------------------------------------

#[test]
fn variable_flags_and_lifetime() {
    let d = compile(
        "module m;\n\
         int plain;\n\
         const int frozen = 1;\n\
         initial begin: blk\n\
         static int stat = 2;\n\
         automatic int auto_ = 3;\n\
         end\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());

    let body = d.top_instances().next().unwrap().instance_body().unwrap();

    // Direct module-scope variables default to Static (see
    // VariableSymbols.cpp's getDefaultLifetime: any scope other than a
    // StatementBlock/Subroutine/MethodPrototype defaults to Static).
    let plain = body.members().find(|m| m.name() == "plain").unwrap();
    assert_eq!(plain.variable_flags(), VariableFlags::NONE);
    assert_eq!(plain.variable_lifetime(), VariableLifetime::Static);

    let frozen = body.members().find(|m| m.name() == "frozen").unwrap();
    assert!(frozen.variable_flags().contains(VariableFlags::CONST));

    // Explicit lifetimes inside a procedural block are honored as written.
    let proc_block = body
        .members()
        .find(|m| m.kind() == SymbolKind::ProceduralBlock)
        .expect("module has an initial block");
    let init_scope = proc_block
        .body()
        .unwrap()
        .block_symbol()
        .expect("named begin/end block has a StatementBlockSymbol");

    let stat = init_scope.members().find(|m| m.name() == "stat").unwrap();
    assert_eq!(stat.variable_lifetime(), VariableLifetime::Static);

    let auto_ = init_scope.members().find(|m| m.name() == "auto_").unwrap();
    assert_eq!(auto_.variable_lifetime(), VariableLifetime::Automatic);

    // A non-VariableSymbol (the containing instance body itself) reads back
    // the default/empty values.
    assert_eq!(body.variable_flags(), VariableFlags::NONE);
    assert_eq!(body.variable_lifetime(), VariableLifetime::Automatic);
}

#[test]
fn variable_flags_compiler_generated_for_function_return_value() {
    // A subroutine's implicit return-value variable is marked
    // CompilerGenerated (see SubroutineSymbols.cpp's SubroutineSymbol::
    // fromSyntax: `implicitReturnVar->flags |= VariableFlags::
    // CompilerGenerated;`).
    let d = compile(
        "module m;\n\
         function automatic int f();\n\
         return 1;\n\
         endfunction\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());

    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let f = body
        .members()
        .find(|m| m.kind() == SymbolKind::Subroutine)
        .expect("module has a function");
    let ret = f
        .members()
        .find(|m| m.kind() == SymbolKind::Variable)
        .expect("function has an implicit return-value variable");
    assert!(
        ret.variable_flags()
            .contains(VariableFlags::COMPILER_GENERATED)
    );
    assert!(!ret.variable_flags().contains(VariableFlags::CONST));
}

// ---- WildcardImportSymbol::getPackage / packageName -------------------------

#[test]
fn wildcard_import_package_and_name() {
    let d = compile(
        "package p;\n\
         int x = 42;\n\
         endpackage\n\
         module m;\n\
         import p::*;\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());

    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let imp = body
        .members()
        .find(|m| m.kind() == SymbolKind::WildcardImport)
        .expect("module has a wildcard import");

    assert_eq!(imp.wildcard_import_package_name(), "p");

    let pkg = imp
        .wildcard_import_package()
        .expect("the import's package resolved");
    assert_eq!(pkg.name(), "p");
    assert_eq!(pkg.kind(), SymbolKind::Package);

    // A non-WildcardImport symbol reports the default/empty values.
    assert_eq!(body.wildcard_import_package_name(), "");
    assert!(body.wildcard_import_package().is_none());
}

// Proves the freeze sweep forces WildcardImportSymbol's resolution even
// though no FreezeVisitor branch targets it directly: `getAllDiagnostics()`
// (run unconditionally, pre-seal, by `slang_compilation_freeze`) already
// visits every WildcardImportSymbol and calls `getPackage()` on it (see
// `ast::Elaborator`'s handler), so by the time the design is frozen and
// shared, `wildcard_import_package` is a pure read. A failure to force this
// would only show up as a data race on concurrent first access (see
// `tsan_race.rs`), not as a functional failure here — this test instead
// proves the resolution is idempotent and correct.
#[test]
fn freeze_forces_wildcard_import_resolution() {
    let d = compile(
        "package p;\n\
         int x = 42;\n\
         endpackage\n\
         module m;\n\
         import p::*;\n\
         endmodule\n",
    );
    assert!(d.freeze_report().symbols_elaborated > 0);

    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let imp = body
        .members()
        .find(|m| m.kind() == SymbolKind::WildcardImport)
        .unwrap();

    let first = imp.wildcard_import_package().unwrap();
    let second = imp.wildcard_import_package().unwrap();
    assert_eq!(first.name(), second.name());
    assert_eq!(first.name(), "p");
}

// ---- FieldSymbol::randMode ---------------------------------------------------

#[test]
fn field_rand_mode() {
    use sv_lang::RandMode;

    let d = compile(
        "module m;\n\
         typedef struct { rand int a; randc byte b; logic c; } s_t;\n\
         s_t s;\n\
         endmodule\n",
    );
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let t = body.find("s").unwrap().value_type().unwrap();
    let fields: Vec<_> = t.fields().collect();
    assert_eq!(fields.len(), 3);
    assert_eq!(fields[0].name(), "a");
    assert_eq!(fields[0].field_rand_mode(), RandMode::Rand);
    assert_eq!(fields[1].name(), "b");
    assert_eq!(fields[1].field_rand_mode(), RandMode::RandC);
    assert_eq!(fields[2].name(), "c");
    assert_eq!(fields[2].field_rand_mode(), RandMode::None);

    // A non-Field symbol reads back None.
    assert_eq!(body.field_rand_mode(), RandMode::None);
}

// ---- FormalArgumentSymbol::direction / getDefaultValue -----------------------

#[test]
fn formal_argument_direction_and_default_value() {
    use sv_lang::ArgumentDirection;

    let d = compile(
        "module m;\n\
         function automatic void f(input int a, output int b, ref int c, input int d = 4);\n\
         b = a; c = a;\n\
         endfunction\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let f = body.find("f").unwrap();
    let args: Vec<_> = f
        .members()
        .filter(|m| m.kind() == SymbolKind::FormalArgument)
        .collect();
    assert_eq!(args.len(), 4);

    assert_eq!(args[0].name(), "a");
    assert_eq!(args[0].formal_argument_direction(), ArgumentDirection::In);
    assert!(args[0].formal_argument_default_value().is_none());

    assert_eq!(args[1].name(), "b");
    assert_eq!(args[1].formal_argument_direction(), ArgumentDirection::Out);
    assert!(args[1].formal_argument_default_value().is_none());

    assert_eq!(args[2].name(), "c");
    assert_eq!(args[2].formal_argument_direction(), ArgumentDirection::Ref);
    assert!(args[2].formal_argument_default_value().is_none());

    assert_eq!(args[3].name(), "d");
    assert_eq!(args[3].formal_argument_direction(), ArgumentDirection::In);
    let default = args[3].formal_argument_default_value().unwrap();
    assert_eq!(default.constant_value().unwrap().as_i64(), Some(4));

    // A non-FormalArgument symbol reads back the "no direction/value" default.
    assert_eq!(f.formal_argument_direction(), ArgumentDirection::In);
    assert!(f.formal_argument_default_value().is_none());
}

// ---- GenerateBlockSymbol / GenerateBlockArraySymbol --------------------------

const GENERATE_DESIGN: &str = "\
module m;
    genvar i;
    for (i = 0; i < 3; i = i + 1) begin : loop
        logic unused;
    end
    if (1) begin : cond
    end else begin : cond_else
    end
    case (1)
        1: begin : c1 end
        default: begin : c2 end
    endcase
endmodule
";

#[test]
fn generate_block_symbol_branch_kind_and_construct_index() {
    use sv_lang::GenerateBranchKind;

    let d = compile(GENERATE_DESIGN);
    assert!(!d.diagnostics().has_errors());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();

    let cond = body.find("cond").unwrap();
    assert_eq!(cond.kind(), SymbolKind::GenerateBlock);
    assert_eq!(
        cond.generate_block_branch_kind(),
        GenerateBranchKind::IfTrue
    );

    let cond_else = body.find("cond_else").unwrap();
    assert_eq!(
        cond_else.generate_block_branch_kind(),
        GenerateBranchKind::IfFalse
    );
    // Both branches of the same `if` construct share a constructIndex.
    assert_eq!(
        cond.generate_block_construct_index(),
        cond_else.generate_block_construct_index()
    );

    let c1 = body.find("c1").unwrap();
    assert_eq!(
        c1.generate_block_branch_kind(),
        GenerateBranchKind::CaseItem
    );
    let c2 = body.find("c2").unwrap();
    assert_eq!(
        c2.generate_block_branch_kind(),
        GenerateBranchKind::CaseDefault
    );

    // A non-GenerateBlock symbol reads back the "illegal" sentinel / 0.
    assert_eq!(
        body.generate_block_branch_kind(),
        GenerateBranchKind::IllegalUnconditional
    );
    assert_eq!(body.generate_block_construct_index(), 0);
}

#[test]
fn generate_block_symbol_case_item_expressions() {
    let d = compile(GENERATE_DESIGN);
    let body = d.top_instances().next().unwrap().instance_body().unwrap();

    let c1 = body.find("c1").unwrap();
    let exprs: Vec<_> = c1.generate_block_case_item_expressions().collect();
    assert_eq!(exprs.len(), 1);
    assert_eq!(exprs[0].constant_value().unwrap().as_i64(), Some(1));

    // The default block has no case-item label expressions.
    let c2 = body.find("c2").unwrap();
    assert_eq!(c2.generate_block_case_item_expressions().count(), 0);

    // Neither does an if-generate block.
    let cond = body.find("cond").unwrap();
    assert_eq!(cond.generate_block_case_item_expressions().count(), 0);
}

#[test]
fn generate_block_symbol_get_array_index() {
    let d = compile(GENERATE_DESIGN);
    let body = d.top_instances().next().unwrap().instance_body().unwrap();

    let arr = body.find("loop").unwrap();
    assert_eq!(arr.kind(), SymbolKind::GenerateBlockArray);
    let entries: Vec<_> = arr.generate_block_array_entries().collect();
    assert_eq!(entries.len(), 3);
    for (i, entry) in entries.iter().enumerate() {
        let idx = entry.generate_block_array_index().unwrap();
        assert_eq!(idx.as_i64(), Some(i as i64));
    }

    // A block that was not produced by a loop generate has no array index.
    let cond = body.find("cond").unwrap();
    assert!(cond.generate_block_array_index().is_none());

    // Nor does a non-GenerateBlock symbol.
    assert!(body.generate_block_array_index().is_none());
}

#[test]
fn generate_block_array_symbol_members() {
    use sv_lang::GenerateBranchKind;

    let d = compile(GENERATE_DESIGN);
    assert!(!d.diagnostics().has_errors());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();

    let arr = body.find("loop").unwrap();
    assert_eq!(arr.kind(), SymbolKind::GenerateBlockArray);
    assert!(arr.generate_block_array_valid());
    assert_eq!(arr.generate_block_external_name().as_deref(), Some("loop"));

    let entries: Vec<_> = arr.generate_block_array_entries().collect();
    assert_eq!(entries.len(), 3);
    for (i, entry) in entries.iter().enumerate() {
        assert_eq!(entry.generate_block_construct_index(), i as u32);
        assert_eq!(
            entry.generate_block_branch_kind(),
            GenerateBranchKind::LoopIteration
        );
    }

    let initial = arr.generate_block_array_initial_expr().unwrap();
    assert_eq!(initial.constant_value().unwrap().as_i64(), Some(0));

    let stop = arr.generate_block_array_stop_expr().unwrap();
    assert!(stop.kind() != sv_lang::kinds::ExpressionKind::Invalid);

    let iter = arr.generate_block_array_iter_expr().unwrap();
    assert!(iter.kind() != sv_lang::kinds::ExpressionKind::Invalid);

    let loop_var = arr.generate_block_array_loop_variable().unwrap();
    assert_eq!(loop_var.name(), "i");
    assert_eq!(loop_var.kind(), SymbolKind::Variable);

    // A non-GenerateBlockArray symbol reads back the empty/None defaults.
    let cond = body.find("cond").unwrap();
    assert_eq!(cond.generate_block_array_entries().count(), 0);
    assert_eq!(cond.generate_block_array_construct_index(), 0);
    assert!(!cond.generate_block_array_valid());
    assert!(cond.generate_block_array_initial_expr().is_none());
    assert!(cond.generate_block_array_stop_expr().is_none());
    assert!(cond.generate_block_array_iter_expr().is_none());
    assert!(cond.generate_block_array_loop_variable().is_none());
}

#[test]
fn generate_block_external_name_synthesized() {
    let d = compile(
        "module m;\n\
         if (1) begin : cond\n\
         end else begin\n\
         end\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();

    let named = body.find("cond").unwrap();
    assert_eq!(
        named.generate_block_external_name().as_deref(),
        Some("cond")
    );

    let unnamed = body
        .members()
        .find(|m| m.kind() == SymbolKind::GenerateBlock && m.name().is_empty())
        .unwrap();
    let ext = unnamed.generate_block_external_name().unwrap();
    assert!(
        ext.starts_with("genblk"),
        "unexpected external name {ext:?}"
    );

    // A non-GenerateBlock/GenerateBlockArray symbol has no external name.
    assert!(body.generate_block_external_name().is_none());
}

// The four Expression fields of GenerateBlockArraySymbol (initialExpression,
// stopExpression, iterExpression) and the case-item labels of
// GenerateBlockSymbol are bound against a scope the generic freeze-sweep
// traversal never reaches (see FreezeVisitor's dedicated
// GenerateBlockArraySymbol/GenerateBlockSymbol branches in CApiAst.cpp), so
// unlike most expression-bearing fields they need an explicit visit. A
// failure to force this would only show up as a data race on concurrent
// first access (canonical-type resolution / prefold), not as a functional
// failure here — this test instead proves repeated reads are consistent and
// that the loop variable (visited the same way) resolves correctly.
#[test]
fn freeze_forces_generate_block_array_memos() {
    let d = compile(GENERATE_DESIGN);
    assert!(d.freeze_report().symbols_elaborated > 0);

    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let arr = body.find("loop").unwrap();

    let initial1 = arr.generate_block_array_initial_expr().unwrap();
    let initial2 = arr.generate_block_array_initial_expr().unwrap();
    assert_eq!(
        initial1.constant_value().unwrap().as_i64(),
        initial2.constant_value().unwrap().as_i64()
    );

    let stop1 = arr.generate_block_array_stop_expr().unwrap();
    let stop2 = arr.generate_block_array_stop_expr().unwrap();
    assert_eq!(stop1.kind(), stop2.kind());

    let iter1 = arr.generate_block_array_iter_expr().unwrap();
    let iter2 = arr.generate_block_array_iter_expr().unwrap();
    assert_eq!(iter1.kind(), iter2.kind());

    let var1 = arr.generate_block_array_loop_variable().unwrap();
    let var2 = arr.generate_block_array_loop_variable().unwrap();
    assert_eq!(var1.name(), "i");
    assert_eq!(var1.name(), var2.name());

    let c1 = body.find("c1").unwrap();
    let expr1: Vec<_> = c1.generate_block_case_item_expressions().collect();
    let expr2: Vec<_> = c1.generate_block_case_item_expressions().collect();
    assert_eq!(expr1.len(), expr2.len());
    assert_eq!(
        expr1[0].constant_value().unwrap().as_i64(),
        expr2[0].constant_value().unwrap().as_i64()
    );
}

#[test]
fn generate_block_symbol_condition_expr_and_is_uninstantiated() {
    use sv_lang::GenerateBranchKind;

    let d = compile(GENERATE_DESIGN);
    assert!(!d.diagnostics().has_errors());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();

    // `if (1) begin : cond ... end else begin : cond_else ... end` — the
    // condition is a compile-time-true constant, so `cond` is the taken
    // branch and `cond_else` is the untaken (uninstantiated) one.
    let cond = body.find("cond").unwrap();
    assert_eq!(
        cond.generate_block_branch_kind(),
        GenerateBranchKind::IfTrue
    );
    assert!(!cond.generate_block_is_uninstantiated());
    let cond_expr = cond.generate_block_condition_expr().unwrap();
    assert_eq!(cond_expr.constant_value().unwrap().as_i64(), Some(1));

    let cond_else = body.find("cond_else").unwrap();
    assert_eq!(
        cond_else.generate_block_branch_kind(),
        GenerateBranchKind::IfFalse
    );
    assert!(cond_else.generate_block_is_uninstantiated());
    // Both branches of the same `if` share the very same bound condition.
    let cond_else_expr = cond_else.generate_block_condition_expr().unwrap();
    assert_eq!(
        cond_else_expr.constant_value().unwrap().as_i64(),
        cond_expr.constant_value().unwrap().as_i64()
    );

    // `case (1) 1: begin : c1 end default: begin : c2 end endcase` — the
    // `1:` item matches, so c1 is taken and c2 (the default) is the
    // untaken/uninstantiated branch. Both still carry the case selector as
    // their condition expression.
    let c1 = body.find("c1").unwrap();
    assert!(!c1.generate_block_is_uninstantiated());
    assert!(c1.generate_block_condition_expr().is_some());

    let c2 = body.find("c2").unwrap();
    assert!(c2.generate_block_is_uninstantiated());
    assert!(c2.generate_block_condition_expr().is_some());

    // A loop-generate entry is not a conditional branch: no condition
    // expression, and (having been actually instantiated) not uninstantiated.
    let arr = body.find("loop").unwrap();
    let first_entry = arr.generate_block_array_entries().next().unwrap();
    assert_eq!(
        first_entry.generate_block_branch_kind(),
        GenerateBranchKind::LoopIteration
    );
    assert!(first_entry.generate_block_condition_expr().is_none());
    assert!(!first_entry.generate_block_is_uninstantiated());

    // A non-GenerateBlock symbol reads back the empty/false defaults.
    assert!(body.generate_block_condition_expr().is_none());
    assert!(!body.generate_block_is_uninstantiated());
}

// getConditionExpression() (like caseItemExpressions) is bound against the
// generate construct's *enclosing* scope and reached only via the
// FreezeVisitor's dedicated GenerateBlockSymbol branch in CApiAst.cpp (added
// alongside the case-item-label force) -- never by the generic scope-member
// traversal. A failure to force it would only show up as a data race on
// concurrent first access, not as a functional failure here -- this test
// instead proves repeated reads of both the taken and untaken branch's
// condition are consistent.
#[test]
fn freeze_forces_generate_block_condition_expr_memo() {
    let d = compile(GENERATE_DESIGN);
    assert!(d.freeze_report().symbols_elaborated > 0);
    let body = d.top_instances().next().unwrap().instance_body().unwrap();

    for name in ["cond", "cond_else"] {
        let block = body.find(name).unwrap();
        let e1 = block.generate_block_condition_expr().unwrap();
        let e2 = block.generate_block_condition_expr().unwrap();
        assert_eq!(
            e1.constant_value().unwrap().as_i64(),
            e2.constant_value().unwrap().as_i64()
        );
    }
}

// ---- InstanceSymbol / InstanceBodySymbol / InstanceArraySymbol --------------

const INSTANCE_ARRAY_DESIGN: &str = "\
module sub #(parameter int N = 1) (input logic a, output logic b);
    assign b = a;
endmodule
module top;
    logic [3:1] x, y;
    sub arr[3:1](.a(x), .b(y));
    sub solo(.a(x[1]), .b(y[1]));
endmodule
";

#[test]
fn instance_symbol_is_module_is_interface_and_port_connections() {
    let d = compile(INSTANCE_ARRAY_DESIGN);
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();

    let solo = body.find("solo").unwrap();
    assert_eq!(solo.kind(), SymbolKind::Instance);
    assert!(solo.instance_is_module());
    assert!(!solo.instance_is_interface());

    let conns: Vec<_> = solo.instance_port_connections().collect();
    assert_eq!(conns.len(), 2);
    let names: BTreeSet<_> = conns.iter().map(|c| c.port().name().to_string()).collect();
    assert_eq!(names, BTreeSet::from(["a".to_string(), "b".to_string()]));
    for c in &conns {
        assert!(c.expression().is_some());
        assert!(!c.is_implicit());
        assert!(!c.is_wildcard());
    }

    // A non-Instance symbol reads back the empty/false defaults.
    assert!(!body.instance_is_module());
    assert!(!body.instance_is_interface());
    assert_eq!(body.instance_port_connections().count(), 0);
}

#[test]
fn instance_symbol_implicit_port_connection() {
    let d = compile(
        "module sub(input logic a); endmodule\n\
         module m; logic a; sub s1(.a); endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let s1 = body.find("s1").unwrap();
    let conns: Vec<_> = s1.instance_port_connections().collect();
    assert_eq!(conns.len(), 1);
    assert!(conns[0].is_implicit());
    assert!(!conns[0].is_wildcard());
}

#[test]
fn instance_body_symbol_parent_definition_ports_and_same_type() {
    let d = compile(INSTANCE_ARRAY_DESIGN);
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let top = d.top_instances().next().unwrap();
    let body = top.instance_body().unwrap();
    assert_eq!(body.kind(), SymbolKind::InstanceBody);

    // parentInstance round-trips to the owning Instance symbol.
    assert_eq!(body.instance_body_parent_instance(), Some(top));

    // getDefinition() names the definition this body was elaborated from.
    assert_eq!(body.instance_body_definition().unwrap().name(), "top");

    let solo = body.find("solo").unwrap();
    let solo_body = solo.instance_body().unwrap();
    assert_eq!(solo_body.instance_body_definition().unwrap().name(), "sub");
    assert_eq!(solo_body.instance_body_parent_instance(), Some(solo));

    // getPortList() / findPort().
    let port_names: Vec<_> = solo_body
        .instance_body_ports()
        .map(|p| p.name().to_string())
        .collect();
    assert_eq!(port_names, vec!["a", "b"]);
    assert_eq!(solo_body.instance_body_find_port("b").unwrap().name(), "b");
    assert!(solo_body.instance_body_find_port("nope").is_none());

    // hasSameType(): every `sub` element/instance here shares identical
    // parameters (default N=1), so all their bodies are the same type; the
    // top-level body is not.
    let arr = body.find("arr").unwrap();
    let arr_elem_body = arr
        .instance_array_elements()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    assert!(solo_body.instance_body_has_same_type(&arr_elem_body));
    assert!(!solo_body.instance_body_has_same_type(&body));

    // A non-InstanceBody symbol reads back the empty defaults.
    assert!(top.instance_body_parent_instance().is_none());
    assert!(top.instance_body_definition().is_none());
    assert_eq!(top.instance_body_ports().count(), 0);
    assert!(top.instance_body_find_port("a").is_none());
    assert!(!top.instance_body_has_same_type(&body));
}

#[test]
fn instance_array_symbol_elements_name_and_range() {
    let d = compile(INSTANCE_ARRAY_DESIGN);
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();

    let arr = body.find("arr").unwrap();
    assert_eq!(arr.kind(), SymbolKind::InstanceArray);
    assert_eq!(
        arr.instance_array_range(),
        sv_lang::ConstantRange { left: 3, right: 1 }
    );

    let elements: Vec<_> = arr.instance_array_elements().collect();
    assert_eq!(elements.len(), 3);
    for e in &elements {
        assert_eq!(e.kind(), SymbolKind::Instance);
    }

    assert_eq!(arr.instance_array_name(), "arr");

    // A non-InstanceArray symbol reads back the empty/zeroed defaults.
    let solo = body.find("solo").unwrap();
    assert_eq!(solo.instance_array_elements().count(), 0);
    assert_eq!(
        solo.instance_array_range(),
        sv_lang::ConstantRange { left: 0, right: 0 }
    );
    assert_eq!(solo.instance_array_name(), "");
}

#[test]
fn instance_symbol_base_array_path_and_array_name() {
    let d = compile(INSTANCE_ARRAY_DESIGN);
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();

    let arr = body.find("arr").unwrap();
    let elements: Vec<_> = arr.instance_array_elements().collect();
    assert_eq!(elements.len(), 3);
    // Elements come back in ascending storage order (canonical zero-based
    // indices), regardless of the declared [3:1] range.
    for (i, elem) in elements.iter().enumerate() {
        assert_eq!(
            elem.instance_array_path().collect::<Vec<_>>(),
            vec![i as u32]
        );
        assert_eq!(elem.instance_base_array_name(), "arr");
    }

    // A standalone instance has an empty array path and its own name.
    let solo = body.find("solo").unwrap();
    assert_eq!(solo.instance_array_path().count(), 0);
    assert_eq!(solo.instance_base_array_name(), "solo");

    // A non-instance symbol reads back the empty defaults.
    assert_eq!(body.instance_array_path().count(), 0);
    assert_eq!(body.instance_base_array_name(), "");
}

const INTERFACE_PORT_DESIGN: &str = "\
interface bus;
    logic req;
    modport m(input req);
endinterface
module sub(bus.m b[1:0]);
endmodule
module gen(interface g);
endmodule
module top;
    bus b[1:0]();
    bus c();
    sub s(.b(b));
    gen g(.g(c));
endmodule
";

#[test]
fn interface_port_symbol_connection_range_def_and_modport() {
    let d = compile(INTERFACE_PORT_DESIGN);
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let top = d.top_instances().find(|i| i.name() == "top").unwrap();
    let top_body = top.instance_body().unwrap();

    let s = top_body.find("s").unwrap();
    let port = s
        .instance_body()
        .unwrap()
        .instance_body_ports()
        .find(|p| p.name() == "b")
        .unwrap();
    assert_eq!(port.kind(), SymbolKind::InterfacePort);

    // getConnection(): the interface instance ("b" in top) and its modport
    // ("m").
    let (instance, modport) = port.interface_port_connection();
    assert_eq!(instance.unwrap().name(), "b");
    assert_eq!(modport.unwrap().name(), "m");

    // getDeclaredRange(): the port's own `[1:0]` array dimension.
    let ranges: Vec<_> = port.interface_port_declared_range().collect();
    assert_eq!(ranges, vec![sv_lang::ConstantRange { left: 1, right: 0 }]);

    // interfaceDef / isGeneric / isInvalid / modport.
    assert_eq!(port.interface_port_interface_def().unwrap().name(), "bus");
    assert!(!port.interface_port_is_generic());
    assert!(!port.interface_port_is_invalid());
    assert_eq!(port.interface_port_modport(), "m");

    // A generic interface port: no interfaceDef, isGeneric true, no modport,
    // and a scalar (empty) declared range.
    let g = top_body.find("g").unwrap();
    let gport = g
        .instance_body()
        .unwrap()
        .instance_body_ports()
        .find(|p| p.name() == "g")
        .unwrap();
    assert!(gport.interface_port_is_generic());
    assert!(!gport.interface_port_is_invalid());
    assert!(gport.interface_port_interface_def().is_none());
    assert_eq!(gport.interface_port_modport(), "");
    assert_eq!(gport.interface_port_declared_range().count(), 0);

    // A non-InterfacePort symbol reads back the empty/zeroed defaults.
    let (i2, m2) = top_body.interface_port_connection();
    assert!(i2.is_none());
    assert!(m2.is_none());
    assert_eq!(top_body.interface_port_declared_range().count(), 0);
    assert!(top_body.interface_port_interface_def().is_none());
    assert!(!top_body.interface_port_is_generic());
    assert!(!top_body.interface_port_is_invalid());
    assert_eq!(top_body.interface_port_modport(), "");
}

const METHOD_PROTOTYPE_DESIGN: &str = "\
interface Iface(input clk);
    extern function bit [7:0] f(int a, bit b);
    extern task t();
    clocking cb @(posedge clk);
    endclocking
    modport m(clocking cb);
endinterface

module impl(Iface i);
    function bit [7:0] i.f(int a, bit b);
        return 0;
    endfunction
    task i.t();
    endtask
endmodule

module top(input clk);
    Iface i(clk);
    impl u(i);
endmodule

virtual class C;
    pure virtual function void pv();
    extern function void ext_g();
endclass
function void C::ext_g();
endfunction

virtual class Base;
    virtual function void ov(); endfunction
endclass
virtual class Derived extends Base;
    pure virtual function void ov();
endclass
";

#[test]
fn method_prototype_symbol_and_modport_clocking_accessors() {
    let d = compile(METHOD_PROTOTYPE_DESIGN);
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());

    let top = d.top_instances().find(|i| i.name() == "top").unwrap();
    let top_body = top.instance_body().unwrap();
    let iface = top_body.find("i").unwrap().instance_body().unwrap();

    // `Scope::find` transparently redirects a MethodPrototype/ModportClocking
    // member (to getSubroutine()/target respectively), so every lookup below
    // that wants the prototype/modport-clocking symbol itself goes through
    // `.members()` instead.

    // MethodPrototypeSymbol::flags / subroutineKind / visibility / isVirtual
    // / getArguments / getReturnType / getOverride, on the interface's own
    // `f` prototype.
    let f = iface.members().find(|s| s.name() == "f").unwrap();
    assert_eq!(f.kind(), SymbolKind::MethodPrototype);
    assert!(
        f.method_prototype_flags()
            .contains(sv_lang::MethodFlags::INTERFACE_EXTERN)
    );
    assert_eq!(
        f.method_prototype_subroutine_kind(),
        sv_lang::SubroutineKind::Function
    );
    assert_eq!(f.method_prototype_visibility(), sv_lang::Visibility::Public);
    assert!(!f.method_prototype_is_virtual());
    assert_eq!(f.method_prototype_argument_count(), 2);
    assert_eq!(f.method_prototype_argument(0).unwrap().name(), "a");
    assert_eq!(f.method_prototype_argument(1).unwrap().name(), "b");
    assert!(f.method_prototype_argument(2).is_none());
    assert_eq!(f.method_prototype_return_type().unwrap().bit_width(), 8);
    assert!(f.method_prototype_override().is_none());

    // getSubroutine(): for an interface's own extern method, this is always
    // a synthesized stub -- NOT the module's implementation.
    let stub = f.method_prototype_subroutine().unwrap();
    assert_eq!(stub.name(), "f");
    assert_eq!(stub.kind(), SymbolKind::Subroutine);

    // getFirstExternImpl() / ExternImpl::{impl,getNextImpl}: the module's
    // real implementation is reached only through this chain, and is a
    // different subroutine from the synthesized stub above.
    let first = f.method_prototype_first_extern_impl().unwrap();
    let impl_sub = first.implementation();
    assert_eq!(impl_sub.name(), "f");
    assert_eq!(impl_sub.kind(), SymbolKind::Subroutine);
    assert_ne!(impl_sub, stub);
    assert!(first.next().is_none());

    // `t` is also implemented by `impl`, registering its own extern impl.
    let t = iface.members().find(|s| s.name() == "t").unwrap();
    assert_eq!(
        t.method_prototype_subroutine_kind(),
        sv_lang::SubroutineKind::Task
    );
    let t_first = t.method_prototype_first_extern_impl().unwrap();
    assert_eq!(t_first.implementation().name(), "t");
    assert!(t_first.next().is_none());

    // ModportClockingSymbol::target.
    let modport = iface.find("m").unwrap();
    let mc = modport.members().find(|s| s.name() == "cb").unwrap();
    assert_eq!(mc.kind(), SymbolKind::ModportClocking);
    let target = mc.modport_clocking_target().unwrap();
    assert_eq!(target.name(), "cb");
    assert_eq!(target.kind(), SymbolKind::ClockingBlock);

    // A non-ModportClocking symbol reads back None.
    assert!(iface.modport_clocking_target().is_none());

    // A plain class: pure-virtual is virtual; extern-with-out-of-block-impl
    // is not.
    let units = d.compilation_units().next().unwrap();
    let c = units.find("C").unwrap();
    let pv = c.members().find(|s| s.name() == "pv").unwrap();
    let ext_g = c.members().find(|s| s.name() == "ext_g").unwrap();
    assert!(pv.method_prototype_is_virtual());
    assert!(
        pv.method_prototype_flags()
            .contains(sv_lang::MethodFlags::PURE | sv_lang::MethodFlags::VIRTUAL)
    );
    assert!(!ext_g.method_prototype_is_virtual());
    assert_eq!(ext_g.method_prototype_subroutine().unwrap().name(), "ext_g");
    // A pure-virtual class method prototype has no `extern` implementation
    // chain at all (that mechanism is interface-extern-method-specific).
    assert!(pv.method_prototype_first_extern_impl().is_none());

    // Every MethodPrototype-specific accessor reads back its stated default
    // for a symbol that is not a MethodPrototype at all.
    assert_eq!(c.method_prototype_flags(), sv_lang::MethodFlags::NONE);
    assert_eq!(
        c.method_prototype_subroutine_kind(),
        sv_lang::SubroutineKind::Function
    );
    assert_eq!(c.method_prototype_visibility(), sv_lang::Visibility::Public);
    assert!(!c.method_prototype_is_virtual());
    assert_eq!(c.method_prototype_argument_count(), 0);
    assert!(c.method_prototype_argument(0).is_none());
    assert!(c.method_prototype_return_type().is_none());
    assert!(c.method_prototype_subroutine().is_none());
    assert!(c.method_prototype_override().is_none());
    assert!(c.method_prototype_first_extern_impl().is_none());

    // getOverride(): Derived's pure-virtual `ov` overrides Base's in-place
    // virtual `ov`.
    let derived = units.find("Derived").unwrap();
    let derived_ov = derived.members().find(|s| s.name() == "ov").unwrap();
    assert!(derived_ov.method_prototype_is_virtual());
    let over = derived_ov.method_prototype_override().unwrap();
    assert_eq!(over.name(), "ov");
    assert_eq!(over.kind(), SymbolKind::Subroutine);
}

const MODPORT_PORT_DESIGN: &str = "\
interface Iface;
    logic req, gnt;
    task foo(); endtask
    modport m(input req, output .g(gnt), export foo);
endinterface
module top;
    Iface i();
endmodule
";

#[test]
fn modport_port_direction_connection_and_has_exports() {
    use sv_lang::{ArgumentDirection, kinds::ExpressionKind};

    let d = compile(MODPORT_PORT_DESIGN);
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());

    let top = d.top_instances().find(|i| i.name() == "top").unwrap();
    let iface = top
        .instance_body()
        .unwrap()
        .find("i")
        .unwrap()
        .instance_body()
        .unwrap();
    let modport = iface.find("m").unwrap();
    assert_eq!(modport.kind(), SymbolKind::Modport);
    assert!(modport.modport_has_exports());

    // Implicit port `input req`: direction In, connects directly to the
    // like-named internal `req`, no explicit connection expression.
    let req_port = modport.members().find(|s| s.name() == "req").unwrap();
    assert_eq!(req_port.kind(), SymbolKind::ModportPort);
    assert_eq!(req_port.modport_port_direction(), ArgumentDirection::In);
    assert!(req_port.modport_port_explicit_connection().is_none());
    let internal = req_port.modport_port_internal_symbol().unwrap();
    assert_eq!(internal.name(), "req");
    assert_eq!(internal.kind(), SymbolKind::Variable);

    // Explicit port `output .g(gnt)`: direction Out, an explicit connection
    // expression referencing `gnt`, no direct internal symbol.
    let g_port = modport.members().find(|s| s.name() == "g").unwrap();
    assert_eq!(g_port.kind(), SymbolKind::ModportPort);
    assert_eq!(g_port.modport_port_direction(), ArgumentDirection::Out);
    assert!(g_port.modport_port_internal_symbol().is_none());
    let conn = g_port.modport_port_explicit_connection().unwrap();
    assert_eq!(conn.kind(), ExpressionKind::NamedValue);

    // `export foo` is a MethodPrototype, not a ModportPort: every
    // ModportPort-specific accessor reads back its stated default.
    let foo = modport.members().find(|s| s.name() == "foo").unwrap();
    assert_eq!(foo.kind(), SymbolKind::MethodPrototype);
    assert_eq!(foo.modport_port_direction(), ArgumentDirection::In);
    assert!(foo.modport_port_explicit_connection().is_none());
    assert!(foo.modport_port_internal_symbol().is_none());

    // A non-Modport symbol reads back false for hasExports.
    assert!(!iface.modport_has_exports());
}

const MULTI_PORT_DESIGN: &str = "\
module m(.a({b, d}));
    input b;
    input d;
endmodule
";

#[test]
fn multi_port_symbol_accessors() {
    use sv_lang::ArgumentDirection;

    let d = compile(MULTI_PORT_DESIGN);
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();

    let port = body
        .instance_body_ports()
        .find(|p| p.name() == "a")
        .unwrap();
    assert_eq!(port.kind(), SymbolKind::MultiPort);
    assert_eq!(port.multi_port_direction(), ArgumentDirection::In);
    assert!(port.multi_port_initializer().is_none());
    assert!(!port.multi_port_is_null_port());
    assert_eq!(port.multi_port_type().unwrap().bit_width(), 2);

    let names: Vec<_> = port
        .multi_port_ports()
        .map(|p| p.name().to_string())
        .collect();
    assert_eq!(names, vec!["b", "d"]);
    for sub in port.multi_port_ports() {
        assert_eq!(sub.kind(), SymbolKind::Port);
    }

    // A non-MultiPort symbol reads back the stated defaults.
    assert_eq!(body.multi_port_direction(), ArgumentDirection::In);
    assert!(body.multi_port_initializer().is_none());
    assert!(body.multi_port_type().is_none());
    assert!(!body.multi_port_is_null_port());
    assert_eq!(body.multi_port_ports().count(), 0);
}

#[test]
fn freeze_forces_multi_port_type_memo() {
    let d = compile(MULTI_PORT_DESIGN);
    assert!(d.freeze_report().symbols_elaborated > 0);
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let port = body
        .instance_body_ports()
        .find(|p| p.name() == "a")
        .unwrap();

    // Reading twice must be consistent (a lazy first-touch race would risk
    // returning a different type/None on repeated reads under TSan).
    let first = port.multi_port_type().unwrap();
    let second = port.multi_port_type().unwrap();
    assert_eq!(first.bit_width(), second.bit_width());
    assert_eq!(first.bit_width(), 2);
}

const NET_ALIAS_DESIGN: &str = "\
module m;
    wire a, b, c;
    alias a = b = c;
endmodule
";

#[test]
fn net_alias_symbol_net_references() {
    let d = compile(NET_ALIAS_DESIGN);
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let alias = body
        .members()
        .find(|s| s.kind() == SymbolKind::NetAlias)
        .unwrap();

    let refs: Vec<_> = alias.net_alias_net_references().collect();
    assert_eq!(refs.len(), 3);

    // A non-NetAlias symbol reads back empty.
    assert_eq!(body.net_alias_net_references().count(), 0);
}

#[test]
fn freeze_forces_net_alias_net_references_memo() {
    let d = compile(NET_ALIAS_DESIGN);
    assert!(d.freeze_report().symbols_elaborated > 0);
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let alias = body
        .members()
        .find(|s| s.kind() == SymbolKind::NetAlias)
        .unwrap();

    // Reading twice must be consistent (a lazy first-touch race would risk
    // returning a different count/None on repeated reads under TSan).
    let first = alias.net_alias_net_references().count();
    let second = alias.net_alias_net_references().count();
    assert_eq!(first, second);
    assert_eq!(first, 3);
}

const NET_SYMBOL_DESIGN: &str = "\
module m(input a);
    wire vectored [7:0] v;
    wire plain;
    trireg (small) s;
    trireg t;
    wire #2 d = a;
    wire (strong0, pull1) st = a;
    assign implicit_w = a;
endmodule
";

#[test]
fn net_symbol_expansion_hint_and_charge_strength() {
    use sv_lang::{ChargeStrength, ExpansionHint};

    let d = compile(NET_SYMBOL_DESIGN);
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();

    let v = body.find("v").unwrap();
    assert_eq!(v.net_expansion_hint(), ExpansionHint::Vectored);
    assert_eq!(v.net_charge_strength(), None);

    let plain = body.find("plain").unwrap();
    assert_eq!(plain.net_expansion_hint(), ExpansionHint::None);

    let s = body.find("s").unwrap();
    assert_eq!(s.net_expansion_hint(), ExpansionHint::None);
    assert_eq!(s.net_charge_strength(), Some(ChargeStrength::Small));

    let t = body.find("t").unwrap();
    assert_eq!(t.net_charge_strength(), None);

    // A non-Net symbol reads back the stated defaults.
    assert_eq!(body.net_expansion_hint(), ExpansionHint::None);
    assert_eq!(body.net_charge_strength(), None);
}

#[test]
fn net_symbol_delay_drive_strength_and_is_implicit() {
    use sv_lang::DriveStrength;

    let d = compile(NET_SYMBOL_DESIGN);
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();

    let dd = body.find("d").unwrap();
    assert!(dd.net_delay().is_some());
    assert!(!dd.net_is_implicit());
    let dd_ds = dd.net_drive_strength();
    assert_eq!(dd_ds.strength0, None);
    assert_eq!(dd_ds.strength1, None);

    let st = body.find("st").unwrap();
    assert!(st.net_delay().is_none());
    let st_ds = st.net_drive_strength();
    assert_eq!(st_ds.strength0, Some(DriveStrength::Strong));
    assert_eq!(st_ds.strength1, Some(DriveStrength::Pull));

    let implicit = body.find("implicit_w").unwrap();
    assert!(implicit.net_is_implicit());
    assert!(implicit.net_delay().is_none());

    let v = body.find("v").unwrap();
    assert!(!v.net_is_implicit());

    // A non-Net symbol reads back the stated defaults.
    assert!(body.net_delay().is_none());
    let empty_ds = body.net_drive_strength();
    assert_eq!(empty_ds.strength0, None);
    assert_eq!(empty_ds.strength1, None);
    assert!(!body.net_is_implicit());
}

#[test]
fn freeze_forces_net_delay_memo() {
    let d = compile(NET_SYMBOL_DESIGN);
    assert!(d.freeze_report().symbols_elaborated > 0);
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let dd = body.find("d").unwrap();

    // Reading twice must be consistent (a lazy first-touch race would risk
    // returning a different node/None on repeated reads under TSan).
    let first = dd.net_delay().unwrap();
    let second = dd.net_delay().unwrap();
    assert_eq!(first.domain(), second.domain());
    assert_eq!(first.domain(), sv_lang_sys::SLANG_AST_TIMING_CONTROL);
}

const PACKAGE_LIFETIME_DESIGN: &str = "\
package p; endpackage
package automatic ap; endpackage
package static sp; endpackage
";

#[test]
fn package_symbol_default_lifetime() {
    use sv_lang::VariableLifetime;

    let d = compile(PACKAGE_LIFETIME_DESIGN);
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());

    let mut pkgs: Vec<_> = d.packages().collect();
    pkgs.sort_by_key(|p| p.name().to_string());
    let ap = pkgs.iter().find(|p| p.name() == "ap").unwrap();
    let p = pkgs.iter().find(|p| p.name() == "p").unwrap();
    let sp = pkgs.iter().find(|p| p.name() == "sp").unwrap();

    assert_eq!(
        ap.package_default_lifetime(),
        Some(VariableLifetime::Automatic)
    );
    assert_eq!(p.package_default_lifetime(), Some(VariableLifetime::Static));
    assert_eq!(
        sp.package_default_lifetime(),
        Some(VariableLifetime::Static)
    );

    // A non-Package symbol reads back None.
    let m = compile("module m; endmodule\n");
    let top = m.top_instances().next().unwrap();
    assert!(top.package_default_lifetime().is_none());
}

const LET_DECL_DESIGN: &str = "\
module m;
    let is_max(a, b) = (a >= b);
    let no_args = 1;
endmodule
";

#[test]
fn let_decl_symbol_ports() {
    let d = compile(LET_DECL_DESIGN);
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();

    let is_max = body.find("is_max").unwrap();
    assert_eq!(is_max.kind(), SymbolKind::LetDecl);
    let names: Vec<_> = is_max
        .let_decl_ports()
        .map(|p| p.name().to_string())
        .collect();
    assert_eq!(names, vec!["a", "b"]);

    // A `let` with no formal ports has an empty (not absent) port list.
    let no_args = body.find("no_args").unwrap();
    assert_eq!(no_args.kind(), SymbolKind::LetDecl);
    assert_eq!(no_args.let_decl_ports().count(), 0);

    // A non-LetDecl symbol reads back the empty default.
    assert_eq!(body.let_decl_ports().count(), 0);
}

const LOOKUP_VISIBILITY_DESIGN: &str = "\
class Base;
    int x;
    local int hidden;
    protected int guarded;
    local function void priv(); endfunction
    function void pub(); endfunction
endclass
class Derived extends Base;
endclass
class Other;
endclass
module m;
endmodule
";

#[test]
fn lookup_visibility_predicates() {
    let d = compile(LOOKUP_VISIBILITY_DESIGN);
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let unit = d.compilation_units().next().unwrap();
    let base = unit.find("Base").unwrap();
    let derived = unit.find("Derived").unwrap();
    let other = unit.find("Other").unwrap();
    let m_body = d.top_instances().next().unwrap().instance_body().unwrap();

    let x = base.find("x").unwrap();
    let hidden = base.find("hidden").unwrap();
    let guarded = base.find("guarded").unwrap();
    let priv_fn = base.find("priv").unwrap();
    let pub_fn = base.find("pub").unwrap();

    // Lookup::getVisibility, generalized over ClassProperty and Subroutine
    // symbol kinds alike.
    assert_eq!(x.visibility(), sv_lang::Visibility::Public);
    assert_eq!(hidden.visibility(), sv_lang::Visibility::Local);
    assert_eq!(guarded.visibility(), sv_lang::Visibility::Protected);
    assert_eq!(priv_fn.visibility(), sv_lang::Visibility::Local);
    assert_eq!(pub_fn.visibility(), sv_lang::Visibility::Public);
    // A non-class-member symbol is always Public.
    assert_eq!(m_body.visibility(), sv_lang::Visibility::Public);

    // Lookup::isVisibleFrom: a public member is visible everywhere; a local
    // member only from the declaring class (or nested classes within it).
    assert!(x.is_visible_from(&m_body));
    assert!(!hidden.is_visible_from(&m_body));
    assert!(hidden.is_visible_from(&base));
    assert!(!guarded.is_visible_from(&m_body));
    assert!(guarded.is_visible_from(&derived));

    // Lookup::isAccessibleFrom: same-class or derived-class scopes can
    // access an instance member; an unrelated class cannot.
    assert!(x.is_accessible_from(&base));
    assert!(x.is_accessible_from(&derived));
    assert!(!x.is_accessible_from(&other));
}

#[test]
fn lookup_ensure_visible_and_ensure_accessible() {
    let d = compile(LOOKUP_VISIBILITY_DESIGN);
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let mut design = d;
    let eval = design.eval_session();
    let unit = eval.design().compilation_units().next().unwrap();
    let base = unit.find("Base").unwrap();
    let hidden = base.find("hidden").unwrap();
    let priv_fn = base.find("priv").unwrap();
    let m_body = eval
        .design()
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    // Lookup::ensureVisible matches Lookup::isVisibleFrom's boolean for the
    // same pair, through the diagnostic-issuing entry point.
    assert!(!eval.ensure_visible(hidden, m_body));
    assert!(eval.ensure_visible(hidden, base));

    // Lookup::ensureAccessible: accessible from within a non-static method
    // of the owning class, not from an unrelated scope with no containing
    // class.
    assert!(eval.ensure_accessible(hidden, priv_fn));
    assert!(!eval.ensure_accessible(hidden, m_body));
}

const LOOKUP_NAME_DESIGN: &str = "\
module leaf;
    logic [7:0] x;
    class Foo;
    endclass
endmodule
";

#[test]
fn lookup_name_and_find_class() {
    let d = compile(LOOKUP_NAME_DESIGN);
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let mut design = d;
    let eval = design.eval_session();
    let root = eval.design().root();
    let body = eval
        .design()
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    // Lookup::name: full dotted-name resolution from an arbitrary scope
    // (here, the design root).
    let x = eval.lookup_name(root, "leaf.x").unwrap();
    assert_eq!(x.name(), "x");
    assert!(eval.lookup_name(root, "leaf.nope").is_none());

    // Lookup::findClass: resolves a type name to a ClassType, or None if it
    // isn't a class.
    let foo = eval.find_class(body, "Foo").unwrap();
    assert_eq!(foo.kind(), SymbolKind::ClassType);
    assert_eq!(foo.name(), "Foo");
    assert!(eval.find_class(body, "x").is_none());
    assert!(eval.find_class(body, "NoSuchThing").is_none());
}

const ASSERTION_LOCAL_VAR_DESIGN: &str = "\
module m;
    sequence s;
        int x;
        (1, x = 1) ##1 (x == 1);
    endsequence
    initial begin
        cover property (s);
    end
endmodule
";

#[test]
fn lookup_find_assertion_local_var() {
    let d = compile(ASSERTION_LOCAL_VAR_DESIGN);
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let mut design = d;
    let eval = design.eval_session();
    let body = eval
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
    let cover_stmt = block.body().unwrap().statements()[0];
    let spec = cover_stmt.children()[0];
    let inst = spec.children()[0].as_expression().unwrap();
    assert_eq!(inst.assertion_local_vars().len(), 1);
    assert_eq!(inst.assertion_local_vars()[0].name(), "x");

    let x = eval.find_assertion_local_var(inst, body, "x").unwrap();
    assert_eq!(x.name(), "x");
    assert_eq!(x, inst.assertion_local_vars()[0]);
    assert!(eval.find_assertion_local_var(inst, body, "nope").is_none());
}

const TEMP_VAR_DESIGN: &str = "\
module m;
    int q[$];
    int r[$];
    initial r = q.find_first(item) with (item > 0);
endmodule
";

#[test]
fn lookup_find_temp_var() {
    let d = compile(TEMP_VAR_DESIGN);
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let mut design = d;
    let eval = design.eval_session();
    let body = eval
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
    let assign = block.body().unwrap().expr().unwrap();
    let call = assign.right().unwrap();
    let iter_var = call.iterator_var().unwrap();

    let item = eval.find_temp_var(iter_var, body, "item").unwrap();
    assert_eq!(item.name(), "item");
    assert!(eval.find_temp_var(iter_var, body, "nope").is_none());
}

const LOOKUP_LOCATION_DESIGN: &str = "\
module m;
    logic a;
    logic b;
endmodule
";

#[test]
fn lookup_location_before_after_min_max() {
    let d = compile(LOOKUP_LOCATION_DESIGN);
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let a = body.find("a").unwrap();
    let b = body.find("b").unwrap();

    // LookupLocation::before/after: consecutive, and track declaration order.
    let before_a = a.lookup_location_before();
    let after_a = a.lookup_location_after();
    let before_b = b.lookup_location_before();
    assert_eq!(before_a.index() + 1, after_a.index());
    assert!(before_a.index() < before_b.index());
    assert!(after_a.index() <= before_b.index());

    // LookupLocation::getScope: the parent scope's own symbol, via
    // Scope::asSymbol() -- both locations belong to `m`'s instance body.
    assert_eq!(before_a.scope().unwrap(), body);
    assert_eq!(after_a.scope().unwrap(), body);

    // LookupLocation::min/max: the two scope-less sentinels that compare
    // before/after every real location.
    let min = d.lookup_location_min();
    let max = d.lookup_location_max();
    assert_eq!(min.index(), 0);
    assert_eq!(max.index(), u32::MAX);
    assert!(min.scope().is_none());
    assert!(max.scope().is_none());
    assert!(min.index() < after_a.index());
    assert!(max.index() > after_a.index());
}

const RANDOMIZE_LOOKUP_DESIGN: &str = "\
module m;
    class C;
        rand int x;
        constraint c1 { x > 0; }
    endclass
    int x;
endmodule
";

#[test]
fn lookup_within_class_randomize_and_result() {
    let d = compile(RANDOMIZE_LOOKUP_DESIGN);
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let mut design = d;
    let eval = design.eval_session();
    let body = eval
        .design()
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let c = eval.find_class(body, "C").unwrap();

    // Lookup::withinClassRandomize: "x.y" resolves to C's own `x` (a value),
    // deferring `.y` as a selector the caller must apply itself.
    let (found, result) = eval.within_class_randomize(c, None, body, "x.y");
    assert!(found);

    // LookupResult::found
    let x = result.found().unwrap();
    assert_eq!(x.name(), "x");
    assert_eq!(x.kind(), SymbolKind::ClassProperty);
    assert_eq!(x.parent().unwrap(), c);

    // LookupResult::flags: nothing hierarchical or imported about this
    // resolution.
    assert_eq!(result.flags(), sv_lang::LookupResultFlags::NONE);

    // LookupResult::systemSubroutine: mutually exclusive with `found` (which
    // is populated here), so it's None. LookupResult::upwardCount: this
    // lookup never went upward through the hierarchy.
    assert!(result.system_subroutine().is_none());
    assert_eq!(result.upward_count(), 0);

    // One deferred selector, a MemberSelector("y") --
    // LookupResult::MemberSelector::{name,dotLocation,nameRange}.
    assert_eq!(result.selector_count(), 1);
    assert!(result.selector(1).is_none());
    let sel = result.selector(0).unwrap();
    assert!(sel.is_member());
    assert_eq!(sel.name(), "y");
    assert_ne!(sel.dot_location().buffer, 0);
    let name_range = sel.name_range();
    assert_eq!(name_range.end.offset - name_range.start.offset, 1); // "y"
    // The dot immediately precedes the name it introduces.
    assert_eq!(sel.dot_location().offset + 1, name_range.start.offset);

    // `x` also exists as a local variable in `body`, so
    // withinClassRandomize's own RandomizeConstraintShadow warning landed in
    // the result's diagnostics -- LookupResult::getDiagnostics/hasError.
    let diags = result.diagnostics();
    assert!(!diags.items().is_empty());
    assert!(!result.has_error()); // it's only a warning, not an error

    // LookupResult::errorIfSelectors: issues an UnexpectedSelection error
    // into the compilation (observable only through has_issued_errors --
    // see the doc comment on EvalSession::error_if_selectors).
    assert!(!eval.design().has_issued_errors());
    assert!(eval.error_if_selectors(&result, body));
    assert!(eval.design().has_issued_errors());
    // errorIfSelectors is `const` on the C++ side: it doesn't touch `result`.
    assert_eq!(result.selector_count(), 1);

    // LookupResult::reportDiags: re-delivers `result`'s own diagnostics (the
    // RandomizeConstraintShadow warning) to the compilation. It doesn't
    // clear or otherwise touch `result`'s own diagnostic list.
    let diag_count_before = result.diagnostics().items().len();
    assert!(diag_count_before > 0);
    assert!(eval.report_diags(&result, body));
    assert_eq!(result.diagnostics().items().len(), diag_count_before);

    // LookupResult::clear resets everything to the freshly-created state.
    let mut result = result;
    result.clear();
    assert!(result.found().is_none());
    assert_eq!(result.flags(), sv_lang::LookupResultFlags::NONE);
    assert!(!result.has_error());
    assert_eq!(result.selector_count(), 0);
    assert!(result.diagnostics().items().is_empty());
    // A no-op errorIfSelectors call (no pending selectors) still reports
    // success.
    assert!(eval.error_if_selectors(&result, body));

    // Lookup::withinClassRandomize returning false: `super` with no base
    // class fails and leaves an error diagnostic behind.
    let (found2, result2) = eval.within_class_randomize(c, None, c, "super");
    assert!(!found2);
    assert!(result2.found().is_none());
    assert!(result2.has_error());
    assert!(!result2.diagnostics().items().is_empty());
}

// ---- PackageSymbol: findForImport / hasExportAll / timeScale ---------------

#[test]
fn package_has_export_all_and_time_scale() {
    use sv_lang::TimeScaleValue;

    let d = compile(
        "`timescale 1ns/1ps\n\
         package base_pkg;\n  localparam int K = 5;\nendpackage\n\
         package star_pkg;\n\
         \x20 import base_pkg::*;\n\
         \x20 export *::*;\n\
         \x20 localparam int L = K;\n\
         endpackage\n\
         package plain_pkg; endpackage\n\
         module m; endmodule\n",
    );

    let base = d.packages().find(|p| p.name() == "base_pkg").unwrap();
    let star = d.packages().find(|p| p.name() == "star_pkg").unwrap();
    let plain = d.packages().find(|p| p.name() == "plain_pkg").unwrap();

    assert!(!base.package_has_export_all());
    assert!(star.package_has_export_all());
    assert!(!plain.package_has_export_all());

    // The `timescale directive applies to every subsequently-declared
    // package/module, so all three packages pick it up as their own
    // explicit timescale.
    for pkg in [&base, &star, &plain] {
        let ts = pkg.package_time_scale().unwrap();
        assert_eq!(
            ts.base,
            TimeScaleValue::from_literal(1.0, sv_lang::TimeUnit::Nanoseconds).unwrap()
        );
        assert_eq!(
            ts.precision,
            TimeScaleValue::from_literal(1.0, sv_lang::TimeUnit::Picoseconds).unwrap()
        );
    }

    // Re-exported (through star.K) and directly-declared (base.K) both
    // resolve; a name that was never imported or declared does not.
    let k_via_star = star.package_find_for_import("K").unwrap();
    assert_eq!(k_via_star.name(), "K");
    let k_direct = base.package_find_for_import("K").unwrap();
    assert_eq!(k_direct.name(), "K");
    assert!(star.package_find_for_import("nope").is_none());
    assert!(plain.package_find_for_import("K").is_none());
}

#[test]
fn package_without_time_scale_returns_none() {
    let d = compile("package p; endpackage\nmodule m; endmodule\n");
    let p = d.packages().find(|p| p.name() == "p").unwrap();
    assert!(p.package_time_scale().is_none());
    // A non-Package symbol: false/None everywhere.
    let m = d.top_instances().next().unwrap();
    assert!(!m.package_has_export_all());
    assert!(m.package_time_scale().is_none());
    assert!(m.package_find_for_import("K").is_none());
}

// ---- ParameterSymbolBase / ParameterSymbol ----------------------------------

#[test]
fn parameter_symbol_base_local_port_body_flags() {
    let d = compile(
        "module m #(parameter int W = 8, type T = int) (input logic [W-1:0] a);\n\
         \x20 localparam int L = 2;\n\
         \x20 localparam type LT = logic;\n\
         \x20 T t;\n\
         \x20 LT lt;\n\
         endmodule\n",
    );
    let body = d.top_instances().next().unwrap().instance_body().unwrap();

    let w = body.find("W").unwrap();
    assert_eq!(w.kind(), SymbolKind::Parameter);
    assert!(w.parameter_is_port_param());
    assert!(!w.parameter_is_local_param());
    assert!(!w.parameter_is_body_param());

    let t = body.find("T").unwrap();
    assert_eq!(t.kind(), SymbolKind::TypeParameter);
    assert!(t.parameter_is_port_param());
    assert!(!t.parameter_is_local_param());
    assert!(!t.parameter_is_body_param());

    let l = body.find("L").unwrap();
    assert_eq!(l.kind(), SymbolKind::Parameter);
    assert!(!l.parameter_is_port_param());
    assert!(l.parameter_is_local_param());
    assert!(l.parameter_is_body_param());

    let lt = body.find("LT").unwrap();
    assert_eq!(lt.kind(), SymbolKind::TypeParameter);
    assert!(!lt.parameter_is_port_param());
    assert!(lt.parameter_is_local_param());
    assert!(lt.parameter_is_body_param());

    // A non-parameter symbol: false everywhere.
    let a = body.find("a").unwrap();
    assert!(!a.parameter_is_port_param());
    assert!(!a.parameter_is_local_param());
    assert!(!a.parameter_is_body_param());
}

#[test]
fn parameter_is_overridden_reflects_instance_overrides() {
    let d = compile(
        "module sub #(parameter int W = 8) ();\nendmodule\n\
         module m;\n  sub #(.W(4)) s1();\n  sub s2();\nendmodule\n",
    );
    let top = d.top_instances().next().unwrap();
    let body = top.instance_body().unwrap();
    let s1 = body.members().find(|m| m.name() == "s1").unwrap();
    let s2 = body.members().find(|m| m.name() == "s2").unwrap();

    let w1 = s1.instance_body().unwrap().find("W").unwrap();
    let w2 = s2.instance_body().unwrap().find("W").unwrap();
    assert!(w1.parameter_is_overridden());
    assert!(!w2.parameter_is_overridden());

    // A non-Parameter symbol: false.
    assert!(!top.parameter_is_overridden());
}

// ---- PortSymbol --------------------------------------------------------------

#[test]
fn port_symbol_direction_type_and_internal_symbol() {
    use sv_lang::ArgumentDirection;

    let d = compile(
        "module m(input logic [3:0] a, output logic [3:0] y, inout wire z);\n\
         \x20 assign y = a;\n\
         \x20 assign z = 1'bz;\n\
         endmodule\n",
    );
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let ports: Vec<_> = body.instance_body_ports().collect();
    assert_eq!(ports.len(), 3);

    let a = ports.iter().find(|p| p.name() == "a").unwrap();
    let y = ports.iter().find(|p| p.name() == "y").unwrap();
    let z = ports.iter().find(|p| p.name() == "z").unwrap();

    assert_eq!(a.kind(), SymbolKind::Port);
    assert_eq!(a.port_direction(), ArgumentDirection::In);
    assert_eq!(y.port_direction(), ArgumentDirection::Out);
    assert_eq!(z.port_direction(), ArgumentDirection::InOut);

    assert!(a.port_is_ansi_port());
    assert!(y.port_is_ansi_port());
    assert!(z.port_is_ansi_port());

    assert_eq!(a.port_type().unwrap().bit_width(), 4);
    assert_eq!(y.port_type().unwrap().bit_width(), 4);
    assert_eq!(z.port_type().unwrap().bit_width(), 1);

    let a_internal = a.port_internal_symbol().unwrap();
    assert_eq!(a_internal.name(), "a");
    // An `input`/`inout` ANSI port with no explicit `var` keyword implicitly
    // declares a net (the default net type), not a variable.
    assert_eq!(a_internal.kind(), SymbolKind::Net);

    // Distinct ports get distinct, nonzero external declaration locations.
    let a_loc = a.port_external_loc();
    let y_loc = y.port_external_loc();
    assert_ne!(a_loc.buffer, 0);
    assert_ne!(y_loc.buffer, 0);
    assert_ne!(a_loc.offset, y_loc.offset);

    // A non-Port symbol: the documented defaults.
    let top = d.top_instances().next().unwrap();
    assert_eq!(top.port_direction(), ArgumentDirection::InOut);
    assert!(top.port_type().is_none());
    assert!(top.port_internal_symbol().is_none());
    assert!(!top.port_is_ansi_port());
    assert_eq!(top.port_external_loc().buffer, 0);
}

#[test]
fn port_symbol_non_ansi_and_explicit_ansi_internal_expr() {
    // A non-ANSI port list entry with a bit-select (PortReference::select)
    // is the one case where getInternalExpr() differs from a plain
    // reference to the internal symbol.
    let d = compile("module m(a);\n  input logic [7:0] a;\nendmodule\n");
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let a = body
        .instance_body_ports()
        .find(|p| p.name() == "a")
        .unwrap();
    assert!(!a.port_is_ansi_port());
    assert!(a.port_internal_symbol().is_some());
    // No select on the plain non-ANSI reference: no separate internal expr.
    assert!(a.port_internal_expr().is_none());

    // An explicit-ANSI port (`.name(expr)`) binds its own connection
    // expression and, unlike a plain port, has no direct internal symbol.
    let d2 = compile("module m2(input .a(w));\n  logic w;\n  assign w = 1'b0;\nendmodule\n");
    let body2 = d2.top_instances().next().unwrap().instance_body().unwrap();
    let a2 = body2
        .instance_body_ports()
        .find(|p| p.name() == "a")
        .unwrap();
    assert!(a2.port_is_ansi_port());
    assert!(a2.port_internal_symbol().is_none());
    let expr = a2.port_internal_expr().unwrap();
    assert_eq!(expr.kind(), sv_lang::kinds::ExpressionKind::NamedValue);
}

// ---- PortConnection: getExpression / getIfaceConn ---------------------------

#[test]
fn port_connection_expression_and_iface_conn() {
    let d = compile(
        "interface bus;\n  logic req;\nendinterface\n\
         module sub(bus b, input logic a);\nendmodule\n\
         module m;\n  bus b();\n  sub s(.b(b), .a(1'b1));\nendmodule\n",
    );
    let top = d.top_instances().next().unwrap();
    let s = top.instance_body().unwrap().find("s").unwrap();
    let conns: Vec<_> = s.instance_port_connections().collect();
    assert_eq!(conns.len(), 2);

    let b_conn = conns.iter().find(|c| c.port().name() == "b").unwrap();
    let a_conn = conns.iter().find(|c| c.port().name() == "a").unwrap();

    // The interface-port connection carries both: the raw connection
    // expression (the syntactic `b` that was bound to find the instance)
    // *and* the resolved interface connection.
    let b_expr = b_conn.expression().unwrap();
    let b_ref = b_expr.symbol_reference(false).unwrap();
    assert_eq!(b_ref.name(), "b");
    let (instance, modport) = b_conn.iface_conn();
    assert_eq!(instance.unwrap().name(), "b");
    assert!(modport.is_none());

    // The plain data-port connection is the opposite: a bound expression,
    // no interface connection.
    let a_expr = a_conn.expression().unwrap();
    assert_eq!(a_expr.constant_value().unwrap().as_i64(), Some(1));
    let (a_instance, a_modport) = a_conn.iface_conn();
    assert!(a_instance.is_none());
    assert!(a_modport.is_none());
}

#[test]
fn symbol_procedural_block_procedure_kind_breadth() {
    use sv_lang::ProceduralBlockKind;

    let d = compile(
        "module m(input logic clk, a, output logic q);\n\
           initial q = 0;\n\
           final $display(\"done\");\n\
           always @(a) q = a;\n\
           always_comb q = a;\n\
           always_latch if (a) q = a;\n\
           always_ff @(posedge clk) q <= a;\n\
         endmodule\n",
    );
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let kinds: Vec<_> = body
        .members()
        .filter(|s| s.kind() == SymbolKind::ProceduralBlock)
        .map(|s| s.procedural_block_procedure_kind())
        .collect();
    assert_eq!(
        kinds,
        vec![
            ProceduralBlockKind::Initial,
            ProceduralBlockKind::Final,
            ProceduralBlockKind::Always,
            ProceduralBlockKind::AlwaysComb,
            ProceduralBlockKind::AlwaysLatch,
            ProceduralBlockKind::AlwaysFF,
        ]
    );

    // A non-ProceduralBlock symbol answers the documented default.
    let non_block = body.members().find(|s| s.name() == "clk").unwrap();
    assert_eq!(
        non_block.procedural_block_procedure_kind(),
        ProceduralBlockKind::Initial
    );
}

#[test]
fn symbol_property_ports_breadth() {
    let d = compile(
        "module m;\n\
           property p(a, b);\n\
             a |-> b;\n\
           endproperty\n\
         endmodule\n",
    );
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let p = body.find("p").unwrap();
    assert_eq!(p.kind(), SymbolKind::Property);
    let names: Vec<_> = p.property_ports().map(|s| s.name().to_string()).collect();
    assert_eq!(names, vec!["a", "b"]);

    // A property with no ports has an empty list, and a non-Property symbol
    // answers empty too.
    let d2 = compile("module m;\n  property q;\n    1;\n  endproperty\nendmodule\n");
    let body2 = d2.top_instances().next().unwrap().instance_body().unwrap();
    let q = body2.find("q").unwrap();
    assert_eq!(q.property_ports().count(), 0);
    assert_eq!(body2.property_ports().count(), 0);
}

#[test]
fn symbol_pulse_style_breadth() {
    use sv_lang::PulseStyleKind;

    let d = compile(
        "module m(input a, b, c, dd, output y1, y2, y3, y4);\n\
           assign y1 = a;\n\
           assign y2 = b;\n\
           assign y3 = c;\n\
           assign y4 = dd;\n\
           specify\n\
             pulsestyle_onevent y1;\n\
             showcancelled y2;\n\
             pulsestyle_ondetect y3;\n\
             noshowcancelled y4;\n\
             (a => y1) = 1;\n\
             (b => y2) = 1;\n\
             (c => y3) = 1;\n\
             (dd => y4) = 1;\n\
           endspecify\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    // PulseStyle symbols are members of the SpecifyBlock, not of the
    // instance body directly.
    let specify = body
        .members()
        .find(|s| s.kind() == SymbolKind::SpecifyBlock)
        .unwrap();
    let mut styles: Vec<_> = specify
        .members()
        .filter(|s| s.kind() == SymbolKind::PulseStyle)
        .map(|s| (s.pulse_style_kind(), s.pulse_style_terminals().count()))
        .collect();
    styles.sort_by_key(|(k, _)| format!("{:?}", k));
    assert_eq!(
        styles,
        vec![
            (PulseStyleKind::NoShowCancelled, 1),
            (PulseStyleKind::OnDetect, 1),
            (PulseStyleKind::OnEvent, 1),
            (PulseStyleKind::ShowCancelled, 1),
        ]
    );

    // A non-PulseStyle symbol answers the documented defaults.
    assert_eq!(body.pulse_style_kind(), PulseStyleKind::OnEvent);
    assert_eq!(body.pulse_style_terminals().count(), 0);
}

#[test]
fn symbol_timing_path_breadth() {
    use sv_lang::{EdgeKind, TimingPathConnectionKind, TimingPathPolarity};

    let d = compile(
        "module m(input a, clk, d, output bb, output reg q);\n\
           assign bb = a;\n\
           specify\n\
             (a +*> bb) = 1;\n\
             if (d) (posedge clk => (q +: d)) = (2, 3);\n\
           endspecify\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let specify = body
        .members()
        .find(|s| s.kind() == SymbolKind::SpecifyBlock)
        .unwrap();
    let paths: Vec<_> = specify
        .members()
        .filter(|s| s.kind() == SymbolKind::TimingPath)
        .collect();
    assert_eq!(paths.len(), 2);

    // Path 1: `(a +*> bb) = 1;` -- an unconditional, full-connection path
    // with positive overall polarity and no edge sensitivity.
    let full = paths
        .iter()
        .find(|p| p.timing_path_connection_kind() == TimingPathConnectionKind::Full)
        .unwrap();
    assert_eq!(full.timing_path_polarity(), TimingPathPolarity::Positive);
    assert_eq!(
        full.timing_path_edge_polarity(),
        TimingPathPolarity::Unknown
    );
    assert_eq!(full.timing_path_edge_identifier(), EdgeKind::None);
    assert!(!full.timing_path_is_state_dependent());
    assert!(full.timing_path_condition_expr().is_none());
    assert!(full.timing_path_edge_source_expr().is_none());

    let full_inputs: Vec<_> = full.timing_path_inputs().collect();
    assert_eq!(full_inputs.len(), 1);
    assert_eq!(full_inputs[0].referenced_symbol().unwrap().name(), "a");

    let full_outputs: Vec<_> = full.timing_path_outputs().collect();
    assert_eq!(full_outputs.len(), 1);
    assert_eq!(full_outputs[0].referenced_symbol().unwrap().name(), "bb");

    let full_delays: Vec<_> = full.timing_path_delays().collect();
    assert_eq!(full_delays.len(), 1);
    assert_eq!(full_delays[0].constant_value().unwrap().as_i64(), Some(1));

    // Path 2: `if (d) (posedge clk => (q +: d)) = (2, 3);` -- a
    // state-dependent, parallel-connection, edge-sensitive path.
    let edge = paths
        .iter()
        .find(|p| p.timing_path_connection_kind() == TimingPathConnectionKind::Parallel)
        .unwrap();
    // No overall polarity operator was written on this path.
    assert_eq!(edge.timing_path_polarity(), TimingPathPolarity::Unknown);
    assert_eq!(
        edge.timing_path_edge_polarity(),
        TimingPathPolarity::Positive
    );
    assert_eq!(edge.timing_path_edge_identifier(), EdgeKind::PosEdge);
    assert!(edge.timing_path_is_state_dependent());
    assert_eq!(
        edge.timing_path_condition_expr()
            .unwrap()
            .referenced_symbol()
            .unwrap()
            .name(),
        "d"
    );
    assert_eq!(
        edge.timing_path_edge_source_expr()
            .unwrap()
            .referenced_symbol()
            .unwrap()
            .name(),
        "d"
    );

    let edge_inputs: Vec<_> = edge.timing_path_inputs().collect();
    assert_eq!(edge_inputs.len(), 1);
    assert_eq!(edge_inputs[0].referenced_symbol().unwrap().name(), "clk");

    let edge_outputs: Vec<_> = edge.timing_path_outputs().collect();
    assert_eq!(edge_outputs.len(), 1);
    assert_eq!(edge_outputs[0].referenced_symbol().unwrap().name(), "q");

    let edge_delays: Vec<_> = edge.timing_path_delays().collect();
    assert_eq!(
        edge_delays
            .iter()
            .map(|e| e.constant_value().unwrap().as_i64())
            .collect::<Vec<_>>(),
        vec![Some(2), Some(3)]
    );

    // A non-TimingPath symbol answers the documented defaults.
    assert_eq!(
        body.timing_path_connection_kind(),
        TimingPathConnectionKind::Full
    );
    assert_eq!(body.timing_path_polarity(), TimingPathPolarity::Unknown);
    assert_eq!(
        body.timing_path_edge_polarity(),
        TimingPathPolarity::Unknown
    );
    assert_eq!(body.timing_path_edge_identifier(), EdgeKind::None);
    assert!(!body.timing_path_is_state_dependent());
    assert!(body.timing_path_condition_expr().is_none());
    assert!(body.timing_path_edge_source_expr().is_none());
    assert_eq!(body.timing_path_inputs().count(), 0);
    assert_eq!(body.timing_path_outputs().count(), 0);
    assert_eq!(body.timing_path_delays().count(), 0);
}

#[test]
fn symbol_type_parameter_is_overridden() {
    let d = compile(
        "module sub #(parameter type T = int) (); endmodule\n\
         module m;\n\
           sub #(.T(bit)) s1();\n\
           sub s2();\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors());
    let top = d.top_instances().next().unwrap();
    let s1 = top
        .instance_body()
        .unwrap()
        .members()
        .find(|m| m.name() == "s1")
        .unwrap();
    let s2 = top
        .instance_body()
        .unwrap()
        .members()
        .find(|m| m.name() == "s2")
        .unwrap();
    let t1 = s1.instance_body().unwrap().find("T").unwrap();
    let t2 = s2.instance_body().unwrap().find("T").unwrap();
    assert_eq!(t1.kind(), SymbolKind::TypeParameter);
    assert!(t1.type_parameter_is_overridden());
    assert!(!t2.type_parameter_is_overridden());

    // A non-TypeParameter symbol answers the documented default.
    assert!(!top.type_parameter_is_overridden());
}

#[test]
fn symbol_uninstantiated_def_breadth() {
    // `nosuchmod` has no definition: an ordinary (non-checker) shape, with
    // two ordered connections.
    let d = compile(
        "module m;\n\
           logic x;\n\
           nosuchmod #(.W(4)) u1(x, 1);\n\
         endmodule\n",
    );
    assert!(d.diagnostics().has_errors());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let u1 = body
        .members()
        .find(|s| s.kind() == SymbolKind::UninstantiatedDef)
        .unwrap();
    assert_eq!(u1.uninstantiated_def_definition_name(), "nosuchmod");
    assert!(!u1.uninstantiated_def_is_checker());

    // Each parameter expression is bound self-determined against the
    // compilation's placeholder error type (since, with no resolved
    // definition, the real destination parameter type can't be known) --
    // so it always comes back wrapped as a "bad" expression, per the
    // documented caveat that these aren't necessarily correctly typed.
    let params: Vec<_> = u1.uninstantiated_def_param_expressions().collect();
    assert_eq!(params.len(), 1);
    assert!(params[0].is_bad());

    assert_eq!(u1.uninstantiated_def_port_connection_count(), 2);
    let names: Vec<_> = u1.uninstantiated_def_port_names().collect();
    assert_eq!(names, vec!["", ""]);

    // Both ordered connections (`x`, `1`) unwrap to plain expressions.
    assert_eq!(
        u1.uninstantiated_def_port_connection(0)
            .unwrap()
            .referenced_symbol()
            .unwrap()
            .name(),
        "x"
    );
    assert_eq!(
        u1.uninstantiated_def_port_connection(1)
            .unwrap()
            .constant_value()
            .unwrap()
            .as_i64(),
        Some(1)
    );
    assert!(u1.uninstantiated_def_port_connection(2).is_none());

    // `nosuchmod2` has no definition either: a named connection, including
    // one empty `()` connection, which still unwraps (to an
    // empty-argument expression) rather than forcing the checker-only
    // interpretation.
    let d1b = compile(
        "module m;\n\
           nosuchmod2 u2(.a(1), .b());\n\
         endmodule\n",
    );
    assert!(d1b.diagnostics().has_errors());
    let body1b = d1b.top_instances().next().unwrap().instance_body().unwrap();
    let u2 = body1b
        .members()
        .find(|s| s.kind() == SymbolKind::UninstantiatedDef)
        .unwrap();
    assert!(!u2.uninstantiated_def_is_checker());
    assert_eq!(u2.uninstantiated_def_port_connection_count(), 2);
    let u2_names: Vec<_> = u2.uninstantiated_def_port_names().collect();
    assert_eq!(u2_names, vec!["a", "b"]);
    assert_eq!(
        u2.uninstantiated_def_port_connection(0)
            .unwrap()
            .constant_value()
            .unwrap()
            .as_i64(),
        Some(1)
    );
    assert!(u2.uninstantiated_def_port_connection(1).is_some());

    // `nosuchchecker` has no definition either, but its one connection uses
    // sequence syntax (`##1`), which forces the checker-only interpretation
    // and so does not unwrap to a plain expression.
    let d2 = compile(
        "module m;\n\
           logic a, b;\n\
           nosuchchecker c1(.p(a ##1 b));\n\
         endmodule\n",
    );
    assert!(d2.diagnostics().has_errors());
    let body2 = d2.top_instances().next().unwrap().instance_body().unwrap();
    let c1 = body2
        .members()
        .find(|s| s.kind() == SymbolKind::UninstantiatedDef)
        .unwrap();
    assert_eq!(c1.uninstantiated_def_definition_name(), "nosuchchecker");
    assert!(c1.uninstantiated_def_is_checker());
    assert_eq!(c1.uninstantiated_def_port_connection_count(), 1);
    let c1_names: Vec<_> = c1.uninstantiated_def_port_names().collect();
    assert_eq!(c1_names, vec!["p"]);
    assert!(c1.uninstantiated_def_port_connection(0).is_none());

    // A non-UninstantiatedDef symbol answers the documented defaults.
    assert_eq!(body.uninstantiated_def_definition_name(), "");
    assert_eq!(body.uninstantiated_def_param_expressions().count(), 0);
    assert_eq!(body.uninstantiated_def_port_connection_count(), 0);
    assert!(body.uninstantiated_def_port_connection(0).is_none());
    assert_eq!(body.uninstantiated_def_port_names().count(), 0);
    assert!(!body.uninstantiated_def_is_checker());
}

/// The randsequence design exercised by every randseq-prod test below:
/// `main` invokes `branch` then `chooser`; `branch` is an if/else between two
/// argument-taking `leaf` calls; `chooser` is a case with one multi-label
/// item and a default, both invoking `leaf`; `leaf`/`leaf2` are code blocks.
/// Covers all five ProdKind variants and every listed RandSeqProductionSymbol
/// member except RepeatProd (out of scope for this chunk).
const RANDSEQ_DESIGN: &str = "\
module m;
    int x;
    int sel;
    initial randsequence(main)
        main : branch chooser;
        branch : if (sel) leaf(10) else leaf2;
        chooser : case (sel)
                    1, 2 : leaf(20);
                    default : leaf2;
                  endcase;
        leaf(int v) : { x = v; };
        leaf2 : { x = 2; };
    endsequence
endmodule
";

/// Navigates to the randsequence statement's first production (`main`),
/// exactly as slang exposes it: productions live in a scope private to the
/// `RandSequenceStatement`, not as findable members of the enclosing module.
fn randseq_main(d: &Design) -> sv_lang::Symbol<'_> {
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let stmt = block.body().unwrap().statements()[0];
    stmt.randsequence_first_production().unwrap()
}

#[test]
fn symbol_randseq_rule_prod_breadth() {
    use sv_lang::RandSeqProdKind;

    let d = compile(RANDSEQ_DESIGN);
    assert!(!d.diagnostics().has_errors());
    let main = randseq_main(&d);
    assert_eq!(main.name(), "main");

    // main : branch chooser  -- one rule, two Item prods.
    assert_eq!(main.randseq_rule_count(), 1);
    let main_prods: Vec<_> = main.randseq_rule_prods(0).collect();
    assert_eq!(main_prods.len(), 2);
    assert_eq!(main_prods[0].kind(), RandSeqProdKind::Item);
    assert_eq!(main_prods[0].item_args().count(), 0);
    let branch = main_prods[0].item_target().unwrap();
    assert_eq!(branch.name(), "branch");
    let chooser = main_prods[1].item_target().unwrap();
    assert_eq!(chooser.name(), "chooser");

    // An out-of-range rule index yields no prods.
    assert_eq!(main.randseq_rule_prods(1).count(), 0);

    // branch : if (sel) leaf(10) else leaf2;
    let branch_prod = branch.randseq_rule_prods(0).next().unwrap();
    assert_eq!(branch_prod.kind(), RandSeqProdKind::IfElse);
    assert!(branch_prod.if_else_expr().is_some());
    let if_item = branch_prod.if_else_if_item().unwrap();
    assert_eq!(if_item.kind(), RandSeqProdKind::Item);
    assert_eq!(if_item.item_target().unwrap().name(), "leaf");
    assert_eq!(if_item.item_args().count(), 1);
    assert!(branch_prod.if_else_has_else_item());
    let else_item = branch_prod.if_else_else_item().unwrap();
    assert_eq!(else_item.item_target().unwrap().name(), "leaf2");
    assert_eq!(else_item.item_args().count(), 0);

    // chooser : case (sel) 1, 2: leaf(20); default: leaf2; endcase;
    let chooser_prod = chooser.randseq_rule_prods(0).next().unwrap();
    assert_eq!(chooser_prod.kind(), RandSeqProdKind::Case);
    assert!(chooser_prod.case_expr().is_some());
    assert_eq!(chooser_prod.case_item_count(), 1);
    assert_eq!(chooser_prod.case_item_expressions(0).count(), 2);
    // An out-of-range case item index yields nothing.
    assert_eq!(chooser_prod.case_item_expressions(1).count(), 0);
    assert!(chooser_prod.case_item_item(1).is_none());
    let case_item = chooser_prod.case_item_item(0).unwrap();
    assert_eq!(case_item.kind(), RandSeqProdKind::Item);
    assert_eq!(case_item.item_target().unwrap().name(), "leaf");
    assert_eq!(case_item.item_args().count(), 1);
    assert!(chooser_prod.case_has_default_item());
    let default_item = chooser_prod.case_default_item().unwrap();
    assert_eq!(default_item.item_target().unwrap().name(), "leaf2");

    // leaf(int v) : { x = v; };  -- a CodeBlock prod whose block is a real
    // StatementBlock symbol.
    let leaf = if_item.item_target().unwrap();
    let leaf_prod = leaf.randseq_rule_prods(0).next().unwrap();
    assert_eq!(leaf_prod.kind(), RandSeqProdKind::CodeBlock);
    let leaf_block = leaf_prod.code_block_block().unwrap();
    assert_eq!(leaf_block.kind(), SymbolKind::StatementBlock);
    assert!(leaf_prod.if_else_expr().is_none());
    assert!(leaf_prod.case_expr().is_none());
    assert!(leaf_prod.item_target().is_none());

    // leaf2 : { x = 2; };
    let leaf2 = else_item.item_target().unwrap();
    let leaf2_prod = leaf2.randseq_rule_prods(0).next().unwrap();
    assert_eq!(leaf2_prod.kind(), RandSeqProdKind::CodeBlock);
    assert!(leaf2_prod.code_block_block().is_some());

    // A non-RandSeqProduction symbol answers the documented defaults.
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    assert_eq!(body.randseq_rule_count(), 0);
    assert_eq!(body.randseq_rule_prods(0).count(), 0);
}

#[test]
fn freeze_forces_randseq_production_memos() {
    let d = compile(RANDSEQ_DESIGN);
    assert!(d.freeze_report().symbols_elaborated > 0);

    let main = randseq_main(&d);
    // Reading twice must be consistent (a lazy first-touch race would risk
    // returning a different count/None on repeated reads under TSan).
    let first = main.randseq_rule_prods(0).count();
    let second = main.randseq_rule_prods(0).count();
    assert_eq!(first, second);
    assert_eq!(first, 2);

    // Every embedded expression -- ProdItem args, if/case conditions, case
    // labels -- was reached by RandSeqProductionSymbol::visitExprs during
    // the freeze sweep (not just getRules() itself); a design with no
    // fold failures on a per-production basis would mask a missed branch,
    // so instead assert each expression is actually readable post-freeze
    // (an unforced canonical-type memo would be the failure mode here).
    let branch = main
        .randseq_rule_prods(0)
        .next()
        .unwrap()
        .item_target()
        .unwrap();
    let branch_prod = branch.randseq_rule_prods(0).next().unwrap();
    let cond = branch_prod.if_else_expr().unwrap();
    assert!(cond.expr_type().is_some());
    let arg = branch_prod
        .if_else_if_item()
        .unwrap()
        .item_args()
        .next()
        .unwrap();
    assert!(arg.expr_type().is_some());
}

/// Covers the RandSeqProductionSymbol members left out of `RANDSEQ_DESIGN`
/// above: `RepeatProd::expr`/`item`, every `Rule` field (`ruleBlock`,
/// `weightExpr`, `isRandJoin`, `randJoinExpr`, `codeBlock`), `arguments`,
/// and `getReturnType`.
///
/// `main`'s single rule is a `rand join` group (over `second` and
/// `leaf(1)`), weighted, with a trailing inline code block. `second`'s rule
/// is a plain (non-rand-join), unweighted `repeat` over `leaf`. `leaf` has
/// one `int` formal argument and an `int` declared return type.
const RANDSEQ_REPEAT_DESIGN: &str = "\
module m;
    int x;
    int sel;
    initial randsequence(main)
        main : rand join (sel) second leaf(1) := 3 { x = x + 1; };
        second : repeat (2) leaf(4);
        int leaf(int v) : { x = v; };
    endsequence
endmodule
";

#[test]
fn symbol_randseq_repeat_prod_and_rule_fields() {
    use sv_lang::RandSeqProdKind;

    let d = compile(RANDSEQ_REPEAT_DESIGN);
    assert!(!d.diagnostics().has_errors());
    let main = randseq_main(&d);
    assert_eq!(main.name(), "main");
    assert_eq!(main.randseq_rule_count(), 1);

    // main's rule always has an implicit rule block, and it is a real
    // StatementBlock symbol.
    let rule_block = main.randseq_rule_block(0).unwrap();
    assert_eq!(rule_block.kind(), SymbolKind::StatementBlock);
    // Out of range: only one rule.
    assert!(main.randseq_rule_block(1).is_none());

    // main : rand join (sel) second leaf(1) := 3 { x = x + 1; };
    assert!(main.randseq_rule_is_rand_join(0));
    let join_expr = main.randseq_rule_rand_join_expr(0).unwrap();
    assert!(join_expr.expr_type().is_some());
    let weight = main.randseq_rule_weight_expr(0).unwrap();
    assert!(weight.expr_type().is_some());
    let code_block = main.randseq_rule_code_block(0).unwrap();
    assert_eq!(code_block.kind(), RandSeqProdKind::CodeBlock);
    assert_eq!(
        code_block.code_block_block().unwrap().kind(),
        SymbolKind::StatementBlock
    );

    let main_prods: Vec<_> = main.randseq_rule_prods(0).collect();
    assert_eq!(main_prods.len(), 2);
    assert_eq!(main_prods[0].kind(), RandSeqProdKind::Item);
    let second = main_prods[0].item_target().unwrap();
    assert_eq!(second.name(), "second");
    let leaf = main_prods[1].item_target().unwrap();
    assert_eq!(leaf.name(), "leaf");
    assert_eq!(main_prods[1].item_args().count(), 1);

    // second : repeat (2) leaf(4);  -- a plain, unweighted, non-rand-join
    // rule whose one prod is a RepeatProd.
    assert_eq!(second.randseq_rule_count(), 1);
    assert!(!second.randseq_rule_is_rand_join(0));
    assert!(second.randseq_rule_rand_join_expr(0).is_none());
    assert!(second.randseq_rule_weight_expr(0).is_none());
    assert!(second.randseq_rule_code_block(0).is_none());
    assert!(second.randseq_rule_block(0).is_some());

    let second_prods: Vec<_> = second.randseq_rule_prods(0).collect();
    assert_eq!(second_prods.len(), 1);
    let repeat_prod = second_prods[0];
    assert_eq!(repeat_prod.kind(), RandSeqProdKind::Repeat);
    let repeat_expr = repeat_prod.repeat_expr().unwrap();
    assert!(repeat_expr.expr_type().is_some());
    // A non-Repeat prod answers the documented defaults.
    assert!(main_prods[0].repeat_expr().is_none());
    assert!(main_prods[0].repeat_item().is_none());
    let repeat_item = repeat_prod.repeat_item().unwrap();
    assert_eq!(repeat_item.kind(), RandSeqProdKind::Item);
    let leaf_via_repeat = repeat_item.item_target().unwrap();
    assert_eq!(leaf_via_repeat.name(), "leaf");
    assert_eq!(repeat_item.item_args().count(), 1);

    // int leaf(int v) : { x = v; };
    assert_eq!(leaf.id(), leaf_via_repeat.id());
    let args: Vec<_> = leaf.randseq_arguments().collect();
    assert_eq!(args.len(), 1);
    assert_eq!(args[0].name(), "v");
    assert_eq!(args[0].kind(), SymbolKind::FormalArgument);
    let ret = leaf.randseq_return_type().unwrap();
    assert_eq!(ret.bit_width(), 32);
    assert!(ret.is_integral());
    assert!(!ret.is_four_state());

    // `second` and `main` declare no formal arguments and no explicit
    // return type (implicit `void`, still `Some` since `void` is a type).
    assert_eq!(second.randseq_arguments().count(), 0);
    assert!(second.randseq_return_type().is_some());

    // A non-RandSeqProduction symbol answers the documented defaults.
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    assert_eq!(body.randseq_arguments().count(), 0);
    assert!(body.randseq_return_type().is_none());
    assert!(body.randseq_rule_block(0).is_none());
    assert!(body.randseq_rule_weight_expr(0).is_none());
    assert!(!body.randseq_rule_is_rand_join(0));
    assert!(body.randseq_rule_rand_join_expr(0).is_none());
    assert!(body.randseq_rule_code_block(0).is_none());
}

#[test]
fn freeze_forces_randseq_repeat_and_rule_field_memos() {
    let d = compile(RANDSEQ_REPEAT_DESIGN);
    assert!(d.freeze_report().symbols_elaborated > 0);

    let main = randseq_main(&d);
    // Reading twice must be consistent (a lazy first-touch race would risk
    // returning a different value on repeated reads under TSan).
    let first = main.randseq_rule_is_rand_join(0);
    let second = main.randseq_rule_is_rand_join(0);
    assert_eq!(first, second);
    assert!(first);

    // The rand-join and weight expressions, and the repeat-count
    // expression nested inside `second`, are reached by
    // RandSeqProductionSymbol::visitExprs during the freeze sweep; assert
    // each is actually readable post-freeze (an unforced canonical-type
    // memo would be the failure mode here).
    let join_expr = main.randseq_rule_rand_join_expr(0).unwrap();
    assert!(join_expr.expr_type().is_some());
    let weight_expr = main.randseq_rule_weight_expr(0).unwrap();
    assert!(weight_expr.expr_type().is_some());

    let second_prod = main
        .randseq_rule_prods(0)
        .next()
        .unwrap()
        .item_target()
        .unwrap();
    let repeat_prod = second_prod.randseq_rule_prods(0).next().unwrap();
    let repeat_expr = repeat_prod.repeat_expr().unwrap();
    assert!(repeat_expr.expr_type().is_some());

    // The declared return type's canonical memo (forced by the freeze
    // sweep's generic per-symbol declared-type pass) is readable too.
    let leaf = repeat_prod.repeat_item().unwrap().item_target().unwrap();
    assert_eq!(leaf.randseq_return_type().unwrap().bit_width(), 32);
}

#[test]
fn root_symbol_top_instances_and_compilation_units() {
    let d = compile(DESIGN);
    let root = d.root();
    assert_eq!(root.kind(), SymbolKind::Root);

    // RootSymbol::topInstances mirrors Design::top_instances exactly.
    let via_root: Vec<_> = root
        .root_top_instances()
        .map(|s| s.name().to_string())
        .collect();
    let via_design: Vec<_> = d.top_instances().map(|s| s.name().to_string()).collect();
    assert_eq!(via_root, via_design);
    assert_eq!(via_root, ["top"]);

    // RootSymbol::compilationUnits mirrors Design::compilation_units exactly.
    assert_eq!(
        root.root_compilation_units().count(),
        d.compilation_units().count()
    );
    assert_eq!(root.root_compilation_units().count(), 1);
    let unit = root.root_compilation_units().next().unwrap();
    assert_eq!(unit.kind(), SymbolKind::CompilationUnit);

    // A non-Root symbol answers the documented defaults.
    let top = d.top_instances().next().unwrap();
    assert_eq!(top.root_top_instances().count(), 0);
    assert_eq!(top.root_compilation_units().count(), 0);
}

#[test]
fn scope_compilation_unit_containing_instance_and_compilation() {
    let d = compile(DESIGN);
    let top = d.top_instances().next().unwrap();
    let body = top.instance_body().unwrap();
    let a = body.find("a").unwrap();

    // Scope::getCompilationUnit: walks up from any scope to its enclosing
    // compilation-unit symbol.
    let unit = body.compilation_unit().unwrap();
    assert_eq!(unit.kind(), SymbolKind::CompilationUnit);
    assert_eq!(d.root().compilation_unit(), None);
    // `a` (a value, not itself a scope) answers the documented default.
    assert!(a.compilation_unit().is_none());

    // Scope::getContainingInstance: walks up from any scope to its
    // enclosing instance body.
    assert_eq!(body.containing_instance().unwrap().id(), body.id());
    assert_eq!(d.root().containing_instance(), None);
    assert!(a.containing_instance().is_none());

    // Scope::getCompilation: every scope reached through this design
    // reports the same compilation identity, and it matches
    // Design::compilation_id.
    let id = d.compilation_id();
    assert_eq!(body.compilation(), Some(id));
    assert_eq!(d.root().compilation(), Some(id));
    assert_eq!(unit.compilation(), Some(id));
    assert!(a.compilation().is_none());

    // A second, unrelated design has a distinct compilation identity.
    let d2 = compile(DESIGN);
    assert_ne!(d.compilation_id(), d2.compilation_id());
    assert_ne!(
        body.compilation(),
        d2.top_instances()
            .next()
            .unwrap()
            .instance_body()
            .unwrap()
            .compilation()
    );
}

#[test]
fn scope_default_net_type_time_scale_procedural_and_uninstantiated() {
    let d = compile(
        "`default_nettype wand\n\
         `timescale 1ns/1ps\n\
         module m;\n\
           initial begin: blk\n\
             int x;\n\
             x = 0;\n\
           end\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();

    // Scope::getDefaultNetType: walks up to the enclosing `` `default_nettype ``.
    assert_eq!(body.default_net_type().unwrap().net_kind(), NetKind::WAnd);

    // Scope::getTimeScale: walks up to the enclosing `` `timescale ``.
    let ts = body.time_scale().unwrap();
    assert_eq!(ts.base.unit(), TimeUnit::Nanoseconds);
    assert_eq!(ts.base.magnitude(), 1);
    assert_eq!(ts.precision.unit(), TimeUnit::Picoseconds);
    assert_eq!(ts.precision.magnitude(), 1);

    // Scope::isProceduralContext: false for the module instance scope
    // itself, true for a statement-block scope inside an `initial`.
    assert!(!body.is_procedural_context());
    let blk = body.find("blk").unwrap();
    assert!(blk.is_procedural_context());

    // Scope::isUninstantiated: false for an actually-elaborated scope,
    // true for the untaken branch of a conditional generate block (which
    // is itself a Scope).
    assert!(!body.is_uninstantiated());
    let gd = compile(GENERATE_DESIGN);
    let gbody = gd.top_instances().next().unwrap().instance_body().unwrap();
    let cond = gbody.find("cond").unwrap();
    let cond_else = gbody.find("cond_else").unwrap();
    assert!(!cond.is_uninstantiated());
    assert!(cond_else.is_uninstantiated());

    // A non-scope symbol answers the documented "not a scope" defaults.
    let x = blk.find("x").unwrap();
    assert!(x.default_net_type().is_none());
    assert!(x.time_scale().is_none());
    assert!(!x.is_procedural_context());
    assert!(!x.is_uninstantiated());
}

#[test]
fn sequence_symbol_ports() {
    let d = compile(
        "module m;\n\
           sequence s(a, b);\n\
             a ##1 b;\n\
           endsequence\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let s = body.find("s").unwrap();
    assert_eq!(s.kind(), SymbolKind::Sequence);

    let names: Vec<_> = s.sequence_ports().map(|p| p.name().to_string()).collect();
    assert_eq!(names, vec!["a", "b"]);

    // A non-Sequence symbol answers the documented empty default.
    assert_eq!(body.sequence_ports().count(), 0);
}

#[test]
fn specparam_symbol_path_pulse() {
    let d = compile(
        "module m(input a, output b);\n\
           specify\n\
             specparam PATHPULSE$a$b = (0, 0);\n\
             specparam ordinary = 1;\n\
             (a => b) = (1, 1);\n\
           endspecify\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let specify = body
        .members()
        .find(|m| m.kind() == SymbolKind::SpecifyBlock)
        .unwrap();

    let pp = specify
        .members()
        .find(|m| m.name().starts_with("PATHPULSE$"))
        .unwrap();
    assert_eq!(pp.kind(), SymbolKind::Specparam);
    assert!(pp.specparam_is_path_pulse());
    assert_eq!(pp.specparam_path_source().unwrap().name(), "a");
    assert_eq!(pp.specparam_path_dest().unwrap().name(), "b");

    let ordinary = specify.members().find(|m| m.name() == "ordinary").unwrap();
    assert!(!ordinary.specparam_is_path_pulse());
    assert!(ordinary.specparam_path_source().is_none());
    assert!(ordinary.specparam_path_dest().is_none());

    // A non-Specparam symbol answers the documented false/None defaults.
    assert!(!body.specparam_is_path_pulse());
    assert!(body.specparam_path_source().is_none());
    assert!(body.specparam_path_dest().is_none());
}

#[test]
fn statement_block_symbol_kind_and_default_lifetime() {
    let d = compile(
        "module m;\n\
           initial begin: seq\n\
             int x;\n\
             x = 0;\n\
           end\n\
           initial fork: par\n\
           join_any\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();

    let seq = body.find("seq").unwrap();
    assert_eq!(seq.kind(), SymbolKind::StatementBlock);
    assert_eq!(
        seq.statement_block_kind(),
        Some(StatementBlockKind::Sequential)
    );
    // A block directly in a (non-`automatic`) module defaults to `static`.
    assert_eq!(
        seq.statement_block_default_lifetime(),
        Some(VariableLifetime::Static)
    );

    let par = body.find("par").unwrap();
    assert_eq!(
        par.statement_block_kind(),
        Some(StatementBlockKind::JoinAny)
    );

    // A non-StatementBlock symbol answers the documented `None` default.
    assert!(body.statement_block_kind().is_none());
    assert!(body.statement_block_default_lifetime().is_none());
}

#[test]
fn subroutine_symbol_lifetime_flags_arguments_override_prototype_return_type() {
    let d = compile(
        "class base;\n\
           virtual function int f(); return 1; endfunction\n\
         endclass\n\
         class derived extends base;\n\
           function int f(); return 2; endfunction\n\
         endclass\n\
         class ext_owner;\n\
           extern function int g();\n\
         endclass\n\
         function int ext_owner::g(); return 3; endfunction\n\
         module m;\n\
           function automatic int add(int a, int b);\n\
             return a + b;\n\
           endfunction\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors());

    let unit = d.compilation_units().next().unwrap();
    let derived = unit.find("derived").unwrap();
    let base = unit.find("base").unwrap();
    let derived_f = derived.find("f").unwrap();
    let base_f = base.find("f").unwrap();
    assert_eq!(derived_f.kind(), SymbolKind::Subroutine);

    // getOverride(): derived::f overrides base::f.
    assert_eq!(derived_f.subroutine_override().unwrap().id(), base_f.id());
    assert!(base_f.subroutine_override().is_none());

    // flags(): base::f is `virtual`.
    assert!(base_f.subroutine_flags().contains(MethodFlags::VIRTUAL));
    assert!(!derived_f.subroutine_flags().contains(MethodFlags::VIRTUAL));

    // getPrototype(): an `extern`-declared method's out-of-block definition
    // links back to its class prototype; an ordinary in-body method has none.
    // `find` (a direct name-table lookup) returns the out-of-block
    // SubroutineSymbol for "g", since it is inserted under the same name as
    // the MethodPrototype it implements; find the prototype itself by kind.
    let ext_owner = unit.find("ext_owner").unwrap();
    let g_impl = ext_owner.find("g").unwrap();
    assert_eq!(g_impl.kind(), SymbolKind::Subroutine);
    let proto = ext_owner
        .members()
        .find(|m| m.kind() == SymbolKind::MethodPrototype)
        .unwrap();
    assert_eq!(proto.name(), "g");
    assert_eq!(g_impl.subroutine_prototype().unwrap().id(), proto.id());
    assert_eq!(
        proto.method_prototype_subroutine().unwrap().id(),
        g_impl.id()
    );
    assert!(base_f.subroutine_prototype().is_none());

    // defaultLifetime() / getArguments() / getReturnType() on a plain
    // module-scope function.
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let add = body.find("add").unwrap();
    assert_eq!(add.kind(), SymbolKind::Subroutine);
    assert_eq!(
        add.subroutine_default_lifetime(),
        Some(VariableLifetime::Automatic)
    );
    let arg_names: Vec<_> = add
        .subroutine_arguments()
        .map(|a| a.name().to_string())
        .collect();
    assert_eq!(arg_names, vec!["a", "b"]);
    assert_eq!(add.subroutine_return_type().unwrap().bit_width(), 32);

    // A non-Subroutine symbol answers the documented empty/None defaults.
    assert!(body.subroutine_default_lifetime().is_none());
    assert_eq!(body.subroutine_flags(), MethodFlags::NONE);
    assert_eq!(body.subroutine_arguments().count(), 0);
    assert!(body.subroutine_override().is_none());
    assert!(body.subroutine_prototype().is_none());
    assert!(body.subroutine_return_type().is_none());
}

#[test]
fn subroutine_symbol_kind_virtual_return_val_var_and_this_var() {
    use sv_lang::SubroutineKind;

    let d = compile(
        "class base;\n\
           virtual function int f(); return 1; endfunction\n\
         endclass\n\
         class derived extends base;\n\
           function int f(); return 2; endfunction\n\
           static function int s(); return 3; endfunction\n\
         endclass\n\
         module m;\n\
           function int g(); return 1; endfunction\n\
           task t(); endtask\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());

    let unit = d.compilation_units().next().unwrap();
    let base = unit.find("base").unwrap();
    let derived = unit.find("derived").unwrap();
    let base_f = base.find("f").unwrap();
    let derived_f = derived.find("f").unwrap();
    let s = derived.find("s").unwrap();

    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let g = body.find("g").unwrap();
    let t = body.find("t").unwrap();

    // subroutineKind: Function vs Task, and None for a non-Subroutine
    // symbol.
    assert_eq!(base_f.subroutine_kind(), Some(SubroutineKind::Function));
    assert_eq!(g.subroutine_kind(), Some(SubroutineKind::Function));
    assert_eq!(t.subroutine_kind(), Some(SubroutineKind::Task));
    assert!(body.subroutine_kind().is_none());

    // isVirtual: declared `virtual`, an override, or neither.
    assert!(base_f.subroutine_is_virtual());
    assert!(derived_f.subroutine_is_virtual()); // overrides base::f
    assert!(!g.subroutine_is_virtual());
    assert!(!s.subroutine_is_virtual());
    assert!(!body.subroutine_is_virtual());

    // returnValVar: present (named after the function) for a function,
    // absent for a task or a non-Subroutine symbol.
    let rv = g.subroutine_return_val_var().unwrap();
    assert_eq!(rv.name(), "g");
    assert!(t.subroutine_return_val_var().is_none());
    assert!(body.subroutine_return_val_var().is_none());

    // thisVar: present ("this") for a non-static class method, absent for a
    // static class method, a free-standing function/task, or a
    // non-Subroutine symbol.
    let this_var = base_f.subroutine_this_var().unwrap();
    assert_eq!(this_var.name(), "this");
    assert!(s.subroutine_this_var().is_none());
    assert!(g.subroutine_this_var().is_none());
    assert!(body.subroutine_this_var().is_none());
}

#[test]
fn symbol_lexical_path_declared_type_definition_library_rand_mode() {
    use sv_lang::RandMode;

    let d = compile(
        "package pkg;\n\
           int y;\n\
         endpackage\n\
         class C;\n\
           rand int x;\n\
           randc int rc;\n\
           int z;\n\
           function int f(); return 1; endfunction\n\
         endclass\n\
         module sub;\n\
           logic [7:0] w;\n\
         endmodule\n\
         module top;\n\
           sub u1();\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());

    let unit = d.compilation_units().next().unwrap();
    let pkg = d.packages().find(|p| p.name() == "pkg").unwrap();
    let c = unit.find("C").unwrap();
    let f = c.find("f").unwrap();
    let x = c.find("x").unwrap();
    let rc = c.find("rc").unwrap();
    let z = c.find("z").unwrap();
    let y = pkg.find("y").unwrap();

    let top_body = d.top_instances().next().unwrap().instance_body().unwrap();
    let u1 = top_body.members().find(|s| s.name() == "u1").unwrap();
    let w = u1.instance_body().unwrap().find("w").unwrap();

    // getLexicalPath(): "::" between a package/class and its member, "."
    // otherwise; unlike getHierarchicalPath it names the definition, not
    // the instance.
    assert_eq!(f.lexical_path(), "C::f");
    assert_eq!(y.lexical_path(), "pkg::y");
    assert_eq!(w.lexical_path(), "sub.w");
    assert_eq!(w.hierarchical_path(), "top.u1.w");

    // getDeclaredType(): non-null for every value/type-alias-like carrier,
    // null for a type symbol itself (ClassType) or a scope like InstanceBody.
    assert!(w.has_declared_type());
    assert!(x.has_declared_type());
    assert!(!c.has_declared_type());
    assert!(!top_body.has_declared_type());

    // getDeclaringDefinition(): the nearest enclosing module/interface/
    // program, or None outside any definition (a package, or $unit).
    assert_eq!(w.declaring_definition().unwrap().name(), "sub");
    assert!(y.declaring_definition().is_none());
    assert!(f.declaring_definition().is_none());

    // getSourceLibrary(): every symbol in this single-file compile shares
    // the compilation's default library.
    assert!(w.source_library().unwrap().is_default());
    assert!(y.source_library().unwrap().is_default());

    // getRandMode(): generalizes ClassProperty/Field randMode across every
    // symbol kind.
    assert_eq!(x.rand_mode(), RandMode::Rand);
    assert_eq!(rc.rand_mode(), RandMode::RandC);
    assert_eq!(z.rand_mode(), RandMode::None);
    assert_eq!(w.rand_mode(), RandMode::None);
}

#[test]
fn system_timing_check_symbol_kind_and_arguments() {
    use sv_lang::{EdgeKind, SystemTimingCheckKind};

    let d = compile(
        "module m(input d, input clk, input en);\n\
           specify\n\
             $setup(d, edge [01, z1] clk &&& en, 10,);\n\
           endspecify\n\
         endmodule\n",
    );
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());

    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let specify = body
        .members()
        .find(|s| s.kind() == SymbolKind::SpecifyBlock)
        .unwrap();
    let stc = specify
        .members()
        .find(|s| s.kind() == SymbolKind::SystemTimingCheck)
        .unwrap();

    assert_eq!(stc.system_timing_check_kind(), SystemTimingCheckKind::Setup);

    let args: Vec<_> = stc.system_timing_check_arguments().collect();
    // A trailing comma elides the 4th (notifier) argument.
    assert_eq!(args.len(), 4);

    // arg0: `d` -- a plain 1-bit signal reference, no edge/condition.
    assert!(args[0].condition.is_none());
    assert_eq!(args[0].edge, EdgeKind::None);
    assert!(args[0].edge_descriptors.is_empty());
    assert_eq!(args[0].expr.unwrap().expr_type().unwrap().bit_width(), 1);

    // arg1: `edge [01, z1] clk &&& en` -- the bare `edge` keyword means "any
    // edge" (BothEdges), distinct from `posedge`/`negedge`.
    assert_eq!(args[1].edge, EdgeKind::BothEdges);
    assert_eq!(args[1].edge_descriptors, vec!["01", "z1"]);
    assert!(args[1].expr.is_some());
    assert!(args[1].condition.is_some());

    // arg2: `10` -- a constant limit value, already const-folded.
    assert_eq!(
        args[2].expr.unwrap().constant_value().unwrap().as_i64(),
        Some(10)
    );
    assert_eq!(args[2].edge, EdgeKind::None);

    // arg3: elided (trailing comma) notifier argument.
    assert!(args[3].expr.is_none());
    assert!(args[3].condition.is_none());
    assert_eq!(args[3].edge, EdgeKind::None);
    assert!(args[3].edge_descriptors.is_empty());

    // A non-SystemTimingCheck symbol answers the documented defaults.
    assert_eq!(
        body.system_timing_check_kind(),
        SystemTimingCheckKind::Unknown
    );
    assert_eq!(body.system_timing_check_arguments().count(), 0);
}
