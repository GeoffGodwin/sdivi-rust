## Planned Tests
- [x] `crates/sdivi-patterns/tests/data_access_fixture.rs` — fixture-level integration: TypeScript & Python fixtures produce non-empty data_access bucket; Go call_expression maps to data_access
- [x] `crates/sdivi-patterns/tests/logging_fixture.rs` — integration sentinel: simple-typescript fixture produces NO `logging` bucket in `build_catalog` output (native pipeline never auto-classifies logging)
- [x] `crates/sdivi-lang-typescript/tests/extract_behavior.rs` — class_hierarchy hints: class_declaration, abstract_class_declaration, interface_declaration collected; plain and extending classes both collected
- [x] `crates/sdivi-lang-python/tests/extract_behavior.rs` — class_hierarchy hints: class_definition collected for bare class and class with base
- [x] `crates/sdivi-lang-java/tests/extract_behavior.rs` — class_hierarchy hints: class_declaration and interface_declaration collected
- [x] `crates/sdivi-lang-rust/tests/extract_behavior.rs` — class_hierarchy hints: impl_item collected for inherent and trait impls (new test file)
- [x] `crates/sdivi-lang-go/tests/extract_behavior.rs` — negative result: Go produces zero class_hierarchy pattern hints

## Test Run Results
Passed: 28  Failed: 0

## Bugs Found
None

## Files Modified
- [x] `crates/sdivi-patterns/tests/data_access_fixture.rs`
- [x] `crates/sdivi-patterns/tests/logging_fixture.rs`
- [x] `crates/sdivi-lang-typescript/tests/extract_behavior.rs`
- [x] `crates/sdivi-lang-python/tests/extract_behavior.rs`
- [x] `crates/sdivi-lang-java/tests/extract_behavior.rs`
- [x] `crates/sdivi-lang-rust/tests/extract_behavior.rs`
- [x] `crates/sdivi-lang-go/tests/extract_behavior.rs`
