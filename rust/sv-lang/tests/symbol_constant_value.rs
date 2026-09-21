//! `Symbol::constant_value()` — the elaborated constant of a Parameter /
//! EnumValue / Specparam symbol as a structured `ConstantValue`, reflecting
//! defparam and instance overrides and preserving full width (unlike the
//! text-based `parameter_value()`, which slang abbreviates above 128 bits).
//! Requested by the svling port to recover wide, overridden parameters.

use sv_lang::{Compilation, Session};

fn compile(src: &str) -> sv_lang::Design {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source(src).unwrap();
    comp.compile().unwrap()
}

#[test]
fn parameter_value_reflects_instance_override() {
    let d = compile(
        "module inner #(parameter int W = 1); endmodule\n\
         module top; inner #(.W(42)) u(); endmodule\n",
    );
    let top = d.top_instances().next().unwrap().instance_body().unwrap();
    let u = top.members().find(|s| s.name() == "u").unwrap();
    let w = u.instance_body().unwrap().find("W").unwrap();
    // The override (42), not the declared default (1).
    assert_eq!(w.constant_value().unwrap().as_i64(), Some(42));
}

#[test]
fn parameter_value_reflects_defparam() {
    let d = compile(
        "module inner; parameter W = 1; endmodule\n\
         module top;\n  inner u();\n  defparam u.W = 7;\nendmodule\n",
    );
    let top = d.top_instances().next().unwrap().instance_body().unwrap();
    let u = top.members().find(|s| s.name() == "u").unwrap();
    let w = u.instance_body().unwrap().find("W").unwrap();
    assert_eq!(w.constant_value().unwrap().as_i64(), Some(7));
}

#[test]
fn wide_parameter_preserves_full_width() {
    // A 200-bit localparam: parameter_value() prints an abbreviated string
    // (slang elides the middle above 128 bits), but constant_value() keeps the
    // full-width SVInt.
    let d = compile("module m; localparam logic [199:0] X = 200'hFF; endmodule\n");
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let x = body.find("X").unwrap();

    let cv = x.constant_value().expect("X has a folded value");
    let svint = cv.as_integer().expect("integer constant");
    assert_eq!(svint.bit_width(), 200);
    assert_eq!(svint.as_u64(), Some(0xFF));

    // The printed form is abbreviated for wide values; the structured value is
    // the reliable path.
    let printed = x.parameter_value().unwrap();
    assert!(
        printed.contains("...") || printed.len() < 60,
        "printed form: {printed:?}"
    );
}

#[test]
fn enum_member_value() {
    let d = compile("module m;\n  typedef enum int { A = 3, B = 9 } e_t;\n  e_t sig;\nendmodule\n");
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    // Enum members are injected into the enclosing scope as EnumValue symbols.
    let a = body.find("A").expect("enum member A");
    let b = body.find("B").expect("enum member B");
    assert_eq!(a.kind(), sv_lang::kinds::SymbolKind::EnumValue);
    assert_eq!(a.constant_value().and_then(|c| c.as_i64()), Some(3));
    assert_eq!(b.constant_value().and_then(|c| c.as_i64()), Some(9));
}

#[test]
fn wide_defparam_preserves_xz_bits() {
    // The svling failure shape: a defparam override with x/z bits into a wide
    // parameter. The abbreviated text is unparseable; only the structured
    // value carries the x/z fidelity.
    use sv_lang::Bit;
    let d = compile(
        "module inner; parameter [151:0] P = 0; endmodule\n\
         module top;\n  inner u();\n  defparam u.P = {8'hxz, 136'd0, 8'bzzzz_xxxx};\nendmodule\n",
    );
    let top = d.top_instances().next().unwrap().instance_body().unwrap();
    let u = top.members().find(|s| s.name() == "u").unwrap();
    let p = u.instance_body().unwrap().find("P").unwrap();

    let cv = p.constant_value().expect("P has an overridden value");
    let v = cv.as_integer().expect("integer");
    assert_eq!(v.bit_width(), 152);
    assert!(v.has_unknown(), "override carries x/z bits");
    // Low byte 8'bzzzz_xxxx: bits [3:0] = x, [7:4] = z.
    assert_eq!(v.bit(0), Some(Bit::X));
    assert_eq!(v.bit(4), Some(Bit::Z));
    // The 0-filled middle.
    assert_eq!(v.bit(8), Some(Bit::Zero));
    // Top byte 8'hxz: bits [147:144] = z, [151:148] = x.
    assert_eq!(v.bit(144), Some(Bit::Z));
    assert_eq!(v.bit(148), Some(Bit::X));
}

#[test]
fn signed_wide_defparam_preserves_sign() {
    // Signed wide parameter overridden by a negative defparam value.
    let d = compile(
        "module inner; parameter signed [199:0] P = 1; endmodule\n\
         module top;\n  inner u();\n  defparam u.P = -200'sd123;\nendmodule\n",
    );
    let top = d.top_instances().next().unwrap().instance_body().unwrap();
    let u = top.members().find(|s| s.name() == "u").unwrap();
    let p = u.instance_body().unwrap().find("P").unwrap();

    let cv = p.constant_value().expect("P has an overridden value");
    let v = cv.as_integer().expect("integer");
    assert_eq!(v.bit_width(), 200);
    assert!(v.is_signed());
    assert_eq!(v.as_i64(), Some(-123));
}

#[test]
fn constant_bit_slice_reads_above_1024_bits() {
    // A parameter wider than SVInt's 1024-bit marshalling cap, overridden by
    // defparam. constant_value() reports the correct width but empty bits; the
    // slice accessor reads any window on slang's full-width value, above and
    // below the 1024-bit line.
    let d = compile(
        "module inner; parameter [2047:0] P = 0; endmodule\n\
         module top;\n  inner u();\n\
         \x20 defparam u.P = (2048'hCAFE << 1024) | 2048'hBEEF;\nendmodule\n",
    );
    let top = d.top_instances().next().unwrap().instance_body().unwrap();
    let u = top.members().find(|s| s.name() == "u").unwrap();
    let p = u.instance_body().unwrap().find("P").unwrap();

    // The full value is too wide to marshal: correct width, but no bits cross.
    let full = p.constant_value().unwrap();
    let fi = full.as_integer().unwrap();
    assert_eq!(fi.bit_width(), 2048);
    assert!(
        fi.bits().is_empty(),
        "wide value should not marshal its bits"
    );

    // Slices read exact windows regardless of width.
    let low = p.constant_bit_slice(15, 0).unwrap();
    assert_eq!(low.as_integer().unwrap().bit_width(), 16);
    assert_eq!(low.as_i64(), Some(0xBEEF));
    let high = p.constant_bit_slice(1039, 1024).unwrap();
    assert_eq!(high.as_i64(), Some(0xCAFE));
}

#[test]
fn non_value_symbol_is_none() {
    let d = compile("module m; logic a; endmodule\n");
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    // A net/variable has no parameter-style constant.
    let a = body.find("a").unwrap();
    assert!(a.constant_value().is_none());
    // The module instance body itself, too.
    let inst = d.top_instances().next().unwrap();
    assert!(inst.constant_value().is_none());
}
