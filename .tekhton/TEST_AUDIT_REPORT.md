## Test Audit Report

### Audit Summary
Tests audited: 1 file, 19 test functions (`crates/sdivi-patterns/tests/http_routing_limitations.rs`)
Verdict: PASS

### Findings

#### SCOPE: Shell-detected orphans are false positives
- File: crates/sdivi-patterns/tests/http_routing_limitations.rs:23
- Issue: The pre-audit orphan detector reported this file as importing the deleted module
  `.tekhton/.commit_decision` (the same entry appears twice). Reading the actual file reveals
  no such import. The `use` statements are:
  ```
  use sdivi_patterns::queries::{classify_hint, http_routing};
  use sdivi_patterns::PatternHintInput;
  ```
  `.tekhton/.commit_decision` is a pipeline state file, not a Rust module — the Rust module
  system cannot import it under any path form. The orphan detector produced spurious output.
  No orphaned tests exist in this file.
- Severity: LOW
- Action: Disregard the orphan warnings. Investigate the detection script's matching logic —
  this is the same false-positive pattern seen in the M40 audit. No test file changes needed.

---

#### NAMING: Test name overstates the assertion
- File: crates/sdivi-patterns/tests/http_routing_limitations.rs:110
- Issue: `classify_hint_idiosyncratic_receiver_falls_through_to_data_access` asserts only
  `!result.contains(&"http_routing")`. The name and inline comment claim the dispatch falls
  through to `data_access` (P9), but no assertion verifies `result.contains(&"data_access")`.
  The claim is factually correct — `\bget\(` in `data_access`'s `TS_JS_GO_RE` does match
  `api.get('/path', h)` — but the test does not enforce it. If the `data_access` regex
  changes to stop matching this input, the test continues to pass silently while the stated
  behavior breaks.
- Severity: LOW
- Action: Either add `assert!(result.contains(&"data_access"), "api.get should fall through to data_access; got {result:?}")`, or rename the test to `classify_hint_idiosyncratic_receiver_is_not_http_routing` to match what is actually asserted.

---

#### COVERAGE: Six assertions lack failure messages
- File: crates/sdivi-patterns/tests/http_routing_limitations.rs:56,64,96,101,106,189
- Issue: The following tests use bare `assert!(...)` without a message string:
  `nextjs_app_router_bare_put_is_not_http_routing` (line 56),
  `nextjs_app_router_bare_delete_is_not_http_routing` (line 64),
  `idiosyncratic_hono_variable_is_not_http_routing` (line 96),
  `idiosyncratic_my_router_variable_is_not_http_routing` (line 101),
  `idiosyncratic_app2_variable_is_not_http_routing` (line 106),
  `python_any_receiver_add_url_rule_matches_callee` (line 189).
  The other 13 tests in the file supply helpful failure messages. The inconsistency makes
  regressions harder to diagnose without inspecting source.
- Severity: LOW
- Action: Add message strings consistent with the rest of the file
  (e.g., `assert!(..., "hono receiver must not match http_routing")`).

---

### No Issues Found In

The following rubric points were checked and found clean:

- **Assertion Honesty**: Every assertion is derived from a real call to
  `http_routing::matches_callee` or `classify_hint`. No hard-coded magic values unconnected
  to implementation logic. Each assertion was cross-checked against the `TS_JS_RE`, `GO_RE`,
  and `PYTHON_RE` statics: all positive expectations match at least one regex arm; all
  negative expectations confirm the receiver/method combination falls outside every allowlist.
- **Edge Case Coverage**: The file covers both documented limitation classes
  (Next.js App Router bare-export handlers; idiosyncratic receiver names), all Go receiver
  tokens from the M41 spec not already in `category_contract_m41.rs`, and the Python
  receiver-agnostic path. Negative cases are the majority, appropriate for a limitations suite.
- **Implementation Exercise**: Tests call `http_routing::matches_callee` directly and also
  exercise the full `classify_hint` → `CALL_DISPATCH` (P7) → `http_routing::matches_callee`
  path. No mocks of any kind.
- **Test Weakening**: The tester added only new tests. No existing test was modified.
  No weakening is possible.
- **Test Naming (other tests)**: All other 18 names encode both the scenario and the expected
  outcome clearly. One naming issue is called out above (LOW).
- **Scope Alignment**: All imports resolve to live modules present in the current codebase.
  No reference to any deleted file. The shell-detected orphan report is a false positive
  (see finding above).
- **Test Isolation**: All 19 tests construct inputs from inline string literals. No test
  reads mutable project files, `.tekhton/` pipeline reports, build artifacts, or config
  state files. Tests have no ordering dependency on each other or on prior pipeline runs.

### Pre-Existing Issues (out of M41 scope — no action required)
- `wasm_package_json_version_matches_workspace`: `package.json` stranded at an older version.
  Pre-existing; not introduced by M41.
- `null_safety_node_kinds_do_not_match_non_ts_js_languages` name inversion (carry-over from
  M37, noted in M40 audit as MEDIUM): still present. Not introduced by M41.
