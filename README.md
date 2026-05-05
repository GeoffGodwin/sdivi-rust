# sdivi-rust

**Structural Divergence Indexer.** Pronounced *Stevie*. A measurement
instrument that tracks how fast the structure of a codebase is changing.

SDIVI captures the structural fingerprint of a repository (dependency graph,
community partition, pattern catalog) into deterministic JSON snapshots, then
reports four divergence metrics between consecutive snapshots:

| Dimension | What it tracks |
|---|---|
| **Pattern entropy rate** | How fast coding patterns are diverging |
| **Convention drift rate** | How fast style and idiom conventions shift |
| **Coupling delta rate** | How fast inter-module coupling changes |
| **Boundary violation rate** | How often code crosses declared module boundaries |

Threshold breaches are observations. SDIVI never opines on code quality. It
produces numbers, and a CI gate (`sdivi check`) decides what to do with them.

sdivi-rust ships as a Cargo workspace with a two-layer library shape:

- **`sdivi-core`**. Pure-compute facade. WASM-compatible. No I/O.
- **`sdivi-pipeline`**. FS orchestration (parsing, snapshot writes, retention).
- **`sdivi-cli`**. The `sdivi` binary.

Languages: **Rust, Python, TypeScript, JavaScript, Go, Java**.

**Documentation site:** <https://geoffgodwin.github.io/sdivi-rust/>

---

## Install

### Pre-built binaries (recommended)

Linux, macOS, and Windows builds are attached to every GitHub Release.

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
`normalize_and_hash`, with `tsify`-derived `.d.ts` for strict-TS callers.

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

The intended cadence is once per merge to your primary branch, plus ad-hoc
`sdivi check` runs from a CI gate. There is no daemon and no watch mode.

---

## CLI reference

All commands accept `--repo <path>` (default: current directory). Every command
that prints structured output also accepts `--format text` (default) or
`--format json`. JSON output goes to **stdout**. Logs and progress go to
**stderr**, so `sdivi show --format json | jq '.'` always works.

| Command | Purpose |
|---|---|
| `sdivi init` | Create `.sdivi/` with a default `config.toml` and `.gitignore` |
| `sdivi snapshot [--commit SHA]` | Capture and persist a snapshot under `.sdivi/snapshots/` |
| `sdivi diff <PREV> <CURR>` | Compare two snapshot files and print a `DivergenceSummary` |
| `sdivi trend [--last N]` | Slope statistics across stored snapshots (oldest to newest) |
| `sdivi check [--no-write]` | Snapshot, compare against the prior, exit 10 if any threshold is breached |
| `sdivi show [ID]` | Inspect a stored snapshot (defaults to latest); ID is the filename stem |
| `sdivi catalog` | Build and display the pattern catalog only (no snapshot write) |
| `sdivi boundaries infer` | Propose module groupings from Leiden community history |
| `sdivi boundaries ratify` | Write accepted groupings to `.sdivi/boundaries.yaml` |
| `sdivi boundaries show` | Print the current boundary specification |

### Exit codes (public API)

| Code | Meaning |
|---|---|
| `0` | Success |
| `1` | Generic runtime error (I/O, parse, internal) |
| `2` | Configuration error (e.g. threshold override missing `expires`) |
| `3` | All detected languages lack tree-sitter grammars (no records produced) |
| `10` | `sdivi check` only. At least one threshold was breached. |

Adding or repurposing an exit code is a breaking change.

Full flag and behaviour reference lives in
[`docs/cli-integration.md`](docs/cli-integration.md).

---

## Configuration

Config loads in this order (later wins):

1. CLI flags
2. Environment variables (`SDIVI_LOG_LEVEL`, `SDIVI_WORKERS`,
   `SDIVI_CONFIG_PATH`, `SDIVI_SNAPSHOT_DIR`, `NO_COLOR`)
3. Project `.sdivi/config.toml`
4. Global `$XDG_CONFIG_HOME/sdivi/config.toml`
5. Built-in defaults

All keys are optional. A representative `.sdivi/config.toml`:

```toml
[core]
languages   = "auto"
exclude     = ["vendor/**", "node_modules/**", "target/**"]
random_seed = 42

[snapshots]
dir       = ".sdivi/snapshots"
retention = 100

[thresholds]
pattern_entropy_rate    = 2.0
convention_drift_rate   = 3.0
coupling_delta_rate     = 0.15
boundary_violation_rate = 2.0

[output]
format = "text"
color  = "auto"
```

### Per-category threshold overrides

A team migrating an idiom can raise the limit for one pattern category for a
fixed window. The `expires` date is mandatory, and after the date passes the
override is ignored automatically.

