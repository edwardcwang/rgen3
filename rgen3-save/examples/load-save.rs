// Load and save a Pokemon savefile as a test.

extern crate rgen3_save;
extern crate rgen3_string;

use rgen3_save::{Pokemon, SaveSectionsMut, TrainerInfo};
use std::collections::HashSet;

fn main() {
    let mut args = std::env::args().skip(1);
    let path_in = args.next().expect("Argument 1: path to load");
    let path_out = args.next().expect("Argument 2: path to write");
    let mut save = rgen3_save::Save::load_from_file(&path_in).unwrap();
    save.save_to_file(path_out).unwrap();
}
