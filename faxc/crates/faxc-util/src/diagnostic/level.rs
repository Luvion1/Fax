//! Diagnostic severity levels and label styles.
//!
//! This module provides types for categorizing diagnostic severity and
//! the visual style of span labels in multi-span diagnostics.
//!
//! # Examples
//!
//! ```
//! use faxc_util::diagnostic::{Level, LabelStyle};
//!
//! // Check diagnostic severity
//! assert!(Level::Error.is_error());
//! assert!(!Level::Warning.is_error());
//!
//! // Label styles for multi-span diagnostics
//! assert_eq!(LabelStyle::Primary.as_str(), "^");
//! assert_eq!(LabelStyle::Secondary.as_str(), "-");
//! ```

use std::fmt;

/// Diagnostic severity level
///
/// Each diagnostic has a severity level that determines:
/// - Whether compilation can continue
/// - How the diagnostic is displayed (color, emphasis)
/// - Whether it can be suppressed
///
/// # Examples
///
//! ```
//! use faxc_util::diagnostic::Level;
//!
//! assert_eq!(format!("{}", Level::Error), "error");
//! assert_eq!(format!("{}", Level::Warning), "warning");
//! ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    /// An error that prevents compilation
    ///
    /// Errors indicate fundamental problems that make the code invalid.
    /// Compilation cannot proceed when errors are present.
    Error,
    /// A warning that doesn't prevent compilation
    ///
    /// Warnings indicate suspicious or potentially problematic code that
    /// is still technically valid. Compilation can proceed with warnings.
    Warning,
    /// Additional information about a diagnostic
    ///
    /// Notes provide context or explanation for why a diagnostic was emitted.
    /// They are always attached to a parent diagnostic.
    Note,
    /// A suggestion for fixing an issue
    ///
    /// Help messages provide actionable suggestions for resolving the
    /// diagnostic. They often include code examples or specific changes.
    Help,
    /// A bug message indicating an internal compiler issue
    ///
    /// Bug diagnostics indicate that the compiler itself encountered
    /// an unexpected situation. These should be reported to the compiler team.
    Bug,
}

impl Level {
    /// Returns true if this level represents an error
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::Level;
    ///
    /// assert!(Level::Error.is_error());
    /// assert!(!Level::Warning.is_error());
    /// ```
    #[inline]
    pub const fn is_error(&self) -> bool {
        matches!(self, Level::Error)
    }

    /// Returns true if this level represents a warning
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::Level;
    ///
    /// assert!(Level::Warning.is_warning());
    /// assert!(!Level::Error.is_warning());
    /// ```
    #[inline]
    pub const fn is_warning(&self) -> bool {
        matches!(self, Level::Warning)
    }

    /// Returns true if this level is informational (Note or Help)
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::Level;
    ///
    /// assert!(Level::Note.is_info());
    /// assert!(Level::Help.is_info());
    /// assert!(!Level::Error.is_info());
    /// ```
    #[inline]
    pub const fn is_info(&self) -> bool {
        matches!(self, Level::Note | Level::Help)
    }

    /// Returns the canonical name for this level
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::Level;
    ///
    /// assert_eq!(Level::Error.name(), "error");
    /// assert_eq!(Level::Warning.name(), "warning");
    /// ```
    #[inline]
    pub const fn name(&self) -> &'static str {
        match self {
            Level::Error => "error",
            Level::Warning => "warning",
            Level::Note => "note",
            Level::Help => "help",
            Level::Bug => "error: internal compiler error",
        }
    }

    /// Returns the color code for this level (ANSI)
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::Level;
    ///
    /// // Error is red (31), Warning is yellow (33)
    /// assert!(Level::Error.color_code().is_some());
    /// ```
    #[inline]
    pub const fn color_code(&self) -> Option<&'static str> {
        match self {
            Level::Error => Some("31"),    // Red
            Level::Warning => Some("33"),  // Yellow
            Level::Note => Some("36"),     // Cyan
            Level::Help => Some("32"),     // Green
            Level::Bug => Some("35"),      // Magenta
        }
    }

    /// Returns the intensity modifier for terminal output
    ///
    /// Errors and bugs are displayed with bold emphasis.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::Level;
    ///
    /// assert_eq!(Level::Error.intensity(), "1");  // Bold
    /// assert_eq!(Level::Note.intensity(), "0");   // Normal
    /// ```
    #[inline]
    pub const fn intensity(&self) -> &'static str {
        match self {
            Level::Error | Level::Bug => "1",  // Bold
            _ => "0",                           // Normal
        }
    }

    /// Returns a short single-character indicator for this level
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::Level;
    ///
    /// assert_eq!(Level::Error.indicator(), "E");
    /// assert_eq!(Level::Warning.indicator(), "W");
    /// ```
    #[inline]
    pub const fn indicator(&self) -> &'static str {
        match self {
            Level::Error => "E",
            Level::Warning => "W",
            Level::Note => "N",
            Level::Help => "H",
            Level::Bug => "!",
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Style for span labels in multi-span diagnostics
///
/// When a diagnostic has multiple spans, each span can have a label
/// with a specific style that affects how it's displayed.
///
/// # Examples
///
/// ```
/// use faxc_util::diagnostic::LabelStyle;
///
/// // Primary spans use ^^^ underline
/// assert_eq!(LabelStyle::Primary.as_str(), "^");
///
/// // Secondary spans use --- underline
/// assert_eq!(LabelStyle::Secondary.as_str(), "-");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LabelStyle {
    /// Primary span - the main location of the issue
    ///
    /// Displayed with `^` characters. There should typically be only
    /// one primary span per diagnostic.
    Primary,
    /// Secondary span - additional context or related locations
    ///
    /// Displayed with `-` characters. Used for showing related code
    /// locations that provide context for the diagnostic.
    Secondary,
}

