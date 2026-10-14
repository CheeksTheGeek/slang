//! `Symbol::id()` used to be just the hierarchical path, which unnamed blocks
//! (and the implicit block of `for`/`foreach`) don't contribute to — so
//! distinct variables shared an id. Reported by the sv-sim session, where a
//! map keyed by it merged a `foreach` index with a later `for` index and a
//! loop silently never ran.

use std::collections::HashMap;

use sv_lang::kinds::SymbolKind;
use sv_lang::{Compilation, Design, Session, Symbol, SymbolId, Walk};

const SRC: &str = "\
module top;
  int k;
  int g[2][2];
  initial begin
    int k;
    k = 1;
  end
  initial begin
    foreach (g[i, j]) g[i][j] = 0;
    for (int i = 0; i < 2; i++) k = i;
  end
endmodule
";

fn design() -> Design {
    let s = Session::new();
    let mut comp = Compilation::new(&s).unwrap();
    comp.add_source(SRC).unwrap();
    let d = comp.compile().unwrap();
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    d
}

/// Every symbol named `name` anywhere under `top`.
fn all_named<'d>(top: Symbol<'d>, name: &str) -> Vec<Symbol<'d>> {
    let mut out = Vec::new();
    top.visit(|s| {
        if s.name() == name {
            out.push(s);
        }
        Walk::Continue
    });
    out
}

#[test]
fn colliding_paths_get_distinct_ids() {
    let d = design();
    let top = d.top_instances().next().unwrap().instance_body().unwrap();

    for name in ["k", "i"] {
        let syms = all_named(top, name);
        assert!(syms.len() >= 2, "{name}: {}", syms.len());
        let paths: Vec<_> = syms.iter().map(|s| s.hierarchical_path()).collect();
        assert!(
            paths.windows(2).any(|w| w[0] == w[1]),
            "{name}: expected a path collision to exercise, got {paths:?}"
        );
        let mut ids: HashMap<SymbolId, Symbol<'_>> = HashMap::new();
        for s in &syms {
            assert!(ids.insert(s.id(), *s).is_none(), "duplicate id for {name}");
        }
    }
}

#[test]
fn ids_round_trip_through_resolve() {
    let mut d = design();
    let ids: Vec<(SymbolId, SymbolKind)> = {
        let top = d.top_instances().next().unwrap().instance_body().unwrap();
        ["k", "i", "j"]
            .iter()
            .flat_map(|n| all_named(top, n))
            .map(|s| (s.id(), s.kind()))
            .collect()
    };
    assert!(ids.len() >= 5);
    for (id, kind) in &ids {
        let s = d.resolve(id).unwrap_or_else(|| panic!("unresolved {id}"));
        assert_eq!(s.id(), *id);
        assert_eq!(s.kind(), *kind);
    }
}

#[test]
fn ids_are_stable_across_a_rebuild() {
    let (a, b) = (design(), design());
    let ids = |d: &Design| -> Vec<SymbolId> {
        let top = d.top_instances().next().unwrap().instance_body().unwrap();
        all_named(top, "k").iter().map(|s| s.id()).collect()
    };
    assert_eq!(ids(&a), ids(&b));
}
