//! Diagnostic module - Error and warning reporting infrastructure.
//!
//! This module provides types for creating, formatting, and reporting
//! compiler diagnostics (errors, warnings, notes, and help messages).
//!
//! # Examples
//!
//! ## Using the simple API (deprecated but still supported)
//!
//! ```
//! use faxc_util::diagnostic::{Handler, Span};
//!
//! let handler = Handler::new();
//! handler.error("unexpected token", Span::DUMMY);
//!
//! if handler.has_errors() {
//!     eprintln!("Compilation failed");
//! }
//! ```
//!
//! ## Using the fluent builder API (recommended)
//!
//! ```
//! use faxc_util::diagnostic::{DiagnosticBuilder, Span, DiagnosticCode};
//!
//! let diag = DiagnosticBuilder::error("unexpected token")
//!     .code(DiagnosticCode::new(2001, "unexpected_token"))
//!     .span(Span::DUMMY)
//!     .help("try checking the syntax")
//!     .build();
//! ```

mod builder;
mod codes;

pub use builder::{DiagnosticBuilder, SourceSnippet};
pub use codes::DiagnosticCode;

// Re-export commonly used diagnostic codes as constants for convenience
pub use codes::{
    E0001, E0002, E0003, E0004, E0005,
    E_LEXER_UNEXPECTED_CHAR, E_LEXER_UNTERMINATED_STRING, E_LEXER_INVALID_NUMBER, E_LEXER_UNKNOWN_TOKEN,
    E_PARSER_UNEXPECTED_TOKEN, E_PARSER_EXPECTED_TOKEN, E_PARSER_UNEXPECTED_EOF, E_PARSER_DUPLICATE_DEF,
    E_SEMANTIC_TYPE_MISMATCH, E_SEMANTIC_UNDEFINED_VAR, E_SEMANTIC_UNDEFINED_FN, E_SEMANTIC_MUT_REQUIRED,
    W0001, W0002, W0003,
    W_UNUSED_VARIABLE, W_UNUSED_FUNCTION, W_DEAD_CODE,
};

use crate::Span;
use std::cell::RefCell;
use std::fmt;

/// Diagnostic severity level
///
/// # Examples
///
/// ```
/// use faxc_util::diagnostic::Level;
///
/// assert_eq!(format!("{}", Level::Error), "error");
/// assert_eq!(format!("{}", Level::Warning), "warning");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    /// An error that prevents compilation
    Error,
    /// A warning that doesn't prevent compilation
    Warning,
    /// Additional information about a diagnostic
    Note,
    /// A suggestion for fixing an issue
    Help,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Level::Error => write!(f, "error"),
            Level::Warning => write!(f, "warning"),
            Level::Note => write!(f, "note"),
            Level::Help => write!(f, "help"),
        }
    }
}

/// A diagnostic message with severity and location
///
/// # Examples
///
/// ```
/// use faxc_util::diagnostic::{Diagnostic, Level, Span};
///
/// let diag = Diagnostic::error("something went wrong", Span::DUMMY);
/// assert_eq!(diag.level, Level::Error);
/// ```
#[derive(Clone, Debug)]
pub struct Diagnostic {
    /// Diagnostic severity level
    pub level: Level,
    /// Main diagnostic message
    pub message: String,
    /// Source location
    pub span: Span,
    /// Optional diagnostic code
    pub code: Option<DiagnosticCode>,
    /// Additional notes for context
    pub notes: Vec<String>,
    /// Help suggestions for fixing the issue
    pub helps: Vec<String>,
    /// Source code snippets for display
    pub snippets: Vec<SourceSnippet>,
}

impl Diagnostic {
    /// Create a new diagnostic
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::{Diagnostic, Level, Span};
    ///
    /// let diag = Diagnostic::new(Level::Error, "error message", Span::DUMMY);
    /// ```
    pub fn new(level: Level, message: impl Into<String>, span: Span) -> Self {
        Self {
            level,
            message: message.into(),
            span,
            code: None,
            notes: Vec::new(),
            helps: Vec::new(),
            snippets: Vec::new(),
        }
    }

