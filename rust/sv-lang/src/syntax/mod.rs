//! The concrete syntax tree: [`SyntaxTree`] and the borrowed [`Node`] / [`Token`]
//! cursors over it, plus mirroring into the pure-Rust [`sv_lang_syntax`] tree.

use core::marker::PhantomData;
use std::sync::Arc;

use sv_lang_kinds::{SyntaxKind, SyntaxStruct, TokenKind, TriviaKind};
use sv_lang_sys as sys;

use crate::{Diagnostics, Session, ffi};

mod generated {
    #![allow(clippy::all, missing_docs)]
    pub mod nodes_0;
    pub mod nodes_1;
    pub mod nodes_2;
    pub mod nodes_3;
    pub mod nodes_4;
    pub mod nodes_5;
    pub mod nodes_6;
    pub mod nodes_7;
    pub mod visitor;
}

/// A generated `syn::visit`-style visitor over the typed syntax tree: a
/// `Visitor` trait with one `visit_*` method per node kind (each defaulting to
/// walk into children), the `walk_*` free functions, and the `visit` entry
/// point that dispatches a [`Node`] to the right method.
///
/// ```
/// # let session = sv_lang::Session::new();
/// # let tree = session.parse("module a; endmodule\nmodule b; endmodule\n").unwrap();
/// use sv_lang::visitor::{Visitor, visit, walk_module_declaration_syntax};
/// use sv_lang::nodes::ModuleDeclarationSyntax;
///
/// #[derive(Default)]
/// struct ModuleCounter { count: usize }
/// impl<'t> Visitor<'t> for ModuleCounter {
///     fn visit_module_declaration_syntax(&mut self, node: ModuleDeclarationSyntax<'t>) {
///         self.count += 1;
///         walk_module_declaration_syntax(self, node);
///     }
/// }
///
/// let mut counter = ModuleCounter::default();
/// visit(&mut counter, tree.root());
/// assert_eq!(counter.count, 2);
/// ```
pub mod visitor {
    pub use super::generated::visitor::*;
}

/// Typed views of every slang syntax node — one `#[repr(transparent)]` struct
/// per concrete syntax kind and one enum per abstract base — generated from
/// slang's schema. Each is a thin, `Copy` typed wrapper over a [`Node`].
pub mod nodes {
    pub use super::generated::nodes_0::*;
    pub use super::generated::nodes_1::*;
    pub use super::generated::nodes_2::*;
    pub use super::generated::nodes_3::*;
    pub use super::generated::nodes_4::*;
    pub use super::generated::nodes_5::*;
    pub use super::generated::nodes_6::*;
    pub use super::generated::nodes_7::*;
}

/// A parsed, lossless syntax tree. Cloning is cheap (a reference-count bump);
/// the tree is immutable after parsing and safe to share across threads.
///
/// ```
/// # fn main() -> Result<(), sv_lang::Error> {
/// let session = sv_lang::Session::new();
/// let tree = session.parse("module counter; endmodule\n")?;
/// assert_eq!(tree.module_names().collect::<Vec<_>>(), ["counter"]);
/// # Ok(()) }
/// ```
#[derive(Clone)]
pub struct SyntaxTree {
    inner: Arc<TreeInner>,
}

struct TreeInner {
    raw: sys::slang_syntax_tree,
    // Keeps the source manager alive for as long as the tree.
    session: Session,
}

// SAFETY: a parsed SyntaxTree is immutable, and its source manager is
// internally synchronized. The raw handle is reference-counted on the C side;
// TreeInner holds exactly one reference, released on drop.
unsafe impl Send for TreeInner {}
// SAFETY: as above — the tree is immutable after parsing.
unsafe impl Sync for TreeInner {}

impl Drop for TreeInner {
    fn drop(&mut self) {
        // SAFETY: releases the one reference this handle owns.
        unsafe { sys::slang_syntax_tree_release(self.raw) };
    }
}

impl core::fmt::Debug for SyntaxTree {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SyntaxTree")
            .field("root", &self.root().kind())
            .finish()
    }
}

impl core::fmt::Debug for Node<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Node({})", self.kind())
    }
}

// Node identity is its address within the tree.
impl PartialEq for Node<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.raw.ptr == other.raw.ptr
    }
}
impl Eq for Node<'_> {}
impl core::hash::Hash for Node<'_> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.raw.ptr.hash(state);
    }
}

// The raw-node "typed view": every node is trivially a `Node`.
impl<'t> AstNode<'t> for Node<'t> {
    fn can_cast(_kind: SyntaxKind) -> bool {
        true
    }
    fn cast(node: Node<'t>) -> Option<Self> {
        Some(node)
    }
    fn syntax(&self) -> Node<'t> {
        *self
    }
}

impl core::fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Token({}, {:?})", self.kind(), self.text())
    }
}

impl SyntaxTree {
    pub(crate) fn from_raw(raw: sys::slang_syntax_tree, session: Session) -> SyntaxTree {
        SyntaxTree {
            inner: Arc::new(TreeInner { raw, session }),
        }
    }

    pub(crate) fn raw(&self) -> sys::slang_syntax_tree {
        self.inner.raw
    }

