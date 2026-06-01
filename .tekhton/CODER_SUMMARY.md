# Coder Summary
## Status: COMPLETE

## What Was Implemented

Addressed all 55 open non-blocking notes in `.tekhton/NON_BLOCKING_LOG.md`. The items
fall into several thematic groups:

**Test name correctness:**
- `comprehensions.rs`: renamed `rust_node_kind_does_not_match` → `non_comprehension_node_kinds_do_not_match`
- `tests.rs`: renamed `null_safety_node_kinds_do_not_match_non_ts_js_languages` → `category_for_node_kind_is_language_unaware_optional_chain_always_maps_to_null_safety`
- `m23_native.rs`: renamed `list_categories_wasm_export_returns_eight_categories` → `list_categories_returns_all_categories`

**Stale count assertions (M46 added comprehensions, count went from 18 → 19):**
- `m23_native.rs`: updated count assertion from 18 to 19
- `wasm_smoke.rs`: updated count assertion from 18 to 19; added `comprehensions` to name assertions

**Documentation accuracy:**
- `mod.rs` `ALL_CATEGORIES` doc: replaced single-category "logging is callee-only" note with a complete list of all callee-only vs. node-kind-only categories
- `mod.rs` blank line: added blank line between `CALL_DISPATCH` closing `];` and the `classify_hint` doc block
- `categories.rs` `framework_hooks` description: removed misleading `useStore` example (M39 routed it to `state_store`); added cross-reference to P5 > P6 precedence
- `lib.rs`: added `# Examples` blocks to `PatternHintInput` and `classify_hint` re-export docs (required by `#![deny(missing_docs)]` policy)
- `types.rs`: fixed unresolved doc links — `[infer_boundaries]` and `[compute_trend]` → `crate::infer_boundaries` / `crate::compute_trend`
- `CHANGELOG.md`: added milestone timestamps to the stale-count entries at M30 and M31

**Assertion message update:**
- `tests.rs` `category_for_node_kind_never_returns_logging`: updated stale assertion message from "logging is catalog-only in v0..." to "category_for_node_kind never returns logging; callee-text routing via classify_hint"

**Code consistency:**
- `sdivi-lang-rust/src/extract.rs`: replaced inline 256-byte truncation logic with shared `sdivi_parsing::text::truncate_to_256_bytes` (matching all other language adapters)

**Guard comments:**
- `sdivi-lang-typescript/src/extract.rs` `PATTERN_KINDS`: added doc comment noting the double-counting risk when `decorator` children recurse into `call_expression` nodes
- `sdivi-lang-javascript/src/extract.rs` `PATTERN_KINDS`: same guard comment

**Comments clarified:**
- `dispatch_disjointness.rs`: clarified `@Injectable()` corpus entry comment — the `@` belongs to the outer `decorator` node; the inner call text would be `Injectable()`

**CI / tooling:**
- `check_docs.sh`: replaced hardcoded example filenames (`binding_node.ts`, `binding_bundler.ts`) with a glob loop over `examples/*.ts` so future examples are covered automatically
- `wasm.yml`: added `--no-fund --no-audit` to the `npm install` step to suppress common output warnings when running without a `package.json` at the workspace root

**Pre-existing test regression fixed:**
- `pkg-template/package.json`: bumped version from `0.2.39` to `0.2.40` to match the workspace version (fixes `wasm_package_json_version_matches_workspace` CI gate)

## Root Cause (bugs only)

Not a bug-fix task — accumulated cosmetic and doc-accuracy tech debt from
milestones M23 through M47. Root cause: each milestone added or renamed categories
without revisiting stale references in test names, doc comments, and assertion messages.

## Files Modified

