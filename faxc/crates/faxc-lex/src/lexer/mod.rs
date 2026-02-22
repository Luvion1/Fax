//! Lexer module.
//!
//! This module organizes the lexer implementation into smaller, focused components:
//! - `core` - Main Lexer struct and dispatch
//! - `identifier` - Identifier and keyword lexing
//! - `number` - Number literal lexing
//! - `string` - String and character literal lexing
//! - `operator` - Operator and punctuation lexing
//! - `comment` - Comment skipping

mod comment;
mod core;
mod identifier;
mod number;
mod operator;
mod string;

pub use core::Lexer;
