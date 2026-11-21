//! Column configuration for the colonnade crate.

use crate::alignment::{Alignment, VerticalAlignment};
use crate::error::ColonnadeError;

/// A struct holding formatting information for a particular column.
#[derive(Debug, Clone)]
pub struct Column {
    pub index: usize,
    pub alignment: Alignment,
    pub vertical_alignment: VerticalAlignment,
    pub left_margin: usize,
    /// the width of the column excluding any left margin
    pub width: usize,
    pub priority: usize,
    pub min_width: Option<usize>,
    pub max_width: Option<usize>,
    pub padding_left: usize,
    pub padding_right: usize,
    pub padding_top: usize,
    pub padding_bottom: usize,
    pub hyphenate: bool,
    pub adjusted: bool,
}

impl Column {
    /// Create a new column with default values.
    pub fn new(index: usize) -> Self {
        Column {
            index,
            alignment: Alignment::default(),
            vertical_alignment: VerticalAlignment::default(),
            left_margin: if index == 0 { 0 } else { 1 },
            width: 0,
            priority: usize::MAX,
            min_width: None,
            max_width: None,
            padding_left: 0,
            padding_right: 0,
            padding_top: 0,
            padding_bottom: 0,
            hyphenate: true,
            adjusted: false,
        }
    }

    /// Calculate the total horizontal padding (left + right).
    pub fn horizontal_padding(&self) -> usize {
        self.padding_left + self.padding_right
    }

    /// Calculate the total vertical padding (top + bottom).
    pub fn vertical_padding(&self) -> usize {
        self.padding_top + self.padding_bottom
    }

    /// Get the minimum width of the column.
    pub fn minimum_width(&self) -> usize {
        let padding = self.horizontal_padding();
        self.min_width.unwrap_or(padding).max(padding)
    }

    /// Get the effective width of the column, respecting min/max constraints.
    pub fn effective_width(&self) -> usize {
        let max = self.max_width.unwrap_or(self.width);
        let width = max.min(self.width);
        width.max(self.minimum_width())
    }

    /// Get the inner width (width minus right padding).
    pub fn inner_width(&self) -> usize {
        self.width.saturating_sub(self.padding_right)
    }

    /// Check if hyphenation should be applied.
    pub fn hyphenating(&self) -> bool {
        self.hyphenate && self.inner_width() > 1
    }

    /// Check if the column can be shrunk.
    pub fn is_shrinkable(&self) -> bool {
        self.minimum_width() < self.width
    }

    /// Shrink the column to the specified width.
    pub fn shrink(&mut self, width: usize) {
        let min = self.minimum_width();
        self.width = width.max(min);
        self.adjusted = false;
    }

    /// Attempt to shrink the column by the specified amount.
    /// Returns whether any shrinkage occurred.
    pub fn shrink_by(&mut self, decrease: usize) -> bool {
        if self.is_shrinkable() {
            let new_width = self.width.saturating_sub(decrease).max(1);
            let before = self.width;
            self.shrink(new_width);
            before != self.width
        } else {
            false
        }
    }

    /// Check if the column can be expanded.
    pub fn is_expandable(&self) -> bool {
        self.max_width.unwrap_or(usize::MAX) > self.width
    }

    /// Expand the column to the specified width.
    /// Returns whether any expansion occurred.
    pub fn expand(&mut self, width: usize) -> bool {
        if width <= self.width {
            return false;
        }
        let max = self.max_width.unwrap_or(width);
        let min = self.minimum_width();
        let new_width = if max < width {
            max
        } else if min > width {
            min
        } else {
            width
        };
        let changed = self.width != new_width;
        if changed {
            self.width = new_width;
            self.adjusted = false;
        }
        changed
    }

    /// Expand the column by the specified amount.
    pub fn expand_by(&mut self, increase: usize) -> bool {
        self.expand(self.width.saturating_add(increase))
    }

    /// Calculate the total outer width (left margin + effective width).
    pub fn outer_width(&self) -> usize {
        self.left_margin + self.effective_width()
    }

    /// Generate a blank line for this column.
    pub fn blank_line(&self) -> String {
        " ".repeat(self.width)
    }

    /// Generate the margin string for this column.
    pub fn margin(&self) -> String {
        " ".repeat(self.left_margin)
    }

