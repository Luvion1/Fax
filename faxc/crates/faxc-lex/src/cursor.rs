//! Character cursor for traversing source code.
//!
//! This module provides the `Cursor` struct which maintains position state
//! while iterating through source code characters. It handles UTF-8 encoding
//! correctly and tracks line/column information for error reporting.

/// A cursor for traversing source code character by character.
///
/// The cursor maintains the current position in the source string and
/// provides methods for advancing, peeking ahead, and checking conditions.
/// It correctly handles UTF-8 encoded text and tracks line/column numbers.
///
/// # Example
///
/// ```
/// use faxc_lex::cursor::Cursor;
///
/// let source = "let x = 42;";
/// let mut cursor = Cursor::new(source);
///
/// assert_eq!(cursor.current_char(), 'l');
/// cursor.advance();
/// assert_eq!(cursor.current_char(), 'e');
/// ```
pub struct Cursor<'a> {
    /// The source text being traversed.
    source: &'a str,

    /// Current byte position in the source.
    position: usize,

    /// Current line number (1-based).
    line: u32,

    /// Current column number (1-based, in characters).
    column: u32,
}

impl<'a> Cursor<'a> {
    /// Creates a new cursor for the given source text.
    ///
    /// # Arguments
    ///
    /// * `source` - The source code to traverse
    ///
    /// # Example
    ///
    /// ```
    /// use faxc_lex::cursor::Cursor;
    ///
    /// let source = "let x = 42;";
    /// let cursor = Cursor::new(source);
    /// ```
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            position: 0,
            line: 1,
            column: 1,
        }
    }

    /// Returns the current character at the cursor position.
    ///
    /// Returns '\0' (null character) if at the end of the source.
    ///
    /// # Example
    ///
    /// ```
    /// use faxc_lex::cursor::Cursor;
    ///
    /// let mut cursor = Cursor::new("abc");
    /// assert_eq!(cursor.current_char(), 'a');
    /// ```
    pub fn current_char(&self) -> char {
        self.char_at(0)
    }

    /// Returns the character at the given byte offset from current position.
    /// This is more efficient than peek_char for small offsets.
    ///
    /// # Arguments
    ///
    /// * `offset` - Number of bytes to look ahead
    #[inline]
    pub fn char_at(&self, offset: usize) -> char {
        let pos = self.position + offset;
        if pos >= self.source.len() {
            return '\0';
        }

        // Fast path for ASCII (most common case)
        let b = self.source.as_bytes()[pos];
        if b < 128 {
            return b as char;
        }

        // Slow path for UTF-8
        self.source[pos..].chars().next().unwrap_or('\0')
    }

    /// Returns the character at the given offset from the current position.
    ///
    /// # Arguments
    ///
    /// * `offset` - Number of characters to look ahead (0 = current)
    ///
    /// # Example
    ///
    /// ```
    /// use faxc_lex::cursor::Cursor;
    ///
    /// let cursor = Cursor::new("abc");
    /// assert_eq!(cursor.peek_char(0), 'a');
    /// assert_eq!(cursor.peek_char(1), 'b');
    /// assert_eq!(cursor.peek_char(2), 'c');
    /// assert_eq!(cursor.peek_char(3), '\0');
    /// ```
    #[inline]
    pub fn peek_char(&self, offset: usize) -> char {
        self.char_at(offset)
    }

    /// Returns true if the current character is ASCII.
    #[inline]
    pub fn is_ascii(&self) -> bool {
        if self.position >= self.source.len() {
            return true; // Treat end as ASCII for simplicity
        }
        self.source.as_bytes()[self.position] < 128
    }

    /// Returns the current character as ASCII byte, or None if not ASCII or at end.
    #[inline]
    pub fn current_byte(&self) -> Option<u8> {
        if self.position >= self.source.len() {
            return None;
        }
        let b = self.source.as_bytes()[self.position];
        if b < 128 {
            Some(b)
        } else {
            None
        }
    }

    /// Peeks at the next character as ASCII byte.
    #[inline]
    pub fn peek_byte(&self, offset: usize) -> Option<u8> {
        let pos = self.position + offset;
        if pos >= self.source.len() {
            return None;
        }
        let b = self.source.as_bytes()[pos];
        if b < 128 {
            Some(b)
        } else {
            None
        }
    }

    /// Advances the cursor to the next character.
    ///
    /// Updates line and column tracking. Does nothing if already at end.
    ///
    /// # Example
    ///
    /// ```
    /// use faxc_lex::cursor::Cursor;
    ///
    /// let mut cursor = Cursor::new("ab");
    /// assert_eq!(cursor.current_char(), 'a');
    /// cursor.advance();
    /// assert_eq!(cursor.current_char(), 'b');
    /// ```
    #[inline]
    pub fn advance(&mut self) {
        if self.position >= self.source.len() {
            return;
        }

        // Fast path for ASCII (most common)
        let b = self.source.as_bytes()[self.position];
        if b < 128 {
            self.position += 1;
            if b == b'\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            return;
        }

        // Slow path for UTF-8 multi-byte characters
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

    /// Advances the cursor by the given number of characters.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of characters to advance
    ///
    /// # Example
    ///
    /// ```
    /// use faxc_lex::cursor::Cursor;
    ///
    /// let mut cursor = Cursor::new("abcdef");
    /// cursor.advance_n(3);
    /// assert_eq!(cursor.current_char(), 'd');
    /// ```
    pub fn advance_n(&mut self, count: usize) {
        for _ in 0..count {
            if self.is_at_end() {
                break;
            }
            self.advance();
        }
    }

    /// Advances by specified byte count (more efficient for ASCII).
    #[inline]
    pub fn advance_bytes(&mut self, count: usize) {
        let remaining = self.source.len() - self.position;
        let advance = count.min(remaining);

        // Count newlines in the advanced portion for line tracking
        let start = self.position;
        let end = self.position + advance;
        for i in start..end {
            if self.source.as_bytes()[i] == b'\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }

        self.position += advance;
    }

    /// Returns true if the cursor is at the end of the source.
    ///
    /// # Example
    ///
    /// ```
    /// use faxc_lex::cursor::Cursor;
    ///
    /// let mut cursor = Cursor::new("a");
    /// assert!(!cursor.is_at_end());
    /// cursor.advance();
    /// assert!(cursor.is_at_end());
    /// ```
    pub fn is_at_end(&self) -> bool {
        self.position >= self.source.len()
    }

    /// Matches and consumes the expected character if present.
    ///
    /// Returns true if the character was matched and consumed, false otherwise.
    ///
    /// # Arguments
    ///
    /// * `expected` - The character to match
    ///
    /// # Example
    ///
    /// ```
    /// use faxc_lex::cursor::Cursor;
    ///
    /// let mut cursor = Cursor::new("=>");
    /// assert!(cursor.match_char('='));
    /// assert!(!cursor.match_char('='));
    /// assert_eq!(cursor.current_char(), '>');
    /// ```
    pub fn match_char(&mut self, expected: char) -> bool {
        if self.current_char() == expected {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Skips whitespace characters.
    ///
    /// This method advances the cursor past all consecutive whitespace
    /// characters including Unicode whitespace (not just ASCII spaces, tabs, newlines).
    /// It does NOT skip comments.
    ///
    /// # Example
    ///
    /// ```
    /// use faxc_lex::cursor::Cursor;
    ///
    /// let mut cursor = Cursor::new("  \t\nlet");
    /// cursor.skip_whitespace();
    /// assert_eq!(cursor.current_char(), 'l');
    /// ```
    pub fn skip_whitespace(&mut self) {
        while !self.is_at_end() && self.current_char().is_whitespace() {
            self.advance();
        }
    }

    /// Returns the current line number (1-based).
    ///
    /// # Example
    ///
    /// ```
    /// use faxc_lex::cursor::Cursor;
    ///
    /// let mut cursor = Cursor::new("line1\nline2");
    /// assert_eq!(cursor.line(), 1);
    /// cursor.advance_n(6); // Skip "line1\n"
    /// assert_eq!(cursor.line(), 2);
    /// ```
    pub fn line(&self) -> u32 {
        self.line
    }

    /// Returns the current column number (1-based).
    ///
    /// # Example
    ///
    /// ```
    /// use faxc_lex::cursor::Cursor;
    ///
    /// let mut cursor = Cursor::new("abc");
    /// assert_eq!(cursor.column(), 1);
    /// cursor.advance();
    /// assert_eq!(cursor.column(), 2);
    /// ```
    pub fn column(&self) -> u32 {
        self.column
    }

    /// Returns the current byte position in the source.
    ///
    /// # Example
    ///
    /// ```
    /// use faxc_lex::cursor::Cursor;
    ///
    /// let mut cursor = Cursor::new("abc");
    /// assert_eq!(cursor.position(), 0);
    /// cursor.advance();
    /// assert_eq!(cursor.position(), 1);
    /// ```
    pub fn position(&self) -> usize {
        self.position
    }

    /// Returns a slice of the source from the given start position to the
    /// current position.
    ///
    /// # Arguments
    ///
    /// * `start` - Starting byte position (inclusive)
    ///
    /// # Example
    ///
    /// ```
    /// use faxc_lex::cursor::Cursor;
    ///
    /// let mut cursor = Cursor::new("let x");
    /// let start = cursor.position();
    /// cursor.advance_n(3);
    /// assert_eq!(cursor.slice_from(start), "let");
    /// ```
    pub fn slice_from(&self, start: usize) -> &'a str {
        &self.source[start..self.position]
    }

    /// Returns the source text from the current position to the end.
    ///
    /// # Example
    ///
    /// ```
    /// use faxc_lex::cursor::Cursor;
    ///
    /// let mut cursor = Cursor::new("let x = 42;");
    /// cursor.advance_n(4); // Skip "let "
    /// assert_eq!(cursor.remaining(), "x = 42;");
    /// ```
    pub fn remaining(&self) -> &'a str {
        &self.source[self.position..]
    }

    /// Returns the full source text.
    ///
    /// # Example
    ///
    /// ```
    /// use faxc_lex::cursor::Cursor;
    ///
    /// let cursor = Cursor::new("let x = 42;");
    /// assert_eq!(cursor.source(), "let x = 42;");
    /// ```
    pub fn source(&self) -> &'a str {
        self.source
    }

    /// Creates a snapshot of the current cursor state.
    ///
    /// This can be used to save position and restore it later.
    ///
    /// # Example
    ///
    /// ```
    /// use faxc_lex::cursor::Cursor;
    ///
    /// let mut cursor = Cursor::new("let x = 42;");
    /// let snapshot = cursor.snapshot();
    /// cursor.advance_n(3);
    /// cursor.restore(snapshot);
    /// assert_eq!(cursor.current_char(), 'l');
    /// ```
    pub fn snapshot(&self) -> CursorSnapshot {
        CursorSnapshot {
            position: self.position,
            line: self.line,
            column: self.column,
        }
    }

    /// Restores the cursor to a previously saved snapshot.
    ///
    /// # Arguments
    ///
    /// * `snapshot` - The snapshot to restore
    ///
    /// # Example
    ///
    /// ```
    /// use faxc_lex::cursor::Cursor;
    ///
    /// let mut cursor = Cursor::new("let x = 42;");
    /// let snapshot = cursor.snapshot();
    /// cursor.advance_n(3);
    /// cursor.restore(snapshot);
    /// assert_eq!(cursor.position(), 0);
    /// ```
    pub fn restore(&mut self, snapshot: CursorSnapshot) {
        self.position = snapshot.position;
        self.line = snapshot.line;
        self.column = snapshot.column;
    }
}