    /// The session this tree was parsed with.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// // The tree's session parses further source into the same manager.
    /// let other = tree.session().parse("module n; endmodule\n")?;
    /// assert_eq!(other.module_names().collect::<Vec<_>>(), ["n"]);
    /// # Ok(()) }
    /// ```
    pub fn session(&self) -> &Session {
        &self.inner.session
    }

    /// The root node (a `CompilationUnit`).
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::kinds::SyntaxKind;
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// assert_eq!(tree.root().kind(), SyntaxKind::CompilationUnit);
    /// # Ok(()) }
    /// ```
    pub fn root(&self) -> Node<'_> {
        // SAFETY: the tree is live.
        let raw = unsafe { sys::slang_syntax_tree_root(self.raw()) };
        Node {
            raw,
            _tree: PhantomData,
        }
    }

    /// The diagnostics produced while lexing, preprocessing and parsing.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// assert!(!tree.diagnostics().has_errors());
    /// # Ok(()) }
    /// ```
    pub fn diagnostics(&self) -> Diagnostics {
        let mut err = ffi::error();
        // SAFETY: the tree is live; the returned handle is consumed by collect.
        let diags = unsafe { sys::slang_syntax_tree_diagnostics(self.raw(), &mut err) };
        if diags.is_null() {
            return Diagnostics::default();
        }
        crate::collect_diagnostics(self.inner.session.raw(), diags)
    }

    /// The exact source text of the tree (byte-identical to the input for an
    /// unmodified tree).
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let src = "module m; endmodule\n";
    /// let tree = session.parse(src)?;
    /// assert_eq!(tree.text(), src);
    /// # Ok(()) }
    /// ```
    pub fn text(&self) -> String {
        let mut err = ffi::error();
        // SAFETY: the tree is live; the returned string is owned.
        unsafe { ffi::owned_str(sys::slang_syntax_tree_to_string(self.raw(), &mut err)) }
    }

    /// The names of the top-level module declarations, in source order.
    ///
    /// This is a convenience over the syntax tree; the semantic API (later
    /// releases) resolves the actual design hierarchy.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module a; endmodule\nmodule b; endmodule\n")?;
    /// assert_eq!(tree.module_names().collect::<Vec<_>>(), ["a", "b"]);
    /// # Ok(()) }
    /// ```
    pub fn module_names(&self) -> impl Iterator<Item = &str> {
        self.root()
            .children()
            .filter(|n| n.kind() == SyntaxKind::ModuleDeclaration)
            .filter_map(|decl| decl.module_name())
    }

    /// Mirrors this tree into a standalone [`sv_lang_syntax`] tree that does not
    /// borrow slang. Use this to keep a tree after the session is dropped, or
    /// to edit and print it with pure-Rust tooling.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let src = "module m; endmodule\n";
    /// let tree = session.parse(src)?;
    /// let mirrored = tree.mirror(); // standalone; no longer borrows slang
    /// drop(session);
    /// assert_eq!(mirrored.to_string(), src);
    /// # Ok(()) }
    /// ```
    pub fn mirror(&self) -> sv_lang_syntax::ResolvedNode {
        let mut sink = mirror::MirrorSink::new();
        let cbs = sys::slang_syntax_sink {
            start_node: Some(mirror::start_node),
            trivia: Some(mirror::trivia),
            token: Some(mirror::token),
            absent: Some(mirror::absent),
            start_list: Some(mirror::start_list),
            finish_list: Some(mirror::finish_list),
            finish_node: Some(mirror::finish_node),
        };
        let mut err = ffi::error();
        // SAFETY: `sink` outlives the call; every callback is set and none unwind
        // across the C boundary (each catches panics into `sink.panic`).
        unsafe {
            sys::slang_syntax_tree_walk(
                self.raw(),
                &cbs,
                (&mut sink as *mut mirror::MirrorSink).cast(),
                &mut err,
            );
        }
        // A builder panic was caught in a callback to keep it from unwinding
        // through C; re-raise it now that the C walk has returned normally.
        if let Some(p) = sink.panic {
            std::panic::resume_unwind(p);
        }
        sink.builder.finish()
    }
}

/// A node cursor into a [`SyntaxTree`], valid while the tree is borrowed.
/// `Copy` and pointer-sized; cheap to pass around.
///
/// ```
/// # fn main() -> Result<(), sv_lang::Error> {
/// use sv_lang::kinds::SyntaxKind;
/// let session = sv_lang::Session::new();
/// let tree = session.parse("module m; endmodule\n")?;
/// let root = tree.root();
/// assert_eq!(root.kind(), SyntaxKind::CompilationUnit);
/// assert_eq!(root.children().count(), 1);
/// # Ok(()) }
/// ```
#[derive(Clone, Copy)]
pub struct Node<'t> {
    raw: sys::slang_node,
    _tree: PhantomData<&'t SyntaxTree>,
}

/// A token cursor into a [`SyntaxTree`].
///
/// ```
/// # fn main() -> Result<(), sv_lang::Error> {
/// let session = sv_lang::Session::new();
/// let tree = session.parse("module m; endmodule\n")?;
/// let first = tree.root().first_token().unwrap();
/// assert_eq!(first.text(), "module");
/// # Ok(()) }
/// ```
#[derive(Clone, Copy)]
pub struct Token<'t> {
    raw: sys::slang_token,
    _tree: PhantomData<&'t SyntaxTree>,
}

