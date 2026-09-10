# sv-lang-lsp

SystemVerilog **language-server intelligence** built on
[`sv-lang`](https://crates.io/crates/sv-lang) and its incremental workspace
[`sv-lang-db`](https://crates.io/crates/sv-lang-db).

The crate is two layers:

- **A transport-free library** (the default build) — pure functions that turn a
  file in a [`Workspace`] into editor-ready results, with a `LineIndex` that maps
  slang's byte offsets to LSP UTF-16 positions:
  - `diagnostics(ws, path)` — parse/lex/preprocess diagnostics with ranges.
  - `document_symbols(ws, path)` — the file's modules, interfaces and programs
    with their ranges, resolved through the elaborated design.
  - `hover(ws, path, pos)` — the name, kind and (for a value) type of the
    identifier under the cursor, as markdown; for an instance, the module it
    instantiates.
  - `definition(ws, path, pos)` — the declaration site of the symbol the
    identifier under the cursor names.
  - `references(ws, path, pos)` — every occurrence of that symbol in the file,
    declaration included. This is a name-based scan of the one file's CST (see
    the doc comment for the limitation); a production server would re-resolve
    each candidate and scan the whole workspace.

  These resolve identifiers by mapping the LSP position to a byte offset
  (`LineIndex::offset`, the inverse of `LineIndex::position`), finding the
  narrowest identifier node covering it, and matching it against the elaborated
  design's symbols by declaration range and name. Because it's just functions
  over the workspace, it is unit-tested directly.

- **A runnable server** behind the `server` feature — a `tower-lsp` binary that
  wires those functions to `textDocument/didOpen`, `didChange`,
  `documentSymbol`, `hover`, `definition` and `references`, republishing
  diagnostics on every edit through the incremental workspace (only the changed
  file reparses).

```console
$ cargo run -p sv-lang-lsp --features server        # speaks LSP over stdio
```

This is the showcase for the incremental workspace: real IDE features over
slang's real semantics, with edits reflected without recompiling the world.
