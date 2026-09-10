//! End-to-end smoke test: the vendored/in-tree build links, the ABI the
//! declarations assume matches the library, and the kind tables agree with
//! the pure-Rust `sv-lang-kinds` crate generated from the same models.

use std::ffi::{CStr, c_void};
use std::ptr;

use sv_lang_sys::*;

unsafe fn text(s: slang_str) -> String {
    let out = String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(s.data.cast(), s.len) })
        .into_owned();
    unsafe { slang_str_free(s) };
    out
}

#[test]
fn version_and_models_agree_with_kinds_crate() {
    unsafe {
        assert_eq!(slang_c_version(), SLANG_C_VERSION);
        assert!(
            slang_build_flags() & SLANG_BUILD_ASSERTIONS != 0,
            "must be built with assertions"
        );
        let version = CStr::from_ptr(slang_version_string()).to_str().unwrap();
        assert!(
            version.starts_with(sv_lang_kinds::SLANG_VERSION),
            "{version}"
        );

        let syntax_hash = CStr::from_ptr(slang_syntax_model_hash()).to_str().unwrap();
        assert_eq!(syntax_hash, sv_lang_kinds::SYNTAX_MODEL_HASH);
        let diag_hash = CStr::from_ptr(slang_diagnostics_model_hash())
            .to_str()
            .unwrap();
        assert_eq!(diag_hash, sv_lang_kinds::DIAGNOSTICS_MODEL_HASH);

        assert_eq!(
            slang_syntax_kind_count(),
            u32::from(sv_lang_kinds::SyntaxKind::COUNT)
        );
        assert_eq!(
            slang_syntax_struct_count(),
            u32::from(sv_lang_kinds::SyntaxStruct::COUNT)
        );
        assert_eq!(
            slang_token_kind_count(),
            u32::from(sv_lang_kinds::TokenKind::COUNT)
        );
        assert_eq!(
            slang_trivia_kind_count(),
            u32::from(sv_lang_kinds::TriviaKind::COUNT)
        );
        assert_eq!(
            slang_ast_kind_count(SLANG_AST_SYMBOL),
            sv_lang_kinds::SymbolKind::ALL.len() as u32
        );
        assert_eq!(
            slang_ast_kind_count(SLANG_AST_EXPRESSION),
            sv_lang_kinds::ExpressionKind::ALL.len() as u32
        );

        for kind in sv_lang_kinds::SyntaxKind::ALL {
            assert_eq!(
                text(slang_syntax_kind_name(u32::from(kind.as_raw()))),
                kind.name()
            );
            let s = slang_syntax_kind_struct(u32::from(kind.as_raw()));
            match kind.syntax_struct() {
                Some(st) => assert_eq!(text(slang_syntax_struct_name(s)), st.name()),
                None => assert_eq!(s, u32::MAX),
            }
        }
        for kind in sv_lang_kinds::TokenKind::ALL {
            assert_eq!(
                text(slang_token_kind_name(u32::from(kind.as_raw()))),
                kind.name()
            );
        }
        for kind in sv_lang_kinds::SymbolKind::ALL {
            assert_eq!(
                text(slang_ast_kind_name(
                    SLANG_AST_SYMBOL,
                    u32::from(kind.as_raw())
                )),
                kind.name()
            );
        }
    }
}

#[test]
fn parse_and_elaborate() {
    unsafe {
        let mut err = slang_error::INIT;
        let sm = slang_source_manager_create(&mut err);
        assert_eq!(err.status, SLANG_OK);

        let options = slang_options_create(&mut err);
        slang_options_set_compilation_flags(options, SLANG_COMP_DISABLE_INSTANCE_CACHING);

        let src = "module top; localparam int W = 8; logic [W-1:0] q; endmodule\n";
        let tree = slang_syntax_tree_from_text(
            sm,
            src.as_ptr().cast(),
            src.len(),
            c"top.sv".as_ptr(),
            6,
            ptr::null(),
            0,
            options,
            &mut err,
        );
        assert_eq!(
            err.status,
            SLANG_OK,
            "{}",
            CStr::from_ptr(err.message.as_ptr()).to_string_lossy()
        );
        assert!(!tree.is_null());

        let root = slang_syntax_tree_root(tree);
        assert_eq!(text(slang_syntax_kind_name(root.kind)), "CompilationUnit");
        assert_eq!(slang_node_child_count(root), 2);

        unsafe extern "C" fn count(_: slang_node, user: *mut c_void) -> slang_visit {
            unsafe { *user.cast::<u32>() += 1 };
            SLANG_VISIT_CONTINUE
        }
        let mut nodes = 0u32;
        slang_node_visit(root, Some(count), (&mut nodes as *mut u32).cast(), &mut err);
        assert_eq!(err.status, SLANG_OK);
        assert!(nodes > 10);

        let comp = slang_compilation_create(options, &mut err);
        slang_compilation_add_tree(comp, tree, &mut err);
        let mut report = slang_freeze_report::default();
        slang_compilation_freeze(comp, SLANG_FREEZE_ALL, &mut report, &mut err);
        assert_eq!(
            err.status,
            SLANG_OK,
            "{}",
            CStr::from_ptr(err.message.as_ptr()).to_string_lossy()
        );
        assert!(slang_compilation_is_sealed(comp));
        assert_eq!(report.fold_failures, 0);
        assert!(report.expressions_folded > 0);

        let top = slang_compilation_top_instance(comp, 0);
        assert_eq!(text(slang_symbol_name(top)), "top");
        let q = slang_scope_lookup(top, c"q".as_ptr(), 1, &mut err);
        if slang_ast_is_null(q) {
            let mut names = Vec::new();
            let mut m = slang_scope_first_member(slang_instance_body(top), &mut err);
            while !slang_ast_is_null(m) {
                names.push(text(slang_symbol_name(m)));
                m = slang_symbol_next_sibling(m);
            }
            panic!(
                "lookup of q failed: status {} ({}); body members: {names:?}",
                err.status,
                CStr::from_ptr(err.message.as_ptr()).to_string_lossy()
            );
        }
        let ty = slang_value_type(q, &mut err);
        assert_eq!(text(slang_type_to_string(ty, &mut err)), "logic[7:0]");
        assert_eq!(slang_type_bit_width(ty), 8);

        let diags = slang_compilation_diagnostics(comp, &mut err);
        assert_eq!(slang_diagnostics_count(diags), 0);
        slang_diagnostics_destroy(diags);

        slang_compilation_destroy(comp);
        slang_syntax_tree_release(tree);
        slang_options_destroy(options);
        slang_source_manager_destroy(sm);
    }
}
