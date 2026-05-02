---
title: Snapshot schema
---

# Snapshot schema

Snapshots are JSON, tagged with `"snapshot_version": "1.0"`, and written
atomically (tempfile + rename) under `.sdivi/snapshots/`. Reading a snapshot
with an incompatible `snapshot_version` produces a stderr warning and baseline
treatment (no delta) and never crashes.

## Top-level shape

```jsonc
{
  "snapshot_version": "1.0",
  "timestamp":        "2026-05-01T12:34:56Z",
  "commit_sha":       "abcdef…",            // optional

  "graph":            { /* node_count, edge_count, hub_nodes, ... */ },
  "partition":        { /* clusters, modularity, stability_score, ... */ },
  "catalog":          { /* per-category PatternStats with fingerprints */ },
  "pattern_metrics":  { /* total_entropy, convention_drift, ... */ },

  "boundary_violations": { /* present iff boundaries.yaml is defined */ },
  "intent_divergence":   { /* present iff boundaries.yaml is defined */ },
  "change_coupling":     { /* present iff git history available */ },

  "path_partition":      { "src/foo.rs": 0, "src/bar.rs": 1 }
}
```

## Field-by-field reference

### `graph`

| Field | Type | Description |
|---|---|---|
| `node_count` | `u32` | Total nodes in the dependency graph |
| `edge_count` | `u32` | Total directed import edges |
| `density` | `f64` | `edges / (n * (n-1))` |
| `hub_nodes` | `[String]` | Top-degree nodes by import count |
| `cycle_count` | `u32` | Strongly-connected components of size > 1 |

### `partition`

| Field | Type | Description |
|---|---|---|
| `cluster_count` | `u32` | Number of detected communities |
| `cluster_assignments` | `BTreeMap<String, u32>` | Node ID to cluster ID |
| `modularity` | `f64` | Modularity score of the partition |
| `stability_score` | `f64` | Fraction of nodes whose assignment matches the prior snapshot |
| `seed` | `u64` | RNG seed used (records the value of `Config::random_seed`) |

### `catalog`

```jsonc
{
  "control_flow": {
    "if_chain":     { "fingerprint": "blake3:...", "occurrences": 23 },
    "match_arm":    { "fingerprint": "blake3:...", "occurrences": 18 }
  },
  "error_handling": { ... }
}
```

Categories are stable across snapshots. New categories may be added without
bumping `snapshot_version`. Removed categories require a major version bump.

### `pattern_metrics`

| Field | Type | Description |
|---|---|---|
| `total_entropy` | `f64` | Aggregate Shannon entropy across the catalog |
| `convention_drift` | `f64` | Average per-category drift |
| `pattern_entropy_per_category` | `BTreeMap<String, f64>` | Entropy split by category |
| `convention_drift_per_category` | `BTreeMap<String, f64>` | Drift split by category |

### `boundary_violations`

Present only when `.sdivi/boundaries.yaml` is declared. Each entry includes
the source node, target node, and the boundary that the import crosses.

### `intent_divergence`

Aggregate count of imports that violate declared boundaries, normalised by
edge count. Present only when boundaries are declared.

### `change_coupling`

```jsonc
{
  "pairs": [
    { "source": "src/auth.rs", "target": "src/session.rs", "frequency": 0.74 }
  ],
  "config": { "min_frequency": 0.6, "history_depth": 500 }
}
```

Present when git history is available and `change_coupling` is enabled in
config. Pairs are filtered by `min_frequency`.

### `path_partition`

Sorted `BTreeMap<String, u32>` mapping every analysed file to its cluster ID.
This is the field downstream tools use to align snapshot results with their
own file lists.

## Delta semantics

`compute_delta(prev, curr)` returns a `DivergenceSummary`:

- First snapshot (no prior): every per-dimension field is `null`.
- Two identical snapshots: every per-dimension field is `0`.

`null` and `0` are distinct on purpose. `null` means "no comparison was
possible". `0` means "compared and unchanged". Treating them the same is a
semantic bug masquerading as a numeric one.

## Atomic write contract

`Pipeline::snapshot` writes to a tempfile in the target snapshot directory
and then renames it into place. A killed process never leaves a half-written
`.json` file in `.sdivi/snapshots/`. Retention is enforced after the rename
succeeds. A failed write does not remove an existing snapshot.

## Schema stability

`snapshot_version` is the literal string `"1.0"` for every release in the
0.x line. Bumping it is a breaking change requiring a major version bump and
a `MIGRATION_NOTES.md` entry. Additive fields (new keys with
`#[serde(default)]`) are not breaking and do not require a version bump.
