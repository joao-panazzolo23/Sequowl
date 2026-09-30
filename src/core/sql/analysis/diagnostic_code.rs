/// Stable identifier of what the analyzer found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticCode {
    /// The parser rejected the statement.
    SyntaxError,
    /// A literal, identifier or comment was not closed.
    UnterminatedLiteral,
    /// The statement ends before it is complete, e.g. while typing.
    IncompleteStatement,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codes_differ() {
        assert_ne!(
            DiagnosticCode::SyntaxError,
            DiagnosticCode::UnterminatedLiteral
        );
        assert_ne!(
            DiagnosticCode::IncompleteStatement,
            DiagnosticCode::UnterminatedLiteral
        );
    }
}
