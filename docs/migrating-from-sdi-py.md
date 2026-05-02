# Migrating from sdi-py to sdi-rust

## Overview

sdi-rust is a full reimplementation of the Structural Divergence Indexer in
Rust. It is **not** backward-compatible with sdi-py snapshot files. The
migration involves:

1. Installing sdi-rust and running `sdi init`
2. Accepting a trend-continuity reset (see Snapshot Schema)
3. Copying your `.sdi/config.toml` and `.sdi/boundaries.yaml` unchanged
4. Running `sdi snapshot` to establish a new baseline

## What Carries Over

| Item | Status |
|---|---|
| `.sdi/config.toml` syntax and keys | **Fully compatible** — same schema |
| `.sdi/boundaries.yaml` syntax | **Fully compatible** — same schema |
| Per-category threshold overrides | **Fully compatible** |
| `NO_COLOR`, `SDI_LOG_LEVEL`, `SDI_WORKERS` env vars | **Fully compatible** |
| Exit codes 0, 1, 2, 10 | **Same semantics** |
| CLI subcommands (`init`, `snapshot`, `check`, `show`, `diff`, `trend`) | **Same interface** |
| Boundary management (`boundaries infer/ratify/show`) | **New in sdi-rust** |

## What Changes

### Snapshot Schema (breaking)

sdi-rust uses `snapshot_version: "1.0"`. sdi-py used `snapshot_version: "0.1.0"`.
sdi-rust **does not read** sdi-py snapshot files.

**Effect:** All trend history is lost on migration. `sdi trend` will show no
data until two or more sdi-rust snapshots have accumulated. This is an
intentional clean break (KDD-1) — sdi-py snapshots are tool-generated and
trivially regeneratable.

**Mitigation:** If you need trend continuity, run `sdi snapshot --commit <sha>`
against each historical commit before going live. There is no automated
backfill command; use a shell loop.

### Exit Code 3 (new)

sdi-rust exits `3` when all detected languages in the repository lack
tree-sitter grammars (e.g. a repo containing only `.xyz` files with no
registered adapter). sdi-py had no equivalent.

### Pattern Fingerprints

sdi-rust uses `blake3` (keyed hash) for pattern fingerprints. sdi-py used a
different hashing scheme. Fingerprint values from sdi-py snapshots are not
comparable to sdi-rust fingerprints.

### Boundary Inference

`sdi boundaries` is new in sdi-rust. sdi-py had no equivalent subcommand.
The `.sdi/boundaries.yaml` format is compatible, but the inference algorithm
(native Leiden community detection) is not bit-identical to sdi-py's output.

### Coupling Topology Metrics

The `graph_metrics` / `coupling_topology` field names differ between sdi-py
and sdi-rust snapshots. sdi-rust uses:
- `graph.node_count`, `graph.edge_count`, `graph.density`, `graph.cycle_count`
- `partition.community_count()`, `partition.modularity`, `partition.seed`

sdi-py used a flat `graph_metrics` object.

### `sdi boundaries ratify` Loses YAML Comments

When `sdi boundaries ratify` rewrites `.sdi/boundaries.yaml`, all YAML
comments are lost. This is a known limitation (KDD-6) accepted for the MVP.

**Workaround:** Keep explanatory comments in a separate document and reference
them by boundary name. Do not run `ratify` on a hand-edited file.

See the [YAML comment loss section](#yaml-comment-loss) below.

## What is NOT Affected

- Manual edits to `.sdi/config.toml` or `.sdi/boundaries.yaml`
- The `.sdi/` directory layout (`snapshots/`, `cache/`, `boundaries.yaml`)
- The meaning and semantics of threshold values
- The `expires` requirement on per-category overrides

## Migration Steps

```sh
# 1. Install sdi-rust
cargo install sdi-cli

# 2. Verify config compatibility
sdi --repo /path/to/your/repo init   # rewrites config only if missing

# 3. Take a first baseline snapshot
sdi snapshot --commit "$(git rev-parse HEAD)"

# 4. Verify the check gate
sdi check
```

## Comparing Results Against sdi-py

When validating sdi-rust output against an sdi-py baseline on the same repo at
the same commit, expect the following metric tolerances:

| Metric | Acceptable variance |
|---|---|
| Modularity | Within 1% |
| Community count | Within ±10% |
| Pattern entropy | Within 5% |

These tolerances exist because the native Leiden port (KDD-2) is verified for
partition quality, not bit identity with the Python `leidenalg` library. Pattern
entropy can differ slightly because sdi-rust's tree-sitter normalisation rules
are stricter than the Python POC's heuristic walk.

## YAML Comment Loss

### What changed

`sdi boundaries ratify` writes `.sdi/boundaries.yaml` programmatically using
`serde_yml`. All YAML comments are lost whenever ratify overwrites the file.

### What you will see

```
sdi: warning: '.sdi/boundaries.yaml' contains YAML comments — comments will be
lost after ratify (see docs/migrating-from-sdi-py.md)
```

The command still succeeds (exit 0). The comment-stripped version is written
atomically.

### Why this happens

Comment-preserving YAML round-trips require a hand-written emitter or an
immature crate; neither is acceptable for the MVP quality bar (KDD-6).

### Workarounds

- Keep comments in a separate doc and link from boundary names
- Do not run `ratify` on a hand-edited file; use `ratify` only on fresh output
- Version-control the file — deleted comments can be recovered from git history
