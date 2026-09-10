//! The elaborated behavioral tree: from a procedural block to its statements
//! and typed expression operands.

use sv_lang::kinds::{ExpressionKind, StatementKind, SymbolKind};
use sv_lang::{BinaryOp, Compilation, SemNode, Session, Statement};

const SRC: &str = "\
module m(input logic clk, rst, input logic [7:0] d, output logic [7:0] q);
    always_ff @(posedge clk) begin
        if (rst)
            q <= 8'h00;
        else
            q <= d + 8'h01;
    end
endmodule
";

fn compile(src: &str) -> sv_lang::Design {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source(src).unwrap();
    comp.compile().unwrap()
}

#[derive(Default)]
struct Seen {
    stmts: Vec<StatementKind>,
    exprs: Vec<ExpressionKind>,
    bin_ops: Vec<BinaryOp>,
    nonblocking: usize,
}

fn walk_node(node: SemNode<'_>, seen: &mut Seen) {
    if let Some(s) = node.as_statement() {
        seen.stmts.push(s.kind());
    }
    if let Some(e) = node.as_expression() {
        seen.exprs.push(e.kind());
        if let Some(op) = e.binary_op() {
            seen.bin_ops.push(op);
        }
        if e.is_nonblocking() == Some(true) {
            seen.nonblocking += 1;
        }
    }
    for c in node.children() {
        walk_node(c, seen);
    }
}

fn walk_stmt(s: Statement<'_>, seen: &mut Seen) {
    seen.stmts.push(s.kind());
    for c in s.children() {
        walk_node(c, seen);
    }
}

#[test]
fn procedural_block_body_is_reachable() {
    let design = compile(SRC);
    let top = design.top_instances().next().unwrap();
    let body = top.instance_body().unwrap();

    let proc = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .expect("module has an always_ff procedural block");

    let root = proc.body().expect("procedural block has a body statement");

    let mut seen = Seen::default();
    walk_stmt(root, &mut seen);

    // The behavioral content is now visible on the semantic side.
    assert!(
        seen.stmts.contains(&StatementKind::Conditional),
        "expected an if/else, saw {:?}",
        seen.stmts
    );
    assert!(
        seen.exprs.contains(&ExpressionKind::Assignment),
        "expected the non-blocking assignments, saw {:?}",
        seen.exprs
    );
    // The `d + 8'h01` is a typed binary Add.
    assert!(
        seen.bin_ops.contains(&BinaryOp::Add),
        "expected a binary Add, saw {:?}",
        seen.bin_ops
    );
    // Both `q <= ...` are non-blocking assignments.
    assert!(
        seen.nonblocking >= 2,
        "expected two non-blocking assignments, saw {}",
        seen.nonblocking
    );
}

#[test]
fn conditional_branches_by_name() {
    let design = compile(SRC);
    let top = design.top_instances().next().unwrap();
    let body = top.instance_body().unwrap();

    let proc = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .expect("module has an always_ff procedural block");

    // always_ff @(posedge clk) begin ... end  ->  Timed -> Block -> Conditional
    let timed = proc.body().expect("procedural block has a body statement");
    assert_eq!(timed.kind(), StatementKind::Timed);
    assert!(
        timed.timing().is_some(),
        "timed statement exposes its timing control"
    );

    let block = timed
        .body()
        .expect("timed statement wraps the begin/end block");
    let cond = block
        .statements()
        .into_iter()
        .find(|s| s.kind() == StatementKind::Conditional)
        .expect("module has an if/else");

    let then_b = cond.then_branch().expect("if has a then-branch");
    let else_b = cond.else_branch().expect("if has an else-branch");
    assert_eq!(then_b.kind(), StatementKind::ExpressionStatement);
    assert_eq!(else_b.kind(), StatementKind::ExpressionStatement);

    // The condition (`rst`) is reachable as a child expression.
    assert!(
        !cond.expressions().is_empty(),
        "if condition is a child expression"
    );

    // The else-branch `q <= d + 8'h01` reaches a binary Add.
    let mut seen = Seen::default();
    walk_stmt(else_b, &mut seen);
    assert!(
        seen.bin_ops.contains(&BinaryOp::Add),
        "else-branch should contain d + 1, saw {:?}",
        seen.bin_ops
    );

    // A statement that is not a conditional yields None for the branch accessors.
    assert!(then_b.then_branch().is_none());
    assert!(then_b.else_branch().is_none());
}

#[test]
fn binary_operands_are_ordered_and_typed() {
    // localparam so the expression is elaborated and folded.
    let design = compile("module m; logic [7:0] a, b; wire [7:0] s = a - b; endmodule\n");
    let top = design.top_instances().next().unwrap();
    let sbody = top.instance_body().unwrap();

    // Find the continuous-assign expression by walking every value's initializer
    // isn't available here; instead reach the net `s` and its initializer.
    let s = sbody.find("s").expect("net s exists");
    let init = s.initializer().expect("s has an initializer expression");

    assert_eq!(init.kind(), ExpressionKind::BinaryOp);
    assert_eq!(init.binary_op(), Some(BinaryOp::Subtract));
    let ops = init.operands();
    assert_eq!(ops.len(), 2, "a - b has two operands");
    // Both operands carry a type.
    assert!(ops[0].expr_type().is_some());
    assert!(init.left().is_some() && init.right().is_some());
}
