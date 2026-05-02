//! Change-coupling result types stored in [`super::snapshot::Snapshot`].

use serde::{Deserialize, Serialize};

/// A single file-pair that co-changes above the configured minimum frequency.
///
/// # Examples
///
/// ```rust
/// use sdivi_snapshot::change_coupling::CoChangePair;
///
/// let pair = CoChangePair {
///     source: "src/a.rs".to_string(),
///     target: "src/b.rs".to_string(),
///     frequency: 0.8,
///     cochange_count: 4,
/// };
/// assert!(pair.source < pair.target);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoChangePair {
    /// The lexicographically smaller file path (always `source < target`).
    pub source: String,
    /// The lexicographically larger file path.
    pub target: String,
    /// Co-change frequency: `cochange_count / commits_analyzed`.
    pub frequency: f64,
    /// Number of commits in the analysis window that touched both files.
    pub cochange_count: u32,
}

/// Result of the change-coupling analysis for one snapshot.
///
/// `None` in the snapshot when the repo has no git history or
/// `history_depth = 0` was configured.
///
/// # Examples
///
/// ```rust
/// use sdivi_snapshot::change_coupling::ChangeCouplingResult;
///
/// let result = ChangeCouplingResult {
///     pairs: vec![],
///     commits_analyzed: 0,
///     distinct_files_touched: 0,
/// };
/// assert_eq!(result.commits_analyzed, 0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChangeCouplingResult {
    /// File pairs whose co-change frequency meets `min_frequency` and
    /// whose `cochange_count >= 2`. Sorted by `(source, target)`.
    pub pairs: Vec<CoChangePair>,
    /// Number of commits actually analyzed (≤ `history_depth`).
    pub commits_analyzed: u32,
    /// Count of unique file paths that appear in at least one analyzed commit.
    pub distinct_files_touched: u32,
}
