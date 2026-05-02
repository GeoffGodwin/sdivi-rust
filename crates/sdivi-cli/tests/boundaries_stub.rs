//! Tests that `sdivi boundaries {infer,ratify,show}` handle the no-snapshots /
//! no-spec case gracefully: exit 0, no stdout, informational message to stderr.

use assert_cmd::Command;
use tempfile::TempDir;

fn sdivi() -> Command {
    Command::cargo_bin("sdivi").expect("sdivi binary must be built")
}

fn empty_repo() -> TempDir {
    tempfile::tempdir().unwrap()
}

/// `sdivi boundaries infer` on a repo with no snapshots exits 0 and reports to
/// stderr (no stdout output).
#[test]
fn boundaries_infer_no_snapshots_exits_zero_stderr_only() {
    let repo = empty_repo();
    let out = sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("boundaries")
        .arg("infer")
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "boundaries infer must exit 0; status={}",
        out.status
    );
    assert!(
        out.stdout.is_empty(),
        "boundaries infer must not write to stdout when there are no proposals"
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        !stderr.is_empty(),
        "boundaries infer must print an informational message to stderr"
    );
}

/// `sdivi boundaries ratify` on a repo with no snapshots exits 0 and reports to
/// stderr (no stdout output, no file written).
#[test]
fn boundaries_ratify_no_snapshots_exits_zero_stderr_only() {
    let repo = empty_repo();
    let out = sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("boundaries")
        .arg("ratify")
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "boundaries ratify must exit 0; status={}",
        out.status
    );
    assert!(
        out.stdout.is_empty(),
        "boundaries ratify must not write to stdout when there is nothing to ratify"
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        !stderr.is_empty(),
        "boundaries ratify must print an informational message to stderr"
    );
    // No boundaries.yaml should have been created.
    let boundary_file = repo.path().join(".sdivi").join("boundaries.yaml");
    assert!(
        !boundary_file.exists(),
        "boundaries ratify with no proposals must not create boundaries.yaml"
    );
}

/// `sdivi boundaries show` on a repo with no `boundaries.yaml` exits 0 and
/// reports to stderr (no stdout output).
#[test]
fn boundaries_show_no_spec_exits_zero_stderr_only() {
    let repo = empty_repo();
    let out = sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("boundaries")
        .arg("show")
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "boundaries show must exit 0; status={}",
        out.status
    );
    assert!(
        out.stdout.is_empty(),
        "boundaries show must not write to stdout when no spec exists"
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        !stderr.is_empty(),
        "boundaries show must print an informational message to stderr when no spec"
    );
}
