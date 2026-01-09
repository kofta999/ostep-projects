use std::{
    ffi::CString,
    io::{self, Write},
    process::exit,
};

use anyhow::{Result, bail};
use nix::{
    sys::wait::waitpid,
    unistd::{AccessFlags, ForkResult, access, execv, fork},
};

const PS1: &str = "wish> ";
const GLOBAL_ERR_MSG: &str = "An error has occurred";
const PATH: [&str; 1] = ["/bin"];

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
            _ => {
                if process_command(command).is_err() {
                    eprintln!("{GLOBAL_ERR_MSG}");
                }
            }
        }

        // print
    }
}

fn handle_exit() {
    exit(0);
}

fn process_command(command: &str) -> Result<()> {
    let args = parse_args(command);

    if args.is_empty() {
        bail!("{GLOBAL_ERR_MSG}")
    }

    for path in PATH {
        let full_path = format!("{path}/{}", args[0]);

        if access(full_path.as_str(), AccessFlags::F_OK).is_ok() {
            match unsafe { fork() } {
                Ok(ForkResult::Parent { child }) => {
                    waitpid(child, None)?;
                    return Ok(());
                }
                Ok(ForkResult::Child) => {
                    let c_args: Vec<CString> = args
                        .iter()
                        .map(|arg| CString::new(*arg).expect("can't make cstring"))
                        .collect();

                    execv(&CString::new(full_path)?, &c_args)?;
                }
                Err(_) => bail!("{GLOBAL_ERR_MSG}"),
            }
        };
    }

    bail!("{GLOBAL_ERR_MSG}")
}

fn parse_args(command: &str) -> Vec<&str> {
    command.split_whitespace().collect()
}
