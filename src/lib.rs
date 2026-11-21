/*!

This crate provides a library for displaying tabular data in, for instance, a command
line app.

# Usage

Colonnade is [on crates.io](https://crates.io/crates/colonnade) and can be
used by adding `colonnade` to your dependencies in your project's `Cargo.toml`.

```toml
[dependencies]
colonnade = "2"
```

# Example

```rust
use colonnade::{Alignment, Colonnade};

// text to put in tabular form
let text = vec![
    vec![
        "Colonnade lets you format text in columns.",
        "As you can see, it supports text alignment, viewport width, and column widths.",
        "If you want to colorize your table, you'll need to use the macerate method.",
    ],
    vec!["", "Two or more rows of columns makes a table.", ""],
];

// 3 columns of text in an 80-character viewport
let mut colonnade = Colonnade::new(3, 80).unwrap();

// configure the table a bit
colonnade.left_margin(4).unwrap();
colonnade.columns[0].left_margin(8);      // the first column should have a left margin 8 spaces wide
colonnade.fixed_width(15).unwrap();                // first we set all the columns to 15 characters wide
colonnade.columns[1].clear_limits();      // but then remove this restriction on the central column
colonnade.columns[0].alignment(Alignment::Right);
colonnade.columns[1].alignment(Alignment::Center);
colonnade.spaces_between_rows(1);         // add a blank link between rows

// now print out the table
for line in colonnade.tabulate(&text).unwrap() {
    println!("{}", line);
}
```
which produces
```plain
         Colonnade lets     As you can see, it supports text     If you want to
        you format text      alignment, viewport width, and      colorize your
            in columns.              column widths.              table, you'll
                                                                 need to use the
                                                                 macerate
                                                                 method.

                           Two or more rows of columns makes
                                        a table.
```
If Colonnade doesn't have enough space in a column to fit the text, it will attempt to
wrap it, splitting on whitespace. If this is not possible because a word in the text is
so long it does not fit in the column, it will fit as much as it can, splitting mid-word
and marking the split with a hyphen (unless the column is only one character wide).

To control the layout you can specify minimum and maximum column widths and column priorities.
If the columns differ in priority, lower priority, higher priority number, columns will
get wrapped first.

# Optional Features

Colonnade by default takes full control over whitespace, stripping away any that exists in the data
it receives and adding it back in as needed to arrange the columns. If you want to regain some
control, to indent the beginning of paragraphs, say, you can use the `nbsp` feature, which causes
Colonnade to treat the non-breaking space character `\u{00A0}` like a non-space character.

To require `nbsp`, specify your Colonnade dependency like so in your Cargo.toml:

```toml
colonnade = { version = "^2.0.0", features = ["nbsp"] }
```

or

```toml
[dependencies.colonnade]
version  = "^2.0.0"
features = ["nbsp"]
```

This feature has a dependency on the `regex` and `lazy_static` crates.
*/

// Re-export public types
pub use alignment::{Alignment, VerticalAlignment};
pub use column::Column;
pub use error::ColonnadeError;

// Modules
mod alignment;
mod column;
mod error;
mod text;

use crate::text::{longest_word, to_words, true_width, width_after_normalization};

#[cfg(feature = "nbsp")]
use lazy_static::lazy_static;

extern crate strip_ansi_escapes;
use unicode_segmentation::UnicodeSegmentation;

/// Type alias for a cell in macerated output: (margin, content)
pub type Cell = (String, String);
/// Type alias for a line of cells
pub type Line = Vec<Cell>;
/// Type alias for a row of lines (one row may wrap to multiple lines)
pub type MaceratedRow = Vec<Line>;
/// Type alias for the complete macerated output
pub type Macerated = Vec<MaceratedRow>;

/// A struct holding formatting information. This is the object which tabulates data.
#[derive(Debug, Clone)]
pub struct Colonnade {
    pub columns: Vec<Column>,
    width: usize,
    spaces_between_rows: usize,
}

