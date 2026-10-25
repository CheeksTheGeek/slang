//! The driven bit range and the unidirectional-port flag of a driver, read
//! through `Analysis::driver_handles` -> `ValueDriver::{bounds,
//! is_unidirectional_port}`. The flattened `DriverInfo` carries neither (it is
//! the small, copyable summary); the handle form is where the range lives.
//!
//! Requested by the Carcilator session so a simulator or linter can tell
//! *partial* drivers of one vector from a real multi-driver conflict.

use sv_lang::{AnalysisFlags, Compilation, Session};

fn bounds_of(src: &str, name: &str) -> Vec<(u64, u64)> {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source(src).unwrap();
    let design = comp.compile().unwrap();
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let analysis = design.analyze(AnalysisFlags::NONE, 1).unwrap();
    let sym = body.find(name).unwrap();
    let mut b: Vec<_> = analysis.driver_handles(sym).map(|d| d.bounds()).collect();
    b.sort_unstable();
    b
}

#[test]
fn two_continuous_assigns_drive_disjoint_halves() {
    // Carcilator's acceptance case.
    let b = bounds_of(
        "module m(input logic [3:0] a, b);\n  wire [7:0] v;\n  \
         assign v[3:0] = a;\n  assign v[7:4] = b;\nendmodule\n",
        "v",
    );
    assert_eq!(b, vec![(0, 3), (4, 7)]);
}

#[test]
fn a_whole_vector_driver_spans_the_full_width() {
    let b = bounds_of(
        "module m(input logic [7:0] a);\n  wire [7:0] v;\n  assign v = a;\nendmodule\n",
        "v",
    );
    assert_eq!(b, vec![(0, 7)]);
}

#[test]
fn single_bit_select_is_a_one_bit_range() {
    let b = bounds_of(
        "module m(input logic a);\n  wire [7:0] v;\n  assign v[5] = a;\nendmodule\n",
        "v",
    );
    assert_eq!(b, vec![(5, 5)]);
}

#[test]
fn procedural_partial_drivers_carry_their_ranges() {
    let b = bounds_of(
        "module m(input logic [3:0] a, b);\n  logic [7:0] v;\n  \
         always_comb v[3:0] = a;\n  always_comb v[7:4] = b;\nendmodule\n",
        "v",
    );
    assert_eq!(b, vec![(0, 3), (4, 7)]);
}

#[test]
fn unidirectional_flag_marks_port_connection_drivers_only() {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source(
        "module child(output logic o);\n  assign o = 1'b1;\nendmodule\n\
         module top;\n  wire w;\n  wire plain;\n  child c(.o(w));\n  assign plain = 1'b0;\n\
         endmodule\n",
    )
    .unwrap();
    let design = comp.compile().unwrap();
    let body = design
        .top_instances()
        .find(|i| i.name() == "top")
        .unwrap()
        .instance_body()
        .unwrap();
    let analysis = design.analyze(AnalysisFlags::NONE, 1).unwrap();

    // `w` is driven through child's output port: a unidirectional-port driver.
    let w = body.find("w").unwrap();
    let w_flags: Vec<bool> = analysis
        .driver_handles(w)
        .map(|d| d.is_unidirectional_port())
        .collect();
    assert_eq!(w_flags, vec![true]);

    // `plain` is driven by an ordinary continuous assign: not a port driver.
    let plain = body.find("plain").unwrap();
    let plain_flags: Vec<bool> = analysis
        .driver_handles(plain)
        .map(|d| d.is_unidirectional_port())
        .collect();
    assert_eq!(plain_flags, vec![false]);
}
