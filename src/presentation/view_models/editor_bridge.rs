use crate::core::entities::database_provider::DatabaseProvider;
use crate::core::sql::analysis::diagnostic_severity::DiagnosticSeverity;
use crate::core::sql::analysis::token_kind::TokenKind;
use crate::core::sql::catalog::schema_catalog::SchemaCatalog;
use crate::core::sql::completion::completion_kind::CompletionKind;
use crate::core::sql::dialect::dialect_for_provider;
use crate::core::sql::document::text_document::TextDocument;
use crate::core::sql::document::text_position::TextPosition;
use crate::core::sql::document::text_range::TextRange;
use crate::core::sql::editor::editor_action::EditorAction;
use crate::core::sql::editor::sql_editor_controller::SqlEditorController;
use crate::{
    CompletionItemData, CompletionKind as UiCompletionKind,
    DiagnosticSeverity as UiDiagnosticSeverity, EditorDiagnosticData, EditorLineData,
    EditorStateData, HighlightSpanData, SyntaxKind as UiSyntaxKind,
};
use slint::ModelRc;
use std::rc::Rc;

/// Backs an array field of a Slint struct with a `VecModel`.
fn to_model<T: Clone + 'static>(items: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(slint::VecModel::from(items)))
}

/// Spaces the Tab key inserts when the popup is closed.
const INDENT: &str = "  ";

/// Provider used until a connection is picked.
const FALLBACK_PROVIDER: DatabaseProvider = DatabaseProvider::PostgreSql;

/// Owns the editor controller and translates UI intents into editor actions.
///
/// The component only reports *what* the user did, using a small string
/// grammar, so the editor logic never sees a key code and stays testable
/// without a window.
pub struct EditorBridge {
    controller: SqlEditorController,
}

impl EditorBridge {
    pub fn new(text: &str, catalog: SchemaCatalog) -> Self {
        let provider = catalog.provider().unwrap_or(FALLBACK_PROVIDER);
        let mut bridge = Self {
            controller: SqlEditorController::new(text, dialect_for_provider(provider), provider),
        };
        bridge.controller.set_catalog(catalog);
        bridge
    }

    pub fn controller(&self) -> &SqlEditorController {
        &self.controller
    }

    /// Text a run should execute: the selection, the statement under the
    /// caret, or the whole document.
    pub fn execution_target(&self) -> String {
        self.controller.execution_target()
    }

    /// Handles one intent coming from the UI.
    ///
    /// Returns `true` when the state may have changed and the caller has to
    /// push a new projection.
    pub fn dispatch(&mut self, intent: &str) -> bool {
        if let Some((rows, columns)) = viewport_action(intent) {
            self.controller.set_viewport(rows, columns);
            return true;
        }
        if let Some((rows, columns)) = scroll_action(intent) {
            self.controller.scroll_by(columns, rows);
            return true;
        }

        match intent_action(intent) {
            Some(action) => {
                self.controller.apply(action);
                true
            }
            None => false,
        }
    }

    /// Replaces the whole buffer, e.g. after a file was opened.
    pub fn set_text(&mut self, text: &str) {
        self.controller
            .apply(EditorAction::ReplaceAll(text.to_string()));
    }

    /// Connects the editor to another database, which also switches dialect.
    pub fn set_catalog(&mut self, catalog: SchemaCatalog) {
        self.controller.set_catalog(catalog);
    }

