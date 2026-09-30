use std::sync::OnceLock;

use crate::core::sql::dialect::firebird_parser_dialect::FirebirdParserDialect;
use crate::core::sql::dialect::functions::{ANSI_FUNCTIONS, FIREBIRD_FUNCTIONS};
use crate::core::sql::dialect::keywords::{ANSI_KEYWORDS, FIREBIRD_KEYWORDS};
use crate::core::sql::dialect::sql_dialect::SqlDialect;
use crate::core::sql::dialect::sql_dialect_kind::SqlDialectKind;
use crate::core::sql::dialect::word_index::WordIndex;

/// Firebird dialect.
///
/// Lexical rules, keywords and functions are Firebird's. Statement validation
/// uses the standard SQL grammar, because the parsing library has no Firebird
/// grammar; see [`FirebirdParserDialect`] for the exact limits.
#[derive(Debug)]
pub struct FirebirdDialect {
    parser: FirebirdParserDialect,
    keywords: &'static WordIndex,
    functions: &'static WordIndex,
}

impl FirebirdDialect {
    pub fn new() -> Self {
        Self {
            parser: FirebirdParserDialect,
            keywords: keywords(),
            functions: functions(),
        }
    }
}

impl Default for FirebirdDialect {
    fn default() -> Self {
        Self::new()
    }
}

impl SqlDialect for FirebirdDialect {
    fn kind(&self) -> SqlDialectKind {
        SqlDialectKind::Firebird
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
    KEYWORDS.get_or_init(|| WordIndex::from_sets(&[ANSI_KEYWORDS, FIREBIRD_KEYWORDS]))
}

fn functions() -> &'static WordIndex {
    static FUNCTIONS: OnceLock<WordIndex> = OnceLock::new();
    FUNCTIONS.get_or_init(|| WordIndex::from_sets(&[ANSI_FUNCTIONS, FIREBIRD_FUNCTIONS]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reports_its_kind() {
        assert_eq!(FirebirdDialect::new().kind(), SqlDialectKind::Firebird);
    }

    #[test]
    fn test_knows_firebird_only_words() {
        let dialect = FirebirdDialect::new();

        assert!(dialect.keywords().contains("POST_EVENT"));
        assert!(dialect.keywords().contains("AUTONOMOUS"));
        assert!(!dialect.keywords().contains("AUTO_INCREMENT"));
    }

    #[test]
    fn test_knows_firebird_functions() {
        let dialect = FirebirdDialect::new();

        assert!(dialect.functions().contains("IIF"));
        assert!(!dialect.functions().contains("STRFTIME"));
    }

    #[test]
    fn test_uses_ansi_quoting_and_separator() {
        let dialect = FirebirdDialect::new();

        assert_eq!(dialect.quote_style(), '"');
        assert_eq!(dialect.statement_separator(), ";");
    }
}
