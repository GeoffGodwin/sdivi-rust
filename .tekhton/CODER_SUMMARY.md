# Coder Summary
## Status: COMPLETE

## What Was Implemented

**Milestone 26 — Resolver: Parent Navigation and Per-Language Module Conventions**

Rewrote the dependency-graph import resolver to actually navigate parent
directories and dispatch per language. The old code stripped `../` and `super::`
characters but never walked up the directory tree, so all parent-relative imports
produced zero edges.

### Changes

**New files:**
- `crates/sdivi-graph/src/resolve.rs` (299 lines) — main dispatcher with helpers:
  `extensions_for_language`, `build_stem_map`, `compute_java_roots`,
  `resolve_imports`, `lang_from_path_ext`, `try_path`, `resolve_relative`,
  `resolve_super`, `find_stem_in_subtree`.
- `crates/sdivi-graph/src/resolve_lang.rs` (140 lines) — Python/Go/Java resolvers:
  `resolve_python_bare`, `resolve_python_relative`, `resolve_go_module`,
  `resolve_java_dotted`.
- `crates/sdivi-graph/tests/resolver_unit.rs` (285 lines) — 17 unit tests covering
  all resolver paths.
- `crates/sdivi-graph/tests/integration_real_world.rs` (99 lines) — multi-language
  synthetic test (TS + Python + Go + Java, 8 edges).
- `crates/sdivi-graph/tests/per_language_baselines.rs` (128 lines) — pinned
  per-language edge-count baselines post-M26.

**Modified files:**
- `crates/sdivi-graph/src/dependency_graph.rs` — removed old resolver; added
  `build_dependency_graph_with_go_module(records, go_module: Option<&str>)`;
  `build_dependency_graph` delegates with `None`.
- `crates/sdivi-graph/src/lib.rs` — added `resolve` and `resolve_lang` modules;
  exported `build_dependency_graph_with_go_module`.
- `crates/sdivi-graph/Cargo.toml` — added dev-deps for all language adapter crates
  and `sdivi-config`.
- `crates/sdivi-pipeline/src/pipeline.rs` (299 lines) — reads `go.mod` at Stage 2,
  calls `build_dependency_graph_with_go_module`.
- `crates/sdivi-graph/tests/dependency_graph.rs` (299 lines) — updated two tests for
  M26 semantics.
- `crates/sdivi-parsing/tests/import_extraction.rs` — updated edge-count expectations.
- `CHANGELOG.md` — M26 entry added to `[Unreleased]` Fixed section.
- `MIGRATION_NOTES.md` — M26 re-baseline guidance prepended to the 0.2.x→0.3.0 section.

## Root Cause (bugs only)

Two bugs in the old `resolve_relative` and `resolve_import`:
1. `../` stripping: the code removed leading `../` tokens but left `base` pointing
   at the importer's own directory — parent-relative imports resolved to nothing.
2. No language dispatch: all non-Rust, non-relative imports were dropped silently;
   Python dotted, Go module-path, and Java dotted specifiers had no handler.

## Design Decisions

- **`lang_from_path_ext`** infers effective language from the importer's file
  extension for `./`/`../` imports, preserving backward compatibility for test
  fixtures that carry `language:"rust"` with `.ts`/`.py` paths.
- **`build_dependency_graph_with_go_module`** exposes Go module prefix as a parameter
  so non-pipeline callers (WASM, tests) can supply it without touching the
  filesystem. The pipeline reads `go.mod` in `pipeline.rs` — FS I/O never enters
  `sdivi-graph` (Rule 22).
- **`compute_java_roots`** discovers multi-module Maven roots dynamically from
  `path_to_node` keys to avoid hardcoding module names.
- **File-over-directory tie-break:** `try_path` tries `{rem}.{ext}` before
  `{rem}/{index}` within each language's extension list.

## Human Notes Status

No explicit human notes beyond the milestone spec.

## Files Modified (auto-detected)
- `.claude/milestones/MANIFEST.cfg`
- `.claude/milestones/m26-resolver-parent-navigation-and-language-conventions.md`
- `.github/workflows/wasm.yml`
- `.tekhton/CODER_SUMMARY.md`
- `.tekhton/DRIFT_LOG.md`
- `.tekhton/HUMAN_ACTION_REQUIRED.md`
- `.tekhton/test_dedup.fingerprint`
- `CHANGELOG.md`
- `Cargo.lock`
- `MIGRATION_NOTES.md`
- `bindings/sdivi-wasm/package.json`
- `crates/sdivi-core/src/categories.rs`
- `crates/sdivi-graph/Cargo.toml`
- `crates/sdivi-graph/src/dependency_graph.rs`
- `crates/sdivi-graph/src/lib.rs`
- `crates/sdivi-graph/tests/dependency_graph.rs`
- `crates/sdivi-lang-go/src/extract.rs`
- `crates/sdivi-lang-java/src/extract.rs`
- `crates/sdivi-lang-javascript/src/extract.rs`
- `crates/sdivi-lang-python/src/extract.rs`
- `crates/sdivi-lang-typescript/src/extract.rs`
- `crates/sdivi-parsing/Cargo.toml`
- `crates/sdivi-parsing/src/lib.rs`
- `crates/sdivi-parsing/tests/import_extraction.rs`
- `crates/sdivi-pipeline/src/pipeline.rs`
- `tools/release.sh`
