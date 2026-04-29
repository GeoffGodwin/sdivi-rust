use std::io::Write;
use std::sync::Mutex;

use tempfile::NamedTempFile;

use sdi_config::{load_or_default, load_with_paths, Config};

/// Mutex to serialize tests that mutate env vars.
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn write_toml(content: &str) -> NamedTempFile {
    let mut f = NamedTempFile::new().expect("tempfile");
    f.write_all(content.as_bytes()).expect("write");
    f
}

// ── Defaults are returned when no config files exist ─────────────────────────

#[test]
fn no_config_files_returns_defaults() {
    let config = load_with_paths(None, None).unwrap();
    assert_eq!(config.core.random_seed, 42);
    assert_eq!(config.snapshots.retention, 100);
    assert!(config.thresholds.overrides.is_empty());
}

// ── Project config overrides defaults key-by-key ─────────────────────────────

#[test]
fn project_config_overrides_default_value() {
    let toml = r#"
[core]
random_seed = 99
"#;
    let file = write_toml(toml);
    let config = load_with_paths(Some(file.path()), None).unwrap();
    assert_eq!(config.core.random_seed, 99);
    // Other defaults are preserved.
    assert_eq!(config.core.languages, "auto");
    assert_eq!(config.snapshots.retention, 100);
}

#[test]
fn project_config_replaces_list_not_merges() {
    let toml = r#"
[core]
exclude = ["**/my_vendor/**"]
"#;
    let file = write_toml(toml);
    let config = load_with_paths(Some(file.path()), None).unwrap();
    assert_eq!(config.core.exclude, vec!["**/my_vendor/**"]);
}

// ── Global config is lower precedence than project config ────────────────────

#[test]
fn project_config_overrides_global_config() {
    let global_toml = r#"
[core]
random_seed = 10
"#;
    let project_toml = r#"
[core]
random_seed = 20
"#;
    let global = write_toml(global_toml);
    let project = write_toml(project_toml);
    let config = load_with_paths(Some(project.path()), Some(global.path())).unwrap();
    assert_eq!(config.core.random_seed, 20);
}

#[test]
fn global_config_wins_when_project_absent() {
    let global_toml = r#"
[snapshots]
retention = 42
"#;
    let global = write_toml(global_toml);
    let config = load_with_paths(None, Some(global.path())).unwrap();
    assert_eq!(config.snapshots.retention, 42);
}

// ── Env var SDI_SNAPSHOT_DIR overrides file config ───────────────────────────

#[test]
fn env_var_snapshot_dir_overrides_file_config() {
    let _lock = ENV_LOCK.lock().unwrap();
    std::env::set_var("SDI_SNAPSHOT_DIR", "/tmp/test_snapshots_sdi");
    let result = load_or_default(std::path::Path::new("."));
    std::env::remove_var("SDI_SNAPSHOT_DIR");
    let config = result.unwrap();
    assert_eq!(config.snapshots.dir, "/tmp/test_snapshots_sdi");
}

// ── Env var NO_COLOR sets color = never ──────────────────────────────────────

#[test]
fn env_var_no_color_sets_never() {
    use sdi_config::ColorChoice;
    let _lock = ENV_LOCK.lock().unwrap();
    std::env::set_var("NO_COLOR", "1");
    let result = load_or_default(std::path::Path::new("."));
    std::env::remove_var("NO_COLOR");
    let config = result.unwrap();
    assert_eq!(config.output.color, ColorChoice::Never);
}

// ── Unknown top-level section produces no error ───────────────────────────────

#[test]
fn unknown_section_does_not_error() {
    let toml = r#"
[unknown_section]
foo = "bar"

[core]
random_seed = 7
"#;
    let file = write_toml(toml);
    let config = load_with_paths(Some(file.path()), None)
        .expect("unknown section must not cause an error");
    // Known config still loaded correctly.
    assert_eq!(config.core.random_seed, 7);
}

// ── Defaults round-trip through the loader unchanged ─────────────────────────

#[test]
fn empty_project_config_is_equivalent_to_defaults() {
    let file = write_toml(""); // empty TOML
    let config = load_with_paths(Some(file.path()), None).unwrap();
    let defaults = Config::default();
    assert_eq!(config.core.random_seed, defaults.core.random_seed);
    assert_eq!(config.core.exclude, defaults.core.exclude);
    assert_eq!(config.snapshots.retention, defaults.snapshots.retention);
}

// ── Global config is loaded via $XDG_CONFIG_HOME in load_or_default ──────────
//
// Covers the reviewer-flagged gap: no integration test exercised the global
// config path resolved from the `$XDG_CONFIG_HOME/sdi/config.toml` env-var
// lookup inside `load_or_default`. Tests using `load_with_paths` bypass that
// resolution entirely and never exercise the `dirs::config_dir()` call.

