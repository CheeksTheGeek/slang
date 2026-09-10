//! Lossless SystemVerilog syntax trees in pure Rust, structurally identical to
//! slang's concrete syntax tree.
//!
//! The tree is a [`cstree`] green tree whose kinds are slang's own kind
//! enumerations, following the same interleaved-trivia convention as
//! rust-analyzer's trees. Its shape follows slang's schema exactly:
//!
//! * The non-trivia children of a node are the members of its syntax struct,
//!   in declaration order — so member `k` is [`NodeExt::member`]`(k)`.
//! * Trivia (whitespace, comments, directives) are [`Kind::Trivia`] tokens
//!   interleaved before the token they lead, exactly where they appear in the
//!   source. [`NodeExt::member`] skips them.
//! * A list member is a nested [`Kind::List`] node holding the list's elements
//!   (and, for separated lists, the separator tokens).
//! * An absent optional member is a zero-width [`Kind::Absent`] token, so
//!   member indices stay aligned with the struct's declared members.
//!
//! The text of any node is therefore the exact source bytes it covers.
//!
//! Trees are produced by the `sv-lang` crate from slang's parser; this crate
//! only defines the representation, so that tooling which keeps trees around
//! (formatters, refactoring tools, editors) does not need slang linked in.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use core::fmt;

mod edit;

pub use cstree;
pub use cstree::text::{TextRange, TextSize};
pub use edit::{Spanned, SyntaxEditor};
pub use sv_lang_kinds::{SyntaxKind, SyntaxStruct, TokenKind, TriviaKind};

/// The kinds of list members a node can contain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ListKind {
    /// A plain list of nodes (`SyntaxList<T>` in slang).
    Nodes = 0,
    /// A list of nodes interleaved with separator tokens (`SeparatedSyntaxList<T>`).
    Separated = 1,
    /// A list of tokens (`TokenList`).
    Tokens = 2,
}

/// The kind of every element in an `sv-lang-syntax` tree.
///
/// Node kinds, token kinds and trivia kinds are slang's; [`Kind::List`] and
/// [`Kind::Absent`] are the two structural additions described in the crate
/// documentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    /// An interior node of the given syntax kind.
    Node(SyntaxKind),
    /// A token.
    Token(TokenKind),
    /// A piece of trivia (whitespace, comment, directive, ...).
    Trivia(TriviaKind),
    /// A list member; its children are the list's elements (and separators).
    List(ListKind),
    /// A zero-width placeholder for an absent optional member.
    Absent,
}

impl Kind {
    const TOKEN_BASE: u32 = SyntaxKind::COUNT as u32;
    const TRIVIA_BASE: u32 = Self::TOKEN_BASE + TokenKind::COUNT as u32;
    const LIST_BASE: u32 = Self::TRIVIA_BASE + TriviaKind::COUNT as u32;
    const ABSENT: u32 = Self::LIST_BASE + 3;

    /// True for element kinds that are interior nodes ([`Kind::Node`] and [`Kind::List`]).
    pub const fn is_node(self) -> bool {
        matches!(self, Kind::Node(_) | Kind::List(_))
    }

    /// True for element kinds that carry text ([`Kind::Token`], [`Kind::Trivia`], [`Kind::Absent`]).
    pub const fn is_token(self) -> bool {
        !self.is_node()
    }
}

impl cstree::Syntax for Kind {
    fn from_raw(raw: cstree::RawSyntaxKind) -> Self {
        let r = raw.0;
        if r < Self::TOKEN_BASE {
            Kind::Node(SyntaxKind::from_raw(r as u16).expect("valid syntax kind"))
        } else if r < Self::TRIVIA_BASE {
            Kind::Token(
                TokenKind::from_raw((r - Self::TOKEN_BASE) as u16).expect("valid token kind"),
            )
        } else if r < Self::LIST_BASE {
            Kind::Trivia(
                TriviaKind::from_raw((r - Self::TRIVIA_BASE) as u8).expect("valid trivia kind"),
            )
        } else if r < Self::ABSENT {
            Kind::List(match r - Self::LIST_BASE {
                0 => ListKind::Nodes,
                1 => ListKind::Separated,
                _ => ListKind::Tokens,
            })
        } else {
            debug_assert_eq!(r, Self::ABSENT, "raw kind out of range");
            Kind::Absent
        }
    }

