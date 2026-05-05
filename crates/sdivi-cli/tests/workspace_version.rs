/// Tests verifying M13 metadata requirements:
/// - Workspace `[workspace.package].version` is set to a 0.1.x semver
/// - bindings/sdivi-wasm/pkg-template/package.json version matches the workspace version
/// - All published crates have `readme`, `keywords`, and `categories` fields
/// - Coverage Gap 2 (REVIEWER_REPORT.md): sdivi-wasm package.json declares
///   the expected artifact files (.wasm, .d.ts) so the npm dry-run is
///   exercising a well-formed package.

fn workspace_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn read_workspace_cargo_toml() -> String {
    let path = workspace_root().join("Cargo.toml");
    std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("could not read {}", path.display()))
}

fn read_crate_toml(crate_name: &str) -> String {
    let path = workspace_root()
        .join("crates")
        .join(crate_name)
        .join("Cargo.toml");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("could not read Cargo.toml for {}", crate_name))
}

/// Extracts the value of the first `version = "x.y.z"` line in `[workspace.package]`.
fn workspace_package_version() -> String {
    let toml = read_workspace_cargo_toml();
    let after_section = toml
        .split("[workspace.package]")
        .nth(1)
        .expect("workspace Cargo.toml must contain a [workspace.package] section");
    let line = after_section
        .lines()
        .find(|l| l.trim_start().starts_with("version"))
        .expect("[workspace.package] must declare a version");
    line.split('"')
        .nth(1)
        .expect("version line must be of the form: version = \"x.y.z\"")
        .to_string()
}

/// Extracts the value of the first `"version": "x.y.z"` field in package.json.
fn wasm_package_json_version() -> String {
    let path = workspace_root()
        .join("bindings")
        .join("sdivi-wasm")
        .join("pkg-template")
        .join("package.json");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("could not read {}", path.display()));
    let line = content
        .lines()
        .find(|l| l.trim_start().starts_with("\"version\""))
        .expect("package.json must declare a version");
    line.split('"')
        .nth(3)
        .expect("version line must be of the form: \"version\": \"x.y.z\"")
        .to_string()
}

// ── Workspace version ─────────────────────────────────────────────────────

#[test]
fn workspace_version_is_v0_semver() {
    let v = workspace_package_version();
    assert!(
        v.starts_with("0."),
        "workspace [workspace.package] version must be a 0.x.y release pre-1.0 (got {v:?})"
    );
}

// ── Release profile flags ─────────────────────────────────────────────────

#[test]
fn release_profile_has_thin_lto() {
    let toml = read_workspace_cargo_toml();
    assert!(
        toml.contains("lto = \"thin\""),
        "release profile must set lto = \"thin\" for smaller binaries"
    );
}

#[test]
fn release_profile_has_strip_true() {
    let toml = read_workspace_cargo_toml();
    assert!(
        toml.contains("strip = true"),
        "release profile must set strip = true for smaller binaries"
    );
}

#[test]
fn release_profile_has_abort_on_panic() {
    let toml = read_workspace_cargo_toml();
    assert!(
        toml.contains("panic = \"abort\""),
        "release profile must set panic = \"abort\""
    );
}

// ── WASM package.json version alignment (Coverage Gap 2) ─────────────────

#[test]
fn wasm_package_json_version_matches_workspace() {
    let workspace_v = workspace_package_version();
    let wasm_v = wasm_package_json_version();
    assert_eq!(
        workspace_v, wasm_v,
        "sdivi-wasm package.json version ({wasm_v}) must match workspace version ({workspace_v}) — bump them together at release time"
    );
}

/// Coverage Gap 2: package.json declares the expected WASM artifact files so
/// the pre-publish `ls -lh pkg/` check would find them after a successful
/// wasm-pack build.  This test verifies the manifest is correctly configured
/// rather than relying on the CI step surfacing a silent misconfiguration.
#[test]
fn wasm_package_json_declares_wasm_artifact() {
    let path = workspace_root()
        .join("bindings")
        .join("sdivi-wasm")
        .join("pkg-template")
        .join("package.json");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("could not read {}", path.display()));
    // Post-M26 the manifest ships per-target directories (`bundler/`, `node/`)
    // rather than naming the wasm binary explicitly; the .wasm file is
    // included transitively via the `bundler/` entry.
    assert!(
        content.contains("\"bundler/\""),
        "pkg-template/package.json must list \"bundler/\" in the files array so the wasm binary ships"
    );
}

#[test]
fn wasm_package_json_declares_dts_artifact() {
    let path = workspace_root()
        .join("bindings")
        .join("sdivi-wasm")
        .join("pkg-template")
        .join("package.json");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("could not read {}", path.display()));
    assert!(
        content.contains("sdivi_wasm.d.ts"),
        "package.json must declare sdivi_wasm.d.ts in the files array"
    );
}

#[test]
fn wasm_package_json_has_types_field() {
    let path = workspace_root()
        .join("bindings")
        .join("sdivi-wasm")
        .join("pkg-template")
        .join("package.json");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("could not read {}", path.display()));
    assert!(
        content.contains("\"types\""),
        "package.json must have a \"types\" field pointing to the .d.ts file"
    );
}

// ── Published crate metadata completeness ────────────────────────────────

const PUBLISHED_CRATES: &[&str] = &[
    "sdivi-config",
    "sdivi-core",
    "sdivi-pipeline",
    "sdivi-cli",
    "sdivi-parsing",
    "sdivi-graph",
    "sdivi-detection",
    "sdivi-patterns",
    "sdivi-snapshot",
    "sdivi-rust",
    "sdivi-lang-rust",
    "sdivi-lang-python",
    "sdivi-lang-typescript",
    "sdivi-lang-javascript",
    "sdivi-lang-go",
    "sdivi-lang-java",
];

#[test]
fn all_published_crates_have_readme_field() {
    for crate_name in PUBLISHED_CRATES {
        let toml = read_crate_toml(crate_name);
        assert!(
            toml.contains("readme ="),
            "crate {} Cargo.toml must have a `readme` field for crates.io display",
            crate_name
        );
    }
}

#[test]
fn all_published_crates_have_keywords_field() {
    for crate_name in PUBLISHED_CRATES {
        let toml = read_crate_toml(crate_name);
        assert!(
            toml.contains("keywords ="),
            "crate {} Cargo.toml must have a `keywords` field for crates.io discoverability",
            crate_name
        );
    }
}

#[test]
fn all_published_crates_have_categories_field() {
    for crate_name in PUBLISHED_CRATES {
        let toml = read_crate_toml(crate_name);
        assert!(
            toml.contains("categories ="),
            "crate {} Cargo.toml must have a `categories` field for crates.io discoverability",
            crate_name
        );
    }
}

#[test]
fn all_published_crates_have_description_field() {
    for crate_name in PUBLISHED_CRATES {
        let toml = read_crate_toml(crate_name);
        assert!(
            toml.contains("description ="),
            "crate {} Cargo.toml must have a `description` field for crates.io",
            crate_name
        );
    }
}

#[test]
fn all_published_crates_readme_files_exist() {
    for crate_name in PUBLISHED_CRATES {
        let readme = workspace_root()
            .join("crates")
            .join(crate_name)
            .join("README.md");
        assert!(
            readme.exists(),
            "crate {} must have a README.md at {} (required by the readme = field)",
            crate_name,
            readme.display()
        );
    }
}
