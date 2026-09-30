use crate::core::sql::analysis::diagnostic::Diagnostic;
use crate::core::sql::analysis::highlight_span::HighlightSpan;
use crate::core::sql::analysis::source_line::SourceLine;
use crate::core::sql::completion::completion_item::CompletionItem;
use crate::core::sql::document::text_position::TextPosition;
use crate::core::sql::document::text_selection::TextSelection;

/// Everything the UI needs to draw the editor, derived from the document.
///
/// The text itself is not part of this state: Slint renders the projection and
/// asks the controller for the characters it needs, so the buffer stays in one
/// place.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EditorState {
    /// Colored runs, ordered by row and column.
    pub spans: Vec<HighlightSpan>,
    /// One entry per source line, used by the gutter.
    pub lines: Vec<SourceLine>,
    pub diagnostics: Vec<Diagnostic>,
    pub completions: Vec<CompletionItem>,
    pub completion_visible: bool,
    pub completion_selected: usize,
    /// Column the popup is anchored to.
    pub completion_offset: usize,
    /// Characters scrolled out of view on the left.
    pub scroll_x: usize,
    /// First visible row.
    pub scroll_y: usize,
    pub visible_rows: usize,
    pub visible_columns: usize,
    pub cursor: TextPosition,
    pub selection: TextSelection,
    pub can_undo: bool,
    pub can_redo: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub statement_count: usize,
    pub dialect_name: String,
    pub database_name: String,
}

impl EditorState {
    /// Caret position as shown in the status bar, counting from one.
    pub fn cursor_position(&self) -> (u32, u32) {
        (self.cursor.line as u32 + 1, self.cursor.column as u32 + 1)
    }

    pub fn has_selection(&self) -> bool {
        !self.selection.is_empty()
    }

    /// Diagnostics anchored to a row, for the gutter markers.
    pub fn diagnostics_on_row(&self, row: usize) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.range.start.line == row)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::sql::analysis::diagnostic_code::DiagnosticCode;
    use crate::core::sql::document::text_range::TextRange;

    fn state() -> EditorState {
        EditorState {
            diagnostics: vec![
                Diagnostic::error(DiagnosticCode::SyntaxError, "bad", TextRange::default()),
                Diagnostic::error(
                    DiagnosticCode::SyntaxError,
                    "also bad",
                    TextRange::new(TextPosition::new(2, 0), TextPosition::new(2, 3)),
                ),
            ],
            ..Default::default()
        }
    }

    #[test]
    fn test_cursor_position_is_one_based() {
        let mut state = state();
        state.cursor = TextPosition::new(3, 7);

        assert_eq!(state.cursor_position(), (4, 8));
    }

    #[test]
    fn test_diagnostics_are_grouped_by_row() {
        let state = state();

        assert_eq!(state.diagnostics_on_row(2).len(), 1);
        assert_eq!(
            state.diagnostics_on_row(2)[0].message,
            "also bad",
            "the diagnostic of the row is reported"
        );
        assert!(state.diagnostics_on_row(1).is_empty());
    }

    #[test]
    fn test_a_default_state_has_no_selection_and_no_completion() {
        let state = EditorState::default();

        assert!(!state.has_selection());
        assert!(!state.completion_visible);
        assert!(state.completions.is_empty());
    }
}
