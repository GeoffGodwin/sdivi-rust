//! End-to-end integration test for the boundary lifecycle:
//! `sdi snapshot` × N → `sdi boundaries infer` → `sdi boundaries ratify`
//! → `sdi boundaries show`.

use std::process::Command;

use assert_cmd::Command as CargoBin;
use tempfile::TempDir;

fn sdi() -> CargoBin {
    CargoBin::cargo_bin("sdi").expect("sdi binary must be built")
}

fn git(dir: &std::path::Path, args: &[&str]) {
    let status = Command::new("git")
        .current_dir(dir)
        .args(args)
        .status()
        .expect("git must be available");
    assert!(status.success(), "git {args:?} failed");
}

/// Creates `n` snapshots by committing a new file per iteration and running
/// `sdi snapshot --commit HEAD`. This produces a distinct resolved SHA per
/// snapshot (→ unique filename), and a genuinely different tree at each point.
fn capture_snapshots(repo: &TempDir, n: usize) {
    let d = repo.path();
    git(d, &["init"]);
    git(d, &["config", "user.email", "test@test.com"]);
    git(d, &["config", "user.name", "Test"]);

    for i in 0..n {
        let fname = format!("file{i}.rs");
        std::fs::write(d.join(&fname), format!("fn f{i}() {{}}")).unwrap();
        git(d, &["add", &fname]);
        git(d, &["commit", "-m", &format!("add file{i}")]);

        sdi()
            .arg("--repo").arg(d)
            .arg("snapshot")
            .arg("--commit").arg("HEAD")
            .assert()
            .success();
    }
}

/// `sdi boundaries infer` after enough snapshots exits 0.
#[test]
fn infer_after_snapshots_exits_zero() {
    let repo = tempfile::tempdir().unwrap();
    capture_snapshots(&repo, 4);

    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("boundaries")
        .arg("infer")
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "boundaries infer must exit 0 after enough snapshots"
    );
}

/// `sdi boundaries ratify` after enough snapshots writes a valid YAML file.
#[test]
fn ratify_writes_valid_yaml() {
    let repo = tempfile::tempdir().unwrap();
    capture_snapshots(&repo, 4);

    sdi()
        .arg("--repo").arg(repo.path())
        .arg("boundaries")
        .arg("ratify")
        .assert()
        .success();

    // The boundaries.yaml may or may not exist depending on whether any
    // stable communities were found. Either way: no crash, exit 0.
    // If it does exist, it must be valid YAML.
    let boundary_path = repo.path().join(".sdi").join("boundaries.yaml");
    if boundary_path.exists() {
        let content = std::fs::read_to_string(&boundary_path).unwrap();
        let _spec: sdi_config::BoundarySpec = serde_yml::from_str(&content)
            .expect("ratify output must be valid BoundarySpec YAML");
    }
}

/// `sdi boundaries show` after ratify prints to stdout and exits 0.
#[test]
fn show_after_ratify_prints_to_stdout() {
    let repo = tempfile::tempdir().unwrap();
    capture_snapshots(&repo, 4);

    // Ratify — may or may not produce proposals for this simple fixture.
    sdi()
        .arg("--repo").arg(repo.path())
        .arg("boundaries")
        .arg("ratify")
        .assert()
        .success();

    let boundary_path = repo.path().join(".sdi").join("boundaries.yaml");
    if !boundary_path.exists() {
        // No stable communities — show will report "no spec" to stderr, which is fine.
        return;
    }

    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("boundaries")
        .arg("show")
        .output()
        .unwrap();

    assert!(out.status.success(), "boundaries show must exit 0");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(!stdout.is_empty(), "boundaries show must print to stdout when spec exists");
}

/// Full infer → ratify → show cycle with JSON format.
#[test]
fn show_json_format_after_ratify() {
    let repo = tempfile::tempdir().unwrap();
    capture_snapshots(&repo, 4);

    sdi()
        .arg("--repo").arg(repo.path())
        .arg("boundaries")
        .arg("ratify")
        .assert()
        .success();

    let boundary_path = repo.path().join(".sdi").join("boundaries.yaml");
    if !boundary_path.exists() {
        return;
    }

    let out = sdi()
        .arg("--repo").arg(repo.path())
        .arg("boundaries")
        .arg("show")
        .arg("--format").arg("json")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .expect("show --format json must produce valid JSON");
    assert!(parsed["boundaries"].is_array());
}

/// `sdi boundaries ratify` round-trip: write then read produces equivalent spec.
#[test]
fn ratify_read_back_equivalent() {
    let repo = tempfile::tempdir().unwrap();
    capture_snapshots(&repo, 4);

    sdi()
        .arg("--repo").arg(repo.path())
        .arg("boundaries")
        .arg("ratify")
        .assert()
        .success();

    let boundary_path = repo.path().join(".sdi").join("boundaries.yaml");
    if !boundary_path.exists() {
        return;
    }

    let content = std::fs::read_to_string(&boundary_path).unwrap();
    let spec: sdi_config::BoundarySpec = serde_yml::from_str(&content).unwrap();

    // A second ratify must produce the same result (idempotent).
    sdi()
        .arg("--repo").arg(repo.path())
        .arg("boundaries")
        .arg("ratify")
        .assert()
        .success();

    let content2 = std::fs::read_to_string(&boundary_path).unwrap();
    let spec2: sdi_config::BoundarySpec = serde_yml::from_str(&content2).unwrap();

    assert_eq!(spec.boundaries.len(), spec2.boundaries.len());
}
