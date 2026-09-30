use crate::core::entities::database_provider::DatabaseProvider;
use crate::core::entities::schema_info::SchemaInfo;

/// A database connection, its dialect and the metadata it exposes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseInfo {
    pub name: String,
    pub provider: DatabaseProvider,
    pub schemas: Vec<SchemaInfo>,
    pub expanded: bool,
}

impl DatabaseInfo {
    /// Case-insensitive schema lookup.
    pub fn find_schema(&self, name: &str) -> Option<&SchemaInfo> {
        self.schemas
            .iter()
            .find(|schema| schema.name.eq_ignore_ascii_case(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::entities::relation_kind::RelationKind;
    use crate::core::entities::table_info::TableInfo;

    fn database() -> DatabaseInfo {
        DatabaseInfo {
            name: "warehouse".to_string(),
            provider: DatabaseProvider::PostgreSql,
            expanded: false,
            schemas: vec![SchemaInfo {
                name: "public".to_string(),
                expanded: false,
                tables: vec![TableInfo {
                    name: "events".to_string(),
                    kind: RelationKind::Table,
                    columns: Vec::new(),
                    expanded: false,
                }],
            }],
        }
    }

    #[test]
    fn test_find_schema_is_case_insensitive() {
        assert!(database().find_schema("Public").is_some());
    }

    #[test]
    fn test_find_schema_unknown_name() {
        assert!(database().find_schema("analytics").is_none());
    }
}
