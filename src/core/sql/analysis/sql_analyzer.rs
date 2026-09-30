use sqlparser::keywords::Keyword;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{
    Span, Token, TokenWithSpan, Tokenizer, TokenizerError, Whitespace, Word,
};

use crate::core::sql::analysis::diagnostic::Diagnostic;
use crate::core::sql::analysis::diagnostic_code::DiagnosticCode;
use crate::core::sql::analysis::highlight_span::HighlightSpan;
use crate::core::sql::analysis::source_line::SourceLine;
use crate::core::sql::analysis::source_token::SourceToken;
use crate::core::sql::analysis::sql_analysis::SqlAnalysis;
use crate::core::sql::analysis::token_kind::TokenKind;
use crate::core::sql::dialect::sql_dialect::SqlDialect;
use crate::core::sql::document::text_position::TextPosition;
use crate::core::sql::document::text_range::TextRange;

/// Turns SQL text into everything the editor needs: colored spans, the token
/// stream used by completion, source lines, statement ranges and
/// diagnostics.
///
/// The analyzer is a pure function of the text and the dialect, so it can be
/// re-run on every keystroke and unit tested without any UI.
pub struct SqlAnalyzer;

/// Guards against pathological nesting while typing, e.g. `((((((`.
const RECURSION_LIMIT: usize = 64;

impl SqlAnalyzer {
    pub fn analyze(sql: &str, dialect: &dyn SqlDialect) -> SqlAnalysis {
        let mut analysis = SqlAnalysis {
            lines: source_lines(sql),
            ..Default::default()
        };

        let mut buffer: Vec<TokenWithSpan> = Vec::new();
        let tokenization = {
            let mut tokenizer = Tokenizer::new(dialect.parser_dialect(), sql);
            tokenizer.tokenize_with_location_into_buf(&mut buffer)
        };

        if let Err(error) = tokenization {
            analysis
                .diagnostics
                .push(unterminated_diagnostic(&error, &analysis.lines));
        }

        collect_tokens(&buffer, dialect, &mut analysis);
        collect_statements(&buffer, dialect, &mut analysis);

        analysis
    }
}

/// One entry per line of the source, needed for the gutter and for clamping
/// spans that cross a line break.
fn source_lines(sql: &str) -> Vec<SourceLine> {
    let mut lines = Vec::new();
    let mut length = 0u32;
    let mut row = 0u32;

    for ch in sql.chars() {
        if ch == '\n' {
            lines.push(SourceLine::new(row, length));
            row += 1;
            length = 0;
        } else {
            length += 1;
        }
    }
    lines.push(SourceLine::new(row, length));
    lines
}

/// Classifies every token once, keeping the significant ones for completion.
fn collect_tokens(buffer: &[TokenWithSpan], dialect: &dyn SqlDialect, analysis: &mut SqlAnalysis) {
    for (index, token) in buffer.iter().enumerate() {
        let kind = classify(&token.token, dialect, is_call(buffer, index));

        if kind == TokenKind::Whitespace {
            continue;
        }

        let range = span_to_range(token.span);
        let row = range.start.line as u32;
        let column = range.start.column as u32;
        // A token that crosses a line break, such as a block comment, is only
        // visible up to the end of the line it starts on.
        let available =
            line_length(&analysis.lines, range.start.line).saturating_sub(column as usize);
        let length = if range.start.line == range.end.line {
            range.char_len().min(available)
        } else {
            available
        };

        analysis
            .spans
            .push(HighlightSpan::new(row, column, length as u32, kind));

        if kind.is_significant() {
            analysis.tokens.push(SourceToken { range, kind });
        }
    }
}

/// Splits the token stream on the statement separator and validates every
/// statement on its own, so one broken statement does not hide the others.
fn collect_statements(
    buffer: &[TokenWithSpan],
    dialect: &dyn SqlDialect,
    analysis: &mut SqlAnalysis,
) {
    let mut group: Vec<&TokenWithSpan> = Vec::new();

    for token in buffer {
        if matches!(token.token, Token::SemiColon) {
            validate_statement(&group, Some(token), dialect, analysis);
            group.clear();
        } else {
            group.push(token);
        }
    }
    validate_statement(&group, None, dialect, analysis);
}

