/// Tests verifying M13's ACP: `sdi-cli` is exposed as a library target with
/// `pub fn run()`, enabling both `cargo install sdi-cli` and
/// `cargo install sdi-rust` to produce a working `sdi` binary.
use assert_cmd::Command;
use predicates::str::contains;

fn sdi() -> Command {
    Command::cargo_bin("sdi").expect("sdi binary must be built")
}

// ── --help exits 0 and shows expected subcommands ─────────────────────────

#[test]
fn help_flag_exits_zero() {
    sdi().arg("--help").assert().success();
}

#[test]
fn help_flag_shows_snapshot_subcommand() {
    sdi()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("snapshot"));
}

#[test]
fn help_flag_shows_check_subcommand() {
    sdi()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("check"));
}

#[test]
fn help_flag_shows_boundaries_subcommand() {
    sdi()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("boundaries"));
}

// ── no subcommand prints hint to stderr and exits 0 ───────────────────────

#[test]
fn no_subcommand_exits_zero() {
    sdi().assert().success();
}

#[test]
fn no_subcommand_prints_hint_to_stderr() {
    sdi()
        .assert()
        .success()
        .stderr(contains("--help"));
}

// ── sdi-rust Cargo.toml declares the sdi binary target ────────────────────
// These tests verify the M13 configuration that makes `cargo install sdi-rust`
// produce the sdi binary, by reading the actual Cargo manifest.

#[test]
fn sdi_rust_cargo_toml_declares_bin_target() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let sdi_rust_toml = manifest_dir
        .parent()
        .unwrap()
        .join("sdi-rust")
        .join("Cargo.toml");
    let content = std::fs::read_to_string(&sdi_rust_toml)
        .unwrap_or_else(|_| panic!("could not read {}", sdi_rust_toml.display()));
    assert!(
        content.contains("[[bin]]"),
        "sdi-rust/Cargo.toml must have a [[bin]] section so `cargo install sdi-rust` works"
    );
    assert!(
        content.contains("name = \"sdi\""),
        "sdi-rust/Cargo.toml [[bin]] must be named \"sdi\""
    );
}

#[test]
fn sdi_rust_cargo_toml_depends_on_sdi_cli() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let sdi_rust_toml = manifest_dir
        .parent()
        .unwrap()
        .join("sdi-rust")
        .join("Cargo.toml");
    let content = std::fs::read_to_string(&sdi_rust_toml)
        .unwrap_or_else(|_| panic!("could not read {}", sdi_rust_toml.display()));
    assert!(
        content.contains("sdi-cli"),
        "sdi-rust/Cargo.toml must depend on sdi-cli to delegate binary logic"
    );
}

#[test]
fn sdi_cli_cargo_toml_declares_lib_target_via_lib_rs() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_rs = manifest_dir.join("src").join("lib.rs");
    assert!(
        lib_rs.exists(),
        "sdi-cli/src/lib.rs must exist so the crate has a library target"
    );
}

#[test]
fn sdi_cli_lib_rs_exports_run_fn() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_rs = manifest_dir.join("src").join("lib.rs");
    let content = std::fs::read_to_string(&lib_rs)
        .expect("sdi-cli/src/lib.rs must be readable");
    assert!(
        content.contains("pub fn run()"),
        "sdi-cli/src/lib.rs must export `pub fn run()` for sdi-rust to delegate to"
    );
}

#[test]
fn sdi_rust_main_rs_delegates_to_sdi_cli_run() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let main_rs = manifest_dir
        .parent()
        .unwrap()
        .join("sdi-rust")
        .join("src")
        .join("main.rs");
    let content = std::fs::read_to_string(&main_rs)
        .unwrap_or_else(|_| panic!("could not read {}", main_rs.display()));
    assert!(
        content.contains("sdi_cli::run()"),
        "sdi-rust/src/main.rs must call sdi_cli::run() to delegate to the library target"
    );
}
