//! AST extraction helpers for the Java language adapter.

use sdivi_parsing::feature_record::PatternHint;
use tree_sitter::Node;

/// Node kinds collected as pattern hints for the patterns stage.
const PATTERN_KINDS: &[&str] = &[
    "try_statement",
    "try_with_resources_statement",
    "catch_clause",
    "lambda_expression",
    "enhanced_for_statement",
    "throw_statement",
];

/// Top-level declaration kinds whose public visibility makes them exports.
const EXPORTABLE_KINDS: &[&str] = &[
    "class_declaration",
    "interface_declaration",
    "enum_declaration",
    "annotation_type_declaration",
    "record_declaration",
];

/// Extracts `import_declaration` text from the AST.
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
            continue;
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                stack.push(child);
            }
        }
    }
    imports
}

/// Extracts names of `public` top-level declarations.
pub(crate) fn extract_exports(root: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut exports = Vec::new();
    for i in 0..root.child_count() {
        let Some(node) = root.child(i) else { continue };
        if EXPORTABLE_KINDS.contains(&node.kind()) && has_public_modifier(node, source) {
            if let Some(name) = first_identifier(node, source) {
                exports.push(name);
            }
        }
    }
    exports
}

/// Extracts `public` method and constructor signatures.
pub(crate) fn extract_signatures(root: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut sigs = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if (node.kind() == "method_declaration" || node.kind() == "constructor_declaration")
            && has_public_modifier(node, source)
        {
            if let Some(sig) = java_signature(node, source) {
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

/// Returns true if the node has a `modifiers` child containing `public`.
fn has_public_modifier(node: Node<'_>, source: &[u8]) -> bool {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "modifiers" {
                if let Ok(text) = child.utf8_text(source) {
                    return text.split_whitespace().any(|t| t == "public");
                }
            }
        }
    }
    false
}

fn first_identifier(node: Node<'_>, source: &[u8]) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "identifier" {
                return child.utf8_text(source).ok().map(|s| s.trim().to_string());
            }
        }
    }
    None
}

fn java_signature(node: Node<'_>, source: &[u8]) -> Option<String> {
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
