use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenType {
    Identifier,
    String,
    Number,
    Boolean,
    Null,
    Keyword,
    Operator,
    Symbol,
    ScopeResolution,
    ReturnType,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Semicolon,
    Colon,
    Dot,
    EOF,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Token {
    #[serde(rename = "type")]
    pub typ: TokenType,
    pub val: String,
    pub line: usize,
    pub col: usize,
    pub span: Span,
    pub trivia: Vec<String>,
}

impl Token {
    pub fn new(typ: TokenType, val: String, line: usize, col: usize, span: Span) -> Self {
        Self {
            typ,
            val,
            line,
            col,
            span,
            trivia: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

pub static KEYWORDS: LazyLock<HashMap<&'static str, TokenType>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    let keys = [
        "fn",
        "let",
        "mut",
        "const",
        "module",
        "use",
        "pub",
        "priv",
        "internal",
        "struct",
        "enum",
        "trait",
        "impl",
        "type",
        "alias",
        "class",
        "interface",
        "extern",
        "as",
        "where",
        "if",
        "else",
        "while",
        "for",
        "loop",
        "match",
        "case",
        "default",
        "return",
        "break",
        "continue",
        "in",
        "extends",
        "implements",
        "super",
        "this",
        "new",
        "delete",
        "async",
        "await",
        "try",
        "catch",
        "finally",
        "throw",
        "spawn",
        "yield",
        "move",
        "ref",
        "unsafe",
        "dyn",
        "static",
        "self",
        "Self",
        "crate",
        "i8",
        "i16",
        "i32",
        "i64",
        "u8",
        "u16",
        "u32",
        "u64",
        "f32",
        "f64",
        "bool",
        "str",
        "char",
        "void",
        "any",
    ];
    for k in keys {
        m.insert(k, TokenType::Keyword);
    }
    m.insert("true", TokenType::Boolean);
    m.insert("false", TokenType::Boolean);
    m.insert("null", TokenType::Null);
    m
});
