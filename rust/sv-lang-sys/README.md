# sv-lang-sys

Raw, `unsafe` FFI bindings to **slang-c**, the stable C API of
[slang](https://sv-lang.com), the SystemVerilog compiler frontend.

You almost certainly want the safe [`sv-lang`](https://crates.io/crates/sv-lang)
crate instead; this crate exists so that `sv-lang` (and anyone else) can link
slang without a C++ build system.

## What it does

`build.rs` compiles slang and its C API from source with the `cc` crate — no
CMake, no Python, no network — and links it statically. The sources are
vendored into the published crate; when building from a checkout of the slang
repository the in-tree sources are used directly (and Python 3 is required to
run slang's code generators).

| Situation | What happens |
|---|---|
| `DOCS_RS` set | Nothing is compiled; only the declarations are documented. |
| feature `system` | Link a prebuilt `slang-c` via `pkg-config`, or `SLANG_C_LIB_DIR` + `SLANG_C_INCLUDE_DIR`. The library's syntax model hash must match this crate's. |
| default | Compile the vendored (or in-tree) sources. |

The library is always built with slang's internal assertions enabled, because
the safe layer's thread-safety guarantees depend on them.

## Version

`0.x.y+slang.M.N` — the crate has its own semver; the slang version it wraps
appears only in the build metadata. The C API itself is versioned separately
(`slang_c_version()`), and this crate is generated for exactly one C API
major version.
