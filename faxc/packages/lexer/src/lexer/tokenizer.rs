use crate::lexer::{
    error::LexErr,
    token::{Span, Token, TokenType, KEYWORDS},
};
use std::{iter::Peekable, str::CharIndices};

pub struct Tokenizer<'a> {
    chars: Peekable<CharIndices<'a>>,
    line: usize,
    col: usize,
    off: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.char_indices().peekable(),
            line: 1,
            col: 1,
            off: 0,
        }
    }

    fn peek(&mut self) -> Option<&(usize, char)> {
        self.chars.peek()
    }

    fn next(&mut self) -> Option<char> {
        if let Some((off, c)) = self.chars.next() {
            self.off = off;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else if c == '\t' {
                self.col = self.col.div_ceil(8) * 8 + 1;
            } else {
                self.col += 1;
            }
            Some(c)
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&(_, c)) = self.peek() {
            if c.is_whitespace() {
                self.next();
            } else {
                break;
            }
        }
    }

    fn lex_string(&mut self) -> Result<Token, LexErr> {
        let (l, c, s) = (self.line, self.col, self.off);
        self.next();
        let mut v = String::new();
        let mut esc = false;

        while let Some(&(_, ch)) = self.peek() {
            match ch {
                '"' if !esc => {
                    self.next();
                    return Ok(Token::new(
                        TokenType::String,
                        v,
                        l,
                        c,
                        Span::new(s, self.off),
                    ));
                }
                '\\' if !esc => {
                    self.next();
                    esc = true;
                }
                _ => {
                    if let Some(nc) = self.next() {
                        if esc {
                            match nc {
                                'n' => v.push('\n'),
                                't' => v.push('\t'),
                                'r' => v.push('\r'),
                                '\\' => v.push('\\'),
                                '"' => v.push('"'),
                                '0' => v.push('\0'),
                                _ => {
                                    return Err(LexErr::InvalidEsc {
                                        char: nc,
                                        line: l,
                                        col: c,
                                        span: Span::new(s, self.off),
                                    })
                                }
                            }
                            esc = false;
                        } else {
                            v.push(nc);
                        }
                    } else {
                        return Err(LexErr::UntermString {
                            line: l,
                            col: c,
                            span: Span::new(s, self.off),
                        });
                    }
                }
            }
        }
        Err(LexErr::UntermString {
            line: l,
            col: c,
            span: Span::new(s, self.off),
        })
    }

    fn lex_number(&mut self) -> Result<Token, LexErr> {
        let (l, c, s) = (self.line, self.col, self.off);
        let mut v = String::new();
        let mut has_dot = false;

        while let Some(&(_, ch)) = self.peek() {
            if ch.is_ascii_digit() || ch == '_' || ch == 'e' || ch == 'E' || ch == 'x' || ch == 'b' {
                if let Some(c) = self.next() {
                    v.push(c);
                } else {
                    break;
                }
            } else if ch == '.' {
                if let Some(nc) = self.chars.clone().nth(1) {
                    if nc.1 == '.' {
                        break;
                    }
                }
                if has_dot {
                    break;
                }
                has_dot = true;
                if let Some(c) = self.next() {
                    v.push(c);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        Ok(Token::new(
            TokenType::Number,
            v,
            l,
            c,
            Span::new(s, self.off),
        ))
    }

    fn lex_identifier(&mut self) -> Result<Token, LexErr> {
        let (l, c, s) = (self.line, self.col, self.off);
        let mut v = String::new();

        while let Some(&(_, ch)) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                if let Some(c) = self.next() {
                    v.push(c);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        let t = KEYWORDS
            .get(v.as_str())
            .copied()
            .unwrap_or(TokenType::Identifier);
        Ok(Token::new(t, v, l, c, Span::new(s, self.off)))
    }

    fn lex_operator(&mut self, first: char) -> Result<Token, LexErr> {
        let (l, c, s) = (self.line, self.col, self.off);
        let mut v = String::from(first);
        self.next();

        if let Some(&(_, sec)) = self.peek() {
            match (first, sec) {
                ('=', '=')
                | ('!', '=')
                | ('<', '=')
                | ('>', '=')
                | ('-', '>')
                | (':', ':')
                | ('.', '.')
                | ('&', '&')
                | ('|', '|')
                | ('=', '>') => {
                    if let Some(c) = self.next() {
                        v.push(c);
                    }
                }
                _ => {}
            }
        }

        let t = match v.as_str() {
            "::" => TokenType::ScopeResolution,
            "->" => TokenType::ReturnType,
            ".." => TokenType::Range,
            "&&" => TokenType::LogicalAnd,
            "||" => TokenType::LogicalOr,
            "=>" => TokenType::Arrow,
            "!" => TokenType::Not,
            "(" => TokenType::LeftParen,
            ")" => TokenType::RightParen,
            "{" => TokenType::LeftBrace,
            "}" => TokenType::RightBrace,
            "[" => TokenType::LeftBracket,
            "]" => TokenType::RightBracket,
            "," => TokenType::Comma,
            ";" => TokenType::Semicolon,
            ":" => TokenType::Colon,
            "." => TokenType::Dot,
            _ => TokenType::Operator,
        };
        Ok(Token::new(t, v, l, c, Span::new(s, self.off)))
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexErr> {
        let mut tokens = Vec::new();

        while let Some(&(_, c)) = self.peek() {
            match c {
                c if c.is_whitespace() => self.skip_whitespace(),
                '/' => {
                    self.next();
                    if let Some(&(_, '/')) = self.peek() {
                        while let Some(nc) = self.next() {
                            if nc == '\n' {
                                break;
                            }
                        }
                    } else if let Some(&(_, '*')) = self.peek() {
                        self.next();
                        while let Some(nc) = self.next() {
                            if nc == '*' {
                                if let Some(&(_, '/')) = self.peek() {
                                    self.next();
                                    break;
                                }
                            }
                        }
                    } else {
                        tokens.push(self.lex_operator('/')?);
                    }
                }
                '"' => tokens.push(self.lex_string()?),
                c if c.is_ascii_digit() => tokens.push(self.lex_number()?),
                c if c.is_alphabetic() || c == '_' => tokens.push(self.lex_identifier()?),
                c => tokens.push(self.lex_operator(c)?),
            }
        }

        tokens.push(Token::new(
            TokenType::EOF,
            String::new(),
            self.line,
            self.col,
            Span::new(self.off, self.off),
        ));

        Ok(tokens)
    }
}
