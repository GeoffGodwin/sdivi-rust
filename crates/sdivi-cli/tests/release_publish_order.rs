/// Coverage Gap 1 (REVIEWER_REPORT.md): Validates that `release.yml`
/// publishes crates in the correct dependency order so that crates.io
/// index propagation sleeps come after a crate's dependencies are live.
///
/// Specifically: sdivi-parsing must be published before any sdivi-lang-* crate,
/// because all six language adapter crates depend on sdivi-parsing.
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
fn sdivi_config_published_before_sdivi_parsing() {
    let yml = release_yml();
    let config_pos = find_offset(&yml, "cargo publish -p sdivi-config");
    let parsing_pos = find_offset(&yml, "cargo publish -p sdivi-parsing");
    assert!(
        config_pos < parsing_pos,
        "sdivi-config must be published before sdivi-parsing (sdivi-parsing depends on sdivi-config)"
    );
}

#[test]
fn sdivi_parsing_published_before_lang_rust() {
    let yml = release_yml();
    let parsing_pos = find_offset(&yml, "cargo publish -p sdivi-parsing");
    let lang_pos = find_offset(&yml, "cargo publish -p sdivi-lang-rust");
    assert!(
        parsing_pos < lang_pos,
        "sdivi-parsing must be published before sdivi-lang-rust (sdivi-lang-rust depends on sdivi-parsing)"
    );
}

#[test]
fn sdivi_parsing_published_before_lang_python() {
    let yml = release_yml();
    let parsing_pos = find_offset(&yml, "cargo publish -p sdivi-parsing");
    let lang_pos = find_offset(&yml, "cargo publish -p sdivi-lang-python");
    assert!(
        parsing_pos < lang_pos,
        "sdivi-parsing must be published before sdivi-lang-python"
    );
}

#[test]
fn sdivi_parsing_published_before_lang_typescript() {
    let yml = release_yml();
    let parsing_pos = find_offset(&yml, "cargo publish -p sdivi-parsing");
    let lang_pos = find_offset(&yml, "cargo publish -p sdivi-lang-typescript");
    assert!(
        parsing_pos < lang_pos,
        "sdivi-parsing must be published before sdivi-lang-typescript"
    );
}

#[test]
fn sdivi_parsing_published_before_lang_javascript() {
    let yml = release_yml();
    let parsing_pos = find_offset(&yml, "cargo publish -p sdivi-parsing");
    let lang_pos = find_offset(&yml, "cargo publish -p sdivi-lang-javascript");
    assert!(
        parsing_pos < lang_pos,
        "sdivi-parsing must be published before sdivi-lang-javascript"
    );
}

#[test]
fn sdivi_parsing_published_before_lang_go() {
    let yml = release_yml();
    let parsing_pos = find_offset(&yml, "cargo publish -p sdivi-parsing");
    let lang_pos = find_offset(&yml, "cargo publish -p sdivi-lang-go");
    assert!(
        parsing_pos < lang_pos,
        "sdivi-parsing must be published before sdivi-lang-go"
    );
}

#[test]
fn sdivi_parsing_published_before_lang_java() {
    let yml = release_yml();
    let parsing_pos = find_offset(&yml, "cargo publish -p sdivi-parsing");
    let lang_pos = find_offset(&yml, "cargo publish -p sdivi-lang-java");
    assert!(
        parsing_pos < lang_pos,
        "sdivi-parsing must be published before sdivi-lang-java"
    );
}

// ── Mid-chain ordering invariants ─────────────────────────────────────────

#[test]
fn lang_adapters_published_before_sdivi_graph() {
    let yml = release_yml();
    // Use the last lang adapter as the reference point — all six are in the
    // same `run: |` block, so the block's start is before sdivi-graph's step.
    let lang_block_pos = find_offset(&yml, "cargo publish -p sdivi-lang-rust");
    let graph_pos = find_offset(&yml, "cargo publish -p sdivi-graph");
    assert!(
        lang_block_pos < graph_pos,
        "sdivi-lang-* adapters must be published before sdivi-graph"
    );
}

