//! CI gate: asserts the runtime `list_categories()` contract matches every
//! category string used in `crates/sdivi-patterns/src/`.
//!
//! This test catches the drift case where a new category string is added to
//! `sdivi_patterns::queries::category_for_node_kind` or `ALL_CATEGORIES` but
//! the author forgets to add it to `list_categories()`.
//!
//! Also asserts that the markdown table in `docs/pattern-categories.md`
//! enumerates the same set of categories as `list_categories()`.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

fn patterns_src_dir() -> std::path::PathBuf {
    // CARGO_MANIFEST_DIR for sdivi-core = crates/sdivi-core
    // parent()               = crates/
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/ parent must exist")
        .join("sdivi-patterns")
        .join("src")
}

fn workspace_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

/// Recursively collect strings that appear as `Some("…")` in `.rs` files.
fn collect_some_strings(dir: &Path) -> HashSet<String> {
    let mut found = HashSet::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(collect_some_strings(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            if let Ok(content) = fs::read_to_string(&path) {
                extract_some_strings(&content, &mut found);
            }
        }
    }
    found
}

/// Pull every string from `Some("…")` patterns in `source`.
fn extract_some_strings(source: &str, out: &mut HashSet<String>) {
    let needle = "Some(\"";
    let mut pos = 0;
    while let Some(start) = source[pos..].find(needle) {
        let after = pos + start + needle.len();
        if let Some(end) = source[after..].find('"') {
            let candidate = &source[after..after + end];
            // Only consider strings that look like category names:
            // lowercase letters and underscores only, length 3+.
            if candidate.len() >= 3
                && candidate
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c == '_')
            {
                out.insert(candidate.to_string());
            }
            pos = after + end + 1;
        } else {
            break;
        }
    }
}

/// Parse the canonical-category-list table rows from docs/pattern-categories.md.
///
/// Looks for lines like `| async_patterns | … |` in the table under the
/// "Canonical category list" heading and extracts the first column value.
fn parse_markdown_table_categories(md: &str) -> HashSet<String> {
    let mut found = HashSet::new();
    let mut in_table = false;
    for line in md.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## Canonical category list") {
            in_table = true;
            continue;
        }
        if in_table && trimmed.starts_with("## ") {
            break; // left the section
        }
        if in_table
            && trimmed.starts_with('|')
            && !trimmed.starts_with("| Category")
            && !trimmed.starts_with("|---")
        {
            // Extract first column: `| name | ... |`
            let cols: Vec<&str> = trimmed.split('|').collect();
            if cols.len() >= 2 {
                let name = cols[1].trim();
                if !name.is_empty() && name.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
                    found.insert(name.to_string());
                }
            }
        }
    }
    found
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[test]
fn list_categories_returns_schema_version_1_0() {
    let catalog = sdivi_core::list_categories();
    assert_eq!(
        catalog.schema_version, "1.0",
        "schema_version must be \"1.0\""
    );
}

#[test]
fn list_categories_returns_non_empty_categories() {
    let catalog = sdivi_core::list_categories();
    assert!(
        !catalog.categories.is_empty(),
        "list_categories must return at least one category"
    );
}

#[test]
fn list_categories_is_referentially_transparent() {
    let a = sdivi_core::list_categories();
    let b = sdivi_core::list_categories();
    assert_eq!(a, b, "list_categories must be referentially transparent");
}

#[test]
fn categories_constant_matches_list_categories() {
    let catalog = sdivi_core::list_categories();
    let runtime_names: HashSet<&str> = catalog.categories.iter().map(|c| c.name.as_str()).collect();
    for &name in sdivi_core::CATEGORIES {
        assert!(
            runtime_names.contains(name),
            "CATEGORIES constant contains {:?} but list_categories() does not",
            name
        );
    }
    assert_eq!(
        sdivi_core::CATEGORIES.len(),
        catalog.categories.len(),
        "CATEGORIES length must match list_categories() length"
    );
}

// ── Drift-gate: grep sdivi-patterns/src ───────────────────────────────────────

#[test]
fn no_category_string_in_patterns_src_missing_from_list_categories() {
    let catalog = sdivi_core::list_categories();
    let contract: HashSet<&str> = catalog.categories.iter().map(|c| c.name.as_str()).collect();

    let patterns_src = patterns_src_dir();
    let found = collect_some_strings(&patterns_src);

    for candidate in &found {
        assert!(
            contract.contains(candidate.as_str()),
            "String {:?} found via Some(\"…\") in crates/sdivi-patterns/src/ \
             but is absent from list_categories(). Either add it to CATEGORIES \
             in crates/sdivi-core/src/categories.rs or remove it from sdivi-patterns.",
            candidate
        );
    }
}

// ── Doc/runtime parity ────────────────────────────────────────────────────────

#[test]
fn markdown_table_matches_list_categories_output() {
    let doc_path = workspace_root().join("docs").join("pattern-categories.md");
    let md =
        fs::read_to_string(&doc_path).unwrap_or_else(|_| panic!("could not read {:?}", doc_path));

    let table_cats = parse_markdown_table_categories(&md);
    let catalog = sdivi_core::list_categories();
    let runtime_cats: HashSet<&str> = catalog.categories.iter().map(|c| c.name.as_str()).collect();

    for name in &table_cats {
        assert!(
            runtime_cats.contains(name.as_str()),
            "Category {:?} is in the docs/pattern-categories.md table \
             but absent from list_categories()",
            name
        );
    }
    for name in &runtime_cats {
        assert!(
            table_cats.contains(*name),
            "Category {:?} is in list_categories() but absent from \
             the docs/pattern-categories.md canonical-category-list table",
            name
        );
    }
}
