//! Error and diagnostic types.

use core::fmt;

use sv_lang_kinds::{DiagCode, DiagSeverity};
use sv_lang_sys as sys;

/// An error from a slang operation.
///
/// Parser and semantic *diagnostics* are not errors — they are collected in
/// [`Diagnostics`] and reported through the normal API. An `Error` means the
/// operation itself could not be carried out (a file could not be read, an
/// argument was invalid, the library ran out of memory, ...).
///
/// # Examples
/// ```
/// let session = sv_lang::Session::new();
/// // Reading a missing file is an operational error, not a diagnostic.
/// let err = session.parse_file("does-not-exist.sv").unwrap_err();
/// assert!(matches!(err, sv_lang::Error::Io(_)));
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Error {
    /// A file could not be read or written.
    Io(String),
    /// An argument was null, out of range, or otherwise invalid.
    InvalidArgument(String),
    /// The object was in the wrong state for the operation.
    InvalidState(String),
    /// The parser gave up on pathologically nested input.
    ParseRecursion(String),
    /// A syntax tree rewrite was rejected.
    Rewrite(String),
    /// Memory allocation failed.
    OutOfMemory,
    /// This build of the library does not support the operation.
    Unsupported(String),
    /// The linked library and these bindings disagree about the schema.
    AbiMismatch(String),
    /// A traversal was cancelled.
    Cancelled,
    /// A C++ exception with no more specific mapping was caught.
    Internal(String),
}

impl Error {
    pub(crate) fn from_status(status: sys::slang_status, message: String) -> Error {
        match status {
            sys::SLANG_ERR_IO => Error::Io(message),
            sys::SLANG_ERR_INVALID_ARG => Error::InvalidArgument(message),
            sys::SLANG_ERR_INVALID_STATE => Error::InvalidState(message),
            sys::SLANG_ERR_PARSE_RECURSION => Error::ParseRecursion(message),
            sys::SLANG_ERR_REWRITE => Error::Rewrite(message),
            sys::SLANG_ERR_OUT_OF_MEMORY => Error::OutOfMemory,
            sys::SLANG_ERR_UNSUPPORTED => Error::Unsupported(message),
            sys::SLANG_ERR_ABI_MISMATCH => Error::AbiMismatch(message),
            sys::SLANG_ERR_CANCELLED => Error::Cancelled,
            _ => Error::Internal(message),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(m) => write!(f, "I/O error: {m}"),
            Error::InvalidArgument(m) => write!(f, "invalid argument: {m}"),
            Error::InvalidState(m) => write!(f, "invalid state: {m}"),
            Error::ParseRecursion(m) => write!(f, "parser recursion limit: {m}"),
            Error::Rewrite(m) => write!(f, "rewrite rejected: {m}"),
            Error::OutOfMemory => f.write_str("out of memory"),
            Error::Unsupported(m) => write!(f, "unsupported: {m}"),
            Error::AbiMismatch(m) => write!(f, "ABI mismatch: {m}"),
            Error::Cancelled => f.write_str("cancelled"),
            Error::Internal(m) => write!(f, "internal error: {m}"),
        }
    }
}

impl std::error::Error for Error {}

/// One diagnostic produced by parsing or elaboration: an owned Rust value that
/// outlives the trees and compilations it came from.
///
/// # Examples
/// ```
/// # fn main() -> Result<(), sv_lang::Error> {
/// let session = sv_lang::Session::new();
/// let tree = session.parse("module m; assign x = missing; endmodule\n")?;
/// let mut comp = sv_lang::Compilation::new(&session)?;
/// comp.add(&tree)?;
/// let design = comp.compile()?;
/// for diag in design.diagnostics().items() {
///     println!("{}: {}", diag.code_name(), diag.message);
/// }
/// # Ok(()) }
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Diagnostic {
    /// The diagnostic's code, mapping to a name, default severity and option.
    pub code: DiagCode,
    /// The effective severity (after any option and pragma adjustments).
    pub severity: DiagSeverity,
    /// The fully formatted message with arguments substituted.
    pub message: String,
    /// 1-based line number of the primary location, if it has a file.
    pub line: usize,
    /// 1-based column number of the primary location.
    pub column: usize,
    /// The file name of the primary location.
    pub file: String,
    /// Byte offset of the primary span within [`source`](Self::source).
    pub byte_offset: usize,
    /// Byte length of the primary span (at least 1 for a point location).
    pub byte_len: usize,
    /// The full source text of the buffer this diagnostic points into, shared
    /// with the other diagnostics in the same buffer. Present when the location
    /// is in a real file (absent for macro-only or location-less diagnostics).
    pub source: Option<std::sync::Arc<str>>,
}

