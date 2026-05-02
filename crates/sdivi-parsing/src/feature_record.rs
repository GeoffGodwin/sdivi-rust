//! [`FeatureRecord`] and [`PatternHint`] — the output types of the parsing stage.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// An AST node captured during parsing for use by the patterns stage.
///
/// Captures enough context to classify patterns without reparsing the file.
///
/// # Examples
///
/// ```rust
/// use sdivi_parsing::feature_record::PatternHint;
///
/// let hint = PatternHint {
///     node_kind: "try_expression".to_string(),
///     start_byte: 42,
///     end_byte: 55,
///     start_row: 10,
///     start_col: 4,
///     text: "some_fn()?".to_string(),
/// };
/// assert_eq!(hint.node_kind, "try_expression");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PatternHint {
    /// The tree-sitter node kind (e.g. `"try_expression"`, `"match_expression"`).
    pub node_kind: String,
    /// Byte offset of the node start in the source file.
    pub start_byte: usize,
    /// Byte offset of the node end in the source file.
    pub end_byte: usize,
    /// Zero-indexed source row (line) of the node start.
    pub start_row: usize,
    /// Zero-indexed source column of the node start.
    pub start_col: usize,
    /// Source text of the node, truncated to 256 bytes if longer.
    pub text: String,
}

/// The parsed output for a single source file.
///
/// Produced by [`crate::adapter::LanguageAdapter::parse_file`] and consumed by
/// the graph (imports), patterns (pattern_hints), and snapshot stages.
///
/// All fields own their data — no reference into the original file content or
/// the tree-sitter CST survives in this struct.
///
/// # Examples
///
/// ```rust
/// use sdivi_parsing::feature_record::FeatureRecord;
/// use std::path::PathBuf;
///
/// let record = FeatureRecord {
///     path: PathBuf::from("src/lib.rs"),
///     language: "rust".to_string(),
///     imports: vec!["std::collections::BTreeMap".to_string()],
///     exports: vec!["Foo".to_string()],
///     signatures: vec!["pub fn new() -> Foo".to_string()],
///     pattern_hints: vec![],
/// };
/// assert_eq!(record.language, "rust");
/// assert_eq!(record.imports.len(), 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureRecord {
    /// Path to the source file, relative to the repository root.
    pub path: PathBuf,
    /// Language name (e.g. `"rust"`, `"python"`).
    pub language: String,
    /// Fully-qualified import paths found in the file (graph stage input).
    pub imports: Vec<String>,
    /// Names of publicly exported items from this file.
    pub exports: Vec<String>,
    /// Function, method, and type signatures as source text.
    pub signatures: Vec<String>,
    /// AST node hints for the patterns stage; no reparsing required.
    pub pattern_hints: Vec<PatternHint>,
}
