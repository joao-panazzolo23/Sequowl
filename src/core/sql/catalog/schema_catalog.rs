use crate::core::entities::database_info::DatabaseInfo;
use crate::core::entities::database_provider::DatabaseProvider;
use crate::core::entities::schema_info::SchemaInfo;
use crate::core::entities::table_info::TableInfo;

/// Metadata of the database the editor is connected to.
///
/// The catalog is the bridge between the connection layer and the SQL engine:
/// it is a plain snapshot of [`DatabaseInfo`], so it can be built from live
/// introspection later without touching the language services.
///
/// All lookups are case-insensitive, because identifier folding rules depend
/// on the live server and are not known while the catalog is static.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SchemaCatalog {
    database: Option<DatabaseInfo>,
}

impl SchemaCatalog {
    pub fn empty() -> Self {
        Self { database: None }
    }

    pub fn from_database(database: DatabaseInfo) -> Self {
        Self {
            database: Some(database),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.database.is_none()
    }

    pub fn database(&self) -> Option<&DatabaseInfo> {
        self.database.as_ref()
    }

    pub fn database_name(&self) -> Option<&str> {
        self.database
            .as_ref()
            .map(|database| database.name.as_str())
    }

    pub fn provider(&self) -> Option<DatabaseProvider> {
        self.database.as_ref().map(|database| database.provider)
    }

    pub fn schemas(&self) -> impl Iterator<Item = &SchemaInfo> {
        self.database
            .iter()
            .flat_map(|database| database.schemas.iter())
    }

    /// Every relation of the database, paired with the schema holding it.
    pub fn relations(&self) -> impl Iterator<Item = (&SchemaInfo, &TableInfo)> {
        self.schemas()
            .flat_map(|schema| schema.tables.iter().map(move |table| (schema, table)))
    }

    pub fn schema(&self, name: &str) -> Option<&SchemaInfo> {
        self.schemas()
            .find(|schema| schema.name.eq_ignore_ascii_case(name))
    }

    /// Relations of one schema.
    pub fn relations_in(&self, schema: &str) -> impl Iterator<Item = &TableInfo> {
        self.schema(schema)
            .into_iter()
            .flat_map(|schema| schema.tables.iter())
    }

    /// Finds a relation, optionally restricted to a schema.
    pub fn relation(&self, schema: Option<&str>, name: &str) -> Option<&TableInfo> {
        match schema {
            Some(schema) => self
                .relations_in(schema)
                .find(|table| table.name.eq_ignore_ascii_case(name)),
            None => self
                .relations()
                .find(|(_, table)| table.name.eq_ignore_ascii_case(name))
                .map(|(_, table)| table),
        }
    }

    /// Columns of a relation, or `None` when it is unknown.
    pub fn columns(
        &self,
        schema: Option<&str>,
        relation: &str,
    ) -> Option<&[crate::core::entities::column_info::ColumnInfo]> {
        self.relation(schema, relation)
            .map(|table| table.columns.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::entities::column_info::ColumnInfo;
    use crate::core::entities::relation_kind::RelationKind;

    fn column(name: &str) -> ColumnInfo {
        ColumnInfo {
            name: name.to_string(),
            data_type: "integer".to_string(),
            is_primary_key: false,
            is_nullable: true,
        }
    }

    fn table(name: &str, kind: RelationKind, columns: Vec<ColumnInfo>) -> TableInfo {
        TableInfo {
            name: name.to_string(),
            kind,
            columns,
            expanded: false,
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
                        table(
                            "users",
                            RelationKind::Table,
                            vec![column("id"), column("email")],
                        ),
                        table("active_users", RelationKind::View, vec![column("id")]),
                    ],
                },
                SchemaInfo {
                    name: "auth".to_string(),
                    expanded: false,
                    tables: vec![table(
                        "sessions",
                        RelationKind::Table,
                        vec![column("token")],
                    )],
                },
            ],
        })
    }

    #[test]
    fn test_empty_catalog_has_nothing_to_offer() {
        let catalog = SchemaCatalog::empty();

        assert!(catalog.is_empty());
        assert_eq!(catalog.schemas().count(), 0);
        assert_eq!(catalog.relations().count(), 0);
        assert!(catalog.relation(None, "users").is_none());
    }

    #[test]
    fn test_catalog_reports_the_connection() {
        let catalog = catalog();

        assert!(!catalog.is_empty());
        assert_eq!(catalog.database_name(), Some("warehouse"));
        assert_eq!(catalog.provider(), Some(DatabaseProvider::PostgreSql));
    }

    #[test]
    fn test_schemas_and_relations_are_listed_in_order() {
        let catalog = catalog();

        let schemas: Vec<&str> = catalog
            .schemas()
            .map(|schema| schema.name.as_str())
            .collect();
        let relations: Vec<&str> = catalog
            .relations()
            .map(|(_, table)| table.name.as_str())
            .collect();

        assert_eq!(schemas, ["public", "auth"]);
        assert_eq!(relations, ["users", "active_users", "sessions"]);
    }

    #[test]
    fn test_relation_lookup_is_case_insensitive() {
        assert!(catalog().relation(None, "USERS").is_some());
        assert!(catalog().relation(Some("AUTH"), "Sessions").is_some());
    }

    #[test]
    fn test_relation_lookup_is_restricted_by_schema() {
        let catalog = catalog();

        assert!(catalog.relation(Some("public"), "users").is_some());
        assert!(
            catalog.relation(Some("auth"), "users").is_none(),
            "users does not live in auth"
        );
    }

    #[test]
    fn test_columns_are_resolved_through_the_relation() {
        let catalog = catalog();
        let columns = catalog
            .columns(Some("public"), "users")
            .expect("columns of users");

        assert_eq!(columns.len(), 2);
        assert_eq!(columns[0].name, "id");
    }

    #[test]
    fn test_view_and_table_are_both_listed_as_relations() {
        let catalog = catalog();

        assert_eq!(
            catalog.relation(Some("public"), "users").map(|t| t.kind),
            Some(RelationKind::Table)
        );
        assert_eq!(
            catalog
                .relation(Some("public"), "active_users")
                .map(|t| t.kind),
            Some(RelationKind::View)
        );
    }

    #[test]
    fn test_relations_of_one_schema() {
        let catalog = catalog();
        let names: Vec<&str> = catalog
            .relations_in("public")
            .map(|table| table.name.as_str())
            .collect();

        assert_eq!(names, ["users", "active_users"]);
    }
}
