/// Database engine a connection points to.
///
/// The provider drives which SQL dialect the editor applies for
/// highlighting, diagnostics and completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DatabaseProvider {
    PostgreSql,
    #[default]
    MySql,
    Sqlite,
    Firebird,
}

impl DatabaseProvider {
    /// Human readable provider name, as shown in the editor status bar.
    pub fn display_name(self) -> &'static str {
        match self {
            DatabaseProvider::PostgreSql => "PostgreSQL",
            DatabaseProvider::MySql => "MySQL",
            DatabaseProvider::Sqlite => "SQLite",
            DatabaseProvider::Firebird => "Firebird",
        }
    }

    /// All supported providers, used to build dialect selectors.
    pub const ALL: [DatabaseProvider; 4] = [
        DatabaseProvider::PostgreSql,
        DatabaseProvider::MySql,
        DatabaseProvider::Sqlite,
        DatabaseProvider::Firebird,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_names_are_distinct() {
        let names: Vec<&str> = DatabaseProvider::ALL
            .iter()
            .map(|provider| provider.display_name())
            .collect();

        assert_eq!(names, ["PostgreSQL", "MySQL", "SQLite", "Firebird"]);
    }

    #[test]
    fn test_default_provider_is_mysql() {
        assert_eq!(DatabaseProvider::default(), DatabaseProvider::MySql);
    }
}
