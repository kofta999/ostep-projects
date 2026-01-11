use anyhow::{Result, anyhow, bail};
use std::path::PathBuf;

#[derive(Debug)]
pub enum ShellCommand<'a> {
    BuiltinExit,
    BuiltinCd(PathBuf),
    BuiltinPath(Vec<PathBuf>),
    External {
        args: Vec<&'a str>,
        redirect: Option<PathBuf>,
    },
}

impl<'a> TryFrom<&'a str> for ShellCommand<'a> {
    type Error = anyhow::Error;

    fn try_from(input: &'a str) -> std::result::Result<Self, Self::Error> {
        let args = Self::tokenize(input);

        match *args.first().expect("args is empty") {
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

impl<'a> ShellCommand<'a> {
    fn tokenize(line: &'a str) -> Vec<&'a str> {
        line.split_whitespace()
            .flat_map(|word| {
                // We split by '>' but keep the '>' as its own slice
                if word.contains('>') {
                    let mut tokens = Vec::new();
                    let mut start = 0;
                    for (i, c) in word.char_indices() {
                        if c == '>' {
                            if i > start {
                                tokens.push(&word[start..i]); // text before >
                            }
                            tokens.push(">"); // the > itself
                            start = i + 1;
                        }
                    }
                    if start < word.len() {
                        tokens.push(&word[start..]); // text after last >
                    }
                    tokens
                } else {
                    vec![word]
                }
            })
            .collect()
    }

    fn parse_cd(args: &[&str]) -> Option<PathBuf> {
        args.get(1).map(PathBuf::from)
    }

    fn get_redirect(mut args: Vec<&'a str>) -> Result<(Option<PathBuf>, Vec<&'a str>)> {
        if let Some(idx) = args.iter().position(|arg| *arg == ">") {
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
