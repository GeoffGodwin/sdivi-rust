# Reviewer Report — M16: Snapshot at Historical Commit

## Verdict
APPROVED_WITH_NOTES

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- `commit_extract.rs:81` — `commit_date_iso` returns `CommitExtractError::RefResolutionFailed` when the date string cannot be parsed. That variant name is misleading (ref resolution succeeded; the failure is date-format parsing). A `CommitDateParseFailed { sha, raw }` variant would be clearer, though the error message is descriptive enough to diagnose.
- `CODER_SUMMARY.md` lists `docs/library-embedding.md` in the `## Docs Updated` section but does NOT list it under `## Files Modified`. Either the file was updated (and should appear in the modified list) or the Docs Updated entry is erroneous. The omission is harmless but creates an audit gap.
- `commit_extract.rs:94-113` — `git archive` is spawned before the `tar --version` availability check. If `tar` is absent, `git archive` is already running; it will exit cleanly once its stdout pipe is closed on early return, but spawning a process that will be immediately abandoned is sub-optimal. The check should precede the spawn.
- Security findings MEDIUM/LOW (rev-parse `--` separator, tar `--no-absolute-filenames`, stderr truncation) are handled by the security pipeline and noted here for completeness only.

## Coverage Gaps
- `change_coupling_ends_at_commit_not_head` (`commit_snapshot.rs:185`) validates that `snap_hist.commit` equals `sha_head1` but does not verify that the change-coupling data actually differs between HEAD and HEAD~1. With only 3 single-file commits and the default `min_frequency = 0.6`, both snapshots likely have empty coupling, so the window-clamping logic is not exercised. A fixture with co-changing files would give this test meaningful coverage.
- No test exercises `normalize_to_utc` edge cases for timezone offsets that cross a day boundary (e.g., `2026-04-30T23:30:00+01:00` → `2026-04-30T22:30:00Z` vs. `2026-05-01T00:30:00+01:00` → day rollover). The existing unit tests cover the happy path; DST/rollover is unexercised.

## Drift Observations
- `commit_extract.rs:158-209` — `normalize_to_utc`, `calendar_to_epoch`, and `epoch_to_iso8601` are hand-rolled ISO 8601 + Proleptic Gregorian arithmetic. `sdi-pipeline` already depends on `chrono` (via `sdi-config`) with `default-features = false`; using `chrono::DateTime::parse_from_rfc3339` would eliminate ~50 lines of custom arithmetic with no WASM impact (pipeline is FS-bearing and not WASM-compatible). This is a simplification opportunity for a future cleanup pass, not a bug.
- `tests/historical_commit_lifecycle.rs` — workspace-level `tests/` placeholder is a comment-only file that explains why the real test lives under `crates/sdi-cli/`. The comment is accurate but the file itself adds no value and will accumulate as noise if the pattern is repeated for future milestones.
