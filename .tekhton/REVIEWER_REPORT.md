# Reviewer Report — M31: Pattern Category `class_hierarchy`
Review cycle: 1 of 4
Reviewed by: reviewer agent

## Verdict
APPROVED_WITH_NOTES

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- `CHANGELOG.md` has a prior-milestone note ("New pattern category `logging` ... `list_categories()` now returns 7 entries") immediately below the M31 entry. That count is now stale for readers scanning the Unreleased block. The historical entry was accurate when written, but a minor wording tweak to the M30 sentence would prevent reader confusion.
- `crates/sdivi-lang-rust/src/extract.rs:102–113` — uses an inline manual 256-byte truncation rather than delegating to `sdivi_parsing::text::truncate_to_256_bytes` as every other adapter does. Pre-existing inconsistency not introduced by M31, but `impl_item` hints can span hundreds of lines and this path is now exercised more heavily. A follow-up alignment to the shared helper is low-risk cleanup.

## Coverage Gaps
- Milestone acceptance criteria call for adapter-level fixture tests: extending `tests/fixtures/simple-typescript` to include a class with `extends`, a plain class, an abstract class, and an interface; analogous additions for `simple-java`, `simple-python`, and `simple-rust`; and a negative-result assertion that `tests/fixtures/simple-go` produces zero `class_hierarchy` instances. None of these fixture changes appear in CODER_SUMMARY.md's modified-files list. The unit routing tests confirm classification logic, but fixture-level integration tests are the only way to verify tree-sitter actually produces the expected node-kind strings and that the Go adapter genuinely produces zero hits.

## Drift Observations
- `crates/sdivi-patterns/src/queries/mod.rs` — `category_for_node_kind` is a linear `if/else if` chain with no `logging` branch (intentionally absent — catalog-only). With eight categories the omission is only visible to someone who knows to look. The existing `category_for_node_kind_never_returns_logging` test is the only guard; a brief inline comment noting the intentional absence of a `logging` branch would prevent future confusion when a ninth category is added.
- `crates/sdivi-core/src/categories.rs:89–98` — `CATEGORIES` const is still the hand-indexed `CATALOG_ENTRIES[N].0` form. The indices are correct post-M31 (0–7 inclusive), but this is the third consecutive milestone where the index-shift hazard has appeared as a Watch For. The Seeds Forward cleanup (`const fn` or `LazyLock` derivation from `CATALOG_ENTRIES`) remains open.
- `docs/pattern-categories.md:74` — Go/Java section claims "these languages share the Rust classifier except for `data_access`" but the table only shows three rows. The other five categories are absent without explanation. Pre-existing; M31 correctly added the `class_hierarchy` row within the existing pattern, but the table is still incomplete for a new reader.
