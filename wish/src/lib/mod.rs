mod shell_command;
mod shell_state;
use anyhow::{Ok, Result};
use shell_command::ShellCommand;
use shell_state::ShellState;
use std::{
    fs::{self},
    io::{self, Write},
};

const PS1: &str = "wish> ";
pub const GLOBAL_ERR_MSG: &str = "An error has occurred";

pub fn run(command_file: Option<&String>) -> Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut shell_state = ShellState::new(vec!["/bin".into()]);

    let mut exec_command_line = |command_line: &str| -> Result<()> {
        let mut commands = parse_command_line(command_line.trim())?;

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

            for command_line in fs::read_to_string(path)?.lines() {
                if let Err(e) = exec_command_line(command_line) {
                    eprintln!("{GLOBAL_ERR_MSG}");
                    #[cfg(debug_assertions)]
                    eprintln!("Detailed error: {:?}", e);
                }
            }
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
                    break;
                }

                // eval
                if let Err(e) = exec_command_line(&command_line) {
                    eprintln!("{GLOBAL_ERR_MSG}");
                    #[cfg(debug_assertions)]
                    eprintln!("Detailed error: {:?}", e);
                }
            }
        }
    }

    Ok(())
}

fn parse_command_line(input: &str) -> Result<Vec<ShellCommand>> {
    input
        .split('&')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|cmd_chunk| ShellCommand::try_from(cmd_chunk.to_string()))
        .collect()
}
