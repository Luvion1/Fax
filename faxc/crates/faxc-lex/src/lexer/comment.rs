//! Comment lexing.
//!
//! This module handles skipping line and block comments.

use crate::Lexer;

impl<'a> Lexer<'a> {
    /// Skips a block comment.
    ///
    /// Handles nested block comments with a depth counter.
    pub fn skip_block_comment(&mut self) {
        const MAX_COMMENT_DEPTH: u32 = 100;
        let mut depth = 1;

        while depth > 0 && !self.cursor.is_at_end() {
            if depth > MAX_COMMENT_DEPTH {
                self.report_error(format!(
                    "block comment nesting too deep (maximum is {} levels)",
                    MAX_COMMENT_DEPTH
                ));
                return;
            }

            if self.cursor.current_char() == '/' && self.cursor.peek_char(1) == '*' {
                self.cursor.advance();
                self.cursor.advance();
                depth += 1;
            } else if self.cursor.current_char() == '*' && self.cursor.peek_char(1) == '/' {
                self.cursor.advance();
                self.cursor.advance();
                depth -= 1;
                if depth == 1 {
                    return;
                }
            } else {
                self.cursor.advance();
            }
        }

        if depth > 0 {
            self.report_error("unterminated block comment".to_string());
        }
    }

    /// Skips whitespace and comments.
    ///
    /// Skips all whitespace characters and comments (both line and block).
    /// This is called before lexing each token.
    pub fn skip_whitespace_and_comments(&mut self) {
        if !self.bom_checked {
            self.bom_checked = true;
            if self.cursor.remaining().starts_with("\u{FEFF}") {
                self.cursor.advance();
            }
        }

        loop {
            if self.cursor.is_at_end() {
                return;
            }

            match self.cursor.current_char() {
                ' ' | '\t' | '\r' | '\n' => {
                    self.cursor.advance();
                },
                '/' => {
                    let next = self.cursor.peek_char(1);
                    if next == '/' {
                        self.skip_line_comment();
                    } else if next == '*' {
                        self.skip_block_comment();
                    } else {
                        return;
                    }
                },
                _ => return,
            }
        }
    }

    /// Skips a line comment (from // to end of line).
    fn skip_line_comment(&mut self) {
        self.cursor.advance();
        self.cursor.advance();

        while !self.cursor.is_at_end() && self.cursor.current_char() != '\n' {
            self.cursor.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use faxc_util::Handler;

    #[test]
    fn test_skip_whitespace() {
        let mut handler = Handler::new();
        let mut lexer = crate::Lexer::new("   hello", &mut handler);
        lexer.skip_whitespace_and_comments();
        assert_eq!(
            lexer.next_token(),
            crate::token::Token::Ident(faxc_util::Symbol::intern("hello"))
        );
    }

    #[test]
    fn test_skip_line_comment() {
        let mut handler = Handler::new();
        let mut lexer = crate::Lexer::new("// comment\nhello", &mut handler);
        lexer.skip_whitespace_and_comments();
        assert_eq!(
            lexer.next_token(),
            crate::token::Token::Ident(faxc_util::Symbol::intern("hello"))
        );
    }

    #[test]
    fn test_skip_block_comment() {
        let mut handler = Handler::new();
        let mut lexer = crate::Lexer::new("/* comment */hello", &mut handler);
        lexer.skip_whitespace_and_comments();
        assert_eq!(
            lexer.next_token(),
            crate::token::Token::Ident(faxc_util::Symbol::intern("hello"))
        );
    }

    #[test]
    fn test_skip_nested_block_comment() {
        let mut handler = Handler::new();
        let mut lexer = crate::Lexer::new("/* outer /* inner */ outer */hello", &mut handler);
        lexer.skip_whitespace_and_comments();
        assert_eq!(
            lexer.next_token(),
            crate::token::Token::Ident(faxc_util::Symbol::intern("hello"))
        );
    }
}
