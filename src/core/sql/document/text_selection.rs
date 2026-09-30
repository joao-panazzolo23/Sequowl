use crate::core::sql::document::text_position::TextPosition;
use crate::core::sql::document::text_range::TextRange;

/// Caret anchor and focus, so a selection keeps its direction.
///
/// The focus is where the caret is drawn and where typed text goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct TextSelection {
    pub anchor: TextPosition,
    pub focus: TextPosition,
}

impl TextSelection {
    pub const fn collapsed(position: TextPosition) -> Self {
        Self {
            anchor: position,
            focus: position,
        }
    }

    pub fn range(&self) -> TextRange {
        TextRange::new(self.anchor, self.focus)
    }

    /// The selected range, always from the earlier to the later position.
    pub fn normalized_range(&self) -> TextRange {
        self.range().normalized()
    }

    pub fn is_empty(&self) -> bool {
        self.anchor == self.focus
    }

    /// Range to replace on the next edit.
    pub fn replacement_range(&self) -> TextRange {
        if self.is_empty() {
            TextRange::collapsed(self.focus)
        } else {
            self.normalized_range()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reversed_selection_replaces_the_whole_range() {
        let selection = TextSelection {
            anchor: TextPosition::new(1, 4),
            focus: TextPosition::new(0, 2),
        };

        assert_eq!(
            selection.replacement_range(),
            TextRange::new(TextPosition::new(0, 2), TextPosition::new(1, 4))
        );
    }

    #[test]
    fn test_collapsed_selection_replaces_nothing() {
        let selection = TextSelection::collapsed(TextPosition::new(3, 1));

        assert!(selection.is_empty());
        assert_eq!(
            selection.replacement_range(),
            TextRange::collapsed(TextPosition::new(3, 1))
        );
    }
}