```toml
[thresholds.overrides.error_handling]
pattern_entropy_rate = 5.0
expires              = "2026-09-30"
reason               = "Migrating to ? operator"
```

A missing `expires` returns a configuration error (exit 2). Full key reference
lives in [`docs/cli-integration.md`](docs/cli-integration.md).

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

A missing `boundaries.yaml` is normal operation. Every metric except
`boundary_violation_rate` is still computed; intent divergence is simply
absent from the snapshot.

After a few snapshots have accumulated, `sdivi boundaries infer` proposes
groupings derived from the Leiden community history (stable across
`stability_threshold` consecutive snapshots). `sdivi boundaries ratify` writes
them to `boundaries.yaml`. Comments in `boundaries.yaml` are lost when ratify
rewrites the file.

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

sdivi-rust is built to be embedded. Two paths exist depending on whether you
want SDIVI to handle file discovery and parsing for you, or you have your own
extractors and want the pure compute layer.

### Rust: full pipeline (`sdivi-pipeline`)

Use this when you want SDIVI to walk the filesystem, parse with tree-sitter,
and write snapshots:

```toml
[dependencies]
sdivi-config    = "0.1"
sdivi-pipeline  = "0.1"
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
runs the pipeline without persisting a snapshot. This is the right call for
PR-preview gates that only want a threshold result.

### Rust: pure compute (`sdivi-core`)

Use this when you already have your own extractors. This is the path
agent runtimes and gardener LLMs (the consumer apps) take.

```toml
[dependencies]
sdivi-core = "0.1"
```

`sdivi-core` exposes `compute_*` functions over plain `serde` `*Input`
structs:

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

`sdivi-core` has no I/O, no clock, no tree-sitter, and no `std::fs`. Every
function that conceptually needs the wall clock takes a `chrono::NaiveDate`
as input (e.g. `compute_thresholds_check` resolves override expiry against
the supplied date).

### WASM, JS, and TS (`@geoffgodwin/sdivi-wasm`)

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

Every input and output type has a `.d.ts` definition derived from `tsify`.
The WASM build pulls only `sdivi-core`'s pure-compute path, so `walkdir`,
`ignore`, `rayon`, and `tree-sitter` are absent from the bundle.

### Public surface

| Crate | Use it for | Stability |
|---|---|---|
| `sdivi-core` | Pure-compute API; WASM target; foreign extractors | Stable, `#![deny(missing_docs)]` |
| `sdivi-pipeline` | Full FS pipeline from Rust | Stable |
| `sdivi-cli` | The `sdivi` binary | Stable CLI surface |
| `sdivi-config` | `Config` loader and types | Stable |
| `sdivi-lang-{rust,python,typescript,javascript,go,java}` | Language adapters | Stable |
| `sdivi-graph`, `sdivi-detection`, `sdivi-patterns`, `sdivi-snapshot`, `sdivi-parsing` | Inner crates. Depend on these only if you need a single stage. | Stable. Most embedders should use `sdivi-core` or `sdivi-pipeline`. |

Adding a `pub` item is deliberate. Removing or renaming one is a breaking
change. SemVer commitment begins at `0.1.0`.

For longer-form embedding guidance, see
[`docs/library-embedding.md`](docs/library-embedding.md).

---

## Documentation

| Document | What it covers |
|---|---|
| [`docs/cli-integration.md`](docs/cli-integration.md) | CI integration, GitHub Actions snippet, exit codes, full flag reference |
| [`docs/library-embedding.md`](docs/library-embedding.md) | Rust + WASM embedding guide, common pitfalls, change-coupling for foreign extractors |
| [`docs/snapshot-schema.md`](docs/snapshot-schema.md) | `Snapshot` JSON schema reference |
| [`docs/determinism.md`](docs/determinism.md) | `BTreeMap` discipline, seed contract, FMA notes, cross-platform guarantees |
| [`docs/migrating-from-the-python-poc.md`](docs/migrating-from-the-python-poc.md) | Migrating from the Python POC |
| [`CHANGELOG.md`](CHANGELOG.md) | Per-release changes |

API reference (rustdoc) is published to docs.rs on every `cargo publish`.

---

## Examples

```sh
cargo run --example embed_pipeline    # full FS pipeline
cargo run --example embed_compute     # pure-compute path with parity check
cargo run --example custom_config     # programmatic Config building
```

Sources live at [`crates/sdivi-cli/examples/`](crates/sdivi-cli/examples/).
A TypeScript embedding example lives at
[`examples/binding_node.ts`](examples/binding_node.ts).

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

Apache 2.0. See [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).
