//! [`LanguageAdapter`] trait — the extension point for per-language AST parsing.

use std::path::Path;

use crate::feature_record::FeatureRecord;

/// Extension point for language-specific parsing implementations.
///
/// Implement this trait to add support for a new programming language.
/// Each adapter identifies files by extension and extracts a [`FeatureRecord`]
/// from the file's tree-sitter AST.
///
/// Implementations must be `Send + Sync` to support rayon parallelism. Because
/// `tree_sitter::Parser` is not `Send`, adapters must use `thread_local!`
/// storage for their parsers rather than storing them in the struct.
///
/// # Examples
///
/// ```rust
/// use sdivi_parsing::adapter::LanguageAdapter;
/// use sdivi_parsing::feature_record::FeatureRecord;
/// use std::path::Path;
///
/// struct NoopAdapter;
///
/// impl LanguageAdapter for NoopAdapter {
///     fn language_name(&self) -> &'static str { "noop" }
///     fn file_extensions(&self) -> &[&'static str] { &[".noop"] }
///     fn parse_file(&self, path: &Path, _content: String) -> FeatureRecord {
///         FeatureRecord {
///             path: path.to_path_buf(),
///             language: "noop".to_string(),
///             imports: vec![],
///             exports: vec![],
///             signatures: vec![],
///             pattern_hints: vec![],
///         }
///     }
/// }
///
/// let adapter = NoopAdapter;
/// assert_eq!(adapter.language_name(), "noop");
/// ```
pub trait LanguageAdapter: Send + Sync {
    /// Returns the canonical lower-case language name (e.g. `"rust"`).
    fn language_name(&self) -> &'static str;

    /// Returns the file extensions handled by this adapter (e.g. `&[".rs"]`).
    ///
    /// Extensions must include the leading dot.
    fn file_extensions(&self) -> &[&'static str];

    /// Parses `content` and returns a [`FeatureRecord`].
    ///
    /// `content` is consumed by value. The returned record must own all its
    /// data — no lifetime ties back to `content` or any internal tree-sitter
    /// structure are permitted. The CST must be dropped before this method
    /// returns.
    fn parse_file(&self, path: &Path, content: String) -> FeatureRecord;
}
