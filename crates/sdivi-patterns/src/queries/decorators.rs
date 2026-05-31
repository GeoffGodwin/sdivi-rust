//! Node kinds classified as decorator usage.
//!
//! Detects TypeScript and JavaScript decorator syntax (`@Injectable()`,
//! `@Component({...})`, `@Get('/')`, `@IsString()`, etc.). Detection is
//! node-kind only — any `decorator` node counts, giving broad collection
//! in the spirit of `class_hierarchy`. Decorator-shape entropy is the signal;
//! no callee allowlist is applied.

/// Tree-sitter node kinds for decorator usage.
///
/// The `decorator` kind is emitted by tree-sitter-typescript (and tree-sitter-
/// javascript for Stage-3 proposals). Every decorator node is classified here
/// regardless of its name or arguments — NestJS `@Injectable()`, Angular
/// `@Component({…})`, TypeORM `@Entity()`, class-validator `@IsString()`, and
/// custom user decorators all produce instances.
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::queries::decorators::NODE_KINDS;
///
/// assert!(NODE_KINDS.contains(&"decorator"));
/// ```
pub const NODE_KINDS: &[&str] = &["decorator"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_kinds_contains_decorator() {
        assert!(NODE_KINDS.contains(&"decorator"));
    }

    #[test]
    fn node_kinds_has_exactly_one_entry() {
        assert_eq!(NODE_KINDS.len(), 1);
    }
}
