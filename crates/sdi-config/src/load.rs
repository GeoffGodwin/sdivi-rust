use std::path::Path;

use crate::{Config, ConfigError};

/// Load configuration for the repository rooted at `repo_root`.
///
/// Walks the 5-level precedence chain (CLI args > env vars > project config >
/// global config > built-in defaults). Returns `Config::default()` until the
/// full precedence resolver is implemented in Milestone 2.
///
/// # Errors
///
/// Returns [`ConfigError`] if any config file is malformed or contains
/// invalid values.
pub fn load_or_default(_repo_root: &Path) -> Result<Config, ConfigError> {
    Ok(Config::default())
}
