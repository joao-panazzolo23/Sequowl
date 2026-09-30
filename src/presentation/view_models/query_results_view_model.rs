use crate::{TableCellData, TableColumnData, TablePaginationInfo, TableRowData};
use slint::{ModelRc, SharedString, VecModel};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct MockCell {
    pub text: String,
    pub is_null: bool,
    pub data_type: String,
}

#[derive(Debug, Clone)]
pub struct MockRow {
    pub id: i32,
    pub cells: Vec<MockCell>,
}

#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: String,
    pub width: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    None,
    Ascending,
    Descending,
}

fn to_slint_sort_direction(sort_direction: SortDirection) -> crate::SortDirection {
    match sort_direction {
        SortDirection::Ascending => crate::SortDirection::Ascending,
        SortDirection::Descending => crate::SortDirection::Descending,
        SortDirection::None => crate::SortDirection::None,
    }
}

pub struct QueryResultsViewModel {
    columns: Vec<ColumnDef>,
    rows: Vec<MockRow>,
    filtered_indices: Vec<usize>,
    current_page: usize,
    page_size: usize,
    is_infinite_scroll: bool,
    infinite_loaded_count: usize,
    sort_column_index: Option<usize>,
    sort_direction: SortDirection,
    filter_query: String,
    selected_row_id: Option<i32>,
    execution_time_ms: i32,
    query_name: String,
}

impl Default for QueryResultsViewModel {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryResultsViewModel {
    pub fn new() -> Self {
        let columns = vec![
            ColumnDef {
                name: "id".to_string(),
                data_type: "uuid".to_string(),
                width: 220.0,
            },
            ColumnDef {
                name: "first_name".to_string(),
                data_type: "varchar".to_string(),
                width: 130.0,
            },
            ColumnDef {
                name: "last_name".to_string(),
                data_type: "varchar".to_string(),
                width: 130.0,
            },
            ColumnDef {
                name: "email".to_string(),
                data_type: "varchar".to_string(),
                width: 210.0,
            },
            ColumnDef {
                name: "role".to_string(),
                data_type: "varchar".to_string(),
                width: 110.0,
            },
            ColumnDef {
                name: "is_active".to_string(),
                data_type: "bool".to_string(),
                width: 90.0,
            },
            ColumnDef {
                name: "balance".to_string(),
                data_type: "numeric".to_string(),
                width: 120.0,
            },
            ColumnDef {
                name: "created_at".to_string(),
                data_type: "timestamp".to_string(),
                width: 180.0,
            },
            ColumnDef {
                name: "notes".to_string(),
                data_type: "varchar".to_string(),
                width: 160.0,
            },
        ];

        let rows = Self::generate_mock_rows(250);
        let row_count = rows.len();
        let default_page_size = 50;

        let mut vm = Self {
            columns,
            rows,
            filtered_indices: (0..row_count).collect(),
            current_page: 1,
            page_size: default_page_size,
            is_infinite_scroll: false,
            infinite_loaded_count: default_page_size.min(row_count),
            sort_column_index: None,
            sort_direction: SortDirection::None,
            filter_query: String::new(),
            selected_row_id: None,
            execution_time_ms: 18,
            query_name: "SELECT * FROM public.users ORDER BY id;".to_string(),
        };

        vm.apply_filter_and_sort();
        vm
    }

