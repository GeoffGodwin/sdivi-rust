use thiserror::Error;

/// Errors produced during configuration loading and validation.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// TOML parse failure.
    #[error("failed to parse config: {0}")]
    Parse(String),

    /// A config value is out of range or semantically invalid.
    #[error("invalid value for '{key}': {message}")]
    InvalidValue { key: String, message: String },

    /// A threshold override block is missing the mandatory `expires` field.
    #[error("threshold override for '{category}' is missing the required 'expires' field")]
    MissingExpiresOnOverride { category: String },

    /// I/O error reading a config file.
    #[error("I/O error reading config: {0}")]
    Io(#[from] std::io::Error),

    /// YAML parse failure reading `.sdivi/boundaries.yaml`.
    #[error("failed to parse boundary spec: {0}")]
    BoundaryParse(String),
}
