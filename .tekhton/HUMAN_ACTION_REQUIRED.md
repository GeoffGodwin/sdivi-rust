# Human Action Required

The pipeline identified items that need your attention. Review each item
and check it off when addressed. The pipeline will display a banner until
all items are resolved.

## Action Items

## Resolved

- [x] **`CLAUDE.md` § Code Conventions — add doc comment placement rule.** Resolved 2026-05-01: added a `Doc comment placement when inserting items` bullet under `## Code Conventions` → `Doc discipline` in `CLAUDE.md`. Captures the `///`-attaches-to-next-item invariant and notes that `#![deny(missing_docs)]` only catches it on `sdi-core`.
- [x] [2026-05-01 | Source: architect] **`crates/sdi-core/src/compute/thresholds.rs` — orphaned `TODO(M09)` for per-category override wiring.** Resolved 2026-05-01 as obsolete: the entry was stale at the time it was filed. Per-category override wiring shipped in M14 (CHANGELOG `[0.1.1]`). Verified: `compute_thresholds_check` reads `cfg.overrides` and applies active overrides at `crates/sdi-core/src/compute/thresholds.rs:202-235`; `DivergenceSummary` carries `pattern_entropy_per_category_delta` and `convention_drift_per_category_delta`; the "active/unexpired override suppressing a breach" test demanded by the drift log exists at `crates/sdi-core/tests/compute_thresholds_check.rs:141-158` (`active_override_raises_per_category_limit`) and again at `:183` (`active_override_blocks_per_category_breach`); no `TODO` marker remains in the file. No code change required.
- [x] [2026-05-01 | Source: architect] **CHANGELOG `[0.0.x]` entries with session-style "[MILESTONE N ✓] feat:" titles.** Resolved 2026-05-01: deleted the `[0.0.1]`–`[0.0.16]` blocks entirely. None of those versions were tagged in git or published to crates.io, so there is no audit-trail value in keeping them; the [0.1.0] block already enumerates M01–M13 as the SemVer commitment baseline. Replaced the deleted region with a brief explanatory note pointing readers at git history for the development trail. Also removed the orphan `- [MILESTONE 12 ✓] feat: M12 (M13)` bullet that was trapped inside the [0.1.0] "Binary and bundle sizes" subsection.
