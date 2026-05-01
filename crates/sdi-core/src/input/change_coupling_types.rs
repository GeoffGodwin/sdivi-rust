//! Change-coupling input types for the pure-compute API.

use serde::{Deserialize, Serialize};

/// A single commit event for change-coupling analysis.
///
/// Supplied by the caller (either from `sdi-pipeline`'s `git log` shell-out
/// or from a foreign extractor such as the Meridian VSCode git index reader).
///
/// # Examples
///
/// ```rust
/// use sdi_core::input::CoChangeEventInput;
///
/// let event = CoChangeEventInput {
///     commit_sha: "abc123".to_string(),
///     commit_date: "2026-05-01T00:00:00Z".to_string(),
///     files: vec!["src/a.rs".to_string(), "src/b.rs".to_string()],
/// };
/// assert_eq!(event.files.len(), 2);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoChangeEventInput {
    /// Git commit SHA (hex string).
    pub commit_sha: String,
    /// ISO-8601 UTC commit date (`"YYYY-MM-DDTHH:MM:SSZ"`).
    pub commit_date: String,
    /// Canonical NodeIds of files touched by this commit.
    ///
    /// Paths are repo-relative, forward-slash separated, no leading `./`.
    pub files: Vec<String>,
}

/// Configuration for [`crate::compute_change_coupling`].
///
/// Mirrors [`sdi_config::ChangeCouplingConfig`] with serde derives.
///
/// # Examples
///
/// ```rust
/// use sdi_core::input::ChangeCouplingConfigInput;
///
/// let cfg = ChangeCouplingConfigInput { min_frequency: 0.6, history_depth: 500 };
/// assert_eq!(cfg.history_depth, 500);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChangeCouplingConfigInput {
    /// Minimum co-change frequency (0.0–1.0) for a pair to be emitted.
    ///
    /// A pair must also have `cochange_count >= 2`.
    pub min_frequency: f64,
    /// Maximum number of commits to analyze (trailing window, oldest-first input).
    pub history_depth: u32,
}
