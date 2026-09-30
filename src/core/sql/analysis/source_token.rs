use crate::core::sql::analysis::token_kind::TokenKind;
use crate::core::sql::document::text_range::TextRange;

/// Token produced by the analyzer, positioned in the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceToken {
    pub range: TextRange,
    pub kind: TokenKind,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::sql::document::text_position::TextPosition;

    #[test]
    fn test_token_keeps_its_range_and_class() {
        let token = SourceToken {
            range: TextRange::new(TextPosition::new(0, 0), TextPosition::new(0, 6)),
            kind: TokenKind::Keyword,
        };

        assert_eq!(token.kind, TokenKind::Keyword);
        assert_eq!(token.range.char_len(), 6);
    }
}
