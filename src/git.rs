use crate::command_runner::CommandRunner;
use std::io;
use std::path::Path;
use std::process::Output;

pub(crate) struct GitIdentityConfig<'a> {
    pub(crate) username: &'a str,
    pub(crate) email: &'a str,
    pub(crate) gpg_key_id: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GitRemoteMutation {
    Added,
    Updated,
}

pub(crate) fn run_command_with(
    runner: &dyn CommandRunner,
    command: &str,
    args: &[&str],
) -> io::Result<Output> {
    runner.run_checked(command, args)
}

pub(crate) fn run_git_with(runner: &dyn CommandRunner, args: &[&str]) -> io::Result<Output> {
    run_command_with(runner, "git", args)
}

pub(crate) fn run_git_in_with(
    runner: &dyn CommandRunner,
    path: &Path,
    args: &[&str],
) -> io::Result<Output> {
    let mut git_args = vec!["-C", path.to_str().unwrap_or(".")];
    git_args.extend_from_slice(args);
    run_git_with(runner, &git_args)
}

pub(crate) fn ensure_git_repository_with(runner: &dyn CommandRunner, path: &Path) -> bool {
    run_git_in_with(runner, path, &["rev-parse", "--git-dir"]).is_ok()
}

pub(crate) fn init_repository_with(runner: &dyn CommandRunner, path: &Path) -> io::Result<()> {
    run_git_in_with(runner, path, &["init"]).map(|_| ())
}

pub fn convert_to_host(url: &str, host_alias: &str) -> String {
    let url = url.trim().trim_end_matches(".git");

    if let Some((_host, rest)) = url.strip_prefix("git@").and_then(|p| p.split_once(':')) {
        return format!("{host_alias}:{rest}");
    }

    if let Some(path) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
    {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if parts.len() >= 2 && is_known_provider(parts[0]) {
            let repo = parts[1..].join("/");
            return format!("{host_alias}:{repo}");
        }
    }

    if let Some(rest) = url.strip_prefix("ssh://") {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() >= 2 && is_known_provider(parts[0]) {
            let repo = parts[1..].join("/");
            return format!("{host_alias}:{repo}");
        }
    }

    if !url.contains('@') && !url.contains("://") {
        let parts: Vec<&str> = url.split('/').collect();
        if let Some(i) = parts
            .iter()
            .position(|p| is_known_provider(p))
            .filter(|&i| i + 1 < parts.len())
        {
            let repo = parts[i + 1..].join("/");
            return format!("{host_alias}:{repo}");
        }
        format!("{host_alias}:{url}")
    } else {
        url.to_string()
    }
}

fn is_known_provider(host: &str) -> bool {
    host.contains("github.com")
        || host.contains("gitlab.com")
        || host.contains("codeberg.org")
        || host.contains("bitbucket.org")
}

pub(crate) fn clone_repository_in_worktree_with(
    runner: &dyn CommandRunner,
    working_dir: &Path,
    remote_url: &str,
) -> io::Result<()> {
    run_git_in_with(runner, working_dir, &["clone", remote_url]).map(|_| ())
}

pub(crate) fn configure_repository_identity_with(
    runner: &dyn CommandRunner,
    repo_path: &Path,
    identity: GitIdentityConfig<'_>,
) -> io::Result<()> {
    run_git_in_with(
        runner,
        repo_path,
        &["config", "user.name", identity.username],
    )?;
    run_git_in_with(runner, repo_path, &["config", "user.email", identity.email])?;

    if let Some(key_id) = identity.gpg_key_id {
        run_git_in_with(runner, repo_path, &["config", "user.signingkey", key_id])?;
        run_git_in_with(runner, repo_path, &["config", "commit.gpgsign", "true"])?;
    }

    Ok(())
}

pub(crate) fn attach_origin_remote_with(
    runner: &dyn CommandRunner,
    repo_path: &Path,
    url: &str,
) -> io::Result<GitRemoteMutation> {
    match run_git_in_with(runner, repo_path, &["remote", "add", "origin", url]) {
        Ok(_) => Ok(GitRemoteMutation::Added),
        Err(add_err) => match set_origin_remote_with(runner, repo_path, url) {
            Ok(_) => Ok(GitRemoteMutation::Updated),
            Err(set_err) => Err(io::Error::other(format!(
                "{}; fallback set-url also failed: {}",
                add_err, set_err
            ))),
        },
    }
}

pub(crate) fn set_origin_remote_with(
    runner: &dyn CommandRunner,
    repo_path: &Path,
    url: &str,
) -> io::Result<()> {
    run_git_in_with(runner, repo_path, &["remote", "set-url", "origin", url]).map(|_| ())
}

pub(crate) struct GitRemoteUrl {
    value: String,
}

impl GitRemoteUrl {
    pub(crate) fn as_str(&self) -> &str {
        &self.value
    }
}

pub(crate) fn get_origin_remote_with(
    runner: &dyn CommandRunner,
    repo_path: &Path,
) -> io::Result<GitRemoteUrl> {
    let output = run_git_in_with(runner, repo_path, &["remote", "get-url", "origin"])?;
    Ok(GitRemoteUrl {
        value: String::from_utf8_lossy(&output.stdout).trim().to_string(),
    })
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
