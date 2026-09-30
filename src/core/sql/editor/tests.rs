use crate::core::entities::column_info::ColumnInfo;
use crate::core::entities::database_info::DatabaseInfo;
use crate::core::entities::database_provider::DatabaseProvider;
use crate::core::entities::relation_kind::RelationKind;
use crate::core::entities::schema_info::SchemaInfo;
use crate::core::entities::table_info::TableInfo;
use crate::core::sql::analysis::token_kind::TokenKind;
use crate::core::sql::catalog::schema_catalog::SchemaCatalog;
use crate::core::sql::dialect::postgres_dialect::PostgreSqlDialect;
use crate::core::sql::document::text_position::TextPosition;
use crate::core::sql::document::text_range::TextRange;
use crate::core::sql::editor::editor_action::EditorAction;
use crate::core::sql::editor::editor_state::EditorState;
use crate::core::sql::editor::sql_editor_controller::SqlEditorController;

fn controller(text: &str) -> SqlEditorController {
    SqlEditorController::new(
        text,
        Box::new(PostgreSqlDialect::new()),
        DatabaseProvider::PostgreSql,
    )
}

fn connected(text: &str) -> SqlEditorController {
    let mut controller = controller(text);
    controller.set_catalog(SchemaCatalog::from_database(DatabaseInfo {
        name: "warehouse".to_string(),
        provider: DatabaseProvider::PostgreSql,
        expanded: false,
        schemas: vec![SchemaInfo {
            name: "public".to_string(),
            expanded: false,
            tables: vec![TableInfo {
                name: "users".to_string(),
                kind: RelationKind::Table,
                columns: vec![
                    ColumnInfo {
                        name: "id".to_string(),
                        data_type: "integer".to_string(),
                        is_primary_key: true,
                        is_nullable: false,
                    },
                    ColumnInfo {
                        name: "email".to_string(),
                        data_type: "text".to_string(),
                        is_primary_key: false,
                        is_nullable: true,
                    },
                ],
                expanded: false,
            }],
        }],
    }));
    controller
}

fn at(line: usize, column: usize) -> TextPosition {
    TextPosition::new(line, column)
}

fn type_text(controller: &mut SqlEditorController, text: &str) {
    for ch in text.chars() {
        controller.apply(EditorAction::Insert(ch));
    }
}

fn labels(state: &EditorState) -> Vec<&str> {
    state
        .completions
        .iter()
        .map(|item| item.label.as_str())
        .collect()
}

#[test]
fn test_a_new_controller_starts_at_the_end() {
    let controller = controller("SELECT 1");

    assert_eq!(controller.cursor(), at(0, 8));
    assert_eq!(controller.dialect_name(), "PostgreSQL");
    assert!(!controller.analysis().spans.is_empty());
}

#[test]
fn test_typing_appends_at_the_caret() {
    let mut controller = controller("SELECT ");

    type_text(&mut controller, "1");

    assert_eq!(controller.text(), "SELECT 1");
    assert_eq!(controller.cursor(), at(0, 8));
    assert!(
        controller.apply(EditorAction::SelectAll).can_undo,
        "typing is undoable"
    );
}

#[test]
fn test_typing_updates_the_analysis() {
    let mut controller = controller("");

    type_text(&mut controller, "SELECT * FROM users");

    let analysis = controller.analysis();
    assert_eq!(analysis.lines.len(), 1);
    assert_eq!(analysis.statements.len(), 1);
    assert!(
        analysis
            .spans
            .iter()
            .any(|span| span.kind == TokenKind::Keyword)
    );
}

#[test]
fn test_backspace_removes_the_character_before_the_caret() {
    let mut controller = controller("SELECT 1");

    controller.apply(EditorAction::Backspace);

    assert_eq!(controller.text(), "SELECT ");
    assert_eq!(controller.cursor(), at(0, 7));
}

#[test]
fn test_backspace_at_the_start_does_nothing() {
    let mut controller = controller("a");
    controller.apply(EditorAction::SetCursor(at(0, 0)));

    controller.apply(EditorAction::Backspace);

    assert_eq!(controller.text(), "a");
}

