//! Typed accessors for the non-Statement/Expression AST families —
//! TimingControl, Constraint, AssertionExpr, BinsSelectExpr, Pattern. Before
//! these were reachable only as opaque `SemNode`s (string `kind_name()` +
//! generic children); these tests prove the typed `.kind()`/navigation is wired.

use sv_lang::kinds::{PatternKind, SymbolKind, TimingControlKind};
use sv_lang::{Compilation, Design, EdgeKind, SemNode, Session};

fn design(src: &str) -> Design {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source(src).unwrap();
    let d = comp.compile().unwrap();
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    d
}

fn procedural_body<'d>(d: &'d Design) -> sv_lang::Statement<'d> {
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .expect("a procedural block");
    block.body().expect("procedural block has a body")
}

/// Recursively collect a statement's entire semantic subtree.
fn collect(node: SemNode<'_>, out: &mut Vec<(u32, String)>) {
    out.push((node.domain(), node.kind_name()));
    for c in node.children() {
        collect(c, out);
    }
}

#[test]
fn timing_control_is_typed_with_edge() {
    let d = design(
        "module m(input logic clk, input logic d, output logic q);\n\
             always_ff @(posedge clk) q <= d;\n\
         endmodule\n",
    );
    let tc = procedural_body(&d)
        .timing_control()
        .expect("always_ff has a timing control");
    assert_eq!(tc.kind(), TimingControlKind::SignalEvent);
    assert_eq!(tc.edge(), EdgeKind::PosEdge);
    assert_eq!(tc.kind_name(), "SignalEvent");
    // the clock expression is reachable as a typed child.
    assert!(
        tc.children().iter().any(|c| c.as_expression().is_some()),
        "the event expression should be a child"
    );
}

#[test]
fn negedge_and_level_edges() {
    let d = design(
        "module m(input logic clk, input logic a, output logic q);\n\
             always_ff @(negedge clk) q <= a;\n\
         endmodule\n",
    );
    assert_eq!(
        procedural_body(&d).timing_control().unwrap().edge(),
        EdgeKind::NegEdge
    );

    // A level-sensitive event has no edge.
    let d2 = design(
        "module m(input logic a, output logic q);\n\
             always @(a) q = a;\n\
         endmodule\n",
    );
    assert_eq!(
        procedural_body(&d2).timing_control().unwrap().edge(),
        EdgeKind::None
    );
}

#[test]
fn assertion_expr_is_typed() {
    // A concurrent assertion's property tree is reachable and typed.
    let d = design(
        "module m(input logic clk, input logic a, input logic b);\n\
             assert property (@(posedge clk) a |-> b);\n\
         endmodule\n",
    );
    let body = procedural_body(&d);
    let mut nodes = Vec::new();
    collect(body.as_sem_node(), &mut nodes);
    // Walk the subtree; at least one node must be an assertion-expr with a
    // real (non-Invalid) kind, and it must type via as_assertion_expr.
    let body_node = body.as_sem_node();
    let mut stack = vec![body_node];
    let mut found = false;
    let mut saw_op = false;
    while let Some(n) = stack.pop() {
        if let Some(ae) = n.as_assertion_expr() {
            assert_ne!(
                ae.kind(),
                sv_lang::kinds::AssertionExprKind::Invalid,
                "assertion-expr node {:?} should have a real kind",
                ae.kind_name()
            );
            // A Unary/Binary assertion expr (e.g. the `|->` implication) exposes
            // its operator by name; other kinds return None.
            use sv_lang::kinds::AssertionExprKind::{Binary, Unary};
            if matches!(ae.kind(), Binary | Unary) {
                assert!(
                    ae.op().is_some(),
                    "unary/binary assertion op should be Some"
                );
                saw_op = true;
            } else {
                assert!(ae.op().is_none());
            }
            found = true;
        }
        stack.extend(n.children());
    }
    assert!(
        saw_op,
        "the `a |-> b` implication should expose a binary op"
    );
    assert!(
        found,
        "expected an assertion-expr node in the property tree; saw {nodes:?}"
    );
}

#[test]
fn pattern_is_typed() {
    // `case ... matches` produces Pattern nodes; reach one and read its kind.
    let d = design(
        "module m(input logic [1:0] sel, output logic y);\n\
             always_comb begin\n\
                 case (sel) matches\n\
                     2'b00: y = 1'b0;\n\
                     default: y = 1'b1;\n\
                 endcase\n\
             end\n\
         endmodule\n",
    );
    let body = procedural_body(&d);
    // Walk to any Pattern node (domain SLANG_AST_PATTERN == 7).
    let mut stack = vec![body.as_sem_node()];
    let mut saw_pattern = false;
    let mut saw_const = false;
    while let Some(n) = stack.pop() {
        if let Some(p) = n.as_pattern() {
            assert_ne!(p.kind(), PatternKind::Invalid);
            let _ = p.kind_name();
            let _ = p.children();
            // The `2'b00` case item is a ConstantPattern whose value expression
            // is reachable by name; other kinds return None.
            if p.kind() == PatternKind::Constant {
                assert!(
                    p.value_expr().is_some(),
                    "a constant pattern should expose its value expression"
                );
                saw_const = true;
            }
            saw_pattern = true;
        }
        stack.extend(n.children());
    }
    assert!(
        saw_pattern,
        "expected a Pattern node under a `case matches`"
    );
    assert!(saw_const, "expected the `2'b00` constant pattern");
}
