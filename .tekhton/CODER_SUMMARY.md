# Coder Summary
## Status: COMPLETE

## What Was Implemented
- Rewrote `extract_imports` in Python, TypeScript, JavaScript, Go, and Java adapters to emit
  module specifiers instead of whole-statement text
- Added per-language helper functions for specifier extraction
- Updated adapter unit tests to assert specifier strings, added coverage for all import variants
- Added 2 new shape-assertion tests to `crates/sdivi-graph/tests/dependency_graph.rs`
- Created per-language fixture directories with source files
- Added `crates/sdivi-parsing/tests/import_extraction.rs` integration test with pinned edge counts
- Updated `crates/sdivi-parsing/Cargo.toml` dev-deps to include non-Rust adapters and sdivi-graph
- Updated `CHANGELOG.md` and `MIGRATION_NOTES.md`
- Trimmed all modified files to under 300 lines (pre-completion self-check)

## Root Cause (bugs only)
Each non-Rust adapter called `node.utf8_text(source)` on the top-level import statement node
and pushed the resulting whole-statement text (e.g. `"import { foo } from '../lib/x';"`) into
`FeatureRecord::imports`. The graph resolver's `resolve_import` then dropped every such string
because it doesn't start with `./`, `../`, `crate::`, `self::`, or `super::`, producing zero
cross-file edges for every non-Rust language.

## Files Modified
- `crates/sdivi-lang-python/src/extract.rs` — rewrite extract_imports to emit specifiers
- `crates/sdivi-lang-typescript/src/extract.rs` — rewrite extract_imports to emit string_fragment
- `crates/sdivi-lang-javascript/src/extract.rs` — rewrite extract_imports; add require() + dynamic import()
- `crates/sdivi-lang-go/src/extract.rs` — rewrite extract_imports to emit path strings per import_spec
- `crates/sdivi-lang-java/src/extract.rs` — rewrite extract_imports to emit scoped_identifier text
- `crates/sdivi-lang-python/tests/extract_behavior.rs` — updated assertions, new variant tests
- `crates/sdivi-lang-typescript/tests/extract_behavior.rs` — updated assertions, new variant tests
- `crates/sdivi-lang-javascript/tests/extract_behavior.rs` — updated assertions, require/dynamic import tests
- `crates/sdivi-lang-go/tests/extract_behavior.rs` — updated assertions, grouped/aliased/blank/dot tests
- `crates/sdivi-lang-java/tests/extract_behavior.rs` — updated assertions, wildcard/static tests
- `crates/sdivi-graph/tests/dependency_graph.rs` — 2 new shape-assertion tests; trimmed to 299 lines
- `crates/sdivi-parsing/Cargo.toml` — added dev-deps
- `tests/fixtures/simple-python/` — (NEW) Python fixture files
- `tests/fixtures/simple-typescript/` — (NEW) TypeScript fixture files
- `tests/fixtures/simple-javascript/` — (NEW) JavaScript fixture files
- `tests/fixtures/simple-go/` — (NEW) Go fixture files
- `tests/fixtures/simple-java/` — (NEW) Java fixture files
- `crates/sdivi-parsing/tests/import_extraction.rs` — (NEW) multi-language integration test
- `CHANGELOG.md` — Fixed entry for M25
- `MIGRATION_NOTES.md` — Re-baseline guidance for existing users

## Human Notes Status
No explicit human notes listed beyond milestone spec.

## Docs Updated
None — no public-surface changes in this task. The `FeatureRecord::imports` field semantics
changed (specifiers instead of statement text), but this is an internal pipeline type not part
of the documented public API surface (`sdivi-core` compute functions, CLI flags, config keys,
or snapshot JSON schema). The CHANGELOG.md and MIGRATION_NOTES.md entries added in the previous
coder run document the behavior change for users who snapshot non-Rust repos.

## Observed Issues (out of scope)
