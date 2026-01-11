mod shell_command;
mod shell_state;
use anyhow::{Context, Result};
use shell_command::ShellCommand;
use shell_state::ShellState;
use std::{
    fs::{self},
    io::{self, Write},
    process::exit,
};

const PS1: &str = "wish> ";
pub const GLOBAL_ERR_MSG: &str = "An error has occurred";

pub fn run(command_file: Option<&String>) -> Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut shell_state = ShellState::new(vec!["/bin".into()]);

    let mut exec = |args: Vec<String>| -> Result<()> {
        let mut commands: Vec<ShellCommand> = args
            .split(|arg| *arg == "&")
            .filter(|c| !c.is_empty())
            .map(|s| {
                ShellCommand::try_from(s.iter().map(|x| x.to_string()).collect::<Vec<String>>())
            })
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("Failed to parse command line")?;

        match commands.len() {
            0 => Ok(()),
            1 => shell_state.execute_single(commands.pop().expect("Command exists")),
            _ => shell_state.execute_parallel(commands),
        }?;

        Ok(())
    };

    match command_file {
        Some(path) => {
            // TODO: more nix

            for line in fs::read_to_string(path)?.lines() {
                let args = parse_args(line.trim());

                if args.is_empty() {
                    continue;
                }

                if exec(args).is_err() {
                    eprintln!("{GLOBAL_ERR_MSG}")
                };
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
                    exit(0);
                }

                // eval
                let args = parse_args(command_line.trim());

                if args.is_empty() {
                    continue;
                }

                if exec(args).is_err() {
                    eprintln!("{GLOBAL_ERR_MSG}")
                };
            }
        }
    }
}

fn parse_args(command: &str) -> Vec<String> {
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
        .map(|s| s.to_string())
        .collect()
}
