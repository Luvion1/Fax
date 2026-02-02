use serde::{Serialize, Deserialize};
use std::env;
use std::fs;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TokenType {
    Keyword, Identifier, String, Number, Boolean, Symbol, Operator,
    ScopeResolution, // ::
    ReturnType,      // ->
    EOF,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Token {
    #[serde(rename = "type")]
    pub token_type: TokenType,
    pub value: String,
    pub line: usize,
    pub column: usize,
}

struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    fn new(content: &'a str) -> Self {
        Self {
            chars: content.chars().peekable(),
            line: 1,
            col: 1,
        }
    }

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    fn next(&mut self) -> Option<char> {
        let c = self.chars.next();
        if let Some('\n') = c {
            self.line += 1;
            self.col = 1;
        } else if c.is_some() {
            self.col += 1;
        }
        c
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.peek() {
            if c.is_whitespace() {
                self.next();
            } else {
                break;
            }
        }
    }

    fn lex(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(&c) = self.peek() {
            if c.is_whitespace() {
                self.skip_whitespace();
                continue;
            }

            let start_line = self.line;
            let start_col = self.col;

            match c {
                // Comments
                '/' => {
                    self.next();
                    if let Some('/') = self.peek() {
                        while let Some(&nc) = self.peek() {
                            if nc == '\n' { break; }
                            self.next();
                        }
                    } else if let Some('*') = self.peek() {
                        self.next();
                        while let Some(&nc) = self.peek() {
                            if nc == '*' {
                                self.next();
                                if let Some('/') = self.peek() {
                                    self.next();
                                    break;
                                }
                            } else {
                                self.next();
                            }
                        }
                    } else {
                        tokens.push(Token { token_type: TokenType::Operator, value: "/".to_string(), line: start_line, column: start_col });
                    }
                }
                ':' => {
                    self.next();
                    if let Some(':') = self.peek() {
                        self.next();
                        tokens.push(Token { token_type: TokenType::ScopeResolution, value: "::".to_string(), line: start_line, column: start_col });
                    } else {
                        tokens.push(Token { token_type: TokenType::Symbol, value: ":".to_string(), line: start_line, column: start_col });
                    }
                }
                '-' => {
                    self.next();
                    if let Some('>') = self.peek() {
                        self.next();
                        tokens.push(Token { token_type: TokenType::ReturnType, value: "->".to_string(), line: start_line, column: start_col });
                    } else if let Some('=') = self.peek() {
                        self.next();
                        tokens.push(Token { token_type: TokenType::Operator, value: "-=".to_string(), line: start_line, column: start_col });
                    } else {
                        tokens.push(Token { token_type: TokenType::Operator, value: "-".to_string(), line: start_line, column: start_col });
                    }
                }
                '=' | '!' | '<' | '>' | '+' | '*' | '&' | '|' | '%' | '^' => {
                    let first = self.next().unwrap();
                    let mut val = first.to_string();
                    if let Some(&next) = self.peek() {
                        if next == '=' || (first == '&' && next == '&') || (first == '|' && next == '|') {
                            val.push(self.next().unwrap());
                        }
                    }
                    tokens.push(Token { token_type: TokenType::Operator, value: val, line: start_line, column: start_col });
                }
                '"' => {
                    self.next();
                    let mut val = String::new();
                    let mut terminated = false;
                    while let Some(&nc) = self.peek() {
                        if nc == '"' { 
                            self.next(); 
                            terminated = true;
                            break; 
                        }
                        if nc == '\\' {
                            self.next();
                            match self.next() {
                                Some('n') => val.push('\n'),
                                Some('t') => val.push('\t'),
                                Some('\\') => val.push('\\'),
                                Some('"') => val.push('"'),
                                _ => {},
                            }
                        } else {
                            val.push(self.next().unwrap());
                        }
                    }
                    if !terminated {
                        eprintln!("Lexer Error: Unterminated string at line {}, col {}", start_line, start_col);
                    }
                    tokens.push(Token { token_type: TokenType::String, value: val, line: start_line, column: start_col });
                }
                _ if c.is_alphabetic() || c == '_' => {
                    let mut val = String::new();
                    while let Some(&nc) = self.peek() {
                        if nc.is_alphanumeric() || nc == '_' {
                            val.push(self.next().unwrap());
                        } else { break; }
                    }
                    let t_type = match val.as_str() {
                        // Declarations & Modules
                        "fn" | "let" | "mut" | "const" | "module" | "use" | "pub" | "priv" | "internal" | "struct" | "enum" | "trait" | "impl" | "type" | "alias" | "class" | "interface" | "extern" | "as" | "where" => TokenType::Keyword,
                        // Control Flow
                        "if" | "else" | "while" | "for" | "loop" | "match" | "case" | "default" | "return" | "break" | "continue" | "in" => TokenType::Keyword,
                        // OOP & Inheritance
                        "extends" | "implements" | "super" | "this" | "new" | "delete" => TokenType::Keyword,
                        // Async & Concurrency
                        "async" | "await" | "try" | "catch" | "finally" | "throw" | "spawn" | "yield" => TokenType::Keyword,
                        // Memory & Special
                        "move" | "ref" | "unsafe" | "dyn" | "static" | "self" | "Self" | "crate" => TokenType::Keyword,
                        // Built-in Types
                        "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "f32" | "f64" | "bool" | "str" | "char" | "void" | "any" => TokenType::Keyword,
                        // Booleans
                        "true" | "false" => TokenType::Boolean,
                        _ => TokenType::Identifier,
                    };
                    tokens.push(Token { token_type: t_type, value: val, line: start_line, column: start_col });
                }
                _ if c.is_numeric() => {
                    let mut val = String::new();
                    let mut dot_count = 0;
                    while let Some(&nc) = self.peek() {
                        if nc.is_numeric() {
                            val.push(self.next().unwrap());
                        } else if nc == '.' {
                            if dot_count > 0 { break; }
                            dot_count += 1;
                            val.push(self.next().unwrap());
                        } else { break; }
                    }
                    tokens.push(Token { token_type: TokenType::Number, value: val, line: start_line, column: start_col });
                }
                _ => {
                    let val = self.next().unwrap().to_string();
                    tokens.push(Token { token_type: TokenType::Symbol, value: val, line: start_line, column: start_col });
                }
            }
        }
        tokens
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { std::process::exit(1); }
    let content = fs::read_to_string(&args[1]).expect("Read error");
    let mut lexer = Lexer::new(&content);
    let tokens = lexer.lex();
    println!("{}", serde_json::to_string(&tokens).unwrap());
}