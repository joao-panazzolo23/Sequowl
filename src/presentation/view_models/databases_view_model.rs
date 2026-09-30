use crate::TreeNodeData;
use crate::core::entities::column_info::ColumnInfo;
use crate::core::entities::database_info::DatabaseInfo;
use crate::core::entities::database_provider::DatabaseProvider;
use crate::core::entities::relation_kind::RelationKind;
use crate::core::entities::schema_info::SchemaInfo;
use crate::core::entities::table_info::TableInfo;
use crate::core::sql::catalog::schema_catalog::SchemaCatalog;
use slint::SharedString;

/// Connections shown in the sidebar, with the expansion and selection state
/// the tree renders.
///
/// The metadata itself lives in `core::entities`, so the editor and the tree
/// read the same structures.
pub struct DatabasesViewModel {
    databases: Vec<DatabaseInfo>,
    selected_id: Option<String>,
}

impl Default for DatabasesViewModel {
    fn default() -> Self {
        Self::new()
    }
}

impl DatabasesViewModel {
    pub fn new() -> Self {
        Self {
            databases: demo_databases(),
            selected_id: None,
        }
    }

    pub fn databases(&self) -> &[DatabaseInfo] {
        &self.databases
    }

    /// Catalog of a connection, which the editor uses for completion.
    pub fn catalog(&self, name: &str) -> Option<SchemaCatalog> {
        self.databases
            .iter()
            .find(|database| database.name == name)
            .map(|database| SchemaCatalog::from_database(database.clone()))
            .filter(|catalog| !catalog.is_empty())
    }

    /// Catalog of the connection that is selected, or the first one.
    pub fn active_catalog(&self) -> Option<SchemaCatalog> {
        let name = self
            .selected_id
            .as_deref()
            .and_then(|id| {
                id.strip_prefix("db:")
                    .or_else(|| id.split_once(':').map(|(_, name)| name))
            })
            .or_else(|| {
                self.databases
                    .first()
                    .map(|database| database.name.as_str())
            })?;

        self.catalog(name)
    }

    pub fn select_node(&mut self, id: &str) {
        self.selected_id = Some(id.to_string());
    }

    pub fn toggle_expand(&mut self, id: &str) {
        if let Some(db_name) = id.strip_prefix("db:") {
            for db in &mut self.databases {
                if db.name == db_name {
                    db.expanded = !db.expanded;
                    return;
                }
            }
            return;
        }

        if let Some((db_name, schema_name)) = split_path(id, "schema:") {
            for db in &mut self.databases {
                if db.name != db_name {
                    continue;
                }
                for schema in &mut db.schemas {
                    if schema.name == schema_name {
                        schema.expanded = !schema.expanded;
                        return;
                    }
                }
            }
            return;
        }

        if let Some((db_name, relation_path)) = split_path(id, "table:") {
            let Some((schema_name, relation_name)) = relation_path.split_once('.') else {
                return;
            };

            for db in &mut self.databases {
                if db.name != db_name {
                    continue;
                }
                for schema in &mut db.schemas {
                    if schema.name != schema_name {
                        continue;
                    }
                    for table in &mut schema.tables {
                        if table.name == relation_name {
                            table.expanded = !table.expanded;
                            return;
                        }
                    }
                }
            }
        }
    }

    pub fn to_tree_nodes(&self) -> Vec<TreeNodeData> {
        let mut nodes = Vec::new();

        for db in &self.databases {
            let db_id = format!("db:{}", db.name);
            nodes.push(TreeNodeData {
                id: SharedString::from(&db_id),
                name: SharedString::from(&db.name),
                extra: SharedString::from(db.provider.display_name()),
                kind: SharedString::from("database"),
                level: 0,
                expanded: db.expanded,
                is_leaf: false,
                is_selected: self.is_selected(&db_id),
            });

            if !db.expanded {
                continue;
            }

            for schema in &db.schemas {
                let schema_id = format!("schema:{}.{}", db.name, schema.name);
                nodes.push(TreeNodeData {
                    id: SharedString::from(&schema_id),
                    name: SharedString::from(&schema.name),
                    extra: SharedString::default(),
                    kind: SharedString::from("schema"),
                    level: 1,
                    expanded: schema.expanded,
                    is_leaf: false,
                    is_selected: self.is_selected(&schema_id),
                });

                if !schema.expanded {
                    continue;
                }

                for table in &schema.tables {
                    let table_id = format!("table:{}.{}.{}", db.name, schema.name, table.name);
                    nodes.push(TreeNodeData {
                        id: SharedString::from(&table_id),
                        name: SharedString::from(&table.name),
                        extra: SharedString::from(table.kind.display_name()),
                        kind: SharedString::from("table"),
                        level: 2,
                        expanded: table.expanded,
                        is_leaf: false,
                        is_selected: self.is_selected(&table_id),
                    });

                    if !table.expanded {
                        continue;
                    }

                    for column in &table.columns {
                        let column_id = format!(
                            "col:{}.{}.{}.{}",
                            db.name, schema.name, table.name, column.name
                        );
                        nodes.push(TreeNodeData {
                            id: SharedString::from(&column_id),
                            name: SharedString::from(&column.name),
                            extra: SharedString::from(&column.describe()),
                            kind: SharedString::from("column"),
                            level: 3,
                            expanded: false,
                            is_leaf: true,
                            is_selected: self.is_selected(&column_id),
                        });
                    }
                }
            }
        }

        nodes
    }

    fn is_selected(&self, id: &str) -> bool {
        self.selected_id.as_deref() == Some(id)
    }
}

