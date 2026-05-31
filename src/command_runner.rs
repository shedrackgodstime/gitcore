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
        Command::new(command).args(args).output().map_err(|err| {
            if err.kind() == io::ErrorKind::NotFound {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!(
                        "Executable '{}' not found in PATH. Please make sure it is installed.",
                        command
                    ),
                )
            } else {
                err
            }
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_command_runner_not_found() {
        let runner = SystemCommandRunner;
        let err = runner.run("nonexistent-executable-12345", &[]).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
        assert!(
            err.to_string()
                .contains("Executable 'nonexistent-executable-12345' not found in PATH")
        );
    }
}