#[test]
fn load_or_default_reads_global_config_via_xdg_config_home() {
    let _lock = ENV_LOCK.lock().unwrap();

    // Create a temp XDG_CONFIG_HOME with a recognisable sdi/config.toml value.
    let xdg_dir = tempfile::TempDir::new().expect("temp xdg dir");
    let sdi_dir = xdg_dir.path().join("sdi");
    std::fs::create_dir_all(&sdi_dir).expect("create sdi config dir");
    std::fs::write(
        sdi_dir.join("config.toml"),
        "[core]\nrandom_seed = 77\n",
    )
    .expect("write global config");

    // Repo root with no .sdi/config.toml so only the global level contributes.
    let repo_root = tempfile::TempDir::new().expect("temp repo root");

    // Snapshot and unset any pre-existing vars that load_or_default reads.
    let prev_xdg = std::env::var("XDG_CONFIG_HOME").ok();
    let prev_sdi_config = std::env::var("SDI_CONFIG_PATH").ok();
    let prev_snapshot_dir = std::env::var("SDI_SNAPSHOT_DIR").ok();
    let prev_no_color = std::env::var("NO_COLOR").ok();

    std::env::set_var("XDG_CONFIG_HOME", xdg_dir.path());
    std::env::remove_var("SDI_CONFIG_PATH");
    std::env::remove_var("SDI_SNAPSHOT_DIR");
    std::env::remove_var("NO_COLOR");

    let result = load_or_default(repo_root.path());

    // Restore env vars before asserting so cleanup always runs.
    match prev_xdg {
        Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
        None => std::env::remove_var("XDG_CONFIG_HOME"),
    }
    match prev_sdi_config {
        Some(v) => std::env::set_var("SDI_CONFIG_PATH", v),
        None => std::env::remove_var("SDI_CONFIG_PATH"),
    }
    match prev_snapshot_dir {
        Some(v) => std::env::set_var("SDI_SNAPSHOT_DIR", v),
        None => std::env::remove_var("SDI_SNAPSHOT_DIR"),
    }
    match prev_no_color {
        Some(v) => std::env::set_var("NO_COLOR", v),
        None => std::env::remove_var("NO_COLOR"),
    }

    let config = result.expect("load_or_default must succeed with valid global config");
    assert_eq!(
        config.core.random_seed, 77,
        "global config from $XDG_CONFIG_HOME must be applied by load_or_default"
    );
    // Other defaults must be preserved (the global config only sets random_seed).
    assert_eq!(config.snapshots.retention, 100);
}

#[test]
fn global_config_is_lower_precedence_than_project_config_via_load_or_default() {
    let _lock = ENV_LOCK.lock().unwrap();

    // Global config sets random_seed = 55; project config overrides to 88.
    let xdg_dir = tempfile::TempDir::new().expect("temp xdg dir");
    let sdi_dir = xdg_dir.path().join("sdi");
    std::fs::create_dir_all(&sdi_dir).expect("create sdi config dir");
    std::fs::write(sdi_dir.join("config.toml"), "[core]\nrandom_seed = 55\n")
        .expect("write global config");

    // Repo root with a .sdi/config.toml that overrides the seed.
    let repo_root = tempfile::TempDir::new().expect("temp repo root");
    let dot_sdi = repo_root.path().join(".sdi");
    std::fs::create_dir_all(&dot_sdi).expect("create .sdi dir");
    std::fs::write(dot_sdi.join("config.toml"), "[core]\nrandom_seed = 88\n")
        .expect("write project config");

    let prev_xdg = std::env::var("XDG_CONFIG_HOME").ok();
    let prev_sdi_config = std::env::var("SDI_CONFIG_PATH").ok();
    let prev_snapshot_dir = std::env::var("SDI_SNAPSHOT_DIR").ok();
    let prev_no_color = std::env::var("NO_COLOR").ok();

    std::env::set_var("XDG_CONFIG_HOME", xdg_dir.path());
    std::env::remove_var("SDI_CONFIG_PATH");
    std::env::remove_var("SDI_SNAPSHOT_DIR");
    std::env::remove_var("NO_COLOR");

    let result = load_or_default(repo_root.path());

    match prev_xdg {
        Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
        None => std::env::remove_var("XDG_CONFIG_HOME"),
    }
    match prev_sdi_config {
        Some(v) => std::env::set_var("SDI_CONFIG_PATH", v),
        None => std::env::remove_var("SDI_CONFIG_PATH"),
    }
    match prev_snapshot_dir {
        Some(v) => std::env::set_var("SDI_SNAPSHOT_DIR", v),
        None => std::env::remove_var("SDI_SNAPSHOT_DIR"),
    }
    match prev_no_color {
        Some(v) => std::env::set_var("NO_COLOR", v),
        None => std::env::remove_var("NO_COLOR"),
    }

    let config = result.expect("load_or_default must succeed");
    assert_eq!(
        config.core.random_seed, 88,
        "project config must win over global config; expected 88, got {}",
        config.core.random_seed
    );
}
