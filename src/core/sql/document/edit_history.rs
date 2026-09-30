use crate::core::sql::document::edit_entry::EditEntry;

/// Undo and redo stacks for document edits.
///
/// Consecutive typing is merged into a single entry so undo behaves like in a
/// text editor instead of removing one character per key press.
#[derive(Debug, Clone)]
pub struct EditHistory {
    undo: Vec<EditEntry>,
    redo: Vec<EditEntry>,
    limit: usize,
}

impl EditHistory {
    pub fn new(limit: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            limit,
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Records an edit that is not part of a typing run.
    pub fn record(&mut self, entry: EditEntry) {
        self.push(entry);
    }

    /// Records an edit that may be merged with the previous one when the user
    /// keeps typing or deleting in the same place.
    pub fn record_typing(&mut self, entry: EditEntry) {
        let merged = self.undo.last_mut().is_some_and(|previous| {
            merge_forward(previous, &entry) || merge_backward(previous, &entry)
        });
        if merged {
            self.redo.clear();
            return;
        }
        self.push(entry);
    }

    /// Returns the entry to apply inverted, moving it to the redo stack.
    pub fn undo(&mut self) -> Option<EditEntry> {
        let entry = self.undo.pop()?;
        self.redo.push(entry.clone());
        Some(entry)
    }

    /// Returns the entry to apply again, moving it back to the undo stack.
    pub fn redo(&mut self) -> Option<EditEntry> {
        let entry = self.redo.pop()?;
        self.undo.push(entry.clone());
        Some(entry)
    }

    /// Drops every recorded edit, used when the whole text is replaced.
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    fn push(&mut self, entry: EditEntry) {
        self.redo.clear();
        self.undo.push(entry);
        if self.undo.len() > self.limit {
            self.undo.remove(0);
        }
    }
}

/// Merges a character typed right after the previous insertion.
///
/// Both entries describe the text before they were applied, so consecutive
/// insertions form one run as long as they are adjacent.
fn merge_forward(previous: &mut EditEntry, next: &EditEntry) -> bool {
    if !previous.is_insertion() || !next.is_insertion() || previous.inserted_end != next.range.start
    {
        return false;
    }

    previous.inserted_end = next.inserted_end;
    previous.inserted.push_str(&next.inserted);
    true
}

/// Merges a character deleted right before the previous deletion.
fn merge_backward(previous: &mut EditEntry, next: &EditEntry) -> bool {
    if !previous.is_deletion() || !next.is_deletion() || previous.range.start != next.range.end {
        return false;
    }

    previous.range.start = next.range.start;
    previous.replaced = format!("{}{}", next.replaced, previous.replaced);
    previous.inserted_end = next.inserted_end;
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::sql::document::text_position::TextPosition;
    use crate::core::sql::document::text_range::TextRange;

    fn range(start: usize, end: usize) -> TextRange {
        TextRange::new(TextPosition::new(0, start), TextPosition::new(0, end))
    }

    /// An insertion of `text` at `column`, described in the coordinates of the
    /// text before it was typed.
    fn insert(column: usize, text: &str) -> EditEntry {
        EditEntry::new(
            TextRange::collapsed(TextPosition::new(0, column)),
            "",
            text,
            TextPosition::new(0, column + text.chars().count()),
        )
    }

    /// A deletion of `text` ending at `column`.
    fn delete(start: usize, end: usize, text: &str) -> EditEntry {
        EditEntry::new(range(start, end), text, "", TextPosition::new(0, start))
    }

    #[test]
    fn test_undo_and_redo_walk_the_same_entry() {
        let mut history = EditHistory::new(10);
        history.record(EditEntry::new(
            range(0, 1),
            "a",
            "b",
            TextPosition::new(0, 1),
        ));

        let entry = history.undo().expect("an entry to undo");
        assert_eq!(entry.inserted, "b");
        assert_eq!(entry.replaced, "a", "undo restores what was there");
        assert!(history.can_redo());

        let entry = history.redo().expect("an entry to redo");
        assert_eq!(entry.inserted, "b", "redo re-applies the same edit");
        assert!(history.can_undo());
    }

    #[test]
    fn test_undo_on_empty_history_returns_nothing() {
        let mut history = EditHistory::new(10);

        assert!(!history.can_undo());
        assert!(history.undo().is_none());
    }

    #[test]
    fn test_typing_is_merged_into_one_entry() {
        let mut history = EditHistory::new(10);
        history.record_typing(insert(0, "S"));
        history.record_typing(insert(1, "E"));
        history.record_typing(insert(2, "L"));

        let entry = history.undo().expect("a merged entry");

        assert_eq!(entry.range, TextRange::collapsed(TextPosition::new(0, 0)));
        assert_eq!(entry.replaced, "");
        assert_eq!(entry.inserted, "SEL");
    }

    #[test]
    fn test_consecutive_backspaces_are_merged() {
        let mut history = EditHistory::new(10);
        history.record_typing(delete(2, 3, "c"));
        history.record_typing(delete(1, 2, "b"));

        let entry = history.undo().expect("a merged entry");

        assert_eq!(entry.range, range(1, 3));
        assert_eq!(entry.replaced, "bc");
    }

    #[test]
    fn test_typing_and_deleting_do_not_merge() {
        let mut history = EditHistory::new(10);
        history.record_typing(insert(0, "a"));
        history.record_typing(delete(0, 1, "a"));

        let entry = history.undo().expect("the deletion is its own entry");

        assert_eq!(entry.replaced, "a");
        assert_eq!(entry.inserted, "");
    }

    #[test]
    fn test_typing_away_from_the_caret_starts_a_new_entry() {
        let mut history = EditHistory::new(10);
        history.record_typing(insert(0, "a"));
        history.record_typing(insert(5, "b"));

        let entry = history.undo().expect("the second entry");

        assert_eq!(entry.inserted, "b", "only the adjacent run was merged");
    }

    #[test]
    fn test_a_new_edit_clears_the_redo_stack() {
        let mut history = EditHistory::new(10);
        history.record(EditEntry::new(
            range(0, 1),
            "a",
            "b",
            TextPosition::new(0, 1),
        ));
        history.undo();

        history.record(EditEntry::new(
            range(0, 1),
            "b",
            "c",
            TextPosition::new(0, 1),
        ));

        assert!(!history.can_redo());
    }

    #[test]
    fn test_history_is_bounded() {
        let mut history = EditHistory::new(2);
        history.record(insert(0, "a"));
        history.record(insert(1, "b"));
        history.record(insert(2, "c"));

        assert_eq!(history.undo().expect("most recent").inserted, "c");
        assert_eq!(history.undo().expect("next").inserted, "b");
        assert!(history.undo().is_none(), "the oldest entry was dropped");
    }

    #[test]
    fn test_clear_drops_both_stacks() {
        let mut history = EditHistory::new(10);
        history.record(insert(0, "a"));
        history.undo();

        history.clear();

        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }
}
