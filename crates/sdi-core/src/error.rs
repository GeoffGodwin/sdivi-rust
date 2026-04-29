use thiserror::Error;

/// Errors produced by the sdi-core analysis pipeline.
///
/// This enum is `#[non_exhaustive]` — new variants will be added as the
/// pipeline grows in later milestones.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AnalysisError {
    /// I/O error during file reading, snapshot writing, or cache access.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// No tree-sitter grammar was found for any language detected in the
    /// repository. Analysis cannot proceed.
    #[error("no grammar available for any detected language in the repository")]
    NoGrammarsAvailable,

    /// Configuration error surfaced from within the pipeline.
    #[error("configuration error: {0}")]
    Config(#[from] sdi_config::ConfigError),
}
