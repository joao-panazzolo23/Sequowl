use crate::core::sql::document::text_position::TextPosition;

/// Half open range of positions, `end` is exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct TextRange {
    pub start: TextPosition,
    pub end: TextPosition,
}

impl TextRange {
    pub const fn new(start: TextPosition, end: TextPosition) -> Self {
        Self { start, end }
    }

    /// A range collapsed on a single position.
    pub const fn collapsed(position: TextPosition) -> Self {
        Self {
            start: position,
            end: position,
        }
    }

    /// Range spanning from the first to the last position, regardless of the
    /// order they were created in.
    pub fn normalized(self) -> Self {
        if self.start <= self.end {
            self
        } else {
            Self {
                start: self.end,
                end: self.start,
            }
        }
    }

    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// Number of characters covered by a range inside a single line.
    /// Ranges that cross a line break have no single line length and report
    /// zero instead of mixing columns of different lines.
    pub fn char_len(self) -> usize {
        if self.start.line == self.end.line {
            self.end.column - self.start.column
        } else {
            0
        }
    }

    pub fn contains(self, position: TextPosition) -> bool {
        self.start <= position && position < self.end
    }

    pub fn intersects(self, other: Self) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Collapses the range when it is empty, otherwise normalizes it.
    pub fn ordered(self) -> Self {
        if self.is_empty() {
            self
        } else {
            self.normalized()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalized_sorts_the_boundaries() {
        let range = TextRange::new(TextPosition::new(2, 0), TextPosition::new(0, 3));

        assert_eq!(
            range.normalized(),
            TextRange::new(TextPosition::new(0, 3), TextPosition::new(2, 0))
        );
    }

    #[test]
    fn test_ordered_keeps_an_empty_range_intact() {
        let empty = TextRange::collapsed(TextPosition::new(2, 0));
        let reversed = TextRange::new(TextPosition::new(2, 0), TextPosition::new(0, 3));

        assert_eq!(empty.ordered().start, TextPosition::new(2, 0));
        assert_eq!(reversed.ordered().start, TextPosition::new(0, 3));
    }

    #[test]
    fn test_char_len_of_a_multi_line_range_is_zero() {
        let range = TextRange::new(TextPosition::new(1, 2), TextPosition::new(3, 5));

        assert_eq!(range.char_len(), 0);
    }

    #[test]
    fn test_char_len_of_a_single_line_range() {
        let range = TextRange::new(TextPosition::new(1, 2), TextPosition::new(1, 7));

        assert_eq!(range.char_len(), 5);
    }

    #[test]
    fn test_contains_is_half_open() {
        let range = TextRange::new(TextPosition::new(0, 1), TextPosition::new(0, 3));

        assert!(range.contains(TextPosition::new(0, 1)));
        assert!(range.contains(TextPosition::new(0, 2)));
        assert!(!range.contains(TextPosition::new(0, 3)));
        assert!(!range.contains(TextPosition::new(0, 0)));
    }

    #[test]
    fn test_intersects() {
        let range = TextRange::new(TextPosition::new(0, 0), TextPosition::new(0, 5));

        assert!(range.intersects(TextRange::new(
            TextPosition::new(0, 4),
            TextPosition::new(1, 0)
        )));
        assert!(!range.intersects(TextRange::new(
            TextPosition::new(0, 5),
            TextPosition::new(0, 8)
        )));
    }
}
