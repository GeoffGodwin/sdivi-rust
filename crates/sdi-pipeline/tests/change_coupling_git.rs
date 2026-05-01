//! Integration test: collect_cochange_events against a real tempdir git repo.

use std::process::Command;
use tempfile::TempDir;
use sdi_pipeline::change_coupling::collect_cochange_events;

fn git(dir: &std::path::Path, args: &[&str]) {
    let status = Command::new("git")
        .current_dir(dir)
        .args(args)
        .status()
        .expect("git command failed");
    assert!(status.success(), "git {:?} failed", args);
}

fn write_and_commit(dir: &std::path::Path, files: &[(&str, &str)], msg: &str) {
    for (name, content) in files {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, content).unwrap();
        git(dir, &["add", name]);
    }
    git(dir, &["commit", "--allow-empty-message", "-m", msg]);
}

fn setup_repo(tmp: &TempDir) -> std::path::PathBuf {
    let dir = tmp.path().to_path_buf();
    git(&dir, &["init"]);
    git(&dir, &["config", "user.email", "test@test.com"]);
    git(&dir, &["config", "user.name", "Test"]);
    dir
}

#[test]
fn collect_events_from_known_commits() {
    let tmp = TempDir::new().unwrap();
    let dir = setup_repo(&tmp);

    write_and_commit(&dir, &[("src/a.rs", "a"), ("src/b.rs", "b")], "add a and b");
    write_and_commit(&dir, &[("src/a.rs", "a2"), ("src/b.rs", "b2")], "update a and b");
    write_and_commit(&dir, &[("src/c.rs", "c")], "add c");

    let events = collect_cochange_events(&dir, 100, None).unwrap();
    assert_eq!(events.len(), 3);

    // Events are oldest-first.
    assert!(events[0].files.contains(&"src/a.rs".to_string()));
    assert!(events[0].files.contains(&"src/b.rs".to_string()));
}

#[test]
fn no_git_returns_empty() {
    let tmp = TempDir::new().unwrap();
    // No git init — no .git/ directory.
    let events = collect_cochange_events(tmp.path(), 100, None).unwrap();
    assert!(events.is_empty());
}

#[test]
fn history_depth_limits_events() {
    let tmp = TempDir::new().unwrap();
    let dir = setup_repo(&tmp);

    for i in 0..5 {
        write_and_commit(&dir, &[(&format!("src/f{i}.rs"), "x")], &format!("commit {i}"));
    }

    let events = collect_cochange_events(&dir, 3, None).unwrap();
    assert_eq!(events.len(), 3);
}

#[test]
fn path_with_space_round_trips() {
    let tmp = TempDir::new().unwrap();
    let dir = setup_repo(&tmp);

    write_and_commit(&dir, &[("src/my file.rs", "x"), ("src/b.rs", "y")], "space in name");
    write_and_commit(&dir, &[("src/my file.rs", "x2"), ("src/b.rs", "y2")], "space again");

    let events = collect_cochange_events(&dir, 100, None).unwrap();
    let has_space = events.iter().any(|e| e.files.iter().any(|f| f.contains("my file")));
    assert!(has_space, "path with space should round-trip");
}
