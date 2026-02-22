//! Number literal lexing.
//!
//! This module handles lexing of integer and floating-point literals.

use crate::token::Token;
use crate::unicode::is_digit_in_base;
use crate::Lexer;

impl<'a> Lexer<'a> {
    /// Lexes a number literal.
    ///
    /// Handles decimal, hexadecimal (0x), binary (0b), octal (0o), and
    /// floating-point formats.
    ///
    /// # Number Formats
    ///
    /// - Decimal: `42`, `123`, `0`
    /// - Hexadecimal: `0xFF`, `0xAB_CD`
    /// - Binary: `0b1010`, `0b1111_0000`
    /// - Octal: `0o777`
    /// - Float: `3.14`, `1e10`, `2.5e-3`
    ///
    /// # Returns
    ///
    /// Either `Token::Number(u64)` or `Token::Float(f64)`
    pub fn lex_number(&mut self) -> Token {
        if self.cursor.current_char() == '0' && !self.cursor.is_at_end() {
            let start = self.cursor.position();
            self.cursor.advance();
            match self.cursor.current_char() {
                'x' | 'X' => {
                    self.cursor.advance();
                    return self.lex_integer(16, start);
                },
                'b' | 'B' => {
                    self.cursor.advance();
                    return self.lex_integer(2, start);
                },
                'o' | 'O' => {
                    self.cursor.advance();
                    return self.lex_integer(8, start);
                },
                _ => {
                    if !self.cursor.current_char().is_ascii_digit()
                        && self.cursor.current_char() != '.'
                        && self.cursor.current_char() != 'e'
                        && self.cursor.current_char() != 'E'
                    {
                        return Token::Number(0);
                    }
                },
            }
        }

        let start = self.cursor.position();

        while self.cursor.current_char().is_ascii_digit() {
            self.cursor.advance();
        }

        let is_float = (self.cursor.current_char() == '.'
            && self.cursor.peek_char(1).is_ascii_digit())
            || self.cursor.current_char() == 'e'
            || self.cursor.current_char() == 'E';

        if is_float {
            if self.cursor.current_char() == '.' {
                self.cursor.advance();
                while self.cursor.current_char().is_ascii_digit() {
                    self.cursor.advance();
                }
            }

            if self.cursor.current_char() == 'e' || self.cursor.current_char() == 'E' {
                let exp_start = self.cursor.position();
                self.cursor.advance();
                if self.cursor.current_char() == '+' || self.cursor.current_char() == '-' {
                    self.cursor.advance();
                }
                while self.cursor.current_char().is_ascii_digit() {
                    self.cursor.advance();
                }

                let after_e_pos = exp_start + 1;
                let has_sign = after_e_pos < self.cursor.position()
                    && (self.cursor.source()[after_e_pos..].starts_with('+')
                        || self.cursor.source()[after_e_pos..].starts_with('-'));
                let min_expected_pos = if has_sign {
                    exp_start + 2
                } else {
                    exp_start + 1
                };

                if self.cursor.position() < min_expected_pos {
                    self.report_error("no digits in float exponent".to_string());
                }
            }

            let text = self.cursor.slice_from(start);
            match text.parse::<f64>() {
                Ok(value) if value.is_finite() => Token::Float(value),
                Ok(_) => {
                    self.report_error(format!("floating point literal '{}' is not finite", text));
                    Token::Float(0.0)
                },
                Err(e) => {
                    self.report_error(format!("invalid floating point literal '{}': {}", text, e));
                    Token::Float(0.0)
                },
            }
        } else {
            let text = self.cursor.slice_from(start);
            match text.parse::<u64>() {
                Ok(value) => Token::Number(value),
                Err(e) => {
                    self.report_error(format!("integer literal overflow: {}", e));
                    Token::Number(0)
                },
            }
        }
    }

    /// Lexes an integer with the specified base.
    ///
    /// # Arguments
    ///
    /// * `base` - The numeric base (2, 8, 10, or 16)
    /// * `start` - The starting position of the number (at the '0' before prefix)
    pub fn lex_integer(&mut self, base: u32, start: usize) -> Token {
        let digit_start = self.cursor.position();

        loop {
            let c = self.cursor.current_char();
            if c == '_' || is_digit_in_base(c, base) {
                self.cursor.advance();
            } else {
                break;
            }
        }

        if digit_start == self.cursor.position() {
            self.report_error(format!("no digits after base-{} prefix", base));
            return Token::Number(0);
        }

        let full_text = self.cursor.slice_from(start);
        let digits_text = &full_text[2..].replace('_', "");

        let value = match u64::from_str_radix(digits_text, base) {
            Ok(v) => v,
            Err(e) => {
                self.report_error(format!("integer literal overflow: {}", e));
                0
            },
        };

        Token::Number(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Token;
    use faxc_util::Handler;

    fn lex_num(source: &str) -> Token {
        let mut handler = Handler::new();
        let mut lexer = crate::Lexer::new(source, &mut handler);
        lexer.lex_number()
    }

    #[test]
    fn test_decimal_integer() {
        assert_eq!(lex_num("42"), Token::Number(42));
        assert_eq!(lex_num("0"), Token::Number(0));
        assert_eq!(lex_num("123456"), Token::Number(123456));
    }

    #[test]
    fn test_hex_integer() {
        assert_eq!(lex_num("0xFF"), Token::Number(0xFF));
        assert_eq!(lex_num("0xAB_CD"), Token::Number(0xABCD));
        assert_eq!(lex_num("0x0"), Token::Number(0));
    }

    #[test]
    fn test_binary_integer() {
        assert_eq!(lex_num("0b1010"), Token::Number(0b1010));
        assert_eq!(lex_num("0b1111_0000"), Token::Number(0b11110000));
    }

    #[test]
    fn test_octal_integer() {
        assert_eq!(lex_num("0o777"), Token::Number(0o777));
        assert_eq!(lex_num("0o0"), Token::Number(0));
    }

    #[test]
    fn test_float() {
        let result = lex_num("3.14");
        assert!(matches!(result, Token::Float(f) if (f - 3.14).abs() < 0.001));
    }

    #[test]
    fn test_float_with_exponent() {
        let result = lex_num("1e10");
        assert!(matches!(result, Token::Float(f) if (f - 1e10).abs() < 1.0));
    }

    #[test]
    fn test_float_negative_exponent() {
        let result = lex_num("2.5e-3");
        assert!(matches!(result, Token::Float(f) if (f - 2.5e-3).abs() < 0.0001));
    }
}