/// A child of a node: either a nested node or a token.
///
/// ```
/// # fn main() -> Result<(), sv_lang::Error> {
/// use sv_lang::Child;
/// let session = sv_lang::Session::new();
/// let tree = session.parse("module m; endmodule\n")?;
/// assert!(matches!(tree.root().child(0), Some(Child::Node(_))));
/// # Ok(()) }
/// ```
#[derive(Clone, Copy)]
pub enum Child<'t> {
    /// A nested node.
    Node(Node<'t>),
    /// A token.
    Token(Token<'t>),
}

/// How a visitor callback steers the traversal (see [`Node::visit`]).
///
/// ```
/// use sv_lang::Walk;
/// // `Skip` visits the node but not its children; `Continue` descends.
/// assert_ne!(Walk::Skip, Walk::Continue);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Walk {
    /// Descend into this node's children, then continue.
    Continue,
    /// Do not descend into this node's children, but continue the walk.
    Skip,
    /// Stop the whole walk.
    Break,
}

enum ControlOutcome {
    Ran,
    Broke,
}

impl<'t> Node<'t> {
    fn wrap(raw: sys::slang_node) -> Node<'t> {
        Node {
            raw,
            _tree: PhantomData,
        }
    }

    /// Builds a node cursor from a raw handle. The caller guarantees the raw
    /// node belongs to a tree borrowed for `'t`.
    pub(crate) fn from_raw_node(raw: sys::slang_node) -> Node<'t> {
        Node::wrap(raw)
    }

    /// The raw handle, for passing to a C accessor that takes a `slang_node`.
    pub(crate) fn raw(&self) -> sys::slang_node {
        self.raw
    }

    /// The node's syntax kind.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::kinds::SyntaxKind;
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// let module = tree.root().children().next().unwrap();
    /// assert_eq!(module.kind(), SyntaxKind::ModuleDeclaration);
    /// # Ok(()) }
    /// ```
    pub fn kind(&self) -> SyntaxKind {
        SyntaxKind::from_raw(self.raw.kind as u16).unwrap_or(SyntaxKind::Unknown)
    }

    /// The node's source span as a byte range into the buffer it was parsed
    /// from — the basis for mapping a node to an editor range (see
    /// `sv-lang-lsp`). An empty range for a node with no location.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let src = "module counter; endmodule\n";
    /// let tree = session.parse(src)?;
    /// let module = tree.root().children().next().unwrap();
    /// let range = module.byte_range();
    /// assert_eq!(&src[range], "module counter; endmodule");
    /// # Ok(()) }
    /// ```
    pub fn byte_range(&self) -> core::ops::Range<usize> {
        // SAFETY: the node is valid for the borrowed tree.
        let r = unsafe { sys::slang_node_range(self.raw) };
        (r.start.offset as usize)..(r.end.offset as usize)
    }

    /// The generated struct this node is an instance of.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::kinds::SyntaxStruct;
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// assert_eq!(tree.root().syntax_struct(), Some(SyntaxStruct::CompilationUnitSyntax));
    /// # Ok(()) }
    /// ```
    pub fn syntax_struct(&self) -> Option<SyntaxStruct> {
        self.kind().syntax_struct()
    }

    /// The parent node, or `None` for the root.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// let root = tree.root();
    /// let module = root.children().next().unwrap();
    /// assert_eq!(module.parent(), Some(root));
    /// assert_eq!(root.parent(), None);
    /// # Ok(()) }
    /// ```
    pub fn parent(&self) -> Option<Node<'t>> {
        // SAFETY: the node is valid.
        let p = unsafe { sys::slang_node_parent(self.raw) };
        (!p.ptr.is_null()).then(|| Node::wrap(p))
    }

    /// The number of direct children (nodes and tokens, absent slots included).
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// // the compilation unit holds the module and the end-of-file token.
    /// assert_eq!(tree.root().child_count(), 2);
    /// # Ok(()) }
    /// ```
    pub fn child_count(&self) -> usize {
        // SAFETY: the node is valid.
        unsafe { sys::slang_node_child_count(self.raw) as usize }
    }

    /// The child at `index`, if present (an absent optional slot yields `None`).
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::{Child, kinds::SyntaxKind};
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// match tree.root().child(0) {
    ///     Some(Child::Node(n)) => assert_eq!(n.kind(), SyntaxKind::ModuleDeclaration),
    ///     _ => panic!("expected a node child"),
    /// }
    /// # Ok(()) }
    /// ```
    pub fn child(&self, index: usize) -> Option<Child<'t>> {
        let mut node = sys::slang_node {
            ptr: core::ptr::null(),
            tree: self.raw.tree,
            kind: 0,
            reserved_: 0,
        };
        let mut token = sys::slang_token {
            owner: core::ptr::null(),
            tree: self.raw.tree,
            index: 0,
            kind: 0,
            flags: 0,
        };
        // SAFETY: the node is valid; both out-params are provided.
        let tag = unsafe { sys::slang_node_child(self.raw, index as u32, &mut node, &mut token) };
        match tag {
            sys::SLANG_CHILD_NODE => Some(Child::Node(Node::wrap(node))),
            sys::SLANG_CHILD_TOKEN => Some(Child::Token(Token {
                raw: token,
                _tree: PhantomData,
            })),
            _ => None,
        }
    }

    /// Iterates the direct child nodes (skipping tokens and absent slots).
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module a; endmodule\nmodule b; endmodule\n")?;
    /// // two module declarations (the trailing EOF token is skipped).
    /// assert_eq!(tree.root().children().count(), 2);
    /// # Ok(()) }
    /// ```
    pub fn children(self) -> impl Iterator<Item = Node<'t>> {
        (0..self.child_count()).filter_map(move |i| match self.child(i) {
            Some(Child::Node(n)) => Some(n),
            _ => None,
        })
    }

    /// Iterates every direct child (nodes and tokens, absent slots skipped).
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::Child;
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// // the module node plus the end-of-file token.
    /// let tokens = tree.root().children_with_tokens()
    ///     .filter(|c| matches!(c, Child::Token(_))).count();
    /// assert_eq!(tokens, 1);
    /// # Ok(()) }
    /// ```
    pub fn children_with_tokens(self) -> impl Iterator<Item = Child<'t>> {
        (0..self.child_count()).filter_map(move |i| self.child(i))
    }

    /// A pre-order iterator over this node and all its descendant nodes.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::kinds::SyntaxKind;
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// assert!(tree.root().descendants().any(|n| n.kind() == SyntaxKind::ModuleDeclaration));
    /// # Ok(()) }
    /// ```
    pub fn descendants(&self) -> Descendants<'t> {
        Descendants { stack: vec![*self] }
    }

    /// Visits this node and its descendants in pre-order, calling `f` on each.
    ///
    /// The callback returns a [`Walk`] to steer the traversal: [`Walk::Continue`]
    /// descends into the node's children, [`Walk::Skip`] does not, and
    /// [`Walk::Break`] stops the whole walk. Returns `true` if the walk ran to
    /// completion, `false` if a callback broke out.
    ///
    /// ```
    /// # let session = sv_lang::Session::new();
    /// # let tree = session.parse("module a; endmodule\nmodule b; endmodule\n").unwrap();
    /// use sv_lang::{Walk, kinds::SyntaxKind};
    /// let mut count = 0;
    /// tree.root().visit(|node| {
    ///     if node.kind() == SyntaxKind::ModuleDeclaration {
    ///         count += 1;
    ///         sv_lang::Walk::Skip // don't descend into module bodies
    ///     } else {
    ///         Walk::Continue
    ///     }
    /// });
    /// assert_eq!(count, 2);
    /// ```
    pub fn visit(self, mut f: impl FnMut(Node<'t>) -> Walk) -> bool {
        let mut stack = vec![self];
        while let Some(node) = stack.pop() {
            match f(node) {
                Walk::Break => return false,
                Walk::Skip => continue,
                Walk::Continue => {
                    let children: Vec<_> = node.children().collect();
                    stack.extend(children.into_iter().rev());
                }
            }
        }
        true
    }

    /// Visits this node and every descendant node and token in source order.
    /// Like [`visit`](Self::visit) but the callback also sees tokens (a token
    /// is a leaf, so [`Walk::Skip`] and [`Walk::Continue`] behave the same on
    /// one).
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::{Child, Walk};
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// let mut tokens = 0;
    /// tree.root().visit_with_tokens(|c| {
    ///     if let Child::Token(_) = c { tokens += 1; }
    ///     Walk::Continue
    /// });
    /// assert!(tokens > 0);
    /// # Ok(()) }
    /// ```
    pub fn visit_with_tokens(self, mut f: impl FnMut(Child<'t>) -> Walk) -> bool {
        matches!(self.visit_with_tokens_impl(&mut f), ControlOutcome::Ran)
    }

    fn visit_with_tokens_impl(self, f: &mut impl FnMut(Child<'t>) -> Walk) -> ControlOutcome {
        match f(Child::Node(self)) {
            Walk::Break => return ControlOutcome::Broke,
            Walk::Skip => return ControlOutcome::Ran,
            Walk::Continue => {}
        }
        for child in self.children_with_tokens() {
            match child {
                Child::Node(n) => {
                    if let ControlOutcome::Broke = n.visit_with_tokens_impl(f) {
                        return ControlOutcome::Broke;
                    }
                }
                Child::Token(_) => {
                    if let Walk::Break = f(child) {
                        return ControlOutcome::Broke;
                    }
                }
            }
        }
        ControlOutcome::Ran
    }

    /// Finds the first descendant node (including `self`) that can be viewed as
    /// the typed node `T`.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::nodes::ModuleDeclarationSyntax;
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// let module: ModuleDeclarationSyntax = tree.root().find_first().unwrap();
    /// # let _ = module;
    /// # Ok(()) }
    /// ```
    pub fn find_first<T: AstNode<'t>>(self) -> Option<T> {
        self.descendants().find_map(T::cast)
    }

    /// Iterates every descendant node (including `self`) that is a `T`.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::nodes::ModuleDeclarationSyntax;
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module a; endmodule\nmodule b; endmodule\n")?;
    /// assert_eq!(tree.root().find_all::<ModuleDeclarationSyntax>().count(), 2);
    /// # Ok(()) }
    /// ```
    pub fn find_all<T: AstNode<'t>>(self) -> impl Iterator<Item = T> {
        self.descendants().filter_map(T::cast)
    }

    /// The first token in the subtree (its leading token), if any.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// assert_eq!(tree.root().first_token().unwrap().text(), "module");
    /// # Ok(()) }
    /// ```
    pub fn first_token(&self) -> Option<Token<'t>> {
        // SAFETY: the node is valid.
        let t = unsafe { sys::slang_node_first_token(self.raw) };
        (!t.owner.is_null()).then_some(Token {
            raw: t,
            _tree: PhantomData,
        })
    }

    /// The node's source text (including trivia and expansions).
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// let module = tree.root().children().next().unwrap();
    /// assert_eq!(module.text(), "module m; endmodule");
    /// # Ok(()) }
    /// ```
    pub fn text(&self) -> String {
        let mut err = ffi::error();
        // SAFETY: the node is valid; the returned string is owned.
        unsafe { ffi::owned_str(sys::slang_node_to_string(self.raw, &mut err)) }
    }

    /// For a `ModuleDeclaration`/`InterfaceDeclaration`/`ProgramDeclaration`,
    /// the declared name.
    fn module_name(&self) -> Option<&'t str> {
        // The header is the first child node; its name is the first Identifier
        // token in the header.
        let header = self.children().next()?;
        for child in header.children_with_tokens() {
            if let Child::Token(t) = child
                && t.kind() == TokenKind::Identifier
            {
                return Some(t.text());
            }
        }
        None
    }
}

