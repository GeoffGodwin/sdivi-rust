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
- **Implemented in M22 (2026-05-02)**: `WasmChangeCouplingInput` + `WasmCoChangePairInput` added to `assemble_types.rs`; `change_coupling: Option<WasmChangeCouplingInput>` field added to `WasmAssembleSnapshotInput`; `exports.rs` now converts and threads the value through to `sdivi_core::assemble_snapshot`. TODO comment removed. ADL-7 is resolved.

## ADL-9: Weighted Leiden exposed in WASM via "source:target" colon keys (Task: "M21")
- **Date**: 2026-05-02
- **Rationale**: `WasmLeidenConfigInput.edge_weights` uses colon-separated keys (`"source:target"`) rather than the native NUL-separated keys (`edge_weight_key`) because NUL is not a valid JSON string character in practice and is opaque to JS callers. The WASM binding layer (`weight_keys.rs`) converts colon keys to NUL keys before passing them to `sdivi_core::detect_boundaries`. The first-colon-wins split rule (splitn 2) means node IDs that themselves contain colons are fully supported. ADL-4 (serde_json round-trip conversion) remains valid; weighted fields bypass the round-trip via explicit extraction before `to_core`. Removes the MVP "unweighted only" limitation recorded informally in `bindings/sdivi-wasm/src/types.rs:46–48`.
- **Source**: Accepted ACP from M21 pipeline run

## ADL-8: pub mod internal (Task: "M17")
- **Date**: 2026-05-02
- **Rationale**: Standard Rust test-plumbing pattern (`#[doc(hidden)]` + explicit "not stable API" prose). Items are in `sdivi-detection`, not in `sdivi-core` (the API-stability boundary). Well-implemented: all re-exp
- **Source**: Accepted ACP from pipeline run

## ADL-10: `assemble_snapshot` takes `boundary_count: Option<usize>`, not `Option<&BoundarySpec>` (Task: "HUMAN_ACTION_REQUIRED 2026-05-02 sweep")
- **Date**: 2026-05-03
- **Rationale**: `sdivi_snapshot::assemble_snapshot` previously took `Option<&sdivi_config::BoundarySpec>` as its fifth parameter and read only `spec.boundaries.len()` from it. Two consequences flowed from this over-specified signature: (1) `sdivi-snapshot` carried a dependency on `sdivi-config` solely to name a type it never used beyond a single `.len()` call; (2) the WASM binding could not construct a `BoundarySpec` (it lives in `sdivi-config`'s YAML loader path) and worked around the gap by calling `assemble_snapshot` with `boundary_spec = None` and then mutating `snap.intent_divergence` after the fact (`bindings/sdivi-wasm/src/exports.rs:181-198`). That created a second `IntentDivergenceInfo` assembly seam outside the canonical function — a class of bug that has previously recurred across milestones. The architect raised this as a HUMAN_ACTION_REQUIRED item on 2026-05-02 with three options: A (additive sibling function), B (replace the parameter type, breaking change), C (accept the seam permanently). Option B was accepted: the cleanest separation of concerns is for the assembly function to ask for the integer it actually needs, and for callers (pipeline, WASM, embedders) to supply that integer from whatever boundary representation they own. The pipeline computes `boundary_spec.as_ref().map(|s| s.boundaries.len())` inline; WASM passes its `boundary_count: Option<u32>` field through directly. The breaking change is acceptable pre-1.0 (the project is treated as alpha; no external production consumers are pinned to the prior signature beyond Meridian, which uses the WASM binding and is unaffected at the JS surface). Snapshot JSON output is byte-identical — only the Rust API surface changed. Side effect: `sdivi-snapshot` no longer depends on `sdivi-config`. Recorded in `CHANGELOG.md` under `[0.2.0]` and `MIGRATION_NOTES.md` under `0.1.x → 0.2.0`.
- **Source**: Human decision on HUMAN_ACTION_REQUIRED item dated 2026-05-02 (architect-raised)
