//! faxc-lex - Lexical Analyzer (Lexer/Tokenizer)
//!
//! ============================================================================
//! LEXICAL ANALYSIS THEORY
//! ============================================================================
//!
//! Lexical analysis is the first phase of compilation. It transforms a stream
//! of characters into a stream of tokens. This process is also called
//! "tokenization" or "scanning".
//!
//! FORMAL DEFINITION:
//! ------------------
//! Let Σ be the alphabet (set of all valid characters).
//! Let Σ* be the set of all strings over Σ.
//!
//! The lexer is a function:
//!   L: Σ* → T*
//! where T is the set of tokens.
//!
//! PROPERTIES:
//! -----------
//! - Linear time complexity: O(n) where n = input length
//! - Single-pass processing (usually)
//! - Context-free (doesn't consider surrounding tokens)
//!
//! LEXEME vs TOKEN:
//! ----------------
//! - Lexeme: The actual string of characters (e.g., "let", "123", "+")
//! - Token: The abstract category + metadata (e.g., Token::Let, Token::Number(123))
//!
//! Example:
//! ```
//! Source: "let x = 42;"
//!
//! Lexemes:  "let", " ", "x", " ", "=", " ", "42", ";"
//! Tokens:   [Let] [Ident("x")] [Eq] [Number(42)] [Semicolon] [Eof]
//!           ↑ skipping whitespace
//! ```
//!
//! ============================================================================
//! TOKEN CATEGORIES
//! ============================================================================
//!
//! 1. KEYWORDS (Reserved Words)
//!    Words with special meaning in the language.
//!    Cannot be used as identifiers.
//!
//! 2. IDENTIFIERS
//!    Names chosen by programmers for variables, functions, types, etc.
//!    Pattern: [a-zA-Z_][a-zA-Z0-9_]*
//!
//! 3. LITERALS
//!    Represent constant values:
//!    - Integer: 42, 0xFF, 0b1010, 0o77
//!    - Float: 3.14, 1e10, 2.5e-3
//!    - String: "hello", "world\n"
//!    - Boolean: true, false
//!
//! 4. OPERATORS
//!    Symbols representing operations:
//!    - Arithmetic: +, -, *, /, %
//!    - Comparison: ==, !=, <, >, <=, >=
//!    - Logical: &&, ||, !
//!    - Assignment: =, +=, -=, etc.
//!
//! 5. ASYNC/AWAIT
//!    Keywords for asynchronous programming:
//!    - async: Marks function or block as async
//!    - await: Suspends execution until future completes
//!
//! 6. PUNCTUATORS/DELIMITERS
//!    Structural symbols:
//!    - Grouping: (), {}, []
//!    - Separation: ,, ;, :
//!    - Access: ., ::, ->
//!
//! 6. SPECIAL
//!    - Whitespace (usually skipped)
//!    - Comments (usually skipped)
//!    - EOF (End of File marker)
//!
//! ============================================================================
//! LEXER IMPLEMENTATION TECHNIQUES
//! ============================================================================
//!
//! TECHNIQUE 1: TABLE-DRIVEN (Finite State Machine)
//! ------------------------------------------------
//! Use a state transition table based on current state and input character.
//!
//! States:
//! - S0: Start state
//! - S1: Reading identifier
//! - S2: Reading number
//! - S3: Reading string
//! - S4: Reading comment
//! - S_accept: Accepting state (emit token)
//! - S_error: Error state
//!
//! Transition Table Example (simplified):
//! ```
//!         letter  digit   "     /     *     other
//! S0      S1      S2      S3    S4    -     error
//! S1      S1      S1      -     -     -     accept(ID)
//! S2      -       S2      -     -     -     accept(NUM)
//! ...
//! ```
//!
//! ADVANTAGES:
//! - Fast execution (table lookup)
//! - Easy to modify
//! - Compact representation
//!
//! DISADVANTAGES:
//! - Large table for complex languages
//! - Harder to add context-sensitive features
//!
//! TECHNIQUE 2: DIRECT-CODED (Recursive Functions)
//! -----------------------------------------------
//! Each token type has its own parsing function.
//! This is what we use in this implementation.
//!
//! Pattern:
//! ```
//! fn lex_identifier(&mut self) -> Token {
//!     while self.is_alphanumeric() {
//!         self.advance();
//!     }
//!     let text = self.current_text();
//!     self.keyword_or_ident(text)
//! }
//! ```
//!
//! ADVANTAGES:
//! - Easy to understand and debug
//! - Flexible (can add complex logic)
//! - Good error messages
//!
//! DISADVANTAGES:
//! - More code to write
//! - Slightly slower than table-driven
//!
//! TECHNIQUE 3: REGEX-BASED
//! ------------------------
//! Define tokens as regular expressions, use regex engine.
//!
//! Example:
//! ```
//! IDENTIFIER = /[a-zA-Z_][a-zA-Z0-9_]*/
//! NUMBER = /[0-9]+/
//! STRING = /"([^"]*)"/
//! ```
//!
//! ADVANTAGES:
//! - Very concise specification
//! - Well-understood theory
//! - Automatic lexer generators available
//!
//! DISADVANTAGES:
//! - Slower than hand-written
//! - Harder to customize
//! - Limited context handling
//!
//! ============================================================================
//! NUMBER LITERAL PARSING
//! ============================================================================
//!
//! INTEGER FORMATS:
//! ----------------
//! - Decimal: 123, 0, 456
//! - Hexadecimal: 0xFF, 0xAB_CD (with separators)
//! - Binary: 0b1010, 0b1111_0000
//! - Octal: 0o777
//!
//! Parsing Algorithm:
//! ```
//! parse_number():
//!   base = 10
//!   if current == '0':
//!     advance()
//!     if current == 'x': base = 16
//!     else if current == 'b': base = 2
//!     else if current == 'o': base = 8
//!     else: // just 0
//!   
//!   value = 0
//!   while is_digit(current, base):
//!     value = value * base + digit_value(current)
//!     advance()
//!   
//!   return Token::Number(value)
//! ```
//!
//! FLOATING POINT:
//! ---------------
//! Format: [digits].[digits][(e|E)[(+|-)]digits]
//!
//! Examples:
//! - 3.14
//! - 1.0e10
//! - 2.5E-3
//! - .5 (optional leading digits)
//! - 5. (optional trailing digits)
//!
//! Parsing must handle:
//! - Decimal point position
//! - Exponent
//! - Overflow/underflow
//! - Special values (NaN, Infinity - if supported)
//!
//! ============================================================================
//! STRING LITERAL PARSING
//! ============================================================================
//!
//! ESCAPE SEQUENCES:
//! -----------------
//! \\n - Newline (0x0A)
//! \\t - Tab (0x09)
//! \\r - Carriage return (0x0D)
//! \\\\ - Backslash
//! \\" - Double quote
//! \\0 - Null (0x00)
//! \\xNN - Hex byte (e.g., \\xFF)
//! \\u{NNNN} - Unicode codepoint (e.g., \\u{1F600})
//!
//! ALGORITHM:
//! ----------
//! ```
//! parse_string():
//!   expect('"')
//!   result = ""
//!   
//!   while current != '"' and not eof:
//!     if current == '\\':
//!       advance()
//!       result += parse_escape()
//!     else:
//!       result += current
//!       advance()
//!   
//!   expect('"')
//!   return Token::String(result)
//! ```
//!
//! MULTILINE STRINGS:
//! ------------------
//! Some languages support multiline strings:
//! - Heredocs: <<<END ... END
//! - Triple quotes: """ ... """
//! - Raw strings: r"..." (no escape processing)
//!
//! ============================================================================
//! COMMENT HANDLING
//! ============================================================================
//!
//! LINE COMMENTS:
//! --------------
//! Format: // comment until end of line
//!
//! Handling:
//! - Skip //
//! - Skip all characters until \\n or EOF
//! - Do not emit token (completely ignored)
//!
//! BLOCK COMMENTS:
//! ---------------
//! Format: /* comment */
//!
//! Handling:
//! - Skip /*
//! - Skip characters until */ found
//! - Handle nesting if language supports it
//!
//! NESTED BLOCK COMMENTS:
//! ----------------------
//! Some languages (Rust, D) support nesting:
//! /* outer /* inner */ still outer */
//!
//! Requires counter:
//! ```
//! depth = 1
//! while depth > 0:
//!   if next == "/*": depth++
//!   if next == "*/": depth--
//! ```
//!
//! DOC COMMENTS:
//! -------------
//! Special comments for documentation:
//! - /// Line doc comment
//! - /** Block doc comment */
//!
//! Preserved and attached to following item.
//!
//! ============================================================================
//! ERROR RECOVERY STRATEGIES
//! ============================================================================
//!
//! STRATEGY 1: PANIC MODE (Skip until sync point)
//! ----------------------------------------------
//! When encountering invalid character:
//! 1. Report error
//! 2. Skip character
//! 3. Continue lexing
//!
//! Example:
//! ```
//! Source: let @x = 5;
//!            ↑ invalid
//!
//! Error: "unexpected character '@'"
//! Recovery: Skip '@', continue with 'x'
//! Result: [Let] [Error] [Ident("x")] [Eq] [Number(5)] [Semicolon]
//! ```
//!
//! STRATEGY 2: INSERT MISSING TOKEN
//! --------------------------------
//! If missing expected character, pretend it was there.
//!
//! Example:
//! ```
//! Source: "hello
//!         ↑ unclosed string
//!
//! Error: "unterminated string literal"
//! Recovery: Insert closing quote at line end
//! ```
//!
//! STRATEGY 3: SUBSTITUTE CHARACTER
//! --------------------------------
//! Replace invalid character with valid one.
//!
//! Example:
//! ```
//! Source: 'ab'  // Multiple characters in char literal
//!
//! Error: "character literal may only contain one character"
//! Recovery: Treat as "a" (first char)
//! ```
//!
//! ============================================================================
//! PERFORMANCE OPTIMIZATIONS
//! ============================================================================
//!
//! 1. TABLE-LOOKUP FOR CHARACTER CLASSIFICATION
//! --------------------------------------------
//! Precompute table [0-255] → character class
//! ```
//! enum CharClass {
//!   Whitespace,  // ' ', \\t, \\n, etc.
//!   Letter,      // a-z, A-Z
//!   Digit,       // 0-9
//!   Underscore,  // _
//!   Quote,       // '"
//!   Slash,       // /
//!   Other,
//! }
//! ```
//!
//! 2. SIMD ACCELERATION
//! --------------------
//! Use SIMD instructions for:
//! - Finding end of whitespace
//! - Finding newline
//! - Checking ASCII validity
//!
//! 3. ZERO-COPY STRINGS
//! --------------------
//! Slice into source buffer instead of allocating.
//! Only copy when necessary (escape processing).
//!
//! 4. KEYWORD PERFECT HASH
//! -----------------------
//! Use perfect hash function for keyword lookup O(1).
//!
//! ============================================================================
//! UNICODE HANDLING
//! ============================================================================
//!
//! UTF-8 DECODING:
//! ---------------
//! Source code is typically UTF-8 encoded.
//!
//! Valid identifier characters:
//! - ASCII: [a-zA-Z0-9_]
//! - Unicode: XID_Start and XID_Continue properties
//!
//! XID_Start: Characters that can start an identifier
//! XID_Continue: Characters that can continue an identifier
//!
//! Example valid Unicode identifiers:
//! - 変数 (Japanese)
//! - переменная (Russian)
//! - αβγ (Greek)
//!
//! BOM (Byte Order Mark):
//! ----------------------
//! UTF-8 BOM: EF BB BF
//! Should be skipped at file start.
//!
//! ============================================================================
//! SOURCE LOCATION TRACKING
//! ============================================================================
//!
//! For error reporting, lexer must track:
//! - Byte offset (0-based position in file)
//! - Line number (1-based)
//! - Column number (1-based, grapheme-based)
//!
//! SPAN:
//! -----
//! Range of source code: start offset..end offset
//!
//! ```
//! Source: "let x = 5;"
//!          0123456789
//!
//! Token 'let': Span { start: 0, end: 3 }
//! Token 'x':   Span { start: 4, end: 5 }
//! Token '5':   Span { start: 8, end: 9 }
//! ```
//!
//! LINE TABLE:
//! -----------
//! Store offset of each line start for fast line/column lookup.
//! ```
//! Line 1: offset 0
//! Line 2: offset 42
//! Line 3: offset 85
//! ...
//! ```
//!
//! To find line of position p:
//! - Binary search line table for largest offset ≤ p

