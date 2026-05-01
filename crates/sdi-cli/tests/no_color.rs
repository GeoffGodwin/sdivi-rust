use assert_cmd::Command;
use tempfile::TempDir;

fn sdi() -> Command {
    Command::cargo_bin("sdi").expect("sdi binary must be built")
}

fn empty_repo_with_snapshot() -> TempDir {
    let repo = tempfile::tempdir().unwrap();
    Command::cargo_bin("sdi")
        .expect("sdi binary")
        .arg("--repo").arg(repo.path())
        .arg("snapshot")
        .assert()
        .success();
    repo
}

fn has_ansi(s: &str) -> bool {
    s.contains("\x1b[")
}

/// `NO_COLOR=1 sdi show` produces no ANSI escape codes on stdout.
#[test]
fn no_color_env_suppresses_ansi_in_show() {
    let repo = empty_repo_with_snapshot();
    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("show")
        .env("NO_COLOR", "1")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        !has_ansi(&stdout),
        "NO_COLOR=1 sdi show stdout must not contain ANSI codes; got: {stdout:?}"
    );
}

/// `NO_COLOR=1 sdi check` produces no ANSI escape codes on stdout.
#[test]
fn no_color_env_suppresses_ansi_in_check() {
    let repo = tempfile::tempdir().unwrap();
    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("check")
        .env("NO_COLOR", "1")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        !has_ansi(&stdout),
        "NO_COLOR=1 sdi check stdout must not contain ANSI codes; got: {stdout:?}"
    );
}

/// `NO_COLOR=1 sdi trend` produces no ANSI escape codes.
#[test]
fn no_color_env_suppresses_ansi_in_trend_insufficient_snapshots() {
    let repo = tempfile::tempdir().unwrap();
    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("trend")
        .env("NO_COLOR", "1")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        !has_ansi(&stdout),
        "NO_COLOR=1 sdi trend stdout must not contain ANSI codes"
    );
}

/// `sdi show` (no flags, no env) produces no ANSI escape codes on stdout.
///
/// The text formatter does not emit ANSI codes by default.  This test
/// verifies that invariant holds without any suppression flag.  A separate
/// test should be added once any formatter gains colour support — it should
/// assert codes ARE present by default and absent with `--no-color`.
#[test]
fn default_show_output_has_no_ansi_codes() {
    let repo = empty_repo_with_snapshot();
    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("show")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        !has_ansi(&stdout),
        "sdi show stdout must not contain ANSI codes by default; got: {stdout:?}"
    );
}
