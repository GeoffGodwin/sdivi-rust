/// Tests verifying M13 metadata requirements:
/// - Workspace version is 0.1.0
/// - bindings/sdi-wasm/package.json version matches workspace version
/// - All published crates have `readme`, `keywords`, and `categories` fields
/// - Coverage Gap 2 (REVIEWER_REPORT.md): sdi-wasm package.json declares
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
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("could not read {}", path.display()))
}

fn read_crate_toml(crate_name: &str) -> String {
    let path = workspace_root()
        .join("crates")
        .join(crate_name)
        .join("Cargo.toml");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("could not read Cargo.toml for {}", crate_name))
}

// ── Workspace version ─────────────────────────────────────────────────────

#[test]
fn workspace_version_is_0_1_0() {
    let toml = read_workspace_cargo_toml();
    assert!(
        toml.contains("version = \"0.1.0\""),
        "workspace Cargo.toml must set version = \"0.1.0\" for M13"
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
fn wasm_package_json_version_is_0_1_0() {
    let path = workspace_root()
        .join("bindings")
        .join("sdi-wasm")
        .join("package.json");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("could not read {}", path.display()));
    assert!(
        content.contains("\"version\": \"0.1.0\""),
        "sdi-wasm package.json version must be \"0.1.0\" to match workspace"
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
        .join("sdi-wasm")
        .join("package.json");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("could not read {}", path.display()));
    assert!(
        content.contains("sdi_wasm_bg.wasm"),
        "package.json must declare sdi_wasm_bg.wasm in the files array"
    );
}

#[test]
fn wasm_package_json_declares_dts_artifact() {
    let path = workspace_root()
        .join("bindings")
        .join("sdi-wasm")
        .join("package.json");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("could not read {}", path.display()));
    assert!(
        content.contains("sdi_wasm.d.ts"),
        "package.json must declare sdi_wasm.d.ts in the files array"
    );
}

#[test]
fn wasm_package_json_has_types_field() {
    let path = workspace_root()
        .join("bindings")
        .join("sdi-wasm")
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
    "sdi-config",
    "sdi-core",
    "sdi-pipeline",
    "sdi-cli",
    "sdi-parsing",
    "sdi-graph",
    "sdi-detection",
    "sdi-patterns",
    "sdi-snapshot",
    "sdi-rust",
    "sdi-lang-rust",
    "sdi-lang-python",
    "sdi-lang-typescript",
    "sdi-lang-javascript",
    "sdi-lang-go",
    "sdi-lang-java",
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
