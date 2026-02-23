//! String and character literal lexing.
//!
//! This module handles lexing of string literals, raw strings, and character literals.

use crate::token::Token;
use crate::unicode::parse_hex_codepoint;
use crate::Lexer;
use faxc_util::Symbol;

impl<'a> Lexer<'a> {
    /// Lexes a string literal.
    ///
    /// Parses a string enclosed in double quotes, handling escape sequences.
    ///
    /// # Returns
    ///
    /// `Token::String(symbol)` with the processed string content
    pub fn lex_string(&mut self) -> Token {
        self.cursor.advance();

        let mut content = String::new();

        loop {
            if self.cursor.is_at_end() {
                self.report_error("unterminated string literal".to_string());
                break;
            }

            let c = self.cursor.current_char();

            if c == '"' {
                self.cursor.advance();
                break;
            }

            if c == '\n' {
                self.report_error("unterminated string literal".to_string());
                break;
            }

            if c == '\\' {
                self.cursor.advance();
                if let Some(escaped) = self.parse_escape() {
                    content.push(escaped);
                }
            } else {
                content.push(c);
                self.cursor.advance();
            }
        }

        Token::String(Symbol::intern(&content))
    }

    /// Lexes a raw string literal (r"..." or r#"..."#).
    ///
    /// Raw strings don't process escape sequences and can contain quotes.
    ///
    /// # Returns
    ///
    /// `Token::RawString(symbol)` with the raw string content
    pub fn lex_raw_string(&mut self) -> Token {
        self.cursor.advance();

        let mut hash_count = 0;
        while self.cursor.current_char() == '#' {
            hash_count += 1;
            self.cursor.advance();
        }

        if self.cursor.current_char() != '"' {
            self.report_error("expected \" after raw string prefix".to_string());
            return Token::Invalid("raw".to_string());
        }
        self.cursor.advance();

        let mut closing_delimiter = String::new();
        for _ in 0..hash_count {
            closing_delimiter.push('#');
        }
        closing_delimiter.push('"');

        let mut content = String::new();
        let mut found_closing = false;

        while !self.cursor.is_at_end() {
            let mut lookahead = String::new();
            for i in 0..closing_delimiter.len() {
                let c = self.cursor.peek_char(i);
                lookahead.push(c);
            }
            if lookahead == closing_delimiter {
                for _ in 0..closing_delimiter.len() {
                    self.cursor.advance();
                }
                found_closing = true;
                break;
            }

            content.push(self.cursor.current_char());
            self.cursor.advance();
        }

        if !found_closing {
            self.report_error("unterminated raw string literal".to_string());
        }

        Token::RawString(Symbol::intern(&content))
    }

    /// Lexes a character literal.
    ///
    /// Parses a character enclosed in single quotes, handling escape sequences.
    ///
    /// # Returns
    ///
    /// `Token::Char` containing the parsed character
    pub fn lex_char(&mut self) -> Token {
        self.cursor.advance();

        if self.cursor.is_at_end() {
            self.report_error("unterminated character literal".to_string());
            return Token::Char('\0');
        }

        let c = if self.cursor.current_char() == '\\' {
            self.cursor.advance();
            self.parse_escape().unwrap_or('\0')
        } else {
            let c = self.cursor.current_char();
            if c == '\'' || c == '\n' {
                self.report_error("empty character literal".to_string());
                return Token::Char('\0');
            }
            self.cursor.advance();
            c
        };

        if self.cursor.current_char() != '\'' {
            self.report_error("unterminated character literal".to_string());
            while !self.cursor.is_at_end()
                && self.cursor.current_char() != '\''
                && self.cursor.current_char() != '\n'
            {
                self.cursor.advance();
            }
        } else {
            self.cursor.advance();
        }

        Token::Char(c)
    }

