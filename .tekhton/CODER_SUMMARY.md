# Coder Summary
## Status: COMPLETE

## What Was Implemented

### M34: Multi-Category Call-Expression Dispatch Framework

Pure refactor of `classify_hint`'s `call_expression`/`call` arm. No behaviour change.
`snapshot_version` stays `"1.0"`.

**`crates/sdivi-patterns/src/queries/mod.rs`**
- Added `CALL_DISPATCH: &[(&str, fn(&str, &str) -> bool)]` const (with
  `#[allow(clippy::type_complexity)]` since fn pointer tuple trips the lint).
  Precedence comment inline: P1=async_patterns > P8=logging > P9=data_access.
- Replaced the three-`if`-block arm with a single `for &(category, matches) in
  CALL_DISPATCH` loop. First match returns; falls through to `vec![]` unchanged.
- Updated dispatch order doc to reference `CALL_DISPATCH` and note P1/P8/P9
  active at M34.
- File held at exactly 300 lines.

**`crates/sdivi-patterns/tests/dispatch_disjointness.rs`** (NEW)
- `KNOWN_OVERLAPS` table: one entry documenting that `fetch(url).catch(err => {})`
  in JavaScript matches both `async_patterns` (P1, `.catch(`) and `data_access`
  (P9, `^fetch\b`). P1 wins. This was a real overlap discovered during testing,
  not hypothetical.
- `CORPUS`: 23 entries covering P1/P8/P9 across TypeScript, JavaScript, Python, Go,
  Java, plus unrecognised callees that must return empty.
- Four tests: `corpus_resolves_to_expected_category`,
  `corpus_resolves_identically_for_call_node_kind`,
  `no_undocumented_overlaps_in_corpus`,
  `known_overlaps_winner_matches_dispatch_order`.

**`docs/pattern-categories.md`**
- Replaced the simple bullet list in "Dispatch order in `classify_hint`" with a
  full canonical precedence table (P1–P11 slots, activation milestone, regex hint).
- Added KNOWN_OVERLAPS policy section with the M34 documented overlap.
- Documented future overlaps that M35–M44 milestones must add to KNOWN_OVERLAPS.

**`CHANGELOG.md`**
- Added `### Changed` entry under `[Unreleased]` describing the internal refactor.

## Root Cause (bugs only)
N/A — pure refactor.

## Files Modified
- `crates/sdivi-patterns/src/queries/mod.rs` — CALL_DISPATCH registry + loop
- `crates/sdivi-patterns/tests/dispatch_disjointness.rs` (NEW)
- `docs/pattern-categories.md` — formalized precedence table + KNOWN_OVERLAPS policy
- `CHANGELOG.md` — internal-refactor Changed entry

## Human Notes Status
- Non-blocking note (reviewer report, line 279 assertion message) — NOT_ADDRESSED;
  out of scope per task instructions. Logged under Observed Issues.

## Docs Updated
- `docs/pattern-categories.md` — "Dispatch order in `classify_hint`" section
  replaced with formal precedence table and KNOWN_OVERLAPS policy.

## Observed Issues (out of scope)
- `crates/sdivi-patterns/src/queries/mod.rs:279` — Test assertion message reads
  "logging is catalog-only in v0 for category_for_node_kind" (stale phrasing from M30).
  Tested behaviour is correct. Flagged by the M23 reviewer; deferred cleanup.
- `crates/sdivi-core/src/categories.rs:90-99` — `CATEGORIES` and `CATALOG_ENTRIES`
  ordering dependency; a comment would help the next maintainer.
- `CHANGELOG.md` at 691 lines — exceeds the coder self-check 300-line ceiling, but
  this is a pre-existing multi-release accumulation. Only 10 lines were added by M34.
- `wasm_package_json_version_matches_workspace` test failure — pre-existing before
  this run (wasm package.json stranded at 0.2.23, workspace at 0.2.24). Not
  introduced by M34.
