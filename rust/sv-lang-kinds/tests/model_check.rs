//! Cross-checks the generated tables against the vendored JSON models they
//! were generated from. If `cargo xtask codegen` was skipped after a model
//! update, or the hand-maintained `DiagSubsystem` list in xtask drifts from
//! slang's `Diagnostics.h`, these tests fail.

use serde_json::Value;
use sv_lang_kinds::*;

fn load(name: &str) -> Value {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../model/");
    let text = std::fs::read_to_string(format!("{path}{name}-{SLANG_VERSION}.json"))
        .expect("vendored model present");
    serde_json::from_str(&text).expect("valid JSON")
}

#[test]
fn syntax_kinds_match_model() {
    let m = load("syntax-model");
    assert_eq!(
        m["modelHash"].as_str().unwrap(),
        SYNTAX_MODEL_HASH,
        "regenerate with `cargo xtask codegen`"
    );

    let kinds = m["syntaxKinds"].as_array().unwrap();
    assert_eq!(kinds.len(), SyntaxKind::COUNT as usize);
    for (i, k) in kinds.iter().enumerate() {
        let kind = SyntaxKind::from_raw(i as u16).unwrap();
        assert_eq!(kind.name(), k.as_str().unwrap());
        assert_eq!(kind.as_raw(), i as u16);
    }
    assert_eq!(SyntaxKind::from_raw(SyntaxKind::COUNT), None);
    assert_eq!(SyntaxKind::Unknown.syntax_struct(), None);

    let structs = m["structs"].as_array().unwrap();
    assert_eq!(structs.len(), SyntaxStruct::COUNT as usize);
    let kind_to_struct = m["kindToStruct"].as_object().unwrap();
    for kind in SyntaxKind::ALL.iter().skip(1) {
        let expected = kind_to_struct[kind.name()].as_str().unwrap();
        assert_eq!(kind.syntax_struct().unwrap().name(), expected, "{kind}");
    }
}

#[test]
fn token_and_trivia_kinds_match_model() {
    let m = load("syntax-model");
    let toks = m["tokenKinds"].as_array().unwrap();
    assert_eq!(toks.len(), TokenKind::COUNT as usize);
    for (i, t) in toks.iter().enumerate() {
        assert_eq!(
            TokenKind::from_raw(i as u16).unwrap().name(),
            t.as_str().unwrap()
        );
    }
    let triv = m["triviaKinds"].as_array().unwrap();
    assert_eq!(triv.len(), TriviaKind::COUNT as usize);
    for (i, t) in triv.iter().enumerate() {
        assert_eq!(
            TriviaKind::from_raw(i as u8).unwrap().name(),
            t.as_str().unwrap()
        );
    }
}

#[test]
fn diagnostics_match_model() {
    let m = load("diagnostics-model");
    assert_eq!(
        m["modelHash"].as_str().unwrap(),
        DIAGNOSTICS_MODEL_HASH,
        "regenerate with `cargo xtask codegen`"
    );

    let mut total = 0;
    for sub in m["subsystems"].as_array().unwrap() {
        let name = sub["name"].as_str().unwrap();
        let subsystem = DiagSubsystem::ALL
            .iter()
            .copied()
            .find(|s| s.name() == name)
            .unwrap_or_else(|| {
                panic!("subsystem {name} missing from xtask's DIAG_SUBSYSTEMS list")
            });
        for d in sub["diagnostics"].as_array().unwrap() {
            let code = DiagCode::new(subsystem, d["index"].as_u64().unwrap() as u16);
            assert_eq!(code.name(), d["name"].as_str().unwrap());
            assert_eq!(code.message_format(), d["message"].as_str().unwrap());
            assert_eq!(code.option_name(), d["option"].as_str());
            assert_eq!(DiagCode::from_raw(code.as_raw()), Some(code));
            total += 1;
        }
    }
    assert!(total > 1000, "expected >1000 diagnostics, got {total}");

    let groups = m["groups"].as_array().unwrap();
    assert_eq!(groups.len(), DIAG_GROUPS.len());
    assert!(DIAG_GROUPS.iter().any(|g| g.name == "default"));
}

#[test]
fn ast_kinds_match_model() {
    let m = load("ast-kinds");
    assert_eq!(
        m["modelHash"].as_str().unwrap(),
        AST_KINDS_MODEL_HASH,
        "regenerate with `cargo xtask codegen`"
    );

    fn check<T: Copy + core::fmt::Display>(
        m: &Value,
        name: &str,
        all: &[T],
        from_raw: fn(u16) -> Option<T>,
    ) {
        let vals = m["enums"][name]["values"].as_array().unwrap();
        assert_eq!(vals.len(), all.len(), "{name} variant count");
        for (i, v) in vals.iter().enumerate() {
            let k = from_raw(i as u16).unwrap_or_else(|| panic!("{name}::from_raw({i})"));
            assert_eq!(k.to_string(), v.as_str().unwrap(), "{name}[{i}]");
        }
        assert!(from_raw(all.len() as u16).is_none());
    }
    check(&m, "SymbolKind", SymbolKind::ALL, SymbolKind::from_raw);
    check(
        &m,
        "ExpressionKind",
        ExpressionKind::ALL,
        ExpressionKind::from_raw,
    );
    check(
        &m,
        "StatementKind",
        StatementKind::ALL,
        StatementKind::from_raw,
    );
    check(
        &m,
        "TimingControlKind",
        TimingControlKind::ALL,
        TimingControlKind::from_raw,
    );
    check(
        &m,
        "ConstraintKind",
        ConstraintKind::ALL,
        ConstraintKind::from_raw,
    );
    check(&m, "PatternKind", PatternKind::ALL, PatternKind::from_raw);
    check(
        &m,
        "AssertionExprKind",
        AssertionExprKind::ALL,
        AssertionExprKind::from_raw,
    );
    check(
        &m,
        "BinsSelectExprKind",
        BinsSelectExprKind::ALL,
        BinsSelectExprKind::from_raw,
    );

    // The enums in slang's headers are the source of truth; when building
    // in-tree, also verify the vendored model hasn't drifted from them.
    let hdr = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../include/slang/ast/Symbol.h"
    );
    if let Ok(text) = std::fs::read_to_string(hdr) {
        let n = text
            .lines()
            .skip_while(|l| !l.contains("#define SYMBOLKIND(x)"))
            .take_while(|l| !l.contains("SLANG_ENUM("))
            .filter(|l| l.trim_start().starts_with("x("))
            .count();
        assert_eq!(
            n,
            SymbolKind::ALL.len(),
            "Symbol.h SYMBOLKIND drifted; regenerate ast-kinds model"
        );
    }
}

#[test]
fn diag_code_raw_roundtrip_rejects_out_of_range() {
    assert_eq!(DiagCode::from_raw(0xFFFF_0000), None);
    let last = DiagSubsystem::ALL[DiagSubsystem::ALL.len() - 1];
    assert_eq!(DiagCode::from_raw(((last as u32) << 16) | 0xFFFF), None);
}
