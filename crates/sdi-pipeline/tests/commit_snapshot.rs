//! Integration tests for `Pipeline::snapshot` with `--commit REF`.
//!
//! Sets up a three-commit fixture git repo (each commit adding one `.rs` file),
//! then verifies SHA labelling, commit-date timestamping, tempdir cleanup, and
//! change-coupling windowing.

use std::process::Command;

use sdi_config::Config;
use sdi_lang_rust::RustAdapter;
use sdi_pipeline::Pipeline;
use tempfile::TempDir;

// ── fixture helpers ──────────────────────────────────────────────────────────

fn git(dir: &std::path::Path, args: &[&str]) {
    let status = Command::new("git")
        .current_dir(dir)
        .args(args)
        .status()
        .expect("git must be available");
    assert!(status.success(), "git {args:?} failed");
}

fn setup_three_commit_repo() -> TempDir {
    let tmp = TempDir::new().unwrap();
    let d = tmp.path();

    git(d, &["init"]);
    git(d, &["config", "user.email", "test@test.com"]);
    git(d, &["config", "user.name", "Test"]);

    // Commit 1: one file.
    std::fs::write(d.join("a.rs"), "fn a() {}").unwrap();
    git(d, &["add", "a.rs"]);
    git(d, &["commit", "-m", "add a"]);

    // Commit 2: add a second file.
    std::fs::write(d.join("b.rs"), "fn b() {}").unwrap();
    git(d, &["add", "b.rs"]);
    git(d, &["commit", "-m", "add b"]);

    // Commit 3 (HEAD): add a third file.
    std::fs::write(d.join("c.rs"), "fn c() {}").unwrap();
    git(d, &["add", "c.rs"]);
    git(d, &["commit", "-m", "add c"]);

    tmp
}

fn adapters() -> Vec<Box<dyn sdi_parsing::adapter::LanguageAdapter>> {
    vec![Box::new(RustAdapter)]
}

fn get_sha(dir: &std::path::Path, rev: &str) -> String {
    let out = Command::new("git")
        .current_dir(dir)
        .args(["rev-parse", rev])
        .output()
        .unwrap();
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

// ── tests ────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_at_head_has_three_nodes() {
    let repo = setup_three_commit_repo();
    let pipeline = Pipeline::new(Config::default(), adapters());

    let snap = pipeline
        .snapshot(repo.path(), Some("HEAD"), "2026-01-01T00:00:00Z")
        .expect("snapshot at HEAD must succeed");

    assert_eq!(snap.graph.node_count, 3, "HEAD tree has three .rs files");
}

#[test]
fn snapshot_at_head_minus_one_has_two_nodes() {
    let repo = setup_three_commit_repo();
    let pipeline = Pipeline::new(Config::default(), adapters());

    let snap = pipeline
        .snapshot(repo.path(), Some("HEAD~1"), "2026-01-01T00:00:00Z")
        .expect("snapshot at HEAD~1 must succeed");

    assert_eq!(snap.graph.node_count, 2, "HEAD~1 tree has two .rs files");
}

#[test]
fn snapshot_at_head_minus_two_has_one_node() {
    let repo = setup_three_commit_repo();
    let pipeline = Pipeline::new(Config::default(), adapters());

    let snap = pipeline
        .snapshot(repo.path(), Some("HEAD~2"), "2026-01-01T00:00:00Z")
        .expect("snapshot at HEAD~2 must succeed");

    assert_eq!(snap.graph.node_count, 1, "HEAD~2 tree has one .rs file");
}

#[test]
fn commit_field_is_resolved_sha_not_ref_name() {
    let repo = setup_three_commit_repo();
    let expected_sha = get_sha(repo.path(), "HEAD");
    let pipeline = Pipeline::new(Config::default(), adapters());

    let snap = pipeline
        .snapshot(repo.path(), Some("HEAD"), "2026-01-01T00:00:00Z")
        .unwrap();

    assert_eq!(
        snap.commit.as_deref(),
        Some(expected_sha.as_str()),
        "commit field must be the 40-char SHA, not 'HEAD'"
    );
    assert_eq!(snap.commit.as_ref().map(|s| s.len()), Some(40));
}

