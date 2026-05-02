//! AST extraction helpers for the TypeScript language adapter.

use sdivi_parsing::feature_record::PatternHint;
use tree_sitter::Node;

/// Node kinds collected as pattern hints for the patterns stage.
const PATTERN_KINDS: &[&str] = &[
    "try_statement",
    "await_expression",
    "arrow_function",
    "call_expression",
    "generator_function",
    "generator_function_declaration",
];

/// Declaration kinds that may appear as the `declaration` child of an
/// `export_statement`.
const DECLARATION_KINDS: &[&str] = &[
    "function_declaration",
    "class_declaration",
    "interface_declaration",
    "enum_declaration",
    "type_alias_declaration",
    "abstract_class_declaration",
    "generator_function_declaration",
    "lexical_declaration",
    "variable_declaration",
];

/// Extracts `import` statement text from the AST.
pub(crate) fn extract_imports(root: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut imports = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.kind() == "import_statement" {
            if let Ok(text) = node.utf8_text(source) {
                let text = text.trim().to_string();
                if !text.is_empty() {
                    imports.push(text);
                }
            }
            continue; // don't recurse into import children
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                stack.push(child);
            }
        }
    }
    imports
}

/// Extracts names of top-level exported items.
pub(crate) fn extract_exports(root: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut exports = Vec::new();
    for i in 0..root.child_count() {
        let Some(node) = root.child(i) else { continue };
        if node.kind() == "export_statement" {
            exports.extend(export_names(node, source));
            // Don't recurse — top-level only.
        }
    }
    exports
}

/// Extracts function signatures as text up to the opening `{`.
pub(crate) fn extract_signatures(root: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut sigs = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.kind() == "function_declaration"
            || node.kind() == "method_definition"
            || node.kind() == "abstract_method_signature"
        {
            if let Some(sig) = ts_signature(node, source) {
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

/// Returns the exported name(s) from an `export_statement` node.
fn export_names(node: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut names = Vec::new();
    for i in 0..node.child_count() {
        let Some(child) = node.child(i) else { continue };
        if DECLARATION_KINDS.contains(&child.kind()) {
            if let Some(name) = decl_identifier(child, source) {
                names.push(name);
            }
        } else if child.kind() == "export_clause" {
            // `export { a, b as c }` — collect specifier local names.
            for j in 0..child.child_count() {
                if let Some(spec) = child.child(j) {
                    if spec.kind() == "export_specifier" {
                        if let Some(name) = first_identifier(spec, source) {
                            names.push(name);
                        }
                    }
                }
            }
        }
    }
    names
}

/// Returns the first `identifier` or `type_identifier` child of a declaration node.
fn decl_identifier(node: Node<'_>, source: &[u8]) -> Option<String> {
    first_identifier(node, source)
}

fn first_identifier(node: Node<'_>, source: &[u8]) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "identifier" || child.kind() == "type_identifier" {
                return child.utf8_text(source).ok().map(|s| s.trim().to_string());
            }
        }
    }
    None
}

fn ts_signature(node: Node<'_>, source: &[u8]) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "statement_block" {
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
