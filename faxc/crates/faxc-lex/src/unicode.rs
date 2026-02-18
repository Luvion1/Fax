//! Unicode utilities for the Fax lexer.
//!
//! This module provides functions for validating Unicode characters in
//! identifiers and handling UTF-8 encoded text correctly.

/// Checks if a character is valid as the start of an identifier.
///
/// Valid identifier start characters:
/// - ASCII letters: a-z, A-Z
/// - Underscore: _
/// - Unicode letters (categories L*)
/// - Unicode marks (categories M*)
///
/// # Arguments
///
/// * `c` - The character to check
///
/// # Example
///
/// ```
/// use faxc_lex::unicode::is_ident_start;
///
/// assert!(is_ident_start('a'));
/// assert!(is_ident_start('_'));
/// assert!(is_ident_start('α'));  // Greek alpha
/// assert!(!is_ident_start('1'));
/// assert!(!is_ident_start('+'));
/// ```
pub fn is_ident_start(c: char) -> bool {
    c == '_' || c.is_alphabetic()
}

/// Checks if a character is valid as a continuation of an identifier.
///
/// Valid identifier continuation characters:
/// - All valid start characters
/// - ASCII digits: 0-9
/// - Unicode decimal numbers (category Nd)
/// - Unicode combining marks (category M)
///
/// # Arguments
///
/// * `c` - The character to check
///
/// # Example
///
/// ```
/// use faxc_lex::unicode::is_ident_continue;
///
/// assert!(is_ident_continue('a'));
/// assert!(is_ident_continue('_'));
/// assert!(is_ident_continue('1'));
/// assert!(is_ident_continue('α'));
/// assert!(!is_ident_continue('+'));
/// assert!(!is_ident_continue(' '));
/// ```
pub fn is_ident_continue(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}

/// Checks if a character is a valid ASCII identifier start.
///
/// This is a stricter version that only allows ASCII characters.
///
/// # Arguments
///
/// * `c` - The character to check
///
/// # Example
///
/// ```
/// use faxc_lex::unicode::is_ascii_ident_start;
///
/// assert!(is_ascii_ident_start('a'));
/// assert!(is_ascii_ident_start('Z'));
/// assert!(is_ascii_ident_start('_'));
/// assert!(!is_ascii_ident_start('α'));  // Greek alpha
/// assert!(!is_ascii_ident_start('1'));
/// ```
pub fn is_ascii_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

/// Checks if a character is a valid ASCII identifier continuation.
///
/// This is a stricter version that only allows ASCII characters.
///
/// # Arguments
///
/// * `c` - The character to check
///
/// # Example
///
/// ```
/// use faxc_lex::unicode::is_ascii_ident_continue;
///
/// assert!(is_ascii_ident_continue('a'));
/// assert!(is_ascii_ident_continue('_'));
/// assert!(is_ascii_ident_continue('1'));
/// assert!(!is_ascii_ident_continue('α'));  // Greek alpha
/// assert!(!is_ascii_ident_continue('+'));
/// ```
pub fn is_ascii_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Checks if a character is a valid digit in the given numeric base.
///
/// # Arguments
///
/// * `c` - The character to check
/// * `base` - The numeric base (2, 8, 10, or 16)
///
/// # Example
///
/// ```
/// use faxc_lex::unicode::is_digit_in_base;
///
/// assert!(is_digit_in_base('0', 10));
/// assert!(is_digit_in_base('9', 10));
/// assert!(!is_digit_in_base('a', 10));
///
/// assert!(is_digit_in_base('0', 16));
/// assert!(is_digit_in_base('f', 16));
/// assert!(is_digit_in_base('F', 16));
///
/// assert!(is_digit_in_base('0', 2));
/// assert!(is_digit_in_base('1', 2));
/// assert!(!is_digit_in_base('2', 2));
/// ```
pub fn is_digit_in_base(c: char, base: u32) -> bool {
    match base {
        2 => matches!(c, '0' | '1'),
        8 => matches!(c, '0'..='7'),
        10 => c.is_ascii_digit(),
        16 => c.is_ascii_hexdigit(),
        _ => false,
    }
}

