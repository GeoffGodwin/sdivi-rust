# Coder Summary
## Status: COMPLETE

## What Was Implemented

**Note 1 — `compute_thresholds_check` overrides/today not yet wired (M09):**
Added `// TODO(M09)` comment at the top of `compute_thresholds_check` body
explicitly naming the future wiring point. The docstring already described this
correctly; the comment makes the gap visible in code review.

**Note 2 — `ThresholdsInput::default().today` footgun:**
Changed the sentinel from `2026-01-01` (which was in the past on first PR day and
therefore mis-evaluates any override expiring 2026-01-02 through today) to
`9999-12-31` (far future). With a far-future sentinel, all per-category overrides
appear expired by default — the conservative, safe failure mode for a threshold
gate. Callers that need override filtering must set `today` explicitly.
Updated struct doc, field doc, and `Default` impl comment accordingly.

**Note 3 / Note 6 — Security findings (TOCTOU + terminal injection):**
Verified both are already resolved in the current code:
- `load_toml_file` calls `read_to_string` directly (no TOCTOU).
- Warning uses `{key:?}` debug format (control characters escaped).
No code changes needed; marked COMPLETED.

**Note 4 — Misleading comment in `leiden/mod.rs:167`:**
Fixed comment from "best_comm is always >= n (offset community) so best_comm != node
always" (which overstates the invariant) to "When best_gain > 1e-10, best_comm is
always an offset community ID (>= n) and != node. best_comm defaults to `node` (< n)
when no neighbour improves the gain; the best_gain threshold below prevents using
that default." Logic was correct; only the comment was wrong.

**Note 5 — `load_with_paths` calls `today_iso8601()` unconditionally:**
Added a NOTE comment at the call site documenting the test limitation: integration
tests cannot inject a specific date through `load_with_paths`; tests of expiry
behaviour must call `validate_and_prune_overrides` directly.

**Notes 7–10 — Merged doc comment on `load_or_default`/`project_config_path`:**
Verified already fixed in current code. `project_config_path` is declared before
`load_or_default`, each with its own doc block directly above it and a blank line
between the functions. No code changes needed; marked COMPLETED.

## Root Cause (bugs only)
N/A — all items are tech-debt cleanup or stale reviewer notes.

## Files Modified
- `crates/sdi-detection/src/leiden/mod.rs` — fixed misleading comment at old line 167
- `crates/sdi-core/src/input/types.rs` — changed `ThresholdsInput::default().today` from
  2026-01-01 to 9999-12-31 sentinel; updated struct doc, field doc, and impl comment
- `crates/sdi-core/src/compute/thresholds.rs` — added `// TODO(M09)` comment
- `crates/sdi-config/src/load.rs` — added NOTE comment about `today_iso8601()` test limitation
- `.tekhton/NON_BLOCKING_LOG.md` — marked all 10 items as resolved with resolution notes

## Docs Updated
None — no public-surface changes (comment and sentinel-date changes only; `ThresholdsInput`
struct shape is unchanged).

## Human Notes Status
- Note 1 (compute_thresholds_check overrides/today not wired): COMPLETED — TODO(M09) comment added
- Note 2 (ThresholdsInput::default() today footgun): COMPLETED — sentinel changed to 9999-12-31
- Note 3 (security TOCTOU + terminal injection not addressed in M08): COMPLETED — already fixed, verified
- Note 4 (leiden/mod.rs:168 misleading comment): COMPLETED — comment corrected
- Note 5 (load_with_paths today_iso8601 unconditional): COMPLETED — NOTE comment added
- Note 6 (security findings resolved): COMPLETED — verified already fixed
- Note 7 (merged doc comment M07): COMPLETED — already fixed in current code, verified
- Note 8 (merged doc comment M06): COMPLETED — already fixed in current code, verified
- Note 9 (merged doc comment M05): COMPLETED — already fixed in current code, verified
- Note 10 (merged doc comment M04): COMPLETED — already fixed in current code, verified