    /// Projects the controller state into the models the component draws.
    pub fn state(&self) -> EditorStateData {
        let state = self.controller.state();
        let document = self.controller.document();
        let selection = state.selection.normalized_range();
        let caret = state.cursor;
        let anchor_column = caret.column.saturating_sub(state.completion_offset);
        let (status_line, status_column) = state.cursor_position();

        EditorStateData {
            lines: to_model(
                state
                    .lines
                    .iter()
                    .map(|line| EditorLineData {
                        row: line.row as i32,
                        length: line.length as i32,
                    })
                    .collect(),
            ),
            spans: to_model(
                state
                    .spans
                    .iter()
                    .map(|span| HighlightSpanData {
                        row: span.row as i32,
                        column: span.column as i32,
                        length: span.length as i32,
                        kind: ui_syntax_kind(span.kind),
                        text: span_text(document, *span).into(),
                    })
                    .collect(),
            ),
            diagnostics: to_model(
                state
                    .diagnostics
                    .iter()
                    .map(|diagnostic| EditorDiagnosticData {
                        severity: ui_diagnostic_severity(diagnostic.severity),
                        message: diagnostic.message.clone().into(),
                        row: diagnostic.range.start.line as i32,
                        column: diagnostic.range.start.column as i32,
                        length: range_width(document, diagnostic.range),
                    })
                    .collect(),
            ),
            completions: to_model(
                state
                    .completions
                    .iter()
                    .map(|item| CompletionItemData {
                        label: item.label.clone().into(),
                        detail: item.detail.clone().into(),
                        kind: ui_completion_kind(item.kind),
                        insert_text: item.label.clone().into(),
                    })
                    .collect(),
            ),
            completion_visible: state.completion_visible,
            completion_selected: state.completion_selected as i32,
            completion_prefix: prefix_text(document, caret, state.completion_offset).into(),
            completion_anchor_row: caret.line as i32,
            completion_anchor_column: anchor_column as i32,
            caret_row: caret.line as i32,
            caret_column: caret.column as i32,
            selection_start_row: selection.start.line as i32,
            selection_start_column: selection.start.column as i32,
            selection_end_row: selection.end.line as i32,
            selection_end_column: selection.end.column as i32,
            has_selection: state.has_selection(),
            scroll_row: state.scroll_y as i32,
            scroll_column: state.scroll_x as i32,
            can_undo: state.can_undo,
            can_redo: state.can_redo,
            error_count: state.error_count as i32,
            warning_count: state.warning_count as i32,
            status_text: format!(
                "{}  ·  {}  ·  Ln {}, Col {}  ·  {} error(s), {} warning(s)",
                state.database_name,
                state.dialect_name,
                status_line,
                status_column,
                state.error_count,
                state.warning_count,
            )
            .into(),
        }
    }
}

/// Text of a highlight span, read from the document the span points into.
fn span_text(
    document: &TextDocument,
    span: crate::core::sql::analysis::highlight_span::HighlightSpan,
) -> String {
    document
        .line_text_at(TextPosition::new(span.row as usize, span.column as usize))
        .chars()
        .skip(span.column as usize)
        .take(span.length as usize)
        .collect()
}

/// How many characters a range covers, at least one so it stays visible.
fn range_width(document: &TextDocument, range: TextRange) -> i32 {
    (document.text_in(range).chars().count() as i32).max(1)
}

/// The prefix the popup is filtering on: the text between the start of the
/// replaced word and the caret.
fn prefix_text(document: &TextDocument, caret: TextPosition, length: usize) -> String {
    let start = caret.column.saturating_sub(length);
    document
        .line_text_at(caret)
        .chars()
        .skip(start)
        .take(caret.column - start)
        .collect()
}

/// Parses the intent grammar the component emits.
///
/// `text:` is the only intent with a free form payload, so its text is taken
/// verbatim and nothing may follow it.
fn intent_action(intent: &str) -> Option<EditorAction> {
    let (name, payload) = match intent.split_once(':') {
        Some((name, payload)) => (name, Some(payload)),
        None => (intent, None),
    };

    let action = match (name, payload) {
        ("text", payload) => EditorAction::InsertText(payload.unwrap_or_default().to_string()),
        ("newline", _) => EditorAction::NewLine,
        ("backspace", _) => EditorAction::Backspace,
        ("delete", _) => EditorAction::Delete,
        ("delete-word-backward", _) => EditorAction::DeleteWordBackward,
        ("indent", _) => EditorAction::InsertText(INDENT.to_string()),
        ("left", _) => EditorAction::MoveLeft { select: false },
        ("select-left", _) => EditorAction::MoveLeft { select: true },
        ("right", _) => EditorAction::MoveRight { select: false },
        ("select-right", _) => EditorAction::MoveRight { select: true },
        ("up", _) => EditorAction::MoveUp { select: false },
        ("select-up", _) => EditorAction::MoveUp { select: true },
        ("down", _) => EditorAction::MoveDown { select: false },
        ("select-down", _) => EditorAction::MoveDown { select: true },
        ("page-up", _) => EditorAction::MovePageUp { select: false },
        ("page-down", _) => EditorAction::MovePageDown { select: false },
        ("word-left", _) => EditorAction::MoveWordLeft { select: false },
        ("word-right", _) => EditorAction::MoveWordRight { select: false },
        ("line-start", _) => EditorAction::MoveToLineStart { select: false },
        ("line-end", _) => EditorAction::MoveToLineEnd { select: false },
        ("select-line-start", _) => EditorAction::MoveToLineStart { select: true },
        ("select-line-end", _) => EditorAction::MoveToLineEnd { select: true },
        ("doc-start", _) => EditorAction::MoveToDocumentStart { select: false },
        ("doc-end", _) => EditorAction::MoveToDocumentEnd { select: false },
        ("select-all", _) => EditorAction::SelectAll,
        ("undo", _) => EditorAction::Undo,
        ("redo", _) => EditorAction::Redo,
        ("complete-request", _) => EditorAction::RequestCompletion,
        ("complete-next", _) => EditorAction::CompleteNext,
        ("complete-prev", _) => EditorAction::CompletePrevious,
        ("complete-dismiss", _) => EditorAction::DismissCompletion,
        ("complete-accept", None) => EditorAction::AcceptCompletion,
        ("complete-accept", Some(index)) => EditorAction::AcceptCompletionAt(index.parse().ok()?),
        ("pointer", Some(position)) => point_action(position, false)?,
        ("drag", Some(position)) => point_action(position, true)?,
        _ => return None,
    };

    Some(action)
}

