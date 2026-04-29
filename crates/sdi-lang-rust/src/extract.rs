//! AST extraction helpers for the Rust language adapter.

use sdi_parsing::feature_record::PatternHint;
use tree_sitter::Node;

/// Node kinds collected as pattern hints for the patterns stage.
const PATTERN_KINDS: &[&str] = &[
    "try_expression",
    "match_expression",
    "macro_invocation",
    "await_expression",
    "closure_expression",
];

/// Node kinds that can be exported from a Rust file.
const EXPORTABLE_KINDS: &[&str] = &[
    "function_item",
    "struct_item",
    "enum_item",
    "trait_item",
    "type_item",
    "const_item",
    "static_item",
    "mod_item",
];

/// Extracts `use` import paths from the root node.
pub(crate) fn extract_imports(root: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut imports = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.kind() == "use_declaration" {
            if let Ok(text) = node.utf8_text(source) {
                // use_declaration text may be `use …;`, `pub use …;`, or
                // `pub(crate) use …;`. Find the `use ` keyword and take
                // everything after it so visibility modifiers are stripped.
                if let Some(pos) = text.find("use ") {
                    let import = text[pos + 4..]
                        .trim_end_matches(';')
                        .trim()
                        .to_string();
                    if !import.is_empty() {
                        imports.push(import);
                    }
                }
            }
            continue; // don't recurse into use_declaration children
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                stack.push(child);
            }
        }
    }
    imports
}

/// Extracts names of publicly exported items.
pub(crate) fn extract_exports(root: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut exports = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if EXPORTABLE_KINDS.contains(&node.kind()) && is_public(node, source) {
            if let Some(name) = first_identifier(node, source) {
                exports.push(name);
            }
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                stack.push(child);
            }
        }
    }
    exports
}

/// Extracts function signatures (text up to the opening `{` of the body).
pub(crate) fn extract_signatures(root: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut sigs = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.kind() == "function_item" {
            if let Some(sig) = function_signature(node, source) {
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
            let text = if raw.len() > 256 {
                let end = raw
                    .char_indices()
                    .take_while(|(i, _)| *i < 256)
                    .last()
                    .map(|(i, c)| i + c.len_utf8())
                    .unwrap_or(0);
                raw[..end].to_string()
            } else {
                raw
            };
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

fn is_public(node: Node<'_>, source: &[u8]) -> bool {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "visibility_modifier" {
                return child
                    .utf8_text(source)
                    .map(|t| t.trim().starts_with("pub"))
                    .unwrap_or(false);
            }
        }
    }
    false
}

fn first_identifier(node: Node<'_>, source: &[u8]) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "identifier" || child.kind() == "type_identifier" {
                return child
                    .utf8_text(source)
                    .ok()
                    .map(|s| s.trim().to_string());
            }
        }
    }
    None
}

fn function_signature(node: Node<'_>, source: &[u8]) -> Option<String> {
    // Find the block child; signature is everything before it.
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
    // Fallback: no block found (e.g. extern fn declaration).
    node.utf8_text(source).ok().map(|s| s.trim().to_string())
}
