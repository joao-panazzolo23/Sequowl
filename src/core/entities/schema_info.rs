use crate::core::entities::table_info::TableInfo;

/// A schema (or namespace/catalog) holding tables and views.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaInfo {
    pub name: String,
    pub tables: Vec<TableInfo>,
    pub expanded: bool,
}

impl SchemaInfo {
    /// Case-insensitive lookup, because identifier folding rules
    /// (lower case for PostgreSQL, upper case for Firebird) depend on the
    /// live server and are not known while the catalog is still static.
    pub fn find_relation(&self, name: &str) -> Option<&TableInfo> {
        self.tables
            .iter()
            .find(|table| table.name.eq_ignore_ascii_case(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::entities::relation_kind::RelationKind;

    fn schema() -> SchemaInfo {
        SchemaInfo {
            name: "public".to_string(),
            expanded: false,
            tables: vec![TableInfo {
                name: "users".to_string(),
                kind: RelationKind::Table,
                columns: Vec::new(),
                expanded: false,
            }],
        }
    }

    #[test]
    fn test_find_relation_is_case_insensitive() {
        assert!(schema().find_relation("USERS").is_some());
    }

    #[test]
    fn test_find_relation_unknown_name() {
        assert!(schema().find_relation("orders").is_none());
    }
}
