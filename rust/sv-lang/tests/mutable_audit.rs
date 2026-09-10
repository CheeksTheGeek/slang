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
const BASELINE_MUTABLE: usize = 139;

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
    // The classification doc must exist and mention the invariant (a cheap tripwire
    // so the audit can't be silently deleted).
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../SOUNDNESS-MEMOS.md");
    let doc_text = std::fs::read_to_string(&doc).expect("rust/SOUNDNESS-MEMOS.md must exist");
    assert!(
        doc_text.contains("FreezeVisitor") && doc_text.contains("B-latent"),
        "SOUNDNESS-MEMOS.md is missing the memo classification"
    );
}
