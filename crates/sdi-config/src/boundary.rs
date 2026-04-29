use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::ConfigError;

/// A ratified boundary specification loaded from `.sdi/boundaries.yaml`.
///
/// A missing file is normal operation (Rule 16): call [`BoundarySpec::load`] and
/// treat `Ok(None)` as "no declared intent." Intent divergence fields are absent
/// from the snapshot when no spec is present.
///
/// # Examples
///
/// ```rust
/// use std::path::Path;
/// use sdi_config::BoundarySpec;
///
/// // Missing file is OK — returns None, not an error.
/// let spec = BoundarySpec::load(Path::new("/nonexistent/boundaries.yaml")).unwrap();
/// assert!(spec.is_none());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundarySpec {
    /// Schema version string (optional; presence enables forward-compat checks).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Declared boundary definitions.
    #[serde(default)]
    pub boundaries: Vec<BoundaryDef>,
}

/// A single named boundary within a [`BoundarySpec`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryDef {
    /// Unique name for this boundary (e.g., `"api"`, `"models"`).
    pub name: String,
    /// Human-readable description of the boundary's role.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Glob patterns matching source files that belong to this boundary.
    #[serde(default)]
    pub modules: Vec<String>,
    /// Boundaries from which this boundary is allowed to import.
    #[serde(default)]
    pub allow_imports_from: Vec<String>,
}

impl BoundarySpec {
    /// Load a boundary specification from a YAML file.
    ///
    /// Returns `Ok(None)` if the file does not exist (missing is normal, not an error).
    /// Returns `Err(ConfigError::BoundaryParse)` if the file exists but is malformed.
    /// Returns `Err(ConfigError::Io)` on read failures.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] on I/O or parse failure.
    pub fn load(path: &Path) -> Result<Option<Self>, ConfigError> {
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(path).map_err(ConfigError::Io)?;
        let spec: Self = serde_yaml::from_str(&content)
            .map_err(|e| ConfigError::BoundaryParse(e.to_string()))?;
        Ok(Some(spec))
    }
}
