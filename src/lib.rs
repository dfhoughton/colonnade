/*!

This crate provides a library for displaying tabular data in, for instance, a command
line app.

# Usage

Colonnade is [on crates.io](https://crates.io/crates/colonnade) and can be
used by adding `colonnade` to your dependencies in your project's `Cargo.toml`.

```toml
[dependencies]
colonnade = "0"
```

# Example

```rust
extern crate colonnade;
use colonnade::{Alignment, Colonnade};

#[allow(unused_must_use)]
fn main() {
    // text to put in tabular form
    let text = vec![
        vec![
            "Colonnade lets you format text in columns.",
            "As you can see, it supports text alignment, viewport width, and column widths.",
            "It doesn't yet support color codes or other formatting, though that may come.",
        ],
        vec!["", "Two or more rows of columns makes a table.", ""],
    ];

    // 3 columns of text in an 80-character viewport
    let mut colonnade = Colonnade::new(3, 80).unwrap();

    // configure the table a bit
    colonnade.left_margin_all(4);
    colonnade.left_margin(0, 8);              // the first column should have a left margin 8 spaces wide
    colonnade.fixed_width_all(15);            // first we set all the columns to 15 characters wide
    colonnade.clear_limits(1);                // but then remove this restriction on the central column
    colonnade.alignment(0, Alignment::Right);
    colonnade.alignment(1, Alignment::Center);
    colonnade.alignment(2, Alignment::Left);
    colonnade.spaces_between_rows(1);         // add a blank link between rows

    // now print out the table
    for line in colonnade.tabulate(&text).unwrap() {
        println!("{}", line);
    }
}
```
which produces
```plain
         Colonnade lets     As you can see, it supports text     It doesn't yet
        you format text      alignment, viewport width, and      support color
            in columns.              column widths.              codes or other
                                                                 formatting,
                                                                 though that may
                                                                 come.

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
*/
use std::fmt;
use std::iter;

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

impl std::fmt::Display for ColonnadeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ColonnadeError {}

/// Alignments one can apply to columns of text.
#[derive(Debug, Clone)]
pub enum Alignment {
    /// Left justification -- the default alignment
    Left,
    /// Right justification
    Right,
    /// Centering
    Center,
}

#[derive(Debug, Clone)]
struct ColumnSpec {
    alignment: Alignment,
    left_margin: usize,
    width: usize,
    priority: usize,
    min_width: Option<usize>,
    max_width: Option<usize>,
    padding_left: usize,
    padding_right: usize,
    padding_top: usize,
    padding_bottom: usize,
}

impl ColumnSpec {
    fn default() -> ColumnSpec {
        ColumnSpec {
            alignment: Alignment::Left,
            left_margin: 1,
            width: 0, // claimed width
            priority: usize::max_value(),
            min_width: None,
            max_width: None,
            padding_left: 0,
            padding_right: 0,
            padding_top: 0,
            padding_bottom: 0,
        }
    }
    fn horizonal_padding(&self) -> usize {
        self.padding_left + self.padding_right
    }
    fn vertical_padding(&self) -> usize {
        self.padding_top + self.padding_bottom
    }
    fn minimum_width(&self) -> usize {
        let w1 = self.horizonal_padding();
        let w2 = self.min_width.unwrap_or(w1);
        if w2 > w1 {
            w2
        } else {
            w1
        }
    }
    fn effective_width(&self) -> usize {
        let w = if self.max_width.unwrap_or(self.width) < self.width {
            self.max_width.unwrap()
        } else {
            self.width
        };
        let m = self.minimum_width();
        if m > w {
            m
        } else {
            w
        }
    }
    fn is_shrinkable(&self) -> bool {
        self.minimum_width() < self.width
    }
    // shrink as close to width as possible
    fn shrink(&mut self, width: usize) {
        let m = self.minimum_width();
        self.width = if m > width { m } else { width }
    }
    // attempt to shrink by decrease amount
    // returns whether there was any shrinkage
    fn shrink_by(&mut self, decrease: usize) -> bool {
        if self.is_shrinkable() {
            // you can't shrink all the way to 0
            let decrease = if decrease >= self.width {
                1
            } else {
                self.width - decrease
            };
            let before = self.width;
            self.shrink(decrease);
            before != self.width
        } else {
            false
        }
    }
    fn is_expandable(&self) -> bool {
        self.max_width.unwrap_or(usize::max_value()) > self.width
    }
    // expands column as much as possible to fit width and as much as necessary to match min_width
    fn expand(&mut self, width: usize) -> bool {
        if width <= self.width {
            return false;
        }
        let change = if self.max_width.unwrap_or(width) < width {
            self.max_width.unwrap()
        } else if self.minimum_width() > width {
            self.minimum_width()
        } else {
            width
        };
        let changed = self.width != change;
        if changed {
            self.width = change
        }
        changed
    }
    fn expand_by(&mut self, increase: usize) -> bool {
        self.expand(self.width + increase)
    }
    fn outer_width(&self) -> usize {
        self.left_margin + self.effective_width()
    }
    fn blank_line(&self) -> String {
        " ".repeat(self.width)
    }
    fn margin(&self) -> String {
        " ".repeat(self.left_margin)
    }
}

