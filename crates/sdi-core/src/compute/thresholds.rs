//! [`compute_thresholds_check`] — pure threshold evaluation for `sdi check`.

use serde::{Deserialize, Serialize};

use crate::input::ThresholdsInput;
use sdi_snapshot::delta::DivergenceSummary;

/// Information about a single threshold breach.
///
/// # Examples
///
/// ```rust
/// use sdi_core::compute::thresholds::ThresholdBreachInfo;
///
/// let b = ThresholdBreachInfo {
///     dimension: "pattern_entropy".to_string(),
///     actual: 3.5,
///     limit: 2.0,
/// };
/// assert!(b.actual > b.limit);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThresholdBreachInfo {
    /// Name of the dimension that exceeded its limit (e.g. `"pattern_entropy"`).
    pub dimension: String,
    /// Observed delta value.
    pub actual: f64,
    /// The limit that was exceeded.
    pub limit: f64,
}

/// Result of [`compute_thresholds_check`].
///
/// `breached` is `true` when at least one threshold was exceeded.  `breaches`
/// is empty on the first-snapshot path (all `DivergenceSummary` fields `None`).
///
/// # Examples
///
/// ```rust
/// use sdi_core::compute::thresholds::{compute_thresholds_check, ThresholdCheckResult};
/// use sdi_core::input::ThresholdsInput;
/// use sdi_snapshot::delta::null_summary;
///
/// let result = compute_thresholds_check(&null_summary(), &ThresholdsInput::default());
/// assert!(!result.breached);
/// assert!(result.breaches.is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThresholdCheckResult {
    /// `true` when at least one threshold was exceeded.
    pub breached: bool,
    /// Per-dimension details for each exceeded threshold.
    pub breaches: Vec<ThresholdBreachInfo>,
}

/// Checks whether any dimension of `summary` exceeds the rates in `cfg`.
///
/// **First-snapshot path:** when `summary` was produced by [`sdi_snapshot::delta::null_summary`]
/// (all fields `None`), no dimension can be checked and `breached` is `false`.
///
/// **Expiry:** per-category overrides in `cfg.overrides` are silently ignored
/// when `cfg.today` is after their `expires` date.  In M08 the per-category
/// rates do not affect the aggregate dimension check — that integration is
/// added when per-category entropy deltas are surfaced in the summary.
///
/// This function is **referentially transparent**: same inputs → same output.
/// It performs no I/O, reads no globals, and uses no clock.
///
/// # Examples
///
/// ```rust
/// use sdi_core::compute::thresholds::compute_thresholds_check;
/// use sdi_core::input::ThresholdsInput;
/// use sdi_snapshot::delta::DivergenceSummary;
///
/// let summary = DivergenceSummary {
///     pattern_entropy_delta: Some(5.0), // exceeds default 2.0
///     convention_drift_delta: Some(0.1),
///     coupling_delta: Some(0.05),
///     community_count_delta: Some(1),
///     boundary_violation_delta: None,
/// };
/// let cfg = ThresholdsInput::default();
/// let result = compute_thresholds_check(&summary, &cfg);
/// assert!(result.breached);
/// assert_eq!(result.breaches[0].dimension, "pattern_entropy");
/// ```
pub fn compute_thresholds_check(
    summary: &DivergenceSummary,
    cfg: &ThresholdsInput,
) -> ThresholdCheckResult {
    // TODO(M09): apply per-category override rates from cfg.overrides once
    // per-category entropy/drift deltas are surfaced in DivergenceSummary.
    // Until then, cfg.overrides and cfg.today are accepted but not read here.
    let mut breaches: Vec<ThresholdBreachInfo> = Vec::new();

    if let Some(delta) = summary.pattern_entropy_delta {
        let limit = cfg.pattern_entropy_rate;
        if delta > limit {
            breaches.push(ThresholdBreachInfo {
                dimension: "pattern_entropy".to_string(),
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
                actual: delta_f,
                limit,
            });
        }
    }

    let breached = !breaches.is_empty();
    ThresholdCheckResult { breached, breaches }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::ThresholdsInput;
    use sdi_snapshot::delta::{DivergenceSummary, null_summary};

    fn summary_with(
        entropy: Option<f64>,
        drift: Option<f64>,
        coupling: Option<f64>,
        violations: Option<i64>,
    ) -> DivergenceSummary {
        DivergenceSummary {
            pattern_entropy_delta: entropy,
            convention_drift_delta: drift,
            coupling_delta: coupling,
            community_count_delta: None,
            boundary_violation_delta: violations,
        }
    }

    #[test]
    fn null_summary_never_breaches() {
        let r = compute_thresholds_check(&null_summary(), &ThresholdsInput::default());
        assert!(!r.breached);
        assert!(r.breaches.is_empty());
    }

    #[test]
    fn entropy_breach_detected() {
        let s = summary_with(Some(3.0), None, None, None);
        let r = compute_thresholds_check(&s, &ThresholdsInput::default());
        assert!(r.breached);
        assert_eq!(r.breaches[0].dimension, "pattern_entropy");
        assert!((r.breaches[0].actual - 3.0).abs() < 1e-10);
        assert!((r.breaches[0].limit - 2.0).abs() < 1e-10);
    }

    #[test]
    fn entropy_at_limit_is_not_breached() {
        let s = summary_with(Some(2.0), None, None, None);
        let r = compute_thresholds_check(&s, &ThresholdsInput::default());
        assert!(!r.breached);
    }

    #[test]
    fn negative_delta_never_breaches() {
        let s = summary_with(Some(-10.0), Some(-10.0), Some(-0.5), Some(-5));
        let r = compute_thresholds_check(&s, &ThresholdsInput::default());
        assert!(!r.breached);
    }

    #[test]
    fn coupling_breach_detected() {
        let s = summary_with(None, None, Some(0.5), None);
        let r = compute_thresholds_check(&s, &ThresholdsInput::default());
        assert!(r.breached);
        assert_eq!(r.breaches[0].dimension, "coupling_delta");
    }

    #[test]
    fn boundary_violation_breach_detected() {
        let s = summary_with(None, None, None, Some(3));
        let r = compute_thresholds_check(&s, &ThresholdsInput::default());
        assert!(r.breached);
        assert_eq!(r.breaches[0].dimension, "boundary_violations");
    }

    #[test]
    fn multiple_breaches_all_reported() {
        let s = summary_with(Some(5.0), Some(5.0), Some(0.5), Some(10));
        let r = compute_thresholds_check(&s, &ThresholdsInput::default());
        assert!(r.breached);
        assert_eq!(r.breaches.len(), 4);
    }

    #[test]
    fn custom_cfg_raises_limit() {
        let mut cfg = ThresholdsInput::default();
        cfg.pattern_entropy_rate = 10.0;
        let s = summary_with(Some(5.0), None, None, None);
        let r = compute_thresholds_check(&s, &cfg);
        assert!(!r.breached);
    }
}
