## Test Audit Report

### Audit Summary
Tests audited: 4 files, ~20 discrete test scenarios
- `bindings/sdivi-wasm/tests/test_check_docs.sh` — 5 test scenarios (15 sub-assertions)
- `bindings/sdivi-wasm/tests/test_negative_integrity.sh` — 13 numbered tests
- `bindings/sdivi-wasm/tests/typecheck/subpath_imports.ts` — TypeScript fixture (validated by tsc)
- `bindings/sdivi-wasm/tests/typecheck/tsconfig.json` — Configuration file for the tsc guard

Verdict: **NEEDS_WORK**

---

### Findings

#### INTEGRITY: Pre-verified orphan detection is entirely wrong — do not delete these files
- File: bindings/sdivi-wasm/tests/test_check_docs.sh, test_negative_integrity.sh, typecheck/subpath_imports.ts, typecheck/tsconfig.json
- Issue: The audit context's "Shell-Detected Orphans (pre-verified)" section claims all four test files import the deleted module `.tekhton/.commit_decision`. A full grep of `bindings/sdivi-wasm/tests/` for `.commit_decision` and `.tekhton` returns zero matches. None of the test files contain any reference to that path. This is the same class of false positive seen in M44–M46 audits: the detection script matches against the audit context preamble (which mentions the deleted file) rather than inspecting the actual test file contents.
- Severity: HIGH
- Action: Do NOT remove or modify any of the four listed test files on the basis of the orphan report. Fix the orphan-detection script — this is at least the fourth consecutive false-positive from it.

#### INTEGRITY: Wrong grep anchor produces false negative for the known spurious directive
- File: bindings/sdivi-wasm/tests/test_negative_integrity.sh:38, :77
- Issue: Tests 1 and 2 detect `@ts-expect-error` directives using the pattern `"^// @ts-expect-error$"` — anchored at the end with `$`. TypeScript recognises any comment line that *starts with* `// @ts-expect-error` (with or without trailing text) as a suppression directive. `negative.ts` line 70 contains `// @ts-expect-error directives above would become unused and tsc would fail.` — TypeScript interprets this as a directive; the immediately following `void _badEdgeWeights;` (line 71) has no type error, so tsc emits TS2578 "Unused '@ts-expect-error' directive". Test 1 counts 4 matching lines (pattern misses line 70) and passes. Test 2 never sets `PREV_WAS_DIRECTIVE=1` for line 70 and therefore skips the check. Both tests produce false negatives; neither would catch a future recurrence of this class of bug.
- Severity: HIGH
- Action: In Test 1 (`test_negative_integrity.sh:38`), change `"^// @ts-expect-error$"` to `"^// @ts-expect-error"` (drop the trailing `$`). In Test 2 (line 77), same change. Then fix `negative.ts:70`: reword the prose comment so it does not begin with `// @ts-expect-error` — e.g., change it to `// The @ts-expect-error directives above would become unused...`.

#### EXERCISE: Shell tests never run tsc; the primary M47 acceptance criterion is untested
- File: bindings/sdivi-wasm/tests/test_check_docs.sh, bindings/sdivi-wasm/tests/test_negative_integrity.sh
- Issue: M47's primary acceptance criterion is that `npx tsc --noEmit -p bindings/sdivi-wasm/tests/typecheck/tsconfig.json` exits 0. Neither shell test invokes tsc. The tester's report explicitly states "tsc --noEmit exits non-zero before and after my changes" due to two unfixed bugs: (1) `negative.ts:70` spurious directive (described in the INTEGRITY finding above) and (2) `tsconfig.json "lib": ["ES2020"]` lacks DOM types, causing TS2584 on `console.log` in `examples/binding_node.ts` and `examples/binding_bundler.ts`. The reported "Passed: 46  Failed: 0" result covers only structural grep checks — none of which exercise the broken tsc gate. The M47 feature adds a CI guard that currently fails in CI.
- Severity: HIGH
- Action: Fix both bugs: (1) fix `negative.ts:70` (see INTEGRITY finding); (2) add `"dom"` to `"lib"` in `tsconfig.json:24` (`"lib": ["ES2020", "dom"]`). Then add a test that actually runs tsc — extend `test_negative_integrity.sh` with a step that invokes `npx tsc --noEmit -p <tsconfig-path>` and asserts exit 0 (guarded by an `npx tsc` availability check so the test degrades gracefully when TypeScript is not installed locally).

