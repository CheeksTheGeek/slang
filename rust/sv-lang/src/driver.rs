//! The command-line driver: accept slang's own CLI flags (file lists, `-I`,
//! `-D`, `--top`, `-W...`, `--std`, diagnostics formatting, ...) and get back
//! the parsed trees and an elaborated design — exactly as the `slang` binary
//! does. This is the drop-in path for tools that shell out to `slang` today.

use core::ffi::c_char;
use core::marker::PhantomData;
use std::sync::Arc;

use sv_lang_sys as sys;

use crate::{Compilation, Design, Diagnostics, Error, Session, SyntaxTree, ffi};

/// Owns the raw driver handle (and, through it, the driver's source manager).
struct DriverInner {
    raw: sys::slang_driver,
}

// SAFETY: the driver handle is only ever touched serially (the building steps
// take `&mut Driver`; afterwards it is only read). It may move between threads.
unsafe impl Send for DriverInner {}
// SAFETY: as above — only `&`-read after building, so it may be shared (the
// shared reads go through the Send+Sync Session/Design keepalive).
unsafe impl Sync for DriverInner {}

impl Drop for DriverInner {
    fn drop(&mut self) {
        // SAFETY: we own the driver handle; every derived Session holds an Arc
        // to this inner, so nothing referencing the source manager outlives it.
        unsafe { sys::slang_driver_destroy(self.raw) };
    }
}

/// The value type of a custom command-line option registered with
/// [`Driver::add_option`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptionKind {
    /// A boolean switch (`--flag` / `--no-flag`), read back with
    /// [`Driver::option_flag`].
    Flag,
    /// An integer value, read back with [`Driver::option_int`].
    Int,
    /// A string value, read back with [`Driver::option_string`].
    String,
}

impl OptionKind {
    fn to_raw(self) -> sys::slang_option_kind {
        match self {
            OptionKind::Flag => sys::SLANG_OPTION_FLAG,
            OptionKind::Int => sys::SLANG_OPTION_INT,
            OptionKind::String => sys::SLANG_OPTION_STRING,
        }
    }
}

/// A driver over slang's standard command line.
///
/// Note: `slang::driver::Driver::createOptionBag` (the driver's internal
/// step that turns parsed options into the `Bag` a compilation is built
/// from) has no separate accessor here — [`compile`](Self::compile) already
/// calls it internally (via `slang_driver_create_compilation`), so there is
/// nothing for a caller to do with the intermediate bag on its own.
///
/// ```no_run
/// # fn main() -> Result<(), sv_lang::Error> {
/// let mut driver = sv_lang::Driver::new()?;
/// driver.parse_args(std::env::args())?;    // e.g. `-I include cpu.sv --top top`
/// driver.process_options()?;
/// driver.parse_sources()?;
/// let design = driver.compile()?;
/// for top in design.top_instances() {
///     println!("{}", top.name());
/// }
/// driver.report_diagnostics(false);        // print like the `slang` CLI
/// # Ok(())
/// # }
/// ```
pub struct Driver {
    inner: Arc<DriverInner>,
    session: Session,
}

impl Driver {
    /// Creates a driver with slang's standard arguments registered.
    ///
    /// ```no_run
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let driver = sv_lang::Driver::new()?;
    /// # let _ = driver;
    /// # Ok(()) }
    /// ```
    pub fn new() -> Result<Driver, Error> {
        let mut err = ffi::error();
        // SAFETY: out-error checked.
        let raw = unsafe { sys::slang_driver_create(&mut err) };
        ffi::check(&err)?;

        let inner = Arc::new(DriverInner { raw });
        // The session borrows the driver's source manager and keeps the driver
        // alive via the keepalive Arc, so any tree/design derived from it
        // outlives neither.
        // SAFETY: the driver owns its SM for as long as `inner` lives.
        let sm = unsafe { sys::slang_driver_source_manager(raw) };
        let session = Session::from_borrowed(sm, inner.clone());
        Ok(Driver { inner, session })
    }

    /// Creates a driver with NO standard arguments registered — `--top`,
    /// `-D`, `-I`, `--std`, and the rest are all unrecognized until
    /// [`add_standard_args`](Self::add_standard_args) is called. Useful for a
    /// tool that wants a fully custom argument set built only from its own
    /// registered options.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let mut driver = sv_lang::Driver::new_bare()?;
    /// // `--top` is not yet a recognized flag on a bare driver...
    /// assert!(driver.parse_args(["slang".to_string(), "--top".into(), "m".into()]).is_err());
    /// // ...until the standard arguments are added.
    /// driver.add_standard_args()?;
    /// driver.parse_args(["slang".to_string(), "--top".into(), "m".into()])?;
    /// # Ok(()) }
    /// ```
    pub fn new_bare() -> Result<Driver, Error> {
        let mut err = ffi::error();
        // SAFETY: out-error checked.
        let raw = unsafe { sys::slang_driver_create_bare(&mut err) };
        ffi::check(&err)?;

        let inner = Arc::new(DriverInner { raw });
        // SAFETY: the driver owns its SM for as long as `inner` lives.
        let sm = unsafe { sys::slang_driver_source_manager(raw) };
        let session = Session::from_borrowed(sm, inner.clone());
        Ok(Driver { inner, session })
    }

    /// Adds slang's standard command-line arguments (`--top`, `-D`, `-I`,
    /// `--std`, warning flags, diagnostics formatting, ...) to this driver.
    /// [`new`](Self::new) already has them registered, so calling this on
    /// such a driver is a harmless no-op; drivers built with
    /// [`new_bare`](Self::new_bare) need this call once, before parsing, to
    /// opt in (see the example on [`new_bare`](Self::new_bare)).
    pub fn add_standard_args(&mut self) -> Result<(), Error> {
        let mut err = ffi::error();
        // SAFETY: the driver is valid; out-error checked.
        unsafe { sys::slang_driver_add_standard_args(self.raw(), &mut err) };
        ffi::check(&err)
    }

    /// Registers a custom command-line option, in addition to slang's own —
    /// call this before [`parse_args`](Self::parse_args). `names` is a
    /// comma-separated list of spellings (e.g. `"-x,--extra"`); the same
    /// string reads the value back with [`option_flag`](Self::option_flag),
    /// [`option_int`](Self::option_int) or [`option_string`](Self::option_string)
    /// (matching `kind`). `value_name` (e.g. `"<n>"`) is shown in
    /// [`help_text`](Self::help_text) and may be empty. Returns
    /// [`Error::InvalidArgument`] if `names` was already registered.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::{Driver, OptionKind};
    /// let mut driver = Driver::new()?;
    /// driver.add_option("--depth", OptionKind::Int, "search depth", "<depth>")?;
    /// driver.parse_args(["slang".to_string(), "--depth".into(), "3".into()])?;
    /// assert_eq!(driver.option_int("--depth"), Some(3));
    /// # Ok(()) }
    /// ```
    pub fn add_option(
        &mut self,
        names: &str,
        kind: OptionKind,
        description: &str,
        value_name: &str,
    ) -> Result<(), Error> {
        let mut err = ffi::error();
        let (n, nl) = ffi::as_ptr_len(names);
        let (d, dl) = ffi::as_ptr_len(description);
        let (v, vl) = ffi::as_ptr_len(value_name);
        // SAFETY: the driver is valid; all string pointers are valid for the
        // call; out-error checked.
        unsafe {
            sys::slang_driver_add_option(self.raw(), n, nl, kind.to_raw(), d, dl, v, vl, &mut err)
        };
        ffi::check(&err)
    }

    /// Reads back a custom boolean option registered with
    /// [`add_option`](Self::add_option) as [`OptionKind::Flag`]. `None` if it
    /// was not given on the command line (see [`add_option`](Self::add_option)
    /// for a full example).
    pub fn option_flag(&self, names: &str) -> Option<bool> {
        let (n, nl) = ffi::as_ptr_len(names);
        let mut out = false;
        // SAFETY: the driver is valid; the string pointer is valid for the call.
        let ok = unsafe { sys::slang_driver_option_flag(self.raw(), n, nl, &mut out) };
        ok.then_some(out)
    }

    /// Reads back a custom integer option registered with
    /// [`add_option`](Self::add_option) as [`OptionKind::Int`]. `None` if it
    /// was not given on the command line (see [`add_option`](Self::add_option)
    /// for a full example).
    pub fn option_int(&self, names: &str) -> Option<i64> {
        let (n, nl) = ffi::as_ptr_len(names);
        let mut out = 0i64;
        // SAFETY: the driver is valid; the string pointer is valid for the call.
        let ok = unsafe { sys::slang_driver_option_int(self.raw(), n, nl, &mut out) };
        ok.then_some(out)
    }

