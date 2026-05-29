## Test Audit Report

### Audit Summary
Tests audited: 1 file (`crates/sdivi-patterns/tests/data_access_fixture.rs`), 3 test functions
Verdict: PASS

### Findings

#### COVERAGE: Happy-path only — no negative fixture coverage
- File: `crates/sdivi-patterns/tests/data_access_fixture.rs:59,97`
- Issue: Both fixture-level tests assert `total >= 1` but neither tests the boundary case where a source file produces zero `call_expression`/`call` nodes. If a future fixture regression strips the function calls from `app.ts` or `main.py`, the sanity-check assertions (lines 71–75 and 109–112) would catch it — but only because they happen to be present. There is no test documenting what the catalog looks like for a file that has no data-access nodes (expected: key absent, not present with count 0).
- Severity: LOW
- Action: Consider adding a test that parses a fixture with no function calls (e.g., a file containing only type declarations or constants) and asserts `data_access` is absent from `catalog.entries`. This clarifies the catalog's contract: entries are only present when count > 0.

#### SCOPE: Test 3 is a unit-level call, not a fixture-level integration test
- File: `crates/sdivi-patterns/tests/data_access_fixture.rs:135`
- Issue: `call_expression_maps_to_data_access_for_go` calls `category_for_node_kind("call_expression", "go")` directly — no Go fixture is parsed, no `build_catalog` is invoked. The file's module-level doc comment (`//! Fixture-level integration tests`) and the tester report both describe this file as fixture-level integration tests, but this test is a pure unit assertion identical in structure to the inline test `call_expression_is_data_access` already present in `crates/sdivi-patterns/src/queries/mod.rs:118–127`. The same claim is tested twice at the same level, adding noise without adding fixture-level coverage.
- Severity: LOW
- Action: Either (a) replace this test with a Go fixture integration test that parses `tests/fixtures/simple-go/` (if that fixture contains function calls) and asserts a non-empty `data_access` bucket, or (b) remove it and rely on the existing inline unit test in `mod.rs`. Do not modify the implementation.

### Findings — None Found for Remaining Rubric Points

#### INTEGRITY
No issues. All three tests derive assertions from real function calls:
- Tests 1 & 2 feed real fixture files through real language adapters (`TypeScriptAdapter`, `PythonAdapter`) into `build_catalog`, then assert on the returned map. The fixtures contain function calls (`helper('/tmp')`, `path.replace(...)`, `os.getcwd()`, `re.sub(...)`) that genuinely produce `call_expression` and `call` nodes in tree-sitter, so the `>= 1` assertions are non-trivial.
- Test 3 asserts `category_for_node_kind("call_expression", "go") == Some("data_access")`. The implementation (`mod.rs:55–56`) checks `data_access::NODE_KINDS.contains(&node_kind)` and `NODE_KINDS = &["call_expression", "call"]` (`data_access.rs:14`), so the expected value is derived directly from the implementation.

No hard-coded magic values, no always-passing tautologies.

#### WEAKENING
Not applicable. `data_access_fixture.rs` is a new file (untracked in git status at session start: `?? crates/sdivi-patterns/tests/data_access_fixture.rs`). No existing test was modified or weakened.

#### NAMING
All three names follow the `<scenario>_<outcome>` convention:
- `simple_typescript_fixture_produces_data_access_bucket` — encodes language, input type, and expected output
- `simple_python_fixture_produces_data_access_bucket` — same structure
- `call_expression_maps_to_data_access_for_go` — encodes node kind, target category, and language context

No opaque names.

#### EXERCISE
Tests 1 and 2 exercise the full pattern-stage stack: file read → real tree-sitter parse via language adapter → `build_catalog` → catalog map inspection. No mocking. Test 3 calls `category_for_node_kind` directly, which is appropriate for a routing-function assertion.

#### ISOLATION
The fixture files read by tests 1 and 2 (`tests/fixtures/simple-typescript/app.ts`, `utils.ts`, `tests/fixtures/simple-python/main.py`, `utils.py`) are committed, stable source files designed as test inputs, not mutable pipeline output or run artifacts. This follows the CLAUDE.md testing strategy: "Use on-disk fixtures for repository-shaped scenarios (parsing, graph, full pipeline)." No test reads `.tekhton/`, `.sdivi/`, or any build artifact. Isolation is sound.
