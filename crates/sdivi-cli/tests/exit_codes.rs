use assert_cmd::Command;
use tempfile::TempDir;

fn sdivi() -> Command {
    Command::cargo_bin("sdivi").expect("sdivi binary must be built")
}

fn empty_repo() -> TempDir {
    tempfile::tempdir().unwrap()
}

// ── init ────────────────────────────────────────────────────────────────────

#[test]
fn init_exits_zero() {
    let repo = empty_repo();
    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("init")
        .assert()
        .success();
}

// ── snapshot ─────────────────────────────────────────────────────────────────

#[test]
fn snapshot_exits_zero_on_empty_repo() {
    let repo = empty_repo();
    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("snapshot")
        .assert()
        .success();
}

// ── snapshot — .sdivi/-only extensioned files ──────────────────────────────

#[test]
fn snapshot_exits_zero_when_all_extensioned_files_are_inside_sdivi_dir() {
    // Regression for the .sdivi/-exclusion branch of the NoGrammarsAvailable
    // check.  A repo where every extensioned file lives inside .sdivi/ (e.g.
    // cached snapshot JSON) is a normal empty-source repo and must NOT exit 3.
    let repo = empty_repo();
    let sdivi_dir = repo.path().join(".sdivi");
    std::fs::create_dir_all(sdivi_dir.join("snapshots")).unwrap();
    // Place an extensioned file exclusively inside .sdivi/ — no source files outside.
    std::fs::write(
        sdivi_dir
            .join("snapshots")
            .join("snapshot_20260101T000000_abcdef01.json"),
        r#"{"snapshot_version":"1.0"}"#,
    )
    .unwrap();

    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("snapshot")
        .assert()
        .success();
}

// ── snapshot — all-unknown-language repo ──────────────────────────────────

#[test]
fn snapshot_exits_three_on_all_unknown_languages() {
    let repo = empty_repo();
    // Create a file with an extension that no built-in adapter handles.
    std::fs::write(repo.path().join("code.xyzunknown"), "not a real language\n").unwrap();

    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("snapshot")
        .assert()
        .code(3);
}

// ── check — exit 10 when a threshold is breached ─────────────────────────

#[test]
fn check_exits_ten_when_threshold_breached() {
    let repo = empty_repo();

    // Write a config with coupling_delta_rate = -1.0.  The check compares
    // `if delta > limit`; any real-valued coupling_delta is > -1.0, so the
    // threshold is guaranteed to be breached even when both snapshots are
    // identical (delta == 0.0 > -1.0).
    let sdivi_dir = repo.path().join(".sdivi");
    std::fs::create_dir_all(&sdivi_dir).unwrap();
    std::fs::write(
        sdivi_dir.join("config.toml"),
        "[thresholds]\ncoupling_delta_rate = -1.0\n",
    )
    .unwrap();

    // Baseline snapshot — the prior that `sdivi check` will compare against.
    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("snapshot")
        .assert()
        .success();

    // `sdivi check` takes a new snapshot and computes the delta.  With
    // coupling_delta_rate = -1.0, the coupling_delta (0.0) exceeds the
    // limit (-1.0) → breached → exit 10.
    let out = sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("check")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert_eq!(
        out.status.code(),
        Some(10),
        "sdivi check must exit 10 when a threshold is breached; got {:?}",
        out.status.code()
    );

    let stdout = String::from_utf8(out.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .expect("check --format json stdout must be valid JSON even on exit 10");
    assert_eq!(
        parsed["exit_code"].as_i64().unwrap(),
        10,
        "JSON exit_code must be 10"
    );
    assert!(
        !parsed["exceeded"].as_array().unwrap().is_empty(),
        "exceeded array must be non-empty when threshold is breached"
    );
}

// ── config error — exit 2 ────────────────────────────────────────────────

#[test]
fn config_error_exits_two() {
    let repo = empty_repo();

    // A threshold override block without `expires` triggers
    // ConfigError::MissingExpiresOnOverride during config loading (before any
    // subcommand dispatch), which maps to ExitCode::ConfigError (2) in main.rs.
    let sdivi_dir = repo.path().join(".sdivi");
    std::fs::create_dir_all(&sdivi_dir).unwrap();
    std::fs::write(
        sdivi_dir.join("config.toml"),
        "[thresholds.overrides.test_category]\npattern_entropy_rate = 5.0\n",
    )
    .unwrap();

    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("snapshot")
        .assert()
        .code(2);
}

// ── trend — not enough snapshots ─────────────────────────────────────────

#[test]
fn trend_zero_snapshots_exits_zero() {
    let repo = empty_repo();
    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("trend")
        .assert()
        .success();
}

#[test]
fn trend_one_snapshot_exits_zero() {
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
        .arg("trend")
        .assert()
        .success();
}

// ── show — no snapshots ───────────────────────────────────────────────────

#[test]
fn show_no_snapshots_exits_one() {
    let repo = empty_repo();
    // No snapshots → runtime error (exit 1).
    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("show")
        .assert()
        .code(1);
}

// ── diff ─────────────────────────────────────────────────────────────────

#[test]
fn diff_nonexistent_file_exits_one() {
    let repo = empty_repo();
    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("diff")
        .arg("/nonexistent/prev.json")
        .arg("/nonexistent/curr.json")
        .assert()
        .code(1);
}

// ── snapshot --commit error paths ────────────────────────────────────────────

#[test]
fn snapshot_commit_nonexistent_exits_one() {
    let repo = empty_repo();
    // Make it a git repo so rev-parse can run (and fail on the unknown ref).
    std::process::Command::new("git")
        .current_dir(repo.path())
        .args(["init"])
        .status()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(repo.path())
        .args(["config", "user.email", "t@t.com"])
        .status()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(repo.path())
        .args(["config", "user.name", "T"])
        .status()
        .unwrap();

    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("snapshot")
        .arg("--commit")
        .arg("refs/heads/no-such-branch-xyz-99999")
        .assert()
        .code(1);
}

#[test]
fn snapshot_commit_non_git_dir_exits_one() {
    // A plain directory with no .git/ — git rev-parse will error.
    let dir = empty_repo();

    sdivi()
        .arg("--repo")
        .arg(dir.path())
        .arg("snapshot")
        .arg("--commit")
        .arg("HEAD")
        .assert()
        .code(1);
}

// ── boundaries stubs exit 0 ───────────────────────────────────────────────

#[test]
fn boundaries_infer_exits_zero() {
    let repo = empty_repo();
    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("boundaries")
        .arg("infer")
        .assert()
        .success();
}

#[test]
fn boundaries_ratify_exits_zero() {
    let repo = empty_repo();
    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("boundaries")
        .arg("ratify")
        .assert()
        .success();
}

#[test]
fn boundaries_show_exits_zero() {
    let repo = empty_repo();
    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("boundaries")
        .arg("show")
        .assert()
        .success();
}
