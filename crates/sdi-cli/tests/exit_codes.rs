use assert_cmd::Command;
use tempfile::TempDir;

fn sdi() -> Command {
    Command::cargo_bin("sdi").expect("sdi binary must be built")
}

fn empty_repo() -> TempDir {
    tempfile::tempdir().unwrap()
}

// ── init ────────────────────────────────────────────────────────────────────

#[test]
fn init_exits_zero() {
    let repo = empty_repo();
    sdi()
        .arg("--repo").arg(repo.path())
        .arg("init")
        .assert()
        .success();
}

// ── snapshot ─────────────────────────────────────────────────────────────────

#[test]
fn snapshot_exits_zero_on_empty_repo() {
    let repo = empty_repo();
    sdi()
        .arg("--repo").arg(repo.path())
        .arg("snapshot")
        .assert()
        .success();
}

// ── check — first-run (no prior snapshot) ─────────────────────────────────

#[test]
fn check_first_run_exits_zero() {
    let repo = empty_repo();
    // No prior snapshot → null DivergenceSummary → no thresholds can be exceeded.
    sdi()
        .arg("--repo").arg(repo.path())
        .arg("check")
        .assert()
        .success();
}

// ── check --no-write ─────────────────────────────────────────────────────

#[test]
fn check_no_write_exits_zero_and_creates_no_snapshot() {
    let repo = empty_repo();
    let snap_dir = repo.path().join(".sdi").join("snapshots");

    sdi()
        .arg("--repo").arg(repo.path())
        .arg("check")
        .arg("--no-write")
        .assert()
        .success();

    // No snapshot file must have been written.
    if snap_dir.exists() {
        let count = std::fs::read_dir(&snap_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let n = e.file_name();
                let s = n.to_string_lossy();
                s.starts_with("snapshot_") && s.ends_with(".json")
            })
            .count();
        assert_eq!(count, 0, "--no-write must not create snapshot files");
    }
}

// ── check — with prior snapshot, below thresholds ─────────────────────────

#[test]
fn check_below_thresholds_exits_zero() {
    let repo = empty_repo();
    // Create prior snapshot.
    sdi()
        .arg("--repo").arg(repo.path())
        .arg("snapshot")
        .arg("--commit").arg("aaa0000000000000000000000000000000000001")
        .assert()
        .success();
    // Check against prior — empty repo has no meaningful drift.
    sdi()
        .arg("--repo").arg(repo.path())
        .arg("check")
        .assert()
        .success();
}

// ── trend — not enough snapshots ─────────────────────────────────────────

#[test]
fn trend_zero_snapshots_exits_zero() {
    let repo = empty_repo();
    sdi()
        .arg("--repo").arg(repo.path())
        .arg("trend")
        .assert()
        .success();
}

#[test]
fn trend_one_snapshot_exits_zero() {
    let repo = empty_repo();
    sdi()
        .arg("--repo").arg(repo.path())
        .arg("snapshot")
        .assert()
        .success();
    sdi()
        .arg("--repo").arg(repo.path())
        .arg("trend")
        .assert()
        .success();
}

// ── show — no snapshots ───────────────────────────────────────────────────

#[test]
fn show_no_snapshots_exits_one() {
    let repo = empty_repo();
    // No snapshots → runtime error (exit 1).
    sdi()
        .arg("--repo").arg(repo.path())
        .arg("show")
        .assert()
        .failure();
}

// ── diff ─────────────────────────────────────────────────────────────────

#[test]
fn diff_nonexistent_file_exits_one() {
    let repo = empty_repo();
    sdi()
        .arg("--repo").arg(repo.path())
        .arg("diff")
        .arg("/nonexistent/prev.json")
        .arg("/nonexistent/curr.json")
        .assert()
        .failure();
}

// ── boundaries stubs exit 0 ───────────────────────────────────────────────

#[test]
fn boundaries_infer_exits_zero() {
    let repo = empty_repo();
    sdi()
        .arg("--repo").arg(repo.path())
        .arg("boundaries")
        .arg("infer")
        .assert()
        .success();
}

#[test]
fn boundaries_ratify_exits_zero() {
    let repo = empty_repo();
    sdi()
        .arg("--repo").arg(repo.path())
        .arg("boundaries")
        .arg("ratify")
        .assert()
        .success();
}

#[test]
fn boundaries_show_exits_zero() {
    let repo = empty_repo();
    sdi()
        .arg("--repo").arg(repo.path())
        .arg("boundaries")
        .arg("show")
        .assert()
        .success();
}
