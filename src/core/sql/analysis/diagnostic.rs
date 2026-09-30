use crate::core::sql::analysis::diagnostic_code::DiagnosticCode;
use crate::core::sql::analysis::diagnostic_severity::DiagnosticSeverity;
use crate::core::sql::document::text_range::TextRange;

/// A problem found in the document, anchored to a range of text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub code: DiagnosticCode,
    pub message: String,
    pub range: TextRange,
}

impl Diagnostic {
    pub fn error(code: DiagnosticCode, message: impl Into<String>, range: TextRange) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            code,
            message: message.into(),
            range,
        }
    }

    pub fn warning(code: DiagnosticCode, message: impl Into<String>, range: TextRange) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            code,
            message: message.into(),
            range,
        }
    }

    pub fn is_error(&self) -> bool {
        self.severity == DiagnosticSeverity::Error
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::sql::document::text_position::TextPosition;

    fn range() -> TextRange {
        TextRange::new(TextPosition::new(0, 0), TextPosition::new(0, 3))
    }

    #[test]
    fn test_severity_is_part_of_the_diagnostic() {
        let error = Diagnostic::error(DiagnosticCode::SyntaxError, "boom", range());
        let warning = Diagnostic::warning(DiagnosticCode::IncompleteStatement, "later", range());

        assert!(error.is_error());
        assert!(!warning.is_error());
    }

    #[test]
    fn test_message_is_stored() {
        let diagnostic = Diagnostic::error(DiagnosticCode::SyntaxError, "boom", range());

        assert_eq!(diagnostic.message, "boom");
    }
}
