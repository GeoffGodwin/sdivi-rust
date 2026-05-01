//! Tests that `sdi boundaries ratify` emits the YAML comment-loss warning
//! (store.rs:108-128) when overwriting a pre-existing `boundaries.yaml` that
//! contains `#` comment lines.
//!
//! Reviewer coverage gap: "write_boundary_spec comment-loss warning path
//! (store.rs:108-128) has no test verifying the eprintln! fires when an
//! existing file contains # lines."
//!
//! This test constructs minimal fake snapshot files so that
//! `infer_from_snapshots` produces at least one proposal, which causes
//! `run_ratify` to call `write_boundary_spec` — the only code path that
//! reaches the warning.

use assert_cmd::Command;
use std::collections::BTreeMap;

fn sdi() -> Command {
    Command::cargo_bin("sdi").expect("sdi binary must be built")
}

/// Serialises a minimal `Snapshot` JSON with a given `path_partition`.
///
/// Only the fields required for deserialisation are included; the rest use
/// sensible zero-values.  The returned string can be written to a
/// `snapshot_*.json` file so that `infer_from_snapshots` can read it.
fn minimal_snapshot_json(path_partition: &BTreeMap<&str, u32>) -> String {
    // Encode path_partition as a JSON object literal.
    let pp_entries: Vec<String> = path_partition
        .iter()
        .map(|(k, v)| format!("\"{}\":{}", k, v))
        .collect();
    let pp_json = format!("{{{}}}", pp_entries.join(","));

    format!(
        r#"{{
  "snapshot_version": "1.0",
  "timestamp": "2026-04-29T00:00:00Z",
  "graph": {{
    "node_count": 2,
    "edge_count": 1,
    "density": 0.5,
    "cycle_count": 0,
    "top_hubs": [],
    "component_count": 1
  }},
  "partition": {{
    "assignments": {{"0": 0, "1": 0}},
    "stability": {{"0": 1.0}},
    "modularity": 0.5,
    "seed": 42
  }},
  "catalog": {{"entries": {{}}}},
  "pattern_metrics": {{
    "entropy_per_category": {{}},
    "total_entropy": 0.0,
    "convention_drift": 0.0
  }},
  "path_partition": {pp_json}
}}"#
    )
}

/// Writes `n` identical fake snapshot files to `<repo>/.sdi/snapshots/`.
///
/// Each snapshot has the same `path_partition` so that the same communities
/// appear consistently across all `n` snapshots, satisfying the stability
/// requirement for `infer_boundaries`.
fn write_stable_snapshots(repo: &std::path::Path, n: usize) {
    let snap_dir = repo.join(".sdi").join("snapshots");
    std::fs::create_dir_all(&snap_dir).unwrap();

    let pp: BTreeMap<&str, u32> = [("a.rs", 0), ("b.rs", 0)].into();
    let json = minimal_snapshot_json(&pp);

    for i in 0..n {
        let name = format!("snapshot_{i:04}.json");
        std::fs::write(snap_dir.join(&name), &json).unwrap();
    }
}

/// `sdi boundaries ratify` emits the YAML comment-loss warning on stderr when
/// the pre-existing `boundaries.yaml` contains `#` lines and proposals exist.
///
/// This is the primary test for the store.rs:108-128 coverage gap.
#[test]
fn ratify_emits_comment_loss_warning_when_spec_has_yaml_hash_comment() {
    let repo = tempfile::tempdir().unwrap();

    // Stability threshold default is 3 → need 4 snapshots.
    write_stable_snapshots(repo.path(), 4);

    // Pre-create a boundaries.yaml with a leading YAML comment.
    let sdi_dir = repo.path().join(".sdi");
    let boundary_path = sdi_dir.join("boundaries.yaml");
    std::fs::write(
        &boundary_path,
        "# Hand-crafted spec — this comment will be lost on ratify\n\
         boundaries:\n\
         - name: old_community\n\
           modules: []\n\
           allow_imports_from: []\n",
    )
    .unwrap();

    let out = sdi()
        .arg("--repo")
        .arg(repo.path())
        .arg("boundaries")
        .arg("ratify")
        .output()
        .unwrap();

    // The command must exit 0 — comment loss is documented behaviour, not an error.
    assert!(
        out.status.success(),
        "ratify must exit 0 even when overwriting a file with comments"
    );

    let stderr = String::from_utf8(out.stderr).unwrap();

    // The comment-loss warning from store.rs:121-125 must appear.
    assert!(
        stderr.contains("comments will be lost"),
        "stderr must contain the comment-loss warning; got: {stderr:?}"
    );
    assert!(
        stderr.contains("boundaries.yaml") || stderr.contains(".sdi"),
        "warning must mention the spec file path; got: {stderr:?}"
    );
}

/// `sdi boundaries ratify` does NOT emit the comment-loss warning when the
/// pre-existing `boundaries.yaml` contains no `#` lines.
#[test]
fn ratify_no_warning_when_spec_has_no_comments() {
    let repo = tempfile::tempdir().unwrap();
    write_stable_snapshots(repo.path(), 4);

    let sdi_dir = repo.path().join(".sdi");
    let boundary_path = sdi_dir.join("boundaries.yaml");
    // Write a comment-free spec.
    std::fs::write(
        &boundary_path,
        "boundaries:\n\
         - name: old_community\n\
           modules: []\n\
           allow_imports_from: []\n",
    )
    .unwrap();

    let out = sdi()
        .arg("--repo")
        .arg(repo.path())
        .arg("boundaries")
        .arg("ratify")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        !stderr.contains("comments will be lost"),
        "no comment-loss warning expected when file has no comments; got: {stderr:?}"
    );
}

/// `sdi boundaries ratify` does NOT emit the comment-loss warning when no
/// pre-existing `boundaries.yaml` exists (first-time ratify).
#[test]
fn ratify_no_warning_when_no_pre_existing_spec() {
    let repo = tempfile::tempdir().unwrap();
    write_stable_snapshots(repo.path(), 4);

    // No pre-existing boundaries.yaml.
    let out = sdi()
        .arg("--repo")
        .arg(repo.path())
        .arg("boundaries")
        .arg("ratify")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        !stderr.contains("comments will be lost"),
        "no comment-loss warning expected for first-time ratify; got: {stderr:?}"
    );
}
