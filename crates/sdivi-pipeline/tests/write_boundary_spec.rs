//! Tests for `sdivi_pipeline::store::write_boundary_spec`.
//!
//! Exercises the atomic-write behavior: new-file creation, existing-file
//! replacement (including files with YAML `#` comments), and parent-directory
//! creation.

use sdivi_config::{BoundaryDef, BoundarySpec};
use sdivi_pipeline::store::write_boundary_spec;
use tempfile::TempDir;

fn simple_spec() -> BoundarySpec {
    BoundarySpec {
        version: None,
        boundaries: vec![BoundaryDef {
            name: "api".to_string(),
            description: Some("API layer".to_string()),
            modules: vec!["src/api/**".to_string()],
            allow_imports_from: vec!["models".to_string()],
        }],
    }
}

/// Writing to a path that does not yet exist creates the file.
#[test]
fn write_to_new_file_creates_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("boundaries.yaml");

    assert!(!path.exists(), "precondition: file must not exist");
    write_boundary_spec(&simple_spec(), &path).expect("write_boundary_spec must succeed");
    assert!(path.exists(), "file must be created");
}

/// Written content is valid YAML that round-trips as a `BoundarySpec`.
///
/// Uses `BoundarySpec::load` (which calls serde_yml internally) to verify
/// the file is parseable without adding serde_yml as a direct test dep.
#[test]
fn written_content_is_valid_boundary_spec_yaml() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("boundaries.yaml");

    write_boundary_spec(&simple_spec(), &path).unwrap();

    let loaded = BoundarySpec::load(&path)
        .expect("BoundarySpec::load must succeed")
        .expect("file must exist after write");
    assert_eq!(loaded.boundaries.len(), 1);
    assert_eq!(loaded.boundaries[0].name, "api");
}

/// Writing to an existing file that has YAML `#` comment lines succeeds;
/// the file content is replaced with the new spec.
///
/// This is the store.rs:108-128 warning path (reviewer coverage gap).
/// The function must not panic or return an error — comment-loss is
/// documented behaviour (KDD-6).
#[test]
fn existing_file_with_hash_comment_is_overwritten() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("boundaries.yaml");

    // Write an existing file with YAML comments.
    let original_with_comments = "# This is a handcrafted boundary spec\n\
        boundaries:\n\
        - name: old_boundary\n\
          modules: []\n\
          allow_imports_from: []\n";
    std::fs::write(&path, original_with_comments).unwrap();

    // Call must succeed despite the `#` comment triggering the warning path.
    write_boundary_spec(&simple_spec(), &path)
        .expect("write_boundary_spec must succeed even when existing file has YAML comments");

    // Old content must be gone; new spec must be present.
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(
        !content.contains("old_boundary"),
        "old boundary name must be replaced"
    );
    assert!(
        content.contains("api"),
        "new spec boundary name must be present"
    );
}

/// A file containing ` #` mid-line (inline comment) also triggers the warning
/// and is successfully overwritten.
#[test]
fn existing_file_with_inline_hash_is_overwritten() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("boundaries.yaml");

    // Inline comment (space then #).
    let original_with_inline_comment = "boundaries: [] # auto-generated\n";
    std::fs::write(&path, original_with_inline_comment).unwrap();

    write_boundary_spec(&simple_spec(), &path)
        .expect("write must succeed with inline # comment in existing file");

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("api"),
        "new spec must replace inline-comment content"
    );
}

/// Overwriting replaces the file atomically — content after write is exactly
/// the new spec serialisation, not old content plus new content.
#[test]
fn write_replaces_content_not_appends() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("boundaries.yaml");

    let original = "boundaries:\n- name: first\n  modules: []\n  allow_imports_from: []\n";
    std::fs::write(&path, original).unwrap();

    write_boundary_spec(&simple_spec(), &path).unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(!content.contains("first"), "old content must not persist");
    assert!(content.contains("api"), "new content must be present");
}

/// `write_boundary_spec` creates parent directories if they do not exist.
#[test]
fn write_creates_parent_directories() {
    let dir = TempDir::new().unwrap();
    let nested = dir
        .path()
        .join(".sdivi")
        .join("config")
        .join("boundaries.yaml");

    assert!(
        !nested.parent().unwrap().exists(),
        "precondition: parent must not exist"
    );

    write_boundary_spec(&simple_spec(), &nested)
        .expect("write must create missing parent directories");

    assert!(
        nested.exists(),
        "file must be created inside new parent dirs"
    );
}

/// An empty `BoundarySpec` (no boundaries) is written as valid YAML.
#[test]
fn empty_spec_is_written_as_valid_yaml() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("boundaries.yaml");

    let empty_spec = BoundarySpec {
        version: None,
        boundaries: vec![],
    };
    write_boundary_spec(&empty_spec, &path).unwrap();

    let loaded = BoundarySpec::load(&path)
        .expect("load must succeed")
        .expect("file must exist");
    assert!(loaded.boundaries.is_empty());
}