    /// Create an error diagnostic
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::{Diagnostic, Span};
    ///
    /// let diag = Diagnostic::error("something went wrong", Span::DUMMY);
    /// ```
    pub fn error(message: impl Into<String>, span: Span) -> Self {
        Self::new(Level::Error, message, span)
    }

    /// Create a warning diagnostic
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::{Diagnostic, Span};
    ///
    /// let diag = Diagnostic::warning("unused variable", Span::DUMMY);
    /// ```
    pub fn warning(message: impl Into<String>, span: Span) -> Self {
        Self::new(Level::Warning, message, span)
    }

    /// Set the diagnostic code
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::{Diagnostic, Span, DiagnosticCode};
    ///
    /// let mut diag = Diagnostic::error("type error", Span::DUMMY);
    /// diag = diag.with_code(DiagnosticCode::new(3001, "type_mismatch"));
    /// ```
    pub fn with_code(mut self, code: DiagnosticCode) -> Self {
        self.code = Some(code);
        self
    }

    /// Add a note to the diagnostic
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::{Diagnostic, Span};
    ///
    /// let diag = Diagnostic::error("type mismatch", Span::DUMMY)
    ///     .with_note("expected type `i32`")
    ///     .with_note("found type `String`");
    /// ```
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Add a help suggestion
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::{Diagnostic, Span};
    ///
    /// let diag = Diagnostic::error("unexpected token", Span::DUMMY)
    ///     .with_help("try removing the semicolon");
    /// ```
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.helps.push(help.into());
        self
    }

    /// Add a source snippet
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::{Diagnostic, Span, SourceSnippet};
    ///
    /// let snippet = SourceSnippet::point("let x = 42;", 1, 5);
    /// let diag = Diagnostic::error("invalid name", Span::DUMMY)
    ///     .with_snippet(snippet);
    /// ```
    pub fn with_snippet(mut self, snippet: SourceSnippet) -> Self {
        self.snippets.push(snippet);
        self
    }
}

/// Handler for collecting and reporting diagnostics
///
/// The `Handler` collects diagnostics and provides methods for querying
/// their counts. It can be configured to panic on errors for testing.
///
/// # Examples
///
/// ```
/// use faxc_util::diagnostic::Handler;
///
/// let mut handler = Handler::new();
/// handler.error("unexpected token", Span::DUMMY);
///
/// if handler.has_errors() {
///     eprintln!("Compilation failed with {} errors", handler.error_count());
/// }
/// ```
pub struct Handler {
    /// Collected diagnostics
    diagnostics: RefCell<Vec<Diagnostic>>,
    /// Whether to panic on errors (for testing)
    panic_on_error: RefCell<bool>,
}

impl Handler {
    /// Create a new handler
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::Handler;
    ///
    /// let handler = Handler::new();
    /// ```
    pub fn new() -> Self {
        Self {
            diagnostics: RefCell::new(Vec::new()),
            panic_on_error: RefCell::new(false),
        }
    }

    /// Create a handler that panics on errors (for testing)
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::Handler;
    ///
    /// let handler = Handler::new_panicking();
    /// ```
    pub fn new_panicking() -> Self {
        Self {
            diagnostics: RefCell::new(Vec::new()),
            panic_on_error: RefCell::new(true),
        }
    }

    /// Report an error
    ///
    /// This is the legacy API. For more control, use [`DiagnosticBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::{Handler, Span};
    ///
    /// let handler = Handler::new();
    /// handler.error("unexpected token", Span::DUMMY);
    /// ```
    #[deprecated(since = "0.2.0", note = "Use `DiagnosticBuilder::error()` for more control")]
    pub fn error(&self, message: impl Into<String>, span: Span) {
        let diag = Diagnostic::error(message, span);
        self.emit(diag);
    }

    /// Report a warning
    ///
    /// This is the legacy API. For more control, use [`DiagnosticBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::{Handler, Span};
    ///
    /// let handler = Handler::new();
    /// handler.warning("unused variable", Span::DUMMY);
    /// ```
    #[deprecated(since = "0.2.0", note = "Use `DiagnosticBuilder::warning()` for more control")]
    pub fn warning(&self, message: impl Into<String>, span: Span) {
        let diag = Diagnostic::warning(message, span);
        self.emit(diag);
    }

