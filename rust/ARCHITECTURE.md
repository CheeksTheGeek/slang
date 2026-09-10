# Architecture of the slang Rust bindings

This workspace binds [slang](https://sv-lang.com), a C++ SystemVerilog compiler
frontend, to Rust. It is layered so that each crate has one job and the crates
above it never see the layer below.

```
┌────────────────────────────────────────────────────────────────────┐
│ sv-lang-db      incremental workspace (cache parses, recompile)      │
├────────────────────────────────────────────────────────────────────┤
│ sv-lang         the safe API: Session · SyntaxTree · Node/Token      │
│                 · Compilation → Design · Symbol/Type/Expression       │
│                 · Analysis · visitors · SyntaxEditor                  │
│   ├── sv-lang-syntax   pure-Rust lossless tree (cstree) + editor      │
│   ├── sv-lang-kinds    slang's kind enumerations (no_std, no deps)    │
│   └── sv-lang-sys      raw FFI to the slang C API                     │
├────────────────────────────────────────────────────────────────────┤
│ slang-c         a C ABI over slang (include/slang/c, source/capi)    │
├────────────────────────────────────────────────────────────────────┤
│ slang (C++)                                                          │
└────────────────────────────────────────────────────────────────────┘
```

## The contract is a C ABI, not C++

Rust talks to slang only through **`slang-c`** — a hand-written C header
(`include/slang/c/slang.h`) with a C++ implementation (`source/capi/`). This is
the load-bearing decision: a C ABI is stable across compilers and slang
releases, where a `cxx`/`bindgen` binding over slang's C++ would break on every
template or layout change. The header follows five promises spelled out in its
own preamble — opaque owners with explicit destructors, value cursors for
positions, in/out error records (no ambient "last error"), nothing unwinds
across the boundary, and append-only versioning.

`slang-c` is generated where it can be (the ~530 syntax kinds, their struct
membership and member layouts, and the 1,255 diagnostics come from slang's own
schema via `scripts/*_gen.py --c-api`) and hand-written where the surface is
small and stable. It has an in-tree consumer, `tools/hier`, so it cannot rot.

## Kinds are data

`sv-lang-kinds` is pure Rust with no dependencies and no native code: just the
enumerations (`SyntaxKind`, `TokenKind`, `SymbolKind`, `DiagCode`, …) generated
from slang's schema, with discriminants matching the C++ enums exactly. Tools
that only need to *name* kinds depend on this without linking anything. Every
generated file is committed and checked by `cargo xtask codegen --check`, so
docs.rs (which has no network, CMake or Python) builds it unchanged.

## Two trees

- **The borrowed CST.** `sv-lang::SyntaxTree` holds slang's parse result; a
  `Node<'t>` / `Token<'t>` is a `Copy` cursor that borrows the tree. Over these,
  435 generated typed views (`ModuleDeclarationSyntax`, …) turn
  `module.header().name()` into method calls. This is zero-copy and fast, but
  tied to the tree's lifetime.
- **The owned mirror.** `SyntaxTree::mirror()` streams the tree through one C
  call into a pure-Rust `sv-lang-syntax` tree (a `cstree` green tree keyed by
  slang's kinds). It outlives the session, supports the `SyntaxEditor`, and
  needs no slang linked — the right representation for formatters, codemods and
  long-lived editor state.

## Soundness: `Design` is `Send + Sync`, earned

The interesting hazard is slang's laziness. A `Compilation` elaborates scopes
and folds constants on demand, mutating a shared arena from logically-const
paths with no atomics. So `sv-lang` never shares a live compilation. Instead
`Compilation::compile()` **freezes** it: elaborate every scope (instance caching
disabled so nothing is skipped), constant-fold every expression, then seal the
arena. The result, `Design`, is `Send + Sync` — every read accessor is a genuine
read of the frozen arena, proven by a test that traverses one design from many
threads at once.

The two operations slang still implements with a mutation — evaluating a
not-yet-folded constant, and full hierarchical name lookup (which records
lint state) — are reachable only through `&mut Design` / an `EvalSession`
(`Send` but `!Sync`), so they cannot race. These invariants are enforced by the
type system; the `trybuild` compile-fail suite proves a handle can't escape its
owner and that the exclusive types really are `!Sync`.

slang is always built with internal assertions on (in every profile), because
those assertions are what make the freeze contract observable — that is how the
seal catches an accidental late allocation instead of silently corrupting the
arena.

## Code generation

`rust/xtask` reads three JSON models (`rust/model/*.json`, produced by
`scripts/*_gen.py --emit-model`) and generates:

- `sv-lang-kinds/src/generated/*.rs` — the kind enums.
- `sv-lang/src/syntax/generated/nodes_*.rs` — the 435 typed node views.

`cargo xtask codegen` writes them; `--check` fails CI if a committed file is
stale. The generated Rust is committed so downstream builds (and docs.rs) need
no Python.

## Build

By default `sv-lang-sys` compiles slang and `slang-c` from source with the `cc`
crate — no CMake, no Python, no network — vendoring slang's header-only
dependencies. In a checkout of the slang repository it uses the in-tree sources
directly (running the generators); a published crate carries a pre-generated
`vendor/` snapshot. `DOCS_RS` skips the native build so the API docs always
render, and the `system` feature links a prebuilt `slang-c` instead.

## Deliberate decisions & non-goals

These are settled architectural choices, not gaps:

- **wasm backend targets `wasm32-wasip1`, not the wasip2 component model.** wasip1 is a
  classic core module importing `wasi_snapshot_preview1`, which a plain `wasmtime::Linker`
  hosts directly and every wasmtime version supports. wasip2 imports the component-model
  `wasi:*` world — harder to host, and still undertested upstream (the `wasi-sdk-p2` preset is
  PR-8's whole point). The public Rust API is identical either way, so wasip2 is a future
  swap behind `sv-lang-wasm`, gated on upstream maturity — not a change users would see.
- **The compiled wasm module is process-global** (`shared_engine_module`): JIT-compiled once,
  then every `Slang::new`/`Slang::fork` builds only a fresh store + instance. `fork()` is the
  intended way to get isolated parallel workers cheaply.
- **No `CancelToken` for in-flight analysis.** slang's `Compilation` and `AnalysisManager`
  expose no cancellation or abort hook — `analyze()` is one synchronous C++ call with no poll
  point — so cooperative cancellation is not possible without patching slang core, and we do
  not ship an API that pretends to cancel work it cannot stop. Bound runaway inputs with the
  wasm backend's fuel/memory limits (which *can* interrupt), or with slang's own
  `maxCaseAnalysisSteps` / `maxLoopAnalysisSteps`. If upstream adds a cancellation hook, a
  `CancelToken` becomes a thin wrapper over it.