impl Colonnade {
    /// Construct a `Colonnade` with default values: left alignment, no column size
    /// constraints, no blank lines between rows, 1 space margin between columns.
    ///
    /// # Arguments
    ///
    /// * `columns` - The number of columns of data to expect
    /// * `width` - Viewport size in characters
    ///
    /// # Errors
    ///
    /// * `ColonnadeError::InsufficientSpace` - the viewport isn't wide enough for the columns and their margins
    ///
    /// # Example
    ///
    /// ```rust
    /// use colonnade::Colonnade;
    /// let colonnade = Colonnade::new(4, 100).unwrap();
    /// ```
    pub fn new(columns: usize, width: usize) -> Result<Colonnade, ColonnadeError> {
        if columns == 0 {
            return Err(ColonnadeError::InsufficientColumns);
        }
        let columns: Vec<Column> = (0..columns).map(Column::new).collect();
        let spec = Colonnade {
            columns,
            width,
            spaces_between_rows: 0,
        };
        if !spec.sufficient_space() {
            return Err(ColonnadeError::InsufficientSpace);
        }
        Ok(spec)
    }

    // the absolute minimal space that might fit this table assuming some data in every column
    fn minimal_width(&self) -> usize {
        self.columns
            .iter()
            .fold(0, |acc, v| acc + v.left_margin + v.min_width.unwrap_or(1))
    }

    fn sufficient_space(&self) -> bool {
        self.minimal_width() <= self.width
    }

    // the amount of space required to display the data given the current column specs
    fn required_width(&self) -> usize {
        self.columns.iter().fold(0, |acc, v| acc + v.outer_width())
    }

    // make a blank line as wide as the table
    fn blank_line(&self) -> String {
        " ".repeat(self.required_width())
    }

    fn maximum_vertical_padding(&self) -> usize {
        self.columns
            .iter()
            .map(|c| c.vertical_padding())
            .max()
            .unwrap_or(0)
    }

    fn len(&self) -> usize {
        self.columns.len()
    }

    /// Returns the width of the colonnade in columns if the colonnade has already laid out data
    /// and knows how much space this data will require.
    pub fn width(&self) -> Option<usize> {
        if self.adjusted() {
            Some(self.required_width())
        } else {
            None
        }
    }

    // returns priorities sorted lowest to highest
    fn priorities(&self) -> Vec<usize> {
        let mut v: Vec<usize> = self.columns.iter().map(|c| c.priority).collect();
        v.sort_unstable();
        v.dedup();
        v.reverse();
        v
    }

    /// Converts the raw data in `table` into a vector of strings representing the data in tabular form.
    pub fn tabulate<T, U, V, W, X>(&mut self, table: T) -> Result<Vec<String>, ColonnadeError>
    where
        T: IntoIterator<Item = U, IntoIter = V>,
        U: IntoIterator<Item = W, IntoIter = X>,
        V: Iterator<Item = U>,
        W: ToString,
        X: Iterator<Item = W>,
    {
        self.macerate(table)
            .map(Colonnade::reconstitute_rows)
    }

    /// Chew up the text into bits suitable for piecemeal layout.
    pub fn macerate<T, U, V, W, X>(&mut self, table: T) -> Result<Macerated, ColonnadeError>
    where
        T: IntoIterator<Item = U, IntoIter = V>,
        U: IntoIterator<Item = W, IntoIter = X>,
        V: Iterator<Item = U>,
        W: ToString,
        X: Iterator<Item = W>,
    {
        self.lay_out(table).map(|owned_table| {
            let ref_table = Colonnade::ref_table(&owned_table);
            let table = &ref_table;
            let mut buffer = vec![];
            let mut p = self.maximum_vertical_padding();
            if p == 0 {
                p = 1;
            }
            for (i, row) in table.iter().enumerate() {
                self.add_row(&mut buffer, row, i == table.len() - 1, p);
            }
            buffer
        })
    }

    // utility function to convert a T table to a String table
    fn own_table<T, U, V, W, X>(&self, table: T) -> Vec<Vec<String>>
    where
        T: IntoIterator<Item = U, IntoIter = V>,
        U: IntoIterator<Item = W, IntoIter = X>,
        V: Iterator<Item = U>,
        W: ToString,
        X: Iterator<Item = W>,
    {
        let mut table = table
            .into_iter()
            .map(|v| {
                v.into_iter()
                    .map(|t| {
                        let s = t.to_string();
                        let bytes = strip_ansi_escapes::strip(&s);
                        std::str::from_utf8(&bytes)
                            .expect("failed to restore bytes to utf8 string after stripping ansi escape sequences")
                            .to_string()
                    })
                    .collect::<Vec<String>>()
            })
            .collect::<Vec<Vec<String>>>();
        // pad rows as necessary
        for row in &mut table {
            while row.len() < self.len() {
                row.push(String::new());
            }
        }
        table
    }

