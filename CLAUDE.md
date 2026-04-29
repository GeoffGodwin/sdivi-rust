# sdi-rust

## Project Identity

sdi-rust is the Rust reimplementation of the Structural Divergence Indexer (SDI), a measurement instrument that tracks the rate of structural drift in a codebase. It is delivered as a Cargo workspace whose primary product is the `sdi-core` library crate, with a thin CLI shell (`sdi-cli`) re-exposing the `sdi` command surface (`init`, `snapshot`, `diff`, `trend`, `check`, `show`, `boundaries`, `catalog`). The audience is software engineers, tech leads, engineering managers responsible for structural health of codebases — and now tooling authors and gardener-LLM developers who want to embed the analysis pipeline directly via Rust, Python, or Node bindings rather than shelling out to a binary.

**Languages:**
- Rust

**Frameworks and key dependencies:** `tree-sitter` (AST parsing, native Rust crate, per-language grammars gated by Cargo features), `petgraph` (default graph representation; custom CSR view possible for the Leiden hot path), `clap` v4 with derive macros (CLI), `ratatui` + `owo-colors` + `anstream` (terminal output), `rayon` (parsing parallelism), `serde` + `serde_json` + `serde_yaml` + `toml` (serialization), `rand` with `StdRng` (explicit seeded RNG), `blake3` (pattern fingerprints) and `xxh3` (cache keys), `thiserror` (library errors) plus `anyhow` (sdi-cli only), `walkdir` + `globset` + `ignore` (file discovery), `tracing` + `tracing-subscriber` (stderr structured logs). PyO3 (`sdi-py`) and napi-rs (`sdi-node`) for bindings — post-MVP.

**Target platforms and deployment model:**
- Tier 1: Linux x86_64 + aarch64 (CI on `ubuntu-latest`); macOS x86_64 + aarch64 (CI on `macos-latest`)
- Tier 2: Windows x86_64 (CI on `windows-latest`, release-only build verification)
- MSRV: "stable latest minus 2", pinned in `rust-toolchain.toml`, verified in CI
- Distribution: crates.io (`sdi-core`, `sdi-cli`, language adapters), GitHub Releases (prebuilt `sdi` binaries Linux/macOS/Windows), PyPI (PyO3 wheel, post-MVP), npm (napi-rs prebuilt, post-MVP). WASM is post-MVP and only when a real consumer exists.
- Invocation: typically once per merge to the primary branch (CI gate) plus ad-hoc human invocations; bindings introduce continuous in-process invocation from long-running agent runtimes. No daemon, no server, no interactive TUI.

**License:** **Apache 2.0** (ratified 2026-04-28). Permissive open source with explicit attribution preservation, contributor patent grant, and broad enterprise acceptance. `LICENSE` and `NOTICE` are at the repo root; every published crate sets `license = "Apache-2.0"` in its `Cargo.toml`.

## Architecture Philosophy

sdi-rust inherits sdi-py's principles unchanged — they are language-agnostic and have already been ratified through the Python POC. Rust upgrades several of them from convention to compiler-enforced invariants.

### Concrete Patterns This Project Follows

- **Five-stage sequential pipeline.** `parsing → graph → detection → patterns → snapshot/delta`. No stage may reach backward; downstream stages consume the previous stage's output as data, not as a live reference into earlier internals.
- **Composition root in `sdi-cli`.** All wiring of config, pipeline, and presentation lives in `sdi-cli`. Library crates expose data and pure functions; they never reach for stdout, env vars, or filesystem state outside what is passed to them.
- **Concrete types over traits.** `FeatureRecord`, `Snapshot`, `BoundarySpec`, `PatternCatalog`, `DependencyGraph`, `LeidenPartition`, and `DivergenceSummary` are concrete `serde::Serialize + Deserialize` structs. Embedders see exact-shape data, not opaque handles. The single trait extension point is `LanguageAdapter`.
- **Ownership-enforced memory discipline.** The parsing API consumes file content + grammar and returns a `FeatureRecord`; the tree-sitter CST is dropped before the function returns. Memory usage is proportional to the largest single source file plus the dependency graph footprint — not the total codebase size.
- **Determinism by construction.** `BTreeMap` over `HashMap` everywhere output ordering matters. RNG is `StdRng` with an explicit seed (default `42`). `BTreeMap`-keyed pattern catalog with `blake3` hashing seeded from a fixed key.
- **Pure functions for derived data.** `Pipeline::delta` and `compute_delta` are referentially transparent. The same two snapshots always produce the same `DivergenceSummary`.
- **Library-shape supremacy (KD12).** The `sdi-cli` crate cannot add code paths that aren't reachable through `sdi-core`. Every CLI feature is a library feature first.
- **Snapshot schema clean break (KD13).** sdi-rust uses `snapshot_version: "1.0"`. We do not import sdi-py snapshots; trend continuity for migrators is acceptably lost.
- **Native Leiden, no FFI (KD11).** A native Rust port of Traag et al. 2019 Leiden, ~1500–2500 LOC, verified against `leidenalg` on partition quality (modularity within 1%, community count within ±10%) — not bit identity.

### Anti-Patterns This Project Avoids

- **No ML/LLM calls in the analysis pipeline.** sdi-rust is a measurement instrument; determinism is non-negotiable.
- **No network calls during analysis.** No telemetry, no update checks, no remote lookups. Snapshots must be producible on an airgapped machine.
- **No opinions about code quality.** Pattern entropy is a measurement, not a judgment. Threshold breaches are reported as "exceeded," never as "violations" or "problems."
- **No automatic alert suppression.** Teams declare migration intent via per-category threshold overrides with explicit `expires` dates. After expiry, defaults resume without manual intervention.
- **No interactive TUI or daemon mode.** CLI invocation only. Run, produce output, exit.
- **No `unsafe` in `sdi-core` or language adapters.** Any future need for `unsafe` lives in a dedicated crate behind a feature flag with a per-block `SAFETY:` comment. Bindings crates may use `unsafe` only as required by the binding macro.
- **No `panic!` for recoverable errors.** `panic!` is reserved for "this should be impossible" invariants. Recoverable errors return `Result<T, E>`.
- **No hidden global state.** No global mutable config, no lazy_static analysis caches with shared write access, no thread-local hidden context.
- **No automatic drift-vs-evolution classification (KD1).** The tool measures divergence from declared intent only.
- **No FFI to the C++ Leiden.** Determinism story requires a native port.

