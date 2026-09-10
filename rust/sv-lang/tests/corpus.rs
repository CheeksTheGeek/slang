//! Round-trip property test over a SystemVerilog corpus: every source file
//! parses, its tree text reproduces the file byte-for-byte, its pure-Rust
//! mirror reproduces the expanded token stream, and the two trees agree on
//! node count.
//!
//! Two corpora run:
//!  * slang's own `tests/` tree (present in a repo checkout; a no-op from a
//!    published crate). A parse or read failure here is a **hard error** unless
//!    the file is in [`KNOWN_UNPARSEABLE`] with a stated reason — skips are
//!    never silent.
//!  * an external corpus named by the `SV_LANG_CORPUS` env var (point it at
//!    ibex / OpenTitan / CVA6 / UVM). Every file there must parse and
//!    round-trip; there is no allowlist.

use std::path::{Path, PathBuf};

use sv_lang::Session;
use sv_lang::green::NodeExt;
use sv_lang::kinds::SyntaxKind;

/// Files in slang's own corpus that intentionally cannot be parsed as a
/// standalone compilation unit (fragments, deliberate hard-error regressions).
/// Each entry is a path suffix + the reason. Keeping the list explicit is the
/// point: a newly-skipped file fails the test until it is triaged here.
const KNOWN_UNPARSEABLE: &[(&str, &str)] = &[
    // (populated by triage; empty means every in-tree source file must parse)
];

fn is_known_unparseable(path: &Path) -> Option<&'static str> {
    let s = path.to_string_lossy().replace('\\', "/");
    KNOWN_UNPARSEABLE
        .iter()
        .find(|(suffix, _)| s.ends_with(suffix))
        .map(|(_, reason)| *reason)
}

fn in_tree_corpus() -> Option<PathBuf> {
    // CARGO_MANIFEST_DIR is rust/sv-lang; the corpus is <repo>/tests.
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests");
    dir.is_dir().then(|| dir.canonicalize().unwrap())
}

fn source_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&path) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else if matches!(
                p.extension().and_then(|e| e.to_str()).unwrap_or(""),
                "sv" | "v" | "svh"
            ) {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

/// Runs the round-trip checks over `files`. `strict` = every file must parse
/// (external corpus); otherwise [`KNOWN_UNPARSEABLE`] entries are tolerated but
/// counted. Returns `(ok, skipped)`; pushes human-readable messages for any
/// unexpected failure into `failures`.
fn check_corpus(
    session: &Session,
    files: &[PathBuf],
    strict: bool,
    failures: &mut Vec<String>,
) -> (usize, usize) {
    let mut ok = 0usize;
    let mut skipped = 0usize;

    for p in files {
        let src = match std::fs::read_to_string(p) {
            Ok(s) => s,
            Err(e) => {
                failures.push(format!("read error for {}: {e}", p.display()));
                continue;
            }
        };
        // Put the path in the name slot (for diagnostics) but leave the
        // registration path empty: several corpus fragments are `include`d by
        // other files, which registers their path in the shared source manager,
        // and re-registering the same path is rejected.
        let tree = match session.parse_named(&src, p.to_str().unwrap_or("corpus"), "") {
            Ok(t) => t,
            Err(e) => {
                if !strict && let Some(reason) = is_known_unparseable(p) {
                    let _ = reason;
                    skipped += 1;
                } else {
                    failures.push(format!("parse failed for {}: {e}", p.display()));
                }
                continue;
            }
        };

        // The tree reproduces the original source exactly.
        if tree.text() != src {
            failures.push(format!("tree text differs for {}", p.display()));
            continue;
        }
        // The mirror reproduces the tree's expanded token stream...
        let green = tree.mirror();
        if green.text().to_string() != tree.root().text() {
            failures.push(format!("mirror text differs for {}", p.display()));
            continue;
        }
        // ...and is a structural copy with the same node count.
        let tree_nodes = tree.root().descendants().count();
        let green_nodes = green
            .descendants()
            .filter(|n| n.node_kind().is_some())
            .count();
        if tree_nodes != green_nodes {
            failures.push(format!(
                "node count differs for {} ({tree_nodes} vs {green_nodes})",
                p.display()
            ));
            continue;
        }
        if tree.root().kind() != SyntaxKind::CompilationUnit {
            failures.push(format!("root is not a CompilationUnit for {}", p.display()));
            continue;
        }
        ok += 1;
    }
    (ok, skipped)
}

#[test]
fn corpus_round_trips() {
    let Some(dir) = in_tree_corpus() else {
        eprintln!("no in-tree corpus; skipping");
        return;
    };

    let session = Session::new();
    let files = source_files(&dir);
    let mut failures = Vec::new();
    let (ok, skipped) = check_corpus(&session, &files, false, &mut failures);

    assert!(
        failures.is_empty(),
        "corpus round-trip failed for {} file(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
    assert!(ok > 50, "expected a substantial corpus, saw {ok} files");
    eprintln!("in-tree corpus: {ok} files round-tripped, {skipped} known-unparseable skipped");
}

/// The vendored real-world slice (`rust/corpus/`, always-on): every file must
/// parse and round-trip byte-exact — no allowlist. Proves the bindings work on
/// genuine RTL, not just slang's small unit-test files.
#[test]
fn curated_corpus_round_trips() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../corpus");
    if !dir.is_dir() {
        eprintln!("no curated corpus; skipping");
        return;
    }
    let session = Session::new();
    let files = source_files(&dir);
    assert!(!files.is_empty(), "rust/corpus/ has no source files");
    let mut failures = Vec::new();
    let (ok, _) = check_corpus(&session, &files, true, &mut failures);
    assert!(
        failures.is_empty(),
        "curated corpus round-trip failed for {} file(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
    eprintln!("curated corpus: {ok} real-world files round-tripped");
}

#[test]
fn external_corpus_round_trips() {
    let Some(root) = std::env::var_os("SV_LANG_CORPUS").map(PathBuf::from) else {
        eprintln!("SV_LANG_CORPUS not set; skipping external corpus");
        return;
    };
    assert!(
        root.is_dir(),
        "SV_LANG_CORPUS={} is not a directory",
        root.display()
    );

    let session = Session::new();
    let files = source_files(&root);
    assert!(
        !files.is_empty(),
        "SV_LANG_CORPUS contains no .sv/.v/.svh files"
    );
    let mut failures = Vec::new();
    let (ok, _) = check_corpus(&session, &files, true, &mut failures);

    assert!(
        failures.is_empty(),
        "external corpus failed for {} of {} file(s):\n{}",
        failures.len(),
        files.len(),
        failures.join("\n")
    );
    eprintln!("external corpus: {ok} files round-tripped");
}
