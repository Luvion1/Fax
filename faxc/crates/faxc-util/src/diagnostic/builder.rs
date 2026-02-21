//! Diagnostic builder for fluent diagnostic construction.
//!
//! This module provides the [`DiagnosticBuilder`] type for constructing
//! diagnostics with a fluent API, including source code snippets.

use super::{Diagnostic, DiagnosticCode, Level, Span};

/// A source code snippet for display in diagnostics
///
/// Contains the source line(s) affected by the diagnostic, with optional
/// highlighting of the specific range.
///
/// # Examples
///
/// ```
/// use faxc_util::diagnostic::SourceSnippet;
///
/// let snippet = SourceSnippet::new(
///     "let x = 42;",
///     5,
///     1,
///     12,
///     Some("expected integer, found string"),
/// );
/// ```
#[derive(Clone, Debug)]
pub struct SourceSnippet {
    /// The source line content
    pub line: String,
    /// Line number (1-based)
    pub line_number: usize,
    /// Column where the issue starts (1-based)
    pub start_column: usize,
    /// Column where the issue ends (1-based)
    pub end_column: usize,
    /// Optional label to display under the highlighted range
    pub label: Option<String>,
}

impl SourceSnippet {
    /// Create a new source snippet
    ///
    /// # Arguments
    ///
    /// * `line` - The source line content
    /// * `line_number` - Line number (1-based)
    /// * `start_column` - Column where the issue starts (1-based)
    /// * `end_column` - Column where the issue ends (1-based)
    /// * `label` - Optional label to display under the highlighted range
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::SourceSnippet;
    ///
    /// let snippet = SourceSnippet::new(
    ///     "let x = 42;",
    ///     1,
    ///     5,
    ///     6,
    ///     Some("variable name"),
    /// );
    /// ```
    pub fn new(
        line: impl Into<String>,
        line_number: usize,
        start_column: usize,
        end_column: usize,
        label: Option<impl Into<String>>,
    ) -> Self {
        Self {
            line: line.into(),
            line_number,
            start_column,
            end_column,
            label: label.map(Into::into),
        }
    }

    /// Create a snippet without highlighting (point span)
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::SourceSnippet;
    ///
    /// let snippet = SourceSnippet::point(
    ///     "let x = 42;",
    ///     1,
    ///     5,
    /// );
    /// ```
    pub fn point(line: impl Into<String>, line_number: usize, column: usize) -> Self {
        Self {
            line: line.into(),
            line_number,
            start_column: column,
            end_column: column,
            label: None,
        }
    }

    /// Set the label for this snippet
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::SourceSnippet;
    ///
    /// let snippet = SourceSnippet::new(
    ///     "let x = 42;",
    ///     1,
    ///     5,
    ///     6,
    ///     None,
    /// ).with_label("variable name");
    /// ```
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Format the snippet for display
    ///
    /// Returns a formatted string showing the source line with a caret (^)
    /// pointing to the relevant range.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::SourceSnippet;
    ///
    /// let snippet = SourceSnippet::new(
    ///     "let x = 42;",
    ///     1,
    ///     5,
    ///     6,
    ///     Some("here"),
    /// );
    /// let formatted = snippet.format();
    /// ```
    pub fn format(&self) -> String {
        let line_num_width = self.line_number.to_string().len().max(3);
        let mut result = String::new();

        // Line number and source
        result.push_str(&format!(
            "{:>width$} | {}\n",
            self.line_number,
            self.line,
            width = line_num_width
        ));

        // Caret line
        result.push_str(&format!("{:>width$} | ", "", width = line_num_width));

        // Calculate underline position (accounting for tab stops if needed)
        let underline_start = self.start_column.saturating_sub(1);
        let underline_len = (self.end_column - self.start_column).max(1);

        // Add spaces before caret
        for _ in 0..underline_start {
            result.push(' ');
        }

        // Add carets
        for _ in 0..underline_len {
            result.push('^');
        }

        // Add label if present
        if let Some(ref label) = self.label {
            result.push_str(&format!(" {}", label));
        }

        result
    }
}

/// Builder for constructing diagnostics with a fluent API
///
/// # Examples
///
/// ```
/// use faxc_util::diagnostic::{DiagnosticBuilder, Level, Span, DiagnosticCode};
///
/// let diag = DiagnosticBuilder::new(Level::Error, "unexpected token")
///     .code(DiagnosticCode::new(2001, "unexpected_token"))
///     .span(Span::DUMMY)
///     .help("try removing the extra character")
///     .build();
/// ```
pub struct DiagnosticBuilder {
    level: Level,
    message: String,
    span: Span,
    code: Option<DiagnosticCode>,
    notes: Vec<String>,
    helps: Vec<String>,
    snippets: Vec<SourceSnippet>,
}

