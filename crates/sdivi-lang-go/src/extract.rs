//! AST extraction helpers for the Go language adapter.

use sdivi_parsing::feature_record::PatternHint;
use tree_sitter::Node;

/// Node kinds collected as pattern hints for the patterns stage.
const PATTERN_KINDS: &[&str] = &[
    "go_statement",
    "defer_statement",
    "select_statement",
    "type_switch_statement",
    "call_expression",
];

/// Extracts import path strings from `import_declaration` nodes.
pub(crate) fn extract_imports(root: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut imports = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.kind() == "import_declaration" {
            if let Ok(text) = node.utf8_text(source) {
                let text = text.trim().to_string();
                if !text.is_empty() {
                    imports.push(text);
                }
            }
            continue; // don't recurse inside import_declaration
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                stack.push(child);
            }
        }
    }
    imports
}

/// Extracts names of exported (capitalized) top-level functions and types.
///
/// In Go, a name is exported if its first letter is uppercase. This adapter
/// collects top-level `function_declaration`, `method_declaration`, and
/// `type_declaration` nodes whose name starts with an uppercase letter.
pub(crate) fn extract_exports(root: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut exports = Vec::new();
    // Walk only direct children of the source_file node.
    for i in 0..root.child_count() {
        let Some(node) = root.child(i) else { continue };
        match node.kind() {
            "function_declaration" | "method_declaration" => {
                if let Some(name) = go_decl_name(node, source) {
                    if is_exported(&name) {
                        exports.push(name);
                    }
                }
            }
            "type_declaration" => {
                // type_declaration may contain multiple type_spec children.
                for j in 0..node.child_count() {
                    if let Some(spec) = node.child(j) {
                        if spec.kind() == "type_spec" {
                            if let Some(name) = first_identifier(spec, source) {
                                if is_exported(&name) {
                                    exports.push(name);
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    exports
}

/// Extracts function signatures as text up to the opening `{`.
pub(crate) fn extract_signatures(root: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut sigs = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.kind() == "function_declaration" || node.kind() == "method_declaration" {
            if let Some(sig) = go_signature(node, source) {
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

fn go_decl_name(node: Node<'_>, source: &[u8]) -> Option<String> {
    first_identifier(node, source)
}

fn first_identifier(node: Node<'_>, source: &[u8]) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "identifier" || child.kind() == "field_identifier" {
                return child.utf8_text(source).ok().map(|s| s.trim().to_string());
            }
        }
    }
    None
}

/// Returns true if the name's first character is uppercase (Go export rule).
fn is_exported(name: &str) -> bool {
    name.chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
}

fn go_signature(node: Node<'_>, source: &[u8]) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "block" {
                let sig_bytes = source
                    .get(node.start_byte()..child.start_byte())
                    .unwrap_or(&[]);
                return std::str::from_utf8(sig_bytes)
                    .ok()
                    .map(|s| s.trim().to_string());
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
