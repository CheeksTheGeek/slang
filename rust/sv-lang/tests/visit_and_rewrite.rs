//! Visitor control flow and text-preserving rewriting.

use sv_lang::green::{NodeExt, SyntaxEditor};
use sv_lang::kinds::{SyntaxKind, TokenKind};
use sv_lang::{Child, Session, Walk};

#[test]
fn visitor_control_flow() {
    let session = Session::new();
    let tree = session
        .parse("module a; initial begin x = 1; end endmodule\nmodule b; endmodule\n")
        .unwrap();

    // Skip descending into module bodies: exactly two ModuleDeclaration nodes,
    // and nothing inside them visited.
    let mut modules = 0;
    let mut saw_statement = false;
    tree.root().visit(|node| {
        if node.kind() == SyntaxKind::ModuleDeclaration {
            modules += 1;
            Walk::Skip
        } else {
            if format!("{:?}", node.kind()).contains("Statement") {
                saw_statement = true;
            }
            Walk::Continue
        }
    });
    assert_eq!(modules, 2);
    assert!(
        !saw_statement,
        "Skip should have prevented descending into bodies"
    );

    // Break stops the whole walk at the first module.
    let mut seen = 0;
    let completed = tree.root().visit(|node| {
        seen += 1;
        if node.kind() == SyntaxKind::ModuleDeclaration {
            Walk::Break
        } else {
            Walk::Continue
        }
    });
    assert!(!completed);
    assert!(seen >= 1);

    // visit_with_tokens sees tokens too.
    let mut idents = 0;
    tree.root().visit_with_tokens(|child| {
        if let Child::Token(t) = child
            && t.kind() == TokenKind::Identifier
        {
            idents += 1;
        }
        Walk::Continue
    });
    assert!(idents >= 2); // a, b (and x)
}

#[test]
fn typed_find_helpers() {
    use sv_lang::nodes::ModuleDeclarationSyntax;
    let session = Session::new();
    let tree = session.parse("module top; endmodule\n").unwrap();

    let module: ModuleDeclarationSyntax = tree.root().find_first().unwrap();
    assert_eq!(module.header().name().unwrap().text(), "top");
    assert_eq!(tree.root().find_all::<ModuleDeclarationSyntax>().count(), 1);
}

#[test]
fn rename_codemod_round_trips() {
    let session = Session::new();
    let src = "module m;\n  logic old_sig;\n  assign old_sig = 1'b0;\nendmodule\n";
    let tree = session.parse(src).unwrap();

    // Mirror into a standalone tree, then rewrite every `old_sig` identifier
    // token to `new_sig` by text splice.
    let mirror = tree.mirror();
    let mut editor = SyntaxEditor::new(&mirror);
    let mut renamed = 0;
    for token in mirror
        .descendants_with_tokens()
        .filter_map(|e| e.into_token())
    {
        if token.text() == "old_sig" {
            editor.replace(token, "new_sig");
            renamed += 1;
        }
    }
    assert_eq!(renamed, 2);
    let rewritten = editor.finish();
    assert_eq!(
        rewritten,
        "module m;\n  logic new_sig;\n  assign new_sig = 1'b0;\nendmodule\n"
    );

    // The rewritten text re-parses cleanly — the codemod produced valid source.
    let reparsed = session.parse(&rewritten).unwrap();
    assert!(!reparsed.diagnostics().has_errors());
    assert_eq!(reparsed.text(), rewritten);
}

#[test]
fn insert_and_delete() {
    let session = Session::new();
    let tree = session.parse("module m; endmodule\n").unwrap();
    let mirror = tree.mirror();

    let module = mirror
        .descendants()
        .find(|n| n.node_kind() == Some(SyntaxKind::ModuleDeclaration))
        .unwrap();

    let mut editor = SyntaxEditor::new(&mirror);
    editor.insert_before(module, "// generated\n");
    assert_eq!(editor.finish(), "// generated\nmodule m; endmodule\n");
}

#[test]
fn generated_visitor_trait() {
    use sv_lang::nodes::{DataDeclarationSyntax, ModuleDeclarationSyntax};
    use sv_lang::visitor::{Visitor, visit, walk_module_declaration_syntax};

    #[derive(Default)]
    struct Collect {
        modules: Vec<String>,
        data_decls: usize,
    }
    impl<'t> Visitor<'t> for Collect {
        fn visit_module_declaration_syntax(&mut self, node: ModuleDeclarationSyntax<'t>) {
            if let Some(name) = node.header().name() {
                self.modules.push(name.text().to_string());
            }
            walk_module_declaration_syntax(self, node); // descend
        }
        fn visit_data_declaration_syntax(&mut self, _node: DataDeclarationSyntax<'t>) {
            self.data_decls += 1;
            // no walk: prune below data declarations
        }
    }

    let session = Session::new();
    let tree = session
        .parse("module a; logic x; logic y; endmodule\nmodule b; endmodule\n")
        .unwrap();
    let mut c = Collect::default();
    visit(&mut c, tree.root());
    assert_eq!(c.modules, ["a", "b"]);
    assert_eq!(c.data_decls, 2);
}
