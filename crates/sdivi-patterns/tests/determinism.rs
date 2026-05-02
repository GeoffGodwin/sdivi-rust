//! Determinism property test: same inputs → bit-identical PatternCatalog JSON.
//!
//! Verifies Critical Rule 1: "A Pipeline::snapshot call against the same repo
//! state with the same Config produces a bit-identical Snapshot JSON."

use proptest::prelude::*;
use sdivi_config::PatternsConfig;
use sdivi_parsing::feature_record::{FeatureRecord, PatternHint};
use sdivi_patterns::build_catalog;
use std::path::PathBuf;

/// Known node kinds the Rust adapter can produce.
const RUST_NODE_KINDS: &[&str] = &[
    "try_expression",
    "match_expression",
    "await_expression",
    "closure_expression",
    "macro_invocation",
];

fn arb_node_kind() -> impl Strategy<Value = String> {
    (0usize..RUST_NODE_KINDS.len()).prop_map(|i| RUST_NODE_KINDS[i].to_string())
}

fn arb_hint() -> impl Strategy<Value = PatternHint> {
    (arb_node_kind(), 0usize..500usize, 0usize..100usize).prop_map(
        |(node_kind, start_byte, start_row)| PatternHint {
            node_kind,
            start_byte,
            end_byte: start_byte + 10,
            start_row,
            start_col: 0,
            text: "stub".to_string(),
        },
    )
}

fn arb_record(path_idx: usize) -> impl Strategy<Value = FeatureRecord> {
    proptest::collection::vec(arb_hint(), 0..20usize).prop_map(move |hints| FeatureRecord {
        path: PathBuf::from(format!("src/file{path_idx}.rs")),
        language: "rust".to_string(),
        imports: vec![],
        exports: vec![],
        signatures: vec![],
        pattern_hints: hints,
    })
}

fn arb_records() -> impl Strategy<Value = Vec<FeatureRecord>> {
    let records: Vec<_> = (0..5).map(arb_record).collect();
    records
        .into_iter()
        .fold(Just(vec![]).boxed(), |acc, rec_strategy| {
            (acc, rec_strategy)
                .prop_map(|(mut v, r)| {
                    v.push(r);
                    v
                })
                .boxed()
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Catalog is deterministic: same records + same config → bit-identical JSON.
    #[test]
    fn catalog_is_deterministic(records in arb_records()) {
        let config = PatternsConfig { min_pattern_nodes: 1, ..PatternsConfig::default() };

        let c1 = build_catalog(&records, &config);
        let c2 = build_catalog(&records, &config);

        let j1 = serde_json::to_string(&c1).expect("catalog must serialize");
        let j2 = serde_json::to_string(&c2).expect("catalog must serialize");
        prop_assert_eq!(j1, j2, "same inputs must produce bit-identical catalog JSON");
    }

    /// Serde round-trip preserves catalog identity.
    #[test]
    fn catalog_serde_round_trip(records in arb_records()) {
        let config = PatternsConfig { min_pattern_nodes: 1, ..PatternsConfig::default() };
        let catalog = build_catalog(&records, &config);

        let json = serde_json::to_string(&catalog).expect("must serialize");
        let decoded: sdivi_patterns::PatternCatalog =
            serde_json::from_str(&json).expect("must deserialize");
        let json2 = serde_json::to_string(&decoded).expect("must re-serialize");

        prop_assert_eq!(json, json2, "serde round-trip must be bit-identical");
    }
}
