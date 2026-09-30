use std::fmt::Debug;

use crate::core::sql::dialect::sql_dialect_kind::SqlDialectKind;
use crate::core::sql::dialect::word_index::WordIndex;

/// Everything the SQL engine needs to know about one SQL flavour.
///
/// Implementations own the parser dialect used for validation and the word
/// lists used for highlighting and completion. The editor only depends on this
/// trait, never on a concrete dialect.
pub trait SqlDialect: Debug {
    fn kind(&self) -> SqlDialectKind;

    /// Parser used to validate statements.
    ///
    /// This is the single unavoidable dependency on the SQL parsing library;
    /// it never leaks into the rest of the engine.
    fn parser_dialect(&self) -> &dyn sqlparser::dialect::Dialect;

    /// Reserved words of this dialect, used for syntax highlighting.
    fn keywords(&self) -> &WordIndex;

    /// Function names of this dialect, used for highlighting and completion.
    fn functions(&self) -> &WordIndex;

    /// Character used to quote identifiers.
    fn quote_style(&self) -> char {
        '"'
    }

    /// Token that separates one statement from the next.
    fn statement_separator(&self) -> &'static str {
        ";"
    }

    /// Comment prefix, used by completion when filtering keywords.
    fn line_comment_prefix(&self) -> &'static str {
        "--"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::sql::dialect::postgres_dialect::PostgreSqlDialect;

    #[test]
    fn test_defaults_match_the_shared_sql_conventions() {
        let dialect = PostgreSqlDialect::new();

        assert_eq!(dialect.quote_style(), '"');
        assert_eq!(dialect.statement_separator(), ";");
        assert_eq!(dialect.line_comment_prefix(), "--");
    }
}
