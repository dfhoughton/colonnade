extern crate colonnade;
use colonnade::{Colonnade, Alignment};

// demonstrating the purpose of reset
fn main() {
    let mut colonnade = Colonnade::new(3, 80).unwrap();
    colonnade.alignment(Alignment::Right);
    for line in colonnade.tabulate(&[[100, 200, 300]]).unwrap() {
        println!("{}", line);
    }
    // 100 200 300
    for line in colonnade.tabulate(&[[1, 2, 3]]).unwrap() {
        println!("{}", line);
    }
    //   1   2   3
    colonnade.reset();
    for line in colonnade.tabulate(&[[1, 2, 3]]).unwrap() {
        println!("{}", line);
    }
    // 1 2 3
}