    fn generate_mock_rows(count: usize) -> Vec<MockRow> {
        let first_names = [
            "Emma",
            "Liam",
            "Olivia",
            "Noah",
            "Ava",
            "Oliver",
            "Sophia",
            "Elijah",
            "Isabella",
            "James",
            "Mia",
            "William",
            "Lucas",
            "Charlotte",
            "Amelia",
            "Benjamin",
            "Harper",
            "Henry",
            "Evelyn",
            "Alexander",
        ];
        let last_names = [
            "Smith",
            "Johnson",
            "Williams",
            "Brown",
            "Jones",
            "Garcia",
            "Miller",
            "Davis",
            "Rodriguez",
            "Martinez",
            "Hernandez",
            "Lopez",
            "Gonzalez",
            "Wilson",
            "Anderson",
            "Thomas",
            "Taylor",
            "Moore",
            "Jackson",
            "Martin",
        ];
        let roles = [
            "admin",
            "developer",
            "customer",
            "support",
            "analyst",
            "viewer",
        ];
        let notes_pool = [
            "Key account owner",
            "Verified via OAuth",
            "Pending email verification",
            "Enterprise tier license",
            "High priority client",
            "Beta feature participant",
        ];

        let mut rows = Vec::with_capacity(count);

        for i in 1..=count {
            let fn_idx = (i * 7 + 3) % first_names.len();
            let ln_idx = (i * 11 + 5) % last_names.len();
            let role_idx = (i * 3) % roles.len();
            let first = first_names[fn_idx];
            let last = last_names[ln_idx];
            let email = format!(
                "{}.{}@sequowl.io",
                first.to_lowercase(),
                last.to_lowercase()
            );
            let is_active = (i % 5) != 0;
            let balance = format!(
                "${:.2}",
                ((i * 137) % 15000) as f64 + ((i % 100) as f64 / 100.0)
            );
            let day = (i % 28) + 1;
            let month = (i % 12) + 1;
            let hour = (i * 3) % 24;
            let min = (i * 7) % 60;
            let created_at = format!("2024-{:02}-{:02} {:02}:{:02}:00", month, day, hour, min);

            let is_null_note = (i % 4) == 0;
            let note = if is_null_note {
                String::new()
            } else {
                notes_pool[i % notes_pool.len()].to_string()
            };

            let uuid = format!(
                "a0ee{:04}-{:04}-4ef8-bb6d-{:012x}",
                i,
                (i * 31) % 10000,
                (i as u64) * 0x1abcdefu64
            );

            let cells = vec![
                MockCell {
                    text: uuid,
                    is_null: false,
                    data_type: "uuid".to_string(),
                },
                MockCell {
                    text: first.to_string(),
                    is_null: false,
                    data_type: "varchar".to_string(),
                },
                MockCell {
                    text: last.to_string(),
                    is_null: false,
                    data_type: "varchar".to_string(),
                },
                MockCell {
                    text: email,
                    is_null: false,
                    data_type: "varchar".to_string(),
                },
                MockCell {
                    text: roles[role_idx].to_string(),
                    is_null: false,
                    data_type: "varchar".to_string(),
                },
                MockCell {
                    text: if is_active {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    },
                    is_null: false,
                    data_type: "bool".to_string(),
                },
                MockCell {
                    text: balance,
                    is_null: false,
                    data_type: "numeric".to_string(),
                },
                MockCell {
                    text: created_at,
                    is_null: false,
                    data_type: "timestamp".to_string(),
                },
                MockCell {
                    text: note,
                    is_null: is_null_note,
                    data_type: "varchar".to_string(),
                },
            ];

            rows.push(MockRow {
                id: i as i32,
                cells,
            });
        }

        rows
    }

    pub fn total_filtered_count(&self) -> usize {
        self.filtered_indices.len()
    }

    pub fn total_pages(&self) -> usize {
        let count = self.total_filtered_count();
        if count == 0 {
            return 1;
        }
        count.div_ceil(self.page_size)
    }

    pub fn execute_query(&mut self, sql: &str) {
        let query = sql.trim();

        self.query_name = query.to_string();
        self.filter_query = String::new();
        self.current_page = 1;
        self.sort_column_index = None;
        self.sort_direction = SortDirection::None;
        self.selected_row_id = None;
        self.execution_time_ms = 12 + (query.len() % 25) as i32;

        if query.is_empty() {
            self.filtered_indices.clear();
            self.infinite_loaded_count = 0;
            return;
        }

        self.infinite_loaded_count = self.page_size.min(self.rows.len());
        self.apply_filter_and_sort();
    }

    pub fn next_page(&mut self) {
        if self.current_page >= self.total_pages() {
            return;
        }
        self.current_page += 1;
    }

    pub fn prev_page(&mut self) {
        if self.current_page <= 1 {
            return;
        }
        self.current_page -= 1;
    }