/// Validates one statement. `terminator` is the separator that closed it, if
/// the source had one: parsing stops there, so a statement followed by `;`
/// reports its errors on the separator while a statement at the end of the
/// file runs into the end of the input.
fn validate_statement(
    group: &[&TokenWithSpan],
    terminator: Option<&TokenWithSpan>,
    dialect: &dyn SqlDialect,
    analysis: &mut SqlAnalysis,
) {
    let Some(first) = group.iter().find(|token| !is_trivia(&token.token)) else {
        return;
    };
    let last = group
        .iter()
        .rposition(|token| !is_trivia(&token.token))
        .map(|index| group[index])
        .unwrap_or(first);

    let range = TextRange::new(
        span_to_position(first.span.start),
        span_to_position(last.span.end),
    );
    analysis.statements.push(range);

    let mut tokens: Vec<TokenWithSpan> = group.iter().map(|token| (*token).clone()).collect();
    if let Some(terminator) = terminator {
        tokens.push(terminator.clone());
    }
    let mut parser = Parser::new(dialect.parser_dialect())
        .with_recursion_limit(RECURSION_LIMIT)
        .with_tokens_with_locations(tokens);

    if let Err(error) = parser.parse_statement() {
        let failing = parser.token_at(parser.get_current_index()).span;

        // An empty span is the end of the input: the statement is unfinished
        // rather than wrong, and the caret sits right after the last token.
        if is_end_of_input(failing) {
            analysis.diagnostics.push(Diagnostic::warning(
                DiagnosticCode::IncompleteStatement,
                format!("Incomplete statement: {error}"),
                range,
            ));
        } else {
            analysis.diagnostics.push(Diagnostic::error(
                DiagnosticCode::SyntaxError,
                error.to_string(),
                span_to_range(failing),
            ));
        }
    }
}

/// The tokenizer stops at the first problem, so the rest of the line is
/// underlined up to its end.
fn unterminated_diagnostic(error: &TokenizerError, lines: &[SourceLine]) -> Diagnostic {
    let start = if error.location.line == 0 {
        last_position(lines)
    } else {
        span_to_position(error.location)
    };
    let end = TextPosition::new(start.line, line_length(lines, start.line));

    Diagnostic::error(
        DiagnosticCode::UnterminatedLiteral,
        error.to_string(),
        TextRange::new(start, end.max(start)),
    )
}

