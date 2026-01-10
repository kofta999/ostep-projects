mod lib;
use lib::GLOBAL_ERR_MSG;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if let Err(e) = lib::run(args.get(1)) {
        dbg!(e);
        eprintln!("{GLOBAL_ERR_MSG}")
    }
}
