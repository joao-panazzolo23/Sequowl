use std::collections::HashSet;

/// Case-insensitive set of words, kept in a stable order so results such as
/// completion lists are deterministic.
///
/// Word lists are expected to be written in upper case; lookups are still
/// case-insensitive.
#[derive(Debug, Clone, Default)]
pub struct WordIndex {
    words: Vec<&'static str>,
    lookup: HashSet<&'static str>,
}

impl WordIndex {
    /// Builds the index from any number of word lists, merging and sorting
    /// them so callers can compose a shared list with dialect additions.
    pub fn from_sets(sets: &[&'static [&'static str]]) -> Self {
        let mut words: Vec<&'static str> =
            sets.iter().flat_map(|set| set.iter().copied()).collect();
        words.sort_unstable();
        words.dedup();

        Self {
            lookup: words.iter().copied().collect(),
            words,
        }
    }

    pub fn contains(&self, word: &str) -> bool {
        self.lookup.contains(word.to_ascii_uppercase().as_str())
    }

    /// All indexed words in ascending order.
    pub fn words(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.words.iter().copied()
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST: &[&str] = &["SELECT", "FROM"];
    const SECOND: &[&str] = &["FROM", "WHERE"];

    fn index() -> WordIndex {
        WordIndex::from_sets(&[FIRST, SECOND])
    }

    #[test]
    fn test_contains_ignores_case() {
        let index = index();

        assert!(index.contains("select"));
        assert!(index.contains("Where"));
    }

    #[test]
    fn test_contains_unknown_word() {
        assert!(!index().contains("teleport"));
    }

    #[test]
    fn test_words_are_merged_deduplicated_and_sorted() {
        assert_eq!(
            index().words().collect::<Vec<_>>(),
            ["FROM", "SELECT", "WHERE"]
        );
        assert_eq!(index().len(), 3);
    }

    #[test]
    fn test_empty_index() {
        let index = WordIndex::from_sets(&[]);

        assert!(index.is_empty());
        assert!(!index.contains("SELECT"));
    }
}
