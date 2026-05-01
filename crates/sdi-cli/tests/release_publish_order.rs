/// Coverage Gap 1 (REVIEWER_REPORT.md): Validates that `release.yml`
/// publishes crates in the correct dependency order so that crates.io
/// index propagation sleeps come after a crate's dependencies are live.
///
/// Specifically: sdi-parsing must be published before any sdi-lang-* crate,
/// because all six language adapter crates depend on sdi-parsing.
///
/// These tests parse the workflow YAML as text rather than building the WASM
/// bundle — they catch publish-order regressions without requiring CI secrets.

fn release_yml() -> String {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let release_yml = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(".github")
        .join("workflows")
        .join("release.yml");
    std::fs::read_to_string(&release_yml)
        .unwrap_or_else(|_| panic!("could not read {}", release_yml.display()))
}

/// Returns the byte offset of the first occurrence of `needle` in `haystack`,
/// or panics with a descriptive message.
fn find_offset(haystack: &str, needle: &str) -> usize {
    haystack
        .find(needle)
        .unwrap_or_else(|| panic!("expected to find {:?} in release.yml", needle))
}

// ── Core ordering invariants ───────────────────────────────────────────────

#[test]
fn sdi_config_published_before_sdi_parsing() {
    let yml = release_yml();
    let config_pos = find_offset(&yml, "cargo publish -p sdi-config");
    let parsing_pos = find_offset(&yml, "cargo publish -p sdi-parsing");
    assert!(
        config_pos < parsing_pos,
        "sdi-config must be published before sdi-parsing (sdi-parsing depends on sdi-config)"
    );
}

#[test]
fn sdi_parsing_published_before_lang_rust() {
    let yml = release_yml();
    let parsing_pos = find_offset(&yml, "cargo publish -p sdi-parsing");
    let lang_pos = find_offset(&yml, "cargo publish -p sdi-lang-rust");
    assert!(
        parsing_pos < lang_pos,
        "sdi-parsing must be published before sdi-lang-rust (sdi-lang-rust depends on sdi-parsing)"
    );
}

#[test]
fn sdi_parsing_published_before_lang_python() {
    let yml = release_yml();
    let parsing_pos = find_offset(&yml, "cargo publish -p sdi-parsing");
    let lang_pos = find_offset(&yml, "cargo publish -p sdi-lang-python");
    assert!(
        parsing_pos < lang_pos,
        "sdi-parsing must be published before sdi-lang-python"
    );
}

#[test]
fn sdi_parsing_published_before_lang_typescript() {
    let yml = release_yml();
    let parsing_pos = find_offset(&yml, "cargo publish -p sdi-parsing");
    let lang_pos = find_offset(&yml, "cargo publish -p sdi-lang-typescript");
    assert!(
        parsing_pos < lang_pos,
        "sdi-parsing must be published before sdi-lang-typescript"
    );
}

#[test]
fn sdi_parsing_published_before_lang_javascript() {
    let yml = release_yml();
    let parsing_pos = find_offset(&yml, "cargo publish -p sdi-parsing");
    let lang_pos = find_offset(&yml, "cargo publish -p sdi-lang-javascript");
    assert!(
        parsing_pos < lang_pos,
        "sdi-parsing must be published before sdi-lang-javascript"
    );
}

#[test]
fn sdi_parsing_published_before_lang_go() {
    let yml = release_yml();
    let parsing_pos = find_offset(&yml, "cargo publish -p sdi-parsing");
    let lang_pos = find_offset(&yml, "cargo publish -p sdi-lang-go");
    assert!(
        parsing_pos < lang_pos,
        "sdi-parsing must be published before sdi-lang-go"
    );
}

#[test]
fn sdi_parsing_published_before_lang_java() {
    let yml = release_yml();
    let parsing_pos = find_offset(&yml, "cargo publish -p sdi-parsing");
    let lang_pos = find_offset(&yml, "cargo publish -p sdi-lang-java");
    assert!(
        parsing_pos < lang_pos,
        "sdi-parsing must be published before sdi-lang-java"
    );
}

// ── Mid-chain ordering invariants ─────────────────────────────────────────