/// A `row:column` pair produced by a click or a drag.
fn point_action(payload: &str, extend: bool) -> Option<EditorAction> {
    let (row, column) = payload.split_once(':')?;
    Some(EditorAction::Select {
        position: TextPosition::new(row.parse().ok()?, column.parse().ok()?),
        extend,
    })
}

/// A `rows:columns` viewport update.
pub fn viewport_action(intent: &str) -> Option<(usize, usize)> {
    let payload = intent.strip_prefix("viewport:")?;
    let (rows, columns) = payload.split_once(':')?;
    Some((rows.parse().ok()?, columns.parse().ok()?))
}

/// A `rows:columns` scroll step.
pub fn scroll_action(intent: &str) -> Option<(i32, i32)> {
    let payload = intent.strip_prefix("scroll:")?;
    let (rows, columns) = payload.split_once(':')?;
    Some((rows.parse().ok()?, columns.parse().ok()?))
}

fn ui_syntax_kind(kind: TokenKind) -> UiSyntaxKind {
    match kind {
        TokenKind::Keyword => UiSyntaxKind::Keyword,
        TokenKind::Function => UiSyntaxKind::Function,
        TokenKind::String => UiSyntaxKind::String,
        TokenKind::QuotedIdentifier => UiSyntaxKind::QuotedIdentifier,
        TokenKind::Number => UiSyntaxKind::Number,
        TokenKind::Comment => UiSyntaxKind::Comment,
        TokenKind::Identifier => UiSyntaxKind::Identifier,
        TokenKind::Parameter => UiSyntaxKind::Parameter,
        TokenKind::Operator => UiSyntaxKind::Operator,
        TokenKind::Punctuation => UiSyntaxKind::Punctuation,
        TokenKind::Whitespace => UiSyntaxKind::Whitespace,
        TokenKind::Unknown => UiSyntaxKind::Unknown,
    }
}

fn ui_diagnostic_severity(severity: DiagnosticSeverity) -> UiDiagnosticSeverity {
    match severity {
        DiagnosticSeverity::Error => UiDiagnosticSeverity::Error,
        DiagnosticSeverity::Warning => UiDiagnosticSeverity::Warning,
    }
}

