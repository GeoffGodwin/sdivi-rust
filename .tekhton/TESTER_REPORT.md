## Planned Tests
- [x] Verify `async_patterns` hybrid classification fix (Note 1) — test `async_patterns_is_hybrid_both_node_kind_and_callee` in `crates/sdivi-patterns/tests/test_all_categories_doc_classification.rs` passes
- [x] Verify all 8 callee-only categories listed in `ALL_CATEGORIES` doc (Note 8, 18, 35) — `logging`, `testing`, `serialization`, `schema_validation`, `state_store`, `framework_hooks`, `http_routing`, `collection_pipelines` all present
- [x] Verify blank line exists between `CALL_DISPATCH` and `classify_hint` doc (Note 17, 34, 40) — blank line present at line 191
- [x] Verify `select_statement` SQL adapter guard comment exists (Note 13) — seed comment added to `concurrency.rs` NODE_KINDS doc
- [x] Verify `framework_hooks` routes to `state_store` in description (Note 20, 36, 66) — routing note in `crates/sdivi-core/src/categories.rs:85-86`
- [x] Verify all 19 category names covered in WASM smoke tests (Note 22, 38) — all 19 asserted in `bindings/sdivi-wasm/tests/wasm_smoke.rs:246-264`
- [x] Verify `null_safety` test name reflects actual behavior (Note 23, 39, 40) — test renamed to `category_for_node_kind_is_language_unaware_optional_chain_always_maps_to_null_safety` at line 290
- [x] Verify `logging` doc reflects M33 native classification (Note 56) — M33 mentioned correctly in `category_for_node_kind` doc
- [x] Verify `CHANGELOG.md` classify_hint signature correct (Note 59) — signature shown as `Vec<&'static str>` (correct)
- [x] Fix WASM package.json version mismatch (Note 29/33/46/53) — version bumped from 0.2.41 to 0.2.42

## Test Run Results
Passed: 294  Failed: 0

## Bugs Found
None

## Files Modified
- [x] `bindings/sdivi-wasm/pkg-template/package.json` — bumped version from 0.2.41 to 0.2.42 to match workspace