### Data Flow

```
config.toml + boundaries.yaml + repo path
       │
       ▼
[Config::load_or_default] ──► Config (precedence resolved)
       │
       ▼
[Pipeline::new(&Config)]
       │
       ▼
Stage 1: parsing       walkdir + ignore + tree-sitter ──► Iterator<FeatureRecord>
       │
       ▼                            (rayon-parallel; CST dropped per-file)
Stage 2: graph         resolve imports, build petgraph ──► DependencyGraph
       │
       ▼
Stage 3: detection     native Leiden(seed, gamma)      ──► LeidenPartition
       │                            (warm-start from .sdi/cache/partition.json if present)
       ▼
Stage 4: patterns      tree-sitter queries + blake3    ──► PatternCatalog
       │
       ▼
Stage 5: snapshot      assemble + load BoundarySpec    ──► Snapshot {snapshot_version: "1.0"}
       │                            (atomic tempfile + rename to .sdi/snapshots/)
       ▼
[Pipeline::delta(prev, curr)] ──► DivergenceSummary  (null per-dim when no prev)
       │
       ▼
sdi-cli formats text/JSON ──► stdout    (logs/progress ──► stderr)
```

### Module Boundaries and Dependency Rules

- `sdi-cli` is the composition root and depends on every library crate.
- `sdi-parsing` depends only on `tree-sitter`, language grammars, and `sdi-config`.
- `sdi-graph` depends on `sdi-parsing` output types and `petgraph`.
- `sdi-detection` depends on `sdi-graph` output types only.
- `sdi-patterns` depends on `sdi-parsing` output. **It must NOT depend on `sdi-graph` or `sdi-detection`.**
- `sdi-snapshot` is the assembly point: depends on `sdi-graph`, `sdi-detection`, `sdi-patterns`, and `sdi-config`.
- `sdi-config` is a leaf crate; depended on by all.
- `sdi-core` re-exports the public pipeline API; **no module imports from `sdi-cli`.**
- Language adapter crates (`sdi-lang-*`) depend only on `sdi-parsing` and `tree-sitter` grammars.
- No cycles between crates. CI fails on a cycle introduced by `cargo metadata` graph inspection.

## Repository Layout

