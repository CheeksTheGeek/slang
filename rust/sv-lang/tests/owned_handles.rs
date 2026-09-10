//! `SymbolId` (a stable, owned key that survives a design rebuild) and
//! `Owned*` handles (a handle bundled with its design so it can be stored past
//! the borrow that produced it) — the pieces an editor server needs to remember
//! symbols across edits.

use sv_lang::{Compilation, Design, IntoOwned, Session};

fn design(src: &str) -> Design {
    let session = Session::new();
    let mut comp = Compilation::new(&session).expect("compilation");
    comp.add_source(src).expect("add source");
    comp.compile().expect("elaborate")
}

#[test]
fn symbol_id_round_trips_and_is_a_map_key() {
    use std::collections::HashMap;

    let mut d = design("module m; logic [7:0] x; logic y; endmodule\n");

    // Take ids through a shared borrow that ends before the &mut resolve.
    let (idx, idy) = {
        let body = d.top_instances().next().unwrap().instance_body().unwrap();
        (body.find("x").unwrap().id(), body.find("y").unwrap().id())
    };

    assert_eq!(idx.path(), "m.x");
    assert_ne!(idx, idy);

    // A plain owned key: hashable, cloneable, storable.
    let mut labels = HashMap::new();
    labels.insert(idx.clone(), "eight");
    labels.insert(idy.clone(), "one");
    assert_eq!(labels[&idx], "eight");

    // Re-resolve against the design.
    assert_eq!(d.resolve(&idx).unwrap().name(), "x");
    assert_eq!(d.resolve(&idy).unwrap().name(), "y");
}

#[test]
fn owned_symbol_outlives_the_design_binding() {
    // The `Owned` clones the design's `Arc`, so it stays valid after the
    // `Design` value that produced it is dropped.
    let owned = {
        let d = design("module m; logic [7:0] x; endmodule\n");
        let body = d.top_instances().next().unwrap().instance_body().unwrap();
        body.find("x").unwrap().into_owned(&d)
    }; // `d` dropped here.

    assert_eq!(owned.get().name(), "x");
    let ty = owned.get().value_type().expect("x has a type");
    assert_eq!(ty.bit_width(), 8);
}

#[test]
fn owned_type_is_reborrowable() {
    let d = design("module m; logic [15:0] w; endmodule\n");
    let owned_ty = {
        let body = d.top_instances().next().unwrap().instance_body().unwrap();
        body.find("w").unwrap().value_type().unwrap().into_owned(&d)
    };
    assert_eq!(owned_ty.get().bit_width(), 16);
}

#[test]
fn owned_handles_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<sv_lang::OwnedSymbol>();
    assert_send_sync::<sv_lang::OwnedType>();
    assert_send_sync::<sv_lang::OwnedExpression>();
    assert_send_sync::<sv_lang::OwnedStatement>();
    assert_send_sync::<sv_lang::SymbolId>();

    // And an owned handle really can cross a thread boundary.
    let owned = {
        let d = design("module m; logic [3:0] z; endmodule\n");
        let body = d.top_instances().next().unwrap().instance_body().unwrap();
        body.find("z").unwrap().into_owned(&d)
    };
    let name = std::thread::spawn(move || owned.get().name().to_string())
        .join()
        .unwrap();
    assert_eq!(name, "z");
}
