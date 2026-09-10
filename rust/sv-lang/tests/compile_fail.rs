//! Compile-fail tests: the type-level soundness invariants must be enforced by
//! the borrow checker, not just documented.

#[test]
fn invariants_are_enforced() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/*.rs");
}
