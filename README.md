# sdivi-rust

**Structural Divergence Indexer** — a measurement instrument that tracks the
rate of structural drift in a codebase over time.

SDIVI captures the structural fingerprint of a repository (dependency graph,
community partition, pattern catalog) into deterministic JSON snapshots, and
reports four divergence metrics between consecutive snapshots:

| Dimension | What it tracks |
|---|---|
| **Pattern entropy rate** | How fast coding patterns are diverging |
| **Convention drift rate** | How fast style and idiom conventions shift |
| **Coupling delta rate** | How fast inter-module coupling changes |
| **Boundary violation rate** | How often code crosses declared module boundaries |

Threshold breaches are observations, not judgements. SDIVI never opines on code
quality — it produces numbers, and a CI gate (`sdivi check`) decides what to do
with them.

sdivi-rust ships as a Cargo workspace with a two-layer library shape:

- **`sdivi-core`** — pure-compute facade, WASM-compatible, no I/O
- **`sdivi-pipeline`** — FS orchestration (parsing, snapshot writes, retention)
- **`sdivi-cli`** — the `sdivi` binary

Languages: **Rust · Python · TypeScript · JavaScript · Go · Java**

---

## Install

### Pre-built binaries (recommended)

Linux / macOS / Windows builds are attached to every GitHub Release.

```sh
# Linux x86_64
curl -Lo sdivi https://github.com/geoffgodwin/sdivi-rust/releases/latest/download/sdivi-linux-x86_64
chmod +x sdivi && mv sdivi ~/.local/bin/
sdivi --version
```

### From crates.io

```sh
cargo install sdivi-cli
sdivi --version
```

### npm (WASM bundle)

```sh
npm install @geoffgodwin/sdivi-wasm
```

The npm package exposes every `sdivi-core::compute_*` function plus
`normalize_and_hash` to JS / TS callers, with `tsify`-derived `.d.ts`.

---

## Quick start

```sh
cd my-repo
sdivi init                  # creates .sdivi/ with default config + .gitignore
sdivi snapshot              # writes the first snapshot under .sdivi/snapshots/
# ...edit code, merge a PR...
sdivi snapshot              # captures a new snapshot; first delta becomes meaningful
sdivi check                 # runs the threshold gate (exit 10 on breach)
sdivi trend --last 10       # slope across the last 10 snapshots
sdivi show                  # pretty-prints the latest snapshot
```

The intended cadence is **once per merge to your primary branch**, plus ad-hoc
`sdivi check` runs from a CI gate. There is no daemon and no watch mode.

---

## CLI reference

All commands accept `--repo <path>` (default: current directory). Every command
that prints structured output also accepts `--format text` (default) or
`--format json`. JSON output goes to **stdout**; logs and progress go to
**stderr**, so `sdivi show --format json | jq '.'` always works.

| Command | Purpose |
|---|---|
| `sdivi init` | Create `.sdivi/` with a default `config.toml` and `.gitignore` |
| `sdivi snapshot [--commit SHA] [--format ...]` | Capture and persist a snapshot under `.sdivi/snapshots/` |
| `sdivi diff <PREV> <CURR> [--format ...]` | Compare two snapshot files and print a `DivergenceSummary` |
| `sdivi trend [--last N] [--format ...]` | Slope statistics across stored snapshots (oldest → newest) |
| `sdivi check [--no-write] [--format ...]` | Snapshot, compare to prior, exit 10 if any threshold is breached |
| `sdivi show [ID] [--format ...]` | Inspect a stored snapshot (defaults to latest); ID is the filename stem |
| `sdivi catalog [--format ...]` | Build and display the pattern catalog only (no snapshot write) |
| `sdivi boundaries infer [--format ...]` | Propose module groupings from Leiden community history |
| `sdivi boundaries ratify` | Write accepted groupings to `.sdivi/boundaries.yaml` |
| `sdivi boundaries show` | Print the current boundary specification |

### Useful flags

