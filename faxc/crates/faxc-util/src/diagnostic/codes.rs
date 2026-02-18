//! Diagnostic codes for categorizing compiler errors and warnings.
//!
//! This module provides the [`DiagnosticCode`] type for uniquely identifying
//! diagnostic messages, enabling users to look up documentation and suppress
//! specific warnings.
//!
//! # Examples
//!
//! ```
//! use faxc_util::diagnostic::DiagnosticCode;
//!
//! let code = DiagnosticCode::E0001;
//! assert_eq!(code.prefix(), "E");
//! assert_eq!(code.number(), 1);
//! assert_eq!(code.as_str(), "E0001");
//! ```

/// A unique code identifying a diagnostic message
///
/// Diagnostic codes follow the format `{prefix}{number}` where:
/// - `prefix` is typically "E" for errors or "W" for warnings
/// - `number` is a 4-digit number (padded with zeros)
///
/// This allows users to reference specific diagnostics in documentation
/// and suppression attributes.
///
/// # Examples
///
/// ```
/// use faxc_util::diagnostic::DiagnosticCode;
///
/// let code = DiagnosticCode::new("E", 1);
/// assert_eq!(code.as_str(), "E0001");
///
/// let warning = DiagnosticCode::W0001;
/// assert_eq!(warning.prefix(), "W");
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiagnosticCode {
    /// The prefix (e.g., "E" for error, "W" for warning)
    pub prefix: &'static str,
    /// The numeric identifier
    pub number: u32,
}

impl DiagnosticCode {
    /// Create a new diagnostic code
    ///
    /// # Arguments
    ///
    /// * `prefix` - The code prefix (typically "E" or "W")
    /// * `number` - The numeric identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::DiagnosticCode;
    ///
    /// let code = DiagnosticCode::new("E", 1001);
    /// assert_eq!(code.prefix(), "E");
    /// assert_eq!(code.number(), 1001);
    /// ```
    #[inline]
    pub const fn new(prefix: &'static str, number: u32) -> Self {
        Self { prefix, number }
    }