    /// Parses an escape sequence.
    ///
    /// Handles: `\n`, `\t`, `\r`, `\\`, `\"`, `\'`, `\0`, `\xNN`, `\u{NNNN}`
    ///
    /// # Returns
    ///
    /// The escaped character, or None on error
    pub fn parse_escape(&mut self) -> Option<char> {
        if self.cursor.is_at_end() {
            self.report_error("unterminated escape sequence".to_string());
            return None;
        }

        let c = self.cursor.current_char();
        self.cursor.advance();

        match c {
            'n' => Some('\n'),
            't' => Some('\t'),
            'r' => Some('\r'),
            '\\' => Some('\\'),
            '"' => Some('"'),
            '\'' => Some('\''),
            '0' => Some('\0'),
            'x' => {
                let mut hex = String::new();
                for _ in 0..2 {
                    let h = self.cursor.current_char();
                    if h.is_ascii_hexdigit() {
                        hex.push(h);
                        self.cursor.advance();
                    } else {
                        break;
                    }
                }
                if hex.len() == 2 {
                    u8::from_str_radix(&hex, 16).ok().map(|b| b as char)
                } else {
                    self.report_error("invalid hex escape sequence".to_string());
                    None
                }
            },
            'u' => {
                if self.cursor.current_char() != '{' {
                    self.report_error("expected {{ after \\u".to_string());
                    return None;
                }
                self.cursor.advance();
                let mut hex = String::new();
                while self.cursor.current_char() != '}' && !self.cursor.is_at_end() {
                    let h = self.cursor.current_char();
                    if h.is_ascii_hexdigit() {
                        hex.push(h);
                        self.cursor.advance();
                    } else {
                        break;
                    }
                }
                if self.cursor.current_char() == '}' {
                    self.cursor.advance();
                }
                parse_hex_codepoint(&hex).and_then(|cp| char::from_u32(cp))
            },
            _ => {
                self.report_error(format!("unknown escape sequence: \\{}", c));
                None
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Token;
    use faxc_util::Handler;

    fn lex_str(source: &str) -> Token {
        let mut handler = Handler::new();
        let mut lexer = crate::Lexer::new(source, &mut handler);
        lexer.lex_string()
    }

    fn lex_raw_str(source: &str) -> Token {
        let mut handler = Handler::new();
        let mut lexer = crate::Lexer::new(source, &mut handler);
        lexer.lex_raw_string()
    }

    fn lex_char(source: &str) -> Token {
        let mut handler = Handler::new();
        let mut lexer = crate::Lexer::new(source, &mut handler);
        lexer.lex_char()
    }

    #[test]
    fn test_simple_string() {
        let token = lex_str("\"hello\"");
        assert_eq!(token, Token::String(Symbol::intern("hello")));
    }

    #[test]
    fn test_string_with_escape() {
        let token = lex_str("\"hello\\nworld\"");
        assert_eq!(token, Token::String(Symbol::intern("hello\nworld")));
    }

    #[test]
    fn test_string_with_tab() {
        let token = lex_str("\"hello\\tworld\"");
        assert_eq!(token, Token::String(Symbol::intern("hello\tworld")));
    }

    #[test]
    fn test_raw_string() {
        let token = lex_raw_str("r\"hello\"");
        assert_eq!(token, Token::RawString(Symbol::intern("hello")));
    }

    #[test]
    fn test_raw_string_with_quotes() {
        let token = lex_raw_str("r#\"hello \"world\" #\"");
        assert_eq!(token, Token::RawString(Symbol::intern("hello \"world\" ")));
    }

    #[test]
    fn test_character() {
        let token = lex_char("'a'");
        assert_eq!(token, Token::Char('a'));
    }

    #[test]
    fn test_character_escape() {
        let token = lex_char("'\\n'");
        assert_eq!(token, Token::Char('\n'));
    }

    #[test]
    fn test_character_hex_escape() {
        let token = lex_char("'\\x41'");
        assert_eq!(token, Token::Char('A'));
    }
}
