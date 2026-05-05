//! Unit tests for tsconfig path-alias resolution (M27).
//!
//! Tests are exercised through `build_dependency_graph_with_tsconfig` with
//! synthetic FeatureRecords and hand-built `TsConfigPaths` structs.  The
//! internal `resolve_tsconfig_alias` helper is covered by in-crate tests
//! inside `tsconfig.rs`.

use sdivi_graph::dependency_graph::build_dependency_graph_with_tsconfig;
use sdivi_graph::{parse_tsconfig_content, TsConfigPaths};
use sdivi_parsing::feature_record::FeatureRecord;
use std::path::{Path, PathBuf};

fn ts_record(path: &str, imports: &[&str]) -> FeatureRecord {
    FeatureRecord {
        path: PathBuf::from(path),
        language: "typescript".to_string(),
        imports: imports.iter().map(|s| s.to_string()).collect(),
        exports: vec![],
        signatures: vec![],
        pattern_hints: vec![],
    }
}

fn js_record(path: &str, imports: &[&str]) -> FeatureRecord {
    FeatureRecord {
        path: PathBuf::from(path),
        language: "javascript".to_string(),
        imports: imports.iter().map(|s| s.to_string()).collect(),
        exports: vec![],
        signatures: vec![],
        pattern_hints: vec![],
    }
}

fn paths(base: &str, mappings: Vec<(&str, Vec<&str>)>) -> TsConfigPaths {
    TsConfigPaths {
        base: PathBuf::from(base),
        mappings: mappings
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect()))
            .collect(),
    }
}

// ── wildcard alias ────────────────────────────────────────────────────────────

#[test]
fn wildcard_alias_at_slash_resolves_edge() {
    // "@/*" → "./*" with base="": @/lib/utils → lib/utils.ts
    let tc = paths("", vec![("@/*", vec!["./*"])]);
    let records = vec![
        ts_record("src/app.ts", &["@/lib/utils"]),
        ts_record("lib/utils.ts", &[]),
    ];
    let dg = build_dependency_graph_with_tsconfig(&records, None, Some(&tc));
    assert_eq!(dg.edge_count(), 1, "@/lib/utils must resolve to lib/utils.ts");
}

#[test]
fn wildcard_alias_with_base_url() {
    // base_url = "src", "@/*" → "./*": @/utils → src/utils.ts
    let tc = paths("src", vec![("@/*", vec!["./*"])]);
    let records = vec![
        ts_record("src/app.ts", &["@/utils"]),
        ts_record("src/utils.ts", &[]),
    ];
    let dg = build_dependency_graph_with_tsconfig(&records, None, Some(&tc));
    assert_eq!(dg.edge_count(), 1, "@/utils must resolve to src/utils.ts");
}

// ── exact alias ───────────────────────────────────────────────────────────────

#[test]
fn exact_alias_resolves_edge() {
    // "~lib" → ["./src/lib/index.ts"] (no wildcard)
    let tc = paths("", vec![("~lib", vec!["./src/lib/index.ts"])]);
    let records = vec![
        ts_record("src/app.ts", &["~lib"]),
        ts_record("src/lib/index.ts", &[]),
    ];
    let dg = build_dependency_graph_with_tsconfig(&records, None, Some(&tc));
    assert_eq!(dg.edge_count(), 1, "~lib must resolve to src/lib/index.ts");
}

// ── multi-target fallback ────────────────────────────────────────────────────

#[test]
fn multi_target_falls_back_to_second() {
    // "@x/*" → ["a/*", "b/*"]; only b/foo.ts exists
    let tc = paths("", vec![("@x/*", vec!["a/*", "b/*"])]);
    let records = vec![
        ts_record("src/app.ts", &["@x/foo"]),
        ts_record("b/foo.ts", &[]),
    ];
    let dg = build_dependency_graph_with_tsconfig(&records, None, Some(&tc));
    assert_eq!(
        dg.edge_count(),
        1,
        "@x/foo must fall through to b/foo.ts when a/foo.ts is absent"
    );
}

