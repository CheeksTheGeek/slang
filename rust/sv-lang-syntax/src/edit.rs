//! Text-preserving edits over a syntax tree.
//!
//! A [`SyntaxEditor`] records replacements, insertions and deletions against
//! the ranges of nodes and tokens in a tree, then applies them to produce new
//! source text. Because the tree is lossless, an editor built from an
//! unmodified tree edits the exact original bytes — ideal for formatters,
//! codemods and refactoring tools, which then re-parse the result.

use core::ops::Range;

use cstree::text::TextRange;

use crate::{Kind, ResolvedNode};

/// Anything with a source range: a node or a token, resolved or not.
pub trait Spanned {
    /// The byte range this element covers in the tree's text.
    fn text_range(&self) -> TextRange;
}

impl<D: 'static> Spanned for cstree::syntax::SyntaxNode<Kind, D> {
    fn text_range(&self) -> TextRange {
        cstree::syntax::SyntaxNode::text_range(self)
    }
}
impl<D: 'static> Spanned for cstree::syntax::SyntaxToken<Kind, D> {
    fn text_range(&self) -> TextRange {
        cstree::syntax::SyntaxToken::text_range(self)
    }
}
impl<D: 'static> Spanned for cstree::syntax::ResolvedNode<Kind, D> {
    fn text_range(&self) -> TextRange {
        cstree::syntax::SyntaxNode::text_range(self)
    }
}
impl<D: 'static> Spanned for cstree::syntax::ResolvedToken<Kind, D> {
    fn text_range(&self) -> TextRange {
        cstree::syntax::SyntaxToken::text_range(self)
    }
}

/// Accumulates edits against a tree's text and applies them in one pass.
///
/// ```
/// # use sv_lang_syntax::{Builder, SyntaxKind, TokenKind, SyntaxEditor};
/// # let mut b = Builder::new();
/// # b.start_node(SyntaxKind::CompilationUnit);
/// # b.token(TokenKind::Identifier, "old_name");
/// # b.token(TokenKind::EndOfFile, "");
/// # b.finish_node();
/// # let root = b.finish();
/// let id = root.first_token().unwrap();
/// let mut editor = SyntaxEditor::new(&root);
/// editor.replace(id, "new_name");
/// assert_eq!(editor.finish(), "new_name");
/// ```
pub struct SyntaxEditor {
    source: String,
    edits: Vec<Edit>,
}

struct Edit {
    range: Range<usize>,
    replacement: String,
}

fn to_range(range: TextRange) -> Range<usize> {
    usize::from(range.start())..usize::from(range.end())
}

impl SyntaxEditor {
    /// Creates an editor over `root`'s text.
    pub fn new(root: &ResolvedNode) -> Self {
        SyntaxEditor {
            source: root.text().to_string(),
            edits: Vec::new(),
        }
    }

    /// The unedited source text.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Replaces a node's or token's text with `text`.
    pub fn replace(&mut self, element: &impl Spanned, text: impl Into<String>) {
        self.replace_range(element.text_range(), text);
    }

    /// Replaces the text spanned by `range` with `text`.
    pub fn replace_range(&mut self, range: TextRange, text: impl Into<String>) {
        self.edits.push(Edit {
            range: to_range(range),
            replacement: text.into(),
        });
    }

    /// Deletes a node's or token's text.
    pub fn delete(&mut self, element: &impl Spanned) {
        self.replace(element, "");
    }

    /// Inserts `text` immediately before `element`.
    pub fn insert_before(&mut self, element: &impl Spanned, text: impl Into<String>) {
        let at = usize::from(element.text_range().start());
        self.edits.push(Edit {
            range: at..at,
            replacement: text.into(),
        });
    }

    /// Inserts `text` immediately after `element`.
    pub fn insert_after(&mut self, element: &impl Spanned, text: impl Into<String>) {
        let at = usize::from(element.text_range().end());
        self.edits.push(Edit {
            range: at..at,
            replacement: text.into(),
        });
    }

    /// True if no edits have been recorded.
    pub fn is_empty(&self) -> bool {
        self.edits.is_empty()
    }

    /// Applies all edits and returns the new text.
    ///
    /// # Panics
    /// Panics if two edits overlap. Insertions at the same offset are applied
    /// in the order they were recorded.
    pub fn finish(mut self) -> String {
        // Stable sort by start keeps same-offset insertions in insertion order.
        self.edits.sort_by_key(|e| e.range.start);

        let mut out = String::with_capacity(self.source.len());
        let mut cursor = 0usize;
        for edit in &self.edits {
            assert!(
                edit.range.start >= cursor,
                "overlapping edits at byte {} (previous edit ended at {cursor})",
                edit.range.start
            );
            out.push_str(&self.source[cursor..edit.range.start]);
            out.push_str(&edit.replacement);
            cursor = edit.range.end;
        }
        out.push_str(&self.source[cursor..]);
        out
    }
}
