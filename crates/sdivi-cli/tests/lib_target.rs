/// Tests verifying M13's ACP: `sdivi-cli` is exposed as a library target with
/// `pub fn run()`, enabling both `cargo install sdivi-cli` and
/// `cargo install sdivi-rust` to produce a working `sdivi` binary.
use assert_cmd::Command;
use predicates::str::contains;

fn sdivi() -> Command {
    Command::cargo_bin("sdivi").expect("sdivi binary must be built")
}

// ── --help exits 0 and shows expected subcommands ─────────────────────────

#[test]
fn help_flag_exits_zero() {
    sdivi().arg("--help").assert().success();
}

#[test]
fn help_flag_shows_snapshot_subcommand() {
    sdivi()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("snapshot"));
}

#[test]
fn help_flag_shows_check_subcommand() {
    sdivi()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("check"));
}

#[test]
fn help_flag_shows_boundaries_subcommand() {
    sdivi()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("boundaries"));
}

// ── no subcommand prints hint to stderr and exits 0 ───────────────────────

#[test]
fn no_subcommand_exits_zero() {
    sdivi().assert().success();
}

#[test]
fn no_subcommand_prints_hint_to_stderr() {
    sdivi().assert().success().stderr(contains("--help"));
}

// ── sdivi-rust Cargo.toml declares the sdivi binary target ────────────────────
// These tests verify the M13 configuration that makes `cargo install sdivi-rust`
// produce the sdivi binary, by reading the actual Cargo manifest.

#[test]
fn sdivi_rust_cargo_toml_declares_bin_target() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let sdivi_rust_toml = manifest_dir
        .parent()
        .unwrap()
        .join("sdivi-rust")
        .join("Cargo.toml");
    let content = std::fs::read_to_string(&sdivi_rust_toml)
        .unwrap_or_else(|_| panic!("could not read {}", sdivi_rust_toml.display()));
    assert!(
        content.contains("[[bin]]"),
        "sdivi-rust/Cargo.toml must have a [[bin]] section so `cargo install sdivi-rust` works"
    );
    assert!(
        content.contains("name = \"sdivi\""),
        "sdivi-rust/Cargo.toml [[bin]] must be named \"sdivi\""
    );
}

#[test]
fn sdivi_rust_cargo_toml_depends_on_sdivi_cli() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let sdivi_rust_toml = manifest_dir
        .parent()
        .unwrap()
        .join("sdivi-rust")
        .join("Cargo.toml");
    let content = std::fs::read_to_string(&sdivi_rust_toml)
        .unwrap_or_else(|_| panic!("could not read {}", sdivi_rust_toml.display()));
    assert!(
        content.contains("sdivi-cli"),
        "sdivi-rust/Cargo.toml must depend on sdivi-cli to delegate binary logic"
    );
}

#[test]
fn sdivi_cli_cargo_toml_declares_lib_target_via_lib_rs() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_rs = manifest_dir.join("src").join("lib.rs");
    assert!(
        lib_rs.exists(),
        "sdivi-cli/src/lib.rs must exist so the crate has a library target"
    );
}

#[test]
fn sdivi_cli_lib_rs_exports_run_fn() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_rs = manifest_dir.join("src").join("lib.rs");
    let content = std::fs::read_to_string(&lib_rs).expect("sdivi-cli/src/lib.rs must be readable");
    assert!(
        content.contains("pub fn run()"),
        "sdivi-cli/src/lib.rs must export `pub fn run()` for sdivi-rust to delegate to"
    );
}

#[test]
fn sdivi_rust_main_rs_delegates_to_sdivi_cli_run() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let main_rs = manifest_dir
        .parent()
        .unwrap()
        .join("sdivi-rust")
        .join("src")
        .join("main.rs");
    let content = std::fs::read_to_string(&main_rs)
        .unwrap_or_else(|_| panic!("could not read {}", main_rs.display()));
    assert!(
        content.contains("sdivi_cli::run()"),
        "sdivi-rust/src/main.rs must call sdivi_cli::run() to delegate to the library target"
    );
}
