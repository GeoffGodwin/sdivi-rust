# sdivi-core

Pure-compute facade for the [Structural Divergence Indexer](https://github.com/geoffgodwin/sdivi-rust).
WASM-compatible — no I/O, no clock, no tree-sitter.

Exposes `compute_*` functions over `*Input` serde structs for use from CLI, Rust embedders,
and WASM / the consumer app.

Part of the `sdivi-rust` workspace. See the [workspace README](../../README.md) for full documentation.
