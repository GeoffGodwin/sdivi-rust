# Library Embedding Guide

sdi-rust exposes two embedding paths:

1. **Orchestration path** (`sdi-pipeline`) — for Rust embedders that want the
   full pipeline: FS walking, tree-sitter parsing, snapshot writes.
2. **Pure-compute path** (`sdi-core`) — for WASM/JS consumers or any embedder
   that supplies its own AST extractors and calls `compute_*` functions directly.

## Orchestration Path — `sdi-pipeline`

### Add the dependency

```toml
[dependencies]
sdi-pipeline = "0.0.14"
sdi-config   = "0.0.14"
```

Add language adapters for the languages you want to analyse:

```toml
sdi-lang-rust       = "0.0.14"
sdi-lang-python     = "0.0.14"
sdi-lang-typescript = "0.0.14"
```

### Minimal embedder

```rust
use sdi_config::Config;
use sdi_lang_rust::RustAdapter;
use sdi_pipeline::{Pipeline, current_timestamp};
use std::path::Path;

fn analyse(repo: &Path) -> anyhow::Result<()> {
    let config = Config::default();
    let pipeline = Pipeline::new(config, vec![Box::new(RustAdapter)]);
    let ts = current_timestamp();
    let snapshot = pipeline.snapshot(repo, None, &ts)?;

    println!("nodes={} communities={} entropy={:.2}",
        snapshot.graph.node_count,
        snapshot.partition.community_count(),
        snapshot.pattern_metrics.total_entropy,
    );
    Ok(())
}
```

### Ephemeral check (no snapshot write)

```rust
use sdi_pipeline::{Pipeline, WriteMode, current_timestamp};
use sdi_core::compute::thresholds::compute_thresholds_check;
use sdi_core::input::ThresholdsInput;
use sdi_pipeline::latest_snapshot;

fn threshold_gate(repo: &std::path::Path) -> anyhow::Result<bool> {
    let config = sdi_config::Config::default();
    let pipeline = Pipeline::new(config.clone(), vec![]);
    let ts = current_timestamp();
    let curr = pipeline.snapshot_with_mode(repo, None, &ts, WriteMode::EphemeralForCheck)?;

    let snapshot_dir = repo.join(&config.snapshots.dir);
    let prev = latest_snapshot(&snapshot_dir).ok().flatten();
    let summary = sdi_pipeline::Pipeline::delta(prev.as_ref(), &curr);

    let result = compute_thresholds_check(&summary, &ThresholdsInput::default());
    Ok(result.breached)
}
```

### Common pitfalls

- **No adapters, no records.** An empty adapter list produces a snapshot with
  zero nodes/edges — useful for integration tests, but not for real analysis.
  Wire up language adapters for the languages in your target repo.
- **Snapshot dir must exist.** `Pipeline::snapshot` writes to
  `.sdi/snapshots/` relative to `repo_root`. Call `sdi init` once (or create
  the directory manually) before embedding.
- **Config is consumed at construction.** Pass `Config::default()` or load via
  `sdi_config::load_or_default(repo_root)`. The pipeline does not re-read the
  config during a run.

## Pure-Compute Path — `sdi-core`

`sdi-core` is WASM-compatible: no FS, no clock, no tree-sitter. Callers supply
pre-extracted data via `*Input` structs.

### Add the dependency

```toml
[dependencies]
sdi-core = "0.0.14"
```

For WASM targets, `sdi-core` compiles without modification — it has no
`std::fs::*`, `walkdir`, or `tree-sitter` in its dependency tree.

### Consumer-app pattern

This is the pattern used by Meridian, the first concrete consumer: the caller
has its own AST extractors, computes `normalize_and_hash` per pattern node,
and ships hashes + a dependency graph to `sdi-core`.

```rust
use sdi_core::{
    compute::coupling::compute_coupling_topology,
    compute::patterns::compute_pattern_metrics,
    compute::thresholds::compute_thresholds_check,
    input::{DependencyGraphInput, EdgeInput, NodeInput, PatternInstanceInput, ThresholdsInput},
    normalize_and_hash,
    null_summary,
};

fn analyse_from_extractor(
    nodes: Vec<(String, String)>,   // (id, language) pairs
    edges: Vec<(String, String)>,   // (source_id, target_id) pairs
    patterns: Vec<(String, String)>, // (kind, category) pairs per AST node
) -> anyhow::Result<()> {
    // Build DependencyGraphInput from caller-supplied data.
    let graph_input = DependencyGraphInput {
        nodes: nodes.iter().map(|(id, lang)| NodeInput {
            id: id.clone(),
            path: id.clone(),
            language: lang.clone(),
        }).collect(),
        edges: edges.iter().map(|(src, tgt)| EdgeInput {
            source: src.clone(),
            target: tgt.clone(),
        }).collect(),
    };

    let coupling = compute_coupling_topology(&graph_input)?;

    // Compute fingerprints for pattern instances using the same algorithm as
    // the native Rust pipeline (blake3 with FINGERPRINT_KEY).
    let pattern_instances: Vec<PatternInstanceInput> = patterns.iter().map(|(kind, cat)| {
        PatternInstanceInput {
            fingerprint: normalize_and_hash(kind, &[]),
            category: cat.clone(),
            node_id: "src/lib.rs".to_string(),
            location: None,
        }
    }).collect();

    let metrics = compute_pattern_metrics(&pattern_instances);
    let check = compute_thresholds_check(&null_summary(), &ThresholdsInput::default());

    println!("nodes={} density={:.4} entropy={:.4} breached={}",
        coupling.node_count, coupling.density,
        metrics.total_entropy, check.breached,
    );
    Ok(())
}
```

### `normalize_and_hash` — canonical fingerprints for foreign extractors

`normalize_and_hash(kind, children)` produces the same `blake3` digest in WASM
as in native sdi-core for the same input. Foreign extractors **must** use this
function to ensure fingerprints are byte-identical to those produced by the
native Rust pipeline. See [`docs/determinism.md`](determinism.md) for the
canonicalization rules.

### `NodeId` canonicalization

Node IDs must be repo-relative UNIX paths (forward-slash separated, no leading
`./`). Use `validate_node_id` to check before submitting to `compute_*`:

```rust
use sdi_core::validate_node_id;

validate_node_id("src/lib.rs")?;   // ok
validate_node_id("./src/lib.rs")?; // error — leading ./
```

### WASM integration (TypeScript / JavaScript)

The `@geoffgodwin/sdi-wasm` npm package exposes every `sdi-core::compute_*`
function with TypeScript types derived from the Rust structs via `tsify`.

```typescript
import { compute_coupling_topology, DependencyGraphInput } from '@geoffgodwin/sdi-wasm';

const graph: DependencyGraphInput = {
    nodes: [
        { id: 'src/lib.ts', path: 'src/lib.ts', language: 'typescript' },
        { id: 'src/models.ts', path: 'src/models.ts', language: 'typescript' },
    ],
    edges: [{ source: 'src/lib.ts', target: 'src/models.ts' }],
};

const result = compute_coupling_topology(graph);
console.log(result.node_count, result.density);
```

See the `bindings/sdi-wasm/README.md` for the npm package setup and full API.

## Choosing Between the Two Paths

| | `sdi-pipeline` | `sdi-core` |
|---|---|---|
| Requires FS | Yes | No |
| Requires tree-sitter | Yes | No |
| WASM-compatible | No | Yes |
| Has its own parser | Yes | No |
| Snapshot writes | Yes | No |
| Use when | Rust CLI / CI tooling | WASM, agent runtimes, custom extractors |
