extern crate colonnade;
use colonnade::{Alignment, Colonnade};

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
    colonnade.alignment_all(Alignment::Left);
    let data = vec![vec![7, 8, 9], vec![10, 11, 12]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "7  8  9 ");
    assert_eq!(lines[1], "10 11 12");
}
#[test]
fn right_justification() {
    let mut colonnade = Colonnade::new(3, 100).unwrap();
    colonnade.alignment_all(Alignment::Right);
    let data = vec![vec![7, 8, 9], vec![10, 11, 12]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], " 7  8  9");
    assert_eq!(lines[1], "10 11 12");
}
#[test]
fn center_justification() {
    let mut colonnade = Colonnade::new(3, 100).unwrap();
    colonnade.alignment_all(Alignment::Center);
    let data = vec![vec![7, 8, 9], vec![100, 110, 120]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], " 7   8   9 ");
    assert_eq!(lines[1], "100 110 120");
}
#[test]
fn left_center_right() {
    let mut colonnade = Colonnade::new(3, 100).unwrap();
    colonnade.alignment(0, Alignment::Left).unwrap();
    colonnade.alignment(1, Alignment::Center).unwrap();
    colonnade.alignment(2, Alignment::Right).unwrap();
    let data = vec![vec![7, 8, 9], vec![100, 110, 120]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "7    8    9");
    assert_eq!(lines[1], "100 110 120");
}
#[test]
fn wrap() {
    let mut colonnade = Colonnade::new(3, 10).unwrap();
    colonnade.left_margin_all(2).unwrap();
    colonnade.left_margin(0, 0).unwrap();
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
    colonnade.left_margin_all(2).unwrap();
    colonnade.left_margin(0, 0).unwrap();
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
    colonnade.min_width(0, 5).unwrap();
    let data = vec![vec!["a", "b"]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "a     b")
}
#[test]
fn max_width() {
    let mut colonnade = Colonnade::new(2, 10).unwrap();
    colonnade.max_width(0, 5).unwrap();
    let data = vec![vec!["abcdef", "g"]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "abcd- g");
    assert_eq!(lines[1], "ef     ");
}
#[test]
fn fixed_width() {
    let mut colonnade = Colonnade::new(2, 11).unwrap();
    colonnade.fixed_width_all(5).unwrap();
    let data = vec![vec!["abcdef", "g"]];
    let lines = colonnade.tabulate(&data).unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "abcd- g    ");
    assert_eq!(lines[1], "ef         ");
}
#[test]
fn priority() {
    let mut colonnade = Colonnade::new(2, 20).unwrap();
    colonnade.priority(0, 0).unwrap();
    let data = vec![vec!["a bunch of words", "these are some words"]];
    let lines = colonnade.tabulate(&data).unwrap();
    println!("{:?}", lines);
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0], "a bunch of     these");
    assert_eq!(lines[1], "words          are  ");
    assert_eq!(lines[2], "               some ");
    assert_eq!(lines[3], "               words");
}
