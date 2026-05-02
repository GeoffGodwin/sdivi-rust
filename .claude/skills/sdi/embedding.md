# Embedding `sdi-core`

The `sdi-core` crate is the canonical surface (KDD-3 / KD12). Every CLI feature
is a library feature first. Embed `sdi-core` directly when building agent
runtimes, gardener tools, or custom analysis pipelines — do not shell out to
the `sdi` binary from in-process code.

`sdi-core` is SemVer-stable from `0.1.0` onward.

## The 80% case — one call

```rust
use sdi_core::{Config, Pipeline};

let cfg = Config::load_or_default(repo_root)?;
let snapshot = Pipeline::new(&cfg).snapshot(repo_root)?;
println!("{}", serde_json::to_string_pretty(&snapshot)?);
```

`Pipeline::new(&Config)` is a cheap constructor; it holds references to the
config and parses nothing yet. `Pipeline::snapshot(&self, &Path)` runs all five
stages.

## Computing a delta

```rust
let prev: Snapshot = serde_json::from_reader(File::open(prev_path)?)?;
let curr: Snapshot = serde_json::from_reader(File::open(curr_path)?)?;
let delta = Pipeline::new(&cfg).delta(&prev, &curr);
```

`Pipeline::delta` is **referentially transparent** — same inputs ⇒ same
`DivergenceSummary`, no I/O, no clock, no globals.

**First-snapshot semantics:** `delta` requires both arguments. There is no
"compare against nothing." Callers that have no prior snapshot should construct
a `DivergenceSummary` with all per-dimension fields `null` themselves, or
simply skip the delta step. `null` means "no prior snapshot." `0` means
"compared and unchanged." Mixing them is a semantic bug.

## Top-level types (all `serde::Serialize + Deserialize`)

| Type                   | Role                                                              |
|------------------------|-------------------------------------------------------------------|
| `Config`               | Resolved configuration; consumed by `Pipeline::new`               |
| `Pipeline`             | Stage orchestrator; cheap to construct                            |
| `Snapshot`             | Carries `snapshot_version: "1.0"`; never hand-edit JSON           |
| `BoundarySpec`         | Read-only; deserialize from `.sdi/boundaries.yaml`                |
| `DivergenceSummary`    | Result of `Pipeline::delta`                                       |
| `ExitCode`             | Closed enum (`Success=0, RuntimeError=1, ConfigError=2, AnalysisError=3, ThresholdExceeded=10`) |
| `FeatureRecord`        | Stage 1 output (per-file)                                         |
| `DependencyGraph`      | Stage 2 output                                                    |
| `LeidenPartition`      | Stage 3 output                                                    |
| `PatternCatalog`, `PatternFingerprint` | Stage 4 output                                    |

Embedders see exact-shape data, not opaque handles. Drive your own downstream
analysis off any stage's output.

## Lower-level entry points (also stable)

```rust
sdi_core::parsing::parse_repository(&Config, &Path) -> impl Iterator<Item = FeatureRecord>;
sdi_core::graph::build_dependency_graph(impl Iterator<Item = FeatureRecord>) -> DependencyGraph;
sdi_core::detection::detect_communities(&DependencyGraph, seed: u64, gamma: f64) -> LeidenPartition;
sdi_core::patterns::build_pattern_catalog(impl Iterator<Item = FeatureRecord>, &Config) -> PatternCatalog;
sdi_core::snapshot::assemble(&DependencyGraph, &LeidenPartition, &PatternCatalog, &BoundarySpec, &Config) -> Snapshot;
sdi_core::snapshot::compute_delta(prev: &Snapshot, curr: &Snapshot) -> DivergenceSummary;
```

Use these when you want to inject between stages — e.g. cache the graph,
substitute a partition, or fingerprint a subset.

## Errors

Library crates return `Result<T, E>` with `thiserror`-derived errors. Top-level
variants: `ConfigError`, `AnalysisError`, `IoError`. **`anyhow` is forbidden in
`sdi-core`** — it lives only in `sdi-cli/src/main.rs`. `panic!` is reserved for
"this should be impossible" invariants; recoverable failures return `Err`.

## Determinism contract for embedders

If you call `Pipeline::snapshot` twice against the same repo state with the
same `Config` (same `random_seed`), you get **bit-identical** `Snapshot` JSON.
Any divergence is a bug. To preserve that:

- Do not mutate `Config` between calls.
- Do not pass a `Config` whose `random_seed` was clock-derived.
- Do not iterate `HashMap`s and feed the order into snapshot fields — use
  `BTreeMap`.

Cross-platform bit-identity is **not** guaranteed (FMA float behavior, Open Q
#10). Aggregate equality is.

## Memory invariant

Tree-sitter CSTs are dropped before `parse_file` returns. Memory usage is
proportional to the largest single source file plus the dependency graph
footprint — not the total codebase. Do not hold any type containing a
`tree_sitter::Tree` across files.

## What not to do

- **Don't shell out to `sdi`** from a Rust process — embed `sdi-core` directly.
- **Don't read `.sdi/snapshots/*.json` with hand-written parsing** — go through
  `serde_json` + `Snapshot`.
- **Don't make network calls** in any extension, hook, or downstream stage you
  add. The pipeline contract forbids it (Rule 13).
- **Don't introduce ML/LLM calls** in the analysis path. SDI is a measurement
  instrument; determinism is the contract.
- **Don't import sdi-py snapshots.** sdi-rust uses `snapshot_version: "1.0"`
  and refuses sdi-py JSON (KDD-1 / KD13).

## Bindings (post-MVP)

Python (`sdi-py`, PyO3) and Node (`sdi-node`, napi-rs) re-expose the same
surface: `sdi.Pipeline(cfg).snapshot(path)`. Bindings ship after `sdi-core` is
feature-stable (≥ m04). Until then, embed `sdi-core` from Rust.
