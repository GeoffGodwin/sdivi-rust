//! Round-trip serialization tests for `BoundarySpec`.

use sdivi_config::{BoundaryDef, BoundarySpec};

fn make_spec() -> BoundarySpec {
    BoundarySpec {
        version: None,
        boundaries: vec![
            BoundaryDef {
                name: "api".to_string(),
                description: Some("Public API layer".to_string()),
                modules: vec!["src/api/**".to_string()],
                allow_imports_from: vec!["models".to_string()],
            },
            BoundaryDef {
                name: "models".to_string(),
                description: None,
                modules: vec!["src/models/**".to_string()],
                allow_imports_from: vec![],
            },
        ],
    }
}

/// `to_yaml()` followed by `serde_yml::from_str` produces an equivalent spec.
#[test]
fn to_yaml_round_trips() {
    let spec = make_spec();
    let yaml = spec.to_yaml();
    assert!(!yaml.is_empty(), "to_yaml must produce non-empty output");

    let decoded: BoundarySpec =
        serde_yml::from_str(&yaml).expect("to_yaml output must be valid YAML");

    assert_eq!(decoded.boundaries.len(), spec.boundaries.len());
    assert_eq!(decoded.boundaries[0].name, "api");
    assert_eq!(decoded.boundaries[1].name, "models");
    assert_eq!(
        decoded.boundaries[0].allow_imports_from,
        vec!["models".to_string()]
    );
}

/// `to_yaml` output contains the boundary names.
#[test]
fn to_yaml_contains_names() {
    let spec = make_spec();
    let yaml = spec.to_yaml();
    assert!(
        yaml.contains("api"),
        "YAML must contain boundary name 'api'"
    );
    assert!(
        yaml.contains("models"),
        "YAML must contain boundary name 'models'"
    );
}

/// Empty spec round-trips cleanly.
#[test]
fn empty_spec_round_trips() {
    let spec = BoundarySpec {
        version: None,
        boundaries: vec![],
    };
    let yaml = spec.to_yaml();
    let decoded: BoundarySpec =
        serde_yml::from_str(&yaml).expect("empty spec YAML must be parseable");
    assert!(decoded.boundaries.is_empty());
}

/// Round-trip through file write + load preserves the spec.
#[test]
fn file_write_load_round_trip() {
    use std::io::Write as _;
    let spec = make_spec();
    let yaml = spec.to_yaml();

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("boundaries.yaml");
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(yaml.as_bytes()).unwrap();
    drop(f);

    let loaded = BoundarySpec::load(&path)
        .expect("load must succeed")
        .expect("loaded spec must be Some");

    assert_eq!(loaded.boundaries.len(), spec.boundaries.len());
    assert_eq!(loaded.boundaries[0].name, spec.boundaries[0].name);
    assert_eq!(loaded.boundaries[0].modules, spec.boundaries[0].modules);
}
