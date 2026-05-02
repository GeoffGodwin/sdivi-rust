use std::path::{Path, PathBuf};

use crate::{thresholds, Config, ConfigError};

const KNOWN_SECTIONS: &[&str] = &[
    "core",
    "snapshots",
    "boundaries",
    "patterns",
    "thresholds",
    "change_coupling",
    "output",
    "determinism",
    "bindings",
];

/// Resolve the project config path for `repo_root`.
///
/// Returns `SDIVI_CONFIG_PATH` if set; otherwise `<repo_root>/.sdivi/config.toml`.
///
/// # Examples
///
/// ```rust
/// use sdivi_config::project_config_path;
/// let p = project_config_path(std::path::Path::new("/my/repo"));
/// assert!(p.ends_with("config.toml"));
/// ```
pub fn project_config_path(repo_root: &Path) -> PathBuf {
    std::env::var("SDIVI_CONFIG_PATH")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.join(".sdivi").join("config.toml"))
}

/// Load configuration for the repository rooted at `repo_root`.
///
/// Walks the 5-level precedence chain:
/// 1. Built-in defaults
/// 2. Global `$XDG_CONFIG_HOME/sdivi/config.toml`
/// 3. Project-local `.sdivi/config.toml` (or `SDIVI_CONFIG_PATH` if set)
/// 4. Environment variables (`SDIVI_SNAPSHOT_DIR`, `NO_COLOR`)
/// 5. CLI flags (applied by the caller on top of the returned [`Config`])
///
/// # Errors
///
/// Returns [`ConfigError`] if any config file is malformed or contains invalid values.
///
/// # Examples
///
/// ```rust
/// use sdivi_config::load_or_default;
///
/// let config = load_or_default(std::path::Path::new(".")).unwrap();
/// assert_eq!(config.core.random_seed, 42);
/// ```
pub fn load_or_default(repo_root: &Path) -> Result<Config, ConfigError> {
    let project_config = project_config_path(repo_root);
    let global_config = dirs::config_dir().map(|d| d.join("sdivi").join("config.toml"));
    let mut config = load_with_paths(Some(&project_config), global_config.as_deref())?;
    apply_env_overrides(&mut config);
    Ok(config)
}

/// Load configuration from explicit file paths, without reading env vars.
///
/// Both paths are optional — if `None` or the file does not exist, that level is skipped.
/// Env variable overrides are **not** applied; use [`load_or_default`] for the full chain.
///
/// # Errors
///
/// Returns [`ConfigError`] on parse or validation failure.
pub fn load_with_paths(
    project_config: Option<&Path>,
    global_config: Option<&Path>,
) -> Result<Config, ConfigError> {
    let default_toml =
        toml::to_string(&Config::default()).expect("Config::default() must serialize to TOML");
    let mut base: toml::Table =
        toml::from_str(&default_toml).expect("serialized Config must round-trip through TOML");

    if let Some(path) = global_config {
        if let Some(overlay) = load_toml_file(path)? {
            warn_unknown_keys(&overlay, "global config");
            merge_into(&mut base, overlay);
        }
    }

    if let Some(path) = project_config {
        if let Some(overlay) = load_toml_file(path)? {
            warn_unknown_keys(&overlay, "project config");
            merge_into(&mut base, overlay);
        }
    }

    // Validate expires format and prune overrides that have already expired.
    // NOTE for test authors: `today_iso8601()` is called unconditionally here,
    // so integration tests cannot inject a specific "today" through the public
    // `load_with_paths` API.  To test expiry behaviour with a controlled date,
    // call `thresholds::validate_and_prune_overrides` directly with an explicit
    // date string instead of going through `load_with_paths`.
    thresholds::validate_and_prune_overrides(&mut base, &thresholds::today_iso8601())?;

    let merged = toml::to_string(&toml::Value::Table(base))
        .map_err(|e| ConfigError::Parse(e.to_string()))?;
    toml::from_str(&merged).map_err(|e| ConfigError::Parse(e.to_string()))
}

fn load_toml_file(path: &Path) -> Result<Option<toml::Table>, ConfigError> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(ConfigError::Io(e)),
    };
    let table: toml::Table =
        toml::from_str(&content).map_err(|e| ConfigError::Parse(e.to_string()))?;
    Ok(Some(table))
}

fn warn_unknown_keys(table: &toml::Table, source: &str) {
    for key in table.keys() {
        if !KNOWN_SECTIONS.contains(&key.as_str()) {
            eprintln!("sdivi: warning: unknown config section '[{key:?}]' in {source} (ignored)");
        }
    }
}

fn merge_into(base: &mut toml::Table, overlay: toml::Table) {
    for (section, section_val) in overlay {
        match base.get_mut(&section) {
            Some(base_val) => {
                if let (toml::Value::Table(base_t), toml::Value::Table(overlay_t)) =
                    (base_val, section_val)
                {
                    merge_section(base_t, overlay_t, &section);
                }
            }
            None => {
                base.insert(section, section_val);
            }
        }
    }
}

fn merge_section(base_section: &mut toml::Table, overlay_section: toml::Table, section: &str) {
    for (key, val) in overlay_section {
        if section == "thresholds" && key == "overrides" {
            merge_overrides(base_section, val);
        } else {
            base_section.insert(key, val);
        }
    }
}

fn merge_overrides(base_section: &mut toml::Table, overlay_val: toml::Value) {
    let toml::Value::Table(overlay_ov) = overlay_val else {
        base_section.insert("overrides".to_string(), overlay_val);
        return;
    };
    match base_section.get_mut("overrides") {
        Some(toml::Value::Table(base_ov)) => {
            for (cat, cat_val) in overlay_ov {
                base_ov.insert(cat, cat_val);
            }
        }
        _ => {
            base_section.insert("overrides".to_string(), toml::Value::Table(overlay_ov));
        }
    }
}

fn apply_env_overrides(config: &mut Config) {
    if let Ok(dir) = std::env::var("SDIVI_SNAPSHOT_DIR") {
        config.snapshots.dir = dir;
    }
    if std::env::var("NO_COLOR").is_ok() {
        config.output.color = crate::ColorChoice::Never;
    }
}
