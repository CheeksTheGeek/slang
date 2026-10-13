//! Thread-sanitizer NEGATIVE gate for the `Design: Sync` soundness claim.
//!
//! `Compilation::compile()` totalizes the freeze, forcing every lazy memo a
//! shared-`&Design` accessor can reach (see `rust/SOUNDNESS-MEMOS.md`). This
//! test builds a wide design BOTH ways and hammers it from many threads:
//!
//!  * `totalized` (normal `compile()`) — the FULL read surface first-touched
//!    concurrently MUST stay TSan-clean (every reachable memo is forced).
//!  * `sealed_only` (skips totalization) MUST race on the typedef-canonical
//!    anchor — the one memo `getAllDiagnostics()` provably does not force —
//!    proving the totalization is load-bearing and the test has teeth.
//!
//! Requires the `__unsound_test_only` feature and a TSan build:
//!   RUSTFLAGS="-Zsanitizer=thread" SV_LANG_TSAN=1 \
//!     cargo +nightly test -Zbuild-std --target x86_64-unknown-linux-gnu \
//!     -p sv-lang --features __unsound_test_only --test tsan_race
#![cfg(feature = "__unsound_test_only")]

use sv_lang::{Compilation, Design, SemNode, Session, Statement};

const SRC: &str = "\
package pkg;
    typedef logic [7:0] byte_t;
    typedef byte_t      word_t;        // alias-of-alias
    typedef byte_t      arr_t [4];     // unpacked array of alias
    typedef enum logic [1:0] { A, B, C } e_t;
    parameter int P = 8;
    class Cls; int x; function int get(); return x; endfunction endclass
endpackage
nettype real myreal;
module sub #(parameter int W = 4) (input logic [W-1:0] i, output logic [W-1:0] o);
    assign o = i;
endmodule
module m;
    import pkg::*;
    byte_t a; word_t b; arr_t c; e_t e;
    myreal nt;
    localparam int Q = P + 1;
    localparam int CL = 0;
    // A statically-dead arm (CL == 0): `a[7:8-CL]` is a reversed select that is
    // suppressed because the arm is never taken, so the whole always_comb body
    // is demoted to InvalidStatement. The freeze sweep (VisitFlags::Bad) must
    // still force the dead arm's type memos; the hammer reaches them via
    // children_with_invalid()/invalid_child(), so the totalized side must stay
    // race-free here too.
    always_comb begin if (CL) b = a[7:8-CL]; else b = a; end
    // Declaration bodies bound in their own scope (bound_body): built pre-seal
    // into a side table and swept, so concurrent reads must stay race-free.
    sequence sq(x = a[0]); x ##1 a[1]; endsequence
    let lt(v = 3) = v + Q;
    sub #(.W(5)) u (.i('0), .o());
    function automatic int g(int x);
        typedef logic [3:0] local_t;
        local_t t = x[3:0]; return t;
    endfunction
    always_ff @(posedge a[0]) b <= a;
endmodule
";

fn build(sealed_only: bool) -> Design {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source(SRC).unwrap();
    if sealed_only {
        comp.compile_sealed_only().unwrap()
    } else {
        comp.compile().unwrap()
    }
}

// --- the broad read-surface sweep (totalized side must stay TSan-clean) ---

fn walk_sem(n: SemNode<'_>) {
    let _ = n.kind_name();
    if let Some(e) = n.as_expression() {
        let _ = (
            e.kind(),
            e.binary_op(),
            e.unary_op(),
            e.operands(),
            e.expr_type(),
        );
        // First-touch the canonical of a (possibly dead-arm) expression's type,
        // the R-A1 race surface; the freeze sweep must have forced it.
        if let Some(t) = e.expr_type() {
            let _ = t.canonical();
        }
        if let Some(c) = e.invalid_child() {
            walk_sem(c.as_sem_node());
        }
    }
    if let Some(s) = n.as_statement()
        && let Some(c) = s.invalid_child()
    {
        walk_sem(c.as_sem_node());
    }
    // children_with_invalid() crosses into statically-dead subtrees whose
    // types/memos are reachable from a shared &Design — so the totalized side
    // must stay race-free here too (the gate for the invalid_child fix).
    for c in n.children_with_invalid() {
        walk_sem(c);
    }
}

fn walk_stmt(s: Statement<'_>) {
    let _ = s.kind();
    if let Some(c) = s.invalid_child() {
        walk_sem(c.as_sem_node());
    }
    for c in s.children_with_invalid() {
        walk_sem(c);
    }
}

fn touch_all(design: &Design) {
    for top in design.top_instances() {
        let Some(body) = top.instance_body() else {
            continue;
        };
        let mut types = Vec::new();
        for m in body.members() {
            let _ = (m.kind(), m.name(), m.hierarchical_path());
            let _ = (
                m.next_sibling(),
                m.definition_kind(),
                m.instance_definition(),
            );
            let _ = m.parameter_value();
            if let Some(t) = m.as_type().or_else(|| m.value_type()) {
                let _ = t.canonical();
                let _ = (t.is_class(), t.is_integral(), t.is_signed());
                let _ = (t.is_four_state(), t.is_unpacked_array(), t.bit_width());
                let _ = t.to_sv_string();
                types.push(t);
            }
            if let Some(ex) = m.initializer() {
                let _ = ex.kind();
                let _ = ex.expr_type().map(|t| t.is_class());
                let _ = ex.constant();
            }
            if let Some(root) = m.body() {
                walk_stmt(root);
            }
            if let Some(bb) = m.bound_body() {
                walk_sem(bb.as_sem_node());
            }
            for p in m.parameters() {
                let _ = p.parameter_value();
            }
            if let Some(inner) = m.instance_body() {
                for im in inner.members() {
                    let _ = im.name();
                    if let Some(t) = im.value_type() {
                        let _ = (t.canonical(), t.bit_width());
                    }
                    let _ = im.parameter_value();
                }
            }
        }
        // cross-operand + element-recursion canonical.
        for a in &types {
            for b in &types {
                let _ = (
                    a.is_equivalent(b),
                    a.is_matching(b),
                    a.is_assignment_compatible(b),
                );
            }
        }
        let _ = body.find("a");
    }
}

fn hammer<F: Fn(&Design) + Sync>(design: &Design, f: F) {
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let d = design;
                let f = &f;
                scope.spawn(move || f(d))
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
    });
}

/// The narrow canonical anchor: first-touch a typedef's canonical from N threads.
fn touch_canonical(design: &Design) {
    for top in design.top_instances() {
        let Some(body) = top.instance_body() else {
            continue;
        };
        for m in body.members() {
            if let Some(t) = m.value_type() {
                let _ = t.is_class();
                let _ = t.canonical().is_integral();
            }
        }
    }
}

/// Totalized design: the WHOLE read surface, hammered concurrently, is race-free.
#[test]
fn totalized_is_race_free() {
    let design = build(false);
    hammer(&design, touch_all);
    assert!(design.freeze_report().types_canonicalized > 0);
}

/// Un-totalized design: first-touching the typedef canonical races. Under TSan
/// this reports a data race (the job is expected to fail), proving the
/// totalization is load-bearing. Meaningful only under a TSan build (CI sets
/// SV_LANG_TSAN); skipped otherwise so a normal feature run stays green.
#[test]
fn sealed_only_races() {
    if std::env::var_os("SV_LANG_TSAN").is_none() {
        eprintln!("skipping sealed-only race probe (set SV_LANG_TSAN under a TSan build)");
        return;
    }
    let design = build(true);
    hammer(&design, touch_canonical);
}
