//! Tests for `sdi boundaries show` format flags and behavior.

use assert_cmd::Command;
use std::io::Write as _;
use tempfile::TempDir;

fn sdi() -> Command {
    Command::cargo_bin("sdi").expect("sdi binary must be built")
}

fn repo_with_boundary_spec(content: &str) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let sdi_dir = dir.path().join(".sdi");
    std::fs::create_dir_all(&sdi_dir).unwrap();
    let mut f = std::fs::File::create(sdi_dir.join("boundaries.yaml")).unwrap();
    f.write_all(content.as_bytes()).unwrap();
    dir
}

const SAMPLE_YAML: &str = "\
boundaries:
- name: api
  modules:
  - src/api/**
  allow_imports_from:
  - models
- name: models
  modules:
  - src/models/**
  allow_imports_from: []
";

/// `sdi boundaries show` with default format outputs valid YAML to stdout.
#[test]
fn show_default_format_outputs_yaml() {
    let repo = repo_with_boundary_spec(SAMPLE_YAML);
    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("boundaries")
        .arg("show")
        .output()
        .unwrap();

    assert!(out.status.success(), "boundaries show must exit 0");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(!stdout.is_empty(), "boundaries show must write to stdout");
    assert!(stdout.contains("api"), "YAML output must contain boundary name 'api'");
    assert!(stdout.contains("models"), "YAML output must contain boundary name 'models'");
}

/// `sdi boundaries show --format yaml` outputs the same YAML as default.
#[test]
fn show_yaml_format_explicit() {
    let repo = repo_with_boundary_spec(SAMPLE_YAML);
    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("boundaries")
        .arg("show")
        .arg("--format").arg("yaml")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("api"));
}

/// `sdi boundaries show --format json` outputs valid JSON with a `boundaries` key.
#[test]
fn show_json_format_outputs_valid_json() {
    let repo = repo_with_boundary_spec(SAMPLE_YAML);
    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("boundaries")
        .arg("show")
        .arg("--format").arg("json")
        .output()
        .unwrap();

    assert!(out.status.success(), "boundaries show --format json must exit 0");
    let stdout = String::from_utf8(out.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .expect("--format json output must be valid JSON");
    assert!(
        parsed["boundaries"].is_array(),
        "JSON must have a 'boundaries' array"
    );
    let names: Vec<&str> = parsed["boundaries"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|b| b["name"].as_str())
        .collect();
    assert!(names.contains(&"api"), "JSON must include boundary 'api'");
    assert!(names.contains(&"models"), "JSON must include boundary 'models'");
}

/// `sdi boundaries show --format json` writes nothing to stderr.
#[test]
fn show_json_no_stderr_contamination() {
    let repo = repo_with_boundary_spec(SAMPLE_YAML);
    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("boundaries")
        .arg("show")
        .arg("--format").arg("json")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        serde_json::from_str::<serde_json::Value>(&stderr).is_err()
            || stderr.trim().is_empty(),
        "stderr must not be valid JSON (CLAUDE.md Rule 8)"
    );
}

/// `sdi boundaries show` with no spec file exits 0 and reports to stderr.
#[test]
fn show_missing_spec_exits_zero() {
    let repo = tempfile::tempdir().unwrap();
    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("boundaries")
        .arg("show")
        .output()
        .unwrap();

    assert!(out.status.success(), "missing spec must exit 0 (normal operation)");
    assert!(out.stdout.is_empty(), "no stdout when spec is missing");
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(!stderr.is_empty(), "must print message to stderr when spec is missing");
}