use faxc_util::{Diagnostic, Handler, Level, Symbol};

/// Token represents a lexical unit in the source code
///
/// Each variant contains all necessary information about the token,
/// including its kind and any associated data (e.g., identifier name).
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    // =========================================================================
    // KEYWORDS
    // =========================================================================
    // Keywords are reserved words that have special meaning in the language.
    // They cannot be used as identifiers.
    /// "let" - Variable binding keyword
    ///
    /// Usage: let x = 5;
    Let,

    /// "fn" - Function declaration keyword
    ///
    /// Usage: fn main() { ... }
    Fn,

    /// "if" - Conditional keyword
    ///
    /// Usage: if condition { ... }
    If,

    /// "else" - Alternative branch keyword
    ///
    /// Usage: if cond { a } else { b }
    Else,

    /// "while" - Loop keyword
    ///
    /// Usage: while condition { ... }
    While,

    /// "for" - Iterator loop keyword
    ///
    /// Usage: for x in iterable { ... }
    For,

    /// "return" - Function return keyword
    ///
    /// Usage: return value;
    Return,

    /// "struct" - Structure definition keyword
    ///
    /// Usage: struct Point { x: int, y: int }
    Struct,

    /// "enum" - Enumeration definition keyword
    ///
    /// Usage: enum Color { Red, Green, Blue }
    Enum,

    /// "impl" - Implementation block keyword
    ///
    /// Usage: impl Trait for Type { ... }
    Impl,

    /// "trait" - Trait definition keyword
    ///
    /// Usage: trait Printable { fn print(&self); }
    Trait,

    /// "pub" - Visibility modifier
    ///
    /// Usage: pub fn public_function() { }
    Pub,

    /// "mut" - Mutability modifier
    ///
    /// Usage: let mut x = 5;
    Mut,

    /// "match" - Pattern matching keyword
    ///
    /// Usage: match value { pattern => expr }
    Match,

    /// "true" - Boolean literal
    True,

    /// "false" - Boolean literal
    False,

    /// "async" - Async function/block keyword
    ///
    /// Usage: async fn foo() { ... } or async { ... }
    Async,

    /// "await" - Await expression keyword
    ///
    /// Usage: let x = await future;
    Await,

    /// "macro_rules" - Macro definition keyword
    ///
    /// Usage: macro_rules! name { ... }
    MacroRules,

    // =========================================================================
    // MACRO-RELATED TOKENS
    // =========================================================================
    /// "!" - Macro invocation or logical NOT
    ///
    /// Usage: println!("hello"), vec![1, 2, 3]
    Bang,

    /// "$" - Macro metavariable prefix
    ///
    /// Usage: $expr, $ident, $ty in macro patterns
    Dollar,

    // =========================================================================
    // IDENTIFIERS
    // =========================================================================
    /// Identifier (variable name, function name, type name, etc.)
    ///
    /// The Symbol contains the interned string representation.
    Ident(Symbol),

    // =========================================================================
    // LITERALS
    // =========================================================================
    /// Integer literal
    ///
    /// Can be decimal, hexadecimal (0x), binary (0b), or octal (0o).
    /// Examples: 42, 0xFF, 0b1010, 0o777
    Number(u64),

    /// Floating point literal
    ///
    /// Examples: 3.14, 1e10, 2.5e-3
    Float(f64),

    /// String literal
    ///
    /// The Symbol contains the interned string value (with escapes processed).
    String(Symbol),

    // =========================================================================
    // ARITHMETIC OPERATORS
    // =========================================================================
    /// "+" - Addition
    Plus,

    /// "-" - Subtraction or negation
    Minus,

    /// "*" - Multiplication
    Star,

    /// "/" - Division
    Slash,

    /// "%" - Modulo/remainder
    Percent,

    // =========================================================================
    // COMPARISON OPERATORS
    // =========================================================================
    /// "==" - Equality
    EqEq,

    /// "!=" - Inequality
    NotEq,

    /// "<" - Less than
    Lt,

    /// ">" - Greater than
    Gt,

    /// "<=" - Less than or equal
    LtEq,

    /// ">=" - Greater than or equal
    GtEq,

    // =========================================================================
    // LOGICAL OPERATORS
    // =========================================================================
    /// "&&" - Logical AND
    AndAnd,

    /// "||" - Logical OR
    OrOr,

    /// "!" - Logical NOT
    Not,

    // =========================================================================
    // ASSIGNMENT OPERATORS
    // =========================================================================
    /// "=" - Assignment
    Eq,

    /// "+=" - Add and assign
    PlusEq,

    /// "-=" - Subtract and assign
    MinusEq,

    /// "*=" - Multiply and assign
    StarEq,

    /// "/=" - Divide and assign
    SlashEq,

    /// "%=" - Modulo and assign
    PercentEq,

    // =========================================================================
    // PUNCTUATORS
    // =========================================================================
    /// "(" - Left parenthesis
    LParen,

    /// ")" - Right parenthesis
    RParen,

    /// "{" - Left brace
    LBrace,

    /// "}" - Right brace
    RBrace,

    /// "[" - Left bracket
    LBracket,

    /// "]" - Right bracket
    RBracket,

    /// "," - Comma
    Comma,

    /// ";" - Semicolon
    Semicolon,

    /// ":" - Colon
    Colon,

    /// "::" - Double colon (path separator)
    ColonColon,

    /// "->" - Arrow (function return type)
    Arrow,

    /// "=>" - Fat arrow (match arm, closure)
    FatArrow,

    /// "." - Dot (field access, method call)
    Dot,

    /// ".." - Double dot (range)
    DotDot,

    /// "..." - Triple dot (variadic, spread)
    DotDotDot,

    /// "&" - Ampersand (reference, bitwise AND)
    Ampersand,

    /// "|" - Pipe (bitwise OR, closure parameter)
    Pipe,

    /// "@" - At (pattern binding)
    At,

    // =========================================================================
    // SPECIAL
    // =========================================================================
    /// End of file marker
    ///
    /// Signals that the entire input has been processed.
    Eof,

    /// Invalid/unrecognized token
    ///
    /// Used for error recovery. Contains the invalid text.
    Invalid(String),
}

