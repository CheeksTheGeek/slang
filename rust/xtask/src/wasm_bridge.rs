//! Generates the wasm marshalling layer: for every non-callback `slang_*` C
//! function declared in `sv-lang-sys`, a raw `impl Slang` method that marshals
//! its arguments and result across the wasm boundary (indirect struct args,
//! sret struct returns, guest allocation, error out-params).
//!
//! The ergonomic semantic API (`compile`, `top_instances`, ...) is hand-written
//! on top of these raw methods; this generator guarantees the *whole* C surface
//! is reachable in wasm and stays in lock-step with the header.

use std::fmt::Write;

/// Guest (wasm32) sizes of the by-value structs — NOT the LP64 native sizes.
fn struct_size(name: &str) -> Option<u32> {
    Some(match name {
        "slang_str" => 12,
        "slang_ast" | "slang_node" | "slang_token" => 16,
        "slang_loc" => 16,
        "slang_range" => 32,
        "slang_error" => 252,
        "slang_diag" => 32,
        _ => return None,
    })
}

#[derive(Clone)]
enum Arg {
    /// A 32-bit scalar/handle/pointer, passed directly.
    I32(String),
    /// A 64-bit scalar (u64/usize).
    I64(String),
    /// A by-value struct, marshalled indirectly (write to guest, pass pointer):
    /// argument name and its wasm32 size in bytes.
    Struct(String, u32),
    /// The trailing `*mut slang_error` out-param.
    Error,
}

#[derive(Clone)]
enum Ret {
    Void,
    I32,
    I64,
    /// A by-value struct return (sret hidden first pointer arg).
    Sret(String, u32),
}

/// A parsed, classifiable extern fn.
struct Fun {
    name: String,
    args: Vec<Arg>,
    ret: Ret,
}

/// Maps a Rust type string to an argument classification.
fn classify_arg(name: &str, ty: &str) -> Result<Arg, String> {
    let ty = ty.replace(' ', "");
    if ty == "*mutslang_error" {
        return Ok(Arg::Error);
    }
    if ty.starts_with("*mut") || ty.starts_with("*const") {
        // A pointer/handle (opaque owner, c_char, c_void, or an out-param). The
        // raw layer treats it as a guest pointer (u32); out-params are the
        // caller's responsibility.
        return Ok(Arg::I32(name.to_string()));
    }
    // A named by-value struct?
    if let Some(sz) = struct_size(&ty) {
        return Ok(Arg::Struct(name.to_string(), sz));
    }
    match ty.as_str() {
        "u64" | "usize" | "i64" => Ok(Arg::I64(name.to_string())),
        "u32" | "i32" | "u16" | "u8" | "bool" | "f64" => Ok(Arg::I32(name.to_string())),
        // enum aliases (slang_*_kind, slang_*_flag, ...) resolve to c_uint.
        t if t.starts_with("slang_") => Ok(Arg::I32(name.to_string())),
        other => Err(format!("unsupported arg type `{other}`")),
    }
}

fn classify_ret(ty: &str) -> Result<Ret, String> {
    let ty = ty.replace(' ', "");
    if ty.is_empty() {
        return Ok(Ret::Void);
    }
    if let Some(sz) = struct_size(&ty) {
        return Ok(Ret::Sret(ty.clone(), sz));
    }
    if ty.starts_with("*mut") || ty.starts_with("*const") {
        return Ok(Ret::I32);
    }
    match ty.as_str() {
        "u64" | "usize" | "i64" => Ok(Ret::I64),
        "u32" | "i32" | "u16" | "u8" | "bool" => Ok(Ret::I32),
        t if t.starts_with("slang_") => Ok(Ret::I32),
        other => Err(format!("unsupported return type `{other}`")),
    }
}

/// Parses the `unsafe extern "C" { ... }` block and returns the (generable,
/// skipped-with-reason) functions.
fn parse_functions(sys_src: &str) -> (Vec<Fun>, Vec<(String, String)>) {
    let file: syn::File = syn::parse_str(sys_src).expect("parse sv-lang-sys lib.rs");
    let mut ok = Vec::new();
    let mut skipped = Vec::new();

    for item in &file.items {
        let syn::Item::ForeignMod(fm) = item else {
            continue;
        };
        for fi in &fm.items {
            let syn::ForeignItem::Fn(f) = fi else {
                continue;
            };
            let name = f.sig.ident.to_string();
            if !name.starts_with("slang_") {
                continue;
            }
            match classify_fn(f) {
                Ok(fun) => ok.push(fun),
                Err(why) => skipped.push((name, why)),
            }
        }
    }
    (ok, skipped)
}

fn ty_string(ty: &syn::Type) -> String {
    use quote_to_string::ToStr;
    ty.to_str()
}