impl<'t> Token<'t> {
    /// The token's kind.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::{Child, Walk, kinds::TokenKind};
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module counter; endmodule\n")?;
    /// let mut ident = None;
    /// tree.root().visit_with_tokens(|c| match c {
    ///     Child::Token(t) if t.kind() == TokenKind::Identifier => { ident = Some(t); Walk::Break }
    ///     _ => Walk::Continue,
    /// });
    /// assert_eq!(ident.unwrap().kind(), TokenKind::Identifier);
    /// # Ok(()) }
    /// ```
    pub fn kind(&self) -> TokenKind {
        TokenKind::from_raw(self.raw.kind).unwrap_or(TokenKind::Unknown)
    }

    /// True if the token was synthesized by error recovery (its text is empty).
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// // a well-formed tree's leading `module` keyword is a real token.
    /// assert!(!tree.root().first_token().unwrap().is_missing());
    /// # Ok(()) }
    /// ```
    pub fn is_missing(&self) -> bool {
        self.raw.flags & sys::SLANG_TOKEN_MISSING as u16 != 0
    }

    /// The token's exact source text (no trivia); borrowed from the tree.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// assert_eq!(tree.root().first_token().unwrap().text(), "module");
    /// # Ok(()) }
    /// ```
    pub fn text(&self) -> &'t str {
        // SAFETY: the bytes are borrowed from the live tree for 't, and are
        // valid UTF-8 (SystemVerilog source).
        unsafe { ffi::str_ref(sys::slang_token_raw_text(self.raw)) }
    }

    /// The token's value text (unescaped string literal, identifier name);
    /// borrowed from the tree.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::{Child, Walk, kinds::TokenKind};
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module counter; endmodule\n")?;
    /// let mut ident = None;
    /// tree.root().visit_with_tokens(|c| match c {
    ///     Child::Token(t) if t.kind() == TokenKind::Identifier => { ident = Some(t); Walk::Break }
    ///     _ => Walk::Continue,
    /// });
    /// assert_eq!(ident.unwrap().value_text(), "counter");
    /// # Ok(()) }
    /// ```
    pub fn value_text(&self) -> &'t str {
        // SAFETY: as `text`.
        unsafe { ffi::str_ref(sys::slang_token_value_text(self.raw)) }
    }

    /// The number of leading trivia items.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// // the leading `module` keyword sits at offset 0, with no preceding trivia.
    /// assert_eq!(tree.root().first_token().unwrap().trivia_count(), 0);
    /// # Ok(()) }
    /// ```
    pub fn trivia_count(&self) -> usize {
        // SAFETY: the token is valid.
        unsafe { sys::slang_token_trivia_count(self.raw) as usize }
    }

    /// The kind of leading trivia item `index`, if in range.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// // the first token has no leading trivia, so index 0 is out of range.
    /// assert!(tree.root().first_token().unwrap().trivia_kind(0).is_none());
    /// # Ok(()) }
    /// ```
    pub fn trivia_kind(&self, index: usize) -> Option<TriviaKind> {
        let mut out = sys::slang_trivia {
            kind: 0,
            reserved_: 0,
            text: sys::slang_str {
                data: core::ptr::null(),
                len: 0,
                owner: core::ptr::null_mut(),
            },
        };
        // SAFETY: the token is valid; `out` is provided.
        let ok = unsafe { sys::slang_token_trivia(self.raw, index as u32, &mut out) };
        if !ok {
            return None;
        }
        // The trivia text may be owned; free it since we only want the kind.
        // SAFETY: freeing a string the library handed us (no-op if borrowed).
        unsafe { sys::slang_str_free(out.text) };
        TriviaKind::from_raw(out.kind as u8)
    }
}