impl DiagnosticBuilder {
    /// Create a new diagnostic builder
    ///
    /// # Arguments
    ///
    /// * `level` - The diagnostic level (error, warning, etc.)
    /// * `message` - The main diagnostic message
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::{DiagnosticBuilder, Level};
    ///
    /// let builder = DiagnosticBuilder::new(Level::Error, "something went wrong");
    /// ```
    pub fn new(level: Level, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            span: Span::DUMMY,
            code: None,
            notes: Vec::new(),
            helps: Vec::new(),
            snippets: Vec::new(),
        }
    }

    /// Create an error builder
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::DiagnosticBuilder;
    ///
    /// let builder = DiagnosticBuilder::error("unexpected token");
    /// ```
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(Level::Error, message)
    }

    /// Create a warning builder
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::DiagnosticBuilder;
    ///
    /// let builder = DiagnosticBuilder::warning("unused variable");
    /// ```
    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(Level::Warning, message)
    }

    /// Set the diagnostic code
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::{DiagnosticBuilder, DiagnosticCode};
    ///
    /// let builder = DiagnosticBuilder::error("type mismatch")
    ///     .code(DiagnosticCode::new(3001, "type_mismatch"));
    /// ```
    pub fn code(mut self, code: DiagnosticCode) -> Self {
        self.code = Some(code);
        self
    }

    /// Set the source span
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::{DiagnosticBuilder, Span};
    ///
    /// let span = Span::new(10, 20, 1, 5);
    /// let builder = DiagnosticBuilder::error("error here").span(span);
    /// ```
    pub fn span(mut self, span: Span) -> Self {
        self.span = span;
        self
    }

    /// Add a note to the diagnostic
    ///
    /// Notes provide additional context about the diagnostic.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::DiagnosticBuilder;
    ///
    /// let builder = DiagnosticBuilder::error("type mismatch")
    ///     .note("expected type `i32`")
    ///     .note("found type `String`");
    /// ```
    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Add a help suggestion to the diagnostic
    ///
    /// Helps provide actionable suggestions for fixing the issue.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::DiagnosticBuilder;
    ///
    /// let builder = DiagnosticBuilder::error("unexpected token")
    ///     .help("try removing the semicolon");
    /// ```
    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.helps.push(help.into());
        self
    }

    /// Add a note to the diagnostic (alias for `note`)
    ///
    /// Notes provide additional context about the diagnostic.
    /// This is an alias for [`DiagnosticBuilder::note`] following the
    /// convention used in the Diagnostic API.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::DiagnosticBuilder;
    ///
    /// let builder = DiagnosticBuilder::error("type mismatch")
    ///     .with_note("expected type `i32`")
    ///     .with_note("found type `String`");
    /// ```
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Add a help suggestion to the diagnostic (alias for `help`)
    ///
    /// Helps provide actionable suggestions for fixing the issue.
    /// This is an alias for [`DiagnosticBuilder::help`] following the
    /// convention used in the Diagnostic API.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::DiagnosticBuilder;
    ///
    /// let builder = DiagnosticBuilder::error("unexpected token")
    ///     .with_help("try removing the semicolon");
    /// ```
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.helps.push(help.into());
        self
    }

    /// Add a source code snippet
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::{DiagnosticBuilder, SourceSnippet};
    ///
    /// let snippet = SourceSnippet::new(
    ///     "let x = 42;",
    ///     1,
    ///     5,
    ///     6,
    ///     Some("variable name"),
    /// );
    ///
    /// let builder = DiagnosticBuilder::error("invalid name")
    ///     .snippet(snippet);
    /// ```
    pub fn snippet(mut self, snippet: SourceSnippet) -> Self {
        self.snippets.push(snippet);
        self
    }

    /// Build the diagnostic
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::{DiagnosticBuilder, Span};
    ///
    /// let diag = DiagnosticBuilder::error("something went wrong")
    ///     .span(Span::DUMMY)
    ///     .build();
    /// ```
    pub fn build(self) -> Diagnostic {
        Diagnostic {
            level: self.level,
            message: self.message,
            span: self.span,
            code: self.code,
            notes: self.notes,
            helps: self.helps,
            snippets: self.snippets,
        }
    }

    /// Build and emit the diagnostic to the given handler
    ///
    /// This is a convenience method that builds the diagnostic and
    /// immediately emits it to the provided handler.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::{DiagnosticBuilder, Handler, Span};
    ///
    /// let handler = Handler::new();
    /// DiagnosticBuilder::error("something went wrong")
    ///     .span(Span::DUMMY)
    ///     .emit(&handler);
    ///
    /// assert!(handler.has_errors());
    /// ```
    pub fn emit(self, handler: &super::Handler) {
        handler.emit_diagnostic(self.build());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_snippet_new() {
        let snippet = SourceSnippet::new("let x = 42;", 1, 5, 6, Some("variable"));
        assert_eq!(snippet.line, "let x = 42;");
        assert_eq!(snippet.line_number, 1);
        assert_eq!(snippet.start_column, 5);
        assert_eq!(snippet.end_column, 6);
        assert_eq!(snippet.label, Some("variable".to_string()));
    }

    #[test]
    fn test_source_snippet_point() {
        let snippet = SourceSnippet::point("let x = 42;", 1, 5);
        assert_eq!(snippet.start_column, 5);
        assert_eq!(snippet.end_column, 5);
        assert_eq!(snippet.label, None);
    }

    #[test]
    fn test_source_snippet_with_label() {
        let snippet = SourceSnippet::new("let x = 42;", 1, 5, 6, None::<String>).with_label("test");
        assert_eq!(snippet.label, Some("test".to_string()));
    }

    #[test]
    fn test_source_snippet_format() {
        let snippet = SourceSnippet::new("let x = 42;", 1, 5, 6, Some("here"));
        let formatted = snippet.format();
        assert!(formatted.contains("let x = 42;"));
        assert!(formatted.contains("^"));
        assert!(formatted.contains("here"));
    }

    #[test]
    fn test_builder_error() {
        let diag = DiagnosticBuilder::error("test error")
            .span(Span::DUMMY)
            .build();

        assert_eq!(diag.level, Level::Error);
        assert_eq!(diag.message, "test error");
    }

    #[test]
    fn test_builder_warning() {
        let diag = DiagnosticBuilder::warning("test warning")
            .span(Span::DUMMY)
            .build();

        assert_eq!(diag.level, Level::Warning);
    }

    #[test]
    fn test_builder_code() {
        let code = DiagnosticCode::new("test", 1001);
        let diag = DiagnosticBuilder::error("test")
            .code(code)
            .span(Span::DUMMY)
            .build();

        assert_eq!(diag.code, Some(code));
    }

    #[test]
    fn test_builder_notes_and_helps() {
        let diag = DiagnosticBuilder::error("test")
            .note("note 1")
            .note("note 2")
            .help("help 1")
            .help("help 2")
            .span(Span::DUMMY)
            .build();

        assert_eq!(diag.notes, vec!["note 1", "note 2"]);
        assert_eq!(diag.helps, vec!["help 1", "help 2"]);
    }

    #[test]
    fn test_builder_snippet() {
        let snippet = SourceSnippet::point("test", 1, 1);
        let diag = DiagnosticBuilder::error("test")
            .snippet(snippet.clone())
            .span(Span::DUMMY)
            .build();

        assert_eq!(diag.snippets.len(), 1);
        assert_eq!(diag.snippets[0].line, "test");
    }

    #[test]
    fn test_builder_fluent() {
        let code = DiagnosticCode::new("unexpected_token", 2001);
        let snippet = SourceSnippet::new("fn main() {", 1, 1, 3, Some("here"));

        let diag = DiagnosticBuilder::error("unexpected token")
            .code(code)
            .span(Span::new(0, 2, 1, 1))
            .note("parser encountered an unexpected token")
            .help("try checking the syntax")
            .snippet(snippet)
            .build();

        assert_eq!(diag.level, Level::Error);
        assert_eq!(diag.code, Some(code));
        assert_eq!(diag.notes.len(), 1);
        assert_eq!(diag.helps.len(), 1);
        assert_eq!(diag.snippets.len(), 1);
    }

    #[test]
    fn test_builder_with_note() {
        let diag = DiagnosticBuilder::error("test")
            .with_note("note 1")
            .with_note("note 2")
            .span(Span::DUMMY)
            .build();

        assert_eq!(diag.notes, vec!["note 1", "note 2"]);
    }

    #[test]
    fn test_builder_with_help() {
        let diag = DiagnosticBuilder::error("test")
            .with_help("help 1")
            .with_help("help 2")
            .span(Span::DUMMY)
            .build();

        assert_eq!(diag.helps, vec!["help 1", "help 2"]);
    }

    #[test]
    fn test_builder_emit() {
        use super::super::Handler;

        let handler = Handler::new();
        DiagnosticBuilder::error("test error")
            .span(Span::DUMMY)
            .emit(&handler);

        assert!(handler.has_errors());
        assert_eq!(handler.error_count(), 1);
    }

    #[test]
    fn test_builder_emit_with_code() {
        use super::super::Handler;

        let handler = Handler::new();
        DiagnosticBuilder::error("type error")
            .code(DiagnosticCode::E0002)
            .span(Span::DUMMY)
            .emit(&handler);

        let diags = handler.diagnostics();
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, Some(DiagnosticCode::E0002));
    }
}
