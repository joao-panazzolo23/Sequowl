use crate::core::sql::completion::completion_clause::CompletionClause;

/// Keywords offered per clause.
///
/// These lists are the ANSI core every dialect shares; dialect specific words
/// are handled by the dialect word lists.
const STATEMENT_KEYWORDS: &[&str] = &[
    "ALTER", "CREATE", "DELETE", "DROP", "INSERT", "MERGE", "SELECT", "TRUNCATE", "UPDATE", "WITH",
];

const SELECT_LIST_KEYWORDS: &[&str] = &["ALL", "AS", "CASE", "DISTINCT", "FROM"];
const FROM_KEYWORDS: &[&str] = &[
    "AS", "CROSS", "INNER", "JOIN", "LEFT", "RIGHT", "FULL", "LATERAL", "ON", "USING",
];
const JOIN_KEYWORDS: &[&str] = &["ON", "USING"];
const PREDICATE_KEYWORDS: &[&str] = &[
    "AND", "ANY", "AS", "BETWEEN", "BY", "CASE", "DISTINCT", "ELSE", "END", "EXISTS", "IN", "IS",
    "LIKE", "NOT", "NULL", "OR", "SOME", "THEN", "WHEN", "WITH",
];
const GROUP_BY_KEYWORDS: &[&str] = &["ALL", "ASC", "BY", "DESC", "HAVING", "ORDER"];
const ORDER_BY_KEYWORDS: &[&str] = &["ASC", "DESC", "LIMIT", "NULLS"];
const SET_KEYWORDS: &[&str] = &["WHERE"];
const VALUES_KEYWORDS: &[&str] = &["AS", "DEFAULT", "NULL", "VALUES"];
const UPDATE_KEYWORDS: &[&str] = &["AS", "FROM", "SET", "USING", "WHERE"];
const INTO_KEYWORDS: &[&str] = &["AS", "SELECT", "VALUES"];
const CREATE_KEYWORDS: &[&str] = &[
    "CREATE",
    "INDEX",
    "OR",
    "REPLACE",
    "SCHEMA",
    "SEQUENCE",
    "TABLE",
    "TEMP",
    "TEMPORARY",
    "VIEW",
];

/// Keywords suggested for a clause.
pub fn keywords_for(clause: CompletionClause) -> &'static [&'static str] {
    match clause {
        CompletionClause::StatementStart => STATEMENT_KEYWORDS,
        CompletionClause::SelectList => SELECT_LIST_KEYWORDS,
        CompletionClause::From => FROM_KEYWORDS,
        CompletionClause::Join => JOIN_KEYWORDS,
        CompletionClause::On => PREDICATE_KEYWORDS,
        CompletionClause::Using => PREDICATE_KEYWORDS,
        CompletionClause::Where => PREDICATE_KEYWORDS,
        CompletionClause::GroupBy => GROUP_BY_KEYWORDS,
        CompletionClause::Having => PREDICATE_KEYWORDS,
        CompletionClause::OrderBy => ORDER_BY_KEYWORDS,
        CompletionClause::Set => SET_KEYWORDS,
        CompletionClause::Values => VALUES_KEYWORDS,
        CompletionClause::Into => INTO_KEYWORDS,
        CompletionClause::Update => UPDATE_KEYWORDS,
        CompletionClause::Create => CREATE_KEYWORDS,
        CompletionClause::Returning => PREDICATE_KEYWORDS,
        CompletionClause::Unknown => PREDICATE_KEYWORDS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_every_clause_has_keywords() {
        for clause in [
            CompletionClause::StatementStart,
            CompletionClause::SelectList,
            CompletionClause::From,
            CompletionClause::Join,
            CompletionClause::On,
            CompletionClause::Using,
            CompletionClause::Where,
            CompletionClause::GroupBy,
            CompletionClause::Having,
            CompletionClause::OrderBy,
            CompletionClause::Set,
            CompletionClause::Values,
            CompletionClause::Into,
            CompletionClause::Update,
            CompletionClause::Create,
            CompletionClause::Returning,
            CompletionClause::Unknown,
        ] {
            assert!(
                !keywords_for(clause).is_empty(),
                "{clause:?} has no keywords"
            );
        }
    }

    #[test]
    fn test_statement_start_suggests_statements() {
        assert!(keywords_for(CompletionClause::StatementStart).contains(&"SELECT"));
        assert!(!keywords_for(CompletionClause::StatementStart).contains(&"WHERE"));
    }

    #[test]
    fn test_keywords_are_upper_case() {
        for keyword in keywords_for(CompletionClause::Where) {
            assert_eq!(*keyword, keyword.to_ascii_uppercase());
        }
    }
}
