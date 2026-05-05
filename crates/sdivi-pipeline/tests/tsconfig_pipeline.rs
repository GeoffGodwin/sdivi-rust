//! Pipeline acceptance tests for M27 tsconfig path-alias support.
//!
//! Verifies the two acceptance criteria that require a running pipeline:
//! 1. A repo with malformed `tsconfig.json` produces a successful snapshot
//!    (alias resolution is disabled gracefully, no crash, no error propagated).
//! 2. A repo with `jsconfig.json` (and no `tsconfig.json`) is used as the
//!    alias source; pipeline succeeds.
//!
//! Both tests copy the `simple-rust` fixture into a tempdir (since RustAdapter
//! is the only adapter available in sdivi-pipeline dev-deps) and inject a
//! synthetic tsconfig / jsconfig file.  Rust files contain no TS-style alias
//! imports, so edge counts are not the point — the point is that the pipeline
//! stage-2 tsconfig discovery path does not abort the run on either file shape.

use sdivi_config::Config;
use sdivi_lang_rust::RustAdapter;
use sdivi_pipeline::{Pipeline, WriteMode};
use std::path::{Path, PathBuf};

fn fixture_root() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/simple-rust"
    ))
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        let name = entry.file_name();
        let from = entry.path();
        let to = dst.join(&name);
        if ft.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else if ft.is_file() {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn isolated_fixture_with_extra(extra_name: &str, extra_content: &str) -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::TempDir::new().expect("temp dir");
    let dst = tmp.path().join("repo");
    copy_dir_recursive(fixture_root(), &dst).expect("copy fixture");
    std::fs::write(dst.join(extra_name), extra_content).expect("write extra file");
    (tmp, dst)
}

fn run_ephemeral(root: &Path) -> Result<sdivi_snapshot::Snapshot, sdivi_pipeline::PipelineError> {
    let adapters: Vec<Box<dyn sdivi_parsing::adapter::LanguageAdapter>> =
        vec![Box::new(RustAdapter)];
    let pipeline = Pipeline::new(Config::default(), adapters);
    pipeline.snapshot_with_mode(root, None, "2026-05-04T00:00:00Z", WriteMode::EphemeralForCheck)
}

// ── malformed tsconfig.json ───────────────────────────────────────────────────

#[test]
fn malformed_tsconfig_json_snapshot_succeeds() {
    // Acceptance criterion: "A repo with malformed tsconfig.json produces a
    // stderr WARN and a successful snapshot (alias resolution disabled, other
    // resolution unaffected)."
    //
    // The malformed content fails to parse even after comment-stripping, so
    // parse_tsconfig_content returns None and the stage-2 graph builder
    // proceeds with tsconfig = None.  The pipeline must not propagate an error.
    let malformed = r#"{ not valid json at all }"#;
    let (_tmp, root) = isolated_fixture_with_extra("tsconfig.json", malformed);

    let result = run_ephemeral(&root);
    assert!(
        result.is_ok(),
        "malformed tsconfig.json must not abort the pipeline; got: {:?}",
        result.err()
    );
    let snap = result.unwrap();
    // Rust files have no alias imports, so edge count is 0; node count confirms
    // parsing still ran correctly.
    assert!(
        snap.graph.node_count > 0,
        "parsing must still produce nodes despite malformed tsconfig"
    );
}

// ── jsconfig.json fallback ────────────────────────────────────────────────────

#[test]
fn jsconfig_json_present_no_tsconfig_snapshot_succeeds() {
    // Acceptance criterion (implicit from spec): when tsconfig.json is absent
    // but jsconfig.json is present, read_tsconfig_paths must use jsconfig.json.
    //
    // With Rust source files there are no TS-style alias imports, so edge
    // resolution is unaffected.  The test verifies the pipeline does not crash
    // and produces a well-formed snapshot — confirming jsconfig.json is accepted
    // without error.
    let jsconfig = r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": { "@/*": ["./*"] }
  }
}"#;
    let (_tmp, root) = isolated_fixture_with_extra("jsconfig.json", jsconfig);

    // Confirm tsconfig.json is absent (we only added jsconfig.json).
    assert!(
        !root.join("tsconfig.json").exists(),
        "tsconfig.json must not exist in this fixture"
    );
    assert!(
        root.join("jsconfig.json").exists(),
        "jsconfig.json must exist in this fixture"
    );

    let result = run_ephemeral(&root);
    assert!(
        result.is_ok(),
        "jsconfig.json fallback must not abort the pipeline; got: {:?}",
        result.err()
    );
    let snap = result.unwrap();
    assert!(
        snap.graph.node_count > 0,
        "parsing must still produce nodes when jsconfig.json is present"
    );
}

// ── no config file ────────────────────────────────────────────────────────────

#[test]
fn no_tsconfig_nor_jsconfig_snapshot_succeeds() {
    // Regression guard: neither file present → alias resolution silently
    // disabled, snapshot still valid.  Mirrors the unit-level
    // no_tsconfig_alias_specifier_is_external test at the pipeline level.
    let (_tmp, root) = {
        let tmp = tempfile::TempDir::new().expect("temp dir");
        let dst = tmp.path().join("repo");
        copy_dir_recursive(fixture_root(), &dst).expect("copy fixture");
        (tmp, dst)
    };

    assert!(!root.join("tsconfig.json").exists());
    assert!(!root.join("jsconfig.json").exists());

    let result = run_ephemeral(&root);
    assert!(
        result.is_ok(),
        "absent tsconfig must not abort pipeline; got: {:?}",
        result.err()
    );
}
