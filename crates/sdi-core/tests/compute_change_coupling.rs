//! Unit tests for compute_change_coupling.

use sdi_core::compute::change_coupling::compute_change_coupling;
use sdi_core::input::{ChangeCouplingConfigInput, CoChangeEventInput};

fn event(sha: &str, files: &[&str]) -> CoChangeEventInput {
    CoChangeEventInput {
        commit_sha: sha.to_string(),
        commit_date: "2026-01-01T00:00:00Z".to_string(),
        files: files.iter().map(|f| f.to_string()).collect(),
    }
}

fn cfg(min_frequency: f64) -> ChangeCouplingConfigInput {
    ChangeCouplingConfigInput { min_frequency, history_depth: 500 }
}

#[test]
fn empty_events_returns_empty() {
    let result = compute_change_coupling(&[], &cfg(0.5)).unwrap();
    assert_eq!(result.pairs.len(), 0);
    assert_eq!(result.commits_analyzed, 0);
    assert_eq!(result.distinct_files_touched, 0);
}

#[test]
fn single_commit_no_pairs() {
    // cochange_count = 1 < 2, so no pairs emitted even though frequency = 1.0
    let events = vec![event("a", &["src/a.rs", "src/b.rs"])];
    let result = compute_change_coupling(&events, &cfg(0.0)).unwrap();
    assert_eq!(result.pairs.len(), 0);
    assert_eq!(result.commits_analyzed, 1);
    assert_eq!(result.distinct_files_touched, 2);
}

#[test]
fn two_commits_same_pair_produces_one_pair() {
    let events = vec![
        event("a", &["src/a.rs", "src/b.rs"]),
        event("b", &["src/a.rs", "src/b.rs"]),
    ];
    let result = compute_change_coupling(&events, &cfg(0.5)).unwrap();
    assert_eq!(result.pairs.len(), 1);
    assert_eq!(result.pairs[0].source, "src/a.rs");
    assert_eq!(result.pairs[0].target, "src/b.rs");
    assert_eq!(result.pairs[0].cochange_count, 2);
    assert!((result.pairs[0].frequency - 1.0).abs() < 1e-9);
}

#[test]
fn history_depth_truncates_window() {
    // 3 events but history_depth=2 → only last 2 analyzed.
    let events = vec![
        event("old", &["src/a.rs", "src/b.rs"]), // outside window
        event("a", &["src/a.rs", "src/b.rs"]),
        event("b", &["src/a.rs", "src/b.rs"]),
    ];
    let cfg2 = ChangeCouplingConfigInput { min_frequency: 0.0, history_depth: 2 };
    let result = compute_change_coupling(&events, &cfg2).unwrap();
    assert_eq!(result.commits_analyzed, 2);
    assert_eq!(result.pairs.len(), 1);
    assert!((result.pairs[0].frequency - 1.0).abs() < 1e-9);
}

#[test]
fn history_depth_uses_trailing_window_not_leading() {
    // history_depth=2 → only the last 2 events are analyzed.
    // The old event touches c.rs + d.rs (different files from the in-window events).
    // A correct trailing window produces an a+b pair (freq=1.0) and no c+d pair.
    // A wrong leading-window implementation would produce a c+d pair instead.
    let events = vec![
        event("old", &["src/c.rs", "src/d.rs"]), // outside window — distinct files
        event("b",   &["src/a.rs", "src/b.rs"]), // in window
        event("c",   &["src/a.rs", "src/b.rs"]), // in window
    ];
    let cfg2 = ChangeCouplingConfigInput { min_frequency: 0.0, history_depth: 2 };
    let result = compute_change_coupling(&events, &cfg2).unwrap();
    assert_eq!(result.commits_analyzed, 2, "only trailing 2 events analyzed");
    let ab = result.pairs.iter().find(|p| p.source == "src/a.rs" && p.target == "src/b.rs");
    assert!(ab.is_some(), "a+b pair must be present: in-window events both touch them");
    let cd = result.pairs.iter().find(|p| p.source == "src/c.rs" && p.target == "src/d.rs");
    assert!(cd.is_none(), "c+d pair must be absent: old event is outside the trailing window");
}

#[test]
fn min_frequency_filter() {
    // 5 commits: a+b appear in all 5, a+c in 3, a+d in 2.
    let evs: Vec<CoChangeEventInput> = (0..5).map(|i| {
        let mut files = vec!["src/a.rs", "src/b.rs"];
        if i < 3 { files.push("src/c.rs"); }
        if i < 2 { files.push("src/d.rs"); }
        event(&i.to_string(), &files)
    }).collect();
    // min_frequency=0.6 → a+b (1.0) and a+c (0.6) survive; a+d (0.4) does not.
    let result = compute_change_coupling(&evs, &cfg(0.6)).unwrap();
    let srcs: Vec<(&str, &str)> = result.pairs.iter()
        .map(|p| (p.source.as_str(), p.target.as_str()))
        .collect();
    assert!(srcs.contains(&("src/a.rs", "src/b.rs")));
    assert!(srcs.contains(&("src/a.rs", "src/c.rs")));
    assert!(!srcs.contains(&("src/a.rs", "src/d.rs")));
}

#[test]
fn pairs_sorted_lexicographically() {
    let events = vec![
        event("a", &["src/z.rs", "src/a.rs"]),
        event("b", &["src/z.rs", "src/a.rs"]),
    ];
    let result = compute_change_coupling(&events, &cfg(0.0)).unwrap();
    assert_eq!(result.pairs[0].source, "src/a.rs");
    assert_eq!(result.pairs[0].target, "src/z.rs");
}

#[test]
fn distinct_files_touched_counts_unique_paths() {
    // commit 1: {a, b}, commit 2: {b, c} → 3 distinct files
    let events = vec![
        event("a", &["src/a.rs", "src/b.rs"]),
        event("b", &["src/b.rs", "src/c.rs"]),
    ];
    let result = compute_change_coupling(&events, &cfg(0.0)).unwrap();
    assert_eq!(result.distinct_files_touched, 3);
}

#[test]
fn invalid_min_frequency_returns_error() {
    let bad_cfg = ChangeCouplingConfigInput { min_frequency: 1.5, history_depth: 100 };
    assert!(compute_change_coupling(&[], &bad_cfg).is_err());
}

#[test]
fn deterministic_output() {
    let events = vec![
        event("a", &["src/a.rs", "src/b.rs", "src/c.rs"]),
        event("b", &["src/a.rs", "src/b.rs"]),
        event("c", &["src/b.rs", "src/c.rs"]),
    ];
    let r1 = compute_change_coupling(&events, &cfg(0.0)).unwrap();
    let r2 = compute_change_coupling(&events, &cfg(0.0)).unwrap();
    assert_eq!(r1, r2);
}
