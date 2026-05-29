# Reviewer Report — M29: Pattern Category `data_access`
Review cycle: 1 of 4
Reviewed by: reviewer agent

## Verdict
APPROVED_WITH_NOTES

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- Pre-existing `cargo doc --workspace --no-deps -D warnings` failure in `bindings/sdivi-wasm/src/types.rs:218,226` (unresolved links to `infer_boundaries` and `compute_trend`) means the doc-clean acceptance criterion does not pass for the workspace as a whole. Not introduced by this milestone; all new pub items in `sdivi-core` carry correct doc comments. Should be tracked as a separate cleanup item.
- The milestone Tests section implies a named `call_is_data_access` test alongside `call_expression_is_data_access`. The implementation merges both TS and Python assertions into the single `call_expression_is_data_access` test. Functionally equivalent and the milestone marks the separate Python test as implicit. No action required.

## Coverage Gaps
- No fixture-level integration test confirms that `snapshot` against `tests/fixtures/simple-typescript` or `tests/fixtures/simple-python` produces a non-empty `data_access` bucket in `pattern_metrics`. The milestone acceptance criteria and Tests section explicitly require this. Not present in any modified file.
- No Go-specific assertion in `call_expression_is_data_access` (milestone flags this as optional insurance). Low priority.

## Drift Observations
- `crates/sdivi-core/src/categories.rs:69-76` — `CATEGORIES` is a hand-indexed slice of `CATALOG_ENTRIES[N].0` literals. Fragile to insertions; the milestone's "Seeds Forward" already flags a `const fn` cleanup. Worth tracking: the next category milestone will shift indices again.
- `docs/pattern-categories.md` — Go/Java section only shows the `data_access` override row and states the other categories inherit from Rust via prose, while the TypeScript/JavaScript section shows all six categories explicitly. The asymmetric table format may confuse readers comparing per-language tables. Pre-existing design; no code change required but a follow-up alignment pass would reduce ambiguity.
