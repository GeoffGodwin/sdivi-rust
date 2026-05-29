//! Property tests for `classify_hint` fall-through consistency.
//!
//! Acceptance criterion: for any `PatternHintInput` whose `node_kind` is not
//! `call_expression`, `call`, or `macro_invocation`, `classify_hint(hint, lang)`
//! returns the same result as
//! `category_for_node_kind(&hint.node_kind, lang).map(|c| vec![c]).unwrap_or_default()`.
//!
//! This proves the `other =>` arm in `classify_hint`'s dispatch is consistent
//! with the existing node-kind-only classifier across all known and unknown kinds.

use proptest::prelude::*;
use sdivi_patterns::queries::{category_for_node_kind, classify_hint};
use sdivi_patterns::PatternHintInput;

/// All non-special node kinds known to the classifier — exhaustively sampled.
/// "Non-special" means not `call_expression`, `call`, or `macro_invocation`,
/// which are the three kinds that get callee-text dispatch.
const NON_SPECIAL_NODE_KINDS: &[&str] = &[
    "await_expression",
    "class_declaration",
    "class_definition",
    "abstract_class_declaration",
    "interface_declaration",
    "impl_item",
    "try_expression",
    "match_expression",
    "closure_expression",
    "as_expression",
    "type_cast_expression",
];

const TEST_LANGUAGES: &[&str] = &["rust", "python", "typescript", "javascript", "go", "java"];

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// For every known non-special node kind × every supported language × arbitrary
    /// callee text, `classify_hint` must return the same result as
    /// `category_for_node_kind`. The `other =>` arm must be a transparent
    /// pass-through — the text field is irrelevant for these node kinds.
    #[test]
    fn prop_fall_through_matches_category_for_node_kind(
        kind_idx in 0usize..NON_SPECIAL_NODE_KINDS.len(),
        lang_idx in 0usize..TEST_LANGUAGES.len(),
        text in "[a-zA-Z0-9._:! ]{0,40}",
    ) {
        let node_kind = NON_SPECIAL_NODE_KINDS[kind_idx];
        let language = TEST_LANGUAGES[lang_idx];
        let hint = PatternHintInput {
            node_kind: node_kind.to_string(),
            text,
        };
        let got = classify_hint(&hint, language);
        let expected: Vec<&'static str> = category_for_node_kind(node_kind, language)
            .map(|c| vec![c])
            .unwrap_or_default();
        prop_assert_eq!(
            got,
            expected,
            "classify_hint must fall through for node_kind={:?}, language={:?}",
            node_kind,
            language
        );
    }

    /// For arbitrary unknown node kinds (prefixed `unknown_test_` to avoid
    /// colliding with any known kind), `classify_hint` must return the same
    /// result as `category_for_node_kind` — always empty in this case because
    /// the prefix is not in any category's NODE_KINDS list.
    #[test]
    fn prop_unknown_kind_falls_through_to_empty(
        suffix in "[a-z]{5,15}",
        lang_idx in 0usize..TEST_LANGUAGES.len(),
        text in "[a-zA-Z0-9._:! ]{0,30}",
    ) {
        let node_kind = format!("unknown_test_{suffix}");
        let language = TEST_LANGUAGES[lang_idx];
        let hint = PatternHintInput {
            node_kind: node_kind.clone(),
            text,
        };
        let got = classify_hint(&hint, language);
        let expected: Vec<&'static str> = category_for_node_kind(&node_kind, language)
            .map(|c| vec![c])
            .unwrap_or_default();
        prop_assert_eq!(
            got,
            expected,
            "classify_hint for unknown kind {:?} (language={:?}) must match \
             category_for_node_kind — both must be empty for unrecognised node kinds",
            node_kind,
            language
        );
    }

    /// The text field must not influence the result for non-special node kinds.
    /// Two hints with the same non-special node_kind but different text must
    /// produce the same classification — proving the fall-through is text-agnostic.
    #[test]
    fn prop_text_does_not_affect_fall_through(
        kind_idx in 0usize..NON_SPECIAL_NODE_KINDS.len(),
        lang_idx in 0usize..TEST_LANGUAGES.len(),
        text_a in "[a-zA-Z0-9._:! ]{0,30}",
        text_b in "[a-zA-Z0-9._:! ]{0,30}",
    ) {
        let node_kind = NON_SPECIAL_NODE_KINDS[kind_idx];
        let language = TEST_LANGUAGES[lang_idx];
        let hint_a = PatternHintInput { node_kind: node_kind.to_string(), text: text_a };
        let hint_b = PatternHintInput { node_kind: node_kind.to_string(), text: text_b };
        let result_a = classify_hint(&hint_a, language);
        let result_b = classify_hint(&hint_b, language);
        prop_assert_eq!(
            result_a,
            result_b,
            "classify_hint for non-special node_kind={:?} must be text-agnostic",
            node_kind
        );
    }
}
