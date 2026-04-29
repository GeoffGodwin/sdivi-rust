# Milestone Archive

Completed milestone definitions archived from CLAUDE.md.
See git history for the commit that completed each milestone.

---

## Archived: 2026-04-28 — Unknown Initiative

#### Milestone 1: Workspace Scaffold and `sdi-core` Skeleton
<!-- milestone-meta
id: "01"
status: "done"
-->


**Scope:** Create the Cargo workspace with all crates as empty shells, wire up CI, finalize MSRV, reserve crate names on crates.io with `0.0.0` placeholders, and stand up the `Config` struct and `ExitCode` enum — the two types every other crate depends on. License (Apache 2.0) and `sdi`-name workaround are already ratified during planning; this milestone just executes them. No real analysis logic yet.

**Deliverables:**
- Cargo workspace with `crates/sdi-core`, `crates/sdi-cli`, `crates/sdi-parsing`, `crates/sdi-graph`, `crates/sdi-detection`, `crates/sdi-patterns`, `crates/sdi-snapshot`, `crates/sdi-config`, and the six `sdi-lang-*` adapter crates as compile-but-empty libraries
- `Config` struct in `sdi-config` with `Default`, full schema mirroring DESIGN, and 5-level precedence loader stub returning defaults
- `ExitCode` closed enum in `sdi-core::exit_code` with explicit `i32` discriminants (`Success=0, RuntimeError=1, ConfigError=2, AnalysisError=3, ThresholdExceeded=10`)
- `sdi-cli` builds a `sdi --version` binary
- `LICENSE` (Apache 2.0) and `NOTICE` already in place from planning; verify contents match upstream; every crate's `Cargo.toml` sets `license = "Apache-2.0"`
- `rust-toolchain.toml` pinning MSRV to "stable latest minus 2"
- GitHub Actions: `ci.yml` (clippy, fmt, test on Linux/macOS/Windows × stable/MSRV); `release.yml` skeleton (no publish yet); `audit.yml` weekly
- Crate names reserved on crates.io with empty `0.0.0` placeholders. Names to publish: `sdi-rust` (the install-discovery meta-crate; users `cargo install sdi-rust`), `sdi-core`, `sdi-cli`, `sdi-config`, `sdi-parsing`, `sdi-graph`, `sdi-detection`, `sdi-patterns`, `sdi-snapshot`, `sdi-lang-rust`, `sdi-lang-python`, `sdi-lang-typescript`, `sdi-lang-javascript`, `sdi-lang-go`, `sdi-lang-java`, `sdi-py`, `sdi-node`. **The bare `sdi` is unavailable** (taken by an unrelated DI library); the binary stays `sdi` via `[[bin]] name = "sdi"` in `sdi-cli`'s `Cargo.toml`