    /// Emit a diagnostic
    fn emit(&self, diagnostic: Diagnostic) {
        if *self.panic_on_error.borrow() && diagnostic.level == Level::Error {
            panic!("Diagnostic error: {}", diagnostic.message);
        }
        self.diagnostics.borrow_mut().push(diagnostic);
    }

    /// Emit a pre-built diagnostic
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::{Handler, Diagnostic, Span};
    ///
    /// let handler = Handler::new();
    /// let diag = Diagnostic::error("test", Span::DUMMY);
    /// handler.emit_diagnostic(diag);
    /// ```
    pub fn emit_diagnostic(&self, diagnostic: Diagnostic) {
        self.emit(diagnostic);
    }

    /// Create a diagnostic builder for an error
    ///
    /// This is the recommended way to create error diagnostics with
    /// the fluent builder API.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::Handler;
    ///
    /// let handler = Handler::new();
    /// handler.build_error("type mismatch")
    ///     .code(faxc_util::diagnostic::DiagnosticCode::E0002)
    ///     .emit(&handler);
    /// ```
    pub fn build_error(&self, span: Span, message: impl Into<String>) -> DiagnosticBuilder {
        DiagnosticBuilder::error(message).span(span)
    }

    /// Create a diagnostic builder for a warning
    ///
    /// This is the recommended way to create warning diagnostics with
    /// the fluent builder API.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::Handler;
    ///
    /// let handler = Handler::new();
    /// handler.build_warning("unused variable")
    ///     .code(faxc_util::diagnostic::DiagnosticCode::W0001)
    ///     .emit(&handler);
    /// ```
    pub fn build_warning(&self, span: Span, message: impl Into<String>) -> DiagnosticBuilder {
        DiagnosticBuilder::warning(message).span(span)
    }

    /// Check if any errors have been reported
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::Handler;
    ///
    /// let handler = Handler::new();
    /// assert!(!handler.has_errors());
    /// ```
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .borrow()
            .iter()
            .any(|d| d.level == Level::Error)
    }

    /// Get the number of errors
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::Handler;
    ///
    /// let handler = Handler::new();
    /// assert_eq!(handler.error_count(), 0);
    /// ```
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .borrow()
            .iter()
            .filter(|d| d.level == Level::Error)
            .count()
    }

    /// Get the number of warnings
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::Handler;
    ///
    /// let handler = Handler::new();
    /// assert_eq!(handler.warning_count(), 0);
    /// ```
    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .borrow()
            .iter()
            .filter(|d| d.level == Level::Warning)
            .count()
    }

    /// Get all diagnostics
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::Handler;
    ///
    /// let handler = Handler::new();
    /// let diags = handler.diagnostics();
    /// ```
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        self.diagnostics.borrow().clone()
    }

    /// Clear all diagnostics
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::Handler;
    ///
    /// let handler = Handler::new();
    /// handler.clear();
    /// ```
    pub fn clear(&self) {
        self.diagnostics.borrow_mut().clear();
    }
}

