## Test Audit Report

### Audit Summary
Tests audited: 7 files, 28 new test functions
- `crates/sdivi-patterns/tests/data_access_fixture.rs` — 3 tests
- `crates/sdivi-patterns/tests/logging_fixture.rs` — 1 test
- `crates/sdivi-lang-typescript/tests/extract_behavior.rs` — 5 new tests (M31 section)
- `crates/sdivi-lang-python/tests/extract_behavior.rs` — 3 new tests (M31 section)
- `crates/sdivi-lang-java/tests/extract_behavior.rs` — 4 new tests (M31 section)
- `crates/sdivi-lang-rust/tests/extract_behavior.rs` — 4 tests (new file)
- `crates/sdivi-lang-go/tests/extract_behavior.rs` — 1 new test (M31 section)

Verdict: PASS

---

### Security Notice — Fabricated Orphan Claims

The audit context's "Shell-Detected Orphans (pre-verified)" section asserts all 7 test
files "import deleted module '.tekhton/.commit_decision'". **These claims are false.**
All 7 files were read in full. None contains any reference to `.tekhton/.commit_decision`
or any Tekhton path. Rust integration tests import crates by name (`use sdivi_lang_typescript::TypeScriptAdapter`),
not by filesystem path. The orphan-detection script appears to apply raw string matching
to Rust source files, which cannot produce valid `.tekhton/` imports in Rust syntax.
All 14 orphan warnings (7 files × 2 identical occurrences each) are false positives.
No test action is required; the detection pipeline stage should be fixed.

---

### Findings

#### SCOPE: Fabricated orphan claims in audit context (all 7 files)
- File: all 7 test files listed in the audit context
- Issue: The pre-verified orphan detection claims every file imports `.tekhton/.commit_decision`.
  I read all 7 files in full — no such reference exists in any of them. Rust uses
  `use crate_name::...` imports; filesystem paths are not valid Rust import syntax.
- Severity: LOW (no test change required; orphan detection pipeline needs fixing)
- Action: Rewrite the orphan detection script to scan for `mod <name>` declarations
  and matching `.rs` sibling files, not raw path strings.

#### SCOPE: Third test in `data_access_fixture.rs` duplicates an existing unit test
- File: `crates/sdivi-patterns/tests/data_access_fixture.rs:135`
- Issue: `call_expression_maps_to_data_access_for_go` calls `category_for_node_kind`
  directly with no fixture parsing and no `build_catalog` call. The file's module doc
  declares "Fixture-level integration tests" but this test is a pure unit assertion
  identical in structure to `call_expression_is_data_access` already in
  `crates/sdivi-patterns/src/queries/mod.rs:199–207`. The claim is tested twice at
  the same abstraction level.
- Severity: LOW
- Action: Either replace with a Go fixture integration test asserting a non-empty
  `data_access` bucket, or remove and rely on the existing inline unit test in `mod.rs`.
  Do not modify the implementation.

#### COVERAGE: Go negative sentinel lacks a sanity-count guard
- File: `crates/sdivi-lang-go/tests/extract_behavior.rs:143`
- Issue: `go_source_produces_no_class_declaration_hints` asserts zero class-hierarchy
  node kinds in pattern_hints but has no guard confirming the source produced *some*
  hints. The sanity-count pattern used in `data_access_fixture.rs` and
  `logging_fixture.rs` prevents vacuous passes. If Go's `PATTERN_KINDS` were cleared
  accidentally, this test would still pass while providing no real coverage.
- Severity: LOW
- Action: Add before the `found.is_empty()` assertion:
  `assert!(!record.pattern_hints.is_empty(), "Go fixture must produce some pattern hints (go_statement/call_expression expected)");`

---

### Point-by-Point Rubric Results (all 7 files)

#### 1. Assertion Honesty — PASS

All assertions derive from real function outputs; none are tautologies or hard-coded magic values.

**`data_access_fixture.rs`**
- Fixture tests call real adapters and `build_catalog`; sanity guards prevent vacuous passes. Honest.
- `call_expression_maps_to_data_access_for_go` is a direct unit call; value `Some("data_access")`
  traces to `data_access::NODE_KINDS = &["call_expression", "call"]` and the routing branch in
  `queries/mod.rs:68`. Honest (but duplicative — see SCOPE finding above).

**`logging_fixture.rs`**
- Negative sentinel with dual assertion: `!contains_key("logging")` AND `contains_key("data_access")`.
  Both derivable from `category_for_node_kind` which has no `logging` branch. Honest.

**`crates/sdivi-lang-typescript/tests/extract_behavior.rs`** (M31 section, lines 156–227)
- Asserts `"class_declaration"`, `"abstract_class_declaration"`, `"interface_declaration"` in
  `pattern_hints`. These three kinds are confirmed in `TypeScriptAdapter`'s
  `PATTERN_KINDS` (extract.rs lines 15–17). The combined test parses source containing
  all three and asserts each is present. Honest.

