//! Property test: `compute_thresholds_check` is a pure function.
//!
//! Same `DivergenceSummary` + same `ThresholdsInput` always produces the same
//! `ThresholdCheckResult` — no side effects, no global state, no clock.

use proptest::prelude::*;
use sdi_core::compute::thresholds::compute_thresholds_check;
use sdi_core::input::ThresholdsInput;
use sdi_snapshot::delta::DivergenceSummary;

fn arb_summary() -> impl Strategy<Value = DivergenceSummary> {
    (
        proptest::option::of(-5.0f64..10.0),
        proptest::option::of(-5.0f64..10.0),
        proptest::option::of(-1.0f64..1.0),
        proptest::option::of(-10i64..10),
        proptest::option::of(-5i64..20),
    )
    .prop_map(|(ped, cdd, cd, ccd, bvd)| DivergenceSummary {
        pattern_entropy_delta: ped,
        convention_drift_delta: cdd,
        coupling_delta: cd,
        community_count_delta: ccd,
        boundary_violation_delta: bvd,
    })
}

fn arb_thresholds() -> impl Strategy<Value = ThresholdsInput> {
    (0.1f64..10.0, 0.1f64..10.0, 0.01f64..1.0, 0.1f64..10.0).prop_map(
        |(pe, cd, coup, bv)| ThresholdsInput {
            pattern_entropy_rate: pe,
            convention_drift_rate: cd,
            coupling_delta_rate: coup,
            boundary_violation_rate: bv,
            ..ThresholdsInput::default()
        },
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// `compute_thresholds_check` is referentially transparent.
    #[test]
    fn prop_test_compute_thresholds_check_pure(
        summary in arb_summary(),
        thresholds in arb_thresholds(),
    ) {
        let r1 = compute_thresholds_check(&summary, &thresholds);
        let r2 = compute_thresholds_check(&summary, &thresholds);
        prop_assert_eq!(
            serde_json::to_string(&r1).unwrap(),
            serde_json::to_string(&r2).unwrap(),
            "compute_thresholds_check must be referentially transparent",
        );
    }

    /// If a delta is `None`, the corresponding dimension is never breached.
    #[test]
    fn prop_none_delta_never_breaches(thresholds in arb_thresholds()) {
        let summary = DivergenceSummary {
            pattern_entropy_delta: None,
            convention_drift_delta: None,
            coupling_delta: None,
            community_count_delta: None,
            boundary_violation_delta: None,
        };
        let r = compute_thresholds_check(&summary, &thresholds);
        prop_assert!(!r.breached, "null summary must never breach");
    }
}