/// Lexical class of a token, using the dialect for words.
fn classify(token: &Token, dialect: &dyn SqlDialect, is_call: bool) -> TokenKind {
    match token {
        Token::Word(word) => classify_word(word, dialect, is_call),
        Token::Number(_, _) | Token::HexStringLiteral(_) => TokenKind::Number,
        Token::Char(_)
        | Token::SingleQuotedString(_)
        | Token::TripleSingleQuotedString(_)
        | Token::TripleDoubleQuotedString(_)
        | Token::DollarQuotedString(_)
        | Token::SingleQuotedByteStringLiteral(_)
        | Token::DoubleQuotedByteStringLiteral(_)
        | Token::TripleSingleQuotedByteStringLiteral(_)
        | Token::TripleDoubleQuotedByteStringLiteral(_)
        | Token::SingleQuotedRawStringLiteral(_)
        | Token::DoubleQuotedRawStringLiteral(_)
        | Token::TripleSingleQuotedRawStringLiteral(_)
        | Token::TripleDoubleQuotedRawStringLiteral(_)
        | Token::NationalStringLiteral(_)
        | Token::QuoteDelimitedStringLiteral(_)
        | Token::NationalQuoteDelimitedStringLiteral(_)
        | Token::EscapedStringLiteral(_)
        | Token::UnicodeStringLiteral(_) => TokenKind::String,
        Token::DoubleQuotedString(_) => TokenKind::QuotedIdentifier,
        Token::Whitespace(Whitespace::SingleLineComment { .. })
        | Token::Whitespace(Whitespace::MultiLineComment(_)) => TokenKind::Comment,
        Token::Whitespace(_) => TokenKind::Whitespace,
        Token::Comma
        | Token::LParen
        | Token::RParen
        | Token::LBracket
        | Token::RBracket
        | Token::LBrace
        | Token::RBrace
        | Token::SemiColon
        | Token::Period
        | Token::Colon
        | Token::DoubleColon
        | Token::AtSign => TokenKind::Punctuation,
        Token::DoubleEq
        | Token::Eq
        | Token::Neq
        | Token::Lt
        | Token::Gt
        | Token::LtEq
        | Token::GtEq
        | Token::Spaceship
        | Token::Plus
        | Token::Minus
        | Token::Mul
        | Token::Div
        | Token::DuckIntDiv
        | Token::Mod
        | Token::StringConcat
        | Token::Assignment
        | Token::Backslash
        | Token::Ampersand
        | Token::Pipe
        | Token::Caret
        | Token::RArrow
        | Token::Sharp
        | Token::DoubleSharp
        | Token::Tilde
        | Token::TildeAsterisk
        | Token::ExclamationMarkTilde
        | Token::ExclamationMarkTildeAsterisk
        | Token::DoubleTilde
        | Token::DoubleTildeAsterisk
        | Token::ExclamationMarkDoubleTilde
        | Token::ExclamationMarkDoubleTildeAsterisk
        | Token::ShiftLeft
        | Token::ShiftRight
        | Token::Overlap
        | Token::ExclamationMark
        | Token::DoubleExclamationMark
        | Token::CaretAt
        | Token::PGSquareRoot
        | Token::PGCubeRoot
        | Token::Arrow
        | Token::LongArrow
        | Token::HashArrow
        | Token::AtDashAt
        | Token::QuestionMarkDash
        | Token::AmpersandLeftAngleBracket
        | Token::AmpersandRightAngleBracket
        | Token::AmpersandLeftAngleBracketVerticalBar
        | Token::VerticalBarAmpersandRightAngleBracket
        | Token::TwoWayArrow
        | Token::LeftAngleBracketCaret
        | Token::RightAngleBracketCaret
        | Token::QuestionMarkSharp
        | Token::QuestionMarkDashVerticalBar
        | Token::QuestionMarkDoubleVerticalBar
        | Token::TildeEqual
        | Token::ShiftLeftVerticalBar
        | Token::VerticalBarShiftRight
        | Token::VerticalBarRightAngleBracket
        | Token::HashLongArrow
        | Token::AtArrow
        | Token::ArrowAt
        | Token::HashMinus
        | Token::AtQuestion
        | Token::AtAt
        | Token::Question
        | Token::QuestionAnd
        | Token::QuestionPipe
        | Token::CustomBinaryOperator(_) => TokenKind::Operator,
        Token::Placeholder(_) => TokenKind::Parameter,
        Token::EOF => TokenKind::Unknown,
    }
}

fn classify_word(word: &Word, dialect: &dyn SqlDialect, is_call: bool) -> TokenKind {
    if word.quote_style.is_some() {
        return TokenKind::QuotedIdentifier;
    }

    let is_known_function = dialect.functions().contains(&word.value);
    if is_call && (is_known_function || word.keyword == Keyword::NoKeyword) {
        return TokenKind::Function;
    }
    if is_known_function {
        return TokenKind::Function;
    }
    // The parser keyword list is far wider than SQL itself, e.g. it contains
    // `ID` and `PUBLIC`, so the dialect word list decides what is a keyword.
    if dialect.keywords().contains(&word.value) {
        return TokenKind::Keyword;
    }

    TokenKind::Identifier
}

/// Whether the token at `index` opens a call, i.e. the next significant token
/// is an opening parenthesis.
fn is_call(buffer: &[TokenWithSpan], index: usize) -> bool {
    buffer[index + 1..]
        .iter()
        .find(|token| !is_trivia(&token.token))
        .is_some_and(|token| matches!(token.token, Token::LParen))
}

fn is_trivia(token: &Token) -> bool {
    matches!(token, Token::Whitespace(_))
}

/// The parser reports the end of the input with a zeroed span.
fn is_end_of_input(span: Span) -> bool {
    span.start.line == 0
}

fn line_length(lines: &[SourceLine], line: usize) -> usize {
    lines
        .get(line)
        .map_or(0, |source_line| source_line.length as usize)
}