/// A snapshot of cursor state that can be restored later.
#[derive(Clone, Copy, Debug)]
pub struct CursorSnapshot {
    /// Byte position in source.
    pub position: usize,
    /// Line number (1-based).
    pub line: u32,
    /// Column number (1-based).
    pub column: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cursor() {
        let cursor = Cursor::new("let x = 42;");
        assert_eq!(cursor.current_char(), 'l');
        assert_eq!(cursor.position(), 0);
        assert_eq!(cursor.line(), 1);
        assert_eq!(cursor.column(), 1);
    }

    #[test]
    fn test_advance() {
        let mut cursor = Cursor::new("abc");
        assert_eq!(cursor.current_char(), 'a');
        cursor.advance();
        assert_eq!(cursor.current_char(), 'b');
        cursor.advance();
        assert_eq!(cursor.current_char(), 'c');
        cursor.advance();
        assert_eq!(cursor.current_char(), '\0');
    }

    #[test]
    fn test_advance_utf8() {
        let mut cursor = Cursor::new("αβγ");
        assert_eq!(cursor.current_char(), 'α');
        cursor.advance();
        assert_eq!(cursor.current_char(), 'β');
        cursor.advance();
        assert_eq!(cursor.current_char(), 'γ');
    }

    #[test]
    fn test_peek_char() {
        let cursor = Cursor::new("abc");
        assert_eq!(cursor.peek_char(0), 'a');
        assert_eq!(cursor.peek_char(1), 'b');
        assert_eq!(cursor.peek_char(2), 'c');
        assert_eq!(cursor.peek_char(3), '\0');
        assert_eq!(cursor.peek_char(100), '\0');
    }