#[test]
fn sdivi_graph_published_before_sdivi_detection() {
    let yml = release_yml();
    let graph_pos = find_offset(&yml, "cargo publish -p sdivi-graph");
    let detection_pos = find_offset(&yml, "cargo publish -p sdivi-detection");
    assert!(
        graph_pos < detection_pos,
        "sdivi-graph must be published before sdivi-detection"
    );
}

#[test]
fn sdivi_detection_published_before_sdivi_patterns() {
    let yml = release_yml();
    let detection_pos = find_offset(&yml, "cargo publish -p sdivi-detection");
    let patterns_pos = find_offset(&yml, "cargo publish -p sdivi-patterns");
    assert!(
        detection_pos < patterns_pos,
        "sdivi-detection must be published before sdivi-patterns"
    );
}

#[test]
fn sdivi_patterns_published_before_sdivi_snapshot() {
    let yml = release_yml();
    let patterns_pos = find_offset(&yml, "cargo publish -p sdivi-patterns");
    let snapshot_pos = find_offset(&yml, "cargo publish -p sdivi-snapshot");
    assert!(
        patterns_pos < snapshot_pos,
        "sdivi-patterns must be published before sdivi-snapshot"
    );
}

#[test]
fn sdivi_snapshot_published_before_sdivi_core() {
    let yml = release_yml();
    let snapshot_pos = find_offset(&yml, "cargo publish -p sdivi-snapshot");
    let core_pos = find_offset(&yml, "cargo publish -p sdivi-core");
    assert!(
        snapshot_pos < core_pos,
        "sdivi-snapshot must be published before sdivi-core"
    );
}

#[test]
fn sdivi_core_published_before_sdivi_pipeline() {
    let yml = release_yml();
    let core_pos = find_offset(&yml, "cargo publish -p sdivi-core");
    let pipeline_pos = find_offset(&yml, "cargo publish -p sdivi-pipeline");
    assert!(
        core_pos < pipeline_pos,
        "sdivi-core must be published before sdivi-pipeline"
    );
}

#[test]
fn sdivi_pipeline_published_before_sdivi_cli() {
    let yml = release_yml();
    let pipeline_pos = find_offset(&yml, "cargo publish -p sdivi-pipeline");
    let cli_pos = find_offset(&yml, "cargo publish -p sdivi-cli");
    assert!(
        pipeline_pos < cli_pos,
        "sdivi-pipeline must be published before sdivi-cli"
    );
}

#[test]
fn sdivi_cli_published_before_sdivi_rust() {
    let yml = release_yml();
    let cli_pos = find_offset(&yml, "cargo publish -p sdivi-cli");
    let rust_pos = find_offset(&yml, "cargo publish -p sdivi-rust");
    assert!(
        cli_pos < rust_pos,
        "sdivi-cli must be published before sdivi-rust (sdivi-rust depends on sdivi-cli)"
    );
}

// ── Sleep guards between batches ──────────────────────────────────────────
// Verify that 30-second propagation sleeps exist between key publish batches
// so crates.io index has time to register each batch before dependents publish.

#[test]
fn sleep_after_sdivi_config_before_sdivi_parsing() {
    let yml = release_yml();
    let config_pos = find_offset(&yml, "cargo publish -p sdivi-config");
    let parsing_pos = find_offset(&yml, "cargo publish -p sdivi-parsing");
    let between = &yml[config_pos..parsing_pos];
    assert!(
        between.contains("sleep 30"),
        "a 'sleep 30' must appear between publishing sdivi-config and sdivi-parsing"
    );
}

#[test]
fn sleep_after_sdivi_parsing_before_lang_adapters() {
    let yml = release_yml();
    let parsing_pos = find_offset(&yml, "cargo publish -p sdivi-parsing");
    // The first lang adapter in the run block
    let lang_pos = find_offset(&yml, "cargo publish -p sdivi-lang-rust");
    let between = &yml[parsing_pos..lang_pos];
    assert!(
        between.contains("sleep 30"),
        "a 'sleep 30' must appear between publishing sdivi-parsing and the sdivi-lang-* block"
    );
}