fn ui_completion_kind(kind: CompletionKind) -> UiCompletionKind {
    match kind {
        CompletionKind::Schema => UiCompletionKind::Schema,
        CompletionKind::Table => UiCompletionKind::Table,
        CompletionKind::View => UiCompletionKind::View,
        CompletionKind::Column => UiCompletionKind::Column,
        CompletionKind::Function => UiCompletionKind::Function,
        CompletionKind::Keyword => UiCompletionKind::Keyword,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slint::Model;

    fn bridge() -> EditorBridge {
        EditorBridge::new("SELECT ", SchemaCatalog::empty())
    }

    #[test]
    fn test_text_intent_keeps_the_payload_verbatim() {
        let mut bridge = bridge();

        assert!(bridge.dispatch("text:a"), "an edit changes the state");
        assert_eq!(bridge.controller().text(), "SELECT a");
    }

    #[test]
    fn test_text_intent_keeps_colons_and_spaces() {
        let mut bridge = bridge();

        bridge.dispatch("text:a: b");

        assert_eq!(bridge.controller().text(), "SELECT a: b");
    }

    #[test]
    fn test_navigation_intents_move_the_caret() {
        let mut bridge = bridge();

        bridge.dispatch("left");
        bridge.dispatch("left");

        assert_eq!(bridge.controller().cursor(), TextPosition::new(0, 5));
    }

    #[test]
    fn test_a_drag_extends_the_selection() {
        let mut bridge = bridge();

        bridge.dispatch("pointer:0:0");
        bridge.dispatch("drag:0:6");

        assert_eq!(
            bridge.controller().selection().replacement_range().start,
            TextPosition::new(0, 0)
        );
        assert_eq!(bridge.controller().execution_target(), "SELECT");
    }

    #[test]
    fn test_a_click_closes_the_popup_and_collapses_the_selection() {
        let mut bridge = bridge();

        bridge.dispatch("select-all");
        bridge.dispatch("pointer:0:3");

        assert!(!bridge.state().has_selection);
        assert!(!bridge.state().completion_visible);
    }

    #[test]
    fn test_undo_and_redo_are_reachable_from_the_ui() {
        let mut bridge = bridge();

        bridge.dispatch("text:x");
        bridge.dispatch("undo");
        assert_eq!(bridge.controller().text(), "SELECT ");

        bridge.dispatch("redo");
        assert_eq!(bridge.controller().text(), "SELECT x");
    }

    #[test]
    fn test_a_click_accepts_the_completion_item_it_names() {
        let mut bridge = EditorBridge::new("SELECT u", SchemaCatalog::empty());

        bridge.dispatch("complete-request");
        let state = bridge.state();
        assert!(state.completion_visible, "the popup is open");
        let label = state
            .completions
            .iter()
            .nth(1)
            .expect("the second item is offered")
            .label
            .clone();

        bridge.dispatch(&format!("complete-accept:{}", 1));

        assert_eq!(bridge.controller().text(), format!("SELECT {label}"));
    }

    #[test]
    fn test_an_unknown_intent_is_ignored() {
        let mut bridge = bridge();

        assert!(!bridge.dispatch("none"), "nothing changed");
        assert!(!bridge.dispatch("teleport:1:2"), "nothing changed");
        assert_eq!(bridge.controller().text(), "SELECT ");
    }

    #[test]
    fn test_viewport_and_scroll_intents_reach_the_controller() {
        let mut bridge = EditorBridge::new(&"a\n".repeat(80), SchemaCatalog::empty());

        bridge.dispatch("pointer:0:0");
        assert!(bridge.dispatch("viewport:10:40"), "the viewport changed");
        assert_eq!(
            bridge.state().scroll_row,
            0,
            "the caret keeps the top visible"
        );

        bridge.dispatch("scroll:5:0");
        assert_eq!(
            bridge.controller().scroll(),
            (0, 5),
            "the view scrolled down"
        );

        bridge.dispatch("scroll:-100:0");
        assert_eq!(
            bridge.controller().scroll(),
            (0, 0),
            "scrolling stops at the top"
        );

        assert_eq!(viewport_action("viewport:12:64"), Some((12, 64)));
        assert_eq!(scroll_action("scroll:-2:5"), Some((-2, 5)));
    }

    #[test]
    fn test_the_projection_carries_the_span_text() {
        let mut bridge = bridge();

        bridge.dispatch("text:1");

        let state = bridge.state();
        let first = state
            .spans
            .iter()
            .next()
            .expect("the keyword is highlighted");

        assert_eq!(first.text, "SELECT");
        assert_eq!(first.kind, UiSyntaxKind::Keyword);
    }

    #[test]
    fn test_the_projection_normalizes_the_selection() {
        let mut bridge = bridge();

        bridge.dispatch("select-all");
        let state = bridge.state();

        assert!(state.has_selection);
        assert_eq!(state.selection_start_column, 0);
        assert_eq!(state.selection_end_column, 7);
    }

    #[test]
    fn test_the_status_bar_reports_the_dialect_and_the_caret() {
        let bridge = bridge();

        let state = bridge.state();

        assert!(
            state.status_text.contains("Ln 1, Col 8"),
            "unexpected status bar: {:?}",
            state.status_text.as_str(),
        );
    }

    #[test]
    fn test_the_execution_target_is_the_statement_under_the_caret() {
        let mut bridge = EditorBridge::new("SELECT 1;\nSELECT 2;\n", SchemaCatalog::empty());

        bridge.dispatch("pointer:1:8");

        assert_eq!(bridge.execution_target(), "SELECT 2");
    }
}
