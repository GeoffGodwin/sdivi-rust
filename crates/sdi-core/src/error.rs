use thiserror::Error;

/// Errors produced by the sdi-core pure-compute API.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AnalysisError {
    /// I/O error (only reachable when the caller bridges to sdi-pipeline).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// No tree-sitter grammar was found for any language detected in the
    /// repository. Analysis cannot proceed.
    #[error("no grammar available for any detected language in the repository")]
    NoGrammarsAvailable,

    /// Configuration error surfaced from within the pipeline.
    #[error("configuration error: {0}")]
    Config(#[from] sdi_config::ConfigError),

    /// I/O error while writing a snapshot file or persisting the partition cache.
    #[error("snapshot write error: {0}")]
    SnapshotIo(std::io::Error),

    /// A node ID failed the canonicalization rules.
    ///
    /// See [`crate::input::validate_node_id`] for the full set of rules.
    #[error("invalid node id {id:?}: {reason}")]
    InvalidNodeId {
        /// The offending node ID string.
        id: String,
        /// Human-readable reason.
        reason: String,
    },

    /// A configuration value is invalid.
    #[error("invalid configuration: {message}")]
    InvalidConfig {
        /// Human-readable description of the problem.
        message: String,
    },
}
