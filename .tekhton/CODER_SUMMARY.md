# Coder Summary — M04: Remaining Language Adapters — 2026-04-29
## Status: COMPLETE

## What Was Implemented

### Prior Run (completed before this session)
- **Python adapter** (`sdi-lang-python`): `lib.rs` + `extract.rs` with import/export/signature/hint extraction; 8 tests in `tests/extract_behavior.rs`
- **TypeScript adapter** (`sdi-lang-typescript`): `lib.rs` + `extract.rs` with dual TS/TSX parser; 8 tests in `tests/extract_behavior.rs`
- **JavaScript adapter** (`sdi-lang-javascript`): `lib.rs` + `extract.rs`; 8 tests in `tests/extract_behavior.rs`
- **Go adapter** (`sdi-lang-go`): `lib.rs` + `extract.rs` with capitalized-name export rule; 8 tests in `tests/extract_behavior.rs`
- **Java adapter** (`sdi-lang-java`): `lib.rs` + `extract.rs` with `public` modifier check; 8 tests in `tests/extract_behavior.rs`
- **Rust adapter bug fixes** (`sdi-lang-rust/src/extract.rs`):
  - `collect_hints`: truncation check now `*i + c.len_utf8() <= 256` (was `*i < 256`)
  - `extract_exports`: `continue` after recording a top-level item prevents recursing into nested items
- **load.rs doc comment misplacement** fixed: `load_or_default` and `project_config_path` doc blocks separated
- **ACTIVE_TREES doc comment** updated to accurately describe invocation tracking

### This Session
- Fixed `crates/sdi-cli/tests/version.rs`: updated hardcoded version from `0.0.3` → `0.0.4` (workspace was bumped in prior run)
- Fixed `crates/sdi-parsing/tests/memory_invariant.rs`: added `COUNTER_LOCK: Mutex<()>` to serialize `tree_counter_zero_after_each_parse` and `parse_many_large_files_completes`; both shared the global `ACTIVE_TREES` counter and were failing due to a race when run in parallel by the test runner

## Root Cause (bugs only)
- **Version test failure**: workspace Cargo.toml version was bumped to `0.0.4` in the prior run but `version.rs` test was only updated to `0.0.3`.
- **Memory invariant race**: `parse_many_large_files_completes` uses rayon to parse 20 files in parallel; while those 20 rayon tasks were in-flight, they held the `ACTIVE_TREES` counter at 20. `tree_counter_zero_after_each_parse` ran concurrently and observed `ACTIVE_TREES == 20` instead of 0 after its own sequential `parse_file` call.

## Files Modified

### Prior run (already in git working tree)
- `crates/sdi-lang-python/Cargo.toml` — added tree-sitter-python dependency
- `crates/sdi-lang-python/src/lib.rs` — Python `LanguageAdapter` impl
- `crates/sdi-lang-python/src/extract.rs` (NEW) — Python AST extraction helpers
- `crates/sdi-lang-python/tests/extract_behavior.rs` (NEW) — Python adapter tests
- `crates/sdi-lang-typescript/Cargo.toml` — added tree-sitter-typescript dependency
- `crates/sdi-lang-typescript/src/lib.rs` — TypeScript `LanguageAdapter` impl
- `crates/sdi-lang-typescript/src/extract.rs` (NEW) — TypeScript AST extraction helpers
- `crates/sdi-lang-typescript/tests/extract_behavior.rs` (NEW) — TypeScript adapter tests
- `crates/sdi-lang-javascript/Cargo.toml` — added tree-sitter-javascript dependency
- `crates/sdi-lang-javascript/src/lib.rs` — JavaScript `LanguageAdapter` impl
- `crates/sdi-lang-javascript/src/extract.rs` (NEW) — JavaScript AST extraction helpers
- `crates/sdi-lang-javascript/tests/extract_behavior.rs` (NEW) — JavaScript adapter tests
- `crates/sdi-lang-go/Cargo.toml` — added tree-sitter-go dependency
- `crates/sdi-lang-go/src/lib.rs` — Go `LanguageAdapter` impl
- `crates/sdi-lang-go/src/extract.rs` (NEW) — Go AST extraction helpers
- `crates/sdi-lang-go/tests/extract_behavior.rs` (NEW) — Go adapter tests
- `crates/sdi-lang-java/Cargo.toml` — added tree-sitter-java dependency
- `crates/sdi-lang-java/src/lib.rs` — Java `LanguageAdapter` impl
- `crates/sdi-lang-java/src/extract.rs` (NEW) — Java AST extraction helpers
- `crates/sdi-lang-java/tests/extract_behavior.rs` (NEW) — Java adapter tests
- `crates/sdi-lang-rust/src/extract.rs` — fixed truncation and nested-export bugs
- `crates/sdi-config/src/load.rs` — doc comment fix (separate `load_or_default` and `project_config_path` doc blocks)
- `crates/sdi-parsing/src/lib.rs` — ACTIVE_TREES doc comment updated
- `Cargo.toml` — workspace version bumped to 0.0.4
- `Cargo.lock` — updated

### This session
- `crates/sdi-cli/tests/version.rs` — bumped expected version from `0.0.3` to `0.0.4`
- `crates/sdi-parsing/tests/memory_invariant.rs` — added `COUNTER_LOCK` mutex to serialize concurrent tests

## Human Notes Status
- Non-blocking note (load.rs doc comment misplacement): COMPLETED — fixed in prior run

## Docs Updated
None — no public-surface changes in this task (all new public items in sdi-lang-* crates are documented inline with rustdoc + `# Examples` blocks).
