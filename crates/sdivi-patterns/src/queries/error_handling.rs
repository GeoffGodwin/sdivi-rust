//! Node kinds classified as error-handling patterns.
//!
//! These node kinds correspond to the `error_handling` category in the
//! [`PatternCatalog`](crate::catalog::PatternCatalog).

/// Tree-sitter node kinds for error-handling patterns.
///
/// - `try_statement`: Python/TypeScript/JavaScript/Java `try` block umbrella
/// - `try_expression`: the `?` operator applied to a `Result` or `Option` (Rust)
/// - `match_expression`: `match` arms used for result/error dispatch (Rust)
/// - `except_clause`: individual Python `except` arms (`except ValueError:`, `except (A, B) as e:`)
/// - `catch_clause`: individual Java `catch` arms (`catch (IOException e) { ... }`)
/// - `throw_statement`: Java throw-site (`throw new RuntimeException(msg)`)
pub const NODE_KINDS: &[&str] = &[
    "try_statement",
    "try_expression",
    "match_expression",
    "except_clause",
    "catch_clause",
    "throw_statement",
];