    #[test]
    fn test_is_at_end() {
        let mut cursor = Cursor::new("a");
        assert!(!cursor.is_at_end());
        cursor.advance();
        assert!(cursor.is_at_end());
    }

    #[test]
    fn test_match_char() {
        let mut cursor = Cursor::new("=>");
        assert!(cursor.match_char('='));
        assert!(!cursor.match_char('='));
        assert!(cursor.match_char('>'));
        assert!(!cursor.match_char('>'));
    }

    #[test]
    fn test_skip_whitespace() {
        let mut cursor = Cursor::new("  \t\n  let");
        cursor.skip_whitespace();
        assert_eq!(cursor.current_char(), 'l');
    }

    #[test]
    fn test_skip_whitespace_only() {
        let mut cursor = Cursor::new("   ");
        cursor.skip_whitespace();
        assert!(cursor.is_at_end());
    }

    #[test]
    fn test_line_column_tracking() {
        let mut cursor = Cursor::new("line1\nline2\nline3");
        assert_eq!(cursor.line(), 1);
        assert_eq!(cursor.column(), 1);

        cursor.advance_n(5); // "line1"
        assert_eq!(cursor.column(), 6);

        cursor.advance(); // '\n'
        assert_eq!(cursor.line(), 2);
        assert_eq!(cursor.column(), 1);

        cursor.advance_n(5); // "line2"
        cursor.advance(); // '\n'
        assert_eq!(cursor.line(), 3);
        assert_eq!(cursor.column(), 1);
    }

