use std::collections::HashSet;

use crate::core::sql::analysis::source_token::SourceToken;
use crate::core::sql::analysis::sql_analysis::SqlAnalysis;
use crate::core::sql::analysis::token_kind::TokenKind;
use crate::core::sql::catalog::schema_catalog::SchemaCatalog;
use crate::core::sql::completion::clause_keywords::keywords_for;
use crate::core::sql::completion::completion_clause::CompletionClause;
use crate::core::sql::completion::completion_context::CompletionContext;
use crate::core::sql::completion::completion_item::CompletionItem;
use crate::core::sql::completion::completion_kind::CompletionKind;
use crate::core::sql::completion::completion_result::CompletionResult;
use crate::core::sql::completion::scope_entry::ScopeEntry;
use crate::core::sql::dialect::sql_dialect::SqlDialect;
use crate::core::sql::document::text_document::TextDocument;
use crate::core::sql::document::text_position::TextPosition;
use crate::core::sql::document::text_range::TextRange;

/// Builds the completion list for a caret position.
///
/// The engine is a pure function of the document, the last analysis and the
/// connected catalog, so it can be unit tested without any UI.
pub struct CompletionEngine;

/// Enough items for a long popup; the list is scrolled, not truncated, in the
/// UI.
const MAX_ITEMS: usize = 100;

impl CompletionEngine {
    pub fn complete(
        document: &TextDocument,
        cursor: TextPosition,
        analysis: &SqlAnalysis,
        catalog: &SchemaCatalog,
        dialect: &dyn SqlDialect,
    ) -> CompletionResult {
        let (prefix, qualifiers, replace_range) = read_prefix(document, cursor, dialect);
        let statement = analysis.statement_at(cursor);
        let context = read_context(
            document,
            analysis,
            prefix,
            qualifiers,
            replace_range,
            statement,
        );
        let items = candidates(&context, catalog, dialect);

        CompletionResult::new(rank(items, &context.prefix), context.replace_range)
    }
}

/// Reads the partially typed name and the dotted parts in front of it.
fn read_prefix(
    document: &TextDocument,
    cursor: TextPosition,
    dialect: &dyn SqlDialect,
) -> (String, Vec<String>, TextRange) {
    let cursor = document.clamp_position(cursor);
    let (prefix, start) = read_word_backwards(document, cursor, dialect.quote_style());
    let mut qualifiers = Vec::new();
    let mut position = TextPosition::new(cursor.line, start);

    // Walk left over `a.b.` chains, keeping the parts in written order.
    while qualifiers.len() < 3 {
        let Some(dot) = last_char_backwards(document, position) else {
            break;
        };
        if document.char_at(dot) != Some('.') {
            break;
        }

        let (qualifier, qualifier_start) =
            read_word_backwards(document, dot, dialect.quote_style());
        if qualifier.is_empty() {
            break;
        }

        qualifiers.insert(0, qualifier);
        position = TextPosition::new(cursor.line, qualifier_start);
    }

    (
        prefix,
        qualifiers,
        TextRange::new(TextPosition::new(cursor.line, start), cursor),
    )
}

/// Reads a name backwards, returning it and the column it starts at.
///
/// A quoted name is read as the text between the quotes and does not include
/// them, so accepting a suggestion keeps the quotes in place.
fn read_word_backwards(
    document: &TextDocument,
    from: TextPosition,
    quote: char,
) -> (String, usize) {
    let line = document.line_text_at(from);
    let chars: Vec<char> = line.chars().collect();
    let end = from.column.min(chars.len());

    if end > 0 && chars[end - 1] == quote {
        let mut start = end - 1;
        while start > 0 && chars[start - 1] != quote {
            start -= 1;
        }
        return (chars[start + 1..end].iter().collect(), start + 1);
    }

    let mut start = end;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }
    (chars[start..end].iter().collect(), start)
}

/// Position of the last non whitespace character at or before a position.
fn last_char_backwards(document: &TextDocument, from: TextPosition) -> Option<TextPosition> {
    let line = document.line_text_at(from);
    let chars: Vec<char> = line.chars().collect();
    let mut column = from.column.min(chars.len());

    while column > 0 && chars[column - 1].is_whitespace() {
        column -= 1;
    }

    (column > 0).then(|| TextPosition::new(from.line, column - 1))
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '$'
}

