use crate::core::entities::database_provider::DatabaseProvider;
use crate::core::sql::analysis::sql_analysis::SqlAnalysis;
use crate::core::sql::analysis::sql_analyzer::SqlAnalyzer;
use crate::core::sql::catalog::schema_catalog::SchemaCatalog;
use crate::core::sql::completion::completion_engine::CompletionEngine;
use crate::core::sql::completion::completion_result::CompletionResult;
use crate::core::sql::dialect::dialect_for_provider;
use crate::core::sql::dialect::sql_dialect::SqlDialect;
use crate::core::sql::document::edit_entry::EditEntry;
use crate::core::sql::document::edit_history::EditHistory;
use crate::core::sql::document::text_document::TextDocument;
use crate::core::sql::document::text_position::TextPosition;
use crate::core::sql::document::text_range::TextRange;
use crate::core::sql::document::text_selection::TextSelection;
use crate::core::sql::editor::editor_action::EditorAction;
use crate::core::sql::editor::editor_state::EditorState;

/// Number of undo steps kept in memory.
const HISTORY_LIMIT: usize = 500;

/// Owns the document and every language service that reads it.
///
/// All mutations go through [`SqlEditorController::apply`], which re-runs the
/// analysis once and refreshes the completion, so the UI only ever observes
/// consistent state.
pub struct SqlEditorController {
    document: TextDocument,
    selection: TextSelection,
    history: EditHistory,
    analysis: SqlAnalysis,
    catalog: SchemaCatalog,
    dialect: Box<dyn SqlDialect>,
    provider: DatabaseProvider,
    completion: CompletionResult,
    completion_visible: bool,
    scroll_x: usize,
    scroll_y: usize,
    visible_rows: usize,
    visible_columns: usize,
}

impl SqlEditorController {
    pub fn new(
        text: impl Into<String>,
        dialect: Box<dyn SqlDialect>,
        provider: DatabaseProvider,
    ) -> Self {
        let document = TextDocument::new(text);
        let mut controller = Self {
            selection: TextSelection::collapsed(document.end_position()),
            document,
            history: EditHistory::new(HISTORY_LIMIT),
            analysis: SqlAnalysis::default(),
            catalog: SchemaCatalog::empty(),
            dialect,
            provider,
            completion: CompletionResult::default(),
            completion_visible: false,
            scroll_x: 0,
            scroll_y: 0,
            visible_rows: 20,
            visible_columns: 80,
        };
        controller.refresh(false);
        controller
    }

    pub fn text(&self) -> &str {
        self.document.text()
    }

    pub fn document(&self) -> &TextDocument {
        &self.document
    }

    pub fn analysis(&self) -> &SqlAnalysis {
        &self.analysis
    }

    pub fn selection(&self) -> TextSelection {
        self.selection
    }

    pub fn cursor(&self) -> TextPosition {
        self.selection.focus
    }

    pub fn provider(&self) -> DatabaseProvider {
        self.provider
    }

    pub fn catalog(&self) -> &SchemaCatalog {
        &self.catalog
    }

