pub mod firebird_dialect;
pub mod firebird_parser_dialect;
pub mod functions;
pub mod keywords;
pub mod mysql_dialect;
pub mod postgres_dialect;
pub mod sql_dialect;
pub mod sql_dialect_kind;
pub mod sqlite_dialect;
pub mod word_index;

pub use firebird_dialect::FirebirdDialect;
pub use mysql_dialect::MySqlDialect;
pub use postgres_dialect::PostgreSqlDialect;
pub use sql_dialect::SqlDialect;
pub use sql_dialect_kind::SqlDialectKind;
pub use sqlite_dialect::SqliteDialect;
pub use word_index::WordIndex;

use crate::core::entities::database_provider::DatabaseProvider;

/// Builds the dialect for a dialect kind.
pub fn dialect_for_kind(kind: SqlDialectKind) -> Box<dyn SqlDialect> {
    match kind {
        SqlDialectKind::PostgreSql => Box::new(PostgreSqlDialect::new()),
        SqlDialectKind::MySql => Box::new(MySqlDialect::new()),
        SqlDialectKind::Sqlite => Box::new(SqliteDialect::new()),
        SqlDialectKind::Firebird => Box::new(FirebirdDialect::new()),
    }
}

/// Builds the dialect matching a database provider.
pub fn dialect_for_provider(provider: DatabaseProvider) -> Box<dyn SqlDialect> {
    dialect_for_kind(SqlDialectKind::from_provider(provider))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_every_kind_builds_its_own_dialect() {
        for kind in SqlDialectKind::ALL {
            let dialect = dialect_for_kind(kind);

            assert_eq!(dialect.kind(), kind);
        }
    }

    #[test]
    fn test_provider_builds_the_matching_dialect() {
        let dialect = dialect_for_provider(DatabaseProvider::Firebird);

        assert_eq!(dialect.kind(), SqlDialectKind::Firebird);
    }

    #[test]
    fn test_keyword_lists_differ_between_dialects() {
        let postgres = dialect_for_kind(SqlDialectKind::PostgreSql);
        let mysql = dialect_for_kind(SqlDialectKind::MySql);

        assert!(postgres.keywords().contains("ILIKE"));
        assert!(!mysql.keywords().contains("ILIKE"));
        assert!(mysql.keywords().contains("AUTO_INCREMENT"));
        assert!(!postgres.keywords().contains("AUTO_INCREMENT"));
    }

    #[test]
    fn test_dialects_can_be_used_as_trait_objects() {
        let dialects: Vec<Box<dyn SqlDialect>> = SqlDialectKind::ALL
            .into_iter()
            .map(dialect_for_kind)
            .collect();

        let names: Vec<&str> = dialects
            .iter()
            .map(|dialect| dialect.kind().display_name())
            .collect();

        assert_eq!(names, ["PostgreSQL", "MySQL", "SQLite", "Firebird"]);
    }
}
