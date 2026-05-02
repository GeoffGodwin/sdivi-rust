//! [`compute_thresholds_check`] — pure threshold evaluation for `sdivi check`.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::input::{ThresholdOverrideInput, ThresholdsInput};
use sdivi_snapshot::delta::DivergenceSummary;

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

/// Result of [`compute_thresholds_check`].
///
/// `breached` is `true` when at least one threshold was exceeded.  `breaches`
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
/// use sdivi_snapshot::delta::null_summary;
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

/// Checks whether any dimension of `summary` exceeds the rates in `cfg`.
///
/// **First-snapshot path:** when `summary` was produced by [`sdivi_snapshot::delta::null_summary`]
/// (all fields `None`), no dimension can be checked and `breached` is `false`.
///
/// **Aggregate dimensions** (`pattern_entropy_delta`, `convention_drift_delta`,
/// `coupling_delta`, `boundary_violation_delta`) always use the global rates in `cfg`,
/// regardless of per-category overrides.
///
/// **Per-category dimensions** (`pattern_entropy_per_category_delta`,
/// `convention_drift_per_category_delta`) use the per-category rate from an active
/// override when one exists for that category; otherwise fall back to the global rate.
/// An override is active when `cfg.today <= override.expires`.
///
/// This function is **referentially transparent**: same inputs → same output.
/// It performs no I/O, reads no globals, and uses no clock.
///
/// # Examples
///
/// ```rust
/// use sdivi_core::compute::thresholds::compute_thresholds_check;
/// use sdivi_core::input::{ThresholdOverrideInput, ThresholdsInput};
/// use sdivi_snapshot::delta::DivergenceSummary;
/// use chrono::NaiveDate;
/// use std::collections::BTreeMap;
///
/// // Aggregate breach (global rate 2.0, delta 5.0).
/// let summary = DivergenceSummary {
///     pattern_entropy_delta: Some(5.0),
///     convention_drift_delta: Some(0.1),
///     coupling_delta: Some(0.05),
///     community_count_delta: Some(1),
///     boundary_violation_delta: None,
///     pattern_entropy_per_category_delta: None,
///     convention_drift_per_category_delta: None,
/// };
/// let cfg = ThresholdsInput::default();
/// let result = compute_thresholds_check(&summary, &cfg);
/// assert!(result.breached);
/// assert_eq!(result.breaches[0].dimension, "pattern_entropy");
/// assert_eq!(result.breaches[0].category, None);
/// ```
pub fn compute_thresholds_check(
    summary: &DivergenceSummary,
    cfg: &ThresholdsInput,
) -> ThresholdCheckResult {
    let (applied_overrides, active_overrides) = resolve_overrides(cfg);
    let mut breaches: Vec<ThresholdBreachInfo> = Vec::new();

    // ── Aggregate dimension checks (always use global rate) ───────────────────
    if let Some(delta) = summary.pattern_entropy_delta {
        let limit = cfg.pattern_entropy_rate;
        if delta > limit {
            breaches.push(ThresholdBreachInfo {
                dimension: "pattern_entropy".to_string(),
                category: None,
                actual: delta,
                limit,
            });
        }
    }

    if let Some(delta) = summary.convention_drift_delta {
        let limit = cfg.convention_drift_rate;
        if delta > limit {
            breaches.push(ThresholdBreachInfo {
                dimension: "convention_drift".to_string(),
                category: None,
                actual: delta,
                limit,
            });
        }
    }

    if let Some(delta) = summary.coupling_delta {
        let limit = cfg.coupling_delta_rate;
        if delta > limit {
            breaches.push(ThresholdBreachInfo {
                dimension: "coupling_delta".to_string(),
                category: None,
                actual: delta,
                limit,
            });
        }
    }

    if let Some(delta) = summary.boundary_violation_delta {
        let limit = cfg.boundary_violation_rate;
        let delta_f = delta as f64;
        if delta_f > limit {
            breaches.push(ThresholdBreachInfo {
                dimension: "boundary_violations".to_string(),
                category: None,
                actual: delta_f,
                limit,
            });
        }
    }

    // ── Per-category checks (active override replaces global rate for that category) ──
    if let Some(per_cat) = &summary.pattern_entropy_per_category_delta {
        for (cat, &delta) in per_cat {
            let limit = active_overrides
                .get(cat)
                .and_then(|ov| ov.pattern_entropy_rate)
                .unwrap_or(cfg.pattern_entropy_rate);
            if delta > limit {
                breaches.push(ThresholdBreachInfo {
                    dimension: "pattern_entropy".to_string(),
                    category: Some(cat.clone()),
                    actual: delta,
                    limit,
                });
            }
        }
    }

    if let Some(per_cat) = &summary.convention_drift_per_category_delta {
        for (cat, &delta) in per_cat {
            let limit = active_overrides
                .get(cat)
                .and_then(|ov| ov.convention_drift_rate)
                .unwrap_or(cfg.convention_drift_rate);
            if delta > limit {
                breaches.push(ThresholdBreachInfo {
                    dimension: "convention_drift".to_string(),
                    category: Some(cat.clone()),
                    actual: delta,
                    limit,
                });
            }
        }
    }

    let breached = !breaches.is_empty();
    ThresholdCheckResult {
        breached,
        breaches,
        applied_overrides,
    }
}

/// Resolves `cfg.overrides` into applied-override diagnostics and a map of active overrides.
///
/// Returns `(diagnostics, active_overrides)`:
/// - `diagnostics`: every entry with `active` flag and `expires` date (for `applied_overrides`)
/// - `active_overrides`: only entries where `cfg.today <= expires` (used for rate lookup)
fn resolve_overrides(
    cfg: &ThresholdsInput,
) -> (
    BTreeMap<String, AppliedOverrideInfo>,
    BTreeMap<String, &ThresholdOverrideInput>,
) {
    let mut diagnostics: BTreeMap<String, AppliedOverrideInfo> = BTreeMap::new();
    let mut active: BTreeMap<String, &ThresholdOverrideInput> = BTreeMap::new();

    for (cat, ov) in &cfg.overrides {
        match NaiveDate::parse_from_str(&ov.expires, "%Y-%m-%d") {
            Err(e) => {
                diagnostics.insert(
                    cat.clone(),
                    AppliedOverrideInfo {
                        active: false,
                        // Sentinel date — parse failed; override is inactive.
                        expires: NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
                        expired_reason: Some(format!("failed to parse expires date: {e}")),
                    },
                );
            }
            Ok(expires) => {
                // today == expires → still active; only today > expires → expired (Rule 12).
                let is_active = cfg.today <= expires;
                let expired_reason = if is_active {
                    None
                } else {
                    Some(format!("expired on {expires}"))
                };
                diagnostics.insert(
                    cat.clone(),
                    AppliedOverrideInfo {
                        active: is_active,
                        expires,
                        expired_reason,
                    },
                );
                if is_active {
                    active.insert(cat.clone(), ov);
                }
            }
        }
    }

    (diagnostics, active)
}
