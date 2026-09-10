# sv-lang

Safe, ergonomic Rust bindings to [slang](https://sv-lang.com) — the fast,
standards-compliant SystemVerilog compiler frontend.

`sv-lang` gives the Rust ecosystem the whole of slang: not just parsing, but the
elaborated semantic model — symbols, types, name resolution, constant
evaluation, and lint/driver analysis — that until now existed only behind
slang's C++ and Python APIs.

```rust
use sv_lang::{Session, Compilation, AnalysisFlags};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let session = Session::new();

// Parse: a lossless concrete syntax tree.
let tree = session.parse("module counter(input clk); logic [7:0] q; endmodule\n")?;
assert_eq!(tree.module_names().collect::<Vec<_>>(), ["counter"]);

// Elaborate: a fully-frozen, thread-shareable semantic design.
let mut comp = Compilation::new(&session)?;
comp.add(&tree)?;
let design = comp.compile()?;

let counter = design.top_instances().next().unwrap();
let q = counter.instance_body().unwrap().find("q").unwrap();
assert_eq!(q.value_type().unwrap().to_sv_string(), "logic[7:0]");

// Analyze: lints and driver tracking.
let analysis = design.analyze(AnalysisFlags::CHECK_UNUSED, 0)?;
for d in analysis.diagnostics().items() {
    println!("{d}");
}
# Ok(())
# }
```

## What you get

- **Parsing** into a lossless syntax tree, with byte-exact source round-trip.
- **Typed syntax** — a generated view struct for every one of slang's ~530
  syntax kinds, so `module.header().name()` is a method call, not an index.
- **A pure-Rust mirror** ([`sv-lang-syntax`](https://crates.io/crates/sv-lang-syntax))
  you can keep, edit and print after the session is gone — for formatters and
  editors that don't want slang linked in.
- **The semantic model** — `Compilation → Design`, with `Symbol`, `Type` and
  `Expression` handles, scope iteration and lookup, and constant values.
- **Analysis** — unused-code and related lints, and per-signal driver tracking.
- **Owned diagnostics** that render exactly as the `slang` binary does.

## Soundness

A `Design` is produced by freezing a fully-elaborated compilation, and is
`Send + Sync`: any number of threads may traverse it concurrently. The two
operations slang implements with a mutation — evaluating a not-yet-folded
constant, and full hierarchical name lookup — require exclusive access and are
reached through `&mut Design` / an `EvalSession`, so they cannot race. These
invariants are enforced by the type system, not merely documented (see the
crate's `trybuild` compile-fail suite).

## Building

By default `sv-lang` compiles slang from vendored sources with a C++ toolchain —
no CMake, Python or network required. To link a system slang instead, enable the
`system` feature and point `SLANG_C_LIB_DIR` / `SLANG_C_INCLUDE_DIR` at it. On
docs.rs the native build is skipped, so the API documentation always renders.

## Crates

| Crate | Native code? | Purpose |
|---|---|---|
| `sv-lang` | via `sv-lang-sys` | the safe API (this crate) — start here |
| `sv-lang-syntax` | no | the pure-Rust lossless tree |
| `sv-lang-kinds` | no | slang's kind enumerations |
| `sv-lang-sys` | yes | raw FFI to the slang C API |
| `sv-lang-db` | via `sv-lang` | incremental workspace: cache parses, recompile on change |
| `sv-lang-lsp` | via `sv-lang` | language-server intelligence (diagnostics, symbols, hover) |
| `sv-lang-wasm` | no (bundled wasm) | run slang in a WebAssembly sandbox, no C++ toolchain |

## License

MIT, the same as slang.
