/// Lexical class of a token, shared by highlighting and completion.
///
/// One classification per token keeps the renderer and the language services
/// reading the same token stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    /// Reserved word of the active dialect.
    Keyword,
    /// Known scalar or aggregate function.
    Function,
    /// String literal.
    String,
    /// Quoted identifier, e.g. `"user name"`.
    QuotedIdentifier,
    /// Numeric literal.
    Number,
    /// Line or block comment.
    Comment,
    /// Bare identifier.
    Identifier,
    /// Placeholder such as `$1`, `?` or `:name`.
    Parameter,
    /// Arithmetic, comparison and logical operators.
    Operator,
    /// Parentheses, commas, semicolons and friends.
    Punctuation,
    /// Space, tabs and line breaks.
    Whitespace,
    /// Anything the tokenizer produced that has no dedicated class.
    Unknown,
}

impl TokenKind {
    /// Whether a token of this class is meaningful for completion, ignoring
    /// layout and comments.
    pub fn is_significant(self) -> bool {
        !matches!(self, TokenKind::Whitespace | TokenKind::Comment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_tokens_are_not_significant() {
        assert!(!TokenKind::Whitespace.is_significant());
        assert!(!TokenKind::Comment.is_significant());
    }

    #[test]
    fn test_words_and_punctuation_are_significant() {
        assert!(TokenKind::Keyword.is_significant());
        assert!(TokenKind::Operator.is_significant());
        assert!(TokenKind::Punctuation.is_significant());
    }
}
