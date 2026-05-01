# sdi-snapshot

Snapshot assembly, delta, and persistence for [sdi-rust](https://github.com/geoffgodwin/sdi-rust).

Provides `assemble_snapshot`, `compute_delta` (returns `null` per-dimension on first snapshot),
`compute_trend`, and `infer_boundaries`. Atomic snapshot writes live in `sdi-pipeline`.

Part of the `sdi-rust` workspace. See the [workspace README](../../README.md) for full documentation.
