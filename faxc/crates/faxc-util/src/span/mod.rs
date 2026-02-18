//! Span module - Source location tracking.
//!
//! This module provides types for representing source code locations,
//! including byte offsets, line/column information, and file identification.
//!
//! # Examples
//!
//! ```
//! use faxc_util::span::{Span, FileId};
//!
//! // Create a span at a specific location
//! let span = Span::new(10, 20, 1, 5);
//!
//! // Create a span associated with a specific file
//! let file_id = FileId(0);
//! let span = Span::with_file(10, 20, file_id, 1, 5);
//! ```

mod source_map;

pub use source_map::{SourceFile, SourceMap};

/// A unique identifier for a source file
///
/// FileIds are assigned sequentially as files are added to the [`SourceMap`].
///
/// # Examples
///
/// ```
/// use faxc_util::span::FileId;
///
/// let id = FileId(0);
/// assert_eq!(id.0, 0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileId(pub usize);

impl FileId {
    /// Create a new FileId
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::FileId;
    ///
    /// let id = FileId::new(0);
    /// ```
    #[inline]
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    /// Get the raw index value
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::FileId;
    ///
    /// let id = FileId(0);
    /// assert_eq!(id.index(), 0);
    /// ```
    #[inline]
    pub const fn index(&self) -> usize {
        self.0
    }

    /// A dummy FileId for testing
    pub const DUMMY: FileId = FileId(0);
}

impl Default for FileId {
    #[inline]
    fn default() -> Self {
        Self::DUMMY
    }
}

/// Source location span
///
/// A `Span` represents a range in source code, identified by:
/// - Byte offsets (start, end)
/// - Line and column numbers (for human-readable output)
/// - File ID (for multi-file projects)
///
/// # Examples
///
/// ```
/// use faxc_util::span::Span;
///
/// // Create a span with byte offsets and line/column info
/// let span = Span::new(10, 20, 1, 5);
///
/// // Create a point span (single location)
/// let point = Span::point(1, 5);
///
/// // Create a span associated with a specific file
/// let span = Span::with_file(10, 20, Default::default(), 1, 5);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    /// Start byte offset in source
    pub start: usize,
    /// End byte offset in source
    pub end: usize,
    /// Line number (1-based)
    pub line: u32,
    /// Column number (1-based)
    pub column: u32,
    /// File identifier
    pub file_id: FileId,
}

impl Span {
    /// Dummy span for testing
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::Span;
    ///
    /// assert_eq!(Span::DUMMY.start, 0);
    /// assert_eq!(Span::DUMMY.end, 0);
    /// ```
    pub const DUMMY: Span = Span {
        start: 0,
        end: 0,
        line: 0,
        column: 0,
        file_id: FileId::DUMMY,
    };

    /// Create a new span
    ///
    /// # Arguments
    ///
    /// * `start` - Start byte offset
    /// * `end` - End byte offset
    /// * `line` - Line number (1-based)
    /// * `column` - Column number (1-based)
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::Span;
    ///
    /// let span = Span::new(10, 20, 1, 5);
    /// assert_eq!(span.start, 10);
    /// assert_eq!(span.end, 20);
    /// ```
    #[inline]
    pub fn new(start: usize, end: usize, line: u32, column: u32) -> Self {
        Self {
            start,
            end,
            line,
            column,
            file_id: FileId::DUMMY,
        }
    }

    /// Create a new span associated with a specific file
    ///
    /// # Arguments
    ///
    /// * `start` - Start byte offset
    /// * `end` - End byte offset
    /// * `file_id` - File identifier
    /// * `line` - Line number (1-based)
    /// * `column` - Column number (1-based)
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::{Span, FileId};
    ///
    /// let file_id = FileId(0);
    /// let span = Span::with_file(10, 20, file_id, 1, 5);
    /// assert_eq!(span.file_id, file_id);
    /// ```
    #[inline]
    pub fn with_file(start: usize, end: usize, file_id: FileId, line: u32, column: u32) -> Self {
        Self {
            start,
            end,
            line,
            column,
            file_id,
        }
    }

