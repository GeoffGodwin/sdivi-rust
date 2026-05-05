//! tsconfig.json / jsconfig.json path-alias parsing for TS/JS resolution.
//!
//! Reads `compilerOptions.baseUrl` and `compilerOptions.paths` from the config
//! at the repository root only (`extends` chains and per-package monorepo
//! tsconfigs are deferred to a follow-up milestone).
//!
//! JSONC tolerance: `//` line comments, `/* */` block comments, and trailing
//! commas are stripped by a state-machine pre-pass before `serde_json`.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use petgraph::graph::NodeIndex;
use tracing::warn;

/// Parsed alias map from `compilerOptions.paths` in a tsconfig / jsconfig file.
///
/// `base` is the effective resolution root (tsconfig dir + `baseUrl` if set)
/// expressed **relative to the repository root**. `mappings` is in BTreeMap key
/// order from serde_json (alphabetical); insertion-order semantics are a
/// known limitation for projects where key order is significant.
#[derive(Debug, Clone)]
pub struct TsConfigPaths {
    /// Effective base for alias resolution, relative to repo root.
    pub base: PathBuf,
    /// Pattern → target list pairs, in JSON key order.
    pub mappings: Vec<(String, Vec<String>)>,
}

// Re-exported publicly via sdivi_graph::TsConfigPaths and sdivi_graph::parse_tsconfig_content.