impl Diagnostic {
    /// True if this diagnostic is an error or fatal.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let mut comp = sv_lang::Compilation::new(&session)?;
    /// comp.add_source("module m; assign x = missing; endmodule\n")?;
    /// let design = comp.compile()?;
    /// let diags = design.diagnostics();
    /// assert!(diags.items().iter().any(|d| d.is_error()));
    /// # Ok(()) }
    /// ```
    pub fn is_error(&self) -> bool {
        matches!(self.severity, DiagSeverity::Error | DiagSeverity::Fatal)
    }

    /// The primary span as a byte range into [`source`](Self::source).
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; assign x = missing; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let diags = design.diagnostics();
    /// let d = diags.items().iter().find(|d| d.is_error()).unwrap();
    /// let span = d.span();
    /// assert!(span.end > span.start); // a span is at least one byte wide
    /// # Ok(()) }
    /// ```
    pub fn span(&self) -> core::ops::Range<usize> {
        self.byte_offset..self.byte_offset + self.byte_len
    }

    /// The diagnostic's symbolic name, e.g. `"UnknownModule"`.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; assign x = missing; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let diags = design.diagnostics();
    /// let d = diags.items().iter().find(|d| d.is_error()).unwrap();
    /// assert!(!d.code_name().is_empty());
    /// # Ok(()) }
    /// ```
    pub fn code_name(&self) -> &'static str {
        self.code.name()
    }
}

impl std::error::Error for Diagnostic {}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sev = severity_label(self.severity);
        if self.file.is_empty() {
            write!(f, "{sev}: {}", self.message)
        } else {
            write!(
                f,
                "{}:{}:{}: {sev}: {}",
                self.file, self.line, self.column, self.message
            )
        }
    }
}

/// An owned list of diagnostics, with the fully rendered text slang would print.
///
/// # Examples
/// ```
/// # fn main() -> Result<(), sv_lang::Error> {
/// let session = sv_lang::Session::new();
/// let tree = session.parse("module m; endmodule\n")?;
/// let diags = tree.diagnostics();
/// assert!(!diags.has_errors());
/// # Ok(()) }
/// ```
#[derive(Debug, Clone, Default)]
pub struct Diagnostics {
    pub(crate) items: Vec<Diagnostic>,
    pub(crate) rendered: String,
}

impl Diagnostics {
    /// The diagnostics, in report order.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let tree = session.parse("module m; endmodule\n")?;
    /// assert_eq!(tree.diagnostics().items().len(), tree.diagnostics().len());
    /// # Ok(()) }
    /// ```
    pub fn items(&self) -> &[Diagnostic] {
        &self.items
    }

    /// True if any diagnostic is an error or fatal.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let mut comp = sv_lang::Compilation::new(&session)?;
    /// comp.add_source("module m; assign x = missing; endmodule\n")?;
    /// let design = comp.compile()?;
    /// assert!(design.diagnostics().has_errors());
    /// # Ok(()) }
    /// ```
    pub fn has_errors(&self) -> bool {
        self.items.iter().any(Diagnostic::is_error)
    }

    /// The number of error/fatal diagnostics.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let mut comp = sv_lang::Compilation::new(&session)?;
    /// comp.add_source("module m; assign x = missing; endmodule\n")?;
    /// let design = comp.compile()?;
    /// assert!(design.diagnostics().error_count() >= 1);
    /// # Ok(()) }
    /// ```
    pub fn error_count(&self) -> usize {
        self.items.iter().filter(|d| d.is_error()).count()
    }

    /// slang's own rendered text for the whole list (with source snippets and
    /// carets), ready to print.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let mut comp = sv_lang::Compilation::new(&session)?;
    /// comp.add_source("module m; assign x = missing; endmodule\n")?;
    /// let design = comp.compile()?;
    /// print!("{}", design.diagnostics().rendered());
    /// # Ok(()) }
    /// ```
    pub fn rendered(&self) -> &str {
        &self.rendered
    }

    /// Consumes the list, yielding the owned diagnostics.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let session = sv_lang::Session::new();
    /// let mut comp = sv_lang::Compilation::new(&session)?;
    /// comp.add_source("module m; assign x = missing; endmodule\n")?;
    /// let design = comp.compile()?;
    /// let items = design.diagnostics().into_items();
    /// assert!(items.iter().any(|d| d.is_error()));
    /// # Ok(()) }
    /// ```
    pub fn into_items(self) -> Vec<Diagnostic> {
        self.items
    }
}

