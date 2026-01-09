use std::{
    io::{self, Write},
    process::exit,
};
const PS1: &str = "wish> ";

pub fn run() {
    let stdin = io::stdin();
    let stdout = io::stdout();

    loop {
        // read
        let mut lock = stdout.lock();
        let mut command = String::new();

        write!(lock, "{PS1}").expect("can't write to stdout");
        lock.flush().expect("can't flush to stdout");

        let read_count = stdin
            .read_line(&mut command)
            .expect("can't read from stdin");

        if read_count == 0 {
            handle_exit();
        }

        let command = command.trim();

        // eval
        match command {
            "exit" => handle_exit(),
            _ => process_command(command),
        }

        // print
    }
}

fn handle_exit() {
    exit(0);
}

fn process_command(command: &str) {
    eprintln!("An error has occurred");
}
