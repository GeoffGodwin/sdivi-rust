use serde::{Deserialize, Serialize};

/// A ratified boundary specification loaded from `.sdivi/boundaries.yaml`.
///
/// A missing file is normal operation (Rule 16): call [`BoundarySpec::load`] and
/// treat `Ok(None)` as "no declared intent." Intent divergence fields are absent
/// from the snapshot when no spec is present.
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
    /// Serialises this spec to a YAML string.
    ///
    /// Does not touch the filesystem — I/O is the caller's responsibility.
    /// Comments in any prior user-written file are lost on the next write
    /// (see `docs/migrating-from-sdi-py.md`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sdivi_config::BoundarySpec;
    ///
    /// let spec = BoundarySpec { version: None, boundaries: vec![] };
    /// let yaml = spec.to_yaml();
    /// assert!(yaml.contains("boundaries"));
    /// ```
    pub fn to_yaml(&self) -> String {
        serde_yml::to_string(self).unwrap_or_default()
    }

    /// Load a boundary specification from a YAML file.
    ///
    /// Returns `Ok(None)` if the file does not exist (missing is normal, not an error).
    /// Returns `Err(ConfigError::BoundaryParse)` if the file exists but is malformed.
    /// Returns `Err(ConfigError::Io)` on read failures.
    ///
    /// Only available with the `loader` feature (default ON).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::Path;
    /// use sdivi_config::BoundarySpec;
    ///
    /// let spec = BoundarySpec::load(Path::new("/nonexistent/boundaries.yaml")).unwrap();
    /// assert!(spec.is_none());
    /// ```
    #[cfg(feature = "loader")]
    pub fn load(path: &std::path::Path) -> Result<Option<Self>, crate::ConfigError> {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(crate::ConfigError::Io(e)),
        };
        let spec: Self = serde_yml::from_str(&content)
            .map_err(|e| crate::ConfigError::BoundaryParse(e.to_string()))?;
        Ok(Some(spec))
    }
}
