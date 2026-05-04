## Test Audit Report

### Audit Summary
Tests audited: 6 files (3 modified this run + 3 freshness samples), 13 Rust test functions + 1 Node validation script
Verdict: PASS

---

### Findings

#### COVERAGE: `accepts_valid_weights` asserts count only
- File: `bindings/sdivi-wasm/src/weight_keys.rs:60-66`
- Issue: The test inserts two entries and asserts `result.len() == 2`. It does not verify that the NUL-separated key format was actually applied. If `parse_wasm_edge_weights` were changed to pass keys through unchanged, this test would still pass; `converted_key_uses_nul_separator` would catch the regression, but `accepts_valid_weights` would provide no signal. The assertion is not false — count-of-inputs = count-of-outputs is real behaviour — but the test is redundant as a happy-path gate since the key-content tests already cover it.
- Severity: LOW
- Action: Either add an assertion that the expected NUL-separated key is present (matching the pattern in `converted_key_uses_nul_separator`), or remove this test and rely on `converted_key_uses_nul_separator` as the canonical happy-path test. No implementation change required.

#### COVERAGE: `exports["./node"]` and `exports["./bundler"]` subpath routing not validated
- File: `bindings/sdivi-wasm/tests/validate_pkg_template.cjs:58-69`
- Issue: The validator checks that `exports["./node"]` and `exports["./bundler"]` are present and non-null objects, but does not verify their internal routing keys. An empty object `{}` at either path would pass all current checks. The M24 constraints — that `./node` must expose a `"require"` key pointing into `node/` and `./bundler` must expose an `"import"` key pointing into `bundler/` — are unverified for the named subpaths (though they ARE verified for the root `"."` entry at lines 45-83).
- Severity: LOW
- Action: After the subpath presence checks, add assertions mirroring the root-entry pattern: verify `typeof pkg.exports['./node']['require'] === 'string'` and that it includes `'node/'`; similarly verify `pkg.exports['./bundler']['import']` includes `'bundler/'`. No implementation change required.

---

### Detailed Per-File Assessment

#### `bindings/sdivi-wasm/src/weight_keys.rs` — 13 test functions

**Assertion Honesty:** All assertions derive from calling the real function. Error-message checks use `contains()` on substrings that appear literally in the implementation's `format!` strings (verified by reading lines 25-48):
- `rejects_nan_weight`: asserts `e.contains("NaN")` — implementation: `"not finite (NaN)"`. ✓
- `rejects_positive_infinity_weight`: asserts `e.contains("finite")` — implementation: `"not finite (inf)"`. ✓
- `rejects_negative_infinity_weight` (NEW): asserts `e.contains("finite") || e.contains("infinite") || e.contains("inf")` — implementation: `"not finite (-inf)"`, satisfying both the first and third conditions. ✓
- `rejects_negative_weight`: asserts `e.contains("negative") || e.contains(">= 0.0")` — implementation: `"is negative (-0.5)"`. ✓
- `converted_key_uses_nul_separator` and `colon_in_node_id_produces_correct_nul_key`: assert `result.contains_key(&sdivi_core::input::edge_weight_key(...))` — tests actual output key, not a hardcoded guess. ✓

**Edge Case Coverage:** Comprehensive. Covers: empty map, valid 0.0 weight, NaN, +∞, −∞, negative float, missing colon, empty source, empty target, colon-in-node-id (two variants), weight value preservation, and NUL-key format. Healthy ratio of 9 error/edge-case tests to 4 happy-path tests.

**Implementation Exercise:** Every test calls `parse_wasm_edge_weights` directly with real `BTreeMap` inputs. No mocking.

**Test Weakening:** The tester added one new test (`rejects_negative_infinity_weight`, line 184) and did not modify any pre-existing test. No weakening.

**Naming:** All 13 names encode scenario and expected outcome (`rejects_empty_source`, `weight_value_preserved_after_key_conversion`, etc.). ✓

**Isolation:** All tests use inline `BTreeMap` literals. No filesystem or external state. ✓

---

#### `bindings/sdivi-wasm/tests/validate_pkg_template.cjs` — structural validation script

**Assertion Honesty:** All checks derive from `JSON.parse()` of the actual committed files. No hardcoded expected values that are independent of implementation logic; every `fail()` branch corresponds to a structural constraint that M24 imposes. ✓

**Edge Case Coverage:** Covers malformed JSON (with `process.exit(1)` fast-fail), missing `exports` field, missing root `"."` entry, missing `"import"`/`"require"` keys, wrong target directory references for those keys, missing `engines.node`, a sub-18 engine floor, and the stdin-redirect anti-pattern in the smoke package's test script. See COVERAGE finding above for the one gap.

**Implementation Exercise:** Reads and parses `pkg-template/package.json` — the implementation file for M24's conditional-exports shape. Every assertion reflects a real M24 requirement. ✓

**Weakening:** New test file; no pre-existing tests modified. ✓

**Isolation:** The script reads two committed source-controlled files (`pkg-template/package.json` and `tests/node_smoke/package.json`). These are static, version-controlled artifacts, not mutable pipeline state, CI run outputs, or `.tekhton/` reports. The rubric's ISOLATION flag targets live build reports and pipeline logs; it does not apply to source files that change only through commits. ✓

---

#### `bindings/sdivi-wasm/tests/node_smoke/package.json` — test project descriptor

No test assertions are present in this file. The tester fixed the `scripts.test` value from a stdin-redirect form to the direct invocation `"node index.cjs && node index.mjs"`, aligning it with the separate `node index.cjs` / `node index.mjs` steps in `wasm.yml:140-145`. The `validate_pkg_template.cjs` script enforces this alignment at line 128-136. No issues.

---

#### Freshness Samples

- `tests/boundary_lifecycle.rs` — layout placeholder (comment only, no executable test code). No tests to evaluate. The placeholder references `crates/sdivi-cli/tests/boundary_lifecycle.rs` as the real location; no scope drift from M24.
- `tests/full_pipeline.rs` — layout placeholder (comment only). Same pattern. No scope drift.
- `tests/fixtures/simple-rust/src/utils.rs` — fixture source file, not a test file. M24 does not touch parsing fixtures. No issues.

None of the freshness sample files reference any symbol or file deleted in this milestone. The deleted `.tekhton/test_dedup.fingerprint` is not referenced by any test under audit.

---

### Verdict Rationale

Zero HIGH findings. Two LOW-severity coverage gaps, both in areas where the failing conditions would manifest as CI regressions rather than silent false positives. The tester's addition (`rejects_negative_infinity_weight`) is correct: `f64::NEG_INFINITY` satisfies `is_infinite()` at line 25, the implementation produces `"not finite (-inf)"`, and the assertion `e.contains("finite")` is satisfied. The `validate_pkg_template.cjs` script faithfully exercises the actual committed `pkg-template/package.json` against the structural constraints M24 requires. No weakening, no orphaned tests, no mocked-out real code, no isolation violations.
