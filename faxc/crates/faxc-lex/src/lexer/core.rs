//! Core lexer implementation.
//!
//! This module contains the main Lexer struct and its core methods.

use faxc_util::{DiagnosticBuilder, Handler, Span};

use crate::cursor::Cursor;
use crate::token::Token;

/// Lexer for the Fax programming language.
///
/// The lexer transforms source code text into a stream of tokens.
/// It handles whitespace, comments, identifiers, keywords, operators,
/// and literals.
pub struct Lexer<'a> {
    /// Character cursor for source traversal.
    pub cursor: Cursor<'a>,

    /// Error handler for reporting lexical errors.
    pub handler: &'a mut Handler,

    /// Starting position of the current token (byte offset).
    pub token_start: usize,

    /// Line number where the current token starts (1-based).
    token_start_line: u32,

    /// Column number where the current token starts (1-based).
    token_start_column: u32,

    /// Whether the BOM (Byte Order Mark) has been checked.
    pub bom_checked: bool,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer for the given source code.
    pub fn new(source: &'a str, handler: &'a mut Handler) -> Self {
        Self {
            cursor: Cursor::new(source),
            handler,
            token_start: 0,
            token_start_line: 1,
            token_start_column: 1,
            bom_checked: false,
        }
    }

    /// Returns the next token from the source code.
    ///
    /// This is the main entry point for tokenization. It skips whitespace
    /// and comments, then dispatches to the appropriate lexing method
    /// based on the current character.
    ///
    /// # Returns
    /// The next token in the source stream, or `Token::Eof` at end of file.
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        self.token_start = self.cursor.position();
        self.token_start_line = self.cursor.line();
        self.token_start_column = self.cursor.column();

        if self.cursor.is_at_end() {
            return Token::Eof;
        }

        match self.cursor.current_char() {
            '(' => {
                self.cursor.advance();
                Token::LParen
            },
            ')' => {
                self.cursor.advance();
                Token::RParen
            },
            '{' => {
                self.cursor.advance();
                Token::LBrace
            },
            '}' => {
                self.cursor.advance();
                Token::RBrace
            },
            '[' => {
                self.cursor.advance();
                Token::LBracket
            },
            ']' => {
                self.cursor.advance();
                Token::RBracket
            },
            ',' => {
                self.cursor.advance();
                Token::Comma
            },
            ';' => {
                self.cursor.advance();
                Token::Semicolon
            },
            '+' => self.lex_plus(),
            '-' => self.lex_minus(),
            '*' => self.lex_star(),
            '/' => self.lex_slash(),
            '%' => self.lex_percent(),
            '=' => self.lex_equals(),
            '!' => self.lex_bang(),
            '<' => self.lex_less(),
            '>' => self.lex_greater(),
            '&' => self.lex_ampersand(),
            '|' => self.lex_pipe(),
            ':' => self.lex_colon(),
            '.' => self.lex_dot(),
            '^' => self.lex_caret(),
            '~' => self.lex_tilde(),
            '"' => self.lex_string(),
            '\'' => self.lex_char(),
            '$' => {
                self.cursor.advance();
                Token::Dollar
            },
            '@' => {
                self.cursor.advance();
                Token::At
            },
            '_' => {
                self.cursor.advance();
                if crate::unicode::is_ascii_ident_continue(self.cursor.current_char()) {
                    self.lex_identifier()
                } else {
                    Token::Underscore
                }
            },
            'r' => {
                let next_char = self.cursor.peek_char(1);
                if next_char == '"' || next_char == '#' {
                    self.lex_raw_string()
                } else {
                    self.lex_identifier()
                }
            },
            c if crate::unicode::is_ascii_ident_start(c) => self.lex_identifier(),
            c if c.is_ascii_digit() => self.lex_number(),
            c => {
                self.report_error(format!("unexpected character '{}'", c));
                self.cursor.advance();
                Token::Invalid(c.to_string())
            },
        }
    }

    /// Reports a lexical error at the current token position.
    ///
    /// # Arguments
    /// * `message` - The error message to display
    pub fn report_error(&mut self, message: String) {
        let span = Span::new(
            self.token_start,
            self.cursor.position(),
            self.token_start_line,
            self.token_start_column,
        );
        DiagnosticBuilder::error(message)
            .span(span)
            .emit(self.handler);
    }

    /// Returns the current line number (1-based).
    ///
    /// # Returns
    /// The line number of the next token to be lexed.
    pub fn line(&self) -> u32 {
        self.cursor.line()
    }

    /// Returns the current column number (1-based).
    ///
    /// # Returns
    /// The column number of the next token to be lexed.
    pub fn column(&self) -> u32 {
        self.cursor.column()
    }

    /// Returns the current byte position in the source.
    ///
    /// # Returns
    /// The byte offset of the next character to be lexed.
    pub fn position(&self) -> usize {
        self.cursor.position()
    }

    /// Returns the starting position of the current token.
    ///
    /// # Returns
    /// The byte offset where the current token began.
    pub fn token_start(&mut self) -> usize {
        self.token_start
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        if token == Token::Eof {
            None
        } else {
            Some(token)
        }
    }
}
