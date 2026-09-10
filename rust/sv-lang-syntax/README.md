# sv-lang-syntax

Lossless SystemVerilog syntax trees in pure Rust, structurally identical to
[slang](https://sv-lang.com)'s concrete syntax tree.

This crate has **no native code**: it defines the tree representation (a
[`cstree`](https://crates.io/crates/cstree) green tree keyed by slang's own
`SyntaxKind`/`TokenKind`/`TriviaKind`), a builder, and cursors. Trees are
produced by the [`sv-lang`](https://crates.io/crates/sv-lang) crate, which
mirrors slang's parse result through one streaming call, and can then be
walked, edited and printed without slang in the loop — ideal for formatters,
refactoring tools and long-lived editor state.

Every node's children are exactly the members of its slang syntax struct, in
declaration order: member `k` is child `k`. Lists are nested `List` nodes,
absent optional members are `Absent` placeholder tokens, and trivia are
sibling tokens, so `text()` of any node reproduces its source bytes exactly.