/// Member accessors used by the generated typed nodes. A member is addressed
/// by its ordinal in the node's struct; the C API's `slang_node_member_span`
/// maps that to the contiguous child indices it occupies.
impl<'t> Node<'t> {
    fn member_span(&self, member: usize) -> Option<(usize, usize)> {
        let mut start = 0u32;
        let mut len = 0u32;
        // SAFETY: the node is valid; both out-params are provided.
        let ok =
            unsafe { sys::slang_node_member_span(self.raw, member as u32, &mut start, &mut len) };
        ok.then_some((start as usize, len as usize))
    }

    /// A scalar token member, or `None` if the (optional) slot is absent.
    ///
    /// Members are addressed by their ordinal in the node's generated struct;
    /// prefer the typed accessors in [`nodes`](crate::nodes) over raw ordinals.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// // an out-of-range member ordinal yields None.
    /// assert!(tree.root().member_token(999).is_none());
    /// # Ok(()) }
    /// ```
    pub fn member_token(&self, member: usize) -> Option<Token<'t>> {
        let (start, _) = self.member_span(member)?;
        match self.child(start) {
            Some(Child::Token(t)) => Some(t),
            _ => None,
        }
    }

    /// A scalar node member as a raw [`Node`], or `None` if absent.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// assert!(tree.root().member_node(999).is_none());
    /// # Ok(()) }
    /// ```
    pub fn member_node(&self, member: usize) -> Option<Node<'t>> {
        let (start, _) = self.member_span(member)?;
        match self.child(start) {
            Some(Child::Node(n)) => Some(n),
            _ => None,
        }
    }

    /// A `SyntaxList<T>` member (a list of nodes).
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::Node;
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// // an out-of-range ordinal yields an empty list.
    /// assert!(tree.root().member_list::<Node>(999).is_empty());
    /// # Ok(()) }
    /// ```
    pub fn member_list<T: AstNode<'t>>(&self, member: usize) -> SyntaxList<'t, T> {
        let (start, len) = self.member_span(member).unwrap_or((0, 0));
        SyntaxList {
            node: *self,
            start,
            len,
            _elem: PhantomData,
        }
    }

    /// A `SeparatedSyntaxList<T>` member (nodes interleaved with separators).
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::Node;
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// assert_eq!(tree.root().member_separated_list::<Node>(999).iter().count(), 0);
    /// # Ok(()) }
    /// ```
    pub fn member_separated_list<T: AstNode<'t>>(&self, member: usize) -> SeparatedList<'t, T> {
        let (start, len) = self.member_span(member).unwrap_or((0, 0));
        SeparatedList {
            node: *self,
            start,
            len,
            _elem: PhantomData,
        }
    }

    /// A `TokenList` member (a list of tokens).
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// assert!(tree.root().member_token_list(999).is_empty());
    /// # Ok(()) }
    /// ```
    pub fn member_token_list(&self, member: usize) -> TokenList<'t> {
        let (start, len) = self.member_span(member).unwrap_or((0, 0));
        TokenList {
            node: *self,
            start,
            len,
        }
    }
}