#[test]
fn timestamp_is_commit_date_not_wall_clock() {
    let repo = setup_three_commit_repo();
    let pipeline = Pipeline::new(Config::default(), adapters());

    let snap = pipeline
        .snapshot(repo.path(), Some("HEAD"), "2099-12-31T23:59:59Z")
        .unwrap();

    // The supplied timestamp (year 2099) must be overridden by the commit date.
    assert!(
        !snap.timestamp.starts_with("2099"),
        "timestamp must be the commit-date, not the wall-clock argument"
    );
    assert!(
        snap.timestamp.ends_with('Z'),
        "timestamp must be UTC ISO 8601"
    );
}

#[test]
fn different_commits_produce_distinct_snapshots() {
    let repo = setup_three_commit_repo();
    let pipeline = Pipeline::new(Config::default(), adapters());

    let head = pipeline
        .snapshot(repo.path(), Some("HEAD"), "2026-01-01T00:00:00Z")
        .unwrap();
    let head1 = pipeline
        .snapshot(repo.path(), Some("HEAD~1"), "2026-01-01T00:00:00Z")
        .unwrap();
    let head2 = pipeline
        .snapshot(repo.path(), Some("HEAD~2"), "2026-01-01T00:00:00Z")
        .unwrap();

    assert_ne!(head.commit, head1.commit, "HEAD and HEAD~1 have different SHAs");
    assert_ne!(head1.commit, head2.commit, "HEAD~1 and HEAD~2 have different SHAs");
    assert_ne!(head.graph.node_count, head1.graph.node_count);
    assert_ne!(head1.graph.node_count, head2.graph.node_count);
}

#[test]
fn nonexistent_ref_returns_error() {
    let repo = setup_three_commit_repo();
    let pipeline = Pipeline::new(Config::default(), adapters());

    let result = pipeline.snapshot(
        repo.path(),
        Some("refs/heads/no-such-branch-xyz-999"),
        "2026-01-01T00:00:00Z",
    );

    assert!(
        result.is_err(),
        "nonexistent ref must produce an error"
    );
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("ref resolution failed") || msg.contains("commit extract"),
        "error message must describe the failure: {msg}"
    );
}

#[test]
fn change_coupling_ends_at_commit_not_head() {
    let repo = setup_three_commit_repo();
    let mut config = Config::default();
    config.change_coupling.history_depth = 100;
    let pipeline = Pipeline::new(config, adapters());

    let sha_head1 = get_sha(repo.path(), "HEAD~1");

    let snap_head = pipeline
        .snapshot(repo.path(), None, "2026-01-01T00:00:00Z")
        .unwrap();
    let snap_hist = pipeline
        .snapshot(repo.path(), Some("HEAD~1"), "2026-01-01T00:00:00Z")
        .unwrap();

    // HEAD includes 3 commits; HEAD~1 includes 2 commits.
    // The change-coupling at HEAD~1 must not see the third commit.
    // We verify the commit SHA label differs as expected.
    assert_eq!(snap_hist.commit.as_deref(), Some(sha_head1.as_str()));
    assert_ne!(snap_head.commit, snap_hist.commit);
}

