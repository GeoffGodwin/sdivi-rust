---
title: Determinism
---

# Determinism in sdivi-rust

sdivi-rust is a measurement instrument. Its core guarantee is that the same repo
state + the same config always produces the same snapshot. This document
describes how that guarantee is implemented and what its limits are.

## BTreeMap Discipline

**Rule:** `BTreeMap` is used everywhere output ordering matters. `HashMap` is
permitted only in hot-path internal structures where iteration order does not
influence output.

Specifically:
- `PatternCatalog.entries: BTreeMap<String, BTreeMap<String, PatternStats>>`
- `Snapshot` fields containing per-file or per-category data
- The `path_partition: BTreeMap<String, u32>` in `Snapshot`
- All `*Input` structs in `sdivi-core::input`

The consequence: sorted output is part of the contract. JSON snapshots emit
keys in lexicographic order. Foreign extractors feeding `DependencyGraphInput`
do not need to pre-sort — the compute functions re-sort internally.

## Refinement Tie-Break Rule

When multiple candidate sub-communities have equal modularity gain during the
Leiden refinement phase, the tie is broken by **smallest sub-community ID** (the
`BTreeMap` over `(comm_id → k_in)` iterates in ascending key order, so the
lowest-ID candidate wins). This produces deterministic output for a given seed
but may select a different partition than leidenalg's random-selection rule
(`exp(ΔQ/θ)` probability). The difference is within the 1 % modularity
tolerance of the `verify-leiden` fixture suite. Faithful Traag-2019 random
selection is filed as a future milestone.

## Seed Contract

`Config::random_seed` (default `42`) controls the `StdRng` used by the Leiden
community-detection algorithm. The seed travels through:

1. `Config::random_seed` → `LeidenConfig::seed` (set in `LeidenConfig::from_sdivi_config`)
2. `LeidenConfig::seed` → `StdRng::seed_from_u64(seed)` at the start of each Leiden run
3. The `seed` is recorded in `LeidenPartition.seed` and in the snapshot JSON

**Invariant:** same seed + same graph → bit-identical partition assignment.
This is property-tested in `crates/sdivi-detection/tests/proptest_seeded.rs`.

**For WASM callers:** pass `LeidenConfigInput { seed: 42, .. }` explicitly.
Do not rely on the default if reproducibility across runs matters.

## Pattern Fingerprints

Pattern fingerprints use `blake3` with a fixed key constant (`FINGERPRINT_KEY`
in `sdivi-patterns::fingerprint`, re-exported as `sdivi_core::FINGERPRINT_KEY`).

`normalize_and_hash(kind, children)` performs a depth-first canonical walk:

```
preimage(node) = kind_bytes + 0x00 + (0x01 + child_digest_bytes)*
```

Properties:
- Leaf nodes: `preimage = kind_bytes + 0x00`
- Empty `children` produces the same digest as `fingerprint_node_kind(kind)`
- Order of children matters (depth-first, left to right)
- The constant is never changed within a `snapshot_version`; changing it
  invalidates all existing fingerprints and requires a snapshot version bump

## normalize_and_hash for Foreign Extractors

Callers that supply their own AST extractors (e.g. the consumer-app consumer app,
WASM-mediated TS tools) **must** use `sdivi_core::normalize_and_hash` to produce
fingerprints, not a custom hasher.

**NodeId canonicalization rule:** node IDs must be repo-relative UNIX paths
with forward slashes and no leading `./` or `/`. Use `validate_node_id` to
validate before submitting to `compute_*`.

```rust
use sdivi_core::{normalize_and_hash, validate_node_id};
use sdivi_core::input::NormalizeNode;

let child = NormalizeNode { kind: "identifier".into(), children: vec![] };
let fp = normalize_and_hash("call_expression", &[child]);
// fp is a 64-character lowercase hex string — byte-identical in native Rust
// and WASM for the same input.
```

## FMA and Cross-Platform Floating-Point

sdivi-rust does not bit-guarantee floating-point results across platforms. The
Leiden modularity objective (`Q`) uses floating-point arithmetic; the compiler
may emit FMA instructions on x86_64 and aarch64 that round differently from
non-FMA code.

**Practical implication:** aggregate metric values (`total_entropy`,
`convention_drift`, `density`) may differ by ≤ 1 ULP between x86_64 and
aarch64 CI runs. This is documented and expected. The cross-platform guarantee
is **aggregate equality within 5%**, not bit identity.

