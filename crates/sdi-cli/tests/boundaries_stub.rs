use assert_cmd::Command;
use tempfile::TempDir;

fn sdi() -> Command {
    Command::cargo_bin("sdi").expect("sdi binary must be built")
}

fn empty_repo() -> TempDir {
    tempfile::tempdir().unwrap()
}

/// `sdi boundaries infer` exits 0 and writes only to stderr.
#[test]
fn boundaries_infer_exits_zero_stderr_only() {
    let repo = empty_repo();
    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("boundaries")
        .arg("infer")
        .output()
        .unwrap();

    assert!(out.status.success(), "boundaries infer must exit 0");
    assert!(
        out.stdout.is_empty(),
        "boundaries infer must not write to stdout"
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("not implemented"),
        "boundaries infer must print 'not implemented' to stderr; got: {stderr}"
    );
}

/// `sdi boundaries ratify` exits 0 and writes only to stderr.
#[test]
fn boundaries_ratify_exits_zero_stderr_only() {
    let repo = empty_repo();
    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("boundaries")
        .arg("ratify")
        .output()
        .unwrap();

    assert!(out.status.success(), "boundaries ratify must exit 0");
    assert!(
        out.stdout.is_empty(),
        "boundaries ratify must not write to stdout"
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("not implemented"),
        "boundaries ratify must print 'not implemented' to stderr; got: {stderr}"
    );
}

/// `sdi boundaries show` exits 0 and writes only to stderr.
#[test]
fn boundaries_show_exits_zero_stderr_only() {
    let repo = empty_repo();
    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("boundaries")
        .arg("show")
        .output()
        .unwrap();

    assert!(out.status.success(), "boundaries show must exit 0");
    assert!(
        out.stdout.is_empty(),
        "boundaries show must not write to stdout"
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("not implemented"),
        "boundaries show must print 'not implemented' to stderr; got: {stderr}"
    );
}
