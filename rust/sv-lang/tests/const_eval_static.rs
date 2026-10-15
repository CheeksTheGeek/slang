//! Explicitly `static` subroutine variables in constant evaluation (ivtest
//! func_init_var1): slang used to skip their initializers (warning
//! ConstEvalStaticSkipped), so `static int acc = 1; acc += n;` folded as if acc
//! started at 0. They are now initialized at the start of each top-level
//! constant function invocation (LRM 13.4.3), shared by calls nested inside it.
//! Reported by the sv-sim session.

use sv_lang::{Compilation, Session};

const SRC: &str = "\
module m;
  function int f(int n);
    static int acc = 1;
    acc = acc + n;
    return acc;
  endfunction
  localparam int A = f(2);
  localparam int B = f(2) + f(3);
  function int g;
    return f(2) + f(3);
  endfunction
  localparam int C = g();
endmodule
";

#[test]
fn static_initializer_runs_per_top_level_invocation() {
    let s = Session::new();
    let mut comp = Compilation::new(&s).unwrap();
    comp.add_source(SRC).unwrap();
    let d = comp.compile().unwrap();
    assert!(
        d.diagnostics().is_empty(),
        "expected no diagnostics (no StaticSkipped warning): {}",
        d.diagnostics()
    );
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let val = |n: &str| body.find(n).unwrap().constant_value().unwrap().as_i64();
    assert_eq!(val("A"), Some(3)); // 1 + 2
    // Sibling top-level calls each start from the initializer: 3 + 4 (LRM
    // 13.4.3; iverilog agrees).
    assert_eq!(val("B"), Some(3 + 4));
    // Calls nested inside one invocation (g) share the storage: 3 + 6.
    assert_eq!(val("C"), Some(3 + 6));
}
