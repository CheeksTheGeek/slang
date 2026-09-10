//! A runnable robustness check: feed the parser truncations and byte-mutations
//! of real SystemVerilog and assert it never panics, produces a tree, and
//! round-trips that tree's text. This is a deterministic stand-in for the
//! `cargo fuzz` target in `rust/fuzz/` (which needs nightly).

use sv_lang::{AnalysisFlags, Compilation, Design, SemNode, Session, Statement, Symbol};

const SEEDS: &[&str] = &[
    "module m #(parameter int W = 8) (input logic [W-1:0] a, output logic o); assign o = |a; endmodule\n",
    "package p; typedef struct packed { logic [3:0] x; logic y; } t; localparam t Z = '{x:0, y:1}; endpackage\n",
    "interface bus_if; logic req, gnt; modport m(input req, output gnt); endinterface\n",
    "class C #(type T = int); T value; function new(T v); value = v; endfunction endclass\n",
    "module top; initial begin for (int i = 0; i < 10; i++) $display(\"%0d\", i); end endmodule\n",
];

fn check(session: &Session, text: &str) {
    // Must not panic. A tree is produced (parse errors are diagnostics, not
    // failures); if produced, its text round-trips.
    if let Ok(tree) = session.parse(text) {
        let out = tree.text();
        assert_eq!(out, text, "round-trip differs");
        // Exercise the cursor API on the result.
        let _ = tree.root().descendants().count();
        let _ = tree.diagnostics().rendered().len();
    }
}

#[test]
fn truncations_never_panic() {
    let session = Session::new();
    for seed in SEEDS {
        // Every prefix and every suffix at a char boundary.
        for i in 0..=seed.len() {
            if seed.is_char_boundary(i) {
                check(&session, &seed[..i]);
                check(&session, &seed[i..]);
            }
        }
    }
}

#[test]
fn byte_mutations_never_panic() {
    let session = Session::new();
    // A cheap deterministic PRNG (no rand dependency).
    let mut state: u64 = 0x9E3779B97F4A7C15;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };

    for seed in SEEDS {
        let base: Vec<u8> = seed.bytes().collect();
        for _ in 0..500 {
            let mut bytes = base.clone();
            // Apply a few random single-byte edits.
            let edits = (next() % 4) + 1;
            for _ in 0..edits {
                if bytes.is_empty() {
                    break;
                }
                let idx = (next() as usize) % bytes.len();
                match next() % 3 {
                    0 => bytes[idx] = (next() % 128) as u8, // substitute ASCII
                    1 => {
                        bytes.remove(idx); // delete
                    }
                    _ => bytes.insert(idx, (next() % 128) as u8), // insert
                }
            }
            if let Ok(text) = std::str::from_utf8(&bytes) {
                check(&session, text);
            }
        }
    }
}

/// Drives every value-returning accessor over one symbol and, recursively, its
/// members. If any underlying C accessor is unguarded and slang trips an
/// assertion on a malformed/partial tree, the process aborts here — so this is
/// the guard-completeness check for the whole AST accessor surface.
fn walk_symbol(sym: Symbol<'_>, depth: u32) {
    if depth > 40 {
        return;
    }
    let _ = sym.kind();
    let _ = sym.name();
    let _ = sym.parent();
    let _ = sym.next_sibling();
    let _ = sym.is_scope();
    let _ = sym.is_type();
    let _ = sym.is_value();
    let _ = sym.hierarchical_path();
    let _ = sym.definition_kind();
    let _ = sym.instance_definition();
    let _ = sym.parameter_value();
    if let Some(t) = sym.as_type().or_else(|| sym.value_type()) {
        let _ = t.canonical();
        let _ = t.to_sv_string();
        let _ = t.bit_width();
        let _ = t.is_integral();
        let _ = t.is_signed();
        let _ = t.is_four_state();
        let _ = t.is_unpacked_array();
        let _ = t.is_class();
        // Exercise the two-handle path (guarded, and identity-checked) against
        // itself — same compilation, so the identity assert holds.
        let _ = t.is_equivalent(&t);
        let _ = t.is_matching(&t);
        let _ = t.is_assignment_compatible(&t);
    }
    if let Some(e) = sym.initializer() {
        let _ = e.kind();
        let _ = e.expr_type();
        let _ = e.is_bad();
        let _ = e.referenced_symbol();
        let _ = e.constant();
    }
    for p in sym.parameters() {
        let _ = p.parameter_value();
    }
    // The behavioral tree (procedural block / subroutine bodies) and its
    // generic child enumerator, over malformed input.
    if let Some(root) = sym.body() {
        walk_stmt(root, 0);
    }
    if let Some(body) = sym.instance_body() {
        for m in body.members() {
            walk_symbol(m, depth + 1);
        }
    }
    for m in sym.members() {
        walk_symbol(m, depth + 1);
    }
}

