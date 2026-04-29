//! Verifies the CST-drop invariant: no tree-sitter Tree survives parse_file.
//!
//! Uses the `test-tree-counter` feature (enabled via dev-dependency on
//! sdi-lang-rust with that feature) to track live Tree counts.

use sdi_config::Config;
use sdi_lang_rust::RustAdapter;
use sdi_parsing::parse::parse_repository;
use std::sync::atomic::Ordering;

// Serializes tests that read/write ACTIVE_TREES to prevent spurious failures
// when cargo test runs them in parallel on the same process.
static COUNTER_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Generates a syntactically valid Rust source file of approximately `size`
/// bytes by repeating simple function declarations.
fn generate_rust_source(size: usize) -> String {
    let chunk = "pub fn placeholder_fn(x: u64) -> u64 { x.wrapping_add(1) }\n";
    let repeats = (size / chunk.len()).max(1);
    chunk.repeat(repeats)
}

#[test]
#[cfg(feature = "test-tree-counter")]
fn tree_counter_zero_after_each_parse() {
    use sdi_parsing::ACTIVE_TREES;

    let _guard = COUNTER_LOCK.lock().unwrap();

    let adapter = RustAdapter;
    use sdi_parsing::adapter::LanguageAdapter;

    for _ in 0..10 {
        let content = generate_rust_source(4_096);
        let _record = adapter.parse_file(std::path::Path::new("test.rs"), content);
        let live = ACTIVE_TREES.load(Ordering::SeqCst);
        assert_eq!(live, 0, "ACTIVE_TREES must be 0 after parse_file returns");
    }
}

#[test]
fn parse_many_large_files_completes() {
    let _guard = COUNTER_LOCK.lock().unwrap();

    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");
    std::fs::create_dir_all(&src).unwrap();

    // Write 20 ~50KB synthetic Rust files.
    for i in 0..20 {
        let content = generate_rust_source(50_000);
        std::fs::write(src.join(format!("file_{i:02}.rs")), content).unwrap();
    }

    let mut config = Config::default();
    config.core.exclude = vec!["**/target/**".to_string()];

    let adapters: Vec<Box<dyn sdi_parsing::adapter::LanguageAdapter>> =
        vec![Box::new(RustAdapter)];

    let records = parse_repository(&config, dir.path(), &adapters);
    assert_eq!(records.len(), 20, "all 20 files should produce FeatureRecords");
}
