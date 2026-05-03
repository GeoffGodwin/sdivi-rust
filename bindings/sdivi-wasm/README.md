# @geoffgodwin/sdivi-wasm

WASM bindings for the [Structural Divergence Indexer](https://github.com/geoffgodwin/sdivi-rust) pure-compute API.

All `compute_*` functions from `sdivi-core` are exported with `tsify`-derived TypeScript types.
The package is **WASM-only** — no Node.js native code, no FS access beyond loading the `.wasm` file.

**Node.js minimum: 18.** The conditional `exports` map is ignored by Node < 18, which falls back
to the `main` field (bundler build) — use the explicit `/node` subpath on older runtimes.

## Installation

```bash
npm install @geoffgodwin/sdivi-wasm
```

## Bundler consumers (webpack, vite, rollup)

Use the default import path. The bundler target uses `import.meta.url`-style WASM loading
and requires a bundler to resolve the `.wasm` asset.

```ts
import init, {
  detect_boundaries,
  compute_coupling_topology,
  list_categories,
} from '@geoffgodwin/sdivi-wasm';

// Must await init() once before calling any other function.
await init();

const catalog = list_categories();
console.log(catalog.schema_version); // "1.0"

const metrics = compute_coupling_topology(graph);
console.log(metrics.density);
```

Or be explicit with the `/bundler` subpath:

```ts
import init, { list_categories } from '@geoffgodwin/sdivi-wasm/bundler';
await init();
```

## Node.js consumers (CLI, server)

Use `require('@geoffgodwin/sdivi-wasm')` or the explicit `/node` subpath. The nodejs target
uses `require('fs')` to load the `.wasm` file **synchronously** — no `init()` call needed,
and no bundler required.

```js
// CommonJS — no init() needed; wasm loads synchronously.
const { list_categories, compute_coupling_topology } = require('@geoffgodwin/sdivi-wasm');

const catalog = list_categories();
console.log(catalog.schema_version); // "1.0"
for (const cat of catalog.categories) {
  console.log(cat.name); // "async_patterns", "error_handling", ...
}
```

```ts
// TypeScript with explicit subpath (for TS moduleResolution: "bundler" or "node16"):
const { list_categories } = require('@geoffgodwin/sdivi-wasm/node');
```

**Important:** Do NOT use the `/node` build with webpack/vite. It uses `require('fs')` which
bundlers cannot resolve in a browser context.

## Full usage example (bundler)

```ts
import init, {
  detect_boundaries,
  compute_coupling_topology,
  compute_pattern_metrics,
  normalize_and_hash,
  compute_change_coupling,
  assemble_snapshot,
  list_categories,
} from '@geoffgodwin/sdivi-wasm';

await init();

const graph = {
  nodes: [
    { id: 'src/lib.rs', path: 'src/lib.rs', language: 'rust' },
    { id: 'src/models.rs', path: 'src/models.rs', language: 'rust' },
  ],
  edges: [{ source: 'src/lib.rs', target: 'src/models.rs' }],
};

const metrics = compute_coupling_topology(graph);
console.log(metrics.density);

// Unweighted Leiden.
const cfg = { seed: 42, gamma: 1.0, iterations: 100, quality: 'Modularity' };
const boundaries = detect_boundaries(graph, cfg, []);
console.log(boundaries.cluster_assignments);

// Weighted Leiden — pass co-change frequencies as edge weights.
// Keys are "source:target" strings; the first colon splits source from target,
// so node IDs that contain colons (e.g. "crates/foo:bar.rs") are supported.
// Weights must be >= 0 and finite; edges absent from the graph are ignored.
const weightedCfg = {
  ...cfg,
  edge_weights: { 'src/lib.rs:src/models.rs': 0.8, 'src/lib.rs:src/errors.rs': 0.5 },
};
const weightedBoundaries = detect_boundaries(graph, weightedCfg, []);
console.log(weightedBoundaries.cluster_assignments);

// normalize_and_hash produces the same blake3 digest as the native Rust pipeline.
const hash = normalize_and_hash('try_expression', []);
console.log(hash); // 64-char lowercase hex

// Change-coupling round-trip: compute and include in snapshot.
const events = [
  { commit_sha: 'abc', commit_date: '2026-01-01T00:00:00Z', files: ['src/a.ts', 'src/b.ts'] },
  { commit_sha: 'def', commit_date: '2026-01-02T00:00:00Z', files: ['src/a.ts', 'src/b.ts'] },
];
const changeCoupling = compute_change_coupling(events, { min_frequency: 0.5, history_depth: 100 });
const snapshot = assemble_snapshot({
  // ... graph / partition / pattern fields ...
  timestamp: new Date().toISOString(),
  change_coupling: changeCoupling,
});
```

## Exports

| Function | Description |
|---|---|
| `compute_coupling_topology(graph)` | Graph density, cycle count, hub nodes |
| `detect_boundaries(graph, cfg, prior)` | Leiden community detection |
| `compute_boundary_violations(graph, spec)` | Cross-boundary dependency check |
| `compute_pattern_metrics(patterns)` | Shannon entropy + convention drift |
| `compute_change_coupling(events, cfg)` | File-pair co-change frequencies from git history |
| `compute_thresholds_check(summary, cfg)` | CI gate — returns breach list |
| `compute_delta(prev, curr)` | Per-dimension divergence between snapshots |
| `compute_trend(snapshots, last_n?)` | Trend slope over snapshot history |
| `infer_boundaries(prior_partitions, stability_threshold)` | Propose boundaries from history |
| `assemble_snapshot(input)` | Build a Snapshot JSON from compute outputs |
| `normalize_and_hash(node_kind, children)` | Canonical blake3 fingerprint |
| `list_categories()` | Canonical pattern-category contract (`schema_version "1.0"`) |

## Pattern category discovery

Embedders that supply their own tree-sitter extractors must use the exact category names
returned by `list_categories()`. The comparison in `compute_pattern_metrics` is case-sensitive.

```ts
import init, { list_categories } from '@geoffgodwin/sdivi-wasm';

await init();
const catalog = list_categories();
console.log(catalog.schema_version); // "1.0"
for (const cat of catalog.categories) {
    console.log(cat.name); // "async_patterns", "error_handling", ...
}
```

See [`docs/pattern-categories.md`](../../docs/pattern-categories.md) for the full contract
including per-language node-kind tables and normalization rules.

> **WASM API parity reached (M22):** `assemble_snapshot` now accepts an optional
> `change_coupling` field. With M21 (weighted Leiden) and M22 (change coupling) both
> shipped, the WASM surface fully matches the native pipeline's `assemble_snapshot`
> capabilities. Consumers can build a complete snapshot in WASM without any gaps.

## TypeScript guarantees

- All input/output types are derived via `tsify` — the `.d.ts` is generated at build time, not hand-maintained.
- Compatible with `--strict --noUncheckedIndexedAccess --exactOptionalPropertyTypes`.
- Optional fields are `T | undefined`, never `T | null`.
- The `types` field in `exports["."]` points to the bundler `.d.ts`; `exports["./node"]` points to the node `.d.ts`.

## `normalize_and_hash` determinism

The blake3 digest produced in WASM is **bit-identical** to the digest produced by the native Rust pipeline for the same input. The key constant (`FINGERPRINT_KEY`) is fixed across all `snapshot_version: "1.0"` output. See `docs/determinism.md` for the full contract.

## Building locally

```bash
# Prerequisites: cargo install wasm-pack && apt install binaryen
./build.sh          # release build: both bundler + nodejs targets + wasm-opt
./build.sh --dev    # dev build: both targets, no optimisation
```

Output is in `pkg/bundler/` and `pkg/node/`, with `pkg/package.json` assembled from `pkg-template/`.

## Version note

`tsify` is pre-1.0. The version is pinned in `Cargo.toml` workspace deps; see `DRIFT_LOG.md` for the watch entry.
