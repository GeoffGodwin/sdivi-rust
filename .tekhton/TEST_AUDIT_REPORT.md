## Test Audit Report

### Audit Summary
Tests audited: 2 files, 31 test functions
- `crates/sdivi-patterns/tests/http_routing_limitations.rs` — 19 tests
- `crates/sdivi-patterns/tests/testing_scope_exclude.rs` — 12 tests
Verdict: PASS

### Findings

#### SCOPE: Shell-detected orphan reports are false positives
- File: crates/sdivi-patterns/tests/http_routing_limitations.rs (all lines)
- File: crates/sdivi-patterns/tests/testing_scope_exclude.rs (all lines)
- Issue: The pre-audit orphan detector flagged both files (four entries total) as importing the
  deleted module `.tekhton/.commit_decision`. Manual inspection of both files confirms neither
  contains any reference to `.tekhton/.commit_decision` or the `.tekhton` directory.
  The only `use` declarations in `http_routing_limitations.rs` are:
  ```
  use sdivi_patterns::queries::{classify_hint, http_routing};
  use sdivi_patterns::PatternHintInput;
  ```
  And `testing_scope_exclude.rs`:
  ```
  use sdivi_config::PatternsConfig;
  use sdivi_parsing::feature_record::{FeatureRecord, PatternHint};
  use sdivi_patterns::build_catalog;
  ```
  `.tekhton/.commit_decision` is a pipeline state file, not a Rust module — the Rust module
  system cannot import it at any path. No orphaned tests exist in either file.
- Severity: LOW
- Action: Disregard the orphan warnings. Investigate the detection script for pattern-matching
  false positives (same issue observed in the M41 audit). No changes to test files needed.

---

#### NAMING: Test name overstates what is asserted
- File: crates/sdivi-patterns/tests/http_routing_limitations.rs:125
- Issue: `classify_hint_idiosyncratic_receiver_falls_through_to_data_access` asserts only
  `!result.contains(&"http_routing")`. The name and inline comment claim the dispatch falls
  through to `data_access` (P9), but no assertion verifies `result.contains(&"data_access")`.
  The claim is plausible given the `data_access` regex, but is not enforced by the test body.
  If the `data_access` regex changes to stop matching `api.get(...)`, the test continues
  to pass silently while the documented behavior breaks.
- Severity: LOW
- Action: Either add `assert!(result.contains(&"data_access"), "api.get should fall through to data_access; got {result:?}")`, or rename to `classify_hint_idiosyncratic_receiver_is_not_http_routing` to match what is actually asserted.

---

#### COVERAGE: No-op iteration block in non_test_call_in_test_file_is_not_testing
- File: crates/sdivi-patterns/tests/testing_scope_exclude.rs:331
- Issue: The `if let Some(testing_entries) = catalog.entries.get("testing") { for (_fp, stats) in testing_entries { let _ = stats; } }` block iterates entries but makes no assertion — `let _ = stats` is a no-op. The surrounding comment ("structural check: each fingerprint is its own entry") describes intent but the loop body does not assert anything. The two `assert!` calls after the block are correct and meaningful; the loop contributes nothing and could mislead a reader into thinking it verified something.
- Severity: LOW
- Action: Remove the no-op loop. The assertions on lines 340–347 (`logging` and `testing` buckets present) stand on their own.

---

### No Issues Found In

The following rubric points were checked and found clean for both files:

- **Assertion Honesty**: All assertions derive from real calls to `matches_callee`, `classify_hint`,
  and `build_catalog`. No hard-coded magic values disconnected from implementation logic.
  Cross-checked against `TS_JS_RE`, `GO_RE`, `PYTHON_RE` (http_routing) and `TS_JS_GLOBALS_RE`,
  `TS_JS_FRAMEWORK_RE`, `GO_RE`, `PYTHON_RE` (testing): every positive expectation matches at
  least one regex arm; every negative expectation falls outside all arms.
- **Edge Case Coverage**: `http_routing_limitations.rs` covers the two documented limitation
  classes (Next.js App Router bare-verb, idiosyncratic receivers), all Go allowlist receivers
  not already in `category_contract_m41.rs`, and the Python receiver-agnostic path.
  `testing_scope_exclude.rs` covers in-scope population (TS/JS, Go, Python, Jest/Vitest
  helpers), exclusion (four glob patterns across all languages), the mixed case, input-slice
  immutability, and the cross-category disambiguation case.
- **Implementation Exercise**: Both files call real implementation code with no mocks.
  `http_routing_limitations.rs` exercises both the direct `matches_callee` path and the full
  `classify_hint` → `CALL_DISPATCH` → module path. `testing_scope_exclude.rs` exercises
  `build_catalog` end-to-end with real `globset` exclusion logic.
- **Test Weakening**: The tester added only new test files. No existing tests were modified.
  No weakening is possible.
- **Test Naming**: All names encode the scenario and expected outcome except the one LOW case
  noted above.
- **Scope Alignment**: All imports resolve to live modules. Both files compile against the
  current codebase. Orphan reports are false positives (see finding above).
- **Test Isolation**: All tests construct their own fixture data inline. No test reads mutable
  project files, `.tekhton/` pipeline reports, build artifacts, or config state files.
  Tests have no ordering dependency on each other or on prior pipeline runs.

### Pre-Existing Issues (out of M42 scope — no action required)
- `wasm_package_json_version_matches_workspace`: `package.json` stranded at 0.2.23 vs
  workspace 0.2.33. Pre-existing; not introduced by M42.
- `null_safety_node_kinds_do_not_match_non_ts_js_languages` name inversion (carry-over from
  M37, noted in prior audits as MEDIUM): still present. Not introduced by M42.
