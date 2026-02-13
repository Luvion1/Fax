use crate::lexer::token::Span;
use std::fmt;

#[derive(Debug, Clone)]
pub enum LexErr {
    UntermString {
        line: usize,
        col: usize,
        span: Span,
    },
    InvalidEsc {
        char: char,
        line: usize,
        col: usize,
        span: Span,
    },
    InvalidNum {
        val: String,
        line: usize,
        col: usize,
        span: Span,
    },
    UnexpectedChar {
        char: char,
        line: usize,
        col: usize,
        span: Span,
    },
    UnclosedCmt {
        line: usize,
        col: usize,
        span: Span,
    },
}

impl fmt::Display for LexErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexErr::UntermString { line, col, .. } => {
                write!(f, "Unterminated string at {}:{}", line, col)
            }
            LexErr::InvalidEsc {
                char, line, col, ..
            } => write!(f, "Invalid escape '\\{}' at {}:{}", char, line, col),
            LexErr::InvalidNum { val, line, col, .. } => {
                write!(f, "Invalid number '{}' at {}:{}", val, line, col)
            }
            LexErr::UnexpectedChar {
                char, line, col, ..
            } => write!(f, "Unexpected '{}' at {}:{}", char, line, col),
            LexErr::UnclosedCmt { line, col, .. } => {
                write!(f, "Unclosed comment at {}:{}", line, col)
            }
        }
    }
}

impl std::error::Error for LexErr {}
