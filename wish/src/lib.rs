use anyhow::{Result, bail};
use nix::{
    sys::wait::waitpid,
    unistd::{AccessFlags, ForkResult, access, chdir, execv, fork},
};
use std::{
    ffi::CString,
    io::{self, Write},
    process::exit,
};

const PS1: &str = "wish> ";
pub const GLOBAL_ERR_MSG: &str = "An error has occurred";

pub fn run() -> Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut path: Vec<String> = vec!["/bin".into()];

    loop {
        // read
        let mut lock = stdout.lock();
        let mut command_line = String::new();

        write!(lock, "{PS1}").expect("can't write to stdout");
        lock.flush().expect("can't flush to stdout");

        let read_count = stdin
            .read_line(&mut command_line)
            .expect("can't read from stdin");

        if read_count == 0 {
            handle_exit();
        }

        // eval
        let args = parse_args(command_line.trim());

        if args.is_empty() {
            continue;
        }

        match args[0] {
            "exit" => {
                handle_exit();
                Ok(())
            }
            "cd" => handle_cd(args),
            "path" => handle_path(&mut path, &args),
            _ => process_command(&path, &args),
        }
        .unwrap_or_else(|_| eprintln!("{GLOBAL_ERR_MSG}"))

        // print
    }
}

fn handle_exit() {
    exit(0);
}

fn process_command(path: &[String], args: &[&str]) -> Result<()> {
    for path in path {
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
                Err(_) => bail!(""),
            }
        };
    }

    bail!("")
}

fn handle_cd(args: Vec<&str>) -> Result<()> {
    if args.len() < 2 {
        bail!("")
    }

    chdir(args[1])?;

    Ok(())
}

fn handle_path(path: &mut Vec<String>, args: &[&str]) -> Result<()> {
    // TODO: Check if dir

    path.clear();
    path.extend(args.iter().skip(1).map(|s| s.to_string()));

    Ok(())
}

fn parse_args(command: &str) -> Vec<&str> {
    command.split_whitespace().collect()
}
