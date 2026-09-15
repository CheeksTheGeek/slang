//! The command-line driver: slang's own flags, then trees and a design.

use sv_lang::{Compilation, Driver, ParseOptions, Session};

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

#[test]
fn parse_options_round_trip_every_field() {
    // A freshly created ParseOptions has every field false...
    let mut options = ParseOptions::new();
    assert!(!options.support_comments());
    assert!(!options.ignore_program_name());
    assert!(!options.expand_env_vars());
    assert!(!options.ignore_duplicates());

    // ...and each setter flips only its own field, readable back through the
    // matching getter.
    options.set_support_comments(true);
    assert!(options.support_comments());
    assert!(!options.ignore_program_name());
    assert!(!options.expand_env_vars());
    assert!(!options.ignore_duplicates());

    options.set_ignore_program_name(true);
    assert!(options.ignore_program_name());

    options.set_expand_env_vars(true);
    assert!(options.expand_env_vars());

    options.set_ignore_duplicates(true);
    assert!(options.ignore_duplicates());

    // Flipping back to false works too.
    options.set_support_comments(false);
    assert!(!options.support_comments());
}

#[test]
fn ignore_duplicates_changes_real_parse_behavior() {
    // `--timescale` is a scalar (non-list) option: giving it twice is
    // rejected by default...
    let args = || ["prog", "--timescale", "1ns/1ps", "--timescale", "1ns/1ps"].map(str::to_string);
    let mut strict = Driver::new().unwrap();
    assert!(strict.parse_args(args()).is_err());

    // ...but succeeds once ParseOptions::ignore_duplicates is set, proving the
    // getter's value is the one that actually reached CommandLine::parse, not
    // just stored inertly.
    let mut options = ParseOptions::new();
    options.set_ignore_duplicates(true);
    let mut lenient = Driver::new().unwrap();
    lenient.parse_args_with_options(args(), &options).unwrap();
}