    #[test]
    fn test_slice_from() {
        let mut cursor = Cursor::new("let x = 42;");
        let start = cursor.position();
        cursor.advance_n(3);
        assert_eq!(cursor.slice_from(start), "let");

        let start2 = cursor.position();
        cursor.advance(); // skip space
        cursor.advance(); // 'x'
        assert_eq!(cursor.slice_from(start2), " x");
    }

    #[test]
    fn test_remaining() {
        let mut cursor = Cursor::new("let x = 42;");
        assert_eq!(cursor.remaining(), "let x = 42;");
        cursor.advance_n(4);
        assert_eq!(cursor.remaining(), "x = 42;");
        cursor.advance_n(7);
        assert_eq!(cursor.remaining(), "");
    }

    #[test]
    fn test_snapshot_restore() {
        let mut cursor = Cursor::new("let x = 42;");
        let snapshot = cursor.snapshot();

        cursor.advance_n(6); // "let x "
        assert_eq!(cursor.current_char(), '=');

        cursor.restore(snapshot);
        assert_eq!(cursor.current_char(), 'l');
        assert_eq!(cursor.position(), 0);
        assert_eq!(cursor.line(), 1);
        assert_eq!(cursor.column(), 1);
    }

    #[test]
    fn test_advance_n() {
        let mut cursor = Cursor::new("abcdef");
        cursor.advance_n(3);
        assert_eq!(cursor.current_char(), 'd');
        cursor.advance_n(10); // More than remaining
        assert!(cursor.is_at_end());
    }

    #[test]
    fn test_empty_source() {
        let mut cursor = Cursor::new("");
        assert!(cursor.is_at_end());
        assert_eq!(cursor.current_char(), '\0');
        cursor.advance();
        assert!(cursor.is_at_end());
    }

    #[test]
    fn test_multiline_source() {
        let source = "fn main() {\n    let x = 42;\n}";
        let mut cursor = Cursor::new(source);

        // First line
        assert_eq!(cursor.line(), 1);
        cursor.advance_n(11); // "fn main() {"
        cursor.advance(); // '\n'

        // Second line
        assert_eq!(cursor.line(), 2);
        assert_eq!(cursor.column(), 1);
        cursor.advance_n(4); // "    "
        assert_eq!(cursor.column(), 5);

        cursor.advance_n(9); // "let x = 4"
        cursor.advance(); // '2'
        cursor.advance(); // ';'
        cursor.advance(); // '\n'

        // Third line
        assert_eq!(cursor.line(), 3);
        assert_eq!(cursor.column(), 1);
    }
}
