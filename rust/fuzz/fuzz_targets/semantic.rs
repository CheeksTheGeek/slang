//! Fuzz the full semantic accessor surface — including the elaborated
//! statement/expression tree — on arbitrary input. Nothing here may panic or
//! abort (every C accessor it reaches is exception-guarded).
//! Run with `cargo +nightly fuzz run semantic`.
#![no_main]
use libfuzzer_sys::fuzz_target;
use sv_lang::{Compilation, SemNode, Session, Statement, Symbol};

fn walk_sem(n: SemNode<'_>, depth: u32) {
    if depth > 64 {
        return;
    }
    let _ = n.kind_name();
    if let Some(e) = n.as_expression() {
        let _ = (e.kind(), e.binary_op(), e.unary_op(), e.operands(), e.expr_type());
    }
    for c in n.children() {
        walk_sem(c, depth + 1);
    }
}

fn walk_stmt(s: Statement<'_>, depth: u32) {
    if depth > 64 {
        return;
    }
    let _ = s.kind();
    for c in s.children() {
        walk_sem(c, depth + 1);
    }
}

fn walk_symbol(sym: Symbol<'_>, depth: u32) {
    if depth > 48 {
        return;
    }
    let _ = (sym.kind(), sym.name(), sym.hierarchical_path());
    if let Some(t) = sym.as_type().or_else(|| sym.value_type()) {
        let _ = (t.bit_width(), t.is_class(), t.canonical(), t.is_equivalent(&t));
    }
    if let Some(root) = sym.body() {
        walk_stmt(root, 0);
    }
    if let Some(body) = sym.instance_body() {
        for m in body.members() {
            walk_symbol(m, depth + 1);
        }
    }
    for m in sym.members() {
        walk_symbol(m, depth + 1);
    }
}

fuzz_target!(|data: &str| {
    let session = Session::new();
    if let Ok(mut comp) = Compilation::new(&session)
        && comp.add_source(data).is_ok()
        && let Ok(design) = comp.compile()
    {
        let _ = design.diagnostics().error_count();
        for top in design.top_instances() {
            walk_symbol(top, 0);
        }
    }
});
