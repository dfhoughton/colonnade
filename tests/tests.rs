extern crate colonnade;
use colonnade::{Alignment, Colonnade, VerticalAlignment};

#[test]
fn minimal_table() {
    let mut colonnade = Colonnade::new(3, 100).unwrap();
    let data = vec![vec![1, 2, 3], vec![4, 5, 6]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "1 2 3");
    assert_eq!(lines[1], "4 5 6");
}
#[test]
fn justification() {
    let mut colonnade = Colonnade::new(3, 100).unwrap();
    let data = vec![vec![7, 8, 9], vec![10, 11, 12]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "7  8  9 ");
    assert_eq!(lines[1], "10 11 12");
}
#[test]
fn left_justification() {
    let mut colonnade = Colonnade::new(3, 100).unwrap();
    colonnade.alignment(Alignment::Left);
    let data = vec![vec![7, 8, 9], vec![10, 11, 12]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "7  8  9 ");
    assert_eq!(lines[1], "10 11 12");
}
#[test]
fn right_justification() {
    let mut colonnade = Colonnade::new(3, 100).unwrap();
    colonnade.alignment(Alignment::Right);
    let data = vec![vec![7, 8, 9], vec![10, 11, 12]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], " 7  8  9");
    assert_eq!(lines[1], "10 11 12");
}
#[test]
fn center_justification() {
    let mut colonnade = Colonnade::new(3, 100).unwrap();
    colonnade.alignment(Alignment::Center);
    let data = vec![vec![7, 8, 9], vec![100, 110, 120]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], " 7   8   9 ");
    assert_eq!(lines[1], "100 110 120");
}
#[test]
fn left_center_right() {
    let mut colonnade = Colonnade::new(3, 100).unwrap();
    colonnade.columns[0].alignment(Alignment::Left);
    colonnade.columns[1].alignment(Alignment::Center);
    colonnade.columns[2].alignment(Alignment::Right);
    let data = vec![vec![7, 8, 9], vec![100, 110, 120]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "7    8    9");
    assert_eq!(lines[1], "100 110 120");
}
#[test]
fn wrap() {
    let mut colonnade = Colonnade::new(3, 10).unwrap();
    colonnade.left_margin(2).unwrap();
    colonnade.columns[0].left_margin(0);
    let data = vec![vec!["1 2 3", "4 5 6", "7 8 9"]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "1   4   7 ");
    assert_eq!(lines[1], "2   5   8 ");
    assert_eq!(lines[2], "3   6   9 ");
}
#[test]
fn wrap2() {
    let mut colonnade = Colonnade::new(3, 13).unwrap();
    colonnade.left_margin(2).unwrap();
    colonnade.columns[0].left_margin(0);
    let data = vec![vec!["1 2 3", "4 5 6", "7 8 9"]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "1 2  4 5  7 8");
    assert_eq!(lines[1], "3    6    9  ");
}
#[test]
fn spaces_between_rows() {
    let mut colonnade = Colonnade::new(3, 10).unwrap();
    colonnade.spaces_between_rows(1);
    let data = vec![vec![1, 2, 3], vec![4, 5, 6]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "1 2 3");
    assert_eq!(lines[1], "");
    assert_eq!(lines[2], "4 5 6");
}
#[test]
fn hyphenation() {
    let mut colonnade = Colonnade::new(1, 10).unwrap();
    let data = vec![vec!["abcdefghijklmnopqrstuvwxyz"]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "abcdefghi-");
    assert_eq!(lines[1], "jklmnopqr-");
    assert_eq!(lines[2], "stuvwxyz  ");
}
#[test]
fn too_skinny_to_hyphenate() {
    let mut colonnade = Colonnade::new(1, 1).unwrap();
    let data = vec![vec!["abc"]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "a");
    assert_eq!(lines[1], "b");
    assert_eq!(lines[2], "c");
}
#[test]
fn min_width() {
    let mut colonnade = Colonnade::new(2, 10).unwrap();
    colonnade.columns[0].min_width(5).unwrap();
    let data = vec![vec!["a", "b"]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "a     b")
}
#[test]
fn max_width() {
    let mut colonnade = Colonnade::new(2, 10).unwrap();
    colonnade.columns[0].max_width(5).unwrap();
    let data = vec![vec!["abcdef", "g"]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "abcd- g");
    assert_eq!(lines[1], "ef     ");
}
#[test]
fn fixed_width() {
    let mut colonnade = Colonnade::new(2, 11).unwrap();
    colonnade.fixed_width(5).unwrap();
    let data = vec![vec!["abcdef", "g"]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "abcd- g    ");
    assert_eq!(lines[1], "ef         ");
}
#[test]
fn priority() {
    let mut colonnade = Colonnade::new(2, 20).unwrap();
    colonnade.columns[0].priority(0);
    let data = vec![vec!["a bunch of words", "these are some words"]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0], "a bunch of     these");
    assert_eq!(lines[1], "words          are  ");
    assert_eq!(lines[2], "               some ");
    assert_eq!(lines[3], "               words");
}

