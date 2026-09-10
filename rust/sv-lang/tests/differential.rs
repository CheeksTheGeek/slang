//! Differential correctness oracle: cross-check the Rust semantic API against
//! slang's own `--ast-json` output. Catches a C-API mapping bug that would be
//! self-consistent (and so invisible to our other tests) but wrong.
//!
//! Needs a `slang` binary: set `SLANG_BIN`, or build it
//! (`cmake -S . -B build-cli -DSLANG_INCLUDE_TOOLS=ON && cmake --build build-cli
//! --target slang_driver`). Skipped when absent.

use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::process::Command;

/// A per-source temp file (tests run in parallel, so the name must be unique).
fn temp_sv(src: &str) -> PathBuf {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    src.hash(&mut h);
    std::env::temp_dir().join(format!("svlang-diff-{:016x}.sv", h.finish()))
}

use sv_lang::kinds::SymbolKind;
use sv_lang::{Compilation, Session, Symbol, Walk};

/// Kinds that appear in ordinary user RTL and are stable across both sides.
fn tracked(kind: &str) -> bool {
    matches!(
        kind,
        "Instance"
            | "Variable"
            | "Net"
            | "Port"
            | "Parameter"
            | "TypeAlias"
            | "Subroutine"
            | "ProceduralBlock"
            | "GenerateBlock"
    )
}

