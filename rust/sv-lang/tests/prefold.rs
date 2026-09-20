//! `Options::with_prefold(false)` / `Driver::set_prefold(false)` — a freeze that
//! matches pyslang's `expr.constant` population (only constants slang caches
//! naturally during binding), instead of pre-folding every foldable expression.
//! Requested by the svling port for byte parity with the Python compiler.

use sv_lang::{Compilation, Design, Options, Session, Statement};

// The svling repro: the inner `<` is constant None in pyslang 11, but Some after
// a PREFOLD sweep.
const SRC: &str =
    "module m; reg [19:0] r; initial r = (r != 3) ? ((7'h16 || 10'h24e) < 5) : 1; endmodule\n";

fn initial_body(d: &Design) -> Statement<'_> {
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == sv_lang::kinds::SymbolKind::ProceduralBlock)
        .expect("the initial block");
    block.body().expect("procedural block has a body")
}

/// Count expressions in the initial block's subtree that carry a cached constant.
fn cached_constants(d: &Design) -> usize {
    let mut stack = vec![initial_body(d).as_sem_node()];
    let mut n = 0;
    while let Some(node) = stack.pop() {
        if let Some(e) = node.as_expression()
            && e.constant_value().is_some()
        {
            n += 1;
        }
        stack.extend(node.children());
    }
    n
}

fn compile(prefold: bool) -> Design {
    let session = Session::new();
    let mut comp = Compilation::new_with(&session, Options::new().with_prefold(prefold)).unwrap();
    comp.add_source(SRC).unwrap();
    comp.compile().unwrap()
}

#[test]
fn prefold_folds_more_than_bind_time() {
    let with = cached_constants(&compile(true));
    let without = cached_constants(&compile(false));
    // PREFOLD force-folds every foldable expression; without it only the
    // naturally-bound constants remain, so strictly fewer.
    assert!(
        with > without,
        "prefold should cache more constants than bind-time-only: {with} vs {without}"
    );
}

#[test]
fn no_prefold_design_is_send_sync() {
    // The no-prefold Design is still Send + Sync (constant() is a pure read).
    fn assert_send_sync<T: Send + Sync>(_: &T) {}
    let d = compile(false);
    assert_send_sync(&d);
    // constant reads work and don't require &mut.
    let _ = cached_constants(&d);
}

#[test]
fn driver_set_prefold_matches() {
    // The Driver path honours set_prefold(false) too.
    let dir = std::env::temp_dir().join("sv_lang_prefold_driver");
    std::fs::create_dir_all(&dir).unwrap();
    let src = dir.join("m.sv");
    std::fs::write(&src, SRC).unwrap();

    let mut d_pre = sv_lang::Driver::from_args([src.to_str().unwrap(), "--top", "m"]).unwrap();
    let with = cached_constants(&d_pre.compile().unwrap());

    let mut d_no = sv_lang::Driver::from_args([src.to_str().unwrap(), "--top", "m"]).unwrap();
    d_no.set_prefold(false);
    let without = cached_constants(&d_no.compile().unwrap());

    assert!(
        with > without,
        "driver prefold should cache more: {with} vs {without}"
    );
    std::fs::remove_dir_all(&dir).ok();
}