    /// Assign a particular priority to the column.
    ///
    /// Priority determines the order in which columns give up space when the viewport lacks sufficient
    /// space to display all columns without wrapping. Lower priority columns give up space first.
    ///
    /// # Arguments
    ///
    /// * `priority` - The column's priority. Lower numbers confer higher priority; 0 is the highest priority.
    pub fn priority(&mut self, priority: usize) -> &mut Self {
        self.priority = priority;
        self.adjusted = false;
        self
    }

    /// Assign a maximum width to this column.
    ///
    /// # Arguments
    ///
    /// * `max_width` - The maximum width.
    ///
    /// # Errors
    ///
    /// * `ColonnadeError::MinGreaterThanMax` - If the maximum width conflicts with the minimum width.
    pub fn max_width(&mut self, max_width: usize) -> Result<&mut Self, ColonnadeError> {
        if self.min_width.unwrap_or(max_width) > max_width {
            return Err(ColonnadeError::MinGreaterThanMax(self.index));
        }
        self.max_width = Some(max_width);
        self.adjusted = false;
        Ok(self)
    }

    /// Assign a minimum width to this column.
    ///
    /// # Arguments
    ///
    /// * `min_width` - The minimum width.
    ///
    /// # Errors
    ///
    /// * `ColonnadeError::MinGreaterThanMax` - If the minimum width conflicts with the maximum width.
    pub fn min_width(&mut self, min_width: usize) -> Result<&mut Self, ColonnadeError> {
        if self.max_width.unwrap_or(min_width) < min_width {
            return Err(ColonnadeError::MinGreaterThanMax(self.index));
        }
        self.width = min_width;
        self.min_width = Some(min_width);
        self.adjusted = false;
        Ok(self)
    }

    /// Assign a fixed width to this column (same min and max).
    ///
    /// # Arguments
    ///
    /// * `width` - The fixed width.
    ///
    /// # Errors
    ///
    /// * `ColonnadeError::MinGreaterThanMax` - If the width constraints conflict.
    pub fn fixed_width(&mut self, width: usize) -> Result<&mut Self, ColonnadeError> {
        self.min_width = None;
        self.max_width = None;
        self.min_width(width)?;
        self.max_width(width)?;
        Ok(self)
    }

    /// Remove maximum or minimum column width constraints.
    pub fn clear_limits(&mut self) -> &mut Self {
        self.max_width = None;
        self.min_width = None;
        self.adjusted = false;
        self
    }

    /// Set the alignment for this column.
    pub fn alignment(&mut self, alignment: Alignment) -> &mut Self {
        self.alignment = alignment;
        self
    }

    /// Set the vertical alignment for this column.
    pub fn vertical_alignment(&mut self, vertical_alignment: VerticalAlignment) -> &mut Self {
        self.vertical_alignment = vertical_alignment;
        self
    }

    /// Set the left margin for this column.
    pub fn left_margin(&mut self, left_margin: usize) -> &mut Self {
        self.left_margin = left_margin;
        self.adjusted = false;
        self
    }

    /// Set padding on all sides.
    pub fn padding(&mut self, padding: usize) -> &mut Self {
        self.padding_left = padding;
        self.padding_right = padding;
        self.padding_top = padding;
        self.padding_bottom = padding;
        self.adjusted = false;
        self
    }

    /// Set horizontal padding (left and right).
    pub fn padding_horizontal(&mut self, padding: usize) -> &mut Self {
        self.padding_left = padding;
        self.padding_right = padding;
        self.adjusted = false;
        self
    }

    /// Set left padding.
    pub fn padding_left(&mut self, padding: usize) -> &mut Self {
        self.padding_left = padding;
        self.adjusted = false;
        self
    }

    /// Set right padding.
    pub fn padding_right(&mut self, padding: usize) -> &mut Self {
        self.padding_right = padding;
        self.adjusted = false;
        self
    }

    /// Set vertical padding (top and bottom).
    pub fn padding_vertical(&mut self, padding: usize) -> &mut Self {
        self.padding_top = padding;
        self.padding_bottom = padding;
        self
    }

    /// Set top padding.
    pub fn padding_top(&mut self, padding: usize) -> &mut Self {
        self.padding_top = padding;
        self
    }

    /// Set bottom padding.
    pub fn padding_bottom(&mut self, padding: usize) -> &mut Self {
        self.padding_bottom = padding;
        self
    }

    /// Toggle whether words too wide to fit in the column are hyphenated when split.
    pub fn hyphenate(&mut self, hyphenate: bool) -> &mut Self {
        self.hyphenate = hyphenate;
        self
    }
}