    fn into_raw(self) -> cstree::RawSyntaxKind {
        cstree::RawSyntaxKind(match self {
            Kind::Node(k) => k.as_raw() as u32,
            Kind::Token(k) => Self::TOKEN_BASE + k.as_raw() as u32,
            Kind::Trivia(k) => Self::TRIVIA_BASE + k.as_raw() as u32,
            Kind::List(k) => Self::LIST_BASE + k as u32,
            Kind::Absent => Self::ABSENT,
        })
    }

    fn static_text(self) -> Option<&'static str> {
        match self {
            Kind::Absent => Some(""),
            _ => None,
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Node(k) => write!(f, "{k}"),
            Kind::Token(k) => write!(f, "{k}"),
            Kind::Trivia(k) => write!(f, "{k}"),
            Kind::List(ListKind::Nodes) => f.write_str("List"),
            Kind::List(ListKind::Separated) => f.write_str("SeparatedList"),
            Kind::List(ListKind::Tokens) => f.write_str("TokenList"),
            Kind::Absent => f.write_str("Absent"),
        }
    }
}

/// A shared, immutable tree ("green" tree) with its string interner.
pub type GreenNode = cstree::green::GreenNode;
/// A node of a materialized ("red") tree with parent pointers and text offsets.
pub type SyntaxNode<D = ()> = cstree::syntax::SyntaxNode<Kind, D>;
/// A token of a materialized tree.
pub type SyntaxToken<D = ()> = cstree::syntax::SyntaxToken<Kind, D>;
/// A node or token of a materialized tree.
pub type SyntaxElement<D = ()> = cstree::syntax::SyntaxElement<Kind, D>;
/// A materialized tree whose root owns the interner needed to resolve token text.
pub type ResolvedNode<D = ()> = cstree::syntax::ResolvedNode<Kind, D>;
/// A token of a materialized tree that can resolve its own text.
pub type ResolvedToken<D = ()> = cstree::syntax::ResolvedToken<Kind, D>;
/// A node or token of a materialized tree that can resolve its own text.
pub type ResolvedElement<D = ()> = cstree::syntax::ResolvedElement<Kind, D>;

/// Builds a tree from a stream of events, in the same order slang's C API
/// `slang_syntax_tree_walk` emits them.
///
/// The builder validates nothing about the schema: `sv-lang` feeds it slang's
/// own walk, which is correct by construction. Tools synthesizing trees by
/// hand should keep to the shape described in the crate documentation.
pub struct Builder {
    inner: cstree::build::GreenNodeBuilder<'static, 'static, Kind>,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Creates an empty builder with a fresh interner.
    pub fn new() -> Self {
        Builder {
            inner: cstree::build::GreenNodeBuilder::new(),
        }
    }

    /// Opens a node of the given syntax kind.
    pub fn start_node(&mut self, kind: SyntaxKind) {
        self.inner.start_node(Kind::Node(kind));
    }

    /// Opens a list member.
    pub fn start_list(&mut self, kind: ListKind) {
        self.inner.start_node(Kind::List(kind));
    }

    /// Adds a trivia token.
    pub fn trivia(&mut self, kind: TriviaKind, text: &str) {
        self.inner.token(Kind::Trivia(kind), text);
    }

    /// Adds a token. Missing tokens (synthesized by error recovery) have empty text.
    pub fn token(&mut self, kind: TokenKind, text: &str) {
        self.inner.token(Kind::Token(kind), text);
    }

    /// Adds the placeholder for an absent optional member.
    pub fn absent(&mut self) {
        self.inner.static_token(Kind::Absent);
    }

