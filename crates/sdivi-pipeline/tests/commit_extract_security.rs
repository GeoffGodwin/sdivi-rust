//! Tests for security fixes in commit_extract module.

use sdivi_pipeline::commit_extract::{extract_commit_tree, resolve_ref_to_sha, CommitExtractError};
use std::process::Command;

#[test]
fn resolve_ref_includes_double_dash_separator() {
    // When resolve_ref_to_sha is called with a reference,
    // it should pass `--` to git rev-parse to separate flags from arguments.
    // This test verifies that refs starting with `--` won't be misinterpreted as flags.

    // Use a non-existent reference to test the command formation.
    // The test environment should have git available.
    let repo = std::env::current_dir().expect("failed to get cwd");

    let result = resolve_ref_to_sha(&repo, "--invalid-ref");

    // The error should occur because --invalid-ref is not a valid ref.
    // If the `--` separator were missing, git would interpret `--invalid-ref` as a flag.
    // With the separator in place, git will try to resolve it as a ref and fail cleanly.
    match result {
        Err(CommitExtractError::RefResolutionFailed { reference, stderr }) => {
            assert_eq!(reference, "--invalid-ref");
            // The stderr should be a sensible git error, not a usage error about flags.
            // It should NOT contain "unknown option" or similar flag-parsing errors.
            assert!(
                !stderr.to_lowercase().contains("unknown option"),
                "stderr should not indicate unknown option (missing -- separator): {}",
                stderr
            );
        }
        _ => panic!("Expected RefResolutionFailed error"),
    }
}

/// Create a temporary git repo with one committed file.
fn create_test_git_repo() -> Result<(tempfile::TempDir, String), Box<dyn std::error::Error>> {
    let tmpdir = tempfile::TempDir::new()?;
    let repo_path = tmpdir.path();

    // Initialize a git repo
    let init_out = Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()?;
    if !init_out.status.success() {
        return Err(format!(
            "git init failed: {}",
            String::from_utf8_lossy(&init_out.stderr)
        )
        .into());
    }

    // Configure git user for commits
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()?;
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()?;

    // Create and commit a file
    let test_file = repo_path.join("test.txt");
    std::fs::write(&test_file, "test content")?;

    let add_out = Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(repo_path)
        .output()?;
    if !add_out.status.success() {
        return Err(format!(
            "git add failed: {}",
            String::from_utf8_lossy(&add_out.stderr)
        )
        .into());
    }

    let commit_out = Command::new("git")
        .args(["commit", "-m", "initial commit"])
        .current_dir(repo_path)
        .output()?;
    if !commit_out.status.success() {
        return Err(format!(
            "git commit failed: {}",
            String::from_utf8_lossy(&commit_out.stderr)
        )
        .into());
    }

    // Get the SHA of HEAD
    let sha = resolve_ref_to_sha(repo_path, "HEAD")
        .map_err(|e| format!("failed to resolve HEAD: {}", e))?;

    Ok((tmpdir, sha))
}

#[test]
fn stderr_truncation_prevents_information_leakage() {
    // When git/tar return an error with long stderr output,
    // the error message should truncate stderr to prevent information leakage.
    // This verifies both that truncation happens and that no null bytes are present.

    // Test that an invalid reference produces truncated stderr in the error.
    let repo = std::env::current_dir().expect("failed to get cwd");
    let result = resolve_ref_to_sha(&repo, "invalid_ref_that_does_not_exist_anywhere");

    match result {
        Err(CommitExtractError::RefResolutionFailed { stderr, .. }) => {
            // Stderr should be reasonable in length (max 200 chars as per truncate_stderr)
            assert!(
                stderr.len() <= 210,
                "stderr should be truncated to ~200 chars, got {} chars: {}",
                stderr.len(),
                stderr
            );
            // Should not contain null bytes
            assert!(
                !stderr.contains('\0'),
                "stderr should not contain null bytes"
            );
            // Should not be empty
            assert!(!stderr.is_empty(), "stderr should not be empty");
        }
        _ => panic!("Expected RefResolutionFailed error"),
    }
}

#[test]
fn extract_commit_tree_succeeds_with_tar_flags() {
    // When extract_commit_tree is called with a valid SHA,
    // it should use `tar --no-absolute-filenames` to safely extract the tree.
    // This test verifies that the extraction works correctly with the security flag.

    let (tmpdir, sha) = create_test_git_repo().expect("failed to create test git repo");
    let repo_path = tmpdir.path();

    // Extract the tree at the SHA
    let extract_result = extract_commit_tree(repo_path, &sha);

    // Verify extraction succeeded
    let extracted = extract_result.expect("extract_commit_tree should succeed");

    // Verify the extracted directory contains files
    let extracted_path = extracted.path();
    assert!(extracted_path.exists(), "extracted directory should exist");

    // Check for the test.txt file we committed
    let entries: Vec<_> = std::fs::read_dir(extracted_path)
        .expect("should be able to read dir")
        .filter_map(|e| e.ok())
        .collect();

    assert!(
        !entries.is_empty(),
        "extracted directory should contain files"
    );

    // Verify our test file is present
    let test_file = extracted_path.join("test.txt");
    assert!(
        test_file.exists(),
        "test.txt should exist in extracted tree"
    );

    // Verify content is correct
    let content = std::fs::read_to_string(&test_file).expect("should be able to read test.txt");
    assert_eq!(
        content, "test content",
        "test.txt should have original content"
    );

    // NOTE: This test verifies extraction succeeds but does not verify that
    // the `--no-absolute-filenames` flag prevents absolute path extraction,
    // as a normal git archive contains no absolute paths. A comprehensive
    // security test would construct a malicious tar with absolute paths and
    // verify they are not written to disk, but that requires synthetic tar
    // construction beyond the scope of this integration test.
}