fn classify_fn(f: &syn::ForeignItemFn) -> Result<Fun, String> {
    let name = f.sig.ident.to_string();
    let mut args = Vec::new();
    let mut n = 0;
    for input in &f.sig.inputs {
        let syn::FnArg::Typed(pt) = input else {
            return Err("has a receiver".into());
        };
        // A function-pointer or slice arg -> not generable.
        let ts = ty_string(&pt.ty);
        if ts.contains("extern") || ts.contains("Option<unsafe") || ts.contains("fn(") {
            return Err("takes a callback".into());
        }
        // Callbacks are also carried by named type aliases / structs-of-fn-ptrs
        // (slang_node_visitor, slang_ast_visitor) and by pointers to a sink /
        // lattice vtable (slang_syntax_sink, slang_dfa_lattice). The uniform raw
        // layer cannot synthesize a guest function pointer for these, so they
        // would generate as non-functional stubs (callback lowered to a bare
        // u32). Skip them here; the working ones (e.g. the DFA lattice) are
        // provided by hand-written guest trampolines in lib.rs / dfa_shim.c.
        if ts.contains("visitor")
            || ts.contains("slang_syntax_sink")
            || ts.contains("slang_dfa_lattice")
        {
            return Err("takes a callback (via type alias / vtable)".into());
        }
        let arg_name = match &*pt.pat {
            syn::Pat::Ident(pi) => pi.ident.to_string(),
            _ => {
                let s = format!("a{n}");
                n += 1;
                s
            }
        };
        args.push(classify_arg(&arg_name, &ts)?);
    }
    // At most one error out-param, and it must be last.
    let err_count = args.iter().filter(|a| matches!(a, Arg::Error)).count();
    if err_count > 1 {
        return Err("multiple error params".into());
    }
    if err_count == 1 && !matches!(args.last(), Some(Arg::Error)) {
        return Err("error param not last".into());
    }
    // Reject non-error `*mut <struct>` OUT-params (name ends with _out / out):
    // these need read-back the raw layer does not model uniformly.
    for a in &args {
        if let Arg::I32(nm) = a
            && (nm.ends_with("_out") || nm == "out" || nm == "node_out" || nm == "token_out")
        {
            return Err("has a struct out-param".into());
        }
    }
    let ret = match &f.sig.output {
        syn::ReturnType::Default => Ret::Void,
        syn::ReturnType::Type(_, ty) => classify_ret(&ty_string(ty))?,
    };
    Ok(Fun { name, args, ret })
}

/// Emits the generated `impl Slang` block.
pub fn gen_wasm_bridge(sys_src: &str) -> String {
    let (funs, skipped) = parse_functions(sys_src);
    let mut out = String::new();
    let _ = writeln!(
        out,
        "//! Generated raw wasm marshalling for the slang C API. See xtask/src/wasm_bridge.rs.\n\
         //!\n\
         //! One `raw_*` method per non-callback C function; the ergonomic API in lib.rs\n\
         //! is written on top. {} functions generated, {} skipped (callbacks / struct out-params).\n",
        funs.len(),
        skipped.len()
    );
    let _ = writeln!(out, "#![allow(clippy::too_many_arguments, dead_code)]\n");
    let _ = writeln!(out, "use wasmtime::Val;\n");
    let _ = writeln!(out, "use crate::{{Error, Slang}};\n");
    let _ = writeln!(out, "impl Slang {{");

    for f in &funs {
        emit_fn(&mut out, f);
    }
    let _ = writeln!(out, "}}");

    // Record what was skipped, so the surface is auditable.
    let _ = writeln!(out, "\n/* skipped (hand-written or N/A):");
    for (n, why) in &skipped {
        let _ = writeln!(out, " *   {n}: {why}");
    }
    let _ = writeln!(out, " */");
    out
}

fn rust_ret(ret: &Ret) -> &'static str {
    match ret {
        Ret::Void => "()",
        Ret::I32 => "u32",
        Ret::I64 => "u64",
        Ret::Sret(n, _) if n == "slang_str" => "String",
        Ret::Sret(_, _) => "Vec<u32>", // the struct's 32-bit words, LSB-first
    }
}

