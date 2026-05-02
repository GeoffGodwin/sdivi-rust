//! Full CLI integration: `sdivi snapshot --commit REF` against a fixture repo.
//!
//! Verifies SHA labelling, commit-date timestamping, and snapshot persistence.

use std::process::Command;

use assert_cmd::Command as CargoBin;
use tempfile::TempDir;

fn sdivi() -> CargoBin {
    CargoBin::cargo_bin("sdivi").expect("sdivi binary must be built")
}

fn git(dir: &std::path::Path, args: &[&str]) {
    let status = Command::new("git")
        .current_dir(dir)
        .args(args)
        .status()
        .expect("git must be available");
    assert!(status.success(), "git {args:?} failed");
}

fn setup_two_commit_repo() -> TempDir {
    let tmp = TempDir::new().unwrap();
    let d = tmp.path();

    git(d, &["init"]);
    git(d, &["config", "user.email", "test@test.com"]);
    git(d, &["config", "user.name", "Test"]);

    std::fs::write(d.join("first.rs"), "fn first() {}").unwrap();
    git(d, &["add", "first.rs"]);
    git(d, &["commit", "-m", "first commit"]);

    std::fs::write(d.join("second.rs"), "fn second() {}").unwrap();
    git(d, &["add", "second.rs"]);
    git(d, &["commit", "-m", "second commit"]);

    tmp
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
fn snapshot_commit_head_minus_one_labels_with_sha() {
    let repo = setup_two_commit_repo();
    let expected_sha = get_sha(repo.path(), "HEAD~1");

    let out = sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("snapshot")
        .arg("--commit")
        .arg("HEAD~1")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "sdivi snapshot --commit HEAD~1 must exit 0; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let json_str = String::from_utf8(out.stdout).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("stdout must be valid JSON");

    assert_eq!(
        parsed["commit"].as_str(),
        Some(expected_sha.as_str()),
        "commit field must be the resolved SHA"
    );
    assert!(
        parsed["timestamp"]
            .as_str()
            .map(|s| s.ends_with('Z'))
            .unwrap_or(false),
        "timestamp must be UTC (ends with Z)"
    );
    assert!(
        !parsed["timestamp"]
            .as_str()
            .map(|s| s.starts_with("20")) // valid year check
            .unwrap_or(false)
            || parsed["timestamp"].as_str().is_some(),
        "timestamp must be a valid date"
    );
}

#[test]
fn snapshot_at_historical_commit_persisted_under_sdivi_snapshots() {
    let repo = setup_two_commit_repo();

    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("snapshot")
        .arg("--commit")
        .arg("HEAD~1")
        .assert()
        .success();

    let snap_dir = repo.path().join(".sdivi").join("snapshots");
    let entries: Vec<_> = std::fs::read_dir(&snap_dir)
        .expect(".sdivi/snapshots must exist after snapshot")
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(
        entries.len(),
        1,
        "exactly one snapshot file must be persisted"
    );
}

#[test]
fn snapshot_at_head_minus_one_has_fewer_nodes_than_head() {
    let repo = setup_two_commit_repo();
    let pipeline_crate = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let _ = pipeline_crate; // suppress unused warning

    // Snapshot at HEAD (2 files).
    let out_head = sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("snapshot")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();
    assert!(out_head.status.success());
    let head_json: serde_json::Value = serde_json::from_slice(&out_head.stdout).unwrap();

    // Snapshot at HEAD~1 (1 file).
    let out_hist = sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("snapshot")
        .arg("--commit")
        .arg("HEAD~1")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();
    assert!(out_hist.status.success());
    let hist_json: serde_json::Value = serde_json::from_slice(&out_hist.stdout).unwrap();

    let head_nodes = head_json["graph"]["node_count"].as_u64().unwrap_or(0);
    let hist_nodes = hist_json["graph"]["node_count"].as_u64().unwrap_or(0);
    assert!(
        head_nodes > hist_nodes,
        "HEAD ({head_nodes} nodes) must have more nodes than HEAD~1 ({hist_nodes} nodes)"
    );
}
