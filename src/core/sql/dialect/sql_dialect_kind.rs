use crate::core::entities::database_provider::DatabaseProvider;

/// Dialects the editor can analyse SQL with.
///
/// Every supported provider maps to exactly one dialect, so a change of
/// connection is enough to change keywords, quoting and validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SqlDialectKind {
    #[default]
    PostgreSql,
    MySql,
    Sqlite,
    Firebird,
}

impl SqlDialectKind {
    /// Dialect used for a database provider.
    pub fn from_provider(provider: DatabaseProvider) -> Self {
        match provider {
            DatabaseProvider::PostgreSql => SqlDialectKind::PostgreSql,
            DatabaseProvider::MySql => SqlDialectKind::MySql,
            DatabaseProvider::Sqlite => SqlDialectKind::Sqlite,
            DatabaseProvider::Firebird => SqlDialectKind::Firebird,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            SqlDialectKind::PostgreSql => "PostgreSQL",
            SqlDialectKind::MySql => "MySQL",
            SqlDialectKind::Sqlite => "SQLite",
            SqlDialectKind::Firebird => "Firebird",
        }
    }

    pub const ALL: [SqlDialectKind; 4] = [
        SqlDialectKind::PostgreSql,
        SqlDialectKind::MySql,
        SqlDialectKind::Sqlite,
        SqlDialectKind::Firebird,
    ];
}

impl From<DatabaseProvider> for SqlDialectKind {
    fn from(provider: DatabaseProvider) -> Self {
        Self::from_provider(provider)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_every_provider_maps_to_a_dialect() {
        for provider in DatabaseProvider::ALL {
            let kind = SqlDialectKind::from_provider(provider);

            assert_eq!(kind.display_name(), provider.display_name());
        }
    }

    #[test]
    fn test_dialect_kinds_have_distinct_names() {
        let names: Vec<&str> = SqlDialectKind::ALL
            .iter()
            .map(|kind| kind.display_name())
            .collect();

        assert_eq!(names.len(), 4);
        for (index, name) in names.iter().enumerate() {
            assert!(!names[index + 1..].contains(name));
        }
    }

    #[test]
    fn test_provider_conversion_uses_the_same_mapping() {
        let provider = DatabaseProvider::Firebird;

        assert_eq!(SqlDialectKind::from(provider), SqlDialectKind::Firebird);
    }
}