fn emit_fn(out: &mut String, f: &Fun) {
    // Method signature.
    let mut params = String::from("&mut self");
    for a in &f.args {
        match a {
            Arg::I32(n) => {
                let _ = write!(params, ", {n}: u32");
            }
            Arg::I64(n) => {
                let _ = write!(params, ", {n}: u64");
            }
            Arg::Struct(n, _) => {
                let _ = write!(params, ", {n}: &[u8]");
            }
            Arg::Error => {} // handled internally
        }
    }
    let ret = rust_ret(&f.ret);
    let _ = writeln!(
        out,
        "    /// Raw marshalling for `{}`.\n    pub fn raw_{}({}) -> Result<{}, Error> {{",
        f.name, f.name, params, ret
    );

    // Body: allocate sret + error, marshal struct args, build the arg vec, call, read back.
    let has_err = f.args.iter().any(|a| matches!(a, Arg::Error));
    let mut argexpr: Vec<String> = Vec::new();

    if let Ret::Sret(_, sz) = &f.ret {
        let _ = writeln!(out, "        let __sret = self.malloc({sz})?;");
        argexpr.push("Val::I32(__sret as i32)".into());
    }
    // struct args -> guest buffers.
    for a in &f.args {
        if let Arg::Struct(n, sz) = a {
            let _ = writeln!(out, "        let __p_{n} = self.malloc({sz})?;");
            let _ = writeln!(out, "        self.write(__p_{n}, &{n}[..{sz}])?;");
        }
    }
    if has_err {
        let _ = writeln!(out, "        let __err = self.malloc(252)?;");
        let _ = writeln!(out, "        self.zero(__err, 252)?;");
    }
    for a in &f.args {
        match a {
            Arg::I32(n) => argexpr.push(format!("Val::I32({n} as i32)")),
            Arg::I64(n) => argexpr.push(format!("Val::I64({n} as i64)")),
            Arg::Struct(n, _) => argexpr.push(format!("Val::I32(__p_{n} as i32)")),
            Arg::Error => argexpr.push("Val::I32(__err as i32)".into()),
        }
    }
    let _ = writeln!(
        out,
        "        let __r = self.call(\"{}\", &[{}]);",
        f.name,
        argexpr.join(", ")
    );
    // free struct arg buffers.
    for a in &f.args {
        if let Arg::Struct(n, _) = a {
            let _ = writeln!(out, "        self.free(__p_{n});");
        }
    }
    // error check.
    if has_err {
        let _ = writeln!(out, "        let __chk = self.check_err(__err);");
        let _ = writeln!(out, "        self.free(__err);");
    }
    // read the result.
    match &f.ret {
        Ret::Void => {
            if has_err {
                let _ = writeln!(out, "        __r?; __chk?; Ok(())");
            } else {
                let _ = writeln!(out, "        __r?; Ok(())");
            }
        }
        Ret::I32 => {
            let tail = if has_err { "__chk?;" } else { "" };
            let _ = writeln!(
                out,
                "        let __v = __r?; {tail} Ok(match __v.first() {{ Some(Val::I32(n)) => *n as u32, _ => 0 }})"
            );
        }
        Ret::I64 => {
            let tail = if has_err { "__chk?;" } else { "" };
            let _ = writeln!(
                out,
                "        let __v = __r?; {tail} Ok(match __v.first() {{ Some(Val::I64(n)) => *n as u64, Some(Val::I32(n)) => *n as u64, _ => 0 }})"
            );
        }
        Ret::Sret(n, _) if n == "slang_str" => {
            let tail = if has_err { "__chk?;" } else { "" };
            let _ = writeln!(
                out,
                "        __r?; {tail} let __s = self.take_slang_str(__sret)?; self.free(__sret); Ok(__s)"
            );
        }
        Ret::Sret(_, sz) => {
            let words = sz / 4;
            let tail = if has_err { "__chk?;" } else { "" };
            let _ = writeln!(
                out,
                "        __r?; {tail} let mut __w = Vec::with_capacity({words}); for __i in 0..{words} {{ __w.push(self.read_u32(__sret + __i * 4)?); }} self.free(__sret); Ok(__w)"
            );
        }
    }
    let _ = writeln!(out, "    }}");
}

/// Tiny helper so we can stringify a syn type without pulling in `quote`.
mod quote_to_string {
    pub trait ToStr {
        fn to_str(&self) -> String;
    }
    impl ToStr for syn::Type {
        fn to_str(&self) -> String {
            let mut s = String::new();
            stringify_ty(self, &mut s);
            s
        }
    }
    fn stringify_ty(ty: &syn::Type, out: &mut String) {
        use std::fmt::Write as _;
        match ty {
            syn::Type::Path(p) => {
                if let Some(seg) = p.path.segments.last() {
                    let _ = write!(out, "{}", seg.ident);
                }
            }
            syn::Type::Ptr(p) => {
                out.push('*');
                out.push_str(if p.mutability.is_some() {
                    "mut "
                } else {
                    "const "
                });
                stringify_ty(&p.elem, out);
            }
            syn::Type::BareFn(_) => out.push_str("fn("),
            syn::Type::Reference(r) => stringify_ty(&r.elem, out),
            _ => out.push('?'),
        }
    }
}
