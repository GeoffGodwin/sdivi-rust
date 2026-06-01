//! Callee-text classification for serialization boundary calls.
//!
//! Detects (de)serialization calls with anchored receiver names:
//!
//! - **TypeScript / JavaScript:** `JSON.parse(…)`, `JSON.stringify(…)`,
//!   `structuredClone(…)`.
//! - **Python:** `json.loads(…)`, `json.dumps(…)`, `json.load(…)`, `json.dump(…)`,
//!   `pickle.loads(…)`, `pickle.dumps(…)`.
//! - **Go:** `json.Marshal(…)`, `json.Unmarshal(…)`, `json.MarshalIndent(…)`,
//!   `json.NewEncoder(…)`, `json.NewDecoder(…)`.
//!
//! ## CALL_DISPATCH slot
//!
//! Registered at P3 — below `testing` (P2) and above `schema_validation` (P4).
//! Receiver-anchored and specific: resolves before broader categories.
//!
//! ## Design: receiver-anchored only
//!
//! Bare `.parse(` is intentionally not matched — it collides with schema validators
//! (`schema.parse`) and config parsers. Only calls whose receiver is `JSON`, `json`,
//! or `pickle` are classified here.
//!
//! ## Seeds forward
//!
//! Protobuf/Avro/MessagePack codecs and ORM serialize hooks are adjacent; defer
//! until requested.

use std::sync::LazyLock;

use regex::Regex;

/// Tree-sitter node kinds for serialization patterns.
///
/// Empty — detection is entirely via callee-text inspection in [`matches_callee`].
/// Classification happens in `classify_hint`'s `CALL_DISPATCH` loop at slot P3.
pub const NODE_KINDS: &[&str] = &[];

// TypeScript / JavaScript — JSON built-in and structuredClone.
// Anchored at `^` on the `JSON.` receiver; structuredClone is a bare global.
static TS_JS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(JSON\.(parse|stringify)\(|structuredClone\()")
        .expect("serialization TS/JS regex is valid")
});

// Python — json and pickle stdlib modules.
static PYTHON_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(json|pickle)\.(loads|dumps|load|dump)\(")
        .expect("serialization Python regex is valid")
});

// Go — encoding/json standard library.
static GO_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^json\.(Marshal|Unmarshal|MarshalIndent|NewEncoder|NewDecoder)\(")
        .expect("serialization Go regex is valid")
});

/// Return `true` when `text` looks like a (de)serialization boundary call.
///
/// Covers `JSON.parse`/`JSON.stringify`/`structuredClone` (TS/JS), `json.*`/`pickle.*`
/// (Python), and `json.*` encoding calls (Go). See module doc for the receiver-anchored
/// design rationale.
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::queries::serialization::matches_callee;
///
/// assert!(matches_callee("JSON.parse(s)", "typescript"));
/// assert!(matches_callee("JSON.stringify(obj)", "javascript"));
/// assert!(matches_callee("structuredClone(data)", "typescript"));
/// assert!(matches_callee("json.loads(s)", "python"));
/// assert!(matches_callee("json.dumps(o)", "python"));
/// assert!(matches_callee("pickle.dumps(o)", "python"));
/// assert!(matches_callee("json.Marshal(v)", "go"));
/// assert!(matches_callee("json.Unmarshal(b, &v)", "go"));
/// assert!(matches_callee("json.NewDecoder(r)", "go"));
/// assert!(!matches_callee("schema.parse(x)", "typescript"));
/// assert!(!matches_callee("JSON.error(x)", "typescript"));
/// assert!(!matches_callee("len(x)", "python"));
/// ```
pub fn matches_callee(text: &str, language: &str) -> bool {
    match language {
        "typescript" | "javascript" => TS_JS_RE.is_match(text),
        "python" => PYTHON_RE.is_match(text),
        "go" => GO_RE.is_match(text),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_parse_matches_ts() {
        assert!(matches_callee("JSON.parse(s)", "typescript"));
    }

    #[test]
    fn json_stringify_matches_js() {
        assert!(matches_callee("JSON.stringify(obj)", "javascript"));
    }

    #[test]
    fn structured_clone_matches_ts() {
        assert!(matches_callee("structuredClone(data)", "typescript"));
    }

    #[test]
    fn json_parse_matches_js() {
        assert!(matches_callee("JSON.parse(text)", "javascript"));
    }

    #[test]
    fn json_loads_matches_python() {
        assert!(matches_callee("json.loads(s)", "python"));
    }

    #[test]
    fn json_dumps_matches_python() {
        assert!(matches_callee("json.dumps(o)", "python"));
    }

    #[test]
    fn json_load_matches_python() {
        assert!(matches_callee("json.load(f)", "python"));
    }

    #[test]
    fn json_dump_matches_python() {
        assert!(matches_callee("json.dump(o, f)", "python"));
    }

    #[test]
    fn pickle_loads_matches_python() {
        assert!(matches_callee("pickle.loads(b)", "python"));
    }

    #[test]
    fn pickle_dumps_matches_python() {
        assert!(matches_callee("pickle.dumps(o)", "python"));
    }

    #[test]
    fn json_marshal_matches_go() {
        assert!(matches_callee("json.Marshal(v)", "go"));
    }

    #[test]
    fn json_unmarshal_matches_go() {
        assert!(matches_callee("json.Unmarshal(b, &v)", "go"));
    }

    #[test]
    fn json_marshal_indent_matches_go() {
        assert!(matches_callee("json.MarshalIndent(v, \"\", \"  \")", "go"));
    }

    #[test]
    fn json_new_encoder_matches_go() {
        assert!(matches_callee("json.NewEncoder(w)", "go"));
    }

    #[test]
    fn json_new_decoder_matches_go() {
        assert!(matches_callee("json.NewDecoder(r)", "go"));
    }

    #[test]
    fn schema_parse_does_not_match_ts() {
        assert!(!matches_callee("schema.parse(x)", "typescript"));
    }

    #[test]
    fn json_other_method_does_not_match_ts() {
        assert!(!matches_callee("JSON.error(x)", "typescript"));
    }

    #[test]
    fn len_does_not_match_python() {
        assert!(!matches_callee("len(x)", "python"));
    }

    #[test]
    fn requests_does_not_match_python() {
        assert!(!matches_callee("requests.get(url)", "python"));
    }

    #[test]
    fn json_other_does_not_match_go() {
        assert!(!matches_callee("json.Something(v)", "go"));
    }

    #[test]
    fn fmt_println_does_not_match_go() {
        assert!(!matches_callee("fmt.Println(x)", "go"));
    }

    #[test]
    fn rust_returns_false() {
        assert!(!matches_callee("serde_json::to_string(&v)", "rust"));
    }

    #[test]
    fn node_kinds_is_empty() {
        // NODE_KINDS is intentionally empty: this category is callee-only (classified
        // via classify_hint). The assertion guards that contract against regressions.
        #[allow(clippy::const_is_empty)]
        let empty = NODE_KINDS.is_empty();
        assert!(empty);
    }
}
