## Test Audit Report

### Audit Summary
Tests audited: 3 files, 39 test functions
Verdict: PASS

### Findings

#### SCOPE: Shell-detected orphans are false positives
- File: crates/sdivi-core/tests/category_contract_m40.rs, crates/sdivi-patterns/src/queries/tests.rs, crates/sdivi-patterns/tests/dispatch_disjointness.rs
- Issue: The pre-audit orphan detector reported all three test files as importing the deleted module `.tekhton/.commit_decision`. None of the three files contain any reference to that path — the shell detection script produced false positives. All imports in these files are valid: `sdivi_patterns::queries::classify_hint`, `sdivi_patterns::PatternHintInput`, `sdivi_core::list_categories`, and the dispatch sub-modules. No orphaned tests exist. This is the same false-positive pattern documented in the M39 audit; the detection script's naive filename/path match is the likely cause.
- Severity: LOW
- Action: Disregard the orphan warnings. Investigate the detection script's matching logic so it does not false-flag future runs. No test file changes needed.

---

#### NAMING: `null_safety_node_kinds_do_not_match_non_ts_js_languages` name contradicts body
- File: crates/sdivi-patterns/src/queries/tests.rs:259
- Issue: The test name reads "do_not_match_non_ts_js_languages" (negation), but the assertion body is `assert_eq!(..., Some("null_safety"))` — asserting that the function *does* return `null_safety` for rust/python/go/java. The accompanying comment correctly explains the behavior (`category_for_node_kind` is language-unaware, so `optional_chain` maps to `null_safety` regardless of language). The assertion is correct; only the name is inverted. This is a pre-existing carry-over from M37 noted in CODER_SUMMARY.md; M40 modified this file so the issue is now in the audit scope.
- Severity: MEDIUM
- Action: Rename to `optional_chain_maps_to_null_safety_regardless_of_language` (or similar) to match what the body actually asserts. No assertion changes needed.

---

#### COVERAGE: No integration-level positive tests for `go` and `java` in collection_pipelines
- File: crates/sdivi-core/tests/category_contract_m40.rs, crates/sdivi-patterns/tests/dispatch_disjointness.rs
- Issue: `collection_pipelines::matches_callee` explicitly returns `true` for `"go"` and `"java"` (same `TS_JS_RE` path as TypeScript/JavaScript). No test in category_contract_m40.rs or the dispatch corpus exercises `("xs.map(f)", "go")` or `("stream.map(f)", "java")` as positive cases. The inline unit tests in collection_pipelines.rs only verify `"python"` and `"rust"` as returning false, not that `"go"` and `"java"` return true. The language arm for Go/Java in `matches_callee` is untested at the integration level.
- Severity: LOW
- Action: Add corpus entries to the P10 section of dispatch_disjointness.rs CORPUS: `("xs.map(f)", "go", "collection_pipelines")` and `("list.stream().map(f)", "java", "collection_pipelines")`. Optionally add a `go_and_java_map_is_collection_pipelines` test to category_contract_m40.rs.

---

### No Issues Found In

The following rubric points were checked and found clean:

- **Assertion Honesty**: All assertions call real functions (`classify_hint`, `list_categories`, `category_for_node_kind`, `matches_callee`) with meaningful inputs. No hard-coded magic values unrelated to implementation logic. The count 14 is directly derived from `ALL_CATEGORIES.len()` and `CATALOG_ENTRIES.len()`.
- **Test Weakening**: The only modification to an existing test was renaming `all_categories_has_thirteen_entries` → `all_categories_has_fourteen_entries` and updating the count 13 → 14. This is a correct update, not a weakening — the precision of the assertion is unchanged.
- **Implementation Exercise**: All three files call real implementation functions with no mocking of classification logic. `corpus_resolves_to_expected_category` exercises all 7 active CALL_DISPATCH entries including the new P10 slot. `list_categories_includes_collection_pipelines` calls the real `sdivi_core::list_categories()`.
- **Test Isolation**: All tests operate on in-memory inputs. No test reads filesystem state, pipeline logs, `.tekhton/` reports, or other mutable project files. No cross-test ordering dependency.
- **Scope Alignment**: All imports resolve to modules that exist in the current codebase. The `collection_pipelines` module is present at the expected path. CALL_DISPATCH slot order and KNOWN_OVERLAPS entries are consistent with what the implementation does.
- **Test Naming (M40 additions)**: All new test names are descriptive and encode both the scenario and the expected outcome (e.g., `data_access_methods_are_not_collection_pipelines`, `known_overlaps_winner_matches_dispatch_order`, `xs_map_f_is_collection_pipelines`).

### Pre-Existing Issues (out of M40 scope — no action required)
- `wasm_package_json_version_matches_workspace`: `package.json` stranded at an older version. Pre-existing; not introduced by M40.
- `null_safety_node_kinds_do_not_match_non_ts_js_languages` name inversion: upgraded to MEDIUM above because M40 modified the enclosing file; the underlying behavior is still correct.