/// Lexer state machine
///
/// The lexer maintains a cursor position in the source text and
/// progressively scans tokens on demand.
pub struct Lexer<'source> {
    /// Source text being lexed
    source: &'source str,

    /// Current byte position in source
    position: usize,

    /// Start position of current token (for error reporting)
    token_start: usize,

    /// Current line number (1-based)
    line: u32,

    /// Current column number (1-based, in bytes)
    column: u32,

    /// Diagnostic handler for error reporting
    handler: &'source mut Handler,
}

impl<'source> Lexer<'source> {
    /// Create a new lexer for the given source text
    ///
    /// # Arguments
    ///
    /// * `source` - The source code to lex
    /// * `handler` - Error handler for reporting lexical errors
    ///
    /// # Example
    ///
    /// ```
    /// let mut handler = Handler::new();
    /// let lexer = Lexer::new("let x = 5;", &mut handler);
    /// ```
    pub fn new(source: &'source str, handler: &'source mut Handler) -> Self {
        Self {
            source,
            position: 0,
            token_start: 0,
            line: 1,
            column: 1,
            handler,
        }
    }

    /// Get the next token from the input
    ///
    /// This is the main entry point for tokenization. It skips whitespace
    /// and comments, then dispatches to the appropriate lexer function
    /// based on the first character.
    ///
    /// # Returns
    ///
    /// The next token, or Token::Eof if at end of input.
    ///
    /// # Algorithm
    ///
    /// 1. Skip whitespace and comments
    /// 2. Record start position
    /// 3. Match on current character to determine token type
    /// 4. Dispatch to specialized lexer function
    /// 5. Return completed token
    pub fn next_token(&mut self) -> Token {
        // Skip insignificant characters (whitespace, comments)
        self.skip_whitespace_and_comments();

        // Record start position of this token
        self.token_start = self.position;

        // Check for end of file
        if self.is_at_end() {
            return Token::Eof;
        }

        // Dispatch based on first character
        match self.current_char() {
            // Single-character tokens
            '(' => {
                self.advance();
                Token::LParen
            }
            ')' => {
                self.advance();
                Token::RParen
            }
            '{' => {
                self.advance();
                Token::LBrace
            }
            '}' => {
                self.advance();
                Token::RBrace
            }
            '[' => {
                self.advance();
                Token::LBracket
            }
            ']' => {
                self.advance();
                Token::RBracket
            }
            ',' => {
                self.advance();
                Token::Comma
            }
            ';' => {
                self.advance();
                Token::Semicolon
            }
            '+' => self.lex_plus(),
            '-' => self.lex_minus(),
            '*' => {
                self.advance();
                Token::Star
            }
            '/' => self.lex_slash(), // Could be /, //, or /*
            '%' => {
                self.advance();
                Token::Percent
            }
            '=' => self.lex_equals(),
            '!' => self.lex_bang(),
            '<' => self.lex_less(),
            '>' => self.lex_greater(),
            '&' => self.lex_ampersand(),
            '|' => self.lex_pipe(),
            ':' => self.lex_colon(),
            '.' => self.lex_dot(),
            '"' => self.lex_string(),
            '\'' => self.lex_char(),

            '$' => {
                self.advance();
                Token::Dollar
            }

            // Whitespace should have been skipped
            c if c.is_whitespace() => {
                panic!("Unexpected whitespace at {}:{}", self.line, self.column)
            }

            // Identifiers and keywords
            c if c.is_ascii_alphabetic() || c == '_' => self.lex_identifier(),

            // Numbers
            c if c.is_ascii_digit() => self.lex_number(),

            // Unknown character
            c => {
                self.report_error(format!("unexpected character '{}'", c));
                self.advance();
                Token::Invalid(c.to_string())
            }
        }
    }

