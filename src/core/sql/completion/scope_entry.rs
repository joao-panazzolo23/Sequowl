/// A relation that is in scope for the current statement, with the alias it
/// was given, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeEntry {
    /// Name the relation is known by in the statement, the alias when present.
    pub name: String,
    /// Original relation name, which may differ from `name` when aliased.
    pub relation: String,
    /// Schema qualifier written in the statement, when present.
    pub schema: Option<String>,
}

impl ScopeEntry {
    pub fn new(relation: impl Into<String>) -> Self {
        let relation = relation.into();
        Self {
            name: relation.clone(),
            relation,
            schema: None,
        }
    }

    pub fn qualified(relation: impl Into<String>, schema: impl Into<String>) -> Self {
        let mut entry = Self::new(relation);
        entry.schema = Some(schema.into());
        entry
    }

    pub fn aliased(mut self, alias: impl Into<String>) -> Self {
        self.name = alias.into();
        self
    }

    /// Whether this entry answers to the given name, alias included.
    pub fn answers_to(&self, name: &str) -> bool {
        self.name.eq_ignore_ascii_case(name) || self.relation.eq_ignore_ascii_case(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_relation_answers_to_its_name() {
        let entry = ScopeEntry::new("users");

        assert!(entry.answers_to("users"));
        assert!(entry.answers_to("USERS"));
        assert!(!entry.answers_to("u"));
    }

    #[test]
    fn test_aliased_relation_answers_to_both_names() {
        let entry = ScopeEntry::new("users").aliased("u");

        assert_eq!(entry.name, "u");
        assert_eq!(entry.relation, "users");
        assert!(entry.answers_to("u"));
        assert!(entry.answers_to("users"));
    }

    #[test]
    fn test_qualified_entry_keeps_its_schema() {
        let entry = ScopeEntry::qualified("users", "public");

        assert_eq!(entry.schema.as_deref(), Some("public"));
    }
}
