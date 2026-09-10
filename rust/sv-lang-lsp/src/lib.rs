//! SystemVerilog language-server *intelligence* over [`sv_lang_db`]'s
//! incremental workspace: turn a file's text into editor-ready diagnostics and
//! document symbols. These are pure functions the (optional) `tower-lsp` server
//! binary calls from its request handlers — kept transport-free so they can be
//! unit-tested directly.
//!
//! Positions are LSP-style: 0-based `line`, and `character` counted in UTF-16
//! code units (the LSP default).
//!
//! ```
//! use sv_lang_db::Workspace;
//! let mut ws = Workspace::new();
//! ws.set_file("m.sv", "module m; assign x = ; endmodule\n");
//! let diags = sv_lang_lsp::diagnostics(&mut ws, "m.sv");
//! assert!(diags.iter().any(|d| d.severity == sv_lang_lsp::Severity::Error));
//! let syms = sv_lang_lsp::document_symbols(&mut ws, "m.sv");
//! assert!(syms.iter().any(|s| s.name == "m"));
//! ```

use sv_lang::kinds::SymbolKind;
use sv_lang_db::Workspace;

/// A 0-based LSP position (`character` in UTF-16 code units).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    /// 0-based line.
    pub line: u32,
    /// 0-based character offset, in UTF-16 code units.
    pub character: u32,
}

/// A half-open `[start, end)` range of [`Position`]s.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    /// Inclusive start.
    pub start: Position,
    /// Exclusive end.
    pub end: Position,
}

/// Diagnostic severity, mirroring the LSP levels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    /// An error.
    Error,
    /// A warning.
    Warning,
    /// An informational note.
    Information,
    /// A hint.
    Hint,
}

/// An editor-ready diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    /// Where it applies.
    pub range: Range,
    /// How severe it is.
    pub severity: Severity,
    /// The slang diagnostic code name (e.g. `"UnknownModule"`).
    pub code: String,
    /// The formatted message.
    pub message: String,
}

/// A document symbol (a top-level definition: module, interface, program).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentSymbol {
    /// The symbol's name.
    pub name: String,
    /// Its slang symbol kind.
    pub kind: SymbolKind,
    /// Its source range.
    pub range: Range,
}

/// A source location: a file path and a range within it — the shape of a
/// `textDocument/definition` result or one `textDocument/references` entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Location {
    /// The file's path (as it was set on the [`Workspace`]).
    pub path: String,
    /// The range within that file.
    pub range: Range,
}

/// A `textDocument/hover` result: the range the hover applies to and its
/// markdown contents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hover {
    /// The identifier range the hover describes.
    pub range: Range,
    /// Markdown contents, e.g. `` `logic [7:0] x` — Variable ``.
    pub contents: String,
}

/// Maps byte offsets in a source file to LSP [`Position`]s.
pub struct LineIndex {
    text: String,
    /// Byte offset at which each line starts.
    line_starts: Vec<usize>,
}

impl LineIndex {
    /// Builds an index over `text`.
    pub fn new(text: &str) -> LineIndex {
        let mut line_starts = vec![0usize];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }
        LineIndex {
            text: text.to_string(),
            line_starts,
        }
    }

    /// The LSP position of a byte offset (clamped to the end of the text).
    pub fn position(&self, byte: usize) -> Position {
        let byte = byte.min(self.text.len());
        // The line is the last line whose start is <= byte.
        let line = self
            .line_starts
            .partition_point(|&s| s <= byte)
            .saturating_sub(1);
        let line_start = self.line_starts[line];
        // UTF-16 code units from the line start to `byte`.
        let character = self.text[line_start..byte]
            .chars()
            .map(char::len_utf16)
            .sum::<usize>();
        Position {
            line: line as u32,
            character: character as u32,
        }
    }

    /// The LSP range of a byte span.
    pub fn range(&self, span: core::ops::Range<usize>) -> Range {
        Range {
            start: self.position(span.start),
            end: self.position(span.end.max(span.start)),
        }
    }

    /// The byte offset of an LSP [`Position`] — the inverse of
    /// [`position`](Self::position). `character` is counted in UTF-16 code
    /// units and clamped to the line's end; an out-of-range line clamps to the
    /// end of the text.
    pub fn offset(&self, pos: Position) -> usize {
        let line = pos.line as usize;
        let Some(&line_start) = self.line_starts.get(line) else {
            return self.text.len();
        };
        let mut utf16 = 0usize;
        let mut byte = line_start;
        for ch in self.text[line_start..].chars() {
            if ch == '\n' || utf16 >= pos.character as usize {
                break;
            }
            utf16 += ch.len_utf16();
            byte += ch.len_utf8();
        }
        byte
    }
}