```
sdi-rust/
├── Cargo.toml                           # workspace manifest, pinned deps with .workspace = true
├── Cargo.lock                           # checked in; required for binary crates
├── rust-toolchain.toml                  # MSRV pin (stable latest minus 2)
├── rustfmt.toml                         # empty / defaults; no project overrides
├── clippy.toml                          # warnings-as-errors config
├── deny.toml                            # cargo-deny / cargo-audit policy
├── README.md                            # quick start, install paths, what is SDI; <200 lines
├── CHANGELOG.md                         # hand-maintained, conventional sections
├── MIGRATION_NOTES.md                   # entries for breaking 0.x → 0.(x+1) bumps
├── DRIFT_LOG.md                         # vendoring / patch decisions, design drift notes
├── LICENSE                              # Apache 2.0 (KDD-8)
├── NOTICE                               # Apache 2.0 attribution notice
├── .github/
│   └── workflows/
│       ├── ci.yml                       # lint+build+test matrix on push/PR
│       ├── release.yml                  # tag-driven; manual approval for crates.io
│       ├── audit.yml                    # weekly cargo audit
│       └── verify-leiden.yml            # gated KD11 verification job (Python+leidenalg)
├── .tekhton/
│   └── DESIGN.md                        # this initiative's design doc
├── crates/
│   ├── sdi-core/
│   │   ├── Cargo.toml                   # public, stable per KD12
│   │   ├── src/
│   │   │   ├── lib.rs                   # re-exports + #![deny(missing_docs)]
│   │   │   ├── pipeline.rs              # Pipeline::{new,snapshot,delta}
│   │   │   ├── exit_code.rs             # closed enum, i32 discriminants
│   │   │   ├── error.rs                 # AnalysisError, IoError aggregator
│   │   │   └── prelude.rs               # commonly-imported items
│   │   └── tests/
│   │       ├── pipeline_smoke.rs
│   │       └── exit_code_contract.rs
│   ├── sdi-cli/
│   │   ├── Cargo.toml                   # binary crate; produces `sdi`
│   │   ├── src/
│   │   │   ├── main.rs                  # composition root; anyhow allowed here
│   │   │   ├── commands/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── init.rs
│   │   │   │   ├── snapshot.rs
│   │   │   │   ├── diff.rs
│   │   │   │   ├── trend.rs
│   │   │   │   ├── check.rs             # exit 10 on threshold breach
│   │   │   │   ├── show.rs
│   │   │   │   ├── boundaries.rs        # infer / ratify / show subcommands
│   │   │   │   └── catalog.rs
│   │   │   ├── output/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── json.rs
│   │   │   │   └── text.rs              # ratatui tables + owo-colors
│   │   │   └── logging.rs               # tracing-subscriber → stderr
│   │   └── tests/
│   │       ├── exit_codes.rs            # assert_cmd + predicates
│   │       ├── stdout_stderr_split.rs
│   │       └── atomic_writes.rs
│   ├── sdi-parsing/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── walker.rs                # walkdir + ignore + globset
│   │   │   ├── adapter.rs               # LanguageAdapter trait
│   │   │   ├── feature_record.rs        # FeatureRecord struct
│   │   │   └── parse.rs                 # parse_repository(); CST dropped per-file
│   │   └── tests/
│   │       ├── memory_invariant.rs      # asserts CST not held across files
│   │       └── walk_ordering.rs         # deterministic stable-sorted output
│   ├── sdi-graph/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── dependency_graph.rs
│   │       ├── metrics.rs               # density, cycles, hubs, components
│   │       └── csr_view.rs              # optional cache-friendly view (open Q #2)
│   ├── sdi-detection/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── leiden/
│   │       │   ├── mod.rs               # KD11 native port, ~1500-2500 LOC
│   │       │   ├── modularity.rs
│   │       │   ├── cpm.rs
│   │       │   ├── refine.rs
│   │       │   └── aggregate.rs
│   │       ├── partition.rs             # LeidenPartition struct
│   │       └── warm_start.rs            # load .sdi/cache/partition.json
│   ├── sdi-patterns/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── catalog.rs               # PatternCatalog (BTreeMap-keyed)
│   │       ├── fingerprint.rs           # blake3 with fixed seed
│   │       ├── entropy.rs
│   │       └── queries/                 # per-category tree-sitter query strings
│   │           ├── error_handling.rs
│   │           ├── async_patterns.rs
│   │           └── ...
│   ├── sdi-snapshot/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── snapshot.rs              # snapshot_version: "1.0"
│   │       ├── delta.rs                 # compute_delta; null when no prev
│   │       ├── trend.rs
│   │       ├── store.rs                 # atomic tempfile + rename writes
│   │       └── retention.rs             # synchronous post-write enforcement
│   ├── sdi-config/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── config.rs                # Config struct + Default
│   │       ├── load.rs                  # 5-level precedence resolver
│   │       ├── boundary.rs              # BoundarySpec read from YAML
│   │       ├── thresholds.rs            # per-category overrides w/ expires
│   │       └── error.rs                 # ConfigError variants
│   ├── sdi-lang-python/
│   │   ├── Cargo.toml                   # feature `lang-python`
│   │   ├── build.rs                     # links tree-sitter-python
│   │   └── src/lib.rs                   # impl LanguageAdapter
│   ├── sdi-lang-typescript/
│   ├── sdi-lang-javascript/
│   ├── sdi-lang-go/
│   ├── sdi-lang-java/
│   └── sdi-lang-rust/
├── bindings/                            # post-MVP — empty placeholders during MVP
│   ├── sdi-py/                          # PyO3 wheel
│   └── sdi-node/                        # napi-rs prebuilt
├── examples/
│   ├── embed_pipeline.rs                # minimal embedder
│   ├── custom_config.rs                 # programmatic Config building
│   └── binding_python.py                # post-MVP usage of sdi-py
├── tests/                               # workspace-level cross-crate scenarios
│   ├── full_pipeline.rs
│   ├── snapshot_diff_trend.rs
│   ├── boundary_lifecycle.rs
│   └── fixtures/
│       ├── simple-rust/                 # 5-10 file Rust crate, known imports
│       ├── simple-python/               # cross-language; verifies vs sdi-py outputs
│       ├── multi-language/              # Python + TypeScript adapter exercise
│       ├── high-entropy/                # deliberate pattern variance
│       └── evolving/                    # built by setup script under target/test-fixtures
├── benches/                             # criterion, gated behind `bench` feature
│   ├── parsing.rs
│   ├── leiden.rs
│   └── full_pipeline.rs
└── docs/
    ├── cli-integration.md               # cargo install sdi && sdi check, GHA snippet
    ├── library-embedding.md             # embed sdi-core in an agent runtime
    ├── migrating-from-sdi-py.md         # what carries vs what changes
    └── determinism.md                   # BTreeMap, seed, FMA notes
```

## Key Design Decisions

### KDD-1: Snapshot schema is a clean break from sdi-py

**Decision:** sdi-rust ships `snapshot_version: "1.0"` and refuses to read sdi-py snapshot JSON. `.sdi/cache/*` is also a clean break.

**Alternatives considered:** A read-side compat shim that translates sdi-py snapshot JSON. Rejected: burdens every release with a frozen translator, and the only benefit is preserving trend continuity for the small population that ran the Python POC against the same repo.

**Rationale (KD13):** Trend continuity for migrators is explicitly accepted as lost. `.sdi/config.toml` and `.sdi/boundaries.yaml` remain read-compatible — those are user-edited and worth migrating. Snapshots are tool-generated and trivially regeneratable.

### KDD-2: Native Leiden port, no FFI to C++

**Decision:** Implement Leiden (Traag et al. 2019, Modularity + CPM) natively in `sdi-detection`, ~1500–2500 LOC.

**Alternatives considered:** FFI bindings to the C++ `leidenalg` underlying library. Rejected: introduces a non-Rust toolchain dependency, complicates single-binary distribution, and undermines the determinism story (different platforms link different builds).

**Rationale (KD11):** The native port preserves the ownership/determinism guarantees and keeps the binary single-file. Verification is by partition quality (modularity within 1%, community count within ±10%) on a fixture suite cross-checked against `leidenalg`, **not bit-identity**.

### KDD-3: `sdi-core` library is the canonical surface

**Decision:** Every CLI feature is a library feature first. `sdi-cli` cannot add code paths that aren't reachable through `sdi-core`.

**Rationale (KD12):** This is the entire reason for the rewrite. Bindings (PyO3, napi-rs) and embedders depend on a stable library surface. SemVer commitment begins at `0.1.0`; adding `pub` is deliberate, removing `pub` is breaking.

### KDD-4: Tree-sitter grammars linked at compile time, gated per-language by Cargo features

**Decision:** Each grammar is a build dependency gated by `lang-<name>`. Default feature set matches sdi-py's supported languages.

**Alternatives considered:** Runtime dynamic loading. Rejected for MVP: more complex, deviates from Rust ecosystem norms, and adds a runtime failure mode for what is effectively a static set of grammars per release.

