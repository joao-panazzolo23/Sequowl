/// How serious a diagnostic is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severities_differ() {
        assert_ne!(DiagnosticSeverity::Error, DiagnosticSeverity::Warning);
    }
}
