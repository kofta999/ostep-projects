use crate::lib::shell_command::ShellCommand;
use anyhow::{Result, bail};
use nix::{
    fcntl::{OFlag, open},
    sys::{stat::Mode, wait::waitpid},
    unistd::{AccessFlags, ForkResult, Pid, access, chdir, dup2_stderr, dup2_stdout, execv, fork},
};
use std::{ffi::CString, path::PathBuf, process::exit};

pub struct ShellState {
    path: Vec<PathBuf>,
}

impl ShellState {
    pub fn new(path: Vec<PathBuf>) -> Self {
        Self { path }
    }

    fn execute<F>(&mut self, cmd: ShellCommand, handler: F) -> Result<()>
    where
        F: FnMut(Pid) -> Result<()>,
    {
        match cmd {
            ShellCommand::BuiltinExit => exit(0),
            ShellCommand::BuiltinCd(path_buf) => Ok(chdir(&path_buf)?),
            ShellCommand::BuiltinPath(items) => {
                self.path = items;
                Ok(())
            }
            ShellCommand::External { args, redirect } => {
                self.process_command(&args, redirect, handler)
            }
        }
    }

    fn process_command<F: FnMut(Pid) -> Result<()>>(
        &self,
        args: &[&str],
        redirect_path: Option<PathBuf>,
        mut parent_handle: F,
    ) -> Result<()> {
        for p in &self.path {
            let full_path = p.join(args[0]);

            if access(
                &full_path,
                AccessFlags::F_OK | AccessFlags::R_OK | AccessFlags::X_OK,
            )
            .is_ok()
            {
                match unsafe { fork() } {
                    Ok(ForkResult::Parent { child }) => {
                        parent_handle(child)?;
                        return Ok(());
                    }
                    Ok(ForkResult::Child) => {
                        let c_args: Vec<CString> = args
                            .iter()
                            .map(|arg| CString::new(*arg).expect("Null byte found"))
                            .collect();

                        if let Some(redirect_path) = redirect_path {
                            let fd = open(
                                &redirect_path,
                                OFlag::O_CREAT | OFlag::O_WRONLY,
                                Mode::S_IRUSR | Mode::S_IWUSR,
                            )?;

                            dup2_stdout(&fd)?;
                            dup2_stderr(&fd)?;
                        }

                        execv(
                            &CString::new(
                                full_path
                                    .to_str()
                                    .expect("Couldn't convert PathBuf to &str"),
                            )?,
                            &c_args,
                        )?;
                        exit(1);
                    }
                    Err(_) => {
                        bail!("Couldn't execute child process")
                    }
                }
            };
        }

        bail!("Couldn't access executable file");
    }

    pub fn execute_single(&mut self, cmd: ShellCommand) -> Result<()> {
        self.execute(cmd, |child| {
            waitpid(child, None)?;
            Ok(())
        })
    }

    pub fn execute_parallel(&mut self, cmds: Vec<ShellCommand>) -> Result<()> {
        let mut child_pids = vec![];

        for cmd in cmds {
            self.execute(cmd, |child| {
                child_pids.push(child);
                Ok(())
            })?;
        }

        for child in child_pids {
            waitpid(child, None)?;
        }

        Ok(())
    }
}
