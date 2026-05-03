//! Parsing and validation of WASM edge-weight keys for [`crate::exports::detect_boundaries`].
//!
//! WASM callers use `"source:target"` colon-separated keys. The native
//! `LeidenConfigInput::edge_weights` uses NUL-separated keys produced by
//! [`sdivi_core::input::edge_weight_key`]. This module converts between the two.

use std::collections::BTreeMap;

/// Parses `"source:target"` edge-weight keys and converts them to the native NUL-separated format.
///
/// Returns `Err(message)` on the first invalid entry. The caller converts the message to
/// `JsError`. Returning `String` here (not `JsError`) allows native unit tests to run.
///
/// Validation rules:
/// - Each key must contain at least one colon; everything after the **first** colon is the
///   target, so node IDs that themselves contain colons are supported.
/// - Source (before first colon) and target (after first colon) must both be non-empty.
/// - Weights must be finite and `>= 0.0`. `0.0` is accepted. `NaN` and negatives are rejected.
/// - Edges absent from the graph are silently ignored by the detection layer.
pub fn parse_wasm_edge_weights(
    weights: BTreeMap<String, f64>,
) -> Result<BTreeMap<String, f64>, String> {
    let mut result = BTreeMap::new();
    for (key, weight) in weights {
        if weight.is_nan() || weight.is_infinite() {
            return Err(format!(
                "edge weight for key \"{key}\" is not finite ({weight}); all weights must be finite and >= 0.0"
            ));
        }
        if weight < 0.0 {
            return Err(format!(
                "edge weight for key \"{key}\" is negative ({weight}); weights must be >= 0.0"
            ));
        }
        let (src, tgt) = key.split_once(':').ok_or_else(|| {
            format!(
                "edge_weights key \"{key}\" is not in \"source:target\" format (no colon found)"
            )
        })?;
        if src.is_empty() {
            return Err(format!(
                "edge_weights key \"{key}\" has empty source before the colon"
            ));
        }
        if tgt.is_empty() {
            return Err(format!(
                "edge_weights key \"{key}\" has empty target after the colon"
            ));
        }
        result.insert(sdivi_core::input::edge_weight_key(src, tgt), weight);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_weights() {
        let mut m = BTreeMap::new();
        m.insert("a:b".to_string(), 1.0);
        m.insert("x:y".to_string(), 0.0);
        let result = parse_wasm_edge_weights(m).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn rejects_no_colon() {
        let mut m = BTreeMap::new();
        m.insert("a-b".to_string(), 1.0);
        let e = parse_wasm_edge_weights(m).unwrap_err();
        assert!(e.contains("no colon found"), "got: {e}");
    }

    #[test]
    fn rejects_empty_source() {
        let mut m = BTreeMap::new();
        m.insert(":target".to_string(), 1.0);
        let e = parse_wasm_edge_weights(m).unwrap_err();
        assert!(e.contains("empty source"), "got: {e}");
    }

    #[test]
    fn rejects_empty_target() {
        let mut m = BTreeMap::new();
        m.insert("source:".to_string(), 1.0);
        let e = parse_wasm_edge_weights(m).unwrap_err();
        assert!(e.contains("empty target"), "got: {e}");
    }

    #[test]
    fn rejects_nan_weight() {
        let mut m = BTreeMap::new();
        m.insert("a:b".to_string(), f64::NAN);
        let e = parse_wasm_edge_weights(m).unwrap_err();
        assert!(e.contains("NaN"), "got: {e}");
    }

    #[test]
    fn rejects_negative_weight() {
        let mut m = BTreeMap::new();
        m.insert("a:b".to_string(), -0.5);
        let e = parse_wasm_edge_weights(m).unwrap_err();
        assert!(e.contains("negative") || e.contains(">= 0.0"), "got: {e}");
    }

    #[test]
    fn handles_colon_in_node_id() {
        // Node IDs like "crates/foo:bar.rs" — first colon splits source, rest is target.
        let mut m = BTreeMap::new();
        m.insert("crates/foo:bar.rs:baz.rs".to_string(), 2.0);
        let result = parse_wasm_edge_weights(m).unwrap();
        // source = "crates/foo", target = "bar.rs:baz.rs"
        assert_eq!(result.len(), 1);
    }

    /// The converted key must equal what `edge_weight_key` produces — i.e. NUL-separated.
    /// Verifies the key CONTENT, not just the count of entries.
    #[test]
    fn converted_key_uses_nul_separator() {
        let mut m = BTreeMap::new();
        m.insert("a:b".to_string(), 1.0);
        let result = parse_wasm_edge_weights(m).unwrap();
        let expected_key = sdivi_core::input::edge_weight_key("a", "b");
        assert!(
            result.contains_key(&expected_key),
            "expected NUL-separated key {:?}, got keys: {:?}",
            expected_key,
            result.keys().collect::<Vec<_>>()
        );
    }

    /// Colon-in-node-id: verify the NUL key uses the part AFTER the first colon as the full target.
    #[test]
    fn colon_in_node_id_produces_correct_nul_key() {
        let mut m = BTreeMap::new();
        // source = "crates/foo", target = "bar.rs:baz.rs"
        m.insert("crates/foo:bar.rs:baz.rs".to_string(), 2.0);
        let result = parse_wasm_edge_weights(m).unwrap();
        let expected_key = sdivi_core::input::edge_weight_key("crates/foo", "bar.rs:baz.rs");
        assert!(
            result.contains_key(&expected_key),
            "expected NUL-separated key {:?}, got: {:?}",
            expected_key,
            result.keys().collect::<Vec<_>>()
        );
    }

    /// An empty input map must return an empty output map — no error.
    #[test]
    fn accepts_empty_map_returns_empty_map() {
        let result = parse_wasm_edge_weights(BTreeMap::new()).unwrap();
        assert!(result.is_empty(), "empty input must produce empty output");
    }

    /// The weight VALUE must be preserved unchanged after key conversion.
    #[test]
    fn weight_value_preserved_after_key_conversion() {
        let mut m = BTreeMap::new();
        m.insert("x:y".to_string(), 3.14);
        let result = parse_wasm_edge_weights(m).unwrap();
        let key = sdivi_core::input::edge_weight_key("x", "y");
        assert_eq!(
            result[&key], 3.14,
            "weight value must survive the colon→NUL key conversion unchanged"
        );
    }

    /// `f64::INFINITY` must be rejected — the doc comment says weights must be "finite".
    #[test]
    fn rejects_positive_infinity_weight() {
        let mut m = BTreeMap::new();
        m.insert("a:b".to_string(), f64::INFINITY);
        let e = parse_wasm_edge_weights(m).unwrap_err();
        assert!(
            e.contains("finite") || e.contains("infinite") || e.contains("NaN"),
            "error message must explain why infinity is rejected: {e}"
        );
    }

    /// `f64::NEG_INFINITY` must be rejected — the `is_infinite()` check must catch both signs.
    #[test]
    fn rejects_negative_infinity_weight() {
        let mut m = BTreeMap::new();
        m.insert("a:b".to_string(), f64::NEG_INFINITY);
        let e = parse_wasm_edge_weights(m).unwrap_err();
        assert!(
            e.contains("finite") || e.contains("infinite") || e.contains("inf"),
            "error message must explain why negative infinity is rejected: {e}"
        );
    }
}
