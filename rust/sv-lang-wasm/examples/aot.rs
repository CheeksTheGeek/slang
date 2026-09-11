//! Ahead-of-time precompilation for instant cold starts (serverless / read-only
//! deployments, where the on-disk cache never persists between runs).
//!
//!   cargo run --release --example aot -- gen slang.cwasm   # once, at build time
//!   cargo run --release --example aot -- run slang.cwasm   # every cold start
//!
//! In a real project the `gen` step lives in your `build.rs` and the bytes are
//! embedded with `include_bytes!`; here we use a file so the two phases can run
//! as separate processes (a true cold start).

use std::time::Instant;

use sv_lang_wasm::{Limits, Slang};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mode, path) = match args.as_slice() {
        [mode, path] => (mode.as_str(), path.clone()),
        _ => {
            eprintln!("usage: aot <gen|run> <path.cwasm>");
            std::process::exit(2);
        }
    };

    match mode {
        // Build-time: compile the module once and save portable native code.
        "gen" => {
            let t = Instant::now();
            let bytes = Slang::precompile()?;
            std::fs::write(&path, &bytes)?;
            eprintln!(
                "precompiled {} bytes in {:?} (do this once, at build time)",
                bytes.len(),
                t.elapsed()
            );
        }
        // Runtime cold start: load native code directly — no compile, no cache.
        "run" => {
            let bytes = std::fs::read(&path)?;
            let t = Instant::now();
            // SAFETY: `bytes` are this crate's own `precompile()` output.
            let mut slang = unsafe { Slang::from_precompiled(&bytes, Limits::default())? };
            let cold = t.elapsed();
            let tree = slang.parse("module top; logic [7:0] q; endmodule\n")?;
            let n = slang.module_count(&tree)?;
            println!("cold start from precompiled native code: {cold:?}  (parsed {n} module)");
        }
        _ => {
            eprintln!("unknown mode {mode:?}; expected `gen` or `run`");
            std::process::exit(2);
        }
    }
    Ok(())
}