    /// Reads back a custom string option registered with
    /// [`add_option`](Self::add_option) as [`OptionKind::String`]. `None` if
    /// it was not given on the command line.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::{Driver, OptionKind};
    /// let mut driver = Driver::new()?;
    /// driver.add_option("--prefix", OptionKind::String, "instance prefix", "<prefix>")?;
    /// driver.parse_args(["slang".to_string(), "--prefix".into(), "u_".into()])?;
    /// assert_eq!(driver.option_string("--prefix").as_deref(), Some("u_"));
    /// assert_eq!(driver.option_string("--unknown-option"), None);
    /// # Ok(()) }
    /// ```
    pub fn option_string(&self, names: &str) -> Option<String> {
        let (n, nl) = ffi::as_ptr_len(names);
        // A zeroed slang_str; the driver fills it in (borrowed from its own
        // storage) only when it returns true.
        let mut out = sys::slang_str {
            data: core::ptr::null(),
            len: 0,
            owner: core::ptr::null_mut(),
        };
        // SAFETY: the driver is valid; the string pointer is valid for the call.
        let ok = unsafe { sys::slang_driver_option_string(self.raw(), n, nl, &mut out) };
        // The result is borrowed from the driver's own storage, not owned.
        ok.then(|| ffi::borrowed_str(out))
    }

