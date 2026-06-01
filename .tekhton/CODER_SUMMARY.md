# Coder Summary
## Status: COMPLETE

## What Was Implemented

### M47: WASM Consumer-Surface Typecheck Guard

Added a deterministic CI guard that typechecks the `@geoffgodwin/sdivi-wasm`
consumer TypeScript surface against the freshly built `.d.ts`, with a
self-verifying negative fixture and a forbidden-pattern doc lint.

**New files:**

- `bindings/sdivi-wasm/tests/typecheck/tsconfig.json` — strict consumer tsconfig
  with `paths` → freshly built `pkg/*.d.ts` and `include` → repo-root examples +
  `./negative.ts`. Settings: `strict`, `noUncheckedIndexedAccess`,
  `exactOptionalPropertyTypes`, `noEmit`, `incremental: false`, `esModuleInterop`,
  `skipLibCheck`, `target: ES2020`, `moduleResolution: bundler`, `module: ESNext`.

- `bindings/sdivi-wasm/tests/typecheck/negative.ts` — self-verifying negative
  fixture with `@ts-expect-error` assertions for:
  - Case 1: `await init()` — module namespace is not callable (TS2349)
  - Case 2a: `edge_weights: { ... }` as plain object literal (not Map<string,number>)
  - Case 2b: `overrides: { ... }` as plain object literal (not Map<string,WasmThresholdOverrideInput>)
  - Case 3: bracket-indexing `Map<string,number>` output (TS7053)
  - Guard section with `void` calls prevents vacuous pass from path typos.

- `bindings/sdivi-wasm/tests/check_docs.sh` — POSIX sh forbidden-pattern lint;
  greps six consumer-facing files for `import init`, `await init(`, `edge_weights: {`;
  exits non-zero with `FAIL: <path>:<line>` on any hit. Verified: passes on current
  tree; fails when a forbidden pattern is injected.

- `DRIFT_LOG.md` — created with M47 entry explaining the M12-era `await init()`
  idiom drift and how M47 closes it.

**Modified files:**

- `.github/workflows/wasm.yml`:
  - Added `TYPESCRIPT_VERSION: "5.5.4"` env key (pinned, like WASM_PACK_VERSION).
  - Added three ubuntu-only steps after "Set up node_modules for Node smoke tests":
    1. Install TypeScript (pinned) via `npm install --no-save --no-package-lock`
    2. Typecheck consumer surface: `npx tsc --noEmit -p bindings/sdivi-wasm/tests/typecheck/tsconfig.json`
    3. Lint consumer docs: `sh bindings/sdivi-wasm/tests/check_docs.sh`
  - All steps are required (no `continue-on-error`), ordered after `pkg/` assembly.

- `bindings/sdivi-wasm/tests/node_smoke/index.mjs`: updated stale "bundler path not
  exercised" comment to accurately state the bundler *type* contract is now validated
  by M47's `tsc` guard, while the bundler *runtime* path remains upstream-validated
  (deferred per M47 Non-Goals).

- `examples/binding_node.ts`: removed `commit: undefined` and `boundary_count: undefined`
  from the `assemble_snapshot` call. These caused TS2322 under `exactOptionalPropertyTypes:
  true` (the optional fields `commit?: string` cannot accept explicit `undefined`). The
  fix is to simply omit the keys. Updated the adjacent comment.

- `CHANGELOG.md`: added M47 "Added" entry under `[Unreleased]`.
- `.gitignore`: added `tsconfig.tsbuildinfo`, `**/tsconfig.tsbuildinfo`, and `node_modules/`
  (the npm install for the typecheck step creates node_modules at repo root in CI).

## Root Cause (bugs only)
N/A — feature addition (CI test infrastructure).

## Architecture Change Proposals
None.

## Files Modified
- `bindings/sdivi-wasm/tests/typecheck/tsconfig.json` (NEW) — 39 lines
- `bindings/sdivi-wasm/tests/typecheck/negative.ts` (NEW) — 76 lines
- `bindings/sdivi-wasm/tests/check_docs.sh` (NEW) — 80 lines
- `DRIFT_LOG.md` (NEW) — 37 lines
- `.github/workflows/wasm.yml` — added TYPESCRIPT_VERSION env, 3 ubuntu steps
- `bindings/sdivi-wasm/tests/node_smoke/index.mjs` — updated stale comment
- `examples/binding_node.ts` — removed `commit: undefined`, `boundary_count: undefined`
- `CHANGELOG.md` — M47 Added entry
- `.gitignore` — tsconfig.tsbuildinfo, node_modules/

## Human Notes Status
Non-blocking notes from reviewer were out of scope for M47:
- `comprehensions.rs:73-76` — test name rename suggestion: NOT_ADDRESSED (M46 scope)
- `mod.rs:37-39` — doc comment coverage: NOT_ADDRESSED (M46 scope)

## Docs Updated
None — no public Rust/TS API surface changes in this milestone.
The examples are consumer documentation and were updated (`binding_node.ts`
had explicit `undefined` values removed, which is an improvement).

## Observed Issues (out of scope)
- Pre-existing: `wasm_package_json_version_matches_workspace` — package.json at
  0.2.23 vs workspace 0.2.38 (not introduced by M47).
- `binding_node.ts` previously had `commit: undefined` and `boundary_count: undefined`
  — fixed in this milestone as a necessary consequence of the typecheck guard (not
  a pre-existing out-of-scope issue).
