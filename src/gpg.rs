use crate::command_runner::{CommandRunner, SystemCommandRunner};
use std::io;

pub struct GpgKey {
    pub id: String,
    pub name: String,
    pub email: String,
}

pub fn list_gpg_keys() -> io::Result<Vec<GpgKey>> {
    list_gpg_keys_with(&SystemCommandRunner)
}

pub(crate) fn list_gpg_keys_with(runner: &dyn CommandRunner) -> io::Result<Vec<GpgKey>> {
    let output = match runner.run("gpg", &["--list-secret-keys", "--with-colons"]) {
        Ok(out) => out,
        Err(_) => return Ok(Vec::new()),
    };

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let content = String::from_utf8_lossy(&output.stdout);
    let mut keys = Vec::new();
    let mut current_id = String::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.is_empty() {
            continue;
        }

        if parts[0] == "sec" {
            if parts.len() > 4 {
                current_id = parts[4].to_string();
            }
        } else if parts[0] == "uid" && !current_id.is_empty() && parts.len() > 9 {
            let identity = parts[9];
            if let (Some(email_start), Some(email_end)) = (identity.find('<'), identity.find('>')) {
                let name = identity[..email_start].trim().to_string();
                let email = identity[email_start + 1..email_end].trim().to_string();

                keys.push(GpgKey {
                    id: current_id.clone(),
                    name,
                    email,
                });
                current_id = String::new();
            }
        }
    }

    Ok(keys)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCommandRunner {
        stdout: String,
        status_success: bool,
    }

    impl CommandRunner for MockCommandRunner {
        fn run(&self, _command: &str, _args: &[&str]) -> io::Result<std::process::Output> {
            #[cfg(unix)]
            use std::os::unix::process::ExitStatusExt;
            #[cfg(windows)]
            use std::os::windows::process::ExitStatusExt;
            use std::process::ExitStatus;

            Ok(std::process::Output {
                status: ExitStatus::from_raw(if self.status_success { 0 } else { 1 }),
                stdout: self.stdout.as_bytes().to_vec(),
                stderr: Vec::new(),
            })
        }
    }

    #[test]
    fn test_list_gpg_keys_parsing() {
        let sample_output = "\
sec:u:3072:1:C57492EDC2E97855:1584617173:::u:::scESC::::::23::
uid:u::::1584617173::41CD10AD71E738597F876617C57492EDC2E97855::John Doe <john@doe.com>::::::::::
sec:u:4096:1:AABBCCDDEEFF0011:1584617174:::u:::scESC::::::23::
uid:u::::1584617174::41CD10AD71E738597F876617C57492EDC2E97855::Alice Smith <alice@example.com>::::::::::
";
        let runner = MockCommandRunner {
            stdout: sample_output.to_string(),
            status_success: true,
        };

        let keys = list_gpg_keys_with(&runner).unwrap();
        assert_eq!(keys.len(), 2);

        assert_eq!(keys[0].id, "C57492EDC2E97855");
        assert_eq!(keys[0].name, "John Doe");
        assert_eq!(keys[0].email, "john@doe.com");

        assert_eq!(keys[1].id, "AABBCCDDEEFF0011");
        assert_eq!(keys[1].name, "Alice Smith");
        assert_eq!(keys[1].email, "alice@example.com");
    }
}
