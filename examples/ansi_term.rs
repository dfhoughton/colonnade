extern crate ansi_term;
extern crate colonnade;
use ansi_term::Colour::{Black, Blue, Green, Red, White, Yellow};
use ansi_term::Style;
use colonnade::{Alignment, Colonnade, VerticalAlignment};

#[allow(unused_must_use)]
fn main() {
    // text to put in tabular form
    let text = vec![
        vec![
            "Let's try colou?r!",
            "Put a bunch of text in this cell so it needs to be laid out in multiple lines. Two sentences should do the trick. But let's add a third just in case.",
            "This one is short.",
            "UTF8 bloß.",
        ],
        vec![
            "This time we'll make this one longish as well so again we get multiple lines.",
            "This one's short.",
            "This one is also on the long side.",
            "And this one goes on and on and on and on and on and on and on and on and on and on and on. That should do it.",
        ],
    ];
    let mut colonnade = Colonnade::new(4, 100).unwrap(); // 4 columns of text in an 80-character viewport

    // configure the table a bit
    colonnade
        .spaces_between_rows(1)
        .left_margin(4)
        .unwrap()
        .padding(1)
        .unwrap()
        .fixed_width(15);
    colonnade.columns[0]
        .alignment(Alignment::Right)
        .left_margin(8);
    colonnade.columns[1]
        .clear_limits()
        .alignment(Alignment::Center)
        .vertical_alignment(VerticalAlignment::Middle);
    colonnade.columns[2].vertical_alignment(VerticalAlignment::Middle);
    colonnade.columns[3].vertical_alignment(VerticalAlignment::Bottom);

    // now print out the table
    let mut style_toggle = 0;
    for (row_num, row) in colonnade.macerate(&text).unwrap().iter().enumerate() {
        for line in row {
            if line.len() > 1 {
                for (cell_num, cell) in line.iter().enumerate() {
                    let colors = match cell_num {
                        0 => (Blue, White),
                        1 => (Black, White),
                        2 => (Red, Yellow),
                        _ => (Blue, Green),
                    };
                    let (fg, bg) = if row_num % 2 == 0 {
                        (colors.0, colors.1)
                    } else {
                        (colors.1, colors.0)
                    };
                    let style = Style::new().fg(fg).on(bg);
                    let style = match (cell_num + style_toggle) % 5 {
                        0 => style,
                        1 => style.italic(),
                        2 => style.bold(),
                        3 => style.underline(),
                        4 => style.strikethrough(),
                        _ => style,
                    };
                    print!("{}{}", cell.0, style.paint(&cell.1));
                }
            }
            println!();
        }
        style_toggle += 1;
    }
}