/// Derives the clause and the relations in scope around the caret.
///
/// The clause only depends on what is written before the caret, while the
/// scope needs the whole statement: `SELECT u.| FROM users u` resolves `u`
/// from text that follows the caret.
fn read_context(
    document: &TextDocument,
    analysis: &SqlAnalysis,
    prefix: String,
    qualifiers: Vec<String>,
    replace_range: TextRange,
    statement: Option<TextRange>,
) -> CompletionContext {
    let before: Vec<&SourceToken> = analysis
        .tokens
        .iter()
        .filter(|token| token.range.end <= replace_range.start)
        .collect();
    let in_statement: Vec<&SourceToken> = analysis
        .tokens
        .iter()
        .filter(|token| match statement {
            Some(range) => range.start <= token.range.start && token.range.end <= range.end,
            None => token.range.end <= replace_range.start,
        })
        .collect();

    CompletionContext {
        prefix,
        qualifiers,
        replace_range,
        clause: read_clause(document, &before),
        scope: read_scope(document, &in_statement),
    }
}

/// Clause the caret sits in, tracked per nesting level so a subquery does not
/// hide the clause of the statement that contains it.
fn read_clause(document: &TextDocument, before: &[&SourceToken]) -> CompletionClause {
    let mut clauses = vec![CompletionClause::StatementStart];

    for token in before {
        let text = document.text_in(token.range);
        match token.kind {
            TokenKind::Keyword => {
                let Some(clause) = clause_of(&text.to_ascii_uppercase()) else {
                    continue;
                };
                let depth = clauses.len() - 1;
                clauses[depth] = clause;
            }
            TokenKind::Punctuation => match text {
                "(" => {
                    let current = *clauses.last().unwrap_or(&CompletionClause::Unknown);
                    clauses.push(current);
                }
                ")" => {
                    clauses.pop();
                }
                _ => {}
            },
            _ => {}
        }
    }

    clauses.last().copied().unwrap_or_default()
}

/// Relations referenced by a statement, with the aliases they were given.
///
/// Only the outer level is collected: a table of a subquery is not visible
/// outside of it.
fn read_scope(document: &TextDocument, tokens: &[&SourceToken]) -> Vec<ScopeEntry> {
    let mut scope: Vec<ScopeEntry> = Vec::new();
    let mut depth = 0usize;
    let mut expect_relation = false;
    let mut pending_qualifier: Option<String> = None;

    for (index, token) in tokens.iter().enumerate() {
        let text = document.text_in(token.range);

        match token.kind {
            TokenKind::Keyword => {
                // A keyword ends whatever was being qualified, so a dangling
                // `alias.` in the select list is not mistaken for a schema.
                pending_qualifier = None;
                expect_relation = matches!(
                    text.to_ascii_uppercase().as_str(),
                    "FROM" | "JOIN" | "UPDATE" | "INTO"
                );
            }
            TokenKind::Punctuation => match text {
                "(" => depth += 1,
                ")" => depth = depth.saturating_sub(1),
                // Another relation of the same FROM or JOIN follows.
                "," => expect_relation = true,
                _ => {}
            },
            TokenKind::Identifier | TokenKind::QuotedIdentifier if depth == 0 => {
                if follows_dot(document, tokens, index) {
                    // A qualifier: either the schema of a relation, or the
                    // alias of a column reference such as `s.token`.
                    pending_qualifier = Some(text.to_string());
                } else if let Some(schema) = pending_qualifier.take() {
                    if expect_relation {
                        scope.push(ScopeEntry::qualified(text, schema));
                    }
                } else if expect_relation {
                    let entry = ScopeEntry::new(text);
                    scope.push(match alias_after(document, tokens, index) {
                        Some(alias) => entry.aliased(alias),
                        None => entry,
                    });
                } else {
                    continue;
                }
                expect_relation = false;
            }
            _ => {}
        }
    }

    scope
}

