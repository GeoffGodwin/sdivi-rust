//! Text utilities shared across language adapter crates.

use tree_sitter::Node;

/// Truncates a string to at most 256 UTF-8 bytes without splitting a char boundary.
pub fn truncate_to_256_bytes(raw: String) -> String {
    if raw.len() <= 256 {
        return raw;
    }
    let end = raw
        .char_indices()
        .take_while(|(i, c)| *i + c.len_utf8() <= 256)
        .last()
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);
    raw[..end].to_string()
}

/// Extracts the unquoted content of a JS/TS `string` AST node.
///
/// Tries the `string_fragment` child first; falls back to stripping the
/// surrounding quote characters from the raw node text.
pub fn js_string_content(string_node: Node<'_>, source: &[u8]) -> Option<String> {
    for i in 0..string_node.child_count() {
        let Some(child) = string_node.child(i) else {
            continue;
        };
        if child.kind() == "string_fragment" {
            return child
                .utf8_text(source)
                .ok()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty());
        }
    }
    // Fallback: strip surrounding quote character.
    let text = string_node.utf8_text(source).ok()?;
    let t = text.trim();
    if t.len() >= 2 {
        let q = &t[..1];
        if (q == "\"" || q == "'") && t.ends_with(q) {
            let inner = &t[1..t.len() - 1];
            if !inner.is_empty() {
                return Some(inner.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_ascii_over_256() {
        let s = "a".repeat(300);
        let result = truncate_to_256_bytes(s);
        assert_eq!(result.len(), 256);
    }

    #[test]
    fn truncate_respects_char_boundaries() {
        let s: String = "é".repeat(128); // 256 bytes exactly — no truncation
        assert_eq!(s.len(), 256);
        let result = truncate_to_256_bytes(s.clone());
        assert_eq!(result, s);
        let s: String = "é".repeat(129); // 258 bytes — must truncate cleanly
        let result = truncate_to_256_bytes(s);
        assert_eq!(result.len(), 256);
        assert!(result.is_char_boundary(result.len()));
    }
}
