//! Error types for the colonnade crate.

use std::fmt;

/// All the things that can go wrong when laying out tabular data.
#[derive(Debug)]
pub enum ColonnadeError {
    /// The data to display is inconsistent with the spec.
    /// The tuple values are the index of the data row, its length, and the expected length.
    InconsistentColumns(usize, usize, usize), // row, row length, spec length
    /// The column index provided is outside the columns available.
    OutOfBounds,
    /// The column count parameter given to the constructor was 0.
    InsufficientColumns,
    /// The minimum space required by the columns is greater than the viewport.
    InsufficientSpace,
    /// The minimum and maximum width of a column conflict. The stored parameter is the column index.
    MinGreaterThanMax(usize), // column
}

impl fmt::Display for ColonnadeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColonnadeError::InconsistentColumns(row, row_len, expected_len) => {
                write!(
                    f,
                    "InconsistentColumns: row {} has {} columns, expected {}",
                    row, row_len, expected_len
                )
            }
            ColonnadeError::OutOfBounds => {
                write!(f, "OutOfBounds: column index is outside the available columns")
            }
            ColonnadeError::InsufficientColumns => {
                write!(f, "InsufficientColumns: column count must be greater than 0")
            }
            ColonnadeError::InsufficientSpace => {
                write!(f, "InsufficientSpace: columns require more space than the viewport provides")
            }
            ColonnadeError::MinGreaterThanMax(col) => {
                write!(f, "MinGreaterThanMax: column {} has min_width > max_width", col)
            }
        }
    }
}

impl std::error::Error for ColonnadeError {}