    pub fn dialect_name(&self) -> &'static str {
        self.dialect.kind().display_name()
    }

    /// Connects the editor to a database, which also picks the dialect.
    pub fn set_catalog(&mut self, catalog: SchemaCatalog) {
        if let Some(provider) = catalog.provider() {
            self.provider = provider;
            self.dialect = dialect_for_provider(provider);
        }
        self.catalog = catalog;
        self.refresh(false);
    }

    /// Tells the controller how much of the editor is on screen, so the caret
    /// can be scrolled into view.
    pub fn set_viewport(&mut self, rows: usize, columns: usize) {
        self.visible_rows = rows.max(1);
        self.visible_columns = columns.max(1);
        self.scroll_into_view();
    }

    /// Current scroll offset, as `(column, row)`.
    pub fn scroll(&self) -> (usize, usize) {
        (self.scroll_x, self.scroll_y)
    }

    /// Scrolls the editor, clamped to the document.
    pub fn scroll_by(&mut self, dx: i32, dy: i32) {
        let max_x = self
            .document
            .line_length(self.scroll_y)
            .saturating_sub(self.visible_columns);
        let max_y = self.document.line_count().saturating_sub(1);
        self.scroll_x = (self.scroll_x as i32 + dx).clamp(0, max_x as i32) as usize;
        self.scroll_y = (self.scroll_y as i32 + dy).clamp(0, max_y as i32) as usize;
    }

    /// Applies an action and returns the refreshed state.
    pub fn apply(&mut self, action: EditorAction) -> EditorState {
        let popup_only = self.handle_completion(&action);
        if popup_only {
            return self.state();
        }

        if matches!(action, EditorAction::RequestCompletion) {
            self.refresh(true);
            return self.state();
        }

        // Typing keeps the popup alive and follows the caret; an explicit
        // request opens it even when the prefix is empty.
        let keep_open = action.wants_completion() || (self.completion_visible && action.is_edit());
        self.run(action);
        self.refresh(keep_open);
        self.state()
    }

    /// Text a run of the editor should execute.
    ///
    /// A selection wins, then the statement under the caret, and the whole
    /// document when the caret is not inside a statement.
    pub fn execution_target(&self) -> String {
        if !self.selection.is_empty() {
            return self
                .document
                .text_in(self.selection.normalized_range())
                .to_string();
        }
        match self.analysis.statement_at(self.cursor()) {
            Some(range) => self.document.text_in(range).to_string(),
            None => self.document.text().to_string(),
        }
    }

    /// Re-runs the analysis and recomputes the completion list.
    fn refresh(&mut self, completion_visible: bool) {
        self.analysis = SqlAnalyzer::analyze(self.document.text(), self.dialect.as_ref());
        self.completion = CompletionEngine::complete(
            &self.document,
            self.cursor(),
            &self.analysis,
            &self.catalog,
            self.dialect.as_ref(),
        );
        self.completion_visible = completion_visible && !self.completion.is_empty();
        if self.completion_visible {
            self.completion.selected = 0;
        }
        self.scroll_into_view();
    }

    /// Actions that only move the popup highlight, handled before the document
    /// is touched.
    fn handle_completion(&mut self, action: &EditorAction) -> bool {
        match action {
            EditorAction::CompleteNext if self.completion_visible => {
                self.completion.select_next();
                true
            }
            EditorAction::CompletePrevious if self.completion_visible => {
                self.completion.select_previous();
                true
            }
            EditorAction::AcceptCompletion if self.completion_visible => {
                self.accept_completion();
                self.refresh(false);
                true
            }
            EditorAction::AcceptCompletionAt(index) if self.completion_visible => {
                if self.completion.select(*index) {
                    self.accept_completion();
                    self.refresh(false);
                }
                true
            }
            EditorAction::DismissCompletion if self.completion_visible => {
                self.completion_visible = false;
                self.refresh(false);
                true
            }
            _ => false,
        }
    }

    /// Carries out an action on the document and the caret.
    fn run(&mut self, action: EditorAction) {
        match action {
            EditorAction::Insert(ch) => {
                let mut buffer = [0u8; 4];
                self.insert(ch.encode_utf8(&mut buffer).to_string().as_str(), true)
            }
            EditorAction::InsertText(text) => self.insert(&text, true),
            EditorAction::NewLine => self.insert("\n", true),
            EditorAction::Backspace => self.backspace(),
            EditorAction::Delete => self.delete(),
            EditorAction::DeleteWordBackward => self.delete_word_backward(),
            EditorAction::MoveLeft { select } => self.move_horizontal(-1, select),
            EditorAction::MoveRight { select } => self.move_horizontal(1, select),
            EditorAction::MoveUp { select } => self.move_vertical(-1, select),
            EditorAction::MoveDown { select } => self.move_vertical(1, select),
            EditorAction::MovePageUp { select } => {
                let delta = -(self.visible_rows as i32 - 1).max(1);
                self.move_vertical(delta, select)
            }
            EditorAction::MovePageDown { select } => {
                let delta = (self.visible_rows as i32 - 1).max(1);
                self.move_vertical(delta, select)
            }
            EditorAction::MoveWordLeft { select } => self.move_word(-1, select),
            EditorAction::MoveWordRight { select } => self.move_word(1, select),
            EditorAction::MoveToLineStart { select } => {
                let line = self.cursor().line;
                self.move_to(TextPosition::new(line, 0), select)
            }
            EditorAction::MoveToLineEnd { select } => {
                let line = self.cursor().line;
                self.move_to(
                    TextPosition::new(line, self.document.line_length(line)),
                    select,
                )
            }
            EditorAction::MoveToDocumentStart { select } => {
                self.move_to(TextPosition::default(), select)
            }
            EditorAction::MoveToDocumentEnd { select } => {
                let end = self.document.end_position();
                self.move_to(end, select)
            }
            EditorAction::SelectAll => {
                self.selection = TextSelection {
                    anchor: TextPosition::default(),
                    focus: self.document.end_position(),
                }
            }
            EditorAction::Undo => self.undo(),
            EditorAction::Redo => self.redo(),
            EditorAction::SetCursor(position) => {
                let position = self.document.clamp_position(position);
                self.selection = TextSelection::collapsed(position);
            }
            EditorAction::Select { position, extend } => {
                let position = self.document.clamp_position(position);
                self.selection = if extend {
                    TextSelection {
                        anchor: self.selection.anchor,
                        focus: position,
                    }
                } else {
                    TextSelection::collapsed(position)
                };
            }
            EditorAction::ReplaceAll(text) => {
                let range = TextRange::new(TextPosition::default(), self.document.end_position());
                self.replace(range, &text, false);
                self.history.clear();
            }
            EditorAction::RequestCompletion | EditorAction::UpdateCompletion => {}
            EditorAction::CompleteNext
            | EditorAction::CompletePrevious
            | EditorAction::AcceptCompletion
            | EditorAction::AcceptCompletionAt(_)
            | EditorAction::DismissCompletion => {}
        }
    }

    fn insert(&mut self, text: &str, typing: bool) {
        let range = self.selection.replacement_range();
        self.replace(range, text, typing);
    }

    fn backspace(&mut self) {
        let range = if self.selection.is_empty() {
            match previous_boundary(&self.document, self.cursor()) {
                Some(position) => TextRange::new(position, self.cursor()),
                None => return,
            }
        } else {
            self.selection.normalized_range()
        };
        self.replace(range, "", true);
    }

    fn delete(&mut self) {
        let range = if self.selection.is_empty() {
            match next_boundary(&self.document, self.cursor()) {
                Some(position) => TextRange::new(self.cursor(), position),
                None => return,
            }
        } else {
            self.selection.normalized_range()
        };
        self.replace(range, "", false);
    }

    fn delete_word_backward(&mut self) {
        let range = if self.selection.is_empty() {
            TextRange::new(
                word_start_backwards(&self.document, self.cursor()),
                self.cursor(),
            )
        } else {
            self.selection.normalized_range()
        };
        self.replace(range, "", false);
    }

    fn move_horizontal(&mut self, delta: i32, select: bool) {
        if !select && !self.selection.is_empty() {
            // A horizontal move collapses to the edge it moves towards.
            let range = self.selection.normalized_range();
            let target = if delta < 0 { range.start } else { range.end };
            self.selection = TextSelection::collapsed(target);
            return;
        }

        let target = next_position(&self.document, self.cursor(), delta);
        self.move_to(target, select);
    }

    fn move_vertical(&mut self, delta: i32, select: bool) {
        let column = self.cursor().column;
        let last_line = self.document.line_count().saturating_sub(1);
        let line = (self.cursor().line as i32 + delta).clamp(0, last_line as i32) as usize;
        let column = column.min(self.document.line_length(line));
        self.move_to(TextPosition::new(line, column), select)
    }

    fn move_word(&mut self, delta: i32, select: bool) {
        let target = if delta < 0 {
            word_start_backwards(&self.document, self.cursor())
        } else {
            word_end_forwards(&self.document, self.cursor())
        };
        self.move_to(target, select)
    }

    fn move_to(&mut self, position: TextPosition, select: bool) {
        let position = self.document.clamp_position(position);
        self.selection = if select {
            TextSelection {
                anchor: self.selection.anchor,
                focus: position,
            }
        } else {
            TextSelection::collapsed(position)
        };
    }

    /// Puts the text that was there before the last edit back.
    fn undo(&mut self) {
        let Some(entry) = self.history.undo() else {
            return;
        };
        let caret = entry.range.start;
        self.document
            .replace(entry.applied_range(), &entry.replaced);
        self.selection = TextSelection::collapsed(self.document.clamp_position(caret));
    }

    /// Applies the edit that was undone last.
    fn redo(&mut self) {
        let Some(entry) = self.history.redo() else {
            return;
        };
        let focus = self
            .document
            .replace(entry.replayed_range(), &entry.inserted);
        self.selection = TextSelection::collapsed(focus);
    }

    /// Replaces a range, records the edit and collapses the caret behind the
    /// inserted text.
    ///
    /// The entry is stored in the coordinates of the text before the edit, so
    /// undo and redo can both be replayed against it.
    fn replace(&mut self, range: TextRange, text: &str, typing: bool) {
        let replaced = self.document.text_in(range).to_string();
        let focus = self.document.replace(range, text);
        let entry = EditEntry::new(range, replaced, text, focus);

        if typing {
            self.history.record_typing(entry);
        } else {
            self.history.record(entry);
        }
        self.selection = TextSelection::collapsed(focus);
    }

    fn accept_completion(&mut self) {
        let Some(item) = self.completion.selected_item().cloned() else {
            return;
        };
        let range = self.completion.replace_range;
        self.replace(range, &item.label, false);
    }

    /// Brings the caret into view, keeping a little context around it so
    /// typing does not push the text away.
    fn scroll_into_view(&mut self) {
        let cursor = self.cursor();
        let margin = 3usize;

        if cursor.line + margin < self.scroll_y {
            self.scroll_y = cursor.line.saturating_sub(margin);
        } else if cursor.line + margin >= self.scroll_y + self.visible_rows {
            self.scroll_y = cursor.line + margin + 1 - self.visible_rows;
        }
        if cursor.column < self.scroll_x + margin {
            self.scroll_x = cursor.column.saturating_sub(margin);
        } else if cursor.column + margin >= self.scroll_x + self.visible_columns {
            self.scroll_x = cursor.column + margin + 1 - self.visible_columns;
        }

        let max_y = self.document.line_count().saturating_sub(1);
        let max_x = self
            .document
            .line_length(self.scroll_y)
            .saturating_sub(self.visible_columns);
        self.scroll_y = self.scroll_y.min(max_y);
        self.scroll_x = self.scroll_x.min(max_x);
    }

    /// Current view of the document, for a renderer or the tests.
    pub fn state(&self) -> EditorState {
        EditorState {
            spans: self.analysis.spans.clone(),
            lines: self.analysis.lines.clone(),
            diagnostics: self.analysis.diagnostics.clone(),
            completions: self.completion.items.clone(),
            completion_visible: self.completion_visible,
            completion_selected: self.completion.selected,
            completion_offset: self.completion.replace_range.start.column,
            scroll_x: self.scroll_x,
            scroll_y: self.scroll_y,
            visible_rows: self.visible_rows,
            visible_columns: self.visible_columns,
            cursor: self.cursor(),
            selection: self.selection,
            can_undo: self.history.can_undo(),
            can_redo: self.history.can_redo(),
            error_count: self.analysis.error_count(),
            warning_count: self.analysis.warning_count(),
            statement_count: self.analysis.statements.len(),
            dialect_name: self.dialect_name().to_string(),
            database_name: self
                .catalog
                .database_name()
                .unwrap_or("no connection")
                .to_string(),
        }
    }
}