fn slang_bin() -> Option<PathBuf> {
    if let Some(b) = std::env::var_os("SLANG_BIN") {
        let p = PathBuf::from(b);
        return p.is_file().then_some(p);
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for rel in ["build-cli/bin/slang", "build-cli/slang", "build/bin/slang"] {
        let p = root.join(rel);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// The set slang's own ast-json reports, as `(kind, name)`, excluding the
/// built-in `std` package subtree.
fn slang_set(bin: &PathBuf, src: &str) -> BTreeSet<(String, String)> {
    let file = temp_sv(src);
    std::fs::write(&file, src).unwrap();

    let out = Command::new(bin)
        .arg(&file)
        .arg("--ast-json")
        .arg("-")
        .arg("--quiet")
        .output()
        .expect("run slang");
    let json: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("slang --ast-json produced valid JSON");

    let mut set = BTreeSet::new();
    collect_json(&json, &mut set, false);
    set
}

fn collect_json(v: &serde_json::Value, set: &mut BTreeSet<(String, String)>, in_std: bool) {
    match v {
        serde_json::Value::Object(map) => {
            let kind = map.get("kind").and_then(|k| k.as_str());
            let name = map.get("name").and_then(|n| n.as_str());
            // Skip the std package and everything under it.
            let entering_std = in_std || (kind == Some("Package") && name == Some("std"));
            if !entering_std
                && let (Some(k), Some(n)) = (kind, name)
                && tracked(k)
                && !n.is_empty()
            {
                set.insert((k.to_string(), n.to_string()));
            }
            for (key, child) in map {
                if key != "addr" {
                    collect_json(child, set, entering_std);
                }
            }
        }
        serde_json::Value::Array(a) => {
            for child in a {
                collect_json(child, set, in_std);
            }
        }
        _ => {}
    }
}

/// The set the Rust API reports, walking from every top instance.
fn rust_set(src: &str) -> BTreeSet<(String, String)> {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source(src).unwrap();
    let design = comp.compile().unwrap();

    fn push(set: &mut BTreeSet<(String, String)>, s: Symbol<'_>) {
        let k = format!("{:?}", s.kind());
        if tracked(&k) && !s.name().is_empty() && s.kind() != SymbolKind::Root {
            set.insert((k, s.name().to_string()));
        }
    }

    let mut set = BTreeSet::new();
    for top in design.top_instances() {
        // include the top instance itself + its whole body.
        push(&mut set, top);
        if let Some(body) = top.instance_body() {
            body.visit(|s| {
                push(&mut set, s);
                Walk::Continue
            });
        }
    }
    set
}

fn check(src: &str) {
    let Some(bin) = slang_bin() else {
        eprintln!("no slang binary (set SLANG_BIN or build build-cli); skipping");
        return;
    };
    let slang = slang_set(&bin, src);
    let rust = rust_set(src);
    // Every symbol the Rust API reports must appear, with the same kind, in
    // slang's own output — i.e. the bindings neither invent nor mis-classify.
    let extra: Vec<_> = rust.difference(&slang).collect();
    assert!(
        extra.is_empty(),
        "Rust reported symbols absent/mis-kinded in slang --ast-json:\n{extra:#?}\n\
         (slang set: {slang:#?})"
    );
    assert!(!rust.is_empty(), "expected some symbols");
}

#[test]
fn hierarchy_and_kinds_match_slang() {
    check(
        "module leaf #(parameter int N = 4) (input logic clk, input logic [N-1:0] d, output logic [N-1:0] q);\n\
           logic [N-1:0] r;\n\
           always_ff @(posedge clk) r <= d;\n\
           assign q = r;\n\
         endmodule\n\
         module top(input logic clk);\n\
           logic [3:0] a, b;\n\
           leaf #(.N(4)) u0 (.clk(clk), .d(a), .q(b));\n\
         endmodule\n",
    );
}

// --- pyslang constant-value parity (ast-json stringifies constants, so an
// independent binding is needed to cross-check the actual numbers) ---

fn pyslang_dir() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("PYSLANG_PATH") {
        let p = PathBuf::from(p);
        return p.is_dir().then_some(p);
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for rel in ["build-py/lib", "build/lib"] {
        let p = root.join(rel);
        if p.join("pyslang").exists()
            || p.read_dir().is_ok_and(|mut d| {
                d.any(|e| {
                    e.map(|e| e.file_name().to_string_lossy().starts_with("pyslang"))
                        .unwrap_or(false)
                })
            })
        {
            return Some(p);
        }
    }
    None
}

/// pyslang's value for each integer parameter: (name -> (int, width, signed, unknown)).
fn pyslang_constants(dir: &PathBuf, src: &str) -> serde_json::Map<String, serde_json::Value> {
    let file = temp_sv(src);
    std::fs::write(&file, src).unwrap();
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/pyslang_constants.py");
    let out = Command::new("python3")
        .env("PYTHONPATH", dir)
        .arg(&script)
        .arg(&file)
        .output()
        .expect("run pyslang harness");
    assert!(
        out.status.success(),
        "pyslang harness failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("pyslang harness produced valid JSON")
}

#[test]
fn constant_values_match_pyslang() {
    let Some(dir) = pyslang_dir() else {
        eprintln!("no pyslang (set PYSLANG_PATH or build build-py); skipping");
        return;
    };
    let src = "module m;\n\
                 localparam logic [7:0] A = 8'd200;\n\
                 localparam int B = -5;\n\
                 localparam logic [15:0] C = 16'hBEEF;\n\
                 localparam int unsigned D = 42;\n\
               endmodule\n";
    let expected = pyslang_constants(&dir, src);

    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source(src).unwrap();
    let design = comp.compile().unwrap();
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let mut checked = 0;
    for (name, exp) in &expected {
        let sym = body.find(name).expect("param exists");
        let cv = sym.initializer().unwrap().constant_value().unwrap();
        let iv = cv.as_integer().expect("integer constant");
        // width / signed / unknown must match slang's own value exactly.
        assert_eq!(
            iv.bit_width() as u64,
            exp["width"].as_u64().unwrap(),
            "{name} width"
        );
        assert_eq!(
            iv.is_signed(),
            exp["signed"].as_bool().unwrap(),
            "{name} signed"
        );
        assert_eq!(
            iv.has_unknown(),
            exp["unknown"].as_bool().unwrap(),
            "{name} unknown"
        );
        if let Some(want) = exp["i"].as_i64() {
            let got = iv.as_i64().or_else(|| iv.as_u64().map(|u| u as i64));
            assert_eq!(got, Some(want), "{name} value");
        }
        checked += 1;
    }
    assert!(checked >= 3, "expected several params, checked {checked}");
}

#[test]
fn types_and_params_match_slang() {
    check(
        "module m;\n\
           typedef logic [7:0] byte_t;\n\
           localparam int W = 8;\n\
           byte_t x;\n\
           wire [W-1:0] y;\n\
           function automatic int f(int a); return a + 1; endfunction\n\
         endmodule\n",
    );
}
