# sv-lang round-trip corpus

Real-world SystemVerilog for the byte-exact round-trip + no-crash test in
`sv-lang/tests/corpus.rs`, so the bindings are exercised on genuine RTL, not only
slang's own small unit-test files.

## Vendored (always-on)

- `ibex/` — the `rtl/` sources of the [lowRISC Ibex](https://github.com/lowRISC/ibex)
  RISC-V core (33 files, ~1.1 MB). Apache-2.0 — see `ibex/LICENSE`. Copyright
  lowRISC contributors, ETH Zurich / University of Bologna, and Microsoft.
  Unmodified; used only as parser input.

`corpus.rs` parses each file, checks its tree text reproduces the source
byte-for-byte, and that its pure-Rust mirror matches — over this whole slice on
every test run.

## Full designs (opt-in)

`scripts/fetch-corpus.sh` clones the full ibex / OpenTitan / CVA6 trees into a
scratch directory and points `SV_LANG_CORPUS` at them, so `corpus.rs`'s
`external_corpus_round_trips` runs the same checks over ~100k+ lines of real
designs. Nothing large is committed for that path.