    /// One-call convenience mirroring the `slang` CLI: create a driver, parse
    /// the given flags, apply options, and load+parse all named sources.
    /// Returns the ready driver — call [`compile`](Self::compile) for the
    /// elaborated design, or [`report_diagnostics`](Self::report_diagnostics)
    /// to print like the CLI.
    ///
    /// Pass only flags (no program name); the same set the `slang` binary and
    /// the `slang-rs` crate accept, e.g. `["-I", "inc", "cpu.sv", "--top", "top"]`.
    ///
    /// ```no_run
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let driver = sv_lang::Driver::from_args(["cpu.sv", "--top", "cpu"])?;
    /// let design = { let mut d = driver; d.compile()? };
    /// assert!(design.top_instances().next().is_some());
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_args<I, S>(args: I) -> Result<Driver, Error>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut driver = Driver::new()?;
        // parse_args expects argv[0] to be the program name; supply a synthetic
        // one so callers pass only flags (as slang-rs does).
        let argv = core::iter::once("slang".to_string()).chain(args.into_iter().map(Into::into));
        driver.parse_args(argv)?;
        driver.process_options()?;
        driver.parse_sources()?;
        Ok(driver)
    }

    fn raw(&self) -> sys::slang_driver {
        self.inner.raw
    }

    /// Parses a command line (the first argument is the program name, as in
    /// `std::env::args()`). Returns [`Error::InvalidArgument`] if the arguments
    /// were rejected; the driver has already printed why.
    ///
    /// ```no_run
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let mut driver = sv_lang::Driver::new()?;
    /// driver.parse_args(["slang".to_string(), "cpu.sv".to_string()])?;
    /// # Ok(()) }
    /// ```
    pub fn parse_args(&mut self, args: impl IntoIterator<Item = String>) -> Result<(), Error> {
        // Keep the C strings alive for the duration of the call.
        let owned: Vec<std::ffi::CString> = args
            .into_iter()
            .map(|a| std::ffi::CString::new(a).unwrap_or_default())
            .collect();
        let ptrs: Vec<*const c_char> = owned.iter().map(|s| s.as_ptr()).collect();
        let mut err = ffi::error();
        // SAFETY: `ptrs` is valid for the call and outlives it; out-error checked.
        let ok = unsafe {
            sys::slang_driver_parse_args(self.raw(), ptrs.len() as i32, ptrs.as_ptr(), &mut err)
        };
        ffi::check(&err)?;
        if ok {
            Ok(())
        } else {
            Err(Error::InvalidArgument("command line rejected".into()))
        }
    }

    /// As [`parse_args`](Self::parse_args), but with explicit [`ParseOptions`]
    /// (comment support, environment-variable expansion, duplicate handling).
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::{Driver, ParseOptions};
    ///
    /// // `--timescale` is a scalar option; giving it twice is a hard error...
    /// let args = || {
    ///     ["slang", "--timescale", "1ns/1ps", "--timescale", "1ns/1ps"]
    ///         .map(str::to_string)
    /// };
    /// let mut strict = Driver::new()?;
    /// assert!(strict.parse_args(args()).is_err());
    ///
    /// // ...unless the parse options say to ignore duplicates.
    /// let mut options = ParseOptions::new();
    /// assert!(!options.ignore_duplicates());
    /// options.set_ignore_duplicates(true);
    /// assert!(options.ignore_duplicates());
    ///
    /// let mut lenient = Driver::new()?;
    /// lenient.parse_args_with_options(args(), &options)?;
    /// # Ok(()) }
    /// ```
    pub fn parse_args_with_options(
        &mut self,
        args: impl IntoIterator<Item = String>,
        options: &ParseOptions,
    ) -> Result<(), Error> {
        let owned: Vec<std::ffi::CString> = args
            .into_iter()
            .map(|a| std::ffi::CString::new(a).unwrap_or_default())
            .collect();
        let ptrs: Vec<*const c_char> = owned.iter().map(|s| s.as_ptr()).collect();
        let mut err = ffi::error();
        // SAFETY: `ptrs` is valid for the call and outlives it; `options.raw`
        // is a valid handle; out-error checked.
        let ok = unsafe {
            sys::slang_driver_parse_args_with_options(
                self.raw(),
                ptrs.len() as i32,
                ptrs.as_ptr(),
                options.raw,
                &mut err,
            )
        };
        ffi::check(&err)?;
        if ok {
            Ok(())
        } else {
            Err(Error::InvalidArgument("command line rejected".into()))
        }
    }

    /// Applies the parsed options (include paths, defines, ...). Returns an
    /// error if an option was invalid (already reported by the driver).
    ///
    /// ```no_run
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let mut driver = sv_lang::Driver::new()?;
    /// driver.parse_args(std::env::args())?;
    /// driver.process_options()?;
    /// # Ok(()) }
    /// ```
    pub fn process_options(&mut self) -> Result<(), Error> {
        let mut err = ffi::error();
        // SAFETY: the driver is valid; out-error checked.
        let ok = unsafe { sys::slang_driver_process_options(self.raw(), &mut err) };
        ffi::check(&err)?;
        if ok {
            Ok(())
        } else {
            Err(Error::InvalidArgument("invalid options".into()))
        }
    }

    /// Loads and parses every source file named on the command line. Returns
    /// `false` if any file had parse errors; they are reported through
    /// [`report_diagnostics`](Self::report_diagnostics).
    ///
    /// ```no_run
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let mut driver = sv_lang::Driver::new()?;
    /// driver.parse_args(std::env::args())?;
    /// driver.process_options()?;
    /// let ok = driver.parse_sources()?;
    /// println!("parsed cleanly: {ok}");
    /// # Ok(()) }
    /// ```
    pub fn parse_sources(&mut self) -> Result<bool, Error> {
        let mut err = ffi::error();
        // SAFETY: the driver is valid; out-error checked.
        let ok = unsafe { sys::slang_driver_parse_sources(self.raw(), &mut err) };
        ffi::check(&err)?;
        Ok(ok)
    }

    /// The parsed syntax trees.
    ///
    /// ```no_run
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let driver = sv_lang::Driver::from_args(["cpu.sv"])?;
    /// for tree in driver.trees() {
    ///     println!("{:?}", tree.module_names().collect::<Vec<_>>());
    /// }
    /// # Ok(()) }
    /// ```
    pub fn trees(&self) -> Vec<SyntaxTree> {
        // SAFETY: the driver is valid.
        let count = unsafe { sys::slang_driver_tree_count(self.raw()) };
        (0..count)
            .filter_map(|i| {
                // SAFETY: index in range.
                let raw = unsafe { sys::slang_driver_tree(self.raw(), i) };
                (!raw.is_null()).then(|| {
                    // The driver owns the tree; retain it for our handle so it
                    // is not freed while we hold it.
                    // SAFETY: `raw` is a valid tree handle.
                    unsafe { sys::slang_syntax_tree_retain(raw) };
                    SyntaxTree::from_raw(raw, self.session.clone())
                })
            })
            .collect()
    }

    /// The driver's session (its source manager).
    ///
    /// ```no_run
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let driver = sv_lang::Driver::from_args(["cpu.sv"])?;
    /// let tree = driver.session().parse("module extra; endmodule\n")?;
    /// # let _ = tree;
    /// # Ok(()) }
    /// ```
    pub fn session(&self) -> &Session {
        &self.session
    }

    /// Creates a compilation from the parsed trees with the command-line
    /// options, then elaborates and freezes it into a [`Design`].
    ///
    /// ```no_run
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let mut driver = sv_lang::Driver::from_args(["cpu.sv", "--top", "cpu"])?;
    /// let design = driver.compile()?;
    /// assert!(design.top_instances().next().is_some());
    /// # Ok(()) }
    /// ```
    pub fn compile(&mut self) -> Result<Design, Error> {
        let mut err = ffi::error();
        // Force DisableInstanceCaching so the design can be fully totalized by
        // the freeze (the precondition for Design: Send + Sync).
        // SAFETY: the driver is valid; out-error checked.
        let comp = unsafe {
            sys::slang_driver_create_compilation(
                self.raw(),
                sys::SLANG_COMP_DISABLE_INSTANCE_CACHING,
                &mut err,
            )
        };
        ffi::check(&err)?;

        let mut freeze_err = ffi::error();
        let mut report = sys::slang_freeze_report::default();
        // SAFETY: we own `comp`; out-error checked.
        unsafe {
            sys::slang_compilation_freeze(
                comp,
                sys::SLANG_FREEZE_ALL,
                &mut report,
                &mut freeze_err,
            );
        }
        ffi::check(&freeze_err)?;
        Ok(crate::ast::design_from_raw(
            comp,
            self.session.clone(),
            report,
        ))
    }

    /// Reports a compilation's diagnostics through this driver's diagnostic
    /// engine (colors, `--diag-*` formatting, error limits) — the step
    /// [`report_diagnostics`](Self::report_diagnostics) assumes already ran.
    /// Also feeds [`diag_engine`](Self::diag_engine)'s error/warning counts.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let dir = std::env::temp_dir().join("sv_lang_report_compilation_doctest");
    /// std::fs::create_dir_all(&dir).unwrap();
    /// let bad = dir.join("bad.sv");
    /// std::fs::write(&bad, "module m; initial $display(nope); endmodule\n").unwrap();
    ///
    /// let mut driver = sv_lang::Driver::new()?;
    /// driver.parse_args(["slang".to_string(), bad.to_string_lossy().into_owned()])?;
    /// driver.process_options()?;
    /// driver.parse_sources()?;
    /// let design = driver.compile()?;
    /// assert_eq!(driver.diag_engine().num_errors(), 0);
    /// driver.report_compilation(&design);
    /// assert!(driver.diag_engine().num_errors() > 0);
    ///
    /// std::fs::remove_dir_all(&dir).ok();
    /// # Ok(()) }
    /// ```
    pub fn report_compilation(&mut self, design: &Design) {
        let mut err = ffi::error();
        // SAFETY: the driver and design are both valid; out-error checked.
        unsafe {
            sys::slang_driver_report_compilation(
                self.raw(),
                design.raw_compilation(),
                /* quiet */ true,
                &mut err,
            );
        }
    }

    /// Prints the compilation summary and returns whether the run had no
    /// errors, exactly as the `slang` CLI does. Call this last.
    ///
    /// ```no_run
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let mut driver = sv_lang::Driver::from_args(["cpu.sv"])?;
    /// let ok = driver.report_diagnostics(false); // print like the CLI
    /// std::process::exit(if ok { 0 } else { 1 });
    /// # }
    /// ```
    pub fn report_diagnostics(&mut self, quiet: bool) -> bool {
        let mut err = ffi::error();
        // SAFETY: the driver is valid.
        unsafe { sys::slang_driver_report_diagnostics(self.raw(), quiet, &mut err) }
    }

    /// The help text for slang's standard options.
    ///
    /// ```no_run
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let driver = sv_lang::Driver::new()?;
    /// let help = driver.help_text("my-tool [options] files...");
    /// println!("{help}");
    /// # Ok(()) }
    /// ```
    pub fn help_text(&self, overview: &str) -> String {
        let mut err = ffi::error();
        let (o, ol) = ffi::as_ptr_len(overview);
        // SAFETY: the driver is valid; the string is owned.
        unsafe { ffi::owned_str(sys::slang_driver_help_text(self.raw(), o, ol, &mut err)) }
    }

    /// The version of the SystemVerilog language this driver will use. Raw
    /// `slang::LanguageVersion` encoding: 0 = 1364-2005, 1 = 1800-2017 (the
    /// default), 2 = 1800-2023. Reflects `--std` once
    /// [`process_options`](Self::process_options) has run. A pure,
    /// allocation-free read.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let driver = sv_lang::Driver::new()?;
    /// assert_eq!(driver.language_version(), 1); // default: 1800-2017
    /// # Ok(()) }
    /// ```
    pub fn language_version(&self) -> u32 {
        // SAFETY: the driver is valid.
        unsafe { sys::slang_driver_language_version(self.raw()) }
    }

    /// Processes command file(s) matching `pattern` for more options, as if
    /// `-f pattern` had been passed on the command line. If `make_relative`
    /// is true, paths within the file are resolved relative to the file
    /// itself rather than the current directory; if `separate_unit` is true,
    /// the file is treated as a separate compilation-unit listing whose
    /// options apply only to that unit. Returns `Ok(false)` (having already
    /// printed why) if the pattern matched nothing or a file's options were
    /// rejected.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let dir = std::env::temp_dir().join("sv_lang_command_file_test");
    /// std::fs::create_dir_all(&dir).unwrap();
    /// let cmd_file = dir.join("opts.f");
    /// std::fs::write(&cmd_file, "-DFOO -DBAR=1\n").unwrap();
    ///
    /// let mut driver = sv_lang::Driver::new()?;
    /// let ok = driver.process_command_files(&cmd_file.to_string_lossy(), false, false)?;
    /// assert!(ok);
    ///
    /// let files = driver.command_file_metadata();
    /// assert_eq!(files.len(), 1);
    /// assert_eq!(files[0].defines, vec!["FOO".to_string(), "BAR=1".to_string()]);
    ///
    /// std::fs::remove_dir_all(&dir).ok();
    /// # Ok(()) }
    /// ```
    pub fn process_command_files(
        &mut self,
        pattern: &str,
        make_relative: bool,
        separate_unit: bool,
    ) -> Result<bool, Error> {
        let (p, pl) = ffi::as_ptr_len(pattern);
        let mut err = ffi::error();
        // SAFETY: `pattern` is valid for the call; out-error checked.
        let ok = unsafe {
            sys::slang_driver_process_command_files(
                self.raw(),
                p,
                pl,
                make_relative,
                separate_unit,
                &mut err,
            )
        };
        ffi::check(&err)?;
        Ok(ok)
    }

    /// Metadata (path and raw `-D` defines) for every command file
    /// [`process_command_files`](Self::process_command_files) has processed
    /// so far on this driver, one entry per file. See
    /// [`CommandFileMetadata`].
    pub fn command_file_metadata(&self) -> Vec<CommandFileMetadata> {
        // SAFETY: the driver is valid.
        let count = unsafe { sys::slang_driver_command_file_metadata_count(self.raw()) };
        (0..count)
            .map(|i| {
                // SAFETY: `i` is in range; the handle is borrowed from the
                // driver's snapshot for the duration of this call.
                let meta = unsafe { sys::slang_driver_command_file_metadata_at(self.raw(), i) };
                // SAFETY: `meta` came from a valid index above.
                let path =
                    unsafe { ffi::borrowed_str(sys::slang_command_file_metadata_path(meta)) };
                // SAFETY: `meta` is valid.
                let define_count = unsafe { sys::slang_command_file_metadata_define_count(meta) };
                let defines = (0..define_count)
                    .map(|j| {
                        // SAFETY: `meta` is valid; `j` is in range.
                        ffi::borrowed_str(unsafe {
                            sys::slang_command_file_metadata_define_at(meta, j)
                        })
                    })
                    .collect();
                CommandFileMetadata { path, defines }
            })
            .collect()
    }

    /// Writes any dependency files requested via the driver's `--depfile`
    /// options (a no-op if none were configured).
    pub fn optionally_write_dep_files(&mut self) {
        // SAFETY: the driver is valid.
        unsafe { sys::slang_driver_optionally_write_dep_files(self.raw(), core::ptr::null_mut()) };
    }

    /// The driver's diagnostic engine, used internally to classify and format
    /// diagnostics as they are issued. A pure, allocation-free read.
    pub fn diag_engine(&self) -> DiagEngine<'_> {
        // SAFETY: the driver is valid; never null.
        let raw = unsafe { sys::slang_driver_diag_engine(self.raw()) };
        DiagEngine {
            raw,
            _driver: core::marker::PhantomData,
        }
    }

    /// The analysis options this driver's configured flags would produce for
    /// a [`Design::analyze`](crate::Design::analyze)-style pass. A pure,
    /// allocation-free read (the returned struct is a snapshot copy).
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let driver = sv_lang::Driver::new()?;
    /// let opts = driver.analysis_options();
    /// assert_eq!(opts.max_case_analysis_steps, 65535);
    /// # Ok(()) }
    /// ```
    pub fn analysis_options(&self) -> AnalysisOptions {
        // SAFETY: the driver is valid.
        let raw = unsafe { sys::slang_driver_get_analysis_options(self.raw()) };
        AnalysisOptions {
            flags: raw.flags,
            max_case_analysis_steps: raw.max_case_analysis_steps,
            max_loop_analysis_steps: raw.max_loop_analysis_steps,
        }
    }

    /// Runs the preprocessor on all of this driver's loaded source buffers
    /// (see [`Driver::source_loader`]) and prints the result to stdout, as
    /// the `slang -E` CLI flag does. Returns `false` (having already printed
    /// why) if a preprocessing error occurred.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let dir = std::env::temp_dir().join("sv_lang_run_preprocessor_doctest");
    /// std::fs::create_dir_all(&dir).unwrap();
    /// let file = dir.join("m.sv");
    /// std::fs::write(&file, "`define FOO 1\nmodule m; endmodule\n").unwrap();
    ///
    /// let mut driver = sv_lang::Driver::new()?;
    /// driver.parse_args(["slang".to_string(), file.to_string_lossy().into_owned()])?;
    /// driver.process_options()?;
    /// let ok = driver.run_preprocessor(sv_lang::PreprocessFlags::NONE)?;
    /// assert!(ok);
    ///
    /// std::fs::remove_dir_all(&dir).ok();
    /// # Ok(()) }
    /// ```
    pub fn run_preprocessor(&mut self, flags: PreprocessFlags) -> Result<bool, Error> {
        let mut err = ffi::error();
        // SAFETY: the driver is valid; out-error checked.
        let ok = unsafe { sys::slang_driver_run_preprocessor(self.raw(), flags.bits(), &mut err) };
        ffi::check(&err)?;
        Ok(ok)
    }

    /// Prints all macros defined while preprocessing this driver's loaded
    /// source buffers to stdout, one per line. If `group_by_file` is true,
    /// macros are grouped and labelled by the file that defined them.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let dir = std::env::temp_dir().join("sv_lang_report_macros_doctest");
    /// std::fs::create_dir_all(&dir).unwrap();
    /// let file = dir.join("m.sv");
    /// std::fs::write(&file, "`define FOO 1\nmodule m; endmodule\n").unwrap();
    ///
    /// let mut driver = sv_lang::Driver::new()?;
    /// driver.parse_args(["slang".to_string(), file.to_string_lossy().into_owned()])?;
    /// driver.process_options()?;
    /// driver.report_macros(false); // prints "FOO 1" to stdout
    ///
    /// std::fs::remove_dir_all(&dir).ok();
    /// # Ok(()) }
    /// ```
    pub fn report_macros(&mut self, group_by_file: bool) {
        // SAFETY: the driver is valid; this can only fail via an internal
        // slang exception, which the C side already guards against.
        unsafe {
            sys::slang_driver_report_macros(self.raw(), group_by_file, core::ptr::null_mut())
        };
    }

    /// Reports (through this driver's diagnostic engine) all diagnostics
    /// found while parsing every source [`trees`](Self::trees) loaded via
    /// [`parse_sources`](Self::parse_sources), plus any library maps loaded
    /// along the way. Returns true if none of them were errors.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let dir = std::env::temp_dir().join("sv_lang_report_parse_diags_doctest");
    /// std::fs::create_dir_all(&dir).unwrap();
    /// let bad = dir.join("bad.sv");
    /// std::fs::write(&bad, "module m; +++ endmodule\n").unwrap();
    ///
    /// let mut driver = sv_lang::Driver::new()?;
    /// driver.parse_args(["slang".to_string(), bad.to_string_lossy().into_owned()])?;
    /// driver.process_options()?;
    /// driver.parse_sources()?;
    /// assert!(!driver.report_parse_diags()); // a syntax error was found
    ///
    /// std::fs::remove_dir_all(&dir).ok();
    /// # Ok(()) }
    /// ```
    pub fn report_parse_diags(&mut self) -> bool {
        let mut err = ffi::error();
        // SAFETY: the driver is valid.
        unsafe { sys::slang_driver_report_parse_diags(self.raw(), &mut err) }
    }

    /// Runs a full compile-report-analyze-report cycle exactly as the
    /// `slang` CLI does: creates a compilation from this driver's loaded
    /// sources and options, reports its diagnostics, runs analysis, and
    /// reports the final summary. If `quiet` is true, non-essential output
    /// is suppressed. Returns true if the run had no errors.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let dir = std::env::temp_dir().join("sv_lang_run_full_compilation_doctest");
    /// std::fs::create_dir_all(&dir).unwrap();
    /// let good = dir.join("good.sv");
    /// std::fs::write(&good, "module m; endmodule\n").unwrap();
    ///
    /// let mut driver = sv_lang::Driver::new()?;
    /// driver.parse_args(["slang".to_string(), good.to_string_lossy().into_owned()])?;
    /// driver.process_options()?;
    /// driver.parse_sources()?;
    /// assert!(driver.run_full_compilation(true));
    ///
    /// std::fs::remove_dir_all(&dir).ok();
    /// # Ok(()) }
    /// ```
    pub fn run_full_compilation(&mut self, quiet: bool) -> bool {
        let mut err = ffi::error();
        // SAFETY: the driver is valid.
        unsafe { sys::slang_driver_run_full_compilation(self.raw(), quiet, &mut err) }
    }

    /// Sets whether this driver's error/warning output and its
    /// [`text_diag_client`](Self::text_diag_client) use terminal color codes.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let mut driver = sv_lang::Driver::new()?;
    /// driver.set_terminal_colors_enabled(false);
    /// # Ok(()) }
    /// ```
    pub fn set_terminal_colors_enabled(&mut self, enable: bool) {
        // SAFETY: the driver is valid.
        unsafe {
            sys::slang_driver_set_terminal_colors_enabled(self.raw(), enable, core::ptr::null_mut())
        };
    }

    /// This driver's source loader, which handles loading and parsing groups
    /// of source files: file-pattern globs, library maps, search
    /// directories, and separately-compiled units. A pure, allocation-free
    /// read (the loader itself is mutated through the returned handle).
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let mut driver = sv_lang::Driver::new_bare()?;
    /// driver.source_loader().add_files("*.sv-does-not-exist")?;
    /// # Ok(()) }
    /// ```
    pub fn source_loader(&mut self) -> SourceLoader<'_> {
        // SAFETY: the driver is valid; never null.
        let raw = unsafe { sys::slang_driver_source_loader(self.raw()) };
        SourceLoader {
            raw,
            session: self.session.clone(),
            _driver: PhantomData,
        }
    }

    /// This driver's text diagnostic client, registered with its diagnostic
    /// engine at construction, which formats and accumulates every
    /// diagnostic issued through it as human-readable text. A pure,
    /// allocation-free read.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let driver = sv_lang::Driver::new()?;
    /// assert!(driver.text_diag_client().empty());
    /// # Ok(()) }
    /// ```
    pub fn text_diag_client(&self) -> TextDiagClient<'_> {
        // SAFETY: the driver is valid; never null.
        let raw = unsafe { sys::slang_driver_text_diag_client(self.raw()) };
        TextDiagClient {
            raw,
            _driver: PhantomData,
        }
    }

    /// Runs analysis over `comp` using this driver's configured analysis
    /// options and thread pool ([`analysis_options`](Self::analysis_options)),
    /// reporting the resulting diagnostics through this driver's diagnostic
    /// engine ([`diag_engine`](Self::diag_engine)) — exactly as
    /// `slang::driver::Driver::runAnalysis`. Unlike [`compile`](Self::compile),
    /// `comp` is a plain, separately-built [`Compilation`] (not necessarily
    /// this driver's own parsed sources) that remains usable (still mutable,
    /// not turned into a [`Design`]) after the call.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let mut driver = sv_lang::Driver::new()?;
    /// let mut comp = sv_lang::Compilation::new(driver.session())?;
    /// comp.add_source("module m; logic x; initial x = 1; endmodule\n")?;
    /// let analysis = driver.run_analysis(&mut comp)?;
    /// let _ = analysis.diagnostics(); // e.g. unused-code lints, if enabled
    /// # Ok(()) }
    /// ```
    pub fn run_analysis<'c>(
        &mut self,
        comp: &'c mut Compilation,
    ) -> Result<DriverAnalysis<'c>, Error> {
        let mut err = ffi::error();
        // SAFETY: the driver and compilation are both valid; out-error checked.
        let raw = unsafe { sys::slang_driver_run_analysis(self.raw(), comp.raw(), &mut err) };
        ffi::check(&err)?;
        Ok(DriverAnalysis {
            raw,
            sm: comp.session().raw(),
            _comp: PhantomData,
        })
    }
}

