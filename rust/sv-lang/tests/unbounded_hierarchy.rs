//! Unconditional recursive instantiation (ivtest pr2728812b/c shape): slang's
//! elaboration stops at `maxInstanceDepth` with a fatal diagnostic, but the
//! freeze sweep used to walk the whole (instance-caching-off) tree — a binary
//! recursion is ~2^128 instances — and never returned. `compile()` now refuses
//! with `Error::UnboundedHierarchy`. Reported by the sv-sim session.

use std::time::{Duration, Instant};

use sv_lang::{Compilation, Error, Options, Session};

const BINARY: &str = "\
module top;
  wire [10:0] s;
  sum #(5, 8) u (s);
endmodule
module sum #(parameter n = 4, parameter width = 8) (output [width:0] s);
  if (n == -1) begin
    assign s = 0;
  end else begin
    wire [width:0] a1, a2;
    sum #(n/2, width) s0 (a1);
    sum #(n-n/2, width) s1 (a2);
    assign s = a1 + a2;
  end
endmodule
";

fn compile(src: &str, opts: Options) -> (Result<sv_lang::Design, Error>, Duration) {
    let s = Session::new();
    let mut comp = Compilation::new_with(&s, opts).unwrap();
    comp.add_source(src).unwrap();
    let t = Instant::now();
    let r = comp.compile();
    (r, t.elapsed())
}

#[test]
fn binary_recursion_is_refused_not_hung() {
    let (r, took) = compile(BINARY, Options::new());
    let Err(err) = r else {
        panic!("an unbounded hierarchy must not produce a Design");
    };
    let Error::UnboundedHierarchy(msg) = &err else {
        panic!("expected UnboundedHierarchy, got {err:?}");
    };
    assert!(
        msg.contains("module instantiation exceeded maximum depth of 128"),
        "{msg}"
    );
    eprintln!("{err}");
    assert!(took < Duration::from_secs(10), "took {took:?}");
}

#[test]
fn max_instance_depth_is_honoured() {
    let (r, _) = compile(BINARY, Options::new().with_max_instance_depth(8));
    match r {
        Err(Error::UnboundedHierarchy(msg)) => {
            assert!(msg.contains("maximum depth of 8"), "{msg}")
        }
        other => panic!("expected UnboundedHierarchy, got {:?}", other.err()),
    }
}

#[test]
fn driver_path_refuses_too() {
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("unbounded_binary.sv");
    std::fs::write(&path, BINARY).unwrap();
    let mut driver =
        sv_lang::Driver::from_args([path.to_str().unwrap(), "--max-hierarchy-depth", "12"])
            .unwrap();
    match driver.compile() {
        Err(Error::UnboundedHierarchy(msg)) => {
            assert!(msg.contains("maximum depth of 12"), "{msg}")
        }
        other => panic!("expected UnboundedHierarchy, got {:?}", other.err()),
    }
}

#[test]
fn bounded_recursion_still_compiles() {
    // Terminates at n == 1: a finite tree, no hierarchy problem.
    let src = BINARY.replace("n == -1", "n <= 1");
    let (r, _) = compile(&src, Options::new());
    let d = r.expect("bounded recursion compiles");
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
}
