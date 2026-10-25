use crate::core::models::{column::Column, sql_value::SqlValue};

pub struct QueryResult {
    pub columns: Vec<Column>,
    pub rows: Vec<Vec<SqlValue>>,
    pub rows_affected: Option<u64>,
}
