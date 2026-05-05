# Migration notes

Breaking-change migration guidance for sdivi-rust adopters. Each `0.x → 0.(x+1)`
bump that touches stable surface gets an entry. Post-1.0, the same applies to
major-version bumps.

For the broader migration story from the Python POC (`sdi-py`), see
[`docs/migrating-from-sdi-py.md`](docs/migrating-from-sdi-py.md).

## 0.2.x → 0.3.0 (M25 adapter fix — no schema break)

### Import specifier extraction: substantial edge-count increase on non-Rust projects

**What changed.** Language adapters for Python, TypeScript, JavaScript, Go, and
Java previously emitted whole import-statement text into `FeatureRecord::imports`
(e.g. `"import { foo } from '../lib/x'"`). The graph resolver silently dropped
every such string, producing zero cross-file edges for all five languages.
Adapters now emit only the module specifier (e.g. `"../lib/x"`). Edges that
were previously invisible now resolve, and all coupling-based metrics become
meaningful.

**Schema impact.** None. `snapshot_version` stays `"1.0"`. Pre-M25 snapshots
are still readable; the change affects only the content of future snapshots.

**Baseline impact.** The first `sdivi snapshot` after upgrading will produce
a large `coupling_delta` and `community_count_delta` against any pre-M25
baseline on a Python/TS/JS/Go/Java project. `boundary_violation_rate` will
likely increase if you have a `.sdivi/boundaries.yaml` declared, because
violations that were previously undetectable now appear.

**Recommended migration:**

Option A — re-baseline (cleanest):
```bash
rm .sdivi/snapshots/*.json   # clear old baselines
sdivi snapshot               # first snapshot under new adapter
```

Option B — one-time override (preserves trend history):
```toml
# .sdivi/config.toml — expires after the spike settles
[thresholds.overrides.coupling]
coupling_delta_rate = 50.0
expires = "2026-06-01"
reason = "M25 import-specifier fix; first post-upgrade snapshot has large coupling_delta"

[thresholds.overrides.boundaries]
boundary_violation_rate = 20.0
expires = "2026-06-01"
reason = "M25 import-specifier fix; first post-upgrade snapshot may spike violations"
```

## 0.1.x

No breaking changes between 0.1.0 and 0.1.14. Every release in the 0.1 line is
backwards-compatible at the public-API and snapshot-schema level. New `Input`
fields are added with `#[serde(default)]` and new snapshot fields are
additive.

The 0.1.7 algorithm correction in the Leiden refinement phase is not a public
API break. It does invalidate trend continuity across the 0.1.6 / 0.1.7
boundary because pre-0.1.7 snapshots have a `modularity` value derived from
the broken refinement. See `CHANGELOG.md` 0.1.7 entry.

## 0.1.x → 0.2.0

### `assemble_snapshot` parameter type change

**What changed.** The fifth positional parameter of
`sdivi_snapshot::assemble_snapshot` (re-exported as
`sdivi_core::assemble_snapshot`) is now `boundary_count: Option<usize>` instead
of `boundary_spec: Option<&sdivi_config::BoundarySpec>`. The function no longer
reaches into a `BoundarySpec` to read `.boundaries.len()`; the caller does that
inline (or supplies the count from any other source).

**Why.** Two reasons:

1. The function only ever read one integer (`spec.boundaries.len()`) from the
   spec; the rest of the type was dead weight in the signature. Asking for the
   bag instead of the integer it contains was unnecessary coupling.
2. WASM and other non-FS callers cannot construct a `BoundarySpec` (it lives in
   `sdivi-config` and is parsed from YAML). The previous WASM binding worked
   around this by calling `assemble_snapshot` with `None` and then mutating
   `snap.intent_divergence` after the fact — a second assembly seam outside the
   canonical function. That seam is gone now.

A side effect: `sdivi-snapshot` no longer depends on `sdivi-config`.

**What to do.** Mechanical replacement at every call site.

```diff
-let snap = assemble_snapshot(
-    graph, partition, catalog, pattern_metrics,
-    boundary_spec.as_ref(),
-    &timestamp, commit, change_coupling, violation_count,
-);
+let boundary_count = boundary_spec.as_ref().map(|spec| spec.boundaries.len());
+let snap = assemble_snapshot(
+    graph, partition, catalog, pattern_metrics,
+    boundary_count,
+    &timestamp, commit, change_coupling, violation_count,
+);
```

Callers that already passed `None` need no change — `None` continues to mean
"omit `intent_divergence` from the snapshot."

**Trend continuity.** Unaffected. Snapshot JSON output is byte-identical for
the same inputs; the change is purely at the Rust API surface. Snapshots
written by 0.1.x can be loaded and diffed by 0.2.0 without conversion.

## Future entries

When a breaking change lands, document:

- **What changed.** A precise description of the renamed, removed, or
  resemanticised item.
- **Why.** The motivation. Often a correctness fix or a SemVer-mandated
  cleanup.
- **What to do.** A concrete migration recipe. A diff or `sed` snippet
  beats prose.
- **Trend continuity.** Whether snapshots from prior versions are still
  comparable.