// ── no match → external ──────────────────────────────────────────────────────

#[test]
fn unmatched_specifier_produces_no_edge() {
    let tc = paths("", vec![("@/*", vec!["./*"])]);
    let records = vec![
        ts_record("src/app.ts", &["react"]),
        ts_record("lib/utils.ts", &[]),
    ];
    let dg = build_dependency_graph_with_tsconfig(&records, None, Some(&tc));
    assert_eq!(dg.edge_count(), 0, "external 'react' must not match alias");
}

// ── match but target absent ──────────────────────────────────────────────────

#[test]
fn matched_alias_target_absent_produces_no_edge() {
    let tc = paths("", vec![("@/*", vec!["./*"])]);
    let records = vec![
        ts_record("src/app.ts", &["@/missing"]),
        ts_record("lib/utils.ts", &[]),
    ];
    let dg = build_dependency_graph_with_tsconfig(&records, None, Some(&tc));
    assert_eq!(dg.edge_count(), 0, "unresolvable alias target must not add edge");
}

// ── no tsconfig → alias specifiers are external ──────────────────────────────

#[test]
fn no_tsconfig_alias_specifier_is_external() {
    let records = vec![
        ts_record("src/app.ts", &["@/lib/utils"]),
        ts_record("lib/utils.ts", &[]),
    ];
    let dg = build_dependency_graph_with_tsconfig(&records, None, None);
    assert_eq!(dg.edge_count(), 0, "without tsconfig @/lib/utils must be external");
}

// ── JavaScript uses alias map too ────────────────────────────────────────────

#[test]
fn javascript_alias_resolves_edge() {
    let tc = paths("", vec![("@/*", vec!["./*"])]);
    let records = vec![
        js_record("src/index.js", &["@/helpers"]),
        js_record("helpers.js", &[]),
    ];
    let dg = build_dependency_graph_with_tsconfig(&records, None, Some(&tc));
    assert_eq!(dg.edge_count(), 1, "JS alias @/helpers must resolve to helpers.js");
}

// ── prefix+suffix pattern ────────────────────────────────────────────────────

#[test]
fn prefix_suffix_pattern_resolves() {
    // "#int/*.types" → ["src/types/*.types.ts"]
    let tc = paths("", vec![("#int/*.types", vec!["src/types/*.types.ts"])]);
    let records = vec![
        ts_record("app.ts", &["#int/foo.types"]),
        ts_record("src/types/foo.types.ts", &[]),
    ];
    let dg = build_dependency_graph_with_tsconfig(&records, None, Some(&tc));
    assert_eq!(dg.edge_count(), 1, "#int/foo.types must resolve via prefix+suffix");
}

// ── relative imports bypass alias dispatch ────────────────────────────────────

#[test]
fn relative_imports_not_affected_by_tsconfig() {
    let tc = paths("", vec![("@/*", vec!["./*"])]);
    let records = vec![
        ts_record("src/app.ts", &["./utils"]),
        ts_record("src/utils.ts", &[]),
    ];
    let dg = build_dependency_graph_with_tsconfig(&records, None, Some(&tc));
    assert_eq!(dg.edge_count(), 1, "./utils must resolve via relative, not alias");
}

// ── parse_tsconfig_content ────────────────────────────────────────────────────

#[test]
fn parse_tsconfig_with_base_url_and_paths() {
    let json = r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": { "@/*": ["./*"] }
  }
}"#;
    let tc = parse_tsconfig_content(json, Path::new("")).expect("must parse");
    assert_eq!(tc.base, PathBuf::from(""));
    assert_eq!(tc.mappings.len(), 1);
    assert_eq!(tc.mappings[0].0, "@/*");
}

#[test]
fn parse_tsconfig_jsonc_with_comments_and_trailing_commas() {
    let json = r#"{
  // inline comment
  "compilerOptions": {
    "baseUrl": "src", /* block comment */
    "paths": {
      "@/*": ["./*"],
    },
  },
}"#;
    let tc = parse_tsconfig_content(json, Path::new("")).expect("must parse JSONC");
    assert_eq!(tc.base, PathBuf::from("src"));
    assert_eq!(tc.mappings.len(), 1);
}

