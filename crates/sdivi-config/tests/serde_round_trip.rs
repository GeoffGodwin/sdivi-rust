use sdivi_config::Config;

/// Verifies that `Config::default()` survives a serde_json round-trip unchanged.
/// The re-serialized JSON must be bit-identical to the first serialization,
/// satisfying the determinism contract (Critical Rule 1).
#[test]
fn config_default_serde_round_trip_is_identity() {
    let config = Config::default();
    let json = serde_json::to_string(&config).expect("Config::default() must serialize");
    let decoded: Config = serde_json::from_str(&json).expect("serialized Config must deserialize");
    let json2 = serde_json::to_string(&decoded).expect("deserialized Config must re-serialize");
    assert_eq!(
        json, json2,
        "re-serialized JSON must be bit-identical to the first serialization"
    );
}

/// Verifies that a Config round-tripped through serde_json preserves every
/// field in the `core` sub-struct.
#[test]
fn core_config_fields_survive_round_trip() {
    let original = Config::default();
    let json = serde_json::to_string(&original).unwrap();
    let decoded: Config = serde_json::from_str(&json).unwrap();

    assert_eq!(original.core.languages, decoded.core.languages);
    assert_eq!(original.core.random_seed, decoded.core.random_seed);
    assert_eq!(original.core.exclude, decoded.core.exclude);
}

/// Verifies nested sub-structs also survive the round-trip intact.
#[test]
fn nested_config_fields_survive_round_trip() {
    let original = Config::default();
    let json = serde_json::to_string(&original).unwrap();
    let decoded: Config = serde_json::from_str(&json).unwrap();

    assert_eq!(original.snapshots.dir, decoded.snapshots.dir);
    assert_eq!(original.snapshots.retention, decoded.snapshots.retention);
    assert!(
        (original.boundaries.leiden_gamma - decoded.boundaries.leiden_gamma).abs() < f64::EPSILON
    );
    assert_eq!(
        original.boundaries.stability_threshold,
        decoded.boundaries.stability_threshold
    );
    assert_eq!(
        original.boundaries.weighted_edges,
        decoded.boundaries.weighted_edges
    );
    assert!(
        (original.thresholds.pattern_entropy_rate - decoded.thresholds.pattern_entropy_rate).abs()
            < f64::EPSILON
    );
    assert!(
        (original.change_coupling.min_frequency - decoded.change_coupling.min_frequency).abs()
            < f64::EPSILON
    );
    assert_eq!(
        original.change_coupling.history_depth,
        decoded.change_coupling.history_depth
    );
    assert!(original.thresholds.overrides.is_empty());
    assert!(decoded.thresholds.overrides.is_empty());
    assert_eq!(
        original.determinism.enforce_btree_order,
        decoded.determinism.enforce_btree_order
    );
}

/// Property: for a range of u64 seed values, a Config with that seed
/// serializes and deserializes back to the same seed.
/// Covers the RNG seed determinism contract across the numeric domain.
#[test]
fn random_seed_survives_serde_round_trip_for_representative_values() {
    let seeds: &[u64] = &[
        0,
        1,
        42,
        u8::MAX as u64,
        u16::MAX as u64,
        u32::MAX as u64,
        u64::MAX / 2,
        u64::MAX - 1,
        u64::MAX,
        0x_dead_beef_cafe_f00d,
    ];
    for &seed in seeds {
        let mut config = Config::default();
        config.core.random_seed = seed;
        let json = serde_json::to_string(&config).unwrap();
        let decoded: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(
            seed, decoded.core.random_seed,
            "seed {seed} changed after serde round-trip"
        );
    }
}

/// Property: for a range of retention values (including 0 = unlimited),
/// serialization round-trip preserves the value.
#[test]
fn snapshot_retention_survives_serde_round_trip_for_representative_values() {
    let values: &[u32] = &[0, 1, 50, 100, 500, u32::MAX];
    for &retention in values {
        let mut config = Config::default();
        config.snapshots.retention = retention;
        let json = serde_json::to_string(&config).unwrap();
        let decoded: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(
            retention, decoded.snapshots.retention,
            "retention {retention} changed after serde round-trip"
        );
    }
}

/// Verifies Config::default() serializes to valid JSON (parseable by serde_json).
#[test]
fn config_default_serializes_to_valid_json() {
    let config = Config::default();
    let json = serde_json::to_string(&config).expect("Config::default() must serialize");
    let value: serde_json::Value =
        serde_json::from_str(&json).expect("serialized Config must be valid JSON");
    assert!(value.is_object(), "top-level Config JSON must be an object");
}

/// Verifies multiple sequential calls to Config::default() produce identical JSON.
/// This is the determinism check: no hidden mutable state, no clock-seeded RNG.
#[test]
fn config_default_is_deterministic_across_calls() {
    let first = serde_json::to_string(&Config::default()).unwrap();
    let second = serde_json::to_string(&Config::default()).unwrap();
    let third = serde_json::to_string(&Config::default()).unwrap();
    assert_eq!(
        first, second,
        "second Config::default() JSON differs from first"
    );
    assert_eq!(
        first, third,
        "third Config::default() JSON differs from first"
    );
}
