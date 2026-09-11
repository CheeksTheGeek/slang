//! Runs slang's real frontend — parse, elaborate, read types — entirely inside
//! a WebAssembly sandbox, with **no native slang and no C++ toolchain**. The
//! ~14 MB slang C ABI is compiled to `wasm32-wasip1`, bundled (2.8 MB gzip),
//! and driven through wasmtime. The sandbox is given a bounded memory and fuel
//! budget, so even a hostile input traps instead of hanging the host.
//!
//!     cargo run --example sandbox
//!
//! (Untrusted input? Use a fresh `Slang` per input — fuel is per instance.)

use sv_lang_wasm::{Limits, Slang};

const SRC: &str = "\
module datapath (input logic [7:0] a, b, input logic sub, output logic [8:0] result);
    logic [7:0] diff, sum;
    assign diff   = a - b;
    assign sum    = a + b;
    assign result = sub ? {1'b0, diff} : {1'b0, sum};
endmodule
";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A bounded sandbox: cap memory and instruction budget.
    let mut slang = Slang::with_limits(Limits {
        max_memory: Some(256 << 20), // 256 MiB
        fuel: Some(5_000_000_000),   // ~5e9 instructions
    })?;

    println!("slang is running inside a WebAssembly sandbox (no native library).\n");

    let tree = slang.parse(SRC)?;
    let design = slang.compile(&tree)?;

    for top in slang.top_instances(&design)? {
        println!("module {}:", slang.name(top)?);
        let Some(body) = slang.instance_body(top)? else {
            continue;
        };
        for m in slang.members(body)? {
            let name = slang.name(m)?;
            if let Some(ty) = slang.value_type(m)? {
                println!(
                    "  {:<8} {:<14} {} bits",
                    name,
                    slang.type_string(ty)?,
                    slang.type_bit_width(ty)?
                );
            }
        }
    }

    // `fork()` makes a fresh, isolated instance cheaply (the compiled module is
    // shared) — a clean sandbox per untrusted input.
    let mut worker = slang.fork()?;
    let tree = worker.parse("module a; endmodule\nmodule b; endmodule\n")?;
    let n = worker.module_count(&tree)?;
    println!("\na forked sandbox parsed {n} modules independently.");
    Ok(())
}
