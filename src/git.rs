use std::io;
use std::path::Path;
use std::process::{Command, Output};

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

pub fn run_command(command: &str, args: &[&str]) -> io::Result<Output> {
    let output = Command::new(command).args(args).output()?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(command_error(command, &output))
    }
}

pub fn run_git(args: &[&str]) -> io::Result<Output> {
    run_command("git", args)
}

pub fn ensure_git_repository(path: &Path) -> bool {
    run_git(&["-C", path.to_str().unwrap_or("."), "rev-parse", "--git-dir"]).is_ok()
}

pub fn convert_to_host(url: &str, host_alias: &str) -> String {
    let url = url.trim().trim_end_matches(".git").to_string();

    if let Some((_host, rest)) = url.strip_prefix("git@").and_then(|p| p.split_once(':')) {
        return format!("{}:{}", host_alias, rest);
    }

    if url.starts_with("https://") || url.starts_with("http://") {
        let path = url
            .strip_prefix("https://")
            .or(url.strip_prefix("http://"))
            .unwrap_or(&url);
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if parts.len() >= 2
            && (parts[0].contains("github.com")
                || parts[0].contains("gitlab.com")
                || parts[0].contains("codeberg.org")
                || parts[0].contains("bitbucket.org"))
        {
            let repo = parts[1..].join("/");
            return format!("{}:{}", host_alias, repo);
        }
    }

    if let Some(rest) = url.strip_prefix("ssh://") {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() >= 2
            && (parts[0].contains("github.com")
                || parts[0].contains("gitlab.com")
                || parts[0].contains("codeberg.org")
                || parts[0].contains("bitbucket.org"))
        {
            let repo = parts[1..].join("/");
            return format!("{}:{}", host_alias, repo);
        }
    }

    if !url.contains('@') && !url.starts_with("http") {
        if url.contains("github.com/")
            || url.contains("gitlab.com/")
            || url.contains("codeberg.org/")
            || url.contains("bitbucket.org/")
        {
            let parts: Vec<&str> = url.split('/').collect();
            let idx = parts.iter().position(|p| {
                p.contains("github.com")
                    || p.contains("gitlab.com")
                    || p.contains("codeberg.org")
                    || p.contains("bitbucket.org")
            });
            if let Some(i) = idx {
                let repo = parts[i + 1..].join("/");
                return format!("{}:{}", host_alias, repo);
            }
        }
        format!("{}:{}", host_alias, url)
    } else {
        url.to_string()
    }
}

pub fn set_git_config(username: &str, email: &str, gpg_key_id: Option<&str>) -> io::Result<()> {
    run_git(&["config", "user.name", username])?;
    run_git(&["config", "user.email", email])?;

    if let Some(key_id) = gpg_key_id {
        run_git(&["config", "user.signingkey", key_id])?;
        run_git(&["config", "commit.gpgsign", "true"])?;
    } else {
        // Optional: disable signing if no key is provided,
        // but usually we just leave existing config alone if not explicitly managing it.
    }
    Ok(())
}

pub fn run_git_remote_add(url: &str) -> io::Result<()> {
    match run_git(&["remote", "add", "origin", url]) {
        Ok(_) => Ok(()),
        Err(add_err) => match run_git(&["remote", "set-url", "origin", url]) {
            Ok(_) => Ok(()),
            Err(set_err) => Err(io::Error::other(format!(
                "{}; fallback set-url also failed: {}",
                add_err, set_err
            ))),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::convert_to_host;

    #[test]
    fn converts_https_provider_url_to_host_alias() {
        let converted = convert_to_host("https://github.com/acme/project.git", "github-work");
        assert_eq!(converted, "github-work:acme/project");
    }

    #[test]
    fn converts_ssh_provider_url_to_host_alias() {
        let converted = convert_to_host("git@github.com:acme/project.git", "github-work");
        assert_eq!(converted, "github-work:acme/project");
    }

    #[test]
    fn converts_plain_owner_repo_to_host_alias() {
        let converted = convert_to_host("acme/project.git", "github-work");
        assert_eq!(converted, "github-work:acme/project");
    }

    #[test]
    fn converts_ssh_url_with_known_host() {
        let converted = convert_to_host(
            "ssh://git@codeberg.org/manicoproject/wiki.git",
            "codeberg-manico",
        );
        assert_eq!(converted, "codeberg-manico:manicoproject/wiki");
    }

    #[test]
    fn preserves_non_provider_ssh_urls() {
        let converted = convert_to_host("ssh://git@example.com/acme/project.git", "github-work");
        assert_eq!(converted, "ssh://git@example.com/acme/project");
    }

    #[test]
    fn converts_existing_alias_to_new_alias() {
        let converted = convert_to_host("git@github-old:acme/project.git", "github-work");
        assert_eq!(converted, "github-work:acme/project");
    }

    #[test]
    fn converts_provider_path_without_scheme() {
        let converted = convert_to_host("github.com/acme/project.git", "github-work");
        assert_eq!(converted, "github-work:acme/project");
    }
}