/// One command file's metadata (see [`Driver::command_file_metadata`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandFileMetadata {
    /// The canonical path of the command file.
    pub path: String,
    /// The raw `-D` defines this command file contributed, in the form the
    /// file spelled them (`"NAME"` or `"NAME=VALUE"`).
    pub defines: Vec<String>,
}

/// A borrowed handle to a [`Driver`]'s diagnostic engine (see
/// [`Driver::diag_engine`]).
pub struct DiagEngine<'a> {
    raw: sys::slang_diag_engine,
    _driver: core::marker::PhantomData<&'a Driver>,
}

impl DiagEngine<'_> {
    /// The number of error-severity diagnostics issued through this engine so
    /// far.
    pub fn num_errors(&self) -> i32 {
        // SAFETY: `raw` is a valid handle borrowed from a live driver.
        unsafe { sys::slang_diag_engine_num_errors(self.raw) }
    }

    /// The number of warning-severity diagnostics issued through this engine
    /// so far.
    pub fn num_warnings(&self) -> i32 {
        // SAFETY: `raw` is a valid handle borrowed from a live driver.
        unsafe { sys::slang_diag_engine_num_warnings(self.raw) }
    }
}

/// A snapshot of the analysis options a driver's configured flags would
/// produce (see [`Driver::analysis_options`]). Raw `slang::analysis::AnalysisFlags`
/// bitmask encoding for `flags`, matching `sv_lang_sys::SLANG_ANALYSIS_*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnalysisOptions {
    /// Bitmask of `sv_lang_sys::SLANG_ANALYSIS_*` flags.
    pub flags: u32,
    /// The maximum number of case analysis steps to perform before giving up.
    pub max_case_analysis_steps: u32,
    /// The maximum number of loop analysis steps to perform before giving up.
    pub max_loop_analysis_steps: u32,
}

