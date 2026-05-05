//! Import resolution strategies for the dependency graph stage.
//!
//! Dispatches to per-language resolvers based on specifier shape and the
//! `language` field of the importing record. Relative-path specifiers
//! (`./` and `../`) always route through [`resolve_relative`] regardless of
//! language, using the importer file's extension to select the extension list.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use petgraph::graph::NodeIndex;
use tracing::debug;

use crate::resolve_lang::{
    resolve_go_module, resolve_java_dotted, resolve_python_bare, resolve_python_relative,
};
use crate::tsconfig::TsConfigPaths;

/// Returns `(file_extensions, directory_index_names)` for a language.
///
/// Files are tried before directory-index files; within each list, order is
/// the tie-break priority. The cross-language fallback (unknown `lang`)
/// tries all extensions to preserve pre-M26 behavior.
pub(crate) fn extensions_for_language(
    lang: &str,
) -> (&'static [&'static str], &'static [&'static str]) {
    match lang {
        "rust" => (&["rs"], &["mod.rs"]),
        "python" => (&["py"], &["__init__.py"]),
        "typescript" => (&["ts", "tsx", "d.ts"], &["index.ts", "index.tsx"]),
        "javascript" => (
            &["js", "jsx", "mjs", "cjs"],
            &["index.js", "index.jsx", "index.mjs", "index.cjs"],
        ),
        "go" => (&["go"], &[]),
        "java" => (&["java"], &[]),
        _ => (
            &[
                "rs", "py", "ts", "tsx", "js", "jsx", "mjs", "cjs", "go", "java", "d.ts",
            ],
            &[
                "mod.rs",
                "__init__.py",
                "index.ts",
                "index.tsx",
                "index.js",
                "index.jsx",
            ],
        ),
    }
}

/// Infers the effective language from a file path extension.
///
/// Used so that test fixtures with `language: "rust"` but `.ts`/`.py` paths
/// still use the correct extension list for relative imports.
fn lang_from_path_ext(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("py") => "python",
        Some("ts" | "tsx") => "typescript",
        Some("js" | "jsx" | "mjs" | "cjs") => "javascript",
        Some("go") => "go",
        Some("java") => "java",
        _ => "rust",
    }
}

/// Builds a stem → `[NodeIndex]` lookup for Rust `crate::` / `self::` / `super::`.
pub(crate) fn build_stem_map(
    path_to_node: &BTreeMap<PathBuf, NodeIndex>,
) -> BTreeMap<String, Vec<NodeIndex>> {
    let mut map: BTreeMap<String, Vec<NodeIndex>> = BTreeMap::new();
    for (path, &ni) in path_to_node {
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            map.entry(stem.to_ascii_lowercase()).or_default().push(ni);
        }
    }
    map
}

fn resolve_stem(stem: &str, stem_map: &BTreeMap<String, Vec<NodeIndex>>) -> Option<NodeIndex> {
    let v = stem_map.get(&stem.to_ascii_lowercase())?;
    if v.len() == 1 {
        Some(v[0])
    } else {
        None
    }
}

fn find_stem_in_subtree(
    stem: &str,
    base: &Path,
    path_to_node: &BTreeMap<PathBuf, NodeIndex>,
) -> Option<NodeIndex> {
    let key = stem.to_ascii_lowercase();
    let hits: Vec<NodeIndex> = path_to_node
        .iter()
        .filter(|(p, _)| {
            p.starts_with(base)
                && p.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_ascii_lowercase() == key)
                    .unwrap_or(false)
        })
        .map(|(_, &ni)| ni)
        .collect();
    if hits.len() == 1 {
        Some(hits[0])
    } else {
        None
    }
}

/// Builds the list of Java source roots for the repository.
///
/// Starts from `["", "src/main/java", "src/test/java", "src", "java"]` and
/// discovers any additional module-specific roots (multi-module Maven/Gradle)
/// by scanning `path_to_node` keys for `/src/main/java` and `/src/test/java`.
pub(crate) fn compute_java_roots(path_to_node: &BTreeMap<PathBuf, NodeIndex>) -> Vec<PathBuf> {
    let mut roots = vec![
        PathBuf::from(""),
        PathBuf::from("src/main/java"),
        PathBuf::from("src/test/java"),
        PathBuf::from("src"),
        PathBuf::from("java"),
    ];
    for path in path_to_node.keys() {
        let s = path.to_string_lossy();
        for suffix in &["/src/main/java", "/src/test/java"] {
            if let Some(idx) = s.find(suffix) {
                let root = PathBuf::from(&s[..idx + suffix.len()]);
                if !roots.contains(&root) {
                    roots.push(root);
                }
            }
        }
    }
    roots
}

