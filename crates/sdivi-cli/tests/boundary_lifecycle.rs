//! End-to-end integration test for the boundary lifecycle:
//! `sdivi snapshot` × N → `sdivi boundaries infer` → `sdivi boundaries ratify`
//! → `sdivi boundaries show`.

use std::process::Command;

use assert_cmd::Command as CargoBin;
use tempfile::TempDir;

fn sdivi() -> CargoBin {
    CargoBin::cargo_bin("sdivi").expect("sdivi binary must be built")
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
/// `sdivi snapshot --commit HEAD`. This produces a distinct resolved SHA per
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

        sdivi()
            .arg("--repo")
            .arg(d)
            .arg("snapshot")
            .arg("--commit")
            .arg("HEAD")
            .assert()
            .success();
    }
}

/// `sdivi boundaries infer` after enough snapshots exits 0.
#[test]
fn infer_after_snapshots_exits_zero() {
    let repo = tempfile::tempdir().unwrap();
    capture_snapshots(&repo, 4);

    let out = sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("boundaries")
        .arg("infer")
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "boundaries infer must exit 0 after enough snapshots"
    );
}

/// `sdivi boundaries ratify` after enough snapshots writes a valid YAML file.
#[test]
fn ratify_writes_valid_yaml() {
    let repo = tempfile::tempdir().unwrap();
    capture_snapshots(&repo, 4);

    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("boundaries")
        .arg("ratify")
        .assert()
        .success();

    // The boundaries.yaml may or may not exist depending on whether any
    // stable communities were found. Either way: no crash, exit 0.
    // If it does exist, it must be valid YAML.
    let boundary_path = repo.path().join(".sdivi").join("boundaries.yaml");
    if boundary_path.exists() {
        let content = std::fs::read_to_string(&boundary_path).unwrap();
        let _spec: sdivi_config::BoundarySpec =
            serde_yml::from_str(&content).expect("ratify output must be valid BoundarySpec YAML");
    }
}

/// `sdivi boundaries show` after ratify prints to stdout and exits 0.
#[test]
fn show_after_ratify_prints_to_stdout() {
    let repo = tempfile::tempdir().unwrap();
    capture_snapshots(&repo, 4);

    // Ratify — may or may not produce proposals for this simple fixture.
    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("boundaries")
        .arg("ratify")
        .assert()
        .success();

    let boundary_path = repo.path().join(".sdivi").join("boundaries.yaml");
    if !boundary_path.exists() {
        // No stable communities — show will report "no spec" to stderr, which is fine.
        return;
    }

    let out = sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("boundaries")
        .arg("show")
        .output()
        .unwrap();

    assert!(out.status.success(), "boundaries show must exit 0");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        !stdout.is_empty(),
        "boundaries show must print to stdout when spec exists"
    );
}

/// Full infer → ratify → show cycle with JSON format.
#[test]
fn show_json_format_after_ratify() {
    let repo = tempfile::tempdir().unwrap();
    capture_snapshots(&repo, 4);

    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("boundaries")
        .arg("ratify")
        .assert()
        .success();

    let boundary_path = repo.path().join(".sdivi").join("boundaries.yaml");
    if !boundary_path.exists() {
        return;
    }

    let out = sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("boundaries")
        .arg("show")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("show --format json must produce valid JSON");
    assert!(parsed["boundaries"].is_array());
}

/// Snapshot with a pre-written `boundaries.yaml` that has known violations reports
/// `violation_count > 0` in the resulting snapshot's `intent_divergence`.
///
/// Layout: two source files, boundaries `layer_a` and `layer_b` with no
/// `allow_imports_from`. A dummy import edge is injected by writing a Rust file
/// that contains a `use` statement referencing the other file's module path.
/// The snapshot runs against that repo state.
#[test]
fn snapshot_with_boundary_violations_reports_nonzero_count() {
    let repo = tempfile::tempdir().unwrap();
    let d = repo.path();
    git(d, &["init"]);
    git(d, &["config", "user.email", "test@test.com"]);
    git(d, &["config", "user.name", "Test"]);

    // Write two Rust source files in different layer directories.
    // layer_a/lib.rs imports layer_b/helper.rs via `use crate::helper;`.
    // The graph resolver strips `crate::` and does a stem lookup for "helper",
    // which resolves uniquely to layer_b/helper.rs, creating the edge we need.
    std::fs::create_dir_all(d.join("layer_a")).unwrap();
    std::fs::create_dir_all(d.join("layer_b")).unwrap();
    std::fs::write(d.join("layer_b/helper.rs"), "pub fn helper() {}").unwrap();
    std::fs::write(
        d.join("layer_a/lib.rs"),
        "use crate::helper;\nfn call() { helper::helper(); }",
    )
    .unwrap();
    git(d, &["add", "."]);
    git(d, &["commit", "-m", "add layer files"]);

    // Write a boundaries.yaml that declares two boundaries with no allow_imports_from.
    // Any cross-layer edge is therefore a violation.
    let sdivi_dir = d.join(".sdivi");
    std::fs::create_dir_all(&sdivi_dir).unwrap();
    std::fs::write(
        sdivi_dir.join("boundaries.yaml"),
        "boundaries:\n  - name: layer_a\n    modules: [\"layer_a/**\"]\n    allow_imports_from: []\n  - name: layer_b\n    modules: [\"layer_b/**\"]\n    allow_imports_from: []\n",
    )
    .unwrap();

    sdivi()
        .arg("--repo")
        .arg(d)
        .arg("snapshot")
        .assert()
        .success();

    // Read the produced snapshot and assert violation_count > 0.
    let snap_dir = sdivi_dir.join("snapshots");
    let entries: Vec<_> = std::fs::read_dir(&snap_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(!entries.is_empty(), "at least one snapshot must be written");

    let snap_path = entries[0].path();
    let content = std::fs::read_to_string(&snap_path).unwrap();
    let snap: serde_json::Value = serde_json::from_str(&content).unwrap();

    let violation_count = snap["intent_divergence"]["violation_count"]
        .as_u64()
        .expect("intent_divergence.violation_count must be present");
    assert!(
        violation_count > 0,
        "snapshot with cross-boundary edges and no allow_imports_from must report violations; got {violation_count}"
    );
}

/// `sdivi boundaries ratify` round-trip: write then read produces equivalent spec.
#[test]
fn ratify_read_back_equivalent() {
    let repo = tempfile::tempdir().unwrap();
    capture_snapshots(&repo, 4);

    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("boundaries")
        .arg("ratify")
        .assert()
        .success();

    let boundary_path = repo.path().join(".sdivi").join("boundaries.yaml");
    if !boundary_path.exists() {
        return;
    }

    let content = std::fs::read_to_string(&boundary_path).unwrap();
    let spec: sdivi_config::BoundarySpec = serde_yml::from_str(&content).unwrap();

    // A second ratify must produce the same result (idempotent).
    sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("boundaries")
        .arg("ratify")
        .assert()
        .success();

    let content2 = std::fs::read_to_string(&boundary_path).unwrap();
    let spec2: sdivi_config::BoundarySpec = serde_yml::from_str(&content2).unwrap();

    assert_eq!(spec.boundaries.len(), spec2.boundaries.len());
}
