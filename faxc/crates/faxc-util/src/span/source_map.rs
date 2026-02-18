//! Source map for managing source files and computing source locations.
//!
//! This module provides the [`SourceMap`] type for managing multiple source files
//! and computing line/column information from byte offsets.

use std::sync::Arc;

use super::{FileId, Span};

/// A source file with its content and metadata
///
/// # Examples
///
/// ```
/// use faxc_util::span::SourceFile;
///
/// let file = SourceFile::new(0, "main.rs", "fn main() {}");
/// assert_eq!(file.name(), "main.rs");
/// assert_eq!(file.content(), "fn main() {}");
/// ```
#[derive(Clone)]
pub struct SourceFile {
    /// Unique file identifier
    id: FileId,
    /// File name (path or display name)
    name: String,
    /// File content
    content: Arc<str>,
    /// Precomputed line start offsets
    line_starts: Arc<[usize]>,
}

impl SourceFile {
    /// Create a new source file
    ///
    /// # Arguments
    ///
    /// * `id` - Unique file identifier
    /// * `name` - File name or path
    /// * `content` - File content
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::SourceFile;
    ///
    /// let file = SourceFile::new(0, "main.rs", "fn main() {}");
    /// ```
    pub fn new(id: usize, name: impl Into<String>, content: impl Into<Arc<str>>) -> Self {
        let content = content.into();
        let line_starts = Self::compute_line_starts(&content);
        Self {
            id: FileId(id),
            name: name.into(),
            content,
            line_starts,
        }
    }

    /// Compute line start offsets from content
    fn compute_line_starts(content: &str) -> Arc<[usize]> {
        let mut line_starts = Vec::new();
        line_starts.push(0);

        for (i, ch) in content.char_indices() {
            if ch == '\n' {
                line_starts.push(i + 1);
            }
        }

        line_starts.into()
    }

    /// Get the file identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::SourceFile;
    ///
    /// let file = SourceFile::new(0, "main.rs", "content");
    /// assert_eq!(file.id().0, 0);
    /// ```
    #[inline]
    pub fn id(&self) -> FileId {
        self.id
    }

    /// Get the file name
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::SourceFile;
    ///
    /// let file = SourceFile::new(0, "main.rs", "content");
    /// assert_eq!(file.name(), "main.rs");
    /// ```
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the file content
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::SourceFile;
    ///
    /// let file = SourceFile::new(0, "main.rs", "fn main() {}");
    /// assert_eq!(file.content(), "fn main() {}");
    /// ```
    #[inline]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Get the total number of lines
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::SourceFile;
    ///
    /// let file = SourceFile::new(0, "main.rs", "line1\nline2\nline3");
    /// assert_eq!(file.line_count(), 3);
    /// ```
    #[inline]
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Get the byte offset where a line starts (0-indexed line number)
    ///
    /// Returns `None` if the line number is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::SourceFile;
    ///
    /// let file = SourceFile::new(0, "main.rs", "line1\nline2\nline3");
    /// assert_eq!(file.line_start(0), Some(0));
    /// assert_eq!(file.line_start(1), Some(6));
    /// assert_eq!(file.line_start(10), None);
    /// ```
    #[inline]
    pub fn line_start(&self, line: usize) -> Option<usize> {
        self.line_starts.get(line).copied()
    }

    /// Convert a byte offset to (line, column) coordinates
    ///
    /// Line and column are 1-indexed. Column is measured in bytes from the
    /// start of the line.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::SourceFile;
    ///
    /// let file = SourceFile::new(0, "main.rs", "fn main() {}");
    /// let (line, col) = file.offset_to_line_col(3);
    /// assert_eq!(line, 1);
    /// assert_eq!(col, 4); // "main" starts at column 4
    /// ```
    pub fn offset_to_line_col(&self, offset: usize) -> (usize, usize) {
        // Binary search for the line
        match self.line_starts.binary_search(&offset) {
            Ok(line) => (line + 1, 1), // Exact match = start of line
            Err(insert_point) => {
                let line = insert_point - 1;
                let line_start = self.line_starts.get(line).copied().unwrap_or(0);
                let col = offset - line_start + 1;
                (line + 1, col)
            }
        }
    }