#[test]
fn padding() {
    let mut colonnade = Colonnade::new(3, 100).unwrap();
    colonnade.padding(1).unwrap();
    let data = vec![vec![1, 2, 3]];
    let lines: Vec<Vec<Vec<(String, String)>>> = colonnade.macerate(&data).unwrap();
    assert_eq!(3, lines[0].len(), "got vertical padding");
    let c = (String::from(""), String::from("   "));
    assert_eq!(c, lines[0][0][0]);
    let c = (String::from(" "), String::from("   "));
    assert_eq!(c, lines[0][0][1]);
    assert_eq!(c, lines[0][0][2]);
    let c = (String::from(""), String::from(" 1 "));
    assert_eq!(c, lines[0][1][0]);
    let c = (String::from(" "), String::from(" 2 "));
    assert_eq!(c, lines[0][1][1]);
    let c = (String::from(" "), String::from(" 3 "));
    assert_eq!(c, lines[0][1][2]);
    let c = (String::from(""), String::from("   "));
    assert_eq!(c, lines[0][2][0]);
    let c = (String::from(" "), String::from("   "));
    assert_eq!(c, lines[0][2][1]);
    assert_eq!(c, lines[0][2][2]);
}

#[test]
fn padding_top() {
    let mut colonnade = Colonnade::new(3, 100).unwrap();
    colonnade.padding_top(1);
    let data = vec![vec![1, 2, 3]];
    let lines: Vec<Vec<Vec<(String, String)>>> = colonnade.macerate(&data).unwrap();
    assert_eq!(2, lines[0].len(), "got vertical padding");
    let c = (String::from(""), String::from(" "));
    assert_eq!(c, lines[0][0][0]);
    let c = (String::from(" "), String::from(" "));
    assert_eq!(c, lines[0][0][1]);
    assert_eq!(c, lines[0][0][2]);
    let c = (String::from(""), String::from("1"));
    assert_eq!(c, lines[0][1][0]);
    let c = (String::from(" "), String::from("2"));
    assert_eq!(c, lines[0][1][1]);
    let c = (String::from(" "), String::from("3"));
    assert_eq!(c, lines[0][1][2]);
}

#[test]
fn padding_bottom() {
    let mut colonnade = Colonnade::new(3, 100).unwrap();
    colonnade.padding_bottom(1);
    let data = vec![vec![1, 2, 3]];
    let lines: Vec<Vec<Vec<(String, String)>>> = colonnade.macerate(&data).unwrap();
    assert_eq!(2, lines[0].len(), "got vertical padding");
    let c = (String::from(""), String::from("1"));
    assert_eq!(c, lines[0][0][0]);
    let c = (String::from(" "), String::from("2"));
    assert_eq!(c, lines[0][0][1]);
    let c = (String::from(" "), String::from("3"));
    assert_eq!(c, lines[0][0][2]);
    let c = (String::from(""), String::from(" "));
    assert_eq!(c, lines[0][1][0]);
    let c = (String::from(" "), String::from(" "));
    assert_eq!(c, lines[0][1][1]);
    assert_eq!(c, lines[0][1][2]);
}

