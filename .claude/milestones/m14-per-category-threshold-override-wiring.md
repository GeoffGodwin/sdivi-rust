#### Milestone 14: Per-Category Threshold Override Wiring
<!-- milestone-meta
id: "14"
status: "done"
-->


**Scope:** Make `ThresholdsInput::overrides` and `ThresholdsInput::today` actually load-bearing. Surface per-category breakouts in `PatternMetricsResult` and `DivergenceSummary` so `compute_thresholds_check` has something to filter against. Wire active (non-expired) overrides as the per-category limit. Retire the orphaned `TODO(M09)` markers and the `M08`/`M09`-named stub tests. Schema stays `"1.0"` — every change is additive.

**Why this milestone exists:** `ThresholdsInput::overrides` and `ThresholdsInput::today` are committed `pub` API as of M08. Per CLAUDE.md Rule 18 these become permanent SemVer commitments at the v0.1.0 tag — and right now they are silent no-ops (`crates/sdivi-core/src/compute/thresholds.rs:92-94`). CLAUDE.md Rule 12 documents per-category overrides as a working feature; init writes `[thresholds.overrides.<cat>]` examples into every new repo's config. Shipping v0 with a documented-but-vacuous knob locks the vacuous behavior. The remaining open `HUMAN_ACTION_REQUIRED` item explicitly demands a decision before tag time; this milestone is that decision.

**Deliverables:**

- **Per-category fields on `PatternMetricsResult`** (additive):
  - `convention_drift_per_category: BTreeMap<String, f64>` — same formula as the existing scalar `convention_drift` (distinct fingerprints divided by total instance count) but kept per-category instead of averaged. The existing `convention_drift` scalar stays as the average for backward-compat consumers; the new field is the source of truth for per-category override filtering.
  - `entropy_per_category` already exists — no change.
  - Computed in `sdivi_core::compute::patterns::compute_pattern_metrics` and in `sdivi-pipeline::pipeline::compute_pattern_metrics_from_catalog` (both call sites must produce the same map for the same input).
- **Per-category delta fields on `DivergenceSummary`** (additive):
  - `pattern_entropy_per_category_delta: Option<BTreeMap<String, f64>>` — `None` on the first-snapshot path (KDD-9 semantics preserved); `Some(map)` otherwise. Map keys are the union of categories present in either `prev` or `curr`; missing-side values are treated as `0.0` so a newly-introduced category surfaces as a positive delta.
  - `convention_drift_per_category_delta: Option<BTreeMap<String, f64>>` — same shape, same null semantics.
  - `null_summary()` updated to set both new fields to `None`.
  - `compute_delta` updated to populate both from `prev.pattern_metrics.{entropy_per_category, convention_drift_per_category}` vs. the same on `curr`.
