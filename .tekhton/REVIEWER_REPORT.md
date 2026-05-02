# Reviewer Report — M18 Leiden Refinement + verify-leiden Green

## Verdict
APPROVED_WITH_NOTES

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- `mod.rs:143-147` — The `debug_assert!(agg_graph.n < graph.n, ...)` is always true at that point because the `if agg_graph.n >= graph.n { break; }` guard two lines above already exits the loop. The assertion is intentional documentation of the invariant (per the milestone spec), but a reader unfamiliar with the intent might wonder if it can ever fire. A brief comment like `// always true past the identity break; here as an invariant marker` would remove that confusion. Non-blocking — the milestone spec explicitly asks for this assertion and documents its purpose.
- `refine.rs:150` — `#[doc(hidden)]` on a function that also has a full `///` doc block is slightly surprising; hidden docs appear in source but not on docs.rs. This is the established pattern for `internal` re-exports in this codebase (consistent with `aggregate_network`, `LeidenGraph`), so no action needed.
- `refine.rs:26` — `RefinementState` is `pub` rather than `pub(crate)` as specified in the milestone. The deviation is intentional (the `internal` module re-export requires `pub`), and it matches the existing pattern for `LeidenGraph` and `AggregateResult`. Confirmed correct.

## Coverage Gaps
- The `well_connected_strong_connection_passes` test checks the exact-threshold case (`k_in_to == threshold`) with floating-point equality (`assert!(well_connected(2.1, 3, 10, 1.0))`). Floating-point rounding could in theory produce `2.1 - epsilon < threshold` on some platforms; a gap of `1e-9` margin would make the boundary test more robust. Low-priority since the threshold computation is simple arithmetic on integer-derived values.
- No explicit test for the `size_s == 0` short-circuit in `well_connected`, but this path is unreachable in normal operation (single-member communities are skipped in `refine_partition`). Gap is negligible.

## Drift Observations
- `mod.rs:138-147` — The pattern `if condition { break; }` immediately followed by `debug_assert!(!condition)` appears only once in the codebase. If this pattern is adopted elsewhere for invariant documentation, a convention note in `CLAUDE.md` would help future contributors distinguish "normal early-return" from "invariant-documenting dead assert."
- `refine.rs` — The `max_iter = 10` constant is a bare literal in `refine_community`. The local-move phase in `mod.rs` uses `cfg.max_iterations` passed down from `LeidenConfig`. Refinement's inner cap being a hardcoded literal (rather than a `LeidenConfig` field or named constant) is a mild inconsistency. No behaviour change needed for v0, but a `const MAX_REFINE_ITER: usize = 10;` at module scope would aid future tuning.
- `refinement.rs:295` — `prop_assert!` tolerance is `1e-9`; elsewhere in the test suite the convention is `1e-12`. Both are far tighter than any practical FMA drift, so this is purely cosmetic.
