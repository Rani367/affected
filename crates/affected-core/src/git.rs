use anyhow::{Context, Result};
use git2::Repository;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::debug;

pub struct GitDiff {
    pub changed_files: Vec<PathBuf>,
    pub repo_root: PathBuf,
}

/// Strip the `\\?\` extended-length path prefix on Windows.
/// libgit2 mishandles these paths, causing discovery and diff failures.
fn normalize_path(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        let s = path.to_string_lossy();
        if let Some(stripped) = s.strip_prefix(r"\\?\") {
            return PathBuf::from(stripped);
        }
    }
    path.to_path_buf()
}

/// Compute changed files between a base ref and HEAD (plus uncommitted changes).
pub fn changed_files(repo_path: &Path, base_ref: &str) -> Result<GitDiff> {
    let normalized = normalize_path(repo_path);
    let repo = Repository::discover(&normalized).context("Not a git repository")?;

    let repo_root = repo
        .workdir()
        .context("Bare repositories are not supported")?
        .to_path_buf();

    debug!("Found git repository at {}", repo_root.display());

    let base_obj = repo
        .revparse_single(base_ref)
        .with_context(|| format!("Could not resolve base ref '{base_ref}'"))?;

    debug!("Resolved base ref '{}' to {}", base_ref, base_obj.id());

    let base_tree = base_obj
        .peel_to_tree()
        .with_context(|| format!("Could not peel '{base_ref}' to a tree"))?;

    let head_ref = repo.head().context("Could not get HEAD")?;
    let head_tree = head_ref
        .peel_to_tree()
        .context("Could not peel HEAD to a tree")?;

    // Refresh index from disk to pick up changes made by the git CLI
    let mut index = repo.index().context("Could not read index")?;
    index.read(true).context("Could not refresh index from disk")?;

    let mut files = HashSet::new();

    // Committed changes: base..HEAD
    let diff_committed = repo
        .diff_tree_to_tree(Some(&base_tree), Some(&head_tree), None)
        .context("Failed to diff base..HEAD")?;

    for delta in diff_committed.deltas() {
        if let Some(p) = delta.new_file().path() {
            files.insert(p.to_path_buf());
        }
        if let Some(p) = delta.old_file().path() {
            files.insert(p.to_path_buf());
        }
    }

    // Uncommitted changes: HEAD vs working tree + index
    let diff_uncommitted = repo
        .diff_tree_to_workdir_with_index(Some(&head_tree), None)
        .context("Failed to diff HEAD vs working tree")?;

    for delta in diff_uncommitted.deltas() {
        if let Some(p) = delta.new_file().path() {
            files.insert(p.to_path_buf());
        }
        if let Some(p) = delta.old_file().path() {
            files.insert(p.to_path_buf());
        }
    }

    let mut changed_files: Vec<PathBuf> = files.into_iter().collect();
    changed_files.sort();

    debug!("Detected {} changed files", changed_files.len());

    Ok(GitDiff {
        changed_files,
        repo_root,
    })
}

/// Compute the merge-base between HEAD and the given branch.
/// Returns the commit SHA as a string. Used when `--merge-base` is passed.
pub fn merge_base(repo_path: &Path, branch: &str) -> Result<String> {
    let normalized = normalize_path(repo_path);
    let repo = Repository::discover(&normalized).context("Not a git repository")?;

    debug!("Computing merge-base between HEAD and '{}'", branch);

    let head_oid = repo
        .head()
        .context("Could not get HEAD")?
        .target()
        .context("HEAD is not a direct reference")?;

    let branch_obj = repo
        .revparse_single(branch)
        .with_context(|| format!("Could not resolve branch '{branch}'"))?;
    let branch_oid = branch_obj.id();

    let merge_base_oid = repo
        .merge_base(head_oid, branch_oid)
        .with_context(|| format!("Could not find merge-base between HEAD and '{branch}'"))?;

    let sha = merge_base_oid.to_string();
    debug!("Merge-base resolved to {}", sha);

    Ok(sha)
}
