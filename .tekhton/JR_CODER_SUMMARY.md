# JR Coder Summary — M35: Pattern Category `framework_hooks`

## What Was Fixed

- **`crates/sdivi-core/tests/category_contract.rs`**: Added two new test functions to explicitly verify the M35 category contract:
  - `list_categories_returns_exactly_nine_categories()` — asserts the catalog contains exactly 9 categories (P1–P9)
  - `list_categories_includes_framework_hooks()` — asserts that `framework_hooks` is present in the runtime category list

- **`bindings/sdivi-wasm/tests/wasm_smoke.rs`**: Updated the WASM integration test `list_categories_returns_schema_version_and_expected_count()`:
  - Changed hardcoded count assertion from 8 to 9 categories
  - Added `framework_hooks` to the explicit category checks
  - Updated comment from "8 categories defined" to "9 categories defined (P1–P9)"
  - Reordered category assertions into alphabetical order for consistency

## Files Modified

- `crates/sdivi-core/tests/category_contract.rs`
- `bindings/sdivi-wasm/tests/wasm_smoke.rs`

## Verification

All tests pass:
- ✓ `cargo test -p sdivi-core --test category_contract` — 8 tests pass (6 original + 2 new)
- ✓ `cargo test -p sdivi-patterns --test framework_hooks` — 12 tests pass
- ✓ Integration test `no_category_string_in_patterns_src_missing_from_list_categories` confirms `framework_hooks` from `sdivi-patterns` is properly registered in `sdivi-core::list_categories()`
- ✓ WASM build compiles successfully with updated test assertions
