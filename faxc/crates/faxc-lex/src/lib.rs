//! faxc-lex - Lexical Analyzer for the Fax Programming Language
//!
//! This crate provides a complete lexer (tokenizer) for the Fax programming
//! language. It transforms source code into a stream of tokens that can be
//! consumed by the parser.
//!
//! # Overview
//!
//! Lexical analysis is the first phase of compilation. It transforms a stream
//! of characters into a stream of tokens. This process is also called
//! "tokenization" or "scanning".
//!
//! # Example Usage
//!
//! ```
//! use faxc_util::Handler;
//! use faxc_lex::{Lexer, Token};
//!
//! let source = "let x = 42;";
//! let mut handler = Handler::new();
//! let mut lexer = Lexer::new(source, &mut handler);
//!
//! // Iterate through tokens
//! for token in &mut lexer {
//!     println!("{:?}", token);
//! }
//!
//! // Or get tokens one at a time
//! let mut lexer = Lexer::new(source, &mut handler);
//! assert_eq!(lexer.next_token(), Token::Let);
//! ```
//!
//! # Module Structure
//!
//! - [`token`] - Token type definitions
//! - [`lexer`] - Main lexer implementation
//! - [`cursor`] - Character cursor for source traversal
//! - [`unicode`] - Unicode utilities for identifier validation
//!
//! # Token Categories
//!
//! The lexer produces the following token types:
//!
//! ## Keywords
//!
//! Reserved words with special meaning (35 total):
//!
//! **Control Flow**: `fn`, `let`, `if`, `else`, `match`, `while`, `for`, `loop`, `break`, `continue`, `return`
//!
//! **Type System**: `struct`, `enum`, `trait`, `impl`, `dyn`, `type`, `where`
//!
//! **Module System**: `mod`, `use`, `as`, `super`, `crate`, `pub`
//!
//! **Advanced**: `async`, `await`, `const`, `static`, `unsafe`, `ref`, `mut`, `self`, `Self`, `true`, `false`, `macro_rules`
//!
//! ## Identifiers
//!
//! Names for variables, functions, types, etc. Pattern: `[a-zA-Z_][a-zA-Z0-9_]*`
//!
//! ## Literals
//!
//! - **Integer**: `42`, `0xFF`, `0b1010`, `0o777`
//! - **Float**: `3.14`, `1.0e10`, `2.5e-3`
//! - **String**: `"hello"`, `"world\n"`
//! - **Boolean**: `true`, `false`
//!
//! ## Operators
//!
//! - **Arithmetic**: `+`, `-`, `*`, `/`, `%`
//! - **Comparison**: `==`, `!=`, `<`, `>`, `<=`, `>=`
//! - **Logical**: `&&`, `||`, `!`
//! - **Assignment**: `=`, `+=`, `-=`, `*=`, `/=`, `%=`
//! - **Bitwise**: `&`, `|`, `^`, `<<`, `>>`
//!
//! ## Delimiters
//!
//! - **Grouping**: `()`, `{}`, `[]`
//! - **Separation**: `,`, `;`
//! - **Type annotation**: `:`, `::`
//! - **Access**: `.`, `..`, `...`, `->`, `=>`
//!
//! ## Special
//!
//! - **EOF**: End of file marker
//! - **Invalid**: Unrecognized characters

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod cursor;
pub mod lexer;
pub mod token;
pub mod unicode;

// Re-export main types for convenience
pub use cursor::Cursor;
pub use lexer::Lexer;
pub use token::{keyword_from_ident, Token};
pub use unicode::{
    codepoint_to_char, hex_digit_to_value, is_ascii_ident_continue, is_ascii_ident_start,
    is_digit_in_base, is_ident_continue, is_ident_start, parse_hex_byte, parse_hex_codepoint,
};

#[cfg(test)]
mod tests {
    use super::*;
    use faxc_util::Handler;

    /// Helper to collect all tokens from source.
    fn lex_all(source: &str) -> Vec<Token> {
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let mut tokens = Vec::new();
        loop {
            let token = lexer.next_token();
            if token == Token::Eof {
                break;
            }
            tokens.push(token);
        }
        tokens
    }

    #[test]
    fn test_hello_world_program() {
        let source = r#"
            fn main() {
                println("Hello, Fax!")
            }
        "#;
        let tokens = lex_all(source);

        assert!(tokens.contains(&Token::Fn));
        assert!(tokens.contains(&Token::Ident(faxc_util::Symbol::intern("main"))));
        assert!(tokens.contains(&Token::Ident(faxc_util::Symbol::intern("println"))));
        assert!(tokens.contains(&Token::String(faxc_util::Symbol::intern(
            "Hello, Fax!"
        ))));
    }

    #[test]
    fn test_fibonacci_program() {
        let source = r#"
            fn fib(n: i32) -> i32 {
                if n <= 1 {
                    n
                } else {
                    fib(n - 1) + fib(n - 2)
                }
            }
        "#;
        let tokens = lex_all(source);

        assert!(tokens.contains(&Token::Fn));
        assert!(tokens.contains(&Token::Ident(faxc_util::Symbol::intern("fib"))));
        assert!(tokens.contains(&Token::If));
        assert!(tokens.contains(&Token::Else));
        assert!(tokens.contains(&Token::LtEq));
        assert!(tokens.contains(&Token::Minus));
        assert!(tokens.contains(&Token::Plus));
    }

