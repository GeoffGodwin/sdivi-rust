//! Tests for `sdivi check --format json` with applied_overrides.

use assert_cmd::Command;
use tempfile::TempDir;

fn sdivi() -> Command {
    Command::cargo_bin("sdivi").expect("sdivi binary must be built")
}

fn empty_repo() -> TempDir {
    tempfile::tempdir().unwrap()
}

/// `sdivi check --format json` always emits an `applied_overrides` field.
#[test]
fn check_json_has_applied_overrides_field() {
    let repo = empty_repo();

    let out = sdivi()
        .arg("--repo")
        .arg(repo.path())
        .arg("check")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    let stdout = String::from_utf8(out.stdout).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("check --format json stdout must be valid JSON");

    assert!(
        parsed.get("applied_overrides").is_some(),
        "check JSON must have 'applied_overrides'; got: {parsed}"
    );
    // With no config overrides, the map is empty.
    assert_eq!(
        parsed["applied_overrides"].as_object().map(|m| m.len()),
        Some(0),
        "applied_overrides must be empty when no overrides are configured"
    );
}

/// `applied_overrides` round-trips through JSON serialization.
#[test]
fn applied_overrides_round_trips_in_json() {
    use sdivi_core::compute::thresholds::{AppliedOverrideInfo, ThresholdCheckResult};

    let result = ThresholdCheckResult {
        breached: false,
        breaches: vec![],
        applied_overrides: {
            let mut m = std::collections::BTreeMap::new();
            m.insert(
                "error_handling".to_string(),
                AppliedOverrideInfo {
                    active: true,
                    expires: chrono::NaiveDate::from_ymd_opt(2030, 1, 1).unwrap(),
                    expired_reason: None,
                },
            );
            m.insert(
                "logging".to_string(),
                AppliedOverrideInfo {
                    active: false,
                    expires: chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                    expired_reason: Some("expired on 2020-01-01".to_string()),
                },
            );
            m
        },
    };

    let json = serde_json::to_string(&result).unwrap();
    let decoded: ThresholdCheckResult = serde_json::from_str(&json).unwrap();
    assert_eq!(
        result, decoded,
        "ThresholdCheckResult must round-trip through JSON"
    );
    assert!(decoded.applied_overrides["error_handling"].active);
    assert!(!decoded.applied_overrides["logging"].active);
    assert!(decoded.applied_overrides["logging"]
        .expired_reason
        .is_some());
}
