//! A connectivity linter: for every signal in a design, ask slang's analysis
//! engine who drives it — and flag the ones nobody drives. Each driver is
//! classified as continuous (`assign` / net connection) or procedural (written
//! inside an `always`/`initial`), so you can see *how* a signal gets its value.
//! Undriven signals (a common source of X-propagation bugs) are called out.
//!
//!     cargo run --example drivers -- your_design.sv
//!
//! With no arguments it runs on a small built-in design.

use sv_lang::{AnalysisFlags, Compilation, DriverKind, Session, kinds::SymbolKind};

const DEMO: &str = "\
module control (input logic clk, input logic a, input logic b, output logic y);
    logic q;
    logic scratch;
    logic dead;                          // declared, never driven
    assign y = a & b;                    // continuous driver
    always_ff @(posedge clk) q <= y;     // procedural driver
    always_comb scratch = a | b;         // procedural driver
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
    let analysis = design.analyze(AnalysisFlags::NONE, 1)?;

    let mut undriven = 0usize;
    for top in design.top_instances() {
        println!(
            "{}:",
            top.instance_definition()
                .map(|d| d.name().to_string())
                .unwrap_or_default()
        );
        let Some(body) = top.instance_body() else {
            continue;
        };
        for sym in body.members() {
            if !matches!(sym.kind(), SymbolKind::Variable | SymbolKind::Net) {
                continue;
            }
            let mut kinds = Vec::new();
            for d in analysis.drivers(sym) {
                kinds.push(match d.kind() {
                    DriverKind::Procedural => "procedural",
                    DriverKind::Continuous => "continuous",
                    DriverKind::Other => "other",
                });
            }
            if kinds.is_empty() {
                undriven += 1;
                println!("  ⚠ {:<10} UNDRIVEN", sym.name());
            } else {
                println!(
                    "  ✓ {:<10} driven by {} ({})",
                    sym.name(),
                    kinds.len(),
                    kinds.join(", ")
                );
            }
        }
    }
    if undriven > 0 {
        println!("\n{undriven} undriven signal(s) found.");
    }
    Ok(())
}