#[test]
fn test_backspace_joins_two_lines() {
    let mut controller = controller("ab\ncd");
    controller.apply(EditorAction::SetCursor(at(1, 0)));

    controller.apply(EditorAction::Backspace);

    assert_eq!(controller.text(), "abcd");
    assert_eq!(controller.cursor(), at(0, 2));
}

#[test]
fn test_delete_removes_the_character_after_the_caret() {
    let mut controller = controller("abcd");
    controller.apply(EditorAction::SetCursor(at(0, 1)));

    controller.apply(EditorAction::Delete);

    assert_eq!(controller.text(), "acd");
    assert_eq!(controller.cursor(), at(0, 1));
}

#[test]
fn test_typing_replaces_the_selection() {
    let mut controller = controller("SELECT old");
    controller.apply(EditorAction::SelectAll);

    type_text(&mut controller, "new");

    assert_eq!(controller.text(), "new");
    assert_eq!(controller.cursor(), at(0, 3));
}

#[test]
fn test_undo_restores_the_previous_text() {
    let mut controller = controller("SELECT 1");
    controller.apply(EditorAction::InsertText(" 2".to_string()));

    let state = controller.apply(EditorAction::Undo);

    assert_eq!(controller.text(), "SELECT 1");
    assert!(!state.can_undo, "the history is empty again");
}

#[test]
fn test_undo_removes_a_whole_typing_run() {
    let mut controller = controller("");

    type_text(&mut controller, "SELECT");
    controller.apply(EditorAction::Undo);

    assert_eq!(controller.text(), "", "typing is undone in one step");
}

#[test]
fn test_redo_reapplies_the_edit() {
    let mut controller = controller("");
    type_text(&mut controller, "SELECT");
    controller.apply(EditorAction::Undo);

    let state = controller.apply(EditorAction::Redo);

    assert_eq!(controller.text(), "SELECT");
    assert!(state.can_undo, "the edit is back on the undo stack");
    assert!(!state.can_redo, "the redo stack is empty again");
}

#[test]
fn test_undo_of_an_empty_history_is_harmless() {
    let mut controller = controller("SELECT 1");

    controller.apply(EditorAction::Undo);
    controller.apply(EditorAction::Redo);

    assert_eq!(controller.text(), "SELECT 1");
}

#[test]
fn test_cursor_moves_by_character() {
    let mut controller = controller("abc");
    controller.apply(EditorAction::SetCursor(at(0, 1)));

    controller.apply(EditorAction::MoveLeft { select: false });
    assert_eq!(controller.cursor(), at(0, 0));

    controller.apply(EditorAction::MoveRight { select: false });
    controller.apply(EditorAction::MoveRight { select: false });
    assert_eq!(controller.cursor(), at(0, 2));
}

#[test]
fn test_cursor_moves_across_lines() {
    let mut controller = controller("ab\ncd");
    controller.apply(EditorAction::SetCursor(at(0, 2)));

    controller.apply(EditorAction::MoveDown { select: false });
    assert_eq!(controller.cursor(), at(1, 2));

    controller.apply(EditorAction::MoveUp { select: false });
    assert_eq!(controller.cursor(), at(0, 2));
}

#[test]
fn test_cursor_clamps_to_the_shortest_line() {
    let mut controller = controller("abcdef\nab");
    controller.apply(EditorAction::SetCursor(at(0, 6)));

    controller.apply(EditorAction::MoveDown { select: false });

    assert_eq!(
        controller.cursor(),
        at(1, 2),
        "the column is clamped to the line"
    );
}

#[test]
fn test_line_home_and_end() {
    let mut controller = controller("hello world");

    controller.apply(EditorAction::MoveToLineEnd { select: false });
    assert_eq!(controller.cursor(), at(0, 11));

    controller.apply(EditorAction::MoveToLineStart { select: false });
    assert_eq!(controller.cursor(), at(0, 0));
}

#[test]
fn test_document_home_and_end() {
    let mut controller = controller("one\ntwo");
    controller.apply(EditorAction::SetCursor(at(0, 1)));

    controller.apply(EditorAction::MoveToDocumentEnd { select: false });
    assert_eq!(controller.cursor(), at(1, 3));

    controller.apply(EditorAction::MoveToDocumentStart { select: false });
    assert_eq!(controller.cursor(), at(0, 0));
}

