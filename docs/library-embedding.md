# Library Embedding Guide

sdivi-rust exposes two embedding paths:

1. **Orchestration path** (`sdivi-pipeline`) — for Rust embedders that want the
   full pipeline: FS walking, tree-sitter parsing, snapshot writes.
2. **Pure-compute path** (`sdivi-core`) — for WASM/JS consumers or any embedder
   that supplies its own AST extractors and calls `compute_*` functions directly.

## Orchestration Path — `sdivi-pipeline`

### Add the dependency

```toml
[dependencies]
sdivi-pipeline = "0.0.14"
sdivi-config   = "0.0.14"
```

Add language adapters for the languages you want to analyse:

```toml
sdivi-lang-rust       = "0.0.14"
sdivi-lang-python     = "0.0.14"
sdivi-lang-typescript = "0.0.14"
```

### Minimal embedder

```rust
use sdivi_config::Config;
use sdivi_lang_rust::RustAdapter;
use sdivi_pipeline::{Pipeline, current_timestamp};
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
use sdivi_pipeline::{Pipeline, WriteMode, current_timestamp};
use sdivi_core::compute::thresholds::compute_thresholds_check;
use sdivi_core::input::ThresholdsInput;
use sdivi_pipeline::latest_snapshot;

fn threshold_gate(repo: &std::path::Path) -> anyhow::Result<bool> {
    let config = sdivi_config::Config::default();
    let pipeline = Pipeline::new(config.clone(), vec![]);
    let ts = current_timestamp();
    let curr = pipeline.snapshot_with_mode(repo, None, &ts, WriteMode::EphemeralForCheck)?;

    let snapshot_dir = repo.join(&config.snapshots.dir);
    let prev = latest_snapshot(&snapshot_dir).ok().flatten();
    let summary = sdivi_pipeline::Pipeline::delta(prev.as_ref(), &curr);

    let result = compute_thresholds_check(&summary, &ThresholdsInput::default());
    Ok(result.breached)
}
```

### Common pitfalls

- **No adapters, no records.** An empty adapter list produces a snapshot with
  zero nodes/edges — useful for integration tests, but not for real analysis.
  Wire up language adapters for the languages in your target repo.
- **Snapshot dir must exist.** `Pipeline::snapshot` writes to
  `.sdivi/snapshots/` relative to `repo_root`. Call `sdivi init` once (or create
  the directory manually) before embedding.
- **Config is consumed at construction.** Pass `Config::default()` or load via
  `sdivi_config::load_or_default(repo_root)`. The pipeline does not re-read the
  config during a run.

## Pure-Compute Path — `sdivi-core`

`sdivi-core` is WASM-compatible: no FS, no clock, no tree-sitter. Callers supply
pre-extracted data via `*Input` structs.

### Add the dependency

```toml
[dependencies]
sdivi-core = "0.0.14"
```

For WASM targets, `sdivi-core` compiles without modification — it has no
`std::fs::*`, `walkdir`, or `tree-sitter` in its dependency tree.

### Consumer-app pattern

This is the pattern used by Meridian, the first concrete consumer: the caller
has its own AST extractors, computes `normalize_and_hash` per pattern node,
and ships hashes + a dependency graph to `sdivi-core`.

```rust
use sdivi_core::{
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

### Supplying `today` and per-category threshold overrides

Meridian-style callers that read override config from their own sources can wire
`today` and `overrides` into `ThresholdsInput` before calling `compute_thresholds_check`.
The function is clock-free — the caller owns the date.

```rust
use chrono::NaiveDate;
use sdivi_core::{
    compute::thresholds::compute_thresholds_check,
    input::{ThresholdOverrideInput, ThresholdsInput},
    null_summary,
};
use std::collections::BTreeMap;

fn check_with_overrides(today: NaiveDate) {
    let mut overrides = BTreeMap::new();
    overrides.insert("error_handling".to_string(), ThresholdOverrideInput {
        pattern_entropy_rate: Some(5.0), // raised limit during migration
        convention_drift_rate: None,
        coupling_delta_rate: None,
        boundary_violation_rate: None,
        expires: "2026-09-30".to_string(),
    });

    let cfg = ThresholdsInput {
        today,
        overrides,
        ..ThresholdsInput::default()
    };

    // summary would come from compute_delta(prev, curr) in a real caller.
    let result = compute_thresholds_check(&null_summary(), &cfg);

    // applied_overrides shows which overrides were active at evaluation time.
    for (cat, info) in &result.applied_overrides {
        println!("{cat}: active={} expires={}", info.active, info.expires);
    }
}
```

### `normalize_and_hash` — canonical fingerprints for foreign extractors

`normalize_and_hash(kind, children)` produces the same `blake3` digest in WASM
as in native sdivi-core for the same input. Foreign extractors **must** use this
function to ensure fingerprints are byte-identical to those produced by the
native Rust pipeline. See [`docs/determinism.md`](determinism.md) for the
canonicalization rules.

### `NodeId` canonicalization

Node IDs must be repo-relative UNIX paths (forward-slash separated, no leading
`./`). Use `validate_node_id` to check before submitting to `compute_*`:

```rust
use sdivi_core::validate_node_id;

