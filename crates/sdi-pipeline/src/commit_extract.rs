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
        return Err(CommitExtractError::RefResolutionFailed {
            reference: reference.to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
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
        return Err(CommitExtractError::ArchiveFailed { stderr: git_err });
    }
    if !tar_status.success() {
        return Err(CommitExtractError::TarFailed { stderr: tar_err });
    }

    Ok(tmpdir)
}

// ── private helpers ──────────────────────────────────────────────────────────

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
    let s = s.trim();
    if s.len() < 19 {
        return None;
    }
    let year: i64 = s[0..4].parse().ok()?;
    let month: i64 = s[5..7].parse().ok()?;
    let day: i64 = s[8..10].parse().ok()?;
    let hour: i64 = s[11..13].parse().ok()?;
    let min: i64 = s[14..16].parse().ok()?;
    let sec: i64 = s[17..19].parse().ok()?;

    let offset_secs: i64 = match s.get(19..) {
        None | Some("Z") | Some("") => 0,
        Some(tz) => {
            let sign: i64 = if tz.starts_with('+') { 1 } else { -1 };
            let oh: i64 = tz.get(1..3).and_then(|x| x.parse().ok())?;
            let om: i64 = tz.get(4..6).and_then(|x| x.parse().ok())?;
            sign * (oh * 3600 + om * 60)
        }
    };

    let epoch = calendar_to_epoch(year, month, day, hour, min, sec) - offset_secs;
    Some(epoch_to_iso8601(epoch))
}

fn calendar_to_epoch(y: i64, m: i64, d: i64, h: i64, mn: i64, s: i64) -> i64 {
    let a = (14 - m) / 12;
    let yr = y + 4800 - a;
    let mo = m + 12 * a - 3;
    let jdn = d + (153 * mo + 2) / 5 + 365 * yr + yr / 4 - yr / 100 + yr / 400 - 32045;
    (jdn - 2_440_588) * 86400 + h * 3600 + mn * 60 + s
}

fn epoch_to_iso8601(secs: i64) -> String {
    let days = secs.div_euclid(86400);
    let time = secs.rem_euclid(86400);
    let j = days + 2_440_588;
    let f = j + 1401 + (((4 * j + 274_277) / 146_097) * 3) / 4 - 38;
    let e = 4 * f + 3;
    let g = (e % 1461) / 4;
    let h = 5 * g + 2;
    let day = (h % 153) / 5 + 1;
    let month = (h / 153 + 2) % 12 + 1;
    let year = e / 1461 - 4716 + (14 - month) / 12;
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        time / 3600,
        (time % 3600) / 60,
        time % 60,
    )
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
}
