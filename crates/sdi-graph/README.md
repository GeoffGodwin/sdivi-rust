# sdi-graph

Dependency graph construction for [sdi-rust](https://github.com/geoffgodwin/sdi-rust).

Builds a `petgraph`-backed `DependencyGraph` from parsed `FeatureRecord`s (pipeline path)
or from a `DependencyGraphInput` struct (pure-compute / WASM path).

Part of the `sdi-rust` workspace. See the [workspace README](../../README.md) for full documentation.
