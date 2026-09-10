//! Structured constant values (integer width/signedness/x-z bits, real, string,
//! arrays), not just printed strings.

use sv_lang::{Bit, Compilation, ConstantValue, Session};

fn compile(src: &str) -> sv_lang::Design {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source(src).unwrap();
    comp.compile().unwrap()
}

fn init_const(design: &sv_lang::Design, name: &str) -> ConstantValue {
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    body.find(name)
        .unwrap()
        .initializer()
        .unwrap()
        .constant_value()
        .unwrap()
}

#[test]
fn integer_constants_are_structured() {
    let design = compile(
        "module m;\n\
           localparam logic [7:0] X = 8'd12;\n\
           localparam int Y = -5;\n\
         endmodule\n",
    );

    let x = init_const(&design, "X");
    assert_eq!(x.as_i64(), Some(12));
    let xi = x.as_integer().unwrap();
    assert_eq!(xi.bit_width(), 8);
    assert!(!xi.has_unknown());
    // 12 = 0b0000_1100 -> bit0=0, bit2=1, bit3=1.
    assert_eq!(xi.bit(0), Some(Bit::Zero));
    assert_eq!(xi.bit(2), Some(Bit::One));
    assert_eq!(xi.bit(3), Some(Bit::One));

    let y = init_const(&design, "Y");
    assert_eq!(y.as_i64(), Some(-5));
    assert!(y.as_integer().unwrap().is_signed());
}

#[test]
fn four_state_bits_and_string() {
    let design = compile(
        "module m;\n\
           localparam logic [3:0] Z = 4'b1x0z;\n\
           localparam string S = \"hi\";\n\
         endmodule\n",
    );

    let z = init_const(&design, "Z");
    let zi = z.as_integer().unwrap();
    assert!(zi.has_unknown());
    assert_eq!(z.as_i64(), None); // unknown bits don't fit an i64
    // 4'b1x0z, MSB-first, so LSB-first bits are z, 0, x, 1.
    assert_eq!(zi.bit(0), Some(Bit::Z));
    assert_eq!(zi.bit(1), Some(Bit::Zero));
    assert_eq!(zi.bit(2), Some(Bit::X));
    assert_eq!(zi.bit(3), Some(Bit::One));

    let s = init_const(&design, "S");
    assert_eq!(s.as_str(), Some("hi"));
}

#[test]
fn array_constant_recurses() {
    let design = compile("module m; localparam int A [3] = '{1, 2, 3}; endmodule\n");
    let a = init_const(&design, "A");
    match a {
        ConstantValue::Array(els) => {
            assert_eq!(els.len(), 3);
            assert_eq!(els[0].as_i64(), Some(1));
            assert_eq!(els[2].as_i64(), Some(3));
        }
        other => panic!("expected array, got {other:?}"),
    }
}

#[test]
fn eval_constant_via_session() {
    let mut design = compile("module m; localparam int W = 8; endmodule\n");
    let eval = design.eval_session();
    let x = eval
        .design()
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap()
        .find("W")
        .unwrap();
    let v = eval.eval_constant(x.initializer().unwrap()).unwrap();
    assert_eq!(v.as_i64(), Some(8));
}
