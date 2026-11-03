//! The freeze sweep reaches class specializations created while sweeping.
//!
//! Binding one specialization's method bodies can create a specialization of
//! another generic class, one the traversal had already finished. Those method
//! bodies used to be bound for the first time after the seal, which asserts and
//! unwinds through `StatementContext`'s destructor (a second throw, so
//! `std::terminate`). Reported by the Carcilator session (BR-freeze-1): every
//! design including `uvm_pkg.sv` aborted in `Design::analyze`.

use sv_lang::kinds::SymbolKind;
use sv_lang::{AnalysisFlags, Compilation, Design, Session, Symbol};

fn compile(src: &str) -> Design {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source(src).unwrap();
    comp.compile().unwrap()
}

/// `B` is declared before `A`, so by the time `A#(shortint)::f` is bound and
/// creates `B#(shortint,3)`, the traversal has already finished `B`. The label
/// in `h` makes binding its body allocate a block scope.
const NESTED: &str = "\
class B #(type T=int, int K=1);
  function void h(); lbl: if (1) ; endfunction
endclass
class A #(type T=int);
  function void f(); B#(T,3) b; endfunction
endclass
module top; A#(shortint) a; endmodule
";

fn find<'d>(scope: Symbol<'d>, name: &str) -> Symbol<'d> {
    scope
        .find(name)
        .unwrap_or_else(|| panic!("no member {name:?}"))
}

#[test]
fn analyze_survives_a_specialization_created_inside_another() {
    let design = compile(NESTED);
    // On the old sweep this aborted the whole process (SIGABRT), so merely
    // returning is the assertion.
    design.analyze(AnalysisFlags::default(), 1).unwrap();
}

#[test]
fn the_nested_specializations_method_body_is_readable() {
    // Not a discriminating test of the sweep (some read paths lift the seal
    // themselves, so this passes on the old code too): the one that fails
    // without the fix is `analyze_survives_...` above, which aborts the process.
    // This one checks the specialization's structure is reachable and real.
    let design = compile(NESTED);
    let top = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    // top.a is an A#(shortint); its method f declares `B#(T,3) b`.
    let a_ty = find(top, "a").value_type().unwrap();
    let a_cls = a_ty.canonical();
    let f = find(a_cls.as_symbol(), "f");
    assert_eq!(f.kind(), SymbolKind::Subroutine);
    // f's body is a pure read; its local b has type B#(shortint,3), whose
    // method h was only created while f was being bound.
    assert!(f.body().is_some(), "f has a body");
    let b = f.members().find(|s| s.name() == "b").expect("f's local b");
    let b_cls = b.value_type().unwrap().canonical();
    let h = find(b_cls.as_symbol(), "h");
    let h_body = h.body().expect("B#(shortint,3)::h has a body");
    // `lbl: if (1);` -- a labelled conditional, so real structure, not a stub.
    assert!(!h_body.children().is_empty());
}

#[test]
fn a_self_referencing_generic_does_not_invalidate_the_sweep() {
    // Binding R#(2)::f creates R#(1), which inserts into the same generic's
    // specialization map that the sweep is walking. Visiting it live would be
    // undefined; the sweep walks a snapshot and finishes.
    let design = compile(
        "class R #(int N=2);\n\
           function void f(); R#(1) x; endfunction\n\
         endclass\n\
         module top; R#(2) r; endmodule\n",
    );
    design.analyze(AnalysisFlags::default(), 1).unwrap();
}