**Files to create or modify:**
- `Cargo.toml` (workspace, pinned dep versions with `.workspace = true`)
- `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `deny.toml`
- `LICENSE`, `NOTICE`, `README.md`, `CHANGELOG.md` (all already exist; `Cargo.toml` workspace metadata wires them in)
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
- Crate name re-check before publishing — availabilities were verified 2026-04-28 but crates.io is first-come; re-run `cargo search` against each name immediately before `cargo publish`
- Publish order matters: leaf crates first (`sdi-config`, `sdi-lang-*`), then `sdi-parsing`/`sdi-graph`/`sdi-detection`/`sdi-patterns`/`sdi-snapshot`, then `sdi-core`, then `sdi-cli`, then `sdi-rust` (meta). For empty `0.0.0` placeholders this ordering is cosmetic but for real publishes in m11 it's load-bearing
- crates.io is **append-only** — once `0.0.0` is published it stays; do not panic about needing version bumps later
- MSRV drift: pin a concrete version in `rust-toolchain.toml` and add an MSRV row to the CI matrix
- Every published crate's `Cargo.toml` needs `license = "Apache-2.0"` and a `description` (crates.io rejects publishes without one)
- Workspace `[workspace.dependencies]` block must list every external dep with a pinned version; member crates use `dep.workspace = true`

**Seeds Forward:**
- Every later milestone consumes `Config` and `ExitCode` — their public shape is now load-bearing
- The empty `LanguageAdapter` trait location (`sdi-parsing::adapter`) is the extension point that all adapter crates will implement
- `crates/sdi-cli/src/commands/` is created in Milestone 8; its skeleton lives here as a directory but is not populated
- The CI matrix established here is extended in later milestones (verify-leiden gate added in Milestone 5, release publish in Milestone 11)

---

---

## Archived: 2026-04-28 — Unknown Initiative

#### Milestone 2: Config Loading + Boundary Spec Reader
<!-- milestone-meta
id: "2"
status: "done"
-->


**Scope:** Make `Config::load_or_default` actually walk the 5-level precedence chain and parse TOML. Implement `BoundarySpec` reader from YAML (read-only — write is Milestone 9). Threshold overrides with `expires` validation. Wire `sdi init` so we have a usable command.

**Deliverables:**
- Working `Config::load_or_default(path)` resolving CLI flags > env > project > global > defaults
- TOML parser with structured `ConfigError` variants (`Parse`, `InvalidValue { key, message }`, `MissingExpiresOnOverride { category }`)
- Per-category threshold overrides parsed; missing `expires` errors; expired overrides silently ignored
- Unknown-key deprecation warnings to stderr (never error)
- `BoundarySpec` reader from `.sdi/boundaries.yaml` via `serde_yaml`
- `sdi init` command writes a default `.sdi/config.toml` and detects languages from file extensions
- Env vars wired: `SDI_LOG_LEVEL`, `SDI_WORKERS`, `SDI_CONFIG_PATH`, `SDI_SNAPSHOT_DIR`, `NO_COLOR`

**Files to create or modify:**
- `crates/sdi-config/src/load.rs` (real implementation)
- `crates/sdi-config/src/thresholds.rs`
- `crates/sdi-config/src/boundary.rs`
- `crates/sdi-config/src/error.rs` (extend variants)
- `crates/sdi-cli/src/commands/init.rs`
- `crates/sdi-cli/src/commands/mod.rs` registers `init`

**Acceptance criteria:**
- `sdi init` in an empty repo writes `.sdi/config.toml` matching the DESIGN default block byte-for-byte
- Config with `[thresholds.overrides.foo]` missing `expires` exits with code 2 and a clear error message naming the category
- An expired override (date in past) is loaded without error and behaves as if absent
- `SDI_CONFIG_PATH=/tmp/x.toml sdi init` reads from that path
- Unknown key like `[unknown_section]` produces a stderr deprecation warning and otherwise succeeds
- `sdi-py`'s real `.sdi/config.toml` (taken from the bifl-tracker fixtures) loads cleanly

**Tests:**
- `crates/sdi-config/tests/precedence.rs`: layered configs, env overrides win, CLI overrides env
- `crates/sdi-config/tests/threshold_overrides.rs`: missing `expires` → error; expired → ignored; valid → applied
- `crates/sdi-config/tests/sdi_py_compat.rs`: load fixture configs from sdi-py, assert success
- `crates/sdi-cli/tests/init.rs`: `sdi init` writes the expected file; running twice does not clobber existing config

**Watch For:**
- Date parsing: `expires` is a date string (`"2026-09-30"`). Use `toml::value::Datetime` and validate it parses as a date, not datetime — sdi-py accepts date-only
- `core.exclude` and `patterns.scope_exclude` are **replaced** on override, not merged — easy to get wrong with a default `extend` reducer
- `.sdi/config.toml` must not be overwritten if it already exists (`sdi init` is idempotent in that direction)
- YAML parser cannot preserve comments — explicitly accepted per KDD-6, but test-cover the read path against a sdi-py boundaries.yaml fixture

**Seeds Forward:**
- The `Config` struct is now real and consumed by `Pipeline::new` in Milestone 6
- `BoundarySpec` reader becomes input to snapshot assembly in Milestone 7
- `ConfigError` variants are stable from here; new variants are non-breaking via `#[non_exhaustive]`
- Milestone 9 (`sdi boundaries ratify`) depends on this read path; the comment-loss-on-write decision lives there