validate_node_id("src/lib.rs")?;   // ok
validate_node_id("./src/lib.rs")?; // error — leading ./
```

### WASM integration (TypeScript / JavaScript)

The `@geoffgodwin/sdivi-wasm` npm package exposes every `sdivi-core::compute_*`
function with TypeScript types derived from the Rust structs via `tsify`.

```typescript
import { compute_coupling_topology, DependencyGraphInput } from '@geoffgodwin/sdivi-wasm';

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

See the `bindings/sdivi-wasm/README.md` for the npm package setup and full API.

## Choosing Between the Two Paths

| | `sdivi-pipeline` | `sdivi-core` |
|---|---|---|
| Requires FS | Yes | No |
| Requires tree-sitter | Yes | No |
| WASM-compatible | No | Yes |
| Has its own parser | Yes | No |
| Snapshot writes | Yes | No |
| Use when | Rust CLI / CI tooling | WASM, agent runtimes, custom extractors |

## Historical commits and the pure-compute path

`sdivi snapshot --commit REF` is an `sdivi-pipeline` / CLI convenience. It shells
out to `git` to resolve the ref, extracts the tree, and feeds it through
`sdivi-pipeline::Pipeline::snapshot`. Embedders using the **pure-compute path**
(`sdivi-core`) do not need any of this machinery: they supply whatever source
tree they have in hand directly to `compute_*` functions via `*Input` structs.

For example, Meridian and other agent runtimes that maintain their own AST
extractor simply call `sdivi_core::detect_boundaries`, `compute_pattern_metrics`,
etc. with pre-extracted data. They own the tree-at-commit extraction
themselves (e.g. via the VSCode git index or their own `git archive` wrapper)
and never call `Pipeline::snapshot` at all.

## Computing change-coupling from a foreign extractor

If you have your own commit-history source (e.g., the VSCode git index in
Meridian), supply a `Vec<CoChangeEventInput>` directly to
`compute_change_coupling` without shelling out to `git log`:

```rust
use sdivi_core::{compute_change_coupling, CoChangeEventInput, ChangeCouplingConfigInput};

let events = vec![
    CoChangeEventInput {
        commit_sha: "abc123".to_string(),
        commit_date: "2026-05-01T00:00:00Z".to_string(),
        files: vec!["src/auth.rs".to_string(), "src/session.rs".to_string()],
    },
    // ... more events
];

let cfg = ChangeCouplingConfigInput { min_frequency: 0.6, history_depth: 500 };
let result = compute_change_coupling(&events, &cfg)?;
for pair in &result.pairs {
    println!("{} ↔ {} @ {:.0}%", pair.source, pair.target, pair.frequency * 100.0);
}
```

The same function is exported from `@geoffgodwin/sdivi-wasm` as
`compute_change_coupling`, making it callable from TypeScript consumers.

## Weighted Leiden — `edge_weight_key`

`LeidenConfigInput::edge_weights` accepts optional per-edge weights for
weighted Leiden. Because `serde_json` cannot serialize tuple-keyed maps, the
field uses `BTreeMap<String, f64>` with NUL-delimited string keys. Use the
`edge_weight_key` helper to construct keys:

```rust
use sdivi_core::input::{edge_weight_key, split_edge_weight_key, LeidenConfigInput};
use std::collections::BTreeMap;

let mut weights = BTreeMap::new();
// Keys must be (source, target) with source < target lexicographically.
weights.insert(edge_weight_key("src/auth.rs", "src/session.rs"), 3.0);
weights.insert(edge_weight_key("src/auth.rs", "src/models.rs"), 1.5);

let cfg = LeidenConfigInput {
    edge_weights: Some(weights),
    ..LeidenConfigInput::default()
};
```

`split_edge_weight_key` inverts the encoding: it splits a key back into
`(source, target)` — useful when iterating the map for display or export.
