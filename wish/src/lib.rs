use anyhow::{Result, anyhow, bail};
use nix::{
    fcntl::{OFlag, open},
    sys::{stat::Mode, wait::waitpid},
    unistd::{AccessFlags, ForkResult, Pid, access, chdir, dup2_stderr, dup2_stdout, execv, fork},
};
use std::{
    ffi::CString,
    fs::{self},
    io::{self, Write},
    process::exit,
};

const PS1: &str = "wish> ";
pub const GLOBAL_ERR_MSG: &str = "An error has occurred";

pub fn run(command_file: Option<&String>) -> Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut path: Vec<String> = vec!["/bin".into()];

    let mut exec = |mut args: Vec<&str>| {
        let commands: Vec<&[&str]> = args
            .split(|arg| *arg == "&")
            .filter(|c| !c.is_empty())
            .collect();

        if commands.len() > 1 {
            run_parallel_commands(&mut path, commands)
        } else {
            if commands.is_empty() {
                return;
            }

            match args[0] {
                "exit" => {
                    if args.len() == 1 {
                        handle_exit();
                    }

                    Err(anyhow!(""))
                }
                "cd" => handle_cd(&args),
                "path" => handle_path(&mut path, &args),
                _ => match get_redirect_path(&mut args) {
                    Ok(redirect_path) => {
                        process_command(&path, &args, redirect_path.as_deref(), |child| {
                            waitpid(child, None)?;
                            Ok(())
                        })
                    }
                    Err(_) => Err(anyhow!("")),
                },
            }
        }
        .unwrap_or_else(|_| eprintln!("{GLOBAL_ERR_MSG}"))
    };

    match command_file {
        Some(path) => {
            // TODO: more nix

            for line in fs::read_to_string(path)?.lines() {
                let args = parse_args(line.trim());

                exec(args);
            }

            Ok(())
        }
        None => {
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

                exec(args);
            }
        }
    }
}

fn handle_exit() {
    exit(0);
}

fn process_command<F: FnMut(Pid) -> Result<()>>(
    path: &[String],
    args: &[&str],
    redirect_path: Option<&str>,
    mut parent_handle: F,
) -> Result<()> {
    for p in path {
        let full_path = format!("{p}/{}", args[0]);

        if access(full_path.as_str(), AccessFlags::F_OK).is_ok() {
            match unsafe { fork() } {
                Ok(ForkResult::Parent { child }) => {
                    parent_handle(child)?;
                    return Ok(());
                }
                Ok(ForkResult::Child) => {
                    let _ = exec_child(args, redirect_path, &full_path);
                    exit(1);
                }
                Err(e) => {
                    dbg!(e);
                    bail!("")
                }
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
    command
        .split_whitespace()
        .flat_map(|word| word.split_inclusive(&['>', '&']))
        .flat_map(|part| {
            if part.len() > 1 {
                if let Some(part) = part.strip_suffix('>') {
                    return vec![&part, ">"];
                } else if let Some(part) = part.strip_suffix('&') {
                    return vec![&part, "&"];
                }
            }
            vec![part]
        })
        .collect()
}

fn get_redirect_path(args: &mut Vec<&str>) -> Result<Option<String>> {
    // TODO: Check if multiple >

    if let Some(idx) = args.iter().position(|arg| *arg == ">") {
        if idx == 0 {
            bail!("")
        }

        if let Some(path) = args.get(idx + 1).map(|s| s.to_string()) {
            if args.drain(idx..).len() > 2 {
                bail!("")
            };
            return Ok(Some(path));
        } else {
            bail!("")
        }
    }

    Ok(None)
}

fn exec_child(args: &[&str], redirect_path: Option<&str>, process_path: &str) -> Result<()> {
    let c_args: Vec<CString> = args
        .iter()
        .map(|arg| CString::new(*arg).expect("can't make cstring"))
        .collect();

    if let Some(redirect_path) = redirect_path {
        let fd = open(
            redirect_path,
            OFlag::O_CREAT | OFlag::O_WRONLY,
            Mode::S_IRUSR | Mode::S_IWUSR,
        )?;

        dup2_stdout(&fd)?;
        dup2_stderr(&fd)?;
    }

    execv(&CString::new(process_path)?, &c_args)?;

    Ok(())
}

fn run_parallel_commands(path: &mut Vec<String>, commands: Vec<&[&str]>) -> Result<()> {
    let mut child_pids = vec![];
    for command_args in commands {
        let mut command_args = command_args.to_vec();
        match command_args[0] {
            "exit" => {
                if command_args.len() == 1 {
                    handle_exit()
                }

                bail!("")
            }
            "cd" => handle_cd(&command_args),
            "path" => handle_path(path, &command_args),
            _ => {
                let redirect_path = get_redirect_path(&mut command_args)?;
                process_command(path, &command_args, redirect_path.as_deref(), |child| {
                    child_pids.push(child);
                    Ok(())
                })
            }
        }?;
    }

    for child in child_pids {
        waitpid(child, None)?;
    }

    Ok(())
}
