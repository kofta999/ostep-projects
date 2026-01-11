use anyhow::{Ok, Result, anyhow, bail};
use std::path::PathBuf;

type Tokens = String;

#[derive(Debug)]
pub enum ShellCommand {
    BuiltinExit,
    BuiltinCd(PathBuf),
    BuiltinPath(Vec<PathBuf>),
    External {
        args: Vec<String>,
        redirect: Option<PathBuf>,
    },
}

impl TryFrom<String> for ShellCommand {
    type Error = anyhow::Error;

    fn try_from(input: String) -> std::result::Result<Self, Self::Error> {
        let args = Self::parse_line(input);

        match args.first().expect("args is empty").as_str() {
            "exit" => {
                if args.len() != 1 {
                    bail!("Extra argument in exit command")
                }
                Ok(Self::BuiltinExit)
            }
            "cd" => {
                let path = Self::parse_cd(&args).ok_or(anyhow!("Couldn't create cd path"))?;
                Ok(Self::BuiltinCd(path))
            }
            "path" => Ok(Self::BuiltinPath(
                args.into_iter().skip(1).map(PathBuf::from).collect(),
            )),
            _ => {
                let (redirect, args) = Self::get_redirect(args)?;
                Ok(Self::External { args, redirect })
            }
        }
    }
}

impl ShellCommand {
    fn parse_line(line: String) -> Vec<Tokens> {
        line.replace(">", " > ")
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }

    fn parse_cd(args: &[String]) -> Option<PathBuf> {
        args.get(1).map(PathBuf::from)
    }

    fn get_redirect(mut args: Vec<String>) -> Result<(Option<PathBuf>, Vec<String>)> {
        if let Some(idx) = args.iter().position(|arg| arg == ">") {
            if idx == 0 || idx != args.len() - 2 {
                bail!("Redirection operator position invalid")
            }

            let path = args.pop().expect("Path exists");
            args.pop();

            return Ok((Some(PathBuf::from(path)), args));
        }

        Ok((None, args))
    }
}
