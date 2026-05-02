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

fn setup_git_repo(tmp: &TempDir) {
    let d = tmp.path();
    git(d, &["init"]);
    git(d, &["config", "user.email", "t@t.com"]);
    git(d, &["config", "user.name", "T"]);
}

fn snap_dir(repo: &TempDir) -> std::path::PathBuf {
    repo.path().join(".sdivi").join("snapshots")
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

/// `sdivi snapshot` against an empty directory exits 0 and creates one snapshot file.
#[test]
fn snapshot_on_empty_repo_exits_zero() {
    let repo = empty_repo();

    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("snapshot")
        .assert()
        .success();

    let files = snapshot_files(&repo);
    assert_eq!(
        files.len(),
        1,
        "expected exactly one snapshot file, got: {files:?}"
    );
}

/// Running `sdivi snapshot --commit HEAD` at two distinct commits creates two snapshot files.
///
/// Each commit produces a unique SHA → unique `commit` field → unique JSON → unique filename hash.
#[test]
fn snapshot_twice_creates_two_files() {
    let repo = TempDir::new().unwrap();
    setup_git_repo(&repo);
    let d = repo.path();

    std::fs::write(d.join("a.rs"), "fn a() {}").unwrap();
    git(d, &["add", "a.rs"]);
    git(d, &["commit", "-m", "add a"]);
    sdivi()
        .arg("--repo")
        .arg(d)
        .arg("snapshot")
        .arg("--commit")
        .arg("HEAD")
        .assert()
        .success();

    std::fs::write(d.join("b.rs"), "fn b() {}").unwrap();
    git(d, &["add", "b.rs"]);
    git(d, &["commit", "-m", "add b"]);
    sdivi()
        .arg("--repo")
        .arg(d)
        .arg("snapshot")
        .arg("--commit")
        .arg("HEAD")
        .assert()
        .success();

    let files = snapshot_files(&repo);
    assert_eq!(
        files.len(),
        2,
        "expected exactly two snapshot files after two runs"
    );
}

/// `sdivi snapshot --format json` writes valid JSON containing `snapshot_version = "1.0"` to stdout.
#[test]
fn snapshot_json_format_valid_json() {
    let repo = empty_repo();

    let output = sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("snapshot")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "sdivi snapshot --format json must exit 0"
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout must be valid JSON");

    assert_eq!(
        parsed["snapshot_version"].as_str().unwrap(),
        "1.0",
        "snapshot_version must be '1.0'"
    );
}

/// `sdivi diff` of two identical snapshot files exits 0.
#[test]
fn diff_of_two_identical_snapshots_exits_zero() {
    let repo = empty_repo();

    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("snapshot")
        .assert()
        .success();

    let files = snapshot_files(&repo);
    assert_eq!(files.len(), 1);
    let snap_path = &files[0];

    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("diff")
        .arg(snap_path)
        .arg(snap_path)
        .assert()
        .success();
}

/// `sdivi diff --format json` of two snapshots produces valid JSON with a `coupling_delta` field.
#[test]
fn diff_outputs_json_format() {
    let repo = TempDir::new().unwrap();
    setup_git_repo(&repo);
    let d = repo.path();

    std::fs::write(d.join("x.rs"), "fn x() {}").unwrap();
    git(d, &["add", "x.rs"]);
    git(d, &["commit", "-m", "add x"]);
    sdivi()
        .arg("--repo")
        .arg(d)
        .arg("snapshot")
        .arg("--commit")
        .arg("HEAD")
        .assert()
        .success();

    std::fs::write(d.join("y.rs"), "fn y() {}").unwrap();
    git(d, &["add", "y.rs"]);
    git(d, &["commit", "-m", "add y"]);
    sdivi()
        .arg("--repo")
        .arg(d)
        .arg("snapshot")
        .arg("--commit")
        .arg("HEAD")
        .assert()
        .success();

    let files = snapshot_files(&repo);
    assert!(
        files.len() >= 2,
        "need at least two snapshot files for diff"
    );
    let prev = &files[0];
    let curr = &files[1];

    let output = sdivi()
        .arg("--repo")
        .arg(d)
        .arg("diff")
        .arg(prev)
        .arg(curr)
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "sdivi diff --format json must exit 0"
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .expect("stdout of sdivi diff --format json must be valid JSON");

    assert!(
        parsed.get("coupling_delta").is_some(),
        "diff JSON must contain a 'coupling_delta' field, got: {parsed}"
    );
}
