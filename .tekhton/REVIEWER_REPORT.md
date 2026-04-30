# Reviewer Report — M08 (Review Cycle 1 of 4)

## Verdict
APPROVED_WITH_NOTES

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- `leiden/mod.rs:168` — Comment "best_comm is always >= n (offset community) so best_comm != node always" overstates the invariant. `best_comm` defaults to `node` (< n) when the neighbour loop finds nothing; the `best_gain > 1e-10` guard is what prevents using that default. Logic is correct; comment should read "when best_gain > 1e-10, best_comm is always an offset community ID (>= n)" to avoid misleading future readers.
- `leiden/mod.rs` — After `local_move_phase` writes back `state.assignment`, `partition` carries mixed IDs: node indices 0..n for singletons, n..n+k for merged communities. This is then passed to `refine_partition` and `aggregate_network` (not in review scope). Pre-fix, the same post-local_move_phase non-dense IDs were already passed; those functions presumably handle arbitrary ID ranges. However no doc comment confirms the invariant. The coder's own Observed Issues section flags a related precondition gap on `remove_node`; a companion note on `refine_partition`'s ID-range expectation would close the loop.
- `sdi-config/src/load.rs:96` — `load_with_paths` calls `thresholds::today_iso8601()` unconditionally, making it impossible to inject a specific "today" for integration tests that exercise expiry behaviour through the public `load_with_paths` API. Tests of expiry must call `validate_and_prune_overrides` directly with an explicit date string. Workable but worth noting for future test authors.
- Security findings (from security agent): both LOW items now resolved in this diff — TOCTOU eliminated (`load_toml_file` calls `read_to_string` directly and matches `ErrorKind::NotFound`); terminal injection fixed (`{key:?}` debug format escapes embedded control characters). The unmaintained `serde_yaml` item remains tracked for M10 as documented.

## Coverage Gaps
- No regression test for the Leiden underflow bug. A test that constructs a small graph, pre-loads a partition where community IDs collide with node indices (e.g. n=4, partition=[0,1,2,3] so community 0 == node 0), runs `local_move_phase`, and asserts no panic/overflow would guard the fix.
- `validate_and_prune_overrides` boundary: `expires` equal to today's date. The predicate is `date_str < today` (strict less-than), so an override expiring today is kept. No test asserts this boundary; a future developer could reasonably misread the semantics.

## Drift Observations
- `leiden/mod.rs:173` — The singleton fallback `let target = if state.size[old_comm] == 0 { node } else { old_comm }` uses a raw node index (0..n) as a community ID. After the offset fix, real community IDs are n..n+k so a singleton at `node` < n is unambiguous — but this invariant (singleton ID == node index, always < n) is undocumented. A future caller that omits the offset guarantee reintroduces the collision silently.
- `thresholds.rs:26` — `validate_date_format` accepts Feb 29 for any year (noted in fn doc comment). Intentional but inconsistent with strict calendar validation. No action required; logged for the audit accumulation.
