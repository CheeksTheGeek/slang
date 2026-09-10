//! A tiny SystemVerilog linter: elaborates the input and reports slang's
//! unused-code, multi-driver and shadow lints.
//!
//!     cargo run --example lint -- design.sv
//!
//! Exits non-zero if any error-level diagnostic is found.

use std::process::ExitCode;

use sv_lang::{AnalysisFlags, Compilation, Session};

const DEMO: &str = "\
module m(input logic clk, output logic o);
    logic unused;         // never read
    logic shadowed;
    logic driven;
    assign o = driven;
    always_ff @(posedge clk) driven <= 1'b1;
endmodule
";

fn main() -> ExitCode {
    let session = Session::new();
    let mut comp = match Compilation::new(&session) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let args: Vec<String> = std::env::args().skip(1).collect();
    let load = |comp: &mut Compilation| -> Result<(), sv_lang::Error> {
        if args.is_empty() {
            comp.add_source(DEMO)
        } else {
            for path in &args {
                comp.add(&session.parse_file(path)?)?;
            }
            Ok(())
        }
    };
    if let Err(e) = load(&mut comp) {
        eprintln!("error: {e}");
        return ExitCode::FAILURE;
    }

    let design = match comp.compile() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    // Parse + semantic diagnostics.
    let sema = design.diagnostics();
    print!("{sema}");

    // Analysis lints.
    let flags = AnalysisFlags::CHECK_UNUSED | AnalysisFlags::CHECK_SHADOW;
    match design.analyze(flags, 0) {
        Ok(analysis) => print!("{}", analysis.diagnostics()),
        Err(e) => eprintln!("analysis error: {e}"),
    }

    if sema.has_errors() {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
