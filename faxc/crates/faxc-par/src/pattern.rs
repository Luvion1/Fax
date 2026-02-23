//! Pattern parsing - wildcard, identifier, literal, tuple, struct, enum patterns

use crate::ast::*;
use crate::Parser;
use faxc_lex::Token;

impl<'a> Parser<'a> {
    /// Parse pattern
    pub fn parse_pattern(&mut self) -> Option<Pattern> {
        match self.current_token() {
            Token::Underscore => {
                self.advance();
                Some(Pattern::Wildcard)
            },
            Token::Ident(name) => {
                self.advance();
                Some(Pattern::Ident(name, Mutability::Immutable))
            },
            Token::Number(n) => {
                self.advance();
                Some(Pattern::Literal(Literal::Int(n as i64)))
            },
            Token::True => {
                self.advance();
                Some(Pattern::Literal(Literal::Bool(true)))
            },
            Token::False => {
                self.advance();
                Some(Pattern::Literal(Literal::Bool(false)))
            },
            Token::String(s) => {
                self.advance();
                Some(Pattern::Literal(Literal::String(s)))
            },
            Token::Char(c) => {
                self.advance();
                Some(Pattern::Literal(Literal::Char(c)))
            },
            Token::LParen => {
                self.advance();

                if self.match_token(Token::RParen) {
                    return Some(Pattern::Tuple(Vec::new()));
                }

                let mut patterns = Vec::new();
                loop {
                    if let Some(pat) = self.parse_pattern() {
                        patterns.push(pat);
                    }
                    if !self.match_token(Token::Comma) {
                        break;
                    }
                }
                self.expect(Token::RParen)?;
                Some(Pattern::Tuple(patterns))
            },
            Token::Self_ | Token::SelfUpper => {
                let path = self.parse_path();

                if self.match_token(Token::LParen) {
                    let mut patterns = Vec::new();
                    while !self.is_at_end() && self.current_token() != Token::RParen {
                        if let Some(pat) = self.parse_pattern() {
                            patterns.push(pat);
                        }
                        if !self.match_token(Token::Comma) {
                            break;
                        }
                    }
                    self.expect(Token::RParen)?;
                    Some(Pattern::TupleStruct(path, patterns))
                } else {
                    Some(Pattern::Path(path))
                }
            },
            _ => {
                self.error("expected pattern");
                None
            },
        }
    }
}