    #[test]
    fn test_struct_and_enum() {
        let source = r#"
            struct Point {
                x: f64,
                y: f64,
            }

            enum Color {
                Red,
                Green,
                Blue,
                RGB(i32, i32, i32),
            }
        "#;
        let tokens = lex_all(source);

        assert!(tokens.contains(&Token::Struct));
        assert!(tokens.contains(&Token::Enum));
        assert!(tokens.contains(&Token::Ident(faxc_util::Symbol::intern("Point"))));
        assert!(tokens.contains(&Token::Ident(faxc_util::Symbol::intern("Color"))));
        assert!(tokens.contains(&Token::Ident(faxc_util::Symbol::intern("RGB"))));
    }

    #[test]
    fn test_match_expression() {
        let source = r#"
            match value {
                0 => "zero",
                1 => "one",
                n if n > 10 => "large",
                _ => "other",
            }
        "#;
        let tokens = lex_all(source);

        assert!(tokens.contains(&Token::Match));
        assert!(tokens.contains(&Token::FatArrow));
        assert!(tokens.contains(&Token::Underscore));
        assert!(tokens.contains(&Token::Gt));
    }

    #[test]
    fn test_async_await() {
        let source = r#"
            async fn fetch(url: str) -> str {
                let response = http_get(url).await;
                response.body
            }
        "#;
        let tokens = lex_all(source);

        assert!(tokens.contains(&Token::Async));
        assert!(tokens.contains(&Token::Await));
        assert!(tokens.contains(&Token::Fn));
    }

    #[test]
    fn test_all_number_formats() {
        let source = "42 0xFF 0b1010 0o777 3.14 1e10 2.5e-3";
        let tokens = lex_all(source);

        assert_eq!(tokens[0], Token::Number(42));
        assert_eq!(tokens[1], Token::Number(0xFF));
        assert_eq!(tokens[2], Token::Number(0b1010));
        assert_eq!(tokens[3], Token::Number(0o777));
        assert!(matches!(tokens[4], Token::Float(f) if (f - 3.14).abs() < 0.001));
        assert!(matches!(tokens[5], Token::Float(f) if (f - 1e10).abs() < 1.0));
        assert!(matches!(tokens[6], Token::Float(f) if (f - 2.5e-3).abs() < 0.0001));
    }

    #[test]
    fn test_complex_string_escapes() {
        let source = r#""line1\nline2\ttab\\backslash\"quote\u{1F600}""#;
        let tokens = lex_all(source);

        assert_eq!(tokens.len(), 1);
        if let Token::String(s) = &tokens[0] {
            let content = s.as_str();
            assert!(content.contains('\n'));
            assert!(content.contains('\t'));
            assert!(content.contains('\\'));
            assert!(content.contains('"'));
            assert!(content.contains('😀'));
        } else {
            panic!("Expected String token");
        }
    }

    #[test]
    fn test_error_recovery_continues() {
        let source = "let x = # 42;";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);

        let token1 = lexer.next_token();
        assert_eq!(token1, Token::Let);

        let token2 = lexer.next_token();
        assert_eq!(token2, Token::Ident(faxc_util::Symbol::intern("x")));

        let token3 = lexer.next_token();
        assert_eq!(token3, Token::Eq);

        // Invalid character should produce error but lexer continues
        let token4 = lexer.next_token();
        assert!(matches!(token4, Token::Invalid(_)));

        // Lexer should continue after error
        let token5 = lexer.next_token();
        assert_eq!(token5, Token::Number(42));
    }

    #[test]
    fn test_line_column_tracking() {
        let source = "let\nx\n=\n42";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);

        // Before consuming any tokens, we're at line 1
        assert_eq!(lexer.line(), 1);
        let _ = lexer.next_token(); // Token::Let (consumes "let")

        // After consuming "let", cursor is at '\n', still line 1
        // After consuming '\n', line becomes 2
        let _ = lexer.next_token(); // This skips the newline, now at line 2
        assert_eq!(lexer.line(), 2);
        let _ = lexer.next_token(); // Token::Ident

        assert_eq!(lexer.line(), 3);
        let _ = lexer.next_token(); // Token::Eq

        assert_eq!(lexer.line(), 4);
        let _ = lexer.next_token(); // Token::Number
    }

    #[test]
    fn test_empty_source() {
        let tokens = lex_all("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_whitespace_only() {
        let tokens = lex_all("   \n\t  \n  ");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_comments_only() {
        let tokens = lex_all("// comment\n/* block */\n// another");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_spec_examples() {
        // Test examples from SPEC.md
        let source = r#"
            let x = 42
            let mut y = 10
            y = 20

            fn add(a: i32, b: i32) -> i32 {
                a + b
            }

            let max = if a > b { a } else { b }

            match value {
                0 => println("zero"),
                1 => println("one"),
                n if n > 10 => println("large"),
                _ => println("other"),
            }

            let mut i = 0
            while i < 5 {
                println(i)
                i = i + 1
            }
        "#;
        let tokens = lex_all(source);

        // Verify key tokens are present
        assert!(tokens.contains(&Token::Let));
        assert!(tokens.contains(&Token::Mut));
        assert!(tokens.contains(&Token::Fn));
        assert!(tokens.contains(&Token::If));
        assert!(tokens.contains(&Token::Else));
        assert!(tokens.contains(&Token::Match));
        assert!(tokens.contains(&Token::While));
        assert!(tokens.contains(&Token::FatArrow));
        assert!(tokens.contains(&Token::Gt));
        assert!(tokens.contains(&Token::Plus));
    }
}
