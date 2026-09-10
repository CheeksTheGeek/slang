//! Prints the elaborated design hierarchy of one or more SystemVerilog files.
//!
//!     cargo run --example hierarchy -- top.sv cpu.sv
//!
//! With no arguments it runs on a small built-in design.

use sv_lang::{Compilation, DefinitionKind, Session, Symbol, Walk, kinds::SymbolKind};

const DEMO: &str = "\
module alu #(parameter int WIDTH = 8) (input logic [WIDTH-1:0] a, b, output logic [WIDTH-1:0] y);
    assign y = a + b;
endmodule
module cpu;
    logic [7:0] x, z;
    alu #(.WIDTH(8)) alu0 (.a(x), .b(x), .y(z));
    alu #(.WIDTH(16)) alu1 (.a('0), .b('0), .y());
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
        print_instance(top, 0);
    }

    let diags = design.diagnostics();
    if diags.has_errors() {
        eprint!("{diags}");
    }
    Ok(())
}

fn print_instance(inst: Symbol<'_>, depth: usize) {
    let indent = "  ".repeat(depth);
    let def = inst.instance_definition();
    let kind = match def.and_then(|d| d.definition_kind()) {
        Some(DefinitionKind::Interface) => "interface",
        Some(DefinitionKind::Program) => "program",
        _ => "module",
    };
    let def_name = def.map(|d| d.name().to_string()).unwrap_or_default();
    println!("{indent}{} : {kind} {def_name}", inst.name());

    // Print the parameters of this instance.
    for p in inst.parameters() {
        if let Some(v) = p.parameter_value() {
            println!("{indent}  #{} = {v}", p.name());
        }
    }

    // Recurse into child instances (skip everything else in the body).
    if let Some(body) = inst.instance_body() {
        body.visit(|sym| {
            if sym == body {
                return Walk::Continue;
            }
            if sym.kind() == SymbolKind::Instance {
                print_instance(sym, depth + 1);
                Walk::Skip
            } else {
                Walk::Continue
            }
        });
    }
}
