//! Alignment types for the colonnade crate.

/// Alignments left-to-right one can apply to columns of text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum Alignment {
    /// Left justification -- the default alignment
    #[default]
    Left,
    /// Right justification
    Right,
    /// Centering
    Center,
    /// Justified to both margins; the last line in a column is left-justified
    Justify,
}


/// Vertical alignments of text within a column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VerticalAlignment {
    /// the default vertical alignment
    #[default]
    Top,
    Middle,
    Bottom,
}
