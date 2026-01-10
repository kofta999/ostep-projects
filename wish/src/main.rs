mod lib;
use std::process::exit;

use lib::GLOBAL_ERR_MSG;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 2 {
        eprintln!("{GLOBAL_ERR_MSG}");
        exit(1);
    }

    if let Err(e) = lib::run(args.get(1)) {
        eprintln!("{GLOBAL_ERR_MSG}");
        exit(1);
    }
}
