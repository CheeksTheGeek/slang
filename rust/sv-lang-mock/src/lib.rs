//! A pure-Rust model of `sv-lang`'s handle/ownership algebra.
//!
//! The real bindings hand out `Copy` handles that are raw pointers into a slang
//! arena, tagged with a lifetime (`PhantomData<&'d Design>`) and a compilation
//! identity, behind an `Arc<DesignInner>` that is `unsafe impl Send + Sync`. Miri
//! cannot see through the FFI to check that algebra on the real crate, so this
//! crate reproduces it over a Rust-owned arena and exercises exactly the same
//! moves — concurrent shared reads, two-handle identity checks, and an
//! exclusive `&mut` session whose `eval` mutates through `&self`. Running these
//! tests under `cargo +nightly miri test` validates the pattern under both
//! Stacked and Tree Borrows.
//!
//! This is a model, not the real bindings; it proves the *shape* is sound.

use std::cell::Cell;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::Arc;

/// One arena node: a name and the indices of its child nodes.
struct NodeData {
    name: String,
    children: Vec<usize>,
    /// Interior-mutable memo, like slang's lazily-filled `mutable` fields. Only
    /// written before the design is shared, or through the exclusive session.
    folded: Cell<Option<u64>>,
}

struct DesignInner {
    arena: Vec<NodeData>,
}

// SAFETY: after construction the arena is never structurally mutated; the only
// interior mutation (`folded`) is forced before sharing or performed behind the
// exclusive `&mut` session, exactly as the real `Design` documents.
unsafe impl Send for DesignInner {}
// SAFETY: as above — shared reads never mutate.
unsafe impl Sync for DesignInner {}

/// The frozen design. `Send + Sync`, cheap to clone (refcount bump).
#[derive(Clone)]
pub struct Design {
    inner: Arc<DesignInner>,
}

/// A `Copy`, pointer-sized handle into a [`Design`]'s arena, branded with the
/// design's lifetime and carrying its compilation identity.
#[derive(Clone, Copy)]
pub struct Symbol<'d> {
    ptr: NonNull<NodeData>,
    comp: *const DesignInner,
    _design: PhantomData<&'d Design>,
}

// A handle is shareable across threads exactly when the design is.
// SAFETY: it only ever reads through the shared arena.
unsafe impl Send for Symbol<'_> {}
// SAFETY: as above.
unsafe impl Sync for Symbol<'_> {}

impl Design {
    /// Builds a design from `(name, children-indices)` pairs, forcing every
    /// memo before the design can be shared (the totalization the real freeze
    /// performs).
    pub fn new(nodes: Vec<(&str, Vec<usize>)>) -> Design {
        let arena = nodes
            .into_iter()
            .enumerate()
            .map(|(i, (name, children))| NodeData {
                name: name.to_string(),
                children,
                // Force the memo up front — post-sharing reads are pure.
                folded: Cell::new(Some(i as u64)),
            })
            .collect();
        Design {
            inner: Arc::new(DesignInner { arena }),
        }
    }

    fn handle(&self, index: usize) -> Symbol<'_> {
        let node = &self.inner.arena[index];
        Symbol {
            ptr: NonNull::from(node),
            comp: Arc::as_ptr(&self.inner),
            _design: PhantomData,
        }
    }

    /// The root handle (node 0).
    pub fn root(&self) -> Symbol<'_> {
        self.handle(0)
    }

    /// An exclusive evaluation session (`&mut`), through which memos may be
    /// (re)computed. `Send` but not `Sync`.
    pub fn eval_session(&mut self) -> EvalSession<'_> {
        EvalSession {
            design: self,
            _not_sync: PhantomData,
        }
    }
}

impl<'d> Symbol<'d> {
    fn data(&self) -> &'d NodeData {
        // SAFETY: the node lives in the arena which outlives 'd (we hold a
        // `PhantomData<&'d Design>`), and is never moved or freed while borrowed.
        unsafe { self.ptr.as_ref() }
    }

    /// The node's name — a pure shared read.
    pub fn name(&self) -> &'d str {
        &self.data().name
    }

    /// The already-forced memo — a pure shared read (never mutates).
    pub fn folded(&self) -> Option<u64> {
        self.data().folded.get()
    }

    /// Child handles, branded with the same design lifetime.
    pub fn children(&self, design: &'d Design) -> Vec<Symbol<'d>> {
        self.data()
            .children
            .iter()
            .map(|&i| design.handle(i))
            .collect()
    }

    /// Two-handle identity check: two handles are equal only when they point at
    /// the same node of the same compilation (a cross-design comparison is
    /// always unequal, never UB).
    pub fn same(&self, other: &Symbol<'d>) -> bool {
        self.comp == other.comp && self.ptr == other.ptr
    }
}

impl PartialEq for Symbol<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.same(other)
    }
}

impl std::fmt::Debug for Symbol<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Symbol({})", self.name())
    }
}

/// An exclusive session over a [`Design`]. Holds a `&mut`, so no shared reader
/// can exist concurrently; `Cell<()>` makes it `!Sync` so its `&self`-mutation
/// cannot be shared across threads.
pub struct EvalSession<'d> {
    design: &'d mut Design,
    _not_sync: PhantomData<Cell<()>>,
}

impl<'d> EvalSession<'d> {
    /// Read access to the design.
    pub fn design(&self) -> &Design {
        self.design
    }

    /// Recomputes a node's memo through `&self` (interior mutation), after
    /// checking the handle belongs to this session's design — the exact move
    /// `EvalSession::eval` makes on the real crate.
    pub fn refold(&self, sym: Symbol<'_>, value: u64) {
        assert!(
            sym.comp == Arc::as_ptr(&self.design.inner),
            "handle belongs to a different design"
        );
        sym.data().folded.set(Some(value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Design {
        Design::new(vec![("top", vec![1, 2]), ("a", vec![]), ("b", vec![])])
    }

    #[test]
    fn concurrent_shared_reads_are_sound() {
        let design = sample();
        let total: usize = std::thread::scope(|scope| {
            (0..4)
                .map(|_| {
                    let d = &design;
                    scope.spawn(move || {
                        let root = d.root();
                        let mut n = root.name().len();
                        for c in root.children(d) {
                            n += c.name().len() + c.folded().unwrap_or(0) as usize;
                        }
                        n
                    })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|h| h.join().unwrap())
                .sum()
        });
        assert!(total > 0);
    }

    #[test]
    fn handles_are_copy_and_compare_by_identity() {
        let design = sample();
        let root = design.root();
        let root2 = root; // Copy
        assert_eq!(root, root2);
        let kids = root.children(&design);
        assert_ne!(kids[0], kids[1]);
    }

    #[test]
    fn cross_design_handles_are_unequal() {
        let a = sample();
        let b = sample();
        // Same index, same name, different design → not equal, no UB.
        assert_ne!(a.root(), b.root());
    }

    #[test]
    fn exclusive_session_mutates_through_shared_ref() {
        let mut design = sample();
        let session = design.eval_session();
        let root = session.design().root();
        session.refold(root, 999);
        assert_eq!(session.design().root().folded(), Some(999));
    }
}
