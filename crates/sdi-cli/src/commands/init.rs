use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use walkdir::WalkDir;

/// Default `.sdi/config.toml` content written by `sdi init`.
///
/// This is the canonical form of the built-in defaults with helpful comments.
/// Loading this file with `Config::load_or_default` must produce `Config::default()`.
pub const DEFAULT_CONFIG_TOML: &str = r#"[core]
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
dir = ".sdi/snapshots"
retention = 100

[boundaries]
spec_file = ".sdi/boundaries.yaml"
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

/// Determine the config path for `sdi init`.
///
/// If `SDI_CONFIG_PATH` is set, that path is used. Otherwise, defaults to
/// `repo_root/.sdi/config.toml`.
pub fn config_path_for(repo_root: &Path) -> PathBuf {
    std::env::var("SDI_CONFIG_PATH")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.join(".sdi").join("config.toml"))
}

/// Run the `sdi init` command in `repo_root`.
///
/// Writes `DEFAULT_CONFIG_TOML` to the config path (determined by `SDI_CONFIG_PATH`
/// or `.sdi/config.toml`) if the file does not already exist.
///
/// If the config file already exists, validates it by loading it — this surfaces
/// unknown-key warnings and returns `ConfigError` for validation failures
/// (e.g. missing `expires` on a threshold override).
///
/// Running `sdi init` twice is idempotent: the second run validates but does not overwrite.
pub fn run(repo_root: &Path) -> Result<()> {
    let config_path = config_path_for(repo_root);

    if config_path.exists() {
        eprintln!("sdi: .sdi/config.toml already exists — skipping");
        // Validate the existing config (surfaces unknown-key warnings and config errors).
        sdi_config::load_with_paths(Some(&config_path), None)
            .with_context(|| format!("config validation failed: {}", config_path.display()))?;
    } else {
        let sdi_dir = config_path
            .parent()
            .expect("config path must have a parent directory");
        std::fs::create_dir_all(sdi_dir)
            .with_context(|| format!("could not create directory: {}", sdi_dir.display()))?;
        std::fs::write(&config_path, DEFAULT_CONFIG_TOML)
            .with_context(|| format!("could not write: {}", config_path.display()))?;
        eprintln!("sdi: created .sdi/config.toml");
    }

    let langs = detect_languages(repo_root);
    if langs.is_empty() {
        eprintln!("sdi: no source files detected (language detection skipped)");
    } else {
        eprintln!("sdi: detected languages: {}", langs.join(", "));
    }

    Ok(())
}

/// Walk `repo_root` (respecting standard excludes) and return sorted language names
/// inferred from file extensions.
fn detect_languages(repo_root: &Path) -> Vec<&'static str> {
    let skip_dirs: &[&str] = &[
        "vendor", "node_modules", "__pycache__", "dist", "build", "target", ".git",
    ];

    let mut found = BTreeSet::new();

    for entry in WalkDir::new(repo_root)
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !skip_dirs.contains(&name.as_ref())
            } else {
                true
            }
        })
        .flatten()
    {
        if entry.file_type().is_file() {
            let path = entry.path();
            if let Some(lang) = lang_from_path(path) {
                found.insert(lang);
            }
        }
    }

    found.into_iter().collect()
}

fn lang_from_path(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?;
    match ext {
        "rs" => Some("rust"),
        "py" => Some("python"),
        "ts" | "tsx" => Some("typescript"),
        "js" | "jsx" | "mjs" | "cjs" => Some("javascript"),
        "go" => Some("go"),
        "java" => Some("java"),
        _ => None,
    }
}
