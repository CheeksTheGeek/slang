//! Fuzz the parser: it must never panic, and any tree it produces must
//! round-trip to its own source text. Run with `cargo +nightly fuzz run parse`.
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    let session = sv_lang::Session::new();
    if let Ok(tree) = session.parse(data) {
        assert_eq!(tree.text(), data);
        let _ = tree.mirror();
    }
});