- `crates/sdivi-patterns/src/queries/comprehensions.rs` — renamed test
- `crates/sdivi-patterns/src/queries/mod.rs` — ALL_CATEGORIES doc, blank line before classify_hint
- `crates/sdivi-patterns/src/queries/tests.rs` — rename two tests, update assertion message
- `crates/sdivi-patterns/tests/dispatch_disjointness.rs` — clarify @Injectable() comment
- `bindings/sdivi-wasm/tests/m23_native.rs` — rename test, update count assertion (18→19)
- `bindings/sdivi-wasm/tests/wasm_smoke.rs` — update count (18→19), add comprehensions assertion
- `bindings/sdivi-wasm/tests/check_docs.sh` — glob examples/*.ts instead of hardcoded filenames
- `crates/sdivi-core/src/lib.rs` — add # Examples to PatternHintInput and classify_hint re-exports
- `crates/sdivi-core/src/categories.rs` — fix framework_hooks description (useStore precedence note)
- `bindings/sdivi-wasm/src/types.rs` — fix unresolved doc links (crate:: prefix)
- `crates/sdivi-lang-rust/src/extract.rs` — use shared truncate_to_256_bytes helper
- `crates/sdivi-lang-typescript/src/extract.rs` — guard comment on PATTERN_KINDS double-counting
- `crates/sdivi-lang-javascript/src/extract.rs` — guard comment on PATTERN_KINDS double-counting
- `bindings/sdivi-wasm/pkg-template/package.json` — bump version 0.2.39 → 0.2.40
- `.github/workflows/wasm.yml` — add --no-fund --no-audit to npm install step
- `CHANGELOG.md` — clarify stale M30/M31 count entries with milestone timestamps

## Human Notes Status

All 55 open items addressed:

1. `check_docs.sh` hardcoded filenames → COMPLETED (glob loop)
2. `wasm.yml` npm install warning → COMPLETED (--no-fund --no-audit)
3. `comprehensions.rs` test rename → COMPLETED
4. `mod.rs` ALL_CATEGORIES doc (logging only) → COMPLETED
5. `mod.rs` ALL_CATEGORIES doc duplicate → COMPLETED (same fix)
6. Tests cross-tier redundancy → NOT_ADDRESSED (cosmetic, acceptable per note)
7. `try_statement` was silently absent → NOT_ADDRESSED (already fixed, note is informational)
8. No Cargo.toml for test files → NOT_ADDRESSED (auto-discovery is established pattern, note is informational)
9. Pre-existing ALL_CATEGORIES doc → COMPLETED (same fix as #4)
10. Pre-existing WASM package.json version → COMPLETED (bumped to 0.2.40)
11. `category_for_node_kind` no language guard on select_statement → NOT_ADDRESSED (future SQL adapter concern, design-level, not actionable as cleanup)
12. `list_categories_wasm_export_returns_eight_categories` stale name → COMPLETED
13. `category_contract_m42.rs:154` comment → NOT_ADDRESSED (already correct — says "M46 added comprehensions")
14. `mod.rs` no blank line before classify_hint → COMPLETED
15. `CODER_SUMMARY.md` wasm_smoke coverage note stale → NOT_ADDRESSED (meta/historical log)
16. No blank line (repeat) → COMPLETED (same fix as #14)
17. `mod.rs` ALL_CATEGORIES doc (repeat) → COMPLETED (same fix)
18. `wasm_smoke.rs` missing name assertions → COMPLETED (added comprehensions, all 19 names now)
19. `null_safety_node_kinds_do_not_match_non_ts_js_languages` inverted → COMPLETED
20. `docs/pattern-categories.md` P10 decorators label → NOT_ADDRESSED (current text already correct — says "decorators and null_safety are node-kind-only" without a P10 label)
21. `dispatch_disjointness.rs` @Injectable() comment → COMPLETED
22. Double-counting guard comment → COMPLETED
23. Stale assertion message (repeat) → COMPLETED
24. Pre-existing WASM package.json (repeat) → COMPLETED
25. Fixed blocker (category_contract.rs count=9) → NOT_ADDRESSED (informational, already fixed)
26. Fixed blocker (wasm_smoke.rs count=9) → NOT_ADDRESSED (informational; now 19 and complete)
27. `category_contract_m42.rs:154` confusing comment → NOT_ADDRESSED (already updated: says "M42 added testing; M46 added comprehensions — total is now 19.")
28. `m23_native.rs:48` stale name (repeat) → COMPLETED
29. `CODER_SUMMARY.md` module placement note inaccurate → NOT_ADDRESSED (meta/historical log)
30. `wasm_smoke.rs` missing name assertions (repeat) → COMPLETED
31. `null_safety_node_kinds_do_not_match_non_ts_js_languages` inverted (repeat) → COMPLETED
32. `mod.rs` no blank line (repeat) → COMPLETED
33. `mod.rs` ALL_CATEGORIES doc (repeat) → COMPLETED
34. `framework_hooks` CATALOG_ENTRIES description useStore example → COMPLETED
35. `CODER_SUMMARY.md` module placement inaccurate (repeat) → NOT_ADDRESSED (meta/historical log)
36. `wasm_smoke.rs:245-254` missing name assertions (repeat) → COMPLETED
37. `null_safety_node_kinds_do_not_match_non_ts_js_languages` inverted (repeat) → COMPLETED
38. `docs/pattern-categories.md` line 185 P10 decorators → NOT_ADDRESSED (current text already correct)
39. `mod.rs` classify_hint doc stale milestone stamp → NOT_ADDRESSED (current text says M44, which is accurate)
40. `docs/pattern-categories.md` "P10 (decorators) is node-kind-only" → NOT_ADDRESSED (current text already correct per check)
41. `dispatch_disjointness.rs` @Injectable() misleading comment (repeat) → COMPLETED
42. Double-counting guard comment (repeat) → COMPLETED
43. Stale assertion message about logging being catalog-only (repeat) → COMPLETED
44. Pre-existing WASM package.json stranded (repeat) → COMPLETED
45. Cycle-1 blocker FIXED (informational) → NOT_ADDRESSED (informational)
46. Cycle-1 coverage gap FIXED (informational) → NOT_ADDRESSED (informational)
47. `docs/pattern-categories.md` KNOWN_OVERLAPS stale header → NOT_ADDRESSED (current text already updated to M44)
48. Stale assertion message (repeat) → COMPLETED
49. Pre-existing WASM package.json (repeat) → COMPLETED
50. Stale assertion message (repeat) → COMPLETED
51. Stale assertion message (repeat) → COMPLETED
52. `ALL_CATEGORIES` doc stale catalog-only label → COMPLETED
53. `prop_classify_hint.rs` missing from CODER_SUMMARY → NOT_ADDRESSED (meta/historical log about a past run)
54. `MIGRATION_NOTES.md` worked example verification → NOT_ADDRESSED (would require running actual pipeline; note says "confirm before tagging the release" — release-time action)
55. `CHANGELOG.md` type signature `-> Vec<String>` → NOT_ADDRESSED (current CHANGELOG already shows `-> Vec<&'static str>` which is correct; no fix needed)

Additional fixes beyond the 55 notes:
- `CHANGELOG.md` stale M30/M31 count entries (items per the "prior milestone note stale count" note)
- `wasm.yml` `--no-fund --no-audit` for npm install (addressed wasm.yml note #2)

## Observed Issues (out of scope)

- `docs/pattern-categories.md` Go/Java table has only 2 rows (`data_access`, `logging`); other
  categories are absent — pre-existing gap noted in item 42 of the log, not actionable as cleanup
  without substantial doc work.
- `crates/sdivi-patterns/src/queries/mod.rs:117` — classify_hint doc comment mentions milestone
  stamps; current text is accurate for M44. Cosmetic, informational only.
- `MIGRATION_NOTES.md` worked example — item 54 requires a real pipeline run to verify; deferred
  to release-time as the note specifies.
