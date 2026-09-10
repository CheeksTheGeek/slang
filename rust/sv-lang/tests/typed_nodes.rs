//! Exercises the generated typed node views.

use sv_lang::nodes::{
    CompilationUnitSyntax, MemberSyntax, ModuleDeclarationSyntax, ModuleHeaderSyntax,
};
use sv_lang::{AstNode, Session};

const SRC: &str = "\
module counter #(parameter int WIDTH = 8) (input logic clk, output logic [WIDTH-1:0] q);
    logic [WIDTH-1:0] next;
endmodule
module alu; endmodule
";

#[test]
fn typed_module_declarations() {
    let session = Session::new();
    let tree = session.parse(SRC).unwrap();

    let unit = CompilationUnitSyntax::cast(tree.root()).expect("root is a compilation unit");

    // The members list yields typed MemberSyntax views; two are modules.
    let modules: Vec<ModuleDeclarationSyntax> = unit
        .members()
        .iter()
        .filter_map(|m| match m {
            MemberSyntax::ModuleDeclaration(decl) => Some(decl),
            _ => None,
        })
        .collect();
    assert_eq!(modules.len(), 2);

    let counter = modules[0];
    let header: ModuleHeaderSyntax = counter.header();
    assert_eq!(header.module_keyword().unwrap().text(), "module");
    assert_eq!(header.name().unwrap().text(), "counter");

    // Parameter port list and ports are present as typed optional members.
    assert!(header.parameters().is_some());
    assert!(header.ports().is_some());

    // The second module has neither.
    let alu_header = modules[1].header();
    assert_eq!(alu_header.name().unwrap().text(), "alu");
    assert!(alu_header.parameters().is_none());
    assert!(alu_header.ports().is_none());
}

#[test]
fn casting_respects_kind() {
    let session = Session::new();
    let tree = session.parse("module m; endmodule\n").unwrap();
    let root = tree.root();

    // The root is a CompilationUnit, not a ModuleDeclaration.
    assert!(CompilationUnitSyntax::cast(root).is_some());
    assert!(ModuleDeclarationSyntax::cast(root).is_none());

    // syntax() round-trips back to the underlying node.
    let unit = CompilationUnitSyntax::cast(root).unwrap();
    assert_eq!(unit.syntax(), root);

    // A module's members are empty here.
    let module = unit
        .members()
        .iter()
        .find_map(|m| match m {
            MemberSyntax::ModuleDeclaration(d) => Some(d),
            _ => None,
        })
        .unwrap();
    assert_eq!(module.members().iter().count(), 0);
    assert!(module.members().is_empty());
    assert_eq!(module.endmodule().unwrap().text(), "endmodule");
}
