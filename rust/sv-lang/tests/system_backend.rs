//! `system`-feature integration test: link a **prebuilt external** slang-c and
//! exercise the startup ABI/model-hash probe against it.
//!
//! Under the default vendored build these assertions are tautologies. Their
//! purpose is to be run under `--features system` against a real external
//! library:
//!
//! ```console
//! $ SLANG_C_LIB_DIR=/path/to/libs \
//!     cargo test -p sv-lang --features system --test system_backend
//! ```
//!
//! A pass then proves two things at once: the `system` link path in
//! `sv-lang-sys/build.rs` resolves and links the external `libslang-c` +
//! `libsvlang`, and the startup probe (`verify_abi`, run by `Session::new`)
//! fired against that library — it panics on an assertions-off or
//! model-mismatched library, so reaching these asserts means it passed.

#[test]
fn linked_library_passes_the_abi_probe() {
    // The three facts the ABI probe enforces, asserted explicitly.
    assert!(
        sv_lang::slang_has_assertions(),
        "linked slang-c must be an assertions-on build (the seal enforcement \
         behind Design: Sync)"
    );
    assert_eq!(
        sv_lang::slang_syntax_model_hash(),
        sv_lang::kinds::SYNTAX_MODEL_HASH,
        "linked slang-c syntax-model hash must match the generated kind tables"
    );
    let v = sv_lang::slang_version();
    assert!(v.starts_with("11."), "unexpected slang version: {v:?}");
}

#[test]
fn end_to_end_through_the_linked_library() {
    // `Session::new` runs the ABI probe; the rest drives the external library
    // through the full pipeline — parse, elaborate, symbol lookup, type read.
    let session = sv_lang::Session::new();
    let tree = session
        .parse("module m; logic [7:0] a, b; wire [7:0] s = a ^ b; endmodule\n")
        .expect("parse via external slang-c");
    let mut comp = sv_lang::Compilation::new(&session).expect("compilation");
    comp.add(&tree).expect("add tree");
    let design = comp.compile().expect("elaborate");

    let top = design.top_instances().next().expect("a top instance");
    let body = top.instance_body().expect("instance body");
    let s = body.find("s").expect("net s exists");
    assert_eq!(s.name(), "s");
    let ty = s.value_type().expect("s has a type");
    assert_eq!(ty.bit_width(), 8);
    assert!(
        ty.to_sv_string().contains("logic"),
        "type = {}",
        ty.to_sv_string()
    );
}
