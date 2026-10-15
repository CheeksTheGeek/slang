//! 64-bit return types for registered non-constant system functions: only the
//! 32-bit `int`/`integer` were available, so a `$simtime`-style tick counter at
//! or above 2^31 read as x (ivtest stime). Reported by the sv-sim session.

use sv_lang::kinds::SymbolKind;
use sv_lang::{BuiltinType, Compilation, Session, SystemFunctionType, Walk};

fn call_type(comp_setup: impl FnOnce(&mut Compilation), src: &str) -> (String, u64) {
    let s = Session::new();
    let mut comp = Compilation::new(&s).unwrap();
    comp_setup(&mut comp);
    comp.add_source(src).unwrap();
    let d = comp.compile().unwrap();
    assert!(!d.diagnostics().has_errors(), "{}", d.diagnostics());
    let body = d.top_instances().next().unwrap().instance_body().unwrap();
    let mut out = None;
    body.visit(|s| {
        if s.kind() == SymbolKind::Variable
            && s.name() == "probe"
            && let Some(init) = s.initializer()
        {
            let t = init.expr_type().unwrap();
            out = Some((t.to_sv_string(), t.bit_width()));
        }
        Walk::Continue
    });
    out.expect("probe variable")
}

#[test]
fn time_and_longint_returns_are_64_bit() {
    let (ty, w) = call_type(
        |c| {
            c.add_nonconstant_system_function_typed("$simtime", SystemFunctionType::Time, &[])
                .unwrap()
        },
        "module m; time probe = $simtime; endmodule\n",
    );
    assert_eq!((ty.as_str(), w), ("time", 64));

    let (ty, w) = call_type(
        |c| {
            c.add_nonconstant_system_function_typed(
                "$ticks",
                SystemFunctionType::LongInt,
                &[SystemFunctionType::ShortInt, SystemFunctionType::RealTime],
            )
            .unwrap()
        },
        "module m; longint probe = $ticks(16'sd1, 1.5); endmodule\n",
    );
    assert_eq!((ty.as_str(), w), ("longint", 64));
}

#[test]
fn builtin_type_still_works_and_converts() {
    let (ty, w) = call_type(
        |c| {
            c.add_nonconstant_system_function("$fputc", BuiltinType::Int, &[BuiltinType::Int])
                .unwrap()
        },
        "module m; int probe = $fputc(1); endmodule\n",
    );
    assert_eq!((ty.as_str(), w), ("int", 32));
    assert_eq!(
        SystemFunctionType::from(BuiltinType::Integer),
        SystemFunctionType::Integer
    );
}

/// Sized packed returns (VPI vpiSizedFunc / vpiSizedSignedFunc, LRM 38.37.1):
/// the call's self-determined width and signedness follow the registration.
#[test]
fn sized_packed_returns() {
    let (ty, w) = call_type(
        |c| {
            c.add_nonconstant_system_function_sized(
                "$sized48",
                48,
                false,
                &[SystemFunctionType::Int],
            )
            .unwrap()
        },
        "module m; logic [47:0] probe = $sized48(3); endmodule\n",
    );
    assert_eq!((ty.as_str(), w), ("logic[47:0]", 48));

    let (ty, w) = call_type(
        |c| {
            c.add_nonconstant_system_function_sized("$sized128s", 128, true, &[])
                .unwrap()
        },
        "module m; logic signed [127:0] probe = $sized128s(); endmodule\n",
    );
    assert_eq!((ty.as_str(), w), ("logic signed[127:0]", 128));
}

#[test]
fn packed_width_out_of_range_is_rejected() {
    let s = Session::new();
    let mut comp = Compilation::new(&s).unwrap();
    for width in [0, 1 << 24] {
        let r = comp.add_nonconstant_system_function_sized("$bad", width, false, &[]);
        assert!(
            matches!(r, Err(sv_lang::Error::InvalidArgument(_))),
            "width {width}: {r:?}"
        );
    }
}