    // utility function to convert a String table to a &str table
    fn ref_table(table: &[Vec<String>]) -> Vec<Vec<&str>> {
        table
            .iter()
            .map(|v| v.iter().map(|s| s.as_str()).collect::<Vec<&str>>())
            .collect::<Vec<Vec<&str>>>()
    }

    fn reconstitute_rows(maceration: Macerated) -> Vec<String> {
        maceration
            .iter()
            .flat_map(|row| {
                row.iter().map(|line| {
                    if line.len() == 1 && line[0].1.is_empty() {
                        String::new()
                    } else {
                        let mut l = String::new();
                        for (margin, text) in line {
                            l += margin;
                            l += text;
                        }
                        l
                    }
                })
            })
            .collect()
    }

    fn add_row(
        &self,
        buffer: &mut Macerated,
        row: &[&str],
        last_row: bool,
        maximum_vertical_padding: usize,
    ) {
        let mut words: Vec<(usize, Vec<&str>, usize)> = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, c)| {
                (
                    c.padding_top,
                    to_words(row.get(i).unwrap_or(&"")),
                    c.padding_bottom,
                )
            })
            .collect();

        let mut current_lines: Vec<Vec<(String, String)>> = Vec::new();

        if words.iter().all(|(_, sentence, _)| sentence.is_empty()) {
            for _ in 0..maximum_vertical_padding {
                current_lines.push(
                    self.columns
                        .iter()
                        .map(|c| (c.margin(), c.blank_line()))
                        .collect(),
                );
            }
            if !last_row {
                for _ in 0..self.spaces_between_rows {
                    current_lines.push(vec![(self.blank_line(), String::new())]);
                }
            }
        } else {
            while !words
                .iter()
                .all(|(pt, sentence, pb)| *pb == 0 && *pt == 0 && sentence.is_empty())
            {
                let mut pieces = vec![];
                for (i, c) in self.columns.iter().enumerate() {
                    let left_margin = c.margin();
                    let mut line = String::new();
                    let tuple = &mut words[i];
                    if tuple.0 > 0 {
                        line = c.blank_line();
                        tuple.0 -= 1;
                    } else if tuple.1.is_empty() {
                        line = c.blank_line();
                        if tuple.2 > 0 {
                            tuple.2 -= 1;
                        }
                    } else {
                        let mut l = c.padding_left;
                        let mut phrase = " ".repeat(l);
                        let mut first = true;
                        while !tuple.1.is_empty() {
                            let w = tuple.1.remove(0);
                            if first {
                                let wl = true_width(w) + c.padding_right;
                                if wl == c.width {
                                    phrase += w;
                                    break;
                                } else if wl > c.width {
                                    let hyphenating = c.hyphenating();
                                    let mut offset = c.inner_width();
                                    if hyphenating {
                                        offset -= 1;
                                    }
                                    let graphemes: Vec<&str> = UnicodeSegmentation::graphemes(w, true).collect();
                                    let prefix = graphemes[..offset].to_vec().join("");
                                    let byte_offset = prefix.len();
                                    phrase += &prefix;
                                    tuple.1.insert(0, &w[byte_offset..]);
                                    if hyphenating {
                                        phrase += "-";
                                    }
                                    break;
                                }
                            }
                            let new_length = l + true_width(w) + if first { 0 } else { 1 };
                            if new_length + c.padding_right > c.width {
                                tuple.1.insert(0, w);
                                break;
                            } else {
                                if first {
                                    first = false;
                                } else {
                                    phrase += " ";
                                }
                                phrase += w;
                                l = new_length;
                            }
                        }
                        let phrase_width = true_width(&phrase);
                        if phrase_width < c.width {
                            let surplus = c.width - phrase_width;
                            match c.alignment {
                                Alignment::Left => {
                                    line += &phrase;
                                    line += &" ".repeat(surplus);
                                }
                                Alignment::Center => {
                                    let left_bit = surplus / 2;
                                    line += &" ".repeat(left_bit);
                                    line += &phrase;
                                    line += &" ".repeat(surplus - left_bit);
                                }
                                Alignment::Right => {
                                    line += &" ".repeat(surplus - c.padding_right);
                                    line += &phrase;
                                    line += &" ".repeat(c.padding_right);
                                }
                                Alignment::Justify => {
                                    let words: Vec<&str> = phrase.split(' ').collect();
                                    let last_line = tuple.1.is_empty();
                                    if last_line || words.len() == 1 {
                                        line += &phrase;
                                        line += &" ".repeat(surplus);
                                    } else {
                                        let gaps = words.len() - 1;
                                        let rearrangeable = surplus + gaps - c.padding_right;
                                        let min_spacer = rearrangeable / gaps;
                                        let extra = rearrangeable - min_spacer * gaps;
                                        let extra_offset = words.len() - extra;
                                        for (i, word) in words.iter().enumerate() {
                                            if i == 0 {
                                                line += word;
                                            } else {
                                                line += &" ".repeat(min_spacer);
                                                if i >= extra_offset {
                                                    line += " ";
                                                }
                                                line += word;
                                            }
                                        }
                                        line += &" ".repeat(c.padding_right);
                                    }
                                }
                            }
                        } else {
                            line += &phrase;
                        }
                    }
                    pieces.push((left_margin, line));
                }
                current_lines.push(pieces);
            }

            'outer: for c in &self.columns {
                match c.vertical_alignment {
                    VerticalAlignment::Top => (),
                    _ => {
                        let blank = c.blank_line();
                        let end = current_lines.len() - c.padding_bottom;
                        let mut movable_lines = 0;
                        let mut pointer = end - 1;
                        let top_pointer = c.padding_top;
                        while pointer < current_lines.len()
                            && current_lines[pointer][c.index].1 == blank
                        {
                            movable_lines += 1;
                            if pointer == top_pointer {
                                continue 'outer;
                            }
                            if pointer == 0 {
                                break;
                            }
                            pointer -= 1;
                        }
                        if movable_lines == 0 {
                            continue 'outer;
                        }
                        let lines_to_move = if c.vertical_alignment == VerticalAlignment::Middle {
                            movable_lines / 2
                        } else {
                            movable_lines
                        };
                        let mut rotator: Vec<(String, String)> = (top_pointer..end)
                            .map(|i| current_lines[i].remove(c.index))
                            .collect();
                        for _ in 0..lines_to_move {
                            let pair = rotator.pop().unwrap();
                            rotator.insert(0, pair);
                        }
                        for i in top_pointer..end {
                            current_lines[i].insert(c.index, rotator.remove(0));
                        }
                    }
                }
            }

            if !last_row {
                for _ in 0..self.spaces_between_rows {
                    current_lines.push(vec![(self.blank_line(), String::new())]);
                }
            }
        }
        buffer.push(current_lines);
    }

    /// Erase column widths established by a previous `tabulate` or `macerate`.
    pub fn reset(&mut self) -> &mut Self {
        for column in &mut self.columns {
            column.adjusted = false;
            column.width = 0;
        }
        self
    }

    fn adjusted(&self) -> bool {
        self.columns.iter().all(|c| c.adjusted)
    }

    fn mark_adjusted(&mut self) {
        for column in &mut self.columns {
            column.adjusted = true;
        }
    }

    fn lay_out<T, U, V, W, X>(&mut self, table: T) -> Result<Vec<Vec<String>>, ColonnadeError>
    where
        T: IntoIterator<Item = U, IntoIter = V>,
        U: IntoIterator<Item = W, IntoIter = X>,
        V: Iterator<Item = U>,
        W: ToString,
        X: Iterator<Item = W>,
    {
        let owned_table = self.own_table(table);
        if self.adjusted() {
            return Ok(owned_table);
        }
        self.reset();
        let ref_table = Colonnade::ref_table(&owned_table);
        let table = &ref_table;

        for (i, row) in table.iter().enumerate() {
            if row.len() != self.len() {
                return Err(ColonnadeError::InconsistentColumns(i, row.len(), self.len()));
            }
        }

        if !self.sufficient_space() {
            return Err(ColonnadeError::InsufficientSpace);
        }

        for row in table {
            for (c_idx, cell) in row.iter().enumerate() {
                let width = width_after_normalization(cell) + self.columns[c_idx].horizontal_padding();
                if width >= self.columns[c_idx].width {
                    self.columns[c_idx].expand(width);
                }
            }
        }

        if self.required_width() <= self.width {
            self.mark_adjusted();
            return Ok(owned_table);
        }

        let mut modified_columns: Vec<usize> = Vec::with_capacity(self.len());
        for priority in self.priorities() {
            for c_idx in 0..self.len() {
                if self.columns[c_idx].priority == priority && self.columns[c_idx].is_shrinkable() {
                    modified_columns.push(c_idx);
                    self.columns[c_idx].shrink(0);
                    for row in table {
                        let word_width = longest_word(row[c_idx]) + self.columns[c_idx].horizontal_padding();
                        if word_width > self.columns[c_idx].width {
                            self.columns[c_idx].expand(word_width);
                        }
                    }
                }
            }
            if self.required_width() <= self.width {
                break;
            }
        }

        if self.required_width() > self.width {
            let truncatable_columns: Vec<usize> = self
                .columns
                .iter()
                .enumerate()
                .filter(|(_, c)| c.is_shrinkable())
                .map(|(i, _)| i)
                .collect();

            let mut priorities: Vec<usize> = truncatable_columns
                .iter()
                .map(|&i| self.columns[i].priority)
                .collect();
            priorities.sort_unstable();
            priorities.dedup();
            priorities.reverse();

            'outer: for priority in priorities {
                let mut shrinkables: Vec<usize> = truncatable_columns
                    .iter()
                    .filter(|&&i| self.columns[i].priority == priority)
                    .copied()
                    .collect();
                loop {
                    let excess = self.required_width().saturating_sub(self.width);
                    if excess == 0 {
                        break 'outer;
                    }
                    if excess <= shrinkables.len() {
                        shrinkables.retain(|&i| self.columns[i].shrink_by(1));
                    } else {
                        let share = excess / shrinkables.len();
                        shrinkables.retain(|&i| self.columns[i].shrink_by(share));
                    }
                    if shrinkables.is_empty() {
                        break;
                    }
                }
            }

            if self.required_width() > self.width {
                return Err(ColonnadeError::InsufficientSpace);
            }
        } else if self.required_width() < self.width {
            modified_columns.retain(|&i| self.columns[i].is_expandable());
            if !modified_columns.is_empty() {
                while self.required_width() < self.width {
                    if let Some(priority) = modified_columns
                        .iter()
                        .map(|&i| self.columns[i].priority)
                        .min()
                    {
                        let mut winners: Vec<usize> = modified_columns
                            .iter()
                            .filter(|&&i| self.columns[i].priority == priority)
                            .copied()
                            .collect();
                        let surplus = self.width.saturating_sub(self.required_width());
                        if surplus <= winners.len() {
                            for &i in &winners[..surplus] {
                                self.columns[i].width += 1;
                            }
                        } else {
                            loop {
                                let surplus = self.width.saturating_sub(self.required_width());
                                if surplus == 0 {
                                    break;
                                }
                                winners.retain(|&i| self.columns[i].is_expandable());
                                if winners.is_empty() {
                                    break;
                                }
                                if surplus <= winners.len() {
                                    for &i in &winners[..surplus] {
                                        self.columns[i].width += 1;
                                    }
                                    break;
                                }
                                let mut changed = false;
                                let share = surplus / winners.len();
                                for &i in &winners {
                                    changed = self.columns[i].expand_by(share) || changed;
                                }
                                if !changed {
                                    break;
                                }
                            }
                            modified_columns.retain(|&i| self.columns[i].priority != priority);
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        self.mark_adjusted();
        Ok(owned_table)
    }

    /// Specify a number of blank lines to insert between table rows.
    pub fn spaces_between_rows(&mut self, n: usize) -> &mut Self {
        self.spaces_between_rows = n;
        self
    }

    /// Assign the same priority to all columns.
    pub fn priority(&mut self, priority: usize) -> &mut Self {
        for column in &mut self.columns {
            column.priority = priority;
        }
        self
    }

    /// Assign the same maximum width to all columns.
    pub fn max_width(&mut self, max_width: usize) -> Result<&mut Self, ColonnadeError> {
        for column in &mut self.columns {
            column.max_width(max_width)?;
        }
        Ok(self)
    }

    /// Assign the same minimum width to all columns.
    pub fn min_width(&mut self, min_width: usize) -> Result<&mut Self, ColonnadeError> {
        for column in &mut self.columns {
            column.min_width(min_width)?;
        }
        if !self.sufficient_space() {
            Err(ColonnadeError::InsufficientSpace)
        } else {
            Ok(self)
        }
    }

    /// Assign the same maximum and minimum width to all columns.
    pub fn fixed_width(&mut self, width: usize) -> Result<&mut Self, ColonnadeError> {
        for column in &mut self.columns {
            column.fixed_width(width)?;
        }
        Ok(self)
    }

    /// Remove any maximum or minimum column widths.
    pub fn clear_limits(&mut self) -> &mut Self {
        for column in &mut self.columns {
            column.clear_limits();
        }
        self
    }

    /// Assign all columns the same alignment.
    pub fn alignment(&mut self, alignment: Alignment) -> &mut Self {
        for column in &mut self.columns {
            column.alignment(alignment);
        }
        self
    }

    /// Assign all columns the same vertical alignment.
    pub fn vertical_alignment(&mut self, vertical_alignment: VerticalAlignment) -> &mut Self {
        for column in &mut self.columns {
            column.vertical_alignment(vertical_alignment);
        }
        self
    }

    /// Assign all columns the same left margin.
    pub fn left_margin(&mut self, left_margin: usize) -> Result<&mut Self, ColonnadeError> {
        for column in &mut self.columns {
            column.left_margin(left_margin);
        }
        if !self.sufficient_space() {
            Err(ColonnadeError::InsufficientSpace)
        } else {
            Ok(self)
        }
    }

    /// Assign all columns the same padding.
    pub fn padding(&mut self, padding: usize) -> Result<&mut Self, ColonnadeError> {
        for column in &mut self.columns {
            column.padding(padding);
        }
        if !self.sufficient_space() {
            Err(ColonnadeError::InsufficientSpace)
        } else {
            Ok(self)
        }
    }

    /// Assign all columns the same horizontal padding.
    pub fn padding_horizontal(&mut self, padding: usize) -> Result<&mut Self, ColonnadeError> {
        for column in &mut self.columns {
            column.padding_horizontal(padding);
        }
        if !self.sufficient_space() {
            Err(ColonnadeError::InsufficientSpace)
        } else {
            Ok(self)
        }
    }

    /// Assign all columns the same left padding.
    pub fn padding_left(&mut self, padding: usize) -> Result<&mut Self, ColonnadeError> {
        for column in &mut self.columns {
            column.padding_left(padding);
        }
        if !self.sufficient_space() {
            Err(ColonnadeError::InsufficientSpace)
        } else {
            Ok(self)
        }
    }

    /// Assign all columns the same right padding.
    pub fn padding_right(&mut self, padding: usize) -> Result<&mut Self, ColonnadeError> {
        for column in &mut self.columns {
            column.padding_right(padding);
        }
        if !self.sufficient_space() {
            Err(ColonnadeError::InsufficientSpace)
        } else {
            Ok(self)
        }
    }

    /// Assign all columns the same vertical padding.
    pub fn padding_vertical(&mut self, padding: usize) -> &mut Self {
        for column in &mut self.columns {
            column.padding_vertical(padding);
        }
        self
    }

    /// Assign all columns the same top padding.
    pub fn padding_top(&mut self, padding: usize) -> &mut Self {
        for column in &mut self.columns {
            column.padding_top(padding);
        }
        self
    }

    /// Assign all columns the same bottom padding.
    pub fn padding_bottom(&mut self, padding: usize) -> &mut Self {
        for column in &mut self.columns {
            column.padding_bottom(padding);
        }
        self
    }

    /// Toggle the hyphenation of all columns.
    pub fn hyphenate(&mut self, hyphenate: bool) -> &mut Self {
        for column in &mut self.columns {
            column.hyphenate(hyphenate);
        }
        self
    }
}
