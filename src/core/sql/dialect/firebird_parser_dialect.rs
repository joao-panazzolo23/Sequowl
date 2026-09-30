use sqlparser::dialect::Dialect;

/// Parser dialect used to validate Firebird statements.
///
/// `sqlparser` does not ship a Firebird dialect, so the lexical rules are
/// implemented here while statement validation falls back to its standard SQL
/// grammar. Firebird-only constructs such as `EXECUTE BLOCK` or
/// `CREATE OR ALTER` are therefore not accepted; this is a documented limit,
/// not a silent approximation.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FirebirdParserDialect;

impl Dialect for FirebirdParserDialect {
    /// Firebird unquoted identifiers may start with a letter, `_` or `$`.
    fn is_identifier_start(&self, ch: char) -> bool {
        ch.is_alphabetic() || ch == '_' || ch == '$'
    }

    fn is_identifier_part(&self, ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_' || ch == '$'
    }

    /// Firebird quotes identifiers with double quotes, never with backticks.
    fn is_delimited_identifier_start(&self, ch: char) -> bool {
        ch == '"'
    }

    fn identifier_quote_style(&self, _identifier: &str) -> Option<char> {
        Some('"')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier_start_follows_firebird_rules() {
        let dialect = FirebirdParserDialect;

        assert!(dialect.is_identifier_start('A'));
        assert!(dialect.is_identifier_start('_'));
        assert!(dialect.is_identifier_start('$'));
        assert!(!dialect.is_identifier_start('1'));
        assert!(!dialect.is_identifier_start('"'));
    }

    #[test]
    fn test_identifier_part_accepts_digits_after_the_first_character() {
        let dialect = FirebirdParserDialect;

        assert!(dialect.is_identifier_part('7'));
        assert!(dialect.is_identifier_part('z'));
        assert!(!dialect.is_identifier_part(' '));
    }

    #[test]
    fn test_only_double_quotes_delimit_identifiers() {
        let dialect = FirebirdParserDialect;

        assert!(dialect.is_delimited_identifier_start('"'));
        assert!(!dialect.is_delimited_identifier_start('`'));
        assert_eq!(dialect.identifier_quote_style("name"), Some('"'));
    }

    #[test]
    fn test_firebird_does_not_use_backslash_escapes() {
        assert!(!FirebirdParserDialect.supports_string_literal_backslash_escape());
        assert!(!FirebirdParserDialect.supports_quote_delimited_string());
        assert!(!FirebirdParserDialect.supports_dollar_placeholder());
    }
}