- **Override wiring in `compute_thresholds_check`:**
  - Build a per-category effective rate map: for each `(category, override)` in `cfg.overrides`, parse `override.expires` as `NaiveDate` (delegated to the existing `validate_date_format` from `sdivi-config`'s pure validators); skip if `cfg.today > expires` (silent ignore, per Rule 12); otherwise the override's `pattern_entropy_rate` / `convention_drift_rate` / `coupling_delta_rate` / `boundary_violation_rate` (each `Option<f64>`) replaces the global rate **only for that category**.
  - Aggregate dimensions (`summary.pattern_entropy_delta`, `summary.convention_drift_delta`) continue to use the global rate. Per-category breaches use the per-category effective rate.
  - The existing aggregate breach evaluation stays exactly as-is. The new evaluation iterates over the per-category delta maps when present and emits one `ThresholdBreachInfo` per breaching category.
- **`ThresholdBreachInfo` gains `category: Option<String>`** (additive). `None` for the existing aggregate breaches; `Some("error_handling")` for per-category breaches. Existing `breaches[0].dimension == "pattern_entropy"` shape is unchanged.
- **`ThresholdCheckResult` gains `applied_overrides: BTreeMap<String, AppliedOverrideInfo>`** (additive) — diagnostic surface for `sdivi check --format json` consumers and for the consumer app: which overrides were considered, which were active, which were expired. `AppliedOverrideInfo { active: bool, expires: NaiveDate, expired_reason: Option<String> }`. CLI text output may render this as a small table under the breach list when non-empty.
- **Retirement of stale milestone markers:**
  - `crates/sdivi-core/src/compute/thresholds.rs:92-94` — delete the `TODO(M09)` block; replace with a one-line note describing the per-category dispatch only if the implementation isn't self-evident.
  - `crates/sdivi-core/tests/compute_thresholds_check.rs:107` — rename `override_not_wired_in_m08_base_rate_applies` to `active_override_raises_per_category_limit` and rewrite to assert the new behavior (an unexpired entropy override at 50.0 prevents a per-category entropy=3.0 from breaching while the global rate of 2.0 is still applied to the aggregate dimension).
  - `crates/sdivi-core/tests/compute_thresholds_check.rs:125` — rename `base_rate_applies_regardless_of_override_state_m08` to `expired_override_falls_back_to_global_rate` and update to assert that an expired override is silently ignored.
  - All `M08:` / `M09:` / `(wired up in M09)` annotation comments in test bodies removed.
- **`compute_thresholds_check` doc test in rustdoc** updated to demonstrate an active override; existing first-snapshot doc test preserved.
- **CHANGELOG.md** entry under the next-release section: "Threshold overrides are now active. `ThresholdsInput.overrides` and `ThresholdsInput.today` filter per-category breaches against expiry. Snapshot schema stays `1.0`; new `pattern_metrics.convention_drift_per_category`, `delta.pattern_entropy_per_category_delta`, `delta.convention_drift_per_category_delta`, and `ThresholdBreachInfo.category` are additive."

**Files to create or modify:**

- `crates/sdivi-snapshot/src/snapshot.rs` — extend `PatternMetricsResult` with `convention_drift_per_category`. Update `Default` impl to empty `BTreeMap`.
- `crates/sdivi-snapshot/src/delta.rs` — extend `DivergenceSummary` with the two new `Option<BTreeMap<String, f64>>` fields. Update `null_summary()` and `compute_delta`.
- `crates/sdivi-snapshot/tests/delta_proptest.rs` (or equivalent existing proptest module) — extend the `prop_test_delta_pure` regression to cover the new fields' purity.
- `crates/sdivi-core/src/compute/patterns.rs` — populate `convention_drift_per_category` in `compute_pattern_metrics`. Update doc test.
- `crates/sdivi-pipeline/src/pipeline.rs` — populate `convention_drift_per_category` in `compute_pattern_metrics_from_catalog` to match the pure-compute path byte-for-byte.
- `crates/sdivi-pipeline/tests/parity.rs` (extend M11's pure-compute parity check, if present) — assert `convention_drift_per_category` matches between pipeline and pure-compute paths.
- `crates/sdivi-core/src/compute/thresholds.rs` — implement override + expiry logic; populate `applied_overrides`; populate `category` on per-category breaches. Delete the orphaned `TODO(M09)` block.
- `crates/sdivi-core/src/input/types.rs` — add `AppliedOverrideInfo` (or equivalent type) used by `ThresholdCheckResult`. Add `category: Option<String>` to `ThresholdBreachInfo` (currently in `compute/thresholds.rs`; keep it there unless input/types is the cleaner home).
- `crates/sdivi-core/tests/compute_thresholds_check.rs` — rename two stub tests, rewrite their bodies, add: `active_override_blocks_per_category_breach`, `expired_override_falls_back_to_global_rate`, `aggregate_dimension_uses_global_rate_when_only_one_category_is_overridden`, `applied_overrides_reports_active_and_expired_separately`.
- `crates/sdivi-cli/src/commands/check.rs` — pass through the new `applied_overrides` to JSON output; CLI text output gains a "applied overrides" line when non-empty.
- `crates/sdivi-cli/tests/check_format.rs` — assert `applied_overrides` round-trips in JSON output; add a fixture exercising an active override.
- `bindings/sdivi-wasm/src/lib.rs` — re-export the extended `ThresholdsInput` / `ThresholdCheckResult` shape; tsify regenerates `.d.ts`. No new exports needed (the extension is on existing types).
- `bindings/sdivi-wasm/tests/wasm_bindgen_thresholds.rs` (or equivalent) — smoke-test the new fields are visible from WASM.
- `CHANGELOG.md` — entry as above.
- `docs/library-embedding.md` — short addendum showing a consumer-app-style caller supplying `today` and `overrides`.

**Acceptance criteria:**

- `compute_thresholds_check` returns `breached = false` when an active override raises the per-category limit above the observed per-category delta, even though the global rate would have flagged the same value. (New test `active_override_blocks_per_category_breach` covers this.)
- `compute_thresholds_check` returns `breached = true` when the override is expired (`cfg.today > expires`), with the breach using the global rate. (New test `expired_override_falls_back_to_global_rate` covers this.)
- `ThresholdCheckResult.applied_overrides` enumerates every `cfg.overrides` entry with its `active` flag and parsed `expires`.
- `compute_delta` populates the two new per-category delta maps with the union-of-keys-zero-fill rule.
- `null_summary()` returns `None` for both new fields.
- `snapshot_version` remains `"1.0"`. Reading an M13-era snapshot produces no warning and yields aggregate-only deltas (the per-category fields default to `None`).
- `cargo build -p sdivi-core --target wasm32-unknown-unknown --no-default-features` still succeeds with zero forbidden deps.
- The existing M11 bifl-tracker validation harness still passes within the documented tolerances against the new code (run as a regression gate).
- No `TODO(M09)`, `M08`, `M09`, or `(wired up in M09)` strings remain in `crates/sdivi-core/src/` or `crates/sdivi-core/tests/`. (Historical-context comments referring to "the M08 offset fix" in unrelated code paths are out of scope.)
- `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` pass.
- `cargo test --workspace` passes including doctests.

**Tests:**

- Unit: `active_override_blocks_per_category_breach`, `expired_override_falls_back_to_global_rate`, `applied_overrides_reports_active_and_expired_separately`, `aggregate_dimension_uses_global_rate_when_only_one_category_is_overridden`, `category_present_in_curr_only_surfaces_positive_delta`, `category_present_in_prev_only_surfaces_negative_delta`.
- Property: extend the existing `prop_test_compute_thresholds_check_pure` to randomly generate override maps with mixed expiry dates and assert purity (same `cfg`, same `summary` → same `ThresholdCheckResult`).
- Property: a new `prop_test_delta_per_category_pure` over random per-category metric maps.
- Doctests on `ThresholdsInput`, `ThresholdCheckResult`, `compute_thresholds_check`, `compute_delta`, `null_summary` updated to match new shapes.
- Schema-stability test (existing or new): a serialized M13 snapshot deserializes cleanly and yields aggregate-only deltas with the per-category fields `None`.
- Cross-binding parity smoke test: serialize a `ThresholdCheckResult` with `applied_overrides` populated, round-trip through `serde-wasm-bindgen`, assert equality.

**Watch For:**

- **Doc-comment placement.** Per CLAUDE.md Code Conventions, when inserting `category: Option<String>` into `ThresholdBreachInfo`, ensure a blank line separates the new field's `///` block from the next field's `///` block. The file is in `sdivi-core` where `#![deny(missing_docs)]` will catch a missing doc, but won't catch a doc that silently re-attaches to the wrong field.
- **Aggregate-vs-per-category semantics.** It is intentional that the global `pattern_entropy_rate` continues to apply to the aggregate `summary.pattern_entropy_delta` even when per-category overrides exist. An override of `pattern_entropy_rate = 5.0` for category `error_handling` only suppresses breaches *of the error_handling per-category delta*, never of the aggregate. Tests must lock this distinction down — otherwise a future refactor could collapse them.
- **Expiry-boundary date.** `cfg.today > expires` is the silent-ignore condition. `cfg.today == expires` should keep the override active (the override "expires" *after* the named date, not on it). Test both boundaries explicitly.
- **`expires` parsing.** `validate_date_format` exists in `sdivi-config` outside the `loader` feature gate (M08 deliverable). If for any reason `compute_thresholds_check` cannot parse a stored `expires` string, the override is treated as inactive and a structured note is added to `applied_overrides[<cat>].expired_reason`. `compute_thresholds_check` does not return `Err` for malformed overrides — config-load-time validation already rejects those at `ConfigError::InvalidValue`.
- **Schema additivity, not bumping.** Snapshot schema stays `"1.0"`. New fields use `#[serde(default)]` so M13-era snapshots deserialize cleanly. No `MIGRATION_NOTES.md` entry required.
- **Stale `M08`/`M09` strings outside thresholds tests.** `git grep "M0[89]"` in `crates/sdivi-core/` is the verification hammer. Historical-context comments in unrelated tests (e.g., `leiden_id_collision.rs`'s "Fix (M08): ...") are descriptive and stay; the test names and TODO markers in `compute_thresholds_check.rs` are the targets.
- **HAR file.** The unchecked `[ ]` item in `.tekhton/HUMAN_ACTION_REQUIRED.md` (currently misfiled under `## Resolved`) should be checked off and moved to a properly-formatted Resolved entry by the milestone-closing commit.

**Seeds Forward:**

- Per-category metric breakouts in `PatternMetricsResult` and `DivergenceSummary` are now public surface. Future analyzers (e.g., the change-coupling work in M15) can follow the same pattern: aggregate field for backward-compat consumers, per-category map for fine-grained gating.
- The `applied_overrides` diagnostic surface is the precedent for any future "rule was considered but not fired" reporting in `compute_*` functions. Consumer-app dashboards will lean on it.
- v0.x can introduce per-category override of `coupling_delta_rate` and `boundary_violation_rate` against future per-category deltas with no schema bump — the input shape already accepts the rates.

---
