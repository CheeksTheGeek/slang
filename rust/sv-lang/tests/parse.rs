//! Integration tests.

use sv_lang::green::NodeExt;
use sv_lang::kinds::{SyntaxKind, TokenKind};
use sv_lang::{Child, Session};

const DESIGN: &str = "\
// a small design
package p;
    localparam int W = 8;
endpackage
module top #(parameter int N = 4) (input logic clk);
    logic [N-1:0] q;
    sub #(.M(N)) u_sub(.clk(clk));
endmodule
interface bus_if; endinterface
";

#[test]
fn parses_and_lists_modules() {
    let session = Session::new();
    let tree = session.parse(DESIGN).unwrap();
    assert_eq!(tree.module_names().collect::<Vec<_>>(), ["top"]);
    assert!(!tree.diagnostics().has_errors(), "{}", tree.diagnostics());
    assert_eq!(tree.root().kind(), SyntaxKind::CompilationUnit);
}

#[test]
fn text_round_trips() {
    let session = Session::new();
    let tree = session.parse(DESIGN).unwrap();
    assert_eq!(tree.text(), DESIGN);
    // The mirror reproduces the source too, without borrowing slang.
    let green = tree.mirror();
    assert_eq!(green.text().to_string(), DESIGN);
}

#[test]
fn mirror_outlives_session() {
    let green = {
        let session = Session::new();
        let tree = session.parse("module m; endmodule\n").unwrap();
        tree.mirror()
        // tree and session dropped here
    };
    assert_eq!(green.text().to_string(), "module m; endmodule\n");
    assert_eq!(
        green.kind(),
        sv_lang::green::Kind::Node(SyntaxKind::CompilationUnit)
    );
    // Every module declaration node is reachable in the mirror.
    let modules = green
        .descendants()
        .filter(|n| n.node_kind() == Some(SyntaxKind::ModuleDeclaration))
        .count();
    assert_eq!(modules, 1);
}

#[test]
fn diagnostics_are_reported_not_errors() {
    let session = Session::new();
    // A syntax error: missing expression.
    let tree = session.parse("module m; int x = ; endmodule\n").unwrap();
    let diags = tree.diagnostics();
    assert!(diags.has_errors());
    assert!(diags.error_count() >= 1);
    assert!(diags.rendered().contains("error"));
    assert!(diags.items().iter().any(|d| d.message.contains("expected")));
    // Display renders slang's own formatted output.
    assert!(format!("{diags}").contains("expected"));
}

#[test]
fn cursor_navigation() {
    let session = Session::new();
    let tree = session.parse("module m; endmodule\n").unwrap();
    let root = tree.root();
    assert!(root.parent().is_none());

    let module = root.children().next().unwrap();
    assert_eq!(module.kind(), SyntaxKind::ModuleDeclaration);
    assert_eq!(module.parent().unwrap().kind(), SyntaxKind::CompilationUnit);

    let first = module.first_token().unwrap();
    assert_eq!(first.kind(), TokenKind::ModuleKeyword);
    assert_eq!(first.text(), "module");
    assert!(!first.is_missing());

    // The header holds the module name.
    let header = module.children().next().unwrap();
    let name = header
        .children_with_tokens()
        .find_map(|c| match c {
            Child::Token(t) if t.kind() == TokenKind::Identifier => Some(t),
            _ => None,
        })
        .unwrap();
    assert_eq!(name.text(), "m");
}

#[test]
fn descendants_are_preorder() {
    let session = Session::new();
    let tree = session
        .parse("module m; initial x = 1; endmodule\n")
        .unwrap();
    let kinds: Vec<_> = tree.root().descendants().map(|n| n.kind()).collect();
    assert_eq!(kinds[0], SyntaxKind::CompilationUnit);
    assert_eq!(kinds[1], SyntaxKind::ModuleDeclaration);
    assert!(kinds.contains(&SyntaxKind::ModuleHeader));
}

#[test]
fn parse_from_file() {
    let dir = std::env::temp_dir();
    let path = dir.join("sv_lang_parse_test.sv");
    std::fs::write(&path, "module fromfile; endmodule\n").unwrap();
    let session = Session::new();
    let tree = session.parse_file(&path).unwrap();
    assert_eq!(tree.module_names().collect::<Vec<_>>(), ["fromfile"]);
    std::fs::remove_file(&path).ok();
}

#[test]
fn missing_file_is_an_error() {
    let session = Session::new();
    let err = session.parse_file("/no/such/file.sv").unwrap_err();
    assert!(matches!(err, sv_lang::Error::Io(_)), "{err:?}");
}