If a consumer needs bit-identical results across platforms, they can:
1. Disable FMA via a target feature flag: `RUSTFLAGS="-C target-feature=-fma"`
2. Use `f64::mul_add` explicitly where the discrepancy matters

Both options are supported but not the default. They will be revisited if a
real adopter requires cross-platform bit identity.

## Threshold Gate Stability

`compute_thresholds_check` uses `delta > limit + THRESHOLD_EPSILON` rather than
`delta > limit` for every dimension comparison. The constant is:

```rust
pub const THRESHOLD_EPSILON: f64 = 1e-9;
```

**Rationale:** User-configured thresholds are typically expressed to 1–2 decimal
places (e.g. `coupling_delta_rate = 0.15`). Documented per-arch FMA drift
between x86_64 and aarch64 is ≤ 1 ULP ≈ `2.2e-16` near 1.0. A delta computed
as `0.04999…` on x86_64 and `0.05000…01` on aarch64 for the same logical value
would cause the CI gate to flip sign across platforms — a flaky test with no
user-meaningful cause. The `1e-9` epsilon is ~7 orders of magnitude larger than
per-arch FMA drift and ~7 orders of magnitude smaller than any plausible user
precision, so it absorbs the noise without changing semantics.

**Asymmetry:** the epsilon is added to `limit`, not subtracted from `delta`. The
gate is *slightly more lenient* by at most `1e-9`. A genuine breach of
`limit + 2e-9` or more still trips. False positives from ULP noise are
suppressed; false negatives at the ULP scale are acceptable.

**Raw delta is unaffected:** `ThresholdBreachInfo.actual` always holds the
unrounded delta value. Consumers reading the JSON breach report see the true
computed value.

**Integer dimensions:** `boundary_violation_delta` is cast from `i64` to `f64`
before comparison, so it is always an exact integer. The epsilon has no
functional effect there but is applied for consistency.

Re-exported as `sdivi_core::THRESHOLD_EPSILON` so WASM callers and other
embedders can reference the same constant in their own documentation.

## Pure-Function Guarantee

Every `sdivi_core::compute_*` function is referentially transparent:

| Function | Guarantee |
|---|---|
| `compute_coupling_topology` | Same graph input → same result |
| `compute_pattern_metrics` | Same pattern instances → same entropy/drift |
| `compute_thresholds_check` | Same summary + thresholds → same breaches |
| `compute_boundary_violations` | Same graph + boundaries → same violations |
| `detect_boundaries` | Same graph + config + prior → same partition |
| `compute_delta` | Same two snapshots → same divergence summary |
| `infer_boundaries` | Same prior partitions + config → same proposals |
| `normalize_and_hash` | Same kind + children → same blake3 hex |

None of these functions:
- Read from disk
- Call `SystemTime::now()`
- Access global mutable state
- Use `thread_rng` or any non-seeded RNG

`detect_boundaries` (Leiden) uses the seeded `StdRng` described above.
`compute_thresholds_check` takes `today: NaiveDate` as a parameter (not the
system clock) so callers control the date used to evaluate `expires` fields.

## Property Tests

The following proptest suites are permanent CI fixtures:

| Test | Crate | What it validates |
|---|---|---|
| `prop_test_leiden_seeded` | `sdivi-detection` | Same seed → same partition |
| `prop_delta_referentially_transparent` | `sdivi-snapshot` | `compute_delta` is pure |
| `prop_test_normalize_and_hash_stable` | `sdivi-patterns` | Same node → same fingerprint |
| `prop_test_compute_thresholds_check_pure` | `sdivi-core` | Thresholds check is pure |

Regression files are committed in `proptest-regressions/` subdirectories.
If proptest shrinks a failing case, commit the resulting `.txt` file so the
regression is checked on every subsequent CI run.

## Change-coupling determinism

`git log` output ordering is deterministic for a fixed `HEAD` ref. The
`collect_cochange_events` function does not reorder events; it reverses
the git log output (newest-first → oldest-first) so that
`compute_change_coupling` operates on a consistent oldest-first slice.

Cross-platform: git paths use forward slashes on all platforms in its
internal representation. The `canonicalize_path` function strips any
leading `./` and converts backslashes to forward slashes. Combined with
`BTreeMap`-ordered output from `compute_change_coupling`, the
`ChangeCouplingResult` is byte-identical on Linux, macOS, and Windows for
the same repo state.

Weighted Leiden verification against `leidenalg`'s weighted mode is
out of scope for v0 — KDD-2's tolerance is partition quality, not
bit-identity, and weighted-mode parity is best deferred to v0.x.
