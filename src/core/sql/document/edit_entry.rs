use crate::core::sql::document::text_position::TextPosition;
use crate::core::sql::document::text_range::TextRange;

/// A single text modification, kept with the text it removed so it can be
/// replayed in both directions.
///
/// `range` covers the text that was replaced, in the coordinates of the text
/// *before* the edit. `inserted_end` is the caret that resulted from the edit,
/// which is what the replacement occupied afterwards, so undoing only needs
/// this one extra position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditEntry {
    /// Range the edit replaced, before it was applied.
    pub range: TextRange,
    /// Text that was there before.
    pub replaced: String,
    /// Text that was put in its place.
    pub inserted: String,
    /// Caret position right after the edit was applied.
    pub inserted_end: TextPosition,
}

impl EditEntry {
    pub fn new(
        range: TextRange,
        replaced: impl Into<String>,
        inserted: impl Into<String>,
        inserted_end: TextPosition,
    ) -> Self {
        Self {
            range,
            replaced: replaced.into(),
            inserted: inserted.into(),
            inserted_end,
        }
    }

    /// The range the replacement occupies in the edited document.
    pub fn applied_range(&self) -> TextRange {
        TextRange::new(self.range.start, self.inserted_end)
    }

    /// The range the replaced text occupies, for replaying the edit again.
    pub fn replayed_range(&self) -> TextRange {
        TextRange::new(self.range.start, advance(self.range.start, &self.replaced))
    }

    /// Whether the edit only added text.
    pub fn is_insertion(&self) -> bool {
        self.replaced.is_empty()
    }

    /// Whether the edit only removed text.
    pub fn is_deletion(&self) -> bool {
        self.inserted.is_empty()
    }
}

/// Walks text to find where a caret ends up, matching
/// [`TextDocument`](crate::core::sql::document::text_document::TextDocument).
pub fn advance(start: TextPosition, text: &str) -> TextPosition {
    let mut position = start;
    for ch in text.chars() {
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

    fn at(line: usize, column: usize) -> TextPosition {
        TextPosition::new(line, column)
    }

    #[test]
    fn test_applied_range_spans_the_inserted_text() {
        let entry = EditEntry::new(TextRange::collapsed(at(0, 3)), "", "hello", at(0, 8));

        assert_eq!(entry.applied_range(), TextRange::new(at(0, 3), at(0, 8)));
        assert_eq!(entry.replayed_range(), TextRange::new(at(0, 3), at(0, 3)));
    }

    #[test]
    fn test_replayed_range_spans_the_replaced_text() {
        let entry = EditEntry::new(TextRange::new(at(0, 2), at(0, 5)), "abc", "xy", at(0, 4));

        assert_eq!(entry.replayed_range(), TextRange::new(at(0, 2), at(0, 5)));
    }

    #[test]
    fn test_insertions_and_deletions_are_recognised() {
        let insert = EditEntry::new(TextRange::collapsed(at(0, 0)), "", "a", at(0, 1));
        let delete = EditEntry::new(TextRange::new(at(0, 0), at(0, 1)), "a", "", at(0, 0));

        assert!(insert.is_insertion());
        assert!(!insert.is_deletion());
        assert!(delete.is_deletion());
        assert!(!delete.is_insertion());
    }

    #[test]
    fn test_advance_walks_a_multi_line_string() {
        assert_eq!(advance(at(1, 4), "ab\ncd"), at(2, 2));
    }
}