    pub fn load_more_infinite(&mut self) {
        let total = self.total_filtered_count();
        if self.infinite_loaded_count >= total {
            return;
        }
        self.infinite_loaded_count = (self.infinite_loaded_count + self.page_size).min(total);
    }

    pub fn set_page_size(&mut self, size: usize) {
        if size == 0 || size == self.page_size {
            return;
        }
        self.page_size = size;
        self.current_page = 1;
        self.infinite_loaded_count = size.min(self.total_filtered_count());
    }

    pub fn toggle_infinite_mode(&mut self) {
        self.is_infinite_scroll = !self.is_infinite_scroll;
        if self.is_infinite_scroll {
            self.infinite_loaded_count =
                (self.current_page * self.page_size).min(self.total_filtered_count());
            if self.infinite_loaded_count == 0 {
                self.infinite_loaded_count = self.page_size.min(self.total_filtered_count());
            }
            return;
        }
        // Switched to page mode
        self.current_page = 1;
    }

    pub fn set_filter(&mut self, query: &str) {
        self.filter_query = query.trim().to_lowercase();
        self.current_page = 1;
        self.apply_filter_and_sort();
        self.infinite_loaded_count = self.page_size.min(self.total_filtered_count());
    }

    pub fn sort_by_column(&mut self, column_index: usize) {
        if column_index >= self.columns.len() {
            return;
        }

        if self.sort_column_index == Some(column_index) {
            self.sort_direction = match self.sort_direction {
                SortDirection::None => SortDirection::Ascending,
                SortDirection::Ascending => SortDirection::Descending,
                SortDirection::Descending => SortDirection::None,
            };
        } else {
            self.sort_column_index = Some(column_index);
            self.sort_direction = SortDirection::Ascending;
        }

        self.current_page = 1;
        self.apply_filter_and_sort();
    }

    pub fn select_row(&mut self, row_num: i32) {
        if self.selected_row_id == Some(row_num) {
            self.selected_row_id = None;
            return;
        }
        self.selected_row_id = Some(row_num);
    }

    pub fn refresh(&mut self) {
        self.execution_time_ms = 12 + ((self.execution_time_ms * 7) % 25);
        self.current_page = 1;
        self.apply_filter_and_sort();
        self.infinite_loaded_count = self.page_size.min(self.total_filtered_count());
    }

    fn apply_filter_and_sort(&mut self) {
        let is_filter_empty = self.filter_query.is_empty();

        let mut indices: Vec<usize> = self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, row)| {
                if is_filter_empty {
                    return true;
                }
                row.cells.iter().any(|cell| {
                    if cell.is_null {
                        return false;
                    }
                    cell.text.to_lowercase().contains(&self.filter_query)
                })
            })
            .map(|(idx, _)| idx)
            .collect();

        if let (Some(col_idx), SortDirection::Ascending | SortDirection::Descending) =
            (self.sort_column_index, self.sort_direction)
        {
            indices.sort_by(|&a, &b| {
                let cell_a = &self.rows[a].cells[col_idx];
                let cell_b = &self.rows[b].cells[col_idx];

                let cmp = match (cell_a.is_null, cell_b.is_null) {
                    (true, true) => std::cmp::Ordering::Equal,
                    (true, false) => std::cmp::Ordering::Greater,
                    (false, true) => std::cmp::Ordering::Less,
                    (false, false) => cell_a.text.cmp(&cell_b.text),
                };

                if self.sort_direction == SortDirection::Descending {
                    cmp.reverse()
                } else {
                    cmp
                }
            });
        }

