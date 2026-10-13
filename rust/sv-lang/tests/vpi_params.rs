//! Parameter / defparam / specparam / genvar reflection for VPI (LRM 37.28),
//! requested by the sv-sim session: defparam target/value/RHS, a parameter's
//! declared default vs. its applied override, the instance `#()` overrides with
//! named-vs-positional (vpiConnByName), specparam values, and what a genvar
//! exposes.

use sv_lang::kinds::SymbolKind;
use sv_lang::{Compilation, ParameterAssignment, Session};

fn design(src: &str) -> sv_lang::Design {
    let s = Session::new();
    let mut comp = Compilation::new(&s).unwrap();
    comp.add_source(src).unwrap();
    comp.compile().unwrap()
}

#[test]
fn defparam_target_value_and_rhs() {
    let d = design(
        "module inner; parameter W = 1; endmodule\n\
         module top;\n  inner u();\n  defparam u.W = 3 + 4;\nendmodule\n",
    );
    let top = d.top_instances().next().unwrap().instance_body().unwrap();
    let dp = top
        .members()
        .find(|s| s.kind() == SymbolKind::DefParam)
        .expect("a defparam");
    let target = dp.defparam_target().expect("target resolves");
    assert_eq!(target.kind(), SymbolKind::Parameter);
    assert_eq!(target.name(), "W");
    assert_eq!(dp.defparam_value().unwrap().as_i64(), Some(7));
    assert!(dp.defparam_initializer().is_some());
    // The overridden parameter itself reflects the defparam value.
    let u = top.members().find(|s| s.name() == "u").unwrap();
    let w = u.instance_body().unwrap().find("W").unwrap();
    assert_eq!(w.constant_value().unwrap().as_i64(), Some(7));
}

#[test]
fn unresolved_defparam_target_is_none_not_a_panic() {
    // `nope` does not exist: slang reports an error and leaves the target
    // unresolved; the accessors must degrade to None, never abort.
    let d = design(
        "module inner; parameter W = 1; endmodule\n\
         module top;\n  inner u();\n  defparam u.nope = 5;\nendmodule\n",
    );
    let top = d.top_instances().next().unwrap().instance_body().unwrap();
    if let Some(dp) = top.members().find(|s| s.kind() == SymbolKind::DefParam) {
        assert!(dp.defparam_target().is_none());
        let _ = dp.defparam_initializer();
        let _ = dp.defparam_value();
    }
    // Non-defparam symbols: all three are None.
    let u = top.members().find(|s| s.name() == "u").unwrap();
    assert!(u.defparam_target().is_none());
    assert!(u.defparam_initializer().is_none());
}

#[test]
fn default_vs_override_and_assignment_kind() {
    let s = Session::new();
    let tree = s
        .parse(
            "module inner #(parameter int W = 2 * 3, parameter int D = 9); endmodule\n\
             module top; inner #(5) u1(); inner #(.D(11)) u2(); inner u3(); endmodule\n",
        )
        .unwrap();
    let mut comp = Compilation::new(&s).unwrap();
    comp.add(&tree).unwrap();
    let d = comp.compile().unwrap();
    let top = d.top_instances().next().unwrap().instance_body().unwrap();
    let body = |n: &str| {
        top.members()
            .find(|s| s.name() == n)
            .unwrap()
            .instance_body()
            .unwrap()
    };

    let w1 = body("u1").find("W").unwrap();
    assert!(w1.parameter_is_overridden());
    assert_eq!(w1.parameter_assignment(), ParameterAssignment::Ordered);
    assert_eq!(w1.constant_value().unwrap().as_i64(), Some(5)); // applied
    assert_eq!(
        w1.parameter_default_syntax(&tree).unwrap().text().trim(),
        "2 * 3"
    ); // declared default

    let d2 = body("u2").find("D").unwrap();
    assert_eq!(d2.parameter_assignment(), ParameterAssignment::Named);
    assert_eq!(d2.constant_value().unwrap().as_i64(), Some(11));

    // Not overridden: assignment None, and the default syntax matches the
    // bound initializer's source text.
    let w3 = body("u3").find("W").unwrap();
    assert!(!w3.parameter_is_overridden());
    assert_eq!(w3.parameter_assignment(), ParameterAssignment::None);
    assert_eq!(w3.constant_value().unwrap().as_i64(), Some(6));
    assert_eq!(
        w3.parameter_default_syntax(&tree).unwrap().text().trim(),
        "2 * 3"
    );

    // instance_parameter_overrides lists only the supplied ones.
    let u1 = top.members().find(|s| s.name() == "u1").unwrap();
    let ovs: Vec<_> = u1.instance_parameter_overrides().collect();
    assert_eq!(ovs.len(), 1);
    assert_eq!(ovs[0].parameter.name(), "W");
    let u3 = top.members().find(|s| s.name() == "u3").unwrap();
    assert_eq!(u3.instance_parameter_overrides().count(), 0);
}

#[test]
fn specparam_value_via_constant_value() {
    // Specparam is covered by Symbol::constant_value() (0.1.8).
    let d = design("module m;\n  specparam tRise = 5 + 1;\nendmodule\n");
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let sp = body.find("tRise").unwrap();
    assert_eq!(sp.kind(), SymbolKind::Specparam);
    assert_eq!(sp.constant_value().unwrap().as_i64(), Some(6));
}

#[test]
fn genvar_exposure() {
    let d = design(
        "module m;\n  genvar i;\n  for (i = 0; i < 3; i++) begin : g\n    logic x;\n  end\nendmodule\n",
    );
    let body = d.top_instances().next().unwrap().instance_body().unwrap();

    // The `genvar i;` declaration itself is a Genvar symbol: name + location
    // only (it has no value of its own outside elaboration).
    let gv = body.find("i").unwrap();
    assert_eq!(gv.kind(), SymbolKind::Genvar);

    // Each generated block carries its iteration index two ways: the block's
    // generate_block_array_index(), and an implicit localparam `i` that is
    // flagged parameter_is_from_genvar().
    let arr = body
        .members()
        .find(|s| s.kind() == SymbolKind::GenerateBlockArray)
        .expect("generate block array g");
    let mut indices = Vec::new();
    for blk in arr
        .members()
        .filter(|s| s.kind() == SymbolKind::GenerateBlock)
    {
        let idx = blk.generate_block_array_index().unwrap().as_i64().unwrap();
        let p = blk.find("i").expect("implicit genvar localparam");
        assert_eq!(p.kind(), SymbolKind::Parameter);
        assert!(p.parameter_is_from_genvar());
        assert_eq!(p.constant_value().unwrap().as_i64(), Some(idx));
        indices.push(idx);
    }
    indices.sort();
    assert_eq!(indices, vec![0, 1, 2]);

    // An ordinary parameter is not from a genvar.
    let x = design("module n; parameter P = 1; endmodule\n");
    let nb = x.top_instances().next().unwrap().instance_body().unwrap();
    assert!(!nb.find("P").unwrap().parameter_is_from_genvar());
}
