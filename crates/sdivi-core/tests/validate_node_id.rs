use sdivi_core::input::validate_node_id;

#[test]
fn accepts_simple_path() {
    assert!(validate_node_id("src/lib.rs").is_ok());
}

#[test]
fn accepts_single_segment() {
    assert!(validate_node_id("Cargo.toml").is_ok());
}

#[test]
fn accepts_deep_path() {
    assert!(validate_node_id("crates/sdivi-core/src/lib.rs").is_ok());
}

#[test]
fn rejects_empty() {
    let err = validate_node_id("").unwrap_err();
    assert!(format!("{err}").contains("empty"));
}

#[test]
fn rejects_leading_dot_slash() {
    let err = validate_node_id("./foo").unwrap_err();
    assert!(format!("{err}").contains("./"));
}

#[test]
fn rejects_trailing_slash() {
    let err = validate_node_id("foo/").unwrap_err();
    assert!(format!("{err}").contains("/"));
}

#[test]
fn rejects_backslash() {
    let err = validate_node_id("foo\\bar").unwrap_err();
    assert!(format!("{err}").contains("forward"));
}

#[test]
fn rejects_dotdot_component() {
    let err = validate_node_id("../foo").unwrap_err();
    assert!(format!("{err}").contains(".."));
}

#[test]
fn rejects_embedded_dotdot() {
    let err = validate_node_id("src/../foo").unwrap_err();
    assert!(format!("{err}").contains(".."));
}

#[test]
fn rejects_absolute_path() {
    let err = validate_node_id("/foo").unwrap_err();
    assert!(format!("{err}").contains("absolute"));
}

#[test]
fn accepts_dot_in_filename() {
    assert!(validate_node_id("src/.env.example").is_ok());
}

// A bare "." is not ".." and does not start with "./" so the current validator
// accepts it.  This test documents that behaviour so a future tightening is a
// deliberate, visible change.
#[test]
fn accepts_standalone_dot() {
    assert!(validate_node_id(".").is_ok());
}

// An embedded "." component (e.g. "src/./lib.rs") passes the same rules for
// the same reason.  Document the current behaviour explicitly.
#[test]
fn accepts_embedded_dot_component() {
    assert!(validate_node_id("src/./lib.rs").is_ok());
}