---

---

## Archived: 2026-04-29 — Unknown Initiative

#### Milestone 3: Parsing Stage with One Language Adapter (Rust)
<!-- milestone-meta
id: "3"
status: "done"
-->


**Scope:** Stand up the parsing pipeline end-to-end with a single language: Rust itself (dogfood). File walker, `LanguageAdapter` trait, `FeatureRecord` struct, parallel parsing via `rayon`. Enforce the CST-drop ownership invariant. The other five adapters land in Milestone 4.

**Deliverables:**
- `LanguageAdapter` trait in `sdi-parsing::adapter` with methods to parse a file and emit a `FeatureRecord`
- `FeatureRecord` struct: path, imports (Vec<String>), exports, function/class/method signatures, pattern instance handles. `serde::Serialize + Deserialize`
- `parse_repository(&Config, &Path) -> impl Iterator<Item = FeatureRecord>` doing breadth-first stable-sorted walk
- `walkdir` + `ignore` + `globset` honoring `.gitignore` and `core.exclude`
- `rayon` parallel parsing; per-worker grammar instance
- `sdi-lang-rust` crate implementing `LanguageAdapter` with `tree-sitter-rust` linked at compile time behind feature `lang-rust`

**Files to create or modify:**
- `crates/sdi-parsing/src/{adapter.rs,feature_record.rs,walker.rs,parse.rs}`
- `crates/sdi-lang-rust/{Cargo.toml,build.rs,src/lib.rs}`
- `tests/fixtures/simple-rust/` with 5–10 known files (cargo crate skeleton, lib.rs with declared modules, mod files)

**Acceptance criteria:**
- `parse_repository` on `tests/fixtures/simple-rust/` returns the same `Vec<FeatureRecord>` (after sorting) on every run
- The fixture has known import counts; assertion in test
- Memory invariant: a test that parses a 1MB Rust file and asserts peak heap stays bounded (use a `tracking-allocator` or count `Tree` allocations via a feature-gated counter)
- Parsing on an empty directory returns zero records, no error
- `core.exclude` glob suppresses files; `.gitignore` is honored

**Tests:**
- `crates/sdi-parsing/tests/walk_ordering.rs`: walk twice, assert identical paths
- `crates/sdi-parsing/tests/memory_invariant.rs`: parse 100 large files, assert no `Tree` survives across files (use a feature-gated `Drop` counter on a wrapper type around `tree_sitter::Tree`)
- `tests/full_pipeline.rs` (top-level): parse fixture, assert `FeatureRecord` count matches a hand-counted constant
- Property test in `crates/sdi-parsing/tests/proptest.rs`: random file content → parse never panics

**Watch For:**
- The parsing API must consume `String` (or `Vec<u8>`) by value and the returned `FeatureRecord` must own no reference into the input — otherwise the CST-drop invariant becomes a lifetime puzzle
- `tree-sitter` grammar instances are not `Send` in some grammar versions; verify before using `rayon::par_iter`. Fall back to per-worker `thread_local!` grammars if needed
- Stable-sort the file list **before** parallelizing; otherwise rayon's internal scheduling can leak ordering nondeterminism into downstream stages
- `walkdir` + `ignore` interaction: use the `ignore` crate's `WalkBuilder` rather than composing manually — `.gitignore` semantics are subtle

**Seeds Forward:**
- The `LanguageAdapter` trait is stable from here. Milestone 4 adds five adapters that implement it without changing the trait
- `FeatureRecord` is the input to Milestone 5 (graph) and Milestone 6 (patterns) — its shape must accommodate both. Pattern instance handles must include enough metadata for the patterns stage without reparsing
- The deterministic walk order is a load-bearing assumption for snapshot bit-stability

---
