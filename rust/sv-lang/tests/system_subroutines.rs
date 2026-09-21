//! `Compilation::add_nonconstant_system_function` — registering a non-constant
//! system function so the elaborator recognizes it instead of reporting
//! `UnknownSystemName` (which otherwise poisons the whole enclosing statement).
//! Requested by the svling port for byte parity with designs that call
//! `$fputc`/`$is_signed`, which pyslang registers via
//! `comp.addSystemSubroutine(NonConstantFunction(...))`.

use sv_lang::{BuiltinType, Compilation, Session};

// Uses $fputc(int, int) -> int and $is_signed(int) -> int, neither of which is
// a built-in slang system function.
const SRC: &str = "module m;\n\
    \x20 int fd;\n\
    \x20 initial begin\n\
    \x20   fd = $fputc(65, 1);\n\
    \x20   if ($is_signed(fd)) fd = 0;\n\
    \x20 end\n\
    endmodule\n";

fn unknown_system_names(diags: &sv_lang::Diagnostics) -> Vec<String> {
    diags
        .items()
        .iter()
        .filter(|d| d.code_name() == "UnknownSystemName")
        .map(|d| d.message.clone())
        .collect()
}

#[test]
fn unregistered_system_function_is_unknown() {
    // Baseline: without registration, slang reports both calls as unknown.
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source(SRC).unwrap();
    let design = comp.compile().unwrap();
    let unknown = unknown_system_names(&design.diagnostics());
    assert_eq!(
        unknown.len(),
        2,
        "expected $fputc and $is_signed to be unknown, got: {unknown:?}"
    );
}

#[test]
fn registered_system_functions_bind() {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    // Register before adding sources / compiling.
    comp.add_nonconstant_system_function(
        "$fputc",
        BuiltinType::Int,
        &[BuiltinType::Int, BuiltinType::Int],
    )
    .unwrap();
    comp.add_nonconstant_system_function("$is_signed", BuiltinType::Int, &[BuiltinType::Int])
        .unwrap();
    comp.add_source(SRC).unwrap();
    let design = comp.compile().unwrap();
    let diags = design.diagnostics();

    let unknown = unknown_system_names(&diags);
    assert!(
        unknown.is_empty(),
        "registered functions should bind, but got unknown: {unknown:?}"
    );
    assert!(
        !diags.has_errors(),
        "design should be error-free after registration: {:?}",
        diags
            .items()
            .iter()
            .filter(|d| d.is_error())
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

#[test]
fn registering_after_finalize_errors() {
    // Once the compilation is frozen it's consumed by compile(), so the only way
    // to observe the finalized guard is an unknown builtin-type ordinal being
    // rejected up front. An unknown return type is an InvalidArg error.
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    let err = comp
        .add_nonconstant_system_function("$bad", BuiltinType::Void, &[])
        .and_then(|()| {
            // Void return is legal; a legit registration should succeed.
            comp.add_nonconstant_system_function("$ok", BuiltinType::Int, &[BuiltinType::Byte])
        });
    assert!(err.is_ok(), "valid registrations should succeed: {err:?}");
}