    /// Get the source line containing a byte offset
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::SourceFile;
    ///
    /// let file = SourceFile::new(0, "main.rs", "line1\nline2\nline3");
    /// assert_eq!(file.line_at_offset(8), Some("line2"));
    /// ```
    pub fn line_at_offset(&self, offset: usize) -> Option<&str> {
        let (line, _) = self.offset_to_line_col(offset);
        self.line_at(line)
    }

    /// Get a specific source line (1-indexed)
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::SourceFile;
    ///
    /// let file = SourceFile::new(0, "main.rs", "line1\nline2\nline3");
    /// assert_eq!(file.line_at(1), Some("line1"));
    /// assert_eq!(file.line_at(2), Some("line2"));
    /// ```
    pub fn line_at(&self, line: usize) -> Option<&str> {
        let start = self.line_start(line - 1)?;
        let end = self
            .line_start(line)
            .unwrap_or(self.content.len());

        // Trim the newline character(s)
        let line_content = &self.content[start..end];
        Some(line_content.trim_end_matches(|c| c == '\n' || c == '\r'))
    }

    /// Extract a substring from the file content
    ///
    /// # Panics
    ///
    /// Panics if the range is out of bounds or not on character boundaries.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::SourceFile;
    ///
    /// let file = SourceFile::new(0, "main.rs", "fn main() {}");
    /// assert_eq!(file.extract(0..2), "fn");
    /// ```
    pub fn extract(&self, range: std::ops::Range<usize>) -> &str {
        &self.content[range]
    }
}

impl std::fmt::Debug for SourceFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceFile")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("line_count", &self.line_count())
            .finish()
    }
}

/// A source map managing multiple source files
///
/// # Examples
///
/// ```
/// use faxc_util::span::SourceMap;
///
/// let mut map = SourceMap::new();
/// let file_id = map.add_file("main.rs", "fn main() {}");
/// let file = map.get(file_id).unwrap();
/// assert_eq!(file.name(), "main.rs");
/// ```
#[derive(Default)]
pub struct SourceMap {
    files: Vec<Arc<SourceFile>>,
}

impl SourceMap {
    /// Create a new empty source map
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::SourceMap;
    ///
    /// let map = SourceMap::new();
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// Add a new source file
    ///
    /// Returns the [`FileId`] for the added file.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::SourceMap;
    ///
    /// let mut map = SourceMap::new();
    /// let file_id = map.add_file("main.rs", "fn main() {}");
    /// ```
    pub fn add_file(&mut self, name: impl Into<String>, content: impl Into<Arc<str>>) -> FileId {
        let id = self.files.len();
        let file = SourceFile::new(id, name, content);
        let file_id = file.id();
        self.files.push(Arc::new(file));
        file_id
    }

    /// Get a source file by its ID
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::SourceMap;
    ///
    /// let mut map = SourceMap::new();
    /// let file_id = map.add_file("main.rs", "fn main() {}");
    /// let file = map.get(file_id).unwrap();
    /// ```
    #[inline]
    pub fn get(&self, id: FileId) -> Option<Arc<SourceFile>> {
        self.files.get(id.0).cloned()
    }