fn last_position(lines: &[SourceLine]) -> TextPosition {
    match lines.last() {
        Some(line) => TextPosition::new(line.row as usize, line.length as usize),
        None => TextPosition::new(0, 0),
    }
}

/// Converts a one based, inclusive token start into a zero based position.
fn span_to_position(location: sqlparser::tokenizer::Location) -> TextPosition {
    TextPosition::new(
        location.line.saturating_sub(1) as usize,
        location.column.saturating_sub(1) as usize,
    )
}

/// A token span end is the position after the last character, so only the
/// start has to be shifted.
fn span_to_range(span: Span) -> TextRange {
    TextRange::new(span_to_position(span.start), span_to_position(span.end))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::sql::analysis::diagnostic_severity::DiagnosticSeverity;
    use crate::core::sql::dialect::mysql_dialect::MySqlDialect;
    use crate::core::sql::dialect::postgres_dialect::PostgreSqlDialect;

    fn analyze(sql: &str) -> SqlAnalysis {
        SqlAnalyzer::analyze(sql, &PostgreSqlDialect::new())
    }

    fn text_at(sql: &str, position: TextPosition, length: usize) -> String {
        sql.chars()
            .skip(position.column)
            .take(length)
            .collect::<String>()
    }

    #[test]
    fn test_lines_are_counted_including_a_trailing_break() {
        let analysis = analyze("SELECT 1;\nSELECT 2;");

        assert_eq!(analysis.lines.len(), 2);
        assert_eq!(analysis.lines[0].length, 9);
        assert_eq!(analysis.lines[1].length, 9);
    }

    #[test]
    fn test_keywords_strings_numbers_and_words_are_classified() {
        let sql = "SELECT id, 'text', 42 FROM public.users";
        let analysis = analyze(sql);

        let kinds: Vec<(usize, TokenKind)> = analysis
            .spans
            .iter()
            .filter(|span| span.row == 0)
            .map(|span| (span.column as usize, span.kind))
            .collect();

        assert_eq!(kinds[0], (0, TokenKind::Keyword));
        assert!(kinds.contains(&(7, TokenKind::Identifier)));
        assert!(kinds.contains(&(11, TokenKind::String)));
        assert!(kinds.contains(&(19, TokenKind::Number)));
        assert!(kinds.contains(&(22, TokenKind::Keyword)));
        assert!(
            kinds.contains(&(27, TokenKind::Identifier)),
            "public is a name"
        );
    }

    #[test]
    fn test_a_word_followed_by_a_parenthesis_is_a_function() {
        let analysis = analyze("SELECT count(id) FROM users");

        assert!(
            analysis
                .spans
                .iter()
                .any(|span| span.column == 7 && span.kind == TokenKind::Function)
        );
    }

    #[test]
    fn test_quoted_identifiers_are_not_treated_as_strings() {
        let analysis = analyze("SELECT \"user name\" FROM users");

        assert!(
            analysis
                .spans
                .iter()
                .any(|span| span.kind == TokenKind::QuotedIdentifier)
        );
        assert!(
            !analysis
                .spans
                .iter()
                .any(|span| span.kind == TokenKind::String)
        );
    }

    #[test]
    fn test_comments_are_kept_and_their_multi_line_span_is_trimmed() {
        let analysis = analyze("/* first\nsecond */ SELECT 1");

        let comment = analysis
            .spans
            .iter()
            .find(|span| span.kind == TokenKind::Comment)
            .expect("a comment span");

        assert_eq!(comment.row, 0);
        assert_eq!(comment.length, 8, "trimmed to the first line");
    }

    #[test]
    fn test_spans_carry_the_source_text() {
        let sql = "SELECT 'héllo' AS x";
        let analysis = analyze(sql);
        let span = analysis
            .spans
            .iter()
            .find(|span| span.kind == TokenKind::String)
            .expect("a string span");

        let text = text_at(
            sql,
            TextPosition::new(0, span.column as usize),
            span.length as usize,
        );

        assert_eq!(text, "'héllo'");
    }

    #[test]
    fn test_statements_are_split_on_the_separator() {
        let analysis = analyze("SELECT 1; SELECT 2; -- trailing\n");

        assert_eq!(analysis.statements.len(), 2);
        assert_eq!(analysis.statements[0].start, TextPosition::new(0, 0));
        assert_eq!(analysis.statements[1].start.line, 0);
    }

    #[test]
    fn test_only_whitespace_and_comments_do_not_form_a_statement() {
        let analysis = analyze("-- just a note\n");

        assert!(analysis.statements.is_empty());
        assert!(analysis.diagnostics.is_empty());
    }

    #[test]
    fn test_valid_queries_produce_no_diagnostics() {
        let analysis = analyze("SELECT u.id, u.name FROM public.users u WHERE u.id = 1;");

        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
    }

    #[test]
    fn test_syntax_error_points_at_the_offending_token() {
        let sql = "SELECT * FROM public.users WHERE;";
        let analysis = analyze(sql);

        let diagnostic = analysis
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.is_error())
            .expect("a syntax error");

        assert_eq!(diagnostic.code, DiagnosticCode::SyntaxError);
        assert_eq!(
            diagnostic.range.start,
            TextPosition::new(0, 32),
            "the separator closes the statement and is where the parser stops"
        );
    }

    #[test]
    fn test_unfinished_statement_is_a_warning_on_the_last_token() {
        let analysis = analyze("SELECT * FROM");

        let diagnostic = analysis
            .diagnostics
            .first()
            .expect("an incomplete statement warning");

        assert_eq!(diagnostic.severity, DiagnosticSeverity::Warning);
        assert_eq!(diagnostic.code, DiagnosticCode::IncompleteStatement);
        assert!(
            diagnostic.message.starts_with("Incomplete statement:"),
            "{}",
            diagnostic.message
        );
    }

    #[test]
    fn test_a_broken_statement_does_not_hide_the_next_one() {
        let analysis = analyze("SELECT FROM; SELECT 1;");

        assert_eq!(analysis.statements.len(), 2);
        assert_eq!(analysis.error_count(), 1);
    }

    #[test]
    fn test_unterminated_string_is_reported_once() {
        let analysis = analyze("SELECT 'unterminated");

        assert_eq!(analysis.error_count(), 1);
        assert_eq!(
            analysis.diagnostics[0].code,
            DiagnosticCode::UnterminatedLiteral
        );
    }

    #[test]
    fn test_empty_input_produces_nothing() {
        let analysis = analyze("");

        assert_eq!(analysis.lines.len(), 1);
        assert!(analysis.spans.is_empty());
        assert!(analysis.diagnostics.is_empty());
    }

    #[test]
    fn test_quoting_follows_the_dialect() {
        let sql = "SELECT `id` FROM t WHERE x = 1";

        let postgres = SqlAnalyzer::analyze(sql, &PostgreSqlDialect::new());
        let mysql = SqlAnalyzer::analyze(sql, &MySqlDialect::new());

        assert!(
            !postgres
                .spans
                .iter()
                .any(|span| span.kind == TokenKind::QuotedIdentifier),
            "PostgreSQL does not accept backticks as identifier quotes"
        );
        assert!(
            mysql
                .spans
                .iter()
                .any(|span| span.kind == TokenKind::QuotedIdentifier),
            "MySQL reads backticks as an identifier"
        );
    }

    #[test]
    fn test_dialect_keywords_drive_highlighting() {
        let sql = "SELECT auto_increment FROM id";

        let postgres = SqlAnalyzer::analyze(sql, &PostgreSqlDialect::new());
        let mysql = SqlAnalyzer::analyze(sql, &MySqlDialect::new());

        assert!(
            !postgres
                .spans
                .iter()
                .any(|span| span.column == 7 && span.kind == TokenKind::Keyword),
            "AUTO_INCREMENT is not a PostgreSQL keyword"
        );
        assert!(
            mysql
                .spans
                .iter()
                .any(|span| span.column == 7 && span.kind == TokenKind::Keyword),
            "AUTO_INCREMENT is a MySQL keyword"
        );
    }

    #[test]
    fn test_deeply_nested_input_does_not_panic() {
        let sql = format!("SELECT {}{}", "(".repeat(200), ")".repeat(200));
        let analysis = analyze(&sql);

        assert!(!analysis.spans.is_empty());
    }
}
