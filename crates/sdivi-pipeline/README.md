# sdivi-pipeline

Orchestration pipeline for the [Structural Divergence Indexer](https://github.com/geoffgodwin/sdivi-rust).
Owns filesystem access, clock, and atomic snapshot writes.

Exposes `Pipeline::snapshot(&Path)` — the five-stage sequential pipeline
(parsing → graph → detection → patterns → snapshot/delta).

Part of the `sdivi-rust` workspace. See the [workspace README](../../README.md) for full documentation.