    /// Get the prefix (e.g., "E" for error, "W" for warning)
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::DiagnosticCode;
    ///
    /// assert_eq!(DiagnosticCode::E0001.prefix(), "E");
    /// assert_eq!(DiagnosticCode::W0001.prefix(), "W");
    /// ```
    #[inline]
    pub const fn prefix(&self) -> &'static str {
        self.prefix
    }

    /// Get the numeric identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::DiagnosticCode;
    ///
    /// assert_eq!(DiagnosticCode::E0001.number(), 1);
    /// assert_eq!(DiagnosticCode::W0001.number(), 1);
    /// ```
    #[inline]
    pub const fn number(&self) -> u32 {
        self.number
    }

    /// Get the full code string (e.g., "E0001", "W0001")
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::diagnostic::DiagnosticCode;
    ///
    /// assert_eq!(DiagnosticCode::E0001.as_str(), "E0001");
    /// assert_eq!(DiagnosticCode::W0001.as_str(), "W0001");
    /// ```
    pub fn as_str(&self) -> String {
        format!("{}{:04}", self.prefix, self.number)
    }

    // =========================================================================
    // PREDEFINED ERROR CODES (E0001-E9999)
    // =========================================================================

    /// E0001: Syntax error
    pub const E0001: Self = Self::new("E", 1);
    /// E0002: Type mismatch
    pub const E0002: Self = Self::new("E", 2);
    /// E0003: Undefined variable
    pub const E0003: Self = Self::new("E", 3);
    /// E0004: Undefined function
    pub const E0004: Self = Self::new("E", 4);
    /// E0005: Duplicate definition
    pub const E0005: Self = Self::new("E", 5);

    /// E1001: Lexer - Unexpected character
    pub const E_LEXER_UNEXPECTED_CHAR: Self = Self::new("E", 1001);
    /// E1002: Lexer - Unterminated string literal
    pub const E_LEXER_UNTERMINATED_STRING: Self = Self::new("E", 1002);
    /// E1003: Lexer - Invalid numeric literal
    pub const E_LEXER_INVALID_NUMBER: Self = Self::new("E", 1003);
    /// E1004: Lexer - Unknown token
    pub const E_LEXER_UNKNOWN_TOKEN: Self = Self::new("E", 1004);

    /// E2001: Parser - Unexpected token
    pub const E_PARSER_UNEXPECTED_TOKEN: Self = Self::new("E", 2001);
    /// E2002: Parser - Expected token
    pub const E_PARSER_EXPECTED_TOKEN: Self = Self::new("E", 2002);
    /// E2003: Parser - Unexpected end of file
    pub const E_PARSER_UNEXPECTED_EOF: Self = Self::new("E", 2003);
    /// E2004: Parser - Duplicate definition
    pub const E_PARSER_DUPLICATE_DEF: Self = Self::new("E", 2004);

    /// E3001: Semantic - Type mismatch
    pub const E_SEMANTIC_TYPE_MISMATCH: Self = Self::new("E", 3001);
    /// E3002: Semantic - Undefined variable
    pub const E_SEMANTIC_UNDEFINED_VAR: Self = Self::new("E", 3002);
    /// E3003: Semantic - Undefined function
    pub const E_SEMANTIC_UNDEFINED_FN: Self = Self::new("E", 3003);
    /// E3004: Semantic - Mutable binding required
    pub const E_SEMANTIC_MUT_REQUIRED: Self = Self::new("E", 3004);

    // =========================================================================
    // PREDEFINED WARNING CODES (W0001-W9999)
    // =========================================================================

    /// W0001: Unused variable
    pub const W0001: Self = Self::new("W", 1);
    /// W0002: Unused function
    pub const W0002: Self = Self::new("W", 2);
    /// W0003: Dead code
    pub const W0003: Self = Self::new("W", 3);

    /// W4001: Warning - Unused variable (legacy alias)
    pub const W_UNUSED_VARIABLE: Self = Self::new("W", 4001);
    /// W4002: Warning - Unused function (legacy alias)
    pub const W_UNUSED_FUNCTION: Self = Self::new("W", 4002);
    /// W4003: Warning - Dead code (legacy alias)
    pub const W_DEAD_CODE: Self = Self::new("W", 4003);
}

impl std::fmt::Debug for DiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DiagnosticCode({})", self.as_str())
    }
}

