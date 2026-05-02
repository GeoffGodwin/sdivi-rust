use std::process::Command;

use assert_cmd::Command as CargoBin;
use tempfile::TempDir;

fn sdivi() -> CargoBin {
    CargoBin::cargo_bin("sdivi").expect("sdivi binary must be built")
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

fn setup_two_commit_repo() -> TempDir {
    let tmp = TempDir::new().unwrap();
    let d = tmp.path();
    git(d, &["init"]);
    git(d, &["config", "user.email", "t@t.com"]);
    git(d, &["config", "user.name", "T"]);
    std::fs::write(d.join("a.rs"), "fn a() {}").unwrap();
    git(d, &["add", "a.rs"]);
    git(d, &["commit", "-m", "add a"]);
    std::fs::write(d.join("b.rs"), "fn b() {}").unwrap();
    git(d, &["add", "b.rs"]);
    git(d, &["commit", "-m", "add b"]);
    tmp
}

fn snapshot_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let snap_dir = dir.join(".sdivi").join("snapshots");
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
    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("snapshot")
        .assert()
        .success();

    let out = sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("show")
        .arg("--format")
        .arg("json")
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
        serde_json::from_str::<serde_json::Value>(&stderr).is_err() || stderr.trim().is_empty(),
        "stderr must not be valid JSON; got: {stderr}"
    );
}

// ── check --format json ────────────────────────────────────────────────────

#[test]
fn check_json_stdout_is_valid_json() {
    let repo = empty_repo();

    let out = sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("check")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "first-run check must exit 0; got {:?}",
        out.status.code()
    );

    let stdout = String::from_utf8(out.stdout).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("check --format json stdout must be valid JSON");
    assert!(
        parsed.get("exit_code").is_some(),
        "check JSON must contain 'exit_code'; got: {parsed}"
    );
    assert_eq!(parsed["exit_code"].as_i64().unwrap(), 0);
}

// ── trend --format json ────────────────────────────────────────────────────

#[test]
fn trend_json_with_two_snapshots_is_valid_json() {
    let repo = setup_two_commit_repo();
    let d = repo.path();

    // Two real commits → two distinct snapshot files (unique SHA → unique JSON → unique hash).
    sdivi()
        .arg("--repo")
        .arg(d)
        .arg("snapshot")
        .arg("--commit")
        .arg("HEAD~1")
        .assert()
        .success();
    sdivi()
        .arg("--repo")
        .arg(d)
        .arg("snapshot")
        .arg("--commit")
        .arg("HEAD")
        .assert()
        .success();

    let out = sdivi()
        .arg("--repo")
        .arg(d)
        .arg("trend")
        .arg("--format")
        .arg("json")
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
    let repo = setup_two_commit_repo();
    let d = repo.path();

    sdivi()
        .arg("--repo")
        .arg(d)
        .arg("snapshot")
        .arg("--commit")
        .arg("HEAD~1")
        .assert()
        .success();
    sdivi()
        .arg("--repo")
        .arg(d)
        .arg("snapshot")
        .arg("--commit")
        .arg("HEAD")
        .assert()
        .success();

    let files = snapshot_files(d);
    assert!(files.len() >= 2);

    let out = sdivi()
        .arg("--repo")
        .arg(d)
        .arg("diff")
        .arg(&files[0])
        .arg(&files[1])
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let _: serde_json::Value =
        serde_json::from_str(&stdout).expect("diff --format json stdout must be valid JSON");
}