        self.filtered_indices = indices;
    }

    pub fn to_columns(&self) -> Vec<TableColumnData> {
        self.columns
            .iter()
            .enumerate()
            .map(|(idx, col)| {
                let sort_direction = if self.sort_column_index == Some(idx) {
                    to_slint_sort_direction(self.sort_direction)
                } else {
                    crate::SortDirection::None
                };

                TableColumnData {
                    name: SharedString::from(&col.name),
                    data_type: SharedString::from(&col.data_type),
                    width: col.width,
                    sort_direction,
                }
            })
            .collect()
    }

    pub fn to_rows(&self) -> Vec<TableRowData> {
        let total = self.total_filtered_count();
        if total == 0 {
            return Vec::new();
        }

        let (start, end) = if self.is_infinite_scroll {
            let loaded = self.infinite_loaded_count.min(total);
            (0, loaded)
        } else {
            let start_idx = (self.current_page - 1) * self.page_size;
            let end_idx = (start_idx + self.page_size).min(total);
            (start_idx, end_idx)
        };

        if start >= end {
            return Vec::new();
        }

        self.filtered_indices[start..end]
            .iter()
            .enumerate()
            .map(|(display_idx, &row_idx)| {
                let row = &self.rows[row_idx];
                let row_num = (start + display_idx + 1) as i32;
                let is_selected = self.selected_row_id == Some(row.id);

                let cells: Vec<TableCellData> = row
                    .cells
                    .iter()
                    .map(|cell| TableCellData {
                        text: SharedString::from(&cell.text),
                        is_null: cell.is_null,
                        data_type: SharedString::from(&cell.data_type),
                    })
                    .collect();

                TableRowData {
                    row_num,
                    cells: ModelRc::from(Rc::new(VecModel::from(cells))),
                    is_selected,
                }
            })
            .collect()
    }

    pub fn to_pagination_info(&self) -> TablePaginationInfo {
        let total = self.total_filtered_count();
        let loaded = if self.is_infinite_scroll {
            self.infinite_loaded_count.min(total)
        } else {
            let start = (self.current_page - 1) * self.page_size;
            let end = (start + self.page_size).min(total);
            end.saturating_sub(start)
        };

        let status_message = format!("Showing {} of {} rows", loaded, total);

        TablePaginationInfo {
            current_page: self.current_page as i32,
            total_pages: self.total_pages() as i32,
            page_size: self.page_size as i32,
            loaded_count: loaded as i32,
            total_count: total as i32,
            execution_time_ms: self.execution_time_ms,
            query_name: SharedString::from(&self.query_name),
            is_infinite_scroll: self.is_infinite_scroll,
            status_message: SharedString::from(status_message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state_pagination() {
        // Arrange
        let vm = QueryResultsViewModel::new();

        // Act
        let info = vm.to_pagination_info();
        let rows = vm.to_rows();
        let cols = vm.to_columns();

        // Assert
        assert_eq!(info.current_page, 1);
        assert_eq!(info.total_pages, 5);
        assert_eq!(info.page_size, 50);
        assert_eq!(info.total_count, 250);
        assert_eq!(info.loaded_count, 50);
        assert_eq!(rows.len(), 50);
        assert_eq!(cols.len(), 9);
        assert_eq!(rows[0].row_num, 1);
    }

    #[test]
    fn test_next_and_prev_page() {
        // Arrange
        let mut vm = QueryResultsViewModel::new();

        // Act & Assert: moving to next page
        vm.next_page();
        let info = vm.to_pagination_info();
        assert_eq!(info.current_page, 2);
        let rows = vm.to_rows();
        assert_eq!(rows[0].row_num, 51);

        // Act & Assert: moving back to previous page
        vm.prev_page();
        assert_eq!(vm.to_pagination_info().current_page, 1);

        // Act & Assert: previous page clamped at 1
        vm.prev_page();
        assert_eq!(vm.to_pagination_info().current_page, 1);
    }

    #[test]
    fn test_set_page_size() {
        // Arrange
        let mut vm = QueryResultsViewModel::new();

        // Act: change page size to 25
        vm.set_page_size(25);

        // Assert
        let info = vm.to_pagination_info();
        assert_eq!(info.page_size, 25);
        assert_eq!(info.total_pages, 10);
        assert_eq!(info.loaded_count, 25);
        assert_eq!(vm.to_rows().len(), 25);
    }

    #[test]
    fn test_infinite_scroll_mode() {
        // Arrange
        let mut vm = QueryResultsViewModel::new();

        // Act: toggle infinite scroll on
        vm.toggle_infinite_mode();
        let info_init = vm.to_pagination_info();

        // Assert
        assert!(info_init.is_infinite_scroll);
        assert_eq!(info_init.loaded_count, 50);
        assert_eq!(vm.to_rows().len(), 50);

        // Act: load next chunk in infinite scroll
        vm.load_more_infinite();
        let info_more = vm.to_pagination_info();

        // Assert: appends next batch without losing previous
        assert_eq!(info_more.loaded_count, 100);
        let rows = vm.to_rows();
        assert_eq!(rows.len(), 100);
        assert_eq!(rows[0].row_num, 1);
        assert_eq!(rows[99].row_num, 100);
    }

    #[test]
    fn test_sort_by_column() {
        // Arrange
        let mut vm = QueryResultsViewModel::new();

        // Act: sort by column 1 (first_name) ascending
        vm.sort_by_column(1);
        let cols = vm.to_columns();
        assert_eq!(cols[1].sort_direction, crate::SortDirection::Ascending);
        assert_eq!(cols[2].sort_direction, crate::SortDirection::None);

        // Act: sort descending
        vm.sort_by_column(1);
        let cols_desc = vm.to_columns();
        assert_eq!(
            cols_desc[1].sort_direction,
            crate::SortDirection::Descending
        );

        // Act: reset sort to none
        vm.sort_by_column(1);
        let cols_none = vm.to_columns();
        assert_eq!(cols_none[1].sort_direction, crate::SortDirection::None);
    }

    #[test]
    fn test_execute_query_stores_the_query_and_returns_rows() {
        // Arrange
        let mut vm = QueryResultsViewModel::new();

        // Act
        vm.execute_query("  SELECT id, email FROM public.users WHERE is_active = true;  ");

        // Assert
        let info = vm.to_pagination_info();
        assert_eq!(
            info.query_name,
            "SELECT id, email FROM public.users WHERE is_active = true;"
        );
        assert_eq!(info.total_count, 250);
        assert_eq!(info.loaded_count, 50);
        assert_eq!(vm.to_rows().len(), 50);
    }

    #[test]
    fn test_execute_query_resets_paging_and_sorting() {
        // Arrange
        let mut vm = QueryResultsViewModel::new();
        vm.sort_by_column(1);
        vm.next_page();
        vm.set_filter("admin");

        // Act
        vm.execute_query("SELECT * FROM public.orders;");

        // Assert: a new execution starts from a clean result set
        let info = vm.to_pagination_info();
        assert_eq!(info.current_page, 1);
        assert_eq!(info.total_count, 250);
        assert_eq!(vm.to_rows()[0].row_num, 1);
        assert_eq!(
            vm.to_columns()[1].sort_direction,
            crate::SortDirection::None
        );
    }

    #[test]
    fn test_execute_query_without_a_query_returns_no_rows() {
        // Arrange
        let mut vm = QueryResultsViewModel::new();

        // Act
        vm.execute_query("   ");

        // Assert
        let info = vm.to_pagination_info();
        assert!(info.query_name.is_empty());
        assert_eq!(info.total_count, 0);
        assert_eq!(info.loaded_count, 0);
        assert!(vm.to_rows().is_empty());
    }

    #[test]
    fn test_filtering() {
        // Arrange
        let mut vm = QueryResultsViewModel::new();

        // Act: filter for "admin"
        vm.set_filter("admin");
        let info = vm.to_pagination_info();

        // Assert
        assert!(info.total_count < 250);
        assert!(info.total_count > 0);

        // Act: clear filter
        vm.set_filter("");
        assert_eq!(vm.to_pagination_info().total_count, 250);
    }

    #[test]
    fn test_select_row() {
        // Arrange
        let mut vm = QueryResultsViewModel::new();

        // Act: select row 1
        vm.select_row(1);
        let rows = vm.to_rows();
        assert!(rows[0].is_selected);

        // Act: select again toggles it off
        vm.select_row(1);
        let rows_unselected = vm.to_rows();
        assert!(!rows_unselected[0].is_selected);
    }
}
