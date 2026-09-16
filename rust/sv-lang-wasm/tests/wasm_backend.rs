//! Drives slang running inside a WebAssembly sandbox end-to-end.

use std::sync::atomic::{AtomicUsize, Ordering};

use sv_lang_wasm::{Limits, Node, Slang, WasmDfaEvent, WasmFlowContext, WasmLattice};

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
    // The xtask-generated raw_* methods (~1015; see bridge.rs header) marshal the
    // C surface. We can't call every one without a compiled design's internal
    // handles, but we spot-check each distinct *marshalling shape* the generator
    // emits, so a codegen offset/ABI bug in any shape is caught:
    let mut slang = Slang::new().unwrap();
    // no-arg, i32-scalar return
    assert!(slang.raw_slang_syntax_kind_count().unwrap() > 500);
    assert!(slang.raw_slang_c_version().unwrap() >= 1);
    // no-arg, guest-pointer (handle/string-ptr) return
    assert!(slang.raw_slang_version_string().unwrap() != 0);
    assert!(slang.raw_slang_syntax_model_hash().unwrap() != 0);
    // one i32 arg, i32-scalar return
    assert!(slang.raw_slang_ast_kind_count(0).unwrap() > 0); // SLANG_AST_SYMBOL
    // one i32 arg, sret-`slang_str` return
    assert_eq!(slang.raw_slang_syntax_kind_name(0).unwrap(), "Unknown");
    // two i32 args, sret-`slang_str` return
    assert!(!slang.raw_slang_ast_kind_name(0, 1).unwrap().is_empty());
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

// Firing counters for the observer test below. This is the only test that
// touches them and `run_lattice` drives the guest synchronously on the calling
// thread, so plain globals are reliable even under the parallel test harness.
static OBS_CONDS: AtomicUsize = AtomicUsize::new(0);
static OBS_CASES: AtomicUsize = AtomicUsize::new(0);
static OBS_LOOPS: AtomicUsize = AtomicUsize::new(0);

#[test]
fn observer_hooks_fire_in_the_sandbox() {
    // The three observer hooks (on_case_begin / on_conditional_begin /
    // on_loop_begin) reach a custom WasmLattice across the sandbox boundary,
    // with a usable WasmFlowContext — full parity with the native Lattice's
    // observers (save eval_constant, documented native-only).
    #[derive(Clone)]
    struct Obs;
    impl WasmLattice for Obs {
        fn top() -> Self {
            Obs
        }
        fn join(&mut self, _other: &Self) {}
        fn transfer(&mut self, _event: &WasmDfaEvent) {}
        fn on_conditional_begin(&mut self, _stmt: Node, ctx: &WasmFlowContext) {
            assert!(!ctx.is_bad(), "analysis should not be in a bad state here");
            let _ = ctx.current_state_addr();
            OBS_CONDS.fetch_add(1, Ordering::Relaxed);
        }
        fn on_case_begin(&mut self, _stmt: Node, _ctx: &WasmFlowContext) {
            OBS_CASES.fetch_add(1, Ordering::Relaxed);
        }
        fn on_loop_begin(&mut self, _stmt: Node, _ctx: &WasmFlowContext) {
            OBS_LOOPS.fetch_add(1, Ordering::Relaxed);
        }
    }

    let mut slang = Slang::new().unwrap();
    let tree = slang
        .parse(
            "module m(input logic clk, input logic rst, input logic [1:0] sel);\n\
             logic [7:0] q, n;\n\
             always_ff @(posedge clk) begin\n\
               if (rst) q <= 8'd0;\n\
               else begin\n\
                 case (sel)\n\
                   2'd0: q <= n;\n\
                   default: q <= q;\n\
                 endcase\n\
                 for (int i = 0; i < 4; i++) q <= q + 8'd1;\n\
               end\n\
             end\n\
             endmodule\n",
        )
        .unwrap();
    let design = slang.compile(&tree).unwrap();
    let tops = slang.top_instances(&design).unwrap();
    let body = slang.instance_body(tops[0]).unwrap().unwrap();
    let proc = slang
        .members(body)
        .unwrap()
        .into_iter()
        .find(|&m| slang.kind_name(m).unwrap().contains("Procedural"))
        .expect("always_ff is a procedural block");

    let _exit: Obs = slang.run_lattice::<Obs>(&design, proc).unwrap();

    assert!(
        OBS_CONDS.load(Ordering::Relaxed) >= 1,
        "on_conditional_begin should fire for the if/else"
    );
    assert!(
        OBS_CASES.load(Ordering::Relaxed) >= 1,
        "on_case_begin should fire for the case"
    );
    assert!(
        OBS_LOOPS.load(Ordering::Relaxed) >= 1,
        "on_loop_begin should fire for the for-loop"
    );
}

#[test]
fn diagnostics_are_visible_in_the_sandbox() {
    let mut slang = Slang::new().unwrap();

    // A clean design has no diagnostics.
    let ok = slang.parse("module m; logic x; endmodule\n").unwrap();
    assert!(slang.diagnostics(&ok).unwrap().is_empty());

    // A malformed design parses (error recovery) but carries diagnostics that
    // are now visible — previously the sandbox dropped them silently.
    let bad = slang.parse("module m; logic [ ; endmodule\n").unwrap();
    let diags = slang.diagnostics(&bad).unwrap();
    assert!(!diags.is_empty(), "expected parse diagnostics, got none");
    let rendered = slang.diagnostics_render(&bad).unwrap();
    assert!(
        !rendered.is_empty(),
        "expected rendered diagnostics text, got empty"
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

#[test]
fn precompiled_sandbox_works() {
    // Precompile the module to portable native bytes, then drive a sandbox
    // loaded from them. This is the serverless/read-only path: embed the bytes
    // at build time and cold-start with no compilation and no cache directory.
    let cwasm = Slang::precompile().expect("precompile");
    assert!(cwasm.len() > 1_000_000, "native code should be substantial");
    // SAFETY: the bytes came from precompile() in this same build.
    let mut slang =
        unsafe { Slang::from_precompiled(&cwasm, Limits::default()) }.expect("from_precompiled");
    let tree = slang
        .parse("module top; logic [3:0] q; endmodule\n")
        .unwrap();
    assert_eq!(slang.module_count(&tree).unwrap(), 1);
    let design = slang.compile(&tree).unwrap();
    assert_eq!(slang.top_instances(&design).unwrap().len(), 1);
}
