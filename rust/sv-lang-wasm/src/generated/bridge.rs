//! Generated raw wasm marshalling for the slang C API. See xtask/src/wasm_bridge.rs.
//!
//! One `raw_*` method per non-callback C function; the ergonomic API in lib.rs
//! is written on top. 197 functions generated, 18 skipped (callbacks / struct out-params).

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
    /// Raw marshalling for `slang_driver_destroy`.
    pub fn raw_slang_driver_destroy(&mut self, driver: u32) -> Result<(), Error> {
        let __r = self.call("slang_driver_destroy", &[Val::I32(driver as i32)]);
        __r?;
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
}

/* skipped (hand-written or N/A):
 *   slang_node_child: has a struct out-param
 *   slang_node_member_span: has a struct out-param
 *   slang_token_trivia: has a struct out-param
 *   slang_diagnostics_at: has a struct out-param
 *   slang_diagnostics_note: has a struct out-param
 *   slang_diagnostics_range: has a struct out-param
 *   slang_expression_cached_constant: has a struct out-param
 *   slang_expression_eval: has a struct out-param
 *   slang_constant_real: has a struct out-param
 *   slang_constant_flat_int: has a struct out-param
 *   slang_svint_as_i64: has a struct out-param
 *   slang_svint_as_u64: has a struct out-param
 *   slang_ast_sem_children: has a struct out-param
 *   slang_driver_parse_args: unsupported arg type `c_int`
 *   slang_driver_option_flag: has a struct out-param
 *   slang_driver_option_int: has a struct out-param
 *   slang_driver_option_string: has a struct out-param
 *   slang_analysis_driver: has a struct out-param
 */
