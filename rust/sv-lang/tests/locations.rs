//! Source-range / syntax back-mapping and public location resolution — the
//! byte-identity path a transpiler needs (reported gaps: Expression had no
//! range()/syntax(), and location lookup was crate-private).

use sv_lang::{Compilation, Session};

#[test]
fn expression_source_range_syntax_and_location() {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    let src = "module m;\n  logic [7:0] x, y;\n  wire [7:0] s = x & y;\nendmodule\n";
    comp.add_source(src).unwrap();
    let design = comp.compile().unwrap();
    let body = design.top_instances().next().unwrap().instance_body().unwrap();
    let init = body.find("s").unwrap().initializer().unwrap();

    // 1. A real byte range with buffer ids.
    let range = init.source_range();
    assert!(range.end.offset > range.start.offset);
    assert_ne!(range.start.buffer, 0);

    // 2. The range maps back to the exact source text (byte identity).
    let (lo, hi) = (range.start.offset as usize, range.end.offset as usize);
    assert_eq!(&src[lo..hi], "x & y");

    // 3. The expression back-maps to its CST node.
    let tree = design.syntax_trees().into_iter().next().unwrap();
    let node = init.syntax(&tree).expect("expression has a syntax node");
    assert!(node.text().contains("x & y"));

    // 4. Public location resolution: byte loc -> file/line/column.
    let at = session
        .resolve_location(range.start)
        .expect("resolvable location");
    assert_eq!(at.line, 3); // the `wire ... = x & y;` line
    assert!(at.column > 1);
    assert!(!at.file.is_empty());

    // A null location resolves to None.
    assert!(
        session
            .resolve_location(sv_lang::SourceLoc {
                buffer: 0,
                offset: 0
            })
            .is_none()
    );
}
