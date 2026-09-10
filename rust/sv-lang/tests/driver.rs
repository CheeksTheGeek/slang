//! The command-line driver: slang's own flags, then trees and a design.

use sv_lang::Driver;

#[test]
fn driver_runs_a_cli_flow() {
    let dir = std::env::temp_dir().join("sv_lang_driver_test");
    std::fs::create_dir_all(&dir).unwrap();
    let leaf = dir.join("leaf.sv");
    let top = dir.join("top.sv");
    std::fs::write(
        &leaf,
        "module leaf #(parameter int N = 4)(input logic clk); endmodule\n",
    )
    .unwrap();
    std::fs::write(
        &top,
        "module top; logic clk; leaf #(.N(8)) u(.clk(clk)); endmodule\n",
    )
    .unwrap();

    let mut driver = Driver::new().unwrap();
    driver
        .parse_args([
            "prog".to_string(),
            leaf.to_string_lossy().into_owned(),
            top.to_string_lossy().into_owned(),
            "--top".into(),
            "top".into(),
        ])
        .unwrap();
    driver.process_options().unwrap();
    driver.parse_sources().unwrap();

    // Two trees were parsed.
    assert_eq!(driver.trees().len(), 2);

    let design = driver.compile().unwrap();
    let tops: Vec<_> = design
        .top_instances()
        .map(|s| s.name().to_string())
        .collect();
    assert_eq!(tops, ["top"]);

    // The instance's parameter came from the -top module's override.
    let u = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap()
        .find("u")
        .unwrap();
    let n = u.parameters().find(|p| p.name() == "N").unwrap();
    assert_eq!(n.parameter_value().as_deref(), Some("8"));

    // The design outlives explicit driver use here; still valid because it
    // keeps the driver alive.
    assert!(
        !design.diagnostics().has_errors(),
        "{}",
        design.diagnostics()
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn design_outlives_driver() {
    let dir = std::env::temp_dir().join("sv_lang_driver_test2");
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("m.sv");
    std::fs::write(&f, "module m; logic [3:0] a; endmodule\n").unwrap();

    let design = {
        let mut driver = Driver::new().unwrap();
        driver
            .parse_args(["prog".to_string(), f.to_string_lossy().into_owned()])
            .unwrap();
        driver.process_options().unwrap();
        driver.parse_sources().unwrap();
        driver.compile().unwrap()
        // driver dropped here; the design keeps its source manager alive
    };

    let a = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap()
        .find("a")
        .unwrap();
    assert_eq!(a.value_type().unwrap().bit_width(), 4);
    // Rendering diagnostics needs the (kept-alive) source manager.
    let _ = design.diagnostics().rendered();

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn help_text_lists_standard_options() {
    let driver = Driver::new().unwrap();
    let help = driver.help_text("test overview");
    assert!(help.contains("test overview"));
    assert!(help.contains("--top") || help.contains("-top"));
}