#[test]
fn test_word_movement_skips_separators() {
    let mut controller = controller("alpha beta");
    controller.apply(EditorAction::SetCursor(at(0, 0)));

    controller.apply(EditorAction::MoveWordRight { select: false });
    assert_eq!(controller.cursor(), at(0, 5), "end of the first word");

    controller.apply(EditorAction::MoveWordRight { select: false });
    assert_eq!(controller.cursor(), at(0, 10), "end of the second word");

    controller.apply(EditorAction::MoveWordLeft { select: false });
    assert_eq!(controller.cursor(), at(0, 6), "start of the second word");

    controller.apply(EditorAction::MoveWordLeft { select: false });
    assert_eq!(controller.cursor(), at(0, 0), "start of the first word");
}

#[test]
fn test_delete_word_backward() {
    let mut controller = controller("SELECT * FROM users");
    controller.apply(EditorAction::SetCursor(at(0, 19)));

    controller.apply(EditorAction::DeleteWordBackward);

    assert_eq!(controller.text(), "SELECT * FROM ");
}

#[test]
fn test_shift_moves_extend_the_selection() {
    let mut controller = controller("abc");

    controller.apply(EditorAction::SetCursor(at(0, 0)));
    controller.apply(EditorAction::MoveRight { select: true });
    controller.apply(EditorAction::MoveRight { select: true });
    let state = controller.apply(EditorAction::MoveRight { select: true });

    assert!(state.has_selection());
    assert_eq!(
        state.selection.normalized_range(),
        TextRange::new(at(0, 0), at(0, 3))
    );
}

#[test]
fn test_a_horizontal_move_collapses_the_selection() {
    let mut controller = controller("abc");
    controller.apply(EditorAction::SelectAll);

    controller.apply(EditorAction::MoveLeft { select: false });

    assert_eq!(controller.cursor(), at(0, 0), "collapsed to the start");
    assert!(controller.selection().is_empty());
}

#[test]
fn test_select_all_then_shift_moves_extend_from_the_original_anchor() {
    let mut controller = controller("abcdef");
    controller.apply(EditorAction::SelectAll);
    controller.apply(EditorAction::SetCursor(at(0, 0)));

    let state = controller.apply(EditorAction::MoveRight { select: true });

    assert_eq!(state.selection.anchor, at(0, 0));
    assert_eq!(state.selection.focus, at(0, 1));
}

#[test]
fn test_clicking_moves_the_caret_and_extending_selects() {
    let mut controller = controller("abc");
    controller.apply(EditorAction::Select {
        position: at(0, 1),
        extend: false,
    });
    assert_eq!(controller.cursor(), at(0, 1));

    let state = controller.apply(EditorAction::Select {
        position: at(0, 3),
        extend: true,
    });

    assert_eq!(state.selection.anchor, at(0, 1));
    assert_eq!(state.selection.focus, at(0, 3));
}

#[test]
fn test_a_newline_splits_the_line_and_keeps_the_column() {
    let mut controller = controller("SELECT users");

    controller.apply(EditorAction::SetCursor(at(0, 7)));
    controller.apply(EditorAction::NewLine);

    assert_eq!(controller.text(), "SELECT \nusers");
    assert_eq!(controller.cursor(), at(1, 0));
}

#[test]
fn test_replace_all_resets_the_history() {
    let mut controller = controller("SELECT 1");
    type_text(&mut controller, "2");

    let state = controller.apply(EditorAction::ReplaceAll("SELECT 3".to_string()));

    assert_eq!(controller.text(), "SELECT 3");
    assert!(!state.can_undo, "replacing everything clears the history");
}

#[test]
fn test_execution_target_prefers_the_selection() {
    let mut controller = controller("SELECT 1; SELECT 2");
    controller.apply(EditorAction::Select {
        position: at(0, 0),
        extend: false,
    });
    let state = controller.apply(EditorAction::Select {
        position: at(0, 8),
        extend: true,
    });
    assert!(state.has_selection());

    assert_eq!(controller.execution_target(), "SELECT 1");
}

#[test]
fn test_execution_target_uses_the_statement_under_the_caret() {
    let mut controller = controller("SELECT 1; SELECT 2");
    controller.apply(EditorAction::SetCursor(at(0, 11)));

    assert_eq!(
        controller.execution_target(),
        "SELECT 2",
        "the statement runs up to the token before the separator"
    );
}

