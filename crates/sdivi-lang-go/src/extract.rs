//! AST extraction helpers for the Go language adapter.

use sdivi_parsing::feature_record::PatternHint;
use sdivi_parsing::text::truncate_to_256_bytes;
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
///
/// One specifier is emitted per `import_spec`, regardless of import form:
/// - `import "fmt"` → `["fmt"]`
/// - `import ( "fmt"; "os" )` → `["fmt", "os"]` (one per spec)
/// - `import f "fmt"` → `["fmt"]` (alias dropped)
/// - `import . "fmt"` → `["fmt"]` (dot import, namespace marker dropped)
/// - `import _ "github.com/lib/pq"` → `["github.com/lib/pq"]` (blank import)
pub(crate) fn extract_imports(root: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut imports = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.kind() == "import_declaration" {
            collect_import_declaration(node, source, &mut imports);
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

/// Collects specifiers from a single `import_declaration`.
fn collect_import_declaration(node: Node<'_>, source: &[u8], imports: &mut Vec<String>) {
    for i in 0..node.child_count() {
        let Some(child) = node.child(i) else { continue };
        match child.kind() {
            "import_spec" => {
                if let Some(s) = import_spec_path(child, source) {
                    imports.push(s);
                }
            }
            "import_spec_list" => {
                for j in 0..child.child_count() {
                    let Some(spec) = child.child(j) else { continue };
                    if spec.kind() == "import_spec" {
                        if let Some(s) = import_spec_path(spec, source) {
                            imports.push(s);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Returns the unquoted path string from an `import_spec` node.
///
/// Uses the `path` field if available; falls back to scanning for an
/// `interpreted_string_literal` child. Strips the surrounding `"` characters.
fn import_spec_path(spec: Node<'_>, source: &[u8]) -> Option<String> {
    if let Some(path_node) = spec.child_by_field_name("path") {
        return strip_go_string(path_node, source);
    }
    for i in 0..spec.child_count() {
        let Some(child) = spec.child(i) else { continue };
        if child.kind() == "interpreted_string_literal" {
            return strip_go_string(child, source);
        }
    }
    None
}

/// Strips the surrounding `"` quotes from a Go interpreted string literal.
fn strip_go_string(node: Node<'_>, source: &[u8]) -> Option<String> {
    let text = node.utf8_text(source).ok()?;
    let t = text.trim();
    if t.starts_with('"') && t.ends_with('"') && t.len() >= 2 {
        Some(t[1..t.len() - 1].to_string())
    } else {
        None
    }
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
