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

/// Extracts module specifiers from `import` and `from … import` statements.
///
/// - `import a, b.c, d as e` → `["a", "b.c", "d"]` (alias dropped)
/// - `from foo.bar import x` → `["foo.bar"]` (imported names are not modules)
/// - `from . import x` → `["."]`; `from .. import x` → `[".."]`
/// - `from ..pkg import x` → `["..pkg"]` (leading dots preserved for M26 resolver)
/// - `from __future__ import …` → nothing (synthetic module, never a real file)
///
/// M26 adds parent-path navigation for dot-relative specifiers.
pub(crate) fn extract_imports(root: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut imports = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        match node.kind() {
            "future_import_statement" => {
                continue; // __future__ is synthetic; never resolves to a file
            }
            "import_statement" => {
                collect_import_statement(node, source, &mut imports);
                continue;
            }
            "import_from_statement" => {
                collect_from_statement(node, source, &mut imports);
                continue;
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

/// Collects specifiers from `import a, b.c, d as e`.
fn collect_import_statement(node: Node<'_>, source: &[u8], imports: &mut Vec<String>) {
    for i in 0..node.child_count() {
        let Some(child) = node.child(i) else { continue };
        match child.kind() {
            "dotted_name" => {
                if let Some(text) = node_text(child, source) {
                    imports.push(text);
                }
            }
            "aliased_import" => {
                // `import a as b`: emit "a", drop the alias
                for j in 0..child.child_count() {
                    let Some(gc) = child.child(j) else { continue };
                    if gc.kind() == "dotted_name" {
                        if let Some(text) = node_text(gc, source) {
                            imports.push(text);
                        }
                        break;
                    }
                }
            }
            _ => {}
        }
    }
}

/// Collects the module specifier from `from [prefix] [name] import …`.
/// Relative imports produce a `relative_import` child; absolute ones produce a `dotted_name`.
fn collect_from_statement(node: Node<'_>, source: &[u8], imports: &mut Vec<String>) {
    let mut past_from = false;

    for i in 0..node.child_count() {
        let Some(child) = node.child(i) else { continue };
        let kind = child.kind();

        if kind == "from" {
            past_from = true;
            continue;
        }
        if kind == "import" && past_from {
            break;
        }
        if !past_from {
            continue;
        }

        match kind {
            "relative_import" => {
                if let Some(spec) = relative_import_specifier(child, source) {
                    imports.push(spec);
                }
                return;
            }
            "dotted_name" => {
                if let Some(text) = node_text(child, source) {
                    imports.push(text);
                }
                return;
            }
            _ => {}
        }
    }
}

/// Extracts the specifier from a `relative_import` node (e.g., `"."`, `"..pkg"`).
/// Structure: `relative_import` → `import_prefix` (dots) + optional `dotted_name`.
fn relative_import_specifier(node: Node<'_>, source: &[u8]) -> Option<String> {
    let mut dots = String::new();
    let mut name = String::new();
    for i in 0..node.child_count() {
        let Some(child) = node.child(i) else { continue };
        match child.kind() {
            "import_prefix" => {
                dots = node_text(child, source).unwrap_or_default();
            }
            "dotted_name" => {
                name = node_text(child, source).unwrap_or_default();
            }
            _ => {}
        }
    }

    if dots.is_empty() {
        return None;
    }
    Some(format!("{dots}{name}"))
}

/// Returns the trimmed UTF-8 text of `node`, or `None` if empty.
fn node_text(node: Node<'_>, source: &[u8]) -> Option<String> {
    node.utf8_text(source)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
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
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "block" {
                let sig_bytes = source.get(node.start_byte()..child.start_byte()).unwrap_or(&[]);
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
