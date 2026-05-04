//! Canonical pattern-category contract for sdivi-rust `snapshot_version "1.0"`.
//!
//! Embedders that supply their own tree-sitter extractors (e.g. the Meridian consumer app)
//! must use the category names returned by [`list_categories`] so that
//! [`crate::compute_pattern_metrics`] and [`crate::compute_delta`] produce meaningful
//! per-category divergence values.

use sdivi_snapshot::snapshot::SNAPSHOT_VERSION;
use serde::{Deserialize, Serialize};

/// Canonical category names in stable alphabetical order.
///
/// Every name here is a permanent contract entry for `snapshot_version "1.0"`.
/// Once a name is in this list it cannot be removed — only deprecated.
///
/// # Examples
///
/// ```rust
/// use sdivi_core::CATEGORIES;
///
/// assert!(CATEGORIES.contains(&"error_handling"));
/// assert_eq!(CATEGORIES.len(), 5);
/// ```
pub const CATEGORIES: &[&str] = &[
    "async_patterns",
    "error_handling",
    "resource_management",
    "state_management",
    "type_assertions",
];

/// (name, description) pairs matching [`CATEGORIES`] exactly.
///
/// The order matches [`CATEGORIES`] so that `zip` produces aligned pairs.
const CATEGORY_DESCRIPTIONS: &[(&str, &str)] = &[
    (
        "async_patterns",
        "Code constructs that implement or leverage asynchronous execution — \
        e.g., `.await` expressions on `Future` values and `async fn` definitions.",
    ),
    (
        "error_handling",
        "Code constructs that propagate, transform, or handle error conditions — \
        e.g., the `?` operator (`try_expression`) and `match` arms that dispatch \
        on `Result` or `Option` variants.",
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
        categories: CATEGORY_DESCRIPTIONS
            .iter()
            .map(|(name, desc)| CategoryInfo {
                name: name.to_string(),
                description: desc.to_string(),
            })
            .collect(),
    }
}