#[test]
fn test_execution_target_falls_back_to_the_whole_document() {
    let controller = controller("   ");

    assert_eq!(controller.execution_target(), "   ");
}

#[test]
fn test_the_state_reports_the_connection() {
    let state = connected("SELECT 1").apply(EditorAction::SetCursor(at(0, 0)));

    assert_eq!(state.database_name, "warehouse");
    assert_eq!(state.dialect_name, "PostgreSQL");
}

#[test]
fn test_the_state_without_a_connection_says_so() {
    let state = controller("SELECT 1").apply(EditorAction::SetCursor(at(0, 0)));

    assert_eq!(state.database_name, "no connection");
}

#[test]
fn test_connecting_switches_the_dialect() {
    let mut controller = controller("");

    controller.set_catalog(SchemaCatalog::from_database(DatabaseInfo {
        name: "legacy".to_string(),
        provider: DatabaseProvider::Firebird,
        expanded: false,
        schemas: Vec::new(),
    }));

    assert_eq!(controller.provider(), DatabaseProvider::Firebird);
    assert_eq!(controller.dialect_name(), "Firebird");
}

#[test]
fn test_completion_is_offered_on_request() {
    let mut controller = connected("SELECT * FROM users WHERE ");
    controller.apply(EditorAction::SetCursor(at(0, 26)));

    let state = controller.apply(EditorAction::RequestCompletion);

    assert!(state.completion_visible);
    assert!(labels(&state).contains(&"id"));
    assert!(labels(&state).contains(&"email"));
}

#[test]
fn test_completion_is_hidden_when_nothing_matches() {
    let mut controller = connected("SELECT * FROM users WHERE zzz");
    controller.apply(EditorAction::SetCursor(at(0, 29)));

    let state = controller.apply(EditorAction::RequestCompletion);

    assert!(!state.completion_visible);
    assert!(state.completions.is_empty());
}

#[test]
fn test_completion_follows_the_caret_while_typing() {
    let mut controller = connected("SELECT * FROM users WHERE ");
    controller.apply(EditorAction::RequestCompletion);

    type_text(&mut controller, "em");

    let state = controller.apply(EditorAction::RequestCompletion);
    assert!(state.completion_visible);
    assert_eq!(state.completions[0].label, "email");
}

#[test]
fn test_completion_is_closed_after_a_moving_action() {
    let mut controller = connected("SELECT * FROM users WHERE ");
    assert!(
        controller
            .apply(EditorAction::RequestCompletion)
            .completion_visible
    );

    let state = controller.apply(EditorAction::MoveLeft { select: false });

    assert!(
        !state.completion_visible,
        "the popup does not survive a caret move"
    );
}

#[test]
fn test_accepting_a_completion_inserts_the_column() {
    let mut controller = connected("SELECT * FROM users WHERE ");
    controller.apply(EditorAction::RequestCompletion);
    controller.apply(EditorAction::CompleteNext);

    controller.apply(EditorAction::AcceptCompletion);

    assert_eq!(controller.text(), "SELECT * FROM users WHERE email");
}

#[test]
fn test_accepting_a_completion_replaces_the_typed_prefix() {
    let mut controller = connected("SELECT * FROM users WHERE ema");
    controller.apply(EditorAction::SetCursor(at(0, 29)));
    controller.apply(EditorAction::RequestCompletion);

    controller.apply(EditorAction::AcceptCompletion);

    assert_eq!(controller.text(), "SELECT * FROM users WHERE email");
}

#[test]
fn test_dismissing_the_completion_keeps_the_text() {
    let mut controller = connected("SELECT * FROM users WHERE ");
    controller.apply(EditorAction::RequestCompletion);

    let state = controller.apply(EditorAction::DismissCompletion);

    assert!(!state.completion_visible);
    assert_eq!(controller.text(), "SELECT * FROM users WHERE ");
}

#[test]
fn test_escape_does_not_type_a_character() {
    let mut controller = connected("SELECT * FROM users WHERE ");

    controller.apply(EditorAction::DismissCompletion);

    assert_eq!(controller.text(), "SELECT * FROM users WHERE ");
}