/// Fixture: three commits where a.rs + b.rs co-change in commit 1 and commit 3,
/// with an unrelated c.rs-only commit 2 in between.
///
/// Design:
///   HEAD~2 (commit 1): create a.rs + b.rs → (a.rs, b.rs) co-change #1
///   HEAD~1 (commit 2): create c.rs alone → no a/b pair
///   HEAD   (commit 3): modify a.rs + b.rs → (a.rs, b.rs) co-change #2
///
/// At HEAD~1: 2 commits visible, (a.rs, b.rs) cochange_count == 1 < 2 → 0 pairs.
/// At HEAD:   3 commits visible, (a.rs, b.rs) cochange_count == 2, freq ≈ 0.67 ≥ 0.6 → 1 pair.
fn setup_cochange_repo() -> TempDir {
    let tmp = TempDir::new().unwrap();
    let d = tmp.path();

    git(d, &["init"]);
    git(d, &["config", "user.email", "test@test.com"]);
    git(d, &["config", "user.name", "Test"]);

    // Commit 1: a.rs and b.rs together (first co-change).
    std::fs::write(d.join("a.rs"), "fn a() {}").unwrap();
    std::fs::write(d.join("b.rs"), "fn b() {}").unwrap();
    git(d, &["add", "a.rs", "b.rs"]);
    git(d, &["commit", "-m", "add a and b together"]);

    // Commit 2: c.rs alone (no a/b pair).
    std::fs::write(d.join("c.rs"), "fn c() {}").unwrap();
    git(d, &["add", "c.rs"]);
    git(d, &["commit", "-m", "add c alone"]);

    // Commit 3 (HEAD): modify a.rs and b.rs together (second co-change).
    std::fs::write(d.join("a.rs"), "fn a() { 1 }").unwrap();
    std::fs::write(d.join("b.rs"), "fn b() { 2 }").unwrap();
    git(d, &["add", "a.rs", "b.rs"]);
    git(d, &["commit", "-m", "modify a and b together again"]);

    tmp
}

#[test]
fn change_coupling_window_clamped_to_commit_not_head() {
    let repo = setup_cochange_repo();
    let mut config = Config::default();
    config.change_coupling.history_depth = 100;
    // min_frequency default is 0.6; we keep it to confirm the pair appears at HEAD
    // but not at HEAD~1 (count=1 < 2).
    let pipeline = Pipeline::new(config, adapters());

    // Snapshot at HEAD~1: only commits 1 and 2 in the window.
    let sha_head1 = get_sha(repo.path(), "HEAD~1");
    let snap_hist = pipeline
        .snapshot(repo.path(), Some("HEAD~1"), "2026-01-01T00:00:00Z")
        .expect("snapshot at HEAD~1 must succeed");

    // Snapshot at HEAD: commits 1, 2, and 3 in the window.
    let snap_head = pipeline
        .snapshot(repo.path(), None, "2026-01-01T00:00:00Z")
        .expect("snapshot at HEAD must succeed");

    // Verify commit labels are correctly set.
    assert_eq!(snap_hist.commit.as_deref(), Some(sha_head1.as_str()));

    // The historical snapshot must have analyzed exactly 2 commits and
    // produced 0 co-change pairs (cochange_count == 1 < 2).
    let cc_hist = snap_hist
        .change_coupling
        .as_ref()
        .expect("change_coupling must be Some for a repo with history");
    assert_eq!(
        cc_hist.commits_analyzed, 2,
        "HEAD~1 window should see exactly 2 commits"
    );
    assert_eq!(
        cc_hist.pairs.len(),
        0,
        "HEAD~1 window: (a.rs, b.rs) co-changed only once — below cochange_count threshold"
    );

    // The HEAD snapshot must have analyzed 3 commits and found the (a.rs, b.rs) pair.
    let cc_head = snap_head
        .change_coupling
        .as_ref()
        .expect("change_coupling must be Some for a repo with history");
    assert_eq!(
        cc_head.commits_analyzed, 3,
        "HEAD window should see all 3 commits"
    );
    assert_eq!(
        cc_head.pairs.len(),
        1,
        "HEAD window: (a.rs, b.rs) co-changed twice — meets both thresholds"
    );
    let pair = &cc_head.pairs[0];
    assert_eq!(pair.cochange_count, 2);
    assert!(
        pair.frequency >= 0.6,
        "pair frequency ({}) must be >= min_frequency (0.6)",
        pair.frequency
    );
    // Lexicographically smaller file is `source`.
    assert_eq!(pair.source, "a.rs");
    assert_eq!(pair.target, "b.rs");
}