/// Diagnostics for one file in the workspace: both parse-level (lex /
/// preprocess / syntax, from the file's tree) and semantic (elaboration errors
/// such as unknown modules or undeclared identifiers, from the elaborated
/// design), the way a real editor shows them.
pub fn diagnostics(ws: &mut Workspace, path: &str) -> Vec<Diagnostic> {
    let Some(text) = ws.file_text(path).map(str::to_owned) else {
        return Vec::new();
    };
    let index = LineIndex::new(&text);
    let mut out = Vec::new();

    // Parse-level diagnostics from this file's tree.
    if let Some(Ok(tree)) = ws.tree(path) {
        for d in tree.diagnostics().items() {
            out.push(convert(d, &index));
        }
    }
    // Semantic diagnostics from the elaborated design, filtered to this file.
    if let Ok(design) = ws.design() {
        for d in design.diagnostics().items() {
            if d.file == path {
                out.push(convert(d, &index));
            }
        }
    }
    out
}

fn convert(d: &sv_lang::Diagnostic, index: &LineIndex) -> Diagnostic {
    Diagnostic {
        range: index.range(d.span()),
        severity: severity_of(d),
        code: d.code_name().to_string(),
        message: d.message.clone(),
    }
}

fn severity_of(d: &sv_lang::Diagnostic) -> Severity {
    use sv_lang::kinds::DiagSeverity::*;
    match d.severity {
        Fatal | Error => Severity::Error,
        Warning => Severity::Warning,
        Note => Severity::Information,
        Ignored => Severity::Hint,
    }
}

/// The top-level definitions (modules, interfaces, programs) declared in one
/// file, with their source ranges — the `textDocument/documentSymbol` response.
pub fn document_symbols(ws: &mut Workspace, path: &str) -> Vec<DocumentSymbol> {
    let Some(text) = ws.file_text(path).map(str::to_owned) else {
        return Vec::new();
    };
    let index = LineIndex::new(&text);
    let Some(Ok(tree)) = ws.tree(path) else {
        return Vec::new();
    };
    let Ok(design) = ws.design() else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for def in design.definitions() {
        // `syntax` returns Some only when the definition lives in *this* tree.
        if let Some(node) = def.syntax(&tree) {
            out.push(DocumentSymbol {
                name: def.name().to_string(),
                kind: def.kind(),
                range: index.range(node.byte_range()),
            });
        }
    }
    out.sort_by_key(|s| (s.range.start.line, s.range.start.character));
    out
}

/// True if `s` is exactly a simple SystemVerilog identifier (no whitespace,
/// no punctuation) — the test that distinguishes a leaf identifier node (a
/// `Declarator` or an `IdentifierName`) from a larger construct whose source
/// text merely happens to contain one.
fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

/// Finds the identifier at `offset`: the narrowest CST node covering the offset
/// whose source text is a single identifier. Returns its name and byte range.
///
/// slang's `Token` has no byte range, so rather than walk raw tokens we lean on
/// the tree's shape: an identifier reference is an `IdentifierName` node and a
/// declared name is a `Declarator` node, each with a byte range tight to the
/// identifier itself (no trivia). The narrowest such node covering the offset
/// is the identifier under the cursor.
fn identifier_at(
    root: sv_lang::Node<'_>,
    text: &str,
    offset: usize,
) -> Option<(String, core::ops::Range<usize>)> {
    // Two passes: first require the offset to fall strictly inside the node
    // (`start <= offset < end`); if that finds nothing, allow the cursor to sit
    // just past the identifier's end (`offset == end`), which some clients send.
    for inclusive_end in [false, true] {
        let mut best: Option<core::ops::Range<usize>> = None;
        for node in root.descendants() {
            let r = node.byte_range();
            if r.start >= r.end {
                continue;
            }
            let covers =
                r.start <= offset && (offset < r.end || (inclusive_end && offset == r.end));
            if !covers {
                continue;
            }
            let Some(slice) = text.get(r.clone()) else {
                continue;
            };
            if !is_identifier(slice) {
                continue;
            }
            let narrower = best
                .as_ref()
                .is_none_or(|b| (r.end - r.start) < (b.end - b.start));
            if narrower {
                best = Some(r);
            }
        }
        if let Some(r) = best {
            return Some((text[r.clone()].to_string(), r));
        }
    }
    None
}

