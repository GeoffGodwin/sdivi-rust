//! Native (non-wasm) tests for M23 — run with `cargo test -p sdivi-wasm --test m23_native`.
//!
//! These cover the serde field-name contract and struct accessibility for
//! `WasmCategoryCatalog` / `WasmCategoryInfo` without requiring a WASM runtime.
//! The wasm-pack suite (`wasm_smoke.rs`) covers the JS-callable path.

use sdivi_wasm::category_types::{WasmCategoryCatalog, WasmCategoryInfo};

#[test]
fn wasm_category_catalog_fields_survive_serde_round_trip() {
    let catalog = WasmCategoryCatalog {
        schema_version: "1.0".into(),
        categories: vec![WasmCategoryInfo {
            name: "error_handling".into(),
            description: "Handles errors".into(),
        }],
    };
    let json = serde_json::to_value(&catalog).unwrap();
    let back: WasmCategoryCatalog = serde_json::from_value(json).unwrap();
    assert_eq!(back.schema_version, "1.0");
    assert_eq!(back.categories.len(), 1);
    assert_eq!(back.categories[0].name, "error_handling");
}

#[test]
fn wasm_category_catalog_json_field_names_are_schema_version_and_categories() {
    let catalog = WasmCategoryCatalog {
        schema_version: "1.0".into(),
        categories: vec![],
    };
    let json = serde_json::to_value(&catalog).unwrap();
    assert!(
        json.get("schema_version").is_some(),
        "field 'schema_version' must be present"
    );
    assert!(
        json.get("categories").is_some(),
        "field 'categories' must be present"
    );
    assert_eq!(
        json.as_object().unwrap().len(),
        2,
        "expected exactly 2 fields: schema_version, categories"
    );
}

#[test]
fn list_categories_wasm_export_returns_five_categories() {
    // Call the export function via sdivi_core directly (wasm-bindgen not involved in native builds)
    let catalog = sdivi_core::list_categories();
    assert_eq!(catalog.schema_version, "1.0");
    assert_eq!(catalog.categories.len(), 5);
}

#[test]
fn list_categories_includes_all_expected_names() {
    let catalog = sdivi_core::list_categories();
    let names: Vec<&str> = catalog.categories.iter().map(|c| c.name.as_str()).collect();
    for expected in [
        "async_patterns",
        "error_handling",
        "resource_management",
        "state_management",
        "type_assertions",
    ] {
        assert!(
            names.contains(&expected),
            "expected category {:?} in list_categories()",
            expected
        );
    }
}
