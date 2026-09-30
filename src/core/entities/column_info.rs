/// Column metadata for a table or view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_primary_key: bool,
    pub is_nullable: bool,
}

impl ColumnInfo {
    /// Short description rendered next to the column in completion.
    pub fn describe(&self) -> String {
        if self.is_primary_key {
            format!("{}, PK", self.data_type)
        } else {
            self.data_type.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn column(name: &str, is_primary_key: bool) -> ColumnInfo {
        ColumnInfo {
            name: name.to_string(),
            data_type: "integer".to_string(),
            is_primary_key,
            is_nullable: !is_primary_key,
        }
    }

    #[test]
    fn test_describe_marks_primary_key() {
        assert_eq!(column("id", true).describe(), "integer, PK");
    }

    #[test]
    fn test_describe_plain_column() {
        assert_eq!(column("label", false).describe(), "integer");
    }
}
