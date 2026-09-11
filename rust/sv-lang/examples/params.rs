//! Shows elaboration-time constant folding: every parameter and localparam is
//! reported with the concrete value slang computed for it. Because each
//! instance is elaborated separately, the *same* module prints different folded
//! values under different overrides — localparams derived by arithmetic
//! (`1 << AWIDTH`, `(1 << DWIDTH) - 1`) are evaluated, not left symbolic.
//!
//!     cargo run --example params -- your_design.sv
//!
//! With no arguments it runs on a small built-in design.

use sv_lang::{Compilation, Session, Symbol, Walk, kinds::SymbolKind};

const DEMO: &str = "\
module fifo #(parameter int AWIDTH = 4, parameter int DWIDTH = 8) (input logic clk);
    localparam int DEPTH   = 1 << AWIDTH;
    localparam int PTR_W   = AWIDTH + 1;
    localparam int MASK    = (1 << DWIDTH) - 1;
    localparam int BYTES   = (DWIDTH + 7) / 8;
endmodule

module top;
    logic clk;
    fifo #(.AWIDTH(4),  .DWIDTH(8))  narrow (.clk(clk));
    fifo #(.AWIDTH(10), .DWIDTH(32)) wide   (.clk(clk));
endmodule
";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = Session::new();
    let mut comp = Compilation::new(&session)?;
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        comp.add_source(DEMO)?;
    } else {
        for path in &args {
            comp.add(&session.parse_file(path)?)?;
        }
    }
    let design = comp.compile()?;

    for top in design.top_instances() {
        walk(top, 0);
    }
    Ok(())
}

fn walk(inst: Symbol<'_>, depth: usize) {
    let indent = "  ".repeat(depth);
    let def = inst
        .instance_definition()
        .map(|d| d.name().to_string())
        .unwrap_or_default();
    println!("{indent}{} : {def}", inst.name());

    for p in inst.parameters() {
        if let Some(v) = p.parameter_value() {
            println!("{indent}  {} = {v}", p.name());
        }
    }

    if let Some(body) = inst.instance_body() {
        body.visit(|s| {
            if s == body {
                return Walk::Continue;
            }
            if s.kind() == SymbolKind::Instance {
                walk(s, depth + 1);
                Walk::Skip
            } else {
                Walk::Continue
            }
        });
    }
}
