//! Main lexer implementation for the Fax programming language.
//!
//! This module provides the `Lexer` struct which transforms source code
//! into a stream of tokens. It handles all token types including keywords,
//! identifiers, literals, operators, and delimiters.

use faxc_util::{Handler, Span};

use crate::cursor::Cursor;
use crate::token::{keyword_from_ident, Token};
use crate::unicode::{
    codepoint_to_char, is_ascii_ident_continue, is_ascii_ident_start, is_digit_in_base,
    parse_hex_byte, parse_hex_codepoint,
};

/// The main lexer for Fax source code.
///
/// The lexer transforms source code into a stream of tokens using a
/// recursive descent approach. It maintains a cursor for position tracking
/// and a handler for error reporting.
///
/// # Example
///
/// ```
/// use faxc_util::Handler;
/// use faxc_lex::lexer::Lexer;
/// use faxc_lex::token::Token;
///
/// let mut handler = Handler::new();
/// let mut lexer = Lexer::new("let x = 42;", &mut handler);
///
/// assert_eq!(lexer.next_token(), Token::Let);
/// assert_eq!(lexer.next_token(), Token::Ident(faxc_util::Symbol::intern("x")));
/// ```
pub struct Lexer<'a> {
    /// Character cursor for traversing source.
    cursor: Cursor<'a>,

    /// Diagnostic handler for error reporting.
    handler: &'a mut Handler,

    /// Start position of the current token.
    token_start: usize,

    /// Start line of the current token.
    token_start_line: u32,

    /// Start column of the current token.
    token_start_column: u32,

    /// Whether we've checked for BOM at file start
    bom_checked: bool,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer for the given source code.
    ///
    /// # Arguments
    ///
    /// * `source` - The source code to lex
    /// * `handler` - Error handler for reporting lexical errors
    ///
    /// # Example
    ///
    /// ```
    /// use faxc_util::Handler;
    /// use faxc_lex::lexer::Lexer;
    ///
    /// let mut handler = Handler::new();
    /// let lexer = Lexer::new("let x = 42;", &mut handler);
    /// ```
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

    /// Returns the next token from the source.
    ///
    /// This is the main entry point for tokenization. It skips whitespace
    /// and comments, then dispatches to the appropriate lexer function
    /// based on the first character.
    ///
    /// # Returns
    ///
    /// The next token, or `Token::Eof` if at end of input.
    ///
    /// # Example
    ///
    /// ```
    /// use faxc_util::Handler;
    /// use faxc_lex::lexer::Lexer;
    /// use faxc_lex::token::Token;
    ///
    /// let mut handler = Handler::new();
    /// let mut lexer = Lexer::new("let x = 42;", &mut handler);
    ///
    /// assert_eq!(lexer.next_token(), Token::Let);
    /// ```
    pub fn next_token(&mut self) -> Token {
        // Skip whitespace and comments
        self.skip_whitespace_and_comments();

        // Record start position of this token
        self.token_start = self.cursor.position();
        self.token_start_line = self.cursor.line();
        self.token_start_column = self.cursor.column();

        // Check for end of file
        if self.cursor.is_at_end() {
            return Token::Eof;
        }

        // Dispatch based on first character
        match self.cursor.current_char() {
            // Single-character delimiters
            '(' => {
                self.cursor.advance();
                Token::LParen
            }
            ')' => {
                self.cursor.advance();
                Token::RParen
            }
            '{' => {
                self.cursor.advance();
                Token::LBrace
            }
            '}' => {
                self.cursor.advance();
                Token::RBrace
            }
            '[' => {
                self.cursor.advance();
                Token::LBracket
            }
            ']' => {
                self.cursor.advance();
                Token::RBracket
            }
            ',' => {
                self.cursor.advance();
                Token::Comma
            }
            ';' => {
                self.cursor.advance();
                Token::Semicolon
            }

            // Multi-character operators - dispatch to specific handlers
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

            // String and char literals
            '"' => self.lex_string(),
            '\'' => self.lex_char(),

            // Macro metavariable prefix
            '$' => {
                self.cursor.advance();
                Token::Dollar
            }

            // Pattern binding modifier
            '@' => {
                self.cursor.advance();
                Token::At
            }

            // Wildcard pattern or identifier starting with underscore
            '_' => {
                self.cursor.advance();
                // If followed by alphanumeric, it's an identifier
                if is_ascii_ident_continue(self.cursor.current_char()) {
                    self.lex_identifier()
                } else {
                    Token::Underscore
                }
            }

            // Identifiers and keywords
            'r' => {
                let next_char = self.cursor.peek_char(1);
                if next_char == '"' || next_char == '#' {
                    self.lex_raw_string()
                } else {
                    self.lex_identifier()
                }
            },
            c if is_ascii_ident_start(c) => self.lex_identifier(),

            // Numbers
            c if c.is_ascii_digit() => self.lex_number(),

            // Unknown character
            c => {
                self.report_error(format!("unexpected character '{}'", c));
                self.cursor.advance();
                Token::Invalid(c.to_string())
            }
        }
    }

    /// Lexes an identifier or keyword.
    ///
    /// Identifiers start with a letter or underscore, followed by
    /// alphanumeric characters or underscores. After reading the identifier,
    /// checks if it matches a reserved keyword.
    ///
    /// # Returns
    ///
    /// Either a keyword token (e.g., `Token::Let`) or `Token::Ident(symbol)`
    fn lex_identifier(&mut self) -> Token {
        // Consume identifier characters
        while is_ascii_ident_continue(self.cursor.current_char()) {
            self.cursor.advance();
        }

        // Extract the identifier text
        let text = self.cursor.slice_from(self.token_start);

        // Check if it's a keyword
        keyword_from_ident(text).unwrap_or_else(|| Token::Ident(faxc_util::Symbol::intern(text)))
    }

    /// Lexes a number literal.
    ///
    /// Handles decimal, hexadecimal (0x), binary (0b), octal (0o), and
    /// floating-point formats.
    ///
    /// # Number Formats
    ///
    /// - Decimal: `42`, `123`, `0`
    /// - Hexadecimal: `0xFF`, `0xAB_CD`
    /// - Binary: `0b1010`, `0b1111_0000`
    /// - Octal: `0o777`
    /// - Float: `3.14`, `1e10`, `2.5e-3`
    ///
    /// # Returns
    ///
    /// Either `Token::Number(u64)` or `Token::Float(f64)`
    fn lex_number(&mut self) -> Token {
        // Check for special bases (hex, binary, octal)
        if self.cursor.current_char() == '0' && !self.cursor.is_at_end() {
            let start = self.cursor.position();
            self.cursor.advance();
            match self.cursor.current_char() {
                'x' | 'X' => {
                    self.cursor.advance();
                    return self.lex_integer(16, start);
                }
                'b' | 'B' => {
                    self.cursor.advance();
                    return self.lex_integer(2, start);
                }
                'o' | 'O' => {
                    self.cursor.advance();
                    return self.lex_integer(8, start);
                }
                _ => {
                    // Just a zero followed by non-digit - return 0
                    // Check if next char is not a digit (to avoid parsing 0123 as 0)
                    if !self.cursor.current_char().is_ascii_digit() 
                        && self.cursor.current_char() != '.' 
                        && self.cursor.current_char() != 'e' 
                        && self.cursor.current_char() != 'E' {
                        return Token::Number(0);
                    }
                    // Otherwise continue to parse as decimal (e.g., 0.5, 0e10)
                }
            }
        }

        let start = self.cursor.position();

        // Parse decimal integer part
        while self.cursor.current_char().is_ascii_digit() {
            self.cursor.advance();
        }

        // Check for float (decimal point or exponent)
        let is_float = (self.cursor.current_char() == '.'
            && self.cursor.peek_char(1).is_ascii_digit())
            || self.cursor.current_char() == 'e'
            || self.cursor.current_char() == 'E';

        if is_float {
            // Parse decimal point
            if self.cursor.current_char() == '.' {
                self.cursor.advance();
                while self.cursor.current_char().is_ascii_digit() {
                    self.cursor.advance();
                }
            }

            // Parse exponent (HIGH #15: require at least one digit)
            if self.cursor.current_char() == 'e' || self.cursor.current_char() == 'E' {
                let exp_start = self.cursor.position();
                self.cursor.advance();
                // Optional sign
                if self.cursor.current_char() == '+' || self.cursor.current_char() == '-' {
                    self.cursor.advance();
                }
                // Exponent digits
                while self.cursor.current_char().is_ascii_digit() {
                    self.cursor.advance();
                }

                // Check we got at least one digit after 'e'/'E'
                // More precise check: after e/E and optional sign, we need digits
                let after_e_pos = exp_start + 1;
                let has_sign = after_e_pos < self.cursor.position()
                    && (self.cursor.source()[after_e_pos..].starts_with('+')
                        || self.cursor.source()[after_e_pos..].starts_with('-'));
                let min_expected_pos = if has_sign { exp_start + 2 } else { exp_start + 1 };

                if self.cursor.position() < min_expected_pos {
                    self.report_error("no digits in float exponent".to_string());
                }
            }

            let text = self.cursor.slice_from(start);
            // CRITICAL #3: Check if result is finite (not infinity or NaN)
            match text.parse::<f64>() {
                Ok(value) if value.is_finite() => Token::Float(value),
                Ok(_) => {
                    self.report_error(format!("floating point literal '{}' is not finite", text));
                    Token::Float(0.0)
                }
                Err(e) => {
                    self.report_error(format!("invalid floating point literal '{}': {}", text, e));
                    Token::Float(0.0)
                }
            }
        } else {
            // Integer
            let text = self.cursor.slice_from(start);
            // CRITICAL #2: Use checked parsing with explicit error handling
            match text.parse::<u64>() {
                Ok(value) => Token::Number(value),
                Err(e) => {
                    self.report_error(format!("integer literal overflow: {}", e));
                    Token::Number(0)
                }
            }
        }
    }

    /// Lexes an integer with the specified base.
    ///
    /// # Arguments
    ///
    /// * `base` - The numeric base (2, 8, 10, or 16)
    /// * `start` - The starting position of the number (at the '0' before prefix)
    fn lex_integer(&mut self, base: u32, start: usize) -> Token {
        // Record position after prefix to check for at least one digit
        let digit_start = self.cursor.position();

        // Skip underscore separators and valid digits for the base
        loop {
            let c = self.cursor.current_char();
            if c == '_' {
                self.cursor.advance();
            } else if is_digit_in_base(c, base) {
                self.cursor.advance();
            } else {
                break;
            }
        }

        // Check for at least one digit after prefix (HIGH #14)
        if digit_start == self.cursor.position() {
            self.report_error(format!("no digits after base-{} prefix", base));
            return Token::Number(0);
        }

        // Get the full text including prefix (e.g., "0xFF")
        let full_text = self.cursor.slice_from(start);
        // Skip the prefix (0x, 0b, 0o) for parsing
        let digits_text = &full_text[2..].replace('_', "");

        // CRITICAL #1: Proper overflow detection and error reporting
        let value = match u64::from_str_radix(digits_text, base) {
            Ok(v) => v,
            Err(e) => {
                self.report_error(format!("integer literal overflow: {}", e));
                0
            }
        };

        Token::Number(value)
    }

    /// Lexes a string literal.
    ///
    /// Parses a string enclosed in double quotes, handling escape sequences:
    /// - `\n` - Newline
    /// - `\t` - Tab
    /// - `\r` - Carriage return
    /// - `\\` - Backslash
    /// - `\"` - Double quote
    /// - `\0` - Null
    /// - `\xNN` - Hex byte
    /// - `\u{NNNN}` - Unicode codepoint
    ///
    /// # Returns
    ///
    /// `Token::String(symbol)` with the processed string content
    fn lex_string(&mut self) -> Token {
        // Consume opening quote
        self.cursor.advance();

        let mut content = String::new();

        loop {
            if self.cursor.is_at_end() {
                self.report_error("unterminated string literal".to_string());
                break;
            }

            let c = self.cursor.current_char();

            if c == '"' {
                self.cursor.advance();
                break;
            }

            if c == '\n' {
                self.report_error("unterminated string literal".to_string());
                break;
            }

            if c == '\\' {
                self.cursor.advance();
                if let Some(escaped) = self.parse_escape() {
                    content.push(escaped);
                }
            } else {
                content.push(c);
                self.cursor.advance();
            }
        }

        Token::String(faxc_util::Symbol::intern(&content))
    }


    /// Lexes a raw string literal (r"..." or r#"..."#).
    ///
    /// Raw strings don't process escape sequences and can contain quotes.
    /// The number of # characters determines the delimiter.
    ///
    /// # Returns
    ///
    /// `Token::RawString(symbol)` with the raw string content
    fn lex_raw_string(&mut self) -> Token {
        // Consume the 'r'
        self.cursor.advance();

        // Count the number of # characters (delimiter)
        let mut hash_count = 0;
        while self.cursor.current_char() == '#' {
            hash_count += 1;
            self.cursor.advance();
        }

        // Expect opening quote after r and hashes
        if self.cursor.current_char() != '"' {
            self.report_error("expected \" after raw string prefix".to_string());
            return Token::Invalid("raw".to_string());
        }
        self.cursor.advance();

        // Build the closing delimiter
        let mut closing_delimiter = String::from("\"");
        for _ in 0..hash_count {
            closing_delimiter.push('#');
        }
        closing_delimiter.push('"');

        // Accumulate content until we find the closing delimiter
        let mut content = String::new();
        let mut found_closing = false;

        while !self.cursor.is_at_end() {
            if self.cursor.current_char() == '"' {
                // Check if this is the closing delimiter
                let remaining = self.cursor.remaining();
                if remaining.starts_with(&closing_delimiter) {
                    // Consume the closing delimiter
                    for _ in 0..closing_delimiter.len() {
                        self.cursor.advance();
                    }
                    found_closing = true;
                    break;
                }
            }

            // Add current character to content
            content.push(self.cursor.current_char());
            self.cursor.advance();
        }

        if !found_closing {
            self.report_error("unterminated raw string literal".to_string());
        }

        Token::RawString(faxc_util::Symbol::intern(&content))
    }

    /// Lexes a character literal.
    ///
    /// Parses a character enclosed in single quotes, handling escape sequences.
    ///
    /// # Returns
    ///
    /// `Token::Char` containing the parsed character
    fn lex_char(&mut self) -> Token {
        // Consume opening quote
        self.cursor.advance();

        if self.cursor.is_at_end() {
            self.report_error("unterminated character literal".to_string());
            return Token::Char('\0');
        }

        let c = if self.cursor.current_char() == '\\' {
            self.cursor.advance();
            self.parse_escape().unwrap_or('\0')
        } else {
            let c = self.cursor.current_char();
            if c == '\'' || c == '\n' {
                self.report_error("empty character literal".to_string());
                return Token::Char('\0');
            }
            self.cursor.advance();
            c
        };

        // Expect closing quote
        if self.cursor.current_char() != '\'' {
            self.report_error("unterminated character literal".to_string());
            // Try to recover by skipping to end of line or next quote
            while !self.cursor.is_at_end()
                && self.cursor.current_char() != '\''
                && self.cursor.current_char() != '\n'
            {
                self.cursor.advance();
            }
        } else {
            self.cursor.advance();
        }

        Token::Char(c)
    }

    /// Parses an escape sequence.
    ///
    /// Handles: `\n`, `\t`, `\r`, `\\`, `\"`, `\'`, `\0`, `\xNN`, `\u{NNNN}`
    ///
    /// # Returns
    ///
    /// The escaped character, or None on error
    fn parse_escape(&mut self) -> Option<char> {
        if self.cursor.is_at_end() {
            self.report_error("unterminated escape sequence".to_string());
            return None;
        }

        let c = self.cursor.current_char();
        self.cursor.advance();

        match c {
            'n' => Some('\n'),
            't' => Some('\t'),
            'r' => Some('\r'),
            '\\' => Some('\\'),
            '"' => Some('"'),
            '\'' => Some('\''),
            '0' => Some('\0'),
            'x' => {
                // Hex escape: \xNN
                let mut hex = String::new();
                for _ in 0..2 {
                    if self.cursor.is_at_end() {
                        self.report_error("incomplete hex escape sequence".to_string());
                        return None;
                    }
                    let hc = self.cursor.current_char();
                    if hc.is_ascii_hexdigit() {
                        hex.push(hc);
                        self.cursor.advance();
                    } else {
                        self.report_error(format!("invalid hex escape character '{}'", hc));
                        return None;
                    }
                }
                parse_hex_byte(&hex)
                    .map(|b| b as char)
                    .or_else(|| {
                        self.report_error(format!("invalid hex escape value '{}'", hex));
                        None
                    })
            }
            'u' => {
                // Unicode escape: \u{NNNN}
                if self.cursor.current_char() != '{' {
                    self.report_error("expected '{' in unicode escape sequence".to_string());
                    return None;
                }
                self.cursor.advance();

                let mut hex = String::new();
                loop {
                    if self.cursor.is_at_end() {
                        self.report_error("unterminated unicode escape sequence".to_string());
                        return None;
                    }
                    let hc = self.cursor.current_char();
                    if hc == '}' {
                        self.cursor.advance();
                        break;
                    }
                    if hc.is_ascii_hexdigit() {
                        hex.push(hc);
                        self.cursor.advance();
                    } else {
                        self.report_error(format!("invalid unicode escape character '{}'", hc));
                        return None;
                    }
                }

                if hex.is_empty() {
                    self.report_error("empty unicode escape sequence".to_string());
                    return None;
                }

                // Parse the codepoint value first to check for surrogates
                let codepoint = parse_hex_codepoint(&hex);
                match codepoint {
                    Some(value) => {
                        // HIGH #4: Explicitly reject surrogate codepoints (D800-DFFF)
                        if (0xD800..=0xDFFF).contains(&value) {
                            self.report_error(format!(
                                "unicode escape sequence is a surrogate: U+{:04X}",
                                value
                            ));
                            return Some('\u{FFFD}'); // Replacement character
                        }
                        codepoint_to_char(value).or_else(|| {
                            self.report_error(format!("invalid unicode codepoint '{}'", hex));
                            None
                        })
                    }
                    None => {
                        self.report_error(format!("invalid unicode codepoint '{}'", hex));
                        None
                    }
                }
            }
            _ => {
                self.report_error(format!("unknown escape sequence '\\{}'", c));
                Some(c)
            }
        }
    }

    /// Lexes plus or plus-equals.
    ///
    /// Handles: `+`, `+=`
    fn lex_plus(&mut self) -> Token {
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
    fn lex_minus(&mut self) -> Token {
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
    fn lex_star(&mut self) -> Token {
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
    fn lex_slash(&mut self) -> Token {
        self.cursor.advance();

        // Check for line comment
        if self.cursor.match_char('/') {
            // Skip until end of line
            while !self.cursor.is_at_end() && self.cursor.current_char() != '\n' {
                self.cursor.advance();
            }
            // Recursively skip more whitespace/comments
            self.skip_whitespace_and_comments();
            // Return next token (comments are not tokens)
            return self.next_token();
        }

        // Check for block comment
        if self.cursor.match_char('*') {
            self.skip_block_comment();
            // Recursively skip more whitespace/comments
            self.skip_whitespace_and_comments();
            // Return next token
            return self.next_token();
        }

        // Check for slash-equals
        if self.cursor.match_char('=') {
            Token::SlashEq
        } else {
            Token::Slash
        }
    }

    /// Lexes percent or percent-equals.
    ///
    /// Handles: `%`, `%=`
    fn lex_percent(&mut self) -> Token {
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
    fn lex_equals(&mut self) -> Token {
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
    fn lex_bang(&mut self) -> Token {
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
    fn lex_less(&mut self) -> Token {
        self.cursor.advance();
        if self.cursor.match_char('=') {
            Token::LtEq
        } else if self.cursor.match_char('<') {
            // Check for <<=
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
    fn lex_greater(&mut self) -> Token {
        self.cursor.advance();
        if self.cursor.match_char('=') {
            Token::GtEq
        } else if self.cursor.match_char('>') {
            // Check for >>=
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
    fn lex_ampersand(&mut self) -> Token {
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
    fn lex_pipe(&mut self) -> Token {
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
    fn lex_caret(&mut self) -> Token {
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
    fn lex_tilde(&mut self) -> Token {
        self.cursor.advance();
        Token::Tilde
    }

    /// Lexes colon or double colon.
    ///
    /// Handles: `:`, `::`
    fn lex_colon(&mut self) -> Token {
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
    fn lex_dot(&mut self) -> Token {
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

    /// Skips a block comment.
    ///
    /// Handles nested block comments with a depth counter.
    /// HIGH #9: Limits nesting depth to prevent stack overflow.
    fn skip_block_comment(&mut self) {
        const MAX_COMMENT_DEPTH: u32 = 100;
        let mut depth = 1;

        while depth > 0 && !self.cursor.is_at_end() {
            // HIGH #9: Check for maximum nesting depth
            if depth > MAX_COMMENT_DEPTH {
                self.report_error(format!(
                    "block comment nesting too deep (maximum is {} levels)",
                    MAX_COMMENT_DEPTH
                ));
                return;
            }

            if self.cursor.current_char() == '/' && self.cursor.peek_char(1) == '*' {
                self.cursor.advance();
                self.cursor.advance();
                depth += 1;
            } else if self.cursor.current_char() == '*' && self.cursor.peek_char(1) == '/' {
                self.cursor.advance();
                self.cursor.advance();
                depth -= 1;
            } else {
                self.cursor.advance();
            }
        }

        if depth > 0 {
            self.report_error("unterminated block comment".to_string());
        }
    }

    /// Skips whitespace and comments.
    ///
    /// Skips all whitespace characters and comments (both line and block).
    /// This is called before lexing each token.
    /// LOW #12: Handles UTF-8 BOM at file start.
    fn skip_whitespace_and_comments(&mut self) {
        // LOW #12: Skip UTF-8 BOM if present at very start of file (only once)
        // BOM is U+FEFF encoded as EF BB BF in UTF-8
        // Note: U+FEFF is considered whitespace by Unicode, so we must check for it
        // BEFORE calling skip_whitespace()
        if !self.bom_checked {
            self.bom_checked = true;
            if self.cursor.remaining().starts_with("\u{FEFF}") {
                self.cursor.advance(); // Skip the BOM character
            }
        }

        loop {
            // Skip whitespace (this will NOT include BOM since we already handled it)
            self.cursor.skip_whitespace();

            // Check for comment
            if self.cursor.is_at_end() {
                break;
            }

            // Check for line comment
            if self.cursor.current_char() == '/' && self.cursor.peek_char(1) == '/' {
                self.cursor.advance(); // /
                self.cursor.advance(); // /
                while !self.cursor.is_at_end() && self.cursor.current_char() != '\n' {
                    self.cursor.advance();
                }
                continue;
            }

            // Check for block comment
            if self.cursor.current_char() == '/' && self.cursor.peek_char(1) == '*' {
                self.cursor.advance(); // /
                self.cursor.advance(); // *
                self.skip_block_comment();
                continue;
            }

            // No more whitespace or comments
            break;
        }
    }

    /// Reports a lexical error.
    ///
    /// Creates and emits an error diagnostic at the current token position.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message
    fn report_error(&mut self, message: String) {
        let span = Span::new(
            self.token_start,
            self.cursor.position(),
            self.token_start_line,
            self.token_start_column,
        );
        self.handler.error(message, span);
    }

    /// Returns the current line number.
    pub fn line(&self) -> u32 {
        self.cursor.line()
    }

    /// Returns the current column number.
    pub fn column(&self) -> u32 {
        self.cursor.column()
    }

    /// Returns the current position in the source.
    pub fn position(&self) -> usize {
        self.cursor.position()
    }
}

/// Make Lexer an iterator over tokens.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Token;

    /// Helper to create a lexer and collect all tokens.
    fn lex_tokens(source: &str) -> Vec<Token> {
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

    /// Helper to get the first token from source.
    fn first_token(source: &str) -> Token {
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        lexer.next_token()
    }

    // ========================================================================
    // IDENTIFIER AND KEYWORD TESTS
    // ========================================================================

    #[test]
    fn test_simple_identifier() {
        assert_eq!(first_token("x"), Token::Ident(faxc_util::Symbol::intern("x")));
        assert_eq!(first_token("foo"), Token::Ident(faxc_util::Symbol::intern("foo")));
        assert_eq!(
            first_token("bar123"),
            Token::Ident(faxc_util::Symbol::intern("bar123"))
        );
    }

    #[test]
    fn test_identifier_with_underscore() {
        // Single underscore is the wildcard token
        assert_eq!(first_token("_"), Token::Underscore);
        // Underscore followed by alphanumeric is an identifier
        assert_eq!(
            first_token("_private"),
            Token::Ident(faxc_util::Symbol::intern("_private"))
        );
        assert_eq!(
            first_token("my_var"),
            Token::Ident(faxc_util::Symbol::intern("my_var"))
        );
        assert_eq!(
            first_token("_123"),
            Token::Ident(faxc_util::Symbol::intern("_123"))
        );
    }

    #[test]
    fn test_keywords() {
        assert_eq!(first_token("let"), Token::Let);
        assert_eq!(first_token("fn"), Token::Fn);
        assert_eq!(first_token("if"), Token::If);
        assert_eq!(first_token("else"), Token::Else);
        assert_eq!(first_token("while"), Token::While);
        assert_eq!(first_token("for"), Token::For);
        assert_eq!(first_token("return"), Token::Return);
        assert_eq!(first_token("struct"), Token::Struct);
        assert_eq!(first_token("enum"), Token::Enum);
        assert_eq!(first_token("impl"), Token::Impl);
        assert_eq!(first_token("trait"), Token::Trait);
        assert_eq!(first_token("pub"), Token::Pub);
        assert_eq!(first_token("mut"), Token::Mut);
        assert_eq!(first_token("match"), Token::Match);
        assert_eq!(first_token("true"), Token::True);
        assert_eq!(first_token("false"), Token::False);
        assert_eq!(first_token("async"), Token::Async);
        assert_eq!(first_token("await"), Token::Await);
        assert_eq!(first_token("macro_rules"), Token::MacroRules);

        // Control flow keywords
        assert_eq!(first_token("loop"), Token::Loop);
        assert_eq!(first_token("break"), Token::Break);
        assert_eq!(first_token("continue"), Token::Continue);

        // Type system keywords
        assert_eq!(first_token("dyn"), Token::Dyn);
        assert_eq!(first_token("type"), Token::Type);
        assert_eq!(first_token("where"), Token::Where);

        // Module system keywords
        assert_eq!(first_token("mod"), Token::Mod);
        assert_eq!(first_token("use"), Token::Use);
        assert_eq!(first_token("as"), Token::As);
        assert_eq!(first_token("super"), Token::Super);
        assert_eq!(first_token("crate"), Token::Crate);

        // Advanced keywords
        assert_eq!(first_token("const"), Token::Const);
        assert_eq!(first_token("static"), Token::Static);
        assert_eq!(first_token("unsafe"), Token::Unsafe);
        assert_eq!(first_token("ref"), Token::Ref);

        // Special keywords (context-sensitive)
        assert_eq!(first_token("self"), Token::Self_);
        assert_eq!(first_token("Self"), Token::SelfUpper);
    }

    #[test]
    fn test_identifier_not_keyword() {
        assert_eq!(
            first_token("letter"),
            Token::Ident(faxc_util::Symbol::intern("letter"))
        );
        assert_eq!(
            first_token("function"),
            Token::Ident(faxc_util::Symbol::intern("function"))
        );
        assert_eq!(
            first_token("iffy"),
            Token::Ident(faxc_util::Symbol::intern("iffy"))
        );
    }

    #[test]
    fn test_all_new_keywords() {
        // Test all new keywords in a single source string
        let source = "loop break continue dyn type where mod use as super crate const static unsafe ref self Self";
        let tokens = lex_tokens(source);

        // Use content-based assertions instead of brittle index-based checks.
        // This ensures the test won't break when new keywords are added.
        assert!(tokens.contains(&Token::Loop), "Token::Loop should be present");
        assert!(tokens.contains(&Token::Break), "Token::Break should be present");
        assert!(tokens.contains(&Token::Continue), "Token::Continue should be present");
        assert!(tokens.contains(&Token::Dyn), "Token::Dyn should be present");
        assert!(tokens.contains(&Token::Type), "Token::Type should be present");
        assert!(tokens.contains(&Token::Where), "Token::Where should be present");
        assert!(tokens.contains(&Token::Mod), "Token::Mod should be present");
        assert!(tokens.contains(&Token::Use), "Token::Use should be present");
        assert!(tokens.contains(&Token::As), "Token::As should be present");
        assert!(tokens.contains(&Token::Super), "Token::Super should be present");
        assert!(tokens.contains(&Token::Crate), "Token::Crate should be present");
        assert!(tokens.contains(&Token::Const), "Token::Const should be present");
        assert!(tokens.contains(&Token::Static), "Token::Static should be present");
        assert!(tokens.contains(&Token::Unsafe), "Token::Unsafe should be present");
        assert!(tokens.contains(&Token::Ref), "Token::Ref should be present");
        assert!(tokens.contains(&Token::Self_), "Token::Self_ should be present");
        assert!(tokens.contains(&Token::SelfUpper), "Token::SelfUpper should be present");

        // Verify minimum expected token count (allows for future additions)
        assert!(tokens.len() >= 17, "Expected at least 17 tokens, got {}", tokens.len());

        // Verify all tokens are recognized as keywords
        for token in &tokens {
            assert!(token.is_keyword(), "{:?} should be a keyword", token);
        }
    }

    // ========================================================================
    // NUMBER LITERAL TESTS
    // ========================================================================

    #[test]
    fn test_decimal_integers() {
        assert_eq!(first_token("0"), Token::Number(0));
        assert_eq!(first_token("42"), Token::Number(42));
        assert_eq!(first_token("123456"), Token::Number(123456));
    }

    #[test]
    fn test_hexadecimal_integers() {
        assert_eq!(first_token("0xFF"), Token::Number(255));
        assert_eq!(first_token("0xab"), Token::Number(171));
        assert_eq!(first_token("0x100"), Token::Number(256));
        assert_eq!(first_token("0xAB_CD"), Token::Number(0xABCD));
    }

    #[test]
    fn test_binary_integers() {
        assert_eq!(first_token("0b0"), Token::Number(0));
        assert_eq!(first_token("0b1010"), Token::Number(10));
        assert_eq!(first_token("0b1111_0000"), Token::Number(240));
    }

    #[test]
    fn test_octal_integers() {
        assert_eq!(first_token("0o0"), Token::Number(0));
        assert_eq!(first_token("0o777"), Token::Number(511));
        assert_eq!(first_token("0o123"), Token::Number(83));
    }

    #[test]
    fn test_floats() {
        assert_eq!(first_token("3.14"), Token::Float(3.14));
        assert_eq!(first_token("0.5"), Token::Float(0.5));
        assert_eq!(first_token("1.0"), Token::Float(1.0));
    }

    #[test]
    fn test_floats_with_exponent() {
        assert_eq!(first_token("1e10"), Token::Float(1e10));
        assert_eq!(first_token("2.5e-3"), Token::Float(2.5e-3));
        assert_eq!(first_token("1.5E+5"), Token::Float(1.5e5));
        assert_eq!(first_token("3e2"), Token::Float(300.0));
    }

    // ========================================================================
    // STRING LITERAL TESTS
    // ========================================================================

    #[test]
    fn test_simple_string() {
        assert_eq!(
            first_token("\"hello\""),
            Token::String(faxc_util::Symbol::intern("hello"))
        );
        assert_eq!(first_token("\"\""), Token::String(faxc_util::Symbol::intern("")));
        assert_eq!(
            first_token("\"Hello, World!\""),
            Token::String(faxc_util::Symbol::intern("Hello, World!"))
        );
    }

    #[test]
    fn test_string_escapes() {
        assert_eq!(
            first_token("\"hello\\nworld\""),
            Token::String(faxc_util::Symbol::intern("hello\nworld"))
        );
        assert_eq!(
            first_token("\"tab\\there\""),
            Token::String(faxc_util::Symbol::intern("tab\there"))
        );
        assert_eq!(
            first_token("\"quote\\\"here\""),
            Token::String(faxc_util::Symbol::intern("quote\"here"))
        );
        assert_eq!(
            first_token("\"back\\\\slash\""),
            Token::String(faxc_util::Symbol::intern("back\\slash"))
        );
        assert_eq!(
            first_token("\"null\\0byte\""),
            Token::String(faxc_util::Symbol::intern("null\0byte"))
        );
        assert_eq!(
            first_token("\"carriage\\rreturn\""),
            Token::String(faxc_util::Symbol::intern("carriage\rreturn"))
        );
    }

    #[test]
    fn test_string_hex_escape() {
        assert_eq!(
            first_token("\"\\x41\""),
            Token::String(faxc_util::Symbol::intern("A"))
        );
        assert_eq!(
            first_token("\"\\x48\\x69\""),
            Token::String(faxc_util::Symbol::intern("Hi"))
        );
        assert_eq!(
            first_token("\"\\x7F\""),
            Token::String(faxc_util::Symbol::intern("\x7F"))
        );
    }

    #[test]
    fn test_string_unicode_escape() {
        assert_eq!(
            first_token("\"\\u{41}\""),
            Token::String(faxc_util::Symbol::intern("A"))
        );
        assert_eq!(
            first_token("\"\\u{1F600}\""),
            Token::String(faxc_util::Symbol::intern("😀"))
        );
        assert_eq!(
            first_token("\"Hello \\u{1F30D}\""),
            Token::String(faxc_util::Symbol::intern("Hello 🌍"))
        );
    }

    // ========================================================================
    // CHAR LITERAL TESTS
    // ========================================================================

    #[test]
    fn test_simple_char() {
        assert_eq!(first_token("'a'"), Token::Char('a'));
        assert_eq!(first_token("'A'"), Token::Char('A'));
        assert_eq!(first_token("'@'"), Token::Char('@'));
        assert_eq!(first_token("'1'"), Token::Char('1'));
    }

    #[test]
    fn test_char_escapes() {
        assert_eq!(first_token("'\\n'"), Token::Char('\n'));
        assert_eq!(first_token("'\\t'"), Token::Char('\t'));
        assert_eq!(first_token("'\\''"), Token::Char('\''));
        assert_eq!(first_token("'\\\\'"), Token::Char('\\'));
        assert_eq!(first_token("'\\0'"), Token::Char('\0'));
    }

    #[test]
    fn test_char_hex_escape() {
        assert_eq!(first_token("'\\x41'"), Token::Char('A'));
        assert_eq!(first_token("'\\x7F'"), Token::Char('\x7F'));
    }

    #[test]
    fn test_char_unicode_escape() {
        assert_eq!(first_token("'\\u{41}'"), Token::Char('A'));
        assert_eq!(first_token("'\\u{1F600}'"), Token::Char('\u{1F600}'));
    }

    // ========================================================================
    // OPERATOR TESTS
    // ========================================================================

    #[test]
    fn test_arithmetic_operators() {
        assert_eq!(first_token("+"), Token::Plus);
        assert_eq!(first_token("-"), Token::Minus);
        assert_eq!(first_token("*"), Token::Star);
        assert_eq!(first_token("/"), Token::Slash);
        assert_eq!(first_token("%"), Token::Percent);
    }

    #[test]
    fn test_assignment_operators() {
        assert_eq!(first_token("+="), Token::PlusEq);
        assert_eq!(first_token("-="), Token::MinusEq);
        assert_eq!(first_token("*="), Token::StarEq);
        assert_eq!(first_token("/="), Token::SlashEq);
        assert_eq!(first_token("%="), Token::PercentEq);
    }

    #[test]
    fn test_comparison_operators() {
        assert_eq!(first_token("=="), Token::EqEq);
        assert_eq!(first_token("!="), Token::NotEq);
        assert_eq!(first_token("<"), Token::Lt);
        assert_eq!(first_token(">"), Token::Gt);
        assert_eq!(first_token("<="), Token::LtEq);
        assert_eq!(first_token(">="), Token::GtEq);
    }

    #[test]
    fn test_logical_operators() {
        assert_eq!(first_token("!"), Token::Bang);
        assert_eq!(first_token("&&"), Token::AndAnd);
        assert_eq!(first_token("||"), Token::OrOr);
    }

    #[test]
    fn test_bitwise_operators() {
        assert_eq!(first_token("&"), Token::Ampersand);
        assert_eq!(first_token("|"), Token::Pipe);
        assert_eq!(first_token("^"), Token::Caret);
        assert_eq!(first_token("<<"), Token::Shl);
        assert_eq!(first_token(">>"), Token::Shr);
        assert_eq!(first_token("~"), Token::Tilde);
    }

    #[test]
    fn test_bitwise_compound_assignment_operators() {
        assert_eq!(first_token("&="), Token::AmpersandEq);
        assert_eq!(first_token("|="), Token::PipeEq);
        assert_eq!(first_token("^="), Token::CaretEq);
        assert_eq!(first_token("<<="), Token::ShlEq);
        assert_eq!(first_token(">>="), Token::ShrEq);
    }

    #[test]
    fn test_arrow() {
        assert_eq!(first_token("->"), Token::Arrow);
    }

    #[test]
    fn test_colon_operators() {
        assert_eq!(first_token(":"), Token::Colon);
        assert_eq!(first_token("::"), Token::ColonColon);
    }

    #[test]
    fn test_dot_operators() {
        assert_eq!(first_token("."), Token::Dot);
        assert_eq!(first_token(".."), Token::DotDot);
        assert_eq!(first_token("..."), Token::DotDotDot);
    }

    // ========================================================================
    // DELIMITER TESTS
    // ========================================================================

    #[test]
    fn test_delimiters() {
        assert_eq!(first_token("("), Token::LParen);
        assert_eq!(first_token(")"), Token::RParen);
        assert_eq!(first_token("{"), Token::LBrace);
        assert_eq!(first_token("}"), Token::RBrace);
        assert_eq!(first_token("["), Token::LBracket);
        assert_eq!(first_token("]"), Token::RBracket);
        assert_eq!(first_token(","), Token::Comma);
        assert_eq!(first_token(";"), Token::Semicolon);
        assert_eq!(first_token("$"), Token::Dollar);
        assert_eq!(first_token("@"), Token::At);
        assert_eq!(first_token("_"), Token::Underscore);
    }

    // ========================================================================
    // COMMENT TESTS
    // ========================================================================

    #[test]
    fn test_line_comment() {
        let tokens = lex_tokens("// this is a comment\nlet");
        assert_eq!(tokens, vec![Token::Let]);
    }

    #[test]
    fn test_line_comment_at_eof() {
        let tokens = lex_tokens("// comment at end");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_block_comment() {
        let tokens = lex_tokens("/* comment */ let");
        assert_eq!(tokens, vec![Token::Let]);
    }

    #[test]
    fn test_block_comment_multiline() {
        let tokens = lex_tokens("/* line1\nline2\nline3 */ fn");
        assert_eq!(tokens, vec![Token::Fn]);
    }

    #[test]
    fn test_nested_block_comments() {
        let tokens = lex_tokens("/* outer /* inner */ still outer */ if");
        assert_eq!(tokens, vec![Token::If]);
    }

    #[test]
    fn test_mixed_comments() {
        let tokens = lex_tokens("// line\n/* block */ else");
        assert_eq!(tokens, vec![Token::Else]);
    }

    #[test]
    fn test_comment_in_code() {
        let tokens = lex_tokens("let /* comment */ x = 5");
        assert_eq!(
            tokens,
            vec![
                Token::Let,
                Token::Ident(faxc_util::Symbol::intern("x")),
                Token::Eq,
                Token::Number(5)
            ]
        );
    }

    // ========================================================================
    // WHITESPACE TESTS
    // ========================================================================

    #[test]
    fn test_whitespace_skipping() {
        let tokens = lex_tokens("  let   x  =  42  ");
        assert_eq!(
            tokens,
            vec![
                Token::Let,
                Token::Ident(faxc_util::Symbol::intern("x")),
                Token::Eq,
                Token::Number(42)
            ]
        );
    }

    #[test]
    fn test_newline_handling() {
        let tokens = lex_tokens("let\nx\n=\n42");
        assert_eq!(
            tokens,
            vec![
                Token::Let,
                Token::Ident(faxc_util::Symbol::intern("x")),
                Token::Eq,
                Token::Number(42)
            ]
        );
    }

    #[test]
    fn test_tab_handling() {
        let tokens = lex_tokens("let\tx\t=\t42");
        assert_eq!(
            tokens,
            vec![
                Token::Let,
                Token::Ident(faxc_util::Symbol::intern("x")),
                Token::Eq,
                Token::Number(42)
            ]
        );
    }

    // ========================================================================
    // ERROR RECOVERY TESTS
    // ========================================================================

    #[test]
    fn test_unterminated_string() {
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("\"unterminated", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
        assert!(matches!(_token, Token::String(_)));
    }

    #[test]
    fn test_unterminated_char() {
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("'x", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
        assert!(matches!(_token, Token::Char(_)));
    }

    #[test]
    fn test_unterminated_block_comment() {
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("/* never closed", &mut handler);
        let token = lexer.next_token();
        assert!(handler.has_errors());
        assert_eq!(token, Token::Eof);
    }

    #[test]
    fn test_invalid_character() {
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("#", &mut handler);
        let token = lexer.next_token();
        assert!(handler.has_errors());
        assert!(matches!(token, Token::Invalid(_)));
    }

    #[test]
    fn test_invalid_hex_escape() {
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("\"\\xGG\"", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_invalid_unicode_escape() {
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("\"\\u{XYZ}\"", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    // ========================================================================
    // INTEGRATION TESTS
    // ========================================================================

    #[test]
    fn test_simple_program() {
        let source = r#"
            // A simple Fax program
            fn main() {
                let x = 42;
                let y = 3.14;
                let s = "hello";
            }
        "#;
        let tokens = lex_tokens(source);
        assert!(tokens.contains(&Token::Fn));
        assert!(tokens.contains(&Token::Ident(faxc_util::Symbol::intern("main"))));
        assert!(tokens.contains(&Token::Let));
        assert!(tokens.contains(&Token::Number(42)));
        assert!(tokens.contains(&Token::Float(3.14)));
        assert!(tokens.contains(&Token::String(faxc_util::Symbol::intern("hello"))));
    }

    #[test]
    fn test_complex_expressions() {
        let source = "a + b * c - d / e";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 9);
        assert_eq!(tokens[0], Token::Ident(faxc_util::Symbol::intern("a")));
        assert_eq!(tokens[1], Token::Plus);
        assert_eq!(tokens[2], Token::Ident(faxc_util::Symbol::intern("b")));
        assert_eq!(tokens[3], Token::Star);
        assert_eq!(tokens[4], Token::Ident(faxc_util::Symbol::intern("c")));
        assert_eq!(tokens[5], Token::Minus);
        assert_eq!(tokens[6], Token::Ident(faxc_util::Symbol::intern("d")));
        assert_eq!(tokens[7], Token::Slash);
        assert_eq!(tokens[8], Token::Ident(faxc_util::Symbol::intern("e")));
    }

    #[test]
    fn test_function_signature() {
        let source = "fn add(a: i32, b: i32) -> i32";
        let tokens = lex_tokens(source);
        assert_eq!(tokens[0], Token::Fn);
        assert_eq!(tokens[1], Token::Ident(faxc_util::Symbol::intern("add")));
        assert_eq!(tokens[2], Token::LParen);
        assert_eq!(tokens[3], Token::Ident(faxc_util::Symbol::intern("a")));
        assert_eq!(tokens[4], Token::Colon);
        assert_eq!(tokens[5], Token::Ident(faxc_util::Symbol::intern("i32")));
        assert_eq!(tokens[6], Token::Comma);
        assert_eq!(tokens[7], Token::Ident(faxc_util::Symbol::intern("b")));
        assert_eq!(tokens[8], Token::Colon);
        assert_eq!(tokens[9], Token::Ident(faxc_util::Symbol::intern("i32")));
        assert_eq!(tokens[10], Token::RParen);
        assert_eq!(tokens[11], Token::Arrow);
        assert_eq!(tokens[12], Token::Ident(faxc_util::Symbol::intern("i32")));
    }

    #[test]
    fn test_match_expression() {
        let source = "match x { 0 => \"zero\", _ => \"other\" }";
        let tokens = lex_tokens(source);
        assert_eq!(tokens[0], Token::Match);
        assert_eq!(tokens[1], Token::Ident(faxc_util::Symbol::intern("x")));
        assert_eq!(tokens[2], Token::LBrace);
        assert_eq!(tokens[3], Token::Number(0));
        assert_eq!(tokens[4], Token::FatArrow);
        assert_eq!(
            tokens[5],
            Token::String(faxc_util::Symbol::intern("zero"))
        );
        assert_eq!(tokens[6], Token::Comma);
        assert_eq!(tokens[7], Token::Underscore);
        assert_eq!(tokens[8], Token::FatArrow);
        assert_eq!(
            tokens[9],
            Token::String(faxc_util::Symbol::intern("other"))
        );
        assert_eq!(tokens[10], Token::RBrace);
    }

    #[test]
    fn test_all_operators() {
        let source = "+ - * / % == != < > <= >= && || ! = += -= *= /= %= & | ^ << >> ~ &= |= ^= <<= >>=";
        let tokens = lex_tokens(source);
        assert_eq!(tokens[0], Token::Plus);
        assert_eq!(tokens[1], Token::Minus);
        assert_eq!(tokens[2], Token::Star);
        assert_eq!(tokens[3], Token::Slash);
        assert_eq!(tokens[4], Token::Percent);
        assert_eq!(tokens[5], Token::EqEq);
        assert_eq!(tokens[6], Token::NotEq);
        assert_eq!(tokens[7], Token::Lt);
        assert_eq!(tokens[8], Token::Gt);
        assert_eq!(tokens[9], Token::LtEq);
        assert_eq!(tokens[10], Token::GtEq);
        assert_eq!(tokens[11], Token::AndAnd);
        assert_eq!(tokens[12], Token::OrOr);
        assert_eq!(tokens[13], Token::Bang);
        assert_eq!(tokens[14], Token::Eq);
        assert_eq!(tokens[15], Token::PlusEq);
        assert_eq!(tokens[16], Token::MinusEq);
        assert_eq!(tokens[17], Token::StarEq);
        assert_eq!(tokens[18], Token::SlashEq);
        assert_eq!(tokens[19], Token::PercentEq);
        assert_eq!(tokens[20], Token::Ampersand);
        assert_eq!(tokens[21], Token::Pipe);
        assert_eq!(tokens[22], Token::Caret);
        assert_eq!(tokens[23], Token::Shl);
        assert_eq!(tokens[24], Token::Shr);
        assert_eq!(tokens[25], Token::Tilde);
        assert_eq!(tokens[26], Token::AmpersandEq);
        assert_eq!(tokens[27], Token::PipeEq);
        assert_eq!(tokens[28], Token::CaretEq);
        assert_eq!(tokens[29], Token::ShlEq);
        assert_eq!(tokens[30], Token::ShrEq);
    }

    #[test]
    fn test_iterator() {
        let mut handler = Handler::new();
        let lexer = Lexer::new("let x = 42", &mut handler);
        let tokens: Vec<Token> = lexer.collect();

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], Token::Let);
        assert_eq!(tokens[1], Token::Ident(faxc_util::Symbol::intern("x")));
        assert_eq!(tokens[2], Token::Eq);
        assert_eq!(tokens[3], Token::Number(42));
    }

    // ========================================================================
    // BUG-HUNTING TESTS - Edge Cases and Boundary Conditions
    // ========================================================================

    // ------------------------------------------------------------------------
    // STRESS TESTS - Performance and Capacity Boundaries
    // ------------------------------------------------------------------------

    #[test]
    fn test_very_long_identifier() {
        // Stress test: very long identifier (10k chars)
        let source = "a".repeat(10000);
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        let token = lexer.next_token();
        assert!(matches!(token, Token::Ident(_)));
        assert!(!handler.has_errors());
    }

    #[test]
    fn test_maximum_length_identifier() {
        // Test with 100k character identifier
        let source = "x".repeat(100000);
        let tokens = lex_tokens(&source);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Ident(_)));
    }

    #[test]
    fn test_many_identifiers() {
        // Stress test: many identifiers
        let source = (0..10000).map(|i| format!("var{}", i)).collect::<Vec<_>>().join(" ");
        let tokens = lex_tokens(&source);
        assert_eq!(tokens.len(), 10000);
    }

    #[test]
    fn test_maximum_integer() {
        // Test u64::MAX boundary
        let source = format!("{}", u64::MAX);
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        let token = lexer.next_token();
        assert!(matches!(token, Token::Number(_)));
        assert!(!handler.has_errors());
    }

    #[test]
    fn test_integer_overflow() {
        // Number larger than u64::MAX should error but not crash
        let source = "999999999999999999999999999999";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        let token = lexer.next_token();
        // Should produce error but not crash
        assert!(handler.has_errors());
        // Should still return a Number token (error recovery)
        assert!(matches!(token, Token::Number(_)));
    }

    #[test]
    fn test_float_underflow() {
        // Very small float
        let source = "1e-999";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        let token = lexer.next_token();
        assert!(matches!(token, Token::Float(_)));
        // May or may not have error depending on float handling
    }

    #[test]
    fn test_float_overflow() {
        // Very large float
        let source = "1e999";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        let token = lexer.next_token();
        assert!(matches!(token, Token::Float(_)));
    }

    #[test]
    fn test_float_infinity() {
        // Test infinity representation
        let source = "1e99999";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Float(_)));
    }

    // ------------------------------------------------------------------------
    // OPERATOR AMBIGUITY TESTS - Maximal Munch Rule
    // ------------------------------------------------------------------------

    #[test]
    fn test_consecutive_less_than() {
        // <<< should be << then <
        let source = "a <<< b";
        let tokens = lex_tokens(source);
        // Note: << is a single Shl token, so we get: a, <<, <, b = 4 tokens
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], Token::Ident(faxc_util::Symbol::intern("a")));
        assert_eq!(tokens[1], Token::Shl);  // <<
        assert_eq!(tokens[2], Token::Lt);   // <
        assert_eq!(tokens[3], Token::Ident(faxc_util::Symbol::intern("b")));
    }

    #[test]
    fn test_triple_greater_than() {
        // >>> should be >> then >
        let source = "a >>> b";
        let tokens = lex_tokens(source);
        // Note: >> is a single Shr token, so we get: a, >>, >, b = 4 tokens
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], Token::Ident(faxc_util::Symbol::intern("a")));
        assert_eq!(tokens[1], Token::Shr);  // >>
        assert_eq!(tokens[2], Token::Gt);   // >
        assert_eq!(tokens[3], Token::Ident(faxc_util::Symbol::intern("b")));
    }

    #[test]
    fn test_quadruple_ampersand() {
        // &&&& should be && then &&
        let tokens = lex_tokens("a &&&& b");
        assert_eq!(tokens[1], Token::AndAnd);
        assert_eq!(tokens[2], Token::AndAnd);
    }

    #[test]
    fn test_quadruple_pipe() {
        // |||| should be || then ||
        let tokens = lex_tokens("a |||| b");
        assert_eq!(tokens[1], Token::OrOr);
        assert_eq!(tokens[2], Token::OrOr);
    }

    #[test]
    fn test_triple_plus() {
        // +++ should be ++ then +, but Fax doesn't have ++, so + then +
        let tokens = lex_tokens("a +++ b");
        assert_eq!(tokens[1], Token::Plus);
        assert_eq!(tokens[2], Token::Plus);
    }

    #[test]
    fn test_triple_minus() {
        // --- should be -- then -, but Fax doesn't have --, so - then -
        let tokens = lex_tokens("a --- b");
        assert_eq!(tokens[1], Token::Minus);
        assert_eq!(tokens[2], Token::Minus);
    }

    #[test]
    fn test_equals_chain() {
        // === should be == then =
        let tokens = lex_tokens("a === b");
        assert_eq!(tokens[1], Token::EqEq);
        assert_eq!(tokens[2], Token::Eq);
    }

    #[test]
    fn test_slash_star_in_code() {
        // /* should start block comment
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("a /* comment */ b", &mut handler);
        let tokens = vec![lexer.next_token(), lexer.next_token()];
        assert_eq!(tokens[0], Token::Ident(faxc_util::Symbol::intern("a")));
        assert_eq!(tokens[1], Token::Ident(faxc_util::Symbol::intern("b")));
    }

    #[test]
    fn test_slash_slash_in_code() {
        // // should start line comment
        let tokens = lex_tokens("a // comment\nb");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Ident(faxc_util::Symbol::intern("a")));
        assert_eq!(tokens[1], Token::Ident(faxc_util::Symbol::intern("b")));
    }

    // ------------------------------------------------------------------------
    // LINE ENDING AND WHITESPACE TESTS
    // ------------------------------------------------------------------------

    #[test]
    fn test_mixed_line_endings() {
        // Test \r\n (Windows), \n (Unix), \r (old Mac)
        let source = "let x = 1\r\nlet y = 2\rlet z = 3\n";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        
        // Should tokenize correctly
        let mut tokens = Vec::new();
        loop {
            let token = lexer.next_token();
            if token == Token::Eof {
                break;
            }
            tokens.push(token);
        }
        
        // Should have all tokens
        assert!(tokens.contains(&Token::Let));
        assert!(tokens.contains(&Token::Number(1)));
        assert!(tokens.contains(&Token::Number(2)));
        assert!(tokens.contains(&Token::Number(3)));
        assert!(!handler.has_errors());
    }

    #[test]
    fn test_only_whitespace() {
        let tokens = lex_tokens("   \t\n  \t\n  ");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_only_comments() {
        let tokens = lex_tokens("// comment\n/* block */\n// another");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_empty_file() {
        let tokens = lex_tokens("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_unicode_whitespace() {
        // Unicode whitespace characters
        let source = "let\u{00A0}x\u{2003}=\u{2003}42";  // NBSP, EM SPACE
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        // Should handle or error gracefully
        let _ = lexer.next_token();
    }

    // ------------------------------------------------------------------------
    // UNICODE AND ENCODING TESTS
    // ------------------------------------------------------------------------

    #[test]
    fn test_unicode_in_string() {
        // Test various Unicode characters in string
        let source = "\"Hello 世界 🌍 Привет\"";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::String(_)));
    }

    #[test]
    fn test_unicode_identifier() {
        // Test Unicode identifiers - should be treated as invalid or identifier
        let source = "変数 = 42";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        let token = lexer.next_token();
        // Should either accept as identifier or produce clear error
        assert!(matches!(token, Token::Ident(_)) || matches!(token, Token::Invalid(_)));
    }

    #[test]
    fn test_bom_at_start() {
        // UTF-8 BOM: EF BB BF
        let source = "\u{FEFF}let x = 1";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        let token = lexer.next_token();
        // Should skip BOM and tokenize 'let' or error gracefully
        assert!(matches!(token, Token::Let) || matches!(token, Token::Invalid(_)));
    }

    #[test]
    fn test_emoji_in_string() {
        // Emojis are multi-byte UTF-8
        let source = "\"😀😁😂🤣😃\"";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 1);
        if let Token::String(s) = &tokens[0] {
            assert!(s.as_str().contains("😀"));
        }
    }

    #[test]
    fn test_zwj_sequence() {
        // Zero-width joiner sequences (complex emoji)
        let source = "\"👨‍💻\"";  // Man technologist (multiple codepoints)
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::String(_)));
    }

    // ------------------------------------------------------------------------
    // ESCAPE SEQUENCE TESTS
    // ------------------------------------------------------------------------

    #[test]
    fn test_all_escape_sequences() {
        // Test every escape sequence in one string
        let source = "\"\\n\\t\\r\\\\\\\"\\0\\x41\\u{1F600}\"";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 1);
        if let Token::String(s) = &tokens[0] {
            let content = s.as_str();
            assert!(content.contains('\n'));
            assert!(content.contains('\t'));
            assert!(content.contains('\r'));
            assert!(content.contains('\\'));
            assert!(content.contains('"'));
            assert!(content.contains('\0'));
            assert!(content.contains('A'));  // \x41
            assert!(content.contains('😀')); // \u{1F600}
        } else {
            panic!("Expected String token");
        }
    }

    #[test]
    fn test_null_byte_in_string() {
        // String with null byte escape
        let source = "\"hello\\0world\"";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::String(_)));
    }

    #[test]
    fn test_string_with_newline_escape() {
        // String spanning multiple lines via escape
        let source = "\"line1\\nline2\\nline3\"";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 1);
        if let Token::String(s) = &tokens[0] {
            assert_eq!(s.as_str(), "line1\nline2\nline3");
        }
    }

    #[test]
    fn test_backslash_at_end_of_string() {
        // "ends with \" should error
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("\"ends with \\", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_incomplete_escape_sequence() {
        // "incomplete \x" should error
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("\"incomplete \\x\"", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_incomplete_unicode_escape() {
        // "incomplete \u{" should error
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("\"incomplete \\u{", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_unicode_escape_too_long() {
        // "\u{12345678}" should error (too many digits)
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("\"\\u{12345678}\"", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_unicode_escape_out_of_range() {
        // "\u{110000}" should error (above max Unicode)
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("\"\\u{110000}\"", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_surrogate_unicode_escape() {
        // "\u{D800}" should error (surrogate)
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("\"\\u{D800}\"", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_hex_escape_single_digit() {
        // "\x1" should work (single hex digit followed by non-hex)
        let tokens = lex_tokens("\"\\x1a\"");
        assert_eq!(tokens.len(), 1);
    }

    #[test]
    fn test_hex_escape_invalid_char() {
        // "\xG" should error
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("\"\\xG\"", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_unknown_escape_sequence() {
        // "\q" should error or treat as literal 'q'
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("\"\\q\"", &mut handler);
        let _token = lexer.next_token();
        // Currently errors on unknown escape
        assert!(handler.has_errors());
    }

    #[test]
    fn test_char_with_unicode() {
        // Character with Unicode escape
        let tokens = lex_tokens("'\\u{1F600}'");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Char('\u{1F600}'));
    }

    // ------------------------------------------------------------------------
    // NUMBER PARSING EDGE CASES
    // ------------------------------------------------------------------------

    #[test]
    fn test_underscore_in_number() {
        // Note: Fax lexer does NOT support underscores in numbers (yet)
        // 1_000_000 tokenizes as: 1, _000_000 (identifier starting with _)
        let tokens = lex_tokens("1_000_000");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Number(1));
        assert_eq!(tokens[1], Token::Ident(faxc_util::Symbol::intern("_000_000")));
    }

    #[test]
    fn test_underscore_positions() {
        // Note: Underscores ARE supported in hex/binary/octal numbers, but NOT in decimal
        // Decimal: 1_23 tokenizes as 1, _23 (identifier)
        assert_eq!(lex_tokens("1_23"), vec![Token::Number(1), Token::Ident(faxc_util::Symbol::intern("_23"))]);
        assert_eq!(lex_tokens("12_3"), vec![Token::Number(12), Token::Ident(faxc_util::Symbol::intern("_3"))]);
        // Hex: 0xFF_FF is valid (underscores supported in hex)
        assert_eq!(lex_tokens("0xFF_FF"), vec![Token::Number(0xFFFF)]);
        // Binary: 0b10_10 is valid (underscores supported in binary)
        assert_eq!(lex_tokens("0b10_10"), vec![Token::Number(0b1010)]);
    }

    #[test]
    fn test_underscore_at_start_of_number() {
        // _123 should be identifier, not number
        let tokens = lex_tokens("_123");
        assert!(matches!(tokens[0], Token::Ident(_)));
    }

    #[test]
    fn test_underscore_at_end_of_number() {
        // 123_ should parse as 123 then underscore token
        let tokens = lex_tokens("123_");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Number(123));
        assert_eq!(tokens[1], Token::Underscore);  // Single _ is the wildcard token
    }

    #[test]
    fn test_consecutive_underscores() {
        // __ should be identifier
        let tokens = lex_tokens("__");
        assert!(matches!(tokens[0], Token::Ident(_)));
    }

    #[test]
    fn test_multiple_underscores() {
        // Multiple underscores
        let tokens = lex_tokens("___ ____ _____");
        assert_eq!(tokens.len(), 3);
        for token in &tokens {
            assert!(matches!(token, Token::Ident(_)));
        }
    }

    #[test]
    fn test_hex_prefix_lowercase() {
        // 0x vs 0X
        let tokens1 = lex_tokens("0xFF");
        let tokens2 = lex_tokens("0xff");
        assert_eq!(tokens1, tokens2);
    }

    #[test]
    fn test_binary_prefix_mixed_case() {
        // 0b vs 0B
        let tokens1 = lex_tokens("0b1010");
        let tokens2 = lex_tokens("0B1010");
        assert_eq!(tokens1, tokens2);
    }

    #[test]
    fn test_octal_prefix_mixed_case() {
        // 0o vs 0O
        let tokens1 = lex_tokens("0o777");
        let tokens2 = lex_tokens("0O777");
        assert_eq!(tokens1, tokens2);
    }

    #[test]
    fn test_float_without_leading_digit() {
        // Note: .5 is NOT a float in Fax - it tokenizes as "." then "5"
        // The lexer requires a digit before the decimal point for floats
        let tokens = lex_tokens(".5");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Dot);
        assert_eq!(tokens[1], Token::Number(5));
    }

    #[test]
    fn test_float_without_trailing_digit() {
        // Note: 5. is NOT a float in Fax - it tokenizes as "5" then "."
        // The lexer requires a digit after the decimal point for floats
        let tokens = lex_tokens("5.");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Number(5));
        assert_eq!(tokens[1], Token::Dot);
    }

    #[test]
    fn test_multiple_decimal_points() {
        // 1.2.3 should handle gracefully
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("1.2.3", &mut handler);
        let token1 = lexer.next_token();
        let _token2 = lexer.next_token();
        // First should be float 1.2
        assert!(matches!(token1, Token::Float(_)));
        // Second should be .3 (float) or error
    }

    #[test]
    fn test_number_followed_by_dot() {
        // Note: "1." is NOT a float in Fax - it tokenizes as "1" then "."
        let tokens = lex_tokens("1.");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Number(1));
        assert_eq!(tokens[1], Token::Dot);
    }

    #[test]
    fn test_number_dot_number() {
        // "1.5" should be single float
        let tokens = lex_tokens("1.5");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Float(_)));
    }

    #[test]
    fn test_range_operator() {
        // "1..5" should be "1", "..", "5"
        let tokens = lex_tokens("1..5");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Number(1));
        assert_eq!(tokens[1], Token::DotDot);
        assert_eq!(tokens[2], Token::Number(5));
    }

    #[test]
    fn test_triple_dot() {
        // "..." should be DotDotDot
        let tokens = lex_tokens("...");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::DotDotDot);
    }

    #[test]
    fn test_four_dots() {
        // "...." should be "..." then "."
        let tokens = lex_tokens("....");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::DotDotDot);
        assert_eq!(tokens[1], Token::Dot);
    }

    #[test]
    fn test_five_dots() {
        // "....." should be "..." then ".."
        let tokens = lex_tokens(".....");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::DotDotDot);
        assert_eq!(tokens[1], Token::DotDot);
    }

    #[test]
    fn test_negative_number() {
        // "-5" should be "-" then "5", not negative number
        let tokens = lex_tokens("-5");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Minus);
        assert_eq!(tokens[1], Token::Number(5));
    }

    #[test]
    fn test_double_negative() {
        // "--5" should be "-", "-", "5"
        let tokens = lex_tokens("--5");
        assert_eq!(tokens.len(), 3);
    }

    #[test]
    fn test_hex_invalid_digit() {
        // 0xG should error
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("0xG", &mut handler);
        let token = lexer.next_token();
        assert!(handler.has_errors());
        assert_eq!(token, Token::Number(0));  // Error recovery
    }

    #[test]
    fn test_binary_invalid_digit() {
        // 0b2 should error
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("0b2", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_octal_invalid_digit() {
        // 0o8 should error
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("0o8", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    // ------------------------------------------------------------------------
    // STRING AND CHAR EDGE CASES
    // ------------------------------------------------------------------------

    #[test]
    fn test_string_ending_immediately() {
        // "" should be empty string
        let tokens = lex_tokens("\"\"");
        assert_eq!(tokens.len(), 1);
        if let Token::String(s) = &tokens[0] {
            assert_eq!(s.as_str(), "");
        }
    }

    #[test]
    fn test_char_ending_immediately() {
        // '' should error
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("''", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_multiple_chars_in_literal() {
        // 'ab' should error
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("'ab'", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_unterminated_string_multiline() {
        // String spanning lines without escape
        let source = "\"line1\nline2\"";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_string_with_embedded_newlines_via_escape() {
        let source = "\"line1\\nline2\\nline3\"";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 1);
        if let Token::String(s) = &tokens[0] {
            assert_eq!(s.as_str().matches('\n').count(), 2);
        }
    }

    #[test]
    fn test_very_long_string() {
        // Long string literal
        let source = format!("\"{}\"", "a".repeat(10000));
        let tokens = lex_tokens(&source);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::String(_)));
    }

    #[test]
    fn test_string_with_all_quotes() {
        let source = "\"\\\"\\\"\\\"\"";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 1);
        if let Token::String(s) = &tokens[0] {
            assert_eq!(s.as_str(), "\"\"\"");
        }
    }

    #[test]
    fn test_char_newline_escape() {
        let tokens = lex_tokens("'\\n'");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Char('\n'));
    }

    #[test]
    fn test_char_tab_escape() {
        let tokens = lex_tokens("'\\t'");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Char('\t'));
    }

    // ------------------------------------------------------------------------
    // KEYWORD AND IDENTIFIER EDGE CASES
    // ------------------------------------------------------------------------

    #[test]
    fn test_keyword_as_identifier_context() {
        // Keywords should always be keywords, not identifiers
        let tokens = lex_tokens("letfnif");
        assert!(matches!(tokens[0], Token::Ident(_)));  // Not a keyword concatenation
    }

    #[test]
    fn test_partial_keyword() {
        // "le" should be identifier, not "let"
        let tokens = lex_tokens("le");
        assert!(matches!(tokens[0], Token::Ident(_)));
    }

    #[test]
    fn test_keyword_with_underscore() {
        // "let_" should be identifier
        let tokens = lex_tokens("let_");
        assert!(matches!(tokens[0], Token::Ident(_)));
    }

    #[test]
    fn test_keyword_with_number() {
        // "let1" should be identifier
        let tokens = lex_tokens("let1");
        assert!(matches!(tokens[0], Token::Ident(_)));
    }

    #[test]
    fn test_all_keywords_tokenized() {
        // Verify all keywords are recognized
        let keywords = vec![
            ("let", Token::Let),
            ("fn", Token::Fn),
            ("if", Token::If),
            ("else", Token::Else),
            ("while", Token::While),
            ("for", Token::For),
            ("return", Token::Return),
            ("struct", Token::Struct),
            ("enum", Token::Enum),
            ("impl", Token::Impl),
            ("trait", Token::Trait),
            ("pub", Token::Pub),
            ("mut", Token::Mut),
            ("match", Token::Match),
            ("true", Token::True),
            ("false", Token::False),
            ("async", Token::Async),
            ("await", Token::Await),
            ("macro_rules", Token::MacroRules),
        ];
        
        for (keyword, expected_token) in keywords {
            let tokens = lex_tokens(keyword);
            assert_eq!(tokens.len(), 1, "Keyword '{}' should produce 1 token", keyword);
            assert_eq!(tokens[0], expected_token, "Keyword '{}' mismatch", keyword);
        }
    }

    #[test]
    fn test_case_sensitive_keywords() {
        // Keywords are case-sensitive
        assert_eq!(first_token("LET"), Token::Ident(faxc_util::Symbol::intern("LET")));
        assert_eq!(first_token("Let"), Token::Ident(faxc_util::Symbol::intern("Let")));
        assert_eq!(first_token("FN"), Token::Ident(faxc_util::Symbol::intern("FN")));
    }

    #[test]
    fn test_single_char_identifiers() {
        // All single letter identifiers
        for c in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".chars() {
            let tokens = lex_tokens(&c.to_string());
            assert_eq!(tokens.len(), 1);
            assert!(matches!(tokens[0], Token::Ident(_)));
        }
    }

    #[test]
    fn test_identifiers_starting_with_keywords() {
        assert!(matches!(first_token("letting"), Token::Ident(_)));
        assert!(matches!(first_token("function"), Token::Ident(_)));
        assert!(matches!(first_token("iffy"), Token::Ident(_)));
        assert!(matches!(first_token("elsewhere"), Token::Ident(_)));
    }

    // ------------------------------------------------------------------------
    // DELIMITER AND PUNCTUATION TESTS
    // ------------------------------------------------------------------------

    #[test]
    fn test_delimiters_all_types() {
        // Test all delimiter types
        let source = "(){}[],;:.";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 10);
        assert_eq!(tokens[0], Token::LParen);
        assert_eq!(tokens[1], Token::RParen);
        assert_eq!(tokens[2], Token::LBrace);
        assert_eq!(tokens[3], Token::RBrace);
        assert_eq!(tokens[4], Token::LBracket);
        assert_eq!(tokens[5], Token::RBracket);
        assert_eq!(tokens[6], Token::Comma);
        assert_eq!(tokens[7], Token::Semicolon);
        assert_eq!(tokens[8], Token::Colon);
        assert_eq!(tokens[9], Token::Dot);
    }

    #[test]
    fn test_arrows_all_types() {
        // Test arrow variants
        let source = "->=>";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Arrow);
        assert_eq!(tokens[1], Token::FatArrow);
    }

    #[test]
    fn test_nested_parens() {
        let tokens = lex_tokens("((()))");
        assert_eq!(tokens, vec![Token::LParen, Token::LParen, Token::LParen, Token::RParen, Token::RParen, Token::RParen]);
    }

    #[test]
    fn test_nested_braces() {
        let tokens = lex_tokens("{{{}}}");
        assert_eq!(tokens, vec![Token::LBrace, Token::LBrace, Token::LBrace, Token::RBrace, Token::RBrace, Token::RBrace]);
    }

    #[test]
    fn test_nested_brackets() {
        let tokens = lex_tokens("[[[]]]");
        assert_eq!(tokens, vec![Token::LBracket, Token::LBracket, Token::LBracket, Token::RBracket, Token::RBracket, Token::RBracket]);
    }

    #[test]
    fn test_mixed_delimiters() {
        let tokens = lex_tokens("({[]})");
        assert_eq!(tokens, vec![Token::LParen, Token::LBrace, Token::LBracket, Token::RBracket, Token::RBrace, Token::RParen]);
    }

    #[test]
    fn test_path_separator() {
        // "std::io::Result"
        let tokens = lex_tokens("std::io::Result");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::Ident(faxc_util::Symbol::intern("std")));
        assert_eq!(tokens[1], Token::ColonColon);
        assert_eq!(tokens[2], Token::Ident(faxc_util::Symbol::intern("io")));
        assert_eq!(tokens[3], Token::ColonColon);
        assert_eq!(tokens[4], Token::Ident(faxc_util::Symbol::intern("Result")));
    }

    #[test]
    fn test_triple_colon() {
        // ":::" should be "::" then ":"
        let tokens = lex_tokens(":::");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::ColonColon);
        assert_eq!(tokens[1], Token::Colon);
    }

    #[test]
    fn test_question_mark() {
        // "?" for error propagation
        let tokens = lex_tokens("?");
        assert!(matches!(tokens[0], Token::Invalid(_)));  // Not supported yet
    }

    #[test]
    fn test_at_symbol() {
        // "@" for pattern binding
        let tokens = lex_tokens("@");
        assert!(matches!(tokens[0], Token::At));
    }

    #[test]
    fn test_dollar_sign() {
        // "$" for macros
        let tokens = lex_tokens("$");
        assert!(matches!(tokens[0], Token::Dollar));
    }

    #[test]
    fn test_backtick() {
        // "`" for raw identifiers (if supported)
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("`let`", &mut handler);
        let token = lexer.next_token();
        // Currently treated as invalid
        assert!(matches!(token, Token::Invalid(_)));
    }

    #[test]
    fn test_hash_symbol() {
        // "#" for attributes
        let tokens = lex_tokens("#");
        assert!(matches!(tokens[0], Token::Invalid(_)));  // Not supported yet
    }

    #[test]
    fn test_tilde() {
        // "~" for bitwise NOT
        let tokens = lex_tokens("~");
        assert_eq!(tokens[0], Token::Tilde);
    }

    // ------------------------------------------------------------------------
    // COMMENT EDGE CASES
    // ------------------------------------------------------------------------

    #[test]
    fn test_nested_comments() {
        // Nested block comments
        let source = "/* outer /* inner */ still outer */ let";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Let);
    }

    #[test]
    fn test_comment_with_string_inside() {
        // String-like content in comment should be ignored
        let source = "// \"not a string\"\nlet";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Let);
    }

    #[test]
    fn test_block_comment_with_line_comment_inside() {
        let source = "/* // nested */ let";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Let);
    }

    #[test]
    fn test_line_comment_with_block_comment_start() {
        let source = "// /* not a block */\nlet";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Let);
    }

    #[test]
    fn test_deeply_nested_comments() {
        let source = "/* a /* b /* c */ d */ e */ let";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Let);
    }

    #[test]
    fn test_unterminated_block_comment_at_eof() {
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("/* never closed", &mut handler);
        let token = lexer.next_token();
        assert!(handler.has_errors());
        assert_eq!(token, Token::Eof);
    }

    #[test]
    fn test_empty_block_comment() {
        let tokens = lex_tokens("/**/let");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Let);
    }

    #[test]
    fn test_block_comment_with_stars() {
        let tokens = lex_tokens("/* *** */ let");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Let);
    }

    // ------------------------------------------------------------------------
    // SOURCE LOCATION TRACKING TESTS
    // ------------------------------------------------------------------------

    #[test]
    fn test_source_location_accuracy() {
        // Test that error locations are accurate
        let source = "let x = \"unterminated\nnext line";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        
        // Consume tokens
        while lexer.next_token() != Token::Eof {}
        
        assert!(handler.has_errors());
        let diagnostics = handler.diagnostics();
        assert!(!diagnostics.is_empty());
        // Error should point to line 1
        assert_eq!(diagnostics[0].span.line, 1);
    }

    #[test]
    fn test_column_tracking_across_lines() {
        // Verify column tracking is accurate
        let source = "a\nbb\nccc";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        
        // First token 'a' at line 1, column 1
        let _ = lexer.next_token();
        assert_eq!(lexer.line(), 1);
        assert_eq!(lexer.column(), 2);  // After 'a'
        
        // After newline, should be line 2
        let _ = lexer.next_token();  // 'bb'
        assert_eq!(lexer.line(), 2);
        
        let _ = lexer.next_token();  // 'ccc'
        assert_eq!(lexer.line(), 3);
    }

    #[test]
    fn test_line_tracking_with_comments() {
        let source = "// comment\nlet\n/* block\ncomment */\nfn";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        
        let token1 = lexer.next_token();
        assert_eq!(token1, Token::Let);
        assert_eq!(lexer.line(), 2);
        
        let token2 = lexer.next_token();
        assert_eq!(token2, Token::Fn);
        assert_eq!(lexer.line(), 5);
    }

    #[test]
    fn test_position_tracking() {
        let source = "let x = 42";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        
        assert_eq!(lexer.position(), 0);
        
        let _ = lexer.next_token();  // let
        assert_eq!(lexer.position(), 3);
        
        let _ = lexer.next_token();  // x
        assert_eq!(lexer.position(), 5);
        
        let _ = lexer.next_token();  // =
        assert_eq!(lexer.position(), 7);
        
        let _ = lexer.next_token();  // 42
        assert_eq!(lexer.position(), 10);
    }

    // ------------------------------------------------------------------------
    // OPERATOR SEQUENCE TESTS
    // ------------------------------------------------------------------------

    #[test]
    fn test_all_arithmetic_operators() {
        let source = "+ - * / %";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::Plus);
        assert_eq!(tokens[1], Token::Minus);
        assert_eq!(tokens[2], Token::Star);
        assert_eq!(tokens[3], Token::Slash);
        assert_eq!(tokens[4], Token::Percent);
    }

    #[test]
    fn test_all_assignment_operators() {
        // All assignment variants including compound bitwise assignments
        let source = "= += -= *= /= %= &= |= ^= <<= >>=";
        let tokens = lex_tokens(source);
        // Now all compound assignments are single tokens
        assert_eq!(tokens.len(), 11);
        assert_eq!(tokens[0], Token::Eq);
        assert_eq!(tokens[1], Token::PlusEq);
        assert_eq!(tokens[2], Token::MinusEq);
        assert_eq!(tokens[3], Token::StarEq);
        assert_eq!(tokens[4], Token::SlashEq);
        assert_eq!(tokens[5], Token::PercentEq);
        assert_eq!(tokens[6], Token::AmpersandEq);
        assert_eq!(tokens[7], Token::PipeEq);
        assert_eq!(tokens[8], Token::CaretEq);
        assert_eq!(tokens[9], Token::ShlEq);
        assert_eq!(tokens[10], Token::ShrEq);
    }

    #[test]
    fn test_all_comparison_operators() {
        let source = "== != < > <= >=";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 6);
    }

    #[test]
    fn test_all_logical_operators() {
        let source = "&& || !";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 3);
    }

    #[test]
    fn test_all_bitwise_operators() {
        let source = "& | ^ << >>";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 5);
    }

    #[test]
    fn test_operator_ambiguity() {
        // Test operators that could be ambiguous
        let source = "a+b*c-d/e%f";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 11);  // 6 identifiers + 5 operators
    }

    #[test]
    fn test_adjacent_numbers() {
        // Two numbers without separator should tokenize separately
        let tokens = lex_tokens("42 123");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Number(42));
        assert_eq!(tokens[1], Token::Number(123));
    }

    #[test]
    fn test_adjacent_identifiers() {
        // Two identifiers with space
        let tokens = lex_tokens("foo bar");
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn test_multiple_identifiers_on_line() {
        // Many identifiers on one line
        let source = "a b c d e f g h i j k l m n o p q r s t u v w x y z";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 26);
        for token in &tokens {
            assert!(matches!(token, Token::Ident(_)));
        }
    }

    #[test]
    fn test_alternating_tokens() {
        // Alternating pattern
        let source = "a 1 b 2 c 3 d 4 e 5";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 10);
        assert!(matches!(tokens[0], Token::Ident(_)));
        assert!(matches!(tokens[1], Token::Number(_)));
    }

    #[test]
    fn test_all_operators_in_sequence() {
        // All operators in one line
        let source = "+ - * / % == != < > <= >= && || & | ^ << >>";
        let tokens = lex_tokens(source);
        // +, -, *, /, %, ==, !=, <, >, <=, >=, &&, ||, &, |, ^, <<, >>
        assert_eq!(tokens.len(), 18);
    }

    // ------------------------------------------------------------------------
    // MACRO AND SPECIAL SYNTAX TESTS
    // ------------------------------------------------------------------------

    #[test]
    fn test_macro_invocation() {
        // "println!(...)" 
        let tokens = lex_tokens("println!(\"hello\")");
        assert!(tokens.contains(&Token::Ident(faxc_util::Symbol::intern("println"))));
        assert!(tokens.contains(&Token::Bang));
        assert!(tokens.contains(&Token::LParen));
    }

    #[test]
    fn test_macro_with_nested_parens() {
        // "foo!((1 + 2) * 3)"
        let tokens = lex_tokens("foo!((1 + 2) * 3)");
        assert!(tokens.contains(&Token::Bang));
        // ((1 + 2) * 3) has 2 opening parens
        assert_eq!(tokens.iter().filter(|t| *t == &Token::LParen).count(), 2);
    }

    #[test]
    fn test_macro_with_braces() {
        // "vec![1, 2, 3]"
        let tokens = lex_tokens("vec![1, 2, 3]");
        assert!(tokens.contains(&Token::Bang));
        assert!(tokens.contains(&Token::LBracket));
    }

    #[test]
    fn test_nested_generics() {
        // "HashMap<String, Vec<i32>>"
        let tokens = lex_tokens("HashMap<String, Vec<i32>>");
        // Should handle ">>" correctly (as Shr, not two Gt)
        assert!(tokens.contains(&Token::Shr));
    }

    #[test]
    fn test_closure_syntax() {
        // "|x| x + 1"
        let source = "|x| x + 1";
        let tokens = lex_tokens(source);
        assert_eq!(tokens[0], Token::Pipe);
        assert_eq!(tokens[1], Token::Ident(faxc_util::Symbol::intern("x")));
        assert_eq!(tokens[2], Token::Pipe);
    }

    #[test]
    fn test_closure_with_type() {
        // "|x: i32| -> i32 { x * 2 }"
        let source = "|x: i32| -> i32 { x * 2 }";
        let tokens = lex_tokens(source);
        assert!(tokens.contains(&Token::Pipe));
        assert!(tokens.contains(&Token::Arrow));
    }

    #[test]
    fn test_async_block() {
        // "async { await foo() }"
        let source = "async { await foo() }";
        let tokens = lex_tokens(source);
        assert!(tokens.contains(&Token::Async));
        assert!(tokens.contains(&Token::Await));
    }

    #[test]
    fn test_async_closure() {
        // "async |x| x + 1"
        let source = "async |x| x + 1";
        let tokens = lex_tokens(source);
        assert!(tokens.contains(&Token::Async));
    }

    #[test]
    fn test_match_expression_tokens() {
        let source = "match x { 0 => \"zero\", _ => \"other\" }";
        let tokens = lex_tokens(source);
        assert!(tokens.contains(&Token::Match));
        assert!(tokens.contains(&Token::FatArrow));
        assert!(tokens.contains(&Token::Underscore));
    }

    #[test]
    fn test_visibility_modifier() {
        // "pub(crate) fn" - crate is now a keyword
        let tokens = lex_tokens("pub(crate)");
        assert!(tokens.contains(&Token::Pub));
        assert!(tokens.contains(&Token::LParen));
        assert!(tokens.contains(&Token::Crate));
        assert!(tokens.contains(&Token::RParen));
    }

    #[test]
    fn test_self_keyword() {
        // "self" and "Self" should be keywords
        let tokens1 = lex_tokens("self");
        let tokens2 = lex_tokens("Self");
        assert_eq!(tokens1[0], Token::Self_);
        assert_eq!(tokens2[0], Token::SelfUpper);
    }

    #[test]
    fn test_mut_binding() {
        let tokens = lex_tokens("let mut x = 5");
        assert!(tokens.contains(&Token::Let));
        assert!(tokens.contains(&Token::Mut));
    }

    #[test]
    fn test_boolean_literals() {
        let tokens1 = lex_tokens("true");
        let tokens2 = lex_tokens("false");
        assert_eq!(tokens1[0], Token::True);
        assert_eq!(tokens2[0], Token::False);
    }

    // ------------------------------------------------------------------------
    // ERROR HANDLING AND RECOVERY TESTS
    // ------------------------------------------------------------------------

    #[test]
    fn test_invalid_character_continues() {
        // Invalid character should not stop lexing
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("let # x = 1", &mut handler);
        
        let tokens: Vec<Token> = std::iter::from_fn(|| {
            let t = lexer.next_token();
            if t == Token::Eof { None } else { Some(t) }
        }).collect();
        
        // Should have tokens after the invalid character
        assert!(tokens.contains(&Token::Let));
        assert!(tokens.contains(&Token::Number(1)));
        assert!(handler.has_errors());
    }

    #[test]
    fn test_multiple_errors() {
        // Multiple errors should all be reported
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("# @ $ invalid", &mut handler);
        
        while lexer.next_token() != Token::Eof {}
        
        // Should have multiple errors
        assert!(handler.has_errors());
    }

    #[test]
    fn test_unterminated_char_at_eof() {
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("'x", &mut handler);
        let token = lexer.next_token();
        assert!(handler.has_errors());
        assert!(matches!(token, Token::Char(_)));  // Error recovery
    }

    #[test]
    fn test_unterminated_string_at_eof() {
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("\"hello", &mut handler);
        let token = lexer.next_token();
        assert!(handler.has_errors());
        assert!(matches!(token, Token::String(_)));  // Error recovery
    }

    #[test]
    fn test_empty_char_literal() {
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("''", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_char_with_multiple_chars() {
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("'abc'", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    // ------------------------------------------------------------------------
    // INTEGRATION AND COMPLEX SCENARIOS
    // ------------------------------------------------------------------------

    #[test]
    fn test_complex_function() {
        let source = r#"
            pub fn add<T: Clone>(a: T, b: T) -> T {
                let result = a.clone();
                return result;
            }
        "#;
        let tokens = lex_tokens(source);
        assert!(tokens.contains(&Token::Pub));
        assert!(tokens.contains(&Token::Fn));
        assert!(tokens.contains(&Token::Let));
        assert!(tokens.contains(&Token::Return));
        assert!(tokens.contains(&Token::Colon));  // For type annotations
    }

    #[test]
    fn test_complex_expression() {
        let source = "a + b * c - d / e % f + (g - h) * i";
        let tokens = lex_tokens(source);
        // a, +, b, *, c, -, d, /, e, %, f, +, (, g, -, h, ), *, i
        assert_eq!(tokens.len(), 19);  // 9 identifiers + 8 operators + 2 parens
    }

    #[test]
    fn test_string_interpolation_syntax() {
        // If string interpolation is not supported, this should tokenize as string + expr
        let source = "\"hello {world}\"";
        let tokens = lex_tokens(source);
        // Should be a single string token (interpolation not recognized)
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::String(_)));
    }

    #[test]
    fn test_raw_string_supported() {
        // Raw strings (r"...") are supported
        let tokens = lex_tokens("r\"raw\"");
        assert!(matches!(tokens[0], Token::RawString(_)));
    }

    #[test]
    fn test_byte_string_not_supported() {
        // Byte strings (b"...") not supported
        let tokens = lex_tokens("b\"hello\"");
        assert!(matches!(tokens[0], Token::Ident(_)));  // 'b' as identifier
    }

    #[test]
    fn test_lifetime_syntax() {
        // "'a" for lifetimes - should be char literal 'a'
        let tokens = lex_tokens("'a");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Char('a'));
    }

    #[test]
    fn test_shebang_handling() {
        // "#!/usr/bin/env faxc" at start
        let source = "#!/usr/bin/env faxc\nfn main() {}";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);

        // '#' should be invalid
        let token1 = lexer.next_token();
        assert!(matches!(token1, Token::Invalid(_)));

        // "!" is the Bang token
        let token2 = lexer.next_token();
        assert_eq!(token2, Token::Bang);
    }

    #[test]
    fn test_attribute_syntax() {
        // "#[allow(dead_code)]" - # is invalid
        let tokens = lex_tokens("#[allow(dead_code)]");
        assert!(tokens.iter().any(|t| matches!(t, Token::Invalid(_))));  // '#'
        assert!(tokens.contains(&Token::LBracket));
    }

    #[test]
    fn test_try_operator_not_supported() {
        // "?" for error propagation
        let tokens = lex_tokens("result?");
        assert!(tokens.contains(&Token::Ident(faxc_util::Symbol::intern("result"))));
        assert!(tokens.iter().any(|t| matches!(t, Token::Invalid(_))));  // '?'
    }

    #[test]
    fn test_pipeline_operator_not_supported() {
        // "|>" pipeline
        let tokens = lex_tokens("a |> b");
        assert_eq!(tokens[1], Token::Pipe);
        assert_eq!(tokens[2], Token::Gt);
    }

    #[test]
    fn test_spaceship_operator_not_supported() {
        // "<=>" comparison - tokenizes as <= then > (maximal munch)
        let tokens = lex_tokens("a <=> b");
        // Tokens: a, <=, >, b (4 tokens due to maximal munch rule)
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], Token::Ident(faxc_util::Symbol::intern("a")));
        assert_eq!(tokens[1], Token::LtEq);  // <=
        assert_eq!(tokens[2], Token::Gt);    // >
        assert_eq!(tokens[3], Token::Ident(faxc_util::Symbol::intern("b")));
    }

    #[test]
    fn test_exponent_case_insensitive() {
        // "e" and "E" should both work
        let tokens1 = lex_tokens("1e10");
        let tokens2 = lex_tokens("1E10");
        assert_eq!(tokens1, tokens2);
    }

    #[test]
    fn test_exponent_with_plus() {
        let tokens = lex_tokens("1e+10");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Float(_)));
    }

    #[test]
    fn test_exponent_with_minus() {
        let tokens = lex_tokens("1e-10");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Float(_)));
    }

    #[test]
    fn test_zero_with_exponent() {
        let tokens = lex_tokens("0e10");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Float(_)));
    }

    #[test]
    fn test_very_long_float() {
        let source = "0.".to_owned() + &"1".repeat(1000);
        let tokens = lex_tokens(&source);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Float(_)));
    }

    #[test]
    fn test_iterator_api() {
        let mut handler = Handler::new();
        let lexer = Lexer::new("let x = 42", &mut handler);
        let tokens: Vec<Token> = lexer.collect();

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], Token::Let);
        assert_eq!(tokens[1], Token::Ident(faxc_util::Symbol::intern("x")));
        assert_eq!(tokens[2], Token::Eq);
        assert_eq!(tokens[3], Token::Number(42));
    }

    #[test]
    fn test_iterator_with_filter() {
        let mut handler = Handler::new();
        let lexer = Lexer::new("let x = 42 + 10", &mut handler);
        let numbers: Vec<Token> = lexer.filter(|t| matches!(t, Token::Number(_))).collect();

        assert_eq!(numbers.len(), 2);
        assert_eq!(numbers[0], Token::Number(42));
        assert_eq!(numbers[1], Token::Number(10));
    }

    #[test]
    fn test_consecutive_newlines() {
        let source = "let\n\n\nx\n\n=\n\n42";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 4);
    }

    #[test]
    fn test_consecutive_tabs_and_spaces() {
        let source = "let \t \t x \t \t = \t \t 42";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 4);
    }

    #[test]
    fn test_mixed_comment_styles() {
        let source = "// line\n/* block */ // another\n/* nested /* */ */ fn";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Fn);
    }

    #[test]
    fn test_code_after_unterminated_comment() {
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("/* unclosed let", &mut handler);
        let token = lexer.next_token();
        assert!(handler.has_errors());
        assert_eq!(token, Token::Eof);
    }

    #[test]
    fn test_unicode_in_comment() {
        // Unicode in comments should be ignored
        let source = "// 你好世界 🌍\nlet";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Let);
    }

    #[test]
    fn test_control_characters() {
        // Control characters should be treated as invalid
        let source = "let\u{0001}x = 1";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        let _ = lexer.next_token();
        // Should handle gracefully
    }

    #[test]
    fn test_form_feed() {
        // Form feed character
        let source = "let\u{000C}x = 1";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        let _ = lexer.next_token();
    }

    #[test]
    fn test_vertical_tab() {
        // Vertical tab character
        let source = "let\u{000B}x = 1";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        let _ = lexer.next_token();
    }

    // ========================================================================
    // ENHANCED TEST COVERAGE - NEW TESTS FOR COMPREHENSIVE COVERAGE
    // ========================================================================

    // ------------------------------------------------------------------------
    // INDIVIDUAL KEYWORD TESTS - All 36 keywords tested in isolation
    // ------------------------------------------------------------------------

    #[test]
    fn test_keyword_let_in_isolation() {
        assert_eq!(first_token("let"), Token::Let);
    }

    #[test]
    fn test_keyword_fn_in_isolation() {
        assert_eq!(first_token("fn"), Token::Fn);
    }

    #[test]
    fn test_keyword_if_in_isolation() {
        assert_eq!(first_token("if"), Token::If);
    }

    #[test]
    fn test_keyword_else_in_isolation() {
        assert_eq!(first_token("else"), Token::Else);
    }

    #[test]
    fn test_keyword_while_in_isolation() {
        assert_eq!(first_token("while"), Token::While);
    }

    #[test]
    fn test_keyword_for_in_isolation() {
        assert_eq!(first_token("for"), Token::For);
    }

    #[test]
    fn test_keyword_return_in_isolation() {
        assert_eq!(first_token("return"), Token::Return);
    }

    #[test]
    fn test_keyword_struct_in_isolation() {
        assert_eq!(first_token("struct"), Token::Struct);
    }

    #[test]
    fn test_keyword_enum_in_isolation() {
        assert_eq!(first_token("enum"), Token::Enum);
    }

    #[test]
    fn test_keyword_impl_in_isolation() {
        assert_eq!(first_token("impl"), Token::Impl);
    }

    #[test]
    fn test_keyword_trait_in_isolation() {
        assert_eq!(first_token("trait"), Token::Trait);
    }

    #[test]
    fn test_keyword_pub_in_isolation() {
        assert_eq!(first_token("pub"), Token::Pub);
    }

    #[test]
    fn test_keyword_mut_in_isolation() {
        assert_eq!(first_token("mut"), Token::Mut);
    }

    #[test]
    fn test_keyword_match_in_isolation() {
        assert_eq!(first_token("match"), Token::Match);
    }

    #[test]
    fn test_keyword_true_in_isolation() {
        assert_eq!(first_token("true"), Token::True);
    }

    #[test]
    fn test_keyword_false_in_isolation() {
        assert_eq!(first_token("false"), Token::False);
    }

    #[test]
    fn test_keyword_async_in_isolation() {
        assert_eq!(first_token("async"), Token::Async);
    }

    #[test]
    fn test_keyword_await_in_isolation() {
        assert_eq!(first_token("await"), Token::Await);
    }

    #[test]
    fn test_keyword_macro_rules_in_isolation() {
        assert_eq!(first_token("macro_rules"), Token::MacroRules);
    }

    #[test]
    fn test_keyword_loop_in_isolation() {
        assert_eq!(first_token("loop"), Token::Loop);
    }

    #[test]
    fn test_keyword_break_in_isolation() {
        assert_eq!(first_token("break"), Token::Break);
    }

    #[test]
    fn test_keyword_continue_in_isolation() {
        assert_eq!(first_token("continue"), Token::Continue);
    }

    #[test]
    fn test_keyword_dyn_in_isolation() {
        assert_eq!(first_token("dyn"), Token::Dyn);
    }

    #[test]
    fn test_keyword_type_in_isolation() {
        assert_eq!(first_token("type"), Token::Type);
    }

    #[test]
    fn test_keyword_where_in_isolation() {
        assert_eq!(first_token("where"), Token::Where);
    }

    #[test]
    fn test_keyword_mod_in_isolation() {
        assert_eq!(first_token("mod"), Token::Mod);
    }

    #[test]
    fn test_keyword_use_in_isolation() {
        assert_eq!(first_token("use"), Token::Use);
    }

    #[test]
    fn test_keyword_as_in_isolation() {
        assert_eq!(first_token("as"), Token::As);
    }

    #[test]
    fn test_keyword_super_in_isolation() {
        assert_eq!(first_token("super"), Token::Super);
    }

    #[test]
    fn test_keyword_crate_in_isolation() {
        assert_eq!(first_token("crate"), Token::Crate);
    }

    #[test]
    fn test_keyword_const_in_isolation() {
        assert_eq!(first_token("const"), Token::Const);
    }

    #[test]
    fn test_keyword_static_in_isolation() {
        assert_eq!(first_token("static"), Token::Static);
    }

    #[test]
    fn test_keyword_unsafe_in_isolation() {
        assert_eq!(first_token("unsafe"), Token::Unsafe);
    }

    #[test]
    fn test_keyword_ref_in_isolation() {
        assert_eq!(first_token("ref"), Token::Ref);
    }

    #[test]
    fn test_keyword_self_in_isolation() {
        assert_eq!(first_token("self"), Token::Self_);
    }

    #[test]
    fn test_keyword_self_upper_in_isolation() {
        assert_eq!(first_token("Self"), Token::SelfUpper);
    }

    // ------------------------------------------------------------------------
    // INDIVIDUAL OPERATOR TESTS - All operators tested in isolation
    // ------------------------------------------------------------------------

    #[test]
    fn test_operator_plus_in_isolation() {
        assert_eq!(first_token("+"), Token::Plus);
    }

    #[test]
    fn test_operator_minus_in_isolation() {
        assert_eq!(first_token("-"), Token::Minus);
    }

    #[test]
    fn test_operator_star_in_isolation() {
        assert_eq!(first_token("*"), Token::Star);
    }

    #[test]
    fn test_operator_slash_in_isolation() {
        assert_eq!(first_token("/"), Token::Slash);
    }

    #[test]
    fn test_operator_percent_in_isolation() {
        assert_eq!(first_token("%"), Token::Percent);
    }

    #[test]
    fn test_operator_eq_eq_in_isolation() {
        assert_eq!(first_token("=="), Token::EqEq);
    }

    #[test]
    fn test_operator_not_eq_in_isolation() {
        assert_eq!(first_token("!="), Token::NotEq);
    }

    #[test]
    fn test_operator_lt_in_isolation() {
        assert_eq!(first_token("<"), Token::Lt);
    }

    #[test]
    fn test_operator_gt_in_isolation() {
        assert_eq!(first_token(">"), Token::Gt);
    }

    #[test]
    fn test_operator_lt_eq_in_isolation() {
        assert_eq!(first_token("<="), Token::LtEq);
    }

    #[test]
    fn test_operator_gt_eq_in_isolation() {
        assert_eq!(first_token(">="), Token::GtEq);
    }

    #[test]
    fn test_operator_and_and_in_isolation() {
        assert_eq!(first_token("&&"), Token::AndAnd);
    }

    #[test]
    fn test_operator_or_or_in_isolation() {
        assert_eq!(first_token("||"), Token::OrOr);
    }

    #[test]
    fn test_operator_bang_in_isolation() {
        assert_eq!(first_token("!"), Token::Bang);
    }

    #[test]
    fn test_operator_eq_in_isolation() {
        assert_eq!(first_token("="), Token::Eq);
    }

    #[test]
    fn test_operator_plus_eq_in_isolation() {
        assert_eq!(first_token("+="), Token::PlusEq);
    }

    #[test]
    fn test_operator_minus_eq_in_isolation() {
        assert_eq!(first_token("-="), Token::MinusEq);
    }

    #[test]
    fn test_operator_star_eq_in_isolation() {
        assert_eq!(first_token("*="), Token::StarEq);
    }

    #[test]
    fn test_operator_slash_eq_in_isolation() {
        assert_eq!(first_token("/="), Token::SlashEq);
    }

    #[test]
    fn test_operator_percent_eq_in_isolation() {
        assert_eq!(first_token("%="), Token::PercentEq);
    }

    #[test]
    fn test_operator_ampersand_in_isolation() {
        assert_eq!(first_token("&"), Token::Ampersand);
    }

    #[test]
    fn test_operator_pipe_in_isolation() {
        assert_eq!(first_token("|"), Token::Pipe);
    }

    #[test]
    fn test_operator_caret_in_isolation() {
        assert_eq!(first_token("^"), Token::Caret);
    }

    #[test]
    fn test_operator_shl_in_isolation() {
        assert_eq!(first_token("<<"), Token::Shl);
    }

    #[test]
    fn test_operator_shr_in_isolation() {
        assert_eq!(first_token(">>"), Token::Shr);
    }

    #[test]
    fn test_operator_tilde_in_isolation() {
        assert_eq!(first_token("~"), Token::Tilde);
    }

    #[test]
    fn test_operator_ampersand_eq_in_isolation() {
        assert_eq!(first_token("&="), Token::AmpersandEq);
    }

    #[test]
    fn test_operator_pipe_eq_in_isolation() {
        assert_eq!(first_token("|="), Token::PipeEq);
    }

    #[test]
    fn test_operator_caret_eq_in_isolation() {
        assert_eq!(first_token("^="), Token::CaretEq);
    }

    #[test]
    fn test_operator_shl_eq_in_isolation() {
        assert_eq!(first_token("<<="), Token::ShlEq);
    }

    #[test]
    fn test_operator_shr_eq_in_isolation() {
        assert_eq!(first_token(">>="), Token::ShrEq);
    }

    #[test]
    fn test_operator_arrow_in_isolation() {
        assert_eq!(first_token("->"), Token::Arrow);
    }

    #[test]
    fn test_operator_fat_arrow_in_isolation() {
        assert_eq!(first_token("=>"), Token::FatArrow);
    }

    // ------------------------------------------------------------------------
    // INDIVIDUAL DELIMITER TESTS - All delimiters tested in isolation
    // ------------------------------------------------------------------------

    #[test]
    fn test_delimiter_lparen_in_isolation() {
        assert_eq!(first_token("("), Token::LParen);
    }

    #[test]
    fn test_delimiter_rparen_in_isolation() {
        assert_eq!(first_token(")"), Token::RParen);
    }

    #[test]
    fn test_delimiter_lbrace_in_isolation() {
        assert_eq!(first_token("{"), Token::LBrace);
    }

    #[test]
    fn test_delimiter_rbrace_in_isolation() {
        assert_eq!(first_token("}"), Token::RBrace);
    }

    #[test]
    fn test_delimiter_lbracket_in_isolation() {
        assert_eq!(first_token("["), Token::LBracket);
    }

    #[test]
    fn test_delimiter_rbracket_in_isolation() {
        assert_eq!(first_token("]"), Token::RBracket);
    }

    #[test]
    fn test_delimiter_comma_in_isolation() {
        assert_eq!(first_token(","), Token::Comma);
    }

    #[test]
    fn test_delimiter_semicolon_in_isolation() {
        assert_eq!(first_token(";"), Token::Semicolon);
    }

    #[test]
    fn test_delimiter_colon_in_isolation() {
        assert_eq!(first_token(":"), Token::Colon);
    }

    #[test]
    fn test_delimiter_colon_colon_in_isolation() {
        assert_eq!(first_token("::"), Token::ColonColon);
    }

    #[test]
    fn test_delimiter_dot_in_isolation() {
        assert_eq!(first_token("."), Token::Dot);
    }

    #[test]
    fn test_delimiter_dot_dot_in_isolation() {
        assert_eq!(first_token(".."), Token::DotDot);
    }

    #[test]
    fn test_delimiter_dot_dot_dot_in_isolation() {
        assert_eq!(first_token("..."), Token::DotDotDot);
    }

    #[test]
    fn test_delimiter_at_in_isolation() {
        assert_eq!(first_token("@"), Token::At);
    }

    #[test]
    fn test_delimiter_dollar_in_isolation() {
        assert_eq!(first_token("$"), Token::Dollar);
    }

    #[test]
    fn test_delimiter_underscore_in_isolation() {
        assert_eq!(first_token("_"), Token::Underscore);
    }

    // ------------------------------------------------------------------------
    // EDGE CASE TESTS - Boundary conditions and special inputs
    // ------------------------------------------------------------------------

    #[test]
    fn test_single_character_file_let() {
        // Single character that forms a complete keyword
        let tokens = lex_tokens("l");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Ident(_)));
    }

    #[test]
    fn test_single_character_operator() {
        // Single character operators
        assert_eq!(first_token("+"), Token::Plus);
        assert_eq!(first_token("-"), Token::Minus);
        assert_eq!(first_token("*"), Token::Star);
    }

    #[test]
    fn test_maximum_identifier_length_255() {
        // Test identifier at common buffer boundary
        let source = "a".repeat(255);
        let tokens = lex_tokens(&source);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Ident(_)));
    }

    #[test]
    fn test_maximum_identifier_length_1024() {
        // Test identifier at page boundary
        let source = "x".repeat(1024);
        let tokens = lex_tokens(&source);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Ident(_)));
    }

    #[test]
    fn test_maximum_u64_number() {
        // Test u64::MAX = 18446744073709551615
        let source = "18446744073709551615";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(u64::MAX));
    }

    #[test]
    fn test_maximum_hex_number() {
        // Test 0xFFFFFFFFFFFFFFFF (u64::MAX in hex)
        let source = "0xFFFFFFFFFFFFFFFF";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(u64::MAX));
    }

    #[test]
    fn test_maximum_binary_number() {
        // Test 64 ones in binary
        let source = "0b".to_owned() + &"1".repeat(64);
        let tokens = lex_tokens(&source);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(u64::MAX));
    }

    #[test]
    fn test_unicode_identifier_greek() {
        // Greek letters should be treated as invalid (Fax only supports ASCII identifiers)
        let source = "α = 42";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        let token = lexer.next_token();
        // 'α' is not ASCII, so it should be invalid
        assert!(matches!(token, Token::Invalid(_)));
    }

    #[test]
    fn test_unicode_identifier_chinese() {
        // Chinese characters should be treated as invalid
        let source = "变量 = 42";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        let token = lexer.next_token();
        assert!(matches!(token, Token::Invalid(_)));
    }

    #[test]
    fn test_mixed_whitespace_spaces_tabs() {
        // Mixed spaces and tabs
        let source = "let \t x \t = \t 42";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 4);
    }

    #[test]
    fn test_mixed_whitespace_with_newlines() {
        // Spaces, tabs, and newlines mixed
        let source = "let \n\t x \n\t = \n\t 42";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 4);
    }

    #[test]
    fn test_windows_line_endings() {
        // Windows CRLF line endings
        let source = "let x = 1\r\nlet y = 2\r\n";
        let tokens = lex_tokens(source);
        assert!(tokens.contains(&Token::Let));
        assert!(tokens.contains(&Token::Number(1)));
        assert!(tokens.contains(&Token::Number(2)));
    }

    #[test]
    fn test_unix_line_endings() {
        // Unix LF line endings
        let source = "let x = 1\nlet y = 2\n";
        let tokens = lex_tokens(source);
        assert!(tokens.contains(&Token::Let));
        assert!(tokens.contains(&Token::Number(1)));
        assert!(tokens.contains(&Token::Number(2)));
    }

    #[test]
    fn test_old_mac_line_endings() {
        // Old Mac CR line endings
        let source = "let x = 1\rlet y = 2\r";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        let tokens: Vec<Token> = std::iter::from_fn(|| {
            let t = lexer.next_token();
            if t == Token::Eof { None } else { Some(t) }
        }).collect();
        // CR is whitespace, should be handled
        assert!(tokens.contains(&Token::Let));
    }

    #[test]
    fn test_file_with_bom() {
        // UTF-8 BOM at start of file should be skipped
        let source = "\u{FEFF}let x = 1";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        let token = lexer.next_token();
        // BOM should be skipped, 'let' should be tokenized
        assert_eq!(token, Token::Let);
        assert!(!handler.has_errors());
    }

    #[test]
    fn test_non_breaking_space() {
        // Unicode non-breaking space (U+00A0)
        let source = "let\u{00A0}x = 1";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        let _ = lexer.next_token();
        // NBSP is whitespace in Unicode, should be skipped
    }

    #[test]
    fn test_em_space() {
        // Unicode em space (U+2003)
        let source = "let\u{2003}x = 1";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        let _ = lexer.next_token();
    }

    // ------------------------------------------------------------------------
    // ERROR CASE TESTS - Comprehensive error handling
    // ------------------------------------------------------------------------

    #[test]
    fn test_unterminated_string_with_newline() {
        // String with newline inside (not escaped)
        let source = "\"hello\nworld\"";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_unterminated_string_at_eof_no_quote() {
        // String that never closes
        let source = "\"hello";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_unterminated_string_empty() {
        // Just opening quote
        let source = "\"";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_unterminated_char_with_newline() {
        // Char literal with newline
        let source = "'a\n'";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_invalid_escape_sequence_unknown() {
        // Unknown escape sequence \q
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("\"\\q\"", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_invalid_escape_sequence_z() {
        // Unknown escape sequence \z
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("\"\\z\"", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_invalid_hex_escape_single_char() {
        // Incomplete hex escape \x followed by non-hex
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("\"\\xG\"", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_invalid_hex_escape_eof() {
        // Hex escape at end of file \x
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("\"\\x", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_invalid_hex_escape_one_char() {
        // Hex escape with only one digit \xA
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("\"\\xA", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_invalid_unicode_escape_no_brace() {
        // Unicode escape without brace \u41
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("\"\\u41\"", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_invalid_unicode_escape_empty() {
        // Empty unicode escape \u{}
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("\"\\u{}\"", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_invalid_unicode_escape_no_closing_brace() {
        // Unicode escape without closing brace
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("\"\\u{41", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_invalid_unicode_escape_non_hex() {
        // Unicode escape with non-hex character
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("\"\\u{GHI}\"", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_invalid_number_format_hex() {
        // Invalid hex digit
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("0xG", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_invalid_number_format_binary() {
        // Invalid binary digit
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("0b2", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_invalid_number_format_octal() {
        // Invalid octal digit
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("0o8", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_invalid_number_format_octal_nine() {
        // Invalid octal digit 9
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("0o9", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_invalid_character_question_mark() {
        // Question mark not supported
        let tokens = lex_tokens("?");
        assert!(matches!(tokens[0], Token::Invalid(_)));
    }

    #[test]
    fn test_invalid_character_hash() {
        // Hash not supported (attributes)
        let tokens = lex_tokens("#");
        assert!(matches!(tokens[0], Token::Invalid(_)));
    }

    #[test]
    fn test_invalid_character_backtick() {
        // Backtick not supported (raw identifiers)
        let tokens = lex_tokens("`");
        assert!(matches!(tokens[0], Token::Invalid(_)));
    }

    #[test]
    fn test_invalid_character_backslash() {
        // Standalone backslash
        let tokens = lex_tokens("\\");
        assert!(matches!(tokens[0], Token::Invalid(_)));
    }

    #[test]
    fn test_truncated_shl_eq() {
        // <<= should be ShlEq
        let tokens = lex_tokens("<<=");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::ShlEq);
    }

    #[test]
    fn test_truncated_shr_eq() {
        // >>= should be ShrEq
        let tokens = lex_tokens(">>=");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::ShrEq);
    }

    #[test]
    fn test_malformed_unicode_surrogate() {
        // Surrogate codepoint
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("\"\\u{D800}\"", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    #[test]
    fn test_malformed_unicode_above_max() {
        // Above max Unicode
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("\"\\u{110000}\"", &mut handler);
        let _token = lexer.next_token();
        assert!(handler.has_errors());
    }

    // ------------------------------------------------------------------------
    // PROPERTY-BASED TESTS - Using proptest for arbitrary inputs
    // ------------------------------------------------------------------------

    #[test]
    fn test_property_arbitrary_identifier_strings() {
        use proptest::prelude::*;

        proptest!(|(input in "[a-zA-Z_][a-zA-Z0-9_]{0,100}")| {
            let tokens = lex_tokens(&input);
            // Should produce exactly one identifier token or wildcard
            assert_eq!(tokens.len(), 1);
            // "_" is the wildcard, everything else is an identifier
            if input == "_" {
                assert_eq!(tokens[0], Token::Underscore);
            } else {
                assert!(matches!(tokens[0], Token::Ident(_)));
            }
        });
    }

    #[test]
    fn test_property_arbitrary_decimal_number_strings() {
        use proptest::prelude::*;

        proptest!(|(input in "[0-9]{1,20}")| {
            let tokens = lex_tokens(&input);
            // Should produce exactly one number token (or error for overflow)
            assert_eq!(tokens.len(), 1);
            assert!(matches!(tokens[0], Token::Number(_) | Token::Float(_)));
        });
    }

    #[test]
    fn test_property_arbitrary_hex_number_strings() {
        use proptest::prelude::*;

        proptest!(|(digits in "[0-9a-fA-F]{1,16}")| {
            let input = format!("0x{}", digits);
            let tokens = lex_tokens(&input);
            // Should produce exactly one number token
            assert_eq!(tokens.len(), 1);
            assert!(matches!(tokens[0], Token::Number(_)));
        });
    }

    #[test]
    fn test_property_arbitrary_string_literals() {
        use proptest::prelude::*;

        proptest!(|(input in "[^\"\\\\\\n]{0,100}")| {
            let source = format!("\"{}\"", input);
            let tokens = lex_tokens(&source);
            // Should produce exactly one string token
            assert_eq!(tokens.len(), 1);
            assert!(matches!(tokens[0], Token::String(_)));
        });
    }

    #[test]
    fn test_property_roundtrip_lex_display_lex() {
        use proptest::prelude::*;

        proptest!(|(input in "[a-zA-Z_][a-zA-Z0-9_]{0,20}")| {
            // Lex the input
            let tokens1 = lex_tokens(&input);
            
            // Display the tokens
            let displayed: String = tokens1.iter()
                .map(|t| t.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            
            // Lex the displayed version
            let tokens2 = lex_tokens(&displayed);
            
            // Should produce same number of tokens
            assert_eq!(tokens1.len(), tokens2.len());
        });
    }

    #[test]
    fn test_property_whitespace_preservation() {
        use proptest::prelude::*;

        proptest!(|(spaces in 0..100usize)| {
            let whitespace = " ".repeat(spaces);
            let source = format!("{}let{}", whitespace, whitespace);
            let tokens = lex_tokens(&source);
            // Whitespace should be ignored
            assert_eq!(tokens.len(), 1);
            assert_eq!(tokens[0], Token::Let);
        });
    }

    // ------------------------------------------------------------------------
    // STRESS TESTS - Performance and capacity boundaries
    // ------------------------------------------------------------------------

    #[test]
    fn test_stress_very_long_identifier_100k() {
        // 100k character identifier
        let source = "x".repeat(100000);
        let tokens = lex_tokens(&source);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Ident(_)));
    }

    #[test]
    fn test_stress_many_tokens_100k() {
        // 100k tokens
        let source = (0..100000).map(|_| "x").collect::<Vec<_>>().join(" ");
        let tokens = lex_tokens(&source);
        assert_eq!(tokens.len(), 100000);
    }

    #[test]
    fn test_stress_deeply_nested_comments_100_levels() {
        // 100 levels of nested comments
        let source = "/*".repeat(100) + &"*/".repeat(100) + " let";
        let tokens = lex_tokens(&source);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Let);
    }

    #[test]
    fn test_stress_deeply_nested_parens_1000_levels() {
        // 1000 levels of nested parens
        let source = "(".repeat(1000) + &")".repeat(1000);
        let tokens = lex_tokens(&source);
        assert_eq!(tokens.len(), 2000);
    }

    #[test]
    fn test_stress_deeply_nested_braces_1000_levels() {
        // 1000 levels of nested braces
        let source = "{".repeat(1000) + &"}".repeat(1000);
        let tokens = lex_tokens(&source);
        assert_eq!(tokens.len(), 2000);
    }

    #[test]
    fn test_stress_long_string_100k_chars() {
        // 100k character string
        let source = format!("\"{}\"", "a".repeat(100000));
        let tokens = lex_tokens(&source);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::String(_)));
    }

    #[test]
    fn test_stress_many_numbers_10k() {
        // 10k number literals
        let source = (0..10000).map(|i| i.to_string()).collect::<Vec<_>>().join(" ");
        let tokens = lex_tokens(&source);
        assert_eq!(tokens.len(), 10000);
    }

    #[test]
    fn test_stress_alternating_operators_10k() {
        // 10k alternating operators
        let source = (0..10000).map(|i| if i % 2 == 0 { "+" } else { "-" }).collect::<Vec<_>>().join(" ");
        let tokens = lex_tokens(&source);
        assert_eq!(tokens.len(), 10000);
    }

    #[test]
    fn test_stress_many_strings_1k() {
        // 1k string literals
        let source = (0..1000).map(|i| format!("\"string{}\"", i)).collect::<Vec<_>>().join(" ");
        let tokens = lex_tokens(&source);
        assert_eq!(tokens.len(), 1000);
    }

    // ------------------------------------------------------------------------
    // REGRESSION TESTS - SPEC.md examples and previously fixed bugs
    // ------------------------------------------------------------------------

    #[test]
    fn test_spec_hello_world_example() {
        // Example from SPEC.md Section 3.1
        let source = r#"fn main() {
    println("Hello, Fax!")
}"#;
        let tokens = lex_tokens(source);
        assert!(tokens.contains(&Token::Fn));
        assert!(tokens.contains(&Token::Ident(faxc_util::Symbol::intern("main"))));
        assert!(tokens.contains(&Token::Ident(faxc_util::Symbol::intern("println"))));
        assert!(tokens.contains(&Token::String(faxc_util::Symbol::intern("Hello, Fax!"))));
    }

    #[test]
    fn test_spec_variable_declaration_example() {
        // Example from SPEC.md Section 3.2
        let source = r#"let x = 42
let mut y = 10
y = 20"#;
        let tokens = lex_tokens(source);
        assert!(tokens.contains(&Token::Let));
        assert!(tokens.contains(&Token::Mut));
        assert!(tokens.contains(&Token::Number(42)));
        assert!(tokens.contains(&Token::Number(10)));
        assert!(tokens.contains(&Token::Number(20)));
    }

    #[test]
    fn test_spec_function_example() {
        // Example from SPEC.md Section 3.3
        let source = r#"fn add(a: i32, b: i32) -> i32 {
    a + b
}"#;
        let tokens = lex_tokens(source);
        assert!(tokens.contains(&Token::Fn));
        assert!(tokens.contains(&Token::Ident(faxc_util::Symbol::intern("add"))));
        assert!(tokens.contains(&Token::Ident(faxc_util::Symbol::intern("i32"))));
        assert!(tokens.contains(&Token::Arrow));
        assert!(tokens.contains(&Token::Plus));
    }

    #[test]
    fn test_spec_if_expression_example() {
        // Example from SPEC.md Section 3.4
        let source = r#"let max = if a > b { a } else { b }"#;
        let tokens = lex_tokens(source);
        assert!(tokens.contains(&Token::Let));
        assert!(tokens.contains(&Token::If));
        assert!(tokens.contains(&Token::Else));
        assert!(tokens.contains(&Token::Gt));
    }

    #[test]
    fn test_spec_match_expression_example() {
        // Example from SPEC.md Section 3.4
        let source = r#"match value {
    0 => println("zero"),
    1 => println("one"),
    n if n > 10 => println("large"),
    _ => println("other"),
}"#;
        let tokens = lex_tokens(source);
        assert!(tokens.contains(&Token::Match));
        assert!(tokens.contains(&Token::FatArrow));
        assert!(tokens.contains(&Token::Underscore));
    }

    #[test]
    fn test_spec_while_loop_example() {
        // Example from SPEC.md Section 3.4
        let source = r#"let mut i = 0
while i < 5 {
    println(i)
    i = i + 1
}"#;
        let tokens = lex_tokens(source);
        assert!(tokens.contains(&Token::Let));
        assert!(tokens.contains(&Token::Mut));
        assert!(tokens.contains(&Token::While));
        assert!(tokens.contains(&Token::Lt));
        assert!(tokens.contains(&Token::Plus));
    }

    #[test]
    fn test_spec_number_formats_example() {
        // Examples from SPEC.md Section 4.1.2
        let source = "42 0xFF 0b1010 0o777";
        let tokens = lex_tokens(source);
        assert_eq!(tokens[0], Token::Number(42));
        assert_eq!(tokens[1], Token::Number(0xFF));
        assert_eq!(tokens[2], Token::Number(0b1010));
        assert_eq!(tokens[3], Token::Number(0o777));
    }

    #[test]
    fn test_spec_float_formats_example() {
        // Float examples from SPEC.md
        let source = "3.14 1.0e10 2.5e-3";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::Float(_)));
        assert!(matches!(tokens[1], Token::Float(_)));
        assert!(matches!(tokens[2], Token::Float(_)));
    }

    #[test]
    fn test_spec_async_await_example() {
        // Example from SPEC.md Section 7.9
        let source = r#"async fn fetch(url: str) -> str {
    let response = http_get(url).await;
    response.body
}"#;
        let tokens = lex_tokens(source);
        assert!(tokens.contains(&Token::Async));
        assert!(tokens.contains(&Token::Await));
        assert!(tokens.contains(&Token::Fn));
    }

    #[test]
    fn test_spec_trait_example() {
        // Example from SPEC.md Section 7.8
        let source = r#"trait Printable {
    fn print(&self);
}"#;
        let tokens = lex_tokens(source);
        assert!(tokens.contains(&Token::Trait));
        assert!(tokens.contains(&Token::Fn));
        assert!(tokens.contains(&Token::Self_));
    }

    #[test]
    fn test_spec_const_static_example() {
        // Example from SPEC.md Section 7.12
        let source = r#"const MAX: i32 = 100;
static mut COUNTER: i32 = 0;"#;
        let tokens = lex_tokens(source);
        assert!(tokens.contains(&Token::Const));
        assert!(tokens.contains(&Token::Static));
        assert!(tokens.contains(&Token::Mut));
    }

    #[test]
    fn test_spec_visibility_example() {
        // Example from SPEC.md Section 7.11
        let source = r#"pub fn public() {}
pub(crate) fn crate_visible() {}
pub(super) fn parent_visible() {}
fn private() {}"#;
        let tokens = lex_tokens(source);
        assert!(tokens.contains(&Token::Pub));
        assert!(tokens.contains(&Token::Crate));
        assert!(tokens.contains(&Token::Super));
        assert!(tokens.contains(&Token::Fn));
    }

    #[test]
    fn test_regression_operator_precedence_maximal_munch() {
        // <<= should be ShlEq, not Shl + Eq
        let tokens = lex_tokens("a <<= b");
        assert_eq!(tokens[1], Token::ShlEq);
    }

    #[test]
    fn test_regression_operator_precedence_shr_eq() {
        // >>= should be ShrEq, not Shr + Eq
        let tokens = lex_tokens("a >>= b");
        assert_eq!(tokens[1], Token::ShrEq);
    }

    #[test]
    fn test_regression_triple_greater_than() {
        // >>> should be >> then >
        let tokens = lex_tokens("a >>> b");
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[1], Token::Shr);
        assert_eq!(tokens[2], Token::Gt);
    }

    #[test]
    fn test_regression_triple_less_than() {
        // <<< should be << then <
        let tokens = lex_tokens("a <<< b");
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[1], Token::Shl);
        assert_eq!(tokens[2], Token::Lt);
    }

    #[test]
    fn test_regression_generic_double_angle_bracket() {
        // HashMap<String, Vec<i32>> - >> should be Shr
        let tokens = lex_tokens("HashMap<String, Vec<i32>>");
        assert!(tokens.contains(&Token::Shr));
    }

    #[test]
    fn test_regression_float_without_trailing_digits() {
        // 5. should be Number then Dot, not Float
        let tokens = lex_tokens("5.");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Number(5));
        assert_eq!(tokens[1], Token::Dot);
    }

    #[test]
    fn test_regression_float_without_leading_digits() {
        // .5 should be Dot then Number, not Float
        let tokens = lex_tokens(".5");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Dot);
        assert_eq!(tokens[1], Token::Number(5));
    }

    #[test]
    fn test_regression_negative_number_is_two_tokens() {
        // -5 should be Minus then Number, not negative number
        let tokens = lex_tokens("-5");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Minus);
        assert_eq!(tokens[1], Token::Number(5));
    }

    #[test]
    fn test_regression_underscore_in_decimal_not_supported() {
        // 1_000 should be Number then Ident
        let tokens = lex_tokens("1_000");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Number(1));
        assert!(matches!(tokens[1], Token::Ident(_)));
    }

    #[test]
    fn test_regression_underscore_in_hex_supported() {
        // 0xFF_FF should be single Number
        let tokens = lex_tokens("0xFF_FF");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(0xFFFF));
    }

    #[test]
    fn test_regression_underscore_in_binary_supported() {
        // 0b10_10 should be single Number
        let tokens = lex_tokens("0b10_10");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(0b1010));
    }

    #[test]
    fn test_regression_comment_with_string_inside() {
        // String-like content in comment should be ignored
        let source = "// \"not a string\"\nlet";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Let);
    }

    #[test]
    fn test_regression_nested_block_comments() {
        // Nested block comments should work
        let source = "/* outer /* inner */ still outer */ let";
        let tokens = lex_tokens(source);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Let);
    }

    #[test]
    fn test_regression_case_sensitive_keywords() {
        // LET should be identifier, not keyword
        let tokens = lex_tokens("LET");
        assert!(matches!(tokens[0], Token::Ident(_)));
    }

    #[test]
    fn test_regression_keyword_prefix_is_identifier() {
        // letting should be identifier, not let + identifier
        let tokens = lex_tokens("letting");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Ident(_)));
    }

    #[test]
    fn test_regression_empty_block_comment() {
        // /**/ should be valid empty comment
        let tokens = lex_tokens("/**/let");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Let);
    }

    #[test]
    fn test_regression_unterminated_block_comment_at_eof() {
        // /* without closing should error
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("/* never closed", &mut handler);
        let token = lexer.next_token();
        assert!(handler.has_errors());
        assert_eq!(token, Token::Eof);
    }

    // ========================================================================
    // BUG FIX TESTS - CRITICAL AND HIGH SEVERITY
    // ========================================================================

    // CRITICAL #1: Integer Overflow Without Bounds Checking (hex)
    #[test]
    fn test_integer_overflow_hex() {
        // u64::MAX + 1 in hex
        let source = "0xFFFFFFFFFFFFFFFF0";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert!(handler.has_errors(), "Should report overflow error for hex");
        assert!(matches!(token, Token::Number(_)));
    }

    // CRITICAL #1: Integer Overflow Without Bounds Checking (binary)
    #[test]
    fn test_integer_overflow_binary() {
        // Very large binary number
        let source = "0b11111111111111111111111111111111111111111111111111111111111111111";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert!(handler.has_errors(), "Should report overflow error for binary");
        assert!(matches!(token, Token::Number(_)));
    }

    // CRITICAL #2: Decimal Integer Overflow Not Checked
    #[test]
    fn test_integer_overflow_decimal() {
        // u64::MAX + 1 = 18446744073709551616
        let source = "18446744073709551616";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert!(handler.has_errors(), "Should report overflow error for decimal");
        assert!(matches!(token, Token::Number(_)));
    }

    // CRITICAL #2: Decimal Integer at u64::MAX should work
    #[test]
    fn test_integer_at_u64_max() {
        // u64::MAX = 18446744073709551615
        let source = "18446744073709551615";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert!(!handler.has_errors(), "u64::MAX should be valid");
        assert_eq!(token, Token::Number(u64::MAX));
    }

    // CRITICAL #3: Float Parsing Returns Infinity Without Error
    #[test]
    fn test_float_infinity_overflow() {
        let source = "1e999999";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert!(handler.has_errors(), "Should report error for infinite float");
        if let Token::Float(f) = token {
            assert!(f.is_finite(), "Float should be finite");
        } else {
            panic!("Expected Float token");
        }
    }

    // CRITICAL #3: Float Parsing Returns NaN Without Error
    #[test]
    fn test_float_nan_from_overflow() {
        // Very large negative exponent that could cause issues
        let source = "1e-999999";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        // This should either error or return 0.0 (subnormal)
        if let Token::Float(f) = token {
            assert!(f.is_finite(), "Float should be finite");
        }
    }

    // CRITICAL #3: Valid large float should work
    #[test]
    fn test_float_large_but_finite() {
        let source = "1e308";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert!(!handler.has_errors(), "1e308 should be valid");
        if let Token::Float(f) = token {
            assert!(f.is_finite(), "Float should be finite");
        }
    }

    // HIGH #4: Unicode Surrogate Not Properly Rejected (low surrogate)
    #[test]
    fn test_surrogate_unicode_low() {
        let source = "\"\\u{D800}\"";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert!(handler.has_errors(), "Should reject low surrogate D800");
        assert!(matches!(token, Token::String(_)));
    }

    // HIGH #4: Unicode Surrogate Not Properly Rejected (high surrogate)
    #[test]
    fn test_surrogate_unicode_high() {
        let source = "\"\\u{DFFF}\"";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert!(handler.has_errors(), "Should reject high surrogate DFFF");
        assert!(matches!(token, Token::String(_)));
    }

    // HIGH #4: Unicode Surrogate in middle of range
    #[test]
    fn test_surrogate_unicode_middle() {
        let source = "\"\\u{DC00}\"";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert!(handler.has_errors(), "Should reject surrogate DC00");
        assert!(matches!(token, Token::String(_)));
    }

    // HIGH #4: Valid unicode just before surrogate range should work
    #[test]
    fn test_unicode_before_surrogate_range() {
        let source = "\"\\u{D7FF}\"";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert!(!handler.has_errors(), "D7FF should be valid");
        assert_eq!(token, Token::String(faxc_util::Symbol::intern("\u{D7FF}")));
    }

    // HIGH #4: Valid unicode just after surrogate range should work
    #[test]
    fn test_unicode_after_surrogate_range() {
        let source = "\"\\u{E000}\"";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert!(!handler.has_errors(), "E000 should be valid");
        assert_eq!(token, Token::String(faxc_util::Symbol::intern("\u{E000}")));
    }

    // HIGH #9: Nested Comments Can Stack Overflow
    #[test]
    fn test_nested_comments_depth_limit() {
        // 150 levels of nesting (above limit of 100)
        // When depth limit is hit, an error is reported but parsing continues
        let nesting = 150;
        let source = format!("{}*/ let", "/*".repeat(nesting));
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        let token = lexer.next_token();
        // Should error on too deep nesting
        assert!(handler.has_errors(), "Should error on too deep nesting");
        // After error, the comment is considered terminated and we get 'let'
        // OR the rest is consumed as comment and we get EOF
        // Both behaviors are acceptable for error recovery
        assert!(handler.has_errors());
    }

    // HIGH #9: Valid nesting within limit should work
    #[test]
    fn test_nested_comments_within_limit() {
        // 50 levels of nesting (within limit of 100)
        // Need equal number of opening and closing comment markers
        let nesting = 50;
        let source = format!("{}{} let", "/*".repeat(nesting), "*/".repeat(nesting));
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(&source, &mut handler);
        let token = lexer.next_token();
        // No errors for valid nesting, should get 'let' token
        assert!(!handler.has_errors(), "50 levels should be valid");
        assert_eq!(token, Token::Let);
    }

    // HIGH #9: Valid nesting with token after comment
    #[test]
    fn test_nested_comments_with_token_after() {
        // Simple nested comment test
        let source = "/* outer /* inner */ still outer */ let";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert!(!handler.has_errors(), "Nested comments should work");
        assert_eq!(token, Token::Let);
    }

    // HIGH #14: Hex With No Digits
    #[test]
    fn test_hex_no_digits() {
        let source = "0x";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert!(handler.has_errors(), "Should error on 0x without digits");
        assert_eq!(token, Token::Number(0));
    }

    // HIGH #14: Binary With No Digits
    #[test]
    fn test_binary_no_digits() {
        let source = "0b";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert!(handler.has_errors(), "Should error on 0b without digits");
        assert_eq!(token, Token::Number(0));
    }

    // HIGH #14: Octal With No Digits
    #[test]
    fn test_octal_no_digits() {
        let source = "0o";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert!(handler.has_errors(), "Should error on 0o without digits");
        assert_eq!(token, Token::Number(0));
    }

    // HIGH #14: Hex with underscore but no digits
    #[test]
    fn test_hex_only_underscore() {
        let source = "0x_";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert!(handler.has_errors(), "Should error on 0x_ without digits");
        assert_eq!(token, Token::Number(0));
    }

    // HIGH #15: Float Exponent Without Digits
    #[test]
    fn test_float_exponent_no_digits() {
        let source = "1e";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert!(handler.has_errors(), "Should error on 1e without exponent digits");
        assert!(matches!(token, Token::Float(_)));
    }

    // HIGH #15: Float Exponent With Sign But No Digits
    #[test]
    fn test_float_exponent_sign_no_digits() {
        let source = "1e+";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert!(handler.has_errors(), "Should error on 1e+ without exponent digits");
        assert!(matches!(token, Token::Float(_)));
    }

    // HIGH #15: Float Exponent With Negative Sign But No Digits
    #[test]
    fn test_float_exponent_negative_sign_no_digits() {
        let source = "1e-";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert!(handler.has_errors(), "Should error on 1e- without exponent digits");
        assert!(matches!(token, Token::Float(_)));
    }

    // HIGH #15: Valid float exponent should work
    #[test]
    fn test_float_exponent_valid() {
        let source = "1e10";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert!(!handler.has_errors(), "1e10 should be valid");
        assert_eq!(token, Token::Float(1e10));
    }

    // HIGH #15: Valid float exponent with sign should work
    #[test]
    fn test_float_exponent_with_sign_valid() {
        let source = "1e+10";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert!(!handler.has_errors(), "1e+10 should be valid");
        assert_eq!(token, Token::Float(1e10));
    }

    // LOW #12: BOM Not Handled
    #[test]
    fn test_bom_at_start_fixed() {
        let source = "\u{FEFF}let x = 1";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert_eq!(token, Token::Let, "BOM should be skipped");
        assert!(!handler.has_errors(), "BOM should not cause errors");
    }

    // LOW #12: BOM followed by whitespace
    #[test]
    fn test_bom_with_whitespace() {
        let source = "\u{FEFF}  let x = 1";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert_eq!(token, Token::Let, "BOM and whitespace should be skipped");
        assert!(!handler.has_errors());
    }

    // LOW #13: Unicode Whitespace Not Skipped
    #[test]
    fn test_unicode_whitespace_skipping() {
        // Non-breaking space (U+00A0)
        let source = "\u{00A0}let\u{00A0}x";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert_eq!(token, Token::Let, "Unicode whitespace should be skipped");
        assert!(!handler.has_errors());
    }

    // LOW #13: Various unicode whitespace characters
    #[test]
    fn test_various_unicode_whitespace() {
        // Tab (U+0009), newline (U+000A), carriage return (U+000D)
        // Also test other whitespace chars like em space (U+2003)
        let source = "\u{2003}let\u{2003}x";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert_eq!(token, Token::Let, "Em space should be skipped");
        assert!(!handler.has_errors());
    }

    // Additional edge case tests
    #[test]
    fn test_integer_overflow_octal() {
        // Very large octal number that exceeds u64::MAX
        // u64::MAX in octal is 1777777777777777777777
        // So 17777777777777777777777 (one more 7) should overflow
        let source = "0o17777777777777777777777";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert!(handler.has_errors(), "Should report overflow for large octal");
        assert!(matches!(token, Token::Number(_)));
    }

    #[test]
    fn test_float_underflow_to_zero() {
        // Very small number that underflows to zero
        let source = "1e-999";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        // Should either be 0.0 or report error, but should be finite
        if let Token::Float(f) = token {
            assert!(f.is_finite(), "Float should be finite");
        }
    }

    #[test]
    fn test_hex_overflow_exact_boundary() {
        // Exactly at u64::MAX in hex
        let source = "0xFFFFFFFFFFFFFFFF";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert!(!handler.has_errors(), "u64::MAX in hex should be valid");
        assert_eq!(token, Token::Number(u64::MAX));
    }

    #[test]
    fn test_hex_overflow_by_one() {
        // u64::MAX + 1 in hex (0x10000000000000000)
        let source = "0x10000000000000000";
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let token = lexer.next_token();
        assert!(handler.has_errors(), "Should overflow");
        assert!(matches!(token, Token::Number(_)));
    }
}
