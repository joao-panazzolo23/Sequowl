use crate::core::sql::analysis::diagnostic::Diagnostic;
use crate::core::sql::analysis::diagnostic_severity::DiagnosticSeverity;
use crate::core::sql::analysis::highlight_span::HighlightSpan;
use crate::core::sql::analysis::source_line::SourceLine;
use crate::core::sql::analysis::source_token::SourceToken;
use crate::core::sql::document::text_position::TextPosition;
use crate::core::sql::document::text_range::TextRange;

/// Everything the editor learned about the current text in one pass.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SqlAnalysis {
    /// Colored runs, ordered by row and column.
    pub spans: Vec<HighlightSpan>,
    /// Token stream without layout, used by completion.
    pub tokens: Vec<SourceToken>,
    /// One entry per source line.
    pub lines: Vec<SourceLine>,
    /// Range of every non empty statement.
    pub statements: Vec<TextRange>,
    /// Problems found, ordered by position.
    pub diagnostics: Vec<Diagnostic>,
}

impl SqlAnalysis {
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Warning)
            .count()
    }

    /// Statement containing a position, preferring the closest following one
    /// so the caret at the end of a line still belongs to that line.
    pub fn statement_at(&self, position: TextPosition) -> Option<TextRange> {
        self.statements
            .iter()
            .find(|range| range.start <= position && position <= range.end)
            .copied()
    }

    /// Line holding a position.
    pub fn line_at(&self, row: u32) -> Option<SourceLine> {
        self.lines.get(row as usize).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::sql::analysis::diagnostic_code::DiagnosticCode;

    fn analysis() -> SqlAnalysis {
        SqlAnalysis {
            spans: Vec::new(),
            tokens: Vec::new(),
            lines: vec![SourceLine::new(0, 6), SourceLine::new(1, 5)],
            statements: vec![TextRange::new(
                TextPosition::new(0, 0),
                TextPosition::new(0, 6),
            )],
            diagnostics: vec![
                Diagnostic::error(DiagnosticCode::SyntaxError, "bad", TextRange::default()),
                Diagnostic::warning(
                    DiagnosticCode::IncompleteStatement,
                    "later",
                    TextRange::default(),
                ),
            ],
        }
    }

    #[test]
    fn test_counts_are_split_by_severity() {
        assert_eq!(analysis().error_count(), 1);
        assert_eq!(analysis().warning_count(), 1);
    }

    #[test]
    fn test_statement_lookup_covers_start_middle_and_end() {
        let analysis = analysis();
        let range = TextRange::new(TextPosition::new(0, 0), TextPosition::new(0, 6));

        assert_eq!(analysis.statement_at(TextPosition::new(0, 0)), Some(range));
        assert_eq!(analysis.statement_at(TextPosition::new(0, 3)), Some(range));
        assert_eq!(analysis.statement_at(TextPosition::new(0, 6)), Some(range));
        assert_eq!(analysis.statement_at(TextPosition::new(1, 0)), None);
    }

    #[test]
    fn test_line_lookup_by_row() {
        assert_eq!(analysis().line_at(1), Some(SourceLine::new(1, 5)));
        assert_eq!(analysis().line_at(7), None);
    }
}
