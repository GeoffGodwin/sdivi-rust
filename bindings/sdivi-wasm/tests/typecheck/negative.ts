/**
 * negative.ts — Self-verifying negative fixture for @geoffgodwin/sdivi-wasm.
 *
 * Every `// @ts-expect-error` directive below MUST be consumed by a genuine
 * TypeScript error. If any one of them becomes unused, `tsc` fails with
 * TS2578 "Unused '@ts-expect-error' directive" — that means the public
 * contract changed (an init() export was added, a Map input was loosened to
 * accept objects, etc.) and the docs/examples AND this fixture must be
 * revisited.
 *
 * Each case corresponds to a regression that shipped before M47 was merged.
 */

// ── Imports ──────────────────────────────────────────────────────────────────

// Default import (synthetic, via esModuleInterop) — the module namespace.
import init from '@geoffgodwin/sdivi-wasm';

// Named value and type imports used in cases 2–3.
import { detect_boundaries } from '@geoffgodwin/sdivi-wasm';
import type {
  WasmLeidenConfigInput,
  WasmThresholdsInput,
  WasmBoundaryDetectionResult,
} from '@geoffgodwin/sdivi-wasm';

// ── Case 1: await init() ──────────────────────────────────────────────────────
//
// The M12-era documented idiom was `import init, { … } from '…'; await init()`.
// The bundler+nodejs build exposes NO callable default — the module namespace
// synthesised by esModuleInterop is not a function.
// TS2349 "This expression is not callable": the module namespace (synthesised
// default) is not a function. See M47 DRIFT_LOG entry.
// @ts-expect-error
await init();

// ── Case 2a: edge_weights as a plain object (not a Map) ──────────────────────
//
// WasmLeidenConfigInput.edge_weights is `Map<string, number> | undefined`.
// A plain object literal is not assignable to Map — serde-wasm-bindgen rejects
// it at runtime with a deserialization error.
// TS2322: "Type '{ 'a:b': number }' is not assignable to type 'Map<string,
// number>'" — edge_weights must be `new Map([…])`.
// @ts-expect-error
const _badEdgeWeights: WasmLeidenConfigInput['edge_weights'] = { 'a:b': 0.8 };

// ── Case 2b: overrides as a plain object (not a Map) ─────────────────────────
//
// WasmThresholdsInput.overrides is `Map<string, WasmThresholdOverrideInput>`.
// Passing an object literal was another instance of the same class of bug.
// TS2322: "Type '{ error_handling: … }' is not assignable to type
// 'Map<string, WasmThresholdOverrideInput>'" — overrides needs new Map.
// @ts-expect-error
const _badOverrides: WasmThresholdsInput['overrides'] = { error_handling: { expires: '2026-12-31' } };

// ── Case 3: bracket-indexing a Map output ────────────────────────────────────
//
// cluster_assignments is `Map<string, number>`. Map has no index signature in
// TypeScript — use .get() / .entries() / for-of instead of bracket notation.
declare const _boundaries: WasmBoundaryDetectionResult;
// TS7053: "Element implicitly has an 'any' type because expression of type
// '"x"' can't be used to index type 'Map<string, number>'".
// @ts-expect-error
const _badBracket = _boundaries.cluster_assignments['x'];

// ── Guards against vacuous pass ───────────────────────────────────────────────
//
// The variables below are declared so tsc cannot tree-shake the negative cases.
// If any path typo in tsconfig.json caused zero files to be included, the
// @ts-expect-error directives above would become unused and tsc would fail.
void _badEdgeWeights;
void _badOverrides;
void _badBracket;
void detect_boundaries;