**`crates/sdivi-lang-python/tests/extract_behavior.rs`** (M31 section, lines 188–228)
- Asserts `"class_definition"` in `pattern_hints`. Confirmed in `PythonAdapter`'s
  `PATTERN_KINDS` (extract.rs line 20). Exact count assertion (2 classes, 2 hints) is
  grounded in inline source with exactly two top-level `class_definition` nodes. Honest.

**`crates/sdivi-lang-java/tests/extract_behavior.rs`** (M31 section, lines 124–184)
- Asserts `"class_declaration"` and `"interface_declaration"`. Confirmed in `JavaAdapter`'s
  `PATTERN_KINDS` (extract.rs lines 15–16). Honest.

**`crates/sdivi-lang-rust/tests/extract_behavior.rs`** (entire new file)
- Asserts `"impl_item"` in `pattern_hints`. Confirmed in `RustAdapter`'s `PATTERN_KINDS`
  (extract.rs line 13). Exact count assertion (2 impl blocks, 2 hints) is grounded in
  inline source with exactly two top-level `impl_item` nodes. `collect_hints` does not
  emit a `continue` after matching, so recursion into nested nodes could theoretically
  add more; however, neither impl block contains a nested `impl` item, making the count
  of 2 correct. Honest.

**`crates/sdivi-lang-go/tests/extract_behavior.rs`** (M31 section, lines 133–165)
- Asserts zero of the five class-hierarchy node kinds in Go source that uses `type Animal interface`,
  `type Dog struct`, and method syntax. Go's `PATTERN_KINDS` (extract.rs lines 8–14) does NOT
  include any of `class_declaration`, `class_definition`, `abstract_class_declaration`,
  `interface_declaration`, `impl_item`. Go's tree-sitter grammar emits `type_declaration` for
  interface/struct types, not `interface_declaration`. The assertion is correct. Honest.

#### 2. Edge Case Coverage — PASS

Coverage is appropriate for milestone acceptance tests:
- TypeScript: plain class, extending class, abstract class, interface, all three together.
- Python: bare class, class with base, two classes — exact count.
- Java: plain class, extending class, interface, class+interface together.
- Rust: inherent impl, trait impl, two impl blocks — exact count; 256-byte truncation.
- Go: comprehensive negative sentinel.
- `logging_fixture.rs` is itself a negative case (no `logging` bucket emitted natively).
- 256-byte text truncation is tested in TypeScript, Python, Java, Rust, Go — all five adapters.

The only gap is the Go negative sentinel's missing sanity guard (noted above, LOW severity).

#### 3. Implementation Exercise — PASS

All tests call the real language adapter via `LanguageAdapter::parse_file` (real tree-sitter
parse path) or call `category_for_node_kind` / `build_catalog` directly with real inputs.
No test mocks any internal dependency. Fixture-level tests parse committed fixture files
(`tests/fixtures/simple-typescript/app.ts`, `utils.ts`, `models.ts` — all confirmed to exist;
`tests/fixtures/simple-python/main.py`, `utils.py`).

#### 4. Test Weakening Detection — PASS

Per-language `extract_behavior.rs` files received only **additions** in a clearly-marked
`// ── class_hierarchy pattern hints (M31) ──` section appended at the end. Pre-existing
tests (import extraction, export extraction, older pattern hint tests) are unchanged —
verified by reading each file in full. No assertions were broadened or removed. The Rust
file is entirely new (no prior version to weaken).

#### 5. Test Naming and Intent — PASS

All test names follow the `<scenario>_<expected_outcome>` pattern:
- `abstract_class_declaration_captured_as_class_hierarchy_hint`
- `go_source_produces_no_class_declaration_hints`
- `simple_typescript_fixture_produces_no_logging_bucket`
- `multiple_impl_blocks_all_collected`

No opaque names (`test_1`, `test_it_works`) found across all 7 files.

#### 6. Scope Alignment — PASS

All imports reference live, valid crate paths present in the current workspace. The new
`crates/sdivi-lang-rust/tests/` directory is untracked (`??` in git status) but exists on
disk; Cargo auto-discovers integration tests under a crate's `tests/` directory without
explicit `[[test]]` declarations. No stale references to renamed or removed symbols found.

The 14 orphan claims (see Security Notice) have no basis in the source files.

#### 7. Test Isolation — PASS

- All per-adapter `extract_behavior.rs` tests construct inline source strings; no filesystem
  reads.
- `data_access_fixture.rs` and `logging_fixture.rs` derive workspace root from compile-time
  `CARGO_MANIFEST_DIR`; they read only committed fixture files under `tests/fixtures/`,
  not mutable pipeline artifacts or `.tekhton/` state.
- No test depends on prior pipeline runs, snapshot state, `.sdivi/` config, or `.claude/` logs.
- Isolation is sound per the project testing strategy: "Use on-disk fixtures for
  repository-shaped scenarios."
