use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;

use smolfs_core::{Result, SmolFsError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub program: PathBuf,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
}

impl CommandSpec {
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
        }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    pub fn run(&self) -> Result<CommandOutput> {
        let output = Command::new(&self.program)
            .args(self.args.iter().map(OsStr::new))
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(SmolFsError::CommandFailed {
                program: self.program.display().to_string(),
                args: self.args.join(" "),
                status: output.status.to_string(),
                stdout,
                stderr,
            });
        }

        Ok(CommandOutput { stdout, stderr })
    }

    pub fn spawn(&self) -> Result<std::process::Child> {
        Ok(Command::new(&self.program)
            .args(self.args.iter().map(OsStr::new))
            .spawn()?)
    }
}
