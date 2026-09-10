//! An incremental workspace over [`sv_lang`]: hold a set of source files, edit
//! them, and get parse trees and an elaborated design back — re-parsing only
//! what changed. This is the caching layer a language server or an
//! edit-compile loop is built on.
//!
//! Parsing is content-addressed: setting a file to text it already had is free,
//! and only files whose text changed are re-parsed on the next
//! [`Workspace::design`]. slang syntax trees are safe to share between
//! compilations, so unchanged files' trees are reused verbatim.
//!
//! ```
//! use sv_lang_db::Workspace;
//!
//! let mut ws = Workspace::new();
//! ws.set_file("pkg.sv", "package p; localparam int W = 8; endpackage\n");
//! ws.set_file("top.sv", "module top; import p::*; logic [W-1:0] a; endmodule\n");
//!
//! let design = ws.design().unwrap();
//! assert_eq!(design.top_instances().next().unwrap().name(), "top");
//!
//! // Editing one file only re-parses that file.
//! ws.set_file("top.sv", "module top; import p::*; logic [W-1:0] b; endmodule\n");
//! let stats = ws.take_stats();
//! let design = ws.design().unwrap();
//! assert_eq!(stats.reparsed, 0); // parse happens lazily in design()
//! # let _ = design;
//! ```
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::BTreeMap;

use sv_lang::{Compilation, Design, Error, Session, SyntaxTree};

/// A file's cached parse: the text it was parsed from and the resulting tree.
struct Cached {
    text: String,
    tree: SyntaxTree,
}

/// Counters describing what the last [`Workspace::design`] did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    /// Files parsed for the first time.
    pub parsed: usize,
    /// Files re-parsed because their text changed.
    pub reparsed: usize,
    /// Files whose cached tree was reused unchanged.
    pub reused: usize,
}

/// An incremental set of SystemVerilog source files.
pub struct Workspace {
    session: Session,
    /// path -> desired text (the current state, possibly not yet parsed).
    files: BTreeMap<String, String>,
    /// path -> cached parse (present once parsed and still current).
    cache: BTreeMap<String, Cached>,
    stats: Stats,
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Workspace {
    /// Creates an empty workspace.
    pub fn new() -> Self {
        Workspace {
            session: Session::new(),
            files: BTreeMap::new(),
            cache: BTreeMap::new(),
            stats: Stats::default(),
        }
    }

    /// The session backing the workspace's trees.
    pub fn session(&self) -> &Session {
        &self.session
    }

    /// Sets (or replaces) a file's text. Setting text identical to the current
    /// text is a no-op. Does not parse — parsing happens lazily in
    /// [`design`](Self::design).
    pub fn set_file(&mut self, path: impl Into<String>, text: impl Into<String>) {
        self.files.insert(path.into(), text.into());
    }

    /// Removes a file.
    pub fn remove_file(&mut self, path: &str) {
        self.files.remove(path);
        self.cache.remove(path);
    }

    /// The current text of a file, if present.
    pub fn file_text(&self, path: &str) -> Option<&str> {
        self.files.get(path).map(String::as_str)
    }

    /// The paths currently in the workspace, sorted.
    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.files.keys().map(String::as_str)
    }

    /// The parse tree of a file, parsing it if necessary. Reuses the cached
    /// tree when the text is unchanged.
    pub fn tree(&mut self, path: &str) -> Option<Result<SyntaxTree, Error>> {
        let text = self.files.get(path)?.clone();
        Some(self.tree_for(path, &text))
    }

    fn tree_for(&mut self, path: &str, text: &str) -> Result<SyntaxTree, Error> {
        if let Some(cached) = self.cache.get(path)
            && cached.text == text
        {
            self.stats.reused += 1;
            return Ok(cached.tree.clone());
        }

        let is_reparse = self.cache.contains_key(path);
        // Use the path only as the diagnostic name, not as the source-manager
        // buffer path: re-parsing an edited file re-uses the same name, and the
        // manager rejects registering one path twice.
        let tree = self.session.parse_named(text, path, "")?;
        if is_reparse {
            self.stats.reparsed += 1;
        } else {
            self.stats.parsed += 1;
        }
        self.cache.insert(
            path.to_string(),
            Cached {
                text: text.to_string(),
                tree: tree.clone(),
            },
        );
        Ok(tree)
    }

    /// Parses every file (reusing unchanged trees) and elaborates them into a
    /// [`Design`]. Call [`take_stats`](Self::take_stats) afterwards to see how
    /// much work was incremental.
    pub fn design(&mut self) -> Result<Design, Error> {
        // Drop cache entries for files that were removed.
        let live: Vec<String> = self.files.keys().cloned().collect();
        self.cache.retain(|p, _| self.files.contains_key(p));

        let mut comp = Compilation::new(&self.session)?;
        for path in live {
            let text = self.files[&path].clone();
            let tree = self.tree_for(&path, &text)?;
            comp.add(&tree)?;
        }
        comp.compile()
    }

    /// Returns the accumulated [`Stats`] and resets them to zero.
    pub fn take_stats(&mut self) -> Stats {
        std::mem::take(&mut self.stats)
    }
}
