//! Integer constants wider than 1024 bits: `SVInt::bits()` used to be empty
//! above that width (and `to_string()` abbreviates), so a consumer could not
//! read a wide parameter at all — e.g. CVA6's 17270-bit config struct.
//! Reported by the sv-sim session.

use sv_lang::{Bit, Compilation, Session};

#[test]
fn every_bit_of_a_wide_struct_parameter() {
    let s = Session::new();
    let mut comp = Compilation::new(&s).unwrap();
    comp.add_source(
        "package p;
  typedef struct packed { logic [1099:0] a; } s_t;
  function automatic int f(s_t x); return x.a[5]; endfunction
  localparam s_t C = '{a: {550{2'b10}}};
  localparam logic [17269:0] BIG = {8635{2'b1x}};
endpackage
module m; int v; initial v = p::f(p::C); endmodule
",
    )
    .unwrap();
    let d = comp.compile().unwrap();
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let pkg = d.packages().find(|p| p.name() == "p").unwrap();

    let c = pkg.find("C").unwrap().constant_value().unwrap();
    let c = c.as_integer().unwrap();
    assert_eq!(c.bit_width(), 1100);
    assert_eq!(c.bits().len(), 1100);
    for (i, b) in c.bits().iter().enumerate() {
        let want = if i % 2 == 0 { Bit::Zero } else { Bit::One };
        assert_eq!(*b, want, "bit {i}");
    }

    let big = pkg.find("BIG").unwrap().constant_value().unwrap();
    let big = big.as_integer().unwrap();
    assert_eq!(big.bits().len(), 17270);
    assert_eq!(big.bit(0), Some(Bit::X));
    assert_eq!(big.bit(17269), Some(Bit::One));
    assert_eq!(
        big.count_xs() as usize,
        big.bits().iter().filter(|b| **b == Bit::X).count()
    );
}
