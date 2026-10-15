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