impl core::ops::Deref for Diagnostics {
    type Target = [Diagnostic];
    fn deref(&self) -> &[Diagnostic] {
        &self.items
    }
}

impl fmt::Display for Diagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.rendered)
    }
}

fn severity_label(severity: DiagSeverity) -> &'static str {
    match severity {
        DiagSeverity::Ignored => "ignored",
        DiagSeverity::Note => "note",
        DiagSeverity::Warning => "warning",
        DiagSeverity::Error => "error",
        DiagSeverity::Fatal => "fatal",
    }
}

// ---- miette integration -----------------------------------------------------

#[cfg(feature = "miette")]
mod miette_impl {
    use super::{DiagSeverity, Diagnostic};

    impl miette::Diagnostic for Diagnostic {
        fn code(&self) -> Option<Box<dyn core::fmt::Display + '_>> {
            Some(Box::new(self.code.name()))
        }

        fn severity(&self) -> Option<miette::Severity> {
            Some(match self.severity {
                DiagSeverity::Error | DiagSeverity::Fatal => miette::Severity::Error,
                DiagSeverity::Warning => miette::Severity::Warning,
                _ => miette::Severity::Advice,
            })
        }

        fn source_code(&self) -> Option<&dyn miette::SourceCode> {
            // miette impls SourceCode for Arc<str>, so borrow the Arc directly.
            self.source.as_ref().map(|s| s as &dyn miette::SourceCode)
        }

        fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
            self.source.as_ref()?;
            let span = miette::LabeledSpan::new(
                Some(self.message.clone()),
                self.byte_offset,
                self.byte_len,
            );
            Some(Box::new(core::iter::once(span)))
        }
    }
}

#[cfg(feature = "miette")]
impl Diagnostic {
    /// Wraps this diagnostic as a [`miette::Report`] for pretty rendering with
    /// source snippets. Requires the `miette` feature and a diagnostic that
    /// carries source (parser/semantic diagnostics from a real file do).
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; assign x = missing; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let d = design.diagnostics().into_items().into_iter().find(|d| d.is_error()).unwrap();
    /// let report = d.into_miette();
    /// assert!(!format!("{report:?}").is_empty());
    /// # Ok(()) }
    /// ```
    pub fn into_miette(self) -> miette::Report {
        miette::Report::new(self)
    }
}

// ---- ariadne integration ----------------------------------------------------

#[cfg(feature = "ariadne")]
impl Diagnostic {
    /// Renders this diagnostic to a string using the
    /// [`ariadne`](https://crates.io/crates/ariadne) reporter, with a source
    /// snippet and an underlined span. Returns the plain message if the
    /// diagnostic carries no source. Requires the `ariadne` feature.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; assign x = missing; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let d = design.diagnostics().into_items().into_iter().find(|d| d.is_error()).unwrap();
    /// assert!(!d.to_ariadne_string().is_empty());
    /// # Ok(()) }
    /// ```
    pub fn to_ariadne_string(&self) -> String {
        use ariadne::{Color, Label, Report, ReportKind, Source};

        let Some(source) = self.source.as_ref() else {
            return format!("{self}");
        };
        let kind = match self.severity {
            DiagSeverity::Error | DiagSeverity::Fatal => ReportKind::Error,
            DiagSeverity::Warning => ReportKind::Warning,
            _ => ReportKind::Advice,
        };
        let name = if self.file.is_empty() {
            "source"
        } else {
            self.file.as_str()
        };
        let mut buf = Vec::new();
        let report = Report::build(kind, (name, self.span()))
            .with_code(self.code.name())
            .with_message(&self.message)
            .with_label(
                Label::new((name, self.span()))
                    .with_message(severity_label(self.severity))
                    .with_color(Color::Red),
            )
            .finish();
        let _ = report.write((name, Source::from(source.as_ref())), &mut buf);
        String::from_utf8_lossy(&buf).into_owned()
    }
}

#[cfg(feature = "ariadne")]
impl Diagnostics {
    /// Renders every diagnostic with [`ariadne`](https://crates.io/crates/ariadne),
    /// concatenated. Requires the `ariadne` feature.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; assign x = missing; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let out = design.diagnostics().to_ariadne_string();
    /// assert!(!out.is_empty());
    /// # Ok(()) }
    /// ```
    pub fn to_ariadne_string(&self) -> String {
        self.items
            .iter()
            .map(Diagnostic::to_ariadne_string)
            .collect()
    }
}
