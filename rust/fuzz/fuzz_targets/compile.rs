//! Fuzz elaboration: parse + compile must never panic on arbitrary input.
//! Run with `cargo +nightly fuzz run compile`.
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    let session = sv_lang::Session::new();
    if let Ok(mut comp) = sv_lang::Compilation::new(&session) {
        if comp.add_source(data).is_ok() {
            if let Ok(design) = comp.compile() {
                let _ = design.diagnostics().error_count();
            }
        }
    }
});
