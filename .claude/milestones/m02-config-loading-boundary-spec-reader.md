#### Milestone 2: Config Loading + Boundary Spec Reader

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
