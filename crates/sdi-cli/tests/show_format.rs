use std::process::Command;

use assert_cmd::Command as CargoBin;
use tempfile::TempDir;

fn sdi() -> CargoBin {
    CargoBin::cargo_bin("sdi").expect("sdi binary must be built")
}

fn empty_repo() -> TempDir {
    tempfile::tempdir().unwrap()
}

fn git(dir: &std::path::Path, args: &[&str]) {
    let status = Command::new("git")
        .current_dir(dir)
        .args(args)
        .status()
        .expect("git must be available");
    assert!(status.success(), "git {args:?} failed");
}

fn get_sha(dir: &std::path::Path, rev: &str) -> String {
    let out = Command::new("git")
        .current_dir(dir)
        .args(["rev-parse", rev])
        .output()
        .unwrap();
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

fn setup_git_repo(tmp: &TempDir) {
    let d = tmp.path();
    git(d, &["init"]);
    git(d, &["config", "user.email", "t@t.com"]);
    git(d, &["config", "user.name", "T"]);
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
/// Two real commits are created; each is snapshotted with `--commit HEAD`.
/// The SHAs differ, producing two distinct snapshot files with unique commit labels.
#[test]
fn show_no_id_selects_latest() {
    let repo = TempDir::new().unwrap();
    setup_git_repo(&repo);
    let d = repo.path();

    std::fs::write(d.join("a.rs"), "fn a() {}").unwrap();
    git(d, &["add", "a.rs"]);
    git(d, &["commit", "-m", "commit A"]);
    let sha_a = get_sha(d, "HEAD");

    sdi()
        .arg("--repo").arg(d)
        .arg("snapshot")
        .arg("--commit").arg("HEAD")
        .assert()
        .success();

    std::fs::write(d.join("b.rs"), "fn b() {}").unwrap();
    git(d, &["add", "b.rs"]);
    git(d, &["commit", "-m", "commit B"]);
    let sha_b = get_sha(d, "HEAD");

    sdi()
        .arg("--repo").arg(d)
        .arg("snapshot")
        .arg("--commit").arg("HEAD")
        .assert()
        .success();

    let files = snapshot_files(&repo);
    assert_eq!(files.len(), 2, "two distinct snapshot files must exist");

    let out = sdi()
        .arg("--repo").arg(d)
        .arg("show")
        .arg("--format").arg("json")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(parsed["snapshot_version"].as_str().unwrap(), "1.0");

    // The returned commit must be one of the two we wrote.
    let commit = parsed["commit"].as_str().unwrap_or("");
    assert!(
        commit == sha_a || commit == sha_b,
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
    let repo = TempDir::new().unwrap();
    setup_git_repo(&repo);
    let d = repo.path();

    std::fs::write(d.join("c.rs"), "fn c() {}").unwrap();
    git(d, &["add", "c.rs"]);
    git(d, &["commit", "-m", "commit C"]);
    let sha_c = get_sha(d, "HEAD");

    sdi()
        .arg("--repo").arg(d)
        .arg("snapshot")
        .arg("--commit").arg("HEAD")
        .assert()
        .success();

    let files = snapshot_files(&repo);
    assert_eq!(files.len(), 1);

    let stem = files[0].file_stem().unwrap().to_string_lossy().to_string();

    let out = sdi()
        .arg("--repo").arg(d)
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
        sha_c.as_str(),
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