/// Options controlling how a command line is tokenized/parsed (see
/// [`Driver::parse_args_with_options`]). Every option defaults to `false`,
/// matching what [`Driver::parse_args`] uses.
///
/// ```
/// let mut options = sv_lang::ParseOptions::new();
/// assert!(!options.support_comments());
/// options.set_support_comments(true);
/// assert!(options.support_comments());
/// ```
pub struct ParseOptions {
    raw: sys::slang_parse_options,
}

impl Drop for ParseOptions {
    fn drop(&mut self) {
        // SAFETY: we own this handle exclusively.
        unsafe { sys::slang_parse_options_destroy(self.raw) };
    }
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl ParseOptions {
    /// Creates a parse-options set with every option `false`.
    ///
    /// # Panics
    /// Panics only if the library fails to allocate, which in practice means
    /// the process is out of memory.
    pub fn new() -> ParseOptions {
        let mut err = ffi::error();
        // SAFETY: standard construction; the out-error is checked.
        let raw = unsafe { sys::slang_parse_options_create(&mut err) };
        assert!(
            !raw.is_null(),
            "failed to create slang parse options: {}",
            ffi::message(&err)
        );
        ParseOptions { raw }
    }

    /// If set, comments (`#`/`//` line comments, `/* */` block comments) are
    /// parsed and ignored rather than treated as arguments.
    ///
    /// # Examples
    /// ```
    /// let mut options = sv_lang::ParseOptions::new();
    /// options.set_support_comments(true);
    /// assert!(options.support_comments());
    /// ```
    pub fn set_support_comments(&mut self, value: bool) -> &mut Self {
        // SAFETY: `raw` is a valid handle.
        unsafe { sys::slang_parse_options_set_support_comments(self.raw, value) };
        self
    }

    /// Whether [`set_support_comments`](Self::set_support_comments) is set.
    pub fn support_comments(&self) -> bool {
        // SAFETY: `raw` is a valid handle.
        unsafe { sys::slang_parse_options_support_comments(self.raw) }
    }

    /// If set, the first argument is NOT treated as the program name (by
    /// default, as in `argv`, it is).
    ///
    /// # Examples
    /// ```
    /// let mut options = sv_lang::ParseOptions::new();
    /// options.set_ignore_program_name(true);
    /// assert!(options.ignore_program_name());
    /// ```
    pub fn set_ignore_program_name(&mut self, value: bool) -> &mut Self {
        // SAFETY: `raw` is a valid handle.
        unsafe { sys::slang_parse_options_set_ignore_program_name(self.raw, value) };
        self
    }

    /// Whether [`set_ignore_program_name`](Self::set_ignore_program_name) is set.
    pub fn ignore_program_name(&self) -> bool {
        // SAFETY: `raw` is a valid handle.
        unsafe { sys::slang_parse_options_ignore_program_name(self.raw) }
    }

    /// If set, environment variables found in argument text are expanded
    /// (forms `$VAR`, `$(VAR)`, `${VAR}`).
    ///
    /// # Examples
    /// ```
    /// let mut options = sv_lang::ParseOptions::new();
    /// options.set_expand_env_vars(true);
    /// assert!(options.expand_env_vars());
    /// ```
    pub fn set_expand_env_vars(&mut self, value: bool) -> &mut Self {
        // SAFETY: `raw` is a valid handle.
        unsafe { sys::slang_parse_options_set_expand_env_vars(self.raw, value) };
        self
    }

    /// Whether [`set_expand_env_vars`](Self::set_expand_env_vars) is set.
    pub fn expand_env_vars(&self) -> bool {
        // SAFETY: `raw` is a valid handle.
        unsafe { sys::slang_parse_options_expand_env_vars(self.raw) }
    }

    /// If set, giving the same (non-list) option more than once does not
    /// raise a "duplicate argument" error — the later value wins.
    ///
    /// # Examples
    /// (see [`Driver::parse_args_with_options`])
    pub fn set_ignore_duplicates(&mut self, value: bool) -> &mut Self {
        // SAFETY: `raw` is a valid handle.
        unsafe { sys::slang_parse_options_set_ignore_duplicates(self.raw, value) };
        self
    }

    /// Whether [`set_ignore_duplicates`](Self::set_ignore_duplicates) is set.
    pub fn ignore_duplicates(&self) -> bool {
        // SAFETY: `raw` is a valid handle.
        unsafe { sys::slang_parse_options_ignore_duplicates(self.raw) }
    }
}

/// Options controlling how [`SourceLoader::load_sources`]'s underlying
/// `loadAndParseSources` divides files into compilation units and threads.
/// `num_threads` defaults to unset and every `bool` option defaults to
/// `false`.
///
/// ```
/// let mut options = sv_lang::SourceOptions::new();
/// assert_eq!(options.num_threads(), None);
/// assert!(!options.single_unit());
/// options.set_single_unit(true);
/// assert!(options.single_unit());
/// ```
pub struct SourceOptions {
    raw: sys::slang_source_options,
}

impl Drop for SourceOptions {
    fn drop(&mut self) {
        // SAFETY: we own this handle exclusively.
        unsafe { sys::slang_source_options_destroy(self.raw) };
    }
}

impl Default for SourceOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceOptions {
    /// Creates a source-options set with `num_threads` unset and every
    /// `bool` option `false`.
    ///
    /// # Panics
    /// Panics only if the library fails to allocate, which in practice means
    /// the process is out of memory.
    pub fn new() -> SourceOptions {
        let mut err = ffi::error();
        // SAFETY: standard construction; the out-error is checked.
        let raw = unsafe { sys::slang_source_options_create(&mut err) };
        assert!(
            !raw.is_null(),
            "failed to create slang source options: {}",
            ffi::message(&err)
        );
        SourceOptions { raw }
    }

    /// Sets the number of threads to use for loading and parsing. `None`
    /// (the default) lets slang pick.
    ///
    /// # Examples
    /// ```
    /// let mut options = sv_lang::SourceOptions::new();
    /// options.set_num_threads(Some(4));
    /// assert_eq!(options.num_threads(), Some(4));
    /// options.set_num_threads(None);
    /// assert_eq!(options.num_threads(), None);
    /// ```
    pub fn set_num_threads(&mut self, value: Option<u32>) -> &mut Self {
        // SAFETY: `raw` is a valid handle.
        unsafe {
            sys::slang_source_options_set_num_threads(self.raw, value.is_some(), value.unwrap_or(0))
        };
        self
    }