/// One caret step, which never lands inside a multi byte character.
fn next_position(document: &TextDocument, from: TextPosition, delta: i32) -> TextPosition {
    if delta < 0 {
        previous_boundary(document, from).unwrap_or_default()
    } else {
        next_boundary(document, from).unwrap_or_else(|| document.end_position())
    }
}

fn previous_boundary(document: &TextDocument, position: TextPosition) -> Option<TextPosition> {
    if position.column > 0 {
        return Some(TextPosition::new(position.line, position.column - 1));
    }
    (position.line > 0).then(|| {
        let line = position.line - 1;
        TextPosition::new(line, document.line_length(line))
    })
}

fn next_boundary(document: &TextDocument, position: TextPosition) -> Option<TextPosition> {
    if position.column < document.line_length(position.line) {
        return Some(TextPosition::new(position.line, position.column + 1));
    }
    (position.line + 1 < document.line_count()).then(|| TextPosition::new(position.line + 1, 0))
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

/// Start of the word left of a position.
///
/// When the caret is inside a word the word itself is found; when it already
/// sits at the start of one, the separators in front of it are skipped so the
/// move continues into the word before, the way a double click does.
fn word_start_backwards(document: &TextDocument, from: TextPosition) -> TextPosition {
    let mut column = from.column;

    while column > 0 && char_at(document, from.line, column - 1).is_some_and(is_word_char) {
        column -= 1;
    }
    if column == from.column {
        while column > 0
            && char_at(document, from.line, column - 1).is_some_and(|ch| !is_word_char(ch))
        {
            column -= 1;
        }
        while column > 0 && char_at(document, from.line, column - 1).is_some_and(is_word_char) {
            column -= 1;
        }
    }
    TextPosition::new(from.line, column)
}

/// End of the word right of a position, symmetric to
/// [`word_start_backwards`].
fn word_end_forwards(document: &TextDocument, from: TextPosition) -> TextPosition {
    let length = document.line_length(from.line);
    let mut column = from.column;

    while column < length && char_at(document, from.line, column).is_some_and(is_word_char) {
        column += 1;
    }
    if column == from.column {
        while column < length
            && char_at(document, from.line, column).is_some_and(|ch| !is_word_char(ch))
        {
            column += 1;
        }
        while column < length && char_at(document, from.line, column).is_some_and(is_word_char) {
            column += 1;
        }
    }
    TextPosition::new(from.line, column)
}

fn char_at(document: &TextDocument, line: usize, column: usize) -> Option<char> {
    document
        .line_text_at(TextPosition::new(line, column))
        .chars()
        .nth(column)
}
