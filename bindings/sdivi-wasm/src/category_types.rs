//! Tsify-derived WASM wrapper types for the pattern-category contract.
//!
//! Mirrors [`sdivi_core::CategoryCatalog`] and [`sdivi_core::CategoryInfo`]
//! with identical field names so the `from_core` serde round-trip converts
//! between them without explicit `From` impls.

use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

/// Metadata for a single canonical pattern category — WASM wrapper.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmCategoryInfo {
    /// Canonical category name.
    pub name: String,
    /// Human-readable description of the code constructs this category covers.
    pub description: String,
}

/// Runtime representation of the canonical pattern-category contract — WASM wrapper.
///
/// Returned by [`list_categories`](crate::exports::list_categories).
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmCategoryCatalog {
    /// The `snapshot_version` string this contract applies to (`"1.0"`).
    pub schema_version: String,
    /// All canonical categories in alphabetical order.
    pub categories: Vec<WasmCategoryInfo>,
}
