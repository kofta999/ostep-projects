mod lib;
use std::process::exit;

use lib::GLOBAL_ERR_MSG;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 2 {
        eprintln!("{GLOBAL_ERR_MSG}");
        exit(1);
    }

    if lib::run(args.get(1)).is_err() {
        eprintln!("{GLOBAL_ERR_MSG}");
        exit(1);
    }
}
