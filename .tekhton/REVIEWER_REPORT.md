# Reviewer Report — M36.2: `decorators` pattern category (Python)
Review cycle: 1 of 4
Reviewed by: reviewer agent

## Verdict
APPROVED_WITH_NOTES

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- `mod.rs:117` — `classify_hint` doc comment says "P1/P6/P8/P9 active at M35"; now at M36.2. Content is technically accurate (decorators is node-kind-only and does not appear in CALL_DISPATCH), but the milestone stamp is stale — same observation carried from M36.1.

## Coverage Gaps
- None

## Drift Observations
- `crates/sdivi-patterns/src/queries/mod.rs:117` — dispatch comment stamps M35 milestone but we are at M36.2; will keep drifting unless updated when each milestone completes.
- `crates/sdivi-patterns/tests/dispatch_disjointness.rs:26` — "At M35, P1/P6/P8/P9 are active" comment is stale (noted by M36.1 reviewer, deferred again as out of scope). Accumulating across milestones; a one-line fix would pay down this drift.