#[test]
fn parse_tsconfig_invalid_json_returns_none() {
    let bad = r#"{ not valid json }"#;
    let tc = parse_tsconfig_content(bad, Path::new(""));
    assert!(tc.is_none(), "invalid JSON must yield None");
}

#[test]
fn parse_tsconfig_two_star_pattern_skipped() {
    let json = r#"{"compilerOptions":{"paths":{"a/*/b/*":["x/*"],"@/*":["y/*"]}}}"#;
    let tc = parse_tsconfig_content(json, Path::new("")).expect("parse ok");
    // a/*/b/* has two stars → skipped; @/* has one → kept
    assert_eq!(tc.mappings.len(), 1, "two-star pattern must be skipped");
    assert_eq!(tc.mappings[0].0, "@/*");
}

// ── parse_tsconfig_content edge cases ────────────────────────────────────────

#[test]
fn parse_tsconfig_no_compiler_options_returns_none() {
    // A valid JSON file with no compilerOptions section must yield None (no alias
    // info is derivable).
    let json = r#"{"include": ["src/**/*"]}"#;
    let result = parse_tsconfig_content(json, Path::new(""));
    assert!(
        result.is_none(),
        "tsconfig without compilerOptions must return None"
    );
}

#[test]
fn parse_tsconfig_only_base_url_no_paths_returns_empty_mappings() {
    // When only baseUrl is set and paths is absent, we return Some with an
    // empty mappings list.  Pure-baseUrl resolution is explicitly deferred per
    // spec; the caller sees Some (alias config present) with nothing to match.
    let json = r#"{"compilerOptions": {"baseUrl": "src"}}"#;
    let tc = parse_tsconfig_content(json, Path::new("")).expect("must return Some");
    assert_eq!(
        tc.base,
        PathBuf::from("src"),
        "base must reflect baseUrl value"
    );
    assert!(
        tc.mappings.is_empty(),
        "no paths block → mappings must be empty"
    );
}