/// A struct holding formatting information. This is the object which tabulates data.
#[derive(Debug, Clone)]
pub struct Colonnade {
    colonnade: Vec<ColumnSpec>,
    width: usize,
    spaces_between_rows: usize,
    adjusted: bool,
}

// find the longest sequence of non-whitespace characters in a string
fn longest_word(s: &str) -> usize {
    s.split_whitespace()
        .fold(0, |acc, v| if v.len() > acc { v.len() } else { acc })
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
    /// # extern crate colonnade;
    /// # use colonnade::Colonnade;
    /// let colonnade = Colonnade::new(4, 100);
    /// ```
    pub fn new(columns: usize, width: usize) -> Result<Colonnade, ColonnadeError> {
        if columns == 0 {
            return Err(ColonnadeError::InsufficientColumns);
        }
        let mut colonnade: Vec<ColumnSpec> =
            iter::repeat(ColumnSpec::default()).take(columns).collect();
        colonnade[0].left_margin = 0;
        let spec = Colonnade {
            colonnade,
            width,
            spaces_between_rows: 0,
            adjusted: false,
        };
        if !spec.sufficient_space() {
            return Err(ColonnadeError::InsufficientSpace);
        }
        Ok(spec)
    }
    // the absolute minimal space that might fit this table assuming some data in every column
    fn minimal_width(&self) -> usize {
        self.colonnade
            .iter()
            .fold(0, |acc, v| acc + v.left_margin + v.min_width.unwrap_or(1)) // assume each column requires at least one character
    }
    fn sufficient_space(&self) -> bool {
        self.minimal_width() <= self.width
    }
    // the amount of space required to display the data given the current column specs
    fn required_width(&self) -> usize {
        self.colonnade
            .iter()
            .fold(0, |acc, v| acc + v.outer_width())
    }
    // make a blank line as wide as the table
    fn blank_line(&self) -> String {
        " ".repeat(self.required_width())
    }
    fn maximum_vertical_padding(&self) -> usize {
        let mut p = 0;
        for c in &self.colonnade {
            let p2 = c.vertical_padding();
            if p2 > p {
                p = p2;
            }
        }
        p
    }
    fn len(&self) -> usize {
        self.colonnade.len()
    }
    // determine the characters required to represent s after whitespace normalization
    fn width_after_normalization(s: &str) -> usize {
        let mut l = 0;
        for w in s.trim().split_whitespace() {
            if l != 0 {
                l += 1;
            }
            l += w.len();
        }
        l
    }
    // returns priorites sorted lowest to highest
    fn priorities(&self) -> Vec<usize> {
        let mut v = self
            .colonnade
            .iter()
            .map(|c| c.priority)
            .collect::<Vec<_>>();
        v.sort_unstable();
        v.dedup();
        v.reverse();
        v
    }
    /// Converts the raw data in `table` into a vector of strings representing the data in tabular form.
    /// Blank lines will be zero-width rather than full-width lines of whitespace.
    ///
    /// # Arguments
    ///
    /// * `table` - The data to display.
    ///
    /// # Errors
    ///
    /// Any errors of [`lay_out`](#method.lay_out). If the data has already been laid out, this method will throw no errors.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::Colonnade;
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// let data = vec![vec!["some", "words", "for", "example"]];
    /// let lines = colonnade.tabulate(&data)?;
    /// # Ok(()) }
    /// ```
    pub fn tabulate<T>(&mut self, table: &Vec<Vec<T>>) -> Result<Vec<String>, ColonnadeError>
    where
        T: ToString,
    {
        match self.macerate(table) {
            Ok(buffer) => Ok(Colonnade::reconstitute_rows(buffer)),
            Err(e) => Err(e),
        }
    }
    /// Converts the raw data in `table` into a vector of vectors of `(String, String)` tuples
    /// representing the data in tabular form. Each tuple consists of a whitespace left margin and
    /// the contents of a column. Separator lines will consist of a margin and text tuple where the
    /// text is zero-width and the "margin" is as wide as the table.
    ///
    /// Maceration is useful if you wish to insert color codes to colorize the data or otherwise
    /// manipulate the data post-layout.
    ///
    /// # Arguments
    ///
    /// * `table` - The data to display.
    ///
    /// # Errors
    ///
    /// Any errors of [`lay_out`](#method.lay_out). If the data has already been laid out, this method will throw no errors.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::Colonnade;
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// let data = vec![vec!["some", "words", "for", "example"]];
    /// let lines = colonnade.tabulate(&data)?;
    /// # Ok(()) }
    /// ```
    pub fn macerate<T>(
        &mut self,
        table: &Vec<Vec<T>>,
    ) -> Result<Vec<Vec<(String, String)>>, ColonnadeError>
    where
        T: ToString,
    {
        if !self.adjusted {
            match self.lay_out(table) {
                Err(e) => return Err(e),
                Ok(()) => (),
            }
        }
        let owned_table = Colonnade::own_table(table);
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
        Ok(buffer)
    }
    // utility function to convert a T table to a String table
    fn own_table<T: ToString>(table: &Vec<Vec<T>>) -> Vec<Vec<String>> {
        table
            .iter()
            .map(|v| v.iter().map(|t| t.to_string()).collect::<Vec<String>>())
            .collect::<Vec<Vec<String>>>()
    }
    // utility function to convert a String table to a &str table
    fn ref_table(table: &Vec<Vec<String>>) -> Vec<Vec<&str>> {
        table
            .iter()
            .map(|v| v.iter().map(|s| s.as_ref()).collect::<Vec<&str>>())
            .collect::<Vec<Vec<&str>>>()
    }
    fn reconstitute_rows(maceration: Vec<Vec<(String, String)>>) -> Vec<String> {
        maceration
            .iter()
            .map(|line| {
                if line.len() == 1 && line[0].1.len() == 0 {
                    String::new() // return empty strings instead of fat lines for blank lines
                } else {
                    let mut l = String::new();
                    for (margin, text) in line {
                        l += margin;
                        l += text;
                    }
                    l
                }
            })
            .collect()
    }
    // take one row of untabulated pieces of text and turn it into one or more vectors of (String,String) tuples,
    // where each tuple represenst a left margin and some column text, the each vector representing one line of tabulated text
    fn add_row(
        &self,
        buffer: &mut Vec<Vec<(String, String)>>,
        row: &Vec<&str>,
        last_row: bool,
        maximum_vertical_padding: usize,
    ) {
        // turn the row, a list of blobs of text, into a list of lists of words, recording also the amount of blank space
        // we need on either side of the words
        let mut words: Vec<(usize, Vec<&str>, usize)> = row
            .iter()
            .enumerate()
            .map(|(i, w)| {
                (
                    self.colonnade[i].padding_top,
                    w.trim().split_whitespace().collect(),
                    self.colonnade[i].padding_bottom,
                )
            })
            .collect();
        // if all these lists are empty, just add a blank line (and maybe additional blank separator lines)
        if words.iter().all(|(_, sentence, _)| sentence.is_empty()) {
            for _ in 0..maximum_vertical_padding {
                buffer.push(
                    self.colonnade
                        .iter()
                        .map(|c| (c.margin(), c.blank_line()))
                        .collect(),
                );
            }
            if !last_row {
                for _ in 0..self.spaces_between_rows {
                    buffer.push(vec![(self.blank_line(), String::new())]);
                }
            }
            return;
        }
        // otherwise, we build these lists into lines, we may use up some of these lists before others
        while !words
            .iter()
            .all(|(pt, sentence, pb)| pb == &0 && pt == &0 && sentence.is_empty())
        {
            let mut pieces = vec![];
            for (i, c) in self.colonnade.iter().enumerate() {
                let left_margin = c.margin();
                let mut line = String::new();
                let mut tuple = &mut words[i];
                if tuple.0 > 0 {
                    line = c.blank_line();
                    tuple.0 -= 1;
                } else if tuple.1.is_empty() {
                    // we've used this one up, but there are still words to deal with in other sentences
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
                            if w.len() == c.width {
                                // word fills column
                                phrase += w;
                                break;
                            } else if w.len() > c.width {
                                // word overflows column and we must split it
                                if c.width > 1 {
                                    phrase += &w[0..(c.width - 1)];
                                    tuple.1.insert(0, &w[(c.width - 1)..w.len()]);
                                    phrase += "-";
                                } else {
                                    phrase += &w[0..1];
                                    tuple.1.insert(0, &w[1..w.len()]);
                                }
                                break;
                            }
                        }
                        // try to tack on a new word
                        let new_length = l + w.len() + if first { 0 } else { 1 };
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
                    // pad phrase out properly in its cell
                    if phrase.len() < c.width {
                        let surplus = c.width - phrase.len();
                        match c.alignment {
                            Alignment::Left => {
                                line += &phrase;
                                for _ in 0..surplus {
                                    line += " "
                                }
                            }
                            Alignment::Center => {
                                let left_bit = surplus / 2;
                                for _ in 0..left_bit {
                                    line += " "
                                }
                                line += &phrase;
                                for _ in 0..(surplus - left_bit) {
                                    line += " "
                                }
                            }
                            Alignment::Right => {
                                for _ in 0..(surplus - c.padding_right) {
                                    line += " "
                                }
                                line += &phrase;
                                for _ in 0..c.padding_right {
                                    line += " "
                                }
                            }
                        }
                    } else {
                        line += &phrase;
                    }
                }
                pieces.push((left_margin, line));
            }
            buffer.push(pieces);
        }
        if !last_row {
            for _ in 0..self.spaces_between_rows {
                buffer.push(vec![(self.blank_line(), String::new())]);
            }
        }
    }
    /// Determine column widths given data.
    ///
    /// Normally you do not need to call this method because it is called when you [`tabulate`](#method.tabulate)
    /// the first batch of data. However, this initial layout will then be used for every subsequent batch
    /// of data regardless of its size. If you want to re-flow the table to better fit the new data, you acn
    /// call `layout`.
    ///
    /// # Arguments
    ///
    /// * `table` - The data to display.
    ///
    /// # Errors
    ///
    /// * `ColonnadeError::InconsistentColumns` - The number of columns in some row of `table` does not match the spec.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::Colonnade;
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// let data = vec![vec!["some", "words", "for", "example"]];
    /// let lines = colonnade.tabulate(&data)?;
    /// // ... later
    /// let data = vec![vec!["a very different, wider", "set of", "words that won't fit comfortably in the old layout"]];
    /// // reflow table
    /// colonnade.lay_out(&data)?;
    /// let lines = colonnade.tabulate(&data)?;
    /// # Ok(()) }
    /// ```
    pub fn lay_out<T>(&mut self, table: &Vec<Vec<T>>) -> Result<(), ColonnadeError>
    where
        T: ToString,
    {
        let owned_table = Colonnade::own_table(table);
        let ref_table = Colonnade::ref_table(&owned_table);
        let table = &ref_table;
        // validate table
        for i in 0..table.len() {
            let row = &table[i];
            if row.len() != self.len() {
                return Err(ColonnadeError::InconsistentColumns(
                    i,
                    row.len(),
                    self.len(),
                ));
            }
        }
        // first try to do it all without splitting
        for i in 0..table.len() {
            for c in 0..self.len() {
                let m = Colonnade::width_after_normalization(&table[i][c]);
                if m >= self.colonnade[c].width {
                    // = to force initial expansion to min width
                    self.colonnade[c].expand(m);
                }
            }
        }
        if self.required_width() <= self.width {
            self.adjusted = true;
            return Ok(());
        }
        let mut modified_columns: Vec<usize> = Vec::with_capacity(self.len());
        // try shrinking columns to their longest word by order of priority
        for p in self.priorities() {
            for c in 0..self.len() {
                if self.colonnade[c].priority == p && self.colonnade[c].is_shrinkable() {
                    modified_columns.push(c);
                    self.colonnade[c].shrink(0);
                    for r in 0..table.len() {
                        let m = longest_word(&table[r][c]);
                        if m > self.colonnade[c].width {
                            self.colonnade[c].expand(m);
                        }
                    }
                }
            }
            if self.required_width() <= self.width {
                break;
            }
        }
        if self.required_width() > self.width {
            // forcibly truncate long columns
            let mut truncatable_columns = self.colonnade.iter().enumerate().collect::<Vec<_>>();
            truncatable_columns.retain(|(_, c)| c.is_shrinkable());
            let truncatable_columns: Vec<usize> =
                truncatable_columns.iter().map(|(i, _)| *i).collect();
            let mut priorities: Vec<usize> = truncatable_columns
                .iter()
                .map(|&i| self.colonnade[i].priority)
                .collect();
            priorities.sort_unstable();
            priorities.dedup();
            priorities.reverse();
            'outer: for p in priorities {
                let mut shrinkables: Vec<&usize> = truncatable_columns
                    .iter()
                    .filter(|&&i| self.colonnade[i].priority == p)
                    .collect();
                loop {
                    let excess = self.required_width() - self.width;
                    if excess == 0 {
                        break 'outer;
                    }
                    if excess <= shrinkables.len() {
                        shrinkables.retain(|&&i| self.colonnade[i].shrink_by(1));
                    } else {
                        let share = excess / shrinkables.len();
                        shrinkables.retain(|&&i| self.colonnade[i].shrink_by(share));
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
            // try to give back surplus space
            modified_columns.retain(|&i| self.colonnade[i].is_expandable());
            if !modified_columns.is_empty() {
                while self.required_width() < self.width {
                    // find highest priority among modified columns
                    if let Some(priority) = modified_columns
                        .iter()
                        .map(|&i| self.colonnade[i].priority)
                        .min()
                    {
                        // there are still some modified columns we haven't restored any space to
                        let mut winners: Vec<&usize> = modified_columns
                            .iter()
                            .filter(|&&i| self.colonnade[i].priority == priority)
                            .collect();
                        let surplus = self.width - self.required_width();
                        if surplus <= winners.len() {
                            // give one column back to as many of the winners as possible and call it a day
                            // we will necessarily break out of the loop after this
                            for &&i in winners.iter().take(surplus) {
                                self.colonnade[i].width += 1;
                            }
                        } else {
                            // give a share back to each winner
                            loop {
                                let surplus = self.width - self.required_width();
                                if surplus == 0 {
                                    break;
                                }
                                winners.retain(|&&i| self.colonnade[i].is_expandable());
                                if winners.is_empty() {
                                    break;
                                }
                                if surplus <= winners.len() {
                                    for &&i in winners.iter().take(surplus) {
                                        self.colonnade[i].width += 1;
                                    }
                                    break;
                                }
                                let mut changed = false;
                                let share = surplus / winners.len();
                                for &&i in winners.iter() {
                                    let change = self.colonnade[i].expand_by(share);
                                    changed = changed || change;
                                }
                                if !changed {
                                    break;
                                }
                            }
                            modified_columns.retain(|&i| self.colonnade[i].priority != priority);
                        }
                    } else {
                        break;
                    }
                }
            }
        }
        self.adjusted = true;
        Ok(())
    }
    /// Specify a number of blank lines to insert between table rows.
    ///
    /// # Arguments
    ///
    /// * `n` - A number of spaces.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::Colonnade;
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// // we want rows to be separated by a single blank line
    /// colonnade.spaces_between_rows(1);
    /// # Ok(()) }
    /// ```
    pub fn spaces_between_rows(&mut self, n: usize) {
        self.spaces_between_rows = n;
    }
    /// Assign the same priority to all columns. By default, all columns have the lowest priority.
    ///
    /// Priority determines the order in which columns give up space when the viewport lacks sufficient
    /// space to display all columns without wrapping. Lower priority columns give up space first.
    ///
    /// # Arguments
    ///
    /// * `priority` - The common priority. Lower numbers confer higher priority; 0 is the highest priority.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::Colonnade;
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// // assign all columns the highest priority
    /// colonnade.priority_all(0);
    /// // now demote the last column
    /// colonnade.priority(3, 1);
    /// # Ok(()) }
    /// ```
    pub fn priority_all(&mut self, priority: usize) {
        for i in 0..self.len() {
            self.colonnade[i].priority = priority;
        }
    }
    /// Assign a particular priority to a particular column.
    ///
    /// Priority determines the order in which columns give up space when the viewport lacks sufficient
    /// space to display all columns without wrapping. Lower priority columns give up space first.
    ///
    /// # Arguments
    ///
    /// * `index` - Which column to which you wish to assign priority.
    /// * `priority` - The column's priority. Lower numbers confer higher priority; 0 is the highest priority.
    ///
    /// # Error
    ///
    /// * `ColonnadeError::OutOfBounds` - The index is beyond the bounds of the spec.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::Colonnade;
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// // assign all columns the highest priority
    /// colonnade.priority_all(0);
    /// // now demote the last column
    /// colonnade.priority(3, 1)?;
    /// # Ok(()) }
    /// ```
    pub fn priority(&mut self, index: usize, priority: usize) -> Result<(), ColonnadeError> {
        if index < self.len() {
            self.colonnade[index].priority = priority;
            Ok(())
        } else {
            Err(ColonnadeError::OutOfBounds)
        }
    }
    /// Assign the same maximum width to all columns. By default columns have no maximum width.
    ///
    /// # Arguments
    ///
    /// * `max_width` - The common maximum width.
    ///
    /// # Errors
    ///
    /// * `ColonnadeError::MinGreaterThanMax` - Assigning a maximum width in conflict with some assigned minimum width.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::Colonnade;
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// // assign all columns a maximum width of 20
    /// colonnade.max_width_all(20)?;
    /// // at most we will now use only 83 of the characters provided by the viewport (until we mess with margins)
    /// # Ok(()) }
    /// ```
    pub fn max_width_all(&mut self, max_width: usize) -> Result<(), ColonnadeError> {
        for i in 0..self.len() {
            if self.colonnade[i].min_width.unwrap_or(0) > max_width {
                return Err(ColonnadeError::MinGreaterThanMax(i));
            }
            self.colonnade[i].max_width = Some(max_width);
        }
        Ok(())
    }
    /// Assign the same maximum width to all columns. By default columns have no maximum width.
    ///
    /// # Arguments
    ///
    /// * `max_width` - The common maximum width.
    ///
    /// # Errors
    ///
    /// * `ColonnadeError::MinGreaterThanMax` - Assigning a maximum width in conflict with some assigned minimum width.
    /// * `ColonnadeError::OutOfBounds` - Attemping to assign a maximum width to a column that does not exist.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::Colonnade;
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// // assign the first column a maximum width of 20
    /// colonnade.max_width(0, 20)?;
    /// # Ok(()) }
    /// ```
    pub fn max_width(&mut self, index: usize, max_width: usize) -> Result<(), ColonnadeError> {
        if index < self.len() {
            if self.colonnade[index].min_width.unwrap_or(max_width) > max_width {
                Err(ColonnadeError::MinGreaterThanMax(index))
            } else {
                self.colonnade[index].max_width = Some(max_width);
                Ok(())
            }
        } else {
            if self.colonnade[index].min_width.unwrap_or(0) > max_width {
                return Err(ColonnadeError::MinGreaterThanMax(index));
            }
            Err(ColonnadeError::OutOfBounds)
        }
    }
    /// Assign the same minimum width to all columns. By default columns have no minimum width.
    ///
    /// # Arguments
    ///
    /// * `min_width` - The common minimum width.
    ///
    /// # Errors
    ///
    /// * `ColonnadeError::MinGreaterThanMax` - Assigning a maximum width in conflict with some assigned minimum width.
    /// * `ColonnadeError::InsufficientSpace` - Assigning this minimum width means the columns require more space than the viewport provides.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::Colonnade;
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// // assign all columns a minimum width of 20
    /// colonnade.min_width_all(20)?;
    /// // we will now use at least 83 of the characters provided by the viewport (until we mess with margins)
    /// # Ok(()) }
    /// ```
    pub fn min_width_all(&mut self, min_width: usize) -> Result<(), ColonnadeError> {
        for i in 0..self.len() {
            if self.colonnade[i].max_width.unwrap_or(min_width) < min_width {
                return Err(ColonnadeError::MinGreaterThanMax(i));
            }
            self.colonnade[i].width = min_width;
            self.colonnade[i].min_width = Some(min_width);
        }
        if !self.sufficient_space() {
            Err(ColonnadeError::InsufficientSpace)
        } else {
            Ok(())
        }
    }
    /// Assign a particular minimum width to a particular column. By default columns have no minimum width.
    ///
    /// # Arguments
    ///
    /// * `index` - The column to which we wish to assign a minimum width.
    /// * `min_width` - The common minimum width.
    ///
    /// # Errors
    ///
    /// * `ColonnadeError::MinGreaterThanMax` - Assigning a maximum width in conflict with some assigned minimum width.
    /// * `ColonnadeError::InsufficientSpace` - Assigning this minimum width means the columns require more space than the viewport provides.
    /// * `ColonnadeError::OutOfBounds` - The index is outside the columns in the spec.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::Colonnade;
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// // assign the first column a minimum width of 20
    /// colonnade.min_width(0, 20)?;
    /// # Ok(()) }
    /// ```
    pub fn min_width(&mut self, index: usize, min_width: usize) -> Result<(), ColonnadeError> {
        if index < self.len() {
            if self.colonnade[index].max_width.unwrap_or(min_width) < min_width {
                return Err(ColonnadeError::MinGreaterThanMax(index));
            }
            self.colonnade[index].width = min_width;
            self.colonnade[index].min_width = Some(min_width);
            if !self.sufficient_space() {
                Err(ColonnadeError::InsufficientSpace)
            } else {
                Ok(())
            }
        } else {
            Err(ColonnadeError::OutOfBounds)
        }
    }
    /// Assign the same maximum and minimum width to all columns. By default columns have neither a maximum nor a minimum width.
    ///
    /// # Arguments
    ///
    /// * `width` - The common width.
    ///
    /// # Errors
    ///
    /// This method is a convenience method which assigns all columns the same maximum and minimum width. Therefore
    /// the errors thrown are those thrown by [`max_width_all`](#method.max_width_all) and [`min_width_all`](#method.min_width_all).
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::Colonnade;
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// // assign all columns a width of 20
    /// colonnade.fixed_width_all(20)?;
    /// // we will now use at exactly 83 of the characters provided by the viewport (until we mess with margins)
    /// # Ok(()) }
    /// ```
    pub fn fixed_width_all(&mut self, width: usize) -> Result<(), ColonnadeError> {
        for i in 0..self.len() {
            self.colonnade[i].min_width = None;
            self.colonnade[i].max_width = None;
        }
        match self.min_width_all(width) {
            Err(e) => return Err(e),
            Ok(_) => (),
        }
        match self.max_width_all(width) {
            Err(e) => return Err(e),
            Ok(_) => (),
        }
        Ok(())
    }
    /// Assign a particular maximum and minimum width to a particular column. By default columns have neither a maximum nor a minimum width.
    ///
    /// # Arguments
    ///
    /// * `index` - The column to configure.
    /// * `width` - The common width.
    ///
    /// # Errors
    ///
    /// This method is a convenience method which assigns the column in question the same maximum and minimum width. Therefore
    /// the errors thrown are those thrown by [`max_width`](#method.max_width) and [`min_width`](#method.min_width).
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::Colonnade;
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// // assign the first column a width of 20
    /// colonnade.fixed_width(0, 20)?;
    /// # Ok(()) }
    /// ```
    pub fn fixed_width(&mut self, index: usize, min_width: usize) -> Result<(), ColonnadeError> {
        self.colonnade[index].min_width = None;
        self.colonnade[index].max_width = None;
        match self.min_width(index, min_width) {
            Err(e) => return Err(e),
            Ok(_) => (),
        }
        match self.max_width(index, min_width) {
            Err(e) => return Err(e),
            Ok(_) => (),
        }
        Ok(())
    }
    /// Remove any maximum or minimum column widths.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::Colonnade;
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// // assign all columns a width of 20
    /// colonnade.fixed_width_all(20)?;
    /// // later ...
    /// colonnade.clear_limits_all();
    /// # Ok(()) }
    /// ```
    pub fn clear_limits_all(&mut self) {
        for i in 0..self.len() {
            self.colonnade[i].max_width = None;
            self.colonnade[i].min_width = None;
        }
    }
    /// Remove maximum or minimum column widths from a particular column.
    ///
    /// # Errors
    ///
    /// * `ColonnadeError::OutOfBounds` - The column specified does not exist in the spec.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::Colonnade;
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// // initially assign all columns a width of 20
    /// colonnade.fixed_width_all(20);
    /// // but we want the first column to be flexible
    /// colonnade.clear_limits(0)?;
    /// # Ok(()) }
    /// ```
    pub fn clear_limits(&mut self, index: usize) -> Result<(), ColonnadeError> {
        if index < self.len() {
            self.colonnade[index].max_width = None;
            self.colonnade[index].min_width = None;
            Ok(())
        } else {
            Err(ColonnadeError::OutOfBounds)
        }
    }
    /// Assign all columns the same alignment. The default alignment is left.
    ///
    /// # Arguments
    ///
    /// * `alignment` - The desired alignment.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::{Alignment,Colonnade};
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// // all columns should be right-aligned (they're numeric)
    /// colonnade.alignment_all(Alignment::Right);
    /// # Ok(()) }
    /// ```
    pub fn alignment_all(&mut self, alignment: Alignment) {
        for i in 0..self.len() {
            self.colonnade[i].alignment = alignment.clone();
        }
    }
    /// Assign a particular column a particular alignment. The default alignment is left.
    ///
    /// # Arguments
    ///
    /// * `index` - The column to modify.
    /// * `alignment` - The desired alignment.
    ///
    /// # Errors
    ///
    /// * `ColonnadeError::OutOfBounds` - The column specified does not exist in the spec.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::{Alignment,Colonnade};
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// // the first column should be right-aligned (it's numeric)
    /// colonnade.alignment(0, Alignment::Right)?;
    /// # Ok(()) }
    /// ```
    pub fn alignment(&mut self, index: usize, alignment: Alignment) -> Result<(), ColonnadeError> {
        if index < self.len() {
            self.colonnade[index].alignment = alignment;
            Ok(())
        } else {
            Err(ColonnadeError::OutOfBounds)
        }
    }
    /// Assign all columns the same left margin. The left margin is a number of blank spaces
    /// before the content of the column. By default the first column has a left margin of 0
    /// and the other columns have a left margin of 1.
    ///
    /// # Arguments
    ///
    /// * `left_margin` - The width in blank spaces of the desired margin.
    ///
    /// # Errors
    ///
    /// * `ColonnadeError::InsufficientSpace` - This margin will require more space than is available in the viewport.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::{Alignment,Colonnade};
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// // all columns should be preceded by 2 blank spaces
    /// colonnade.left_margin_all(2)?;
    /// # Ok(()) }
    /// ```
    pub fn left_margin_all(&mut self, left_margin: usize) -> Result<(), ColonnadeError> {
        for i in 0..self.len() {
            self.colonnade[i].left_margin = left_margin;
        }
        if !self.sufficient_space() {
            Err(ColonnadeError::InsufficientSpace)
        } else {
            Ok(())
        }
    }
    /// Assign a particular column a particular left margin. The left margin is a number of blank spaces
    /// before the content of the column. By default the first column has a left margin of 0
    /// and the other columns have a left margin of 1.
    ///
    /// # Arguments
    ///
    /// * `index` - The column to configure.
    /// * `left_margin` - The width in blank spaces of the desired margin.
    ///
    /// # Errors
    ///
    /// * `ColonnadeError::InsufficientSpace` - This margin will require more space than is available in the viewport.
    /// * `ColonnadeError::OutOfBounds` - The column specified does not exist in the spec.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::{Alignment,Colonnade};
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// // the first column should be preceded by 2 blank spaces
    /// colonnade.left_margin(0, 2)?;
    /// # Ok(()) }
    /// ```
    pub fn left_margin(&mut self, index: usize, left_margin: usize) -> Result<(), ColonnadeError> {
        if index < self.len() {
            self.colonnade[index].left_margin = left_margin;
            if !self.sufficient_space() {
                Err(ColonnadeError::InsufficientSpace)
            } else {
                Ok(())
            }
        } else {
            Err(ColonnadeError::OutOfBounds)
        }
    }
    pub fn padding_all(&mut self, padding: usize) -> Result<(), ColonnadeError> {
        for i in 0..self.len() {
            self.colonnade[i].padding_left = padding;
            self.colonnade[i].padding_right = padding;
            self.colonnade[i].padding_top = padding;
            self.colonnade[i].padding_bottom = padding;
        }
        if !self.sufficient_space() {
            Err(ColonnadeError::InsufficientSpace)
        } else {
            Ok(())
        }
    }
    pub fn padding(&mut self, index: usize, padding: usize) -> Result<(), ColonnadeError> {
        self.colonnade[index].padding_left = padding;
        self.colonnade[index].padding_right = padding;
        self.colonnade[index].padding_top = padding;
        self.colonnade[index].padding_bottom = padding;
        if !self.sufficient_space() {
            Err(ColonnadeError::InsufficientSpace)
        } else {
            Ok(())
        }
    }
    pub fn padding_horizontal_all(&mut self, padding: usize) -> Result<(), ColonnadeError> {
        for i in 0..self.len() {
            self.colonnade[i].padding_left = padding;
            self.colonnade[i].padding_right = padding;
        }
        if !self.sufficient_space() {
            Err(ColonnadeError::InsufficientSpace)
        } else {
            Ok(())
        }
    }
    pub fn padding_horizontal(
        &mut self,
        index: usize,
        padding: usize,
    ) -> Result<(), ColonnadeError> {
        self.colonnade[index].padding_left = padding;
        self.colonnade[index].padding_right = padding;
        if !self.sufficient_space() {
            Err(ColonnadeError::InsufficientSpace)
        } else {
            Ok(())
        }
    }
    pub fn padding_left_all(&mut self, padding: usize) -> Result<(), ColonnadeError> {
        for i in 0..self.len() {
            self.colonnade[i].padding_left = padding;
        }
        if !self.sufficient_space() {
            Err(ColonnadeError::InsufficientSpace)
        } else {
            Ok(())
        }
    }
    pub fn padding_left(&mut self, index: usize, padding: usize) -> Result<(), ColonnadeError> {
        self.colonnade[index].padding_left = padding;
        if !self.sufficient_space() {
            Err(ColonnadeError::InsufficientSpace)
        } else {
            Ok(())
        }
    }
    pub fn padding_right_all(&mut self, padding: usize) -> Result<(), ColonnadeError> {
        for i in 0..self.len() {
            self.colonnade[i].padding_right = padding;
        }
        if !self.sufficient_space() {
            Err(ColonnadeError::InsufficientSpace)
        } else {
            Ok(())
        }
    }
    pub fn padding_right(&mut self, index: usize, padding: usize) -> Result<(), ColonnadeError> {
        self.colonnade[index].padding_right = padding;
        if !self.sufficient_space() {
            Err(ColonnadeError::InsufficientSpace)
        } else {
            Ok(())
        }
    }
    pub fn padding_vertical_all(&mut self, padding: usize) {
        for i in 0..self.len() {
            self.colonnade[i].padding_top = padding;
            self.colonnade[i].padding_bottom = padding;
        }
    }
    pub fn padding_vertical(&mut self, index: usize, padding: usize) {
        self.colonnade[index].padding_top = padding;
        self.colonnade[index].padding_bottom = padding;
    }
    pub fn padding_top_all(&mut self, padding: usize) {
        for i in 0..self.len() {
            self.colonnade[i].padding_top = padding;
        }
    }
    pub fn padding_top(&mut self, index: usize, padding: usize) {
        self.colonnade[index].padding_top = padding;
    }
    pub fn padding_bottom_all(&mut self, padding: usize) {
        for i in 0..self.len() {
            self.colonnade[i].padding_bottom = padding;
        }
    }
    pub fn padding_bottom(&mut self, index: usize, padding: usize) {
        self.colonnade[index].padding_bottom = padding;
    }
}