/// Whether the token is directly followed by a dot, e.g. `public` in
/// `public.users`.
fn follows_dot(document: &TextDocument, tokens: &[&SourceToken], index: usize) -> bool {
    tokens.get(index + 1).is_some_and(|next| {
        next.kind == TokenKind::Punctuation && document.text_in(next.range) == "."
    })
}

/// Clause a keyword switches to, if any.
fn clause_of(keyword: &str) -> Option<CompletionClause> {
    let clause = match keyword {
        "SELECT" => CompletionClause::SelectList,
        "FROM" => CompletionClause::From,
        "INNER" | "LEFT" | "RIGHT" | "FULL" | "CROSS" | "OUTER" | "NATURAL" => {
            CompletionClause::Join
        }
        "ON" => CompletionClause::On,
        "USING" => CompletionClause::Using,
        "WHERE" => CompletionClause::Where,
        "GROUP" => CompletionClause::GroupBy,
        "HAVING" => CompletionClause::Having,
        "ORDER" => CompletionClause::OrderBy,
        "SET" => CompletionClause::Set,
        "VALUES" => CompletionClause::Values,
        "INTO" => CompletionClause::Into,
        "INSERT" => CompletionClause::Into,
        "UPDATE" => CompletionClause::Update,
        "RETURNING" => CompletionClause::Returning,
        "CREATE" | "ALTER" | "DROP" | "TRUNCATE" => CompletionClause::Create,
        _ => return None,
    };
    Some(clause)
}

/// Alias written right after a relation, e.g. `users u`.
fn alias_after(document: &TextDocument, tokens: &[&SourceToken], index: usize) -> Option<String> {
    let next = tokens.get(index + 1)?;
    if next.kind != TokenKind::Identifier && next.kind != TokenKind::QuotedIdentifier {
        return None;
    }
    // `count(*)` and `cast(x AS y)` are not aliases.
    tokens
        .get(index + 2)
        .filter(|following| {
            following.kind == TokenKind::Punctuation && document.text_in(following.range) != ";"
        })
        .is_none()
        .then(|| document.text_in(next.range).to_string())
}

/// Everything that could be inserted at this position.
fn candidates(
    context: &CompletionContext,
    catalog: &SchemaCatalog,
    dialect: &dyn SqlDialect,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let push =
        |items: &mut Vec<CompletionItem>, seen: &mut HashSet<String>, item: CompletionItem| {
            if seen.insert(format!(
                "{:?}{}",
                item.kind,
                item.label.to_ascii_lowercase()
            )) {
                items.push(item);
            }
        };

    // `schema.relation.` leaves only the columns of that relation.
    if let Some(relation) = context.qualified_relation() {
        push_columns(
            &mut items,
            &mut seen,
            catalog,
            context.qualified_schema(),
            relation,
        );
    } else if let Some(qualifier) = context.qualifier() {
        // `qualifier.` is either something in scope or a schema.
        if let Some(entry) = context
            .scope
            .iter()
            .find(|entry| entry.answers_to(qualifier))
        {
            let (schema, relation) = (entry.schema.clone(), entry.relation.clone());
            push_columns(&mut items, &mut seen, catalog, schema.as_deref(), &relation);
        } else if catalog.schema(qualifier).is_some() {
            push_relations(&mut items, &mut seen, catalog, Some(qualifier));
        } else {
            push_columns(&mut items, &mut seen, catalog, None, qualifier);
        }
    } else {
        if context.clause.expects_columns() {
            for entry in &context.scope {
                let (schema, relation) = (entry.schema.clone(), entry.relation.clone());
                push_columns(&mut items, &mut seen, catalog, schema.as_deref(), &relation);
            }
        }
        if context.clause.expects_relations() || context.scope.is_empty() {
            for schema in catalog.schemas() {
                let name = schema.name.clone();
                let count = schema.tables.len();
                push(
                    &mut items,
                    &mut seen,
                    CompletionItem::new(name, CompletionKind::Schema, format!("{count} relations")),
                );
            }
            push_relations(&mut items, &mut seen, catalog, None);
        }
    }

    for keyword in keywords_for(context.clause) {
        push(
            &mut items,
            &mut seen,
            CompletionItem::keyword((*keyword).to_string()),
        );
    }

    if context.clause.expects_functions() {
        for function in dialect.functions().words() {
            push(
                &mut items,
                &mut seen,
                CompletionItem::function(function.to_string()),
            );
        }
    }

    items
}

