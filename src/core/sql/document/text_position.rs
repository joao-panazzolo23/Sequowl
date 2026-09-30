use std::cmp::Ordering;

/// Zero based position inside a document.
///
/// `column` counts characters (not bytes) so it can be used directly for
/// monospace layout, caret math and diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct TextPosition {
    pub line: usize,
    pub column: usize,
}

impl TextPosition {
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    /// Clamps the position into the document, keeping it ordered.
    pub fn clamped(self, other: Self) -> Self {
        if self <= other { self } else { other }
    }
}

impl PartialOrd for TextPosition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TextPosition {
    fn cmp(&self, other: &Self) -> Ordering {
        self.line
            .cmp(&other.line)
            .then_with(|| self.column.cmp(&other.column))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positions_sort_by_line_then_column() {
        let mut positions = vec![
            TextPosition::new(1, 4),
            TextPosition::new(0, 9),
            TextPosition::new(1, 2),
        ];

        positions.sort();

        assert_eq!(
            positions,
            vec![
                TextPosition::new(0, 9),
                TextPosition::new(1, 2),
                TextPosition::new(1, 4),
            ]
        );
    }

    #[test]
    fn test_clamped_returns_the_earlier_position() {
        assert_eq!(
            TextPosition::new(2, 0).clamped(TextPosition::new(1, 5)),
            TextPosition::new(1, 5)
        );
    }
}
