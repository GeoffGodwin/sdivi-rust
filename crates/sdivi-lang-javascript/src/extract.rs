//! AST extraction helpers for the JavaScript language adapter.

use sdivi_parsing::feature_record::PatternHint;
use sdivi_parsing::text::{js_string_content, truncate_to_256_bytes};
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
    "generator_function_declaration",
    "lexical_declaration",
    "variable_declaration",
];

/// Extracts module specifiers from `import` statements, `require()` calls, and
/// dynamic `import()` expressions in the AST.
///
/// - `import { foo } from "./utils"` → `["./utils"]`
/// - `const x = require("./utils")` → `["./utils"]`
/// - `import("./utils")` → `["./utils"]` when the grammar represents the callee
///   as an `import` node; best-effort and grammar-version-dependent
/// - `require(varName)` / `import(expr)` with non-string arg → skipped
pub(crate) fn extract_imports(root: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut imports = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        match node.kind() {
            "import_statement" => {
                if let Some(spec) = import_string_specifier(node, source) {
                    imports.push(spec);
                }
                continue; // don't recurse into import children
            }
            "call_expression" => {
                if let Some(spec) = require_or_dynamic_import_specifier(node, source) {
                    imports.push(spec);
                }
                // Still recurse — nested require() calls inside other expressions.
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

/// Returns the string specifier from a `require("…")` or dynamic `import("…")`.
///
/// Returns `None` if:
/// - the call is not `require` or `import`
/// - the argument is not a string literal (variable, template, etc.)
fn require_or_dynamic_import_specifier(call: Node<'_>, source: &[u8]) -> Option<String> {
    let func = call.child_by_field_name("function")?;
    let func_kind = func.kind();
    let is_require = func_kind == "identifier" && func.utf8_text(source).ok()? == "require";
    let is_dynamic_import = func_kind == "import";
    if !is_require && !is_dynamic_import {
        return None;
    }
    let args = call.child_by_field_name("arguments")?;
    for i in 0..args.child_count() {
        let Some(child) = args.child(i) else { continue };
        if child.kind() == "string" {
            return js_string_content(child, source);
        }
    }
    None
}

/// Finds the `string` child of an `import_statement` and returns its content.
fn import_string_specifier(import_node: Node<'_>, source: &[u8]) -> Option<String> {
    for i in 0..import_node.child_count() {
        let Some(child) = import_node.child(i) else {
            continue;
        };
        if child.kind() == "string" {
            return js_string_content(child, source);
        }
    }
    None
}

/// Extracts names of top-level exported items.
pub(crate) fn extract_exports(root: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut exports = Vec::new();
    for i in 0..root.child_count() {
        let Some(node) = root.child(i) else { continue };
        if node.kind() == "export_statement" {
            exports.extend(export_names(node, source));
        }
    }
    exports
}

/// Extracts function signatures as text up to the opening `{`.
pub(crate) fn extract_signatures(root: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut sigs = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.kind() == "function_declaration" || node.kind() == "method_definition" {
            if let Some(sig) = js_signature(node, source) {
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
            if let Some(name) = first_identifier(child, source) {
                names.push(name);
            }
        } else if child.kind() == "export_clause" {
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

fn js_signature(node: Node<'_>, source: &[u8]) -> Option<String> {
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
