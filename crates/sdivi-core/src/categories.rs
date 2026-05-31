//! Canonical pattern-category contract for sdivi-rust `snapshot_version "1.0"`.
//!
//! Embedders that supply their own tree-sitter extractors (e.g. the Meridian consumer app)
//! must use the category names returned by [`list_categories`] so that
//! [`crate::compute_pattern_metrics`] and [`crate::compute_delta`] produce meaningful
//! per-category divergence values.

use sdivi_snapshot::snapshot::SNAPSHOT_VERSION;
use serde::{Deserialize, Serialize};

/// Single source of truth: (name, description) for every canonical category.
///
/// Every entry here is a permanent contract entry for `snapshot_version "1.0"`.
/// Once a name is in this list it cannot be removed — only deprecated.
/// [`CATEGORIES`] and [`list_categories`] are both derived from this array,
/// so the two cannot silently diverge.
const CATALOG_ENTRIES: &[(&str, &str)] = &[
    (
        "async_patterns",
        "Code constructs that implement or leverage asynchronous execution — \
        e.g., `.await` expressions on `Future` values and `async fn` definitions.",
    ),
    (
        "class_hierarchy",
        "Code constructs that establish inheritance, interface implementation, or trait \
        conformance relationships — e.g. classes with `extends`/`implements` clauses, \
        Python classes with base classes, and Rust `impl Trait for Type` blocks. All \
        declaration kinds are classified here regardless of whether they carry a \
        heritage clause; heritage-aware narrowing is the embedder's responsibility.",
    ),
    (
        "data_access",
        "Code constructs that perform I/O against data stores or external resources — \
        e.g., database queries (`query`, `cursor.*`), HTTP fetches (`fetch`), \
        file reads (`open`, `read`), and ORM method calls. All `call_expression` / \
        `call` nodes are classified here; callee-name narrowing is the embedder's \
        responsibility.",
    ),
    (
        "error_handling",
        "Code constructs that propagate, transform, or handle error conditions — \
        e.g., the `?` operator (`try_expression`) and `match` arms that dispatch \
        on `Result` or `Option` variants.",
    ),
    (
        "framework_hooks",
        "Component-composition hook calls in React, Preact, Vue (composables), and \
        Svelte-style runtimes — any `call_expression` callee matching `^use[A-Z]` in \
        TypeScript or JavaScript. Covers built-in hooks (`useState`, `useEffect`, \
        `useMemo`, `useCallback`, `useRef`, `useContext`, `useReducer`, \
        `useLayoutEffect`) and the full custom-hook ecosystem (`useAuth`, `useStore`, \
        etc.). Other languages produce no instances.",
    ),
    (
        "logging",
        "Code constructs that produce diagnostic or observability output — \
        e.g., `console.*` calls, structured logger invocations (`logger.info`), \
        `print` statements, and logging macros (`tracing::info!`, `log::debug!`). \
        Natively classified since M33 via `classify_hint` callee-text inspection: \
        `category_for_node_kind` does not return `Some(\"logging\")` (the relevant \
        node kinds overlap with `data_access` and `resource_management`), but \
        `classify_hint` routes matching callees to this category. Foreign extractors \
        may also emit `PatternInstanceInput { category: \"logging\", … }` directly; \
        those instances merge with natively classified ones.",
    ),
    (
        "resource_management",
        "Code constructs that allocate, release, or manage system or heap resources — \
        e.g., macro invocations such as `drop!`, `vec!`, or standard I/O macros.",
    ),
    (
        "state_management",
        "Code constructs that capture, transform, or share mutable or shared state — \
        e.g., closures that close over mutable bindings or shared references.",
    ),
    (
        "type_assertions",
        "Code constructs that assert or coerce between types at compile or runtime — \
        e.g., `as` casts (`as_expression`) and language-specific type-cast expressions.",
    ),
];

/// Canonical category names in stable alphabetical order.
///
/// Derived from the private `CATALOG_ENTRIES` table — the two cannot diverge.
/// Every name here is a permanent contract entry for `snapshot_version "1.0"`.
/// Once a name is in this list it cannot be removed — only deprecated.
///
/// # Examples
///
/// ```rust
/// use sdivi_core::CATEGORIES;
///
/// assert!(CATEGORIES.contains(&"error_handling"));
/// assert!(CATEGORIES.contains(&"framework_hooks"));
/// assert!(CATEGORIES.contains(&"logging"));
/// assert_eq!(CATEGORIES.len(), 9);
/// ```
pub const CATEGORIES: &[&str] = &[
    CATALOG_ENTRIES[0].0,
    CATALOG_ENTRIES[1].0,
    CATALOG_ENTRIES[2].0,
    CATALOG_ENTRIES[3].0,
    CATALOG_ENTRIES[4].0,
    CATALOG_ENTRIES[5].0,
    CATALOG_ENTRIES[6].0,
    CATALOG_ENTRIES[7].0,
    CATALOG_ENTRIES[8].0,
];

/// Metadata for a single canonical pattern category.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CategoryInfo {
    /// Canonical category name — stable across all `snapshot_version "1.0"` output.
    pub name: String,
    /// Human-readable description of the code constructs this category covers.
    pub description: String,
}

/// Runtime representation of the canonical pattern-category contract.
///
/// Returned by [`list_categories`]. Embedders should call this function
/// instead of hard-coding category names so they stay aligned with the contract.
///
/// # Examples
///
/// ```rust
/// use sdivi_core::list_categories;
///
/// let catalog = list_categories();
/// assert_eq!(catalog.schema_version, "1.0");
/// assert!(!catalog.categories.is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CategoryCatalog {
    /// The `snapshot_version` string this contract applies to.
    pub schema_version: String,
    /// All canonical categories in alphabetical order.
    pub categories: Vec<CategoryInfo>,
}

/// Return the canonical pattern-category contract for `snapshot_version "1.0"`.
///
/// The returned [`CategoryCatalog`] is the authoritative source of truth for:
///
/// - Which category names are valid in [`crate::input::PatternInstanceInput::category`].
/// - Which category names appear as keys in per-category divergence maps.
/// - Which category names are accepted by `[thresholds.overrides.<cat>]` in `config.toml`.
///
/// Embedders that supply their own tree-sitter extractors MUST use these names
/// verbatim — the comparison in `compute_pattern_metrics` is case-sensitive.
///
/// This function is referentially transparent: two calls return equal values.
///
/// # Examples
///
/// ```rust
/// use sdivi_core::list_categories;
///
/// let a = list_categories();
/// let b = list_categories();
/// assert_eq!(a, b, "list_categories must be referentially transparent");
/// assert_eq!(a.schema_version, "1.0");
///
/// let names: Vec<&str> = a.categories.iter().map(|c| c.name.as_str()).collect();
/// assert!(names.contains(&"error_handling"));
/// assert!(names.contains(&"async_patterns"));
/// ```
pub fn list_categories() -> CategoryCatalog {
    CategoryCatalog {
        schema_version: SNAPSHOT_VERSION.to_string(),
        categories: CATALOG_ENTRIES
            .iter()
            .map(|(name, desc)| CategoryInfo {
                name: name.to_string(),
                description: desc.to_string(),
            })
            .collect(),
    }
}
