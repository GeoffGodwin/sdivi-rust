//! [`LanguageAdapter`] trait — the extension point for per-language AST parsing.

/// Extension point for language-specific parsing implementations.
///
/// Implement this trait to add support for a new programming language.
/// Each adapter identifies files by extension and extracts feature records
/// from their tree-sitter ASTs.
///
/// All implementations must be `Send + Sync` to support rayon parallelism.
///
/// # Examples
///
/// ```rust
/// use sdi_parsing::adapter::LanguageAdapter;
///
/// struct RustAdapter;
///
/// impl LanguageAdapter for RustAdapter {
///     fn language_name(&self) -> &'static str { "rust" }
///     fn file_extensions(&self) -> &[&'static str] { &[".rs"] }
/// }
///
/// let adapter = RustAdapter;
/// assert_eq!(adapter.language_name(), "rust");
/// ```
pub trait LanguageAdapter: Send + Sync {
    /// Returns the canonical lower-case name of the language (e.g. `"rust"`).
    fn language_name(&self) -> &'static str;

    /// Returns file extensions this adapter handles (e.g. `&[".rs"]`).
    ///
    /// Extensions must include the leading dot.
    fn file_extensions(&self) -> &[&'static str];
}
