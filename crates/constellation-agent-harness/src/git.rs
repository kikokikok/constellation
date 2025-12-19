use git2::{Repository, Commit, Signature, ErrorCode};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct RepositoryState {
    pub path: PathBuf,
    pub branch: String,
    pub last_commit: String,
    pub has_uncommitted_changes: bool,
    pub staged_files: Vec<PathBuf>,
    pub unstaged_files: Vec<PathBuf>,
    pub untracked_files: Vec<PathBuf>,
}

pub struct GitManager {
    repo: Repository,
}

impl GitManager {
    pub fn open(path: &Path) -> Result<Self> {
        let repo = Repository::open(path)
            .map_err(|e| Error::RepositoryNotInitialized(e.to_string()))?;
        
        Ok(Self { repo })
    }

    pub fn init(path: &Path) -> Result<Self> {
        let repo = Repository::init(path)
            .map_err(|e| Error::InitializationFailed(e.to_string()))?;
        
        Ok(Self { repo })
    }

    pub fn get_state(&self) -> Result<RepositoryState> {
        let head = self.repo.head().ok();
        let branch = head
            .as_ref()
            .and_then(|h| h.shorthand())
            .unwrap_or("main")
            .to_string();
        
        let last_commit = head
            .as_ref()
            .and_then(|h| h.target())
            .map(|oid| oid.to_string())
            .unwrap_or_default();
        
        let mut status_options = git2::StatusOptions::new();
        status_options
            .show(git2::StatusShow::IndexAndWorkdir)
            .include_untracked(true)
            .renames_head_to_index(true)
            .renames_index_to_workdir(true);
        
        let statuses = self.repo.statuses(Some(&mut status_options))?;
        
        let mut staged_files = Vec::new();
        let mut unstaged_files = Vec::new();
        let mut untracked_files = Vec::new();
        let mut has_uncommitted_changes = false;
        
        for entry in statuses.iter() {
            let status = entry.status();
            let path = entry.path().unwrap_or("");
            
            if status.is_index_new() || status.is_index_modified() || status.is_index_deleted() || status.is_index_renamed() || status.is_index_typechange() {
                staged_files.push(PathBuf::from(path));
                has_uncommitted_changes = true;
            }
            
            if status.is_wt_new() {
                untracked_files.push(PathBuf::from(path));
                has_uncommitted_changes = true;
            } else if status.is_wt_modified() || status.is_wt_deleted() || status.is_wt_renamed() || status.is_wt_typechange() {
                unstaged_files.push(PathBuf::from(path));
                has_uncommitted_changes = true;
            }
        }
        
        Ok(RepositoryState {
            path: self.repo.path().parent().unwrap_or(Path::new(".")).to_path_buf(),
            branch,
            last_commit,
            has_uncommitted_changes,
            staged_files,
            unstaged_files,
            untracked_files,
        })
    }

    pub fn commit(&self, message: &str, author_name: &str, author_email: &str) -> Result<String> {
        let mut index = self.repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;
        
        let head = self.repo.head();
        let parent_commit = head.ok().and_then(|h| h.peel_to_commit().ok());
        
        let signature = Signature::now(author_name, author_email)?;
        
        let commit_id = self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parent_commit
                .as_ref()
                .map(|c| vec![c])
                .unwrap_or_else(Vec::new)
                .as_slice(),
        )?;
        
        Ok(commit_id.to_string())
    }

    pub fn add_all(&self) -> Result<()> {
        let mut index = self.repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(())
    }

    pub fn revert_to_last_good_state(&self) -> Result<()> {
        let head = self.repo.head()?;
        let commit = head.peel_to_commit()?;
        
        self.repo.reset(
            &commit.into_object(),
            git2::ResetType::Hard,
            None,
        )?;
        
        Ok(())
    }

    pub fn create_branch(&self, name: &str) -> Result<()> {
        let head = self.repo.head()?;
        let commit = head.peel_to_commit()?;
        self.repo.branch(name, &commit, false)?;
        Ok(())
    }

    pub fn checkout_branch(&self, name: &str) -> Result<()> {
        let obj = self.repo.revparse_single(&format!("refs/heads/{}", name))?;
        self.repo.checkout_tree(&obj, None)?;
        
        self.repo.set_head(&format!("refs/heads/{}", name))?;
        Ok(())
    }

    pub fn get_commit_history(&self, limit: usize) -> Result<Vec<CommitInfo>> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.set_sorting(git2::Sort::TIME)?;
        revwalk.push_head()?;
        
        let mut commits = Vec::new();
        for (i, oid) in revwalk.enumerate() {
            if i >= limit {
                break;
            }
            
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;
            
            let info = CommitInfo {
                id: oid.to_string(),
                author: commit.author().name().unwrap_or("").to_string(),
                message: commit.message().unwrap_or("").to_string(),
                timestamp: DateTime::from_timestamp(commit.time().seconds(), 0)
                    .unwrap_or_else(Utc::now),
            };
            
            commits.push(info);
        }
        
        Ok(commits)
    }

    pub fn is_clean(&self) -> Result<bool> {
        let state = self.get_state()?;
        Ok(!state.has_uncommitted_changes && state.staged_files.is_empty())
    }
}

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub id: String,
    pub author: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

impl std::fmt::Display for CommitInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} - {}",
            &self.id[..8],
            self.author,
            self.message.lines().next().unwrap_or("")
        )
    }
}