impl std::fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// Standalone constant exports for convenience
pub const E0001: DiagnosticCode = DiagnosticCode::E0001;
pub const E0002: DiagnosticCode = DiagnosticCode::E0002;
pub const E0003: DiagnosticCode = DiagnosticCode::E0003;
pub const E0004: DiagnosticCode = DiagnosticCode::E0004;
pub const E0005: DiagnosticCode = DiagnosticCode::E0005;
pub const E_LEXER_UNEXPECTED_CHAR: DiagnosticCode = DiagnosticCode::E_LEXER_UNEXPECTED_CHAR;
pub const E_LEXER_UNTERMINATED_STRING: DiagnosticCode = DiagnosticCode::E_LEXER_UNTERMINATED_STRING;
pub const E_LEXER_INVALID_NUMBER: DiagnosticCode = DiagnosticCode::E_LEXER_INVALID_NUMBER;
pub const E_LEXER_UNKNOWN_TOKEN: DiagnosticCode = DiagnosticCode::E_LEXER_UNKNOWN_TOKEN;
pub const E_PARSER_UNEXPECTED_TOKEN: DiagnosticCode = DiagnosticCode::E_PARSER_UNEXPECTED_TOKEN;
pub const E_PARSER_EXPECTED_TOKEN: DiagnosticCode = DiagnosticCode::E_PARSER_EXPECTED_TOKEN;
pub const E_PARSER_UNEXPECTED_EOF: DiagnosticCode = DiagnosticCode::E_PARSER_UNEXPECTED_EOF;
pub const E_PARSER_DUPLICATE_DEF: DiagnosticCode = DiagnosticCode::E_PARSER_DUPLICATE_DEF;
pub const E_SEMANTIC_TYPE_MISMATCH: DiagnosticCode = DiagnosticCode::E_SEMANTIC_TYPE_MISMATCH;
pub const E_SEMANTIC_UNDEFINED_VAR: DiagnosticCode = DiagnosticCode::E_SEMANTIC_UNDEFINED_VAR;
pub const E_SEMANTIC_UNDEFINED_FN: DiagnosticCode = DiagnosticCode::E_SEMANTIC_UNDEFINED_FN;
pub const E_SEMANTIC_MUT_REQUIRED: DiagnosticCode = DiagnosticCode::E_SEMANTIC_MUT_REQUIRED;
pub const W0001: DiagnosticCode = DiagnosticCode::W0001;
pub const W0002: DiagnosticCode = DiagnosticCode::W0002;
pub const W0003: DiagnosticCode = DiagnosticCode::W0003;
pub const W_UNUSED_VARIABLE: DiagnosticCode = DiagnosticCode::W_UNUSED_VARIABLE;
pub const W_UNUSED_FUNCTION: DiagnosticCode = DiagnosticCode::W_UNUSED_FUNCTION;
pub const W_DEAD_CODE: DiagnosticCode = DiagnosticCode::W_DEAD_CODE;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_code() {
        let code = DiagnosticCode::new("E", 1001);
        assert_eq!(code.prefix(), "E");
        assert_eq!(code.number(), 1001);
    }

    #[test]
    fn test_as_str() {
        let code = DiagnosticCode::new("E", 1);
        assert_eq!(code.as_str(), "E0001");

        let code = DiagnosticCode::new("W", 1);
        assert_eq!(code.as_str(), "W0001");

        let code = DiagnosticCode::new("E", 1001);
        assert_eq!(code.as_str(), "E1001");
    }

    #[test]
    fn test_display() {
        let code = DiagnosticCode::new("E", 1001);
        assert_eq!(format!("{}", code), "E1001");
    }

    #[test]
    fn test_debug() {
        let code = DiagnosticCode::new("E", 1001);
        assert_eq!(format!("{:?}", code), "DiagnosticCode(E1001)");
    }

    #[test]
    fn test_predefined_error_codes() {
        assert_eq!(DiagnosticCode::E0001.prefix(), "E");
        assert_eq!(DiagnosticCode::E0001.number(), 1);
        assert_eq!(DiagnosticCode::E0001.as_str(), "E0001");

        assert_eq!(DiagnosticCode::E0002.prefix(), "E");
        assert_eq!(DiagnosticCode::E0002.number(), 2);
    }

    #[test]
    fn test_predefined_warning_codes() {
        assert_eq!(DiagnosticCode::W0001.prefix(), "W");
        assert_eq!(DiagnosticCode::W0001.number(), 1);
        assert_eq!(DiagnosticCode::W0001.as_str(), "W0001");
    }

    #[test]
    fn test_legacy_codes() {
        assert_eq!(DiagnosticCode::W_UNUSED_VARIABLE.prefix(), "W");
        assert_eq!(DiagnosticCode::W_UNUSED_VARIABLE.number(), 4001);

        assert_eq!(DiagnosticCode::E_LEXER_UNEXPECTED_CHAR.prefix(), "E");
        assert_eq!(DiagnosticCode::E_LEXER_UNEXPECTED_CHAR.number(), 1001);
    }

    #[test]
    fn test_code_equality() {
        let code1 = DiagnosticCode::new("E", 1001);
        let code2 = DiagnosticCode::new("E", 1001);
        let code3 = DiagnosticCode::new("E", 1002);

        assert_eq!(code1, code2);
        assert_ne!(code1, code3);
    }

    #[test]
    fn test_const_codes() {
        // Verify const codes work correctly
        const CODE: DiagnosticCode = DiagnosticCode::E0001;
        assert_eq!(CODE.prefix(), "E");
        assert_eq!(CODE.number(), 1);
    }
}