pub(crate) fn strip_jsonc(input: &str) -> String {
    let without_comments = {
        let mut out = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();
        let (mut in_str, mut in_block, mut in_line) = (false, false, false);
        while let Some(c) = chars.next() {
            if in_line {
                if c == '\n' {
                    out.push('\n');
                    in_line = false;
                }
                continue;
            }
            if in_block {
                if c == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    in_block = false;
                }
                continue;
            }
            if in_str {
                out.push(c);
                if c == '\\' {
                    if let Some(e) = chars.next() {
                        out.push(e);
                    }
                } else if c == '"' {
                    in_str = false;
                }
                continue;
            }
            match c {
                '"' => {
                    in_str = true;
                    out.push(c);
                }
                '/' => match chars.peek() {
                    Some('/') => {
                        chars.next();
                        in_line = true;
                    }
                    Some('*') => {
                        chars.next();
                        in_block = true;
                    }
                    _ => out.push(c),
                },
                _ => out.push(c),
            }
        }
        out
    };
    // Strip trailing commas before } or ]
    let chars: Vec<char> = without_comments.chars().collect();
    let mut out = String::with_capacity(without_comments.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == ',' {
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {
                i += 1;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn normalize_rel(p: PathBuf) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in p.components() {
        if let Component::Normal(n) = comp {
            out.push(n);
        }
    }
    out
}

/// Parses tsconfig.json / jsconfig.json content and returns alias info.
///
/// `tsconfig_rel_dir` is the tsconfig directory relative to the repository root
/// (pass `""` for a root-level config). Returns `None` on parse failure (logs a
/// `WARN`) or when neither `baseUrl` nor `paths` is present. Patterns with two
/// or more `*` chars are skipped with a `WARN`.
pub fn parse_tsconfig_content(content: &str, tsconfig_rel_dir: &Path) -> Option<TsConfigPaths> {
    let value: serde_json::Value = match serde_json::from_str(&strip_jsonc(content)) {
        Ok(v) => v,
        Err(e) => {
            warn!(
                "tsconfig.json unparseable after comment-strip: {}; alias resolution disabled",
                e
            );
            return None;
        }
    };
    let opts = value.get("compilerOptions")?;
    let base_url = opts.get("baseUrl").and_then(|v| v.as_str());
    let base = if let Some(url) = base_url {
        normalize_rel(tsconfig_rel_dir.join(url))
    } else {
        normalize_rel(tsconfig_rel_dir.to_path_buf())
    };
    let paths_obj = match opts.get("paths").and_then(|v| v.as_object()) {
        Some(obj) => obj,
        None => {
            return Some(TsConfigPaths {
                base,
                mappings: vec![],
            })
        }
    };
    let mut mappings = Vec::new();
    for (key, val) in paths_obj {
        if key.matches('*').count() > 1 {
            warn!("tsconfig.json: pattern '{}' has >1 '*'; skipping", key);
            continue;
        }
        let targets: Vec<String> = val
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        mappings.push((key.clone(), targets));
    }
    Some(TsConfigPaths { base, mappings })
}

/// Returns the wildcard capture if `specifier` matches `pattern`, else `None`.
fn match_alias(pattern: &str, specifier: &str) -> Option<String> {
    match pattern.find('*') {
        None => (specifier == pattern).then(String::new),
        Some(star) => {
            let (pre, suf) = (&pattern[..star], &pattern[star + 1..]);
            if specifier.starts_with(pre)
                && specifier.ends_with(suf)
                && specifier.len() >= pre.len() + suf.len()
            {
                Some(specifier[pre.len()..specifier.len() - suf.len()].to_string())
            } else {
                None
            }
        }
    }
}

fn apply_capture(target: &str, capture: &str) -> String {
    match target.find('*') {
        None => target.to_string(),
        Some(s) => format!("{}{}{}", &target[..s], capture, &target[s + 1..]),
    }
}

/// Resolves a TS/JS import specifier against tsconfig path aliases.
///
/// Iterates `paths.mappings` in declaration order; first pattern match wins.
/// Within a pattern, first resolving target wins. Returns empty `Vec` if no
/// alias matched or no target resolved to a known node.
pub(crate) fn resolve_tsconfig_alias(
    specifier: &str,
    paths: &TsConfigPaths,
    path_to_node: &BTreeMap<PathBuf, NodeIndex>,
) -> Vec<NodeIndex> {
    for (pattern, targets) in &paths.mappings {
        let Some(capture) = match_alias(pattern, specifier) else {
            continue;
        };
        for target in targets {
            let sub = apply_capture(target, &capture);
            let rem = sub.strip_prefix("./").unwrap_or(&sub);
            let node = crate::resolve::try_path(&paths.base, rem, "typescript", path_to_node)
                .or_else(|| crate::resolve::try_path(&paths.base, rem, "javascript", path_to_node));
            if let Some(ni) = node {
                return vec![ni];
            }
        }
    }
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_line_comment() {
        let v: serde_json::Value =
            serde_json::from_str(&strip_jsonc("{ \"a\": 1 // c\n}")).unwrap();
        assert_eq!(v["a"], 1);
    }
    #[test]
    fn strip_block_comment() {
        let v: serde_json::Value =
            serde_json::from_str(&strip_jsonc(r#"{"a": /* c */ 2}"#)).unwrap();
        assert_eq!(v["a"], 2);
    }
    #[test]
    fn preserve_url_in_string() {
        let v: serde_json::Value =
            serde_json::from_str(&strip_jsonc(r#"{"u":"http://x.com"}"#)).unwrap();
        assert_eq!(v["u"], "http://x.com");
    }
    #[test]
    fn trailing_comma_object() {
        let v: serde_json::Value = serde_json::from_str(&strip_jsonc(r#"{"a":1,}"#)).unwrap();
        assert_eq!(v["a"], 1);
    }
    #[test]
    fn trailing_comma_array() {
        let v: serde_json::Value = serde_json::from_str(&strip_jsonc(r#"[1,2,]"#)).unwrap();
        assert_eq!(v, serde_json::json!([1, 2]));
    }
    #[test]
    fn match_exact() {
        assert_eq!(match_alias("~lib", "~lib"), Some(String::new()));
        assert_eq!(match_alias("~lib", "~other"), None);
    }
    #[test]
    fn match_wildcard_prefix() {
        assert_eq!(match_alias("@/*", "@/lib/foo"), Some("lib/foo".to_string()));
        assert_eq!(match_alias("@/*", "other"), None);
    }
    #[test]
    fn match_prefix_suffix() {
        assert_eq!(
            match_alias("#int/*.t", "#int/foo.t"),
            Some("foo".to_string())
        );
    }
    #[test]
    fn apply_no_star() {
        assert_eq!(apply_capture("./index.ts", ""), "./index.ts");
    }
    #[test]
    fn apply_star() {
        assert_eq!(apply_capture("./*", "lib/foo"), "./lib/foo");
    }
}
