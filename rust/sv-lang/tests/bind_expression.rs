//! `EvalSession::bind_expression[_at]`: bind expression syntax the design never
//! bound itself (overridden parameter defaults, formal defaults) into a real
//! semantic Expression. Requested by the sv-sim session (VPI expression
//! objects).

use sv_lang::kinds::{ExpressionKind, SymbolKind};
use sv_lang::{Compilation, Session};

const SRC: &str = "\
module m #(parameter int W = 4, parameter int D = W * 2 + K) ();
  localparam int K = 3;
endmodule
module top;
  m #(.W(8), .D(1)) u ();
endmodule
";

#[test]
fn binds_overridden_defaults_in_their_scope() {
    let session = Session::new();
    let tree = session.parse(SRC).unwrap();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add(&tree).unwrap();
    let mut design = comp.compile().unwrap();
    assert!(!design.diagnostics().has_errors());
    let n_diags = design.diagnostics().len();

    {
        let eval = design.eval_session();
        let top = eval.design().top_instances().next().unwrap();
        let body = top
            .instance_body()
            .unwrap()
            .find("u")
            .unwrap()
            .instance_body()
            .unwrap();

        // `D = W * 2 + K` with every declaration visible: uses this instance's
        // W (8, the override) and the later localparam K.
        let d = body.find("D").unwrap();
        let dflt = d.parameter_default_syntax(&tree).unwrap();
        let e = eval.bind_expression(dflt, body).unwrap();
        assert_eq!(e.kind(), ExpressionKind::BinaryOp);
        assert_eq!(eval.eval_constant(e).unwrap().as_i64(), Some(8 * 2 + 3));

        // As of D's own position, K (declared later) is not visible: the
        // result is an Invalid expression, not a crash or a None.
        let at = eval
            .bind_expression_at(dflt, d.lookup_location_before())
            .unwrap();
        assert!(at.is_bad());

        // Non-scope scope / non-expression syntax: None.
        assert!(eval.bind_expression(dflt, d).is_none());
        let module_decl = tree.root().children().next().unwrap();
        assert!(eval.bind_expression(module_decl, body).is_none());
    }

    // Binding added nothing to the design's reported diagnostics.
    assert_eq!(design.diagnostics().len(), n_diags);
}

#[test]
fn rejects_syntax_from_a_foreign_tree() {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source(SRC).unwrap();
    let mut design = comp.compile().unwrap();

    // A tree that is not part of `design`'s compilation.
    let other = session
        .parse("module x #(parameter int P = 1 + 2) (); endmodule\n")
        .unwrap();
    // Grab an expression syntax node from `other` by walking for one.
    let mut found = None;
    let mut stack = vec![other.root()];
    while let Some(n) = stack.pop() {
        if n.kind() == sv_lang::kinds::SyntaxKind::AddExpression {
            found = Some(n);
            break;
        }
        stack.extend(n.children());
    }
    let foreign = found.expect("an AddExpression in the foreign tree");

    let eval = design.eval_session();
    let top = eval.design().top_instances().next().unwrap();
    assert_eq!(top.kind(), SymbolKind::Instance);
    assert!(
        eval.bind_expression(foreign, top.instance_body().unwrap())
            .is_none()
    );
}