#[test]
fn test_navigating_the_popup_does_not_change_the_document() {
    let mut controller = connected("SELECT * FROM users WHERE ");
    controller.apply(EditorAction::RequestCompletion);

    let state = controller.apply(EditorAction::CompleteNext);

    assert!(state.completion_visible);
    assert_eq!(state.completion_selected, 1);
    assert_eq!(controller.text(), "SELECT * FROM users WHERE ");
}

#[test]
fn test_the_state_reports_diagnostics() {
    let mut controller = controller("SELECT FROM");

    let state = controller.apply(EditorAction::SetCursor(at(0, 0)));

    assert!(
        state.error_count > 0 || state.warning_count > 0,
        "a broken statement is reported"
    );
    assert!(!state.diagnostics_on_row(0).is_empty());
}

#[test]
fn test_a_valid_statement_has_no_diagnostics() {
    let mut controller = controller("SELECT 1;");

    let state = controller.apply(EditorAction::SetCursor(at(0, 0)));

    assert_eq!(state.error_count, 0);
    assert_eq!(state.warning_count, 0);
    assert_eq!(state.statement_count, 1);
}

#[test]
fn test_the_state_reports_the_scroll_offset() {
    let mut controller = controller("SELECT 1");
    controller.apply(EditorAction::SetCursor(at(0, 0)));

    let state = controller.apply(EditorAction::SetCursor(at(0, 0)));

    assert_eq!(state.scroll_x, 0);
    assert_eq!(state.scroll_y, 0);
}

#[test]
fn test_the_caret_scrolls_into_a_small_viewport() {
    let mut controller = controller("one\ntwo\nthree\nfour\nfive");
    controller.set_viewport(2, 80);
    controller.apply(EditorAction::SetCursor(at(0, 0)));

    controller.apply(EditorAction::SetCursor(at(4, 0)));

    let state = controller.apply(EditorAction::SetCursor(at(4, 0)));
    assert!(
        state.scroll_y > 0,
        "the caret row is below the viewport and has scrolled into view"
    );
}

#[test]
fn test_scrolling_is_clamped_to_the_document() {
    let mut controller = controller("one\ntwo");
    controller.set_viewport(1, 80);

    controller.scroll_by(0, 50);

    assert_eq!(
        controller.scroll().1,
        1,
        "there is nothing below the last line"
    );
}

#[test]
fn test_scrolling_left_is_clamped_to_the_start_of_the_line() {
    let mut controller = controller("short");
    controller.set_viewport(10, 2);

    controller.scroll_by(-50, 0);

    assert_eq!(controller.scroll().0, 0);
}

#[test]
fn test_the_caret_is_never_pushed_out_of_view_horizontally() {
    let mut controller = controller("SELECT a, b, c, d, e, f, g, h, i, j");
    controller.set_viewport(10, 6);
    controller.apply(EditorAction::SetCursor(at(0, 0)));

    controller.apply(EditorAction::SetCursor(at(0, 24)));

    let (scroll_x, _) = controller.scroll();
    let state = controller.apply(EditorAction::SetCursor(at(0, 24)));
    let window = scroll_x..scroll_x + state.visible_columns;
    assert!(
        window.contains(&state.cursor.column),
        "column {} is inside the visible window {window:?}",
        state.cursor.column
    );
}

#[test]
fn test_the_caret_scrolls_back_to_the_left_of_a_line() {
    let mut controller = controller("SELECT a, b, c, d, e, f, g, h, i, j");
    controller.set_viewport(10, 6);
    controller.apply(EditorAction::SetCursor(at(0, 24)));
    assert!(controller.scroll().0 > 0, "the line was scrolled right");

    controller.apply(EditorAction::SetCursor(at(0, 0)));

    assert_eq!(
        controller.scroll().0,
        0,
        "the line is scrolled back to the start"
    );
}

#[test]
fn test_the_end_of_a_line_does_not_scroll_past_the_text() {
    let mut controller = controller("SELECT 1");
    controller.set_viewport(10, 4);
    controller.apply(EditorAction::SetCursor(at(0, 8)));

    assert_eq!(
        controller.scroll().0,
        4,
        "there is nothing left to scroll, the caret rests on the last column"
    );
}
