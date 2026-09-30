use crate::core::sql::analysis::token_kind::TokenKind;

/// Colored run of characters on a single line.
///
/// Stored in layout coordinates so the renderer only has to multiply, never
/// re-derive positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighlightSpan {
    pub row: u32,
    pub column: u32,
    pub length: u32,
    pub kind: TokenKind,
}

impl HighlightSpan {
    pub fn new(row: u32, column: u32, length: u32, kind: TokenKind) -> Self {
        Self {
            row,
            column,
            length,
            kind,
        }
    }

    /// Whether the span ends on the given row.
    pub fn is_on_row(&self, row: u32) -> bool {
        self.row == row
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_is_matched_by_row() {
        let span = HighlightSpan::new(3, 2, 6, TokenKind::Keyword);

        assert!(span.is_on_row(3));
        assert!(!span.is_on_row(4));
    }
}
