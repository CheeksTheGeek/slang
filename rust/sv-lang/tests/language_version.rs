//! Parsing under a language version: `Session::parse*` always passed NULL
//! options, so trees parsed as 1800-2017 even when the compilation used
//! 1800-2023, and `covergroup ... extends` (LRM 19.4.1, 2023-only) failed to
//! parse. Reported by the sv-sim session.

use sv_lang::{Compilation, LanguageVersion, Options, Session};

const SRC: &str = "\
class C;
  bit x;
  covergroup cg; coverpoint x; endgroup
  function new; cg = new; endfunction
endclass
class D extends C;
  covergroup extends cg; endgroup
  function new; super.new(); endfunction
endclass
module top; D d = new; endmodule
";

fn diag_names(d: &sv_lang::Diagnostics) -> Vec<&'static str> {
    d.iter()
        .filter(|d| d.is_error())
        .map(|d| d.code_name())
        .collect()
}

#[test]
fn default_session_rejects_2023_syntax() {
    let tree = Session::new().parse(SRC).unwrap();
    assert!(
        diag_names(&tree.diagnostics()).contains(&"WrongLanguageVersion"),
        "{}",
        tree.diagnostics()
    );
}

#[test]
fn session_version_reaches_parser_and_compilation() {
    let session = Session::new().with_language_version(LanguageVersion::V1800_2023);
    let tree = session.parse_named(SRC, "cg.sv", "").unwrap();
    assert!(!tree.diagnostics().has_errors(), "{}", tree.diagnostics());

    let opts = Options::new().with_language_version(LanguageVersion::V1800_2023);
    let mut comp = Compilation::new_with(&session, opts).unwrap();
    comp.add(&tree).unwrap();
    let design = comp.compile().unwrap();
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    assert_eq!(design.options().language_version, 2);
}

#[test]
fn add_source_inherits_the_session_version() {
    let session = Session::new().with_language_version(LanguageVersion::V1800_2023);
    let opts = Options::new().with_language_version(LanguageVersion::V1800_2023);
    let mut comp = Compilation::new_with(&session, opts).unwrap();
    comp.add_source(SRC).unwrap();
    let design = comp.compile().unwrap();
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
}

#[test]
fn parse_file_honours_the_version() {
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("lv_cg_extends.sv");
    std::fs::write(&path, SRC).unwrap();
    let tree = Session::new()
        .with_language_version(LanguageVersion::V1800_2023)
        .parse_file(&path)
        .unwrap();
    assert!(!tree.diagnostics().has_errors(), "{}", tree.diagnostics());
}
