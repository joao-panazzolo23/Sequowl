use crate::core::sql::completion::completion_kind::CompletionKind;

/// One entry of the completion popup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionItem {
    /// Text shown in the popup and inserted by default.
    pub label: String,
    pub kind: CompletionKind,
    /// Secondary text, e.g. the column type or the owning schema.
    pub detail: String,
}

impl CompletionItem {
    pub fn new(label: impl Into<String>, kind: CompletionKind, detail: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            kind,
            detail: detail.into(),
        }
    }

    pub fn column(label: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::new(label, CompletionKind::Column, detail)
    }

    pub fn keyword(label: impl Into<String>) -> Self {
        Self::new(label, CompletionKind::Keyword, "")
    }

    pub fn function(label: impl Into<String>) -> Self {
        Self::new(label, CompletionKind::Function, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_keeps_label_kind_and_detail() {
        let item = CompletionItem::new("id", CompletionKind::Column, "integer, PK");

        assert_eq!(item.label, "id");
        assert_eq!(item.kind, CompletionKind::Column);
        assert_eq!(item.detail, "integer, PK");
    }

    #[test]
    fn test_keyword_helper_has_no_detail() {
        assert_eq!(CompletionItem::keyword("SELECT").detail, "");
    }
}
