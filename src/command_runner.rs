use std::io;
use std::process::{Command, Output};

pub(crate) trait CommandRunner: Send + Sync {
    fn run(&self, command: &str, args: &[&str]) -> io::Result<Output>;

    fn run_checked(&self, command: &str, args: &[&str]) -> io::Result<Output> {
        let output = self.run(command, args)?;
        if output.status.success() {
            Ok(output)
        } else {
            Err(command_error(command, &output))
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct SystemCommandRunner;

impl CommandRunner for SystemCommandRunner {
    fn run(&self, command: &str, args: &[&str]) -> io::Result<Output> {
        Command::new(command).args(args).output()
    }
}

fn command_error(command: &str, output: &Output) -> io::Error {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        format!("exit status: {}", output.status)
    };

    io::Error::other(format!("{command}: {detail}"))
}
