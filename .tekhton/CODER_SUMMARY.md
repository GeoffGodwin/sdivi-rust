# Coder Summary
## Status: COMPLETE

## What Was Implemented

### M32: Callee-Text Classification API (`classify_hint`)

- **`crates/sdivi-patterns/src/hint_input.rs`** (NEW) — `PatternHintInput` struct with `node_kind: String` and `text: String`. WASM-safe, no sdivi-parsing dependency.

- **`crates/sdivi-patterns/src/lib.rs`** — Added `pub mod hint_input; pub use hint_input::PatternHintInput;`.

- **`crates/sdivi-patterns/src/queries/data_access.rs`** — Added `pub fn matches_callee(text, language) -> bool` with `LazyLock<Regex>` tables for TypeScript/JavaScript/Go and Python. Rust/Java return `false` in v0.

- **`crates/sdivi-patterns/src/queries/logging.rs`** — Added `pub fn matches_callee(text, language) -> bool` with `LazyLock<Regex>` tables for TypeScript/JavaScript, Python, Go, Rust, and Java.

- **`crates/sdivi-patterns/src/queries/async_patterns.rs`** — Added `pub fn matches_callee(text, language) -> bool` for Promise-chain `.then/.catch/.finally` in TypeScript/JavaScript.

- **`crates/sdivi-patterns/src/queries/resource_management.rs`** — Added `pub fn excludes_callee(text, language) -> bool` (inverted). Rust logging macros return `true`; all others return `false`.

- **`crates/sdivi-patterns/src/queries/mod.rs`** — Added `pub fn classify_hint(hint: &PatternHintInput, language) -> Vec<&'static str>` with dispatch logic. Updated rustdoc on `category_for_node_kind` with "See also" cross-reference.

- **`crates/sdivi-core/src/lib.rs`** — Re-exported `classify_hint` and `PatternHintInput` from `sdivi-patterns`.

- **`bindings/sdivi-wasm/src/types.rs`** — Added `WasmPatternHintInput` (Tsify-derived).

- **`bindings/sdivi-wasm/src/exports.rs`** — Added `#[wasm_bindgen] classify_hint(hint, language) -> Vec<String>` export.

- **`crates/sdivi-patterns/tests/classify_hint.rs`** (NEW) — 22 unit tests covering all per-language matches_callee cases, dispatch priority, disjoint-regex invariant.

- **`bindings/sdivi-wasm/tests/classify_hint_wasm.rs`** (NEW) — 8 WASM integration tests including cross-platform determinism assertion.

- **`docs/pattern-categories.md`** — "Callee-text classification (`classify_hint`)" section with per-language regex tables, worked examples, dispatch order, regex change log. Updated embedder responsibilities §6.

- **`CHANGELOG.md`** — M32 entries for `classify_hint`, `PatternHintInput`, and the four callee-helper functions.

- **`Cargo.toml` (workspace)** — Added `regex = "1"` workspace dep.

- **`crates/sdivi-patterns/Cargo.toml`** — Added `regex = { workspace = true }`.

### Native pipeline unchanged
`crates/sdivi-patterns/src/catalog.rs` is untouched. `Pipeline::snapshot` still calls `category_for_node_kind`. Snapshot output is bit-identical pre/post M32.

### M30 sentinel still green
`category_for_node_kind_never_returns_logging` test passes unchanged. Updated comment to note `classify_hint` as the precision-aware alternative.

## Root Cause (bugs only)
N/A — feature implementation

## Files Modified
- `crates/sdivi-patterns/src/hint_input.rs` (NEW)
- `crates/sdivi-patterns/src/lib.rs`
- `crates/sdivi-patterns/src/queries/mod.rs`
- `crates/sdivi-patterns/src/queries/data_access.rs`
- `crates/sdivi-patterns/src/queries/logging.rs`
- `crates/sdivi-patterns/src/queries/async_patterns.rs`
- `crates/sdivi-patterns/src/queries/resource_management.rs`
- `crates/sdivi-patterns/tests/classify_hint.rs` (NEW)
- `crates/sdivi-core/src/lib.rs`
- `bindings/sdivi-wasm/src/types.rs`
- `bindings/sdivi-wasm/src/exports.rs`
- `bindings/sdivi-wasm/tests/classify_hint_wasm.rs` (NEW)
- `docs/pattern-categories.md`
- `CHANGELOG.md`
- `Cargo.toml` (workspace)
- `crates/sdivi-patterns/Cargo.toml`

## Human Notes Status
- M31 reviewer APPROVED_WITH_NOTES — no blockers to fix from prior run
- Non-Blocking Notes from reviewer: noted, out of scope for M32
- Tester passed all 28 tests, no bugs to fix

## Docs Updated
- `docs/pattern-categories.md` — "Callee-text classification" section + embedder responsibilities §6 update.
- `CHANGELOG.md` — M32 Added entries.

## Architecture Change Proposals

### PatternHintInput in sdivi-patterns (not re-exporting PatternHint from sdivi-parsing)

- **Current constraint:** Milestone spec said "Re-export `PatternHint` from `sdivi-parsing` if not already exposed." Implicit assumption it could go via `sdivi-core`.
- **What triggered this:** `sdivi-parsing` is explicitly excluded from the WASM dep tree (Rule 21). `sdivi-core` cannot depend on it. Re-exporting `PatternHint` would pull in tree-sitter.
- **Proposed change:** Defined `PatternHintInput` in `sdivi-patterns::hint_input` with the two fields `classify_hint` needs. Re-exported via `sdivi-core`. Follows the existing `*Input` struct family pattern.
- **Backward compatible:** Yes — classify_hint is new.
- **ARCHITECTURE.md update needed:** No — CLAUDE.md already describes the `*Input` struct family pattern for sdivi-core.

## Observed Issues (out of scope)
- `wasm_package_json_version_matches_workspace` — pre-existing: package.json at 0.2.18 vs workspace 0.2.21.
- `cargo build -p sdivi-core --target wasm32-unknown-unknown --no-default-features` — pre-existing getrandom build failure.
- `tempfile` in wasm32 dep tree for sdivi-core — pre-existing (confirmed via git stash comparison).
- `RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps` — pre-existing unresolved links in `bindings/sdivi-wasm/`.
- `crates/sdivi-core/src/categories.rs` — `CATEGORIES` const still uses explicit index references (Seeds Forward debt from M29–M31).