/// Converts a hex character to its numeric value.
///
/// # Arguments
///
/// * `c` - The hex character (0-9, a-f, A-F)
///
/// # Returns
///
/// The numeric value (0-15) if valid, None otherwise.
///
/// # Example
///
/// ```
/// use faxc_lex::unicode::hex_digit_to_value;
///
/// assert_eq!(hex_digit_to_value('0'), Some(0));
/// assert_eq!(hex_digit_to_value('9'), Some(9));
/// assert_eq!(hex_digit_to_value('a'), Some(10));
/// assert_eq!(hex_digit_to_value('F'), Some(15));
/// assert_eq!(hex_digit_to_value('g'), None);
/// ```
pub fn hex_digit_to_value(c: char) -> Option<u8> {
    match c {
        '0'..='9' => Some(c as u8 - b'0'),
        'a'..='f' => Some(c as u8 - b'a' + 10),
        'A'..='F' => Some(c as u8 - b'A' + 10),
        _ => None,
    }
}

/// Parses a hex string into a u8 value.
///
/// # Arguments
///
/// * `hex` - The hex string to parse
///
/// # Returns
///
/// The parsed value if valid, None otherwise.
///
/// # Example
///
/// ```
/// use faxc_lex::unicode::parse_hex_byte;
///
/// assert_eq!(parse_hex_byte("41"), Some(65));  // 'A'
/// assert_eq!(parse_hex_byte("FF"), Some(255));
/// assert_eq!(parse_hex_byte("00"), Some(0));
/// assert_eq!(parse_hex_byte("GG"), None);  // Invalid
/// ```
pub fn parse_hex_byte(hex: &str) -> Option<u8> {
    if hex.len() != 2 {
        return None;
    }
    let mut value = 0u8;
    for c in hex.chars() {
        value = value.checked_mul(16)?;
        value = value.checked_add(hex_digit_to_value(c)?)?;
    }
    Some(value)
}

/// Parses a hex string into a u32 value (for Unicode codepoints).
///
/// # Arguments
///
/// * `hex` - The hex string to parse
///
/// # Returns
///
/// The parsed value if valid, None otherwise.
///
/// # Example
///
/// ```
/// use faxc_lex::unicode::parse_hex_codepoint;
///
/// assert_eq!(parse_hex_codepoint("41"), Some(65));  // 'A'
/// assert_eq!(parse_hex_codepoint("1F600"), Some(0x1F600));  // 😀
/// assert_eq!(parse_hex_codepoint("10FFFF"), Some(0x10FFFF));  // Max valid
/// assert_eq!(parse_hex_codepoint("110000"), None);  // Too large
/// ```
pub fn parse_hex_codepoint(hex: &str) -> Option<u32> {
    if hex.is_empty() || hex.len() > 8 {
        return None;
    }
    let mut value = 0u32;
    for c in hex.chars() {
        value = value.checked_mul(16)?;
        value = value.checked_add(hex_digit_to_value(c)? as u32)?;
    }
    // Unicode codepoints must be <= 0x10FFFF
    if value > 0x10FFFF {
        return None;
    }
    Some(value)
}

/// Checks if a codepoint is a valid Unicode scalar value.
///
/// Valid scalar values are:
/// - 0x0000 to 0xD7FF
/// - 0xE000 to 0x10FFFF
///
/// (Excludes surrogate pairs: 0xD800 to 0xDFFF)
///
/// # Arguments
///
/// * `codepoint` - The codepoint to check
///
/// # Example
///
/// ```
/// use faxc_lex::unicode::is_valid_scalar;
///
/// assert!(is_valid_scalar(0x41));  // 'A'
/// assert!(is_valid_scalar(0x1F600));  // 😀
/// assert!(!is_valid_scalar(0xD800));  // Surrogate
/// assert!(!is_valid_scalar(0x110000));  // Too large
/// ```
pub fn is_valid_scalar(codepoint: u32) -> bool {
    if codepoint > 0x10FFFF {
        return false;
    }
    // Exclude surrogate pairs
    !(0xD800..=0xDFFF).contains(&codepoint)
}

