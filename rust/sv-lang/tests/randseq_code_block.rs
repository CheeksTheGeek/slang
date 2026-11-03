//! `Symbol::statement_block_body()` — the statement inside a `randsequence`
//! `{ ... }` code block. `RandSeqProd::code_block_block()` gives the block
//! symbol, but nothing reached its statement. Reported by the Carcilator
//! session (BR-randseq-1), which lowers `randsequence` and needs the code.

use sv_lang::kinds::SymbolKind;
use sv_lang::{Compilation, Design, Session, Symbol};

fn compile(src: &str) -> Design {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source(src).unwrap();
    comp.compile().unwrap()
}

/// Both code-block positions: between productions, and trailing a rule.
const SRC: &str = "\
module m;
  int x;
  initial randsequence(main)
    main : first { x = 1; } second { x = 2 + 3; } ;
    first : { x = 10; } ;
    second : { x = 20; } ;
  endsequence
endmodule
";

fn production<'d>(design: &'d Design, name: &str) -> Symbol<'d> {
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();
    let stmt = block.body().unwrap().statements()[0];
    let main = stmt.randsequence_first_production().unwrap();
    // The first production is `main`; the others are its siblings.
    if name == "main" {
        return main;
    }
    main.parent()
        .and_then(|p| p.find(name))
        .unwrap_or_else(|| panic!("no production {name:?}"))
}

#[test]
fn a_code_block_has_a_statement() {
    let design = compile(SRC);
    let main = production(&design, "main");
    let blocks: Vec<_> = main
        .randseq_rule_prods(0)
        .filter_map(|p| p.code_block_block())
        .collect();
    assert_eq!(blocks.len(), 2, "two code blocks between the productions");
    for b in &blocks {
        assert_eq!(b.kind(), SymbolKind::StatementBlock);
        let stmt = b.statement_block_body().expect("the block's statement");
        assert!(!stmt.children().is_empty());
    }
}

#[test]
fn the_statement_is_swept_so_its_expressions_are_readable() {
    // `x = 2 + 3;` is the second code block. Its right-hand side folds to 5, and
    // the type of every expression inside is already forced: reading them on the
    // shared, sealed design must not need the arena.
    let design = compile(SRC);
    let main = production(&design, "main");
    let second = main
        .randseq_rule_prods(0)
        .filter_map(|p| p.code_block_block())
        .nth(1)
        .unwrap();
    let stmt = second.statement_block_body().unwrap();

    let mut stack = vec![stmt.as_sem_node()];
    let mut folded = None;
    let mut seen = 0;
    while let Some(node) = stack.pop() {
        if let Some(e) = node.as_expression() {
            seen += 1;
            assert!(e.expr_type().is_some(), "expression type must be forced");
            if let Some(v) = e.constant_value().and_then(|c| c.as_i64()) {
                if v == 5 {
                    folded = Some(v);
                }
            }
        }
        stack.extend(node.children());
    }
    assert!(seen >= 3, "assignment, binary op and literals: {seen}");
    assert_eq!(folded, Some(5), "2 + 3 folds inside the code block");
}

#[test]
fn a_rules_trailing_code_block_has_a_statement() {
    let design = compile(
        "module m;\n  int x;\n  initial randsequence(main)\n    main : a := 3 { x = 7; } ;\n    a : ;\n  endsequence\nendmodule\n",
    );
    let main = production(&design, "main");
    let trailing = main
        .randseq_rule_code_block(0)
        .expect("the rule's trailing code block")
        .code_block_block()
        .unwrap();
    assert!(trailing.statement_block_body().is_some());
}

#[test]
fn other_symbols_have_no_statement_block_body() {
    let design = compile(SRC);
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    assert!(body.statement_block_body().is_none());
    assert!(body.find("x").unwrap().statement_block_body().is_none());
}