- `sdivi snapshot --commit <sha>` — record a snapshot at the given git ref instead
  of the working tree (M16). The git checkout is performed in a worktree so
  your working directory is untouched.
- `sdivi check --no-write` — run the threshold gate without persisting a
  snapshot. Useful for ephemeral PR-preview checks.
- `--format json` — machine-readable output on stdout. Stable across patch
  versions within `snapshot_version: "1.0"`.

### Exit codes (public API — Rule 9)

| Code | Meaning |
|---|---|
| `0` | Success |
| `1` | Generic runtime error (I/O, parse, internal) |
| `2` | Configuration error (e.g. threshold override missing `expires`) |
| `3` | All detected languages lack tree-sitter grammars (no records produced) |
| `10` | `sdivi check` only — at least one threshold breached |

Adding or repurposing an exit code is a breaking change.

---

## Configuration

Config loads in this order (later wins):

1. CLI flags
2. Environment variables (`SDIVI_LOG_LEVEL`, `SDIVI_WORKERS`, `SDIVI_CONFIG_PATH`,
   `SDIVI_SNAPSHOT_DIR`, `NO_COLOR`)
3. Project `.sdivi/config.toml`
4. Global `$XDG_CONFIG_HOME/sdivi/config.toml`
5. Built-in defaults

All keys are optional. A representative `.sdivi/config.toml`:

```toml
[core]
languages   = "auto"          # or ["rust", "python"]
exclude     = ["vendor/**", "node_modules/**", "target/**"]
random_seed = 42              # all RNG is StdRng seeded from this

[snapshots]
dir       = ".sdivi/snapshots"
retention = 100               # 0 = unlimited

[boundaries]
spec_file           = ".sdivi/boundaries.yaml"
leiden_gamma        = 1.0
stability_threshold = 3
weighted_edges      = false   # set true to use change-coupling edge weights

[patterns]
categories        = "auto"
min_pattern_nodes = 5
scope_exclude     = []        # patterns excluded from catalog only — files remain in graph

[thresholds]
pattern_entropy_rate    = 2.0
convention_drift_rate   = 3.0
coupling_delta_rate     = 0.15
boundary_violation_rate = 2.0

[change_coupling]
min_frequency = 0.6           # M15 — co-change edge inclusion threshold
history_depth = 500           # M15 — number of recent commits considered

[output]
format = "text"               # or "json"
color  = "auto"               # or "always" / "never"
```

### Per-category threshold overrides (M14)

Declare a temporary override **with a mandatory `expires` date**:

```toml
[thresholds.overrides.error_handling]
pattern_entropy_rate = 5.0
expires              = "2026-09-30"
reason               = "Migrating to ? operator from match Err(_) chains"
```

After the expiry date the override is silently ignored and defaults resume —
no manual reset. A missing `expires` is a `ConfigError` (exit 2).

---

## Boundaries

A `.sdivi/boundaries.yaml` file declares your intended module structure:

```yaml
boundaries:
  parsing:
    paths: ["crates/sdivi-parsing/**"]
    allowed_imports: ["sdivi-config", "tree-sitter"]
  detection:
    paths: ["crates/sdivi-detection/**"]
    allowed_imports: ["sdivi-graph", "petgraph"]
```

A missing `boundaries.yaml` is **normal operation**: every metric except
`boundary_violation_rate` is still computed; intent divergence is simply absent
from the snapshot.

**Workflow.** Run `sdivi boundaries infer` after a few snapshots have been
captured — it proposes groupings derived from the Leiden community history
(stable across `stability_threshold` consecutive snapshots). Then
`sdivi boundaries ratify` writes them. *Comments in `boundaries.yaml` are lost
when ratify rewrites the file* (KDD-6).

---

## CI integration

```yaml
# .github/workflows/sdivi.yml
- uses: actions/checkout@v4
  with: { fetch-depth: 0 }       # change-coupling needs git history
- run: cargo install sdivi-cli
- run: sdivi check                 # exits 0 if healthy, 10 if thresholds exceeded
```