/// A typed view of a syntax [`Node`], as generated for each syntax kind and
/// abstract base. Implementors are `Copy` wrappers that borrow the tree.
///
/// ```
/// # fn main() -> Result<(), sv_lang::Error> {
/// use sv_lang::{AstNode, nodes::CompilationUnitSyntax, kinds::SyntaxKind};
/// let session = sv_lang::Session::new();
/// let tree = session.parse("module m; endmodule\n")?;
/// assert!(CompilationUnitSyntax::can_cast(SyntaxKind::CompilationUnit));
/// let unit = CompilationUnitSyntax::cast(tree.root()).unwrap();
/// assert_eq!(unit.syntax().kind(), SyntaxKind::CompilationUnit);
/// # Ok(()) }
/// ```
pub trait AstNode<'t>: Sized + Copy {
    /// Whether a node of this syntax kind can be viewed as `Self`.
    fn can_cast(kind: SyntaxKind) -> bool;
    /// Views `node` as `Self`, or `None` if its kind does not match.
    fn cast(node: Node<'t>) -> Option<Self>;
    /// The underlying node.
    fn syntax(&self) -> Node<'t>;
}

/// A list of node members of type `T` (`SyntaxList<T>` in slang).
#[derive(Clone, Copy)]
pub struct SyntaxList<'t, T> {
    node: Node<'t>,
    start: usize,
    len: usize,
    _elem: PhantomData<fn() -> T>,
}

impl<'t, T: AstNode<'t>> SyntaxList<'t, T> {
    /// The number of elements.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::nodes::CompilationUnitSyntax;
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module a; endmodule\nmodule b; endmodule\n")?;
    /// let unit = tree.root().find_first::<CompilationUnitSyntax>().unwrap();
    /// assert_eq!(unit.members().len(), 2);
    /// # Ok(()) }
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }
    /// Whether the list is empty.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::nodes::CompilationUnitSyntax;
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module a; endmodule\n")?;
    /// let unit = tree.root().find_first::<CompilationUnitSyntax>().unwrap();
    /// assert!(!unit.members().is_empty());
    /// # Ok(()) }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// Iterates the elements, viewing each as `T`.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::nodes::CompilationUnitSyntax;
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module a; endmodule\nmodule b; endmodule\n")?;
    /// let unit = tree.root().find_first::<CompilationUnitSyntax>().unwrap();
    /// assert_eq!(unit.members().iter().count(), 2);
    /// # Ok(()) }
    /// ```
    pub fn iter(&self) -> ListIter<'t, T> {
        ListIter::new(self.node, self.start, self.len)
    }
}