    /// The value set by [`set_num_threads`](Self::set_num_threads).
    pub fn num_threads(&self) -> Option<u32> {
        let mut value = 0u32;
        // SAFETY: `raw` is a valid handle; `value` is a valid out-param.
        let has_value = unsafe { sys::slang_source_options_num_threads(self.raw, &mut value) };
        has_value.then_some(value)
    }

    /// If set, all source files are treated as part of a single compilation
    /// unit, meaning all of their text is merged together.
    ///
    /// # Examples
    /// ```
    /// let mut options = sv_lang::SourceOptions::new();
    /// options.set_single_unit(true);
    /// assert!(options.single_unit());
    /// ```
    pub fn set_single_unit(&mut self, value: bool) -> &mut Self {
        // SAFETY: `raw` is a valid handle.
        unsafe { sys::slang_source_options_set_single_unit(self.raw, value) };
        self
    }

    /// Whether [`set_single_unit`](Self::set_single_unit) is set.
    pub fn single_unit(&self) -> bool {
        // SAFETY: `raw` is a valid handle.
        unsafe { sys::slang_source_options_single_unit(self.raw) }
    }

    /// If set, only lint the code rather than elaborating a full hierarchy.
    ///
    /// # Examples
    /// ```
    /// let mut options = sv_lang::SourceOptions::new();
    /// options.set_only_lint(true);
    /// assert!(options.only_lint());
    /// ```
    pub fn set_only_lint(&mut self, value: bool) -> &mut Self {
        // SAFETY: `raw` is a valid handle.
        unsafe { sys::slang_source_options_set_only_lint(self.raw, value) };
        self
    }

    /// Whether [`set_only_lint`](Self::set_only_lint) is set.
    pub fn only_lint(&self) -> bool {
        // SAFETY: `raw` is a valid handle.
        unsafe { sys::slang_source_options_only_lint(self.raw) }
    }

    /// If set, library files inherit macro definitions from primary source
    /// files.
    ///
    /// # Examples
    /// ```
    /// let mut options = sv_lang::SourceOptions::new();
    /// options.set_libraries_inherit_macros(true);
    /// assert!(options.libraries_inherit_macros());
    /// ```
    pub fn set_libraries_inherit_macros(&mut self, value: bool) -> &mut Self {
        // SAFETY: `raw` is a valid handle.
        unsafe { sys::slang_source_options_set_libraries_inherit_macros(self.raw, value) };
        self
    }

    /// Whether [`set_libraries_inherit_macros`](Self::set_libraries_inherit_macros)
    /// is set.
    pub fn libraries_inherit_macros(&self) -> bool {
        // SAFETY: `raw` is a valid handle.
        unsafe { sys::slang_source_options_libraries_inherit_macros(self.raw) }
    }
}

/// Flags controlling [`Driver::run_preprocessor`]'s output, combined with `|`.
///
/// # Examples
/// ```
/// use sv_lang::PreprocessFlags;
/// let flags = PreprocessFlags::INCLUDE_COMMENTS | PreprocessFlags::INCLUDE_DIRECTIVES;
/// assert!(flags.contains(PreprocessFlags::INCLUDE_COMMENTS));
/// assert!(!flags.contains(PreprocessFlags::OBFUSCATE_IDS));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PreprocessFlags(u32);

impl PreprocessFlags {
    /// No flags: comments and directives stripped, identifiers untouched.
    pub const NONE: PreprocessFlags = PreprocessFlags(0);
    /// Include comments in the output (otherwise stripped).
    pub const INCLUDE_COMMENTS: PreprocessFlags =
        PreprocessFlags(sys::SLANG_PREPROCESS_INCLUDE_COMMENTS);
    /// Include preprocessor directives in the output (otherwise stripped).
    pub const INCLUDE_DIRECTIVES: PreprocessFlags =
        PreprocessFlags(sys::SLANG_PREPROCESS_INCLUDE_DIRECTIVES);
    /// Obfuscate identifiers by replacing them with randomized alphanumeric
    /// strings.
    pub const OBFUSCATE_IDS: PreprocessFlags = PreprocessFlags(sys::SLANG_PREPROCESS_OBFUSCATE_IDS);
    /// With [`OBFUSCATE_IDS`](Self::OBFUSCATE_IDS), use a fixed randomization
    /// seed so obfuscated names are reproducible across runs.
    pub const USE_FIXED_OBFUSCATION_SEED: PreprocessFlags =
        PreprocessFlags(sys::SLANG_PREPROCESS_USE_FIXED_OBFUSCATION_SEED);
    /// Include source line information in the output.
    pub const INCLUDE_SOURCE_INFO: PreprocessFlags =
        PreprocessFlags(sys::SLANG_PREPROCESS_INCLUDE_SOURCE_INFO);

