//! `cond ? a : b` with an x/z predicate and equal non-integral operands
//! (ivtest pr2453002): slang folded `udef ? 6 : 6.0` to 0.0 instead of 6.0
//! (LRM 11.4.11). Reported by the sv-sim session.

use sv_lang::{Compilation, Session};

#[test]
fn equal_real_arms_survive_an_unknown_predicate() {
    let s = Session::new();
    let mut comp = Compilation::new(&s).unwrap();
    comp.add_source(
        "module m;
  parameter udef = 1'bx;
  localparam real A = udef ? 6 : 6.0;
  localparam real B = udef ? 1.0 : 2.0;
endmodule
",
    )
    .unwrap();
    let d = comp.compile().unwrap();
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let real = |n: &str| match body.find(n).unwrap().constant_value().unwrap() {
        sv_lang::ConstantValue::Real(r) => r,
        other => panic!("{n}: {other:?}"),
    };
    assert_eq!(real("A"), 6.0);
    assert_eq!(real("B"), 0.0); // unequal: the type's default
}
