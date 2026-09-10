# sv-lang-db

An incremental workspace over [`sv-lang`](https://crates.io/crates/sv-lang):
hold a set of SystemVerilog source files, edit them, and get parse trees and an
elaborated `Design` back — re-parsing only what changed.

This is the building block for an editor integration or a language server: on
each keystroke you `set_file` the changed buffer and ask for a fresh `design`,
and only the edited file is re-parsed (slang syntax trees are safe to reuse
across compilations, so unchanged files cost nothing).

```rust
use sv_lang_db::Workspace;

let mut ws = Workspace::new();
ws.set_file("pkg.sv", "package p; localparam int W = 8; endpackage\n");
ws.set_file("top.sv", "module top; import p::*; logic [W-1:0] a; endmodule\n");

let design = ws.design()?;
for top in design.top_instances() {
    println!("{}", top.name());
}
# Ok::<(), sv_lang::Error>(())
```

The current implementation is a content-addressed query cache. It intentionally
does not pull in a framework like [salsa](https://crates.io/crates/salsa); the
same `Workspace` API can be re-implemented on salsa later without changing
callers.
