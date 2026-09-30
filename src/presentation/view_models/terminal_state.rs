use crate::presentation::view_models::QueryResultsViewModel;
use crate::{TableColumnData, TablePaginationInfo, TableRowData};
use slint::{ModelRc, VecModel};
use std::rc::Rc;

pub struct TerminalState {
    view_model: Option<QueryResultsViewModel>,
    columns_model: Option<Rc<VecModel<TableColumnData>>>,
    rows_model: Option<Rc<VecModel<TableRowData>>>,
}

impl Default for TerminalState {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalState {
    pub fn new() -> Self {
        Self {
            view_model: None,
            columns_model: None,
            rows_model: None,
        }
    }

    pub fn is_loaded(&self) -> bool {
        self.view_model.is_some()
    }

    pub fn columns_model(&self) -> ModelRc<TableColumnData> {
        self.columns_model
            .clone()
            .map_or_else(ModelRc::default, ModelRc::from)
    }

    pub fn rows_model(&self) -> ModelRc<TableRowData> {
        self.rows_model
            .clone()
            .map_or_else(ModelRc::default, ModelRc::from)
    }

    pub fn load(&mut self) {
        if self.is_loaded() {
            return;
        }

        let view_model = QueryResultsViewModel::new();
        self.columns_model = Some(Rc::new(VecModel::from(view_model.to_columns())));
        self.rows_model = Some(Rc::new(VecModel::from(view_model.to_rows())));
        self.view_model = Some(view_model);
    }

    pub fn update_view_model(&mut self, update: impl FnOnce(&mut QueryResultsViewModel)) {
        let Some(view_model) = self.view_model.as_mut() else {
            return;
        };

        update(view_model);
    }

    pub fn push_state_to_models(&self, update_columns: bool) -> Option<TablePaginationInfo> {
        let view_model = self.view_model.as_ref()?;
        let columns_model = self.columns_model.as_ref()?;
        let rows_model = self.rows_model.as_ref()?;

        if update_columns {
            columns_model.set_vec(view_model.to_columns());
        }

        rows_model.set_vec(view_model.to_rows());

        Some(view_model.to_pagination_info())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_state_is_not_loaded_on_creation() {
        let state = TerminalState::new();

        assert!(!state.is_loaded());
        assert!(state.push_state_to_models(true).is_none());
    }

    #[test]
    fn test_terminal_state_load_is_idempotent() {
        let mut state = TerminalState::new();

        state.load();
        let first_load = state.push_state_to_models(true).expect("loaded state");

        state.load();
        let second_load = state.push_state_to_models(true).expect("loaded state");

        assert!(state.is_loaded());
        assert_eq!(first_load.total_count, second_load.total_count);
        assert_eq!(first_load.loaded_count, second_load.loaded_count);
    }

    #[test]
    fn test_terminal_state_keeps_instance_between_loads() {
        let mut state = TerminalState::new();

        state.load();
        state.update_view_model(QueryResultsViewModel::next_page);

        state.load();
        let pagination = state.push_state_to_models(true).expect("loaded state");

        assert_eq!(pagination.current_page, 2);
    }

    #[test]
    fn test_updates_are_ignored_while_not_loaded() {
        let mut state = TerminalState::new();

        state.update_view_model(QueryResultsViewModel::next_page);
        state.update_view_model(QueryResultsViewModel::toggle_infinite_mode);

        assert!(!state.is_loaded());

        state.load();
        let pagination = state.push_state_to_models(true).expect("loaded state");

        assert_eq!(pagination.current_page, 1);
        assert!(!pagination.is_infinite_scroll);
    }
}