    /// Closes the innermost open list.
    pub fn finish_list(&mut self) {
        self.inner.finish_node();
    }

    /// Closes the innermost open node.
    pub fn finish_node(&mut self) {
        self.inner.finish_node();
    }

    /// Finishes building, returning the root as a tree that can resolve its
    /// own token text.
    pub fn finish(self) -> ResolvedNode {
        let (green, cache) = self.inner.finish();
        let interner = cache
            .expect("builder owns its cache")
            .into_interner()
            .expect("builder owns its interner");
        SyntaxNode::new_root_with_resolver(green, interner)
    }
}

/// Convenience accessors on materialized nodes. `D` is the node data type.
pub trait NodeExt<D: 'static> {
    /// The syntax kind, if this is a syntax node (not a list placeholder).
    fn node_kind(&self) -> Option<SyntaxKind>;
    /// The generated struct this node is an instance of.
    fn node_struct(&self) -> Option<SyntaxStruct>;
    /// Member `index` of the node's struct: the `index`-th non-trivia child.
    fn member(&self, index: usize) -> Option<SyntaxElement<D>>;
    /// Iterates the node's members (its non-trivia children) in order.
    fn members(&self) -> impl Iterator<Item = SyntaxElement<D>>;
}

impl<D: 'static> NodeExt<D> for cstree::syntax::SyntaxNode<Kind, D> {
    fn node_kind(&self) -> Option<SyntaxKind> {
        match self.kind() {
            Kind::Node(k) => Some(k),
            _ => None,
        }
    }

    fn node_struct(&self) -> Option<SyntaxStruct> {
        self.node_kind().and_then(SyntaxKind::syntax_struct)
    }

    fn member(&self, index: usize) -> Option<SyntaxElement<D>> {
        self.members().nth(index)
    }

    fn members(&self) -> impl Iterator<Item = SyntaxElement<D>> {
        use cstree::util::NodeOrToken;
        self.children_with_tokens().filter_map(|e| match e {
            NodeOrToken::Node(n) => Some(NodeOrToken::Node(n.clone())),
            // Trivia are interleaved siblings, not members.
            NodeOrToken::Token(t) if matches!(t.kind(), Kind::Trivia(_)) => None,
            NodeOrToken::Token(t) => Some(NodeOrToken::Token(t.clone())),
        })
    }
}

/// Structured constructors for synthesizing well-formed syntax trees by hand —
/// the composable alternative to text-splicing with [`SyntaxEditor`].
///
/// Each function drives a [`Builder`] to emit a node with slang's exact member
/// layout and canonical formatting, so the result is byte-identical to what the
/// parser produces for the equivalent source (verified in `sv-lang` against the
/// real mirror). Container builders take a closure that fills the contained
/// member list, so trees compose:
///
/// ```
/// use sv_lang_syntax::make;
/// let unit = make::empty_module("counter");
/// assert_eq!(unit.text().to_string(), "module counter;\nendmodule\n");
/// ```
pub mod make {
    use crate::{Builder, ListKind, ResolvedNode, SyntaxKind, TokenKind, TriviaKind};

    /// Builds a `CompilationUnit` whose top-level member list is filled by
    /// `items`, terminated by an end-of-file token carrying a trailing newline.
    /// This is the root every synthesized tree needs.
    ///
    /// ```
    /// use sv_lang_syntax::make;
    /// let unit = make::compilation_unit(|b| {
    ///     make::module(b, "a", |_| {});
    ///     make::module(b, "b", |_| {});
    /// });
    /// assert_eq!(
    ///     unit.text().to_string(),
    ///     "module a;\nendmodule\nmodule b;\nendmodule\n"
    /// );
    /// ```
    pub fn compilation_unit(items: impl FnOnce(&mut Builder)) -> ResolvedNode {
        let mut b = Builder::new();
        b.start_node(SyntaxKind::CompilationUnit);
        b.start_list(ListKind::Nodes);
        items(&mut b);
        b.finish_list();
        // Each member emits its own trailing newline, so end-of-file is bare.
        b.token(TokenKind::EndOfFile, "");
        b.finish_node();
        b.finish()
    }

