/// Kind of relation stored inside a schema.
///
/// Completion presents tables and views differently, so the kind has to
/// travel with the metadata instead of being guessed from the name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RelationKind {
    #[default]
    Table,
    View,
}

impl RelationKind {
    pub fn display_name(self) -> &'static str {
        match self {
            RelationKind::Table => "Table",
            RelationKind::View => "View",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_names() {
        assert_eq!(RelationKind::Table.display_name(), "Table");
        assert_eq!(RelationKind::View.display_name(), "View");
    }

    #[test]
    fn test_default_kind_is_table() {
        assert_eq!(RelationKind::default(), RelationKind::Table);
    }
}
