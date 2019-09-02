extern crate colonnade;
use colonnade::{Alignment, Colonnade};

#[allow(unused_must_use)]
fn main() {
    let text = vec![
        vec![
            "Colonnade lets you format text in columns.",
            "As you can see, it supports text alignment, viewport width, and column widths.",
            "It doesn't yet support color codes or other formatting, though that may come.",
        ],
        vec![
            "",
            "Two or more rows of columns makes a table.",
            "",
        ],
    ];
    let mut colonnade = Colonnade::new(3, 80).unwrap();
    colonnade.left_margin_all(4);
    colonnade.left_margin(0, 8);
    colonnade.fixed_width_all(15);
    colonnade.clear_limits(1);
    colonnade.alignment(0, Alignment::Right);
    colonnade.alignment(1, Alignment::Center);
    colonnade.alignment(2, Alignment::Left);
    colonnade.spaces_between_rows(1);
    for line in colonnade.tabulate(&text).unwrap() {
        println!("{}", line);
    }
}