    /// Get the number of files in the source map
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::SourceMap;
    ///
    /// let mut map = SourceMap::new();
    /// assert_eq!(map.file_count(), 0);
    /// map.add_file("main.rs", "");
    /// assert_eq!(map.file_count(), 1);
    /// ```
    #[inline]
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Convert a span to a human-readable string with source context
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::span::{SourceMap, Span};
    ///
    /// let mut map = SourceMap::new();
    /// let file_id = map.add_file("main.rs", "fn main() {}");
    /// let span = Span::with_file(0, 2, file_id, 1, 1);
    /// let formatted = map.format_span(span);
    /// ```
    pub fn format_span(&self, span: Span) -> Option<String> {
        let file = self.get(span.file_id)?;
        let start_line = span.line;
        let start_col = span.column;

        let line = file.line_at(start_line as usize)?;
        let line_num_width = file.line_count().to_string().len().max(3);

        let mut result = String::new();
        result.push_str(&format!(
            "--> {}:{}:{}\n",
            file.name(),
            start_line,
            start_col
        ));
        result.push_str(&format!(
            "{:>width$} | {}\n",
            start_line,
            line,
            width = line_num_width
        ));
        result.push_str(&format!("{:>width$} | ", "", width = line_num_width));

        // Add carets for the span
        let underline_start = (start_col as usize).saturating_sub(1);
        let underline_len = if span.start == span.end {
            1
        } else {
            (span.end - span.start).max(1)
        };

        for _ in 0..underline_start {
            result.push(' ');
        }
        for _ in 0..underline_len {
            result.push('^');
        }

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_file_new() {
        let file = SourceFile::new(0, "test.rs", "fn main() {}");
        assert_eq!(file.id().0, 0);
        assert_eq!(file.name(), "test.rs");
        assert_eq!(file.content(), "fn main() {}");
    }

    #[test]
    fn test_source_file_line_count() {
        let file = SourceFile::new(0, "test.rs", "line1\nline2\nline3");
        assert_eq!(file.line_count(), 3);
    }

    #[test]
    fn test_source_file_line_start() {
        let file = SourceFile::new(0, "test.rs", "line1\nline2\nline3");
        assert_eq!(file.line_start(0), Some(0));
        assert_eq!(file.line_start(1), Some(6));
        assert_eq!(file.line_start(2), Some(12));
        assert_eq!(file.line_start(3), None);
    }

    #[test]
    fn test_source_file_offset_to_line_col() {
        let file = SourceFile::new(0, "test.rs", "fn main() {}");
        assert_eq!(file.offset_to_line_col(0), (1, 1));
        assert_eq!(file.offset_to_line_col(3), (1, 4));
        assert_eq!(file.offset_to_line_col(11), (1, 12));
    }

    #[test]
    fn test_source_file_line_at_offset() {
        let file = SourceFile::new(0, "test.rs", "line1\nline2\nline3");
        assert_eq!(file.line_at_offset(0), Some("line1"));
        assert_eq!(file.line_at_offset(7), Some("line2"));
    }

    #[test]
    fn test_source_file_line_at() {
        let file = SourceFile::new(0, "test.rs", "line1\nline2\nline3");
        assert_eq!(file.line_at(1), Some("line1"));
        assert_eq!(file.line_at(2), Some("line2"));
        assert_eq!(file.line_at(3), Some("line3"));
        assert_eq!(file.line_at(4), None);
    }

    #[test]
    fn test_source_file_extract() {
        let file = SourceFile::new(0, "test.rs", "fn main() {}");
        assert_eq!(file.extract(0..2), "fn");
        assert_eq!(file.extract(3..7), "main");
    }

    #[test]
    fn test_source_map_add_file() {
        let mut map = SourceMap::new();
        let file_id = map.add_file("main.rs", "fn main() {}");
        assert_eq!(file_id.0, 0);
    }

    #[test]
    fn test_source_map_get() {
        let mut map = SourceMap::new();
        let file_id = map.add_file("main.rs", "fn main() {}");
        let file = map.get(file_id).unwrap();
        assert_eq!(file.name(), "main.rs");
    }

    #[test]
    fn test_source_map_get_invalid() {
        let map = SourceMap::new();
        assert!(map.get(FileId(0)).is_none());
    }

    #[test]
    fn test_source_map_file_count() {
        let mut map = SourceMap::new();
        assert_eq!(map.file_count(), 0);
        map.add_file("main.rs", "");
        assert_eq!(map.file_count(), 1);
        map.add_file("lib.rs", "");
        assert_eq!(map.file_count(), 2);
    }

    #[test]
    fn test_source_map_format_span() {
        let mut map = SourceMap::new();
        let file_id = map.add_file("main.rs", "fn main() {}");
        let span = Span::with_file(0, 2, file_id, 1, 1);
        let formatted = map.format_span(span).unwrap();
        assert!(formatted.contains("main.rs"));
        assert!(formatted.contains("fn main"));
    }

    #[test]
    fn test_multiline_file() {
        let content = "fn main() {\n    println!(\"Hello\");\n}";
        let file = SourceFile::new(0, "test.rs", content);

        assert_eq!(file.line_count(), 3);
        assert_eq!(file.line_start(0), Some(0));
        assert_eq!(file.line_start(1), Some(13));
        assert_eq!(file.line_start(2), Some(33));

        let (line, col) = file.offset_to_line_col(15);
        assert_eq!(line, 2);
        assert!(col >= 1);
    }

    #[test]
    fn test_empty_file() {
        let file = SourceFile::new(0, "empty.rs", "");
        assert_eq!(file.line_count(), 1);
        assert_eq!(file.line_start(0), Some(0));
        assert_eq!(file.offset_to_line_col(0), (1, 1));
    }
}
