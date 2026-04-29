use assert_cmd::Command;
use tempfile::TempDir;

fn sdi() -> Command {
    Command::cargo_bin("sdi").expect("sdi binary must be built")
}

fn empty_repo() -> TempDir {
    tempfile::tempdir().unwrap()
}

fn snap_dir(repo: &TempDir) -> std::path::PathBuf {
    repo.path().join(".sdi").join("snapshots")
}

fn snapshot_files(repo: &TempDir) -> Vec<std::path::PathBuf> {
    let dir = snap_dir(repo);
    if !dir.exists() {
        return vec![];
    }
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| {
            let e = e.unwrap();
            let name = e.file_name().into_string().unwrap();
            if name.starts_with("snapshot_") && name.ends_with(".json") {
                Some(e.path())
            } else {
                None
            }
        })
        .collect();
    files.sort();
    files
}

/// `sdi snapshot` against an empty directory exits 0 and creates one snapshot file.
#[test]
fn snapshot_on_empty_repo_exits_zero() {
    let repo = empty_repo();

    sdi()
        .arg("--repo")
        .arg(repo.path())
        .arg("snapshot")
        .assert()
        .success();

    let files = snapshot_files(&repo);
    assert_eq!(files.len(), 1, "expected exactly one snapshot file, got: {files:?}");
}

/// Running `sdi snapshot` twice with distinct commit SHAs creates two snapshot files.
#[test]
fn snapshot_twice_creates_two_files() {
    let repo = empty_repo();

    sdi()
        .arg("--repo")
        .arg(repo.path())
        .arg("snapshot")
        .arg("--commit")
        .arg("aaa0000000000000000000000000000000000001")
        .assert()
        .success();

    // Pass a different commit SHA so the JSON content (and thus the blake3
    // filename hash) differs even when both runs happen in the same second.
    sdi()
        .arg("--repo")
        .arg(repo.path())
        .arg("snapshot")
        .arg("--commit")
        .arg("bbb0000000000000000000000000000000000002")
        .assert()
        .success();

    let files = snapshot_files(&repo);
    assert_eq!(files.len(), 2, "expected exactly two snapshot files after two runs");
}

/// `sdi snapshot --format json` writes valid JSON containing `snapshot_version = "1.0"` to stdout.
#[test]
fn snapshot_json_format_valid_json() {
    let repo = empty_repo();

    let output = sdi()
        .arg("--repo")
        .arg(repo.path())
        .arg("snapshot")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(output.status.success(), "sdi snapshot --format json must exit 0");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout must be valid JSON");

    assert_eq!(
        parsed["snapshot_version"].as_str().unwrap(),
        "1.0",
        "snapshot_version must be '1.0'"
    );
}

/// `sdi diff` of two identical snapshot files exits 0.
#[test]
fn diff_of_two_identical_snapshots_exits_zero() {
    let repo = empty_repo();

    sdi()
        .arg("--repo")
        .arg(repo.path())
        .arg("snapshot")
        .assert()
        .success();

    let files = snapshot_files(&repo);
    assert_eq!(files.len(), 1);
    let snap_path = &files[0];

    sdi()
        .arg("--repo")
        .arg(repo.path())
        .arg("diff")
        .arg(snap_path)
        .arg(snap_path)
        .assert()
        .success();
}

/// `sdi diff --format json` of two snapshots produces valid JSON with a `coupling_delta` field.
#[test]
fn diff_outputs_json_format() {
    let repo = empty_repo();

    sdi()
        .arg("--repo")
        .arg(repo.path())
        .arg("snapshot")
        .arg("--commit")
        .arg("ccc0000000000000000000000000000000000003")
        .assert()
        .success();

    // Pass a different commit SHA so the two snapshots have distinct filenames.
    sdi()
        .arg("--repo")
        .arg(repo.path())
        .arg("snapshot")
        .arg("--commit")
        .arg("ddd0000000000000000000000000000000000004")
        .assert()
        .success();

    let files = snapshot_files(&repo);
    assert!(files.len() >= 2, "need at least two snapshot files for diff");
    let prev = &files[0];
    let curr = &files[1];

    let output = sdi()
        .arg("--repo")
        .arg(repo.path())
        .arg("diff")
        .arg(prev)
        .arg(curr)
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(output.status.success(), "sdi diff --format json must exit 0");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout of sdi diff --format json must be valid JSON");

    assert!(
        parsed.get("coupling_delta").is_some(),
        "diff JSON must contain a 'coupling_delta' field, got: {parsed}"
    );
}
