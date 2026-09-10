# Fuzzing sv-lang

These targets use [`cargo fuzz`](https://github.com/rust-fuzz/cargo-fuzz) (which
needs a nightly toolchain and libFuzzer):

```
cargo install cargo-fuzz
cargo +nightly fuzz run parse     # parser: no panic, tree round-trips
cargo +nightly fuzz run compile   # parse + elaborate: no panic
```

`sv-lang-sys` builds slang with assertions on, so a violated internal
invariant aborts rather than corrupting memory — exactly what a fuzzer should
catch. The `parse` corpus round-trip is also checked deterministically, without
nightly, by `sv-lang/tests/robustness.rs`.