    /// The raw bitmask.
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// True if all of `other`'s flags are set.
    pub const fn contains(self, other: PreprocessFlags) -> bool {
        self.0 & other.0 == other.0
    }
}

impl core::ops::BitOr for PreprocessFlags {
    type Output = PreprocessFlags;
    fn bitor(self, rhs: PreprocessFlags) -> PreprocessFlags {
        PreprocessFlags(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for PreprocessFlags {
    fn bitor_assign(&mut self, rhs: PreprocessFlags) {
        self.0 |= rhs.0;
    }
}

/// A borrowed handle to a [`Driver`]'s source loader (see
/// [`Driver::source_loader`]), which handles loading and parsing groups of
/// source files: file-pattern globs, library maps, search directories, and
/// separately-compiled units.
pub struct SourceLoader<'a> {
    raw: sys::slang_source_loader,
    session: Session,
    _driver: PhantomData<&'a mut Driver>,
}

impl SourceLoader<'_> {
    /// Adds files to be loaded, specified via the given glob `pattern`. All
    /// files matching the pattern are queued for loading; if none match and
    /// the pattern is actually a specific filename, an error is recorded
    /// (surfaced the next time sources are parsed, not from this call). Must
    /// be called before [`Driver::process_options`], which requires at least
    /// one file already queued.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let dir = std::env::temp_dir().join("sv_lang_source_loader_add_files_doctest");
    /// std::fs::create_dir_all(&dir).unwrap();
    /// let file = dir.join("m.sv");
    /// std::fs::write(&file, "module m; endmodule\n").unwrap();
    ///
    /// let mut driver = sv_lang::Driver::new()?;
    /// driver.source_loader().add_files(&file.to_string_lossy())?;
    /// driver.process_options()?;
    /// assert!(driver.parse_sources()?);
    /// assert_eq!(driver.trees().len(), 1);
    ///
    /// std::fs::remove_dir_all(&dir).ok();
    /// # Ok(()) }
    /// ```
    pub fn add_files(&mut self, pattern: &str) -> Result<(), Error> {
        let mut err = ffi::error();
        let (p, pl) = ffi::as_ptr_len(pattern);
        // SAFETY: the loader is valid; `pattern` is valid for the call;
        // out-error checked.
        unsafe { sys::slang_source_loader_add_files(self.raw, p, pl, &mut err) };
        ffi::check(&err)
    }

    /// Adds library files to be loaded under the library named
    /// `library_name`, specified via the given glob `pattern`. Library files
    /// are only considered "used" if referenced from the main source; their
    /// modules are not automatically instantiated.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let dir = std::env::temp_dir().join("sv_lang_add_library_files_doctest");
    /// std::fs::create_dir_all(&dir).unwrap();
    /// let file = dir.join("lib_mod.sv");
    /// std::fs::write(&file, "module lib_mod; endmodule\n").unwrap();
    ///
    /// let mut driver = sv_lang::Driver::new()?;
    /// driver.source_loader().add_library_files("mylib", &file.to_string_lossy())?;
    /// driver.process_options()?;
    /// assert!(driver.parse_sources()?);
    ///
    /// std::fs::remove_dir_all(&dir).ok();
    /// # Ok(()) }
    /// ```
    pub fn add_library_files(&mut self, library_name: &str, pattern: &str) -> Result<(), Error> {
        let mut err = ffi::error();
        let (n, nl) = ffi::as_ptr_len(library_name);
        let (p, pl) = ffi::as_ptr_len(pattern);
        // SAFETY: the loader is valid; both strings are valid for the call;
        // out-error checked.
        unsafe { sys::slang_source_loader_add_library_files(self.raw, n, nl, p, pl, &mut err) };
        ffi::check(&err)
    }

    /// Loads and parses library map files matching `pattern`, creating the
    /// libraries they declare and queuing the files they reference for
    /// loading. `base_path` resolves relative paths within the map.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let dir = std::env::temp_dir().join("sv_lang_add_library_maps_doctest");
    /// std::fs::create_dir_all(&dir).unwrap();
    /// std::fs::write(dir.join("lib_mod.sv"), "module lib_mod; endmodule\n").unwrap();
    /// let map = dir.join("libs.map");
    /// std::fs::write(&map, "library mylib lib_mod.sv;\n").unwrap();
    ///
    /// let mut driver = sv_lang::Driver::new()?;
    /// driver
    ///     .source_loader()
    ///     .add_library_maps(&map.to_string_lossy(), &dir.to_string_lossy())?;
    /// driver.process_options()?;
    /// assert!(driver.parse_sources()?);
    ///
    /// std::fs::remove_dir_all(&dir).ok();
    /// # Ok(()) }
    /// ```
    pub fn add_library_maps(&mut self, pattern: &str, base_path: &str) -> Result<(), Error> {
        let mut err = ffi::error();
        let (p, pl) = ffi::as_ptr_len(pattern);
        let (b, bl) = ffi::as_ptr_len(base_path);
        // SAFETY: the loader is valid; both strings are valid for the call;
        // null options means slang's default parse options; out-error checked.
        unsafe {
            sys::slang_source_loader_add_library_maps(
                self.raw,
                p,
                pl,
                b,
                bl,
                core::ptr::null_mut(),
                &mut err,
            )
        };
        ffi::check(&err)
    }

    /// Adds directories in which to search for library module files,
    /// specified via the given glob `pattern`. A search for a library module
    /// occurs when there are instantiations found for unknown modules (or
    /// interfaces or programs); the given directories are searched for files
    /// named after the missing module plus any registered search extensions
    /// (see [`SourceLoader::add_search_extension`]).
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let dir = std::env::temp_dir().join("sv_lang_add_search_directories_doctest");
    /// std::fs::create_dir_all(&dir).unwrap();
    /// std::fs::write(dir.join("lib_mod.sv"), "module lib_mod; endmodule\n").unwrap();
    /// let top = dir.join("top.sv");
    /// std::fs::write(&top, "module top; lib_mod u(); endmodule\n").unwrap();
    ///
    /// let mut driver = sv_lang::Driver::new()?;
    /// driver.source_loader().add_files(&top.to_string_lossy())?;
    /// driver
    ///     .source_loader()
    ///     .add_search_directories(&dir.to_string_lossy())?;
    /// driver.process_options()?;
    /// assert!(driver.parse_sources()?);
    /// let design = driver.compile()?;
    /// assert_eq!(design.top_instances().count(), 1);
    ///
    /// std::fs::remove_dir_all(&dir).ok();
    /// # Ok(()) }
    /// ```
    pub fn add_search_directories(&mut self, pattern: &str) -> Result<(), Error> {
        let mut err = ffi::error();
        let (p, pl) = ffi::as_ptr_len(pattern);
        // SAFETY: the loader is valid; `pattern` is valid for the call;
        // out-error checked.
        unsafe { sys::slang_source_loader_add_search_directories(self.raw, p, pl, &mut err) };
        ffi::check(&err)
    }

    /// Adds a file extension (without the leading `.`) used when searching
    /// for library module files in directories registered via
    /// [`add_search_directories`](Self::add_search_directories). The
    /// extensions `.v` and `.sv` are always included automatically.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let mut driver = sv_lang::Driver::new()?;
    /// driver.source_loader().add_search_extension("vh")?;
    /// # Ok(()) }
    /// ```
    pub fn add_search_extension(&mut self, extension: &str) -> Result<(), Error> {
        let mut err = ffi::error();
        let (e, el) = ffi::as_ptr_len(extension);
        // SAFETY: the loader is valid; `extension` is valid for the call;
        // out-error checked.
        unsafe { sys::slang_source_loader_add_search_extension(self.raw, e, el, &mut err) };
        ffi::check(&err)
    }

    /// Adds a group of files as a separately compiled compilation unit.
    /// Unlike files added via [`add_files`](Self::add_files), every file
    /// matching one of the `file_patterns` globs is guaranteed to be grouped
    /// into a single compilation unit and preprocessed with the given
    /// `include_paths` and `defines` (each `NAME` or `NAME=VALUE`, without a
    /// leading `-D`). If `library_name` is non-empty the unit is included in
    /// the library of that name; otherwise it is included in the default
    /// library as a non-library unit. `warning_options` are `-W`-style
    /// strings (without the leading `-W`) applied to diagnostics from this
    /// unit.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let dir = std::env::temp_dir().join("sv_lang_add_separate_unit_doctest");
    /// std::fs::create_dir_all(&dir).unwrap();
    /// let file = dir.join("unit.sv");
    /// std::fs::write(&file, "module unit; endmodule\n").unwrap();
    ///
    /// let mut driver = sv_lang::Driver::new()?;
    /// driver.source_loader().add_separate_unit(
    ///     &[file.to_string_lossy().into_owned()],
    ///     &[],
    ///     &[],
    ///     "",
    ///     &[],
    /// )?;
    /// driver.process_options()?;
    /// assert!(driver.parse_sources()?);
    /// assert_eq!(driver.trees().len(), 1);
    ///
    /// std::fs::remove_dir_all(&dir).ok();
    /// # Ok(()) }
    /// ```
    pub fn add_separate_unit(
        &mut self,
        file_patterns: &[String],
        include_paths: &[String],
        defines: &[String],
        library_name: &str,
        warning_options: &[String],
    ) -> Result<(), Error> {
        // Keep every CString alive until after the call.
        let to_cstrings = |items: &[String]| -> Vec<std::ffi::CString> {
            items
                .iter()
                .map(|s| std::ffi::CString::new(s.as_str()).unwrap_or_default())
                .collect()
        };
        let to_ptrs = |owned: &[std::ffi::CString]| -> Vec<*const c_char> {
            owned.iter().map(|s| s.as_ptr()).collect()
        };

        let patterns_owned = to_cstrings(file_patterns);
        let patterns_ptrs = to_ptrs(&patterns_owned);
        let includes_owned = to_cstrings(include_paths);
        let includes_ptrs = to_ptrs(&includes_owned);
        let defines_owned = to_cstrings(defines);
        let defines_ptrs = to_ptrs(&defines_owned);
        let warnings_owned = to_cstrings(warning_options);
        let warnings_ptrs = to_ptrs(&warnings_owned);
        let (lib, lib_len) = ffi::as_ptr_len(library_name);

        let mut err = ffi::error();
        // SAFETY: the loader is valid; every array pointer is either null (for
        // a 0-length array) or valid+live for the call; out-error checked.
        unsafe {
            sys::slang_source_loader_add_separate_unit(
                self.raw,
                patterns_ptrs.as_ptr(),
                patterns_ptrs.len(),
                includes_ptrs.as_ptr(),
                includes_ptrs.len(),
                defines_ptrs.as_ptr(),
                defines_ptrs.len(),
                lib,
                lib_len,
                warnings_ptrs.as_ptr(),
                warnings_ptrs.len(),
                &mut err,
            )
        };
        ffi::check(&err)
    }

    /// True if there is at least one source file queued to load.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let mut driver = sv_lang::Driver::new()?;
    /// assert!(!driver.source_loader().has_files());
    ///
    /// let dir = std::env::temp_dir().join("sv_lang_has_files_doctest");
    /// std::fs::create_dir_all(&dir).unwrap();
    /// let file = dir.join("m.sv");
    /// std::fs::write(&file, "module m; endmodule\n").unwrap();
    /// driver.source_loader().add_files(&file.to_string_lossy())?;
    /// assert!(driver.source_loader().has_files());
    ///
    /// std::fs::remove_dir_all(&dir).ok();
    /// # Ok(()) }
    /// ```
    pub fn has_files(&self) -> bool {
        // SAFETY: the loader is valid.
        unsafe { sys::slang_source_loader_has_files(self.raw) }
    }

    /// The errors recorded while loading files so far (e.g. a glob pattern
    /// that matched nothing).
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let mut driver = sv_lang::Driver::new()?;
    /// assert!(driver.source_loader().errors().is_empty());
    /// // A literal (non-glob) filename that does not exist is an error; a
    /// // wildcard pattern that simply matches nothing is not.
    /// driver.source_loader().add_files("sv_lang_errors_doctest_missing.sv")?;
    /// assert!(!driver.source_loader().errors().is_empty());
    /// # Ok(()) }
    /// ```
    pub fn errors(&self) -> Vec<String> {
        // SAFETY: the loader is valid.
        let count = unsafe { sys::slang_source_loader_error_count(self.raw) };
        (0..count)
            .map(|i| {
                // SAFETY: index in range.
                let s = unsafe { sys::slang_source_loader_error_at(self.raw, i) };
                ffi::borrowed_str(s)
            })
            .collect()
    }

    /// The library map syntax trees loaded and parsed so far via
    /// [`add_library_maps`](Self::add_library_maps).
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let dir = std::env::temp_dir().join("sv_lang_library_maps_doctest");
    /// std::fs::create_dir_all(&dir).unwrap();
    /// std::fs::write(dir.join("lib_mod.sv"), "module lib_mod; endmodule\n").unwrap();
    /// let map = dir.join("libs.map");
    /// std::fs::write(&map, "library mylib lib_mod.sv;\n").unwrap();
    ///
    /// let mut driver = sv_lang::Driver::new()?;
    /// driver
    ///     .source_loader()
    ///     .add_library_maps(&map.to_string_lossy(), &dir.to_string_lossy())?;
    /// assert_eq!(driver.source_loader().library_maps().len(), 1);
    ///
    /// std::fs::remove_dir_all(&dir).ok();
    /// # Ok(()) }
    /// ```
    pub fn library_maps(&mut self) -> Vec<SyntaxTree> {
        // SAFETY: the loader is valid.
        let count = unsafe { sys::slang_source_loader_library_map_count(self.raw) };
        (0..count)
            .filter_map(|i| {
                // SAFETY: index in range.
                let raw = unsafe { sys::slang_source_loader_library_map_at(self.raw, i) };
                (!raw.is_null()).then(|| {
                    // The loader owns the tree; retain it for our handle so it
                    // is not freed while we hold it (see Driver::trees).
                    // SAFETY: `raw` is a valid tree handle.
                    unsafe { sys::slang_syntax_tree_retain(raw) };
                    SyntaxTree::from_raw(raw, self.session.clone())
                })
            })
            .collect()
    }

    /// Loads (but does not parse) every source that has been added to the
    /// loader so far, replacing any buffers loaded by a previous call.
    /// Returns each buffer's assigned ID paired with its source text.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let dir = std::env::temp_dir().join("sv_lang_load_sources_doctest");
    /// std::fs::create_dir_all(&dir).unwrap();
    /// let file = dir.join("m.sv");
    /// std::fs::write(&file, "module m; endmodule\n").unwrap();
    ///
    /// let mut driver = sv_lang::Driver::new()?;
    /// driver.source_loader().add_files(&file.to_string_lossy())?;
    /// let buffers = driver.source_loader().load_sources()?;
    /// assert_eq!(buffers.len(), 1);
    /// assert!(buffers[0].text.contains("module m"));
    /// assert!(buffers[0].id != 0);
    ///
    /// std::fs::remove_dir_all(&dir).ok();
    /// # Ok(()) }
    /// ```
    pub fn load_sources(&mut self) -> Result<Vec<LoadedSourceBuffer>, Error> {
        let mut err = ffi::error();
        // SAFETY: the loader is valid; out-error checked.
        let count = unsafe { sys::slang_source_loader_load_sources(self.raw, &mut err) };
        ffi::check(&err)?;
        Ok((0..count)
            .map(|i| {
                // SAFETY: index in range, valid immediately after the call above.
                let id = unsafe { sys::slang_source_loader_loaded_buffer_id(self.raw, i) };
                // SAFETY: as above.
                let text = unsafe { sys::slang_source_loader_loaded_buffer_text(self.raw, i) };
                LoadedSourceBuffer {
                    id,
                    text: ffi::borrowed_str(text),
                }
            })
            .collect())
    }
}

/// One source buffer loaded by [`SourceLoader::load_sources`]: its assigned
/// buffer ID and its source text.
#[derive(Debug, Clone)]
pub struct LoadedSourceBuffer {
    /// The buffer ID slang assigned this source (as tracked by its source
    /// manager).
    pub id: u32,
    /// The buffer's source text.
    pub text: String,
}

/// A borrowed handle to a [`Driver`]'s text diagnostic client (see
/// [`Driver::text_diag_client`]), which formats and accumulates every
/// diagnostic issued through the driver's diagnostic engine as
/// human-readable text.
pub struct TextDiagClient<'a> {
    raw: sys::slang_text_diag_client,
    _driver: PhantomData<&'a Driver>,
}

impl TextDiagClient<'_> {
    /// The client's currently accumulated formatted-text buffer.
    ///
    /// Note: [`Driver::new`]/[`new_bare`](Driver::new_bare) construct this
    /// client as a `StderrDiagnosticClient` (a `TextDiagnosticClient`
    /// subclass), which streams each diagnostic straight to the process's
    /// stderr and clears its own buffer immediately afterward — so this
    /// reads back empty right after a report call, even though real
    /// diagnostic text was printed (proven below via
    /// [`diag_engine`](Driver::diag_engine)'s error count).
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let dir = std::env::temp_dir().join("sv_lang_text_diag_client_doctest");
    /// std::fs::create_dir_all(&dir).unwrap();
    /// let bad = dir.join("bad.sv");
    /// std::fs::write(&bad, "module m; initial $display(nope); endmodule\n").unwrap();
    ///
    /// let mut driver = sv_lang::Driver::new()?;
    /// driver.parse_args(["slang".to_string(), bad.to_string_lossy().into_owned()])?;
    /// driver.process_options()?;
    /// driver.parse_sources()?;
    /// let design = driver.compile()?;
    /// assert_eq!(driver.diag_engine().num_errors(), 0);
    /// driver.report_compilation(&design); // streams the error to stderr, then clears
    /// assert!(driver.diag_engine().num_errors() > 0); // it really was reported
    /// assert!(driver.text_diag_client().get_string().is_empty()); // already cleared
    ///
    /// std::fs::remove_dir_all(&dir).ok();
    /// # Ok(()) }
    /// ```
    pub fn get_string(&self) -> String {
        let mut err = ffi::error();
        // SAFETY: `raw` is a valid handle borrowed from a live driver;
        // out-error checked.
        unsafe { ffi::owned_str(sys::slang_text_diag_client_get_string(self.raw, &mut err)) }
    }

