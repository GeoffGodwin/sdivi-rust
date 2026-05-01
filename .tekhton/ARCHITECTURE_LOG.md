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

## ADL-3: Re-exporting inner types from sdi-core (Task: "M12")
- **Date**: 2026-05-01
- **Rationale**: - ACP: Re-exporting inner types from sdi-core — ACCEPT. Additive re-exports, WASM-safe, required by the KDD-12 architecture where `sdi-wasm` depends only on `sdi-core`. The corresponding CLAUDE.md u
- **Source**: Accepted ACP from pipeline run

## ADL-4: WASM wrapper types use serde_json round-trip conversion (Task: "M12")
- **Date**: 2026-05-01
- **Rationale**: - ACP: WASM wrapper types use serde_json round-trip conversion — ACCEPT. Internal to `sdi-wasm`; avoids ~200 lines of boilerplate `From` impls with no observable behavior difference to callers.
- **Source**: Accepted ACP from pipeline run

## ADL-5: wasm-pack --profile syntax (wasm-pack 0.12+) (Task: "M12")
- **Date**: 2026-05-01
- **Rationale**: - ACP: wasm-pack --profile syntax (wasm-pack 0.12+) — ACCEPT. Correct fix for the `--release` + `--profile` Cargo flag conflict; documented in build.sh and wasm.yml.
- **Source**: Accepted ACP from pipeline run

## ADL-6: `sdi-cli` exposed as library target to enable `cargo install sdi-rust` (Task: "Implement Milestone 13: Release Pipeline and Distribution")
- **Date**: 2026-05-01
- **Rationale**: - ACP: `sdi-cli` exposed as library target to enable `cargo install sdi-rust` — **ACCEPT** (confirmed from cycle 1; no rework changed this decision).
- **Source**: Accepted ACP from pipeline run
