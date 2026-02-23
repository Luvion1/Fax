//! Type parsing - primitive types, compound types, generics

use crate::ast::*;
use crate::Parser;
use faxc_lex::Token;

impl<'a> Parser<'a> {
    /// Parse type expression
    pub fn parse_type(&mut self) -> Option<Type> {
        match self.current_token() {
            Token::Ident(name) => {
                self.advance();
                let path = Path {
                    segments: vec![PathSegment {
                        ident: name,
                        args: None,
                    }],
                };

                if self.match_token(Token::Lt) {
                    let mut args = Vec::new();
                    while !self.is_at_end() && self.current_token() != Token::Gt {
                        if let Some(ty) = self.parse_type() {
                            args.push(ty);
                        }
                        if !self.match_token(Token::Comma) {
                            break;
                        }
                    }
                    self.expect(Token::Gt)?;
                    return Some(Type::Generic(Box::new(Type::Path(path)), args));
                }

                Some(Type::Path(path))
            },
            Token::LParen => {
                self.advance();

                if self.match_token(Token::RParen) {
                    return Some(Type::Unit);
                }

                let mut types = Vec::new();
                loop {
                    if let Some(ty) = self.parse_type() {
                        types.push(ty);
                    }
                    if !self.match_token(Token::Comma) {
                        break;
                    }
                }
                self.expect(Token::RParen)?;

                if types.len() == 1 {
                    Some(types.into_iter().next().unwrap())
                } else {
                    Some(Type::Tuple(types))
                }
            },
            Token::Ampersand => {
                self.advance();
                let mutable = self.match_token(Token::Mut);
                let ty = self.parse_type()?;
                Some(Type::Reference(
                    Box::new(ty),
                    if mutable {
                        Mutability::Mutable
                    } else {
                        Mutability::Immutable
                    },
                ))
            },
            Token::LBracket => {
                self.advance();
                let ty = self.parse_type()?;

                if self.match_token(Token::Semicolon) {
                    let _size = self.parse_expr();
                    self.expect(Token::RBracket)?;
                    Some(Type::Array(Box::new(ty), 0))
                } else {
                    self.expect(Token::RBracket)?;
                    Some(Type::Slice(Box::new(ty)))
                }
            },
            Token::Star => {
                self.advance();
                let mutable = self.match_token(Token::Mut);
                let ty = self.parse_type()?;
                Some(Type::Pointer(
                    Box::new(ty),
                    if mutable {
                        Mutability::Mutable
                    } else {
                        Mutability::Immutable
                    },
                ))
            },
            Token::Fn => {
                self.advance();
                self.expect(Token::LParen)?;

                let mut param_types = Vec::new();
                while !self.is_at_end() && self.current_token() != Token::RParen {
                    if let Some(ty) = self.parse_type() {
                        param_types.push(ty);
                    }
                    if !self.match_token(Token::Comma) {
                        break;
                    }
                }
                self.expect(Token::RParen)?;

                let ret_type = if self.match_token(Token::Arrow) {
                    self.parse_type()?
                } else {
                    Type::Unit
                };

                Some(Type::Fn(param_types, Box::new(ret_type)))
            },
            Token::Dyn => {
                self.advance();
                let mut traits = Vec::new();
                loop {
                    let path = self.parse_path();
                    traits.push(Type::Path(path));
                    if !self.match_token(Token::Plus) {
                        break;
                    }
                }
                Some(Type::TraitObject(traits))
            },
            Token::Underscore => {
                self.advance();
                Some(Type::Inferred)
            },
            _ => {
                self.error("expected type");
                None
            },
        }
    }

    /// Parse path (e.g., `std::io::Result`)
    pub fn parse_path(&mut self) -> Path {
        let mut segments = Vec::new();

        loop {
            let ident = match self.current_token() {
                Token::Ident(sym) => {
                    self.advance();
                    sym
                },
                Token::Self_ => {
                    self.advance();
                    faxc_util::Symbol::intern("self")
                },
                Token::SelfUpper => {
                    self.advance();
                    faxc_util::Symbol::intern("Self")
                },
                Token::Super => {
                    self.advance();
                    faxc_util::Symbol::intern("super")
                },
                Token::Crate => {
                    self.advance();
                    faxc_util::Symbol::intern("crate")
                },
                _ => break,
            };

            let args =
                if self.current_token() == Token::ColonColon && self.peek_token() == Token::Lt {
                    self.advance();
                    self.advance();
                    let mut types = Vec::new();
                    while !self.is_at_end() && self.current_token() != Token::Gt {
                        if let Some(ty) = self.parse_type() {
                            types.push(ty);
                        }
                        if !self.match_token(Token::Comma) {
                            break;
                        }
                    }
                    self.expect(Token::Gt);
                    Some(types)
                } else {
                    None
                };

            segments.push(PathSegment { ident, args });

            if !self.match_token(Token::ColonColon) {
                break;
            }

            if !matches!(
                self.current_token(),
                Token::Ident(_) | Token::Self_ | Token::SelfUpper | Token::Super | Token::Crate
            ) {
                break;
            }
        }

        Path { segments }
    }

    /// Parse identifier
    pub fn parse_ident(&mut self) -> Option<faxc_util::Symbol> {
        let sym = match self.current_token() {
            Token::Ident(s) => {
                self.advance();
                s
            },
            Token::Self_ => {
                self.advance();
                faxc_util::Symbol::intern("self")
            },
            Token::SelfUpper => {
                self.advance();
                faxc_util::Symbol::intern("Self")
            },
            Token::Super => {
                self.advance();
                faxc_util::Symbol::intern("super")
            },
            Token::Crate => {
                self.advance();
                faxc_util::Symbol::intern("crate")
            },
            _ => {
                self.error("expected identifier");
                return None;
            },
        };
        Some(sym)
    }
}
