/// A source line, as the renderer needs it: its index and how many
/// characters it holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLine {
    pub row: u32,
    pub length: u32,
}

impl SourceLine {
    pub fn new(row: u32, length: u32) -> Self {
        Self { row, length }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_keeps_row_and_length() {
        let line = SourceLine::new(4, 12);

        assert_eq!(line.row, 4);
        assert_eq!(line.length, 12);
    }
}
