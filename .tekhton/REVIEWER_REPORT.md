# Reviewer Report — M08 sdi-core Pure-Compute Reshape
Review cycle: 1 of 4 (completion pass)

## Verdict
APPROVED_WITH_NOTES

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- `compute_thresholds_check` accepts `ThresholdsInput.overrides` and `.today` but does not read them in the M08 implementation. This is intentional and documented in the docstring ("per-category rates do not affect the aggregate dimension check — that integration is added in M09"). No action required for this cycle; M09 must wire them in and revisit the coverage gap below.
- `ThresholdsInput::default()` hardcodes `today = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()`. Any caller using `::default()` without overriding `.today` will silently mis-evaluate expiry for overrides that expired before 2026-01-01. The doc says callers must supply `today` explicitly, but a static past date default is a footgun worth flagging for M09.
- Two override tests (`override_applied_when_not_expired`, `expired_today_date_consistent` in `compute_thresholds_check.rs`) assert only `let _ = r;`. They verify no panic but make no behavioral claim, while test names imply behavioral coverage they don't provide. A future reader may trust them incorrectly.
- Security agent flagged two LOW/fixable items from prior cycles (`sdi-config/src/load.rs:98` TOCTOU, `sdi-config/src/load.rs:111` terminal injection). Neither is addressed in this M08 completion pass. Both should be resolved before M13 release prep.

## Coverage Gaps
- Per-category threshold override → rate substitution path is untested (explicitly deferred to M09). M09 must add behavioral assertions before the override infrastructure can be considered tested.
- `validate_node_id(".")` — a single-dot path component is neither `..` nor a leading `./`, so it passes the current validation. Untested; low risk but worth covering.

## Drift Observations
- `crates/sdi-core/tests/compute_thresholds_check.rs:96-104` — `override_expiry_ignored_when_expired` passes correctly, but because overrides are entirely unused in M08, not because expiry logic executed. A bug in M09's expiry path would not be caught by this test as written. Revisit when M09 adds override integration.
- `crates/sdi-core/tests/compute_thresholds_check.rs:106-128` — two test functions whose bodies end in `let _ = r;` produce false-positive coverage statistics. Restructure when M09 adds behavioral assertions.
- `.claude/milestones/MANIFEST.cfg` and the M08 milestone file could not be updated to `done` (permission denied, per coder note). Human operator should mark M08 done manually.
