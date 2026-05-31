//! Node kinds classified as decorator usage.
//!
//! Detects TypeScript and JavaScript decorator syntax (`@Injectable()`,
//! `@Component({...})`, `@Get('/')`, `@IsString()`, etc.) via the `decorator`
//! node kind, and Python decorator syntax (`@dataclass`, `@app.get(...)`,
//! `@pytest.fixture`) via the `decorated_definition` node kind. Detection is
//! node-kind only — no callee allowlist is applied.
//!
//! ## Count semantics
//!
//! TypeScript/JavaScript: one instance **per decorator** (`decorator` node) —
//! three stacked `@`-lines on one class = three instances.
//!
//! Python: one instance **per decorated function or class**
//! (`decorated_definition` wrapper node) — three stacked `@`-lines on one
//! function = **one** instance. This v0 simplification is intentional; see
//! `docs/pattern-categories.md` for the rationale and future-alignment note.

/// Tree-sitter node kinds for decorator usage.
///
/// Two node kinds are classified here:
///
/// - `decorator` — TypeScript/JavaScript `@`-syntax (tree-sitter-typescript
///   and tree-sitter-javascript Stage-3 proposals). One instance per decorator
///   line.
/// - `decorated_definition` — Python `@`-decorated function or class wrapper
///   (tree-sitter-python). One instance per decorated function/class definition,
///   regardless of how many `@`-lines are stacked above it.
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::queries::decorators::NODE_KINDS;
///
/// assert!(NODE_KINDS.contains(&"decorator"));
/// assert!(NODE_KINDS.contains(&"decorated_definition"));
/// ```
pub const NODE_KINDS: &[&str] = &["decorator", "decorated_definition"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_kinds_contains_decorator() {
        assert!(NODE_KINDS.contains(&"decorator"));
    }

    #[test]
    fn node_kinds_contains_decorated_definition() {
        assert!(NODE_KINDS.contains(&"decorated_definition"));
    }

    #[test]
    fn node_kinds_has_two_entries() {
        assert_eq!(NODE_KINDS.len(), 2);
    }
}
