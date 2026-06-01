## Test Audit Report

### Audit Summary
Tests audited: 2 files, 25 test functions
(`crates/sdivi-core/tests/category_contract_m43.rs`: 17 functions;
`crates/sdivi-patterns/tests/dispatch_disjointness_supplement.rs`: 8 functions)
Verdict: PASS

### Findings

#### SCOPE: Shell-detected orphan claims are false positives
- File: crates/sdivi-core/tests/category_contract_m43.rs (all lines)
- File: crates/sdivi-patterns/tests/dispatch_disjointness_supplement.rs (all lines)
- Issue: The "Shell-Detected Orphans (pre-verified)" section claims both files import
  the deleted module `.tekhton/.commit_decision`. Manual inspection of both files (full
  read + grep) confirms neither contains any reference to `.tekhton/.commit_decision`.
  The only `use` declarations in `category_contract_m43.rs` are:
  ```
  use sdivi_patterns::queries::classify_hint;
  use sdivi_patterns::PatternHintInput;
  ```
  And in `dispatch_disjointness_supplement.rs`:
  ```
  use sdivi_patterns::queries::classify_hint;
  use sdivi_patterns::PatternHintInput;
  ```
  `.tekhton/.commit_decision` is a pipeline state file, not a Rust module — the Rust
  module system cannot import it at any path. The detection script produced four false
  positive entries (each file listed twice). No orphaned tests exist in either file.
- Severity: LOW
- Action: Disregard the orphan warnings. Investigate the detection script for its pattern-
  matching heuristic — same class of false positive observed in prior milestone audits.
  No changes to test files are needed.

#### COVERAGE: Unsupported language not exercised at integration level
- File: crates/sdivi-core/tests/category_contract_m43.rs (no specific line)
- Issue: `serialization::matches_callee` returns `false` for languages other than
  `typescript`, `javascript`, `python`, and `go`. This branch is covered in the inline
  unit tests within `serialization.rs` (`rust_returns_false`), but no function in
  `category_contract_m43.rs` exercises `classify_hint` with an unsupported-language
  input against a serialization-shaped callee text. The gap is minor — the unit test
  directly covers the path — but the integration layer has no representative case.
- Severity: LOW
- Action: Optional addition:
  `classify_hint(&hint("call_expression", "json.Marshal(v)"), "java")` asserting empty
  result (Go regex does not apply to Java). Not blocking; the unit coverage in
  `serialization.rs` is sufficient.

### No Issues Found In

The following rubric points were checked and found clean for both files:

**Assertion Honesty — PASS**
All 25 assertions derive their expected values from real implementation logic, verified
against the source regexes:
- TS_JS_RE (`^(JSON\.(parse|stringify)\(|structuredClone\()`) explains every
  TypeScript/JavaScript positive and confirms `schema.parse` is correctly excluded
  (anchored to `JSON.` prefix, not bare `.parse`).
- PYTHON_RE (`^(json|pickle)\.(loads|dumps|load|dump)\(`) explains every Python positive.
- GO_RE (`^json\.(Marshal|Unmarshal|MarshalIndent|NewEncoder|NewDecoder)\(`) explains
  every Go positive, including the two tester-added Gap 1 variants (`MarshalIndent`,
  `NewEncoder`).
- `z.object({})` → `schema_validation` verified: serialization P3 does not match
  `z.object`; schema_validation P4 does.
- `requests.get(url)` → `data_access` verified: Python data_access regex
  `^(open\(|requests\.|httpx\.|cursor\.|session\.|conn\.)` matches; serialization
  PYTHON_RE does not.
- `json.loads(s)` → not `data_access` verified: serialization PYTHON_RE matches at P3
  before data_access P9 is reached.
- `fmt.Print("x")` and `fmt.Errorf(...)` → `logging` verified against Go logging regex
  `^fmt\.(Print|Println|Printf|Errorf|Fprint|Sprint)` at CALL_DISPATCH P8.
- `Math.sqrt(x)` → empty Vec verified against all 10 CALL_DISPATCH matchers (no match in
  async_patterns, testing, serialization, schema_validation, state_store, framework_hooks,
  http_routing, logging, data_access, or collection_pipelines).
- Count assertion of 17 matches `CATALOG_ENTRIES` length in `categories.rs`.
No hard-coded magic values disconnected from implementation logic were found.

**Edge Case Coverage — PASS**
The suite covers: positives across three language families (TS/JS, Python, Go); cross-
category disambiguation (serialization vs schema_validation, serialization vs data_access,
dispatch-order priority); a genuinely unrecognized callee returning empty Vec; and catalog
metadata assertions. The ratio of negative/boundary tests to happy-path tests (~6:11 in
`category_contract_m43.rs`) is appropriate for a feature acceptance suite.

**Implementation Exercise — PASS**
Both files call real implementation functions with no mocking. `classify_hint` exercises
the full CALL_DISPATCH chain through `sdivi_patterns`. `sdivi_core::list_categories`
exercises `CATALOG_ENTRIES` directly.

**Test Weakening Detection — PASS**
The 7 corpus entries removed from `dispatch_disjointness.rs` at M43 (5 collection_pipelines
methods, 2 Go fmt variants, 1 unrecognized callee) are fully restored in
`dispatch_disjointness_supplement.rs` with matching or stronger assertions. No assertion
was removed; no expected value was broadened. All 8 supplement functions cover the
documented trimmed entries.

**Test Naming and Intent — PASS**
Every function name encodes the scenario and expected outcome:
`json_marshal_indent_is_serialization_go`, `schema_parse_is_not_serialization`,
`json_loads_not_data_access_in_python`, `requests_get_is_data_access_not_serialization`,
`unrecognised_callee_returns_empty`, `list_categories_count_is_seventeen`. No generic names.

**Scope Alignment — PASS** (false orphan claims notwithstanding — see finding above)
All imports resolve to current public API. The two tester-added tests
(`json_marshal_indent_is_serialization_go`, `json_new_encoder_is_serialization_go`) close
the reviewer-flagged Gap 1 documented in the tester report. Both variants are confirmed
covered by GO_RE in the implementation.

**Test Isolation — PASS**
Both files use a local `hint()` factory function to create all fixture data inline. Neither
file reads `.tekhton/` reports, pipeline logs, build artifacts, `package.json`, or any
mutable project state. Pass/fail outcomes are independent of prior pipeline runs and repo
state.

### Pre-Existing Issues (out of M43 scope — no action required)
- `wasm_package_json_version_matches_workspace`: `package.json` stranded at 0.2.23 vs
  workspace 0.2.34. Pre-existing; not introduced by M43.
- `null_safety_node_kinds_do_not_match_non_ts_js_languages` name inversion (carry-over
  from M37, noted in prior audits). Not introduced by M43.
- `category_for_node_kind` doc comment listing only `logging` as callee-only — several
  other categories are now also callee-only. Staleness accumulated across milestones.
