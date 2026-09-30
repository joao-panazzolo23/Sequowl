use std::sync::OnceLock;

use sqlparser::dialect::MySqlDialect as MySqlParserDialect;

use crate::core::sql::dialect::functions::{ANSI_FUNCTIONS, MYSQL_FUNCTIONS};
use crate::core::sql::dialect::keywords::{ANSI_KEYWORDS, MYSQL_KEYWORDS};
use crate::core::sql::dialect::sql_dialect::SqlDialect;
use crate::core::sql::dialect::sql_dialect_kind::SqlDialectKind;
use crate::core::sql::dialect::word_index::WordIndex;

/// MySQL dialect, including MySQL only quoting and word lists.
#[derive(Debug)]
pub struct MySqlDialect {
    parser: MySqlParserDialect,
    keywords: &'static WordIndex,
    functions: &'static WordIndex,
}

impl MySqlDialect {
    pub fn new() -> Self {
        Self {
            parser: MySqlParserDialect {},
            keywords: keywords(),
            functions: functions(),
        }
    }
}

impl Default for MySqlDialect {
    fn default() -> Self {
        Self::new()
    }
}

impl SqlDialect for MySqlDialect {
    fn kind(&self) -> SqlDialectKind {
        SqlDialectKind::MySql
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

    /// MySQL quotes identifiers with backticks, double quotes are strings.
    fn quote_style(&self) -> char {
        '`'
    }
}

fn keywords() -> &'static WordIndex {
    static KEYWORDS: OnceLock<WordIndex> = OnceLock::new();
    KEYWORDS.get_or_init(|| WordIndex::from_sets(&[ANSI_KEYWORDS, MYSQL_KEYWORDS]))
}

fn functions() -> &'static WordIndex {
    static FUNCTIONS: OnceLock<WordIndex> = OnceLock::new();
    FUNCTIONS.get_or_init(|| WordIndex::from_sets(&[ANSI_FUNCTIONS, MYSQL_FUNCTIONS]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reports_its_kind() {
        assert_eq!(MySqlDialect::new().kind(), SqlDialectKind::MySql);
    }

    #[test]
    fn test_uses_backtick_quoting() {
        assert_eq!(MySqlDialect::new().quote_style(), '`');
    }

    #[test]
    fn test_knows_mysql_only_words() {
        let dialect = MySqlDialect::new();

        assert!(dialect.keywords().contains("AUTO_INCREMENT"));
        assert!(!dialect.keywords().contains("ILIKE"));
    }

    #[test]
    fn test_knows_mysql_functions() {
        let dialect = MySqlDialect::new();

        assert!(dialect.functions().contains("GROUP_CONCAT"));
        assert!(!dialect.functions().contains("DATE_TRUNC"));
    }
}