/// Converts a codepoint to a char if valid.
///
/// # Arguments
///
/// * `codepoint` - The codepoint to convert
///
/// # Returns
///
/// The char if valid, None otherwise.
///
/// # Example
///
/// ```
/// use faxc_lex::unicode::codepoint_to_char;
///
/// assert_eq!(codepoint_to_char(65), Some('A'));
/// assert_eq!(codepoint_to_char(0x1F600), Some('😀'));
/// assert_eq!(codepoint_to_char(0xD800), None);  // Surrogate
/// ```
pub fn codepoint_to_char(codepoint: u32) -> Option<char> {
    if is_valid_scalar(codepoint) {
        char::from_u32(codepoint)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // IDENTIFIER TESTS
    // ========================================================================

    #[test]
    fn test_is_ident_start_ascii() {
        for c in 'a'..='z' {
            assert!(is_ident_start(c), "{} should be ident start", c);
        }
        for c in 'A'..='Z' {
            assert!(is_ident_start(c), "{} should be ident start", c);
        }
        assert!(is_ident_start('_'));
    }

    #[test]
    fn test_is_ident_start_unicode() {
        assert!(is_ident_start('α'));  // Greek
        assert!(is_ident_start('あ'));  // Hiragana
        assert!(is_ident_start('中'));  // CJK
        assert!(is_ident_start('ñ'));  // Latin with tilde
    }

    #[test]
    fn test_is_ident_start_invalid() {
        for c in '0'..='9' {
            assert!(!is_ident_start(c), "{} should not be ident start", c);
        }
        assert!(!is_ident_start('+'));
        assert!(!is_ident_start('-'));
        assert!(!is_ident_start(' '));
        assert!(!is_ident_start('\n'));
    }

    #[test]
    fn test_is_ident_continue() {
        assert!(is_ident_continue('a'));
        assert!(is_ident_continue('Z'));
        assert!(is_ident_continue('_'));
        for c in '0'..='9' {
            assert!(is_ident_continue(c), "{} should be ident continue", c);
        }
        assert!(is_ident_continue('α'));
        assert!(is_ident_continue('1'));
    }

    #[test]
    fn test_is_ident_continue_invalid() {
        assert!(!is_ident_continue('+'));
        assert!(!is_ident_continue('-'));
        assert!(!is_ident_continue(' '));
        assert!(!is_ident_continue('.'));
        assert!(!is_ident_continue(';'));
    }

    #[test]
    fn test_is_ascii_ident_start() {
        assert!(is_ascii_ident_start('a'));
        assert!(is_ascii_ident_start('Z'));
        assert!(is_ascii_ident_start('_'));
        assert!(!is_ascii_ident_start('α'));
        assert!(!is_ascii_ident_start('1'));
    }

    #[test]
    fn test_is_ascii_ident_continue() {
        assert!(is_ascii_ident_continue('a'));
        assert!(is_ascii_ident_continue('1'));
        assert!(is_ascii_ident_continue('_'));
        assert!(!is_ascii_ident_continue('α'));
    }

    // ========================================================================
    // DIGIT TESTS
    // ========================================================================

    #[test]
    fn test_is_digit_in_base_binary() {
        assert!(is_digit_in_base('0', 2));
        assert!(is_digit_in_base('1', 2));
        assert!(!is_digit_in_base('2', 2));
        assert!(!is_digit_in_base('a', 2));
    }

    #[test]
    fn test_is_digit_in_base_octal() {
        for c in '0'..='7' {
            assert!(is_digit_in_base(c, 8), "{} should be octal digit", c);
        }
        assert!(!is_digit_in_base('8', 8));
        assert!(!is_digit_in_base('9', 8));
        assert!(!is_digit_in_base('a', 8));
    }

    #[test]
    fn test_is_digit_in_base_decimal() {
        for c in '0'..='9' {
            assert!(is_digit_in_base(c, 10), "{} should be decimal digit", c);
        }
        assert!(!is_digit_in_base('a', 10));
        assert!(!is_digit_in_base('A', 10));
    }

    #[test]
    fn test_is_digit_in_base_hex() {
        for c in '0'..='9' {
            assert!(is_digit_in_base(c, 16), "{} should be hex digit", c);
        }
        for c in 'a'..='f' {
            assert!(is_digit_in_base(c, 16), "{} should be hex digit", c);
        }
        for c in 'A'..='F' {
            assert!(is_digit_in_base(c, 16), "{} should be hex digit", c);
        }
        assert!(!is_digit_in_base('g', 16));
    }

    #[test]
    fn test_is_digit_in_base_invalid() {
        assert!(!is_digit_in_base('0', 1));
        assert!(!is_digit_in_base('0', 3));
        assert!(!is_digit_in_base('0', 100));
    }

    // ========================================================================
    // HEX CONVERSION TESTS
    // ========================================================================

    #[test]
    fn test_hex_digit_to_value() {
        for (c, expected) in [('0', 0), ('1', 1), ('9', 9)] {
            assert_eq!(hex_digit_to_value(c), Some(expected));
        }
        for (c, expected) in [('a', 10), ('f', 15), ('A', 10), ('F', 15)] {
            assert_eq!(hex_digit_to_value(c), Some(expected));
        }
        assert_eq!(hex_digit_to_value('g'), None);
        assert_eq!(hex_digit_to_value('G'), None);
        assert_eq!(hex_digit_to_value(' '), None);
    }

    #[test]
    fn test_parse_hex_byte() {
        assert_eq!(parse_hex_byte("00"), Some(0));
        assert_eq!(parse_hex_byte("41"), Some(65));  // 'A'
        assert_eq!(parse_hex_byte("FF"), Some(255));
        assert_eq!(parse_hex_byte("ff"), Some(255));
        assert_eq!(parse_hex_byte("0a"), Some(10));
        assert_eq!(parse_hex_byte("GG"), None);
        assert_eq!(parse_hex_byte("1"), None);  // Too short
        assert_eq!(parse_hex_byte("123"), None);  // Too long
        assert_eq!(parse_hex_byte(""), None);
    }

    #[test]
    fn test_parse_hex_codepoint() {
        assert_eq!(parse_hex_codepoint("41"), Some(65));  // 'A'
        assert_eq!(parse_hex_codepoint("1F600"), Some(0x1F600));  // 😀
        assert_eq!(parse_hex_codepoint("10FFFF"), Some(0x10FFFF));  // Max
        assert_eq!(parse_hex_codepoint("0"), Some(0));
        assert_eq!(parse_hex_codepoint("110000"), None);  // Too large
        assert_eq!(parse_hex_codepoint("GGGG"), None);  // Invalid
        assert_eq!(parse_hex_codepoint(""), None);
    }

    // ========================================================================
    // SCALAR VALUE TESTS
    // ========================================================================

    #[test]
    fn test_is_valid_scalar() {
        assert!(is_valid_scalar(0));  // Null
        assert!(is_valid_scalar(0x41));  // 'A'
        assert!(is_valid_scalar(0xD7FF));  // Before surrogates
        assert!(is_valid_scalar(0xE000));  // After surrogates
        assert!(is_valid_scalar(0x10FFFF));  // Max
    }

    #[test]
    fn test_is_valid_scalar_invalid() {
        assert!(!is_valid_scalar(0xD800));  // Surrogate start
        assert!(!is_valid_scalar(0xDFFF));  // Surrogate end
        assert!(!is_valid_scalar(0x110000));  // Too large
        assert!(!is_valid_scalar(0xFFFFFFFF));  // Way too large
    }

    #[test]
    fn test_codepoint_to_char() {
        assert_eq!(codepoint_to_char(65), Some('A'));
        assert_eq!(codepoint_to_char(0x1F600), Some('😀'));
        assert_eq!(codepoint_to_char(0), Some('\0'));
        assert_eq!(codepoint_to_char(0xD800), None);  // Surrogate
        assert_eq!(codepoint_to_char(0x110000), None);  // Too large
    }
}
