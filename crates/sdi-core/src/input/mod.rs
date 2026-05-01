//! Input structs for the pure-compute API.
//!
//! These are the types that WASM consumers and other embedders supply to the
//! `compute_*` functions.  All are plain `serde` structs with no I/O, no
//! tree-sitter, and no `std::time`.

mod change_coupling_types;
mod types;

pub use change_coupling_types::{ChangeCouplingConfigInput, CoChangeEventInput};
pub use types::{
    BoundaryDefInput, BoundarySpecInput, DependencyGraphInput, EdgeInput, LeidenConfigInput,
    NodeInput, NormalizeNode, PatternInstanceInput, PatternLocationInput, PriorPartition,
    QualityFunctionInput, ThresholdOverrideInput, ThresholdsInput,
};

use crate::error::AnalysisError;

/// Validates a node ID according to canonical rules.
///
/// A valid node ID is:
/// - non-empty
/// - uses forward slashes only (no backslashes)
/// - no leading `./`
/// - no trailing `/`
/// - no absolute path component (no leading `/`)
/// - no `..` components
///
/// # Errors
///
/// Returns [`AnalysisError::InvalidNodeId`] with the offending string on failure.
///
/// # Examples
///
/// ```rust
/// use sdi_core::input::validate_node_id;
///
/// assert!(validate_node_id("src/lib.rs").is_ok());
/// assert!(validate_node_id("Cargo.toml").is_ok());
/// assert!(validate_node_id("./foo").is_err());
/// assert!(validate_node_id("foo/").is_err());
/// assert!(validate_node_id("").is_err());
/// assert!(validate_node_id("../foo").is_err());
/// assert!(validate_node_id("/foo").is_err());
/// ```
pub fn validate_node_id(s: &str) -> Result<(), AnalysisError> {
    if s.is_empty() {
        return Err(AnalysisError::InvalidNodeId {
            id: s.to_string(),
            reason: "must not be empty".to_string(),
        });
    }
    if s.contains('\\') {
        return Err(AnalysisError::InvalidNodeId {
            id: s.to_string(),
            reason: "must use forward slashes only".to_string(),
        });
    }
    if s.starts_with("./") {
        return Err(AnalysisError::InvalidNodeId {
            id: s.to_string(),
            reason: "must not start with './'".to_string(),
        });
    }
    if s.ends_with('/') {
        return Err(AnalysisError::InvalidNodeId {
            id: s.to_string(),
            reason: "must not end with '/'".to_string(),
        });
    }
    if s.starts_with('/') {
        return Err(AnalysisError::InvalidNodeId {
            id: s.to_string(),
            reason: "must not be an absolute path".to_string(),
        });
    }
    for component in s.split('/') {
        if component == ".." {
            return Err(AnalysisError::InvalidNodeId {
                id: s.to_string(),
                reason: "must not contain '..' components".to_string(),
            });
        }
    }
    Ok(())
}
