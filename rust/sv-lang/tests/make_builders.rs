//! `sv_lang_syntax::make` structured builders, verified against the real
//! parser: a hand-built tree must have the same node/token skeleton as the one
//! slang produces for the equivalent source, and must print the exact canonical
//! text. (Trivia *attachment* — which token a newline hangs off — is a
//! formatting detail that does not affect the typed structure, so the skeleton
//! comparison drops trivia; the separate text assertion pins the formatting.)

use sv_lang_syntax::make;

/// The pretty-printed tree with `@range` suffixes removed and trivia lines
/// dropped — i.e. the node/token/absent structure, position- and
/// formatting-independent.
fn skeleton(debug: &str) -> Vec<String> {
    debug
        .lines()
        .filter(|l| !l.trim_start().starts_with("Trivia("))
        .map(|line| match line.find('@') {
            Some(at) => {
                let head = &line[..at];
                // Keep a token's ` "text"` payload; drop the `@start..end` range.
                match line[at..].find(" \"") {
                    Some(q) => format!("{head}{}", &line[at + q..]),
                    None => head.to_string(),
                }
            }
            None => line.to_string(),
        })
        .collect()
}

fn parser_skeleton(src: &str) -> Vec<String> {
    let session = sv_lang::Session::new();
    let tree = session.parse(src).expect("parse");
    skeleton(&format!("{:#?}", tree.mirror()))
}

#[test]
fn empty_module_matches_parser() {
    let built = make::empty_module("m");
    assert_eq!(built.text().to_string(), "module m;\nendmodule\n");
    assert_eq!(
        skeleton(&format!("{built:#?}")),
        parser_skeleton("module m;\nendmodule\n"),
        "make::empty_module skeleton differs from the parser's"
    );
}

#[test]
fn named_module_matches_parser() {
    let built = make::empty_module("counter");
    let src = "module counter;\nendmodule\n";
    assert_eq!(built.text().to_string(), src);
    assert_eq!(skeleton(&format!("{built:#?}")), parser_skeleton(src));
}

#[test]
fn two_modules_match_parser() {
    let built = make::compilation_unit(|b| {
        make::module(b, "a", |_| {});
        make::module(b, "b", |_| {});
    });
    let src = "module a;\nendmodule\nmodule b;\nendmodule\n";
    assert_eq!(built.text().to_string(), src);
    assert_eq!(skeleton(&format!("{built:#?}")), parser_skeleton(src));
}

#[test]
fn built_tree_reparses_identically() {
    // The strongest check: the text a builder emits, fed back through the
    // parser, yields the same skeleton the builder produced.
    let built = make::empty_module("top");
    let built_skel = skeleton(&format!("{built:#?}"));
    let reparsed = parser_skeleton(&built.text().to_string());
    assert_eq!(built_skel, reparsed);
}

#[test]
fn logic_vector_declaration_matches_parser() {
    let built = make::compilation_unit(|b| {
        make::module(b, "m", |b| make::logic(b, "x", Some((7, 0))));
    });
    let src = "module m;\n    logic [7:0] x;\nendmodule\n";
    assert_eq!(built.text().to_string(), src);
    assert_eq!(skeleton(&format!("{built:#?}")), parser_skeleton(src));
}

#[test]
fn logic_scalar_declaration_matches_parser() {
    let built = make::compilation_unit(|b| {
        make::module(b, "m", |b| make::logic(b, "en", None));
    });
    let src = "module m;\n    logic en;\nendmodule\n";
    assert_eq!(built.text().to_string(), src);
    assert_eq!(skeleton(&format!("{built:#?}")), parser_skeleton(src));
}

#[test]
fn module_with_several_declarations_matches_parser() {
    let built = make::compilation_unit(|b| {
        make::module(b, "regs", |b| {
            make::logic(b, "bus", Some((31, 0)));
            make::logic(b, "flag", None);
        });
    });
    let src = "module regs;\n    logic [31:0] bus;\n    logic flag;\nendmodule\n";
    assert_eq!(built.text().to_string(), src);
    assert_eq!(skeleton(&format!("{built:#?}")), parser_skeleton(src));
}