    /// Create a span at a single point
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::Span;
    ///
    /// let point = Span::point(1, 5);
    /// assert_eq!(point.start, point.end);
    /// ```
    #[inline]
    pub fn point(line: u32, column: u32) -> Self {
        Self {
            start: 0,
            end: 0,
            line,
            column,
            file_id: FileId::DUMMY,
        }
    }

    /// Create a point span associated with a specific file
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::{Span, FileId};
    ///
    /// let file_id = FileId(0);
    /// let point = Span::point_with_file(file_id, 1, 5);
    /// assert_eq!(point.file_id, file_id);
    /// ```
    #[inline]
    pub fn point_with_file(file_id: FileId, line: u32, column: u32) -> Self {
        Self {
            start: 0,
            end: 0,
            line,
            column,
            file_id,
        }
    }

    /// Returns true if this span is empty (start == end)
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::Span;
    ///
    /// let span = Span::new(10, 10, 1, 5);
    /// assert!(span.is_empty());
    ///
    /// let span = Span::new(10, 20, 1, 5);
    /// assert!(!span.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Returns the length of the span in bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::Span;
    ///
    /// let span = Span::new(10, 20, 1, 5);
    /// assert_eq!(span.len(), 10);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Check if this span contains a byte offset
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::Span;
    ///
    /// let span = Span::new(10, 20, 1, 5);
    /// assert!(span.contains(15));
    /// assert!(!span.contains(25));
    /// ```
    #[inline]
    pub fn contains(&self, offset: usize) -> bool {
        self.start <= offset && offset < self.end
    }

