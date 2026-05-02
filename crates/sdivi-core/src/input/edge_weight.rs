//! Helper for constructing [`LeidenConfigInput::edge_weights`] keys.

/// Separator used in [`edge_weight_key`] to encode an edge pair as a single string.
///
/// NUL (`\x00`) never appears in canonical node IDs (which are repo-relative
/// file paths) so it is safe as a delimiter.
const SEP: char = '\x00';

/// Encodes a `(source, target)` node-ID pair as an edge-weight map key.
///
/// The key format required by [`super::LeidenConfigInput::edge_weights`] is
/// `"source\x00target"` (NUL-separated). Callers should canonicalize
/// (`source < target` by lexicographic order); mis-ordered pairs are
/// normalized by the detection layer, so no weight is silently discarded.
///
/// # Examples
///
/// ```rust
/// use sdivi_core::input::edge_weight_key;
///
/// let key = edge_weight_key("src/a.rs", "src/b.rs");
/// assert_eq!(key, "src/a.rs\x00src/b.rs");
/// ```
pub fn edge_weight_key(source: &str, target: &str) -> String {
    format!("{source}{SEP}{target}")
}

/// Splits an edge-weight map key produced by [`edge_weight_key`] into `(source, target)`.
///
/// Returns `None` if the key does not contain the separator.
///
/// # Examples
///
/// ```rust
/// use sdivi_core::input::{edge_weight_key, split_edge_weight_key};
///
/// let key = edge_weight_key("src/a.rs", "src/b.rs");
/// let (s, t) = split_edge_weight_key(&key).unwrap();
/// assert_eq!(s, "src/a.rs");
/// assert_eq!(t, "src/b.rs");
/// ```
pub fn split_edge_weight_key(key: &str) -> Option<(&str, &str)> {
    key.split_once(SEP)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let k = edge_weight_key("foo/a.rs", "foo/b.rs");
        let (s, t) = split_edge_weight_key(&k).unwrap();
        assert_eq!(s, "foo/a.rs");
        assert_eq!(t, "foo/b.rs");
    }

    #[test]
    fn split_missing_sep_returns_none() {
        assert!(split_edge_weight_key("no-separator-here").is_none());
    }

    #[test]
    fn edge_weight_key_with_repo_paths() {
        let k = edge_weight_key("src/components/ui.rs", "src/utils/helpers.rs");
        assert_eq!(k, "src/components/ui.rs\x00src/utils/helpers.rs");

        let (s, t) = split_edge_weight_key(&k).unwrap();
        assert_eq!(s, "src/components/ui.rs");
        assert_eq!(t, "src/utils/helpers.rs");
    }

    #[test]
    fn edge_weight_key_with_special_path_characters() {
        let k = edge_weight_key("src/file-name_1.rs", "tests/util.rs");
        assert_eq!(k, "src/file-name_1.rs\x00tests/util.rs");

        let (s, t) = split_edge_weight_key(&k).unwrap();
        assert_eq!(s, "src/file-name_1.rs");
        assert_eq!(t, "tests/util.rs");
    }

    #[test]
    fn edge_weight_key_with_nested_paths() {
        let k = edge_weight_key("a/b/c/d.rs", "x/y/z/w.rs");
        assert_eq!(k, "a/b/c/d.rs\x00x/y/z/w.rs");

        let (s, t) = split_edge_weight_key(&k).unwrap();
        assert_eq!(s, "a/b/c/d.rs");
        assert_eq!(t, "x/y/z/w.rs");
    }

    #[test]
    fn edge_weight_key_multiple_separators_in_path_splits_on_first() {
        // If a path somehow contained NUL (which shouldn't happen in practice),
        // split should split on the first occurrence.
        let malformed = format!("a{}b{}c", SEP, SEP);
        let (s, t) = split_edge_weight_key(&malformed).unwrap();
        assert_eq!(s, "a");
        assert_eq!(t, format!("b{}c", SEP));
    }
}
