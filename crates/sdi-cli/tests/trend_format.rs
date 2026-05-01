use assert_cmd::Command;
use tempfile::TempDir;

fn sdi() -> Command {
    Command::cargo_bin("sdi").expect("sdi binary must be built")
}

fn empty_repo() -> TempDir {
    tempfile::tempdir().unwrap()
}

/// `sdi trend` with 0 snapshots prints friendly message to stderr and exits 0.
#[test]
fn trend_zero_snapshots_prints_friendly_message() {
    let repo = empty_repo();
    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("trend")
        .output()
        .unwrap();

    assert!(out.status.success(), "trend with 0 snapshots must exit 0");
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("not enough snapshots"),
        "stderr must contain 'not enough snapshots'; got: {stderr}"
    );
}

/// `sdi trend` with 1 snapshot prints friendly message to stderr and exits 0.
#[test]
fn trend_one_snapshot_prints_friendly_message() {
    let repo = empty_repo();
    sdi()
        .arg("--repo").arg(repo.path())
        .arg("snapshot")
        .assert()
        .success();

    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("trend")
        .output()
        .unwrap();

    assert!(out.status.success(), "trend with 1 snapshot must exit 0");
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("not enough snapshots"),
        "stderr must contain 'not enough snapshots'; got: {stderr}"
    );
}

/// `sdi trend --format json` with 2 snapshots emits valid JSON with `snapshot_count`.
#[test]
fn trend_json_two_snapshots_valid() {
    let repo = empty_repo();
    sdi()
        .arg("--repo").arg(repo.path())
        .arg("snapshot")
        .arg("--commit").arg("aaa0000000000000000000000000000000000001")
        .assert()
        .success();
    sdi()
        .arg("--repo").arg(repo.path())
        .arg("snapshot")
        .arg("--commit").arg("bbb0000000000000000000000000000000000002")
        .assert()
        .success();

    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("trend")
        .arg("--format").arg("json")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .expect("trend --format json stdout must be valid JSON");
    assert_eq!(
        parsed["snapshot_count"].as_i64().unwrap(),
        2,
        "snapshot_count must be 2"
    );
    assert!(
        parsed.get("coupling_slope").is_some(),
        "trend JSON must contain 'coupling_slope'"
    );
}

/// `sdi trend --last 9999` with 3 snapshots silently uses all 3 (no error).
#[test]
fn trend_last_n_larger_than_available_silently_clamps() {
    let repo = empty_repo();
    for sha in ["aaa", "bbb", "ccc"] {
        sdi()
            .arg("--repo").arg(repo.path())
            .arg("snapshot")
            .arg("--commit").arg(format!("{sha}0000000000000000000000000000000000001"))
            .assert()
            .success();
    }

    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("trend")
        .arg("--last").arg("9999")
        .arg("--format").arg("json")
        .output()
        .unwrap();

    assert!(out.status.success(), "trend --last 9999 must exit 0 when only 3 snapshots exist");
    let stdout = String::from_utf8(out.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        parsed["snapshot_count"].as_i64().unwrap(),
        3,
        "snapshot_count must be 3 (silently clamped from 9999)"
    );
}

/// `sdi trend --last 2` with 3 snapshots uses only the 2 most recent.
#[test]
fn trend_last_n_selects_tail() {
    let repo = empty_repo();
    for sha in ["aaa", "bbb", "ccc"] {
        sdi()
            .arg("--repo").arg(repo.path())
            .arg("snapshot")
            .arg("--commit").arg(format!("{sha}0000000000000000000000000000000000001"))
            .assert()
            .success();
    }

    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("trend")
        .arg("--last").arg("2")
        .arg("--format").arg("json")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        parsed["snapshot_count"].as_i64().unwrap(),
        2,
        "--last 2 must use only 2 snapshots"
    );
}