See [`docs/cli-integration.md`](docs/cli-integration.md) for the full recipe
(snapshot retention, artifact upload, comment-on-PR patterns).

---

## Embedding

### Rust — full pipeline (`sdivi-pipeline`)

For Rust callers that want SDIVI to walk the filesystem, parse with tree-sitter,
and write snapshots:

```toml
[dependencies]
sdivi-config   = "0.1"
sdivi-pipeline = "0.1"
sdivi-lang-rust = "0.1"   # plus any other languages you want
```

```rust
use sdivi_config::Config;
use sdivi_lang_rust::RustAdapter;
use sdivi_pipeline::{Pipeline, current_timestamp};

let config   = Config::default();
let pipeline = Pipeline::new(config, vec![Box::new(RustAdapter)]);
let ts       = current_timestamp();
let snapshot = pipeline.snapshot(repo_root, None, &ts)?;

println!(
    "nodes={} communities={} entropy={:.2}",
    snapshot.graph.node_count,
    snapshot.partition.community_count(),
    snapshot.pattern_metrics.total_entropy,
);
```

`Pipeline::snapshot_with_mode(repo, commit, ts, WriteMode::EphemeralForCheck)`
runs the pipeline without persisting a snapshot — useful for PR-preview gates.

### Rust — pure compute (`sdivi-core`)

For Rust callers that already have their own extractors (e.g. agent runtimes,
gardener LLMs, the consumer app):

```toml
[dependencies]
sdivi-core = "0.1"
```

`sdivi-core` exposes `compute_*` functions over plain `serde` `*Input` structs:

```rust
use sdivi_core::input::{DependencyGraphInput, NodeInput, EdgeInput, ThresholdsInput};
use sdivi_core::{
    compute_coupling_topology, compute_pattern_metrics, compute_thresholds_check,
    compute_delta, normalize_and_hash, null_summary,
};

let graph = DependencyGraphInput {
    nodes: vec![ /* your NodeInputs */ ],
    edges: vec![ /* your EdgeInputs */ ],
};
let coupling = compute_coupling_topology(&graph)?;

// Foreign extractors must use this for fingerprints to match the Rust pipeline.
let fp = normalize_and_hash("function_item", &[]);

let check = compute_thresholds_check(&null_summary(), &ThresholdsInput::default());
assert!(!check.breached);
```

`sdivi-core` has **no I/O, no clock, no tree-sitter, no `std::fs`** — every
function that conceptually needs the wall clock takes a `chrono::NaiveDate` as
input (e.g. `compute_thresholds_check` for override-expiry resolution).

### WASM / JS / TS (`@geoffgodwin/sdivi-wasm`)

```ts
import init, {
  compute_coupling_topology,
  compute_pattern_metrics,
  compute_thresholds_check,
  normalize_and_hash,
} from "@geoffgodwin/sdivi-wasm";

await init();

const fp       = normalize_and_hash("function_item", []);
const coupling = compute_coupling_topology({ nodes, edges });
const check    = compute_thresholds_check(summary, thresholds);
```

Every input/output type has a `.d.ts` definition derived from `tsify`. The
WASM build pulls only `sdivi-core`'s pure-compute path — no `walkdir`, `ignore`,
`rayon`, or `tree-sitter` in the bundle.

### Public surface

| Crate | Use it for | Stability |
|---|---|---|
| `sdivi-core` | Pure-compute API; WASM target; foreign extractors | Stable, `#![deny(missing_docs)]` |
| `sdivi-pipeline` | Full FS pipeline from Rust | Stable |
| `sdivi-cli` | The `sdivi` binary | Stable CLI surface; not intended as a library |
| `sdivi-config` | `Config` loader and types | Stable |
| `sdivi-lang-{rust,python,typescript,javascript,go,java}` | Language adapters | Stable |
| `sdivi-graph`, `sdivi-detection`, `sdivi-patterns`, `sdivi-snapshot`, `sdivi-parsing` | Inner crates; depend on these only if you need a single stage | Stable, but most embedders should use `sdivi-core` or `sdivi-pipeline` instead |