    /// Emits a `ModuleDeclaration` (`module <name>; <body> endmodule`) into an
    /// open member list. `body` fills the module's own member list; pass
    /// `|_| {}` for an empty module.
    pub fn module(b: &mut Builder, name: &str, body: impl FnOnce(&mut Builder)) {
        b.start_node(SyntaxKind::ModuleDeclaration);
        // attributes: an always-present (here empty) list.
        b.start_list(ListKind::Nodes);
        b.finish_list();
        module_header(b, name);
        // members: filled by the caller.
        b.start_list(ListKind::Nodes);
        body(b);
        b.finish_list();
        b.trivia(TriviaKind::EndOfLine, "\n");
        b.token(TokenKind::EndModuleKeyword, "endmodule");
        b.absent(); // blockName
        // Trailing newline so a sequence of modules reads one per line.
        b.trivia(TriviaKind::EndOfLine, "\n");
        b.finish_node();
    }

    /// Emits the `ModuleHeader` (`module <name>;`) — the seven members slang's
    /// schema requires, with the identifier's leading space and the absent
    /// lifetime / parameter-port / port members in their exact positions.
    fn module_header(b: &mut Builder, name: &str) {
        b.start_node(SyntaxKind::ModuleHeader);
        b.token(TokenKind::ModuleKeyword, "module");
        b.absent(); // lifetime
        b.trivia(TriviaKind::Whitespace, " ");
        b.token(TokenKind::Identifier, name);
        b.start_list(ListKind::Nodes); // imports
        b.finish_list();
        b.absent(); // parameters
        b.absent(); // ports
        b.token(TokenKind::Semicolon, ";");
        b.finish_node();
    }

    /// A whole compilation unit containing a single empty module — the common
    /// case, and the shortest bridge from a name to a parseable tree.
    pub fn empty_module(name: &str) -> ResolvedNode {
        compilation_unit(|b| module(b, name, |_| {}))
    }

    /// Emits a variable declaration into an open member list: a vector
    /// `logic [<msb>:<lsb>] <name>;` when `range` is `Some`, or a scalar
    /// `logic <name>;` when `None`. Indented one level — the building block for a
    /// non-empty [`module`] body.
    ///
    /// ```
    /// use sv_lang_syntax::make;
    /// let unit = make::compilation_unit(|b| {
    ///     make::module(b, "m", |b| {
    ///         make::logic(b, "bus", Some((7, 0)));
    ///         make::logic(b, "flag", None);
    ///     });
    /// });
    /// assert_eq!(
    ///     unit.text().to_string(),
    ///     "module m;\n    logic [7:0] bus;\n    logic flag;\nendmodule\n"
    /// );
    /// ```
    pub fn logic(b: &mut Builder, name: &str, range: Option<(u32, u32)>) {
        b.start_node(SyntaxKind::DataDeclaration);
        b.start_list(ListKind::Nodes); // attributes
        b.finish_list();
        b.start_list(ListKind::Tokens); // modifiers (const/var/...)
        b.finish_list();
        // Type: `logic`, optionally with one packed `[msb:lsb]` dimension.
        b.start_node(SyntaxKind::LogicType);
        b.trivia(TriviaKind::EndOfLine, "\n");
        b.trivia(TriviaKind::Whitespace, "    ");
        b.token(TokenKind::LogicKeyword, "logic");
        b.absent(); // signing
        b.start_list(ListKind::Nodes); // packed dimensions
        if let Some((msb, lsb)) = range {
            packed_range(b, msb, lsb);
        }
        b.finish_list();
        b.finish_node(); // LogicType
        // A single declarator.
        b.start_list(ListKind::Separated);
        b.start_node(SyntaxKind::Declarator);
        b.trivia(TriviaKind::Whitespace, " ");
        b.token(TokenKind::Identifier, name);
        b.start_list(ListKind::Nodes); // unpacked dimensions
        b.finish_list();
        b.absent(); // initializer
        b.finish_node(); // Declarator
        b.finish_list();
        b.token(TokenKind::Semicolon, ";");
        b.finish_node(); // DataDeclaration
    }

