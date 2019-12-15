/*!

This crate provides a library for displaying tabular data in, for instance, a command
line app.

# Usage

Colonnade is [on crates.io](https://crates.io/crates/colonnade) and can be
used by adding `colonnade` to your dependencies in your project's `Cargo.toml`.

```toml
[dependencies]
colonnade = "1"
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
            "If you want to colorize your table, you'll need to use the macerate method.",
        ],
        vec!["", "Two or more rows of columns makes a table.", ""],
    ];

    // 3 columns of text in an 80-character viewport
    let mut colonnade = Colonnade::new(3, 80).unwrap();

    // configure the table a bit
    colonnade.left_margin(4);
    colonnade.columns[0].left_margin(8);      // the first column should have a left margin 8 spaces wide
    colonnade.fixed_width(15);                // first we set all the columns to 15 characters wide
    colonnade.columns[1].clear_limits();      // but then remove this restriction on the central column
    colonnade.columns[0].alignment(Alignment::Right);
    colonnade.columns[1].alignment(Alignment::Center);
    colonnade.spaces_between_rows(1);         // add a blank link between rows

    // now print out the table
    for line in colonnade.tabulate(&text).unwrap() {
        println!("{}", line);
    }
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
*/
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

impl std::fmt::Display for ColonnadeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ColonnadeError {}

/// Alignments left-to-right one can apply to columns of text.
#[derive(Debug, Clone)]
pub enum Alignment {
    /// Left justification -- the default alignment
    Left,
    /// Right justification
    Right,
    /// Centering
    Center,
}

/// Vertical alignments of text within a column.
#[derive(Debug, Clone, PartialEq)]
pub enum VerticalAlignment {
    /// the default vertical alignment
    Top,
    Middle,
    Bottom,
}

/// A struct holding formatting information for a particular column.
#[derive(Debug, Clone)]
pub struct Column {
    index: usize,
    alignment: Alignment,
    vertical_alignment: VerticalAlignment,
    left_margin: usize,
    /// the width of the column excluding any left margin
    pub width: usize,
    priority: usize,
    min_width: Option<usize>,
    max_width: Option<usize>,
    padding_left: usize,
    padding_right: usize,
    padding_top: usize,
    padding_bottom: usize,
    hyphenate: bool,
    adjusted: bool,
}

