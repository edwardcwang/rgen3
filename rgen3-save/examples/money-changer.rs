// Test reading and writing money.

extern crate rgen3_save;
extern crate rgen3_string;

use rgen3_save::{Pokemon, SaveSectionsMut, TrainerInfo};
use std::collections::HashSet;

fn main() {
    let mut args = std::env::args().skip(1);
    let path_in = args.next().expect("Argument 1: path to load");
    let path_out = args.next().expect("Argument 2: path to write");
    let money = args.next().expect("Argument 3: new amount of money");
    let mut save = rgen3_save::Save::load_from_file(&path_in).unwrap();

    println!("Current money: {}", save.sections().getMoney());

    let new_money: u32 = money.parse().unwrap();
    let mut m = save.sections_mut();
    m.setMoney(new_money);
    println!("New money: {}", new_money);

    save.save_to_file(path_out).unwrap();
}
