use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn version_flag_exits_ok() {
    Command::cargo_bin("sdi")
        .unwrap()
        .arg("--version")
        .assert()
        .success();
}

#[test]
fn version_flag_prints_crate_version() {
    Command::cargo_bin("sdi")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(contains(env!("CARGO_PKG_VERSION")));
}
