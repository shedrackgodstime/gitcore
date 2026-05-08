use super::*;

impl Gitcore {
    /// Clones a repository for the selected account or reuses an existing checkout in place.
    ///
    /// # Errors
    /// Returns [`GitcoreError::AccountNotFound`] if the account is missing, or a Git error if
    /// cloning or configuration fails.
    ///
    /// # Examples
    /// ```no_run
    /// # use gitcore::{Gitcore, CloneRequest};
    /// # use std::path::PathBuf;
    /// # let service = Gitcore::new();
    /// let request = CloneRequest {
    ///     account_name: "work".to_string(),
    ///     repo_url: "github.com/acme/project.git".to_string(),
    ///     working_dir: PathBuf::from("./worktrees"),
    /// };
    /// let report = service.clone_repository(request)?;
    /// println!("Repo cloned to: {:?}", report.repo_path);
    /// # Ok::<(), gitcore::GitcoreError>(())
    /// ```
    pub fn clone_repository(&self, request: CloneRequest) -> Result<CloneReport> {
        let account = self.find_account(&request.account_name)?;
        let remote_url = git::convert_to_host(&request.repo_url, &account.host_alias);
        let repo_path = repository_path(&request.working_dir, &request.repo_url);
        fs::create_dir_all(&request.working_dir)?;

        let reused_existing_repo = if repo_path.exists() {
            if !git::ensure_git_repository_with(self.runner(), &repo_path) {
                return Err(GitcoreError::NotGitRepository(repo_path));
            }
            true
        } else {
            git::clone_repository_in_worktree_with(
                self.runner(),
                &request.working_dir,
                &remote_url,
            )?;
            false
        };

        if reused_existing_repo {
            git::attach_origin_remote_with(self.runner(), &repo_path, &remote_url)?;
        }

        git::configure_repository_identity_with(self.runner(), &repo_path, git_identity(&account))?;

        Ok(CloneReport {
            repo_path,
            remote_url,
            username: account.username,
            email: account.email,
            reused_existing_repo,
        })
    }

    /// Rewrites a repository remote for the selected account and applies its Git identity.
    ///
    /// # Errors
    /// Returns [`GitcoreError::NotGitRepository`] if the path is not a Git repo, or
    /// [`GitcoreError::AccountNotFound`] if the account is missing.
    pub fn add_remote(&self, request: RemoteAddRequest) -> Result<RemoteReport> {
        let account = self.find_account(&request.account_name)?;
        if !git::ensure_git_repository_with(self.runner(), &request.repo_path) {
            return Err(GitcoreError::NotGitRepository(request.repo_path));
        }

        let remote_url = git::convert_to_host(&request.repo_url, &account.host_alias);
        git::configure_repository_identity_with(
            self.runner(),
            &request.repo_path,
            git_identity(&account),
        )?;
        git::attach_origin_remote_with(self.runner(), &request.repo_path, &remote_url)?;

        Ok(RemoteReport {
            repo_path: request.repo_path,
            remote_url,
            username: account.username,
            email: account.email,
        })
    }

    /// Switches an existing repository's `origin` to the selected account's host alias.
    ///
    /// # Errors
    /// Returns [`GitcoreError::MissingOriginRemote`] if the repository does not have an
    /// origin remote to switch.
    pub fn switch_remote(&self, request: RemoteSwitchRequest) -> Result<RemoteReport> {
        let account = self.find_account(&request.account_name)?;
        if !git::ensure_git_repository_with(self.runner(), &request.repo_path) {
            return Err(GitcoreError::NotGitRepository(request.repo_path));
        }

        let current_remote = git::get_origin_remote_with(self.runner(), &request.repo_path)
            .map_err(|_| GitcoreError::MissingOriginRemote(request.repo_path.clone()))?;
        let remote_url = git::convert_to_host(current_remote.as_str(), &account.host_alias);

        git::set_origin_remote_with(self.runner(), &request.repo_path, &remote_url)?;
        git::configure_repository_identity_with(
            self.runner(),
            &request.repo_path,
            git_identity(&account),
        )?;

        Ok(RemoteReport {
            repo_path: request.repo_path,
            remote_url,
            username: account.username,
            email: account.email,
        })
    }

    /// Detects which managed account (if any) is currently used in the repository at `path`.
    ///
    /// # Errors
    /// Returns an error if the path is not a Git repository or cannot be accessed.
    pub fn detect_account_in_repository(&self, path: &Path) -> Result<Option<Account>> {
        if !git::ensure_git_repository_with(self.runner(), path) {
            return Err(GitcoreError::NotGitRepository(path.to_path_buf()));
        }

        let remote_url = match git::get_origin_remote_with(self.runner(), path) {
            Ok(url) => url,
            Err(_) => return Ok(None),
        };

        let config = self.load_config()?;
        let account = config.accounts.into_iter().find(|acc| {
            remote_url
                .as_str()
                .starts_with(&format!("{}:", acc.host_alias))
                || remote_url
                    .as_str()
                    .starts_with(&format!("git@{}:", acc.host_alias))
        });

        Ok(account)
    }
}

fn git_identity(account: &Account) -> git::GitIdentityConfig<'_> {
    git::GitIdentityConfig {
        username: &account.username,
        email: &account.email,
        gpg_key_id: account.gpg_key_id.as_deref(),
    }
}