    /// True if [`get_string`](Self::get_string) is empty — always true for
    /// this driver's client once any diagnostic has streamed through it (see
    /// [`get_string`](Self::get_string)), and trivially true before that.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let driver = sv_lang::Driver::new()?;
    /// assert!(driver.text_diag_client().empty()); // nothing has been reported yet
    /// # Ok(()) }
    /// ```
    pub fn empty(&self) -> bool {
        // SAFETY: `raw` is a valid handle borrowed from a live driver.
        unsafe { sys::slang_text_diag_client_empty(self.raw) }
    }
}

/// The result of [`Driver::run_analysis`]: analysis run over a plain
/// [`Compilation`] using a driver's configured options.
pub struct DriverAnalysis<'c> {
    raw: sys::slang_analysis,
    sm: sys::slang_source_manager,
    _comp: PhantomData<&'c mut Compilation>,
}

// SAFETY: the analysis result is immutable once produced and its only
// accessor is a read; it borrows an exclusively-held Compilation.
unsafe impl Send for DriverAnalysis<'_> {}

impl Drop for DriverAnalysis<'_> {
    fn drop(&mut self) {
        // SAFETY: we own this handle exclusively.
        unsafe { sys::slang_analysis_destroy(self.raw) };
    }
}

impl DriverAnalysis<'_> {
    /// The diagnostics analysis produced (already also reported through the
    /// driver's diagnostic engine by [`Driver::run_analysis`]).
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let mut driver = sv_lang::Driver::new()?;
    /// let mut comp = sv_lang::Compilation::new(driver.session())?;
    /// comp.add_source("module m; logic x; initial x = 1; endmodule\n")?;
    /// let analysis = driver.run_analysis(&mut comp)?;
    /// let _diags = analysis.diagnostics();
    /// # Ok(()) }
    /// ```
    pub fn diagnostics(&self) -> Diagnostics {
        let mut err = ffi::error();
        // SAFETY: `raw` is a valid handle owned by this struct.
        let raw = unsafe { sys::slang_analysis_diagnostics(self.raw, &mut err) };
        assert!(
            !raw.is_null(),
            "slang_analysis_diagnostics: {}",
            ffi::message(&err)
        );
        // `self.sm` is the source manager the borrowed compilation was built
        // with; `raw` is a valid owned handle, consumed by this call.
        crate::collect_diagnostics(self.sm, raw)
    }
}