    /// Check if this span contains another span
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::Span;
    ///
    /// let outer = Span::new(10, 30, 1, 5);
    /// let inner = Span::new(15, 25, 1, 10);
    /// assert!(outer.contains_span(inner));
    /// ```
    #[inline]
    pub fn contains_span(&self, other: Span) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    /// Merge two spans into a single span covering both
    ///
    /// The resulting span starts at the minimum of both starts
    /// and ends at the maximum of both ends.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::Span;
    ///
    /// let span1 = Span::new(10, 20, 1, 5);
    /// let span2 = Span::new(25, 35, 1, 10);
    /// let merged = span1.merge(span2);
    /// assert_eq!(merged.start, 10);
    /// assert_eq!(merged.end, 35);
    /// ```
    #[inline]
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line.min(other.line),
            column: self.column.min(other.column),
            file_id: self.file_id, // Use self's file_id
        }
    }

    /// Join two adjacent spans into a single span
    ///
    /// Returns `None` if the spans are not adjacent (self.end != other.start).
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::Span;
    ///
    /// let span1 = Span::new(10, 20, 1, 5);
    /// let span2 = Span::new(20, 30, 1, 10);
    /// let joined = span1.join(span2).unwrap();
    /// assert_eq!(joined.start, 10);
    /// assert_eq!(joined.end, 30);
    /// ```
    #[inline]
    pub fn join(self, other: Span) -> Option<Span> {
        if self.end == other.start {
            Some(Span {
                start: self.start,
                end: other.end,
                line: self.line,
                column: self.column,
                file_id: self.file_id,
            })
        } else {
            None
        }
    }

    /// Shrink the span by the specified number of bytes from each end
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::Span;
    ///
    /// let span = Span::new(10, 20, 1, 5);
    /// let shrunk = span.shrink(2);
    /// assert_eq!(shrunk.start, 12);
    /// assert_eq!(shrunk.end, 18);
    /// ```
    #[inline]
    pub fn shrink(self, amount: usize) -> Span {
        Span {
            start: self.start + amount,
            end: self.end.saturating_sub(amount),
            ..self
        }
    }

    /// Expand the span by the specified number of bytes from each end
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::Span;
    ///
    /// let span = Span::new(10, 20, 1, 5);
    /// let expanded = span.expand(2);
    /// assert_eq!(expanded.start, 8);
    /// assert_eq!(expanded.end, 22);
    /// ```
    #[inline]
    pub fn expand(self, amount: usize) -> Span {
        Span {
            start: self.start.saturating_sub(amount),
            end: self.end + amount,
            ..self
        }
    }

    /// Set the file ID for this span
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::{Span, FileId};
    ///
    /// let file_id = FileId(0);
    /// let span = Span::new(10, 20, 1, 5).with_file_id(file_id);
    /// assert_eq!(span.file_id, file_id);
    /// ```
    #[inline]
    pub fn with_file_id(mut self, file_id: FileId) -> Self {
        self.file_id = file_id;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_id_new() {
        let id = FileId::new(5);
        assert_eq!(id.0, 5);
    }

    #[test]
    fn test_file_id_index() {
        let id = FileId(10);
        assert_eq!(id.index(), 10);
    }

    #[test]
    fn test_file_id_default() {
        let id = FileId::default();
        assert_eq!(id, FileId::DUMMY);
    }

    #[test]
    fn test_span_new() {
        let span = Span::new(10, 20, 1, 5);
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 20);
        assert_eq!(span.line, 1);
        assert_eq!(span.column, 5);
    }

    #[test]
    fn test_span_with_file() {
        let file_id = FileId(0);
        let span = Span::with_file(10, 20, file_id, 1, 5);
        assert_eq!(span.file_id, file_id);
    }

    #[test]
    fn test_span_point() {
        let span = Span::point(1, 5);
        assert_eq!(span.start, span.end);
        assert_eq!(span.line, 1);
        assert_eq!(span.column, 5);
    }

    #[test]
    fn test_span_point_with_file() {
        let file_id = FileId(0);
        let span = Span::point_with_file(file_id, 1, 5);
        assert_eq!(span.file_id, file_id);
        assert_eq!(span.start, span.end);
    }

    #[test]
    fn test_span_is_empty() {
        let empty = Span::new(10, 10, 1, 5);
        assert!(empty.is_empty());

        let non_empty = Span::new(10, 20, 1, 5);
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_span_len() {
        let span = Span::new(10, 20, 1, 5);
        assert_eq!(span.len(), 10);
    }

    #[test]
    fn test_span_contains() {
        let span = Span::new(10, 20, 1, 5);
        assert!(span.contains(10));
        assert!(span.contains(15));
        assert!(!span.contains(20));
        assert!(!span.contains(25));
    }

    #[test]
    fn test_span_contains_span() {
        let outer = Span::new(10, 30, 1, 5);
        let inner = Span::new(15, 25, 1, 10);
        assert!(outer.contains_span(inner));
        assert!(!inner.contains_span(outer));
    }

    #[test]
    fn test_span_merge() {
        let span1 = Span::new(10, 20, 1, 5);
        let span2 = Span::new(25, 35, 1, 10);
        let merged = span1.merge(span2);
        assert_eq!(merged.start, 10);
        assert_eq!(merged.end, 35);
    }

    #[test]
    fn test_span_join() {
        let span1 = Span::new(10, 20, 1, 5);
        let span2 = Span::new(20, 30, 1, 10);
        let joined = span1.join(span2).unwrap();
        assert_eq!(joined.start, 10);
        assert_eq!(joined.end, 30);

        // Non-adjacent spans
        let span3 = Span::new(10, 20, 1, 5);
        let span4 = Span::new(25, 35, 1, 10);
        assert!(span3.join(span4).is_none());
    }

    #[test]
    fn test_span_shrink() {
        let span = Span::new(10, 20, 1, 5);
        let shrunk = span.shrink(2);
        assert_eq!(shrunk.start, 12);
        assert_eq!(shrunk.end, 18);
    }

    #[test]
    fn test_span_expand() {
        let span = Span::new(10, 20, 1, 5);
        let expanded = span.expand(2);
        assert_eq!(expanded.start, 8);
        assert_eq!(expanded.end, 22);
    }

    #[test]
    fn test_span_with_file_id() {
        let file_id = FileId(0);
        let span = Span::new(10, 20, 1, 5).with_file_id(file_id);
        assert_eq!(span.file_id, file_id);
    }

    #[test]
    fn test_span_dummy() {
        assert_eq!(Span::DUMMY.start, 0);
        assert_eq!(Span::DUMMY.end, 0);
        assert_eq!(Span::DUMMY.line, 0);
        assert_eq!(Span::DUMMY.column, 0);
    }

    #[test]
    fn test_span_default() {
        let span = Span::default();
        assert_eq!(span, Span::DUMMY);
    }
}
