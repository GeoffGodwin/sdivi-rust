## Test Audit Report

### Audit Summary
Tests audited: 1 file (`crates/sdivi-core/tests/category_contract_m39.rs`), 21 test functions
Verdict: PASS

### Findings

#### SCOPE: Orphan detector false positive
- File: crates/sdivi-core/tests/category_contract_m39.rs
- Issue: The shell-detected orphan report claims this file "imports deleted module
  `.tekhton/.commit_decision`". Reading the actual file confirms no such import exists.
  The file imports only `sdivi_patterns::queries::classify_hint`,
  `sdivi_patterns::PatternHintInput`, and uses `sdivi_core::list_categories()` via
  fully-qualified path. The orphan detector produced a false positive — likely a
  naive filename pattern match that associated the deleted `.tekhton/.commit_decision`
  entry with this file. The "pre-verified" label is incorrect.
- Severity: MEDIUM
- Action: Investigate the orphan detection tool's input. Do not modify or remove this
  test file on the basis of this claim. No code change is needed.

#### SCOPE: Tester-added tests are off-milestone scope for the file name
- File: crates/sdivi-core/tests/category_contract_m39.rs:188-217
- Issue: The three tests added by the tester (`list_categories_includes_resource_management`,
  `list_categories_includes_state_management`, `list_categories_includes_type_assertions`)
  back-fill coverage for categories introduced in milestones earlier than M39. Their
  placement in a file named `category_contract_m39.rs` makes the file's scope ambiguous.
  The tester report correctly identifies the rationale ("native counterparts were missing
  from category_contract.rs") but does not justify the file choice.
- Severity: LOW
- Action: Accept these tests — the assertions are honest and fill a real coverage gap.
  If future maintainers find the mixed scope confusing, move the three tests to
  `category_contract.rs`; this is not required to pass the milestone.

#### None (Assertion Honesty, Test Weakening, Isolation, Naming, Exercise, Coverage)

All 21 tests call real implementation functions with meaningful inputs. Each assertion
was traced through the actual implementation:

- `use_selector_is_state_store_not_framework_hooks`: `classify_hint` iterates
  `CALL_DISPATCH`; `state_store::matches_callee("useSelector(s => s.user)", "typescript")`
  matches `^use(Selector|Dispatch|Store)\b` at slot P5 before `framework_hooks` reaches
  slot P6. Assertion `vec!["state_store"]` is grounded in real dispatch order.
- `use_effect_is_framework_hooks_not_state_store`: `useEffect` does not match the
  state_store regex; it then hits `framework_hooks::matches_callee` at P6 via `^use[A-Z]`.
  Assertion `vec!["framework_hooks"]` is grounded.
- `prisma_create_is_not_state_store`: `prisma.user.create(data)` does not start with
  `create(` (the `^create\(` anchor excludes member-access paths). Assertion grounded.
- `state_store_does_not_fire_for_python`: `state_store::matches_callee` returns `false`
  for any language other than `"typescript"` | `"javascript"`. Assertion grounded.
- `list_categories_includes_state_store` and the three tester-added
  `list_categories_includes_*` tests: `list_categories()` iterates all 13
  `CATALOG_ENTRIES`; `state_store` is at index 11, `resource_management` at 8,
  `state_management` at 10, `type_assertions` at 12. All four assertions grounded.

No existing tests were weakened. The tester only added lines 184–217 to the file
the coder created (lines 1–182); no assertions were broadened or removed.
Tests create no external state and read no mutable project files — all test data
is constructed inline via the `call_hint` helper or direct struct literals.
Test names encode the scenario and expected outcome throughout.

### Pre-Existing Issues (out of M39 scope — no action required)
- `wasm_package_json_version_matches_workspace`: `package.json` stranded at an older
  version. Pre-existing; not introduced by M39.
- `null_safety_node_kinds_do_not_match_non_ts_js_languages` in `queries/tests.rs`:
  test name implies a negative but the body asserts a match. Pre-existing from M37;
  out of M39 audit scope.
