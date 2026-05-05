//! AST extraction helpers for the Java language adapter.

use sdivi_parsing::feature_record::PatternHint;
use sdivi_parsing::text::truncate_to_256_bytes;
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

/// Extracts the module specifier from each `import_declaration` in the AST.
///
/// - `import java.util.List;` → `["java.util.List"]`
/// - `import java.util.*;` → `["java.util.*"]` (wildcard preserved)
/// - `import static org.junit.Assert.assertEquals;` → `["org.junit.Assert"]`
///   (trailing member name stripped — the class is the module)
/// - `import static org.junit.Assert.*;` → `["org.junit.Assert"]`
///   (wildcard static import; `*` stripped, scoped_identifier is the class)
pub(crate) fn extract_imports(root: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut imports = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.kind() == "import_declaration" {
            if let Some(spec) = java_import_specifier(node, source) {
                imports.push(spec);
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

/// Derives the module specifier from a single `import_declaration` node.
///
/// Wildcard detection uses the node's source text (`".*"` substring) rather
/// than child node kind matching, because tree-sitter-java 0.21.x represents
/// the anonymous `*` terminal with a kind that varies across grammar versions.
fn java_import_specifier(node: Node<'_>, source: &[u8]) -> Option<String> {
    let mut is_static = false;
    let mut qualified: Option<String> = None;

    for i in 0..node.child_count() {
        let Some(child) = node.child(i) else { continue };
        match child.kind() {
            "static" => {
                is_static = true;
            }
            "scoped_identifier" | "identifier" => {
                qualified = child
                    .utf8_text(source)
                    .ok()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());
            }
            _ => {}
        }
    }

    let qualified = qualified?;
    // ".*" in the declaration source text always means wildcard in Java imports.
    let has_wildcard = node.utf8_text(source).unwrap_or("").contains(".*");

    if is_static {
        if has_wildcard {
            // `import static a.b.C.*` → "a.b.C" (qualified stops at the class)
            Some(qualified)
        } else {
            // `import static a.b.C.method` → "a.b.C" (strip trailing member)
            let pos = qualified.rfind('.')?;
            Some(qualified[..pos].to_string())
        }
    } else if has_wildcard {
        // `import a.b.*` → "a.b.*"
        Some(format!("{qualified}.*"))
    } else {
        // `import a.b.C` → "a.b.C"
        Some(qualified)
    }
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