/// The narrowest byte-width of an ancestor *scope* of `sym` whose declaration
/// syntax (in `tree`) covers `offset` — used to pick, among several symbols
/// sharing a name, the one whose enclosing scope actually contains the cursor.
fn enclosing_scope_width(
    sym: sv_lang::Symbol<'_>,
    tree: &sv_lang::SyntaxTree,
    offset: usize,
) -> Option<usize> {
    let mut cur = sym.parent();
    let mut best: Option<usize> = None;
    while let Some(scope) = cur {
        if let Some(node) = scope.syntax(tree) {
            let r = node.byte_range();
            if r.start <= offset && offset < r.end {
                let w = r.end - r.start;
                best = Some(best.map_or(w, |b| b.min(w)));
            }
        }
        cur = scope.parent();
    }
    best
}

/// Resolves the identifier at `offset` to the design symbol it names, returning
/// the symbol plus the identifier's name and byte range.
///
/// A cursor on a *declaration* (a symbol's own `syntax` node covers the offset)
/// resolves to that symbol; otherwise the offset is a *reference*, resolved to
/// the same-named symbol whose enclosing scope most tightly contains the cursor
/// (falling back to any symbol of that name).
fn resolve_symbol<'d>(
    design: &'d sv_lang::Design,
    tree: &sv_lang::SyntaxTree,
    text: &str,
    offset: usize,
) -> Option<(sv_lang::Symbol<'d>, String, core::ops::Range<usize>)> {
    let (name, id_range) = identifier_at(tree.root(), text, offset)?;

    // Every symbol reachable in the design; filtered to this file via `syntax`.
    let mut syms: Vec<sv_lang::Symbol<'d>> = Vec::new();
    design.root().visit(|s| {
        syms.push(s);
        sv_lang::Walk::Continue
    });

    // 1. Declaration hit: a same-named symbol whose own syntax covers the offset.
    for &s in &syms {
        if s.name() == name
            && let Some(node) = s.syntax(tree)
        {
            let r = node.byte_range();
            if r.start <= offset && offset < r.end {
                return Some((s, name, id_range));
            }
        }
    }

    // 2. Reference: the same-named symbol whose enclosing scope tightest-covers
    //    the offset. Ties and scope-less matches fall back to the first found.
    let mut best: Option<(Option<usize>, sv_lang::Symbol<'d>)> = None;
    for &s in &syms {
        if s.name() != name {
            continue;
        }
        let width = enclosing_scope_width(s, tree, offset);
        let better = match (&best, width) {
            (None, _) => true,
            (Some((Some(bw), _)), Some(w)) => w < *bw,
            (Some((None, _)), Some(_)) => true,
            _ => false,
        };
        if better {
            best = Some((width, s));
        }
    }
    best.map(|(_, s)| (s, name, id_range))
}

/// Human-readable hover markdown for a symbol: `` `<type> <name>` — <Kind> ``
/// for a value, the instantiated module for an instance, or just the name and
/// kind otherwise.
fn hover_contents(sym: &sv_lang::Symbol<'_>) -> String {
    let kind = format!("{:?}", sym.kind());
    if sym.kind() == SymbolKind::Instance {
        match sym.instance_definition() {
            Some(def) => format!("`{}` — {} of module `{}`", sym.name(), kind, def.name()),
            None => format!("`{}` — {}", sym.name(), kind),
        }
    } else if let Some(ty) = sym.value_type() {
        format!("`{} {}` — {}", ty.to_sv_string(), sym.name(), kind)
    } else {
        format!("`{}` — {}", sym.name(), kind)
    }
}

/// The `textDocument/hover` response: describes the identifier under the cursor
/// (its name, kind and — for a value — type; for an instance, the module it
/// instantiates), or `None` if the position is not on a resolvable identifier.
pub fn hover(ws: &mut Workspace, path: &str, pos: Position) -> Option<Hover> {
    let text = ws.file_text(path)?.to_owned();
    let index = LineIndex::new(&text);
    let offset = index.offset(pos);
    let tree = ws.tree(path)?.ok()?;
    let design = ws.design().ok()?;
    let (sym, _name, id_range) = resolve_symbol(&design, &tree, &text, offset)?;
    Some(Hover {
        range: index.range(id_range),
        contents: hover_contents(&sym),
    })
}

/// The `textDocument/definition` response: the declaration site of the symbol
/// the identifier under the cursor names. A cursor already on a declaration
/// resolves to that declaration itself.
pub fn definition(ws: &mut Workspace, path: &str, pos: Position) -> Option<Location> {
    let text = ws.file_text(path)?.to_owned();
    let index = LineIndex::new(&text);
    let offset = index.offset(pos);
    let tree = ws.tree(path)?.ok()?;
    let design = ws.design().ok()?;
    let (sym, _name, _id_range) = resolve_symbol(&design, &tree, &text, offset)?;
    let node = sym.syntax(&tree)?;
    Some(Location {
        path: path.to_string(),
        range: index.range(node.byte_range()),
    })
}

/// The `textDocument/references` response: every occurrence of the symbol under
/// the cursor within the same file, its declaration included.
///
/// **Limitation (by design, for the demo):** this is a *name-based* scan of the
/// one file's CST — it returns every identifier node whose text equals the
/// symbol's name, without re-resolving each occurrence, so it does not account
/// for shadowing by a distinct same-named symbol in a nested scope, and does
/// not search other files in the workspace. A production server would resolve
/// each candidate back to the target symbol and scan every file.
pub fn references(ws: &mut Workspace, path: &str, pos: Position) -> Vec<Location> {
    let Some(text) = ws.file_text(path).map(str::to_owned) else {
        return Vec::new();
    };
    let index = LineIndex::new(&text);
    let offset = index.offset(pos);
    let Some(Ok(tree)) = ws.tree(path) else {
        return Vec::new();
    };
    let Ok(design) = ws.design() else {
        return Vec::new();
    };
    let Some((_sym, name, _id_range)) = resolve_symbol(&design, &tree, &text, offset) else {
        return Vec::new();
    };

    let mut ranges: Vec<core::ops::Range<usize>> = Vec::new();
    for node in tree.root().descendants() {
        let r = node.byte_range();
        if r.start >= r.end {
            continue;
        }
        if text.get(r.clone()) == Some(name.as_str()) {
            ranges.push(r);
        }
    }
    ranges.sort_by_key(|r| r.start);
    ranges.dedup();
    ranges
        .into_iter()
        .map(|r| Location {
            path: path.to_string(),
            range: index.range(r),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_index_maps_bytes_to_positions() {
        let text = "ab\ncd\n\u{00e9}f\n"; // é is 2 UTF-8 bytes, 1 UTF-16 unit
        let li = LineIndex::new(text);
        assert_eq!(
            li.position(0),
            Position {
                line: 0,
                character: 0
            }
        );
        assert_eq!(
            li.position(1),
            Position {
                line: 0,
                character: 1
            }
        );
        assert_eq!(
            li.position(3),
            Position {
                line: 1,
                character: 0
            }
        );
        // Byte 6 starts line 2; the 'f' is one UTF-16 unit past 'é'.
        assert_eq!(
            li.position(6),
            Position {
                line: 2,
                character: 0
            }
        );
        assert_eq!(
            li.position(8),
            Position {
                line: 2,
                character: 1
            }
        );
    }

    #[test]
    fn diagnostics_have_ranges() {
        let mut ws = Workspace::new();
        ws.set_file("m.sv", "module m; assign w = nope; endmodule\n");
        let diags = diagnostics(&mut ws, "m.sv");
        assert!(
            diags.iter().any(|d| d.severity == Severity::Error),
            "expected an error diagnostic, got {diags:?}"
        );
        // Every diagnostic points somewhere on the (single) line.
        assert!(diags.iter().all(|d| d.range.start.line == 0));
    }

    #[test]
    fn document_symbols_lists_definitions() {
        let mut ws = Workspace::new();
        ws.set_file(
            "top.sv",
            "module alu; endmodule\ninterface bus; endinterface\nmodule cpu; endmodule\n",
        );
        let syms = document_symbols(&mut ws, "top.sv");
        let names: Vec<&str> = syms.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"alu"));
        assert!(names.contains(&"cpu"));
        assert!(names.contains(&"bus"));
        // `interface bus` starts on line 1.
        let bus = syms.iter().find(|s| s.name == "bus").unwrap();
        assert_eq!(bus.range.start.line, 1);
    }

    #[test]
    fn incremental_edit_updates_results() {
        let mut ws = Workspace::new();
        ws.set_file("m.sv", "module m; endmodule\n");
        assert_eq!(document_symbols(&mut ws, "m.sv").len(), 1);
        // Editing the file re-derives symbols.
        ws.set_file("m.sv", "module m; endmodule\nmodule n; endmodule\n");
        assert_eq!(document_symbols(&mut ws, "m.sv").len(), 2);
    }

    /// Byte offset of the first occurrence of `needle` in `src`, as an LSP
    /// position via a round-trip through [`LineIndex`].
    fn pos_of(src: &str, needle: &str) -> Position {
        let byte = src.find(needle).expect("needle not found");
        LineIndex::new(src).position(byte)
    }

    #[test]
    fn line_index_offset_inverts_position() {
        let text = "ab\ncd\n\u{00e9}f\n"; // é is 2 UTF-8 bytes, 1 UTF-16 unit
        let li = LineIndex::new(text);
        for byte in [0usize, 1, 2, 3, 5, 6, 8] {
            let pos = li.position(byte);
            assert_eq!(li.offset(pos), byte, "round-trip failed at byte {byte}");
        }
        // A character past the end of a line clamps to the newline.
        assert_eq!(
            li.offset(Position {
                line: 0,
                character: 99
            }),
            2
        );
    }

    #[test]
    fn hover_reports_name_kind_and_type() {
        let mut ws = Workspace::new();
        let src = "module m; logic [7:0] x; endmodule\n";
        ws.set_file("m.sv", src);
        let hov = hover(&mut ws, "m.sv", pos_of(src, "x")).expect("expected a hover");
        assert!(hov.contents.contains('x'), "contents: {}", hov.contents);
        assert!(hov.contents.contains("logic"), "contents: {}", hov.contents);
        // The hover range covers exactly the `x` identifier.
        let x = src.find('x').unwrap();
        assert_eq!(hov.range, LineIndex::new(src).range(x..x + 1));
    }

    #[test]
    fn definition_from_use_points_at_declaration() {
        let mut ws = Workspace::new();
        let src = "module m; logic x; logic y; assign y = x; endmodule\n";
        ws.set_file("m.sv", src);
        // The `x` inside `assign y = x` is the *use* (the second `x`).
        let use_byte = src.rfind('x').unwrap();
        let pos = LineIndex::new(src).position(use_byte);
        let loc = definition(&mut ws, "m.sv", pos).expect("expected a definition");
        assert_eq!(loc.path, "m.sv");
        // The declaration range covers the declared `x` (the Declarator node).
        let decl_byte = src.find('x').unwrap();
        let decl_pos = LineIndex::new(src).position(decl_byte);
        assert_eq!(loc.range.start, decl_pos);
        assert!(loc.range.start.line == 0 && loc.range.start.character < pos.character);
    }

    #[test]
    fn references_include_declaration_and_use() {
        let mut ws = Workspace::new();
        let src = "module m; logic x; logic y; assign y = x; endmodule\n";
        ws.set_file("m.sv", src);
        // Position on the declaration of `x`.
        let decl_pos = LineIndex::new(src).position(src.find('x').unwrap());
        let refs = references(&mut ws, "m.sv", decl_pos);
        assert_eq!(
            refs.len(),
            2,
            "expected declaration + one use, got {refs:?}"
        );
        let index = LineIndex::new(src);
        // The declaration `x` and the `x` in `assign` are both present.
        let decl = index.range(src.find('x').unwrap()..src.find('x').unwrap() + 1);
        let use_ = index.range(src.rfind('x').unwrap()..src.rfind('x').unwrap() + 1);
        let ranges: Vec<Range> = refs.iter().map(|l| l.range).collect();
        assert!(ranges.contains(&decl), "missing declaration: {ranges:?}");
        assert!(ranges.contains(&use_), "missing use: {ranges:?}");
    }

    #[test]
    fn references_from_use_also_finds_all() {
        let mut ws = Workspace::new();
        let src = "module m; logic x; logic y; assign y = x; endmodule\n";
        ws.set_file("m.sv", src);
        // Starting from the *use* resolves the same symbol and finds both sites.
        let use_pos = LineIndex::new(src).position(src.rfind('x').unwrap());
        assert_eq!(references(&mut ws, "m.sv", use_pos).len(), 2);
    }

    #[test]
    fn hover_off_identifier_is_none() {
        let mut ws = Workspace::new();
        let src = "module m; logic x; endmodule\n";
        ws.set_file("m.sv", src);
        // Column 0 sits on the `module` keyword, not a resolvable identifier.
        let none = hover(
            &mut ws,
            "m.sv",
            Position {
                line: 0,
                character: 0,
            },
        );
        assert!(none.is_none());
    }
}
