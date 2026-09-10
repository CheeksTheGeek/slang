//! Incremental re-parse behaviour of the workspace.

use sv_lang_db::Workspace;

#[test]
fn reuses_unchanged_files() {
    let mut ws = Workspace::new();
    ws.set_file("pkg.sv", "package p; localparam int W = 8; endpackage\n");
    ws.set_file(
        "a.sv",
        "module a; import p::*; logic [W-1:0] x; endmodule\n",
    );
    ws.set_file("b.sv", "module b; endmodule\n");

    // First build parses all three.
    let design = ws.design().unwrap();
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );
    let stats = ws.take_stats();
    assert_eq!(stats.parsed, 3);
    assert_eq!(stats.reparsed, 0);

    // No change: the next build reuses all three, parses none.
    let _ = ws.design().unwrap();
    let stats = ws.take_stats();
    assert_eq!(stats.parsed, 0);
    assert_eq!(stats.reparsed, 0);
    assert_eq!(stats.reused, 3);

    // Edit one file: only that file is re-parsed, the others reused.
    ws.set_file(
        "a.sv",
        "module a; import p::*; logic [W-1:0] y; endmodule\n",
    );
    let _ = ws.design().unwrap();
    let stats = ws.take_stats();
    assert_eq!(stats.reparsed, 1);
    assert_eq!(stats.reused, 2);
    assert_eq!(stats.parsed, 0);
}

#[test]
fn tracks_edits_semantically() {
    let mut ws = Workspace::new();
    ws.set_file("m.sv", "module m; logic [3:0] a; endmodule\n");
    let design = ws.design().unwrap();
    let a = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap()
        .find("a")
        .unwrap();
    assert_eq!(a.value_type().unwrap().bit_width(), 4);

    // Widen the vector; the new design reflects it.
    ws.set_file("m.sv", "module m; logic [15:0] a; endmodule\n");
    let design = ws.design().unwrap();
    let a = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap()
        .find("a")
        .unwrap();
    assert_eq!(a.value_type().unwrap().bit_width(), 16);
}

#[test]
fn removing_a_file_drops_its_cache() {
    let mut ws = Workspace::new();
    ws.set_file("a.sv", "module a; endmodule\n");
    ws.set_file("b.sv", "module b; endmodule\n");
    let _ = ws.design().unwrap();
    ws.take_stats();

    ws.remove_file("b.sv");
    let design = ws.design().unwrap();
    let defs: Vec<_> = design.definitions().map(|d| d.name().to_string()).collect();
    assert!(defs.contains(&"a".to_string()));
    assert!(!defs.contains(&"b".to_string()));
    // `a` was reused, nothing re-parsed.
    let stats = ws.take_stats();
    assert_eq!(stats.reused, 1);
    assert_eq!(stats.parsed + stats.reparsed, 0);
}
