//! Guards the soundness invariant recorded in `rust/SOUNDNESS-MEMOS.md`: the
//! number of `mutable` fields in slang's public headers is a proxy for how much
//! lazy, const-path mutation the frozen-`Design` `Sync` claim has to account
//! for. Every such memo must be classified there (forced-by-sweep /
//! gated-behind-`&mut` / not-reached-by-any-C-accessor). If this count rises
//! upstream, a NEW unclassified `mutable` may have appeared — this test fails
//! and forces a review of `SOUNDNESS-MEMOS.md` (and a matching `FreezeVisitor`
//! force if a value-returning `&Design` accessor now reaches it) before the
//! `Sync` guarantee is re-shipped.
//!
//! When building from a published crate (no repository present) the test is a
//! no-op.

use std::path::PathBuf;

/// The audited baseline. Update this ONLY together with a review of the new
/// `mutable` fields and, if needed, SAFETY.md.
const BASELINE_MUTABLE: usize = 140;

/// The audited size of the public accessor surface (`pub fn` in `ast.rs`). The
/// `mutable`-count check catches a NEW lazy memo appearing UPSTREAM, but not a
/// new Rust `&Design`/`&self` accessor that reaches an EXISTING unforced memo.
/// This tripwire fires whenever the accessor surface grows: the author must then
/// confirm any new value-returning `&Design` accessor has a matching
/// `FreezeVisitor` force (per SOUNDNESS-MEMOS.md's B-latent rule) before bumping
/// this baseline. Coarse by design — it forces the review, it does not perform it.
const BASELINE_AST_ACCESSORS: usize = 850;

#[test]
fn mutable_field_count_is_audited() {
    // include/ is <repo>/include; this test crate is <repo>/rust/sv-lang.
    let include = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../include/slang");
    if !include.is_dir() {
        eprintln!("no in-tree headers; skipping mutable audit");
        return;
    }

    let mut count = 0usize;
    let mut stack = vec![include];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().and_then(|e| e.to_str()) == Some("h") {
                if let Ok(text) = std::fs::read_to_string(&p) {
                    count += text
                        .split(|c: char| !c.is_alphanumeric() && c != '_')
                        .filter(|w| *w == "mutable")
                        .count();
                }
            }
        }
    }

    assert!(
        count <= BASELINE_MUTABLE,
        "slang gained `mutable` fields ({count} > audited baseline {BASELINE_MUTABLE}).\n\
         Classify each new field in rust/SOUNDNESS-MEMOS.md (forced-by-sweep / gated / \
         not-reached) and, if a value-returning &Design accessor now reaches it, add a \
         matching FreezeVisitor force; then update BASELINE_MUTABLE."
    );
    // The public accessor surface tripwire (see BASELINE_AST_ACCESSORS).
    let ast_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/ast.rs");
    let accessors = std::fs::read_to_string(&ast_rs)
        .expect("src/ast.rs is part of this crate")
        .matches("pub fn ")
        .count();
    assert!(
        accessors <= BASELINE_AST_ACCESSORS,
        "the public accessor surface grew ({accessors} > audited baseline \
         {BASELINE_AST_ACCESSORS}).\nFor each new value-returning &Design/&self accessor, \
         confirm the lazy memo it reads is forced in FreezeVisitor (pre-seal if it \
         allocates) per the B-latent rule in rust/SOUNDNESS-MEMOS.md, then bump \
         BASELINE_AST_ACCESSORS."
    );

    // The classification doc, when present, must mention the invariant (a cheap
    // tripwire so it can't be silently gutted). SOUNDNESS-MEMOS.md is kept
    // local-only (not committed), so this check is skipped when it is absent —
    // e.g. in CI or a published-crate checkout.
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../SOUNDNESS-MEMOS.md");
    if let Ok(doc_text) = std::fs::read_to_string(&doc) {
        assert!(
            doc_text.contains("FreezeVisitor") && doc_text.contains("B-latent"),
            "SOUNDNESS-MEMOS.md is missing the memo classification"
        );
    } else {
        eprintln!("SOUNDNESS-MEMOS.md not present (local-only); skipping the doc tripwire");
    }
}
