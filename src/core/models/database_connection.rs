use crate::core::models::query_result::QueryResult;

pub trait DatabaseConnection {
    fn query(&mut self, sql: &str) -> anyhow::Result<QueryResult>;

    fn execute(&mut self, sql: &str) -> anyhow::Result<u64>;

    fn ping(&mut self) -> anyhow::Result<()>;
}