#[test]
fn jsonc_escaped_quote_inside_string_value_preserved() {
    // JSONC stripping must not treat '\"' inside a string as the end of the
    // string.  A tsconfig with an escaped-quote in a description field must
    // still parse correctly, and the paths entry must survive.
    let json = r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": { "@/*": ["./*"] }
  },
  "description": "project \"alias\" config"
}"#;
    let tc = parse_tsconfig_content(json, Path::new("")).expect("must parse");
    assert_eq!(tc.mappings.len(), 1);
    assert_eq!(tc.mappings[0].0, "@/*");
}

#[test]
fn determinism_two_builds_produce_identical_edge_list() {
    // Critical Rule 1: same inputs → bit-identical outputs.  Run
    // build_dependency_graph_with_tsconfig twice with the same records and
    // tsconfig and assert the edge list is identical both times.
    let tc = paths("", vec![("@/*", vec!["./*"]), ("~lib", vec!["./src/lib/index.ts"])]);
    let records = vec![
        ts_record("src/app.ts", &["@/lib/utils", "~lib"]),
        ts_record("lib/utils.ts", &[]),
        ts_record("src/lib/index.ts", &[]),
    ];

    let dg1 = build_dependency_graph_with_tsconfig(&records, None, Some(&tc));
    let dg2 = build_dependency_graph_with_tsconfig(&records, None, Some(&tc));

    let mut edges1 = dg1.edges_as_pairs();
    let mut edges2 = dg2.edges_as_pairs();
    edges1.sort_unstable();
    edges2.sort_unstable();

    assert_eq!(
        edges1, edges2,
        "two identical builds must produce the same edge list"
    );
    assert_eq!(
        dg1.edge_count(),
        dg2.edge_count(),
        "edge count must be identical across runs"
    );
}

#[test]
fn no_panic_on_varied_specifiers_and_patterns() {
    // Property-style guard: the resolver must not panic and must return either
    // a node in path_to_node or an empty Vec for every (specifier, alias config)
    // combination below.  Covers: exact, wildcard, prefix+suffix, empty
    // specifier, path-separator edge cases, Unicode, very long strings.
    let cases: &[(&str, Vec<(&str, Vec<&str>)>)] = &[
        ("@/foo/bar", vec![("@/*", vec!["./*"])]),
        ("~lib", vec![("~lib", vec!["./src/lib/index.ts"])]),
        ("#int/x.types", vec![("#int/*.types", vec!["src/types/*.types.ts"])]),
        ("", vec![("@/*", vec!["./*"])]),
        ("@/", vec![("@/*", vec!["./*"])]),
        ("react", vec![("@/*", vec!["./*"])]),
        ("../relative", vec![("@/*", vec!["./*"])]),
        ("./local", vec![("@/*", vec!["./*"])]),
        // Unicode alias
        ("α/foo", vec![("α/*", vec!["./*"])]),
        // Very long specifier (well beyond any reasonable path)
        (
            "@/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            vec![("@/*", vec!["./*"])],
        ),
        // Specifier matches prefix of pattern but not suffix
        ("#int/foo", vec![("#int/*.types", vec!["src/types/*.types.ts"])]),
        // Multiple patterns; second matches
        (
            "@x/bar",
            vec![("@y/*", vec!["nope/*"]), ("@x/*", vec!["lib/*"])],
        ),
    ];

    let known_nodes = vec![
        ts_record("lib/foo/bar.ts", &[]),
        ts_record("src/lib/index.ts", &[]),
        ts_record("src/types/x.types.ts", &[]),
        ts_record("lib/bar.ts", &[]),
    ];

    for (specifier, mapping) in cases {
        let tc = paths("", mapping.clone());
        let importer = ts_record("src/app.ts", &[specifier]);
        let mut records = vec![importer];
        records.extend(known_nodes.iter().cloned());

        // Must not panic; result must be 0 or 1 edges (never fabricates a node
        // outside path_to_node).
        let dg = build_dependency_graph_with_tsconfig(&records, None, Some(&tc));
        assert!(
            dg.edge_count() <= 1,
            "specifier {:?} must resolve to at most one edge, got {}",
            specifier,
            dg.edge_count()
        );
    }
}

// ── fixture regression sentinel ───────────────────────────────────────────────

#[test]
fn tsconfig_alias_fixture_edge_count() {
    let fixture_root = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/tsconfig-alias"
    ));
    let json = std::fs::read_to_string(fixture_root.join("tsconfig.json"))
        .expect("tsconfig.json must exist in fixture");
    let tc = parse_tsconfig_content(&json, Path::new("")).expect("fixture tsconfig must parse");

    use sdivi_config::Config;
    use sdivi_lang_typescript::TypeScriptAdapter;
    use sdivi_parsing::adapter::LanguageAdapter;
    use sdivi_parsing::parse::parse_repository;
    let adapters: Vec<Box<dyn LanguageAdapter>> = vec![Box::new(TypeScriptAdapter)];
    let records = parse_repository(&Config::default(), &fixture_root, &adapters);
    let dg = build_dependency_graph_with_tsconfig(&records, None, Some(&tc));
    // src/app.ts → src/lib/utils.ts (@/src/lib/utils) and src/lib/index.ts (~lib)
    assert_eq!(dg.edge_count(), 2, "tsconfig-alias fixture must produce 2 alias-resolved edges");
}
