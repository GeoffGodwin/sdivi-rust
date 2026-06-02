# Reviewer Report — Cycle 2

**Date:** 2026-06-02
**Branch:** feature/MorePatterns
**Reviewer:** code-review agent
**Review cycle:** 2 of 2

---

## Verdict
APPROVED_WITH_NOTES

---

## Complex Blockers (senior coder)
None

---

## Simple Blockers (jr coder)
None

---

## Non-Blocking Notes
- `DRIFT_LOG.md`: `## Decisions (Declined / Will Not Implement)` section (line 61) remains empty despite six items being explicitly declined (`fmt.Errorf` Go logging, `PYTHON_RE` receiver asymmetry, `rejects_nan_weight`, `list_categories()` placement, `LeidenConfigInput` range check, `wasm.yml` dual node_modules). All declined items are merged into `## Resolved` alongside code-fixed items. Future reviewers cannot distinguish "fixed" from "acknowledged/deferred" without reading every entry body. Consider moving declined items to the Declined section for structural clarity.
- `leiden/mod.rs:197`: `pub(super) fn renumber` is more permissive than strictly necessary — a child module (refine) already has access to private items in its parent under Rust's visibility rules. The change is not a bug, but it extends visibility of `renumber` to `leiden`'s sibling modules within `sdivi_detection`, which is a slightly wider surface than needed for the delegation goal alone.
- `categories.rs:231`: Compile-time assert line is long (>100 chars). Minor style point; the assert is correct and will fire at compile time on any CATALOG_ENTRIES / CATEGORIES length mismatch.

---

## Coverage Gaps
- `leiden/refine.rs`: The delegation `renumber_in_place → super::renumber` is not directly covered by a unit test that verifies `refine_partition` produces densely renumbered IDs starting from 0. Covered indirectly by integration tests; a targeted unit test would pin the contract.
- DRIFT_LOG entries for `category_for_node_kind("go_statement", "go")` and `call_expression` → `data_access` doc asymmetry are marked "worth a unit test sentinel on the next pass" with no corresponding test or tracking ticket. Risk of silent skip in future cycles.

---

## Drift Observations
- `leiden/refine.rs:270–272`: `renumber_in_place` is now a one-line private wrapper that only preserves a name. The single call site at line 181 (`renumber_in_place(&mut refined)`) could call `super::renumber(&mut refined)` directly, eliminating the wrapper entirely without any loss of clarity.
- `DRIFT_LOG.md:63–88`: All resolved items (code-fixed and acknowledged/declined) share the `## Resolved` section. The structural separation between the `## Resolved` and `## Decisions (Declined / Will Not Implement)` sections is not being used, which will degrade the log's usefulness as a historical audit trail as the entry count grows.
