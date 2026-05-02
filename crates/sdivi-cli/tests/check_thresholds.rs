use assert_cmd::Command;
use tempfile::TempDir;

fn sdivi() -> Command {
    Command::cargo_bin("sdivi").expect("sdivi binary must be built")
}

fn empty_repo() -> TempDir {
    tempfile::tempdir().unwrap()
}

fn snapshot_count(repo: &TempDir) -> usize {
    let snap_dir = repo.path().join(".sdivi").join("snapshots");
    if !snap_dir.exists() {
        return 0;
    }
    std::fs::read_dir(&snap_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let n = e.file_name();
            let s = n.to_string_lossy();
            s.starts_with("snapshot_") && s.ends_with(".json")
        })
        .count()
}

/// First-run `sdivi check`: no prior snapshot → null DivergenceSummary → exit 0.
#[test]
fn first_run_check_exits_zero() {
    let repo = empty_repo();
    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("check")
        .assert()
        .success();
}

/// `sdivi check` with a prior snapshot on an empty (unchanged) repo exits 0.
#[test]
fn check_with_prior_below_thresholds_exits_zero() {
    let repo = empty_repo();
    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("snapshot")
        .assert()
        .success();
    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("check")
        .assert()
        .success();
}

/// `sdivi check --no-write` does not create a snapshot file.
#[test]
fn check_no_write_creates_no_snapshot() {
    let repo = empty_repo();

    let before = snapshot_count(&repo);
    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("check")
        .arg("--no-write")
        .assert()
        .success();
    let after = snapshot_count(&repo);

    assert_eq!(
        before, after,
        "sdivi check --no-write must not create snapshot files"
    );
}

/// `sdivi check` (with write, no prior) creates exactly one snapshot.
#[test]
fn check_with_write_creates_one_snapshot() {
    let repo = empty_repo();

    assert_eq!(snapshot_count(&repo), 0);
    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("check")
        .assert()
        .success();
    assert_eq!(
        snapshot_count(&repo),
        1,
        "sdivi check must write exactly one snapshot"
    );
}

/// `sdivi check --format json` emits valid JSON with `exit_code` field.
#[test]
fn check_json_has_exit_code_field() {
    let repo = empty_repo();

    let out = sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("check")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    let stdout = String::from_utf8(out.stdout).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("check --format json stdout must be valid JSON");

    assert!(
        parsed.get("exit_code").is_some(),
        "check JSON must have 'exit_code'; got: {parsed}"
    );
    assert_eq!(parsed["exit_code"].as_i64().unwrap(), 0);
    assert!(
        parsed.get("exceeded").is_some(),
        "check JSON must have 'exceeded'"
    );
    assert!(
        parsed.get("summary").is_some(),
        "check JSON must have 'summary'"
    );
}

/// `sdivi check --format json | .exit_code` returns the same value as the process exit code.
#[test]
fn check_json_exit_code_matches_process_exit() {
    let repo = empty_repo();

    let out = sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("check")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    let stdout = String::from_utf8(out.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let json_code = parsed["exit_code"].as_i64().unwrap() as i32;
    let process_code = out.status.code().unwrap_or(-1);

    assert_eq!(
        json_code, process_code,
        "JSON exit_code ({json_code}) must match process exit code ({process_code})"
    );
}
