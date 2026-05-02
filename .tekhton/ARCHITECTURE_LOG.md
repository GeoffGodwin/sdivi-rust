# Architecture Decision Log

Accepted Architecture Change Proposals are recorded here for institutional memory.
Each entry captures why a structural change was made, preventing future developers
(or agents) from reverting to the old approach without understanding the context.

## ADL-1: sdivi-rust meta-crate has no `[[bin]]` section (Task: "M01")
- **Date**: 2026-04-28
- **Rationale**: Two workspace crates cannot both declare `[[bin]] name = "sdivi"`; KD12 gives the binary to `sdivi-cli`. The lib-only meta-crate with `pub use sdivi_core as core` is the correct name-reservation pattern. In
- **Source**: Accepted ACP from pipeline run

## ADL-2: Add `path_partition: BTreeMap<String, u32>` to `Snapshot` (Task: "Implement Milestone 10: Boundaries — Infer, Ratify, Show")
- **Date**: 2026-04-30
- **Rationale**: - ACP: Add `path_partition: BTreeMap<String, u32>` to `Snapshot` — ACCEPT. Backward-compatible (`serde(default, skip_serializing_if)`). Correctly populated in `sdivi-pipeline`, never in `sdivi-core` (WA
- **Source**: Accepted ACP from pipeline run

## ADL-3: Re-exporting inner types from sdivi-core (Task: "M12")
- **Date**: 2026-05-01
- **Rationale**: - ACP: Re-exporting inner types from sdivi-core — ACCEPT. Additive re-exports, WASM-safe, required by the KDD-12 architecture where `sdivi-wasm` depends only on `sdivi-core`. The corresponding CLAUDE.md u
- **Source**: Accepted ACP from pipeline run

## ADL-4: WASM wrapper types use serde_json round-trip conversion (Task: "M12")
- **Date**: 2026-05-01
- **Rationale**: - ACP: WASM wrapper types use serde_json round-trip conversion — ACCEPT. Internal to `sdivi-wasm`; avoids ~200 lines of boilerplate `From` impls with no observable behavior difference to callers.
- **Source**: Accepted ACP from pipeline run

## ADL-5: wasm-pack --profile syntax (wasm-pack 0.12+) (Task: "M12")
- **Date**: 2026-05-01
- **Rationale**: - ACP: wasm-pack --profile syntax (wasm-pack 0.12+) — ACCEPT. Correct fix for the `--release` + `--profile` Cargo flag conflict; documented in build.sh and wasm.yml.
- **Source**: Accepted ACP from pipeline run

## ADL-6: `sdivi-cli` exposed as library target to enable `cargo install sdivi-rust` (Task: "Implement Milestone 13: Release Pipeline and Distribution")
- **Date**: 2026-05-01
- **Rationale**: - ACP: `sdivi-cli` exposed as library target to enable `cargo install sdivi-rust` — **ACCEPT** (confirmed from cycle 1; no rework changed this decision).
- **Source**: Accepted ACP from pipeline run

## ADL-7: WASM `change_coupling` field hardcoded to `None` in MVP (Task: "M16 Non-Blocking Notes Sweep")
- **Date**: 2026-05-01
- **Rationale**: WASM bindings for `assemble_snapshot` hardcode `change_coupling: None` because `compute_change_coupling` is not yet exposed in the WASM API. This is an MVP limitation. Post-MVP, add `change_coupling` field to `WasmAssembleSnapshotInput` and expose `compute_change_coupling` as a WASM export. Tracked in `bindings/sdivi-wasm/src/exports.rs:160-162` with TODO comment.
- **Source**: Non-blocking notes from post-M16 review cycle

## ADL-8: pub mod internal (Task: "M17")
- **Date**: 2026-05-02
- **Rationale**: Standard Rust test-plumbing pattern (`#[doc(hidden)]` + explicit "not stable API" prose). Items are in `sdivi-detection`, not in `sdivi-core` (the API-stability boundary). Well-implemented: all re-exp
- **Source**: Accepted ACP from pipeline run
