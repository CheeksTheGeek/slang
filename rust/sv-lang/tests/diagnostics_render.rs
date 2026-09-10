//! Diagnostic span capture and the optional miette / ariadne renderers.

use sv_lang::Session;

const SRC: &str = "module m;\n  int x = ;\nendmodule\n";

#[test]
fn diagnostics_carry_source_and_span() {
    let session = Session::new();
    let tree = session.parse(SRC).unwrap();
    let diags = tree.diagnostics();
    let d = diags
        .items()
        .iter()
        .find(|d| d.is_error())
        .expect("an error");

    // The span points into the captured source text. slang appends a sentinel
    // to buffer text, so the source starts with (but may be longer than) SRC.
    let src = d.source.as_ref().expect("source captured");
    assert!(src.starts_with("module m;"), "{src:?}");
    assert!(src.contains("int x"));
    assert!(d.byte_len >= 1);
    assert!(d.byte_offset + d.byte_len <= src.len());
    // The error lands on line 2 (the bad `int x = ;`).
    assert_eq!(d.line, 2);

    // Every error carries a usable span into its source.
    for d in diags.items().iter().filter(|d| d.is_error()) {
        assert!(d.source.is_some());
        assert!(!src[d.span()].is_empty());
    }
}

#[cfg(feature = "miette")]
#[test]
fn miette_renders_with_source() {
    let session = Session::new();
    let tree = session.parse(SRC).unwrap();
    let d = tree
        .diagnostics()
        .into_items()
        .into_iter()
        .find(|d| d.is_error())
        .unwrap();

    // The miette Diagnostic impl exposes code, severity, source and a label.
    use miette::Diagnostic as _;
    assert!(d.code().is_some());
    assert!(d.source_code().is_some());
    assert!(d.labels().is_some());

    // Rendering through a miette handler produces source-annotated output.
    let report = d.into_miette();
    let rendered = format!("{report:?}");
    assert!(rendered.contains("expected expression"));
}

#[cfg(feature = "ariadne")]
#[test]
fn ariadne_renders_with_source() {
    let session = Session::new();
    let tree = session.parse(SRC).unwrap();
    let d = tree
        .diagnostics()
        .into_items()
        .into_iter()
        .find(|d| d.is_error())
        .unwrap();
    let out = d.to_ariadne_string();
    // The message is rendered plainly; the source snippet is colorized
    // per-character, so check the message and the location header.
    assert!(out.contains("expected expression"), "{out}");
    assert!(out.contains("source:2"), "{out}");
    assert!(!out.is_empty());
}