impl LabelStyle {
    /// Returns the underline character for this style
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::LabelStyle;
    ///
    /// assert_eq!(LabelStyle::Primary.underline_char(), '^');
    /// assert_eq!(LabelStyle::Secondary.underline_char(), '-');
    /// ```
    #[inline]
    pub const fn underline_char(&self) -> char {
        match self {
            LabelStyle::Primary => '^',
            LabelStyle::Secondary => '-',
        }
    }

    /// Returns the string representation for formatting
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::LabelStyle;
    ///
    /// assert_eq!(LabelStyle::Primary.as_str(), "^");
    /// ```
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            LabelStyle::Primary => "^",
            LabelStyle::Secondary => "-",
        }
    }

    /// Returns true if this is a primary label
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::LabelStyle;
    ///
    /// assert!(LabelStyle::Primary.is_primary());
    /// assert!(!LabelStyle::Secondary.is_primary());
    /// ```
    #[inline]
    pub const fn is_primary(&self) -> bool {
        matches!(self, LabelStyle::Primary)
    }

    /// Returns true if this is a secondary label
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::LabelStyle;
    ///
    /// assert!(LabelStyle::Secondary.is_secondary());
    /// assert!(!LabelStyle::Primary.is_secondary());
    /// ```
    #[inline]
    pub const fn is_secondary(&self) -> bool {
        matches!(self, LabelStyle::Secondary)
    }
}

impl fmt::Display for LabelStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Color configuration for diagnostic rendering
///
/// Controls how colors are applied when rendering diagnostics to the terminal.
///
/// # Examples
///
/// ```
/// use faxc_util::diagnostic::ColorConfig;
///
/// // Auto-detect terminal support
/// let config = ColorConfig::Auto;
///
/// // Force colors on
/// let config = ColorConfig::Always;
///
/// // Disable colors
/// let config = ColorConfig::Never;
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ColorConfig {
    /// Automatically detect terminal color support
    #[default]
    Auto,
    /// Always use colors, even in pipes
    Always,
    /// Never use colors
    Never,
    /// Use ANSI colors only
    Ansi,
}

impl ColorConfig {
    /// Returns true if colors should be used for the given environment
    ///
    /// # Arguments
    ///
    /// * `is_tty` - Whether the output is a terminal
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::ColorConfig;
    ///
    /// assert!(ColorConfig::Always.use_color(true));
    /// assert!(!ColorConfig::Never.use_color(true));
    /// ```
    pub fn use_color(&self, is_tty: bool) -> bool {
        match self {
            ColorConfig::Auto => is_tty,
            ColorConfig::Always => true,
            ColorConfig::Never => false,
            ColorConfig::Ansi => true,
        }
    }