impl Default for Handler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;

    #[test]
    fn test_level_display() {
        assert_eq!(format!("{}", Level::Error), "error");
        assert_eq!(format!("{}", Level::Warning), "warning");
        assert_eq!(format!("{}", Level::Note), "note");
        assert_eq!(format!("{}", Level::Help), "help");
    }

    #[test]
    fn test_diagnostic_new() {
        let diag = Diagnostic::new(Level::Error, "test", Span::DUMMY);
        assert_eq!(diag.level, Level::Error);
        assert_eq!(diag.message, "test");
    }

    #[test]
    fn test_diagnostic_error() {
        let diag = Diagnostic::error("error message", Span::DUMMY);
        assert_eq!(diag.level, Level::Error);
    }

    #[test]
    fn test_diagnostic_warning() {
        let diag = Diagnostic::warning("warning message", Span::DUMMY);
        assert_eq!(diag.level, Level::Warning);
    }

    #[test]
    fn test_diagnostic_with_code() {
        let code = DiagnosticCode::new(1001, "test");
        let diag = Diagnostic::error("test", Span::DUMMY).with_code(code);
        assert_eq!(diag.code, Some(code));
    }

    #[test]
    fn test_diagnostic_with_note() {
        let diag = Diagnostic::error("test", Span::DUMMY)
            .with_note("note 1")
            .with_note("note 2");
        assert_eq!(diag.notes, vec!["note 1", "note 2"]);
    }

    #[test]
    fn test_diagnostic_with_help() {
        let diag = Diagnostic::error("test", Span::DUMMY)
            .with_help("help 1")
            .with_help("help 2");
        assert_eq!(diag.helps, vec!["help 1", "help 2"]);
    }

    #[test]
    fn test_handler_new() {
        let handler = Handler::new();
        assert!(!handler.has_errors());
        assert_eq!(handler.error_count(), 0);
        assert_eq!(handler.warning_count(), 0);
    }

    #[test]
    #[allow(deprecated)]
    fn test_handler_error() {
        let handler = Handler::new();
        handler.error("test error", Span::DUMMY);
        assert!(handler.has_errors());
        assert_eq!(handler.error_count(), 1);
    }

    #[test]
    #[allow(deprecated)]
    fn test_handler_warning() {
        let handler = Handler::new();
        handler.warning("test warning", Span::DUMMY);
        assert!(!handler.has_errors());
        assert_eq!(handler.warning_count(), 1);
    }

    #[test]
    fn test_handler_emit_diagnostic() {
        let handler = Handler::new();
        let diag = Diagnostic::error("test", Span::DUMMY);
        handler.emit_diagnostic(diag);
        assert!(handler.has_errors());
    }

    #[test]
    fn test_handler_clear() {
        let handler = Handler::new();
        handler.emit_diagnostic(Diagnostic::error("test", Span::DUMMY));
        handler.clear();
        assert!(!handler.has_errors());
        assert_eq!(handler.error_count(), 0);
    }

    #[test]
    fn test_handler_diagnostics() {
        let handler = Handler::new();
        handler.emit_diagnostic(Diagnostic::error("test1", Span::DUMMY));
        handler.emit_diagnostic(Diagnostic::warning("test2", Span::DUMMY));

        let diags = handler.diagnostics();
        assert_eq!(diags.len(), 2);
    }

    #[test]
    fn test_handler_panicking() {
        let handler = Handler::new_panicking();
        let result = std::panic::catch_unwind(|| {
            handler.emit_diagnostic(Diagnostic::error("test", Span::DUMMY));
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_handler_build_error() {
        let handler = Handler::new();
        handler.build_error(Span::DUMMY, "test error")
            .code(DiagnosticCode::E0001)
            .emit(&handler);

        assert!(handler.has_errors());
        assert_eq!(handler.error_count(), 1);

        let diags = handler.diagnostics();
        assert_eq!(diags[0].code, Some(DiagnosticCode::E0001));
    }

    #[test]
    fn test_handler_build_warning() {
        let handler = Handler::new();
        handler.build_warning(Span::DUMMY, "test warning")
            .code(DiagnosticCode::W0001)
            .emit(&handler);

        assert!(!handler.has_errors());
        assert_eq!(handler.warning_count(), 1);

        let diags = handler.diagnostics();
        assert_eq!(diags[0].code, Some(DiagnosticCode::W0001));
    }

    #[test]
    fn test_handler_build_with_note_and_help() {
        let handler = Handler::new();
        handler.build_error(Span::DUMMY, "type mismatch")
            .code(DiagnosticCode::E0002)
            .with_note("expected `i32`")
            .with_help("try adding a type annotation")
            .emit(&handler);

        let diags = handler.diagnostics();
        assert_eq!(diags[0].notes, vec!["expected `i32`"]);
        assert_eq!(diags[0].helps, vec!["try adding a type annotation"]);
    }
}