**Rationale (Open Q #3):** Compile-time is simpler and matches ecosystem norms. The feature-flag knob lets binary-size-sensitive consumers strip languages they don't need.

### KDD-5: `petgraph` is the default; CSR view **RATIFIED: NO CSR VIEW**

**Decision (ratified Milestone 5):** Use `petgraph::Graph<NodeId, EdgeWeight>` everywhere in `sdi-graph`. The Leiden algorithm builds its own internal `LeidenGraph` (`Vec<Vec<usize>>` adjacency list) at the start of each run; a separate CSR module in `sdi-graph::csr_view` would duplicate that conversion without benefit. Decided during Milestone 5 profiling; recorded in `DRIFT_LOG.md`.

**Rationale:** The Leiden hot path already converts the petgraph to an internal adjacency list. Adding a CSR module would cost an additional copy of the same data. `petgraph` is fast enough for the 5000-node benchmark graphs. Revisit only if a measured bottleneck demands it.

### KDD-6: YAML write — accept comment loss for MVP

**Decision:** `serde_yaml` for read; programmatic write of `.sdi/boundaries.yaml` may regress comment preservation. Document the limitation; revisit only if users complain.

**Alternatives considered:** A comment-preserving Rust YAML crate (maturity unverified) or a hand-written minimal YAML emitter. Both rejected for MVP scope.

**Rationale (Open Q #1):** Boundary specs are mostly user-edited; programmatic writes happen on `sdi boundaries ratify`. Accepting comment loss on that path is a small UX regression versus shipping a brittle custom emitter.

### KDD-7: MSRV is "stable latest minus 2"

**Decision:** Pin in `rust-toolchain.toml`; verify in CI matrix.

**Rationale (Open Q #4):** Generous enough for distros, conservative enough to use modern features. Bump deliberately, not opportunistically.

### KDD-8: License is Apache 2.0

**Decision:** Apache 2.0 across the workspace. Replaced the initial MIT `LICENSE` (GitHub auto-init) on 2026-04-28; `NOTICE` file added per Apache 2.0 conventions.

**Rationale (Open Q #6 → ratified):** Goal is broad adoption including paid/enterprise use, with attribution preserved. Apache 2.0 is permissive enough for corporate compliance teams to accept by default, includes an explicit contributor patent grant (which MIT lacks), and requires preservation of copyright and the `NOTICE` file. Every published crate's `Cargo.toml` sets `license = "Apache-2.0"`.

### KDD-9: Deltas are `null` on first snapshot, not zero

**Decision:** `compute_delta(prev: &Snapshot, curr: &Snapshot)` requires both arguments. The "first snapshot" path returns a `DivergenceSummary` with all per-dimension fields `null`. Identical consecutive snapshots yield `0`, distinguishing "no comparison possible" from "no change."

**Rationale:** Carries from sdi-py rule 14. Mixing the two is a semantic bug masquerading as a numeric one.

### KDD-10: `BTreeMap` over `IndexMap` for catalogs

**Decision:** Use `BTreeMap` everywhere output ordering matters.

**Rationale (Open Q #9):** `BTreeMap` orders by key (deterministic without relying on insertion order). Revisit only if profiling shows comparison cost dominates a hot path.

### KDD-11: Bindings live in-repo until they earn their own repos

**Decision:** `bindings/sdi-py` and `bindings/sdi-node` ship in this workspace.

**Rationale (Open Q #7):** Cross-repo CI complexity outweighs the workspace benefit only after non-trivial consumer-side surface area. Split out then, not before.

### Unresolved Open Questions and Default Posture

- **Open Q #5 (crate name reservation on crates.io):** verify and reserve `sdi`, `sdi-core`, `sdi-cli` before Milestone 1 closes. Default posture if a name is taken: prefix with `sdi-rs-` and document.
- **Open Q #10 (FMA bit determinism across platforms):** document in `docs/determinism.md`. Aggregate equality only across platforms. Revisit via a build flag if a real adopter needs bit-identity.
- **Open Q #8 (snapshot file naming):** carry forward `snapshot_<timestamp>_<sha>.json`.

## Config Architecture

Config is loaded via TOML files plus environment variables, resolved through a 5-level precedence chain. The schema is **read-compatible with sdi-py** (KD13) — sdi-py users can drop in their existing `.sdi/config.toml` unchanged. New `sdi-rust`-only sections (`[determinism]`, `[bindings]`) are additive.

### Loading Strategy

`Config::load_or_default(repo_root)` walks the precedence order and returns a fully-resolved `Config`. `Config::default()` returns built-in defaults. All keys are optional. Malformed TOML returns `ConfigError::Parse`; out-of-range values return `ConfigError::InvalidValue { key, message }`. Unknown keys produce a deprecation warning to stderr but never error (rule 12 from sdi-py: keys are reserved forever once introduced).

### Precedence (Highest to Lowest)

1. Function arguments (library) / CLI flags (binary)
2. Environment variables: `SDI_LOG_LEVEL`, `SDI_WORKERS`, `SDI_CONFIG_PATH`, `SDI_SNAPSHOT_DIR`, `NO_COLOR`
3. Project-local `.sdi/config.toml`
4. Global `$XDG_CONFIG_HOME/sdi/config.toml` (fallback `~/.config/sdi/config.toml`)
5. Built-in defaults compiled into `sdi-core`

### Complete Default Configuration

```toml
[core]
languages = "auto"
exclude = [
  "**/vendor/**",
  "**/node_modules/**",
  "**/__pycache__/**",
  "**/dist/**",
  "**/build/**",
  "**/target/**",
  "**/.git/**",
]
random_seed = 42

[snapshots]
dir = ".sdi/snapshots"
retention = 100        # 0 = unlimited

[boundaries]
spec_file = ".sdi/boundaries.yaml"
leiden_gamma = 1.0     # manual override only — KD5: no auto-tuning
stability_threshold = 3
weighted_edges = false # KD4

[patterns]
categories = "auto"
min_pattern_nodes = 5
scope_exclude = []     # excludes from catalog only — files remain in graph

[thresholds]
pattern_entropy_rate = 2.0
convention_drift_rate = 3.0
coupling_delta_rate = 0.15
boundary_violation_rate = 2.0

[change_coupling]
min_frequency = 0.6
history_depth = 500

[output]
format = "text"        # "text" | "json"
color = "auto"         # "auto" | "always" | "never"

[determinism]
enforce_btree_order = true   # sdi-rust-only; reserved for FMA toggles

[bindings]
# Reserved for future binding-specific knobs.
```

### Per-Category Threshold Overrides

`expires` is **mandatory**. Missing it returns `ConfigError::InvalidValue` (CLI exit 2). After expiry the override is silently ignored and defaults resume — no manual reset.

```toml
[thresholds.overrides.error_handling]
pattern_entropy_rate = 5.0
expires = "2026-09-30"
reason = "Migrating to ? operator from `match Err(_)` chains"
```

### Merge Strategy

Section-by-section override, later wins. Within a section, key-by-key override. Lists in `core.exclude` and `patterns.scope_exclude` are **replaced**, not merged. Threshold override sub-tables merge per-category: each `[thresholds.overrides.<cat>]` block is added; an existing category override is replaced wholesale.

### Required vs Optional

**All keys are optional.** Missing keys fall through to defaults. The single mandatory field is `expires` *within* a `[thresholds.overrides.<cat>]` block — only required if that block exists at all. There is no top-level required key.

### CLI ↔ Config Mapping

| Flag             | Config key / effect                         |
|------------------|---------------------------------------------|
| `--format json`  | `output.format = "json"`                    |
| `--no-color`     | `output.color = "never"` (also `NO_COLOR=1`)|
| `--workers N`    | Effective parallelism (no config key)       |
| `--seed N`       | `core.random_seed = N`                      |

### Runtime Mutability

`Config` is consumed at `Pipeline::new`. The pipeline does not mutate config during a snapshot run. Per-call overrides build a new `Config`. There is no global mutable config in `sdi-core`.

## Non-Negotiable Rules

1. **`unsafe` is forbidden in `sdi-core` and language adapter crates.** Bindings crates (`sdi-py`, `sdi-node`) may use `unsafe` only as required by the binding macro. Any other `unsafe` lives in a dedicated crate behind a feature flag with a per-block `// SAFETY:` comment justifying the invariant.
2. **No network calls anywhere in the analysis pipeline.** No telemetry, no update checks, no remote lookups. A snapshot must be producible on an airgapped machine. CI tests must not require network.
3. **No ML/LLM calls in the pipeline.** Determinism is the contract. A measurement instrument cannot depend on a stochastic model.
4. **Tree-sitter CSTs are dropped before the parsing function returns.** The `sdi-parsing` API consumes file content + grammar and returns a `FeatureRecord`. No type containing a `Tree` may escape `parse_file`. Memory usage stays proportional to the largest single file, not the codebase total.
5. **`BTreeMap` is the default ordered map.** `HashMap` is allowed only when iteration order does not influence output. Pattern catalogs, snapshot fields, and serde-emitted maps use `BTreeMap`.
6. **All RNG is `StdRng` seeded explicitly from `Config::random_seed`.** Default seed is `42`. No `thread_rng`, no `SystemTime`-based seeding, no implicit RNG.
7. **Pattern fingerprints use `blake3` with a fixed key constant.** The key constant is defined once in `sdi-patterns::fingerprint` and never changes within a `snapshot_version`.
8. **Logs, progress bars, and warnings go to stderr. Snapshot JSON, summaries, and table output go to stdout.** `sdi show --format json | jq '.'` must work without contamination. CI test `tests/stdout_stderr_split.rs` is non-negotiable.
9. **Exit codes are public API: `0`, `1`, `2`, `3`, `10`.** Code `10` is exclusively `sdi check`. Adding or repurposing an exit code is a breaking change requiring a major version bump.
10. **`.sdi/config.toml` and `.sdi/boundaries.yaml` are read-compatible with sdi-py.** New config keys are additive. Existing key semantics may not change. Removed keys are reserved forever.
11. **`snapshot_version` is `"1.0"` for all sdi-rust output.** sdi-rust does not read sdi-py snapshots. Reading an incompatible `snapshot_version` produces a warning and baseline treatment (no delta), never a crash.
12. **Per-category threshold overrides require an `expires` field.** Missing `expires` is a config error (exit 2). After expiry the override is silently ignored — no manual reset, no retention.
13. **Snapshot writes are atomic.** Write to a tempfile in the target snapshot directory, then rename. A killed process must never leave a half-written `.json` file. Retention is enforced synchronously after each successful write.
14. **First-snapshot deltas are `null`, not zero.** `null` means "no prior snapshot to compare." `0` means "snapshots compared and no change observed." These are different and observable in the CI gate.
15. **Missing tree-sitter grammars are warnings unless all detected languages lack grammars.** A single missing grammar logs to stderr and skips those files. Only when *all* detected languages lack grammars does `sdi snapshot` exit with code 3.
16. **Missing `BoundarySpec` is normal operation.** All metrics except intent divergence are computed; no warning is emitted. Intent divergence is simply absent from the snapshot.
17. **`sdi-cli` cannot add code paths unreachable through `sdi-core`.** Every CLI feature has a library entry point. The CLI is a thin presentation layer.
18. **Public API stability begins at `0.1.0`.** Adding a `pub` item is deliberate; removing or renaming a `pub` item is a breaking change. Internal-only items live in `pub(crate)` modules.
19. **`#![deny(missing_docs)]` is enabled on `sdi-core`.** Every public item has at least one rustdoc comment with an `# Examples` block where it is meaningful. Doc tests run in CI.
20. **`cargo clippy -- -D warnings` and `cargo fmt --check` are part of CI.** No `#[allow(...)]` on public items without an inline justification comment.

## Implementation Milestones

<!-- Milestones are managed as individual files in .claude/milestones/.
     See MANIFEST.cfg for ordering and dependencies. -->

## Code Conventions

- **Crate names:** `kebab-case`, `sdi-` prefix (`sdi-core`, `sdi-cli`, `sdi-lang-python`).
- **Modules:** `snake_case` (`parsing`, `snapshot`).
- **Types and traits:** `PascalCase` (`PatternFingerprint`, `BoundarySpec`, `LanguageAdapter`).
- **Functions, methods, fields, locals:** `snake_case` (`parse_repository`, `random_seed`).
- **Constants:** `SCREAMING_SNAKE_CASE` (`DEFAULT_GAMMA`, `MAX_RETENTION`, `EXIT_THRESHOLD_EXCEEDED`).
- **Generics:** single-letter when unambiguous (`T`, `E`); descriptive when multiple parameters in scope (`Lang`, `Adapter`).
- **Lifetimes:** `'a` by default; descriptive when more than one in scope (`'src`, `'cfg`).
- **File ceiling:** 500 lines guideline. Above that, split by sub-concern.
- **Tests:** unit tests co-located via `#[cfg(test)] mod tests`; integration tests in `tests/<crate-or-feature>.rs`.
- **Imports / `use` ordering:** rustfmt default — no project-specific grouping rules.
- **Error handling pattern:**
  - Library crates use `thiserror` to derive named variants with structured fields (`#[error("missing expires on {category}")] MissingExpires { category: String }`).
  - `sdi-cli` uses `anyhow::Result` at the binary boundary in `main.rs` only.
  - `panic!` is reserved for "this should be impossible" invariant violations.
  - `Result<T, E>` is the default return style for any function that can fail.
  - All error variants carry context (file path, line number, key name) so callers can surface them meaningfully.
- **Public API discipline:**
  - Adding `pub` is deliberate; removing or renaming `pub` is a breaking change.
  - Internal-only items live in `pub(crate)` modules.
  - Enums that may grow new variants over time use `#[non_exhaustive]` (`BoundaryViolation`, `PatternInstance`).
  - `ExitCode` is closed (no `#[non_exhaustive]`) because the contract is fixed.
- **Doc discipline:**
  - `#![deny(missing_docs)]` on `sdi-core`.
  - Every public item has a doc comment; an `# Examples` block where meaningful.
  - Doc tests run in CI; broken examples fail the build.
- **Lint discipline:**
  - `cargo clippy -- -D warnings` and `cargo fmt --check` are CI gates.
  - `#[allow(...)]` on public items requires an inline justification comment.
- **Git workflow:**
  - Branches: `feat/<topic>`, `fix/<topic>`, `docs/<topic>`, `chore/<topic>`.
  - Commits: conventional-style summary lines (`feat(sdi-detection): native Leiden CPM quality`); imperative mood; no trailing period.
  - PRs reference the milestone they belong to in the title (`[M5] Leiden modularity quality function`).
  - Squash-merge to `main`. Tags on `main` only.
  - `CHANGELOG.md` updated by the PR author for any user-visible change.

## Critical System Rules

1. A `Pipeline::snapshot` call against the same repo state with the same `Config` (including the same `random_seed`) produces a **bit-identical** `Snapshot` JSON. Any divergence is a bug.
2. The parsing stage never holds two `tree_sitter::Tree` values in scope simultaneously across the same execution unit — the per-file `parse_file` API drops its `Tree` before returning.
3. `compute_delta(prev, curr)` is referentially transparent: same inputs → same `DivergenceSummary`. It performs no I/O, reads no globals, and uses no clock.
4. `sdi check` is the only command that exits with code `10`. Every other command's success path must exit `0`.
5. A first-snapshot `DivergenceSummary` has `null` per-dimension fields. `0` is reserved for "compared and unchanged."
6. Threshold overrides without `expires` are a `ConfigError::MissingExpiresOnOverride { category }` with exit `2`. After expiry, the override is silently ignored — defaults resume.
7. `sdi snapshot` exits `3` only if **all** detected languages lack tree-sitter grammars. A single missing grammar logs to stderr and skips files.
8. A missing `.sdi/boundaries.yaml` is **normal operation**. No warning is emitted. Intent divergence fields are simply absent from the snapshot.
9. Snapshot files are written atomically (tempfile in target dir + rename). A killed process never leaves a half-written `.json` in `.sdi/snapshots/`.
10. Retention is enforced **synchronously after** the rename. A failed write does not remove an old snapshot.
11. Logs, progress, warnings → **stderr**. Snapshot JSON, summaries, table output → **stdout**. `sdi show --format json | jq '.'` must always work.
12. `Config` is consumed at `Pipeline::new`. The pipeline mutates no config field during a run. There is no global mutable config.
13. The pipeline performs **zero network calls**. Tests assert this by running with network disabled when supported by the CI runner.
14. `sdi-patterns` does not import or depend on `sdi-graph` or `sdi-detection`. Violation is a `cargo metadata` graph cycle and must be CI-blocked.
15. `sdi-cli` adds no analysis code paths unreachable through `sdi-core`. Every CLI feature is callable from the library.
16. `snapshot_version` is the literal string `"1.0"` for all sdi-rust output. Bumping it is a breaking change requiring a major version bump and a `MIGRATION_NOTES.md` entry.
17. Reading a `Snapshot` JSON with an incompatible `snapshot_version` produces a stderr warning and baseline treatment (no delta) — never a crash.
18. RNG is `StdRng` seeded from `Config::random_seed`. No `thread_rng`, no `SystemTime`-derived seeds, no implicit RNG anywhere in the analysis pipeline.
19. Pattern fingerprints use `blake3` with a single fixed key constant defined once in `sdi-patterns::fingerprint`. Changing the constant invalidates all existing snapshot fingerprints and requires a snapshot version bump.
20. Adding a new variant to `ExitCode` is a breaking change. Reusing or repurposing an existing exit code is a breaking change.

## What Not to Build Yet

- **GitHub Actions reusable action** — easier with a single binary, but still post-MVP polish. Document manual `cargo install sdi && sdi check` for m01–m03; revisit after a stable schema.
- **WASM bindings** — KD14: not MVP. Lands when a concrete consumer exists.
- **IDE / editor plugin** — requires a stable API and snapshot schema. Post-1.0.
- **SaaS dashboard or web UI** — sdi-rust is a measurement instrument, not a platform. Output is JSON; existing dashboards (Grafana, Datadog) consume it.
- **Auto-remediation / gardener agent** — sdi-rust detects and measures drift; it never fixes it. A companion tool generating consolidation PRs is a separate project.
- **Plugin system for custom analyzers** — built-in pattern categories only at MVP (KD6 from sdi-py). Extensibility design after real user feedback.
- **Cross-language dependency inference** — v0 tracks only explicit in-language imports. Modeling cross-language coupling (TypeScript → Python via API) requires API contract parsing — out of scope.
- **Historical backfill UX** — `sdi snapshot --commit REF` works for individual commits. Batch backfill across hundreds of commits (parallelism, progress, storage) is not designed; users script it with a bash loop.
- **Real-time / watch mode** — no file watcher daemon. CLI invocation on merge events is the intended cadence. Watch mode violates the Unix-philosophy constraint.
- **Automatic drift-vs-evolution classification** — explicitly rejected (KD1 from sdi-py). Humans declare migration intent via threshold overrides.
- **Stdin input for `sdi diff`** — carries forward as deferred from sdi-py.
- **`sdi config` subcommand** — edit `.sdi/config.toml` directly. Same deferral as sdi-py.
- **Comment-preserving YAML write** — KDD-6 accepts comment loss for MVP. Revisit only on user complaint.
- **CSR-view custom graph type** — KDD-5 ratified NO in Milestone 5. No CSR module; `petgraph` is fast enough.
- **Importing sdi-py snapshots** — KDD-1 clean break. Trend continuity for migrators is acceptably lost.
- **Bit-identical snapshot output across platforms** — Open Q #10. Aggregate equality only across platforms; revisit via build flag if a real adopter needs it.
- **Bindings split into separate repos** — KDD-11 in-repo until non-trivial cross-repo CI complexity earns the split.

## Testing Strategy

### Frameworks and Tools

- `cargo test` is the entry point for everything default-runnable.
- `proptest` for property-based determinism tests.
- `criterion` for benchmarks, gated behind feature `bench`, run on release tags only.
- `assert_cmd` + `predicates` for CLI integration testing.
- `tempfile` for filesystem fixtures.
- Python + `leidenalg` for the `verify-leiden` cross-check suite, gated behind feature `verify-leiden`.

### Test Tiers

- **Unit tests** in each crate via `#[cfg(test)] mod tests` blocks or per-crate `tests/` files. Coverage targets:
  - 80%+ for `sdi-core`, `sdi-parsing`, `sdi-graph`, `sdi-detection`, `sdi-patterns`, `sdi-snapshot`
  - 60%+ for `sdi-cli` (rest covered by integration tests)
- **Per-crate integration tests** in each crate's `tests/` directory exercising real tree-sitter parsing, real graphs, real fixture repos.
- **Workspace-level integration tests** in `tests/` (top level) covering cross-crate scenarios: full pipeline, snapshot/diff/trend lifecycles, boundary lifecycle.
- **Doc tests** via `cargo test --doc`. Every public function with an `# Examples` block has a runnable doc test. Broken examples fail CI.
- **Property tests** in `crates/<crate>/tests/proptest.rs` files: `prop_test_pipeline_deterministic`, `prop_test_delta_pure`, `prop_test_leiden_seeded`. `proptest-regressions/` directories committed.
- **KD11 verification suite** in `crates/sdi-detection/tests/leiden_quality.rs`, gated `#[cfg(feature = "verify-leiden")]`. Pass criteria: modularity within 1%, community count within ±10%. **Not** bit-identity.
- **CLI exit-code tests** in `crates/sdi-cli/tests/exit_codes.rs` exhaustively covering 0/1/2/3/10.
- **Stdout/stderr discipline tests** in `crates/sdi-cli/tests/stdout_stderr_split.rs`.
- **Atomic-write tests** in `crates/sdi-snapshot/tests/atomic_write.rs` simulating mid-write panic.
- **Memory-invariant test** in `crates/sdi-parsing/tests/memory_invariant.rs` asserting the CST-drop ownership rule.
- **Benchmarks** under `benches/` in each crate; gated `#[cfg(feature = "bench")]`; tracked in `CHANGELOG.md` per release.

### Test Fixtures

Under `tests/fixtures/`:

- `simple-rust/` — small Cargo crate with known imports
- `simple-python/`, `simple-typescript/`, `simple-javascript/`, `simple-go/`, `simple-java/` — per-language minimal fixtures
- `multi-language/` — Python + TypeScript exercise
- `high-entropy/` — deliberate pattern variance
- `evolving/` — git repo with progressive drift, built by setup script under `target/test-fixtures` before tests run
- `leiden-graphs/{small,medium,large}/` — adjacency lists + reference modularities for KD11 verification

### Patterns

- Use **factory functions** for test data, not on-disk fixtures, where the data is small and synthetic.
- Use **on-disk fixtures** for repository-shaped scenarios (parsing, graph, full pipeline).
- **Mock no internals.** No mock for `Pipeline`, no mock for `LanguageAdapter`. Real types, real fixtures.
- **Mock no network.** There is no network code to mock — Rule 13 forbids it.
- **Use real filesystem** via `tempfile` for any test that touches `.sdi/`.

### What We Do NOT Test

- Real network access (Rule 13).
- Cross-version migration of sdi-py snapshot JSON (KD13 clean break).
- Bit-identity of snapshot output across platforms (Open Q #10 — aggregate equality only).

### Commands

- Default: `cargo test --workspace`
- With Leiden verification: `cargo test --workspace --features verify-leiden` (requires Python + leidenalg)
- Doc tests only: `cargo test --doc --workspace`
- Single crate: `cargo test -p sdi-detection`
- Benchmarks: `cargo bench --features bench`
- Coverage (CI nightly): `cargo llvm-cov --workspace --html`

## Development Environment

### Prerequisites

- Rust toolchain matching `rust-toolchain.toml` (MSRV: stable latest minus 2). Install via `rustup`.
- `git`
- C compiler (cc on Linux/macOS, MSVC build tools on Windows) — required by tree-sitter grammar build dependencies.
- For the `verify-leiden` feature only: Python ≥ 3.9 and `pip install leidenalg igraph`.
- `cargo audit` for the audit job: `cargo install cargo-audit` (CI installs automatically; local dev optional).

### Setup

```bash
git clone https://github.com/<org>/sdi-rust.git
cd sdi-rust
rustup show              # confirms rust-toolchain.toml pin
cargo build --workspace  # downloads + compiles all crates
cargo test --workspace   # runs default test suite
```

### Build Commands

- `cargo build --workspace` — debug build of every crate.
- `cargo build --release` — produces optimized `target/release/sdi`.
- `cargo build -p sdi-core` — library-only build for embedding consumers.
- `cargo build --no-default-features --features lang-python` — minimal binary supporting only Python.
- `cargo doc --workspace --no-deps --open` — build and view rustdoc locally.

### Run Commands

- `./target/release/sdi --help` — CLI help.
- `./target/release/sdi init` — initialize a `.sdi/` directory in CWD.
- `./target/release/sdi snapshot` — capture a snapshot.
- `./target/release/sdi check` — run the threshold gate; exit 10 on breach.
- `cargo run --example embed_pipeline -- <repo-path>` — run an embedder example.
- `cargo run -p sdi-cli -- show --format json | jq '.'` — inspect latest snapshot.

### Environment Variables

- `SDI_LOG_LEVEL=debug` — `tracing` level.
- `SDI_WORKERS=4` — parallel parsing workers.
- `SDI_CONFIG_PATH=/abs/path/config.toml` — override config search path.
- `SDI_SNAPSHOT_DIR=.sdi/snapshots` — override snapshot directory.
- `NO_COLOR=1` — disable ANSI color in output (also `--no-color`).
- For development only: `RUST_BACKTRACE=1` for panic traces.

### IDE / Editor Recommendations

- **VS Code** with `rust-analyzer` extension. Enable `clippy` on save: `"rust-analyzer.checkOnSave.command": "clippy"`.
- **JetBrains RustRover** also supported; no project-specific settings.
- The repo ships no `.vscode/` or `.idea/` directory — editor config is per-developer.

## Documentation Responsibilities

### Documentation Sources

- **`README.md`** at repo root — quick start, install paths, one-paragraph SDI overview, links. Under 200 lines.
- **`CHANGELOG.md`** — hand-maintained, conventional sections (Added / Changed / Deprecated / Removed / Fixed / Security). One entry per release tag.
- **`MIGRATION_NOTES.md`** — entries for breaking changes between 0.x → 0.(x+1) bumps and post-1.0 majors.
- **`DRIFT_LOG.md`** — vendoring decisions, `[patch.crates-io]` justifications, design drift between `.tekhton/DESIGN.md` and reality.
- **rustdoc on `sdi-core`** — canonical API reference, published to docs.rs on `cargo publish`. `#![deny(missing_docs)]` enforced. Every public item has a doc comment with an `# Examples` block where meaningful.
- **`docs/cli-integration.md`** — manual CI integration recipe, GitHub Actions snippet, exit-code reference.
- **`docs/library-embedding.md`** — embedding `sdi-core` in a Rust agent runtime; minimal viable consumer; common pitfalls.
- **`docs/migrating-from-sdi-py.md`** — what carries vs what changes vs explicit non-goals.
- **`docs/determinism.md`** — `BTreeMap` discipline, seed contract, FMA notes.
- **`examples/` directory** — runnable consumer snippets, not a published crate.

### Ownership

- **Feature author** owns doc updates for the surface they introduce — rustdoc on new public items, `docs/*.md` updates if the feature changes embedder-facing behavior, `CHANGELOG.md` entry.
- **The Tekhton milestone author** owns the relevant section of `MIGRATION_NOTES.md` if the milestone introduces a breaking change.
- **The release manager** owns the `CHANGELOG.md` finalization at tag time.
- **Auto-generated:** rustdoc HTML on docs.rs (publishes on `cargo publish`). Nothing else is auto-generated.

### Update Cadence

- **Per feature** (in the same PR): rustdoc on new public items, `docs/*.md` if behavior changes for embedders, `CHANGELOG.md` entry under the next-release section.
- **Per milestone** (Tekhton checkpoint): `docs/migrating-from-sdi-py.md` if the milestone changes anything visible to migrators.
- **Per release** (tag): `CHANGELOG.md` finalized into a versioned section, `MIGRATION_NOTES.md` extended if breaking, binary size noted.

### Public Surface

The "public surface" requiring docs:

- Every `pub` item in `sdi-core` and the language adapter crates (rustdoc, doc test where meaningful).
- Every `sdi` CLI command, flag, and exit code.
- Every `.sdi/config.toml` key (both `sdi-py`-shared and sdi-rust-only).
- The `Snapshot` JSON schema (versioned via `snapshot_version`).
- The `BoundarySpec` YAML schema.
- The `ExitCode` enum.
- Bindings (post-MVP): every `sdi` Python and Node entry point.

### Doc Freshness Policy

**Strict (block).** Specifically:

- `cargo doc --workspace --no-deps` must produce zero warnings.
- `cargo test --doc --workspace` must pass — broken doc-test examples fail CI.
- `#![deny(missing_docs)]` blocks compilation if a public item lacks a doc comment.
- Adding a `pub` item without a doc test is a CI failure on `sdi-core`.
- A breaking change without a `MIGRATION_NOTES.md` entry blocks the release workflow's manual approval gate.
- The audit cron blocks the release workflow if a yanked dep is in use.
<!-- tekhton-managed -->
