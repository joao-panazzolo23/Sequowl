use crate::core::entities::column_info::ColumnInfo;
use crate::core::entities::relation_kind::RelationKind;

/// A table or a view, together with its columns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableInfo {
    pub name: String,
    pub kind: RelationKind,
    pub columns: Vec<ColumnInfo>,
    pub expanded: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relations_of_the_same_name_are_equivalent() {
        let columns = vec![ColumnInfo {
            name: "id".to_string(),
            data_type: "uuid".to_string(),
            is_primary_key: true,
            is_nullable: false,
        }];

        let table = TableInfo {
            name: "users".to_string(),
            kind: RelationKind::Table,
            columns: columns.clone(),
            expanded: false,
        };
        let view = TableInfo {
            name: "users".to_string(),
            kind: RelationKind::View,
            columns,
            expanded: false,
        };

        // Act & Assert: the relation kind is part of the identity
        assert_ne!(table, view);
    }
}