/// Tries `base/rem` with language-specific extensions; files beat directory-index.
///
/// Order: (1) file extensions, (2) verbatim path, (3) directory-index files.
pub(crate) fn try_path(
    base: &Path,
    rem: &str,
    lang: &str,
    path_to_node: &BTreeMap<PathBuf, NodeIndex>,
) -> Option<NodeIndex> {
    let (file_exts, dir_exts) = extensions_for_language(lang);
    for ext in file_exts {
        if let Some(&ni) = path_to_node.get(&base.join(format!("{rem}.{ext}"))) {
            return Some(ni);
        }
    }
    if let Some(&ni) = path_to_node.get(&base.join(rem)) {
        return Some(ni);
    }
    for idx in dir_exts {
        if let Some(&ni) = path_to_node.get(&base.join(rem).join(idx)) {
            return Some(ni);
        }
    }
    None
}

/// Resolves `import` from `from_path` and returns matching [`NodeIndex`] targets.
///
/// An empty vec means the import is external or unresolvable; the caller logs
/// at `DEBUG` and drops it. A vec with more than one element occurs for Java
/// wildcard imports and Go package-directory imports.
#[allow(clippy::too_many_arguments)] // 8 args: per-language resolver inputs (stem map, path map, go module, java roots, tsconfig) all load-bearing; bundling would just push the same fields into a struct
pub(crate) fn resolve_imports(
    import: &str,
    from_path: &Path,
    language: &str,
    stem_map: &BTreeMap<String, Vec<NodeIndex>>,
    path_to_node: &BTreeMap<PathBuf, NodeIndex>,
    go_mod_prefix: Option<&str>,
    java_roots: &[PathBuf],
    tsconfig: Option<&TsConfigPaths>,
) -> Vec<NodeIndex> {
    // Relative specifiers always use resolve_relative; extension list is inferred
    // from the importer's file extension so test fixtures with language:"rust"
    // and .ts/.py paths resolve correctly.
    if import.starts_with("./") || import.starts_with("../") {
        let eff_lang = lang_from_path_ext(from_path);
        return resolve_relative(import, from_path, eff_lang, path_to_node);
    }

    match language {
        "python" => {
            if import.starts_with('.') {
                resolve_python_relative(import, from_path, path_to_node)
            } else {
                resolve_python_bare(import, path_to_node)
            }
        }
        "go" => match go_mod_prefix {
            Some(prefix) => resolve_go_module(import, prefix, path_to_node),
            None => vec![],
        },
        "java" => resolve_java_dotted(import, java_roots, path_to_node),
        "typescript" | "javascript" => {
            if let Some(tc) = tsconfig {
                let v = crate::tsconfig::resolve_tsconfig_alias(import, tc, path_to_node);
                if !v.is_empty() {
                    return v;
                }
            }
            vec![]
        }
        _ => {
            if let Some(local) = import
                .strip_prefix("crate::")
                .or_else(|| import.strip_prefix("self::"))
            {
                let first = local.split("::").next().unwrap_or(local);
                return resolve_stem(first, stem_map).into_iter().collect();
            }
            if import.starts_with("super::") {
                return resolve_super(import, from_path, stem_map, path_to_node);
            }
            vec![]
        }
    }
}

fn resolve_relative(
    import: &str,
    from_path: &Path,
    lang: &str,
    path_to_node: &BTreeMap<PathBuf, NodeIndex>,
) -> Vec<NodeIndex> {
    let mut rest = import;
    let mut levels_up: usize = 0;

    if let Some(s) = rest.strip_prefix("./") {
        rest = s;
    } else {
        while let Some(s) = rest.strip_prefix("../") {
            levels_up += 1;
            rest = s;
        }
    }

    let from_dir = match from_path.parent() {
        Some(d) => d.to_path_buf(),
        None => return vec![],
    };

    let mut base = from_dir;
    for _ in 0..levels_up {
        match base.parent().map(|p| p.to_path_buf()) {
            Some(p) => base = p,
            None => {
                debug!(%import, path = ?from_path, "relative import overshoots repo root, dropped");
                return vec![];
            }
        }
    }

    try_path(&base, rest, lang, path_to_node)
        .into_iter()
        .collect()
}

fn resolve_super(
    import: &str,
    from_path: &Path,
    stem_map: &BTreeMap<String, Vec<NodeIndex>>,
    path_to_node: &BTreeMap<PathBuf, NodeIndex>,
) -> Vec<NodeIndex> {
    let mut rest = import;
    let mut levels_up: usize = 0;
    while let Some(s) = rest.strip_prefix("super::") {
        levels_up += 1;
        rest = s;
    }
    let stem = rest.split("::").next().unwrap_or(rest);

    let from_dir = match from_path.parent() {
        Some(d) => d.to_path_buf(),
        None => return resolve_stem(stem, stem_map).into_iter().collect(),
    };

    let mut base = from_dir;
    for _ in 0..levels_up {
        match base.parent().map(|p| p.to_path_buf()) {
            Some(p) => base = p,
            None => return resolve_stem(stem, stem_map).into_iter().collect(),
        }
    }

    find_stem_in_subtree(stem, &base, path_to_node)
        .or_else(|| resolve_stem(stem, stem_map))
        .into_iter()
        .collect()
}