#[test]
fn lang_adapters_published_before_sdi_graph() {
    let yml = release_yml();
    // Use the last lang adapter as the reference point — all six are in the
    // same `run: |` block, so the block's start is before sdi-graph's step.
    let lang_block_pos = find_offset(&yml, "cargo publish -p sdi-lang-rust");
    let graph_pos = find_offset(&yml, "cargo publish -p sdi-graph");
    assert!(
        lang_block_pos < graph_pos,
        "sdi-lang-* adapters must be published before sdi-graph"
    );
}

#[test]
fn sdi_graph_published_before_sdi_detection() {
    let yml = release_yml();
    let graph_pos = find_offset(&yml, "cargo publish -p sdi-graph");
    let detection_pos = find_offset(&yml, "cargo publish -p sdi-detection");
    assert!(
        graph_pos < detection_pos,
        "sdi-graph must be published before sdi-detection"
    );
}

#[test]
fn sdi_detection_published_before_sdi_patterns() {
    let yml = release_yml();
    let detection_pos = find_offset(&yml, "cargo publish -p sdi-detection");
    let patterns_pos = find_offset(&yml, "cargo publish -p sdi-patterns");
    assert!(
        detection_pos < patterns_pos,
        "sdi-detection must be published before sdi-patterns"
    );
}

#[test]
fn sdi_patterns_published_before_sdi_snapshot() {
    let yml = release_yml();
    let patterns_pos = find_offset(&yml, "cargo publish -p sdi-patterns");
    let snapshot_pos = find_offset(&yml, "cargo publish -p sdi-snapshot");
    assert!(
        patterns_pos < snapshot_pos,
        "sdi-patterns must be published before sdi-snapshot"
    );
}

#[test]
fn sdi_snapshot_published_before_sdi_core() {
    let yml = release_yml();
    let snapshot_pos = find_offset(&yml, "cargo publish -p sdi-snapshot");
    let core_pos = find_offset(&yml, "cargo publish -p sdi-core");
    assert!(
        snapshot_pos < core_pos,
        "sdi-snapshot must be published before sdi-core"
    );
}

#[test]
fn sdi_core_published_before_sdi_pipeline() {
    let yml = release_yml();
    let core_pos = find_offset(&yml, "cargo publish -p sdi-core");
    let pipeline_pos = find_offset(&yml, "cargo publish -p sdi-pipeline");
    assert!(
        core_pos < pipeline_pos,
        "sdi-core must be published before sdi-pipeline"
    );
}

#[test]
fn sdi_pipeline_published_before_sdi_cli() {
    let yml = release_yml();
    let pipeline_pos = find_offset(&yml, "cargo publish -p sdi-pipeline");
    let cli_pos = find_offset(&yml, "cargo publish -p sdi-cli");
    assert!(
        pipeline_pos < cli_pos,
        "sdi-pipeline must be published before sdi-cli"
    );
}

#[test]
fn sdi_cli_published_before_sdi_rust() {
    let yml = release_yml();
    let cli_pos = find_offset(&yml, "cargo publish -p sdi-cli");
    let rust_pos = find_offset(&yml, "cargo publish -p sdi-rust");
    assert!(
        cli_pos < rust_pos,
        "sdi-cli must be published before sdi-rust (sdi-rust depends on sdi-cli)"
    );
}

// ── Sleep guards between batches ──────────────────────────────────────────
// Verify that 30-second propagation sleeps exist between key publish batches
// so crates.io index has time to register each batch before dependents publish.

#[test]
fn sleep_after_sdi_config_before_sdi_parsing() {
    let yml = release_yml();
    let config_pos = find_offset(&yml, "cargo publish -p sdi-config");
    let parsing_pos = find_offset(&yml, "cargo publish -p sdi-parsing");
    let between = &yml[config_pos..parsing_pos];
    assert!(
        between.contains("sleep 30"),
        "a 'sleep 30' must appear between publishing sdi-config and sdi-parsing"
    );
}

#[test]
fn sleep_after_sdi_parsing_before_lang_adapters() {
    let yml = release_yml();
    let parsing_pos = find_offset(&yml, "cargo publish -p sdi-parsing");
    // The first lang adapter in the run block
    let lang_pos = find_offset(&yml, "cargo publish -p sdi-lang-rust");
    let between = &yml[parsing_pos..lang_pos];
    assert!(
        between.contains("sleep 30"),
        "a 'sleep 30' must appear between publishing sdi-parsing and the sdi-lang-* block"
    );
}
