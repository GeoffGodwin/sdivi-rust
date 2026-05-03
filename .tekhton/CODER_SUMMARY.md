# Coder Summary
## Status: COMPLETE

## What Was Implemented

### M24: Node.js WASM Distribution Target

- **`bindings/sdivi-wasm/build.sh`** (MODIFIED): Updated to produce both wasm-pack targets:
  - `pkg/bundler/` — `wasm-pack build --target bundler` (ESM, `import.meta.url`-style wasm loading for webpack/vite/rollup)
  - `pkg/node/` — `wasm-pack build --target nodejs` (CJS, synchronous `require('fs')` wasm loading for Node 18+ CLI/server)
  - `pkg/package.json` assembled from `pkg-template/package.json` after each build.
  - `wasm-opt` applied to both targets in release mode.
  - Combined 5 MB budget check (instead of per-target 1.2 MB only).

- **`bindings/sdivi-wasm/pkg-template/package.json`** (NEW): Conditional-exports template with:
  - `"."` exports: `"import"` → `./bundler/sdivi_wasm.js`, `"require"` → `./node/sdivi_wasm.js`
  - `"./node"` subpath for explicit nodejs target
  - `"./bundler"` subpath for explicit bundler target
  - `"files"`: `["bundler/", "node/", "README.md", "LICENSE"]`
  - `"engines": {"node": ">=18"}`

- **`bindings/sdivi-wasm/tests/node_smoke/package.json`** (NEW): Minimal test project descriptor.

- **`bindings/sdivi-wasm/tests/node_smoke/index.cjs`** (NEW): CommonJS smoke test — `require('@geoffgodwin/sdivi-wasm')` triggers the `"require"` conditional → nodejs target; calls `list_categories()` (no `init()` needed for nodejs target); asserts schema_version, non-empty categories; emits `CJS_CATEGORIES:…` for cross-target comparison.

- **`bindings/sdivi-wasm/tests/node_smoke/index.mjs`** (NEW): ESM smoke test — `import from '@geoffgodwin/sdivi-wasm'` triggers the `"import"` conditional → bundler target; calls `await init()` then `list_categories()`; asserts schema_version, non-empty categories; emits `ESM_CATEGORIES:…` for cross-target comparison.

- **`.github/workflows/wasm.yml`** (MODIFIED): Extended to:
  - Build both bundler and nodejs targets with separate `wasm-pack build` steps.
  - Check per-target (1.2 MB) and combined (5 MB) bundle size budgets.
  - Assemble `pkg/package.json` from template after build.
  - Create `tests/node_smoke/node_modules/@geoffgodwin/sdivi-wasm` symlink pointing to `pkg/` (avoids publish/install round-trip while testing real conditional exports resolution).
  - Run CJS smoke test (`node index.cjs`).
  - Run ESM smoke test (`node index.mjs`).
  - Compare CJS and ESM `list_categories()` output to assert both targets return the same data.
  - Verify `npm pack --dry-run` lists both `bundler/` and `node/` subdirs.
  - Scoped to `ubuntu-latest` for the Node smoke steps (macOS runs build+size checks only).

- **`.github/workflows/release.yml`** (MODIFIED): npm publish job now builds both targets, assembles the conditional-exports `pkg/package.json`, and publishes from the restructured `pkg/` root.

- **`bindings/sdivi-wasm/README.md`** (MODIFIED): Added "Bundler consumers" and "Node.js consumers" sections with import snippets and notes on which target each uses; Node-18 minimum documented; updated "Building locally" section.

- **`CHANGELOG.md`** (MODIFIED): Added `[0.1.14]` entry under Added for all M24 deliverables.

- **`.tekhton/DESIGN.md`** (MODIFIED): Updated Distribution Model and Embedding Environments sections to reflect dual-target npm distribution shape.

- **`examples/binding_node.ts`** (MODIFIED): Updated comment to reference `./build.sh` instead of the old single-target command.

## Root Cause (bugs only)
N/A — this is a new feature milestone.

## Files Modified
- `bindings/sdivi-wasm/build.sh` — updated for dual targets (68 lines ✓)
- `bindings/sdivi-wasm/pkg-template/package.json` — NEW: conditional-exports template (39 lines ✓)
- `bindings/sdivi-wasm/tests/node_smoke/package.json` — NEW: smoke test project (12 lines ✓)
- `bindings/sdivi-wasm/tests/node_smoke/index.cjs` — NEW: CJS smoke test (27 lines ✓)
- `bindings/sdivi-wasm/tests/node_smoke/index.mjs` — NEW: ESM smoke test (27 lines ✓)
- `.github/workflows/wasm.yml` — extended for dual targets + Node smoke tests (213 lines ✓)
- `.github/workflows/release.yml` — updated npm publish for dual targets (284 lines ✓)
- `bindings/sdivi-wasm/README.md` — added dual-target usage docs (194 lines ✓)
- `CHANGELOG.md` — added [0.1.14] M24 entry
- `.tekhton/DESIGN.md` — updated distribution model sections
- `examples/binding_node.ts` — updated build command comment

## Human Notes Status
- Reviewer note: WasmCategoryInfo/WasmCategoryCatalog missing PartialEq — NOT_ADDRESSED (out of scope for M24)
- Reviewer note: list_categories() placement in exports.rs — NOT_ADDRESSED (out of scope for M24)
- Reviewer note: CATEGORIES/CATEGORY_DESCRIPTIONS parallel arrays — NOT_ADDRESSED (out of scope for M24)
- Tester report: rejects_negative_infinity_weight test added — COMPLETED (by tester in prior run)

## Docs Updated
- `bindings/sdivi-wasm/README.md` — dual-target usage sections (Bundler consumers + Node.js consumers)

## Observed Issues (out of scope)
- `bindings/sdivi-wasm/package.json` (the old single-target manifest at repo root) is now superseded by `pkg-template/package.json` but left in place — it's not published (only `pkg/package.json` assembled from the template is published). It may cause confusion; could be deleted or annotated in a future cleanup PR.
- `.claude/milestones/MANIFEST.cfg` M24 status left as `in_progress` — permission denied when attempting to update to `done`. The pipeline runner should update it.
