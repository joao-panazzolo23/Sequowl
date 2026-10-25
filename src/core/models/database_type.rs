#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DatabaseType {
    PostgreSql,
    MySql,
    SqlServer,
}
