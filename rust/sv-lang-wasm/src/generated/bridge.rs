//! Generated raw wasm marshalling for the slang C API. See xtask/src/wasm_bridge.rs.
//!
//! One `raw_*` method per non-callback C function; the ergonomic API in lib.rs
//! is written on top. 1015 functions generated, 44 skipped (callbacks / struct out-params).

#![allow(clippy::too_many_arguments, dead_code)]

use wasmtime::Val;

use crate::{Error, Slang};

impl Slang {
    /// Raw marshalling for `slang_status_name`.
    pub fn raw_slang_status_name(&mut self, status: u32) -> Result<u32, Error> {
        let __r = self.call("slang_status_name", &[Val::I32(status as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_c_version`.
    pub fn raw_slang_c_version(&mut self) -> Result<u32, Error> {
        let __r = self.call("slang_c_version", &[]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_version_string`.
    pub fn raw_slang_version_string(&mut self) -> Result<u32, Error> {
        let __r = self.call("slang_version_string", &[]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_syntax_model_hash`.
    pub fn raw_slang_syntax_model_hash(&mut self) -> Result<u32, Error> {
        let __r = self.call("slang_syntax_model_hash", &[]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_diagnostics_model_hash`.
    pub fn raw_slang_diagnostics_model_hash(&mut self) -> Result<u32, Error> {
        let __r = self.call("slang_diagnostics_model_hash", &[]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_build_flags`.
    pub fn raw_slang_build_flags(&mut self) -> Result<u32, Error> {
        let __r = self.call("slang_build_flags", &[]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_str_free`.
    pub fn raw_slang_str_free(&mut self, str: &[u8]) -> Result<(), Error> {
        let __p_str = self.malloc(12)?;
        self.write(__p_str, &str[..12])?;
        let __r = self.call("slang_str_free", &[Val::I32(__p_str as i32)]);
        self.free(__p_str);
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_syntax_kind_count`.
    pub fn raw_slang_syntax_kind_count(&mut self) -> Result<u32, Error> {
        let __r = self.call("slang_syntax_kind_count", &[]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_syntax_kind_name`.
    pub fn raw_slang_syntax_kind_name(&mut self, kind: u32) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __r = self.call(
            "slang_syntax_kind_name",
            &[Val::I32(__sret as i32), Val::I32(kind as i32)],
        );
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_syntax_kind_struct`.
    pub fn raw_slang_syntax_kind_struct(&mut self, kind: u32) -> Result<u32, Error> {
        let __r = self.call("slang_syntax_kind_struct", &[Val::I32(kind as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_syntax_struct_count`.
    pub fn raw_slang_syntax_struct_count(&mut self) -> Result<u32, Error> {
        let __r = self.call("slang_syntax_struct_count", &[]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_syntax_struct_name`.
    pub fn raw_slang_syntax_struct_name(&mut self, syntax_struct: u32) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __r = self.call(
            "slang_syntax_struct_name",
            &[Val::I32(__sret as i32), Val::I32(syntax_struct as i32)],
        );
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_syntax_struct_member_count`.
    pub fn raw_slang_syntax_struct_member_count(
        &mut self,
        syntax_struct: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_syntax_struct_member_count",
            &[Val::I32(syntax_struct as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_syntax_member_name`.
    pub fn raw_slang_syntax_member_name(
        &mut self,
        syntax_struct: u32,
        member: u32,
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __r = self.call(
            "slang_syntax_member_name",
            &[
                Val::I32(__sret as i32),
                Val::I32(syntax_struct as i32),
                Val::I32(member as i32),
            ],
        );
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_syntax_member_form`.
    pub fn raw_slang_syntax_member_form(
        &mut self,
        syntax_struct: u32,
        member: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_syntax_member_form",
            &[Val::I32(syntax_struct as i32), Val::I32(member as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_token_kind_count`.
    pub fn raw_slang_token_kind_count(&mut self) -> Result<u32, Error> {
        let __r = self.call("slang_token_kind_count", &[]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_token_kind_name`.
    pub fn raw_slang_token_kind_name(&mut self, kind: u32) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __r = self.call(
            "slang_token_kind_name",
            &[Val::I32(__sret as i32), Val::I32(kind as i32)],
        );
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_trivia_kind_count`.
    pub fn raw_slang_trivia_kind_count(&mut self) -> Result<u32, Error> {
        let __r = self.call("slang_trivia_kind_count", &[]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_trivia_kind_name`.
    pub fn raw_slang_trivia_kind_name(&mut self, kind: u32) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __r = self.call(
            "slang_trivia_kind_name",
            &[Val::I32(__sret as i32), Val::I32(kind as i32)],
        );
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_ast_kind_name`.
    pub fn raw_slang_ast_kind_name(&mut self, domain: u32, kind: u32) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __r = self.call(
            "slang_ast_kind_name",
            &[
                Val::I32(__sret as i32),
                Val::I32(domain as i32),
                Val::I32(kind as i32),
            ],
        );
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_ast_kind_count`.
    pub fn raw_slang_ast_kind_count(&mut self, domain: u32) -> Result<u32, Error> {
        let __r = self.call("slang_ast_kind_count", &[Val::I32(domain as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_source_manager_create`.
    pub fn raw_slang_source_manager_create(&mut self) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call("slang_source_manager_create", &[Val::I32(__err as i32)]);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_source_manager_destroy`.
    pub fn raw_slang_source_manager_destroy(&mut self, sm: u32) -> Result<(), Error> {
        let __r = self.call("slang_source_manager_destroy", &[Val::I32(sm as i32)]);
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_source_manager_add_include_dir`.
    pub fn raw_slang_source_manager_add_include_dir(
        &mut self,
        sm: u32,
        pattern: u32,
        pattern_len: u64,
        system: u32,
    ) -> Result<(), Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_source_manager_add_include_dir",
            &[
                Val::I32(sm as i32),
                Val::I32(pattern as i32),
                Val::I64(pattern_len as i64),
                Val::I32(system as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_source_manager_assign_text`.
    pub fn raw_slang_source_manager_assign_text(
        &mut self,
        sm: u32,
        path: u32,
        path_len: u64,
        text: u32,
        text_len: u64,
    ) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_source_manager_assign_text",
            &[
                Val::I32(sm as i32),
                Val::I32(path as i32),
                Val::I64(path_len as i64),
                Val::I32(text as i32),
                Val::I64(text_len as i64),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_source_manager_read_file`.
    pub fn raw_slang_source_manager_read_file(
        &mut self,
        sm: u32,
        path: u32,
        path_len: u64,
    ) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_source_manager_read_file",
            &[
                Val::I32(sm as i32),
                Val::I32(path as i32),
                Val::I64(path_len as i64),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_source_manager_file_name`.
    pub fn raw_slang_source_manager_file_name(
        &mut self,
        sm: u32,
        loc: &[u8],
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_loc = self.malloc(16)?;
        self.write(__p_loc, &loc[..16])?;
        let __r = self.call(
            "slang_source_manager_file_name",
            &[
                Val::I32(__sret as i32),
                Val::I32(sm as i32),
                Val::I32(__p_loc as i32),
            ],
        );
        self.free(__p_loc);
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_source_manager_text`.
    pub fn raw_slang_source_manager_text(&mut self, sm: u32, buffer: u32) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __r = self.call(
            "slang_source_manager_text",
            &[
                Val::I32(__sret as i32),
                Val::I32(sm as i32),
                Val::I32(buffer as i32),
            ],
        );
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_source_manager_line`.
    pub fn raw_slang_source_manager_line(&mut self, sm: u32, loc: &[u8]) -> Result<u64, Error> {
        let __p_loc = self.malloc(16)?;
        self.write(__p_loc, &loc[..16])?;
        let __r = self.call(
            "slang_source_manager_line",
            &[Val::I32(sm as i32), Val::I32(__p_loc as i32)],
        );
        self.free(__p_loc);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I64(n)) => *n as u64,
            Some(Val::I32(n)) => *n as u64,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_source_manager_column`.
    pub fn raw_slang_source_manager_column(&mut self, sm: u32, loc: &[u8]) -> Result<u64, Error> {
        let __p_loc = self.malloc(16)?;
        self.write(__p_loc, &loc[..16])?;
        let __r = self.call(
            "slang_source_manager_column",
            &[Val::I32(sm as i32), Val::I32(__p_loc as i32)],
        );
        self.free(__p_loc);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I64(n)) => *n as u64,
            Some(Val::I32(n)) => *n as u64,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_source_manager_is_macro_loc`.
    pub fn raw_slang_source_manager_is_macro_loc(
        &mut self,
        sm: u32,
        loc: &[u8],
    ) -> Result<u32, Error> {
        let __p_loc = self.malloc(16)?;
        self.write(__p_loc, &loc[..16])?;
        let __r = self.call(
            "slang_source_manager_is_macro_loc",
            &[Val::I32(sm as i32), Val::I32(__p_loc as i32)],
        );
        self.free(__p_loc);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_source_manager_original_loc`.
    pub fn raw_slang_source_manager_original_loc(
        &mut self,
        sm: u32,
        loc: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_loc = self.malloc(16)?;
        self.write(__p_loc, &loc[..16])?;
        let __r = self.call(
            "slang_source_manager_original_loc",
            &[
                Val::I32(__sret as i32),
                Val::I32(sm as i32),
                Val::I32(__p_loc as i32),
            ],
        );
        self.free(__p_loc);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_options_create`.
    pub fn raw_slang_options_create(&mut self) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call("slang_options_create", &[Val::I32(__err as i32)]);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_options_destroy`.
    pub fn raw_slang_options_destroy(&mut self, options: u32) -> Result<(), Error> {
        let __r = self.call("slang_options_destroy", &[Val::I32(options as i32)]);
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_options_set_language_version`.
    pub fn raw_slang_options_set_language_version(
        &mut self,
        options: u32,
        version: u32,
    ) -> Result<(), Error> {
        let __r = self.call(
            "slang_options_set_language_version",
            &[Val::I32(options as i32), Val::I32(version as i32)],
        );
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_options_define`.
    pub fn raw_slang_options_define(
        &mut self,
        options: u32,
        name: u32,
        name_len: u64,
        value: u32,
        value_len: u64,
    ) -> Result<(), Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_options_define",
            &[
                Val::I32(options as i32),
                Val::I32(name as i32),
                Val::I64(name_len as i64),
                Val::I32(value as i32),
                Val::I64(value_len as i64),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_options_add_top_module`.
    pub fn raw_slang_options_add_top_module(
        &mut self,
        options: u32,
        name: u32,
        name_len: u64,
    ) -> Result<(), Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_options_add_top_module",
            &[
                Val::I32(options as i32),
                Val::I32(name as i32),
                Val::I64(name_len as i64),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_options_set_compilation_flags`.
    pub fn raw_slang_options_set_compilation_flags(
        &mut self,
        options: u32,
        flags: u32,
    ) -> Result<(), Error> {
        let __r = self.call(
            "slang_options_set_compilation_flags",
            &[Val::I32(options as i32), Val::I32(flags as i32)],
        );
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_syntax_tree_from_text`.
    pub fn raw_slang_syntax_tree_from_text(
        &mut self,
        sm: u32,
        text: u32,
        text_len: u64,
        name: u32,
        name_len: u64,
        path: u32,
        path_len: u64,
        options: u32,
    ) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_syntax_tree_from_text",
            &[
                Val::I32(sm as i32),
                Val::I32(text as i32),
                Val::I64(text_len as i64),
                Val::I32(name as i32),
                Val::I64(name_len as i64),
                Val::I32(path as i32),
                Val::I64(path_len as i64),
                Val::I32(options as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_syntax_tree_from_file`.
    pub fn raw_slang_syntax_tree_from_file(
        &mut self,
        sm: u32,
        path: u32,
        path_len: u64,
        options: u32,
    ) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_syntax_tree_from_file",
            &[
                Val::I32(sm as i32),
                Val::I32(path as i32),
                Val::I64(path_len as i64),
                Val::I32(options as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_syntax_tree_from_buffer`.
    pub fn raw_slang_syntax_tree_from_buffer(
        &mut self,
        sm: u32,
        buffer: u32,
        options: u32,
    ) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_syntax_tree_from_buffer",
            &[
                Val::I32(sm as i32),
                Val::I32(buffer as i32),
                Val::I32(options as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_syntax_tree_retain`.
    pub fn raw_slang_syntax_tree_retain(&mut self, tree: u32) -> Result<u32, Error> {
        let __r = self.call("slang_syntax_tree_retain", &[Val::I32(tree as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_syntax_tree_release`.
    pub fn raw_slang_syntax_tree_release(&mut self, tree: u32) -> Result<(), Error> {
        let __r = self.call("slang_syntax_tree_release", &[Val::I32(tree as i32)]);
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_syntax_tree_root`.
    pub fn raw_slang_syntax_tree_root(&mut self, tree: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_syntax_tree_root",
            &[Val::I32(__sret as i32), Val::I32(tree as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_syntax_tree_diagnostics`.
    pub fn raw_slang_syntax_tree_diagnostics(&mut self, tree: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_syntax_tree_diagnostics",
            &[Val::I32(tree as i32), Val::I32(__err as i32)],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_syntax_tree_source_manager`.
    pub fn raw_slang_syntax_tree_source_manager(&mut self, tree: u32) -> Result<u32, Error> {
        let __r = self.call("slang_syntax_tree_source_manager", &[Val::I32(tree as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_syntax_tree_to_string`.
    pub fn raw_slang_syntax_tree_to_string(&mut self, tree: u32) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_syntax_tree_to_string",
            &[
                Val::I32(__sret as i32),
                Val::I32(tree as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_node_is_null`.
    pub fn raw_slang_node_is_null(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call("slang_node_is_null", &[Val::I32(__p_node as i32)]);
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_node_parent`.
    pub fn raw_slang_node_parent(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_node_parent",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_node_struct`.
    pub fn raw_slang_node_struct(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call("slang_node_struct", &[Val::I32(__p_node as i32)]);
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_node_child_count`.
    pub fn raw_slang_node_child_count(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call("slang_node_child_count", &[Val::I32(__p_node as i32)]);
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_node_range`.
    pub fn raw_slang_node_range(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(32)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_node_range",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(8);
        for __i in 0..8 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_node_to_string`.
    pub fn raw_slang_node_to_string(&mut self, node: &[u8]) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_node_to_string",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_node);
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_node_first_token`.
    pub fn raw_slang_node_first_token(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_node_first_token",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_node_last_token`.
    pub fn raw_slang_node_last_token(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_node_last_token",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_node_is_equivalent`.
    pub fn raw_slang_node_is_equivalent(&mut self, a: &[u8], b: &[u8]) -> Result<u32, Error> {
        let __p_a = self.malloc(16)?;
        self.write(__p_a, &a[..16])?;
        let __p_b = self.malloc(16)?;
        self.write(__p_b, &b[..16])?;
        let __r = self.call(
            "slang_node_is_equivalent",
            &[Val::I32(__p_a as i32), Val::I32(__p_b as i32)],
        );
        self.free(__p_a);
        self.free(__p_b);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_token_location`.
    pub fn raw_slang_token_location(&mut self, token: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_token = self.malloc(16)?;
        self.write(__p_token, &token[..16])?;
        let __r = self.call(
            "slang_token_location",
            &[Val::I32(__sret as i32), Val::I32(__p_token as i32)],
        );
        self.free(__p_token);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_token_range`.
    pub fn raw_slang_token_range(&mut self, token: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(32)?;
        let __p_token = self.malloc(16)?;
        self.write(__p_token, &token[..16])?;
        let __r = self.call(
            "slang_token_range",
            &[Val::I32(__sret as i32), Val::I32(__p_token as i32)],
        );
        self.free(__p_token);
        __r?;
        let mut __w = Vec::with_capacity(8);
        for __i in 0..8 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_token_raw_text`.
    pub fn raw_slang_token_raw_text(&mut self, token: &[u8]) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_token = self.malloc(16)?;
        self.write(__p_token, &token[..16])?;
        let __r = self.call(
            "slang_token_raw_text",
            &[Val::I32(__sret as i32), Val::I32(__p_token as i32)],
        );
        self.free(__p_token);
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_token_value_text`.
    pub fn raw_slang_token_value_text(&mut self, token: &[u8]) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_token = self.malloc(16)?;
        self.write(__p_token, &token[..16])?;
        let __r = self.call(
            "slang_token_value_text",
            &[Val::I32(__sret as i32), Val::I32(__p_token as i32)],
        );
        self.free(__p_token);
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_token_trivia_count`.
    pub fn raw_slang_token_trivia_count(&mut self, token: &[u8]) -> Result<u32, Error> {
        let __p_token = self.malloc(16)?;
        self.write(__p_token, &token[..16])?;
        let __r = self.call("slang_token_trivia_count", &[Val::I32(__p_token as i32)]);
        self.free(__p_token);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_token_trivia_syntax`.
    pub fn raw_slang_token_trivia_syntax(
        &mut self,
        token: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_token = self.malloc(16)?;
        self.write(__p_token, &token[..16])?;
        let __r = self.call(
            "slang_token_trivia_syntax",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_token as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_token);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_node_visit`.
    pub fn raw_slang_node_visit(
        &mut self,
        node: &[u8],
        visitor: u32,
        user: u32,
    ) -> Result<(), Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_node_visit",
            &[
                Val::I32(__p_node as i32),
                Val::I32(visitor as i32),
                Val::I32(user as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_node);
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_syntax_tree_walk`.
    pub fn raw_slang_syntax_tree_walk(
        &mut self,
        tree: u32,
        sink: u32,
        user: u32,
    ) -> Result<(), Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_syntax_tree_walk",
            &[
                Val::I32(tree as i32),
                Val::I32(sink as i32),
                Val::I32(user as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_diagnostics_destroy`.
    pub fn raw_slang_diagnostics_destroy(&mut self, diags: u32) -> Result<(), Error> {
        let __r = self.call("slang_diagnostics_destroy", &[Val::I32(diags as i32)]);
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_diagnostics_count`.
    pub fn raw_slang_diagnostics_count(&mut self, diags: u32) -> Result<u32, Error> {
        let __r = self.call("slang_diagnostics_count", &[Val::I32(diags as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_diagnostics_message`.
    pub fn raw_slang_diagnostics_message(
        &mut self,
        diags: u32,
        index: u32,
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_diagnostics_message",
            &[
                Val::I32(__sret as i32),
                Val::I32(diags as i32),
                Val::I32(index as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_diagnostics_note_message`.
    pub fn raw_slang_diagnostics_note_message(
        &mut self,
        diags: u32,
        index: u32,
        note: u32,
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_diagnostics_note_message",
            &[
                Val::I32(__sret as i32),
                Val::I32(diags as i32),
                Val::I32(index as i32),
                Val::I32(note as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_diagnostics_symbol`.
    pub fn raw_slang_diagnostics_symbol(
        &mut self,
        diags: u32,
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_diagnostics_symbol",
            &[
                Val::I32(__sret as i32),
                Val::I32(diags as i32),
                Val::I32(index as i32),
            ],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_diagnostics_render`.
    pub fn raw_slang_diagnostics_render(
        &mut self,
        diags: u32,
        options: u32,
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_diagnostics_render",
            &[
                Val::I32(__sret as i32),
                Val::I32(diags as i32),
                Val::I32(options as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_compilation_create`.
    pub fn raw_slang_compilation_create(&mut self, options: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_compilation_create",
            &[Val::I32(options as i32), Val::I32(__err as i32)],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_destroy`.
    pub fn raw_slang_compilation_destroy(&mut self, comp: u32) -> Result<(), Error> {
        let __r = self.call("slang_compilation_destroy", &[Val::I32(comp as i32)]);
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_compilation_add_tree`.
    pub fn raw_slang_compilation_add_tree(&mut self, comp: u32, tree: u32) -> Result<(), Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_compilation_add_tree",
            &[
                Val::I32(comp as i32),
                Val::I32(tree as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_compilation_freeze`.
    pub fn raw_slang_compilation_freeze(
        &mut self,
        comp: u32,
        flags: u32,
        report: u32,
    ) -> Result<(), Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_compilation_freeze",
            &[
                Val::I32(comp as i32),
                Val::I32(flags as i32),
                Val::I32(report as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_compilation_is_sealed`.
    pub fn raw_slang_compilation_is_sealed(&mut self, comp: u32) -> Result<u32, Error> {
        let __r = self.call("slang_compilation_is_sealed", &[Val::I32(comp as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_unfreeze`.
    pub fn raw_slang_compilation_unfreeze(&mut self, comp: u32) -> Result<(), Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_compilation_unfreeze",
            &[Val::I32(comp as i32), Val::I32(__err as i32)],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_compilation_root`.
    pub fn raw_slang_compilation_root(&mut self, comp: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_compilation_root",
            &[
                Val::I32(__sret as i32),
                Val::I32(comp as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_root_top_instance_count`.
    pub fn raw_slang_symbol_root_top_instance_count(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_root_top_instance_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_root_top_instance`.
    pub fn raw_slang_symbol_root_top_instance(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_root_top_instance",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_root_compilation_unit_count`.
    pub fn raw_slang_symbol_root_compilation_unit_count(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_root_compilation_unit_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_root_compilation_unit`.
    pub fn raw_slang_symbol_root_compilation_unit(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_root_compilation_unit",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_diagnostics`.
    pub fn raw_slang_compilation_diagnostics(&mut self, comp: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_compilation_diagnostics",
            &[Val::I32(comp as i32), Val::I32(__err as i32)],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_source_manager`.
    pub fn raw_slang_compilation_source_manager(&mut self, comp: u32) -> Result<u32, Error> {
        let __r = self.call("slang_compilation_source_manager", &[Val::I32(comp as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_top_instance_count`.
    pub fn raw_slang_compilation_top_instance_count(&mut self, comp: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_compilation_top_instance_count",
            &[Val::I32(comp as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_top_instance`.
    pub fn raw_slang_compilation_top_instance(
        &mut self,
        comp: u32,
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_compilation_top_instance",
            &[
                Val::I32(__sret as i32),
                Val::I32(comp as i32),
                Val::I32(index as i32),
            ],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_definition_count`.
    pub fn raw_slang_compilation_definition_count(&mut self, comp: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_compilation_definition_count",
            &[Val::I32(comp as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_definition`.
    pub fn raw_slang_compilation_definition(
        &mut self,
        comp: u32,
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_compilation_definition",
            &[
                Val::I32(__sret as i32),
                Val::I32(comp as i32),
                Val::I32(index as i32),
            ],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_package_count`.
    pub fn raw_slang_compilation_package_count(&mut self, comp: u32) -> Result<u32, Error> {
        let __r = self.call("slang_compilation_package_count", &[Val::I32(comp as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_package`.
    pub fn raw_slang_compilation_package(
        &mut self,
        comp: u32,
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_compilation_package",
            &[
                Val::I32(__sret as i32),
                Val::I32(comp as i32),
                Val::I32(index as i32),
            ],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_get_bit_type`.
    pub fn raw_slang_compilation_get_bit_type(&mut self, comp: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_compilation_get_bit_type",
            &[Val::I32(__sret as i32), Val::I32(comp as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_get_byte_type`.
    pub fn raw_slang_compilation_get_byte_type(&mut self, comp: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_compilation_get_byte_type",
            &[Val::I32(__sret as i32), Val::I32(comp as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_unit_for_syntax`.
    pub fn raw_slang_compilation_unit_for_syntax(
        &mut self,
        comp: u32,
        syntax: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_syntax = self.malloc(16)?;
        self.write(__p_syntax, &syntax[..16])?;
        let __r = self.call(
            "slang_compilation_unit_for_syntax",
            &[
                Val::I32(__sret as i32),
                Val::I32(comp as i32),
                Val::I32(__p_syntax as i32),
            ],
        );
        self.free(__p_syntax);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_unit_count`.
    pub fn raw_slang_compilation_unit_count(&mut self, comp: u32) -> Result<u32, Error> {
        let __r = self.call("slang_compilation_unit_count", &[Val::I32(comp as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_unit_at`.
    pub fn raw_slang_compilation_unit_at(
        &mut self,
        comp: u32,
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_compilation_unit_at",
            &[
                Val::I32(__sret as i32),
                Val::I32(comp as i32),
                Val::I32(index as i32),
            ],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_create_script_scope`.
    pub fn raw_slang_compilation_create_script_scope(
        &mut self,
        comp: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_compilation_create_script_scope",
            &[
                Val::I32(__sret as i32),
                Val::I32(comp as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_add_diagnostics`.
    pub fn raw_slang_compilation_add_diagnostics(
        &mut self,
        comp: u32,
        diags: u32,
    ) -> Result<(), Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_compilation_add_diagnostics",
            &[
                Val::I32(comp as i32),
                Val::I32(diags as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_dpi_export_is_null`.
    pub fn raw_slang_dpi_export_is_null(&mut self, exp: u32) -> Result<u32, Error> {
        let __r = self.call("slang_dpi_export_is_null", &[Val::I32(exp as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_dpi_export_count`.
    pub fn raw_slang_compilation_dpi_export_count(&mut self, comp: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_compilation_dpi_export_count",
            &[Val::I32(comp as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_dpi_export`.
    pub fn raw_slang_compilation_dpi_export(
        &mut self,
        comp: u32,
        index: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_compilation_dpi_export",
            &[Val::I32(comp as i32), Val::I32(index as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_dpi_export_subroutine`.
    pub fn raw_slang_dpi_export_subroutine(&mut self, exp: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_dpi_export_subroutine",
            &[Val::I32(__sret as i32), Val::I32(exp as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_dpi_export_c_identifier`.
    pub fn raw_slang_dpi_export_c_identifier(&mut self, exp: u32) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __r = self.call(
            "slang_dpi_export_c_identifier",
            &[Val::I32(__sret as i32), Val::I32(exp as i32)],
        );
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_dpi_export_syntax`.
    pub fn raw_slang_dpi_export_syntax(&mut self, exp: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_dpi_export_syntax",
            &[Val::I32(__sret as i32), Val::I32(exp as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_try_get_definition`.
    pub fn raw_slang_compilation_try_get_definition(
        &mut self,
        comp: u32,
        name: u32,
        name_len: u64,
        scope: &[u8],
    ) -> Result<u32, Error> {
        let __p_scope = self.malloc(16)?;
        self.write(__p_scope, &scope[..16])?;
        let __r = self.call(
            "slang_compilation_try_get_definition",
            &[
                Val::I32(comp as i32),
                Val::I32(name as i32),
                Val::I64(name_len as i64),
                Val::I32(__p_scope as i32),
            ],
        );
        self.free(__p_scope);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_get_int_type`.
    pub fn raw_slang_compilation_get_int_type(&mut self, comp: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_compilation_get_int_type",
            &[Val::I32(__sret as i32), Val::I32(comp as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_get_integer_type`.
    pub fn raw_slang_compilation_get_integer_type(&mut self, comp: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_compilation_get_integer_type",
            &[Val::I32(__sret as i32), Val::I32(comp as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_get_logic_type`.
    pub fn raw_slang_compilation_get_logic_type(&mut self, comp: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_compilation_get_logic_type",
            &[Val::I32(__sret as i32), Val::I32(comp as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_get_real_type`.
    pub fn raw_slang_compilation_get_real_type(&mut self, comp: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_compilation_get_real_type",
            &[Val::I32(__sret as i32), Val::I32(comp as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_get_short_real_type`.
    pub fn raw_slang_compilation_get_short_real_type(
        &mut self,
        comp: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_compilation_get_short_real_type",
            &[Val::I32(__sret as i32), Val::I32(comp as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_get_error_type`.
    pub fn raw_slang_compilation_get_error_type(&mut self, comp: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_compilation_get_error_type",
            &[Val::I32(__sret as i32), Val::I32(comp as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_get_null_type`.
    pub fn raw_slang_compilation_get_null_type(&mut self, comp: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_compilation_get_null_type",
            &[Val::I32(__sret as i32), Val::I32(comp as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_get_gate_type`.
    pub fn raw_slang_compilation_get_gate_type(
        &mut self,
        comp: u32,
        name: u32,
        name_len: u64,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_compilation_get_gate_type",
            &[
                Val::I32(__sret as i32),
                Val::I32(comp as i32),
                Val::I32(name as i32),
                Val::I64(name_len as i64),
            ],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_get_package`.
    pub fn raw_slang_compilation_get_package(
        &mut self,
        comp: u32,
        name: u32,
        name_len: u64,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_compilation_get_package",
            &[
                Val::I32(__sret as i32),
                Val::I32(comp as i32),
                Val::I32(name as i32),
                Val::I64(name_len as i64),
            ],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_get_net_type`.
    pub fn raw_slang_compilation_get_net_type(
        &mut self,
        comp: u32,
        kind: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_compilation_get_net_type",
            &[
                Val::I32(__sret as i32),
                Val::I32(comp as i32),
                Val::I32(kind as i32),
            ],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_get_options`.
    pub fn raw_slang_compilation_get_options(&mut self, comp: u32) -> Result<u32, Error> {
        let __r = self.call("slang_compilation_get_options", &[Val::I32(comp as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_time_scale_base`.
    pub fn raw_slang_time_scale_base(&mut self, ts: u32) -> Result<u32, Error> {
        let __r = self.call("slang_time_scale_base", &[Val::I32(ts as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_time_scale_precision`.
    pub fn raw_slang_time_scale_precision(&mut self, ts: u32) -> Result<u32, Error> {
        let __r = self.call("slang_time_scale_precision", &[Val::I32(ts as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_time_scale_value_unit`.
    pub fn raw_slang_time_scale_value_unit(&mut self, v: u32) -> Result<u32, Error> {
        let __r = self.call("slang_time_scale_value_unit", &[Val::I32(v as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_time_scale_value_magnitude`.
    pub fn raw_slang_time_scale_value_magnitude(&mut self, v: u32) -> Result<u32, Error> {
        let __r = self.call("slang_time_scale_value_magnitude", &[Val::I32(v as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_top_module_count`.
    pub fn raw_slang_compilation_top_module_count(&mut self, comp: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_compilation_top_module_count",
            &[Val::I32(comp as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_top_module_at`.
    pub fn raw_slang_compilation_top_module_at(
        &mut self,
        comp: u32,
        index: u32,
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __r = self.call(
            "slang_compilation_top_module_at",
            &[
                Val::I32(__sret as i32),
                Val::I32(comp as i32),
                Val::I32(index as i32),
            ],
        );
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_compilation_param_override_count`.
    pub fn raw_slang_compilation_param_override_count(&mut self, comp: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_compilation_param_override_count",
            &[Val::I32(comp as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_param_override_at`.
    pub fn raw_slang_compilation_param_override_at(
        &mut self,
        comp: u32,
        index: u32,
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __r = self.call(
            "slang_compilation_param_override_at",
            &[
                Val::I32(__sret as i32),
                Val::I32(comp as i32),
                Val::I32(index as i32),
            ],
        );
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_compilation_default_liblist_count`.
    pub fn raw_slang_compilation_default_liblist_count(&mut self, comp: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_compilation_default_liblist_count",
            &[Val::I32(comp as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_default_liblist_at`.
    pub fn raw_slang_compilation_default_liblist_at(
        &mut self,
        comp: u32,
        index: u32,
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __r = self.call(
            "slang_compilation_default_liblist_at",
            &[
                Val::I32(__sret as i32),
                Val::I32(comp as i32),
                Val::I32(index as i32),
            ],
        );
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_source_library_name`.
    pub fn raw_slang_source_library_name(&mut self, lib: u32) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __r = self.call(
            "slang_source_library_name",
            &[Val::I32(__sret as i32), Val::I32(lib as i32)],
        );
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_source_library_priority`.
    pub fn raw_slang_source_library_priority(&mut self, lib: u32) -> Result<u32, Error> {
        let __r = self.call("slang_source_library_priority", &[Val::I32(lib as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_source_library_is_default`.
    pub fn raw_slang_source_library_is_default(&mut self, lib: u32) -> Result<u32, Error> {
        let __r = self.call("slang_source_library_is_default", &[Val::I32(lib as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_get_default_library`.
    pub fn raw_slang_compilation_get_default_library(&mut self, comp: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_compilation_get_default_library",
            &[Val::I32(comp as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_get_source_library`.
    pub fn raw_slang_compilation_get_source_library(
        &mut self,
        comp: u32,
        name: u32,
        name_len: u64,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_compilation_get_source_library",
            &[
                Val::I32(comp as i32),
                Val::I32(name as i32),
                Val::I64(name_len as i64),
            ],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_get_parse_diagnostics`.
    pub fn raw_slang_compilation_get_parse_diagnostics(&mut self, comp: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_compilation_get_parse_diagnostics",
            &[Val::I32(comp as i32), Val::I32(__err as i32)],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_get_semantic_diagnostics`.
    pub fn raw_slang_compilation_get_semantic_diagnostics(
        &mut self,
        comp: u32,
    ) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_compilation_get_semantic_diagnostics",
            &[Val::I32(comp as i32), Val::I32(__err as i32)],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_is_finalized`.
    pub fn raw_slang_compilation_is_finalized(&mut self, comp: u32) -> Result<u32, Error> {
        let __r = self.call("slang_compilation_is_finalized", &[Val::I32(comp as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_is_elaborated`.
    pub fn raw_slang_compilation_is_elaborated(&mut self, comp: u32) -> Result<u32, Error> {
        let __r = self.call("slang_compilation_is_elaborated", &[Val::I32(comp as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_has_issued_errors`.
    pub fn raw_slang_compilation_has_issued_errors(&mut self, comp: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_compilation_has_issued_errors",
            &[Val::I32(comp as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_has_fatal_errors`.
    pub fn raw_slang_compilation_has_fatal_errors(&mut self, comp: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_compilation_has_fatal_errors",
            &[Val::I32(comp as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_syntax_tree_count`.
    pub fn raw_slang_compilation_syntax_tree_count(&mut self, comp: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_compilation_syntax_tree_count",
            &[Val::I32(comp as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_syntax_tree_at`.
    pub fn raw_slang_compilation_syntax_tree_at(
        &mut self,
        comp: u32,
        index: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_compilation_syntax_tree_at",
            &[Val::I32(comp as i32), Val::I32(index as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_parse_name`.
    pub fn raw_slang_compilation_parse_name(
        &mut self,
        comp: u32,
        name: u32,
        name_len: u64,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_compilation_parse_name",
            &[
                Val::I32(__sret as i32),
                Val::I32(comp as i32),
                Val::I32(name as i32),
                Val::I64(name_len as i64),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_get_std_package`.
    pub fn raw_slang_compilation_get_std_package(&mut self, comp: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_compilation_get_std_package",
            &[Val::I32(__sret as i32), Val::I32(comp as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_get_string_type`.
    pub fn raw_slang_compilation_get_string_type(&mut self, comp: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_compilation_get_string_type",
            &[Val::I32(__sret as i32), Val::I32(comp as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_get_void_type`.
    pub fn raw_slang_compilation_get_void_type(&mut self, comp: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_compilation_get_void_type",
            &[Val::I32(__sret as i32), Val::I32(comp as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_get_unbounded_type`.
    pub fn raw_slang_compilation_get_unbounded_type(
        &mut self,
        comp: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_compilation_get_unbounded_type",
            &[Val::I32(__sret as i32), Val::I32(comp as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_get_type_ref_type`.
    pub fn raw_slang_compilation_get_type_ref_type(
        &mut self,
        comp: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_compilation_get_type_ref_type",
            &[Val::I32(__sret as i32), Val::I32(comp as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_get_unsigned_int_type`.
    pub fn raw_slang_compilation_get_unsigned_int_type(
        &mut self,
        comp: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_compilation_get_unsigned_int_type",
            &[Val::I32(__sret as i32), Val::I32(comp as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_compilation_get_wire_net_type`.
    pub fn raw_slang_compilation_get_wire_net_type(
        &mut self,
        comp: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_compilation_get_wire_net_type",
            &[Val::I32(__sret as i32), Val::I32(comp as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_system_subroutine_name`.
    pub fn raw_slang_system_subroutine_name(&mut self, sub: u32) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __r = self.call(
            "slang_system_subroutine_name",
            &[Val::I32(__sret as i32), Val::I32(sub as i32)],
        );
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_system_subroutine_is_task`.
    pub fn raw_slang_system_subroutine_is_task(&mut self, sub: u32) -> Result<u32, Error> {
        let __r = self.call("slang_system_subroutine_is_task", &[Val::I32(sub as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_compilation_get_system_method`.
    pub fn raw_slang_compilation_get_system_method(
        &mut self,
        comp: u32,
        type_kind: u32,
        name: u32,
        name_len: u64,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_compilation_get_system_method",
            &[
                Val::I32(comp as i32),
                Val::I32(type_kind as i32),
                Val::I32(name as i32),
                Val::I64(name_len as i64),
            ],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_system_subroutine_allow_empty_argument`.
    pub fn raw_slang_system_subroutine_allow_empty_argument(
        &mut self,
        sub: u32,
        arg_index: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_system_subroutine_allow_empty_argument",
            &[Val::I32(sub as i32), Val::I32(arg_index as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_system_subroutine_allow_clocking_argument`.
    pub fn raw_slang_system_subroutine_allow_clocking_argument(
        &mut self,
        sub: u32,
        arg_index: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_system_subroutine_allow_clocking_argument",
            &[Val::I32(sub as i32), Val::I32(arg_index as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_call_system_subroutine`.
    pub fn raw_slang_expr_call_system_subroutine(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_call_system_subroutine",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_system_subroutine_check_arguments`.
    pub fn raw_slang_system_subroutine_check_arguments(
        &mut self,
        call: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_call = self.malloc(16)?;
        self.write(__p_call, &call[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_system_subroutine_check_arguments",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_call as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_call);
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_system_subroutine_bind_argument`.
    pub fn raw_slang_system_subroutine_bind_argument(
        &mut self,
        call: &[u8],
        arg_index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_call = self.malloc(16)?;
        self.write(__p_call, &call[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_system_subroutine_bind_argument",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_call as i32),
                Val::I32(arg_index as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_call);
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_system_subroutine_eval`.
    pub fn raw_slang_system_subroutine_eval(&mut self, call: &[u8]) -> Result<u32, Error> {
        let __p_call = self.malloc(16)?;
        self.write(__p_call, &call[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_system_subroutine_eval",
            &[Val::I32(__p_call as i32), Val::I32(__err as i32)],
        );
        self.free(__p_call);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_system_subroutine_has_output_args`.
    pub fn raw_slang_system_subroutine_has_output_args(&mut self, sub: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_system_subroutine_has_output_args",
            &[Val::I32(sub as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_system_subroutine_kind`.
    pub fn raw_slang_system_subroutine_kind(&mut self, sub: u32) -> Result<u32, Error> {
        let __r = self.call("slang_system_subroutine_kind", &[Val::I32(sub as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_system_subroutine_known_name_id`.
    pub fn raw_slang_system_subroutine_known_name_id(&mut self, sub: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_system_subroutine_known_name_id",
            &[Val::I32(sub as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_system_subroutine_with_clause_mode`.
    pub fn raw_slang_system_subroutine_with_clause_mode(&mut self, sub: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_system_subroutine_with_clause_mode",
            &[Val::I32(sub as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_system_subroutine_kind_str`.
    pub fn raw_slang_system_subroutine_kind_str(&mut self, sub: u32) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __r = self.call(
            "slang_system_subroutine_kind_str",
            &[Val::I32(__sret as i32), Val::I32(sub as i32)],
        );
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_system_subroutine_bad_arg`.
    pub fn raw_slang_system_subroutine_bad_arg(
        &mut self,
        call: &[u8],
        arg_index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_call = self.malloc(16)?;
        self.write(__p_call, &call[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_system_subroutine_bad_arg",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_call as i32),
                Val::I32(arg_index as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_call);
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_system_subroutine_check_arg_count`.
    pub fn raw_slang_system_subroutine_check_arg_count(
        &mut self,
        call: &[u8],
        is_method: u32,
        min: u32,
        max: u32,
    ) -> Result<u32, Error> {
        let __p_call = self.malloc(16)?;
        self.write(__p_call, &call[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_system_subroutine_check_arg_count",
            &[
                Val::I32(__p_call as i32),
                Val::I32(is_method as i32),
                Val::I32(min as i32),
                Val::I32(max as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_call);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_system_subroutine_no_hierarchical`.
    pub fn raw_slang_system_subroutine_no_hierarchical(
        &mut self,
        call: &[u8],
        arg_index: u32,
    ) -> Result<u32, Error> {
        let __p_call = self.malloc(16)?;
        self.write(__p_call, &call[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_system_subroutine_no_hierarchical",
            &[
                Val::I32(__p_call as i32),
                Val::I32(arg_index as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_call);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_system_subroutine_not_const`.
    pub fn raw_slang_system_subroutine_not_const(&mut self, call: &[u8]) -> Result<u32, Error> {
        let __p_call = self.malloc(16)?;
        self.write(__p_call, &call[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_system_subroutine_not_const",
            &[Val::I32(__p_call as i32), Val::I32(__err as i32)],
        );
        self.free(__p_call);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_system_subroutine_unevaluated_context_clears_static_initializer`.
    pub fn raw_slang_system_subroutine_unevaluated_context_clears_static_initializer(
        &mut self,
        call: &[u8],
    ) -> Result<u32, Error> {
        let __p_call = self.malloc(16)?;
        self.write(__p_call, &call[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_system_subroutine_unevaluated_context_clears_static_initializer",
            &[Val::I32(__p_call as i32), Val::I32(__err as i32)],
        );
        self.free(__p_call);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_ast_is_null`.
    pub fn raw_slang_ast_is_null(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call("slang_ast_is_null", &[Val::I32(__p_node as i32)]);
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_ast_range`.
    pub fn raw_slang_ast_range(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(32)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_ast_range",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(8);
        for __i in 0..8 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_ast_syntax`.
    pub fn raw_slang_ast_syntax(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_ast_syntax",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_ast_visit`.
    pub fn raw_slang_ast_visit(
        &mut self,
        root: &[u8],
        visitor: u32,
        user: u32,
    ) -> Result<(), Error> {
        let __p_root = self.malloc(16)?;
        self.write(__p_root, &root[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_ast_visit",
            &[
                Val::I32(__p_root as i32),
                Val::I32(visitor as i32),
                Val::I32(user as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_root);
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_symbol_name`.
    pub fn raw_slang_symbol_name(&mut self, symbol: &[u8]) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_symbol = self.malloc(16)?;
        self.write(__p_symbol, &symbol[..16])?;
        let __r = self.call(
            "slang_symbol_name",
            &[Val::I32(__sret as i32), Val::I32(__p_symbol as i32)],
        );
        self.free(__p_symbol);
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_symbol_location`.
    pub fn raw_slang_symbol_location(&mut self, symbol: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_symbol = self.malloc(16)?;
        self.write(__p_symbol, &symbol[..16])?;
        let __r = self.call(
            "slang_symbol_location",
            &[Val::I32(__sret as i32), Val::I32(__p_symbol as i32)],
        );
        self.free(__p_symbol);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_parent_scope`.
    pub fn raw_slang_symbol_parent_scope(&mut self, symbol: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_symbol = self.malloc(16)?;
        self.write(__p_symbol, &symbol[..16])?;
        let __r = self.call(
            "slang_symbol_parent_scope",
            &[Val::I32(__sret as i32), Val::I32(__p_symbol as i32)],
        );
        self.free(__p_symbol);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_next_sibling`.
    pub fn raw_slang_symbol_next_sibling(&mut self, symbol: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_symbol = self.malloc(16)?;
        self.write(__p_symbol, &symbol[..16])?;
        let __r = self.call(
            "slang_symbol_next_sibling",
            &[Val::I32(__sret as i32), Val::I32(__p_symbol as i32)],
        );
        self.free(__p_symbol);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_hierarchical_path`.
    pub fn raw_slang_symbol_hierarchical_path(&mut self, symbol: &[u8]) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_symbol = self.malloc(16)?;
        self.write(__p_symbol, &symbol[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_symbol_hierarchical_path",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_symbol as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_symbol);
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_symbol_lexical_path`.
    pub fn raw_slang_symbol_lexical_path(&mut self, symbol: &[u8]) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_symbol = self.malloc(16)?;
        self.write(__p_symbol, &symbol[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_symbol_lexical_path",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_symbol as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_symbol);
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_symbol_is_scope`.
    pub fn raw_slang_symbol_is_scope(&mut self, symbol: &[u8]) -> Result<u32, Error> {
        let __p_symbol = self.malloc(16)?;
        self.write(__p_symbol, &symbol[..16])?;
        let __r = self.call("slang_symbol_is_scope", &[Val::I32(__p_symbol as i32)]);
        self.free(__p_symbol);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_is_type`.
    pub fn raw_slang_symbol_is_type(&mut self, symbol: &[u8]) -> Result<u32, Error> {
        let __p_symbol = self.malloc(16)?;
        self.write(__p_symbol, &symbol[..16])?;
        let __r = self.call("slang_symbol_is_type", &[Val::I32(__p_symbol as i32)]);
        self.free(__p_symbol);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_is_value`.
    pub fn raw_slang_symbol_is_value(&mut self, symbol: &[u8]) -> Result<u32, Error> {
        let __p_symbol = self.malloc(16)?;
        self.write(__p_symbol, &symbol[..16])?;
        let __r = self.call("slang_symbol_is_value", &[Val::I32(__p_symbol as i32)]);
        self.free(__p_symbol);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_has_declared_type`.
    pub fn raw_slang_symbol_has_declared_type(&mut self, symbol: &[u8]) -> Result<u32, Error> {
        let __p_symbol = self.malloc(16)?;
        self.write(__p_symbol, &symbol[..16])?;
        let __r = self.call(
            "slang_symbol_has_declared_type",
            &[Val::I32(__p_symbol as i32)],
        );
        self.free(__p_symbol);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_declaring_definition`.
    pub fn raw_slang_symbol_declaring_definition(
        &mut self,
        symbol: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_symbol = self.malloc(16)?;
        self.write(__p_symbol, &symbol[..16])?;
        let __r = self.call(
            "slang_symbol_declaring_definition",
            &[Val::I32(__sret as i32), Val::I32(__p_symbol as i32)],
        );
        self.free(__p_symbol);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_source_library`.
    pub fn raw_slang_symbol_source_library(&mut self, symbol: &[u8]) -> Result<u32, Error> {
        let __p_symbol = self.malloc(16)?;
        self.write(__p_symbol, &symbol[..16])?;
        let __r = self.call(
            "slang_symbol_source_library",
            &[Val::I32(__p_symbol as i32)],
        );
        self.free(__p_symbol);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_rand_mode`.
    pub fn raw_slang_symbol_rand_mode(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call("slang_symbol_rand_mode", &[Val::I32(__p_sym as i32)]);
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_scope_first_member`.
    pub fn raw_slang_scope_first_member(&mut self, scope: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_scope = self.malloc(16)?;
        self.write(__p_scope, &scope[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_scope_first_member",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_scope as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_scope);
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_scope_find`.
    pub fn raw_slang_scope_find(
        &mut self,
        scope: &[u8],
        name: u32,
        name_len: u64,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_scope = self.malloc(16)?;
        self.write(__p_scope, &scope[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_scope_find",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_scope as i32),
                Val::I32(name as i32),
                Val::I64(name_len as i64),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_scope);
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_scope_lookup`.
    pub fn raw_slang_scope_lookup(
        &mut self,
        scope: &[u8],
        name: u32,
        name_len: u64,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_scope = self.malloc(16)?;
        self.write(__p_scope, &scope[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_scope_lookup",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_scope as i32),
                Val::I32(name as i32),
                Val::I64(name_len as i64),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_scope);
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_scope_get_compilation`.
    pub fn raw_slang_scope_get_compilation(&mut self, scope: &[u8]) -> Result<u32, Error> {
        let __p_scope = self.malloc(16)?;
        self.write(__p_scope, &scope[..16])?;
        let __r = self.call("slang_scope_get_compilation", &[Val::I32(__p_scope as i32)]);
        self.free(__p_scope);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_scope_get_compilation_unit`.
    pub fn raw_slang_scope_get_compilation_unit(
        &mut self,
        scope: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_scope = self.malloc(16)?;
        self.write(__p_scope, &scope[..16])?;
        let __r = self.call(
            "slang_scope_get_compilation_unit",
            &[Val::I32(__sret as i32), Val::I32(__p_scope as i32)],
        );
        self.free(__p_scope);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_scope_get_containing_instance`.
    pub fn raw_slang_scope_get_containing_instance(
        &mut self,
        scope: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_scope = self.malloc(16)?;
        self.write(__p_scope, &scope[..16])?;
        let __r = self.call(
            "slang_scope_get_containing_instance",
            &[Val::I32(__sret as i32), Val::I32(__p_scope as i32)],
        );
        self.free(__p_scope);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_scope_get_default_net_type`.
    pub fn raw_slang_scope_get_default_net_type(
        &mut self,
        scope: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_scope = self.malloc(16)?;
        self.write(__p_scope, &scope[..16])?;
        let __r = self.call(
            "slang_scope_get_default_net_type",
            &[Val::I32(__sret as i32), Val::I32(__p_scope as i32)],
        );
        self.free(__p_scope);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_scope_is_procedural_context`.
    pub fn raw_slang_scope_is_procedural_context(&mut self, scope: &[u8]) -> Result<u32, Error> {
        let __p_scope = self.malloc(16)?;
        self.write(__p_scope, &scope[..16])?;
        let __r = self.call(
            "slang_scope_is_procedural_context",
            &[Val::I32(__p_scope as i32)],
        );
        self.free(__p_scope);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_scope_is_uninstantiated`.
    pub fn raw_slang_scope_is_uninstantiated(&mut self, scope: &[u8]) -> Result<u32, Error> {
        let __p_scope = self.malloc(16)?;
        self.write(__p_scope, &scope[..16])?;
        let __r = self.call(
            "slang_scope_is_uninstantiated",
            &[Val::I32(__p_scope as i32)],
        );
        self.free(__p_scope);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_value_type`.
    pub fn raw_slang_value_type(&mut self, symbol: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_symbol = self.malloc(16)?;
        self.write(__p_symbol, &symbol[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_value_type",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_symbol as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_symbol);
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_value_initializer`.
    pub fn raw_slang_value_initializer(&mut self, symbol: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_symbol = self.malloc(16)?;
        self.write(__p_symbol, &symbol[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_value_initializer",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_symbol as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_symbol);
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_declared_type_type`.
    pub fn raw_slang_declared_type_type(&mut self, sym: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_declared_type_type",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_declared_type_initializer`.
    pub fn raw_slang_declared_type_initializer(&mut self, sym: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_declared_type_initializer",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_declared_type_initializer_location`.
    pub fn raw_slang_declared_type_initializer_location(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_declared_type_initializer_location",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_declared_type_initializer_syntax`.
    pub fn raw_slang_declared_type_initializer_syntax(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_declared_type_initializer_syntax",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_declared_type_type_syntax`.
    pub fn raw_slang_declared_type_type_syntax(&mut self, sym: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_declared_type_type_syntax",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_declared_type_is_evaluating`.
    pub fn raw_slang_declared_type_is_evaluating(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_declared_type_is_evaluating",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_instance_body`.
    pub fn raw_slang_instance_body(&mut self, instance: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_instance = self.malloc(16)?;
        self.write(__p_instance, &instance[..16])?;
        let __r = self.call(
            "slang_instance_body",
            &[Val::I32(__sret as i32), Val::I32(__p_instance as i32)],
        );
        self.free(__p_instance);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_instance_definition`.
    pub fn raw_slang_instance_definition(&mut self, instance: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_instance = self.malloc(16)?;
        self.write(__p_instance, &instance[..16])?;
        let __r = self.call(
            "slang_instance_definition",
            &[Val::I32(__sret as i32), Val::I32(__p_instance as i32)],
        );
        self.free(__p_instance);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_instance_parameter_count`.
    pub fn raw_slang_instance_parameter_count(&mut self, instance: &[u8]) -> Result<u32, Error> {
        let __p_instance = self.malloc(16)?;
        self.write(__p_instance, &instance[..16])?;
        let __r = self.call(
            "slang_instance_parameter_count",
            &[Val::I32(__p_instance as i32)],
        );
        self.free(__p_instance);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_instance_parameter`.
    pub fn raw_slang_instance_parameter(
        &mut self,
        instance: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_instance = self.malloc(16)?;
        self.write(__p_instance, &instance[..16])?;
        let __r = self.call(
            "slang_instance_parameter",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_instance as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_instance);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_instance_is_module`.
    pub fn raw_slang_instance_is_module(&mut self, instance: &[u8]) -> Result<u32, Error> {
        let __p_instance = self.malloc(16)?;
        self.write(__p_instance, &instance[..16])?;
        let __r = self.call("slang_instance_is_module", &[Val::I32(__p_instance as i32)]);
        self.free(__p_instance);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_instance_is_interface`.
    pub fn raw_slang_instance_is_interface(&mut self, instance: &[u8]) -> Result<u32, Error> {
        let __p_instance = self.malloc(16)?;
        self.write(__p_instance, &instance[..16])?;
        let __r = self.call(
            "slang_instance_is_interface",
            &[Val::I32(__p_instance as i32)],
        );
        self.free(__p_instance);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_instance_port_connection_count`.
    pub fn raw_slang_instance_port_connection_count(
        &mut self,
        instance: &[u8],
    ) -> Result<u32, Error> {
        let __p_instance = self.malloc(16)?;
        self.write(__p_instance, &instance[..16])?;
        let __r = self.call(
            "slang_instance_port_connection_count",
            &[Val::I32(__p_instance as i32)],
        );
        self.free(__p_instance);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_instance_port_connection_port`.
    pub fn raw_slang_instance_port_connection_port(
        &mut self,
        instance: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_instance = self.malloc(16)?;
        self.write(__p_instance, &instance[..16])?;
        let __r = self.call(
            "slang_instance_port_connection_port",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_instance as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_instance);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_instance_port_connection_expression`.
    pub fn raw_slang_instance_port_connection_expression(
        &mut self,
        instance: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_instance = self.malloc(16)?;
        self.write(__p_instance, &instance[..16])?;
        let __r = self.call(
            "slang_instance_port_connection_expression",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_instance as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_instance);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_instance_port_connection_is_implicit`.
    pub fn raw_slang_instance_port_connection_is_implicit(
        &mut self,
        instance: &[u8],
        index: u32,
    ) -> Result<u32, Error> {
        let __p_instance = self.malloc(16)?;
        self.write(__p_instance, &instance[..16])?;
        let __r = self.call(
            "slang_instance_port_connection_is_implicit",
            &[Val::I32(__p_instance as i32), Val::I32(index as i32)],
        );
        self.free(__p_instance);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_instance_port_connection_is_wildcard`.
    pub fn raw_slang_instance_port_connection_is_wildcard(
        &mut self,
        instance: &[u8],
        index: u32,
    ) -> Result<u32, Error> {
        let __p_instance = self.malloc(16)?;
        self.write(__p_instance, &instance[..16])?;
        let __r = self.call(
            "slang_instance_port_connection_is_wildcard",
            &[Val::I32(__p_instance as i32), Val::I32(index as i32)],
        );
        self.free(__p_instance);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_instance_body_parent_instance`.
    pub fn raw_slang_symbol_instance_body_parent_instance(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_instance_body_parent_instance",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_instance_body_definition`.
    pub fn raw_slang_symbol_instance_body_definition(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_instance_body_definition",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_instance_body_port_count`.
    pub fn raw_slang_symbol_instance_body_port_count(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_instance_body_port_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_instance_body_port`.
    pub fn raw_slang_symbol_instance_body_port(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_instance_body_port",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_instance_body_find_port`.
    pub fn raw_slang_symbol_instance_body_find_port(
        &mut self,
        sym: &[u8],
        name: u32,
        name_len: u64,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_instance_body_find_port",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(name as i32),
                Val::I64(name_len as i64),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_instance_body_has_same_type`.
    pub fn raw_slang_symbol_instance_body_has_same_type(
        &mut self,
        sym: &[u8],
        other: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __p_other = self.malloc(16)?;
        self.write(__p_other, &other[..16])?;
        let __r = self.call(
            "slang_symbol_instance_body_has_same_type",
            &[Val::I32(__p_sym as i32), Val::I32(__p_other as i32)],
        );
        self.free(__p_sym);
        self.free(__p_other);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_instance_array_path_count`.
    pub fn raw_slang_symbol_instance_array_path_count(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_instance_array_path_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_instance_array_path`.
    pub fn raw_slang_symbol_instance_array_path(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_instance_array_path",
            &[Val::I32(__p_sym as i32), Val::I32(index as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_instance_base_array_name`.
    pub fn raw_slang_symbol_instance_base_array_name(
        &mut self,
        sym: &[u8],
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_instance_base_array_name",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_symbol_instance_array_element_count`.
    pub fn raw_slang_symbol_instance_array_element_count(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_instance_array_element_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_instance_array_element`.
    pub fn raw_slang_symbol_instance_array_element(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_instance_array_element",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_instance_array_range`.
    pub fn raw_slang_symbol_instance_array_range(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_instance_array_range",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_instance_array_name`.
    pub fn raw_slang_symbol_instance_array_name(&mut self, sym: &[u8]) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_instance_array_name",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_parameter_value`.
    pub fn raw_slang_parameter_value(&mut self, parameter: &[u8]) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_parameter = self.malloc(16)?;
        self.write(__p_parameter, &parameter[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_parameter_value",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_parameter as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_parameter);
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_definition_kind_of`.
    pub fn raw_slang_definition_kind_of(&mut self, definition: &[u8]) -> Result<u32, Error> {
        let __p_definition = self.malloc(16)?;
        self.write(__p_definition, &definition[..16])?;
        let __r = self.call(
            "slang_definition_kind_of",
            &[Val::I32(__p_definition as i32)],
        );
        self.free(__p_definition);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_definition_cell_define`.
    pub fn raw_slang_definition_cell_define(&mut self, definition: &[u8]) -> Result<u32, Error> {
        let __p_definition = self.malloc(16)?;
        self.write(__p_definition, &definition[..16])?;
        let __r = self.call(
            "slang_definition_cell_define",
            &[Val::I32(__p_definition as i32)],
        );
        self.free(__p_definition);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_definition_default_lifetime`.
    pub fn raw_slang_definition_default_lifetime(
        &mut self,
        definition: &[u8],
    ) -> Result<u32, Error> {
        let __p_definition = self.malloc(16)?;
        self.write(__p_definition, &definition[..16])?;
        let __r = self.call(
            "slang_definition_default_lifetime",
            &[Val::I32(__p_definition as i32)],
        );
        self.free(__p_definition);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_definition_unconnected_drive`.
    pub fn raw_slang_definition_unconnected_drive(
        &mut self,
        definition: &[u8],
    ) -> Result<u32, Error> {
        let __p_definition = self.malloc(16)?;
        self.write(__p_definition, &definition[..16])?;
        let __r = self.call(
            "slang_definition_unconnected_drive",
            &[Val::I32(__p_definition as i32)],
        );
        self.free(__p_definition);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_definition_kind_string`.
    pub fn raw_slang_definition_kind_string(&mut self, definition: &[u8]) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_definition = self.malloc(16)?;
        self.write(__p_definition, &definition[..16])?;
        let __r = self.call(
            "slang_definition_kind_string",
            &[Val::I32(__sret as i32), Val::I32(__p_definition as i32)],
        );
        self.free(__p_definition);
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_definition_article_kind_string`.
    pub fn raw_slang_definition_article_kind_string(
        &mut self,
        definition: &[u8],
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_definition = self.malloc(16)?;
        self.write(__p_definition, &definition[..16])?;
        let __r = self.call(
            "slang_definition_article_kind_string",
            &[Val::I32(__sret as i32), Val::I32(__p_definition as i32)],
        );
        self.free(__p_definition);
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_definition_instance_count`.
    pub fn raw_slang_definition_instance_count(&mut self, definition: &[u8]) -> Result<u64, Error> {
        let __p_definition = self.malloc(16)?;
        self.write(__p_definition, &definition[..16])?;
        let __r = self.call(
            "slang_definition_instance_count",
            &[Val::I32(__p_definition as i32)],
        );
        self.free(__p_definition);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I64(n)) => *n as u64,
            Some(Val::I32(n)) => *n as u64,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_assertion_port_direction`.
    pub fn raw_slang_symbol_assertion_port_direction(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_assertion_port_direction",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_assertion_port_is_local_var`.
    pub fn raw_slang_symbol_assertion_port_is_local_var(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_assertion_port_is_local_var",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_attribute_value`.
    pub fn raw_slang_symbol_attribute_value(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_symbol_attribute_value",
            &[Val::I32(__p_sym as i32), Val::I32(__err as i32)],
        );
        self.free(__p_sym);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_checker_instance_body_parent_instance`.
    pub fn raw_slang_symbol_checker_instance_body_parent_instance(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_checker_instance_body_parent_instance",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_checker_port_count`.
    pub fn raw_slang_symbol_checker_port_count(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_checker_port_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_checker_port`.
    pub fn raw_slang_symbol_checker_port(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_checker_port",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_checker_instance_connection_count`.
    pub fn raw_slang_symbol_checker_instance_connection_count(
        &mut self,
        instance: &[u8],
    ) -> Result<u32, Error> {
        let __p_instance = self.malloc(16)?;
        self.write(__p_instance, &instance[..16])?;
        let __r = self.call(
            "slang_symbol_checker_instance_connection_count",
            &[Val::I32(__p_instance as i32)],
        );
        self.free(__p_instance);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_checker_instance_connection_actual`.
    pub fn raw_slang_symbol_checker_instance_connection_actual(
        &mut self,
        instance: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_instance = self.malloc(16)?;
        self.write(__p_instance, &instance[..16])?;
        let __r = self.call(
            "slang_symbol_checker_instance_connection_actual",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_instance as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_instance);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_checker_instance_connection_attribute_count`.
    pub fn raw_slang_symbol_checker_instance_connection_attribute_count(
        &mut self,
        instance: &[u8],
        index: u32,
    ) -> Result<u32, Error> {
        let __p_instance = self.malloc(16)?;
        self.write(__p_instance, &instance[..16])?;
        let __r = self.call(
            "slang_symbol_checker_instance_connection_attribute_count",
            &[Val::I32(__p_instance as i32), Val::I32(index as i32)],
        );
        self.free(__p_instance);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_checker_instance_connection_attribute`.
    pub fn raw_slang_symbol_checker_instance_connection_attribute(
        &mut self,
        instance: &[u8],
        index: u32,
        attr_index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_instance = self.malloc(16)?;
        self.write(__p_instance, &instance[..16])?;
        let __r = self.call(
            "slang_symbol_checker_instance_connection_attribute",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_instance as i32),
                Val::I32(index as i32),
                Val::I32(attr_index as i32),
            ],
        );
        self.free(__p_instance);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_checker_instance_connection_output_initial_expr`.
    pub fn raw_slang_symbol_checker_instance_connection_output_initial_expr(
        &mut self,
        instance: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_instance = self.malloc(16)?;
        self.write(__p_instance, &instance[..16])?;
        let __r = self.call(
            "slang_symbol_checker_instance_connection_output_initial_expr",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_instance as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_instance);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_class_property_visibility`.
    pub fn raw_slang_symbol_class_property_visibility(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_class_property_visibility",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_class_property_rand_mode`.
    pub fn raw_slang_symbol_class_property_rand_mode(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_class_property_rand_mode",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_clocking_skew_has_value`.
    pub fn raw_slang_clocking_skew_has_value(&mut self, skew: u32) -> Result<u32, Error> {
        let __r = self.call("slang_clocking_skew_has_value", &[Val::I32(skew as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_clock_var_direction`.
    pub fn raw_slang_symbol_clock_var_direction(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_clock_var_direction",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_clock_var_input_skew`.
    pub fn raw_slang_symbol_clock_var_input_skew(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_clock_var_input_skew",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_clock_var_output_skew`.
    pub fn raw_slang_symbol_clock_var_output_skew(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_clock_var_output_skew",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_clocking_block_default_input_skew`.
    pub fn raw_slang_symbol_clocking_block_default_input_skew(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_clocking_block_default_input_skew",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_clocking_block_default_output_skew`.
    pub fn raw_slang_symbol_clocking_block_default_output_skew(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_clocking_block_default_output_skew",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_clocking_block_event`.
    pub fn raw_slang_symbol_clocking_block_event(&mut self, sym: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_clocking_block_event",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_continuous_assign_assignment`.
    pub fn raw_slang_symbol_continuous_assign_assignment(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_continuous_assign_assignment",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_continuous_assign_delay`.
    pub fn raw_slang_symbol_continuous_assign_delay(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_continuous_assign_delay",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_continuous_assign_drive_strength`.
    pub fn raw_slang_symbol_continuous_assign_drive_strength(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_continuous_assign_drive_strength",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_cover_cross_body_queue_type`.
    pub fn raw_slang_symbol_cover_cross_body_queue_type(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_cover_cross_body_queue_type",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_cover_cross_iff_expr`.
    pub fn raw_slang_symbol_cover_cross_iff_expr(&mut self, sym: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_cover_cross_iff_expr",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_cover_cross_target_count`.
    pub fn raw_slang_symbol_cover_cross_target_count(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_cover_cross_target_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_cover_cross_target`.
    pub fn raw_slang_symbol_cover_cross_target(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_cover_cross_target",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_cover_cross_option_count`.
    pub fn raw_slang_symbol_cover_cross_option_count(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_cover_cross_option_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_cover_cross_option_is_type_option`.
    pub fn raw_slang_symbol_cover_cross_option_is_type_option(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_cover_cross_option_is_type_option",
            &[Val::I32(__p_sym as i32), Val::I32(index as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_cover_cross_option_name`.
    pub fn raw_slang_symbol_cover_cross_option_name(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_cover_cross_option_name",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_symbol_cover_cross_option_expression`.
    pub fn raw_slang_symbol_cover_cross_option_expression(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_cover_cross_option_expression",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_coverpoint_coverage_expr`.
    pub fn raw_slang_symbol_coverpoint_coverage_expr(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_coverpoint_coverage_expr",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_coverpoint_iff_expr`.
    pub fn raw_slang_symbol_coverpoint_iff_expr(&mut self, sym: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_coverpoint_iff_expr",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_coverage_bin_kind`.
    pub fn raw_slang_symbol_coverage_bin_kind(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_coverage_bin_kind",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_coverage_bin_is_array`.
    pub fn raw_slang_symbol_coverage_bin_is_array(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_coverage_bin_is_array",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_coverage_bin_is_wildcard`.
    pub fn raw_slang_symbol_coverage_bin_is_wildcard(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_coverage_bin_is_wildcard",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_coverage_bin_is_default`.
    pub fn raw_slang_symbol_coverage_bin_is_default(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_coverage_bin_is_default",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_coverage_bin_is_default_sequence`.
    pub fn raw_slang_symbol_coverage_bin_is_default_sequence(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_coverage_bin_is_default_sequence",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_coverage_bin_iff_expr`.
    pub fn raw_slang_symbol_coverage_bin_iff_expr(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_coverage_bin_iff_expr",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_coverage_bin_number_of_bins_expr`.
    pub fn raw_slang_symbol_coverage_bin_number_of_bins_expr(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_coverage_bin_number_of_bins_expr",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_coverage_bin_set_coverage_expr`.
    pub fn raw_slang_symbol_coverage_bin_set_coverage_expr(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_coverage_bin_set_coverage_expr",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_coverage_bin_with_expr`.
    pub fn raw_slang_symbol_coverage_bin_with_expr(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_coverage_bin_with_expr",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_coverage_bin_cross_select_expr`.
    pub fn raw_slang_symbol_coverage_bin_cross_select_expr(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_coverage_bin_cross_select_expr",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_coverage_bin_value_count`.
    pub fn raw_slang_symbol_coverage_bin_value_count(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_coverage_bin_value_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_coverage_bin_value`.
    pub fn raw_slang_symbol_coverage_bin_value(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_coverage_bin_value",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_coverage_bin_trans_set_count`.
    pub fn raw_slang_symbol_coverage_bin_trans_set_count(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_coverage_bin_trans_set_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_coverage_bin_trans_range_count`.
    pub fn raw_slang_symbol_coverage_bin_trans_range_count(
        &mut self,
        sym: &[u8],
        set_index: u32,
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_coverage_bin_trans_range_count",
            &[Val::I32(__p_sym as i32), Val::I32(set_index as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_coverage_bin_trans_range_item_count`.
    pub fn raw_slang_symbol_coverage_bin_trans_range_item_count(
        &mut self,
        sym: &[u8],
        set_index: u32,
        range_index: u32,
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_coverage_bin_trans_range_item_count",
            &[
                Val::I32(__p_sym as i32),
                Val::I32(set_index as i32),
                Val::I32(range_index as i32),
            ],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_coverage_bin_trans_range_item`.
    pub fn raw_slang_symbol_coverage_bin_trans_range_item(
        &mut self,
        sym: &[u8],
        set_index: u32,
        range_index: u32,
        item_index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_coverage_bin_trans_range_item",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(set_index as i32),
                Val::I32(range_index as i32),
                Val::I32(item_index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_coverage_bin_trans_range_repeat_kind`.
    pub fn raw_slang_symbol_coverage_bin_trans_range_repeat_kind(
        &mut self,
        sym: &[u8],
        set_index: u32,
        range_index: u32,
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_coverage_bin_trans_range_repeat_kind",
            &[
                Val::I32(__p_sym as i32),
                Val::I32(set_index as i32),
                Val::I32(range_index as i32),
            ],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_coverage_bin_trans_range_repeat_from`.
    pub fn raw_slang_symbol_coverage_bin_trans_range_repeat_from(
        &mut self,
        sym: &[u8],
        set_index: u32,
        range_index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_coverage_bin_trans_range_repeat_from",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(set_index as i32),
                Val::I32(range_index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_coverage_bin_trans_range_repeat_to`.
    pub fn raw_slang_symbol_coverage_bin_trans_range_repeat_to(
        &mut self,
        sym: &[u8],
        set_index: u32,
        range_index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_coverage_bin_trans_range_repeat_to",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(set_index as i32),
                Val::I32(range_index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_elab_system_task_kind`.
    pub fn raw_slang_symbol_elab_system_task_kind(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_elab_system_task_kind",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_elab_system_task_assert_condition`.
    pub fn raw_slang_symbol_elab_system_task_assert_condition(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_elab_system_task_assert_condition",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_explicit_import_name`.
    pub fn raw_slang_symbol_explicit_import_name(&mut self, sym: &[u8]) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_explicit_import_name",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_symbol_explicit_import_package_name`.
    pub fn raw_slang_symbol_explicit_import_package_name(
        &mut self,
        sym: &[u8],
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_explicit_import_package_name",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_symbol_explicit_import_package`.
    pub fn raw_slang_symbol_explicit_import_package(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_explicit_import_package",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_explicit_import_imported_symbol`.
    pub fn raw_slang_symbol_explicit_import_imported_symbol(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_explicit_import_imported_symbol",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_variable_flags`.
    pub fn raw_slang_symbol_variable_flags(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call("slang_symbol_variable_flags", &[Val::I32(__p_sym as i32)]);
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_variable_lifetime`.
    pub fn raw_slang_symbol_variable_lifetime(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_variable_lifetime",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_wildcard_import_package_name`.
    pub fn raw_slang_symbol_wildcard_import_package_name(
        &mut self,
        sym: &[u8],
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_wildcard_import_package_name",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_symbol_wildcard_import_package`.
    pub fn raw_slang_symbol_wildcard_import_package(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_wildcard_import_package",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_type_canonical`.
    pub fn raw_slang_type_canonical(&mut self, ty: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_canonical",
            &[Val::I32(__sret as i32), Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_type_to_string`.
    pub fn raw_slang_type_to_string(&mut self, ty: &[u8]) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_type_to_string",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_ty as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_ty);
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_type_bit_width`.
    pub fn raw_slang_type_bit_width(&mut self, ty: &[u8]) -> Result<u64, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_bit_width", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I64(n)) => *n as u64,
            Some(Val::I32(n)) => *n as u64,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_integral`.
    pub fn raw_slang_type_is_integral(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_integral", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_signed`.
    pub fn raw_slang_type_is_signed(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_signed", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_four_state`.
    pub fn raw_slang_type_is_four_state(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_four_state", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_unpacked_array`.
    pub fn raw_slang_type_is_unpacked_array(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_unpacked_array", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_class`.
    pub fn raw_slang_type_is_class(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_class", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_enum`.
    pub fn raw_slang_type_is_enum(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_enum", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_struct`.
    pub fn raw_slang_type_is_struct(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_struct", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_union`.
    pub fn raw_slang_type_is_union(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_union", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_array`.
    pub fn raw_slang_type_is_array(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_array", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_string`.
    pub fn raw_slang_type_is_string(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_string", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_array_element`.
    pub fn raw_slang_type_array_element(&mut self, ty: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_array_element",
            &[Val::I32(__sret as i32), Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_type_enum_base`.
    pub fn raw_slang_type_enum_base(&mut self, ty: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_enum_base",
            &[Val::I32(__sret as i32), Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_enum_member_count`.
    pub fn raw_slang_enum_member_count(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_enum_member_count", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_enum_member`.
    pub fn raw_slang_enum_member(&mut self, ty: &[u8], index: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_enum_member",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_ty as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_ty);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_enum_member_value`.
    pub fn raw_slang_enum_member_value(&mut self, member: &[u8]) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_member = self.malloc(16)?;
        self.write(__p_member, &member[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_enum_member_value",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_member as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_member);
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_type_field_count`.
    pub fn raw_slang_type_field_count(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_field_count", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_field`.
    pub fn raw_slang_type_field(&mut self, ty: &[u8], index: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_field",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_ty as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_ty);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_field_bit_offset`.
    pub fn raw_slang_field_bit_offset(&mut self, field: &[u8]) -> Result<u64, Error> {
        let __p_field = self.malloc(16)?;
        self.write(__p_field, &field[..16])?;
        let __r = self.call("slang_field_bit_offset", &[Val::I32(__p_field as i32)]);
        self.free(__p_field);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I64(n)) => *n as u64,
            Some(Val::I32(n)) => *n as u64,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_field_index`.
    pub fn raw_slang_field_index(&mut self, field: &[u8]) -> Result<u32, Error> {
        let __p_field = self.malloc(16)?;
        self.write(__p_field, &field[..16])?;
        let __r = self.call("slang_field_index", &[Val::I32(__p_field as i32)]);
        self.free(__p_field);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_field_rand_mode`.
    pub fn raw_slang_field_rand_mode(&mut self, field: &[u8]) -> Result<u32, Error> {
        let __p_field = self.malloc(16)?;
        self.write(__p_field, &field[..16])?;
        let __r = self.call("slang_field_rand_mode", &[Val::I32(__p_field as i32)]);
        self.free(__p_field);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_class_base`.
    pub fn raw_slang_type_class_base(&mut self, ty: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_class_base",
            &[Val::I32(__sret as i32), Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_type_class_generic`.
    pub fn raw_slang_type_class_generic(&mut self, ty: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_class_generic",
            &[Val::I32(__sret as i32), Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_type_class_base_constructor_call`.
    pub fn raw_slang_type_class_base_constructor_call(
        &mut self,
        ty: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_class_base_constructor_call",
            &[Val::I32(__sret as i32), Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_type_class_constructor`.
    pub fn raw_slang_type_class_constructor(&mut self, ty: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_class_constructor",
            &[Val::I32(__sret as i32), Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_type_class_first_forward_decl`.
    pub fn raw_slang_type_class_first_forward_decl(
        &mut self,
        ty: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_class_first_forward_decl",
            &[Val::I32(__sret as i32), Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_type_class_implemented_interface_count`.
    pub fn raw_slang_type_class_implemented_interface_count(
        &mut self,
        ty: &[u8],
    ) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_class_implemented_interface_count",
            &[Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_class_implemented_interface`.
    pub fn raw_slang_type_class_implemented_interface(
        &mut self,
        ty: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_class_implemented_interface",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_ty as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_ty);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_type_class_is_abstract`.
    pub fn raw_slang_type_class_is_abstract(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_class_is_abstract", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_class_is_final`.
    pub fn raw_slang_type_class_is_final(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_class_is_final", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_class_is_interface`.
    pub fn raw_slang_type_class_is_interface(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_class_is_interface", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_class_this_var`.
    pub fn raw_slang_type_class_this_var(&mut self, ty: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_class_this_var",
            &[Val::I32(__sret as i32), Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_constraint_block_flags`.
    pub fn raw_slang_symbol_constraint_block_flags(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_constraint_block_flags",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_constraint_block_this_var`.
    pub fn raw_slang_symbol_constraint_block_this_var(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_constraint_block_this_var",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_constraint_block_constraints`.
    pub fn raw_slang_symbol_constraint_block_constraints(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_constraint_block_constraints",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_formal_argument_direction`.
    pub fn raw_slang_symbol_formal_argument_direction(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_formal_argument_direction",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_formal_argument_default_value`.
    pub fn raw_slang_symbol_formal_argument_default_value(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_formal_argument_default_value",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_generate_block_branch_kind`.
    pub fn raw_slang_symbol_generate_block_branch_kind(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_generate_block_branch_kind",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_generate_block_condition_expr`.
    pub fn raw_slang_symbol_generate_block_condition_expr(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_generate_block_condition_expr",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_generate_block_is_uninstantiated`.
    pub fn raw_slang_symbol_generate_block_is_uninstantiated(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_generate_block_is_uninstantiated",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_generate_block_case_item_expr_count`.
    pub fn raw_slang_symbol_generate_block_case_item_expr_count(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_generate_block_case_item_expr_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_generate_block_case_item_expr`.
    pub fn raw_slang_symbol_generate_block_case_item_expr(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_generate_block_case_item_expr",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_generate_block_construct_index`.
    pub fn raw_slang_symbol_generate_block_construct_index(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_generate_block_construct_index",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_generate_block_array_index`.
    pub fn raw_slang_symbol_generate_block_array_index(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_symbol_generate_block_array_index",
            &[Val::I32(__p_sym as i32), Val::I32(__err as i32)],
        );
        self.free(__p_sym);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_generate_block_array_entry_count`.
    pub fn raw_slang_symbol_generate_block_array_entry_count(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_generate_block_array_entry_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_generate_block_array_entry`.
    pub fn raw_slang_symbol_generate_block_array_entry(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_generate_block_array_entry",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_generate_block_array_construct_index`.
    pub fn raw_slang_symbol_generate_block_array_construct_index(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_generate_block_array_construct_index",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_generate_block_array_valid`.
    pub fn raw_slang_symbol_generate_block_array_valid(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_generate_block_array_valid",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_generate_block_array_initial_expr`.
    pub fn raw_slang_symbol_generate_block_array_initial_expr(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_generate_block_array_initial_expr",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_generate_block_array_stop_expr`.
    pub fn raw_slang_symbol_generate_block_array_stop_expr(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_generate_block_array_stop_expr",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_generate_block_array_iter_expr`.
    pub fn raw_slang_symbol_generate_block_array_iter_expr(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_generate_block_array_iter_expr",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_generate_block_array_loop_variable`.
    pub fn raw_slang_symbol_generate_block_array_loop_variable(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_generate_block_array_loop_variable",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_generate_block_external_name`.
    pub fn raw_slang_symbol_generate_block_external_name(
        &mut self,
        sym: &[u8],
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_symbol_generate_block_external_name",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_sym);
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_type_covergroup_argument_count`.
    pub fn raw_slang_type_covergroup_argument_count(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_covergroup_argument_count",
            &[Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_covergroup_argument`.
    pub fn raw_slang_type_covergroup_argument(
        &mut self,
        ty: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_covergroup_argument",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_ty as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_ty);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_type_covergroup_base_group`.
    pub fn raw_slang_type_covergroup_base_group(&mut self, ty: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_covergroup_base_group",
            &[Val::I32(__sret as i32), Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_type_covergroup_coverage_event`.
    pub fn raw_slang_type_covergroup_coverage_event(
        &mut self,
        ty: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_covergroup_coverage_event",
            &[Val::I32(__sret as i32), Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_type_dpi_open_array_is_packed`.
    pub fn raw_slang_type_dpi_open_array_is_packed(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_dpi_open_array_is_packed",
            &[Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_enum_system_id`.
    pub fn raw_slang_type_enum_system_id(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_enum_system_id", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_floating_kind`.
    pub fn raw_slang_type_floating_kind(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_floating_kind", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_fixed_unpacked_array_range`.
    pub fn raw_slang_type_fixed_unpacked_array_range(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_fixed_unpacked_array_range",
            &[Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_forwarding_typedef_type_restriction`.
    pub fn raw_slang_forwarding_typedef_type_restriction(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_forwarding_typedef_type_restriction",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_generic_class_is_interface`.
    pub fn raw_slang_generic_class_is_interface(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_generic_class_is_interface",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_generic_class_default_specialization`.
    pub fn raw_slang_generic_class_default_specialization(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_generic_class_default_specialization",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_generic_class_first_forward_decl`.
    pub fn raw_slang_generic_class_first_forward_decl(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_generic_class_first_forward_decl",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_generic_class_invalid_specialization`.
    pub fn raw_slang_generic_class_invalid_specialization(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_generic_class_invalid_specialization",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_sym);
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_type_is_matching`.
    pub fn raw_slang_type_is_matching(&mut self, a: &[u8], b: &[u8]) -> Result<u32, Error> {
        let __p_a = self.malloc(16)?;
        self.write(__p_a, &a[..16])?;
        let __p_b = self.malloc(16)?;
        self.write(__p_b, &b[..16])?;
        let __r = self.call(
            "slang_type_is_matching",
            &[Val::I32(__p_a as i32), Val::I32(__p_b as i32)],
        );
        self.free(__p_a);
        self.free(__p_b);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_equivalent`.
    pub fn raw_slang_type_is_equivalent(&mut self, a: &[u8], b: &[u8]) -> Result<u32, Error> {
        let __p_a = self.malloc(16)?;
        self.write(__p_a, &a[..16])?;
        let __p_b = self.malloc(16)?;
        self.write(__p_b, &b[..16])?;
        let __r = self.call(
            "slang_type_is_equivalent",
            &[Val::I32(__p_a as i32), Val::I32(__p_b as i32)],
        );
        self.free(__p_a);
        self.free(__p_b);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_assignment_compatible`.
    pub fn raw_slang_type_is_assignment_compatible(
        &mut self,
        a: &[u8],
        b: &[u8],
    ) -> Result<u32, Error> {
        let __p_a = self.malloc(16)?;
        self.write(__p_a, &a[..16])?;
        let __p_b = self.malloc(16)?;
        self.write(__p_b, &b[..16])?;
        let __r = self.call(
            "slang_type_is_assignment_compatible",
            &[Val::I32(__p_a as i32), Val::I32(__p_b as i32)],
        );
        self.free(__p_a);
        self.free(__p_b);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_scalar_kind`.
    pub fn raw_slang_type_scalar_kind(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_scalar_kind", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_can_be_string_like`.
    pub fn raw_slang_type_can_be_string_like(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_can_be_string_like", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_associative_index_type`.
    pub fn raw_slang_type_associative_index_type(&mut self, ty: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_associative_index_type",
            &[Val::I32(__sret as i32), Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_type_bitstream_width`.
    pub fn raw_slang_type_bitstream_width(&mut self, ty: &[u8]) -> Result<u64, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_bitstream_width", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I64(n)) => *n as u64,
            Some(Val::I32(n)) => *n as u64,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_common_base`.
    pub fn raw_slang_type_common_base(&mut self, a: &[u8], b: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_a = self.malloc(16)?;
        self.write(__p_a, &a[..16])?;
        let __p_b = self.malloc(16)?;
        self.write(__p_b, &b[..16])?;
        let __r = self.call(
            "slang_type_common_base",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_a as i32),
                Val::I32(__p_b as i32),
            ],
        );
        self.free(__p_a);
        self.free(__p_b);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_type_integral_flags`.
    pub fn raw_slang_type_integral_flags(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_integral_flags", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_selectable_width`.
    pub fn raw_slang_type_selectable_width(&mut self, ty: &[u8]) -> Result<u64, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_selectable_width", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I64(n)) => *n as u64,
            Some(Val::I32(n)) => *n as u64,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_has_fixed_range`.
    pub fn raw_slang_type_has_fixed_range(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_has_fixed_range", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_implements`.
    pub fn raw_slang_type_implements(
        &mut self,
        ty: &[u8],
        iface_class: &[u8],
    ) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __p_iface_class = self.malloc(16)?;
        self.write(__p_iface_class, &iface_class[..16])?;
        let __r = self.call(
            "slang_type_implements",
            &[Val::I32(__p_ty as i32), Val::I32(__p_iface_class as i32)],
        );
        self.free(__p_ty);
        self.free(__p_iface_class);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_aggregate`.
    pub fn raw_slang_type_is_aggregate(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_aggregate", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_alias`.
    pub fn raw_slang_type_is_alias(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_alias", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_associative_array`.
    pub fn raw_slang_type_is_associative_array(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_is_associative_array",
            &[Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_bitstream_castable`.
    pub fn raw_slang_type_is_bitstream_castable(
        &mut self,
        ty: &[u8],
        rhs: &[u8],
    ) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __p_rhs = self.malloc(16)?;
        self.write(__p_rhs, &rhs[..16])?;
        let __r = self.call(
            "slang_type_is_bitstream_castable",
            &[Val::I32(__p_ty as i32), Val::I32(__p_rhs as i32)],
        );
        self.free(__p_ty);
        self.free(__p_rhs);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_bitstream_type`.
    pub fn raw_slang_type_is_bitstream_type(
        &mut self,
        ty: &[u8],
        destination: u32,
    ) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_is_bitstream_type",
            &[Val::I32(__p_ty as i32), Val::I32(destination as i32)],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_boolean_convertible`.
    pub fn raw_slang_type_is_boolean_convertible(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_is_boolean_convertible",
            &[Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_byte_array`.
    pub fn raw_slang_type_is_byte_array(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_byte_array", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_chandle`.
    pub fn raw_slang_type_is_chandle(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_chandle", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_cast_compatible`.
    pub fn raw_slang_type_is_cast_compatible(
        &mut self,
        ty: &[u8],
        rhs: &[u8],
    ) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __p_rhs = self.malloc(16)?;
        self.write(__p_rhs, &rhs[..16])?;
        let __r = self.call(
            "slang_type_is_cast_compatible",
            &[Val::I32(__p_ty as i32), Val::I32(__p_rhs as i32)],
        );
        self.free(__p_ty);
        self.free(__p_rhs);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_covergroup`.
    pub fn raw_slang_type_is_covergroup(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_covergroup", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_derived_from`.
    pub fn raw_slang_type_is_derived_from(&mut self, ty: &[u8], base: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __p_base = self.malloc(16)?;
        self.write(__p_base, &base[..16])?;
        let __r = self.call(
            "slang_type_is_derived_from",
            &[Val::I32(__p_ty as i32), Val::I32(__p_base as i32)],
        );
        self.free(__p_ty);
        self.free(__p_base);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_dynamically_sized_array`.
    pub fn raw_slang_type_is_dynamically_sized_array(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_is_dynamically_sized_array",
            &[Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_error`.
    pub fn raw_slang_type_is_error(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_error", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_event`.
    pub fn raw_slang_type_is_event(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_event", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_fixed_size`.
    pub fn raw_slang_type_is_fixed_size(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_fixed_size", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_floating`.
    pub fn raw_slang_type_is_floating(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_floating", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_handle_type`.
    pub fn raw_slang_type_is_handle_type(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_handle_type", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_iterable`.
    pub fn raw_slang_type_is_iterable(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_iterable", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_null`.
    pub fn raw_slang_type_is_null(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_null", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_numeric`.
    pub fn raw_slang_type_is_numeric(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_numeric", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_object_handle_type`.
    pub fn raw_slang_type_is_object_handle_type(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_is_object_handle_type",
            &[Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_packed_array`.
    pub fn raw_slang_type_is_packed_array(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_packed_array", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_packed_union`.
    pub fn raw_slang_type_is_packed_union(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_packed_union", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_predefined_integer`.
    pub fn raw_slang_type_is_predefined_integer(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_is_predefined_integer",
            &[Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_property_type`.
    pub fn raw_slang_type_is_property_type(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_property_type", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_queue`.
    pub fn raw_slang_type_is_queue(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_queue", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_scalar`.
    pub fn raw_slang_type_is_scalar(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_scalar", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_sequence_type`.
    pub fn raw_slang_type_is_sequence_type(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_sequence_type", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_simple_bit_vector`.
    pub fn raw_slang_type_is_simple_bit_vector(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_is_simple_bit_vector",
            &[Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_simple_type`.
    pub fn raw_slang_type_is_simple_type(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_simple_type", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_singular`.
    pub fn raw_slang_type_is_singular(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_singular", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_tagged_union`.
    pub fn raw_slang_type_is_tagged_union(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_tagged_union", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_type_ref_type`.
    pub fn raw_slang_type_is_type_ref_type(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_type_ref_type", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_unbounded`.
    pub fn raw_slang_type_is_unbounded(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_unbounded", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_unpacked_struct`.
    pub fn raw_slang_type_is_unpacked_struct(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_unpacked_struct", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_unpacked_union`.
    pub fn raw_slang_type_is_unpacked_union(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_unpacked_union", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_untyped_type`.
    pub fn raw_slang_type_is_untyped_type(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_untyped_type", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_valid_for_dpi_arg`.
    pub fn raw_slang_type_is_valid_for_dpi_arg(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_is_valid_for_dpi_arg",
            &[Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_valid_for_dpi_return`.
    pub fn raw_slang_type_is_valid_for_dpi_return(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_is_valid_for_dpi_return",
            &[Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_valid_for_rand`.
    pub fn raw_slang_type_is_valid_for_rand(
        &mut self,
        ty: &[u8],
        mode: u32,
        language_version: u32,
    ) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_is_valid_for_rand",
            &[
                Val::I32(__p_ty as i32),
                Val::I32(mode as i32),
                Val::I32(language_version as i32),
            ],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_valid_for_sequence`.
    pub fn raw_slang_type_is_valid_for_sequence(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_is_valid_for_sequence",
            &[Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_virtual_interface`.
    pub fn raw_slang_type_is_virtual_interface(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_is_virtual_interface",
            &[Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_void`.
    pub fn raw_slang_type_is_void(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_void", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_alias_visibility`.
    pub fn raw_slang_type_alias_visibility(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_alias_visibility", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_printer_create`.
    pub fn raw_slang_type_printer_create(&mut self) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call("slang_type_printer_create", &[Val::I32(__err as i32)]);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_printer_destroy`.
    pub fn raw_slang_type_printer_destroy(&mut self, printer: u32) -> Result<(), Error> {
        let __r = self.call("slang_type_printer_destroy", &[Val::I32(printer as i32)]);
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_type_printer_append`.
    pub fn raw_slang_type_printer_append(&mut self, printer: u32, ty: &[u8]) -> Result<(), Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_printer_append",
            &[Val::I32(printer as i32), Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_type_printer_clear`.
    pub fn raw_slang_type_printer_clear(&mut self, printer: u32) -> Result<(), Error> {
        let __r = self.call("slang_type_printer_clear", &[Val::I32(printer as i32)]);
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_type_printer_to_string`.
    pub fn raw_slang_type_printer_to_string(&mut self, printer: u32) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_type_printer_to_string",
            &[
                Val::I32(__sret as i32),
                Val::I32(printer as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_type_printer_options`.
    pub fn raw_slang_type_printer_options(&mut self, printer: u32) -> Result<u32, Error> {
        let __r = self.call("slang_type_printer_options", &[Val::I32(printer as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_printer_set_options`.
    pub fn raw_slang_type_printer_set_options(
        &mut self,
        printer: u32,
        options: u32,
    ) -> Result<(), Error> {
        let __r = self.call(
            "slang_type_printer_set_options",
            &[Val::I32(printer as i32), Val::I32(options as i32)],
        );
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_type_fixed_range`.
    pub fn raw_slang_type_fixed_range(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_fixed_range", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_bit_vector_range`.
    pub fn raw_slang_type_bit_vector_range(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_bit_vector_range", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_is_declared_reg`.
    pub fn raw_slang_type_is_declared_reg(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_is_declared_reg", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_packed_array_range`.
    pub fn raw_slang_type_packed_array_range(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_packed_array_range", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_packed_struct_system_id`.
    pub fn raw_slang_type_packed_struct_system_id(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_packed_struct_system_id",
            &[Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_packed_union_is_soft`.
    pub fn raw_slang_type_packed_union_is_soft(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_packed_union_is_soft",
            &[Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_packed_union_is_tagged`.
    pub fn raw_slang_type_packed_union_is_tagged(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_packed_union_is_tagged",
            &[Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_packed_union_system_id`.
    pub fn raw_slang_type_packed_union_system_id(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_packed_union_system_id",
            &[Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_packed_union_tag_bits`.
    pub fn raw_slang_type_packed_union_tag_bits(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_packed_union_tag_bits",
            &[Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_unpacked_struct_system_id`.
    pub fn raw_slang_type_unpacked_struct_system_id(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_unpacked_struct_system_id",
            &[Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_unpacked_union_is_tagged`.
    pub fn raw_slang_type_unpacked_union_is_tagged(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_unpacked_union_is_tagged",
            &[Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_unpacked_union_system_id`.
    pub fn raw_slang_type_unpacked_union_system_id(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_unpacked_union_system_id",
            &[Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_predefined_integer_kind`.
    pub fn raw_slang_type_predefined_integer_kind(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_type_predefined_integer_kind",
            &[Val::I32(__p_ty as i32)],
        );
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_queue_max_bound`.
    pub fn raw_slang_type_queue_max_bound(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call("slang_type_queue_max_bound", &[Val::I32(__p_ty as i32)]);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_net_type_net_kind`.
    pub fn raw_slang_net_type_net_kind(&mut self, net_type: &[u8]) -> Result<u32, Error> {
        let __p_net_type = self.malloc(16)?;
        self.write(__p_net_type, &net_type[..16])?;
        let __r = self.call("slang_net_type_net_kind", &[Val::I32(__p_net_type as i32)]);
        self.free(__p_net_type);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_net_type_resolution_function`.
    pub fn raw_slang_net_type_resolution_function(
        &mut self,
        net_type: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_net_type = self.malloc(16)?;
        self.write(__p_net_type, &net_type[..16])?;
        let __r = self.call(
            "slang_net_type_resolution_function",
            &[Val::I32(__sret as i32), Val::I32(__p_net_type as i32)],
        );
        self.free(__p_net_type);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_net_type_is_built_in`.
    pub fn raw_slang_net_type_is_built_in(&mut self, net_type: &[u8]) -> Result<u32, Error> {
        let __p_net_type = self.malloc(16)?;
        self.write(__p_net_type, &net_type[..16])?;
        let __r = self.call(
            "slang_net_type_is_built_in",
            &[Val::I32(__p_net_type as i32)],
        );
        self.free(__p_net_type);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_net_type_is_error`.
    pub fn raw_slang_net_type_is_error(&mut self, net_type: &[u8]) -> Result<u32, Error> {
        let __p_net_type = self.malloc(16)?;
        self.write(__p_net_type, &net_type[..16])?;
        let __r = self.call("slang_net_type_is_error", &[Val::I32(__p_net_type as i32)]);
        self.free(__p_net_type);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_net_type_get_simulated`.
    pub fn raw_slang_net_type_get_simulated(
        &mut self,
        internal_net: &[u8],
        external_net: &[u8],
        out_should_warn: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_internal_net = self.malloc(16)?;
        self.write(__p_internal_net, &internal_net[..16])?;
        let __p_external_net = self.malloc(16)?;
        self.write(__p_external_net, &external_net[..16])?;
        let __r = self.call(
            "slang_net_type_get_simulated",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_internal_net as i32),
                Val::I32(__p_external_net as i32),
                Val::I32(out_should_warn as i32),
            ],
        );
        self.free(__p_internal_net);
        self.free(__p_external_net);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expression_type`.
    pub fn raw_slang_expression_type(&mut self, expr: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_expr = self.malloc(16)?;
        self.write(__p_expr, &expr[..16])?;
        let __r = self.call(
            "slang_expression_type",
            &[Val::I32(__sret as i32), Val::I32(__p_expr as i32)],
        );
        self.free(__p_expr);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expression_is_bad`.
    pub fn raw_slang_expression_is_bad(&mut self, expr: &[u8]) -> Result<u32, Error> {
        let __p_expr = self.malloc(16)?;
        self.write(__p_expr, &expr[..16])?;
        let __r = self.call("slang_expression_is_bad", &[Val::I32(__p_expr as i32)]);
        self.free(__p_expr);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expression_symbol`.
    pub fn raw_slang_expression_symbol(&mut self, expr: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_expr = self.malloc(16)?;
        self.write(__p_expr, &expr[..16])?;
        let __r = self.call(
            "slang_expression_symbol",
            &[Val::I32(__sret as i32), Val::I32(__p_expr as i32)],
        );
        self.free(__p_expr);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_symbol_reference`.
    pub fn raw_slang_expr_symbol_reference(
        &mut self,
        expr: &[u8],
        allow_packed: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_expr = self.malloc(16)?;
        self.write(__p_expr, &expr[..16])?;
        let __r = self.call(
            "slang_expr_symbol_reference",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_expr as i32),
                Val::I32(allow_packed as i32),
            ],
        );
        self.free(__p_expr);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_has_hierarchical_reference`.
    pub fn raw_slang_expr_has_hierarchical_reference(&mut self, expr: &[u8]) -> Result<u32, Error> {
        let __p_expr = self.malloc(16)?;
        self.write(__p_expr, &expr[..16])?;
        let __r = self.call(
            "slang_expr_has_hierarchical_reference",
            &[Val::I32(__p_expr as i32)],
        );
        self.free(__p_expr);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_is_equivalent_to`.
    pub fn raw_slang_expr_is_equivalent_to(
        &mut self,
        expr: &[u8],
        other: &[u8],
    ) -> Result<u32, Error> {
        let __p_expr = self.malloc(16)?;
        self.write(__p_expr, &expr[..16])?;
        let __p_other = self.malloc(16)?;
        self.write(__p_other, &other[..16])?;
        let __r = self.call(
            "slang_expr_is_equivalent_to",
            &[Val::I32(__p_expr as i32), Val::I32(__p_other as i32)],
        );
        self.free(__p_expr);
        self.free(__p_other);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_is_implicit_string`.
    pub fn raw_slang_expr_is_implicit_string(&mut self, expr: &[u8]) -> Result<u32, Error> {
        let __p_expr = self.malloc(16)?;
        self.write(__p_expr, &expr[..16])?;
        let __r = self.call(
            "slang_expr_is_implicit_string",
            &[Val::I32(__p_expr as i32)],
        );
        self.free(__p_expr);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_is_implicitly_assignable_to`.
    pub fn raw_slang_expr_is_implicitly_assignable_to(
        &mut self,
        expr: &[u8],
        ty: &[u8],
    ) -> Result<u32, Error> {
        let __p_expr = self.malloc(16)?;
        self.write(__p_expr, &expr[..16])?;
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __r = self.call(
            "slang_expr_is_implicitly_assignable_to",
            &[Val::I32(__p_expr as i32), Val::I32(__p_ty as i32)],
        );
        self.free(__p_expr);
        self.free(__p_ty);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_is_unsized_integer`.
    pub fn raw_slang_expr_is_unsized_integer(&mut self, expr: &[u8]) -> Result<u32, Error> {
        let __p_expr = self.malloc(16)?;
        self.write(__p_expr, &expr[..16])?;
        let __r = self.call(
            "slang_expr_is_unsized_integer",
            &[Val::I32(__p_expr as i32)],
        );
        self.free(__p_expr);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expression_constant_value`.
    pub fn raw_slang_expression_constant_value(&mut self, expr: &[u8]) -> Result<u32, Error> {
        let __p_expr = self.malloc(16)?;
        self.write(__p_expr, &expr[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_expression_constant_value",
            &[Val::I32(__p_expr as i32), Val::I32(__err as i32)],
        );
        self.free(__p_expr);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expression_eval_constant`.
    pub fn raw_slang_expression_eval_constant(&mut self, expr: &[u8]) -> Result<u32, Error> {
        let __p_expr = self.malloc(16)?;
        self.write(__p_expr, &expr[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_expression_eval_constant",
            &[Val::I32(__p_expr as i32), Val::I32(__err as i32)],
        );
        self.free(__p_expr);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_integer_literal_value`.
    pub fn raw_slang_expr_integer_literal_value(&mut self, expr: &[u8]) -> Result<u32, Error> {
        let __p_expr = self.malloc(16)?;
        self.write(__p_expr, &expr[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_expr_integer_literal_value",
            &[Val::I32(__p_expr as i32), Val::I32(__err as i32)],
        );
        self.free(__p_expr);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_unbased_unsized_literal_value`.
    pub fn raw_slang_expr_unbased_unsized_literal_value(
        &mut self,
        expr: &[u8],
    ) -> Result<u32, Error> {
        let __p_expr = self.malloc(16)?;
        self.write(__p_expr, &expr[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_expr_unbased_unsized_literal_value",
            &[Val::I32(__p_expr as i32), Val::I32(__err as i32)],
        );
        self.free(__p_expr);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_default_value`.
    pub fn raw_slang_type_default_value(&mut self, ty: &[u8]) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_type_default_value",
            &[Val::I32(__p_ty as i32), Val::I32(__err as i32)],
        );
        self.free(__p_ty);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_type_coerce_value`.
    pub fn raw_slang_type_coerce_value(&mut self, ty: &[u8], value: u32) -> Result<u32, Error> {
        let __p_ty = self.malloc(16)?;
        self.write(__p_ty, &ty[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_type_coerce_value",
            &[
                Val::I32(__p_ty as i32),
                Val::I32(value as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_ty);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_destroy`.
    pub fn raw_slang_constant_destroy(&mut self, c: u32) -> Result<(), Error> {
        let __r = self.call("slang_constant_destroy", &[Val::I32(c as i32)]);
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_constant_kind_of`.
    pub fn raw_slang_constant_kind_of(&mut self, c: u32) -> Result<u32, Error> {
        let __r = self.call("slang_constant_kind_of", &[Val::I32(c as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_has_unknown`.
    pub fn raw_slang_constant_has_unknown(&mut self, c: u32) -> Result<u32, Error> {
        let __r = self.call("slang_constant_has_unknown", &[Val::I32(c as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_string`.
    pub fn raw_slang_constant_string(&mut self, c: u32) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __r = self.call(
            "slang_constant_string",
            &[Val::I32(__sret as i32), Val::I32(c as i32)],
        );
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_constant_size`.
    pub fn raw_slang_constant_size(&mut self, c: u32) -> Result<u64, Error> {
        let __r = self.call("slang_constant_size", &[Val::I32(c as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I64(n)) => *n as u64,
            Some(Val::I32(n)) => *n as u64,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_element`.
    pub fn raw_slang_constant_element(&mut self, c: u32, index: u64) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_constant_element",
            &[
                Val::I32(c as i32),
                Val::I64(index as i64),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_to_string`.
    pub fn raw_slang_constant_to_string(&mut self, c: u32) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __r = self.call(
            "slang_constant_to_string",
            &[Val::I32(__sret as i32), Val::I32(c as i32)],
        );
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_constant_integer`.
    pub fn raw_slang_constant_integer(&mut self, c: u32) -> Result<u32, Error> {
        let __r = self.call("slang_constant_integer", &[Val::I32(c as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_empty`.
    pub fn raw_slang_constant_empty(&mut self, c: u32) -> Result<u32, Error> {
        let __r = self.call("slang_constant_empty", &[Val::I32(c as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_is_container`.
    pub fn raw_slang_constant_is_container(&mut self, c: u32) -> Result<u32, Error> {
        let __r = self.call("slang_constant_is_container", &[Val::I32(c as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_is_true`.
    pub fn raw_slang_constant_is_true(&mut self, c: u32) -> Result<u32, Error> {
        let __r = self.call("slang_constant_is_true", &[Val::I32(c as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_is_false`.
    pub fn raw_slang_constant_is_false(&mut self, c: u32) -> Result<u32, Error> {
        let __r = self.call("slang_constant_is_false", &[Val::I32(c as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_bitstream_width`.
    pub fn raw_slang_constant_bitstream_width(&mut self, c: u32) -> Result<u64, Error> {
        let __r = self.call("slang_constant_bitstream_width", &[Val::I32(c as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I64(n)) => *n as u64,
            Some(Val::I32(n)) => *n as u64,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_get_slice`.
    pub fn raw_slang_constant_get_slice(
        &mut self,
        c: u32,
        upper: u32,
        lower: u32,
    ) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_constant_get_slice",
            &[
                Val::I32(c as i32),
                Val::I32(upper as i32),
                Val::I32(lower as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_convert_to_real`.
    pub fn raw_slang_constant_convert_to_real(&mut self, c: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_constant_convert_to_real",
            &[Val::I32(c as i32), Val::I32(__err as i32)],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_convert_to_short_real`.
    pub fn raw_slang_constant_convert_to_short_real(&mut self, c: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_constant_convert_to_short_real",
            &[Val::I32(c as i32), Val::I32(__err as i32)],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_convert_to_str`.
    pub fn raw_slang_constant_convert_to_str(&mut self, c: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_constant_convert_to_str",
            &[Val::I32(c as i32), Val::I32(__err as i32)],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_convert_to_byte_array`.
    pub fn raw_slang_constant_convert_to_byte_array(
        &mut self,
        c: u32,
        size: u32,
        is_signed: u32,
    ) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_constant_convert_to_byte_array",
            &[
                Val::I32(c as i32),
                Val::I32(size as i32),
                Val::I32(is_signed as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_convert_to_byte_queue`.
    pub fn raw_slang_constant_convert_to_byte_queue(
        &mut self,
        c: u32,
        is_signed: u32,
    ) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_constant_convert_to_byte_queue",
            &[
                Val::I32(c as i32),
                Val::I32(is_signed as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_bit_width`.
    pub fn raw_slang_svint_bit_width(&mut self, v: u32) -> Result<u32, Error> {
        let __r = self.call("slang_svint_bit_width", &[Val::I32(v as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_is_signed`.
    pub fn raw_slang_svint_is_signed(&mut self, v: u32) -> Result<u32, Error> {
        let __r = self.call("slang_svint_is_signed", &[Val::I32(v as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_has_unknown`.
    pub fn raw_slang_svint_has_unknown(&mut self, v: u32) -> Result<u32, Error> {
        let __r = self.call("slang_svint_has_unknown", &[Val::I32(v as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_get_bit`.
    pub fn raw_slang_svint_get_bit(&mut self, v: u32, index: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_svint_get_bit",
            &[Val::I32(v as i32), Val::I32(index as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_to_string`.
    pub fn raw_slang_svint_to_string(&mut self, v: u32) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __r = self.call(
            "slang_svint_to_string",
            &[Val::I32(__sret as i32), Val::I32(v as i32)],
        );
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_svint_count_leading_ones`.
    pub fn raw_slang_svint_count_leading_ones(&mut self, v: u32) -> Result<u32, Error> {
        let __r = self.call("slang_svint_count_leading_ones", &[Val::I32(v as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_count_leading_zeros`.
    pub fn raw_slang_svint_count_leading_zeros(&mut self, v: u32) -> Result<u32, Error> {
        let __r = self.call("slang_svint_count_leading_zeros", &[Val::I32(v as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_count_leading_unknowns`.
    pub fn raw_slang_svint_count_leading_unknowns(&mut self, v: u32) -> Result<u32, Error> {
        let __r = self.call("slang_svint_count_leading_unknowns", &[Val::I32(v as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_count_leading_zs`.
    pub fn raw_slang_svint_count_leading_zs(&mut self, v: u32) -> Result<u32, Error> {
        let __r = self.call("slang_svint_count_leading_zs", &[Val::I32(v as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_count_ones`.
    pub fn raw_slang_svint_count_ones(&mut self, v: u32) -> Result<u32, Error> {
        let __r = self.call("slang_svint_count_ones", &[Val::I32(v as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_count_zeros`.
    pub fn raw_slang_svint_count_zeros(&mut self, v: u32) -> Result<u32, Error> {
        let __r = self.call("slang_svint_count_zeros", &[Val::I32(v as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_count_xs`.
    pub fn raw_slang_svint_count_xs(&mut self, v: u32) -> Result<u32, Error> {
        let __r = self.call("slang_svint_count_xs", &[Val::I32(v as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_count_zs`.
    pub fn raw_slang_svint_count_zs(&mut self, v: u32) -> Result<u32, Error> {
        let __r = self.call("slang_svint_count_zs", &[Val::I32(v as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_active_bits`.
    pub fn raw_slang_svint_active_bits(&mut self, v: u32) -> Result<u32, Error> {
        let __r = self.call("slang_svint_active_bits", &[Val::I32(v as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_min_represented_bits`.
    pub fn raw_slang_svint_min_represented_bits(&mut self, v: u32) -> Result<u32, Error> {
        let __r = self.call("slang_svint_min_represented_bits", &[Val::I32(v as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_is_even`.
    pub fn raw_slang_svint_is_even(&mut self, v: u32) -> Result<u32, Error> {
        let __r = self.call("slang_svint_is_even", &[Val::I32(v as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_is_odd`.
    pub fn raw_slang_svint_is_odd(&mut self, v: u32) -> Result<u32, Error> {
        let __r = self.call("slang_svint_is_odd", &[Val::I32(v as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_is_negative`.
    pub fn raw_slang_svint_is_negative(&mut self, v: u32) -> Result<u32, Error> {
        let __r = self.call("slang_svint_is_negative", &[Val::I32(v as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_is_sign_extended_from`.
    pub fn raw_slang_svint_is_sign_extended_from(
        &mut self,
        v: u32,
        msb: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_svint_is_sign_extended_from",
            &[Val::I32(v as i32), Val::I32(msb as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_reduction_and`.
    pub fn raw_slang_svint_reduction_and(&mut self, v: u32) -> Result<u32, Error> {
        let __r = self.call("slang_svint_reduction_and", &[Val::I32(v as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_reduction_or`.
    pub fn raw_slang_svint_reduction_or(&mut self, v: u32) -> Result<u32, Error> {
        let __r = self.call("slang_svint_reduction_or", &[Val::I32(v as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_reduction_xor`.
    pub fn raw_slang_svint_reduction_xor(&mut self, v: u32) -> Result<u32, Error> {
        let __r = self.call("slang_svint_reduction_xor", &[Val::I32(v as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_logical_impl`.
    pub fn raw_slang_svint_logical_impl(&mut self, lhs: u32, rhs: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_svint_logical_impl",
            &[Val::I32(lhs as i32), Val::I32(rhs as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_logical_equiv`.
    pub fn raw_slang_svint_logical_equiv(&mut self, lhs: u32, rhs: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_svint_logical_equiv",
            &[Val::I32(lhs as i32), Val::I32(rhs as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_logic_value_value`.
    pub fn raw_slang_logic_value_value(&mut self, v: u32) -> Result<u32, Error> {
        let __r = self.call("slang_logic_value_value", &[Val::I32(v as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_logic_value_x`.
    pub fn raw_slang_logic_value_x(&mut self) -> Result<u32, Error> {
        let __r = self.call("slang_logic_value_x", &[]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_logic_value_z`.
    pub fn raw_slang_logic_value_z(&mut self) -> Result<u32, Error> {
        let __r = self.call("slang_logic_value_z", &[]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_logic_value_is_unknown`.
    pub fn raw_slang_logic_value_is_unknown(&mut self, v: u32) -> Result<u32, Error> {
        let __r = self.call("slang_logic_value_is_unknown", &[Val::I32(v as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_logic_value_and`.
    pub fn raw_slang_logic_value_and(&mut self, lhs: u32, rhs: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_logic_value_and",
            &[Val::I32(lhs as i32), Val::I32(rhs as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_logic_value_or`.
    pub fn raw_slang_logic_value_or(&mut self, lhs: u32, rhs: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_logic_value_or",
            &[Val::I32(lhs as i32), Val::I32(rhs as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_logic_value_xor`.
    pub fn raw_slang_logic_value_xor(&mut self, lhs: u32, rhs: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_logic_value_xor",
            &[Val::I32(lhs as i32), Val::I32(rhs as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_logic_value_not`.
    pub fn raw_slang_logic_value_not(&mut self, v: u32) -> Result<u32, Error> {
        let __r = self.call("slang_logic_value_not", &[Val::I32(v as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_pow`.
    pub fn raw_slang_svint_pow(&mut self, v: u32, rhs: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_pow",
            &[
                Val::I32(v as i32),
                Val::I32(rhs as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_replicate`.
    pub fn raw_slang_svint_replicate(&mut self, v: u32, times: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_replicate",
            &[
                Val::I32(v as i32),
                Val::I32(times as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_resize`.
    pub fn raw_slang_svint_resize(&mut self, v: u32, bits: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_resize",
            &[
                Val::I32(v as i32),
                Val::I32(bits as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_reverse`.
    pub fn raw_slang_svint_reverse(&mut self, v: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_reverse",
            &[Val::I32(v as i32), Val::I32(__err as i32)],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_sext`.
    pub fn raw_slang_svint_sext(&mut self, v: u32, bits: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_sext",
            &[
                Val::I32(v as i32),
                Val::I32(bits as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_zext`.
    pub fn raw_slang_svint_zext(&mut self, v: u32, bits: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_zext",
            &[
                Val::I32(v as i32),
                Val::I32(bits as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_extend`.
    pub fn raw_slang_svint_extend(
        &mut self,
        v: u32,
        bits: u32,
        is_signed: u32,
    ) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_extend",
            &[
                Val::I32(v as i32),
                Val::I32(bits as i32),
                Val::I32(is_signed as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_trunc`.
    pub fn raw_slang_svint_trunc(&mut self, v: u32, bits: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_trunc",
            &[
                Val::I32(v as i32),
                Val::I32(bits as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_slice`.
    pub fn raw_slang_svint_slice(&mut self, v: u32, msb: u32, lsb: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_slice",
            &[
                Val::I32(v as i32),
                Val::I32(msb as i32),
                Val::I32(lsb as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_xnor`.
    pub fn raw_slang_svint_xnor(&mut self, lhs: u32, rhs: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_xnor",
            &[
                Val::I32(lhs as i32),
                Val::I32(rhs as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_and`.
    pub fn raw_slang_svint_and(&mut self, lhs: u32, rhs: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_and",
            &[
                Val::I32(lhs as i32),
                Val::I32(rhs as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_or`.
    pub fn raw_slang_svint_or(&mut self, lhs: u32, rhs: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_or",
            &[
                Val::I32(lhs as i32),
                Val::I32(rhs as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_xor`.
    pub fn raw_slang_svint_xor(&mut self, lhs: u32, rhs: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_xor",
            &[
                Val::I32(lhs as i32),
                Val::I32(rhs as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_not`.
    pub fn raw_slang_svint_not(&mut self, v: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_not",
            &[Val::I32(v as i32), Val::I32(__err as i32)],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_iand`.
    pub fn raw_slang_svint_iand(&mut self, v: u32, rhs: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_iand",
            &[
                Val::I32(v as i32),
                Val::I32(rhs as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_ior`.
    pub fn raw_slang_svint_ior(&mut self, v: u32, rhs: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_ior",
            &[
                Val::I32(v as i32),
                Val::I32(rhs as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_ixor`.
    pub fn raw_slang_svint_ixor(&mut self, v: u32, rhs: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_ixor",
            &[
                Val::I32(v as i32),
                Val::I32(rhs as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_set`.
    pub fn raw_slang_svint_set(
        &mut self,
        v: u32,
        msb: u32,
        lsb: u32,
        value: u32,
    ) -> Result<(), Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_set",
            &[
                Val::I32(v as i32),
                Val::I32(msb as i32),
                Val::I32(lsb as i32),
                Val::I32(value as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_svint_set_all_ones`.
    pub fn raw_slang_svint_set_all_ones(&mut self, v: u32) -> Result<(), Error> {
        let __r = self.call("slang_svint_set_all_ones", &[Val::I32(v as i32)]);
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_svint_set_all_zeros`.
    pub fn raw_slang_svint_set_all_zeros(&mut self, v: u32) -> Result<(), Error> {
        let __r = self.call("slang_svint_set_all_zeros", &[Val::I32(v as i32)]);
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_svint_set_all_x`.
    pub fn raw_slang_svint_set_all_x(&mut self, v: u32) -> Result<(), Error> {
        let __r = self.call("slang_svint_set_all_x", &[Val::I32(v as i32)]);
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_svint_set_all_z`.
    pub fn raw_slang_svint_set_all_z(&mut self, v: u32) -> Result<(), Error> {
        let __r = self.call("slang_svint_set_all_z", &[Val::I32(v as i32)]);
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_svint_set_signed`.
    pub fn raw_slang_svint_set_signed(&mut self, v: u32, is_signed: u32) -> Result<(), Error> {
        let __r = self.call(
            "slang_svint_set_signed",
            &[Val::I32(v as i32), Val::I32(is_signed as i32)],
        );
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_svint_flatten_unknowns`.
    pub fn raw_slang_svint_flatten_unknowns(&mut self, v: u32) -> Result<(), Error> {
        let __r = self.call("slang_svint_flatten_unknowns", &[Val::I32(v as i32)]);
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_svint_shrink_to_fit`.
    pub fn raw_slang_svint_shrink_to_fit(&mut self, v: u32) -> Result<(), Error> {
        let __r = self.call("slang_svint_shrink_to_fit", &[Val::I32(v as i32)]);
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_svint_sign_extend_from`.
    pub fn raw_slang_svint_sign_extend_from(&mut self, v: u32, msb: u32) -> Result<(), Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_sign_extend_from",
            &[
                Val::I32(v as i32),
                Val::I32(msb as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_svint_create_fill_x`.
    pub fn raw_slang_svint_create_fill_x(
        &mut self,
        bits: u32,
        is_signed: u32,
    ) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_create_fill_x",
            &[
                Val::I32(bits as i32),
                Val::I32(is_signed as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_create_fill_z`.
    pub fn raw_slang_svint_create_fill_z(
        &mut self,
        bits: u32,
        is_signed: u32,
    ) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_create_fill_z",
            &[
                Val::I32(bits as i32),
                Val::I32(is_signed as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_from_digits`.
    pub fn raw_slang_svint_from_digits(
        &mut self,
        bits: u32,
        base: u32,
        is_signed: u32,
        any_unknown: u32,
        digits: u32,
        digit_count: u32,
    ) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_from_digits",
            &[
                Val::I32(bits as i32),
                Val::I32(base as i32),
                Val::I32(is_signed as i32),
                Val::I32(any_unknown as i32),
                Val::I32(digits as i32),
                Val::I32(digit_count as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_from_double`.
    pub fn raw_slang_svint_from_double(
        &mut self,
        bits: u32,
        value: u32,
        is_signed: u32,
        round: u32,
    ) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_from_double",
            &[
                Val::I32(bits as i32),
                Val::I32(value as i32),
                Val::I32(is_signed as i32),
                Val::I32(round as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_concat`.
    pub fn raw_slang_svint_concat(&mut self, operands: u32, count: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_concat",
            &[
                Val::I32(operands as i32),
                Val::I32(count as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_svint_conditional`.
    pub fn raw_slang_svint_conditional(
        &mut self,
        condition: u32,
        lhs: u32,
        rhs: u32,
    ) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_svint_conditional",
            &[
                Val::I32(condition as i32),
                Val::I32(lhs as i32),
                Val::I32(rhs as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_body`.
    pub fn raw_slang_symbol_body(&mut self, sym: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_body",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_ast_sem_child_count`.
    pub fn raw_slang_ast_sem_child_count(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call("slang_ast_sem_child_count", &[Val::I32(__p_node as i32)]);
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_ast_sem_child`.
    pub fn raw_slang_ast_sem_child(&mut self, node: &[u8], index: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_ast_sem_child",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_binary_op`.
    pub fn raw_slang_expr_binary_op(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call("slang_expr_binary_op", &[Val::I32(__p_node as i32)]);
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_unary_op`.
    pub fn raw_slang_expr_unary_op(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call("slang_expr_unary_op", &[Val::I32(__p_node as i32)]);
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_assignment_is_nonblocking`.
    pub fn raw_slang_expr_assignment_is_nonblocking(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_assignment_is_nonblocking",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_call_subroutine`.
    pub fn raw_slang_expr_call_subroutine(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_call_subroutine",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_member_symbol`.
    pub fn raw_slang_expr_member_symbol(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_member_symbol",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_then_branch`.
    pub fn raw_slang_stmt_then_branch(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_then_branch",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_else_branch`.
    pub fn raw_slang_stmt_else_branch(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_else_branch",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_body`.
    pub fn raw_slang_stmt_body(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_body",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_cond`.
    pub fn raw_slang_stmt_cond(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_cond",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_expr`.
    pub fn raw_slang_stmt_expr(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_expr",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_timing`.
    pub fn raw_slang_stmt_timing(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_timing",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_block_kind`.
    pub fn raw_slang_stmt_block_kind(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call("slang_stmt_block_kind", &[Val::I32(__p_node as i32)]);
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_block_symbol`.
    pub fn raw_slang_stmt_block_symbol(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_block_symbol",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_conditional_check`.
    pub fn raw_slang_stmt_conditional_check(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call("slang_stmt_conditional_check", &[Val::I32(__p_node as i32)]);
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_conditional_condition_count`.
    pub fn raw_slang_stmt_conditional_condition_count(
        &mut self,
        node: &[u8],
    ) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_conditional_condition_count",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_conditional_condition_expr`.
    pub fn raw_slang_stmt_conditional_condition_expr(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_conditional_condition_expr",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_conditional_condition_pattern`.
    pub fn raw_slang_stmt_conditional_condition_pattern(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_conditional_condition_pattern",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_case_condition`.
    pub fn raw_slang_stmt_case_condition(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call("slang_stmt_case_condition", &[Val::I32(__p_node as i32)]);
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_case_check`.
    pub fn raw_slang_stmt_case_check(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call("slang_stmt_case_check", &[Val::I32(__p_node as i32)]);
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_case_default`.
    pub fn raw_slang_stmt_case_default(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_case_default",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_case_item_count`.
    pub fn raw_slang_stmt_case_item_count(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call("slang_stmt_case_item_count", &[Val::I32(__p_node as i32)]);
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_case_item_stmt`.
    pub fn raw_slang_stmt_case_item_stmt(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_case_item_stmt",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_case_item_expr_count`.
    pub fn raw_slang_stmt_case_item_expr_count(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_case_item_expr_count",
            &[Val::I32(__p_node as i32), Val::I32(index as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_case_item_expr`.
    pub fn raw_slang_stmt_case_item_expr(
        &mut self,
        node: &[u8],
        index: u32,
        j: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_case_item_expr",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
                Val::I32(j as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_assertion_kind`.
    pub fn raw_slang_stmt_assertion_kind(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call("slang_stmt_assertion_kind", &[Val::I32(__p_node as i32)]);
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_assertion_if_true`.
    pub fn raw_slang_stmt_assertion_if_true(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_assertion_if_true",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_assertion_if_false`.
    pub fn raw_slang_stmt_assertion_if_false(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_assertion_if_false",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_event_trigger_is_nonblocking`.
    pub fn raw_slang_stmt_event_trigger_is_nonblocking(
        &mut self,
        node: &[u8],
    ) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_event_trigger_is_nonblocking",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_for_loop_initializer_count`.
    pub fn raw_slang_stmt_for_loop_initializer_count(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_for_loop_initializer_count",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_for_loop_initializer`.
    pub fn raw_slang_stmt_for_loop_initializer(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_for_loop_initializer",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_for_loop_var_count`.
    pub fn raw_slang_stmt_for_loop_var_count(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_for_loop_var_count",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_for_loop_var`.
    pub fn raw_slang_stmt_for_loop_var(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_for_loop_var",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_for_loop_step_count`.
    pub fn raw_slang_stmt_for_loop_step_count(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_for_loop_step_count",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_for_loop_step`.
    pub fn raw_slang_stmt_for_loop_step(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_for_loop_step",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_foreach_loop_dim_count`.
    pub fn raw_slang_stmt_foreach_loop_dim_count(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_foreach_loop_dim_count",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_foreach_loop_dim_var`.
    pub fn raw_slang_stmt_foreach_loop_dim_var(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_foreach_loop_dim_var",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_constant_range_left`.
    pub fn raw_slang_constant_range_left(&mut self, r: u32) -> Result<u32, Error> {
        let __r = self.call("slang_constant_range_left", &[Val::I32(r as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_range_right`.
    pub fn raw_slang_constant_range_right(&mut self, r: u32) -> Result<u32, Error> {
        let __r = self.call("slang_constant_range_right", &[Val::I32(r as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_range_width`.
    pub fn raw_slang_constant_range_width(&mut self, r: u32) -> Result<u32, Error> {
        let __r = self.call("slang_constant_range_width", &[Val::I32(r as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_range_lower`.
    pub fn raw_slang_constant_range_lower(&mut self, r: u32) -> Result<u32, Error> {
        let __r = self.call("slang_constant_range_lower", &[Val::I32(r as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_range_upper`.
    pub fn raw_slang_constant_range_upper(&mut self, r: u32) -> Result<u32, Error> {
        let __r = self.call("slang_constant_range_upper", &[Val::I32(r as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_range_is_descending`.
    pub fn raw_slang_constant_range_is_descending(&mut self, r: u32) -> Result<u32, Error> {
        let __r = self.call("slang_constant_range_is_descending", &[Val::I32(r as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_range_reverse`.
    pub fn raw_slang_constant_range_reverse(&mut self, r: u32) -> Result<u32, Error> {
        let __r = self.call("slang_constant_range_reverse", &[Val::I32(r as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_range_subrange`.
    pub fn raw_slang_constant_range_subrange(&mut self, r: u32, select: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_constant_range_subrange",
            &[Val::I32(r as i32), Val::I32(select as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_range_contains_point`.
    pub fn raw_slang_constant_range_contains_point(
        &mut self,
        r: u32,
        index: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_constant_range_contains_point",
            &[Val::I32(r as i32), Val::I32(index as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_range_overlaps`.
    pub fn raw_slang_constant_range_overlaps(&mut self, r: u32, other: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_constant_range_overlaps",
            &[Val::I32(r as i32), Val::I32(other as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_constant_range_translate_index`.
    pub fn raw_slang_constant_range_translate_index(
        &mut self,
        r: u32,
        index: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_constant_range_translate_index",
            &[Val::I32(r as i32), Val::I32(index as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_immediate_assertion_kind`.
    pub fn raw_slang_stmt_immediate_assertion_kind(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_immediate_assertion_kind",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_immediate_assertion_if_true`.
    pub fn raw_slang_stmt_immediate_assertion_if_true(
        &mut self,
        node: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_immediate_assertion_if_true",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_immediate_assertion_if_false`.
    pub fn raw_slang_stmt_immediate_assertion_if_false(
        &mut self,
        node: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_immediate_assertion_if_false",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_immediate_assertion_is_deferred`.
    pub fn raw_slang_stmt_immediate_assertion_is_deferred(
        &mut self,
        node: &[u8],
    ) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_immediate_assertion_is_deferred",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_immediate_assertion_is_final`.
    pub fn raw_slang_stmt_immediate_assertion_is_final(
        &mut self,
        node: &[u8],
    ) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_immediate_assertion_is_final",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_pattern_case_check`.
    pub fn raw_slang_stmt_pattern_case_check(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_pattern_case_check",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_pattern_case_item_count`.
    pub fn raw_slang_stmt_pattern_case_item_count(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_pattern_case_item_count",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_pattern_case_item_pattern`.
    pub fn raw_slang_stmt_pattern_case_item_pattern(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_pattern_case_item_pattern",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_pattern_case_item_filter`.
    pub fn raw_slang_stmt_pattern_case_item_filter(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_pattern_case_item_filter",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_pattern_case_item_stmt`.
    pub fn raw_slang_stmt_pattern_case_item_stmt(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_pattern_case_item_stmt",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_pattern_case_condition`.
    pub fn raw_slang_stmt_pattern_case_condition(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_pattern_case_condition",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_pattern_case_default`.
    pub fn raw_slang_stmt_pattern_case_default(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_pattern_case_default",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_wait_order_event_count`.
    pub fn raw_slang_stmt_wait_order_event_count(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_wait_order_event_count",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_wait_order_event`.
    pub fn raw_slang_stmt_wait_order_event(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_wait_order_event",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_wait_order_if_true`.
    pub fn raw_slang_stmt_wait_order_if_true(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_wait_order_if_true",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_wait_order_if_false`.
    pub fn raw_slang_stmt_wait_order_if_false(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_wait_order_if_false",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_procedural_assign_is_force`.
    pub fn raw_slang_stmt_procedural_assign_is_force(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_procedural_assign_is_force",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_procedural_deassign_is_release`.
    pub fn raw_slang_stmt_procedural_deassign_is_release(
        &mut self,
        node: &[u8],
    ) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_procedural_deassign_is_release",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_randcase_item_count`.
    pub fn raw_slang_stmt_randcase_item_count(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_randcase_item_count",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_randcase_item_expr`.
    pub fn raw_slang_stmt_randcase_item_expr(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_randcase_item_expr",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_randcase_item_stmt`.
    pub fn raw_slang_stmt_randcase_item_stmt(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_randcase_item_stmt",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_randsequence_first_production`.
    pub fn raw_slang_stmt_randsequence_first_production(
        &mut self,
        node: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_randsequence_first_production",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_procedural_checker_instance_count`.
    pub fn raw_slang_stmt_procedural_checker_instance_count(
        &mut self,
        node: &[u8],
    ) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_procedural_checker_instance_count",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_procedural_checker_instance`.
    pub fn raw_slang_stmt_procedural_checker_instance(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_stmt_procedural_checker_instance",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_stmt_is_bad`.
    pub fn raw_slang_stmt_is_bad(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call("slang_stmt_is_bad", &[Val::I32(__p_node as i32)]);
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_stmt_eval`.
    pub fn raw_slang_stmt_eval(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_stmt_eval",
            &[Val::I32(__p_node as i32), Val::I32(__err as i32)],
        );
        self.free(__p_node);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_cond_true`.
    pub fn raw_slang_expr_cond_true(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_cond_true",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_cond_false`.
    pub fn raw_slang_expr_cond_false(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_cond_false",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_select_value`.
    pub fn raw_slang_expr_select_value(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_select_value",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_select_selector`.
    pub fn raw_slang_expr_select_selector(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_select_selector",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_range_left`.
    pub fn raw_slang_expr_range_left(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_range_left",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_range_right`.
    pub fn raw_slang_expr_range_right(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_range_right",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_range_selection_kind`.
    pub fn raw_slang_expr_range_selection_kind(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_range_selection_kind",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_conversion_operand`.
    pub fn raw_slang_expr_conversion_operand(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_conversion_operand",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_conversion_kind`.
    pub fn raw_slang_expr_conversion_kind(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call("slang_expr_conversion_kind", &[Val::I32(__p_node as i32)]);
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_replication_count`.
    pub fn raw_slang_expr_replication_count(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_replication_count",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_replication_concat`.
    pub fn raw_slang_expr_replication_concat(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_replication_concat",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_integer_literal_is_declared_unsized`.
    pub fn raw_slang_expr_integer_literal_is_declared_unsized(
        &mut self,
        node: &[u8],
    ) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_integer_literal_is_declared_unsized",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_unbased_unsized_literal_bit`.
    pub fn raw_slang_expr_unbased_unsized_literal_bit(
        &mut self,
        node: &[u8],
    ) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_unbased_unsized_literal_bit",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_inside_left`.
    pub fn raw_slang_expr_inside_left(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_inside_left",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_inside_range_count`.
    pub fn raw_slang_expr_inside_range_count(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_inside_range_count",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_inside_range`.
    pub fn raw_slang_expr_inside_range(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_inside_range",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_new_array_size`.
    pub fn raw_slang_expr_new_array_size(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_new_array_size",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_new_array_init`.
    pub fn raw_slang_expr_new_array_init(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_new_array_init",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_new_class_constructor_call`.
    pub fn raw_slang_expr_new_class_constructor_call(
        &mut self,
        node: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_new_class_constructor_call",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_new_class_is_super_class`.
    pub fn raw_slang_expr_new_class_is_super_class(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_new_class_is_super_class",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_new_covergroup_argument_count`.
    pub fn raw_slang_expr_new_covergroup_argument_count(
        &mut self,
        node: &[u8],
    ) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_new_covergroup_argument_count",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_new_covergroup_argument`.
    pub fn raw_slang_expr_new_covergroup_argument(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_new_covergroup_argument",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_tagged_union_value`.
    pub fn raw_slang_expr_tagged_union_value(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_tagged_union_value",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_arbitrary_symbol`.
    pub fn raw_slang_expr_arbitrary_symbol(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_arbitrary_symbol",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_assignment_is_compound`.
    pub fn raw_slang_expr_assignment_is_compound(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_assignment_is_compound",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_assignment_is_lvalue_arg`.
    pub fn raw_slang_expr_assignment_is_lvalue_arg(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_assignment_is_lvalue_arg",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_assignment_op`.
    pub fn raw_slang_expr_assignment_op(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call("slang_expr_assignment_op", &[Val::I32(__p_node as i32)]);
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_assignment_timing`.
    pub fn raw_slang_expr_assignment_timing(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_assignment_timing",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_pattern_element_count`.
    pub fn raw_slang_expr_pattern_element_count(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_pattern_element_count",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_pattern_element`.
    pub fn raw_slang_expr_pattern_element(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_pattern_element",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_assertion_instance_is_recursive`.
    pub fn raw_slang_expr_assertion_instance_is_recursive(
        &mut self,
        node: &[u8],
    ) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_assertion_instance_is_recursive",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_assertion_instance_local_var_count`.
    pub fn raw_slang_expr_assertion_instance_local_var_count(
        &mut self,
        node: &[u8],
    ) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_assertion_instance_local_var_count",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_assertion_instance_local_var`.
    pub fn raw_slang_expr_assertion_instance_local_var(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_assertion_instance_local_var",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_assertion_instance_argument_count`.
    pub fn raw_slang_expr_assertion_instance_argument_count(
        &mut self,
        node: &[u8],
    ) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_assertion_instance_argument_count",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_assertion_instance_argument_port`.
    pub fn raw_slang_expr_assertion_instance_argument_port(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_assertion_instance_argument_port",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_assertion_instance_argument_actual`.
    pub fn raw_slang_expr_assertion_instance_argument_actual(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_assertion_instance_argument_actual",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_call_iterator_expr`.
    pub fn raw_slang_expr_call_iterator_expr(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_call_iterator_expr",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_call_iterator_var`.
    pub fn raw_slang_expr_call_iterator_var(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_call_iterator_var",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_call_randomize_inline_constraints`.
    pub fn raw_slang_expr_call_randomize_inline_constraints(
        &mut self,
        node: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_call_randomize_inline_constraints",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_call_extra_info_kind`.
    pub fn raw_slang_expr_call_extra_info_kind(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_call_extra_info_kind",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_call_system_scope`.
    pub fn raw_slang_expr_call_system_scope(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_call_system_scope",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_call_subroutine_kind`.
    pub fn raw_slang_expr_call_subroutine_kind(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_call_subroutine_kind",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_call_subroutine_name`.
    pub fn raw_slang_expr_call_subroutine_name(&mut self, node: &[u8]) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_call_subroutine_name",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_expr_call_is_system_call`.
    pub fn raw_slang_expr_call_is_system_call(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_call_is_system_call",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_call_this_class`.
    pub fn raw_slang_expr_call_this_class(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_call_this_class",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_cond_condition_count`.
    pub fn raw_slang_expr_cond_condition_count(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_cond_condition_count",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_cond_condition_expr`.
    pub fn raw_slang_expr_cond_condition_expr(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_cond_condition_expr",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_cond_condition_pattern`.
    pub fn raw_slang_expr_cond_condition_pattern(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_cond_condition_pattern",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_conversion_is_const_cast`.
    pub fn raw_slang_expr_conversion_is_const_cast(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_conversion_is_const_cast",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_conversion_is_implicit`.
    pub fn raw_slang_expr_conversion_is_implicit(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_conversion_is_implicit",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_copy_class_source`.
    pub fn raw_slang_expr_copy_class_source(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_copy_class_source",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_dist_left`.
    pub fn raw_slang_expr_dist_left(&mut self, node: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_dist_left",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_dist_item_count`.
    pub fn raw_slang_expr_dist_item_count(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call("slang_expr_dist_item_count", &[Val::I32(__p_node as i32)]);
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_dist_item_value`.
    pub fn raw_slang_expr_dist_item_value(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_dist_item_value",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_dist_item_weight_kind`.
    pub fn raw_slang_expr_dist_item_weight_kind(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_dist_item_weight_kind",
            &[Val::I32(__p_node as i32), Val::I32(index as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_dist_item_weight_expr`.
    pub fn raw_slang_expr_dist_item_weight_expr(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_dist_item_weight_expr",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_dist_default_weight_kind`.
    pub fn raw_slang_expr_dist_default_weight_kind(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_dist_default_weight_kind",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_dist_default_weight_expr`.
    pub fn raw_slang_expr_dist_default_weight_expr(
        &mut self,
        node: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_dist_default_weight_expr",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_eval_lvalue`.
    pub fn raw_slang_expr_eval_lvalue(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_expr_eval_lvalue",
            &[Val::I32(__p_node as i32), Val::I32(__err as i32)],
        );
        self.free(__p_node);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lvalue_destroy`.
    pub fn raw_slang_lvalue_destroy(&mut self, lval: u32) -> Result<(), Error> {
        let __r = self.call("slang_lvalue_destroy", &[Val::I32(lval as i32)]);
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_lvalue_is_bad`.
    pub fn raw_slang_lvalue_is_bad(&mut self, lval: u32) -> Result<u32, Error> {
        let __r = self.call("slang_lvalue_is_bad", &[Val::I32(lval as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lvalue_load`.
    pub fn raw_slang_lvalue_load(&mut self, lval: u32) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __r = self.call(
            "slang_lvalue_load",
            &[Val::I32(__sret as i32), Val::I32(lval as i32)],
        );
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_lvalue_store_int`.
    pub fn raw_slang_lvalue_store_int(&mut self, lval: u32, value: u64) -> Result<u32, Error> {
        let __r = self.call(
            "slang_lvalue_store_int",
            &[Val::I32(lval as i32), Val::I64(value as i64)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_replicated_pattern_count`.
    pub fn raw_slang_expr_replicated_pattern_count(
        &mut self,
        node: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_replicated_pattern_count",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_structured_pattern_member_setter_count`.
    pub fn raw_slang_expr_structured_pattern_member_setter_count(
        &mut self,
        node: &[u8],
    ) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_structured_pattern_member_setter_count",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_structured_pattern_member_setter_member`.
    pub fn raw_slang_expr_structured_pattern_member_setter_member(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_structured_pattern_member_setter_member",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_structured_pattern_member_setter_expr`.
    pub fn raw_slang_expr_structured_pattern_member_setter_expr(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_structured_pattern_member_setter_expr",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_structured_pattern_type_setter_count`.
    pub fn raw_slang_expr_structured_pattern_type_setter_count(
        &mut self,
        node: &[u8],
    ) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_structured_pattern_type_setter_count",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_structured_pattern_type_setter_type`.
    pub fn raw_slang_expr_structured_pattern_type_setter_type(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_structured_pattern_type_setter_type",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_structured_pattern_type_setter_expr`.
    pub fn raw_slang_expr_structured_pattern_type_setter_expr(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_structured_pattern_type_setter_expr",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_structured_pattern_index_setter_count`.
    pub fn raw_slang_expr_structured_pattern_index_setter_count(
        &mut self,
        node: &[u8],
    ) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_structured_pattern_index_setter_count",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_structured_pattern_index_setter_index`.
    pub fn raw_slang_expr_structured_pattern_index_setter_index(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_structured_pattern_index_setter_index",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_structured_pattern_index_setter_expr`.
    pub fn raw_slang_expr_structured_pattern_index_setter_expr(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_structured_pattern_index_setter_expr",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_structured_pattern_default_setter`.
    pub fn raw_slang_expr_structured_pattern_default_setter(
        &mut self,
        node: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_structured_pattern_default_setter",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_streaming_bitstream_width`.
    pub fn raw_slang_expr_streaming_bitstream_width(&mut self, node: &[u8]) -> Result<u64, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_streaming_bitstream_width",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I64(n)) => *n as u64,
            Some(Val::I32(n)) => *n as u64,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_streaming_slice_size`.
    pub fn raw_slang_expr_streaming_slice_size(&mut self, node: &[u8]) -> Result<u64, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_streaming_slice_size",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I64(n)) => *n as u64,
            Some(Val::I32(n)) => *n as u64,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_streaming_is_fixed_size`.
    pub fn raw_slang_expr_streaming_is_fixed_size(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_streaming_is_fixed_size",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_streaming_stream_count`.
    pub fn raw_slang_expr_streaming_stream_count(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_streaming_stream_count",
            &[Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_expr_streaming_stream_operand`.
    pub fn raw_slang_expr_streaming_stream_operand(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_streaming_stream_operand",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_streaming_stream_with_expr`.
    pub fn raw_slang_expr_streaming_stream_with_expr(
        &mut self,
        node: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_streaming_stream_with_expr",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_node as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_node);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_expr_string_literal_value`.
    pub fn raw_slang_expr_string_literal_value(&mut self, node: &[u8]) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_string_literal_value",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_expr_string_literal_raw_value`.
    pub fn raw_slang_expr_string_literal_raw_value(
        &mut self,
        node: &[u8],
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __r = self.call(
            "slang_expr_string_literal_raw_value",
            &[Val::I32(__sret as i32), Val::I32(__p_node as i32)],
        );
        self.free(__p_node);
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_expr_string_literal_int_value`.
    pub fn raw_slang_expr_string_literal_int_value(&mut self, node: &[u8]) -> Result<u32, Error> {
        let __p_node = self.malloc(16)?;
        self.write(__p_node, &node[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_expr_string_literal_int_value",
            &[Val::I32(__p_node as i32), Val::I32(__err as i32)],
        );
        self.free(__p_node);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_driver_create`.
    pub fn raw_slang_driver_create(&mut self) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call("slang_driver_create", &[Val::I32(__err as i32)]);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_driver_create_bare`.
    pub fn raw_slang_driver_create_bare(&mut self) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call("slang_driver_create_bare", &[Val::I32(__err as i32)]);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_driver_destroy`.
    pub fn raw_slang_driver_destroy(&mut self, driver: u32) -> Result<(), Error> {
        let __r = self.call("slang_driver_destroy", &[Val::I32(driver as i32)]);
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_driver_add_standard_args`.
    pub fn raw_slang_driver_add_standard_args(&mut self, driver: u32) -> Result<(), Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_driver_add_standard_args",
            &[Val::I32(driver as i32), Val::I32(__err as i32)],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_driver_add_option`.
    pub fn raw_slang_driver_add_option(
        &mut self,
        driver: u32,
        names: u32,
        names_len: u64,
        kind: u32,
        description: u32,
        description_len: u64,
        value_name: u32,
        value_name_len: u64,
    ) -> Result<(), Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_driver_add_option",
            &[
                Val::I32(driver as i32),
                Val::I32(names as i32),
                Val::I64(names_len as i64),
                Val::I32(kind as i32),
                Val::I32(description as i32),
                Val::I64(description_len as i64),
                Val::I32(value_name as i32),
                Val::I64(value_name_len as i64),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_parse_options_create`.
    pub fn raw_slang_parse_options_create(&mut self) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call("slang_parse_options_create", &[Val::I32(__err as i32)]);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_parse_options_destroy`.
    pub fn raw_slang_parse_options_destroy(&mut self, options: u32) -> Result<(), Error> {
        let __r = self.call("slang_parse_options_destroy", &[Val::I32(options as i32)]);
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_parse_options_set_support_comments`.
    pub fn raw_slang_parse_options_set_support_comments(
        &mut self,
        options: u32,
        value: u32,
    ) -> Result<(), Error> {
        let __r = self.call(
            "slang_parse_options_set_support_comments",
            &[Val::I32(options as i32), Val::I32(value as i32)],
        );
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_parse_options_support_comments`.
    pub fn raw_slang_parse_options_support_comments(&mut self, options: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_parse_options_support_comments",
            &[Val::I32(options as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_parse_options_set_ignore_program_name`.
    pub fn raw_slang_parse_options_set_ignore_program_name(
        &mut self,
        options: u32,
        value: u32,
    ) -> Result<(), Error> {
        let __r = self.call(
            "slang_parse_options_set_ignore_program_name",
            &[Val::I32(options as i32), Val::I32(value as i32)],
        );
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_parse_options_ignore_program_name`.
    pub fn raw_slang_parse_options_ignore_program_name(
        &mut self,
        options: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_parse_options_ignore_program_name",
            &[Val::I32(options as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_parse_options_set_expand_env_vars`.
    pub fn raw_slang_parse_options_set_expand_env_vars(
        &mut self,
        options: u32,
        value: u32,
    ) -> Result<(), Error> {
        let __r = self.call(
            "slang_parse_options_set_expand_env_vars",
            &[Val::I32(options as i32), Val::I32(value as i32)],
        );
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_parse_options_expand_env_vars`.
    pub fn raw_slang_parse_options_expand_env_vars(&mut self, options: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_parse_options_expand_env_vars",
            &[Val::I32(options as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_parse_options_set_ignore_duplicates`.
    pub fn raw_slang_parse_options_set_ignore_duplicates(
        &mut self,
        options: u32,
        value: u32,
    ) -> Result<(), Error> {
        let __r = self.call(
            "slang_parse_options_set_ignore_duplicates",
            &[Val::I32(options as i32), Val::I32(value as i32)],
        );
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_parse_options_ignore_duplicates`.
    pub fn raw_slang_parse_options_ignore_duplicates(
        &mut self,
        options: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_parse_options_ignore_duplicates",
            &[Val::I32(options as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_driver_help_text`.
    pub fn raw_slang_driver_help_text(
        &mut self,
        driver: u32,
        overview: u32,
        overview_len: u64,
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_driver_help_text",
            &[
                Val::I32(__sret as i32),
                Val::I32(driver as i32),
                Val::I32(overview as i32),
                Val::I64(overview_len as i64),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_driver_process_options`.
    pub fn raw_slang_driver_process_options(&mut self, driver: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_driver_process_options",
            &[Val::I32(driver as i32), Val::I32(__err as i32)],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_driver_parse_sources`.
    pub fn raw_slang_driver_parse_sources(&mut self, driver: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_driver_parse_sources",
            &[Val::I32(driver as i32), Val::I32(__err as i32)],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_driver_source_manager`.
    pub fn raw_slang_driver_source_manager(&mut self, driver: u32) -> Result<u32, Error> {
        let __r = self.call("slang_driver_source_manager", &[Val::I32(driver as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_driver_tree_count`.
    pub fn raw_slang_driver_tree_count(&mut self, driver: u32) -> Result<u32, Error> {
        let __r = self.call("slang_driver_tree_count", &[Val::I32(driver as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_driver_tree`.
    pub fn raw_slang_driver_tree(&mut self, driver: u32, index: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_driver_tree",
            &[Val::I32(driver as i32), Val::I32(index as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_driver_create_compilation`.
    pub fn raw_slang_driver_create_compilation(
        &mut self,
        driver: u32,
        extra_flags: u32,
    ) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_driver_create_compilation",
            &[
                Val::I32(driver as i32),
                Val::I32(extra_flags as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_driver_report_compilation`.
    pub fn raw_slang_driver_report_compilation(
        &mut self,
        driver: u32,
        comp: u32,
        quiet: u32,
    ) -> Result<(), Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_driver_report_compilation",
            &[
                Val::I32(driver as i32),
                Val::I32(comp as i32),
                Val::I32(quiet as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_driver_report_diagnostics`.
    pub fn raw_slang_driver_report_diagnostics(
        &mut self,
        driver: u32,
        quiet: u32,
    ) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_driver_report_diagnostics",
            &[
                Val::I32(driver as i32),
                Val::I32(quiet as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_driver_language_version`.
    pub fn raw_slang_driver_language_version(&mut self, driver: u32) -> Result<u32, Error> {
        let __r = self.call("slang_driver_language_version", &[Val::I32(driver as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_driver_process_command_files`.
    pub fn raw_slang_driver_process_command_files(
        &mut self,
        driver: u32,
        pattern: u32,
        pattern_len: u64,
        make_relative: u32,
        separate_unit: u32,
    ) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_driver_process_command_files",
            &[
                Val::I32(driver as i32),
                Val::I32(pattern as i32),
                Val::I64(pattern_len as i64),
                Val::I32(make_relative as i32),
                Val::I32(separate_unit as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_driver_command_file_metadata_count`.
    pub fn raw_slang_driver_command_file_metadata_count(
        &mut self,
        driver: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_driver_command_file_metadata_count",
            &[Val::I32(driver as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_driver_command_file_metadata_at`.
    pub fn raw_slang_driver_command_file_metadata_at(
        &mut self,
        driver: u32,
        index: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_driver_command_file_metadata_at",
            &[Val::I32(driver as i32), Val::I32(index as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_command_file_metadata_path`.
    pub fn raw_slang_command_file_metadata_path(&mut self, meta: u32) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __r = self.call(
            "slang_command_file_metadata_path",
            &[Val::I32(__sret as i32), Val::I32(meta as i32)],
        );
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_command_file_metadata_define_count`.
    pub fn raw_slang_command_file_metadata_define_count(
        &mut self,
        meta: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_command_file_metadata_define_count",
            &[Val::I32(meta as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_command_file_metadata_define_at`.
    pub fn raw_slang_command_file_metadata_define_at(
        &mut self,
        meta: u32,
        index: u32,
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __r = self.call(
            "slang_command_file_metadata_define_at",
            &[
                Val::I32(__sret as i32),
                Val::I32(meta as i32),
                Val::I32(index as i32),
            ],
        );
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_driver_optionally_write_dep_files`.
    pub fn raw_slang_driver_optionally_write_dep_files(
        &mut self,
        driver: u32,
    ) -> Result<(), Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_driver_optionally_write_dep_files",
            &[Val::I32(driver as i32), Val::I32(__err as i32)],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_driver_diag_engine`.
    pub fn raw_slang_driver_diag_engine(&mut self, driver: u32) -> Result<u32, Error> {
        let __r = self.call("slang_driver_diag_engine", &[Val::I32(driver as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_diag_engine_num_errors`.
    pub fn raw_slang_diag_engine_num_errors(&mut self, engine: u32) -> Result<u32, Error> {
        let __r = self.call("slang_diag_engine_num_errors", &[Val::I32(engine as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_diag_engine_num_warnings`.
    pub fn raw_slang_diag_engine_num_warnings(&mut self, engine: u32) -> Result<u32, Error> {
        let __r = self.call("slang_diag_engine_num_warnings", &[Val::I32(engine as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_driver_get_analysis_options`.
    pub fn raw_slang_driver_get_analysis_options(&mut self, driver: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_driver_get_analysis_options",
            &[Val::I32(driver as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_driver_run_preprocessor`.
    pub fn raw_slang_driver_run_preprocessor(
        &mut self,
        driver: u32,
        flags: u32,
    ) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_driver_run_preprocessor",
            &[
                Val::I32(driver as i32),
                Val::I32(flags as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_driver_report_macros`.
    pub fn raw_slang_driver_report_macros(
        &mut self,
        driver: u32,
        group_by_file: u32,
    ) -> Result<(), Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_driver_report_macros",
            &[
                Val::I32(driver as i32),
                Val::I32(group_by_file as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_driver_report_parse_diags`.
    pub fn raw_slang_driver_report_parse_diags(&mut self, driver: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_driver_report_parse_diags",
            &[Val::I32(driver as i32), Val::I32(__err as i32)],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_driver_run_full_compilation`.
    pub fn raw_slang_driver_run_full_compilation(
        &mut self,
        driver: u32,
        quiet: u32,
    ) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_driver_run_full_compilation",
            &[
                Val::I32(driver as i32),
                Val::I32(quiet as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_driver_set_terminal_colors_enabled`.
    pub fn raw_slang_driver_set_terminal_colors_enabled(
        &mut self,
        driver: u32,
        enable: u32,
    ) -> Result<(), Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_driver_set_terminal_colors_enabled",
            &[
                Val::I32(driver as i32),
                Val::I32(enable as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_driver_source_loader`.
    pub fn raw_slang_driver_source_loader(&mut self, driver: u32) -> Result<u32, Error> {
        let __r = self.call("slang_driver_source_loader", &[Val::I32(driver as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_source_loader_add_files`.
    pub fn raw_slang_source_loader_add_files(
        &mut self,
        loader: u32,
        pattern: u32,
        pattern_len: u64,
    ) -> Result<(), Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_source_loader_add_files",
            &[
                Val::I32(loader as i32),
                Val::I32(pattern as i32),
                Val::I64(pattern_len as i64),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_source_loader_add_library_files`.
    pub fn raw_slang_source_loader_add_library_files(
        &mut self,
        loader: u32,
        library_name: u32,
        library_name_len: u64,
        pattern: u32,
        pattern_len: u64,
    ) -> Result<(), Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_source_loader_add_library_files",
            &[
                Val::I32(loader as i32),
                Val::I32(library_name as i32),
                Val::I64(library_name_len as i64),
                Val::I32(pattern as i32),
                Val::I64(pattern_len as i64),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_source_loader_add_library_maps`.
    pub fn raw_slang_source_loader_add_library_maps(
        &mut self,
        loader: u32,
        pattern: u32,
        pattern_len: u64,
        base_path: u32,
        base_path_len: u64,
        options: u32,
    ) -> Result<(), Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_source_loader_add_library_maps",
            &[
                Val::I32(loader as i32),
                Val::I32(pattern as i32),
                Val::I64(pattern_len as i64),
                Val::I32(base_path as i32),
                Val::I64(base_path_len as i64),
                Val::I32(options as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_source_loader_add_search_directories`.
    pub fn raw_slang_source_loader_add_search_directories(
        &mut self,
        loader: u32,
        pattern: u32,
        pattern_len: u64,
    ) -> Result<(), Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_source_loader_add_search_directories",
            &[
                Val::I32(loader as i32),
                Val::I32(pattern as i32),
                Val::I64(pattern_len as i64),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_source_loader_add_search_extension`.
    pub fn raw_slang_source_loader_add_search_extension(
        &mut self,
        loader: u32,
        extension: u32,
        extension_len: u64,
    ) -> Result<(), Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_source_loader_add_search_extension",
            &[
                Val::I32(loader as i32),
                Val::I32(extension as i32),
                Val::I64(extension_len as i64),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_source_loader_add_separate_unit`.
    pub fn raw_slang_source_loader_add_separate_unit(
        &mut self,
        loader: u32,
        file_patterns: u32,
        file_patterns_count: u64,
        include_paths: u32,
        include_paths_count: u64,
        defines: u32,
        defines_count: u64,
        library_name: u32,
        library_name_len: u64,
        warning_options: u32,
        warning_options_count: u64,
    ) -> Result<(), Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_source_loader_add_separate_unit",
            &[
                Val::I32(loader as i32),
                Val::I32(file_patterns as i32),
                Val::I64(file_patterns_count as i64),
                Val::I32(include_paths as i32),
                Val::I64(include_paths_count as i64),
                Val::I32(defines as i32),
                Val::I64(defines_count as i64),
                Val::I32(library_name as i32),
                Val::I64(library_name_len as i64),
                Val::I32(warning_options as i32),
                Val::I64(warning_options_count as i64),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_source_loader_has_files`.
    pub fn raw_slang_source_loader_has_files(&mut self, loader: u32) -> Result<u32, Error> {
        let __r = self.call("slang_source_loader_has_files", &[Val::I32(loader as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_source_loader_error_count`.
    pub fn raw_slang_source_loader_error_count(&mut self, loader: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_source_loader_error_count",
            &[Val::I32(loader as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_source_loader_error_at`.
    pub fn raw_slang_source_loader_error_at(
        &mut self,
        loader: u32,
        index: u32,
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __r = self.call(
            "slang_source_loader_error_at",
            &[
                Val::I32(__sret as i32),
                Val::I32(loader as i32),
                Val::I32(index as i32),
            ],
        );
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_source_loader_library_map_count`.
    pub fn raw_slang_source_loader_library_map_count(&mut self, loader: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_source_loader_library_map_count",
            &[Val::I32(loader as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_source_loader_library_map_at`.
    pub fn raw_slang_source_loader_library_map_at(
        &mut self,
        loader: u32,
        index: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_source_loader_library_map_at",
            &[Val::I32(loader as i32), Val::I32(index as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_source_loader_load_sources`.
    pub fn raw_slang_source_loader_load_sources(&mut self, loader: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_source_loader_load_sources",
            &[Val::I32(loader as i32), Val::I32(__err as i32)],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_source_loader_loaded_buffer_id`.
    pub fn raw_slang_source_loader_loaded_buffer_id(
        &mut self,
        loader: u32,
        index: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_source_loader_loaded_buffer_id",
            &[Val::I32(loader as i32), Val::I32(index as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_source_loader_loaded_buffer_text`.
    pub fn raw_slang_source_loader_loaded_buffer_text(
        &mut self,
        loader: u32,
        index: u32,
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __r = self.call(
            "slang_source_loader_loaded_buffer_text",
            &[
                Val::I32(__sret as i32),
                Val::I32(loader as i32),
                Val::I32(index as i32),
            ],
        );
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_source_options_create`.
    pub fn raw_slang_source_options_create(&mut self) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call("slang_source_options_create", &[Val::I32(__err as i32)]);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_source_options_destroy`.
    pub fn raw_slang_source_options_destroy(&mut self, options: u32) -> Result<(), Error> {
        let __r = self.call("slang_source_options_destroy", &[Val::I32(options as i32)]);
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_source_options_set_num_threads`.
    pub fn raw_slang_source_options_set_num_threads(
        &mut self,
        options: u32,
        has_value: u32,
        value: u32,
    ) -> Result<(), Error> {
        let __r = self.call(
            "slang_source_options_set_num_threads",
            &[
                Val::I32(options as i32),
                Val::I32(has_value as i32),
                Val::I32(value as i32),
            ],
        );
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_source_options_set_single_unit`.
    pub fn raw_slang_source_options_set_single_unit(
        &mut self,
        options: u32,
        value: u32,
    ) -> Result<(), Error> {
        let __r = self.call(
            "slang_source_options_set_single_unit",
            &[Val::I32(options as i32), Val::I32(value as i32)],
        );
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_source_options_single_unit`.
    pub fn raw_slang_source_options_single_unit(&mut self, options: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_source_options_single_unit",
            &[Val::I32(options as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_source_options_set_only_lint`.
    pub fn raw_slang_source_options_set_only_lint(
        &mut self,
        options: u32,
        value: u32,
    ) -> Result<(), Error> {
        let __r = self.call(
            "slang_source_options_set_only_lint",
            &[Val::I32(options as i32), Val::I32(value as i32)],
        );
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_source_options_only_lint`.
    pub fn raw_slang_source_options_only_lint(&mut self, options: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_source_options_only_lint",
            &[Val::I32(options as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_source_options_set_libraries_inherit_macros`.
    pub fn raw_slang_source_options_set_libraries_inherit_macros(
        &mut self,
        options: u32,
        value: u32,
    ) -> Result<(), Error> {
        let __r = self.call(
            "slang_source_options_set_libraries_inherit_macros",
            &[Val::I32(options as i32), Val::I32(value as i32)],
        );
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_source_options_libraries_inherit_macros`.
    pub fn raw_slang_source_options_libraries_inherit_macros(
        &mut self,
        options: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_source_options_libraries_inherit_macros",
            &[Val::I32(options as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_driver_text_diag_client`.
    pub fn raw_slang_driver_text_diag_client(&mut self, driver: u32) -> Result<u32, Error> {
        let __r = self.call("slang_driver_text_diag_client", &[Val::I32(driver as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_text_diag_client_get_string`.
    pub fn raw_slang_text_diag_client_get_string(&mut self, client: u32) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_text_diag_client_get_string",
            &[
                Val::I32(__sret as i32),
                Val::I32(client as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_text_diag_client_empty`.
    pub fn raw_slang_text_diag_client_empty(&mut self, client: u32) -> Result<u32, Error> {
        let __r = self.call("slang_text_diag_client_empty", &[Val::I32(client as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_analysis_run`.
    pub fn raw_slang_analysis_run(
        &mut self,
        comp: u32,
        flags: u32,
        threads: u32,
    ) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_analysis_run",
            &[
                Val::I32(comp as i32),
                Val::I32(flags as i32),
                Val::I32(threads as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_analysis_run_listening`.
    pub fn raw_slang_analysis_run_listening(
        &mut self,
        comp: u32,
        flags: u32,
        threads: u32,
        listeners: u32,
    ) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_analysis_run_listening",
            &[
                Val::I32(comp as i32),
                Val::I32(flags as i32),
                Val::I32(threads as i32),
                Val::I32(listeners as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_driver_run_analysis`.
    pub fn raw_slang_driver_run_analysis(&mut self, driver: u32, comp: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_driver_run_analysis",
            &[
                Val::I32(driver as i32),
                Val::I32(comp as i32),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_analysis_destroy`.
    pub fn raw_slang_analysis_destroy(&mut self, analysis: u32) -> Result<(), Error> {
        let __r = self.call("slang_analysis_destroy", &[Val::I32(analysis as i32)]);
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_analysis_diagnostics`.
    pub fn raw_slang_analysis_diagnostics(&mut self, analysis: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_analysis_diagnostics",
            &[Val::I32(analysis as i32), Val::I32(__err as i32)],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_analysis_scope_procedure_count`.
    pub fn raw_slang_analysis_scope_procedure_count(
        &mut self,
        analysis: u32,
        scope: &[u8],
    ) -> Result<u32, Error> {
        let __p_scope = self.malloc(16)?;
        self.write(__p_scope, &scope[..16])?;
        let __r = self.call(
            "slang_analysis_scope_procedure_count",
            &[Val::I32(analysis as i32), Val::I32(__p_scope as i32)],
        );
        self.free(__p_scope);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_analysis_scope_procedure`.
    pub fn raw_slang_analysis_scope_procedure(
        &mut self,
        analysis: u32,
        scope: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_scope = self.malloc(16)?;
        self.write(__p_scope, &scope[..16])?;
        let __r = self.call(
            "slang_analysis_scope_procedure",
            &[
                Val::I32(__sret as i32),
                Val::I32(analysis as i32),
                Val::I32(__p_scope as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_scope);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_analysis_procedure_has_clock`.
    pub fn raw_slang_analysis_procedure_has_clock(
        &mut self,
        analysis: u32,
        scope: &[u8],
        index: u32,
    ) -> Result<u32, Error> {
        let __p_scope = self.malloc(16)?;
        self.write(__p_scope, &scope[..16])?;
        let __r = self.call(
            "slang_analysis_procedure_has_clock",
            &[
                Val::I32(analysis as i32),
                Val::I32(__p_scope as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_scope);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_analysis_driver_count`.
    pub fn raw_slang_analysis_driver_count(
        &mut self,
        analysis: u32,
        value: &[u8],
    ) -> Result<u32, Error> {
        let __p_value = self.malloc(16)?;
        self.write(__p_value, &value[..16])?;
        let __r = self.call(
            "slang_analysis_driver_count",
            &[Val::I32(analysis as i32), Val::I32(__p_value as i32)],
        );
        self.free(__p_value);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_analysis_driver_handle`.
    pub fn raw_slang_analysis_driver_handle(
        &mut self,
        analysis: u32,
        value: &[u8],
        index: u32,
    ) -> Result<u32, Error> {
        let __p_value = self.malloc(16)?;
        self.write(__p_value, &value[..16])?;
        let __r = self.call(
            "slang_analysis_driver_handle",
            &[
                Val::I32(analysis as i32),
                Val::I32(__p_value as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_value);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_value_driver_flags`.
    pub fn raw_slang_value_driver_flags(&mut self, driver: u32) -> Result<u32, Error> {
        let __r = self.call("slang_value_driver_flags", &[Val::I32(driver as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_value_driver_symbol`.
    pub fn raw_slang_value_driver_symbol(&mut self, driver: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_value_driver_symbol",
            &[Val::I32(__sret as i32), Val::I32(driver as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_value_driver_bounds`.
    pub fn raw_slang_value_driver_bounds(
        &mut self,
        driver: u32,
        lo: u32,
        hi: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_value_driver_bounds",
            &[
                Val::I32(driver as i32),
                Val::I32(lo as i32),
                Val::I32(hi as i32),
            ],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_value_driver_source_range`.
    pub fn raw_slang_value_driver_source_range(&mut self, driver: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(32)?;
        let __r = self.call(
            "slang_value_driver_source_range",
            &[Val::I32(__sret as i32), Val::I32(driver as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(8);
        for __i in 0..8 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_value_driver_is_unidirectional_port`.
    pub fn raw_slang_value_driver_is_unidirectional_port(
        &mut self,
        driver: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_value_driver_is_unidirectional_port",
            &[Val::I32(driver as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_value_driver_is_in_single_driver_procedure`.
    pub fn raw_slang_value_driver_is_in_single_driver_procedure(
        &mut self,
        driver: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_value_driver_is_in_single_driver_procedure",
            &[Val::I32(driver as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_value_driver_source`.
    pub fn raw_slang_value_driver_source(&mut self, driver: u32) -> Result<u32, Error> {
        let __r = self.call("slang_value_driver_source", &[Val::I32(driver as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_value_driver_path`.
    pub fn raw_slang_value_driver_path(&mut self, driver: u32) -> Result<u32, Error> {
        let __r = self.call("slang_value_driver_path", &[Val::I32(driver as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_value_path_root_symbol`.
    pub fn raw_slang_value_path_root_symbol(&mut self, path: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_value_path_root_symbol",
            &[Val::I32(__sret as i32), Val::I32(path as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_analysis_scope_procedure_handle`.
    pub fn raw_slang_analysis_scope_procedure_handle(
        &mut self,
        analysis: u32,
        scope: &[u8],
        index: u32,
    ) -> Result<u32, Error> {
        let __p_scope = self.malloc(16)?;
        self.write(__p_scope, &scope[..16])?;
        let __r = self.call(
            "slang_analysis_scope_procedure_handle",
            &[
                Val::I32(analysis as i32),
                Val::I32(__p_scope as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_scope);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_analysis_assertion_count`.
    pub fn raw_slang_analysis_assertion_count(
        &mut self,
        analysis: u32,
        containing_symbol: &[u8],
    ) -> Result<u32, Error> {
        let __p_containing_symbol = self.malloc(16)?;
        self.write(__p_containing_symbol, &containing_symbol[..16])?;
        let __r = self.call(
            "slang_analysis_assertion_count",
            &[
                Val::I32(analysis as i32),
                Val::I32(__p_containing_symbol as i32),
            ],
        );
        self.free(__p_containing_symbol);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_analysis_assertion_at`.
    pub fn raw_slang_analysis_assertion_at(
        &mut self,
        analysis: u32,
        containing_symbol: &[u8],
        index: u32,
    ) -> Result<u32, Error> {
        let __p_containing_symbol = self.malloc(16)?;
        self.write(__p_containing_symbol, &containing_symbol[..16])?;
        let __r = self.call(
            "slang_analysis_assertion_at",
            &[
                Val::I32(analysis as i32),
                Val::I32(__p_containing_symbol as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_containing_symbol);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_analyzed_assertion_containing_symbol`.
    pub fn raw_slang_analyzed_assertion_containing_symbol(
        &mut self,
        assertion: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_analyzed_assertion_containing_symbol",
            &[Val::I32(__sret as i32), Val::I32(assertion as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_analyzed_assertion_procedure`.
    pub fn raw_slang_analyzed_assertion_procedure(&mut self, assertion: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_analyzed_assertion_procedure",
            &[Val::I32(assertion as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_analyzed_assertion_ast_node`.
    pub fn raw_slang_analyzed_assertion_ast_node(
        &mut self,
        assertion: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_analyzed_assertion_ast_node",
            &[Val::I32(__sret as i32), Val::I32(assertion as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_analyzed_assertion_root`.
    pub fn raw_slang_analyzed_assertion_root(&mut self, assertion: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_analyzed_assertion_root",
            &[Val::I32(__sret as i32), Val::I32(assertion as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_analyzed_assertion_semantic_leading_clock`.
    pub fn raw_slang_analyzed_assertion_semantic_leading_clock(
        &mut self,
        assertion: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_analyzed_assertion_semantic_leading_clock",
            &[Val::I32(__sret as i32), Val::I32(assertion as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_analyzed_assertion_clock`.
    pub fn raw_slang_analyzed_assertion_clock(
        &mut self,
        assertion: u32,
        expr: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_expr = self.malloc(16)?;
        self.write(__p_expr, &expr[..16])?;
        let __r = self.call(
            "slang_analyzed_assertion_clock",
            &[
                Val::I32(__sret as i32),
                Val::I32(assertion as i32),
                Val::I32(__p_expr as i32),
            ],
        );
        self.free(__p_expr);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_analyzed_procedure_symbol`.
    pub fn raw_slang_analyzed_procedure_symbol(
        &mut self,
        procedure: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_analyzed_procedure_symbol",
            &[Val::I32(__sret as i32), Val::I32(procedure as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_analyzed_procedure_parent`.
    pub fn raw_slang_analyzed_procedure_parent(&mut self, procedure: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_analyzed_procedure_parent",
            &[Val::I32(procedure as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_analyzed_procedure_inferred_clock`.
    pub fn raw_slang_analyzed_procedure_inferred_clock(
        &mut self,
        procedure: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_analyzed_procedure_inferred_clock",
            &[Val::I32(__sret as i32), Val::I32(procedure as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_analyzed_procedure_driver_count`.
    pub fn raw_slang_analyzed_procedure_driver_count(
        &mut self,
        procedure: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_analyzed_procedure_driver_count",
            &[Val::I32(procedure as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_analyzed_procedure_call_expression_count`.
    pub fn raw_slang_analyzed_procedure_call_expression_count(
        &mut self,
        procedure: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_analyzed_procedure_call_expression_count",
            &[Val::I32(procedure as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_analyzed_procedure_call_expression_at`.
    pub fn raw_slang_analyzed_procedure_call_expression_at(
        &mut self,
        procedure: u32,
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_analyzed_procedure_call_expression_at",
            &[
                Val::I32(__sret as i32),
                Val::I32(procedure as i32),
                Val::I32(index as i32),
            ],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_analyzed_procedure_timing_control_count`.
    pub fn raw_slang_analyzed_procedure_timing_control_count(
        &mut self,
        procedure: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_analyzed_procedure_timing_control_count",
            &[Val::I32(procedure as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_analyzed_procedure_timing_control_at`.
    pub fn raw_slang_analyzed_procedure_timing_control_at(
        &mut self,
        procedure: u32,
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_analyzed_procedure_timing_control_at",
            &[
                Val::I32(__sret as i32),
                Val::I32(procedure as i32),
                Val::I32(index as i32),
            ],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_analyzed_procedure_read_set_count`.
    pub fn raw_slang_analyzed_procedure_read_set_count(
        &mut self,
        procedure: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_analyzed_procedure_read_set_count",
            &[Val::I32(procedure as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_analyzed_procedure_read_set_at`.
    pub fn raw_slang_analyzed_procedure_read_set_at(
        &mut self,
        procedure: u32,
        index: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_analyzed_procedure_read_set_at",
            &[Val::I32(procedure as i32), Val::I32(index as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_analyzed_procedure_implicit_event_read_set_count`.
    pub fn raw_slang_analyzed_procedure_implicit_event_read_set_count(
        &mut self,
        procedure: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_analyzed_procedure_implicit_event_read_set_count",
            &[Val::I32(procedure as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_analyzed_procedure_implicit_event_read_set_at`.
    pub fn raw_slang_analyzed_procedure_implicit_event_read_set_at(
        &mut self,
        procedure: u32,
        index: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_analyzed_procedure_implicit_event_read_set_at",
            &[Val::I32(procedure as i32), Val::I32(index as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_analyzed_procedure_sensitivity_list`.
    pub fn raw_slang_analyzed_procedure_sensitivity_list(
        &mut self,
        procedure: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_analyzed_procedure_sensitivity_list",
            &[Val::I32(procedure as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_implicit_event_read_set_statement`.
    pub fn raw_slang_implicit_event_read_set_statement(
        &mut self,
        read_set: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_implicit_event_read_set_statement",
            &[Val::I32(__sret as i32), Val::I32(read_set as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_implicit_event_read_set_read_count`.
    pub fn raw_slang_implicit_event_read_set_read_count(
        &mut self,
        read_set: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_implicit_event_read_set_read_count",
            &[Val::I32(read_set as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_implicit_event_read_set_read_at`.
    pub fn raw_slang_implicit_event_read_set_read_at(
        &mut self,
        read_set: u32,
        index: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_implicit_event_read_set_read_at",
            &[Val::I32(read_set as i32), Val::I32(index as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_read_range_symbol`.
    pub fn raw_slang_read_range_symbol(&mut self, range: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_read_range_symbol",
            &[Val::I32(__sret as i32), Val::I32(range as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_read_range_bit_range`.
    pub fn raw_slang_read_range_bit_range(
        &mut self,
        range: u32,
        lo: u32,
        hi: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_read_range_bit_range",
            &[
                Val::I32(range as i32),
                Val::I32(lo as i32),
                Val::I32(hi as i32),
            ],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_sensitivity_list_kind`.
    pub fn raw_slang_sensitivity_list_kind(&mut self, list: u32) -> Result<u32, Error> {
        let __r = self.call("slang_sensitivity_list_kind", &[Val::I32(list as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_sensitivity_list_timing_control`.
    pub fn raw_slang_sensitivity_list_timing_control(
        &mut self,
        list: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_sensitivity_list_timing_control",
            &[Val::I32(__sret as i32), Val::I32(list as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_sensitivity_list_read_count`.
    pub fn raw_slang_sensitivity_list_read_count(&mut self, list: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_sensitivity_list_read_count",
            &[Val::I32(list as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_sensitivity_list_read_at`.
    pub fn raw_slang_sensitivity_list_read_at(
        &mut self,
        list: u32,
        index: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_sensitivity_list_read_at",
            &[Val::I32(list as i32), Val::I32(index as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_dfa_run`.
    pub fn raw_slang_dfa_run(
        &mut self,
        comp: u32,
        procedure: &[u8],
        lattice: u32,
        user: u32,
    ) -> Result<u32, Error> {
        let __p_procedure = self.malloc(16)?;
        self.write(__p_procedure, &procedure[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_dfa_run",
            &[
                Val::I32(comp as i32),
                Val::I32(__p_procedure as i32),
                Val::I32(lattice as i32),
                Val::I32(user as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_procedure);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_dfa_ctx_state`.
    pub fn raw_slang_dfa_ctx_state(&mut self, ctx: u32) -> Result<u32, Error> {
        let __r = self.call("slang_dfa_ctx_state", &[Val::I32(ctx as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_dfa_ctx_is_bad`.
    pub fn raw_slang_dfa_ctx_is_bad(&mut self, ctx: u32) -> Result<u32, Error> {
        let __r = self.call("slang_dfa_ctx_is_bad", &[Val::I32(ctx as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_dfa_ctx_eval_context`.
    pub fn raw_slang_dfa_ctx_eval_context(&mut self, ctx: u32) -> Result<u32, Error> {
        let __r = self.call("slang_dfa_ctx_eval_context", &[Val::I32(ctx as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_eval_ctx_evaluate`.
    pub fn raw_slang_eval_ctx_evaluate(&mut self, ectx: u32, expr: &[u8]) -> Result<u32, Error> {
        let __p_expr = self.malloc(16)?;
        self.write(__p_expr, &expr[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_eval_ctx_evaluate",
            &[
                Val::I32(ectx as i32),
                Val::I32(__p_expr as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_expr);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_script_session_create`.
    pub fn raw_slang_script_session_create(&mut self, options: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_script_session_create",
            &[Val::I32(options as i32), Val::I32(__err as i32)],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_script_session_destroy`.
    pub fn raw_slang_script_session_destroy(&mut self, session: u32) -> Result<(), Error> {
        let __r = self.call("slang_script_session_destroy", &[Val::I32(session as i32)]);
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_script_session_eval`.
    pub fn raw_slang_script_session_eval(
        &mut self,
        session: u32,
        text: u32,
        text_len: u64,
    ) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_script_session_eval",
            &[
                Val::I32(session as i32),
                Val::I32(text as i32),
                Val::I64(text_len as i64),
                Val::I32(__err as i32),
            ],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_script_session_eval_expression`.
    pub fn raw_slang_script_session_eval_expression(
        &mut self,
        session: u32,
        expr: &[u8],
    ) -> Result<u32, Error> {
        let __p_expr = self.malloc(16)?;
        self.write(__p_expr, &expr[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_script_session_eval_expression",
            &[
                Val::I32(session as i32),
                Val::I32(__p_expr as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_expr);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_script_session_eval_statement`.
    pub fn raw_slang_script_session_eval_statement(
        &mut self,
        session: u32,
        stmt: &[u8],
    ) -> Result<(), Error> {
        let __p_stmt = self.malloc(16)?;
        self.write(__p_stmt, &stmt[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_script_session_eval_statement",
            &[
                Val::I32(session as i32),
                Val::I32(__p_stmt as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_stmt);
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        Ok(())
    }
    /// Raw marshalling for `slang_script_session_compilation`.
    pub fn raw_slang_script_session_compilation(&mut self, session: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_script_session_compilation",
            &[Val::I32(session as i32), Val::I32(__err as i32)],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_script_session_diagnostics`.
    pub fn raw_slang_script_session_diagnostics(&mut self, session: u32) -> Result<u32, Error> {
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_script_session_diagnostics",
            &[Val::I32(session as i32), Val::I32(__err as i32)],
        );
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_method_prototype_flags`.
    pub fn raw_slang_symbol_method_prototype_flags(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_method_prototype_flags",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_method_prototype_subroutine_kind`.
    pub fn raw_slang_symbol_method_prototype_subroutine_kind(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_method_prototype_subroutine_kind",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_method_prototype_visibility`.
    pub fn raw_slang_symbol_method_prototype_visibility(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_method_prototype_visibility",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_method_prototype_is_virtual`.
    pub fn raw_slang_symbol_method_prototype_is_virtual(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_method_prototype_is_virtual",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_method_prototype_argument_count`.
    pub fn raw_slang_symbol_method_prototype_argument_count(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_method_prototype_argument_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_method_prototype_argument`.
    pub fn raw_slang_symbol_method_prototype_argument(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_method_prototype_argument",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_method_prototype_return_type`.
    pub fn raw_slang_symbol_method_prototype_return_type(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_method_prototype_return_type",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_method_prototype_subroutine`.
    pub fn raw_slang_symbol_method_prototype_subroutine(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_method_prototype_subroutine",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_method_prototype_override`.
    pub fn raw_slang_symbol_method_prototype_override(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_method_prototype_override",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_method_prototype_first_extern_impl`.
    pub fn raw_slang_symbol_method_prototype_first_extern_impl(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_method_prototype_first_extern_impl",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_modport_clocking_target`.
    pub fn raw_slang_symbol_modport_clocking_target(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_modport_clocking_target",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_interface_port_connection`.
    pub fn raw_slang_symbol_interface_port_connection(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_interface_port_connection",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_instance_port_connection_iface_conn`.
    pub fn raw_slang_instance_port_connection_iface_conn(
        &mut self,
        instance: &[u8],
        index: u32,
    ) -> Result<u32, Error> {
        let __p_instance = self.malloc(16)?;
        self.write(__p_instance, &instance[..16])?;
        let __r = self.call(
            "slang_instance_port_connection_iface_conn",
            &[Val::I32(__p_instance as i32), Val::I32(index as i32)],
        );
        self.free(__p_instance);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_interface_port_declared_range_count`.
    pub fn raw_slang_symbol_interface_port_declared_range_count(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_interface_port_declared_range_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_interface_port_declared_range_at`.
    pub fn raw_slang_symbol_interface_port_declared_range_at(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_interface_port_declared_range_at",
            &[Val::I32(__p_sym as i32), Val::I32(index as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_interface_port_interface_def`.
    pub fn raw_slang_symbol_interface_port_interface_def(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_interface_port_interface_def",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_interface_port_is_generic`.
    pub fn raw_slang_symbol_interface_port_is_generic(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_interface_port_is_generic",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_interface_port_is_invalid`.
    pub fn raw_slang_symbol_interface_port_is_invalid(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_interface_port_is_invalid",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_interface_port_modport`.
    pub fn raw_slang_symbol_interface_port_modport(&mut self, sym: &[u8]) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_interface_port_modport",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_symbol_let_decl_port_count`.
    pub fn raw_slang_symbol_let_decl_port_count(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_let_decl_port_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_let_decl_port`.
    pub fn raw_slang_symbol_let_decl_port(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_let_decl_port",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_lookup_get_visibility`.
    pub fn raw_slang_lookup_get_visibility(&mut self, symbol: &[u8]) -> Result<u32, Error> {
        let __p_symbol = self.malloc(16)?;
        self.write(__p_symbol, &symbol[..16])?;
        let __r = self.call(
            "slang_lookup_get_visibility",
            &[Val::I32(__p_symbol as i32)],
        );
        self.free(__p_symbol);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lookup_is_visible_from`.
    pub fn raw_slang_lookup_is_visible_from(
        &mut self,
        symbol: &[u8],
        scope: &[u8],
    ) -> Result<u32, Error> {
        let __p_symbol = self.malloc(16)?;
        self.write(__p_symbol, &symbol[..16])?;
        let __p_scope = self.malloc(16)?;
        self.write(__p_scope, &scope[..16])?;
        let __r = self.call(
            "slang_lookup_is_visible_from",
            &[Val::I32(__p_symbol as i32), Val::I32(__p_scope as i32)],
        );
        self.free(__p_symbol);
        self.free(__p_scope);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lookup_is_accessible_from`.
    pub fn raw_slang_lookup_is_accessible_from(
        &mut self,
        target: &[u8],
        source_scope: &[u8],
    ) -> Result<u32, Error> {
        let __p_target = self.malloc(16)?;
        self.write(__p_target, &target[..16])?;
        let __p_source_scope = self.malloc(16)?;
        self.write(__p_source_scope, &source_scope[..16])?;
        let __r = self.call(
            "slang_lookup_is_accessible_from",
            &[
                Val::I32(__p_target as i32),
                Val::I32(__p_source_scope as i32),
            ],
        );
        self.free(__p_target);
        self.free(__p_source_scope);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lookup_ensure_visible`.
    pub fn raw_slang_lookup_ensure_visible(
        &mut self,
        symbol: &[u8],
        context_scope: &[u8],
    ) -> Result<u32, Error> {
        let __p_symbol = self.malloc(16)?;
        self.write(__p_symbol, &symbol[..16])?;
        let __p_context_scope = self.malloc(16)?;
        self.write(__p_context_scope, &context_scope[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_lookup_ensure_visible",
            &[
                Val::I32(__p_symbol as i32),
                Val::I32(__p_context_scope as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_symbol);
        self.free(__p_context_scope);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lookup_ensure_accessible`.
    pub fn raw_slang_lookup_ensure_accessible(
        &mut self,
        symbol: &[u8],
        context_scope: &[u8],
    ) -> Result<u32, Error> {
        let __p_symbol = self.malloc(16)?;
        self.write(__p_symbol, &symbol[..16])?;
        let __p_context_scope = self.malloc(16)?;
        self.write(__p_context_scope, &context_scope[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_lookup_ensure_accessible",
            &[
                Val::I32(__p_symbol as i32),
                Val::I32(__p_context_scope as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_symbol);
        self.free(__p_context_scope);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lookup_name`.
    pub fn raw_slang_lookup_name(
        &mut self,
        context_scope: &[u8],
        name: u32,
        name_len: u64,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_context_scope = self.malloc(16)?;
        self.write(__p_context_scope, &context_scope[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_lookup_name",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_context_scope as i32),
                Val::I32(name as i32),
                Val::I64(name_len as i64),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_context_scope);
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_lookup_find_class`.
    pub fn raw_slang_lookup_find_class(
        &mut self,
        context_scope: &[u8],
        name: u32,
        name_len: u64,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_context_scope = self.malloc(16)?;
        self.write(__p_context_scope, &context_scope[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_lookup_find_class",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_context_scope as i32),
                Val::I32(name as i32),
                Val::I64(name_len as i64),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_context_scope);
        let __chk = self.check_err(__err);
        self.free(__err);
        __r?;
        __chk?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_lookup_find_assertion_local_var`.
    pub fn raw_slang_lookup_find_assertion_local_var(
        &mut self,
        assertion_inst: &[u8],
        context_scope: &[u8],
        name: u32,
        name_len: u64,
        out_symbol: u32,
    ) -> Result<u32, Error> {
        let __p_assertion_inst = self.malloc(16)?;
        self.write(__p_assertion_inst, &assertion_inst[..16])?;
        let __p_context_scope = self.malloc(16)?;
        self.write(__p_context_scope, &context_scope[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_lookup_find_assertion_local_var",
            &[
                Val::I32(__p_assertion_inst as i32),
                Val::I32(__p_context_scope as i32),
                Val::I32(name as i32),
                Val::I64(name_len as i64),
                Val::I32(out_symbol as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_assertion_inst);
        self.free(__p_context_scope);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lookup_find_temp_var`.
    pub fn raw_slang_lookup_find_temp_var(
        &mut self,
        temp_var: &[u8],
        context_scope: &[u8],
        name: u32,
        name_len: u64,
        out_symbol: u32,
    ) -> Result<u32, Error> {
        let __p_temp_var = self.malloc(16)?;
        self.write(__p_temp_var, &temp_var[..16])?;
        let __p_context_scope = self.malloc(16)?;
        self.write(__p_context_scope, &context_scope[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_lookup_find_temp_var",
            &[
                Val::I32(__p_temp_var as i32),
                Val::I32(__p_context_scope as i32),
                Val::I32(name as i32),
                Val::I64(name_len as i64),
                Val::I32(out_symbol as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_temp_var);
        self.free(__p_context_scope);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lookup_location_before`.
    pub fn raw_slang_lookup_location_before(&mut self, symbol: &[u8]) -> Result<u32, Error> {
        let __p_symbol = self.malloc(16)?;
        self.write(__p_symbol, &symbol[..16])?;
        let __r = self.call(
            "slang_lookup_location_before",
            &[Val::I32(__p_symbol as i32)],
        );
        self.free(__p_symbol);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lookup_location_after`.
    pub fn raw_slang_lookup_location_after(&mut self, symbol: &[u8]) -> Result<u32, Error> {
        let __p_symbol = self.malloc(16)?;
        self.write(__p_symbol, &symbol[..16])?;
        let __r = self.call(
            "slang_lookup_location_after",
            &[Val::I32(__p_symbol as i32)],
        );
        self.free(__p_symbol);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lookup_location_max`.
    pub fn raw_slang_lookup_location_max(&mut self, comp: u32) -> Result<u32, Error> {
        let __r = self.call("slang_lookup_location_max", &[Val::I32(comp as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lookup_location_min`.
    pub fn raw_slang_lookup_location_min(&mut self, comp: u32) -> Result<u32, Error> {
        let __r = self.call("slang_lookup_location_min", &[Val::I32(comp as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lookup_location_get_scope`.
    pub fn raw_slang_lookup_location_get_scope(&mut self, loc: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_lookup_location_get_scope",
            &[Val::I32(__sret as i32), Val::I32(loc as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_lookup_location_get_index`.
    pub fn raw_slang_lookup_location_get_index(&mut self, loc: u32) -> Result<u32, Error> {
        let __r = self.call("slang_lookup_location_get_index", &[Val::I32(loc as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lookup_result_create`.
    pub fn raw_slang_lookup_result_create(&mut self) -> Result<u32, Error> {
        let __r = self.call("slang_lookup_result_create", &[]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lookup_result_destroy`.
    pub fn raw_slang_lookup_result_destroy(&mut self, result: u32) -> Result<(), Error> {
        let __r = self.call("slang_lookup_result_destroy", &[Val::I32(result as i32)]);
        __r?;
        Ok(())
    }
    /// Raw marshalling for `slang_lookup_within_class_randomize`.
    pub fn raw_slang_lookup_within_class_randomize(
        &mut self,
        class_type: &[u8],
        this_var: &[u8],
        context_scope: &[u8],
        name: u32,
        name_len: u64,
        result: u32,
    ) -> Result<u32, Error> {
        let __p_class_type = self.malloc(16)?;
        self.write(__p_class_type, &class_type[..16])?;
        let __p_this_var = self.malloc(16)?;
        self.write(__p_this_var, &this_var[..16])?;
        let __p_context_scope = self.malloc(16)?;
        self.write(__p_context_scope, &context_scope[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_lookup_within_class_randomize",
            &[
                Val::I32(__p_class_type as i32),
                Val::I32(__p_this_var as i32),
                Val::I32(__p_context_scope as i32),
                Val::I32(name as i32),
                Val::I64(name_len as i64),
                Val::I32(result as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_class_type);
        self.free(__p_this_var);
        self.free(__p_context_scope);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lookup_result_found`.
    pub fn raw_slang_lookup_result_found(&mut self, result: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_lookup_result_found",
            &[Val::I32(__sret as i32), Val::I32(result as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_lookup_result_flags`.
    pub fn raw_slang_lookup_result_flags(&mut self, result: u32) -> Result<u32, Error> {
        let __r = self.call("slang_lookup_result_flags", &[Val::I32(result as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lookup_result_has_error`.
    pub fn raw_slang_lookup_result_has_error(&mut self, result: u32) -> Result<u32, Error> {
        let __r = self.call("slang_lookup_result_has_error", &[Val::I32(result as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lookup_result_system_subroutine`.
    pub fn raw_slang_lookup_result_system_subroutine(&mut self, result: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_lookup_result_system_subroutine",
            &[Val::I32(result as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lookup_result_upward_count`.
    pub fn raw_slang_lookup_result_upward_count(&mut self, result: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_lookup_result_upward_count",
            &[Val::I32(result as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lookup_result_clear`.
    pub fn raw_slang_lookup_result_clear(&mut self, result: u32) -> Result<u32, Error> {
        let __r = self.call("slang_lookup_result_clear", &[Val::I32(result as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lookup_result_error_if_selectors`.
    pub fn raw_slang_lookup_result_error_if_selectors(
        &mut self,
        result: u32,
        context_scope: &[u8],
    ) -> Result<u32, Error> {
        let __p_context_scope = self.malloc(16)?;
        self.write(__p_context_scope, &context_scope[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_lookup_result_error_if_selectors",
            &[
                Val::I32(result as i32),
                Val::I32(__p_context_scope as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_context_scope);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lookup_result_report_diags`.
    pub fn raw_slang_lookup_result_report_diags(
        &mut self,
        result: u32,
        context_scope: &[u8],
    ) -> Result<u32, Error> {
        let __p_context_scope = self.malloc(16)?;
        self.write(__p_context_scope, &context_scope[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_lookup_result_report_diags",
            &[
                Val::I32(result as i32),
                Val::I32(__p_context_scope as i32),
                Val::I32(__err as i32),
            ],
        );
        self.free(__p_context_scope);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lookup_result_diagnostics`.
    pub fn raw_slang_lookup_result_diagnostics(&mut self, result: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_lookup_result_diagnostics",
            &[Val::I32(result as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lookup_result_selector_count`.
    pub fn raw_slang_lookup_result_selector_count(&mut self, result: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_lookup_result_selector_count",
            &[Val::I32(result as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lookup_result_selector_is_member`.
    pub fn raw_slang_lookup_result_selector_is_member(
        &mut self,
        result: u32,
        index: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_lookup_result_selector_is_member",
            &[Val::I32(result as i32), Val::I32(index as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_lookup_result_selector_name`.
    pub fn raw_slang_lookup_result_selector_name(
        &mut self,
        result: u32,
        index: u32,
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __r = self.call(
            "slang_lookup_result_selector_name",
            &[
                Val::I32(__sret as i32),
                Val::I32(result as i32),
                Val::I32(index as i32),
            ],
        );
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_lookup_result_selector_dot_location`.
    pub fn raw_slang_lookup_result_selector_dot_location(
        &mut self,
        result: u32,
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_lookup_result_selector_dot_location",
            &[
                Val::I32(__sret as i32),
                Val::I32(result as i32),
                Val::I32(index as i32),
            ],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_lookup_result_selector_name_range`.
    pub fn raw_slang_lookup_result_selector_name_range(
        &mut self,
        result: u32,
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(32)?;
        let __r = self.call(
            "slang_lookup_result_selector_name_range",
            &[
                Val::I32(__sret as i32),
                Val::I32(result as i32),
                Val::I32(index as i32),
            ],
        );
        __r?;
        let mut __w = Vec::with_capacity(8);
        for __i in 0..8 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_modport_port_direction`.
    pub fn raw_slang_symbol_modport_port_direction(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_modport_port_direction",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_modport_port_explicit_connection`.
    pub fn raw_slang_symbol_modport_port_explicit_connection(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_modport_port_explicit_connection",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_modport_port_internal_symbol`.
    pub fn raw_slang_symbol_modport_port_internal_symbol(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_modport_port_internal_symbol",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_modport_has_exports`.
    pub fn raw_slang_symbol_modport_has_exports(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_modport_has_exports",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_multi_port_direction`.
    pub fn raw_slang_symbol_multi_port_direction(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_multi_port_direction",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_multi_port_initializer`.
    pub fn raw_slang_symbol_multi_port_initializer(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_multi_port_initializer",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_multi_port_type`.
    pub fn raw_slang_symbol_multi_port_type(&mut self, sym: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_multi_port_type",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_multi_port_is_null_port`.
    pub fn raw_slang_symbol_multi_port_is_null_port(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_multi_port_is_null_port",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_multi_port_port_count`.
    pub fn raw_slang_symbol_multi_port_port_count(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_multi_port_port_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_multi_port_port`.
    pub fn raw_slang_symbol_multi_port_port(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_multi_port_port",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_net_alias_reference_count`.
    pub fn raw_slang_symbol_net_alias_reference_count(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_net_alias_reference_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_net_alias_reference`.
    pub fn raw_slang_symbol_net_alias_reference(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_net_alias_reference",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_net_expansion_hint`.
    pub fn raw_slang_symbol_net_expansion_hint(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_net_expansion_hint",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_net_charge_strength`.
    pub fn raw_slang_symbol_net_charge_strength(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_net_charge_strength",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_net_delay`.
    pub fn raw_slang_symbol_net_delay(&mut self, sym: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_net_delay",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_net_drive_strength`.
    pub fn raw_slang_symbol_net_drive_strength(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_net_drive_strength",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_net_is_implicit`.
    pub fn raw_slang_symbol_net_is_implicit(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call("slang_symbol_net_is_implicit", &[Val::I32(__p_sym as i32)]);
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_package_default_lifetime`.
    pub fn raw_slang_symbol_package_default_lifetime(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_package_default_lifetime",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_package_find_for_import`.
    pub fn raw_slang_symbol_package_find_for_import(
        &mut self,
        sym: &[u8],
        name: u32,
        name_len: u64,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_package_find_for_import",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(name as i32),
                Val::I64(name_len as i64),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_package_has_export_all`.
    pub fn raw_slang_symbol_package_has_export_all(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_package_has_export_all",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_parameter_is_local_param`.
    pub fn raw_slang_symbol_parameter_is_local_param(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_parameter_is_local_param",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_parameter_is_port_param`.
    pub fn raw_slang_symbol_parameter_is_port_param(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_parameter_is_port_param",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_parameter_is_body_param`.
    pub fn raw_slang_symbol_parameter_is_body_param(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_parameter_is_body_param",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_parameter_is_overridden`.
    pub fn raw_slang_symbol_parameter_is_overridden(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_parameter_is_overridden",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_type_parameter_is_overridden`.
    pub fn raw_slang_symbol_type_parameter_is_overridden(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_type_parameter_is_overridden",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_port_direction`.
    pub fn raw_slang_symbol_port_direction(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call("slang_symbol_port_direction", &[Val::I32(__p_sym as i32)]);
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_port_external_loc`.
    pub fn raw_slang_symbol_port_external_loc(&mut self, sym: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_port_external_loc",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_port_initializer`.
    pub fn raw_slang_symbol_port_initializer(&mut self, sym: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_port_initializer",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_port_internal_expr`.
    pub fn raw_slang_symbol_port_internal_expr(&mut self, sym: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_port_internal_expr",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_port_type`.
    pub fn raw_slang_symbol_port_type(&mut self, sym: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_port_type",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_port_internal_symbol`.
    pub fn raw_slang_symbol_port_internal_symbol(&mut self, sym: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_port_internal_symbol",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_port_is_ansi_port`.
    pub fn raw_slang_symbol_port_is_ansi_port(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_port_is_ansi_port",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_port_is_net_port`.
    pub fn raw_slang_symbol_port_is_net_port(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call("slang_symbol_port_is_net_port", &[Val::I32(__p_sym as i32)]);
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_port_is_null_port`.
    pub fn raw_slang_symbol_port_is_null_port(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_port_is_null_port",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_primitive_port_direction`.
    pub fn raw_slang_symbol_primitive_port_direction(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_primitive_port_direction",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_primitive_kind`.
    pub fn raw_slang_symbol_primitive_kind(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call("slang_symbol_primitive_kind", &[Val::I32(__p_sym as i32)]);
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_primitive_is_sequential`.
    pub fn raw_slang_symbol_primitive_is_sequential(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_primitive_is_sequential",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_primitive_init_val`.
    pub fn raw_slang_symbol_primitive_init_val(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __err = self.malloc(252)?;
        self.zero(__err, 252)?;
        let __r = self.call(
            "slang_symbol_primitive_init_val",
            &[Val::I32(__p_sym as i32), Val::I32(__err as i32)],
        );
        self.free(__p_sym);
        let __chk = self.check_err(__err);
        self.free(__err);
        let __v = __r?;
        __chk?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_primitive_port_count`.
    pub fn raw_slang_symbol_primitive_port_count(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_primitive_port_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_primitive_port`.
    pub fn raw_slang_symbol_primitive_port(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_primitive_port",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_primitive_table_count`.
    pub fn raw_slang_symbol_primitive_table_count(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_primitive_table_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_primitive_table_entry_inputs`.
    pub fn raw_slang_symbol_primitive_table_entry_inputs(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_primitive_table_entry_inputs",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_symbol_primitive_table_entry_output`.
    pub fn raw_slang_symbol_primitive_table_entry_output(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_primitive_table_entry_output",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_symbol_primitive_table_entry_state`.
    pub fn raw_slang_symbol_primitive_table_entry_state(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_primitive_table_entry_state",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_symbol_primitive_instance_port_connection_count`.
    pub fn raw_slang_symbol_primitive_instance_port_connection_count(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_primitive_instance_port_connection_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_primitive_instance_port_connection`.
    pub fn raw_slang_symbol_primitive_instance_port_connection(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_primitive_instance_port_connection",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_primitive_instance_delay`.
    pub fn raw_slang_symbol_primitive_instance_delay(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_primitive_instance_delay",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_primitive_instance_drive_strength`.
    pub fn raw_slang_symbol_primitive_instance_drive_strength(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_primitive_instance_drive_strength",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_procedural_block_block_count`.
    pub fn raw_slang_symbol_procedural_block_block_count(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_procedural_block_block_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_procedural_block_block`.
    pub fn raw_slang_symbol_procedural_block_block(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_procedural_block_block",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_procedural_block_procedure_kind`.
    pub fn raw_slang_symbol_procedural_block_procedure_kind(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_procedural_block_procedure_kind",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_procedural_block_is_single_driver_block`.
    pub fn raw_slang_symbol_procedural_block_is_single_driver_block(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_procedural_block_is_single_driver_block",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_property_port_count`.
    pub fn raw_slang_symbol_property_port_count(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_property_port_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_property_port`.
    pub fn raw_slang_symbol_property_port(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_property_port",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_timing_path_connection_kind`.
    pub fn raw_slang_symbol_timing_path_connection_kind(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_timing_path_connection_kind",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_timing_path_polarity`.
    pub fn raw_slang_symbol_timing_path_polarity(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_timing_path_polarity",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_timing_path_edge_polarity`.
    pub fn raw_slang_symbol_timing_path_edge_polarity(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_timing_path_edge_polarity",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_timing_path_edge_identifier`.
    pub fn raw_slang_symbol_timing_path_edge_identifier(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_timing_path_edge_identifier",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_timing_path_is_state_dependent`.
    pub fn raw_slang_symbol_timing_path_is_state_dependent(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_timing_path_is_state_dependent",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_timing_path_condition_expr`.
    pub fn raw_slang_symbol_timing_path_condition_expr(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_timing_path_condition_expr",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_timing_path_edge_source_expr`.
    pub fn raw_slang_symbol_timing_path_edge_source_expr(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_timing_path_edge_source_expr",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_timing_path_input_count`.
    pub fn raw_slang_symbol_timing_path_input_count(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_timing_path_input_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_timing_path_input`.
    pub fn raw_slang_symbol_timing_path_input(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_timing_path_input",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_timing_path_output_count`.
    pub fn raw_slang_symbol_timing_path_output_count(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_timing_path_output_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_timing_path_output`.
    pub fn raw_slang_symbol_timing_path_output(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_timing_path_output",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_timing_path_delay_count`.
    pub fn raw_slang_symbol_timing_path_delay_count(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_timing_path_delay_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_timing_path_delay`.
    pub fn raw_slang_symbol_timing_path_delay(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_timing_path_delay",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_pulse_style_kind`.
    pub fn raw_slang_symbol_pulse_style_kind(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call("slang_symbol_pulse_style_kind", &[Val::I32(__p_sym as i32)]);
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_pulse_style_terminal_count`.
    pub fn raw_slang_symbol_pulse_style_terminal_count(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_pulse_style_terminal_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_pulse_style_terminal`.
    pub fn raw_slang_symbol_pulse_style_terminal(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_pulse_style_terminal",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_randseq_prod_is_null`.
    pub fn raw_slang_randseq_prod_is_null(&mut self, prod: u32) -> Result<u32, Error> {
        let __r = self.call("slang_randseq_prod_is_null", &[Val::I32(prod as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_randseq_rule_count`.
    pub fn raw_slang_symbol_randseq_rule_count(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_randseq_rule_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_randseq_rule_prod_count`.
    pub fn raw_slang_symbol_randseq_rule_prod_count(
        &mut self,
        sym: &[u8],
        rule_index: u32,
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_randseq_rule_prod_count",
            &[Val::I32(__p_sym as i32), Val::I32(rule_index as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_randseq_rule_prod`.
    pub fn raw_slang_symbol_randseq_rule_prod(
        &mut self,
        sym: &[u8],
        rule_index: u32,
        prod_index: u32,
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_randseq_rule_prod",
            &[
                Val::I32(__p_sym as i32),
                Val::I32(rule_index as i32),
                Val::I32(prod_index as i32),
            ],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_randseq_prod_get_kind`.
    pub fn raw_slang_randseq_prod_get_kind(&mut self, prod: u32) -> Result<u32, Error> {
        let __r = self.call("slang_randseq_prod_get_kind", &[Val::I32(prod as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_randseq_prod_item_target`.
    pub fn raw_slang_randseq_prod_item_target(&mut self, prod: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_randseq_prod_item_target",
            &[Val::I32(__sret as i32), Val::I32(prod as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_randseq_prod_item_arg_count`.
    pub fn raw_slang_randseq_prod_item_arg_count(&mut self, prod: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_randseq_prod_item_arg_count",
            &[Val::I32(prod as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_randseq_prod_item_arg`.
    pub fn raw_slang_randseq_prod_item_arg(
        &mut self,
        prod: u32,
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_randseq_prod_item_arg",
            &[
                Val::I32(__sret as i32),
                Val::I32(prod as i32),
                Val::I32(index as i32),
            ],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_randseq_prod_code_block_block`.
    pub fn raw_slang_randseq_prod_code_block_block(
        &mut self,
        prod: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_randseq_prod_code_block_block",
            &[Val::I32(__sret as i32), Val::I32(prod as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_randseq_prod_if_else_expr`.
    pub fn raw_slang_randseq_prod_if_else_expr(&mut self, prod: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_randseq_prod_if_else_expr",
            &[Val::I32(__sret as i32), Val::I32(prod as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_randseq_prod_if_else_if_item`.
    pub fn raw_slang_randseq_prod_if_else_if_item(&mut self, prod: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_randseq_prod_if_else_if_item",
            &[Val::I32(prod as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_randseq_prod_if_else_has_else_item`.
    pub fn raw_slang_randseq_prod_if_else_has_else_item(
        &mut self,
        prod: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_randseq_prod_if_else_has_else_item",
            &[Val::I32(prod as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_randseq_prod_if_else_else_item`.
    pub fn raw_slang_randseq_prod_if_else_else_item(&mut self, prod: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_randseq_prod_if_else_else_item",
            &[Val::I32(prod as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_randseq_prod_repeat_expr`.
    pub fn raw_slang_randseq_prod_repeat_expr(&mut self, prod: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_randseq_prod_repeat_expr",
            &[Val::I32(__sret as i32), Val::I32(prod as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_randseq_prod_repeat_item`.
    pub fn raw_slang_randseq_prod_repeat_item(&mut self, prod: u32) -> Result<u32, Error> {
        let __r = self.call("slang_randseq_prod_repeat_item", &[Val::I32(prod as i32)]);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_randseq_prod_case_expr`.
    pub fn raw_slang_randseq_prod_case_expr(&mut self, prod: u32) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_randseq_prod_case_expr",
            &[Val::I32(__sret as i32), Val::I32(prod as i32)],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_randseq_prod_case_item_count`.
    pub fn raw_slang_randseq_prod_case_item_count(&mut self, prod: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_randseq_prod_case_item_count",
            &[Val::I32(prod as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_randseq_prod_case_item_expression_count`.
    pub fn raw_slang_randseq_prod_case_item_expression_count(
        &mut self,
        prod: u32,
        item_index: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_randseq_prod_case_item_expression_count",
            &[Val::I32(prod as i32), Val::I32(item_index as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_randseq_prod_case_item_expression`.
    pub fn raw_slang_randseq_prod_case_item_expression(
        &mut self,
        prod: u32,
        item_index: u32,
        expr_index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __r = self.call(
            "slang_randseq_prod_case_item_expression",
            &[
                Val::I32(__sret as i32),
                Val::I32(prod as i32),
                Val::I32(item_index as i32),
                Val::I32(expr_index as i32),
            ],
        );
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_randseq_prod_case_item_item`.
    pub fn raw_slang_randseq_prod_case_item_item(
        &mut self,
        prod: u32,
        item_index: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_randseq_prod_case_item_item",
            &[Val::I32(prod as i32), Val::I32(item_index as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_randseq_prod_case_has_default_item`.
    pub fn raw_slang_randseq_prod_case_has_default_item(
        &mut self,
        prod: u32,
    ) -> Result<u32, Error> {
        let __r = self.call(
            "slang_randseq_prod_case_has_default_item",
            &[Val::I32(prod as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_randseq_prod_case_default_item`.
    pub fn raw_slang_randseq_prod_case_default_item(&mut self, prod: u32) -> Result<u32, Error> {
        let __r = self.call(
            "slang_randseq_prod_case_default_item",
            &[Val::I32(prod as i32)],
        );
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_randseq_rule_block`.
    pub fn raw_slang_symbol_randseq_rule_block(
        &mut self,
        sym: &[u8],
        rule_index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_randseq_rule_block",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(rule_index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_randseq_rule_weight_expr`.
    pub fn raw_slang_symbol_randseq_rule_weight_expr(
        &mut self,
        sym: &[u8],
        rule_index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_randseq_rule_weight_expr",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(rule_index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_randseq_rule_is_rand_join`.
    pub fn raw_slang_symbol_randseq_rule_is_rand_join(
        &mut self,
        sym: &[u8],
        rule_index: u32,
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_randseq_rule_is_rand_join",
            &[Val::I32(__p_sym as i32), Val::I32(rule_index as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_randseq_rule_rand_join_expr`.
    pub fn raw_slang_symbol_randseq_rule_rand_join_expr(
        &mut self,
        sym: &[u8],
        rule_index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_randseq_rule_rand_join_expr",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(rule_index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_randseq_rule_has_code_block`.
    pub fn raw_slang_symbol_randseq_rule_has_code_block(
        &mut self,
        sym: &[u8],
        rule_index: u32,
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_randseq_rule_has_code_block",
            &[Val::I32(__p_sym as i32), Val::I32(rule_index as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_randseq_rule_code_block`.
    pub fn raw_slang_symbol_randseq_rule_code_block(
        &mut self,
        sym: &[u8],
        rule_index: u32,
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_randseq_rule_code_block",
            &[Val::I32(__p_sym as i32), Val::I32(rule_index as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_randseq_argument_count`.
    pub fn raw_slang_symbol_randseq_argument_count(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_randseq_argument_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_randseq_argument`.
    pub fn raw_slang_symbol_randseq_argument(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_randseq_argument",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_randseq_return_type`.
    pub fn raw_slang_symbol_randseq_return_type(&mut self, sym: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_randseq_return_type",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_sequence_port_count`.
    pub fn raw_slang_symbol_sequence_port_count(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_sequence_port_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_sequence_port`.
    pub fn raw_slang_symbol_sequence_port(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_sequence_port",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_specparam_is_path_pulse`.
    pub fn raw_slang_symbol_specparam_is_path_pulse(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_specparam_is_path_pulse",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_specparam_path_source`.
    pub fn raw_slang_symbol_specparam_path_source(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_specparam_path_source",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_specparam_path_dest`.
    pub fn raw_slang_symbol_specparam_path_dest(&mut self, sym: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_specparam_path_dest",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_statement_block_kind`.
    pub fn raw_slang_symbol_statement_block_kind(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_statement_block_kind",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_statement_block_default_lifetime`.
    pub fn raw_slang_symbol_statement_block_default_lifetime(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_statement_block_default_lifetime",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_subroutine_default_lifetime`.
    pub fn raw_slang_symbol_subroutine_default_lifetime(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_subroutine_default_lifetime",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_subroutine_flags`.
    pub fn raw_slang_symbol_subroutine_flags(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call("slang_symbol_subroutine_flags", &[Val::I32(__p_sym as i32)]);
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_subroutine_argument_count`.
    pub fn raw_slang_symbol_subroutine_argument_count(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_subroutine_argument_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_subroutine_argument`.
    pub fn raw_slang_symbol_subroutine_argument(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_subroutine_argument",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_subroutine_override`.
    pub fn raw_slang_symbol_subroutine_override(&mut self, sym: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_subroutine_override",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_subroutine_prototype`.
    pub fn raw_slang_symbol_subroutine_prototype(&mut self, sym: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_subroutine_prototype",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_subroutine_return_type`.
    pub fn raw_slang_symbol_subroutine_return_type(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_subroutine_return_type",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_subroutine_kind`.
    pub fn raw_slang_symbol_subroutine_kind(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call("slang_symbol_subroutine_kind", &[Val::I32(__p_sym as i32)]);
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_subroutine_is_virtual`.
    pub fn raw_slang_symbol_subroutine_is_virtual(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_subroutine_is_virtual",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_subroutine_return_val_var`.
    pub fn raw_slang_symbol_subroutine_return_val_var(
        &mut self,
        sym: &[u8],
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_subroutine_return_val_var",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_subroutine_this_var`.
    pub fn raw_slang_symbol_subroutine_this_var(&mut self, sym: &[u8]) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_subroutine_this_var",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_system_timing_check_kind`.
    pub fn raw_slang_symbol_system_timing_check_kind(&mut self, sym: &[u8]) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_system_timing_check_kind",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_system_timing_check_argument_count`.
    pub fn raw_slang_symbol_system_timing_check_argument_count(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_system_timing_check_argument_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_system_timing_check_argument_expr`.
    pub fn raw_slang_symbol_system_timing_check_argument_expr(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_system_timing_check_argument_expr",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_system_timing_check_argument_condition`.
    pub fn raw_slang_symbol_system_timing_check_argument_condition(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_system_timing_check_argument_condition",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_system_timing_check_argument_edge`.
    pub fn raw_slang_symbol_system_timing_check_argument_edge(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_system_timing_check_argument_edge",
            &[Val::I32(__p_sym as i32), Val::I32(index as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_system_timing_check_argument_edge_descriptor_count`.
    pub fn raw_slang_symbol_system_timing_check_argument_edge_descriptor_count(
        &mut self,
        sym: &[u8],
        arg_index: u32,
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_system_timing_check_argument_edge_descriptor_count",
            &[Val::I32(__p_sym as i32), Val::I32(arg_index as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_system_timing_check_argument_edge_descriptor`.
    pub fn raw_slang_symbol_system_timing_check_argument_edge_descriptor(
        &mut self,
        sym: &[u8],
        arg_index: u32,
        desc_index: u32,
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_system_timing_check_argument_edge_descriptor",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(arg_index as i32),
                Val::I32(desc_index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_symbol_uninstantiated_def_definition_name`.
    pub fn raw_slang_symbol_uninstantiated_def_definition_name(
        &mut self,
        sym: &[u8],
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_uninstantiated_def_definition_name",
            &[Val::I32(__sret as i32), Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_symbol_uninstantiated_def_param_expression_count`.
    pub fn raw_slang_symbol_uninstantiated_def_param_expression_count(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_uninstantiated_def_param_expression_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_uninstantiated_def_param_expression`.
    pub fn raw_slang_symbol_uninstantiated_def_param_expression(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_uninstantiated_def_param_expression",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_uninstantiated_def_port_connection_count`.
    pub fn raw_slang_symbol_uninstantiated_def_port_connection_count(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_uninstantiated_def_port_connection_count",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
    /// Raw marshalling for `slang_symbol_uninstantiated_def_port_connection`.
    pub fn raw_slang_symbol_uninstantiated_def_port_connection(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<Vec<u32>, Error> {
        let __sret = self.malloc(16)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_uninstantiated_def_port_connection",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let mut __w = Vec::with_capacity(4);
        for __i in 0..4 {
            __w.push(self.read_u32(__sret + __i * 4)?);
        }
        self.free(__sret);
        Ok(__w)
    }
    /// Raw marshalling for `slang_symbol_uninstantiated_def_port_name`.
    pub fn raw_slang_symbol_uninstantiated_def_port_name(
        &mut self,
        sym: &[u8],
        index: u32,
    ) -> Result<String, Error> {
        let __sret = self.malloc(12)?;
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_uninstantiated_def_port_name",
            &[
                Val::I32(__sret as i32),
                Val::I32(__p_sym as i32),
                Val::I32(index as i32),
            ],
        );
        self.free(__p_sym);
        __r?;
        let __s = self.take_slang_str(__sret)?;
        self.free(__sret);
        Ok(__s)
    }
    /// Raw marshalling for `slang_symbol_uninstantiated_def_is_checker`.
    pub fn raw_slang_symbol_uninstantiated_def_is_checker(
        &mut self,
        sym: &[u8],
    ) -> Result<u32, Error> {
        let __p_sym = self.malloc(16)?;
        self.write(__p_sym, &sym[..16])?;
        let __r = self.call(
            "slang_symbol_uninstantiated_def_is_checker",
            &[Val::I32(__p_sym as i32)],
        );
        self.free(__p_sym);
        let __v = __r?;
        Ok(match __v.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }
}

/* skipped (hand-written or N/A):
 *   slang_node_child: has a struct out-param
 *   slang_node_member_span: has a struct out-param
 *   slang_token_trivia: has a struct out-param
 *   slang_diagnostics_at: has a struct out-param
 *   slang_diagnostics_note: has a struct out-param
 *   slang_diagnostics_range: has a struct out-param
 *   slang_compilation_get_default_time_scale: has a struct out-param
 *   slang_time_scale_apply: unsupported return type `f64`
 *   slang_time_scale_from_string: has a struct out-param
 *   slang_time_scale_value_from_literal: has a struct out-param
 *   slang_time_scale_value_from_string: has a struct out-param
 *   slang_compilation_try_parse_name: has a struct out-param
 *   slang_scope_get_time_scale: has a struct out-param
 *   slang_declared_type_resolved_dimensions: has a struct out-param
 *   slang_definition_time_scale: has a struct out-param
 *   slang_symbol_compilation_unit_time_scale: has a struct out-param
 *   slang_symbol_elab_system_task_message: has a struct out-param
 *   slang_forwarding_typedef_visibility: has a struct out-param
 *   slang_expression_cached_constant: has a struct out-param
 *   slang_expression_eval: has a struct out-param
 *   slang_constant_real: has a struct out-param
 *   slang_constant_flat_int: has a struct out-param
 *   slang_svint_as_i64: has a struct out-param
 *   slang_svint_as_u64: has a struct out-param
 *   slang_svint_from_float: unsupported arg type `f32`
 *   slang_ast_sem_children: has a struct out-param
 *   slang_stmt_foreach_loop_dim_range: has a struct out-param
 *   slang_constant_range_get_indexed_range: has a struct out-param
 *   slang_expr_real_literal_value: unsupported return type `f64`
 *   slang_expr_time_literal_value: unsupported return type `f64`
 *   slang_expr_time_literal_scale: has a struct out-param
 *   slang_expr_effective_width: has a struct out-param
 *   slang_driver_parse_args: unsupported arg type `c_int`
 *   slang_driver_parse_args_with_options: unsupported arg type `c_int`
 *   slang_driver_option_flag: has a struct out-param
 *   slang_driver_option_int: has a struct out-param
 *   slang_driver_option_string: has a struct out-param
 *   slang_source_options_num_threads: has a struct out-param
 *   slang_analysis_driver: has a struct out-param
 *   slang_value_driver_override_range: has a struct out-param
 *   slang_analyzed_procedure_driver_at: has a struct out-param
 *   slang_extern_impl_next: takes a callback
 *   slang_extern_impl_impl: takes a callback
 *   slang_symbol_package_time_scale: has a struct out-param
 */
