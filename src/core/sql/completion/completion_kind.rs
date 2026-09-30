/// What a completion item stands for, used for the icon and the accent colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompletionKind {
    Schema,
    Table,
    View,
    Column,
    Function,
    Keyword,
}

impl CompletionKind {
    /// Lower sorts first, so the closest match stays at the top.
    pub fn priority(self) -> u8 {
        match self {
            CompletionKind::Column => 0,
            CompletionKind::Table => 1,
            CompletionKind::View => 2,
            CompletionKind::Schema => 3,
            CompletionKind::Function => 4,
            CompletionKind::Keyword => 5,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            CompletionKind::Schema => "schema",
            CompletionKind::Table => "table",
            CompletionKind::View => "view",
            CompletionKind::Column => "column",
            CompletionKind::Function => "function",
            CompletionKind::Keyword => "keyword",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_columns_are_offered_before_keywords() {
        assert!(CompletionKind::Column.priority() < CompletionKind::Keyword.priority());
    }

    #[test]
    fn test_every_kind_has_a_label() {
        let kinds = [
            CompletionKind::Schema,
            CompletionKind::Table,
            CompletionKind::View,
            CompletionKind::Column,
            CompletionKind::Function,
            CompletionKind::Keyword,
        ];

        for kind in kinds {
            assert!(!kind.label().is_empty());
        }
    }
}
