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
fn list_categories_wasm_export_returns_eight_categories() {
    // Test name is a historical artifact from M23 (8 categories). Count grows
    // with each milestone; update the assertion when new categories are added.
    let catalog = sdivi_core::list_categories();
    assert_eq!(catalog.schema_version, "1.0");
    assert_eq!(catalog.categories.len(), 15); // M41: 15 categories
    let names: Vec<&str> = catalog.categories.iter().map(|c| c.name.as_str()).collect();
    assert!(
        names.contains(&"data_access"),
        "expected category \"data_access\" in list_categories()"
    );
    assert!(
        names.contains(&"logging"),
        "expected category \"logging\" in list_categories()"
    );
    assert!(
        names.contains(&"class_hierarchy"),
        "expected category \"class_hierarchy\" in list_categories()"
    );
    assert!(
        names.contains(&"http_routing"),
        "expected category \"http_routing\" in list_categories()"
    );
}

#[test]
fn list_categories_includes_all_expected_names() {
    let catalog = sdivi_core::list_categories();
    let names: Vec<&str> = catalog.categories.iter().map(|c| c.name.as_str()).collect();
    for expected in [
        "async_patterns",
        "class_hierarchy",
        "data_access",
        "error_handling",
        "logging",
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