/// Splits `prefix:database.rest` into the database name and the rest.
fn split_path<'a>(id: &'a str, prefix: &str) -> Option<(&'a str, &'a str)> {
    let path = id.strip_prefix(prefix)?;
    path.split_once('.')
}

fn column(name: &str, data_type: &str, primary_key: bool) -> ColumnInfo {
    ColumnInfo {
        name: name.to_string(),
        data_type: data_type.to_string(),
        is_primary_key: primary_key,
        is_nullable: !primary_key,
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

fn demo_databases() -> Vec<DatabaseInfo> {
    vec![
        DatabaseInfo {
            name: "postgres".to_string(),
            provider: DatabaseProvider::PostgreSql,
            expanded: true,
            schemas: vec![
                SchemaInfo {
                    name: "public".to_string(),
                    expanded: true,
                    tables: vec![
                        table(
                            "users",
                            RelationKind::Table,
                            vec![
                                column("id", "uuid", true),
                                column("name", "varchar(255)", false),
                                column("email", "varchar(255)", false),
                            ],
                        ),
                        table(
                            "orders",
                            RelationKind::Table,
                            vec![column("id", "uuid", true), column("user_id", "uuid", false)],
                        ),
                        table(
                            "active_users",
                            RelationKind::View,
                            vec![column("id", "uuid", true), column("name", "text", false)],
                        ),
                    ],
                },
                SchemaInfo {
                    name: "auth".to_string(),
                    expanded: false,
                    tables: vec![table(
                        "sessions",
                        RelationKind::Table,
                        vec![column("id", "uuid", true), column("user_id", "uuid", false)],
                    )],
                },
            ],
        },
        DatabaseInfo {
            name: "analytics".to_string(),
            provider: DatabaseProvider::MySql,
            expanded: false,
            schemas: vec![SchemaInfo {
                name: "public".to_string(),
                expanded: false,
                tables: vec![table(
                    "events",
                    RelationKind::Table,
                    vec![
                        column("id", "bigint", true),
                        column("event_name", "text", false),
                    ],
                )],
            }],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_tree_nodes() {
        // Arrange
        let vm = DatabasesViewModel::new();

        // Act
        let nodes = vm.to_tree_nodes();

        // Assert: initial state has postgres and analytics, with postgres expanded
        assert!(!nodes.is_empty());
        assert_eq!(nodes[0].name.as_str(), "postgres");
        assert_eq!(nodes[0].kind.as_str(), "database");
        assert_eq!(nodes[0].level, 0);
    }

    #[test]
    fn test_toggle_database_expand() {
        // Arrange
        let mut vm = DatabasesViewModel::new();
        let initial_count = vm.to_tree_nodes().len();

        // Act: collapse postgres database
        vm.toggle_expand("db:postgres");
        let collapsed_count = vm.to_tree_nodes().len();

        // Assert: collapsing should reduce visible nodes
        assert!(collapsed_count < initial_count);

        // Act: re-expand postgres
        vm.toggle_expand("db:postgres");
        let reexpanded_count = vm.to_tree_nodes().len();

        // Assert
        assert_eq!(reexpanded_count, initial_count);
    }

    #[test]
    fn test_toggle_table_expand() {
        // Arrange
        let mut vm = DatabasesViewModel::new();
        let initial_count = vm.to_tree_nodes().len();

        // Act: expand users table
        vm.toggle_expand("table:postgres.public.users");
        let expanded_count = vm.to_tree_nodes().len();

        // Assert: expanding table exposes columns
        assert!(expanded_count > initial_count);

        // Act: collapse users table
        vm.toggle_expand("table:postgres.public.users");
        let collapsed_count = vm.to_tree_nodes().len();

        // Assert
        assert_eq!(collapsed_count, initial_count);
    }

    #[test]
    fn test_select_node() {
        // Arrange
        let mut vm = DatabasesViewModel::new();

        // Act
        vm.select_node("db:postgres");
        let nodes = vm.to_tree_nodes();

        // Assert
        assert!(nodes[0].is_selected);
    }

    #[test]
    fn test_a_database_shows_its_provider() {
        let vm = DatabasesViewModel::new();

        let nodes = vm.to_tree_nodes();

        assert_eq!(nodes[0].extra.as_str(), "PostgreSQL");
    }

    #[test]
    fn test_a_relation_shows_whether_it_is_a_table_or_a_view() {
        let mut vm = DatabasesViewModel::new();

        vm.select_node("db:postgres");
        vm.toggle_expand("table:postgres.public.active_users");
        let nodes = vm.to_tree_nodes();
        let view = nodes
            .iter()
            .find(|node| node.name.as_str() == "active_users")
            .expect("the view is listed");

        assert_eq!(view.extra.as_str(), "View");
    }

    #[test]
    fn test_the_catalog_of_a_connection_carries_its_metadata() {
        let vm = DatabasesViewModel::new();

        let catalog = vm.catalog("postgres").expect("the connection has metadata");

        assert_eq!(catalog.database_name(), Some("postgres"));
        assert_eq!(catalog.provider(), Some(DatabaseProvider::PostgreSql));
        assert!(
            catalog
                .relation(Some("public"), "users")
                .is_some_and(|users| users.columns.len() == 3)
        );
    }

    #[test]
    fn test_the_active_catalog_follows_the_selection() {
        let mut vm = DatabasesViewModel::new();

        vm.select_node("db:analytics");
        let catalog = vm.active_catalog().expect("the selection has metadata");

        assert_eq!(catalog.database_name(), Some("analytics"));
        assert_eq!(catalog.provider(), Some(DatabaseProvider::MySql));
    }

    #[test]
    fn test_an_unknown_connection_has_no_catalog() {
        let vm = DatabasesViewModel::new();

        assert!(vm.catalog("nope").is_none());
    }
}
