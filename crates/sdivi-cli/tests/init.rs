use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Canonical content of the default config written by `sdivi init`.
/// Must match `commands::init::DEFAULT_CONFIG_TOML`.
const EXPECTED_DEFAULT_CONFIG: &str = r#"[core]
languages = "auto"
exclude = [
  "**/vendor/**",
  "**/node_modules/**",
  "**/__pycache__/**",
  "**/dist/**",
  "**/build/**",
  "**/target/**",
  "**/.git/**",
]
random_seed = 42

[snapshots]
dir = ".sdivi/snapshots"
retention = 100

[boundaries]
spec_file = ".sdivi/boundaries.yaml"
leiden_gamma = 1.0
stability_threshold = 3
weighted_edges = false

[patterns]
categories = "auto"
min_pattern_nodes = 5
scope_exclude = []

[thresholds]
pattern_entropy_rate = 2.0
convention_drift_rate = 3.0
coupling_delta_rate = 0.15
boundary_violation_rate = 2.0

[change_coupling]
min_frequency = 0.6
history_depth = 500

[output]
format = "text"
color = "auto"

[determinism]
enforce_btree_order = true

[bindings]
"#;

fn sdivi() -> Command {
    Command::cargo_bin("sdivi").expect("sdivi binary must be built")
}

/// `sdivi init` in an empty directory creates `.sdivi/config.toml`.
#[test]
fn init_creates_config_toml() {
    let dir = TempDir::new().unwrap();
    sdivi()
        .arg("--repo")
        .arg(dir.path())
        .arg("init")
        .assert()
        .success();

    let config_path = dir.path().join(".sdivi").join("config.toml");
    assert!(config_path.exists(), ".sdivi/config.toml must be created");
}

/// The written config.toml parses cleanly to `Config::default()` values.
#[test]
fn init_config_toml_parses_to_defaults() {
    let dir = TempDir::new().unwrap();
    sdivi()
        .arg("--repo")
        .arg(dir.path())
        .arg("init")
        .assert()
        .success();

    let config_path = dir.path().join(".sdivi").join("config.toml");
    let config = sdivi_config::load_with_paths(Some(&config_path), None)
        .expect("written config.toml must parse cleanly");

    assert_eq!(config.core.random_seed, 42);
    assert_eq!(config.core.languages, "auto");
    assert_eq!(config.snapshots.retention, 100);
    assert_eq!(config.snapshots.dir, ".sdivi/snapshots");
    assert!((config.boundaries.leiden_gamma - 1.0).abs() < f64::EPSILON);
    assert_eq!(config.boundaries.stability_threshold, 3);
    assert!(!config.boundaries.weighted_edges);
    assert_eq!(config.patterns.min_pattern_nodes, 5);
    assert!(config.thresholds.overrides.is_empty());
    assert!((config.thresholds.pattern_entropy_rate - 2.0).abs() < f64::EPSILON);
}

/// The written config.toml content matches the canonical default string.
#[test]
fn init_config_content_matches_canonical() {
    let dir = TempDir::new().unwrap();
    sdivi()
        .arg("--repo")
        .arg(dir.path())
        .arg("init")
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join(".sdivi").join("config.toml"))
        .expect("config.toml must exist after init");
    assert_eq!(
        written, EXPECTED_DEFAULT_CONFIG,
        "written config.toml must match canonical default"
    );
}

/// Running `sdivi init` twice does not clobber the existing config.
#[test]
fn init_is_idempotent() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join(".sdivi").join("config.toml");

    // First run creates the file.
    sdivi()
        .arg("--repo")
        .arg(dir.path())
        .arg("init")
        .assert()
        .success();

    // Overwrite with custom content.
    std::fs::write(&config_path, "[core]\nrandom_seed = 999\n").unwrap();

    // Second run must not overwrite.
    sdivi()
        .arg("--repo")
        .arg(dir.path())
        .arg("init")
        .assert()
        .success()
        .stderr(predicate::str::contains("already exists"));

    let content = std::fs::read_to_string(&config_path).unwrap();
    assert!(
        content.contains("999"),
        "existing config.toml must not be overwritten"
    );
}

/// `sdivi init` exits 0 and prints a success message referencing config.toml.
/// Per CLAUDE.md Rule 8, progress/status lines go to stderr — `sdivi init` produces
/// no stdout payload, so this test pins the success message on stderr.
#[test]
fn init_exits_zero_and_prints_success() {
    let dir = TempDir::new().unwrap();
    sdivi()
        .arg("--repo")
        .arg(dir.path())
        .arg("init")
        .assert()
        .success()
        .stderr(predicate::str::contains("config.toml"));
}

/// `SDIVI_CONFIG_PATH=/tmp/x.toml sdivi init` writes to that path, not `.sdivi/config.toml`.
#[test]
fn init_respects_sdivi_config_path_env_var() {
    let dir = TempDir::new().unwrap();
    let custom_config = dir.path().join("custom_config.toml");

    sdivi()
        .arg("--repo")
        .arg(dir.path())
        .arg("init")
        .env("SDIVI_CONFIG_PATH", &custom_config)
        .assert()
        .success();

    assert!(
        custom_config.exists(),
        "config must be written to SDIVI_CONFIG_PATH"
    );
    assert!(
        !dir.path().join(".sdivi").join("config.toml").exists(),
        ".sdivi/config.toml must NOT be created when SDIVI_CONFIG_PATH is set"
    );
}

/// When an existing config at `SDIVI_CONFIG_PATH` has a missing `expires`, init exits 2.
#[test]
fn init_exits_2_for_missing_expires_in_existing_config() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("bad_config.toml");

    std::fs::write(
        &config_path,
        "[thresholds.overrides.error_handling]\npattern_entropy_rate = 5.0\n",
    )
    .unwrap();

    sdivi()
        .arg("--repo")
        .arg(dir.path())
        .arg("init")
        .env("SDIVI_CONFIG_PATH", &config_path)
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("error_handling"));
}

/// Unknown config sections produce a deprecation warning on stderr but `init` succeeds.
#[test]
fn init_unknown_section_warns_on_stderr() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("unknown_section.toml");

    std::fs::write(
        &config_path,
        "[unknown_section]\nfoo = \"bar\"\n\n[core]\nrandom_seed = 7\n",
    )
    .unwrap();

    sdivi()
        .arg("--repo")
        .arg(dir.path())
        .arg("init")
        .env("SDIVI_CONFIG_PATH", &config_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("unknown").or(predicate::str::contains("warning")));
}
