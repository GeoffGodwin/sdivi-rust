use assert_cmd::Command;
use tempfile::TempDir;

fn sdi() -> Command {
    Command::cargo_bin("sdi").expect("sdi binary must be built")
}

fn empty_repo() -> TempDir {
    tempfile::tempdir().unwrap()
}

fn snapshot_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let snap_dir = dir.join(".sdi").join("snapshots");
    if !snap_dir.exists() {
        return vec![];
    }
    let mut files: Vec<_> = std::fs::read_dir(&snap_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let n = e.file_name();
            let s = n.to_string_lossy();
            s.starts_with("snapshot_") && s.ends_with(".json")
        })
        .map(|e| e.path())
        .collect();
    files.sort();
    files
}

// ── show --format json ────────────────────────────────────────────────────

#[test]
fn show_json_stdout_is_valid_json_with_no_stderr_contamination() {
    let repo = empty_repo();
    sdi()
        .arg("--repo").arg(repo.path())
        .arg("snapshot")
        .assert()
        .success();

    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("show")
        .arg("--format").arg("json")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("show --format json stdout must be valid JSON");
    assert_eq!(
        parsed["snapshot_version"].as_str().unwrap(),
        "1.0",
        "snapshot_version must be '1.0'"
    );

    // stderr must not contain JSON.
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        serde_json::from_str::<serde_json::Value>(&stderr).is_err()
            || stderr.trim().is_empty(),
        "stderr must not be valid JSON; got: {stderr}"
    );
}

// ── check --format json ────────────────────────────────────────────────────

#[test]
fn check_json_stdout_is_valid_json() {
    let repo = empty_repo();

    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("check")
        .arg("--format").arg("json")
        .output()
        .unwrap();

    // First-run check is always exit 0.
    assert!(
        out.status.success(),
        "first-run check must exit 0; got {:?}", out.status.code()
    );

    let stdout = String::from_utf8(out.stdout).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("check --format json stdout must be valid JSON");
    assert!(
        parsed.get("exit_code").is_some(),
        "check JSON must contain 'exit_code'; got: {parsed}"
    );
    assert_eq!(
        parsed["exit_code"].as_i64().unwrap(),
        0,
        "first-run exit_code must be 0"
    );
}

// ── trend --format json ────────────────────────────────────────────────────

#[test]
fn trend_json_with_two_snapshots_is_valid_json() {
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
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("trend --format json stdout must be valid JSON");
    assert!(
        parsed.get("snapshot_count").is_some(),
        "trend JSON must contain 'snapshot_count'; got: {parsed}"
    );
    assert_eq!(parsed["snapshot_count"].as_i64().unwrap(), 2);
}

// ── diff --format json ─────────────────────────────────────────────────────

#[test]
fn diff_json_stdout_is_valid_json() {
    let repo = empty_repo();
    sdi()
        .arg("--repo").arg(repo.path())
        .arg("snapshot")
        .arg("--commit").arg("ccc0000000000000000000000000000000000003")
        .assert()
        .success();
    sdi()
        .arg("--repo").arg(repo.path())
        .arg("snapshot")
        .arg("--commit").arg("ddd0000000000000000000000000000000000004")
        .assert()
        .success();

    let files = snapshot_files(repo.path());
    assert!(files.len() >= 2);

    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("diff")
        .arg(&files[0])
        .arg(&files[1])
        .arg("--format").arg("json")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let _: serde_json::Value =
        serde_json::from_str(&stdout).expect("diff --format json stdout must be valid JSON");
}
