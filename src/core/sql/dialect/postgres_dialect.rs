use std::sync::OnceLock;

use sqlparser::dialect::PostgreSqlDialect as PostgresParserDialect;

use crate::core::sql::dialect::functions::{ANSI_FUNCTIONS, POSTGRES_FUNCTIONS};
use crate::core::sql::dialect::keywords::{ANSI_KEYWORDS, POSTGRES_KEYWORDS};
use crate::core::sql::dialect::sql_dialect::SqlDialect;
use crate::core::sql::dialect::sql_dialect_kind::SqlDialectKind;
use crate::core::sql::dialect::word_index::WordIndex;

/// PostgreSQL dialect: ANSI words plus the PostgreSQL specific ones.
#[derive(Debug)]
pub struct PostgreSqlDialect {
    parser: PostgresParserDialect,
    keywords: &'static WordIndex,
    functions: &'static WordIndex,
}

impl PostgreSqlDialect {
    pub fn new() -> Self {
        Self {
            parser: PostgresParserDialect {},
            keywords: keywords(),
            functions: functions(),
        }
    }
}

impl Default for PostgreSqlDialect {
    fn default() -> Self {
        Self::new()
    }
}

impl SqlDialect for PostgreSqlDialect {
    fn kind(&self) -> SqlDialectKind {
        SqlDialectKind::PostgreSql
    }

    fn parser_dialect(&self) -> &dyn sqlparser::dialect::Dialect {
        &self.parser
    }

    fn keywords(&self) -> &WordIndex {
        self.keywords
    }

    fn functions(&self) -> &WordIndex {
        self.functions
    }
}

fn keywords() -> &'static WordIndex {
    static KEYWORDS: OnceLock<WordIndex> = OnceLock::new();
    KEYWORDS.get_or_init(|| WordIndex::from_sets(&[ANSI_KEYWORDS, POSTGRES_KEYWORDS]))
}

fn functions() -> &'static WordIndex {
    static FUNCTIONS: OnceLock<WordIndex> = OnceLock::new();
    FUNCTIONS.get_or_init(|| WordIndex::from_sets(&[ANSI_FUNCTIONS, POSTGRES_FUNCTIONS]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reports_its_kind_and_name() {
        let dialect = PostgreSqlDialect::new();

        assert_eq!(dialect.kind(), SqlDialectKind::PostgreSql);
        assert_eq!(dialect.kind().display_name(), "PostgreSQL");
    }

    #[test]
    fn test_knows_shared_and_postgres_only_words() {
        let dialect = PostgreSqlDialect::new();

        assert!(dialect.keywords().contains("SELECT"));
        assert!(dialect.keywords().contains("ILIKE"));
        assert!(!dialect.keywords().contains("TELEPORT"));
    }

    #[test]
    fn test_knows_postgres_functions() {
        let dialect = PostgreSqlDialect::new();

        assert!(dialect.functions().contains("count"));
        assert!(dialect.functions().contains("DATE_TRUNC"));
        assert!(!dialect.functions().contains("RDB$GET_CONTEXT"));
    }

    #[test]
    fn test_word_indexes_are_shared_between_instances() {
        let first = PostgreSqlDialect::new();
        let second = PostgreSqlDialect::new();

        assert!(std::ptr::eq(first.keywords(), second.keywords()));
    }
}