fn push_columns(
    items: &mut Vec<CompletionItem>,
    seen: &mut HashSet<String>,
    catalog: &SchemaCatalog,
    schema: Option<&str>,
    relation: &str,
) {
    let Some(columns) = catalog.columns(schema, relation) else {
        return;
    };
    for column in columns {
        let key = format!(
            "{:?}{}",
            CompletionKind::Column,
            column.name.to_ascii_lowercase()
        );
        if seen.insert(key) {
            items.push(CompletionItem::column(
                column.name.clone(),
                column.describe(),
            ));
        }
    }
}

fn push_relations(
    items: &mut Vec<CompletionItem>,
    seen: &mut HashSet<String>,
    catalog: &SchemaCatalog,
    schema: Option<&str>,
) {
    for (owner, table) in catalog.relations() {
        if schema.is_some_and(|schema| !owner.name.eq_ignore_ascii_case(schema)) {
            continue;
        }
        let kind = match table.kind {
            crate::core::entities::relation_kind::RelationKind::Table => CompletionKind::Table,
            crate::core::entities::relation_kind::RelationKind::View => CompletionKind::View,
        };
        let key = format!("{kind:?}{}", table.name.to_ascii_lowercase());
        if seen.insert(key) {
            items.push(CompletionItem::new(
                table.name.clone(),
                kind,
                owner.name.clone(),
            ));
        }
    }
}

/// Sorts by how close the label is, then by kind, length and spelling.
fn rank(items: Vec<CompletionItem>, prefix: &str) -> Vec<CompletionItem> {
    let needle = prefix.to_ascii_lowercase();
    let mut scored: Vec<(RankKey, CompletionItem)> = items
        .into_iter()
        .filter_map(|item| {
            let key = RankKey {
                kind: item.kind.priority(),
                distance: match_score(&item.label, &needle)?,
                length: item.label.chars().count(),
                label: item.label.clone(),
            };
            Some((key, item))
        })
        .collect();

    scored.sort_by(|(left, _), (right, _)| left.cmp(right));

    scored
        .into_iter()
        .take(MAX_ITEMS)
        .map(|(_, item)| item)
        .collect()
}

/// Sort key of a suggestion: the closest match of the closest kind comes first.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct RankKey {
    kind: u8,
    distance: u8,
    length: usize,
    label: String,
}

/// 0 exact match, 1 prefix match, 2 characters in order, `None` no match.
fn match_score(label: &str, needle: &str) -> Option<u8> {
    if needle.is_empty() {
        return Some(0);
    }
    let haystack = label.to_ascii_lowercase();
    if haystack == needle {
        return Some(0);
    }
    if haystack.starts_with(needle) {
        return Some(1);
    }
    is_subsequence(needle, &haystack).then_some(2)
}

