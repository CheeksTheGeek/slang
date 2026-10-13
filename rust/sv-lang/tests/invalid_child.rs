//! `Statement::invalid_child()` / `Expression::invalid_child()` — recover the
//! real node wrapped by an `Invalid` node. slang demotes a whole statement to
//! `InvalidStatement` when any part fails to bind, even a statically-dead
//! if/case arm; this exposes `InvalidStatement::child` so a consumer can walk
//! the surviving statement and lower its live arm. Requested by the sv-sim
//! session (verilog-axis designs with `$clog2(1) == 0` guards).

use sv_lang::kinds::{StatementKind, SymbolKind};
use sv_lang::{Compilation, Session};

// CL = $clog2(1) = 0, so the `if (CL)` arm (with the `+: 0` select) is
// statically dead; slang demotes the whole always_comb body to Invalid.
const DEAD_ARM: &str = "module m #(parameter CL = 0)(input logic [7:0] y, output logic [7:0] x);\n\
    logic [7:0] q;\n\
    always_comb begin if (CL) q = y[0 +: CL]; else q = y; end\n\
    assign x = q;\n\
    endmodule\n";

fn proc_body(d: &sv_lang::Design) -> sv_lang::Statement<'_> {
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let proc = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    proc.body().unwrap()
}

#[test]
fn invalid_statement_exposes_its_child() {
    let s = Session::new();
    let mut comp = Compilation::new(&s).unwrap();
    comp.add_source(DEAD_ARM).unwrap();
    let d = comp.compile().unwrap();

    // With the dead-arm `+: 0` width error suppressed in unevaluated branches,
    // this design (like the reversed-range one) now compiles with ZERO errors.
    let errs: Vec<_> = d
        .diagnostics()
        .items()
        .iter()
        .filter(|x| x.is_error())
        .map(|x| format!("{}: {}", x.code_name(), x.message))
        .collect();
    assert!(errs.is_empty(), "expected zero errors, got: {errs:?}");

    let stmt = proc_body(&d);
    // Confirms the symptom: the whole body is Invalid, with no normal children.
    assert_eq!(stmt.kind(), StatementKind::Invalid);
    assert!(stmt.children().is_empty());

    // The real statement (the always_comb `begin…end` block) is recoverable
    // and walkable through the normal API.
    let inner = stmt.invalid_child().expect("invalid statement has a child");
    assert_eq!(inner.kind(), StatementKind::Block);

    // The conditional, and thus its live arm, is reachable by walking the
    // recovered subtree. Invalid nodes nest — the always block was demoted
    // because the conditional inside it was itself demoted (its dead arm's
    // `+: 0` select is bad) — so a consumer unwraps invalid_child recursively.
    let mut stack = vec![stmt.as_sem_node()];
    let mut found_conditional = false;
    while let Some(n) = stack.pop() {
        if let Some(s) = n.as_statement() {
            if s.kind() == StatementKind::Conditional {
                found_conditional = true;
            }
            if let Some(c) = s.invalid_child() {
                stack.push(c.as_sem_node());
            }
        }
        if let Some(e) = n.as_expression()
            && let Some(c) = e.invalid_child()
        {
            stack.push(c.as_sem_node());
        }
        // children_with_invalid() surfaces bad members (e.g. the conditional
        // demoted to Invalid inside the block) that children() would skip.
        stack.extend(n.children_with_invalid());
    }
    assert!(
        found_conditional,
        "the conditional should be reachable by unwrapping invalid children"
    );
}

// The reversed-range variant the peer named: `i[7:8-CL]` with CL=0 is a
// reversed select in a statically-dead arm. slang suppresses that diagnostic in
// dead arms, so unlike the `+: 0` case this design has ZERO errors — yet the
// body is still demoted to Invalid, so recovery is the only way to reach it.
const DEAD_ARM_REVERSED: &str = "module m #(parameter CL = 0)(input logic [7:0] i, output logic [7:0] o);\n\
     logic [7:0] r;\n\
     always_comb begin if (CL) r = i[7:8-CL]; else r = i; end\n\
     assign o = r;\n\
     endmodule\n";

#[test]
fn reversed_range_dead_arm_recovers_with_zero_errors() {
    let s = Session::new();
    let mut comp = Compilation::new(&s).unwrap();
    comp.add_source(DEAD_ARM_REVERSED).unwrap();
    let d = comp.compile().unwrap();

    // No hard error for this class (the dead-arm diagnostic is suppressed).
    let errs: Vec<_> = d
        .diagnostics()
        .items()
        .iter()
        .filter(|x| x.is_error())
        .map(|x| format!("{}: {}", x.code_name(), x.message))
        .collect();
    assert!(errs.is_empty(), "expected zero errors, got: {errs:?}");

    let stmt = proc_body(&d);
    assert_eq!(stmt.kind(), StatementKind::Invalid);

    // Recover the conditional, confirm its condition folds to 0 (the dead arm),
    // and that the live else arm is reachable — exactly what a consumer needs to
    // lower only the taken arm.
    let mut stack = vec![stmt.as_sem_node()];
    let mut cond = None;
    while let Some(n) = stack.pop() {
        if let Some(st) = n.as_statement() {
            if st.kind() == StatementKind::Conditional {
                cond = Some(st);
            }
            if let Some(c) = st.invalid_child() {
                stack.push(c.as_sem_node());
            }
        }
        stack.extend(n.children_with_invalid());
    }
    let cond = cond.expect("conditional recovered");
    // The condition expression (CL) is constant 0 -> the else arm is live.
    let found_zero_cond = cond.children_with_invalid().iter().any(|c| {
        c.as_expression()
            .and_then(|e| e.constant_value())
            .and_then(|v| v.as_i64())
            == Some(0)
    });
    assert!(found_zero_cond, "the folded condition should be 0");
}

#[test]
fn normal_statement_has_no_invalid_child() {
    let s = Session::new();
    let mut comp = Compilation::new(&s).unwrap();
    comp.add_source("module m(input logic a, output logic x);\n  always_comb x = a;\nendmodule\n")
        .unwrap();
    let d = comp.compile().unwrap();
    let stmt = proc_body(&d);
    assert_ne!(stmt.kind(), StatementKind::Invalid);
    assert!(stmt.invalid_child().is_none());
}
