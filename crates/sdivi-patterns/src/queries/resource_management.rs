//! Node kinds classified as resource-management patterns.
//!
//! These node kinds correspond to the `resource_management` category in the
//! [`PatternCatalog`](crate::catalog::PatternCatalog).

use std::sync::LazyLock;

use regex::Regex;

/// Tree-sitter node kinds for resource-management patterns.
///
/// - `macro_invocation`: Rust macro calls (e.g. `drop!`, `vec!`, `println!`)
/// - `with_statement`: Python context managers (`with open(p) as f:`, `with lock:`)
/// - `defer_statement`: Go deferred cleanup (`defer f.Close()`, `defer mu.Unlock()`)
/// - `try_with_resources_statement`: Java try-with-resources (`try (var r = open()) { ... }`)
///
/// These four forms are the canonical scoped-resource-release idioms in their
/// respective languages and are structurally distinct even though they serve the
/// same semantic purpose (acquire → use → release on scope exit).
pub const NODE_KINDS: &[&str] = &[
    "macro_invocation",
    "with_statement",
    "defer_statement",
    "try_with_resources_statement",
];

// Rust: logging macros that should fall through to the `logging` category.
// Must match logging::RUST_RE exactly — update both together.
// Same pattern as logging::RUST_RE — if a macro_invocation text matches,
// it is logging, not resource_management.
//   ^(tracing|log)::  — tracing::info!, log::debug!, etc.
//   ^(println|eprintln|print|eprint|dbg)!  — standard output macros
static RUST_LOGGING_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(tracing|log)::|^(println|eprintln|print|eprint|dbg)!")
        .expect("resource_management Rust logging exclusion regex is valid")
});

/// Return `true` when `text` indicates the `macro_invocation` is **not**
/// resource-management and should fall through to another category.
///
/// The semantic is **inverted** compared to the `matches_callee` functions:
/// `true` means "this macro is excluded from resource_management."
/// Currently the only exclusion is Rust logging macros (`tracing::*!`,
/// `log::*!`, `println!`, `eprintln!`, `print!`, `eprint!`, `dbg!`), which
/// belong in the `logging` category instead.
///
/// Other languages always return `false` — no macro disambiguation needed.
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::queries::resource_management::excludes_callee;
///
/// // Logging macros are excluded from resource_management in Rust.
/// assert!(excludes_callee("tracing::info!(\"hi\")", "rust"));
/// assert!(excludes_callee("println!(\"x\")", "rust"));
///
/// // Non-logging macros stay in resource_management.
/// assert!(!excludes_callee("vec![1, 2, 3]", "rust"));
/// assert!(!excludes_callee("drop!(handle)", "rust"));
///
/// // Other languages never exclude.
/// assert!(!excludes_callee("tracing::info!(\"x\")", "typescript"));
/// ```
pub fn excludes_callee(text: &str, language: &str) -> bool {
    match language {
        "rust" => RUST_LOGGING_RE.is_match(text),
        _ => false,
    }
}
