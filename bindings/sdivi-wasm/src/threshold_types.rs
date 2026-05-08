//! Threshold-related Tsify wrapper types split from `types.rs` to keep that
//! file under the 300-line guideline.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

/// Per-category threshold override.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmThresholdOverrideInput {
    #[tsify(optional)]
    pub pattern_entropy_rate: Option<f64>,
    #[tsify(optional)]
    pub convention_drift_rate: Option<f64>,
    #[tsify(optional)]
    pub coupling_delta_rate: Option<f64>,
    #[tsify(optional)]
    pub boundary_violation_rate: Option<f64>,
    /// ISO-8601 expiry date `"YYYY-MM-DD"`.
    pub expires: String,
}

/// Threshold configuration for [`compute_thresholds_check`].
/// `today` is an ISO-8601 date string `"YYYY-MM-DD"`.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmThresholdsInput {
    pub pattern_entropy_rate: f64,
    pub convention_drift_rate: f64,
    pub coupling_delta_rate: f64,
    pub boundary_violation_rate: f64,
    #[serde(default)]
    pub overrides: BTreeMap<String, WasmThresholdOverrideInput>,
    /// ISO-8601 date for threshold expiry evaluation (e.g. `"2026-05-01"`).
    pub today: String,
}

/// A single threshold breach.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmThresholdBreachInfo {
    /// Name of the dimension that exceeded its limit.
    pub dimension: String,
    /// Category name for per-category breaches; absent for aggregate breaches.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[tsify(optional)]
    pub category: Option<String>,
    /// Observed delta value.
    pub actual: f64,
    /// The limit that was exceeded.
    pub limit: f64,
}

/// Diagnostic info for one entry in `WasmThresholdCheckResult::applied_overrides`.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmAppliedOverrideInfo {
    /// Whether the override was active (not expired) at evaluation time.
    pub active: bool,
    /// Expiry date as `"YYYY-MM-DD"`.
    pub expires: String,
    /// Human-readable explanation when inactive.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[tsify(optional)]
    pub expired_reason: Option<String>,
}

/// Output of [`compute_thresholds_check`].
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmThresholdCheckResult {
    /// `true` when at least one threshold was exceeded.
    pub breached: bool,
    /// Per-dimension details for each exceeded threshold.
    pub breaches: Vec<WasmThresholdBreachInfo>,
    /// Diagnostic map of every override entry with `active` flag and `expires` date.
    pub applied_overrides: BTreeMap<String, WasmAppliedOverrideInfo>,
}