    /// Lex an identifier or keyword
    ///
    /// Identifiers start with letter or underscore, followed by alphanumeric
    /// or underscore characters.
    fn lex_identifier(&mut self) -> Token {
        unimplemented!("Identifier lexing not implemented")
    }

    /// Lex a number literal
    ///
    /// Handles decimal, hexadecimal (0x), binary (0b), and octal (0o) formats.
    fn lex_number(&mut self) -> Token {
        unimplemented!("Number lexing not implemented")
    }

    /// Lex a string literal
    fn lex_string(&mut self) -> Token {
        unimplemented!("String lexing not implemented")
    }

    /// Lex a character literal
    fn lex_char(&mut self) -> Token {
        unimplemented!("Char lexing not implemented")
    }

    /// Lex plus or plus-equals
    fn lex_plus(&mut self) -> Token {
        self.advance();
        if self.match_char('=') {
            Token::PlusEq
        } else {
            Token::Plus
        }
    }

    /// Lex minus, arrow, or minus-equals
    fn lex_minus(&mut self) -> Token {
        unimplemented!("Minus lexing not implemented")
    }

    /// Lex slash, comment start, or slash-equals
    fn lex_slash(&mut self) -> Token {
        unimplemented!("Slash lexing not implemented")
    }

    /// Lex equals or equals-equals
    fn lex_equals(&mut self) -> Token {
        unimplemented!("Equals lexing not implemented")
    }

