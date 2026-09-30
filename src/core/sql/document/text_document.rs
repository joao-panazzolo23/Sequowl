use crate::core::sql::document::text_position::TextPosition;
use crate::core::sql::document::text_range::TextRange;

/// UTF-8 text buffer with a line index, so positions and byte offsets can be
/// converted cheaply in both directions.
///
/// The document is the single place that knows about bytes; the rest of the
/// editor only speaks in [`TextPosition`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextDocument {
    text: String,
    lines: Vec<LineEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LineEntry {
    start: usize,
    chars: usize,
}

impl TextDocument {
    pub fn new(text: impl Into<String>) -> Self {
        let mut document = Self {
            text: text.into(),
            lines: Vec::new(),
        };
        document.reindex();
        document
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Number of lines. A trailing line break yields a final empty line.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Text of a line without its trailing line break.
    pub fn line(&self, line: usize) -> &str {
        let line = line.min(self.lines.len().saturating_sub(1));
        &self.text[self.lines[line].start..self.line_end_offset(line)]
    }

    /// Character count of a line, without its trailing line break.
    pub fn line_length(&self, line: usize) -> usize {
        self.lines[line.min(self.lines.len().saturating_sub(1))].chars
    }

    /// Line holding the given position, without its trailing line break.
    pub fn line_text_at(&self, position: TextPosition) -> &str {
        self.line(position.line)
    }

    /// Clamps a position into the document.
    pub fn clamp_position(&self, position: TextPosition) -> TextPosition {
        let line = position.line.min(self.lines.len().saturating_sub(1));
        TextPosition::new(line, position.column.min(self.lines[line].chars))
    }

    /// Position just after the last character.
    pub fn end_position(&self) -> TextPosition {
        let last = self.lines.len().saturating_sub(1);
        TextPosition::new(last, self.lines[last].chars)
    }

    /// Byte offset of a position, clamped to the document.
    pub fn offset_at(&self, position: TextPosition) -> usize {
        let line = position.line.min(self.lines.len().saturating_sub(1));
        let entry = self.lines[line];
        let limit = self.line_end_offset(line);
        let mut offset = entry.start;
        for (column, ch) in self.text[entry.start..limit].chars().enumerate() {
            if column >= position.column {
                break;
            }
            offset += ch.len_utf8();
        }
        offset
    }

    /// Position of a byte offset, clamped to the document.
    pub fn position_at(&self, offset: usize) -> TextPosition {
        let offset = offset.min(self.text.len());
        let line = match self
            .lines
            .binary_search_by(|entry| entry.start.cmp(&offset))
        {
            Ok(line) => line,
            Err(0) => 0,
            Err(line) => line - 1,
        };
        let column = self.text[self.lines[line].start..offset].chars().count();
        TextPosition::new(line, column)
    }

    /// Character right before a position, if any.
    pub fn char_before(&self, position: TextPosition) -> Option<char> {
        if position.column == 0 {
            return None;
        }
        self.line(position.line).chars().nth(position.column - 1)
    }

    /// Character at a position, if any.
    pub fn char_at(&self, position: TextPosition) -> Option<char> {
        self.line(position.line).chars().nth(position.column)
    }

    /// Text covered by a position range.
    pub fn text_in(&self, range: TextRange) -> &str {
        let range = range.normalized();
        let start = self.offset_at(range.start);
        let end = self.offset_at(range.end);
        &self.text[start..end]
    }

    /// Replaces a position range and returns the caret position after the
    /// inserted text.
    pub fn replace(&mut self, range: TextRange, replacement: &str) -> TextPosition {
        let range = range.normalized();
        let mut start = self.offset_at(range.start);
        let mut end = self.offset_at(range.end);
        if start > end {
            std::mem::swap(&mut start, &mut end);
        }

        self.text.replace_range(start..end, replacement);
        self.reindex();

        position_after(range.start, replacement)
    }

    /// Inserts text at a position, returning the caret position afterwards.
    pub fn insert_at(&mut self, position: TextPosition, text: &str) -> TextPosition {
        self.replace(TextRange::collapsed(position), text)
    }

    fn line_end_offset(&self, line: usize) -> usize {
        match self.lines.get(line + 1) {
            Some(next) => next.start - 1,
            None => self.text.len(),
        }
    }

    fn reindex(&mut self) {
        self.lines.clear();
        self.lines.reserve(self.text.len() / 32 + 1);

        let mut start = 0;
        for (offset, ch) in self.text.char_indices() {
            if ch == '\n' {
                self.lines.push(LineEntry {
                    start,
                    chars: self.text[start..offset].chars().count(),
                });
                start = offset + 1;
            }
        }
        self.lines.push(LineEntry {
            start,
            chars: self.text[start..].chars().count(),
        });
    }
}

/// Walks inserted text to find where the caret ends up.
fn position_after(start: TextPosition, inserted: &str) -> TextPosition {
    let mut position = start;
    for ch in inserted.chars() {
        if ch == '\n' {
            position.line += 1;
            position.column = 0;
        } else {
            position.column += 1;
        }
    }
    position
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_index_handles_a_trailing_newline() {
        let document = TextDocument::new("SELECT 1;\n");

        assert_eq!(document.line_count(), 2);
        assert_eq!(document.line(0), "SELECT 1;");
        assert_eq!(document.line(1), "");
        assert_eq!(document.line_length(1), 0);
    }

    #[test]
    fn test_empty_document_has_one_line() {
        let document = TextDocument::new("");

        assert_eq!(document.line_count(), 1);
        assert_eq!(document.end_position(), TextPosition::new(0, 0));
    }

    #[test]
    fn test_offsets_and_positions_round_trip_over_multibyte_text() {
        let document = TextDocument::new("SELECT 'héllo'");

        let position = TextPosition::new(0, 11);
        let offset = document.offset_at(position);

        assert_eq!(offset, 12, "the é before the position takes two bytes");
        assert_eq!(document.position_at(offset), position);
        assert_eq!(document.line_length(0), 14, "the line holds 14 characters");
    }

    #[test]
    fn test_offset_at_clamps_beyond_the_line() {
        let document = TextDocument::new("ab\ncd");

        assert_eq!(document.offset_at(TextPosition::new(0, 99)), 2);
        assert_eq!(document.clamp_position(TextPosition::new(5, 0)).line, 1);
    }

    #[test]
    fn test_position_at_clamps_beyond_the_document() {
        let document = TextDocument::new("ab\ncd");

        assert_eq!(document.position_at(999), TextPosition::new(1, 2));
    }

    #[test]
    fn test_replace_updates_lines_and_caret() {
        let mut document = TextDocument::new("SELECT a\nFROM t");

        let caret = document.replace(
            TextRange::new(TextPosition::new(0, 7), TextPosition::new(0, 8)),
            "b, c",
        );

        assert_eq!(document.text(), "SELECT b, c\nFROM t");
        assert_eq!(caret, TextPosition::new(0, 11));
        assert_eq!(document.line_count(), 2);
    }

    #[test]
    fn test_insert_with_newline_advances_the_caret_to_the_next_line() {
        let mut document = TextDocument::new("SELECT 1");

        let caret = document.insert_at(TextPosition::new(0, 8), ";\nSELECT 2");

        assert_eq!(document.line_count(), 2);
        assert_eq!(document.line(1), "SELECT 2");
        assert_eq!(caret, TextPosition::new(1, 8));
    }

    #[test]
    fn test_replace_with_an_empty_string_deletes_the_range() {
        let mut document = TextDocument::new("SELECT  abc");

        document.replace(
            TextRange::new(TextPosition::new(0, 6), TextPosition::new(0, 10)),
            "",
        );

        assert_eq!(document.text(), "SELECTc");
    }

    #[test]
    fn test_text_in_returns_the_selected_slice() {
        let document = TextDocument::new("SELECT a FROM users");

        let range = TextRange::new(TextPosition::new(0, 9), TextPosition::new(0, 15));

        assert_eq!(document.text_in(range), "FROM u");
    }

    #[test]
    fn test_char_accessors() {
        let document = TextDocument::new("aé b");

        assert_eq!(document.char_at(TextPosition::new(0, 1)), Some('é'));
        assert_eq!(document.char_before(TextPosition::new(0, 1)), Some('a'));
        assert_eq!(document.char_at(TextPosition::new(0, 3)), Some('b'));
        assert_eq!(document.char_at(TextPosition::new(0, 4)), None);
        assert_eq!(document.char_before(TextPosition::new(0, 0)), None);
    }
}