#[test]
fn padding_left() {
    let mut colonnade = Colonnade::new(3, 100).unwrap();
    colonnade.padding_left(1).unwrap();
    let data = vec![vec![1, 2, 3]];
    let lines: Vec<Vec<Vec<(String, String)>>> = colonnade.macerate(&data).unwrap();
    assert_eq!(1, lines[0].len(), "no vertical padding");
    let c = (String::from(""), String::from(" 1"));
    assert_eq!(c, lines[0][0][0]);
    let c = (String::from(" "), String::from(" 2"));
    assert_eq!(c, lines[0][0][1]);
    let c = (String::from(" "), String::from(" 3"));
    assert_eq!(c, lines[0][0][2]);
}

#[test]
fn padding_right() {
    let mut colonnade = Colonnade::new(3, 100).unwrap();
    colonnade.padding_right(1).unwrap();
    let data = vec![vec![1, 2, 3]];
    let lines: Vec<Vec<Vec<(String, String)>>> = colonnade.macerate(&data).unwrap();
    assert_eq!(1, lines[0].len(), "no vertical padding");
    let c = (String::from(""), String::from("1 "));
    assert_eq!(c, lines[0][0][0]);
    let c = (String::from(" "), String::from("2 "));
    assert_eq!(c, lines[0][0][1]);
    let c = (String::from(" "), String::from("3 "));
    assert_eq!(c, lines[0][0][2]);
}

