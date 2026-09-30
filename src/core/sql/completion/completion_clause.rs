/// The SQL clause the caret is in, which decides what may be suggested.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompletionClause {
    /// Start of a statement.
    #[default]
    StatementStart,
    SelectList,
    From,
    Join,
    On,
    Using,
    Where,
    GroupBy,
    Having,
    OrderBy,
    Set,
    Values,
    Into,
    Update,
    Create,
    Returning,
    /// Inside a parenthesised expression of unknown shape.
    Unknown,
}

impl CompletionClause {
    /// Whether columns of the tables in scope make sense here.
    pub fn expects_columns(self) -> bool {
        matches!(
            self,
            CompletionClause::SelectList
                | CompletionClause::Where
                | CompletionClause::GroupBy
                | CompletionClause::Having
                | CompletionClause::OrderBy
                | CompletionClause::On
                | CompletionClause::Set
                | CompletionClause::Values
                | CompletionClause::Returning
                | CompletionClause::Using
                | CompletionClause::Unknown
        )
    }

    /// Whether relations make sense here, either bare or qualified.
    pub fn expects_relations(self) -> bool {
        matches!(
            self,
            CompletionClause::From
                | CompletionClause::Join
                | CompletionClause::Into
                | CompletionClause::Update
        )
    }

    /// Whether functions make sense here.
    pub fn expects_functions(self) -> bool {
        !matches!(
            self,
            CompletionClause::StatementStart
                | CompletionClause::From
                | CompletionClause::Join
                | CompletionClause::Create
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_where_expects_columns_and_functions() {
        let clause = CompletionClause::Where;

        assert!(clause.expects_columns());
        assert!(clause.expects_functions());
        assert!(!clause.expects_relations());
    }

    #[test]
    fn test_from_expects_relations_only() {
        let clause = CompletionClause::From;

        assert!(clause.expects_relations());
        assert!(!clause.expects_columns());
        assert!(!clause.expects_functions());
    }

    #[test]
    fn test_statement_start_offers_no_expressions() {
        let clause = CompletionClause::StatementStart;

        assert!(!clause.expects_columns());
        assert!(!clause.expects_functions());
    }
}
