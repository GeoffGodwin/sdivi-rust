//! AST extraction helpers for the Python language adapter.

use sdivi_parsing::feature_record::PatternHint;
use tree_sitter::Node;

/// Node kinds collected as pattern hints for the patterns stage.
const PATTERN_KINDS: &[&str] = &[
    "try_statement",
    "except_clause",
    "with_statement",
    "await",
    "lambda",
    "generator_expression",
    "list_comprehension",
    "dictionary_comprehension",
    "set_comprehension",
    "decorated_definition",
];

/// Top-level definition kinds that can be exported from a Python module.
const TOP_LEVEL_KINDS: &[&str] = &[
    "function_definition",
    "class_definition",
    "decorated_definition",
];

/// Extracts import paths from `import` and `from … import` statements.
pub(crate) fn extract_imports(root: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut imports = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        match node.kind() {
            "import_statement" | "import_from_statement" | "future_import_statement" => {
                if let Ok(text) = node.utf8_text(source) {
                    let text = text.trim().to_string();
                    if !text.is_empty() {
                        imports.push(text);
                    }
                }
                continue; // don't recurse into import children
            }
            _ => {}
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                stack.push(child);
            }
        }
    }
    imports
}

/// Extracts names of top-level public (non-underscore-prefixed) definitions.
pub(crate) fn extract_exports(root: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut exports = Vec::new();
    // Only walk direct children of the module root — top-level only.
    for i in 0..root.child_count() {
        let Some(node) = root.child(i) else { continue };
        if TOP_LEVEL_KINDS.contains(&node.kind()) {
            if let Some(name) = definition_name(node, source) {
                if !name.starts_with('_') {
                    exports.push(name);
                }
            }
        }
    }
    exports
}

/// Extracts function and class signatures as source text up to the first `:`.
pub(crate) fn extract_signatures(root: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut sigs = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.kind() == "function_definition" || node.kind() == "class_definition" {
            if let Some(sig) = python_signature(node, source) {
                sigs.push(sig);
            }
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                stack.push(child);
            }
        }
    }
    sigs
}

/// Collects pattern-relevant AST nodes as [`PatternHint`]s.
pub(crate) fn collect_hints(root: Node<'_>, source: &[u8]) -> Vec<PatternHint> {
    let mut hints = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if PATTERN_KINDS.contains(&node.kind()) {
            let raw = node.utf8_text(source).unwrap_or("").to_string();
            let text = truncate_to_256_bytes(raw);
            hints.push(PatternHint {
                node_kind: node.kind().to_string(),
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
                start_row: node.start_position().row,
                start_col: node.start_position().column,
                text,
            });
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                stack.push(child);
            }
        }
    }
    hints
}

/// Returns the definition name for a `function_definition`, `class_definition`,
/// or `decorated_definition` node.
fn definition_name(node: Node<'_>, source: &[u8]) -> Option<String> {
    if node.kind() == "decorated_definition" {
        // The inner definition is the last child of the decorated_definition.
        for i in (0..node.child_count()).rev() {
            if let Some(child) = node.child(i) {
                if child.kind() == "function_definition" || child.kind() == "class_definition" {
                    return definition_name(child, source);
                }
            }
        }
        return None;
    }
    // function_definition / class_definition: identifier is a direct child.
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "identifier" {
                return child.utf8_text(source).ok().map(|s| s.trim().to_string());
            }
        }
    }
    None
}

fn python_signature(node: Node<'_>, source: &[u8]) -> Option<String> {
    // Find the `block` child; signature is everything before it.
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "block" {
                let sig_bytes = source
                    .get(node.start_byte()..child.start_byte())
                    .unwrap_or(&[]);
                return std::str::from_utf8(sig_bytes)
                    .ok()
                    .map(|s| s.trim_end_matches(':').trim().to_string());
            }
        }
    }
    node.utf8_text(source).ok().map(|s| s.trim().to_string())
}

/// Truncates a string to at most 256 UTF-8 bytes without splitting a char.
pub(crate) fn truncate_to_256_bytes(raw: String) -> String {
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
        // 128 two-byte chars = 256 bytes exactly → no truncation needed
        let s: String = "é".repeat(128);
        assert_eq!(s.len(), 256);
        let result = truncate_to_256_bytes(s.clone());
        assert_eq!(result, s);

        // 129 two-byte chars = 258 bytes → truncate to 128 chars = 256 bytes
        let s: String = "é".repeat(129);
        let result = truncate_to_256_bytes(s);
        assert_eq!(result.len(), 256);
        assert!(result.is_char_boundary(result.len()));
    }
}
