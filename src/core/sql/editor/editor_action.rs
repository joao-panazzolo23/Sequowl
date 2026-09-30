use crate::core::sql::document::text_position::TextPosition;

/// What the UI asked the editor to do.
///
/// Key presses are translated into actions in the presentation layer, so the
/// editor logic never depends on key codes and can be tested without a UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorAction {
    Insert(char),
    InsertText(String),
    NewLine,
    Backspace,
    Delete,
    DeleteWordBackward,
    MoveLeft {
        select: bool,
    },
    MoveRight {
        select: bool,
    },
    MoveUp {
        select: bool,
    },
    MoveDown {
        select: bool,
    },
    /// Moves by a viewport worth of rows, PageUp and PageDown.
    MovePageUp {
        select: bool,
    },
    MovePageDown {
        select: bool,
    },
    MoveWordLeft {
        select: bool,
    },
    MoveWordRight {
        select: bool,
    },
    MoveToLineStart {
        select: bool,
    },
    MoveToLineEnd {
        select: bool,
    },
    MoveToDocumentStart {
        select: bool,
    },
    MoveToDocumentEnd {
        select: bool,
    },
    SelectAll,
    Undo,
    Redo,
    /// Explicit request, e.g. Ctrl+Space.
    RequestCompletion,
    /// Automatic refresh while typing.
    UpdateCompletion,
    CompleteNext,
    CompletePrevious,
    AcceptCompletion,
    /// Accepts the item at a given index, e.g. after a click in the popup.
    AcceptCompletionAt(usize),
    DismissCompletion,
    SetCursor(TextPosition),
    /// A click, with the position and whether it extends the selection.
    Select {
        position: TextPosition,
        extend: bool,
    },
    ReplaceAll(String),
}

impl EditorAction {
    /// Whether the action changes the text, which decides if the completion
    /// popup should be refreshed and if the edit joins the typing run.
    pub fn is_edit(&self) -> bool {
        matches!(
            self,
            EditorAction::Insert(_)
                | EditorAction::InsertText(_)
                | EditorAction::NewLine
                | EditorAction::Backspace
                | EditorAction::Delete
                | EditorAction::DeleteWordBackward
                | EditorAction::ReplaceAll(_)
        )
    }

    /// Whether the popup should be shown after the action ran.
    pub fn wants_completion(&self) -> bool {
        matches!(
            self,
            EditorAction::RequestCompletion | EditorAction::UpdateCompletion
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_changing_actions_are_edits() {
        assert!(EditorAction::Insert('a').is_edit());
        assert!(EditorAction::NewLine.is_edit());
        assert!(EditorAction::Backspace.is_edit());
        assert!(!EditorAction::Undo.is_edit());
        assert!(!EditorAction::MoveLeft { select: false }.is_edit());
        assert!(!EditorAction::MovePageDown { select: true }.is_edit());
    }

    #[test]
    fn test_only_completion_actions_request_the_popup() {
        assert!(EditorAction::RequestCompletion.wants_completion());
        assert!(EditorAction::UpdateCompletion.wants_completion());
        assert!(!EditorAction::Insert('a').wants_completion());
    }
}
