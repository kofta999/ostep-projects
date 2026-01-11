use std::path::PathBuf;

use anyhow::{Ok, Result, anyhow, bail};

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

impl TryFrom<Vec<String>> for ShellCommand {
    type Error = anyhow::Error;

    fn try_from(args: Vec<String>) -> std::result::Result<Self, Self::Error> {
        match args[0].as_str() {
            "exit" => {
                if args.len() != 1 {
                    bail!("Extra argument in exit command")
                }
                Ok(Self::BuiltinExit)
            }
            "cd" => {
                let path = Self::parse_cd(&args).ok_or(anyhow!(""))?;
                Ok(Self::BuiltinCd(path))
            }
            "path" => Ok(Self::BuiltinPath(
                args.into_iter().skip(1).map(PathBuf::from).collect(),
            )),
            _ => {
                let (redirect, args) = Self::get_redirect(args).map_err(|_| anyhow!(""))?;
                Ok(Self::External { args, redirect })
            }
        }
    }
}

impl ShellCommand {
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