Adding a `pub` item is deliberate; removing or renaming one is a breaking
change. SemVer commitment begins at `0.1.0`.

---

## Snapshot schema

Snapshots are JSON, tagged with `"snapshot_version": "1.0"`, written atomically
(tempfile + rename) under `.sdivi/snapshots/`. Top-level fields:

```jsonc
{
  "snapshot_version": "1.0",
  "timestamp": "2026-05-01T12:34:56Z",
  "commit_sha": "abcdef…",         // optional
  "graph":            { /* node_count, edge_count, hub_nodes, ... */ },
  "partition":        { /* clusters, modularity, stability_score, ... */ },
  "catalog":          { /* per-category PatternStats with fingerprints */ },
  "pattern_metrics":  { /* total_entropy, convention_drift, ... */ },
  "boundary_violations": { /* present iff boundaries.yaml is defined */ },
  "intent_divergence":   { /* present iff boundaries.yaml is defined */ },
  "change_coupling":     { /* present iff git history available */ },
  "path_partition":      { "src/foo.rs": 0, "src/bar.rs": 1, ... }
}
```

Reading a snapshot with an incompatible `snapshot_version` produces a stderr
warning and baseline treatment (no delta) — never a crash.

---

## Determinism guarantees

- A `Pipeline::snapshot` against the same repo state with the same `Config`
  (same `random_seed`) produces a **bit-identical** Snapshot JSON.
- All RNG is `StdRng` seeded from `Config::random_seed` (default `42`).
  No `thread_rng`, no `SystemTime`-derived seeds.
- `BTreeMap` everywhere output ordering matters.
- Pattern fingerprints use `blake3` with a fixed key constant
  (`sdivi_core::PATTERN_FINGERPRINT_KEY`); `normalize_and_hash` is the canonical
  entry point and produces the same digest in WASM as in native sdivi-core for
  the same `NormalizeNode` input.
- First-snapshot deltas are `null` per dimension (not `0`). `0` means
  "compared and unchanged."
- Aggregate equality across platforms (Linux / macOS / Windows). Bit-identity
  across platforms is **not** guaranteed; see
  [`docs/determinism.md`](docs/determinism.md).

---

## Documentation

| Document | What it covers |
|---|---|
| [`docs/cli-integration.md`](docs/cli-integration.md) | CI integration, GitHub Actions snippet, exit codes |
| [`docs/library-embedding.md`](docs/library-embedding.md) | Rust + WASM embedding guide and pitfalls |
| [`docs/determinism.md`](docs/determinism.md) | `BTreeMap` discipline, seed contract, FMA notes |
| [`docs/migrating-from-sdi-py.md`](docs/migrating-from-sdi-py.md) | Migrating from the Python POC |
| [`CHANGELOG.md`](CHANGELOG.md) | Per-release changes |
| [`MIGRATION_NOTES.md`](MIGRATION_NOTES.md) | Breaking-change migration notes |

API reference (rustdoc) is published to docs.rs on every `cargo publish`.

---

## Examples

```sh
cargo run --example embed_pipeline    # full FS pipeline
cargo run --example embed_compute     # pure-compute path with parity check
cargo run --example custom_config     # programmatic Config building
```

Source: [`crates/sdivi-cli/examples/`](crates/sdivi-cli/examples/).

---

## Contributing

```sh
git clone https://github.com/geoffgodwin/sdivi-rust
cd sdivi-rust
cargo test --workspace          # full test suite
cargo clippy -- -D warnings     # CI-equivalent lint pass
cargo fmt --check
```

The MSRV is `stable - 2`, pinned in `rust-toolchain.toml`. WASM tests are
gated behind the `wasm.yml` CI workflow.

## License

Apache 2.0 — see [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).
