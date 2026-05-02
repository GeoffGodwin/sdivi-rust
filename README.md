# sdi-rust

**Structural Divergence Indexer** — a measurement instrument that tracks the
rate of structural drift in a codebase over time.

SDI captures the structural fingerprint of a repository (dependency graph,
community partition, pattern catalog) into deterministic JSON snapshots, and
reports four divergence metrics between consecutive snapshots:

| Dimension | What it tracks |
|---|---|
| **Pattern entropy rate** | How fast coding patterns are diverging |
| **Convention drift rate** | How fast style and idiom conventions shift |
| **Coupling delta rate** | How fast inter-module coupling changes |
| **Boundary violation rate** | How often code crosses declared module boundaries |

Threshold breaches are observations, not judgements. SDI never opines on code
quality — it produces numbers, and a CI gate (`sdi check`) decides what to do
with them.

sdi-rust ships as a Cargo workspace with a two-layer library shape:

- **`sdi-core`** — pure-compute facade, WASM-compatible, no I/O
- **`sdi-pipeline`** — FS orchestration (parsing, snapshot writes, retention)
- **`sdi-cli`** — the `sdi` binary

Languages: **Rust · Python · TypeScript · JavaScript · Go · Java**

---

## Install

### Pre-built binaries (recommended)

Linux / macOS / Windows builds are attached to every GitHub Release.

```sh
# Linux x86_64
curl -Lo sdi https://github.com/geoffgodwin/sdi-rust/releases/latest/download/sdi-linux-x86_64
chmod +x sdi && mv sdi ~/.local/bin/
sdi --version
```

### From crates.io

```sh
cargo install sdi-cli
sdi --version
```

### npm (WASM bundle)

```sh
npm install @geoffgodwin/sdi-wasm
```

The npm package exposes every `sdi-core::compute_*` function plus
`normalize_and_hash` to JS / TS callers, with `tsify`-derived `.d.ts`.

---

## Quick start

```sh
cd my-repo
sdi init                  # creates .sdi/ with default config + .gitignore
sdi snapshot              # writes the first snapshot under .sdi/snapshots/
# ...edit code, merge a PR...
sdi snapshot              # captures a new snapshot; first delta becomes meaningful
sdi check                 # runs the threshold gate (exit 10 on breach)
sdi trend --last 10       # slope across the last 10 snapshots
sdi show                  # pretty-prints the latest snapshot
```

The intended cadence is **once per merge to your primary branch**, plus ad-hoc
`sdi check` runs from a CI gate. There is no daemon and no watch mode.

---

## CLI reference

All commands accept `--repo <path>` (default: current directory). Every command
that prints structured output also accepts `--format text` (default) or
`--format json`. JSON output goes to **stdout**; logs and progress go to
**stderr**, so `sdi show --format json | jq '.'` always works.

| Command | Purpose |
|---|---|
| `sdi init` | Create `.sdi/` with a default `config.toml` and `.gitignore` |
| `sdi snapshot [--commit SHA] [--format ...]` | Capture and persist a snapshot under `.sdi/snapshots/` |
| `sdi diff <PREV> <CURR> [--format ...]` | Compare two snapshot files and print a `DivergenceSummary` |
| `sdi trend [--last N] [--format ...]` | Slope statistics across stored snapshots (oldest → newest) |
| `sdi check [--no-write] [--format ...]` | Snapshot, compare to prior, exit 10 if any threshold is breached |
| `sdi show [ID] [--format ...]` | Inspect a stored snapshot (defaults to latest); ID is the filename stem |
| `sdi catalog [--format ...]` | Build and display the pattern catalog only (no snapshot write) |
| `sdi boundaries infer [--format ...]` | Propose module groupings from Leiden community history |
| `sdi boundaries ratify` | Write accepted groupings to `.sdi/boundaries.yaml` |
| `sdi boundaries show` | Print the current boundary specification |

### Useful flags

- `sdi snapshot --commit <sha>` — record a snapshot at the given git ref instead
  of the working tree (M16). The git checkout is performed in a worktree so
  your working directory is untouched.
- `sdi check --no-write` — run the threshold gate without persisting a
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
| `10` | `sdi check` only — at least one threshold breached |

Adding or repurposing an exit code is a breaking change.

---

## Configuration

Config loads in this order (later wins):

1. CLI flags
2. Environment variables (`SDI_LOG_LEVEL`, `SDI_WORKERS`, `SDI_CONFIG_PATH`,
   `SDI_SNAPSHOT_DIR`, `NO_COLOR`)
3. Project `.sdi/config.toml`
4. Global `$XDG_CONFIG_HOME/sdi/config.toml`
5. Built-in defaults

All keys are optional. A representative `.sdi/config.toml`:

