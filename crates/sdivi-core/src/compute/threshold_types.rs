//! Public types for [`super::thresholds::compute_thresholds_check`].

use std::collections::BTreeMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Epsilon added to threshold limits to absorb cross-arch ULP drift.
///
/// Gates use `delta > limit + THRESHOLD_EPSILON` rather than `delta > limit`.
/// `1e-9` is ~7 orders of magnitude below any user-meaningful threshold
/// (typically 1–2 decimal places) and well above documented per-arch FMA
/// drift (~1 ULP ≈ `2.2e-16`). The gate is *slightly more lenient* by at
/// most `1e-9`; genuine breaches ≥ `limit + 2e-9` still trip.
///
/// For integer-valued dimensions (`boundary_violation_delta`) the epsilon
/// has no functional effect but is applied for consistency.
///
/// If a future delta dimension is added to `DivergenceSummary`, use the
/// same `delta > limit + THRESHOLD_EPSILON` form at the new comparison site.
///
/// See `docs/determinism.md § Threshold gate stability` for the full rationale.
pub const THRESHOLD_EPSILON: f64 = 1e-9;

/// Information about a single threshold breach.
///
/// `category` is `None` for aggregate-dimension breaches; `Some("error_handling")`
/// for per-category breaches produced by per-category delta maps.
///
/// # Examples
///
/// ```rust
/// use sdivi_core::compute::thresholds::ThresholdBreachInfo;
///
/// let b = ThresholdBreachInfo {
///     dimension: "pattern_entropy".to_string(),
///     category: None,
///     actual: 3.5,
///     limit: 2.0,
/// };
/// assert!(b.actual > b.limit);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThresholdBreachInfo {
    /// Name of the dimension that exceeded its limit (e.g. `"pattern_entropy"`).
    pub dimension: String,

    /// Category name for per-category breaches; `None` for aggregate-dimension breaches.
    ///
    /// Absent from JSON when `None` so existing aggregate-breach consumers are unaffected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Observed delta value.
    pub actual: f64,

    /// The limit that was exceeded.
    pub limit: f64,
}

/// Diagnostic information for one entry in `ThresholdCheckResult::applied_overrides`.
///
/// # Examples
///
/// ```rust
/// use sdivi_core::compute::thresholds::AppliedOverrideInfo;
/// use chrono::NaiveDate;
///
/// let info = AppliedOverrideInfo {
///     active: true,
///     expires: NaiveDate::from_ymd_opt(2030, 1, 1).unwrap(),
///     expired_reason: None,
/// };
/// assert!(info.active);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppliedOverrideInfo {
    /// `true` when the override's `expires` date is on or after `today`.
    pub active: bool,
    /// Parsed expiry date.
    pub expires: NaiveDate,
    /// Human-readable explanation when the override is inactive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expired_reason: Option<String>,
}

/// Result of [`super::thresholds::compute_thresholds_check`].
///
/// `breached` is `true` when at least one threshold was exceeded. `breaches`
/// is empty on the first-snapshot path (all `DivergenceSummary` fields `None`).
///
/// `applied_overrides` enumerates every entry from `cfg.overrides` with an
/// `active` flag and parsed `expires`, for diagnostic consumers and `sdivi check --format json`.
///
/// # Examples
///
/// ```rust
/// use sdivi_core::compute::thresholds::{compute_thresholds_check, ThresholdCheckResult};
/// use sdivi_core::input::ThresholdsInput;
/// use sdivi_core::null_summary;
///
/// let result = compute_thresholds_check(&null_summary(), &ThresholdsInput::default());
/// assert!(!result.breached);
/// assert!(result.breaches.is_empty());
/// assert!(result.applied_overrides.is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThresholdCheckResult {
    /// `true` when at least one threshold was exceeded.
    pub breached: bool,
    /// Per-dimension details for each exceeded threshold.
    pub breaches: Vec<ThresholdBreachInfo>,
    /// Diagnostic map of every override entry: which were active, which expired.
    pub applied_overrides: BTreeMap<String, AppliedOverrideInfo>,
}
