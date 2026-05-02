//! Workspace-level end-to-end test: snapshot a repo and verify change_coupling.

use sdivi_config::Config;
use sdivi_pipeline::Pipeline;
use std::process::Command;
use tempfile::TempDir;

fn git(dir: &std::path::Path, args: &[&str]) {
    let status = Command::new("git")
        .current_dir(dir)
        .args(args)
        .status()
        .expect("git command failed");
    assert!(status.success(), "git {:?} failed", args);
}

fn setup_repo_with_history(tmp: &TempDir) -> std::path::PathBuf {
    let dir = tmp.path().to_path_buf();
    git(&dir, &["init"]);
    git(&dir, &["config", "user.email", "t@t.com"]);
    git(&dir, &["config", "user.name", "T"]);

    // Create two files that co-change twice.
    // Files have no extension so the pipeline's NoGrammarsAvailable check is
    // not triggered (only extensioned files are considered source candidates).
    for i in 0..2u32 {
        std::fs::write(dir.join("fileA"), format!("content A {}", i)).unwrap();
        std::fs::write(dir.join("fileB"), format!("content B {}", i)).unwrap();
        git(&dir, &["add", "fileA", "fileB"]);
        git(&dir, &["commit", "-m", &format!("commit {i}")]);
    }
    dir
}

#[test]
fn snapshot_populates_change_coupling() {
    let tmp = TempDir::new().unwrap();
    let dir = setup_repo_with_history(&tmp);

    // Create .sdivi/snapshots dir.
    std::fs::create_dir_all(dir.join(".sdivi/snapshots")).unwrap();

    let cfg = Config::default();
    let pipeline = Pipeline::new(cfg, vec![]);
    let snap = pipeline
        .snapshot_with_mode(
            &dir,
            None,
            "2026-05-01T00:00:00Z",
            sdivi_pipeline::WriteMode::EphemeralForCheck,
        )
        .unwrap();

    // Should have change_coupling populated with the (fileA, fileB) pair.
    assert!(snap.change_coupling.is_some());
    let cc = snap.change_coupling.unwrap();
    assert_eq!(cc.commits_analyzed, 2);
    assert_eq!(cc.pairs.len(), 1);
    assert!((cc.pairs[0].frequency - 1.0).abs() < 1e-9);
}

#[test]
fn no_git_produces_none_change_coupling() {
    let tmp = TempDir::new().unwrap();
    // No git init — no .git/ directory.
    std::fs::create_dir_all(tmp.path().join(".sdivi/snapshots")).unwrap();

    let cfg = Config::default();
    let pipeline = Pipeline::new(cfg, vec![]);
    let snap = pipeline
        .snapshot_with_mode(
            tmp.path(),
            None,
            "2026-05-01T00:00:00Z",
            sdivi_pipeline::WriteMode::EphemeralForCheck,
        )
        .unwrap();

    assert!(snap.change_coupling.is_none());
}