    /// Returns true if this configuration enables colors
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::ColorConfig;
    ///
    /// assert!(ColorConfig::Always.is_enabled());
    /// assert!(!ColorConfig::Never.is_enabled());
    /// ```
    pub const fn is_enabled(&self) -> bool {
        matches!(self, ColorConfig::Always | ColorConfig::Ansi)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_is_error() {
        assert!(Level::Error.is_error());
        assert!(!Level::Warning.is_error());
        assert!(!Level::Note.is_error());
        assert!(!Level::Help.is_error());
    }

    #[test]
    fn test_level_is_warning() {
        assert!(Level::Warning.is_warning());
        assert!(!Level::Error.is_warning());
        assert!(!Level::Note.is_warning());
    }

    #[test]
    fn test_level_is_info() {
        assert!(Level::Note.is_info());
        assert!(Level::Help.is_info());
        assert!(!Level::Error.is_info());
        assert!(!Level::Warning.is_info());
    }

    #[test]
    fn test_level_name() {
        assert_eq!(Level::Error.name(), "error");
        assert_eq!(Level::Warning.name(), "warning");
        assert_eq!(Level::Note.name(), "note");
        assert_eq!(Level::Help.name(), "help");
        assert_eq!(Level::Bug.name(), "error: internal compiler error");
    }

    #[test]
    fn test_level_display() {
        assert_eq!(format!("{}", Level::Error), "error");
        assert_eq!(format!("{}", Level::Warning), "warning");
        assert_eq!(format!("{}", Level::Note), "note");
        assert_eq!(format!("{}", Level::Help), "help");
    }

    #[test]
    fn test_level_color_code() {
        assert_eq!(Level::Error.color_code(), Some("31"));
        assert_eq!(Level::Warning.color_code(), Some("33"));
        assert_eq!(Level::Note.color_code(), Some("36"));
        assert_eq!(Level::Help.color_code(), Some("32"));
        assert_eq!(Level::Bug.color_code(), Some("35"));
    }

    #[test]
    fn test_level_intensity() {
        assert_eq!(Level::Error.intensity(), "1");
        assert_eq!(Level::Bug.intensity(), "1");
        assert_eq!(Level::Warning.intensity(), "0");
        assert_eq!(Level::Note.intensity(), "0");
    }

    #[test]
    fn test_level_indicator() {
        assert_eq!(Level::Error.indicator(), "E");
        assert_eq!(Level::Warning.indicator(), "W");
        assert_eq!(Level::Note.indicator(), "N");
        assert_eq!(Level::Help.indicator(), "H");
        assert_eq!(Level::Bug.indicator(), "!");
    }

    #[test]
    fn test_level_ordering() {
        // Errors are most severe
        assert!(Level::Error > Level::Warning);
        assert!(Level::Warning > Level::Note);
        assert!(Level::Note > Level::Help);
    }

    #[test]
    fn test_label_style_underline_char() {
        assert_eq!(LabelStyle::Primary.underline_char(), '^');
        assert_eq!(LabelStyle::Secondary.underline_char(), '-');
    }

    #[test]
    fn test_label_style_as_str() {
        assert_eq!(LabelStyle::Primary.as_str(), "^");
        assert_eq!(LabelStyle::Secondary.as_str(), "-");
    }

    #[test]
    fn test_label_style_is_primary() {
        assert!(LabelStyle::Primary.is_primary());
        assert!(!LabelStyle::Secondary.is_primary());
    }

    #[test]
    fn test_label_style_is_secondary() {
        assert!(LabelStyle::Secondary.is_secondary());
        assert!(!LabelStyle::Primary.is_secondary());
    }

    #[test]
    fn test_label_style_display() {
        assert_eq!(format!("{}", LabelStyle::Primary), "^");
        assert_eq!(format!("{}", LabelStyle::Secondary), "-");
    }

    #[test]
    fn test_color_config_use_color() {
        assert!(ColorConfig::Always.use_color(true));
        assert!(ColorConfig::Always.use_color(false));
        assert!(ColorConfig::Auto.use_color(true));
        assert!(!ColorConfig::Auto.use_color(false));
        assert!(!ColorConfig::Never.use_color(true));
        assert!(!ColorConfig::Never.use_color(false));
    }

    #[test]
    fn test_color_config_is_enabled() {
        assert!(ColorConfig::Always.is_enabled());
        assert!(ColorConfig::Ansi.is_enabled());
        assert!(!ColorConfig::Never.is_enabled());
        assert!(!ColorConfig::Auto.is_enabled());
    }

    #[test]
    fn test_color_config_default() {
        assert_eq!(ColorConfig::default(), ColorConfig::Auto);
    }

    #[test]
    fn test_level_hash_and_eq() {
        use std::collections::HashSet;
        
        let mut set = HashSet::new();
        set.insert(Level::Error);
        set.insert(Level::Warning);
        set.insert(Level::Error); // Duplicate
        
        assert_eq!(set.len(), 2);
        assert!(set.contains(&Level::Error));
        assert!(set.contains(&Level::Warning));
    }
}
