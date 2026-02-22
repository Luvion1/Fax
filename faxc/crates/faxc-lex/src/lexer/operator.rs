//! Operator and punctuation lexing.
//!
//! This module handles lexing of operators, delimiters, and punctuation.

use crate::token::Token;
use crate::Lexer;

impl<'a> Lexer<'a> {
    /// Lexes plus or plus-equals.
    ///
    /// Handles: `+`, `+=`
    pub fn lex_plus(&mut self) -> Token {
        self.cursor.advance();
        if self.cursor.match_char('=') {
            Token::PlusEq
        } else {
            Token::Plus
        }
    }

    /// Lexes minus, arrow, or minus-equals.
    ///
    /// Handles: `-`, `->`, `-=`
    pub fn lex_minus(&mut self) -> Token {
        self.cursor.advance();
        if self.cursor.match_char('>') {
            Token::Arrow
        } else if self.cursor.match_char('=') {
            Token::MinusEq
        } else {
            Token::Minus
        }
    }

    /// Lexes star or star-equals.
    ///
    /// Handles: `*`, `*=`
    pub fn lex_star(&mut self) -> Token {
        self.cursor.advance();
        if self.cursor.match_char('=') {
            Token::StarEq
        } else {
            Token::Star
        }
    }

    /// Lexes slash, comment start, or slash-equals.
    ///
    /// Handles: `/`, `//`, `/* */`, `/=`
    pub fn lex_slash(&mut self) -> Token {
        self.cursor.advance();

        if self.cursor.match_char('/') {
            while !self.cursor.is_at_end() && self.cursor.current_char() != '\n' {
                self.cursor.advance();
            }
            self.skip_whitespace_and_comments();
            return self.next_token();
        }

        if self.cursor.match_char('*') {
            self.skip_block_comment();
            self.skip_whitespace_and_comments();
            return self.next_token();
        }

        if self.cursor.match_char('=') {
            Token::SlashEq
        } else {
            Token::Slash
        }
    }

    /// Lexes percent or percent-equals.
    ///
    /// Handles: `%`, `%=`
    pub fn lex_percent(&mut self) -> Token {
        self.cursor.advance();
        if self.cursor.match_char('=') {
            Token::PercentEq
        } else {
            Token::Percent
        }
    }

    /// Lexes equals, equals-equals, or fat arrow.
    ///
    /// Handles: `=`, `==`, `=>`
    pub fn lex_equals(&mut self) -> Token {
        self.cursor.advance();
        if self.cursor.match_char('=') {
            Token::EqEq
        } else if self.cursor.match_char('>') {
            Token::FatArrow
        } else {
            Token::Eq
        }
    }

    /// Lexes bang or not-equals.
    ///
    /// Handles: `!`, `!=`
    pub fn lex_bang(&mut self) -> Token {
        self.cursor.advance();
        if self.cursor.match_char('=') {
            Token::NotEq
        } else {
            Token::Bang
        }
    }

    /// Lexes less, less-equals, left shift, or shift-left-equals.
    ///
    /// Handles: `<`, `<=`, `<<`, `<<=`
    pub fn lex_less(&mut self) -> Token {
        self.cursor.advance();
        if self.cursor.match_char('=') {
            Token::LtEq
        } else if self.cursor.match_char('<') {
            if self.cursor.match_char('=') {
                Token::ShlEq
            } else {
                Token::Shl
            }
        } else {
            Token::Lt
        }
    }

    /// Lexes greater, greater-equals, right shift, or shift-right-equals.
    ///
    /// Handles: `>`, `>=`, `>>`, `>>=`
    pub fn lex_greater(&mut self) -> Token {
        self.cursor.advance();
        if self.cursor.match_char('=') {
            Token::GtEq
        } else if self.cursor.match_char('>') {
            if self.cursor.match_char('=') {
                Token::ShrEq
            } else {
                Token::Shr
            }
        } else {
            Token::Gt
        }
    }

    /// Lexes ampersand, logical and, or ampersand-equals.
    ///
    /// Handles: `&`, `&&`, `&=`
    pub fn lex_ampersand(&mut self) -> Token {
        self.cursor.advance();
        if self.cursor.match_char('&') {
            Token::AndAnd
        } else if self.cursor.match_char('=') {
            Token::AmpersandEq
        } else {
            Token::Ampersand
        }
    }

    /// Lexes pipe, logical or, or pipe-equals.
    ///
    /// Handles: `|`, `||`, `|=`
    pub fn lex_pipe(&mut self) -> Token {
        self.cursor.advance();
        if self.cursor.match_char('|') {
            Token::OrOr
        } else if self.cursor.match_char('=') {
            Token::PipeEq
        } else {
            Token::Pipe
        }
    }

