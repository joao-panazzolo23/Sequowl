use sequowl::App;
use sequowl::core::sql::catalog::schema_catalog::SchemaCatalog;
use sequowl::presentation::view_models::{
    DatabasesViewModel, EditorBridge, QueryResultsViewModel, TerminalState,
};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::cell::RefCell;
use std::rc::Rc;

/// Buffer the editor starts with.
const INITIAL_QUERY: &str = "SELECT u.\nFROM public.users AS u\nORDER BY id;";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new()?;

    // 1. Databases Tree ViewModel
    let db_view_model = Rc::new(RefCell::new(DatabasesViewModel::new()));
    let initial_nodes = db_view_model.borrow().to_tree_nodes();
    let db_model = Rc::new(VecModel::from(initial_nodes));
    app.set_database_tree_nodes(ModelRc::from(db_model.clone()));

    app.on_toggle_database_node({
        let db_view_model = db_view_model.clone();
        let db_model = db_model.clone();
        move |id| {
            let mut vm = db_view_model.borrow_mut();
            vm.toggle_expand(id.as_str());
            let updated = vm.to_tree_nodes();
            db_model.set_vec(updated);
        }
    });

    // 2. SQL editor, driven by the core controller
    let bridge = Rc::new(RefCell::new(EditorBridge::new(
        INITIAL_QUERY,
        db_view_model
            .borrow()
            .active_catalog()
            .unwrap_or_else(SchemaCatalog::empty),
    )));

    let push_editor_state = {
        let app_weak = app.as_weak();
        let bridge = bridge.clone();
        move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            app.set_editor_state(bridge.borrow().state());
        }
    };
    push_editor_state();

    app.on_editor_action({
        let bridge = bridge.clone();
        let push_editor_state = push_editor_state.clone();
        move |intent| {
            if bridge.borrow_mut().dispatch(intent.as_str()) {
                push_editor_state();
            }
        }
    });

    // Selecting a connection re-points the editor at its dialect and metadata.
    app.on_select_database_node({
        let db_view_model = db_view_model.clone();
        let db_model = db_model.clone();
        let bridge = bridge.clone();
        let push_editor_state = push_editor_state.clone();
        move |id| {
            {
                let mut vm = db_view_model.borrow_mut();
                vm.select_node(id.as_str());
                db_model.set_vec(vm.to_tree_nodes());
            }
            if let Some(catalog) = db_view_model.borrow().active_catalog() {
                bridge.borrow_mut().set_catalog(catalog);
            }
            push_editor_state();
        }
    });

    let terminal_state = Rc::new(RefCell::new(TerminalState::new()));

    let sync_table_state = {
        let app_weak = app.as_weak();
        let terminal_state = terminal_state.clone();
        move |update_columns: bool| {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let state = terminal_state.borrow();
            if let Some(pagination) = state.push_state_to_models(update_columns) {
                app.set_table_pagination(pagination);
            }
        }
    };

    app.on_load_terminal({
        let app_weak = app.as_weak();
        let terminal_state = terminal_state.clone();
        let sync = sync_table_state.clone();
        move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };

            terminal_state.borrow_mut().load();
            app.set_table_columns(terminal_state.borrow().columns_model());
            app.set_table_rows(terminal_state.borrow().rows_model());
            sync(true);
        }
    });

    // The editor decides what a run means: the selection, the statement under
    // the caret, or the whole buffer.
    app.on_execute_query({
        let bridge = bridge.clone();
        let terminal_state = terminal_state.clone();
        let sync = sync_table_state.clone();
        move || {
            let query = bridge.borrow().execution_target();
            terminal_state
                .borrow_mut()
                .update_view_model(|view_model| view_model.execute_query(&query));
            sync(true);
        }
    });

    app.on_table_next_page({
        let terminal_state = terminal_state.clone();
        let sync = sync_table_state.clone();
        move || {
            terminal_state
                .borrow_mut()
                .update_view_model(QueryResultsViewModel::next_page);
            sync(false);
        }
    });

    app.on_table_prev_page({
        let terminal_state = terminal_state.clone();
        let sync = sync_table_state.clone();
        move || {
            terminal_state
                .borrow_mut()
                .update_view_model(QueryResultsViewModel::prev_page);
            sync(false);
        }
    });

    app.on_table_load_more({
        let terminal_state = terminal_state.clone();
        let sync = sync_table_state.clone();
        move || {
            terminal_state
                .borrow_mut()
                .update_view_model(QueryResultsViewModel::load_more_infinite);
            sync(false);
        }
    });

    app.on_table_set_page_size({
        let terminal_state = terminal_state.clone();
        let sync = sync_table_state.clone();
        move |size| {
            terminal_state.borrow_mut().update_view_model(|view_model| {
                view_model.set_page_size(size as usize);
            });
            sync(false);
        }
    });

    app.on_table_toggle_infinite_mode({
        let terminal_state = terminal_state.clone();
        let sync = sync_table_state.clone();
        move || {
            terminal_state
                .borrow_mut()
                .update_view_model(QueryResultsViewModel::toggle_infinite_mode);
            sync(false);
        }
    });

    app.on_table_sort_column({
        let terminal_state = terminal_state.clone();
        let sync = sync_table_state.clone();
        move |col_idx| {
            terminal_state.borrow_mut().update_view_model(|view_model| {
                view_model.sort_by_column(col_idx as usize);
            });
            sync(true);
        }
    });

    app.on_table_filter_changed({
        let terminal_state = terminal_state.clone();
        let sync = sync_table_state.clone();
        move |query| {
            terminal_state.borrow_mut().update_view_model(|view_model| {
                view_model.set_filter(query.as_str());
            });
            sync(false);
        }
    });

    app.on_table_select_row({
        let terminal_state = terminal_state.clone();
        let sync = sync_table_state.clone();
        move |row_num| {
            terminal_state
                .borrow_mut()
                .update_view_model(|view_model| view_model.select_row(row_num));
            sync(false);
        }
    });

    app.on_table_refresh({
        let terminal_state = terminal_state.clone();
        let sync = sync_table_state.clone();
        move || {
            terminal_state
                .borrow_mut()
                .update_view_model(QueryResultsViewModel::refresh);
            sync(true);
        }
    });

    app.run()?;
    Ok(())
}
