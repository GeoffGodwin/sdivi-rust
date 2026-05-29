//! Node kinds classified as class-hierarchy patterns.
//!
//! These node kinds correspond to the `class_hierarchy` category in the
//! [`PatternCatalog`](crate::catalog::PatternCatalog).

/// Tree-sitter node kinds for class-hierarchy patterns.
///
/// All declaration kinds are classified here regardless of whether the
/// declaration has an `extends`/`implements`/`for Trait` clause. Heritage-
/// aware narrowing is the embedder's responsibility (or a future native
/// enhancement); the entropy/drift signal survives the broader collection
/// because hierarchy-free declarations have low structural variance and
/// therefore contribute low entropy.
///
/// - `class_declaration`: TypeScript / Java / JavaScript class declarations.
/// - `class_definition`: Python class definitions.
/// - `abstract_class_declaration`: TypeScript abstract classes (always part
///   of a hierarchy by definition — they cannot be instantiated directly).
/// - `interface_declaration`: TypeScript / Java interface declarations
///   (define the contract another type implements).
/// - `impl_item`: Rust `impl Type {…}` and `impl Trait for Type {…}` blocks.
pub const NODE_KINDS: &[&str] = &[
    "class_declaration",
    "class_definition",
    "abstract_class_declaration",
    "interface_declaration",
    "impl_item",
];