```toml
[core]
languages   = "auto"          # or ["rust", "python"]
exclude     = ["vendor/**", "node_modules/**", "target/**"]
random_seed = 42              # all RNG is StdRng seeded from this

[snapshots]
dir       = ".sdi/snapshots"
retention = 100               # 0 = unlimited

[boundaries]
spec_file           = ".sdi/boundaries.yaml"
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

A `.sdi/boundaries.yaml` file declares your intended module structure:

```yaml
boundaries:
  parsing:
    paths: ["crates/sdi-parsing/**"]
    allowed_imports: ["sdi-config", "tree-sitter"]
  detection:
    paths: ["crates/sdi-detection/**"]
    allowed_imports: ["sdi-graph", "petgraph"]
```

A missing `boundaries.yaml` is **normal operation**: every metric except
`boundary_violation_rate` is still computed; intent divergence is simply absent
from the snapshot.

**Workflow.** Run `sdi boundaries infer` after a few snapshots have been
captured — it proposes groupings derived from the Leiden community history
(stable across `stability_threshold` consecutive snapshots). Then
`sdi boundaries ratify` writes them. *Comments in `boundaries.yaml` are lost
when ratify rewrites the file* (KDD-6).

---

## CI integration

```yaml
# .github/workflows/sdi.yml
- uses: actions/checkout@v4
  with: { fetch-depth: 0 }       # change-coupling needs git history
- run: cargo install sdi-cli
- run: sdi check                 # exits 0 if healthy, 10 if thresholds exceeded
```

See [`docs/cli-integration.md`](docs/cli-integration.md) for the full recipe
(snapshot retention, artifact upload, comment-on-PR patterns).

---

## Embedding

### Rust — full pipeline (`sdi-pipeline`)

For Rust callers that want SDI to walk the filesystem, parse with tree-sitter,
and write snapshots:

```toml
[dependencies]
sdi-config   = "0.1"
sdi-pipeline = "0.1"
sdi-lang-rust = "0.1"   # plus any other languages you want
```

```rust
use sdi_config::Config;
use sdi_lang_rust::RustAdapter;
use sdi_pipeline::{Pipeline, current_timestamp};

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

### Rust — pure compute (`sdi-core`)

For Rust callers that already have their own extractors (e.g. agent runtimes,
gardener LLMs, the consumer app):

```toml
[dependencies]
sdi-core = "0.1"
```

`sdi-core` exposes `compute_*` functions over plain `serde` `*Input` structs:

```rust
use sdi_core::input::{DependencyGraphInput, NodeInput, EdgeInput, ThresholdsInput};
use sdi_core::{
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

`sdi-core` has **no I/O, no clock, no tree-sitter, no `std::fs`** — every
function that conceptually needs the wall clock takes a `chrono::NaiveDate` as
input (e.g. `compute_thresholds_check` for override-expiry resolution).

### WASM / JS / TS (`@geoffgodwin/sdi-wasm`)

```ts
import init, {
  compute_coupling_topology,
  compute_pattern_metrics,
  compute_thresholds_check,
  normalize_and_hash,
} from "@geoffgodwin/sdi-wasm";

await init();

const fp       = normalize_and_hash("function_item", []);
const coupling = compute_coupling_topology({ nodes, edges });
const check    = compute_thresholds_check(summary, thresholds);
```

Every input/output type has a `.d.ts` definition derived from `tsify`. The
WASM build pulls only `sdi-core`'s pure-compute path — no `walkdir`, `ignore`,
`rayon`, or `tree-sitter` in the bundle.

### Public surface

| Crate | Use it for | Stability |
|---|---|---|
| `sdi-core` | Pure-compute API; WASM target; foreign extractors | Stable, `#![deny(missing_docs)]` |
| `sdi-pipeline` | Full FS pipeline from Rust | Stable |
| `sdi-cli` | The `sdi` binary | Stable CLI surface; not intended as a library |
| `sdi-config` | `Config` loader and types | Stable |
| `sdi-lang-{rust,python,typescript,javascript,go,java}` | Language adapters | Stable |
| `sdi-graph`, `sdi-detection`, `sdi-patterns`, `sdi-snapshot`, `sdi-parsing` | Inner crates; depend on these only if you need a single stage | Stable, but most embedders should use `sdi-core` or `sdi-pipeline` instead |

Adding a `pub` item is deliberate; removing or renaming one is a breaking
change. SemVer commitment begins at `0.1.0`.

---

## Snapshot schema

Snapshots are JSON, tagged with `"snapshot_version": "1.0"`, written atomically
(tempfile + rename) under `.sdi/snapshots/`. Top-level fields:

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
  (`sdi_core::PATTERN_FINGERPRINT_KEY`); `normalize_and_hash` is the canonical
  entry point and produces the same digest in WASM as in native sdi-core for
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

Source: [`crates/sdi-cli/examples/`](crates/sdi-cli/examples/).

---

## Contributing

```sh
git clone https://github.com/geoffgodwin/sdi-rust
cd sdi-rust
cargo test --workspace          # full test suite
cargo clippy -- -D warnings     # CI-equivalent lint pass
cargo fmt --check
```

The MSRV is `stable - 2`, pinned in `rust-toolchain.toml`. WASM tests are
gated behind the `wasm.yml` CI workflow.

## License

Apache 2.0 — see [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).
