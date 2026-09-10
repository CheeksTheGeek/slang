//! Parallel traversal (feature `rayon`): `par_visit` must cover exactly the
//! same symbols as the sequential `visit`, proving the earned `Design: Sync` is
//! usable for real concurrent work.
#![cfg(feature = "rayon")]

use std::sync::atomic::{AtomicUsize, Ordering};

use sv_lang::{Compilation, Session, Walk};

const SRC: &str = "\
module leaf #(parameter int N = 4) (input logic clk, input logic [N-1:0] d, output logic [N-1:0] q);
    always_ff @(posedge clk) q <= d;
endmodule
module mid(input logic clk);
    logic [3:0] a, b;
    leaf #(.N(4)) u0 (.clk(clk), .d(a), .q(b));
    leaf #(.N(4)) u1 (.clk(clk), .d(b), .q(a));
endmodule
module top(input logic clk);
    mid m0 (.clk(clk));
    mid m1 (.clk(clk));
endmodule
";

fn compile(src: &str) -> sv_lang::Design {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source(src).unwrap();
    comp.compile().unwrap()
}

#[test]
fn par_visit_matches_sequential() {
    let design = compile(SRC);

    // Sequential traversal from every top instance.
    let mut seq = 0usize;
    for top in design.top_instances() {
        top.visit(|_| {
            seq += 1;
            Walk::Continue
        });
    }

    // Parallel traversal over the same roots.
    let par = AtomicUsize::new(0);
    for top in design.top_instances() {
        design.par_visit_from(top, |_| {
            par.fetch_add(1, Ordering::Relaxed);
        });
    }

    assert!(seq > 10, "expected a non-trivial design, saw {seq} symbols");
    assert_eq!(
        par.load(Ordering::Relaxed),
        seq,
        "parallel traversal visited a different symbol count"
    );
}

#[test]
fn par_visit_whole_design() {
    let design = compile(SRC);
    let count = AtomicUsize::new(0);
    design.par_visit(|_| {
        count.fetch_add(1, Ordering::Relaxed);
    });
    assert!(count.load(Ordering::Relaxed) > 10);
}