    /// Lexes caret (bitwise XOR) or caret-equals.
    ///
    /// Handles: `^`, `^=`
    pub fn lex_caret(&mut self) -> Token {
        self.cursor.advance();
        if self.cursor.match_char('=') {
            Token::CaretEq
        } else {
            Token::Caret
        }
    }

    /// Lexes tilde (bitwise NOT).
    ///
    /// Handles: `~`
    pub fn lex_tilde(&mut self) -> Token {
        self.cursor.advance();
        Token::Tilde
    }

    /// Lexes colon or double colon.
    ///
    /// Handles: `:`, `::`
    pub fn lex_colon(&mut self) -> Token {
        self.cursor.advance();
        if self.cursor.match_char(':') {
            Token::ColonColon
        } else {
            Token::Colon
        }
    }

    /// Lexes dot, double dot, inclusive range, or triple dot.
    ///
    /// Handles: `.`, `..`, `..=`, `...`
    pub fn lex_dot(&mut self) -> Token {
        self.cursor.advance();
        if self.cursor.match_char('.') {
            if self.cursor.match_char('=') {
                Token::DotDotEq
            } else if self.cursor.match_char('.') {
                Token::DotDotDot
            } else {
                Token::DotDot
            }
        } else {
            Token::Dot
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Token;
    use faxc_util::Handler;

    fn lex_op(source: &str) -> Token {
        let mut handler = Handler::new();
        let mut lexer = crate::Lexer::new(source, &mut handler);
        lexer.next_token()
    }

    #[test]
    fn test_plus() {
        assert_eq!(lex_op("+"), Token::Plus);
    }

    #[test]
    fn test_plus_eq() {
        assert_eq!(lex_op("+="), Token::PlusEq);
    }

    #[test]
    fn test_minus() {
        assert_eq!(lex_op("-"), Token::Minus);
    }

    #[test]
    fn test_minus_eq() {
        assert_eq!(lex_op("-="), Token::MinusEq);
    }

    #[test]
    fn test_arrow() {
        assert_eq!(lex_op("->"), Token::Arrow);
    }

    #[test]
    fn test_star() {
        assert_eq!(lex_op("*"), Token::Star);
    }

    #[test]
    fn test_star_eq() {
        assert_eq!(lex_op("*="), Token::StarEq);
    }

    #[test]
    fn test_slash() {
        assert_eq!(lex_op("/"), Token::Slash);
    }

    #[test]
    fn test_slash_eq() {
        assert_eq!(lex_op("/="), Token::SlashEq);
    }

    #[test]
    fn test_eq() {
        assert_eq!(lex_op("="), Token::Eq);
    }

    #[test]
    fn test_eq_eq() {
        assert_eq!(lex_op("=="), Token::EqEq);
    }

    #[test]
    fn test_fat_arrow() {
        assert_eq!(lex_op("=>"), Token::FatArrow);
    }

    #[test]
    fn test_bang() {
        assert_eq!(lex_op("!"), Token::Bang);
    }

    #[test]
    fn test_not_eq() {
        assert_eq!(lex_op("!="), Token::NotEq);
    }

    #[test]
    fn test_lt() {
        assert_eq!(lex_op("<"), Token::Lt);
    }

    #[test]
    fn test_lt_eq() {
        assert_eq!(lex_op("<="), Token::LtEq);
    }

    #[test]
    fn test_gt() {
        assert_eq!(lex_op(">"), Token::Gt);
    }

    #[test]
    fn test_gt_eq() {
        assert_eq!(lex_op(">="), Token::GtEq);
    }

    #[test]
    fn test_and() {
        assert_eq!(lex_op("&"), Token::Ampersand);
    }

    #[test]
    fn test_and_and() {
        assert_eq!(lex_op("&&"), Token::AndAnd);
    }

    #[test]
    fn test_or() {
        assert_eq!(lex_op("|"), Token::Pipe);
    }

    #[test]
    fn test_or_or() {
        assert_eq!(lex_op("||"), Token::OrOr);
    }

    #[test]
    fn test_colon() {
        assert_eq!(lex_op(":"), Token::Colon);
    }

    #[test]
    fn test_colon_colon() {
        assert_eq!(lex_op("::"), Token::ColonColon);
    }

    #[test]
    fn test_dot() {
        assert_eq!(lex_op("."), Token::Dot);
    }

    #[test]
    fn test_dot_dot() {
        assert_eq!(lex_op(".."), Token::DotDot);
    }

    #[test]
    fn test_dot_dot_eq() {
        assert_eq!(lex_op("..="), Token::DotDotEq);
    }

    #[test]
    fn test_dot_dot_dot() {
        assert_eq!(lex_op("..."), Token::DotDotDot);
    }

    #[test]
    fn test_shl() {
        assert_eq!(lex_op("<<"), Token::Shl);
    }

    #[test]
    fn test_shr() {
        assert_eq!(lex_op(">>"), Token::Shr);
    }
}