impl Column {
    fn default(index: usize) -> Column {
        Column {
            index: index,
            alignment: Alignment::Left,
            vertical_alignment: VerticalAlignment::Top,
            left_margin: 1,
            width: 0, // claimed width
            priority: usize::max_value(),
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
    fn horizontal_padding(&self) -> usize {
        self.padding_left + self.padding_right
    }
    fn vertical_padding(&self) -> usize {
        self.padding_top + self.padding_bottom
    }
    fn minimum_width(&self) -> usize {
        let w1 = self.horizontal_padding();
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
    fn inner_width(&self) -> usize {
        self.width - self.padding_right
    }
    fn hyphenating(&self) -> bool {
        self.hyphenate && self.inner_width() > 1
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
    /// Assign a particular priority to the column.
    ///
    /// Priority determines the order in which columns give up space when the viewport lacks sufficient
    /// space to display all columns without wrapping. Lower priority columns give up space first.
    ///
    /// # Arguments
    ///
    /// * `priority` - The column's priority. Lower numbers confer higher priority; 0 is the highest priority.
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
    /// colonnade.priority(0);
    /// // now demote the last column
    /// colonnade.columns[3].priority(1);
    /// # Ok(()) }
    /// ```
    pub fn priority(&mut self, priority: usize) -> &mut Self {
        self.adjusted = false;
        self.priority = priority;
        self
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
    /// colonnade.columns[0].max_width(20)?;
    /// # Ok(()) }
    /// ```
    pub fn max_width(&mut self, max_width: usize) -> Result<&mut Self, ColonnadeError> {
        if self.min_width.unwrap_or(max_width) > max_width {
            Err(ColonnadeError::MinGreaterThanMax(self.index))
        } else {
            self.max_width = Some(max_width);
            self.adjusted = false;
            Ok(self)
        }
    }
    /// Assign a particular minimum width to a particular column. By default columns have no minimum width.
    ///
    /// # Arguments
    ///
    /// * `min_width` - The common minimum width.
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
    /// // assign the first column a minimum width of 20
    /// colonnade.columns[0].min_width(20)?;
    /// # Ok(()) }
    /// ```
    pub fn min_width(&mut self, min_width: usize) -> Result<&mut Self, ColonnadeError> {
        if self.max_width.unwrap_or(min_width) < min_width {
            return Err(ColonnadeError::MinGreaterThanMax(self.index));
        }
        self.width = min_width;
        self.min_width = Some(min_width);
        self.adjusted = false;
        Ok(self)
    }
    /// Assign a particular maximum and minimum width to a particular column. By default columns have neither a maximum nor a minimum width.
    ///
    /// # Arguments
    ///
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
    /// colonnade.columns[0].fixed_width(20)?;
    /// # Ok(()) }
    /// ```
    pub fn fixed_width(&mut self, width: usize) -> Result<&mut Self, ColonnadeError> {
        self.min_width = None;
        self.max_width = None;
        match self.min_width(width) {
            Err(e) => return Err(e),
            Ok(_) => (),
        }
        match self.max_width(width) {
            Err(e) => return Err(e),
            Ok(_) => (),
        }
        Ok(self)
    }
    /// Remove maximum or minimum column widths from a particular column.
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
    /// colonnade.fixed_width(20);
    /// // but we want the first column to be flexible
    /// colonnade.columns[0].clear_limits();
    /// # Ok(()) }
    /// ```
    pub fn clear_limits(&mut self) -> &mut Self {
        self.max_width = None;
        self.min_width = None;
        self.adjusted = false;
        self
    }
    /// Assign a particular column a particular alignment. The default alignment is left.
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
    /// // the first column should be right-aligned (it's numeric)
    /// colonnade.columns[0].alignment(Alignment::Right);
    /// # Ok(()) }
    /// ```
    pub fn alignment(&mut self, alignment: Alignment) -> &mut Self {
        self.alignment = alignment;
        self
    }
    /// Assign a particular column a particular vertical alignment. The default alignment is top.
    ///
    /// # Arguments
    ///
    /// * `vertical_alignment` - The desired alignment.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::{Alignment,VerticalAlignment,Colonnade};
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// // the first column should be right-aligned (it's numeric)
    /// colonnade.columns[0].vertical_alignment(VerticalAlignment::Middle);
    /// # Ok(()) }
    /// ```
    pub fn vertical_alignment(&mut self, vertical_alignment: VerticalAlignment) -> &mut Self {
        self.vertical_alignment = vertical_alignment;
        self
    }
    /// Assign a particular column a particular left margin. The left margin is a number of blank spaces
    /// before the content of the column. By default the first column has a left margin of 0
    /// and the other columns have a left margin of 1.
    ///
    /// # Arguments
    ///
    /// * `left_margin` - The width in blank spaces of the desired margin.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::{Alignment,Colonnade};
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// colonnade.columns[0].left_margin(2);
    /// # Ok(()) }
    /// ```
    pub fn left_margin(&mut self, left_margin: usize) -> &mut Self {
        self.left_margin = left_margin;
        self.adjusted = false;
        self
    }
    /// Assign a particular column a particular padding.
    ///
    /// See [`Colonnade::padding`](struct.Colonade.html#method.padding).
    ///
    /// # Arguments
    ///
    /// * `padding` - The width in blank spaces/lines of the desired padding.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::{Alignment,Colonnade};
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// colonnade.columns[0].padding(1);
    /// # Ok(()) }
    /// ```
    pub fn padding(&mut self, padding: usize) -> &mut Self {
        self.padding_left = padding;
        self.padding_right = padding;
        self.padding_top = padding;
        self.padding_bottom = padding;
        self.adjusted = false;
        self
    }
    /// Assign a particular column a particular horizontal padding -- space before and after the column's text.
    ///
    /// See [`Colonnade::padding`](struct.Colonade.html#method.padding).
    ///
    /// # Arguments
    ///
    /// * `padding` - The width in blank spaces/lines of the desired padding.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::{Alignment,Colonnade};
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// colonnade.columns[0].padding_horizontal(1);
    /// # Ok(()) }
    /// ```
    pub fn padding_horizontal(&mut self, padding: usize) -> &mut Self {
        self.padding_left = padding;
        self.padding_right = padding;
        self.adjusted = false;
        self
    }
    /// Assign a particular column a particular left padding -- space before the column's text.
    ///
    /// See [`Colonnade::padding`](struct.Colonade.html#method.padding).
    ///
    /// # Arguments
    ///
    /// * `padding` - The width in blank spaces/lines of the desired padding.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::{Alignment,Colonnade};
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// colonnade.columns[0].padding_left(1);
    /// # Ok(()) }
    /// ```
    pub fn padding_left(&mut self, padding: usize) -> &mut Self {
        self.padding_left = padding;
        self.adjusted = false;
        self
    }
    /// Assign a particular column a particular right padding -- space after the column's text.
    ///
    /// See [`Colonnade::padding`](struct.Colonade.html#method.padding).
    ///
    /// # Arguments
    ///
    /// * `padding` - The width in blank spaces/lines of the desired padding.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::{Alignment,Colonnade};
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// colonnade.columns[0].padding_right(1);
    /// # Ok(()) }
    /// ```
    pub fn padding_right(&mut self, padding: usize) -> &mut Self {
        self.padding_right = padding;
        self.adjusted = false;
        self
    }
    /// Assign a particular column a particular vertical padding -- blank lines before and after the column's text.
    ///
    /// See [`Colonnade::padding`](struct.Colonade.html#method.padding).
    ///
    /// # Arguments
    ///
    /// * `padding` - The width in blank spaces/lines of the desired padding.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::{Alignment,Colonnade};
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// colonnade.columns[0].padding_vertical(1);
    /// # Ok(()) }
    /// ```
    pub fn padding_vertical(&mut self, padding: usize) -> &mut Self {
        self.padding_top = padding;
        self.padding_bottom = padding;
        self
    }
    /// Assign a particular column a particular top padding -- blank lines before the column's text.
    ///
    /// See [`Colonnade::padding`](struct.Colonade.html#method.padding).
    ///
    /// # Arguments
    ///
    /// * `padding` - The width in blank spaces/lines of the desired padding.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::{Alignment,Colonnade};
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// colonnade.columns[0].padding_top(1);
    /// # Ok(()) }
    /// ```
    pub fn padding_top(&mut self, padding: usize) -> &mut Self {
        self.padding_top = padding;
        self
    }
    /// Assign a particular column a particular bottom padding -- blank lines after the column's text.
    ///
    /// See [`Colonnade::padding`](struct.Colonade.html#method.padding).
    ///
    /// # Arguments
    ///
    /// * `padding` - The width in blank spaces/lines of the desired padding.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::{Alignment,Colonnade};
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// colonnade.columns[0].padding_bottom(1);
    /// # Ok(()) }
    /// ```
    pub fn padding_bottom(&mut self, padding: usize) -> &mut Self {
        self.padding_bottom = padding;
        self
    }
    /// Toggle whether words too wide to fit in the column are hyphenated when spit. By
    /// default this is `true`. If there is only 1 character of available space in a column,
    /// though, there is never any hyphenation.
    ///
    /// # Arguments
    ///
    /// * `hyphenate` - Whether to hyphenate when splitting words.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::{Alignment,Colonnade};
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(1, 3)?;
    /// colonnade.alignment(Alignment::Right);
    /// for line in colonnade.tabulate(&[[1234]])? {
    ///     println!("{}", line);
    /// }
    /// // 12-
    /// //  34
    /// colonnade.columns[0].hyphenate(false);
    /// for line in colonnade.tabulate(&[[1234]])? {
    ///     println!("{}", line);
    /// }
    /// // 123
    /// //   4
    /// # Ok(()) }
    /// ```
    pub fn hyphenate(&mut self, hyphenate: bool) -> &mut Self {
        self.hyphenate = hyphenate;
        self
    }
}

/// A struct holding formatting information. This is the object which tabulates data.
#[derive(Debug, Clone)]
pub struct Colonnade {
    pub columns: Vec<Column>,
    width: usize,
    spaces_between_rows: usize,
}

// find the longest sequence of non-whitespace characters in a string
fn longest_word(s: &str) -> usize {
    s.split_whitespace().fold(0, |acc, v| {
        let c = v.chars().count();
        if c > acc {
            c
        } else {
            acc
        }
    })
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
        let mut columns: Vec<Column> = (0..columns).map(|i| Column::default(i)).collect();
        columns[0].left_margin = 0;
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
            .fold(0, |acc, v| acc + v.left_margin + v.min_width.unwrap_or(1)) // assume each column requires at least one character
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
        let mut p = 0;
        for c in &self.columns {
            let p2 = c.vertical_padding();
            if p2 > p {
                p = p2;
            }
        }
        p
    }
    fn len(&self) -> usize {
        self.columns.len()
    }
    // determine the characters required to represent s after whitespace normalization
    fn width_after_normalization(s: &str) -> usize {
        let mut l = 0;
        for w in s.trim().split_whitespace() {
            if l != 0 {
                l += 1;
            }
            l += w.chars().count();
        }
        l
    }
    /// Returns the width of the colonnade in columns if the colonnade has already laid out data
    /// and knows how much space this data will require.
    pub fn width(&self) -> Option<usize> {
        if self.adjusted() {
            Some(self.width)
        } else {
            None
        }
    }
    // returns priorites sorted lowest to highest
    fn priorities(&self) -> Vec<usize> {
        let mut v = self.columns.iter().map(|c| c.priority).collect::<Vec<_>>();
        v.sort_unstable();
        v.dedup();
        v.reverse();
        v
    }
    /// Converts the raw data in `table` into a vector of strings representing the data in tabular form.
    /// Blank lines will be zero-width rather than full-width lines of whitespace.
    ///
    /// If you need finer control over the text, for instance, if you want to add color codes, see
    /// [`macerate`](#method.macerate).
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
    pub fn tabulate<T, U, V, W, X>(&mut self, table: T) -> Result<Vec<String>, ColonnadeError>
    where
        T: IntoIterator<Item = U, IntoIter = V>,
        U: IntoIterator<Item = W, IntoIter = X>,
        V: Iterator<Item = U>,
        W: ToString,
        X: Iterator<Item = W>,
    {
        self.macerate(table)
            .and_then(|buffer| Ok(Colonnade::reconstitute_rows(buffer)))
    }
    /// Chew up the text into bits suitable for piecemeal layout.
    ///
    /// More specifically, `macerate` digests the raw data in `table` into a vector of vectors of `(String, String)` tuples
    /// representing the data in tabular form. Each tuple consists of a whitespace left margin and
    /// the contents of a column. Separator lines will consist of a margin and text tuple where the
    /// text is zero-width and the "margin" is as wide as the table.
    ///
    /// Maceration is useful if you wish to insert color codes to colorize the data or otherwise
    /// manipulate the data post-layout. If you don't want to do this, see [`tabulate`](#method.tabulate).
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
    /// extern crate term;
    /// // ... [some details omitted]
    /// # extern crate colonnade;
    /// # use colonnade::{Alignment, Colonnade};
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// // text to put in tabular form
    /// let text = vec![
    ///     vec![
    ///         "Colonnade lets you format text in columns.",
    ///         "As you can see, it supports text alignment, viewport width, and column widths.",
    ///         "It doesn't natively support color codes, but it is easy enough to combine with a crate like term.",
    ///     ],
    ///     vec!["", "Two or more rows of columns makes a table.", ""],
    /// ];
    /// let mut colonnade = Colonnade::new(3, 80)?;
    ///
    /// // configure the table a bit
    /// colonnade.spaces_between_rows(1).left_margin(4)?.fixed_width(15)?;
    /// colonnade.columns[0].alignment(Alignment::Right).left_margin(8);
    /// colonnade.columns[1].alignment(Alignment::Center).clear_limits();
    /// // if the text is in colored cells, you will probably want some padding
    /// colonnade.padding(1)?;
    /// ///
    /// // now print out the table
    /// let mut t = term::stdout().unwrap();
    /// for row in colonnade.macerate(&text)? {
    ///     for line in row {
    ///         for (i, (margin, text)) in line.iter().enumerate() {
    ///             write!(t, "{}", margin)?;
    ///             let background_color = if i % 2 == 0 {
    ///                 term::color::WHITE
    ///             } else {
    ///                 term::color::BLACK
    ///             };
    ///             let foreground_color = match i % 3 {
    ///                 1 => term::color::GREEN,
    ///                 2 => term::color::RED,
    ///                 _ => term::color::BLUE,
    ///             };
    ///             t.bg(background_color)?;
    ///             t.fg(foreground_color)?;
    ///             write!(t, "{}", text)?;
    ///             t.reset()?;
    ///         }
    ///         println!();
    ///     }
    /// }
    /// # Ok(()) }
    /// ```
    pub fn macerate<T, U, V, W, X>(
        &mut self,
        table: T,
    ) -> Result<Vec<Vec<Vec<(String, String)>>>, ColonnadeError>
    where
        T: IntoIterator<Item = U, IntoIter = V>,
        U: IntoIterator<Item = W, IntoIter = X>,
        V: Iterator<Item = U>,
        W: ToString,
        X: Iterator<Item = W>,
    {
        self.lay_out(table).and_then(|owned_table| {
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
                    .map(|t| t.to_string())
                    .collect::<Vec<String>>()
            })
            .collect::<Vec<Vec<String>>>();
        // pad rows as necessary
        for i in 0..table.len() {
            while table[i].len() < self.len() {
                table[i].push(String::new());
            }
        }
        table
    }
    // utility function to convert a String table to a &str table
    fn ref_table(table: &Vec<Vec<String>>) -> Vec<Vec<&str>> {
        table
            .iter()
            .map(|v| v.iter().map(|s| s.as_ref()).collect::<Vec<&str>>())
            .collect::<Vec<Vec<&str>>>()
    }
    fn reconstitute_rows(maceration: Vec<Vec<Vec<(String, String)>>>) -> Vec<String> {
        maceration
            .iter()
            .flat_map(|row| {
                row.iter().map(|line| {
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
            })
            .collect()
    }
    // take one row of untabulated pieces of text and turn it into one or more vectors of (String,String) tuples,
    // where each tuple represenst a left margin and some column text, the each vector representing one line of tabulated text
    // these vectors are gathered into a vector and added to the buffer
    fn add_row(
        &self,
        buffer: &mut Vec<Vec<Vec<(String, String)>>>,
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
                    self.columns[i].padding_top,
                    w.trim().split_whitespace().collect(),
                    self.columns[i].padding_bottom,
                )
            })
            .collect();
        let mut current_lines: Vec<Vec<(String, String)>> = Vec::new();
        // if all these lists are empty, just add a blank line (and maybe additional blank separator lines)
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
            // otherwise, we build these lists into lines, we may use up some of these lists before others
            while !words
                .iter()
                .all(|(pt, sentence, pb)| pb == &0 && pt == &0 && sentence.is_empty())
            {
                let mut pieces = vec![];
                for (i, c) in self.columns.iter().enumerate() {
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
                            let w = tuple.1.remove(0); // shift off the next word
                            if first {
                                let wl = w.chars().count() + c.padding_right;
                                if wl == c.width {
                                    // word fills column
                                    phrase += w;
                                    break;
                                } else if wl > c.width {
                                    // word overflows column and we must split it
                                    let hyphenating = c.hyphenating();
                                    let mut offset = c.inner_width();
                                    if hyphenating {
                                        offset -= 1;
                                    }
                                    let mut byte_offset = 0;
                                    for c in w.chars().take(offset) {
                                        byte_offset += c.len_utf8();
                                    }
                                    phrase += &w[0..byte_offset];
                                    tuple.1.insert(0, &w[byte_offset..w.len()]); // unshift back the remaining fragment
                                    if hyphenating {
                                        phrase += "-";
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
                            let surplus = c.width - phrase.chars().count();
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
                current_lines.push(pieces);
            }
            // now fix vertical alignment
            'outer: for c in self.columns.iter() {
                match c.vertical_alignment {
                    VerticalAlignment::Top => (),
                    _ => {
                        let blank = c.blank_line();
                        let end = current_lines.len() - c.padding_bottom;
                        let mut movable_lines = 0;
                        let mut pointer = end - 1;
                        let top_pointer = c.padding_top;
                        while current_lines[pointer][c.index].1 == blank {
                            movable_lines += 1;
                            if pointer == top_pointer {
                                // this cell contains nothing but blank lines so alignment is irrelevant
                                continue 'outer;
                            }
                            pointer -= 1;
                        }
                        if movable_lines == 0 {
                            continue 'outer;
                        }
                        // pointer now points to the last movable line
                        // top_pointer points to the insertion index where we can put blank lines
                        // end points to an immovable index (perhaps beyond the end of the vector)
                        let lines_to_move = if c.vertical_alignment == VerticalAlignment::Middle {
                            movable_lines / 2
                        } else {
                            movable_lines
                        };
                        // we extract the tuples for the relevant column from top_pointer to end, rotate
                        // them lines_to_move times, and reinstall them
                        let mut rotator = Vec::with_capacity(end - top_pointer);
                        for i in top_pointer..end {
                            rotator.push(current_lines[i].remove(c.index));
                        }
                        for _ in 0..lines_to_move {
                            let pair = rotator.remove(rotator.len() - 1);
                            rotator.insert(0, pair);
                        }
                        for i in top_pointer..end {
                            current_lines[i].insert(c.index, rotator.remove(0));
                        }
                    }
                }
            }
            // add row-separating lines
            if !last_row {
                for _ in 0..self.spaces_between_rows {
                    current_lines.push(vec![(self.blank_line(), String::new())]);
                }
            }
        }
        buffer.push(current_lines);
    }
    /// Erase column widths established by a previous `tabulate` or `macerate`.
    /// 
    /// Note that adjusting any configuration that may affect the horizontal layout of data
    /// has an equivalent effect, forcing a fresh layout of the columns.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::{Alignment, Colonnade};
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(3, 80)?;
    /// colonnade.alignment(Alignment::Right);
    /// for line in colonnade.tabulate(&[[100, 200, 300]])? {
    ///     println!("{}", line);
    /// }
    /// // 100 200 300
    /// for line in colonnade.tabulate(&[[1, 2, 3]])? {
    ///     println!("{}", line);
    /// }
    /// //   1   2   3
    /// colonnade.reset();
    /// for line in colonnade.tabulate(&[[1, 2, 3]])? {
    ///     println!("{}", line);
    /// }
    /// // 1 2 3
    /// # Ok(()) }
    /// ```
    pub fn reset(&mut self) {
        for i in 0..self.len() {
            self.columns[i].adjusted = false;
            self.columns[i].width = 0;
        }
    }
    fn adjusted(&self) -> bool {
        self.columns.iter().all(|c| c.adjusted)
    }
    // determine the optimal widths of the columns given the data and the specified constraints
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
        if !self.sufficient_space() {
            return Err(ColonnadeError::InsufficientSpace);
        }
        // first try to do it all without splitting
        for i in 0..table.len() {
            for c in 0..self.len() {
                let m = Colonnade::width_after_normalization(&table[i][c])
                    + self.columns[c].horizontal_padding();
                if m >= self.columns[c].width {
                    // to force initial expansion to min width
                    self.columns[c].expand(m);
                }
            }
        }
        if self.required_width() <= self.width {
            self.mark_adjusted();
            return Ok(owned_table);
        }
        let mut modified_columns: Vec<usize> = Vec::with_capacity(self.len());
        // try shrinking columns to their longest word by order of priority
        for p in self.priorities() {
            for c in 0..self.len() {
                if self.columns[c].priority == p && self.columns[c].is_shrinkable() {
                    modified_columns.push(c);
                    self.columns[c].shrink(0);
                    for r in 0..table.len() {
                        let m = longest_word(&table[r][c]) + self.columns[c].horizontal_padding();
                        if m > self.columns[c].width {
                            self.columns[c].expand(m);
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
            let mut truncatable_columns = self.columns.iter().enumerate().collect::<Vec<_>>();
            truncatable_columns.retain(|(_, c)| c.is_shrinkable());
            let truncatable_columns: Vec<usize> =
                truncatable_columns.iter().map(|(i, _)| *i).collect();
            let mut priorities: Vec<usize> = truncatable_columns
                .iter()
                .map(|&i| self.columns[i].priority)
                .collect();
            priorities.sort_unstable();
            priorities.dedup();
            priorities.reverse();
            'outer: for p in priorities {
                let mut shrinkables: Vec<&usize> = truncatable_columns
                    .iter()
                    .filter(|&&i| self.columns[i].priority == p)
                    .collect();
                loop {
                    let excess = self.required_width() - self.width;
                    if excess == 0 {
                        break 'outer;
                    }
                    if excess <= shrinkables.len() {
                        shrinkables.retain(|&&i| self.columns[i].shrink_by(1));
                    } else {
                        let share = excess / shrinkables.len();
                        shrinkables.retain(|&&i| self.columns[i].shrink_by(share));
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
            modified_columns.retain(|&i| self.columns[i].is_expandable());
            if !modified_columns.is_empty() {
                while self.required_width() < self.width {
                    // find highest priority among modified columns
                    if let Some(priority) = modified_columns
                        .iter()
                        .map(|&i| self.columns[i].priority)
                        .min()
                    {
                        // there are still some modified columns we haven't restored any space to
                        let mut winners: Vec<&usize> = modified_columns
                            .iter()
                            .filter(|&&i| self.columns[i].priority == priority)
                            .collect();
                        let surplus = self.width - self.required_width();
                        if surplus <= winners.len() {
                            // give one column back to as many of the winners as possible and call it a day
                            // we will necessarily break out of the loop after this
                            for &&i in winners.iter().take(surplus) {
                                self.columns[i].width += 1;
                            }
                        } else {
                            // give a share back to each winner
                            loop {
                                let surplus = self.width - self.required_width();
                                if surplus == 0 {
                                    break;
                                }
                                winners.retain(|&&i| self.columns[i].is_expandable());
                                if winners.is_empty() {
                                    break;
                                }
                                if surplus <= winners.len() {
                                    for &&i in winners.iter().take(surplus) {
                                        self.columns[i].width += 1;
                                    }
                                    break;
                                }
                                let mut changed = false;
                                let share = surplus / winners.len();
                                for &&i in winners.iter() {
                                    let change = self.columns[i].expand_by(share);
                                    changed = changed || change;
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
    fn mark_adjusted(&mut self) {
        for i in 0..self.len() {
            self.columns[i].adjusted = true;
        }
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
    pub fn spaces_between_rows(&mut self, n: usize) -> &mut Self {
        self.spaces_between_rows = n;
        self
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
    /// colonnade.priority(0);
    /// // now demote the last column
    /// colonnade.columns[3].priority(1);
    /// # Ok(()) }
    /// ```
    pub fn priority(&mut self, priority: usize) -> &mut Self {
        for i in 0..self.len() {
            self.columns[i].priority = priority;
        }
        self
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
    /// colonnade.max_width(20)?;
    /// // at most we will now use only 83 of the characters provided by the viewport (until we mess with margins)
    /// # Ok(()) }
    /// ```
    pub fn max_width(&mut self, max_width: usize) -> Result<&mut Self, ColonnadeError> {
        for i in 0..self.len() {
            match self.columns[i].max_width(max_width) {
                Err(e) => return Err(e),
                Ok(_) => (),
            }
        }
        Ok(self)
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
    /// colonnade.min_width(20)?;
    /// // we will now use at least 83 of the characters provided by the viewport (until we mess with margins)
    /// # Ok(()) }
    /// ```
    pub fn min_width(&mut self, min_width: usize) -> Result<&mut Self, ColonnadeError> {
        for i in 0..self.len() {
            match self.columns[i].min_width(min_width) {
                Err(e) => return Err(e),
                Ok(_) => (),
            }
        }
        if !self.sufficient_space() {
            Err(ColonnadeError::InsufficientSpace)
        } else {
            Ok(self)
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
    /// // assign all columns a width of 20
    /// colonnade.fixed_width(20)?;
    /// // we will now use at exactly 83 of the characters provided by the viewport (until we mess with margins)
    /// # Ok(()) }
    /// ```
    pub fn fixed_width(&mut self, width: usize) -> Result<&mut Self, ColonnadeError> {
        for i in 0..self.len() {
            match self.columns[i].fixed_width(width) {
                Err(e) => return Err(e),
                Ok(_) => (),
            }
        }
        Ok(self)
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
    /// colonnade.fixed_width(20)?;
    /// // later ...
    /// colonnade.clear_limits();
    /// # Ok(()) }
    /// ```
    pub fn clear_limits(&mut self) -> &mut Self {
        for i in 0..self.len() {
            self.columns[i].clear_limits();
        }
        self
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
    /// colonnade.alignment(Alignment::Right);
    /// # Ok(()) }
    /// ```
    pub fn alignment(&mut self, alignment: Alignment) -> &mut Self {
        for i in 0..self.len() {
            self.columns[i].alignment(alignment.clone());
        }
        self
    }
    /// Assign all columns the same vertical alignment. The default alignment is top.
    ///
    /// # Arguments
    ///
    /// * `vertical_alignment` - The desired alignment.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::{Alignment, VerticalAlignment, Colonnade};
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// // all columns should be right-aligned (they're numeric)
    /// colonnade.vertical_alignment(VerticalAlignment::Middle);
    /// # Ok(()) }
    /// ```
    pub fn vertical_alignment(&mut self, vertical_alignment: VerticalAlignment) -> &mut Self {
        for i in 0..self.len() {
            self.columns[i].vertical_alignment(vertical_alignment.clone());
        }
        self
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
    /// colonnade.left_margin(2)?;
    /// # Ok(()) }
    /// ```
    pub fn left_margin(&mut self, left_margin: usize) -> Result<&mut Self, ColonnadeError> {
        for i in 0..self.len() {
            self.columns[i].left_margin(left_margin);
        }
        if !self.sufficient_space() {
            Err(ColonnadeError::InsufficientSpace)
        } else {
            Ok(self)
        }
    }
    /// Assign all columns the same padding. The padding is a number of blank spaces
    /// before and after the contents of the column and a number of blank lines above and below
    /// it. By default the padding is 0. You most likely don't want any padding unless you are
    /// colorizing the text -- text immediately after color transitions is more difficult to read
    /// and less aesthetically pleasing.
    ///
    /// # Arguments
    ///
    /// * `padding` - The width in blank spaces/lines of the desired padding.
    ///
    /// # Errors
    ///
    /// * `ColonnadeError::InsufficientSpace` - This padding will require more space than is available in the viewport.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::{Alignment,Colonnade};
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// colonnade.padding(1)?;
    /// # Ok(()) }
    /// ```
    pub fn padding(&mut self, padding: usize) -> Result<&mut Self, ColonnadeError> {
        for i in 0..self.len() {
            self.columns[i].padding(padding);
        }
        if !self.sufficient_space() {
            Err(ColonnadeError::InsufficientSpace)
        } else {
            Ok(self)
        }
    }
    /// Assign all columns the same horizontal padding -- space before and after the column's text.
    ///
    /// See [`padding`](#method.padding).
    ///
    /// # Arguments
    ///
    /// * `padding` - The width in blank spaces/lines of the desired padding.
    ///
    /// # Errors
    ///
    /// * `ColonnadeError::InsufficientSpace` - This padding will require more space than is available in the viewport.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::{Alignment,Colonnade};
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// colonnade.padding_horizontal(1)?;
    /// # Ok(()) }
    /// ```
    pub fn padding_horizontal(&mut self, padding: usize) -> Result<&mut Self, ColonnadeError> {
        for i in 0..self.len() {
            self.columns[i].padding_horizontal(padding);
        }
        if !self.sufficient_space() {
            Err(ColonnadeError::InsufficientSpace)
        } else {
            Ok(self)
        }
    }
    /// Assign all columns the same left padding -- space before the column's text.
    ///
    /// See [`padding`](#method.padding).
    ///
    /// # Arguments
    ///
    /// * `padding` - The width in blank spaces/lines of the desired padding.
    ///
    /// # Errors
    ///
    /// * `ColonnadeError::InsufficientSpace` - This padding will require more space than is available in the viewport.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::{Alignment,Colonnade};
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// colonnade.padding_left(1)?;
    /// # Ok(()) }
    /// ```
    pub fn padding_left(&mut self, padding: usize) -> Result<&mut Self, ColonnadeError> {
        for i in 0..self.len() {
            self.columns[i].padding_left(padding);
        }
        if !self.sufficient_space() {
            Err(ColonnadeError::InsufficientSpace)
        } else {
            Ok(self)
        }
    }
    /// Assign all columns the same right padding -- space after the column's text.
    ///
    /// See [`padding`](#method.padding).
    ///
    /// # Arguments
    ///
    /// * `padding` - The width in blank spaces/lines of the desired padding.
    ///
    /// # Errors
    ///
    /// * `ColonnadeError::InsufficientSpace` - This padding will require more space than is available in the viewport.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::{Alignment,Colonnade};
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// colonnade.padding_right(1)?;
    /// # Ok(()) }
    /// ```
    pub fn padding_right(&mut self, padding: usize) -> Result<&mut Self, ColonnadeError> {
        for i in 0..self.len() {
            self.columns[i].padding_right(padding);
        }
        if !self.sufficient_space() {
            Err(ColonnadeError::InsufficientSpace)
        } else {
            Ok(self)
        }
    }
    /// Assign all columns the same vertical padding -- blank lines before and after the column's text.
    ///
    /// See [`padding`](#method.padding).
    ///
    /// # Arguments
    ///
    /// * `padding` - The width in blank spaces/lines of the desired padding.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::{Alignment,Colonnade};
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// colonnade.padding_vertical(1);
    /// # Ok(()) }
    /// ```
    pub fn padding_vertical(&mut self, padding: usize) -> &mut Self {
        for i in 0..self.len() {
            self.columns[i].padding_vertical(padding);
        }
        self
    }
    /// Assign all columns the same top padding -- blank lines before the column's text.
    ///
    /// See [`padding`](#method.padding).
    ///
    /// # Arguments
    ///
    /// * `padding` - The width in blank spaces/lines of the desired padding.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::{Alignment,Colonnade};
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// colonnade.padding_top(1);
    /// # Ok(()) }
    /// ```
    pub fn padding_top(&mut self, padding: usize) -> &mut Self {
        for i in 0..self.len() {
            self.columns[i].padding_top(padding);
        }
        self
    }
    /// Assign all columns the same bottom padding -- blank lines after the column's text.
    ///
    /// See [`padding`](#method.padding).
    ///
    /// # Arguments
    ///
    /// * `padding` - The width in blank spaces/lines of the desired padding.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate colonnade;
    /// # use colonnade::{Alignment,Colonnade};
    /// # use std::error::Error;
    /// # fn demo() -> Result<(), Box<dyn Error>> {
    /// let mut colonnade = Colonnade::new(4, 100)?;
    /// colonnade.padding_bottom(1);
    /// # Ok(()) }
    /// ```
    pub fn padding_bottom(&mut self, padding: usize) -> &mut Self {
        for i in 0..self.len() {
            self.columns[i].padding_bottom(padding);
        }
        self
    }
    /// Toggle the hyphenation of all columns.
    ///
    /// See [`Column::hyphenate`](struct.Column.html#method.hyphenate).
    ///
    /// # Arguments
    ///
    /// * `hyphenate` - Whether long words will be hyphenated when split.
    pub fn hyphenate(&mut self, hyphenate: bool) -> &mut Self {
        for i in 0..self.len() {
            self.columns[i].hyphenate(hyphenate);
        }
        self
    }
}
