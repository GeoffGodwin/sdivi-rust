#### Milestone 1: Workspace Scaffold and `sdi-core` Skeleton

**Scope:** Create the Cargo workspace with all crates as empty shells, wire up CI, settle license and MSRV. Stand up the `Config` struct and `ExitCode` enum — the two types every other crate depends on. Reserve crate names on crates.io. No real analysis logic yet.

**Deliverables:**
- Cargo workspace with `crates/sdi-core`, `crates/sdi-cli`, `crates/sdi-parsing`, `crates/sdi-graph`, `crates/sdi-detection`, `crates/sdi-patterns`, `crates/sdi-snapshot`, `crates/sdi-config`, and the six `sdi-lang-*` adapter crates as compile-but-empty libraries
- `Config` struct in `sdi-config` with `Default`, full schema mirroring DESIGN, and 5-level precedence loader stub returning defaults
- `ExitCode` closed enum in `sdi-core::exit_code` with explicit `i32` discriminants (`Success=0, RuntimeError=1, ConfigError=2, AnalysisError=3, ThresholdExceeded=10`)
- `sdi-cli` builds a `sdi --version` binary
- `LICENSE` set to Apache 2.0 (replacing initial MIT commit)
- `rust-toolchain.toml` pinning MSRV to "stable latest minus 2"
- GitHub Actions: `ci.yml` (clippy, fmt, test on Linux/macOS/Windows × stable/MSRV); `release.yml` skeleton (no publish yet); `audit.yml` weekly
- Crate names reserved on crates.io with empty 0.0.0 placeholders if needed

**Files to create or modify:**
- `Cargo.toml` (workspace, pinned dep versions with `.workspace = true`)
- `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `deny.toml`
- `LICENSE`, `README.md`, `CHANGELOG.md`
- `crates/sdi-core/{Cargo.toml,src/lib.rs,src/exit_code.rs,src/error.rs}`
- `crates/sdi-cli/{Cargo.toml,src/main.rs}`
- `crates/sdi-config/{Cargo.toml,src/lib.rs,src/config.rs,src/load.rs,src/error.rs}`
- Empty `crates/sdi-{parsing,graph,detection,patterns,snapshot}/{Cargo.toml,src/lib.rs}`
- Empty `crates/sdi-lang-{python,typescript,javascript,go,java,rust}/{Cargo.toml,src/lib.rs}`
- `.github/workflows/{ci.yml,release.yml,audit.yml}`

**Acceptance criteria:**
- `cargo build --workspace` succeeds on Linux, macOS, Windows
- `cargo build -p sdi-cli --release` produces an `sdi` binary
- `sdi --version` prints the version from `Cargo.toml`
- `cargo clippy --workspace -- -D warnings` is clean
- `cargo fmt --all --check` is clean
- `Config::default()` returns a struct matching every default in DESIGN's complete config block
- `ExitCode::Success as i32 == 0`, etc., for all five variants
- CI runs green on push and PR

**Tests:**
- `crates/sdi-config/tests/defaults.rs`: assert every field of `Config::default()` matches DESIGN
- `crates/sdi-core/tests/exit_code_contract.rs`: assert each variant casts to its documented `i32`
- `crates/sdi-cli/tests/version.rs`: `assert_cmd::Command::cargo_bin("sdi").arg("--version")` succeeds

**Watch For:**
- Crate name conflicts on crates.io — check `sdi`, `sdi-core`, `sdi-cli` immediately; fall back to `sdi-rs-*` prefix and document if any taken
- MSRV drift: pin a concrete version in `rust-toolchain.toml` and add an MSRV row to the CI matrix
- LICENSE replacement must update the SPDX header in every published crate's `Cargo.toml` (`license = "Apache-2.0"`)
- Workspace `[workspace.dependencies]` block must list every external dep with a pinned version; member crates use `dep.workspace = true`

**Seeds Forward:**
- Every later milestone consumes `Config` and `ExitCode` — their public shape is now load-bearing
- The empty `LanguageAdapter` trait location (`sdi-parsing::adapter`) is the extension point that all adapter crates will implement
- `crates/sdi-cli/src/commands/` is created in Milestone 8; its skeleton lives here as a directory but is not populated
- The CI matrix established here is extended in later milestones (verify-leiden gate added in Milestone 5, release publish in Milestone 11)

---