#[test]
fn compilation_options_expose_driver_configured_lists() {
    // topModules/paramOverrides/defaultLiblist have no way to be set except
    // through driver command-line flags (--top, -G, -L); a bare Compilation
    // never populates them, so this must go through a real Driver flow.
    let dir = std::env::temp_dir().join("sv_lang_driver_options_test");
    std::fs::create_dir_all(&dir).unwrap();
    let leaf = dir.join("leaf.sv");
    std::fs::write(
        &leaf,
        "module leaf #(parameter int N = 4)(input logic clk); endmodule\n",
    )
    .unwrap();

    let mut driver = Driver::new().unwrap();
    driver
        .parse_args([
            "prog".to_string(),
            leaf.to_string_lossy().into_owned(),
            "--top".into(),
            "leaf".into(),
            "-G".into(),
            "N=8".into(),
            "-L".into(),
            "worklib".into(),
        ])
        .unwrap();
    driver.process_options().unwrap();
    driver.parse_sources().unwrap();
    let design = driver.compile().unwrap();

    assert_eq!(design.top_modules(), vec!["leaf".to_string()]);
    assert_eq!(design.param_overrides(), vec!["N=8".to_string()]);
    assert_eq!(design.default_liblist(), vec!["worklib".to_string()]);

    // The override actually took effect, proving these aren't just recorded
    // strings but the real values the compilation was built with.
    let leaf_inst = design.top_instances().next().unwrap();
    let n = leaf_inst.parameters().find(|p| p.name() == "N").unwrap();
    assert_eq!(n.parameter_value().as_deref(), Some("8"));

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn compilation_options_lists_empty_without_driver_flags() {
    let design = {
        let session = sv_lang::Session::new();
        let mut comp = sv_lang::Compilation::new(&session).unwrap();
        comp.add_source("module m; endmodule\n").unwrap();
        comp.compile().unwrap()
    };
    assert!(design.top_modules().is_empty());
    assert!(design.param_overrides().is_empty());
    assert!(design.default_liblist().is_empty());
}

#[test]
fn driver_new_bare_has_no_standard_args_until_added() {
    let mut bare = Driver::new_bare().unwrap();
    // `--top` is not registered on a bare driver...
    assert!(
        bare.parse_args(["slang".to_string(), "--top".into(), "m".into()])
            .is_err()
    );

    // ...until slang's standard arguments are added, after which the exact
    // same flags parse successfully.
    bare.add_standard_args().unwrap();
    bare.parse_args(["slang".to_string(), "--top".into(), "m".into()])
        .unwrap();

    // A driver created with the standard args already present treats a
    // second call as a harmless no-op (it does not re-register and error).
    let mut normal = Driver::new().unwrap();
    normal.add_standard_args().unwrap();
    normal
        .parse_args(["slang".to_string(), "--top".into(), "m".into()])
        .unwrap();
}

#[test]
fn driver_language_version_reflects_std_flag() {
    let dir = std::env::temp_dir().join("sv_lang_language_version_test");
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("m.sv");
    std::fs::write(&f, "module m; endmodule\n").unwrap();

    let mut driver = Driver::new().unwrap();
    // Default before --std is processed.
    assert_eq!(driver.language_version(), 1); // 1800-2017
    driver
        .parse_args([
            "slang".to_string(),
            f.to_string_lossy().into_owned(),
            "--std".into(),
            "1800-2023".into(),
        ])
        .unwrap();
    driver.process_options().unwrap();
    assert_eq!(driver.language_version(), 2); // 1800-2023

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn driver_process_command_files_populates_metadata_and_defines() {
    let dir = std::env::temp_dir().join("sv_lang_command_file_test2");
    std::fs::create_dir_all(&dir).unwrap();
    let cmd_file = dir.join("opts.f");
    std::fs::write(&cmd_file, "-DFOO -DBAR=1\n").unwrap();

    let mut driver = Driver::new().unwrap();
    assert!(driver.command_file_metadata().is_empty());

    let ok = driver
        .process_command_files(&cmd_file.to_string_lossy(), false, false)
        .unwrap();
    assert!(ok);

    let files = driver.command_file_metadata();
    assert_eq!(files.len(), 1);
    let canonical = std::fs::canonicalize(&cmd_file).unwrap();
    assert_eq!(files[0].path, canonical.to_string_lossy());
    assert_eq!(
        files[0].defines,
        vec!["FOO".to_string(), "BAR=1".to_string()]
    );

    // A second, distinct command file adds a second entry rather than
    // replacing the first.
    let cmd_file2 = dir.join("opts2.f");
    std::fs::write(&cmd_file2, "-DBAZ\n").unwrap();
    driver
        .process_command_files(&cmd_file2.to_string_lossy(), false, false)
        .unwrap();
    let files = driver.command_file_metadata();
    assert_eq!(files.len(), 2);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn driver_process_command_files_fails_on_bad_options() {
    // A pattern matching zero files is not itself an error (an empty glob is
    // a no-op success); a command file whose own options are rejected is.
    let dir = std::env::temp_dir().join("sv_lang_command_file_bad_test");
    std::fs::create_dir_all(&dir).unwrap();
    let cmd_file = dir.join("bad.f");
    std::fs::write(&cmd_file, "--not-a-real-flag-xyz\n").unwrap();

    let mut driver = Driver::new().unwrap();
    let ok = driver
        .process_command_files(&cmd_file.to_string_lossy(), false, false)
        .unwrap();
    assert!(!ok);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn driver_diag_engine_counts_real_diagnostics() {
    let dir = std::env::temp_dir().join("sv_lang_diag_engine_test");
    std::fs::create_dir_all(&dir).unwrap();
    let bad = dir.join("bad.sv");
    // A completely undeclared identifier reference: a real semantic error.
    std::fs::write(
        &bad,
        "module m; initial $display(does_not_exist); endmodule\n",
    )
    .unwrap();

    let mut driver = Driver::new().unwrap();
    driver
        .parse_args(["slang".to_string(), bad.to_string_lossy().into_owned()])
        .unwrap();
    driver.process_options().unwrap();
    driver.parse_sources().unwrap();

    assert_eq!(driver.diag_engine().num_errors(), 0);

    let design = driver.compile().unwrap();
    driver.report_compilation(&design);

    assert!(driver.diag_engine().num_errors() > 0);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn driver_optionally_write_dep_files_is_a_harmless_no_op_without_config() {
    // No --depfile-style option was configured, so this must not panic or
    // write anything.
    let mut driver = Driver::new().unwrap();
    driver.optionally_write_dep_files();
}

#[test]
fn driver_analysis_options_reflects_default_flags() {
    let driver = Driver::new().unwrap();
    let opts = driver.analysis_options();
    // Driver::getAnalysisOptions() always turns on unused/shadow checks.
    assert_ne!(opts.flags & sv_lang_sys::SLANG_ANALYSIS_CHECK_UNUSED, 0);
    assert_ne!(opts.flags & sv_lang_sys::SLANG_ANALYSIS_CHECK_SHADOW, 0);
    assert_eq!(opts.max_case_analysis_steps, 65535);
    assert_eq!(opts.max_loop_analysis_steps, 65535);
}

/// `report_diagnostics` prints the CLI-style summary and returns whether the
/// run had no errors -- assert both directions of that bool, the way
/// `driver_diag_engine_counts_real_diagnostics` above already does for
/// `diag_engine().num_errors()`.
#[test]
fn driver_report_diagnostics_returns_false_on_real_errors() {
    let dir = std::env::temp_dir().join("sv_lang_report_diagnostics_errors_test");
    std::fs::create_dir_all(&dir).unwrap();
    let bad = dir.join("bad.sv");
    // A completely undeclared identifier reference: a real semantic error.
    std::fs::write(
        &bad,
        "module m; initial $display(does_not_exist); endmodule\n",
    )
    .unwrap();

    let mut driver = Driver::new().unwrap();
    driver
        .parse_args(["slang".to_string(), bad.to_string_lossy().into_owned()])
        .unwrap();
    driver.process_options().unwrap();
    driver.parse_sources().unwrap();

    let design = driver.compile().unwrap();
    driver.report_compilation(&design);

    assert!(!driver.report_diagnostics(/* quiet */ true));

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn driver_report_diagnostics_returns_true_when_clean() {
    let dir = std::env::temp_dir().join("sv_lang_report_diagnostics_clean_test");
    std::fs::create_dir_all(&dir).unwrap();
    let good = dir.join("good.sv");
    std::fs::write(&good, "module m; logic x; assign x = 1'b0; endmodule\n").unwrap();

    let mut driver = Driver::new().unwrap();
    driver
        .parse_args(["slang".to_string(), good.to_string_lossy().into_owned()])
        .unwrap();
    driver.process_options().unwrap();
    driver.parse_sources().unwrap();

    let design = driver.compile().unwrap();
    driver.report_compilation(&design);

    assert!(driver.report_diagnostics(/* quiet */ true));

    std::fs::remove_dir_all(&dir).ok();
}

/// `slang_driver_add_option`/`slang_driver_option_flag`/`_int`/`_string`: a
/// custom option of each kind round-trips through the command line, and an
/// option never given, or never registered, reads back as `None`.
#[test]
fn driver_custom_options_round_trip_every_kind() {
    use sv_lang::OptionKind;

    let mut driver = Driver::new().unwrap();
    driver
        .add_option("--params", OptionKind::Flag, "show params", "")
        .unwrap();
    driver
        .add_option("--max-depth", OptionKind::Int, "depth", "<depth>")
        .unwrap();
    driver
        .add_option("--inst-prefix", OptionKind::String, "prefix", "<prefix>")
        .unwrap();

    // Registering the same option twice is an error.
    assert!(
        driver
            .add_option("--params", OptionKind::Flag, "dup", "")
            .is_err()
    );

    driver
        .parse_args([
            "slang".to_string(),
            "--params".into(),
            "--max-depth".into(),
            "3".into(),
            "--inst-prefix".into(),
            "u_".into(),
        ])
        .unwrap();

    assert_eq!(driver.option_flag("--params"), Some(true));
    assert_eq!(driver.option_int("--max-depth"), Some(3));
    assert_eq!(driver.option_string("--inst-prefix").as_deref(), Some("u_"));

    // Registered but not given on this command line.
    let mut unset = Driver::new().unwrap();
    unset
        .add_option("--verbose", OptionKind::Flag, "verbose", "")
        .unwrap();
    unset.parse_args(["slang".to_string()]).unwrap();
    assert_eq!(unset.option_flag("--verbose"), None);

    // Never registered at all.
    assert_eq!(driver.option_int("--never-registered"), None);
}

/// `slang_compilation_source_manager`, via `Compilation::source_manager`:
/// the compilation reports back the exact session it was built from — not
/// merely an equal-content one, but the actual same underlying manager,
/// distinguishable from an unrelated session.
#[test]
fn compilation_source_manager_matches_its_own_session_only() {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    // The underlying manager is adopted from the first added tree (empty is
    // untracked, hence `add_source` before asserting).
    comp.add_source("module m; endmodule\n").unwrap();
    assert_eq!(comp.source_manager(), session);

    let other = Session::new();
    assert_ne!(comp.source_manager(), other);
}
