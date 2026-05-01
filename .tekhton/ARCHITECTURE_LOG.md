# Architecture Decision Log

Accepted Architecture Change Proposals are recorded here for institutional memory.
Each entry captures why a structural change was made, preventing future developers
(or agents) from reverting to the old approach without understanding the context.

## ADL-1: sdi-rust meta-crate has no `[[bin]]` section (Task: "M01")
- **Date**: 2026-04-28
- **Rationale**: Two workspace crates cannot both declare `[[bin]] name = "sdi"`; KD12 gives the binary to `sdi-cli`. The lib-only meta-crate with `pub use sdi_core as core` is the correct name-reservation pattern. In
- **Source**: Accepted ACP from pipeline run

## ADL-2: Add `path_partition: BTreeMap<String, u32>` to `Snapshot` (Task: "Implement Milestone 10: Boundaries — Infer, Ratify, Show")
- **Date**: 2026-04-30
- **Rationale**: - ACP: Add `path_partition: BTreeMap<String, u32>` to `Snapshot` — ACCEPT. Backward-compatible (`serde(default, skip_serializing_if)`). Correctly populated in `sdi-pipeline`, never in `sdi-core` (WA
- **Source**: Accepted ACP from pipeline run
