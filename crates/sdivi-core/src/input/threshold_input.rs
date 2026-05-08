//! Threshold input structs for the pure-compute API.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

// ── Threshold inputs ──────────────────────────────────────────────────────────
/// Per-category threshold override for [`ThresholdsInput`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThresholdOverrideInput {
    /// Overridden pattern entropy rate (uses default if absent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern_entropy_rate: Option<f64>,
    /// Overridden convention drift rate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub convention_drift_rate: Option<f64>,
    /// Overridden coupling delta rate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coupling_delta_rate: Option<f64>,
    /// Overridden boundary violation rate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boundary_violation_rate: Option<f64>,
    /// ISO-8601 expiry date (`"YYYY-MM-DD"`).
    pub expires: String,
}

/// Threshold configuration for [`crate::compute_thresholds_check`].
///
/// The caller supplies `today` explicitly — no clock access in sdivi-core.
///
/// **IMPORTANT:** `ThresholdsInput::default()` sets `today` to a far-future
/// sentinel (`9999-12-31`) so that all per-category overrides are treated as
/// expired (i.e., the global rates apply).  Callers that use per-category
/// overrides MUST supply the real current date:
///
/// ```rust
/// # use chrono::NaiveDate;
/// use sdivi_core::input::ThresholdsInput;
///
/// let today = NaiveDate::from_ymd_opt(2026, 4, 30).unwrap();
/// let t = ThresholdsInput { today, ..ThresholdsInput::default() };
/// assert_eq!(t.pattern_entropy_rate, 2.0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThresholdsInput {
    /// Maximum allowed pattern entropy rate.
    pub pattern_entropy_rate: f64,
    /// Maximum allowed convention drift rate.
    pub convention_drift_rate: f64,
    /// Maximum allowed coupling delta rate.
    pub coupling_delta_rate: f64,
    /// Maximum allowed boundary violation rate.
    pub boundary_violation_rate: f64,
    /// Per-category overrides (may include expired entries — `today` determines which apply).
    #[serde(default)]
    pub overrides: BTreeMap<String, ThresholdOverrideInput>,
    /// Today's date for expiry evaluation.  Caller supplies this (no clock in sdivi-core).
    /// `Default` uses `9999-12-31`; override with the real date to enable per-category filtering.
    pub today: chrono::NaiveDate,
}

impl Default for ThresholdsInput {
    fn default() -> Self {
        ThresholdsInput {
            pattern_entropy_rate: 2.0,
            convention_drift_rate: 3.0,
            coupling_delta_rate: 0.15,
            boundary_violation_rate: 2.0,
            overrides: BTreeMap::new(),
            // Far-future sentinel — callers must supply the real `today` to enable override filtering.
            today: chrono::NaiveDate::from_ymd_opt(9999, 12, 31).unwrap(),
        }
    }
}
