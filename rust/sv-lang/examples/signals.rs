//! Prints a resolved-type report for every signal in a design: the *elaborated*
//! type of each variable, net and port — widths, signedness, 4-state, and the
//! members of enums, the fields (with bit offsets) of packed structs, and the
//! element type of arrays. This is the semantic type system, not the syntax:
//! `logic [W-1:0]` is reported as its concrete width after parameter folding.
//!
//!     cargo run --example signals -- your_design.sv
//!
//! With no arguments it runs on a small built-in design.

use sv_lang::{Compilation, Session, Symbol, Type, kinds::SymbolKind};

const DEMO: &str = "\
package types;
    typedef enum logic [1:0] { IDLE, BUSY, DONE } state_t;
    typedef struct packed { logic valid; logic [6:0] tag; } entry_t;
endpackage

module regfile #(parameter int AWIDTH = 4, parameter int DWIDTH = 32) (
    input  logic                 clk,
    input  logic signed [15:0]   offset,
    input  logic [AWIDTH-1:0]    addr,
    output logic [DWIDTH-1:0]    rdata
);
    import types::*;
    state_t          state;
    entry_t          head;
    logic [DWIDTH-1:0] mem [0:(1<<AWIDTH)-1];
    assign rdata = mem[addr];
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
        let def = top.instance_definition();
        let def_name = def.map(|d| d.name().to_string()).unwrap_or_default();
        println!("module {def_name} (instance {})", top.name());
        if let Some(body) = top.instance_body() {
            for m in body.members() {
                if matches!(
                    m.kind(),
                    SymbolKind::Variable | SymbolKind::Net | SymbolKind::Port
                ) {
                    print_signal(m);
                }
            }
        }
        println!();
    }
    Ok(())
}

fn print_signal(sym: Symbol<'_>) {
    let Some(ty) = sym.value_type() else { return };
    let mut attrs = vec![format!("{} bits", ty.bit_width())];
    if ty.is_signed() {
        attrs.push("signed".into());
    }
    attrs.push(
        if ty.is_four_state() {
            "4-state"
        } else {
            "2-state"
        }
        .into(),
    );
    println!(
        "  {:<10} {:<22} [{}]",
        sym.name(),
        ty.to_sv_string(),
        attrs.join(", ")
    );
    describe_type(ty, "    ");
}

/// Recursively unfolds the interesting structure of a type.
fn describe_type(ty: Type<'_>, indent: &str) {
    if ty.is_enum() {
        for member in ty.enum_members() {
            let val = member.enum_member_value().unwrap_or_default();
            println!("{indent}• {} = {val}", member.name());
        }
    } else if ty.is_struct() {
        for field in ty.fields() {
            let ft = field.value_type();
            let width = ft.map(|t| t.bit_width()).unwrap_or(0);
            let tystr = ft.map(|t| t.to_sv_string()).unwrap_or_default();
            println!(
                "{indent}• {:<8} {:<12} @ bit {} ({} wide)",
                field.name(),
                tystr,
                field.field_bit_offset(),
                width
            );
        }
    } else if ty.is_unpacked_array() {
        if let Some(elem) = ty.element_type() {
            println!(
                "{indent}• array of {} ({} bits each)",
                elem.to_sv_string(),
                elem.bit_width()
            );
        }
    }
}
