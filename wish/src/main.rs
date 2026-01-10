mod lib;
use lib::GLOBAL_ERR_MSG;

fn main() {
    if let Err(e) = lib::run() {
        dbg!(e);
        eprintln!("{GLOBAL_ERR_MSG}")
    }
}
