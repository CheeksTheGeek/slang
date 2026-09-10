//! The command-line driver: accept slang's own CLI flags (file lists, `-I`,
//! `-D`, `--top`, `-W...`, `--std`, diagnostics formatting, ...) and get back
//! the parsed trees and an elaborated design — exactly as the `slang` binary
//! does. This is the drop-in path for tools that shell out to `slang` today.

use core::ffi::c_char;
use std::sync::Arc;

use sv_lang_sys as sys;

use crate::{Design, Error, Session, SyntaxTree, ffi};

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

/// A driver over slang's standard command line.
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
}
