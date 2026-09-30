use crate::core::sql::completion::completion_item::CompletionItem;
use crate::core::sql::document::text_range::TextRange;

/// Suggestions for one caret position, with the range they replace.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompletionResult {
    pub items: Vec<CompletionItem>,
    pub replace_range: TextRange,
    /// Index of the highlighted item.
    pub selected: usize,
}

impl CompletionResult {
    pub fn new(items: Vec<CompletionItem>, replace_range: TextRange) -> Self {
        Self {
            items,
            replace_range,
            selected: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn selected_item(&self) -> Option<&CompletionItem> {
        self.items.get(self.selected)
    }

    /// Moves the highlight, wrapping around at both ends.
    pub fn select_next(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + 1) % self.items.len();
        }
    }

    pub fn select_previous(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + self.items.len() - 1) % self.items.len();
        }
    }

    /// Highlights an item by index, e.g. after a click in the popup.
    pub fn select(&mut self, index: usize) -> bool {
        match self.items.get(index) {
            Some(_) => {
                self.selected = index;
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::sql::completion::completion_kind::CompletionKind;

    fn result() -> CompletionResult {
        CompletionResult::new(
            vec![
                CompletionItem::new("id", CompletionKind::Column, "integer"),
                CompletionItem::new("name", CompletionKind::Column, "text"),
            ],
            TextRange::default(),
        )
    }

    #[test]
    fn test_selection_wraps_around() {
        let mut result = result();
        assert_eq!(result.selected, 0);

        result.select_next();
        assert_eq!(result.selected, 1);

        result.select_next();
        assert_eq!(result.selected, 0, "wraps to the first item");

        result.select_previous();
        assert_eq!(result.selected, 1, "wraps to the last item");
    }

    #[test]
    fn test_empty_result_has_no_selection() {
        let mut result = CompletionResult::default();

        result.select_next();
        result.select_previous();

        assert!(result.is_empty());
        assert!(result.selected_item().is_none());
    }

    #[test]
    fn test_selected_item_is_the_highlighted_one() {
        let mut result = result();

        assert_eq!(
            result.selected_item().map(|item| item.label.as_str()),
            Some("id")
        );

        result.select_next();
        assert_eq!(
            result.selected_item().map(|item| item.label.as_str()),
            Some("name")
        );
    }

    #[test]
    fn test_select_moves_the_highlight_to_an_index() {
        let mut result = result();

        assert!(result.select(1));
        assert_eq!(
            result.selected_item().map(|item| item.label.as_str()),
            Some("name")
        );
    }

    #[test]
    fn test_select_ignores_an_index_outside_the_list() {
        let mut result = result();

        assert!(!result.select(7), "there is no such item");
        assert_eq!(result.selected, 0, "the highlight stays where it was");
    }
}
