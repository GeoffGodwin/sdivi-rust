use thiserror::Error;

use crate::commit_extract::CommitExtractError;

/// Errors produced by the sdivi-pipeline orchestration layer.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum PipelineError {
    /// I/O error during file reading, snapshot writing, or cache access.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// No tree-sitter grammar was found for any language detected in the
    /// repository. Analysis cannot proceed.
    #[error("no grammar available for any detected language in the repository")]
    NoGrammarsAvailable,

    /// Configuration error surfaced from within the pipeline.
    #[error("configuration error: {0}")]
    Config(#[from] sdivi_config::ConfigError),

    /// I/O error while writing a snapshot file or persisting the partition cache.
    #[error("snapshot write error: {0}")]
    SnapshotIo(std::io::Error),

    /// Failure during historical-commit ref resolution or tree extraction.
    #[error("commit extract error: {0}")]
    CommitExtract(#[from] CommitExtractError),
}
