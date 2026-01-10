use anyhow::{Result, bail};
use nix::{
    fcntl::{OFlag, open},
    sys::{stat::Mode, wait::waitpid},
    unistd::{AccessFlags, ForkResult, access, chdir, dup2_stderr, dup2_stdout, execv, fork},
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
        let mut args = parse_args(command_line.trim());

        if args.is_empty() {
            continue;
        }

        match args[0] {
            "exit" => {
                handle_exit();
                Ok(())
            }
            "cd" => handle_cd(&args),
            "path" => handle_path(&mut path, &args),
            _ => {
                let redirect_path = get_redirect_path(&mut args);
                process_command(&path, &args, redirect_path)
            }
        }
        .unwrap_or_else(|_| eprintln!("{GLOBAL_ERR_MSG}"))
    }
}

fn handle_exit() {
    exit(0);
}

fn process_command(path: &[String], args: &[&str], redirect_path: Option<String>) -> Result<()> {
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

                    if let Some(redirect_path) = &redirect_path {
                        let fd = open(
                            redirect_path.as_str(),
                            OFlag::O_CREAT | OFlag::O_WRONLY,
                            Mode::S_IRUSR | Mode::S_IWUSR,
                        )?;

                        dup2_stdout(&fd)?;
                        dup2_stderr(&fd)?;
                    }

                    execv(&CString::new(full_path)?, &c_args)?;
                }
                Err(_) => bail!(""),
            }
        };
    }

    bail!("")
}

fn handle_cd(args: &[&str]) -> Result<()> {
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

fn get_redirect_path(args: &mut Vec<&str>) -> Option<String> {
    // TODO: Check if multiple >

    if let Some(idx) = args.iter().position(|arg| *arg == ">") {
        let path = args.get(idx + 1).map(|s| s.to_string());
        args.drain(idx..);
        return path;
    }

    None
}
