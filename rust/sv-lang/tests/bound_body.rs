//! `Symbol::bound_body()` / `Symbol::assertion_port_default_syntax()` — a
//! sequence / property / let declaration's body bound in its own scope, with no
//! instantiation needed (slang's AssertionInstanceExpression::makeDefault, kept
//! by the freeze sweep). Requested by the sv-sim session for VPI vpiExpr on
//! declaration bodies and formal defaults (LRM 37.x).

use sv_lang::kinds::{ExpressionKind, SymbolKind};
use sv_lang::{Compilation, Session};

const SRC: &str = "module m;\n\
    logic clk, a, b;\n\
    sequence s(x = a, y); x ##1 y; endsequence\n\
    property p(e = b); @(posedge clk) a |-> e; endproperty\n\
    let twice(v = 4) = v * 2;\n\
    let open(v) = v + 1;\n\
    endmodule\n";

#[test]
fn sequence_property_and_let_bodies() {
    let s = Session::new();
    let mut comp = Compilation::new(&s).unwrap();
    comp.add_source(SRC).unwrap();
    let d = comp.compile().unwrap();
    let body = d.top_instances().next().unwrap().instance_body().unwrap();

    for name in ["s", "p"] {
        let decl = body.find(name).unwrap();
        let inst = decl
            .bound_body()
            .unwrap_or_else(|| panic!("{name} has a bound body"));
        assert_eq!(inst.kind(), ExpressionKind::AssertionInstance, "{name}");
        // The body assertion expression is the instance's semantic child.
        let kids = inst.children();
        assert!(
            kids.iter().any(|k| k.as_assertion_expr().is_some()),
            "{name}: body AssertionExpr reachable"
        );
    }

    // A let whose formal has a default folds through it.
    let twice = body.find("twice").unwrap().bound_body().unwrap();
    assert_eq!(twice.constant_value().unwrap().as_i64(), Some(8));

    // A let whose formal has no default still binds (placeholder formal): the
    // body exists but cannot fold.
    let open = body
        .find("open")
        .unwrap()
        .bound_body()
        .expect("placeholder-bound");
    assert!(open.constant_value().is_none());
}

#[test]
fn formal_default_syntax() {
    let s = Session::new();
    let tree = s.parse(SRC).unwrap();
    let mut comp = Compilation::new(&s).unwrap();
    comp.add(&tree).unwrap();
    let d = comp.compile().unwrap();
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let seq = body.find("s").unwrap();
    let ports: Vec<_> = seq
        .members()
        .filter(|m| m.kind() == SymbolKind::AssertionPort)
        .collect();
    let x = ports.iter().find(|p| p.name() == "x").unwrap();
    let y = ports.iter().find(|p| p.name() == "y").unwrap();
    assert_eq!(
        x.assertion_port_default_syntax(&tree)
            .unwrap()
            .text()
            .trim(),
        "a"
    );
    assert!(y.assertion_port_default_syntax(&tree).is_none());
}

#[test]
fn non_declarations_have_no_bound_body() {
    let s = Session::new();
    let mut comp = Compilation::new(&s).unwrap();
    comp.add_source(SRC).unwrap();
    let d = comp.compile().unwrap();
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    assert!(body.find("clk").unwrap().bound_body().is_none());
    assert!(body.bound_body().is_none());
}

#[test]
fn rebinding_adds_no_diagnostics() {
    // The freeze sweep re-runs the exact binding slang performs during
    // elaboration; a declaration whose body has an error must still report it
    // exactly once, and a clean design must stay clean.
    let s = Session::new();
    let mut comp = Compilation::new(&s).unwrap();
    comp.add_source(
        "module m;\n  logic a;\n  let bad(v) = v + not_declared;\n  sequence q; a ##1 not_declared2; endsequence\nendmodule\n",
    )
    .unwrap();
    let d = comp.compile().unwrap();
    let undeclared: Vec<_> = d
        .diagnostics()
        .items()
        .iter()
        .filter(|x| x.code_name() == "UndeclaredIdentifier")
        .map(|x| x.message.clone())
        .collect();
    assert_eq!(
        undeclared.len(),
        2,
        "each reported exactly once: {undeclared:?}"
    );

    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    // Still recoverable despite the error.
    assert!(body.find("bad").unwrap().bound_body().is_some());

    let s2 = Session::new();
    let mut clean = Compilation::new(&s2).unwrap();
    clean.add_source(SRC).unwrap();
    let cd = clean.compile().unwrap();
    assert!(!cd.diagnostics().has_errors());
}
