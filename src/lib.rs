use std::iter;

#[derive(Debug)]
pub enum ColonnadeError {
    InconsistentColumns(usize, usize, usize), // row, row length, spec length
    OutOfBounds,
    InsufficientColumns,
    InsufficientSpace,
    MinGreaterThanMax(usize), // column
}

#[derive(Debug, Clone)]
pub enum Alignment {
    Left,
    Right,
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
        }
    }
    fn effective_width(&self) -> usize {
        let w = if self.max_width.unwrap_or(self.width) < self.width {
            self.max_width.unwrap()
        } else {
            self.width
        };
        if self.min_width.unwrap_or(w) > w {
            self.min_width.unwrap()
        } else {
            w
        }
    }
    fn is_shrinkable(&self) -> bool {
        self.min_width.unwrap_or(0) < self.width
    }
    // shrink as close to width as possible
    fn shrink(&mut self, width: usize) {
        self.width = if self.min_width.unwrap_or(width) > width {
            self.min_width.unwrap()
        } else {
            width
        }
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
        } else if self.min_width.unwrap_or(width) > width {
            self.min_width.unwrap()
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
}

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
    pub fn tabulate(&mut self, table: &Vec<Vec<&str>>) -> Result<Vec<String>, ColonnadeError> {
        if !self.adjusted {
            match self.lay_out(table) {
                Err(e) => return Err(e),
                Ok(()) => (),
            }
        }
        let mut buffer = vec![];
        for (i, row) in table.iter().enumerate() {
            self.add_row(&mut buffer, row, i == table.len() - 1);
        }
        Ok(buffer)
    }
    // take one row of untabulated pieces of text and turn it into one or more lines of tabulated text
    fn add_row(&self, buffer: &mut Vec<String>, row: &Vec<&str>, last_row: bool) {
        let mut words: Vec<Vec<&str>> = row
            .iter()
            .map(|w| w.trim().split_whitespace().collect())
            .collect();
        if words.iter().all(|sentence| sentence.is_empty()) {
            // blank line
            buffer.push(String::new());
            if !last_row {
                for _ in 0..self.spaces_between_rows {
                    buffer.push(String::new());
                }
            }
            return;
        }
        while !words.iter().all(|sentence| sentence.is_empty()) {
            let mut line = String::new();
            for (i, c) in self.colonnade.iter().enumerate() {
                for _ in 0..c.left_margin {
                    line += " "
                }
                if words[i].is_empty() {
                    // we've used this one up, but there are still words to deal with in other sentences
                    for _ in 0..c.width {
                        line += " "
                    }
                } else {
                    let mut l = 0;
                    let mut phrase = String::new();
                    let mut first = true;
                    while !words[i].is_empty() {
                        let w = words[i].remove(0);
                        if first {
                            if w.len() == c.width {
                                // word fills column
                                phrase += w;
                                break;
                            } else if w.len() > c.width {
                                // word overflows column and we must split it
                                if c.width > 1 {
                                    phrase += &w[0..(c.width - 1)];
                                    words[i].insert(0, &w[(c.width - 1)..w.len()]);
                                    phrase += "-";
                                } else {
                                    phrase += &w[0..1];
                                    words[i].insert(0, &w[1..w.len()]);
                                }
                                break;
                            }
                        }
                        // try to tack on a new word
                        let new_length = l + w.len() + if first { 0 } else { 1 };
                        if new_length > c.width {
                            words[i].insert(0, w);
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
                    // pad phrase out propery in its cell
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
                                for _ in 0..surplus {
                                    line += " "
                                }
                                line += &phrase;
                            }
                        }
                    } else {
                        line += &phrase;
                    }
                }
            }
            buffer.push(line);
        }
        if !last_row {
            for _ in 0..self.spaces_between_rows {
                buffer.push(String::new());
            }
        }
    }
    // determine column widths given data
    pub fn lay_out(&mut self, table: &Vec<Vec<&str>>) -> Result<(), ColonnadeError> {
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
                return Err(ColonnadeError::InsufficientSpace)
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
    pub fn spaces_between_rows(&mut self, n: usize) {
        self.spaces_between_rows = n;
    }
    pub fn priority_all(&mut self, priority: usize) {
        for i in 0..self.len() {
            self.colonnade[i].priority = priority;
        }
    }
    pub fn priority(&mut self, index: usize, priority: usize) -> Result<(), ColonnadeError> {
        if index < self.len() {
            self.colonnade[index].priority = priority;
            Ok(())
        } else {
            Err(ColonnadeError::OutOfBounds)
        }
    }
    pub fn max_width_all(&mut self, max_width: usize) -> Result<(), ColonnadeError> {
        for i in 0..self.len() {
            if self.colonnade[i].min_width.unwrap_or(0) > max_width {
                return Err(ColonnadeError::MinGreaterThanMax(i));
            }
            self.colonnade[i].max_width = Some(max_width);
        }
        Ok(())
    }
    pub fn max_width(&mut self, index: usize, max_width: usize) -> Result<(), ColonnadeError> {
        if index < self.len() {
            self.colonnade[index].max_width = Some(max_width);
            Ok(())
        } else {
            if self.colonnade[index].min_width.unwrap_or(0) > max_width {
                return Err(ColonnadeError::MinGreaterThanMax(index));
            }
            Err(ColonnadeError::OutOfBounds)
        }
    }
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
    pub fn fixed_width_all(&mut self, min_width: usize) -> Result<(), ColonnadeError> {
        match self.min_width_all(min_width) {
            Err(e) => return Err(e),
            Ok(_) => (),
        }
        match self.max_width_all(min_width) {
            Err(e) => return Err(e),
            Ok(_) => (),
        }
        Ok(())
    }
    pub fn fixed_width(&mut self, index: usize, min_width: usize) -> Result<(), ColonnadeError> {
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
    pub fn clear_limits_all(&mut self) {
       for i in 0..self.len() {
           self.colonnade[i].max_width = None;
           self.colonnade[i].min_width = None;
       }
    }
    pub fn clear_limits(&mut self, index: usize) -> Result<(), ColonnadeError> {
        if index < self.len() {
            self.colonnade[index].max_width = None;
            self.colonnade[index].min_width = None;
                Ok(())
        } else {
            Err(ColonnadeError::OutOfBounds)
        }
    }
    pub fn alignment_all(&mut self, alignment: Alignment) {
        for i in 0..self.len() {
            self.colonnade[i].alignment = alignment.clone();
        }
    }
    pub fn alignment(&mut self, index: usize, alignment: Alignment) -> Result<(), ColonnadeError> {
        if index < self.len() {
            self.colonnade[index].alignment = alignment;
            Ok(())
        } else {
            Err(ColonnadeError::OutOfBounds)
        }
    }
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
}