impl<'t, T: AstNode<'t>> IntoIterator for SyntaxList<'t, T> {
    type Item = T;
    type IntoIter = ListIter<'t, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Lazy iterator over the `T` node members of a [`SyntaxList`] or
/// [`SeparatedList`], skipping any interleaved separator tokens. Holds only the
/// parent node and the child-index range still to visit — it allocates nothing.
#[derive(Clone)]
pub struct ListIter<'t, T> {
    node: Node<'t>,
    range: core::ops::Range<usize>,
    _elem: PhantomData<fn() -> T>,
}

impl<'t, T> ListIter<'t, T> {
    fn new(node: Node<'t>, start: usize, len: usize) -> Self {
        ListIter {
            node,
            range: start..start + len,
            _elem: PhantomData,
        }
    }
}

impl<'t, T: AstNode<'t>> Iterator for ListIter<'t, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        for i in self.range.by_ref() {
            if let Some(Child::Node(n)) = self.node.child(i)
                && let Some(t) = T::cast(n)
            {
                return Some(t);
            }
        }
        None
    }
}

/// A list of node members of type `T` interleaved with separator tokens
/// (`SeparatedSyntaxList<T>` in slang).
#[derive(Clone, Copy)]
pub struct SeparatedList<'t, T> {
    node: Node<'t>,
    start: usize,
    len: usize,
    _elem: PhantomData<fn() -> T>,
}

impl<'t, T: AstNode<'t>> SeparatedList<'t, T> {
    /// Iterates the elements, skipping separator tokens.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::nodes::DataDeclarationSyntax;
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; logic [7:0] x, y; endmodule\n")?;
    /// let decl = tree.root().find_first::<DataDeclarationSyntax>().unwrap();
    /// assert_eq!(decl.declarators().iter().count(), 2); // x and y
    /// # Ok(()) }
    /// ```
    pub fn iter(&self) -> ListIter<'t, T> {
        ListIter::new(self.node, self.start, self.len)
    }

    /// Iterates the separator tokens.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::nodes::DataDeclarationSyntax;
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; logic [7:0] x, y; endmodule\n")?;
    /// let decl = tree.root().find_first::<DataDeclarationSyntax>().unwrap();
    /// // two declarators are joined by one comma separator.
    /// assert_eq!(decl.declarators().separators().count(), 1);
    /// # Ok(()) }
    /// ```
    pub fn separators(&self) -> impl Iterator<Item = Token<'t>> + 't {
        let node = self.node;
        let range = self.start..self.start + self.len;
        range.filter_map(move |i| match node.child(i) {
            Some(Child::Token(t)) => Some(t),
            _ => None,
        })
    }
}

impl<'t, T: AstNode<'t>> IntoIterator for SeparatedList<'t, T> {
    type Item = T;
    type IntoIter = ListIter<'t, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A list of token members (`TokenList` in slang).
#[derive(Clone, Copy)]
pub struct TokenList<'t> {
    node: Node<'t>,
    start: usize,
    len: usize,
}

impl<'t> TokenList<'t> {
    /// The number of tokens.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::nodes::DataDeclarationSyntax;
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; logic [7:0] x; endmodule\n")?;
    /// let decl = tree.root().find_first::<DataDeclarationSyntax>().unwrap();
    /// // a plain declaration carries no leading modifiers (const/var/...).
    /// assert_eq!(decl.modifiers().len(), 0);
    /// # Ok(()) }
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }
    /// Whether the list is empty.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::nodes::DataDeclarationSyntax;
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; logic [7:0] x; endmodule\n")?;
    /// let decl = tree.root().find_first::<DataDeclarationSyntax>().unwrap();
    /// assert!(decl.modifiers().is_empty());
    /// # Ok(()) }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// Iterates the tokens.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::nodes::DataDeclarationSyntax;
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; logic [7:0] x; endmodule\n")?;
    /// let decl = tree.root().find_first::<DataDeclarationSyntax>().unwrap();
    /// assert_eq!(decl.modifiers().iter().count(), 0);
    /// # Ok(()) }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = Token<'t>> + 't {
        let node = self.node;
        let range = self.start..self.start + self.len;
        range.filter_map(move |i| match node.child(i) {
            Some(Child::Token(t)) => Some(t),
            _ => None,
        })
    }
}

/// Pre-order iterator over a node and its descendant nodes.
///
/// ```
/// # fn main() -> Result<(), sv_lang::Error> {
/// use sv_lang::kinds::SyntaxKind;
/// let session = sv_lang::Session::new();
/// let tree = session.parse("module a; endmodule\nmodule b; endmodule\n")?;
/// let modules = tree.root().descendants()
///     .filter(|n| n.kind() == SyntaxKind::ModuleDeclaration).count();
/// assert_eq!(modules, 2);
/// # Ok(()) }
/// ```
pub struct Descendants<'t> {
    stack: Vec<Node<'t>>,
}

