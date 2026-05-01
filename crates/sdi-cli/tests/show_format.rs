use assert_cmd::Command;
use tempfile::TempDir;

fn sdi() -> Command {
    Command::cargo_bin("sdi").expect("sdi binary must be built")
}

fn empty_repo() -> TempDir {
    tempfile::tempdir().unwrap()
}

fn snapshot_files(repo: &TempDir) -> Vec<std::path::PathBuf> {
    let snap_dir = repo.path().join(".sdi").join("snapshots");
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

/// `sdi show --format json` parses as a Snapshot with `snapshot_version = "1.0"`.
#[test]
fn show_json_parses_as_snapshot() {
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
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .expect("show --format json stdout must be valid JSON");
    assert_eq!(
        parsed["snapshot_version"].as_str().unwrap(),
        "1.0",
        "`sdi show --format json | jq '.snapshot_version'` must return '1.0'"
    );
}

/// `sdi show` with no id selects the lexicographically-last snapshot.
///
/// When multiple snapshots exist, `sdi show` returns whichever has the
/// lexicographically-last filename (the M07 `snapshot_<ts>_<hash>` scheme
/// makes this chronological unless two snapshots share a timestamp, in which
/// case the blake3 hash breaks the tie).  We verify that a valid Snapshot
/// is returned and the commit is one of the two we wrote.
#[test]
fn show_no_id_selects_latest() {
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

    let files = snapshot_files(&repo);
    assert_eq!(files.len(), 2);

    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("show")
        .arg("--format").arg("json")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        parsed["snapshot_version"].as_str().unwrap(),
        "1.0",
        "show with no id must return a valid snapshot"
    );
    // The commit must be one of the two we wrote.
    let commit = parsed["commit"].as_str().unwrap_or("");
    let known_commits = [
        "aaa0000000000000000000000000000000000001",
        "bbb0000000000000000000000000000000000002",
    ];
    assert!(
        known_commits.contains(&commit),
        "show returned unexpected commit '{commit}'"
    );
    // The returned snapshot must be the lexicographically-last file.
    let latest_file = files.last().unwrap();
    let latest_content = std::fs::read_to_string(latest_file).unwrap();
    let latest: serde_json::Value = serde_json::from_str(&latest_content).unwrap();
    assert_eq!(
        parsed["commit"], latest["commit"],
        "show with no id must match the lexicographically-last snapshot file"
    );
}

/// `sdi show <id>` selects the specific snapshot by filename stem.
#[test]
fn show_with_id_selects_specific_snapshot() {
    let repo = empty_repo();
    sdi()
        .arg("--repo").arg(repo.path())
        .arg("snapshot")
        .arg("--commit").arg("ccc0000000000000000000000000000000000003")
        .assert()
        .success();

    let files = snapshot_files(&repo);
    assert_eq!(files.len(), 1);

    // Get the filename stem (without .json).
    let stem = files[0]
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();

    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("show")
        .arg(&stem)
        .arg("--format").arg("json")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        parsed["commit"].as_str().unwrap_or(""),
        "ccc0000000000000000000000000000000000003",
        "show <id> must load the specific snapshot"
    );
}

/// `sdi show` with no snapshots exits non-zero.
#[test]
fn show_no_snapshots_fails() {
    let repo = empty_repo();
    sdi()
        .arg("--repo").arg(repo.path())
        .arg("show")
        .assert()
        .failure();
}

/// `sdi show --format json` produces no stderr JSON contamination.
#[test]
fn show_json_has_no_stderr_json() {
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
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        serde_json::from_str::<serde_json::Value>(&stderr).is_err()
            || stderr.trim().is_empty(),
        "stderr must not be valid JSON (CLAUDE.md Rule 8)"
    );
}
