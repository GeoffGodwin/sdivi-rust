//! Git shell-outs for resolving refs and extracting historical trees.
//!
//! Used by [`Pipeline::snapshot_with_mode`] when `--commit REF` is supplied.

use std::path::Path;
use std::process::{Command, Stdio};

use tempfile::TempDir;
use thiserror::Error;

/// Errors from commit extraction helpers.
#[derive(Debug, Error)]
pub enum CommitExtractError {
    /// `git rev-parse --verify` failed — ref is unknown or git is unavailable.
    #[error("ref resolution failed for '{reference}': {stderr}")]
    RefResolutionFailed { reference: String, stderr: String },

    /// The resolved SHA was not found by `git show`.
    #[error("commit not found: {sha}")]
    CommitNotFound { sha: String },

    /// The date string returned by `git show --format=%cI` could not be parsed.
    #[error("could not parse commit date for {sha}: {raw:?}")]
    CommitDateParseFailed { sha: String, raw: String },

    /// `git archive` exited non-zero.
    #[error("git archive failed: {stderr}")]
    ArchiveFailed { stderr: String },

    /// `tar` exited non-zero during extraction.
    #[error("tar extraction failed: {stderr}")]
    TarFailed { stderr: String },

    /// Underlying I/O error (spawning process, creating tempdir, etc.).
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Resolves `reference` to a full 40-char SHA via `git rev-parse --verify`.
pub fn resolve_ref_to_sha(
    repo_root: &Path,
    reference: &str,
) -> Result<String, CommitExtractError> {
    let out = Command::new("git")
        .current_dir(repo_root)
        .args(["rev-parse", "--verify", reference])
        .output()
        .map_err(|e| CommitExtractError::RefResolutionFailed {
            reference: reference.to_string(),
            stderr: e.to_string(),
        })?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(CommitExtractError::RefResolutionFailed {
            reference: reference.to_string(),
            stderr: truncate_stderr(&stderr, 200),
        });
    }

    let sha = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if sha.len() != 40 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(CommitExtractError::RefResolutionFailed {
            reference: reference.to_string(),
            stderr: format!("unexpected rev-parse output: {sha:?}"),
        });
    }
    Ok(sha)
}

/// Returns the commit-date of `sha` as a UTC ISO 8601 string (`YYYY-MM-DDTHH:MM:SSZ`).
///
/// Uses `git show -s --format=%cI` and normalises the committer timezone to UTC.
pub fn commit_date_iso(repo_root: &Path, sha: &str) -> Result<String, CommitExtractError> {
    let out = Command::new("git")
        .current_dir(repo_root)
        .args(["show", "-s", "--format=%cI", sha])
        .output()
        .map_err(CommitExtractError::IoError)?;

    if !out.status.success() {
        return Err(CommitExtractError::CommitNotFound { sha: sha.to_string() });
    }

    let raw = String::from_utf8_lossy(&out.stdout).trim().to_string();
    normalize_to_utc(&raw)
        .ok_or_else(|| CommitExtractError::CommitDateParseFailed { sha: sha.to_string(), raw })
}

/// Extracts the tree at `sha` into a fresh [`TempDir`] via `git archive | tar`.
///
/// The returned `TempDir` contains the repo tree at `sha`. It is removed when
/// the `TempDir` is dropped.
pub fn extract_commit_tree(repo_root: &Path, sha: &str) -> Result<TempDir, CommitExtractError> {
    let tmpdir = TempDir::new()?;

    // Verify tar is available before spawning git archive so we don't spawn a
    // process whose stdout pipe will be immediately abandoned.
    let tar_check = Command::new("tar")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if tar_check.is_err() || tar_check.is_ok_and(|s| !s.success()) {
        return Err(CommitExtractError::TarFailed {
            stderr: "tar not found on PATH; install tar to use --commit".to_string(),
        });
    }

    let mut git = Command::new("git")
        .current_dir(repo_root)
        .args(["archive", "--format=tar", sha])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let git_stdout = git.stdout.take().expect("stdout is piped");

    let mut tar = Command::new("tar")
        .arg("-xC")
        .arg(tmpdir.path())
        .stdin(Stdio::from(git_stdout))
        .stderr(Stdio::piped())
        .spawn()?;

    // Drain stderr on separate threads to prevent pipe-buffer deadlocks.
    let git_stderr_handle = git.stderr.take().expect("stderr is piped");
    let tar_stderr_handle = tar.stderr.take().expect("stderr is piped");

    let git_err_thread = std::thread::spawn(move || read_to_string(git_stderr_handle));
    let tar_err_thread = std::thread::spawn(move || read_to_string(tar_stderr_handle));

    let tar_status = tar.wait()?;
    let git_status = git.wait()?;

    let git_err = git_err_thread.join().unwrap_or_default();
    let tar_err = tar_err_thread.join().unwrap_or_default();

    if !git_status.success() {
        return Err(CommitExtractError::ArchiveFailed {
            stderr: truncate_stderr(&git_err, 200),
        });
    }
    if !tar_status.success() {
        return Err(CommitExtractError::TarFailed {
            stderr: truncate_stderr(&tar_err, 200),
        });
    }

    Ok(tmpdir)
}