impl<'t> Iterator for Descendants<'t> {
    type Item = Node<'t>;
    fn next(&mut self) -> Option<Node<'t>> {
        let node = self.stack.pop()?;
        // Push children in reverse so they pop in source order.
        let children: Vec<_> = node.children().collect();
        self.stack.extend(children.into_iter().rev());
        Some(node)
    }
}

/// The `slang_syntax_sink` callbacks that drive an `sv_lang_syntax::Builder`.
/// Each receives a [`MirrorSink`] as `user`. A well-formed slang walk never
/// makes the builder panic, but should one ever panic (a builder-invariant
/// violation), the panic must NOT unwind across the C walk — that is UB. Each
/// callback runs under a `catch_unwind` barrier, records the first panic, and
/// turns later callbacks into no-ops; [`SyntaxTree::mirror`] resumes the panic
/// on the Rust side once the C walk has returned.
mod mirror {
    use core::any::Any;
    use core::ffi::{c_char, c_void};
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use sv_lang_kinds::{SyntaxKind, TokenKind, TriviaKind};
    use sv_lang_syntax::{Builder, ListKind};
    use sv_lang_sys as sys;

    /// The `user` payload for the walk: the builder plus any captured panic.
    pub(super) struct MirrorSink {
        pub builder: Builder,
        pub panic: Option<Box<dyn Any + Send + 'static>>,
    }

    impl MirrorSink {
        pub(super) fn new() -> MirrorSink {
            MirrorSink {
                builder: Builder::new(),
                panic: None,
            }
        }
    }

    unsafe fn sink<'a>(user: *mut c_void) -> &'a mut MirrorSink {
        // SAFETY: `user` is the &mut MirrorSink passed to slang_syntax_tree_walk.
        unsafe { &mut *user.cast::<MirrorSink>() }
    }

    /// Runs one builder operation under a `catch_unwind` barrier so a panic can
    /// never cross the C boundary; the first panic is recorded and every later
    /// operation becomes a no-op.
    fn guard(s: &mut MirrorSink, f: impl FnOnce(&mut Builder)) {
        if s.panic.is_some() {
            return;
        }
        let b = &mut s.builder;
        if let Err(p) = catch_unwind(AssertUnwindSafe(|| f(b))) {
            s.panic = Some(p);
        }
    }

    unsafe fn text<'a>(ptr: *const c_char, len: usize) -> &'a str {
        if ptr.is_null() || len == 0 {
            return "";
        }
        // SAFETY: valid for the duration of the callback.
        let slice = unsafe { core::slice::from_raw_parts(ptr.cast::<u8>(), len) };
        core::str::from_utf8(slice).unwrap_or("")
    }

    pub(super) unsafe extern "C" fn start_node(user: *mut c_void, kind: u32, _s: u32) {
        // SAFETY: see module docs.
        let s = unsafe { sink(user) };
        guard(s, |b| {
            b.start_node(SyntaxKind::from_raw(kind as u16).unwrap_or(SyntaxKind::Unknown));
        });
    }

    pub(super) unsafe extern "C" fn trivia(
        user: *mut c_void,
        kind: u32,
        t: *const c_char,
        len: usize,
    ) {
        // SAFETY: see module docs.
        let s = unsafe { sink(user) };
        // SAFETY: `t`/`len` are valid for the duration of the callback.
        let txt = unsafe { text(t, len) };
        guard(s, |b| {
            b.trivia(
                TriviaKind::from_raw(kind as u8).unwrap_or(TriviaKind::Unknown),
                txt,
            );
        });
    }

    pub(super) unsafe extern "C" fn token(
        user: *mut c_void,
        kind: u32,
        t: *const c_char,
        len: usize,
        _loc: sys::slang_loc,
        _flags: u16,
    ) {
        // SAFETY: see module docs.
        let s = unsafe { sink(user) };
        // SAFETY: `t`/`len` are valid for the duration of the callback.
        let txt = unsafe { text(t, len) };
        guard(s, |b| {
            b.token(
                TokenKind::from_raw(kind as u16).unwrap_or(TokenKind::Unknown),
                txt,
            );
        });
    }

    pub(super) unsafe extern "C" fn absent(user: *mut c_void) {
        // SAFETY: see module docs.
        let s = unsafe { sink(user) };
        guard(s, |b| b.absent());
    }

    pub(super) unsafe extern "C" fn start_list(user: *mut c_void, form: sys::slang_member_form) {
        let kind = match form {
            sys::SLANG_MEMBER_SEPARATED_LIST => ListKind::Separated,
            sys::SLANG_MEMBER_TOKEN_LIST => ListKind::Tokens,
            _ => ListKind::Nodes,
        };
        // SAFETY: see module docs.
        let s = unsafe { sink(user) };
        guard(s, |b| b.start_list(kind));
    }

    pub(super) unsafe extern "C" fn finish_list(user: *mut c_void) {
        // SAFETY: see module docs.
        let s = unsafe { sink(user) };
        guard(s, |b| b.finish_list());
    }

    pub(super) unsafe extern "C" fn finish_node(user: *mut c_void) {
        // SAFETY: see module docs.
        let s = unsafe { sink(user) };
        guard(s, |b| b.finish_node());
    }
}
