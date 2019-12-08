extern crate colonnade;
use colonnade::{Alignment, Colonnade};
extern crate term;

#[allow(unused_must_use)]
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
    let mut colonnade = Colonnade::new(3, 80).unwrap(); // 3 columns of text in an 80-character viewport

    // configure the table a bit
    colonnade.left_margin_all(4);
    colonnade.left_margin(0, 8); // the first column should have a left margin 8 spaces wide
    colonnade.fixed_width_all(15);
    colonnade.clear_limits(1); // the central column has no fixed size limits
    colonnade.alignment(0, Alignment::Right);
    colonnade.alignment(1, Alignment::Center);
    colonnade.alignment(2, Alignment::Left);
    colonnade.spaces_between_rows(1); // add a blank link between rows
    colonnade.padding_all(1);

    let mut t = term::stdout().unwrap();
    // now print out the table
    for line in colonnade.macerate(&text).unwrap() {
        for (i, (margin, text)) in line.iter().enumerate() {
            write!(t, "{}", margin);
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
            t.reset();
        }
        println!();
    }
}
