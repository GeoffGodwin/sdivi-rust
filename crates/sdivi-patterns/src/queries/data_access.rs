//! Node kinds classified as data-access patterns.
//!
//! These node kinds correspond to the `data_access` category in the
//! [`PatternCatalog`](crate::catalog::PatternCatalog).

use std::sync::LazyLock;

use regex::Regex;

/// Tree-sitter node kinds for data-access patterns.
///
/// - `call_expression`: function/method calls that access data stores or
///   external resources (TypeScript/JavaScript/Go: `fetch`, `query`, `read`,
///   `write`, `db.*`, `sql.*`, etc.). All `call_expression` nodes are
///   classified here; callee-name narrowing is the consumer's responsibility.
/// - `call`: Python function calls accessing data (`cursor.*`, `session.*`,
///   `open`). Same broad-classification rule as above.
pub const NODE_KINDS: &[&str] = &["call_expression", "call"];

// TypeScript / JavaScript / Go:
//   ^(fetch|axios)\b  — top-level fetch/axios calls
//   \b(query|read|write|get|post|put|delete|patch)\(  — method calls by name
//   \b(db|sql)\.  — db.* and sql.* receiver calls
//   \.(query|read|write|fetch)\(  — chained method calls
// ^ anchors the first alternative only; \b guards the rest from false prefixes.
static TS_JS_GO_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(fetch|axios)\b|\b(query|read|write|get|post|put|delete|patch)\(|\b(db|sql)\.|\.(query|read|write|fetch)\(",
    )
    .expect("data_access TS/JS/Go regex is valid")
});

// Python:
//   ^ anchors each alternative to the start of the text.
//   open(  — built-in file open
//   requests. / httpx. / cursor. / session. / conn.  — common data-access libs
static PYTHON_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(open\(|requests\.|httpx\.|cursor\.|session\.|conn\.)")
        .expect("data_access Python regex is valid")
});

/// Return `true` when `text` looks like a data-access callee for `language`.
///
/// Rust and Java always return `false` in v0 — data-access detection for those
/// languages is library-shaped (`sqlx::query!`, `reqwest::get`) and deferred to
/// a future regex pass. TypeScript, JavaScript, and Go share one regex table;
/// Python has its own.
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::queries::data_access::matches_callee;
///
/// assert!(matches_callee("fetch(\"/api/users\")", "typescript"));
/// assert!(matches_callee("cursor.execute(sql)", "python"));
/// assert!(!matches_callee("Math.max(a, b)", "typescript"));
/// assert!(!matches_callee("len(x)", "python"));
/// ```
pub fn matches_callee(text: &str, language: &str) -> bool {
    match language {
        "typescript" | "javascript" | "go" => TS_JS_GO_RE.is_match(text),
        "python" => PYTHON_RE.is_match(text),
        _ => false,
    }
}
