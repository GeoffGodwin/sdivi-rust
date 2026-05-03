#### Milestone 20: Threshold-Comparison Epsilon for Cross-Arch Stability

<!-- milestone-meta
id: "20"
status: "todo"
-->

**Scope:** Add a small fixed epsilon (`1e-9`) inside `compute_thresholds_check` so that the strict `>` comparisons against threshold rates cannot flip sign across platforms purely from documented per-arch ULP drift in upstream `compute_delta` / `compute_trend` outputs. Raw `DivergenceSummary` and trend values are left untouched — only the gate-comparison is rounded. Document the new contract in `docs/determinism.md`.

**Why this milestone exists:** `docs/determinism.md` (M11) accepts that aggregate float results may diverge by ~1 ULP between x86_64 and aarch64 runners due to FMA differences. That's fine for display, but `compute_thresholds_check` uses raw `delta > limit` comparisons — so a user-configured threshold of *exactly* `0.05` could return `breached: false` on x86_64 (where the delta computed as `0.04999999999999998`) and `breached: true` on aarch64 (`0.05000000000000001`). For a CI gate, that's a flaky test. The Meridian consumer-app integration is the first concrete user that runs `compute_thresholds_check` on multiple architectures from the same snapshot inputs; getting ahead of this before adopters report it.

**Theoretical basis:** The thresholds are user-facing dial values typically expressed to 1–2 decimal places (e.g. `coupling_delta_rate = 0.15`). A `1e-9` epsilon is ~7 orders of magnitude smaller than any plausible user-meaningful precision; it absorbs ULP drift without changing semantics for any user who isn't already at sub-nanounit precision (and we have no such user). The epsilon is added to `limit`, not subtracted from `delta`, so the gate still triggers on any genuine breach: `breached := delta > limit + EPSILON`. A delta of `limit + 2e-9` still trips; a delta of `limit + 5e-10` does not. The asymmetry is deliberate — false positives (CI gates flapping on noise) are more costly than false negatives at the ULP scale.

**Deliverables:**
- Add `pub const THRESHOLD_EPSILON: f64 = 1e-9;` at the top of `crates/sdivi-core/src/compute/thresholds.rs` with a doc comment explaining the rationale and citing `docs/determinism.md`.
- Replace each of the four aggregate-dimension `delta > limit` comparisons (lines ~155, ~167, ~179, ~192 in current thresholds.rs) with `delta > limit + THRESHOLD_EPSILON`. Same for the per-category comparison around line 209 and any remaining sites within the function.
- The `boundary_violation_delta` comparison: `delta` is integer-valued (`i64` cast to `f64`). Apply the epsilon for consistency, but note in a code comment that it has no functional effect for integer deltas.
- Document the epsilon in `docs/determinism.md` under a new "Threshold gate stability" subsection: state the constant, the rationale, the asymmetric application, and the guarantee that *any* real breach above ULP noise still trips the gate.
- Re-export `THRESHOLD_EPSILON` from `sdivi-core::lib` so consumers (including the WASM bindings) can reference the same constant when documenting their own gates.

**Migration Impact:** Behaviour change is bounded by `1e-9` per dimension. A user whose threshold is `0.05` and whose computed delta is exactly `0.05000000000000001` will see `breached: false` post-M20 where they saw `breached: true` pre-M20 (or vice versa, depending on which arch they were on). Anyone running the gate on the same arch consistently sees no behavioural change beyond the epsilon's 1e-9 margin. CHANGELOG entry: "Threshold gate now applies a 1e-9 epsilon to the limit, eliminating cross-arch flap from documented per-arch ULP drift in delta computations. Behaviour for any user-meaningful threshold is unchanged."

**Files to create or modify:**
- **Modify:** `crates/sdivi-core/src/compute/thresholds.rs` — add constant, replace comparisons.
- **Modify:** `crates/sdivi-core/src/lib.rs` — re-export `THRESHOLD_EPSILON`.
- **Modify:** `docs/determinism.md` — add "Threshold gate stability" subsection.
- **Modify:** `CHANGELOG.md` — under Changed.

**Acceptance criteria:**
- `cargo test -p sdivi-core` passes, with a new test asserting a delta of `limit + 5e-10` does not breach and a delta of `limit + 2e-9` does breach.
- Existing threshold-check tests continue to pass without modification (the epsilon does not affect any test that uses non-borderline values).
- `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` pass.
- `cargo doc --workspace --no-deps` passes with `RUSTDOCFLAGS=-D warnings`.
- The `THRESHOLD_EPSILON` constant appears in the public docs of `sdivi-core` (visible on docs.rs after publish).

**Tests:**
- Unit (thresholds.rs): two cases per dimension — `limit + 5e-10` not breached, `limit + 2e-9` breached. Includes the per-category override path.
- Unit: `THRESHOLD_EPSILON` is exactly `1e-9` (regression gate; changing the value is a deliberate decision).
- Property (proptest): for any `limit > 0` and any `delta`, `breached(delta, limit) == (delta > limit + THRESHOLD_EPSILON)`. Trivially true given the implementation but catches accidental refactors.

**Watch For:**
- **Don't apply epsilon to the actual reported `delta`.** The `ThresholdBreachInfo.actual` field is the raw delta, unrounded. Only the gate comparison is rounded. A user reading the breach report sees the true value.
- **Don't subtract the epsilon from `delta` instead of adding to `limit`.** Mathematically equivalent in IEEE-754 only if both are representable; `limit + EPSILON` is the cleaner form and avoids subtracting at the dimension boundary.
- **Don't apply per-arch conditional epsilons.** The epsilon is a constant. Branching on `cfg!(target_arch = "aarch64")` would create a new flap source.
- **Don't bump the epsilon casually.** `1e-9` was chosen because it's well above documented per-arch FMA drift (typically 1 ULP on values near 1.0, ~`2.2e-16`) and well below any plausible user-meaningful threshold. Bumping to `1e-6` would mask legitimate drift in a "true breach is 0.150001 vs limit 0.15" scenario. Keep it small.
- **Documentation must call out the asymmetry.** A user reading `docs/determinism.md` should understand that the gate is *slightly more lenient* by an absorbed 1 ULP, not symmetric.

**Seeds Forward:**
- A future milestone could expose `THRESHOLD_EPSILON` as a config-overridable knob (`[determinism] threshold_epsilon = 1e-9`) if a downstream user has unusual precision needs. Not in scope for v0.
- If a future delta-dimension is added to `DivergenceSummary`, it must use the same epsilon. Document this in the rustdoc on `THRESHOLD_EPSILON` so the next contributor doesn't miss a comparison site.
- The epsilon does not address per-snapshot non-determinism (which Rule 1 forbids outright). It only addresses the documented cross-arch ULP variance after the snapshot is produced.

---
