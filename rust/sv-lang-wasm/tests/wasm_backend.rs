//! Drives slang running inside a WebAssembly sandbox end-to-end.

use sv_lang_wasm::{Limits, Slang};

#[test]
fn fuel_limit_traps_instead_of_hanging() {
    // A tiny fuel budget must cause the guest to trap (an Err), not run
    // unbounded — this is the sandbox's run-time guard for untrusted input.
    let limits = Limits {
        max_memory: None,
        fuel: Some(50_000),
    };
    let result =
        Slang::with_limits(limits).and_then(|mut s| s.parse("module m; endmodule\n").map(|_| ()));
    assert!(
        result.is_err(),
        "a tiny fuel budget should trap, not succeed"
    );
}

#[test]
fn generous_limits_still_parse() {
    let mut slang = Slang::with_limits(Limits::default()).expect("load with default limits");
    let tree = slang
        .parse("module a; endmodule\nmodule b; endmodule\n")
        .expect("parse under default limits");
    assert_eq!(slang.module_count(&tree).unwrap(), 2);
}

#[test]
fn semantic_layer_elaborates_in_the_sandbox() {
    // The full semantic pipeline — compile, walk symbols, read types — runs
    // inside wasm over slang's real C ABI.
    let mut slang = Slang::new().unwrap();
    let tree = slang
        .parse("module m; logic [7:0] a, b; wire [7:0] s = a - b; endmodule\n")
        .unwrap();
    let design = slang.compile(&tree).unwrap();

    let tops = slang.top_instances(&design).unwrap();
    assert_eq!(tops.len(), 1);

    let body = slang.instance_body(tops[0]).unwrap().unwrap();
    let s = slang.find(body, "s").unwrap().expect("net s exists");
    assert_eq!(slang.name(s).unwrap(), "s");

    let ty = slang.value_type(s).unwrap().expect("s has a type");
    assert_eq!(slang.type_bit_width(ty).unwrap(), 8);
    assert!(slang.type_string(ty).unwrap().contains("logic"));

    let members = slang.members(body).unwrap();
    let names: Vec<String> = members.iter().map(|&m| slang.name(m).unwrap()).collect();
    assert!(names.iter().any(|n| n == "a"));
    assert!(names.iter().any(|n| n == "b"));
}

#[test]
fn behavioral_tree_walks_in_the_sandbox() {
    let mut slang = Slang::new().unwrap();
    let tree = slang
        .parse("module m; logic clk, x, y; always_ff @(posedge clk) x <= y; endmodule\n")
        .unwrap();
    let design = slang.compile(&tree).unwrap();
    let tops = slang.top_instances(&design).unwrap();
    let body = slang.instance_body(tops[0]).unwrap().unwrap();

    let members = slang.members(body).unwrap();
    let mut proc = None;
    for &m in &members {
        if slang.kind_name(m).unwrap().contains("Procedural") {
            proc = Some(m);
            break;
        }
    }
    let proc = proc.expect("always_ff is a procedural block");
    let root = slang
        .body(proc)
        .unwrap()
        .expect("procedural block has a body");
    // We can walk the elaborated statement/expression tree inside the sandbox.
    let kids = slang.sem_children(root).unwrap();
    let _ = kids;
    assert!(slang.kind_name(root).unwrap().len() > 3);
}

#[test]
fn generated_raw_bridge_marshals() {
    // The xtask-generated raw_* methods (196 of them) marshal the full C
    // surface; spot-check a scalar-return and an sret-string-return one.
    let mut slang = Slang::new().unwrap();
    assert!(slang.raw_slang_syntax_kind_count().unwrap() > 500);
    assert_eq!(slang.raw_slang_syntax_kind_name(0).unwrap(), "Unknown");
    // Version string comes back as a guest pointer (raw layer returns the ptr).
    assert!(slang.raw_slang_version_string().unwrap() != 0);
}

#[test]
fn custom_dataflow_bridge_runs_in_the_sandbox() {
    // slang's custom DataFlow/Lattice bridge, driven across the wasm boundary:
    // the guest runs the analysis and calls back out to the host lattice.
    let mut slang = Slang::new().unwrap();
    let tree = slang
        .parse("module m; logic clk, x, y; always_ff @(posedge clk) x <= y; endmodule\n")
        .unwrap();
    let design = slang.compile(&tree).unwrap();
    let tops = slang.top_instances(&design).unwrap();
    let body = slang.instance_body(tops[0]).unwrap().unwrap();

    let members = slang.members(body).unwrap();
    let mut proc = None;
    for &m in &members {
        if slang.kind_name(m).unwrap().contains("Procedural") {
            proc = Some(m);
            break;
        }
    }
    let proc = proc.expect("always_ff is a procedural block");

    let written = slang.reaching_writes(&design, proc).unwrap();
    assert!(
        written.contains(&"x".to_string()),
        "expected `x` in the reaching-writes set, got {written:?}"
    );
}

#[test]
fn version_and_kinds() {
    let mut slang = Slang::new().expect("load wasm");
    let v = slang.version();
    assert!(v.starts_with("11."), "version = {v:?}");

    assert!(slang.syntax_kind_count() > 500);
    assert_eq!(slang.syntax_kind_name(0).unwrap(), "Unknown");
    // Some well-known kind exists somewhere in the table.
    let count = slang.syntax_kind_count();
    let mut saw_module = false;
    for k in 0..count {
        if slang.syntax_kind_name(k).unwrap() == "ModuleDeclaration" {
            saw_module = true;
            break;
        }
    }
    assert!(saw_module);
}

#[test]
fn parse_in_the_sandbox() {
    let mut slang = Slang::new().expect("load wasm");
    let tree = slang
        .parse("module top; endmodule\nmodule alu; endmodule\n")
        .expect("parse");
    assert_eq!(slang.root_kind_name(&tree).unwrap(), "CompilationUnit");
    assert_eq!(slang.module_count(&tree).unwrap(), 2);
}

#[test]
fn fork_shares_the_compiled_module() {
    // Forking reuses the process-wide compiled module; each fork is a fresh,
    // isolated, working instance.
    let a = Slang::new().unwrap();
    let mut b = a.fork().unwrap();
    let mut c = a.fork().unwrap();
    let tb = b.parse("module x; endmodule\n").unwrap();
    assert_eq!(b.module_count(&tb).unwrap(), 1);
    let tc = c
        .parse("module y; endmodule\nmodule z; endmodule\n")
        .unwrap();
    assert_eq!(c.module_count(&tc).unwrap(), 2);
}

#[test]
fn two_instances_are_independent() {
    // Each Slang is a fresh, isolated wasm instance.
    let mut a = Slang::new().unwrap();
    let mut b = Slang::new().unwrap();
    let ta = a.parse("module only_a; endmodule\n").unwrap();
    let tb = b
        .parse("module x; endmodule\nmodule y; endmodule\n")
        .unwrap();
    assert_eq!(a.module_count(&ta).unwrap(), 1);
    assert_eq!(b.module_count(&tb).unwrap(), 2);
}
