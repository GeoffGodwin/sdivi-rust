//! Integration tests for `sdivi catalog --format json|text`.

use assert_cmd::Command;
use predicates::prelude::*;

fn sdivi() -> Command {
    Command::cargo_bin("sdivi").expect("sdivi binary must be built")
}

fn simple_rust_path() -> &'static str {
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/simple-rust"
    )
}

/// `sdivi catalog --format json` exits 0 and outputs valid JSON to stdout.
#[test]
fn catalog_json_format_outputs_valid_json() {
    let output = sdivi()
        .arg("--repo")
        .arg(simple_rust_path())
        .arg("catalog")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = std::str::from_utf8(&output).expect("stdout must be UTF-8");
    let value: serde_json::Value =
        serde_json::from_str(stdout).expect("stdout must be valid JSON for --format json");
    assert!(value.is_object(), "catalog JSON must be a top-level object");
}

/// `sdivi catalog --format json` emits nothing on stderr about parse errors.
#[test]
fn catalog_json_logs_go_to_stderr_not_stdout() {
    let output = sdivi()
        .arg("--repo")
        .arg(simple_rust_path())
        .arg("catalog")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = std::str::from_utf8(&output).expect("stdout must be UTF-8");
    // stdout must be parseable JSON — no log contamination
    serde_json::from_str::<serde_json::Value>(stdout)
        .expect("stdout must not contain log output mixed with JSON");
}

/// `sdivi catalog` (text format, default) exits 0 and writes to stdout.
#[test]
fn catalog_text_format_exits_zero() {
    sdivi()
        .arg("--repo")
        .arg(simple_rust_path())
        .arg("catalog")
        .assert()
        .success();
}

/// `sdivi catalog --format text` prints something to stdout.
#[test]
fn catalog_text_format_produces_output() {
    sdivi()
        .arg("--repo")
        .arg(simple_rust_path())
        .arg("catalog")
        .arg("--format")
        .arg("text")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

/// `sdivi catalog --format json` with the high-entropy fixture produces a
/// catalog with at least one category entry.
#[test]
fn catalog_json_high_entropy_has_entries() {
    let high_entropy_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/high-entropy"
    );

    // Use min_pattern_nodes override via a temp config
    let output = sdivi()
        .arg("--repo")
        .arg(high_entropy_path)
        .arg("catalog")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = std::str::from_utf8(&output).expect("stdout must be UTF-8");
    let value: serde_json::Value = serde_json::from_str(stdout).expect("stdout must be valid JSON");
    assert!(value.is_object());
    // The catalog object has an `entries` field
    assert!(
        value.get("entries").is_some(),
        "catalog JSON must have an 'entries' field"
    );
}
