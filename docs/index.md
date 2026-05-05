---
title: sdivi-rust documentation
---

# sdivi-rust

**Structural Divergence Indexer.** Pronounced *Stevie*. A measurement
instrument that tracks how fast the structure of a codebase is changing.

SDIVI captures the structural fingerprint of a repository (dependency graph,
community partition, pattern catalog) into deterministic JSON snapshots, then
reports four divergence metrics between consecutive snapshots:

- **Pattern entropy rate.** How fast coding patterns are diverging.
- **Convention drift rate.** How fast style and idiom conventions shift.
- **Coupling delta rate.** How fast inter-module coupling changes.
- **Boundary violation rate.** How often code crosses declared module boundaries.

Threshold breaches are observations. SDIVI never opines on code quality. It
produces numbers, and a CI gate (`sdivi check`) decides what to do with them.

> sdivi-rust is the Rust reimplementation of the Python POC
> ([`structural-divergence-indexer`](https://github.com/GeoffGodwin/structural-divergence-indexer)).
> Snapshots are not backward compatible with the Python POC. See
> [Migrating from the Python POC](migrating-from-the-python-poc).

---

## Where to start

| If you want to... | Read this |
|---|---|
| Install SDIVI and run your first snapshot | [README on GitHub](https://github.com/geoffgodwin/sdivi-rust#install) |
| Wire `sdivi check` into a CI gate | [CLI integration guide](cli-integration) |
| Embed SDIVI from Rust (full pipeline) | [Library embedding guide](library-embedding#orchestration-path-sdivi-pipeline) |
| Embed SDIVI from your own AST extractor (consumer-app-style) | [Library embedding guide](library-embedding#pure-compute-path-sdivi-core) |
| Call SDIVI from TypeScript / WASM | [Library embedding guide](library-embedding#wasm-integration-typescript--javascript) |
| Read or generate snapshot JSON | [Snapshot schema reference](snapshot-schema) |
| Understand reproducibility guarantees | [Determinism notes](determinism) |
| Migrate from the Python POC | [Migrating from the Python POC](migrating-from-the-python-poc) |

---

## Architecture in one diagram

sdivi-rust ships as a Cargo workspace with a two-layer library shape.

```
                ── orchestration path (sdivi-pipeline) ──

config + boundaries + repo
       │
       ▼
[ Pipeline::new(&Config) ]
       │
       ▼
parsing → graph → detection → patterns → snapshot/delta
       │
       ▼
sdivi-cli formats text/JSON ──► stdout    (logs/progress ──► stderr)


                ── pure-compute path (sdivi-core / WASM) ──

caller-supplied: graph, patterns, leiden config, thresholds, prior partitions
       │
       ▼
sdivi_core::{ detect_boundaries, compute_coupling_topology,
              compute_pattern_metrics, compute_thresholds_check,
              compute_delta, normalize_and_hash, ... }
       │
       ▼
caller assembles its own report
```

`sdivi-core` is WASM-compatible. No I/O, no clock, no tree-sitter. Every
function that conceptually needs the wall clock takes a `chrono::NaiveDate`
as input.

---

## Public surface

| Crate | Use it for | Stability |
|---|---|---|
| `sdivi-core` | Pure-compute API; WASM target; foreign extractors | Stable, `#![deny(missing_docs)]` |
| `sdivi-pipeline` | Full FS pipeline from Rust | Stable |
| `sdivi-cli` | The `sdivi` binary | Stable CLI surface |
| `sdivi-config` | `Config` loader and types | Stable |
| `sdivi-lang-{rust,python,typescript,javascript,go,java}` | Language adapters | Stable |
| `sdivi-graph`, `sdivi-detection`, `sdivi-patterns`, `sdivi-snapshot`, `sdivi-parsing` | Inner crates. Use only if you need a single stage. | Stable |

Adding a `pub` item is deliberate. Removing or renaming one is a breaking
change. SemVer commitment begins at `0.1.0`.

API reference (rustdoc) is published to docs.rs on every `cargo publish`:

- [`sdivi-core` on docs.rs](https://docs.rs/sdivi-core)
- [`sdivi-pipeline` on docs.rs](https://docs.rs/sdivi-pipeline)
- [`sdivi-cli` on docs.rs](https://docs.rs/sdivi-cli)

---

## Source

- [Repository on GitHub](https://github.com/geoffgodwin/sdivi-rust)
- [Latest release](https://github.com/geoffgodwin/sdivi-rust/releases/latest)
- [`@geoffgodwin/sdivi-wasm` on npm](https://www.npmjs.com/package/@geoffgodwin/sdivi-wasm)

Released under [Apache 2.0](https://github.com/geoffgodwin/sdivi-rust/blob/main/LICENSE).
