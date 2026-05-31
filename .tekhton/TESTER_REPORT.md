## Planned Tests
- [x] `crates/sdivi-lang-typescript/tests/extract_behavior.rs` — decorator hint integration tests via real tree-sitter parse (NestJS-shaped fixture)
- [x] `crates/sdivi-lang-javascript/tests/extract_behavior.rs` — JS decorator hint test (grammar-dependent; documents behavior)
- [x] `crates/sdivi-core/tests/category_contract.rs` — verify `decorated_definition_python_is_decorators` (M36.2 acceptance criterion) and full category contract suite
- [x] `crates/sdivi-lang-python/tests/extract_behavior.rs` — verify M36.2 decorator hint tests: happy path, FastAPI/pytest integration, stacked-decorator wrapper-granularity, negative case
- [x] `crates/sdivi-patterns/src/queries/decorators.rs` (inline tests) — verify `node_kinds_contains_decorated_definition` and `node_kinds_has_two_entries`

## Test Run Results
Passed: 51  Failed: 0

## Bugs Found
None

## Files Modified
- [x] `crates/sdivi-lang-typescript/tests/extract_behavior.rs`
- [x] `crates/sdivi-lang-javascript/tests/extract_behavior.rs`
- [x] `crates/sdivi-core/tests/category_contract.rs`
- [x] `crates/sdivi-lang-python/tests/extract_behavior.rs`
- [x] `crates/sdivi-patterns/src/queries/decorators.rs`
