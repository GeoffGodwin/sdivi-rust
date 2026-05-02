# sdivi-snapshot

Snapshot assembly, delta, and persistence for [sdivi-rust](https://github.com/geoffgodwin/sdivi-rust).

Provides `assemble_snapshot`, `compute_delta` (returns `null` per-dimension on first snapshot),
`compute_trend`, and `infer_boundaries`. Atomic snapshot writes live in `sdivi-pipeline`.

Part of the `sdivi-rust` workspace. See the [workspace README](../../README.md) for full documentation.
