## Test Audit Report

### Audit Summary
Tests audited: 1 file (modified this run), 2 freshness-sample test files
Test functions audited: 19 (extract_behavior.rs) + 5 (proptest_seeded.rs freshness)
Verdict: PASS

---

### Findings

#### SCOPE: Shell orphan detector produced false-positive warnings
- File: `crates/sdivi-lang-java/tests/extract_behavior.rs` (all lines)
- Issue: The "Shell-Detected Orphans (pre-verified)" section claims this file
  "imports deleted module '.tekhton/.commit_decision'" (reported twice).
  Manual inspection of the actual file confirms the only `use` declarations are:
  ```
  use sdivi_lang_java::JavaAdapter;
  use sdivi_parsing::adapter::LanguageAdapter;
  use sdivi_parsing::feature_record::FeatureRecord;
  use std::path::Path;
  ```
  `.tekhton/.commit_decision` is a pipeline state file, not a Rust module path.
  It cannot appear as a Rust `use` declaration. No orphaned import exists.
  This is the same class of false positive identified in the M44 and M43 audits.
  The double-report of the same entry for the same file suggests the detection
  script is running its heuristic twice or pattern-matching across non-Rust
  source in the audit context preamble rather than in the file itself.
- Severity: MEDIUM
- Action: Fix the orphan-detection script. No change needed in
  `extract_behavior.rs`. This recurrence across three consecutive milestone
  audits warrants a prioritized script fix.

---

### No Issues Found In

The following rubric points were checked and found clean for
`crates/sdivi-lang-java/tests/extract_behavior.rs`.

**1. Assertion Honesty — PASS**

All three M45.1 assertions derive their expected values from real implementation
behavior:

- `try_with_resources_statement_captured_as_pattern_hint` checks that the
  string `"try_with_resources_statement"` appears as a `node_kind` in
  `pattern_hints` after a real `JavaAdapter.parse_file` call.  This string is
  the exact tree-sitter Java grammar node kind and is the same value added to
  `NODE_KINDS` in `resource_management.rs:24`.  Not invented.
- `try_with_resources_hint_text_starts_with_try` asserts `hint.text.starts_with("try")`.
  Grounded: Java try-with-resources syntax mandates `try` as the first lexeme,
  so this assertion will fail if the adapter emits wrong text, and cannot be a
  vacuous pass.
- `try_with_resources_is_distinct_from_try_statement` independently parses two
  distinct Java source strings and asserts each produces exactly its correct
  node kind while not producing the other.  This is the strongest form — it
  would catch both false-positive and false-negative classification bugs.

No hard-coded magic numbers.  No tautological assertions (`x == x`, `true`).

**2. Edge Case Coverage — PASS**

M45.1 additions cover:
- Happy-path emission: `try_with_resources_statement_captured_as_pattern_hint`
- Text-content property: `try_with_resources_hint_text_starts_with_try`
  (also re-checks the 256-byte limit for this new node kind)
- Boundary / anti-conflation: `try_with_resources_is_distinct_from_try_statement`

Pre-existing tests provide additional coverage:
- `pattern_hints_text_does_not_exceed_256_bytes` — length-limit invariant
  with multi-byte (`"á"`) input, still present and unmodified
- `package_private_class_is_not_exported` — negative path (no false positives)

No test for empty-source or syntactically invalid Java, but those are orthogonal
to M45.1 scope (adapter resilience, not classification correctness), and the
file had no such coverage before this change.

**3. Implementation Exercise — PASS**

Every M45.1 test calls `parse()`, which delegates directly to
`JavaAdapter.parse_file(Path::new("Test.java"), source.to_string())`.  This is
the real tree-sitter parse path; no internals are mocked.  The tester's
commentary accurately notes that the companion `resource_management_fixture.rs`
tests use *synthetic* `FeatureRecord` structs and therefore never exercise the
Java grammar.  This file covers the adapter's actual AST traversal.

**4. Test Weakening Detection — PASS**

The tester added three new tests at lines 123–225.  No existing test was
modified.  All 16 pre-existing test functions are intact and their assertions
are unchanged.  No assertion was broadened (no `assert_eq` → `assert!`
substitutions).

**5. Test Naming and Intent — PASS**

All three new test names encode both the scenario and the expected outcome:
- `try_with_resources_statement_captured_as_pattern_hint`
- `try_with_resources_hint_text_starts_with_try`
- `try_with_resources_is_distinct_from_try_statement`

These follow the same convention established by the file's pre-existing names
(`plain_import_yields_qualified_name`, `package_private_class_is_not_exported`,
etc.).

**6. Scope Alignment — PASS** (false orphan claim notwithstanding — see finding above)

All `use` declarations resolve to current public API.  The three new tests
exercise `JavaAdapter` behavior introduced/confirmed by M45.1.  No test
references a deleted function, renamed symbol, or removed feature.

**7. Test Isolation — PASS**

All 19 tests construct inputs via the inline `parse()` factory with literal
source strings.  No test reads from `.tekhton/`, `target/`, build reports,
pipeline logs, `Cargo.lock`, or any mutable project file.  Pass/fail outcome is
fully independent of prior pipeline runs and repo state.

---

### Freshness Sample Assessment

**`crates/sdivi-detection/tests/proptest_seeded.rs`** — FRESH

Not modified this run.  No stale references; all imports resolve to current API
(`sdivi_detection::leiden::run_leiden`, `sdivi_detection::partition::LeidenConfig`,
`sdivi_graph::dependency_graph::build_dependency_graph`).  Coverage:
determinism (100 identical runs at seed=42), seed variation, disconnected
components, and the empty-graph boundary case.  The `proptest!` block for
`prop_any_seed_deterministic` exercises the seeding invariant over the full
u32 seed range.  No integrity concerns.

**`crates/sdivi-detection/tests/proptest_seeded.proptest-regressions`** — FRESH

Contains 2 saved regression seeds from prior proptest shrinking runs
(seeds 0 and 2860851616).  Both are valid `cc`-format entries.  No stale
content.

**`.tekhton/test_dedup.fingerprint`** — NOT A TEST FILE

Contains a single SHA-1 hash.  This is a pipeline dedup artifact; no test
assertions to audit.

---

### Summary Table

| File | Functions | Verdict | Notes |
|---|---|---|---|
| `crates/sdivi-lang-java/tests/extract_behavior.rs` | 19 | PASS | 3 M45.1 additions; pre-existing 16 unchanged |
| `crates/sdivi-detection/tests/proptest_seeded.rs` | 5 | PASS (freshness) | Unchanged; no stale references |
| `proptest_seeded.proptest-regressions` | — | PASS (freshness) | 2 saved seeds; valid |

**Overall: PASS** — No HIGH findings.  One MEDIUM finding (recurring false-positive
in the orphan detection script — no action required on any test file).  The M45.1
additions to `extract_behavior.rs` are honest, well-exercised, well-named, and
fully isolated.
