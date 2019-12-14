extern crate colonnade;
extern crate term;
use colonnade::{Alignment, Colonnade};
fn main() {
    // text to put in tabular form
    let text = vec![
        vec![
            "Colonnade lets you format text in columns.",
            "As you can see, it supports text alignment, viewport width, and column widths.",
            "It doesn't natively support color codes, but it is easy enough to combine with a crate like term.",
        ],
        vec!["", "Two or more rows of columns makes a table.", ""],
    ];
    let mut colonnade = Colonnade::new(3, 80).unwrap();

    // configure the table a bit
    colonnade.spaces_between_rows(1).left_margin(4).unwrap().fixed_width(15).unwrap();
    colonnade.columns[0].alignment(Alignment::Right).left_margin(8);
    colonnade.columns[1].alignment(Alignment::Center).clear_limits();
    // if the text is in colored cells, you will probably want some padding
    colonnade.padding(1).unwrap();

    // now print out the table
    let mut t = term::stdout().unwrap();
    for row in colonnade.macerate(&text).unwrap() {
        for line in row {
            for (i, (margin, text)) in line.iter().enumerate() {
                write!(t, "{}", margin).unwrap();
                let background_color = if i % 2 == 0 {
                    term::color::WHITE
                } else {
                    term::color::BLACK
                };
                let foreground_color = match i % 3 {
                    1 => term::color::GREEN,
                    2 => term::color::RED,
                    _ => term::color::BLUE,
                };
                t.bg(background_color).unwrap();
                t.fg(foreground_color).unwrap();
                write!(t, "{}", text).unwrap();
                t.reset().unwrap();
            }
            println!();
        }
    }
}
