# sv-lang-wasm

Run [slang](https://sv-lang.com) — the SystemVerilog compiler frontend — inside a
**WebAssembly sandbox**, with no native slang and no C++ toolchain on the host.

The whole compiler is compiled to `wasm32-wasip1` and embedded in this crate
(gzip-compressed, ~2.8 MB); [`wasmtime`](https://crates.io/crates/wasmtime) loads
and runs it, and the crate marshals arguments and results across the wasm
boundary through the guest's 32-bit linear memory.

## Startup

The ~14 MB module is expensive to JIT-compile — seconds in a release build,
much longer in a debug build. wasmtime's on-disk compilation cache is enabled
by default, so that cost is paid **once, ever**: the compiled native code is
content-addressed and cached across processes, so the second run and every run
afterward start in milliseconds (`Slang::new()` ≈ 0.1 s vs ≈ 7 s cold on a
release build). The cache is best-effort — if no cache directory is available it
is silently skipped and the module is compiled each run.

```rust
let mut slang = sv_lang_wasm::Slang::new()?;
assert!(slang.version().starts_with("11."));

let tree = slang.parse("module top; endmodule\n")?;
assert_eq!(slang.root_kind_name(&tree)?, "CompilationUnit");
assert_eq!(slang.module_count(&tree)?, 1);
# Ok::<(), sv_lang_wasm::Error>(())
```

## Why

This is the `backend-wasm` half of the slang Rust bindings. The native
[`sv-lang`](https://crates.io/crates/sv-lang) crate links slang's C ABI directly;
this crate reaches the **same C ABI** across a wasm boundary instead. That buys:

- **No C++ toolchain** to `cargo add` and run — the compiler is a data file.
- **Sandboxing** — untrusted SystemVerilog is parsed inside wasmtime's memory
  isolation, with **bounded memory and run time**: `Slang::with_limits` caps
  linear memory and gives the guest a fuel budget, so a pathological input traps
  (an `Err`) instead of hanging the host thread or growing to the 4 GiB ceiling.
  A trap, hang-guard, or OOM is contained to the instance, not the host process.

The trade-offs are the usual ones for wasm: ~2× slower than native, a 32-bit
4 GiB memory ceiling, and no threads (the module is built `SLANG_USE_THREADS=OFF`).

## How the module is built

`wasm/build.sh` rebuilds `wasm/slang_c.wasm.gz` from a checkout of slang plus a
[wasi-sdk](https://github.com/WebAssembly/wasi-sdk): it configures slang with the
`wasi-sdk-p1` toolchain (`-DSLANG_INCLUDE_CAPI=ON -DSLANG_USE_THREADS=OFF`),
builds the `slang_c` static library, links it into a wasm **reactor** module that
exports every `slang_*` C API function, and gzips the result.

## Status

This crate drives slang's real C ABI end-to-end inside the sandbox, with the
resource limits above enforced. Reachable today:

- **Syntax** — parse, root/kind reflection, syntax-tree walking.
- **Semantics** — `compile` (elaborate + freeze), top-level instances, scope
  members, name lookup, symbol names/kinds, declared types (`bit_width`,
  `type_string`), and the elaborated **behavioral tree**: a procedural block's
  root statement and its statement/expression children, with binary operators.
- **Custom dataflow across the boundary** — `reaching_writes` drives slang's
  DataFlow/Lattice bridge *inside* the guest with the lattice logic on the host:
  slang runs the analysis in wasm and calls back out through an `env.dfa_dispatch`
  import (a guest trampoline, `wasm/dfa_shim.c`) to a Rust lattice. This is the
  hard case — a **guest→host callback** — working end-to-end.

All of it runs over the same `slang_*` C ABI the native
[`sv-lang`](https://crates.io/crates/sv-lang) crate links directly — the
`wasm/build.sh` reactor exports every function, and the crate marshals them
through a small, uniform helper layer (indirect struct args, sret returns, guest
allocation, and the callback trampoline).

The **entire C surface is already reachable** through the generated `raw_*`
bridge — `xtask gen_wasm_bridge` emits a marshalling wrapper for every function
in the C ABI (only callbacks and struct out-parameters are hand-written), so the
raw layer never lags the native backend. The ergonomic high-level `Slang` methods
cover the common path and grow as needed on top of that bridge; a generic
host-lattice API (beyond the built-in reaching-writes lattice) is the main
remaining ergonomic extension.
