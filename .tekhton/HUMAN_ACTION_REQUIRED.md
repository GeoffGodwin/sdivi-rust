# Human Action Required

The pipeline identified items that need your attention. Review each item
and check it off when addressed. The pipeline will display a banner until
all items are resolved.

## Action Items

## Resolved

- [x] **`CLAUDE.md` § Code Conventions — add doc comment placement rule.** Resolved 2026-05-01: added a `Doc comment placement when inserting items` bullet under `## Code Conventions` → `Doc discipline` in `CLAUDE.md`. Captures the `///`-attaches-to-next-item invariant and notes that `#![deny(missing_docs)]` only catches it on `sdi-core`.
- [ ] [2026-05-01 | Source: architect] **`crates/sdi-core/src/compute/thresholds.rs:92-94`** (and `crates/sdi-core/tests/compute_thresholds_check.rs:106-139`) — The `TODO(M09)` comment is now orphaned: M09 has shipped, but per-category override wiring was not actually part of M09's delivered scope. The TODO correctly explains *why* overrides aren't wired — it requires surfacing per-category entropy/drift deltas in `DivergenceSummary`, which doesn't exist yet. Consequences: 1. `override_expiry_ignored_when_expired` (test line 96–104) passes vacuously: the override is never read, so expiry logic is never exercised. 2. `override_not_wired_in_m08_base_rate_applies` and `base_rate_applies_regardless_of_override_state_m08` carry M08/M09 milestone labels that are now stale. 3. The companion test demanded by the drift log (an active/unexpired override suppressing a breach) was never written. **Decision required:** Does per-category override wiring ship before v0 (in M12 as a pre-0.1.0 cleanup), or does `cfg.overrides` remain accepted-but-not-read at v0 with explicit documentation? The current public API (`ThresholdsInput::overrides`) is already committed as `pub`; if it ships silent-no-op at 0.1.0, that is a permanent SemVer commitment to the current (vacuous) behaviour. If wiring is deferred, the two M08/M09 stub tests must be renamed and their TODOs updated to reference the blocking feature (per-category delta surfacing), not a milestone number. ---
- [ ] [2026-05-01 | Source: architect] `[0.0.15]`: "[MILESTONE 10 ✓] feat: Implement Milestone 10…"
- [ ] [2026-05-01 | Source: architect] `[0.0.14]`: "[MILESTONE 9 ✓] feat: Implement Milestone 9…"
- [ ] [2026-05-01 | Source: architect] And similar entries throughout. Per CLAUDE.md doc conventions, the release manager owns CHANGELOG finalization at tag time. These entries are not user-visible feature descriptions — they are internal session notes. They need to be rewritten or collapsed into a single `[0.0.1]` entry covering the pre-release development period before the public `0.1.0` tag is cut. This is not a coder task — it is a documentation authorship decision about what to expose publicly. Drift log reference: `[2026-05-01 | "Implement Milestone 13"] CHANGELOG.md` ---