    /// Emits one packed dimension `[<msb>:<lsb>]` into an open dimensions list.
    fn packed_range(b: &mut Builder, msb: u32, lsb: u32) {
        b.start_node(SyntaxKind::VariableDimension);
        b.trivia(TriviaKind::Whitespace, " ");
        b.token(TokenKind::OpenBracket, "[");
        b.start_node(SyntaxKind::RangeDimensionSpecifier);
        b.start_node(SyntaxKind::SimpleRangeSelect);
        integer_literal(b, msb);
        b.token(TokenKind::Colon, ":");
        integer_literal(b, lsb);
        b.finish_node(); // SimpleRangeSelect
        b.finish_node(); // RangeDimensionSpecifier
        b.token(TokenKind::CloseBracket, "]");
        b.finish_node(); // VariableDimension
    }

    /// Emits an integer-literal expression `<value>`.
    fn integer_literal(b: &mut Builder, value: u32) {
        b.start_node(SyntaxKind::IntegerLiteralExpression);
        let text = value.to_string();
        b.token(TokenKind::IntegerLiteral, &text);
        b.finish_node();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cstree::Syntax;

    #[test]
    fn kinds_round_trip_through_raw() {
        let all = SyntaxKind::ALL
            .iter()
            .map(|k| Kind::Node(*k))
            .chain(TokenKind::ALL.iter().map(|k| Kind::Token(*k)))
            .chain(TriviaKind::ALL.iter().map(|k| Kind::Trivia(*k)))
            .chain([
                Kind::List(ListKind::Nodes),
                Kind::List(ListKind::Separated),
                Kind::List(ListKind::Tokens),
                Kind::Absent,
            ]);
        let mut seen = std::collections::HashSet::new();
        for k in all {
            let raw = k.into_raw();
            assert_eq!(Kind::from_raw(raw), k);
            assert!(seen.insert(raw.0), "raw kind {} used twice", raw.0);
        }
    }

    #[test]
    fn builder_reproduces_text() {
        let mut b = Builder::new();
        b.start_node(SyntaxKind::CompilationUnit);
        b.start_list(ListKind::Nodes);
        b.start_node(SyntaxKind::ModuleDeclaration);
        b.token(TokenKind::ModuleKeyword, "module");
        b.trivia(TriviaKind::Whitespace, " ");
        b.token(TokenKind::Identifier, "m");
        b.absent();
        b.token(TokenKind::Semicolon, ";");
        b.finish_node();
        b.finish_list();
        b.trivia(TriviaKind::EndOfLine, "\n");
        b.token(TokenKind::EndOfFile, "");
        b.finish_node();
        let root = b.finish();
        assert_eq!(root.text().to_string(), "module m;\n");
        assert_eq!(root.kind(), Kind::Node(SyntaxKind::CompilationUnit));
        let list = root.first_child().unwrap();
        assert_eq!(list.kind(), Kind::List(ListKind::Nodes));
        let module = list.first_child().unwrap();
        assert_eq!(
            module.node_struct(),
            Some(SyntaxStruct::ModuleDeclarationSyntax)
        );
        // module, <ws>, m, <absent>, ; — five children, four members.
        assert_eq!(module.arity_with_tokens(), 5);
        assert_eq!(module.members().count(), 4);
        assert_eq!(
            module.member(0).unwrap().kind(),
            Kind::Token(TokenKind::ModuleKeyword)
        );
        assert_eq!(
            module.member(1).unwrap().kind(),
            Kind::Token(TokenKind::Identifier)
        );
        assert_eq!(module.member(2).unwrap().kind(), Kind::Absent);
        assert_eq!(
            module.member(3).unwrap().kind(),
            Kind::Token(TokenKind::Semicolon)
        );
    }
}
