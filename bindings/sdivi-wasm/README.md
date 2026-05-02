# @geoffgodwin/sdivi-wasm

WASM bindings for the [Structural Divergence Indexer](https://github.com/geoffgodwin/sdivi-rust) pure-compute API.

All `compute_*` functions from `sdivi-core` are exported with `tsify`-derived TypeScript types.
The package is **WASM-only** — no Node.js native code, no FS, no network.

## Installation

```bash
npm install @geoffgodwin/sdivi-wasm
```

## Usage

```ts
import init, {
  detect_boundaries,
  compute_coupling_topology,
  compute_pattern_metrics,
  normalize_and_hash,
} from '@geoffgodwin/sdivi-wasm';

// Must await init() once before calling any other function.
await init();

const graph = {
  nodes: [
    { id: 'src/lib.rs', path: 'src/lib.rs', language: 'rust' },
    { id: 'src/models.rs', path: 'src/models.rs', language: 'rust' },
  ],
  edges: [{ source: 'src/lib.rs', target: 'src/models.rs' }],
};

const metrics = compute_coupling_topology(graph);
console.log(metrics.density); // graph density

const cfg = { seed: 42, gamma: 1.0, iterations: 100, quality: 'Modularity' };
const boundaries = detect_boundaries(graph, cfg, []);
console.log(boundaries.cluster_assignments);

// normalize_and_hash produces the same blake3 digest as the native Rust pipeline.
const hash = normalize_and_hash('try_expression', []);
console.log(hash); // 64-char lowercase hex
```

## Exports

| Function | Description |
|---|---|
| `compute_coupling_topology(graph)` | Graph density, cycle count, hub nodes |
| `detect_boundaries(graph, cfg, prior)` | Leiden community detection |
| `compute_boundary_violations(graph, spec)` | Cross-boundary dependency check |
| `compute_pattern_metrics(patterns)` | Shannon entropy + convention drift |
| `compute_thresholds_check(summary, cfg)` | CI gate — returns breach list |
| `compute_delta(prev, curr)` | Per-dimension divergence between snapshots |
| `compute_trend(snapshots, last_n?)` | Trend slope over snapshot history |
| `infer_boundaries(prior_partitions, stability_threshold)` | Propose boundaries from history |
| `assemble_snapshot(input)` | Build a Snapshot JSON from compute outputs |
| `normalize_and_hash(node_kind, children)` | Canonical blake3 fingerprint |

## TypeScript guarantees

- All input/output types are derived via `tsify` — the `.d.ts` is generated at build time, not hand-maintained.
- Compatible with `--strict --noUncheckedIndexedAccess --exactOptionalPropertyTypes`.
- Optional fields are `T | undefined`, never `T | null`.

## `normalize_and_hash` determinism

The blake3 digest produced in WASM is **bit-identical** to the digest produced by the native Rust pipeline for the same input. The key constant (`FINGERPRINT_KEY`) is fixed across all `snapshot_version: "1.0"` output. See `docs/determinism.md` for the full contract.

## Building locally

```bash
# Prerequisites: cargo install wasm-pack && apt install binaryen
./build.sh          # release build + wasm-opt
./build.sh --dev    # dev build (no optimisation)
```

Output is in `pkg/`.

## Version note

`tsify` is pre-1.0. The version is pinned in `Cargo.toml` workspace deps; see `DRIFT_LOG.md` for the watch entry.