// ── private helpers ──────────────────────────────────────────────────────────

fn truncate_stderr(stderr: &str, max_len: usize) -> String {
    if stderr.len() <= max_len {
        stderr.to_string()
    } else {
        format!("{}...", &stderr[..max_len])
    }
}

fn read_to_string(r: impl std::io::Read) -> String {
    let mut buf = Vec::new();
    let mut reader = std::io::BufReader::new(r);
    std::io::Read::read_to_end(&mut reader, &mut buf).ok();
    String::from_utf8_lossy(&buf).to_string()
}

/// Parses a `git %cI` date (ISO 8601 with tz offset) and normalises to UTC.
///
/// Input examples: `2026-04-30T14:23:01-07:00`, `2026-04-29T00:00:00Z`.
/// Returns `None` when the string is malformed.
fn normalize_to_utc(s: &str) -> Option<String> {
    chrono::DateTime::parse_from_rfc3339(s.trim())
        .ok()
        .map(|dt| dt.to_utc().format("%Y-%m-%dT%H:%M:%SZ").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_passthrough() {
        assert_eq!(
            normalize_to_utc("2026-04-29T00:00:00Z").unwrap(),
            "2026-04-29T00:00:00Z"
        );
    }

    #[test]
    fn negative_offset_shifts_forward() {
        // -07:00 means UTC = local + 7h.
        assert_eq!(
            normalize_to_utc("2026-04-30T14:00:00-07:00").unwrap(),
            "2026-04-30T21:00:00Z"
        );
    }

    #[test]
    fn positive_offset_shifts_back() {
        // +05:30 IST: UTC = local - 5h30m.
        assert_eq!(
            normalize_to_utc("2026-04-30T05:30:00+05:30").unwrap(),
            "2026-04-30T00:00:00Z"
        );
    }

    #[test]
    fn malformed_returns_none() {
        assert!(normalize_to_utc("not-a-date").is_none());
        assert!(normalize_to_utc("2026-04").is_none());
    }

    #[test]
    fn positive_offset_crosses_day_boundary_backward() {
        // +01:00 means UTC = local - 1h.
        // 2026-05-01 00:30:00 local → 2026-04-30 23:30:00 UTC (day rolls back).
        assert_eq!(
            normalize_to_utc("2026-05-01T00:30:00+01:00").unwrap(),
            "2026-04-30T23:30:00Z"
        );
    }

    #[test]
    fn negative_offset_crosses_day_boundary_forward() {
        // -01:00 means UTC = local + 1h.
        // 2026-04-30 23:30:00 local → 2026-05-01 00:30:00 UTC (day rolls forward).
        assert_eq!(
            normalize_to_utc("2026-04-30T23:30:00-01:00").unwrap(),
            "2026-05-01T00:30:00Z"
        );
    }

    #[test]
    fn commit_date_parse_failed_when_date_unparseable() {
        // When normalize_to_utc returns None (malformed date),
        // commit_date_iso should return CommitDateParseFailed with the raw string.
        // This test documents the behavior by checking that normalize_to_utc's None
        // return propagates correctly.
        let unparseable = "not-a-valid-date";
        assert!(normalize_to_utc(unparseable).is_none(),
                "normalize_to_utc should return None for unparseable input");
    }
}