#[test]
fn centered_text() {
    let mut colonnade = Colonnade::new(2, 3).unwrap();
    colonnade.columns[0].vertical_alignment(VerticalAlignment::Middle);
    let data = vec![vec!["1", "2 3 4"]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(3, lines.len(), "got the right number of lines");
    assert_eq!("  2", lines[0]);
    assert_eq!("1 3", lines[1]);
    assert_eq!("  4", lines[2]);
}

#[test]
fn centered_text_with_padding() {
    let mut colonnade = Colonnade::new(2, 3).unwrap();
    colonnade.columns[0].vertical_alignment(VerticalAlignment::Middle);
    colonnade.columns[0].padding_vertical(1);
    let data = vec![vec!["1", "2 3 4"]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(3, lines.len(), "got the right number of lines");
    assert_eq!("  2", lines[0]);
    assert_eq!("1 3", lines[1]);
    assert_eq!("  4", lines[2]);
}

#[test]
fn bottom_text() {
    let mut colonnade = Colonnade::new(2, 3).unwrap();
    colonnade.columns[0].vertical_alignment(VerticalAlignment::Bottom);
    let data = vec![vec!["1", "2 3 4"]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(3, lines.len(), "got the right number of lines");
    assert_eq!("  2", lines[0]);
    assert_eq!("  3", lines[1]);
    assert_eq!("1 4", lines[2]);
}

#[test]
fn bottom_text_with_padding() {
    let mut colonnade = Colonnade::new(2, 3).unwrap();
    colonnade.columns[0].vertical_alignment(VerticalAlignment::Bottom);
    colonnade.columns[0].padding_vertical(1);
    let data = vec![vec!["1", "2 3 4"]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(3, lines.len(), "got the right number of lines");
    assert_eq!("  2", lines[0]);
    assert_eq!("1 3", lines[1]);
    assert_eq!("  4", lines[2]);
}

#[test]
fn bottom_text_with_bottom_padding() {
    let mut colonnade = Colonnade::new(2, 3).unwrap();
    colonnade.columns[0].vertical_alignment(VerticalAlignment::Bottom);
    colonnade.columns[0].padding_bottom(1);
    let data = vec![vec!["1", "2 3 4"]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(3, lines.len(), "got the right number of lines");
    assert_eq!("  2", lines[0]);
    assert_eq!("1 3", lines[1]);
    assert_eq!("  4", lines[2]);
}

#[test]
fn bottom_text_with_top_padding() {
    let mut colonnade = Colonnade::new(2, 3).unwrap();
    colonnade.columns[0].vertical_alignment(VerticalAlignment::Bottom);
    colonnade.columns[0].padding_top(1);
    let data = vec![vec!["1", "2 3 4"]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(3, lines.len(), "got the right number of lines");
    assert_eq!("  2", lines[0]);
    assert_eq!("  3", lines[1]);
    assert_eq!("1 4", lines[2]);
}

#[test]
fn centered_text_two_rows() {
    let mut colonnade = Colonnade::new(2, 3).unwrap();
    colonnade.columns[0].vertical_alignment(VerticalAlignment::Middle);
    let data = vec![vec!["5", "6"], vec!["1", "2 3 4"]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(4, lines.len(), "got the right number of lines");
    assert_eq!("5 6", lines[0]);
    assert_eq!("  2", lines[1]);
    assert_eq!("1 3", lines[2]);
    assert_eq!("  4", lines[3]);
}

#[test]
fn variable_length_rows() {
    let mut colonnade = Colonnade::new(2, 80).unwrap();
    let lines = colonnade.tabulate(&[vec![1], vec![2, 3]]).unwrap();
    assert_eq!(2, lines.len(), "got the right number of lines");
    assert_eq!("1  ", lines[0]);
    assert_eq!("2 3", lines[1]);
}

#[test]
fn reset() {
    let mut colonnade = Colonnade::new(3, 80).unwrap();
    let lines = colonnade.tabulate(&[[100, 200, 300]]).unwrap();
    assert_eq!(1, lines.len(), "got the right number of lines");
    assert_eq!("100 200 300", lines[0]);
    let lines = colonnade.tabulate(&[[1, 2, 3]]).unwrap();
    assert_eq!(1, lines.len(), "got the right number of lines");
    assert_eq!("1   2   3  ", lines[0]);
    colonnade.reset();
    let lines = colonnade.tabulate(&[[1, 2, 3]]).unwrap();
    assert_eq!(1, lines.len(), "got the right number of lines");
    assert_eq!("1 2 3", lines[0]);
}

#[test]
fn wide_char_wrapping() {
    let mut colonnade = Colonnade::new(1, 1).unwrap();
    let lines = colonnade.tabulate(&[["ßßß"]]).unwrap();
    assert_eq!(3, lines.len(), "got the right number of lines");
    for i in 0..3 {
        assert_eq!("ß", lines[i]);
    }
    colonnade = Colonnade::new(1, 2).unwrap();
    let lines = colonnade.tabulate(&[["ßßß"]]).unwrap();
    assert_eq!(2, lines.len(), "got the right number of lines");
    assert_eq!("ß-", lines[0]);
    assert_eq!("ßß", lines[1]);
    colonnade = Colonnade::new(1, 2).unwrap();
    let lines = colonnade.tabulate(&[["bloß"]]).unwrap();
    assert_eq!(3, lines.len(), "got the right number of lines");
    assert_eq!("b-", lines[0]);
    assert_eq!("l-", lines[1]);
    assert_eq!("oß", lines[2]);
}

#[test]
fn regression1() {
    let attributes = [
        ["day-length", "8"],
        ["editor", "/usr/bin/vim"],
        ["length-pay-period", "7"],
        ["precision", "2"],
        ["start-pay-period", "2016 10 3"],
        ["sunday-begins-week", "true"],
        ["workdays", "MTWHF"],
        ["max-width", ""],
    ];
    let mut colonnade = Colonnade::new(2, 127).unwrap();
    colonnade.columns[1]
        .alignment(Alignment::Right)
        .left_margin(2);
    colonnade.tabulate(&attributes).unwrap();
    assert!(true, "no panic");
}

#[cfg(feature = "nbsp")]
#[test]
fn nbsp() {
    let data = [
        ["foo", " bar"],
        ["baz", "\u{00A0}plugh"]
    ];
    let mut colonnade = Colonnade::new(2, 100).unwrap();
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(2, lines.len());
    assert_eq!("foo bar   ", lines[0]);
    assert_eq!("baz \u{00A0}plugh", lines[1]);
}