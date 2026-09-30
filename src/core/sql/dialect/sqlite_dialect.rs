use std::sync::OnceLock;

use sqlparser::dialect::SQLiteDialect as SqliteParserDialect;

use crate::core::sql::dialect::functions::{ANSI_FUNCTIONS, SQLITE_FUNCTIONS};
use crate::core::sql::dialect::keywords::{ANSI_KEYWORDS, SQLITE_KEYWORDS};
use crate::core::sql::dialect::sql_dialect::SqlDialect;
use crate::core::sql::dialect::sql_dialect_kind::SqlDialectKind;
use crate::core::sql::dialect::word_index::WordIndex;

/// SQLite dialect.
#[derive(Debug)]
pub struct SqliteDialect {
    parser: SqliteParserDialect,
    keywords: &'static WordIndex,
    functions: &'static WordIndex,
}

impl SqliteDialect {
    pub fn new() -> Self {
        Self {
            parser: SqliteParserDialect {},
            keywords: keywords(),
            functions: functions(),
        }
    }
}

impl Default for SqliteDialect {
    fn default() -> Self {
        Self::new()
    }
}

impl SqlDialect for SqliteDialect {
    fn kind(&self) -> SqlDialectKind {
        SqlDialectKind::Sqlite
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
    KEYWORDS.get_or_init(|| WordIndex::from_sets(&[ANSI_KEYWORDS, SQLITE_KEYWORDS]))
}

fn functions() -> &'static WordIndex {
    static FUNCTIONS: OnceLock<WordIndex> = OnceLock::new();
    FUNCTIONS.get_or_init(|| WordIndex::from_sets(&[ANSI_FUNCTIONS, SQLITE_FUNCTIONS]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reports_its_kind() {
        assert_eq!(SqliteDialect::new().kind(), SqlDialectKind::Sqlite);
    }

    #[test]
    fn test_knows_sqlite_only_words() {
        let dialect = SqliteDialect::new();

        assert!(dialect.keywords().contains("PRAGMA"));
        assert!(!dialect.keywords().contains("AUTO_INCREMENT"));
    }

    #[test]
    fn test_knows_sqlite_functions() {
        let dialect = SqliteDialect::new();

        assert!(dialect.functions().contains("STRFTIME"));
        assert!(!dialect.functions().contains("GROUP_CONCAT"));
    }
}