fn is_subsequence(needle: &str, haystack: &str) -> bool {
    let mut chars = haystack.chars();
    needle
        .chars()
        .all(|wanted| chars.any(|candidate| candidate == wanted))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::entities::column_info::ColumnInfo;
    use crate::core::entities::database_info::DatabaseInfo;
    use crate::core::entities::database_provider::DatabaseProvider;
    use crate::core::entities::relation_kind::RelationKind;
    use crate::core::entities::schema_info::SchemaInfo;
    use crate::core::entities::table_info::TableInfo;
    use crate::core::sql::analysis::sql_analyzer::SqlAnalyzer;
    use crate::core::sql::dialect::postgres_dialect::PostgreSqlDialect;

    fn column(name: &str) -> ColumnInfo {
        ColumnInfo {
            name: name.to_string(),
            data_type: "integer".to_string(),
            is_primary_key: name == "id",
            is_nullable: name != "id",
        }
    }

    fn catalog() -> SchemaCatalog {
        SchemaCatalog::from_database(DatabaseInfo {
            name: "warehouse".to_string(),
            provider: DatabaseProvider::PostgreSql,
            expanded: false,
            schemas: vec![
                SchemaInfo {
                    name: "public".to_string(),
                    expanded: false,
                    tables: vec![
                        TableInfo {
                            name: "users".to_string(),
                            kind: RelationKind::Table,
                            columns: vec![column("id"), column("email"), column("created_at")],
                            expanded: false,
                        },
                        TableInfo {
                            name: "active_users".to_string(),
                            kind: RelationKind::View,
                            columns: vec![column("id")],
                            expanded: false,
                        },
                    ],
                },
                SchemaInfo {
                    name: "auth".to_string(),
                    expanded: false,
                    tables: vec![TableInfo {
                        name: "sessions".to_string(),
                        kind: RelationKind::Table,
                        columns: vec![column("token")],
                        expanded: false,
                    }],
                },
            ],
        })
    }

    /// Completes at the end of `sql`, marked with `|`.
    fn complete_at(sql: &str) -> CompletionResult {
        complete_with(sql, &catalog())
    }

    fn complete_with(sql: &str, catalog: &SchemaCatalog) -> CompletionResult {
        let marker = sql.find('|').expect("the caret marker");
        let sql = sql.replace('|', "");
        let document = TextDocument::new(sql);
        let cursor = document.position_at(marker);
        let dialect = PostgreSqlDialect::new();
        let analysis = SqlAnalyzer::analyze(document.text(), &dialect);

        CompletionEngine::complete(&document, cursor, &analysis, catalog, &dialect)
    }

    fn labels(result: &CompletionResult) -> Vec<&str> {
        result
            .items
            .iter()
            .map(|item| item.label.as_str())
            .collect()
    }

    fn contains(result: &CompletionResult, label: &str) -> bool {
        labels(result).contains(&label)
    }

    #[test]
    fn test_column_completion_after_a_where_clause() {
        let result = complete_at("SELECT * FROM users WHERE ema|");

        assert!(contains(&result, "email"), "{:?}", labels(&result));
        assert!(!contains(&result, "id"), "the prefix filters the list");
    }

    #[test]
    fn test_relation_completion_after_from() {
        let result = complete_at("SELECT * FROM us|");

        assert!(contains(&result, "users"));
        assert!(contains(&result, "active_users"));
        assert!(!contains(&result, "id"), "columns do not belong in FROM");
    }

    #[test]
    fn test_columns_of_a_qualified_relation() {
        let result = complete_at("SELECT users.| FROM users");

        assert!(contains(&result, "id"));
        assert!(contains(&result, "email"));
        assert!(
            !contains(&result, "users"),
            "the relation itself is not suggested"
        );
    }

    #[test]
    fn test_columns_of_an_alias() {
        let result = complete_at("SELECT u.| FROM users u");

        assert!(contains(&result, "id"));
        assert_eq!(result.items[0].kind, CompletionKind::Column);
    }

    #[test]
    fn test_partial_schema_name_matches_a_schema() {
        let result = complete_at("SELECT * FROM au|");

        assert!(contains(&result, "auth"), "the schema is offered in FROM");
        assert!(
            contains(&result, "active_users"),
            "characters in order still match"
        );
        assert!(!contains(&result, "sessions"), "nothing in common with au");
    }

    #[test]
    fn test_schema_qualifier_dot_offers_its_relations() {
        let result = complete_at("SELECT * FROM public.|");

        assert!(contains(&result, "users"));
        assert!(contains(&result, "active_users"));
        assert!(!contains(&result, "sessions"), "sessions live in auth");
    }

    #[test]
    fn test_views_are_offered_with_their_own_kind() {
        let result = complete_at("SELECT * FROM active|");

        let view = result
            .items
            .iter()
            .find(|item| item.label == "active_users")
            .expect("the view");

        assert_eq!(view.kind, CompletionKind::View);
        assert_eq!(view.detail, "public");
    }

    #[test]
    fn test_statements_are_offered_at_the_start() {
        let result = complete_at("SEL|");

        assert!(contains(&result, "SELECT"));
        assert!(!contains(&result, "FROM"), "clauses are not statements");
    }

    #[test]
    fn test_keywords_are_offered_for_the_clause() {
        let result = complete_at("SELECT * FROM users ORDER BY |");

        assert!(contains(&result, "ASC"));
        assert!(contains(&result, "DESC"));
    }

    #[test]
    fn test_functions_are_offered_in_expressions() {
        let result = complete_at("SELECT cou| FROM users");

        assert!(contains(&result, "COUNT"));
    }

    #[test]
    fn test_columns_from_a_join() {
        let result = complete_at("SELECT | FROM users u JOIN sessions s ON s.tok = u.id");

        assert!(contains(&result, "id"));
        assert!(contains(&result, "token"));
    }

    #[test]
    fn test_a_subquery_does_not_leak_its_clause_or_its_tables() {
        let result =
            complete_at("SELECT * FROM users u WHERE u.id IN (SELECT id FROM sessions) AND |");

        assert!(contains(&result, "id"), "columns of users are in scope");
        assert!(
            !contains(&result, "token"),
            "a table of the subquery is not visible outside of it"
        );
        assert!(contains(&result, "AND"), "the outer WHERE clause is active");
    }

    #[test]
    fn test_update_set_offers_columns() {
        let result = complete_at("UPDATE users SET ema|");

        assert!(contains(&result, "email"));
    }

    #[test]
    fn test_insert_into_offers_relations() {
        let result = complete_at("INSERT INTO us|");

        assert!(contains(&result, "users"));
    }

    #[test]
    fn test_quoted_prefix_is_replaced_inside_the_quotes() {
        let result = complete_at("SELECT * FROM users WHERE \"ema|");

        assert!(contains(&result, "email"));
        assert_eq!(
            result.replace_range.start.column, 27,
            "the quotes stay, the name inside is replaced"
        );
    }

    #[test]
    fn test_replace_range_covers_the_prefix_only() {
        let result = complete_at("SELECT * FROM public.us|");

        assert_eq!(result.replace_range.start.column, 21);
        assert_eq!(result.replace_range.end.column, 23);
        assert!(
            !contains(&result, "public"),
            "the qualifier is not replaced"
        );
    }

    #[test]
    fn test_exact_matches_come_first() {
        let result = complete_at("SELECT * FROM users u WHERE u.emai|");

        assert_eq!(result.items[0].label, "email");
    }

    #[test]
    fn test_subsequence_matching_finds_later_names() {
        let result = complete_at("SELECT * FROM users WHERE crat|");

        assert!(contains(&result, "created_at"));
    }

    #[test]
    fn test_no_match_yields_no_items() {
        let result = complete_at("SELECT * FROM users WHERE zzz|");

        assert!(result.is_empty());
    }

    #[test]
    fn test_empty_catalog_still_offers_keywords() {
        let result = complete_with("SELECT * FROM users WHERE |", &SchemaCatalog::empty());

        assert!(!result.is_empty());
        assert!(contains(&result, "AND"));
    }

    #[test]
    fn test_duplicate_columns_of_a_join_are_merged() {
        let result = complete_at("SELECT i| FROM users u JOIN sessions s ON s.id = u.id");

        assert_eq!(
            labels(&result)
                .iter()
                .filter(|label| **label == "id")
                .count(),
            1
        );
    }

    #[test]
    fn test_column_detail_carries_the_type() {
        let result = complete_at("SELECT * FROM users WHERE ema|");

        let item = result
            .items
            .iter()
            .find(|item| item.label == "email")
            .expect("the email column");

        assert_eq!(item.detail, "integer");
    }

    #[test]
    fn test_primary_key_detail_is_marked() {
        let result = complete_at("SELECT * FROM users WHERE i|");

        let item = result.items[0].clone();

        assert_eq!(item.label, "id");
        assert_eq!(item.detail, "integer, PK");
    }

    #[test]
    fn test_caret_in_the_middle_of_a_line() {
        let sql = "SELECT * FROM users WHERE ema";
        let document = TextDocument::new(sql);
        let dialect = PostgreSqlDialect::new();
        let analysis = SqlAnalyzer::analyze(document.text(), &dialect);

        let result = CompletionEngine::complete(
            &document,
            TextPosition::new(0, 29),
            &analysis,
            &catalog(),
            &dialect,
        );

        assert_eq!(result.items[0].label, "email");
        assert_eq!(result.replace_range.start.column, 26);
        assert_eq!(
            result.replace_range.end.column, 29,
            "the text after the caret is kept"
        );
    }
}
