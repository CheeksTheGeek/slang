//! Tagged-union constants: `ConstantValue` used to carry only the union's
//! string form, so an initializer like an array of tagged unions could not be
//! encoded. `TaggedUnion { member, value, text }` now exposes the active member
//! and its value. Requested by the sv-sim session.

use sv_lang::{Compilation, ConstantValue, Session};

const SRC: &str = "\
module m;
  typedef union tagged { int a; bit [7:0] b; } U;
  localparam U ARR [2] = '{tagged a 5, tagged b 3};
  localparam U ONE = tagged a (-7);
endmodule
";

fn design() -> sv_lang::Design {
    let s = Session::new();
    let mut comp = Compilation::new(&s).unwrap();
    comp.add_source(SRC).unwrap();
    let d = comp.compile().unwrap();
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    d
}

fn tagged(cv: &ConstantValue) -> (u32, &ConstantValue) {
    match cv {
        ConstantValue::TaggedUnion { member, value, .. } => (*member, value),
        other => panic!("expected TaggedUnion, got {other:?}"),
    }
}

#[test]
fn array_of_tagged_unions() {
    let d = design();
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let arr = body.find("ARR").unwrap().constant_value().unwrap();
    let els = arr.as_array().expect("unpacked array");
    assert_eq!(els.len(), 2);

    let (m0, v0) = tagged(&els[0]);
    assert_eq!((m0, v0.as_i64()), (0, Some(5)));
    let (m1, v1) = tagged(&els[1]);
    assert_eq!(m1, 1);
    assert_eq!(v1.as_integer().unwrap().bit_width(), 8);
    assert_eq!(v1.as_i64(), Some(3));
}

#[test]
fn display_and_predicates_unchanged() {
    let d = design();
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let one = body.find("ONE").unwrap().constant_value().unwrap();
    let (m, v) = tagged(&one);
    assert_eq!((m, v.as_i64()), (0, Some(-7)));
    // Display is slang's own string form, as it was for Union(String).
    assert!(!one.to_string().is_empty());
    assert!(!one.is_container() && !one.is_true() && !one.is_false() && one.empty());
}
