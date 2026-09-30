use crate::core::sql::completion::completion_clause::CompletionClause;
use crate::core::sql::completion::scope_entry::ScopeEntry;
use crate::core::sql::document::text_range::TextRange;

/// What the caret looks like, derived from the text around it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionContext {
    /// Partially typed name, without the qualifier.
    pub prefix: String,
    /// Dotted parts before the prefix, e.g. `["public"]` for `public.us`.
    pub qualifiers: Vec<String>,
    /// Range the prefix occupies, which is what accepting replaces.
    pub replace_range: TextRange,
    pub clause: CompletionClause,
    /// Relations visible in the current statement.
    pub scope: Vec<ScopeEntry>,
}

impl CompletionContext {
    /// A qualifier written as `schema.relation` or `schema.`.
    pub fn qualifier(&self) -> Option<&str> {
        self.qualifiers.first().map(String::as_str)
    }

    /// The relation name of a two part qualifier such as `public.users.`.
    pub fn qualified_relation(&self) -> Option<&str> {
        (self.qualifiers.len() >= 2).then(|| self.qualifiers[1].as_str())
    }

    /// Schema of a two part qualifier.
    pub fn qualified_schema(&self) -> Option<&str> {
        (self.qualifiers.len() >= 2).then(|| self.qualifiers[0].as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::sql::document::text_position::TextPosition;

    fn context(qualifiers: Vec<&str>) -> CompletionContext {
        CompletionContext {
            prefix: "us".to_string(),
            qualifiers: qualifiers.iter().map(|part| part.to_string()).collect(),
            replace_range: TextRange::new(TextPosition::new(0, 0), TextPosition::new(0, 2)),
            clause: CompletionClause::From,
            scope: Vec::new(),
        }
    }

    #[test]
    fn test_single_qualifier_has_no_relation() {
        let context = context(vec!["public"]);

        assert_eq!(context.qualifier(), Some("public"));
        assert_eq!(context.qualified_relation(), None);
        assert_eq!(context.qualified_schema(), None);
    }

    #[test]
    fn test_two_qualifiers_are_schema_and_relation() {
        let context = context(vec!["public", "users"]);

        assert_eq!(context.qualified_schema(), Some("public"));
        assert_eq!(context.qualified_relation(), Some("users"));
    }

    #[test]
    fn test_no_qualifier() {
        assert_eq!(context(Vec::new()).qualifier(), None);
    }
}