fn walk_stmt(s: Statement<'_>, depth: u32) {
    if depth > 60 {
        return;
    }
    let _ = s.kind();
    for c in s.children() {
        walk_sem(c, depth + 1);
    }
}

fn walk_sem(n: SemNode<'_>, depth: u32) {
    if depth > 60 {
        return;
    }
    let _ = n.kind_name();
    let _ = n.domain();
    if let Some(e) = n.as_expression() {
        let _ = e.kind();
        let _ = e.binary_op();
        let _ = e.unary_op();
        let _ = e.is_nonblocking();
        let _ = e.call_subroutine();
        let _ = e.member_symbol();
        let _ = e.operands();
        let _ = e.expr_type();
    }
    for c in n.children() {
        walk_sem(c, depth + 1);
    }
}

fn walk_design(design: &Design) {
    let _ = design.diagnostics().rendered();
    let _ = design.root().kind();
    for top in design.top_instances() {
        walk_symbol(top, 0);
    }
    for d in design.definitions() {
        walk_symbol(d, 0);
    }
    for p in design.packages() {
        walk_symbol(p, 0);
    }
    if let Ok(analysis) = design.analyze(AnalysisFlags::CHECK_UNUSED, 1) {
        let _ = analysis.diagnostics().rendered();
        for top in design.top_instances() {
            if let Some(body) = top.instance_body() {
                for proc in analysis.procedures(body) {
                    let _ = proc.symbol().kind();
                    let _ = proc.has_inferred_clock();
                }
                for m in body.members() {
                    for drv in analysis.drivers(m) {
                        let _ = drv.kind();
                        let _ = drv.containing_symbol().kind();
                    }
                }
            }
        }
    }
}

/// The guard-completeness test: drive the full semantic + analysis accessor
/// surface over many malformed / truncated / degenerate inputs. Every C
/// accessor reached here must be exception-guarded, or slang's assertions on a
/// partial tree would unwind across the FFI boundary and abort the process.
#[test]
fn semantic_accessors_never_abort_on_malformed() {
    let session = Session::new();

    let mut inputs: Vec<String> = SEEDS
        .iter()
        .flat_map(|seed| {
            (0..=seed.len())
                .filter(move |i| seed.is_char_boundary(*i))
                .map(move |i| seed[..i].to_string())
        })
        .collect();
    inputs.extend(
        [
            "module",
            "typedef",
            "module m; assign x = ; endmodule",
            "module m; logic [ : ] a; endmodule",
            "module m; always_ff @(posedge ) if () x <= ; endmodule",
            "class C extends ; endclass",
            "package p; localparam Z = 1/0; endpackage",
            "module m; foo #(.A(1)) i(); endmodule",
            "module m; enum { A, B = , C } e; endmodule",
            "interface i; modport mp(input , output ); endinterface",
        ]
        .into_iter()
        .map(String::from),
    );

    for text in &inputs {
        if let Ok(tree) = session.parse(text) {
            for node in tree.root().descendants() {
                let _ = node.kind();
            }
            let _ = tree.text();
            let _ = tree.mirror();
            let _ = tree.diagnostics().rendered();
        }

        let Ok(mut comp) = Compilation::new(&session) else {
            continue;
        };
        if comp.add_source(text).is_err() {
            continue;
        }
        let Ok(design) = comp.compile() else {
            continue;
        };
        walk_design(&design);
    }
}