    /// Lex bang or not-equals
    fn lex_bang(&mut self) -> Token {
        unimplemented!("Bang lexing not implemented")
    }

    /// Lex less, less-equals, or left shift
    fn lex_less(&mut self) -> Token {
        unimplemented!("Less lexing not implemented")
    }

    /// Lex greater, greater-equals, or right shift
    fn lex_greater(&mut self) -> Token {
        unimplemented!("Greater lexing not implemented")
    }

    /// Lex ampersand or logical and
    fn lex_ampersand(&mut self) -> Token {
        unimplemented!("Ampersand lexing not implemented")
    }

    /// Lex pipe or logical or
    fn lex_pipe(&mut self) -> Token {
        unimplemented!("Pipe lexing not implemented")
    }

    /// Lex colon or double colon
    fn lex_colon(&mut self) -> Token {
        unimplemented!("Colon lexing not implemented")
    }

    /// Lex dot, double dot, or triple dot
    fn lex_dot(&mut self) -> Token {
        unimplemented!("Dot lexing not implemented")
    }

    /// Skip whitespace and comments
    fn skip_whitespace_and_comments(&mut self) {
        unimplemented!("Whitespace skipping not implemented")
    }

    /// Get current character
    fn current_char(&self) -> char {
        self.source[self.position..].chars().next().unwrap_or('\0')
    }

    /// Advance to next character
    fn advance(&mut self) {
        if let Some(c) = self.source[self.position..].chars().next() {
            self.position += c.len_utf8();
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
    }

    /// Check if at end of source
    fn is_at_end(&self) -> bool {
        self.position >= self.source.len()
    }

    /// Match and consume expected character
    fn match_char(&mut self, expected: char) -> bool {
        if self.current_char() == expected {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Report a lexical error
    fn report_error(&mut self, message: String) {
        // Create diagnostic with current location
        unimplemented!("Error reporting not implemented")
    }
}

/// Make Lexer an iterator over tokens
impl<'source> Iterator for Lexer<'source> {
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
