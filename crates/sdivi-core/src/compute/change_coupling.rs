//! [`compute_change_coupling`] — pure change-coupling analysis.

use std::collections::{BTreeMap, BTreeSet};

use sdivi_snapshot::change_coupling::{ChangeCouplingResult, CoChangePair};

use crate::error::AnalysisError;
use crate::input::{ChangeCouplingConfigInput, CoChangeEventInput};

/// Computes file-pair co-change frequencies from a slice of commit events.
///
/// Pure function — no I/O, no clock, referentially transparent.
///
/// # Algorithm
///
/// 1. Take the trailing `history_depth` events (oldest-first slice).
/// 2. For each commit, enumerate every unordered pair of files.
/// 3. A pair must appear in at least 2 distinct commits (`cochange_count >= 2`).
/// 4. Emit pairs with `frequency = cochange_count / commits_analyzed` that
///    are `>= min_frequency`.
/// 5. Output is sorted by `(source, target)` for byte-identical determinism.
///
/// # Errors
///
/// Returns [`AnalysisError::InvalidConfig`] if `min_frequency` is outside `[0.0, 1.0]`.
///
/// # Examples
///
/// ```rust
/// use sdivi_core::input::{CoChangeEventInput, ChangeCouplingConfigInput};
/// use sdivi_core::compute::change_coupling::compute_change_coupling;
///
/// let events = vec![
///     CoChangeEventInput {
///         commit_sha: "a".to_string(),
///         commit_date: "2026-01-01T00:00:00Z".to_string(),
///         files: vec!["src/a.rs".to_string(), "src/b.rs".to_string()],
///     },
///     CoChangeEventInput {
///         commit_sha: "b".to_string(),
///         commit_date: "2026-01-02T00:00:00Z".to_string(),
///         files: vec!["src/a.rs".to_string(), "src/b.rs".to_string()],
///     },
/// ];
/// let cfg = ChangeCouplingConfigInput { min_frequency: 0.5, history_depth: 500 };
/// let result = compute_change_coupling(&events, &cfg).unwrap();
/// assert_eq!(result.pairs.len(), 1);
/// assert_eq!(result.pairs[0].source, "src/a.rs");
/// assert_eq!(result.pairs[0].cochange_count, 2);
/// ```
pub fn compute_change_coupling(
    events: &[CoChangeEventInput],
    cfg: &ChangeCouplingConfigInput,
) -> Result<ChangeCouplingResult, AnalysisError> {
    if cfg.min_frequency < 0.0 || cfg.min_frequency > 1.0 {
        return Err(AnalysisError::InvalidConfig {
            message: format!(
                "min_frequency must be in [0.0, 1.0], got {}",
                cfg.min_frequency
            ),
        });
    }

    let history = cfg.history_depth as usize;
    let window: &[CoChangeEventInput] = if events.len() > history {
        &events[events.len() - history..]
    } else {
        events
    };

    let commits_analyzed = window.len() as u32;

    if commits_analyzed == 0 {
        return Ok(ChangeCouplingResult {
            pairs: vec![],
            commits_analyzed: 0,
            distinct_files_touched: 0,
        });
    }

    // Count co-occurrences per pair.
    let mut pair_counts: BTreeMap<(String, String), u32> = BTreeMap::new();
    let mut all_files: BTreeSet<String> = BTreeSet::new();

    for event in window {
        for f in &event.files {
            all_files.insert(f.clone());
        }
        // Enumerate every unordered pair in this commit.
        let mut sorted_files = event.files.clone();
        sorted_files.sort_unstable();
        sorted_files.dedup();
        for i in 0..sorted_files.len() {
            for j in (i + 1)..sorted_files.len() {
                // sorted_files is sorted, so i < j guarantees sorted_files[i] < sorted_files[j].
                let key = (sorted_files[i].clone(), sorted_files[j].clone());
                *pair_counts.entry(key).or_insert(0) += 1;
            }
        }
    }

    let distinct_files_touched = all_files.len() as u32;
    let denom = commits_analyzed as f64;

    let mut pairs: Vec<CoChangePair> = pair_counts
        .into_iter()
        .filter_map(|((source, target), count)| {
            if count < 2 {
                return None;
            }
            let frequency = count as f64 / denom;
            if frequency < cfg.min_frequency {
                return None;
            }
            Some(CoChangePair {
                source,
                target,
                frequency,
                cochange_count: count,
            })
        })
        .collect();

    // Sort by (source, target) for byte-identical determinism.
    pairs.sort_unstable_by(|a, b| (&a.source, &a.target).cmp(&(&b.source, &b.target)));

    Ok(ChangeCouplingResult {
        pairs,
        commits_analyzed,
        distinct_files_touched,
    })
}
