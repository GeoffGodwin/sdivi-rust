//! Git shell-out for change-coupling event collection.
//!
//! `collect_cochange_events` is the I/O half of the change-coupling analyzer.
//! The pure-compute half lives in `sdi_core::compute_change_coupling`.

use std::path::Path;

use sdi_core::input::CoChangeEventInput;
use thiserror::Error;

/// Errors from [`collect_cochange_events`].
#[derive(Debug, Error)]
pub enum ChangeCouplingError {
    /// `git log` exited with a non-zero status.
    #[error("git log failed (exit {code}): {stderr}")]
    GitFailed { code: i32, stderr: String },
    /// Output from `git log` could not be decoded as UTF-8.
    #[error("git log output is not valid UTF-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

/// Collects co-change events by shelling out to `git log`.
///
/// Returns an empty `Vec` (not an error) when `repo_root` has no `.git/`
/// directory — missing git history is normal operation (Rule 16-style).
///
/// `ending_at = None` defaults to `HEAD`. Pass a commit SHA for M16's
/// `--commit REF` support.
///
/// Events are returned oldest-first (matching `git log --reverse` order).
pub fn collect_cochange_events(
    repo_root: &Path,
    history_depth: u32,
    ending_at: Option<&str>,
) -> Result<Vec<CoChangeEventInput>, ChangeCouplingError> {
    // Missing .git/ is normal (Rule 16 analog: missing input → silent).
    if !repo_root.join(".git").exists() {
        tracing::info!("no .git directory found at {:?}; skipping change-coupling", repo_root);
        return Ok(vec![]);
    }

    if history_depth == 0 {
        return Ok(vec![]);
    }

    let rev = ending_at.unwrap_or("HEAD");
    let output = std::process::Command::new("git")
        .current_dir(repo_root)
        .args([
            "--no-pager",
            "log",
            "-z",
            "--name-only",
            "--format=%x00COMMIT%x00%H%x00%cI%x00",
            "-n",
            &history_depth.to_string(),
            rev,
        ])
        .output();

    let out = match output {
        Ok(o) => o,
        Err(e) => {
            // git not on PATH — treat as no-history (common in CI containers).
            tracing::info!("could not run git: {e}; skipping change-coupling");
            return Ok(vec![]);
        }
    };

    if !out.status.success() {
        let code = out.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
        return Err(ChangeCouplingError::GitFailed { code, stderr });
    }

    let raw = String::from_utf8(out.stdout)?;
    let events = parse_git_log_output(&raw);

    // git log without --reverse gives newest-first.
    // compute_change_coupling expects oldest-first, so we reverse.
    let mut reversed = events;
    reversed.reverse();
    Ok(reversed)
}

/// Parses the NUL-separated output of `git log -z --name-only --format=%x00COMMIT%x00%H%x00%cI%x00`.
///
/// Each commit block looks like:
///   \0COMMIT\0<sha>\0<date>\0\n\nfile1\nfile2\n\0COMMIT\0...
/// We split on the sentinel `COMMIT` token (surrounded by NULs).
fn parse_git_log_output(raw: &str) -> Vec<CoChangeEventInput> {
    let parts: Vec<&str> = raw.split('\0').collect();
    let mut events = Vec::new();
    let mut i = 0;

    while i < parts.len() {
        if parts[i] == "COMMIT" {
            let sha = parts.get(i + 1).map(|s| s.trim()).unwrap_or("").to_string();
            let date = parts.get(i + 2).map(|s| s.trim()).unwrap_or("").to_string();

            if sha.is_empty() {
                i += 1;
                continue;
            }

            let mut files = Vec::new();

            // Files appear in segments starting at i+3, until the next COMMIT sentinel.
            let mut j = i + 3;
            while j < parts.len() && parts[j] != "COMMIT" {
                for line in parts[j].lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        let canonical = canonicalize_path(trimmed);
                        if !canonical.is_empty() {
                            files.push(canonical);
                        }
                    }
                }
                j += 1;
            }

            if !sha.is_empty() {
                events.push(CoChangeEventInput { commit_sha: sha, commit_date: date, files });
            }
            i = j;
        } else {
            i += 1;
        }
    }

    events
}

/// Converts a git-log path to canonical NodeId form: forward slashes, no leading `./`.
fn canonicalize_path(path: &str) -> String {
    let p = path.replace('\\', "/");
    let p = p.strip_prefix("./").unwrap_or(&p);
    p.to_string()
}
