extern crate colonnade;
use colonnade::{Alignment, Colonnade};

// demonstrating Alignment::Justify
fn main() {
    let mut colonnade = Colonnade::new(2, 50).unwrap();
    colonnade
        .alignment(Alignment::Justify)
        .spaces_between_rows(1)
        .left_margin(2)
        .unwrap();
    let data = [
        ["one line", "more"],
        ["This is a bunch of text so we can see what happens with non final lines. The last line shouldn't be justified.", "Let's see what it looks like when there are two columns."],
    ];
    for line in colonnade.tabulate(&data).unwrap() {
        println!("{}", line);
    }
}