#### COVERAGE: check_docs.sh failure path has no end-to-end coverage
- File: bindings/sdivi-wasm/tests/test_check_docs.sh:60-111
- Issue: Tests 2 and 3 exercise `grep -nF` in isolation against controlled temp files. They prove that the grep tool can detect the patterns — not that `check_docs.sh` handles the failure path correctly. Test 1 is the only test that calls the actual script, and it covers only the happy path (exit 0). There is no test that injects a forbidden pattern into a file that `check_docs.sh` actually scans and then asserts the script exits 1 with a `FAIL:` prefixed line. If `check_docs.sh`'s exit-code or output-format logic were broken, Tests 2-3 would still pass because they bypass the script entirely.
- Severity: MEDIUM
- Action: Add an end-to-end failure-path test: copy one of the scanned files to a temp path, inject a forbidden pattern, temporarily redirect `check_docs.sh`'s file list to include only the temp file (or add a `FILES` env-override to the script), run the script, and assert exit 1 plus a `FAIL:` prefix in stdout. Alternatively, refactor `check_docs.sh` to accept an explicit file list argument so tests can supply a fully controlled input without patching.

#### SCOPE: Two implementation bugs identified by tester remain unfixed despite tester modifying the affected file
- File: bindings/sdivi-wasm/tests/typecheck/negative.ts:70; bindings/sdivi-wasm/tests/typecheck/tsconfig.json:24
- Issue: The tester correctly identified both bugs. `tsconfig.json` is in the tester's "Files Modified" list — the tester added paths/include entries but did not fix `"lib": ["ES2020"]` even while editing the file. `negative.ts` is not in the tester's "Files Modified" list and was not fixed either. The tester report documents these as "Bugs Found" with no corresponding fix.
- Severity: MEDIUM
- Action: Fix `negative.ts:70` (reword comment) and `tsconfig.json:24` (add `"dom"` to lib). Both are one-line fixes. They are blocking: no part of M47's tsc guard is functional until both are applied.

---

### Notes

**subpath_imports.ts quality**: The TypeScript fixture is well-designed. It imports from both `/bundler` and `/node` subpaths, uses `edge_weights: new Map([...])` (correct shape), iterates `Map` outputs with for-of rather than bracket indexing, and has anti-vacuous-pass `void` guards. Once the tsconfig bugs are fixed and tsc can successfully compile the file, the fixture closes the subpath coverage gap it was written to address. No issues with the fixture itself.

**test_negative_integrity.sh Test 1 count**: The count of 4 is correct for the *intended* directives (lines 34, 44, 53, 63). After the grep pattern fix, it will also detect line 70, raising the count to 5. The fix to `negative.ts:70` (rewording the comment) must be applied first, then the pattern fix — otherwise the test will fail on count 5 ≠ 4 with no prior fix having been made.

**test_check_docs.sh Tests 4-5**: These tests read live version-controlled source files (`examples/binding_node.ts`, `examples/binding_bundler.ts`). This is intentional and appropriate — they are checking the same files the production guard is designed to protect. These are not mutable pipeline artifacts; they are source files. No isolation concern here.

**False-positive orphan detection**: This is the fourth consecutive audit with false orphan claims (M44, M45.1, M45.2, M47). The detection script is a systemic reliability problem. Recommend fixing it before the next audit cycle or removing the pre-verification step and relying solely on the auditor's direct file inspection.
