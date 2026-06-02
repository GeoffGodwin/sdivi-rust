/**
 * subpath_imports.ts — Typecheck fixture for the /bundler and /node subpaths.
 *
 * The tsconfig `paths` map declares three entries:
 *   "@geoffgodwin/sdivi-wasm"         → pkg/bundler/sdivi_wasm.d.ts
 *   "@geoffgodwin/sdivi-wasm/bundler" → pkg/bundler/sdivi_wasm.d.ts
 *   "@geoffgodwin/sdivi-wasm/node"    → pkg/node/sdivi_wasm.d.ts
 *
 * The main examples (binding_node.ts, binding_bundler.ts) only import via the
 * bare specifier. Without this file, type drift specific to a /bundler or /node
 * subpath condition would go undetected — the coverage gap flagged in the M47
 * reviewer report.
 *
 * This file imports at least one named export from each subpath and exercises
 * a Map-typed field to confirm that Map<…> constraint is enforced under both
 * subpath conditions. It must typecheck cleanly under all flags declared in
 * tsconfig.json (strict, noUncheckedIndexedAccess, exactOptionalPropertyTypes).
 *
 * IMPORTANT: pkg/ is gitignored and must be built before running tsc locally.
 * Run ./build.sh in bindings/sdivi-wasm/ first.
 */

// ── Bundler subpath ──────────────────────────────────────────────────────────

import type {
  WasmLeidenConfigInput,
  WasmBoundaryDetectionResult,
} from '@geoffgodwin/sdivi-wasm/bundler';

import { detect_boundaries as detect_bundler } from '@geoffgodwin/sdivi-wasm/bundler';

// Verify that edge_weights requires Map<string, number>, not a plain object.
// Using new Map([...]) — the correct shape. If the .d.ts ever loosens this
// to accept objects, this assignment would still compile but the negative
// fixture would catch the drift via an unused @ts-expect-error.
const bundlerCfg: WasmLeidenConfigInput = {
  seed: 42,
  gamma: 1.0,
  iterations: 100,
  quality: 'Modularity',
  edge_weights: new Map([['src/a.rs:src/b.rs', 0.9]]),
};

const graph = {
  nodes: [
    { id: 'src/a.rs', path: 'src/a.rs', language: 'rust' },
    { id: 'src/b.rs', path: 'src/b.rs', language: 'rust' },
  ],
  edges: [{ source: 'src/a.rs', target: 'src/b.rs' }],
};

// Call through the bundler subpath — exercises the export function signature.
const bundlerResult: WasmBoundaryDetectionResult = detect_bundler(graph, bundlerCfg, []);

// cluster_assignments is Map<string, number> — iterate via .entries(), not bracket index.
for (const [nodeId, communityId] of bundlerResult.cluster_assignments) {
  void nodeId;
  void communityId;
}

// ── Node subpath ─────────────────────────────────────────────────────────────

import type {
  WasmCouplingTopologyResult,
} from '@geoffgodwin/sdivi-wasm/node';

import { compute_coupling_topology as coupling_node } from '@geoffgodwin/sdivi-wasm/node';

// Call through the node subpath — verifies the /node subpath resolves correctly.
const nodeResult: WasmCouplingTopologyResult = coupling_node(graph);

// top_hubs is [string, number][] — read with .entries() or indexing into the array.
for (const [hubId, degree] of nodeResult.top_hubs) {
  void hubId;
  void degree;
}

// ── Anti-vacuous guard ───────────────────────────────────────────────────────
//
// If any path alias in tsconfig.json is wrong, tsc will fail here with
// "Cannot find module" rather than silently succeeding with zero files.
void bundlerResult;
void nodeResult;
void detect_bundler;
void coupling_node;
