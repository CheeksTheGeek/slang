//! A `randsequence` case item with several labels must keep its own
//! production: `createCaseProd` used to push one production per label but
//! read one per item, shifting every later item's production. Reported by the
//! Carcilator session (BR-randseq-2).

use sv_lang::{Compilation, RandSeqProdKind, Session};

const SRC: &str = "\
module m;
  int sel;
  initial randsequence(main)
    main : case (sel) 0, 1 : a; 3 : b; 2 : c; default : d; endcase ;
    a : { $display(\"a\"); } ;
    b : { $display(\"b\"); } ;
    c : { $display(\"c\"); } ;
    d : { $display(\"d\"); } ;
  endsequence
endmodule
";

#[test]
fn each_case_item_keeps_its_own_production() {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source(SRC).unwrap();
    let design = comp.compile().unwrap();

    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let block = body
        .members()
        .find(|s| s.kind() == sv_lang::kinds::SymbolKind::ProceduralBlock)
        .unwrap();
    let main = block.body().unwrap().statements()[0]
        .randsequence_first_production()
        .unwrap();
    let case = main.randseq_rule_prods(0).next().unwrap();
    assert_eq!(case.kind(), RandSeqProdKind::Case);
    assert_eq!(case.case_item_count(), 3);

    let target = |i| {
        case.case_item_item(i)
            .and_then(|p| p.item_target())
            .map(|s| s.name().to_string())
    };
    assert_eq!(case.case_item_expressions(0).count(), 2);
    assert_eq!(target(0).as_deref(), Some("a"));
    assert_eq!(target(1).as_deref(), Some("b"));
    assert_eq!(target(2).as_deref(), Some("c"));
}
