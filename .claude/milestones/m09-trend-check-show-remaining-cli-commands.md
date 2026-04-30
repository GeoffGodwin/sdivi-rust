#### Milestone 9: Trend, Check, Show — Remaining CLI Commands

**Scope:** The four remaining CLI commands — `trend`, `check`, `show`, plus the `boundaries` parent (subcommands in Milestone 10). Wire the threshold-exceeded exit-10 contract through the pure `sdi-core::compute_thresholds_check` function delivered in M08. Polish stdout/stderr discipline and JSON output shape.

**Deliverables:**
- `sdi trend [--last N]` aggregating across stored snapshots — calls `sdi_core::compute_trend` (pure) and `sdi_pipeline` for the snapshot-store read. With fewer than 2 snapshots, prints `"sdi trend: not enough snapshots (need ≥2)"` to stderr and exits 0. When `N > stored_count`, silently uses what's available (no error).
- `sdi check` — captures a fresh snapshot, compares it to the most recent stored prior, and routes through `sdi_core::compute_thresholds_check` for the exit decision. Exits `10` if any threshold exceeded, `0` otherwise. **First-run case** (no prior snapshot): `compute_thresholds_check` receives a null `DivergenceSummary` and returns `exit_code = 0` and `exceeded = []` — first-run check is always green, by design (Critical System Rule 5). Today's date is populated into `cfg.today: NaiveDate` by the CLI from `chrono::Local::now().date_naive()` before the call (clock read happens in CLI, not core). **Flag:** `--no-write` skips writing the freshly-captured snapshot to `.sdi/snapshots/` — useful for CI gates that don't want to pollute history. With `--no-write`, retention is also not enforced. Default behavior writes the snapshot (matching `sdi snapshot` semantics).
- `sdi show [<id>] [--format json|text]` inspects a snapshot. `<id>` is optional; with no argument, the **latest snapshot** by lexicographic filename order (= chronological, per the M07 file-naming scheme `snapshot_<YYYYMMDDTHHMMSS>_<hash>.json`) is shown. With `--format json`, output is the raw `Snapshot` JSON on stdout (so `sdi show --format json | jq '.snapshot_version'` returns `"1.0"` per the existing acceptance test).
- `sdi boundaries` parent command with subcommand stubs (`infer`, `ratify`, `show`). Each stub prints `"sdi boundaries <subcmd>: not implemented until M10"` to stderr and exits **0** — keeps CI scripts that survey command help working. M10 fills them in.
- Text formatter using `ratatui` for tables and `owo-colors`/`anstream` for color (auto-detected via `Config::output.color` and `NO_COLOR`)
- JSON formatter producing schema-stable output:
  - `sdi show --format json` → raw `Snapshot` JSON
  - `sdi check --format json` → `ThresholdCheckResult` from `sdi-core` (`{ exit_code, exceeded: [string], summary: DivergenceSummary, applied_overrides: { ... } }`); the CLI process exit code still tracks `result.exit_code`, JSON is informational
  - `sdi trend --format json` → `TrendResult` from `sdi-snapshot`
  - `sdi diff --format json` → `DivergenceSummary` (already in M07's shape)

**Files to create or modify:**
- **New:** `crates/sdi-cli/src/commands/{trend.rs,check.rs,show.rs,boundaries.rs}`
- **Modify:** `crates/sdi-cli/src/main.rs` — register the four new subcommands in the `Commands` enum and dispatch in `main`. `Check` carries `--no-write: bool` and `--format: String`. `Trend` carries `--last: Option<usize>` and `--format: String`. `Show` carries `<id>: Option<String>` and `--format: String`. `Boundaries` carries a nested subcommand enum with `Infer`, `Ratify`, `Show` variants (stubs for M09).
- **Modify:** `crates/sdi-cli/src/output/{json.rs,text.rs}` — extend with formatters for `TrendResult`, `ThresholdCheckResult`, and the standalone `Snapshot` show path. JSON path is a thin `serde_json::to_string_pretty` per result struct; text path uses `ratatui` rendering to a `Vec<u8>` buffer (no alternate-screen / no raw-mode TUI).
- **No-op for `crates/sdi-cli/src/logging.rs`** — already exists from M01 (tracing-subscriber → stderr at `warn` default). Touch only if `--verbose` flag is added to `sdi check`; not needed for the M09 acceptance criteria.
- **Extend:** `crates/sdi-pipeline/src/store.rs` — add `read_snapshots(dir: &Path) -> std::io::Result<Vec<Snapshot>>` returning chronologically-ordered (oldest→newest) snapshots and `read_snapshot_by_id(dir: &Path, id: &str) -> std::io::Result<Snapshot>` for `sdi show <id>`. Add `latest_snapshot(dir: &Path) -> std::io::Result<Option<Snapshot>>` for `sdi check` and `sdi show` with no id.
- **Extend:** `crates/sdi-pipeline/src/pipeline.rs` — `Pipeline::snapshot` gains an internal `WriteMode::{Persist, EphemeralForCheck}` enum so `sdi check --no-write` can capture without touching `.sdi/snapshots/`. Default callers stay on `Persist`.

**Acceptance criteria:**
- `sdi check` exits `0` on a fresh snapshot below thresholds; `10` when any threshold is exceeded. Exit logic is a thin wrapper around `compute_thresholds_check`'s `ThresholdCheckResult.exit_code`.
- **First-run `sdi check` exits `0`** (no prior snapshot → null `DivergenceSummary` → `compute_thresholds_check` returns `exit_code = 0` with `exceeded = []`). Asserted in `crates/sdi-cli/tests/exit_codes.rs`.
- `sdi check --no-write` does not create a file in `.sdi/snapshots/`. Asserted by counting the snapshot directory before/after.
- An expired threshold override is silently ignored — `sdi check` uses defaults after expiry. Expiry comparison happens inside `compute_thresholds_check` against the `today` argument supplied by the CLI; the config retains the override block as data (no load-time pruning).
- `sdi show` with no `<id>` prints the latest snapshot (lexicographically last `snapshot_*.json` in `.sdi/snapshots/`).
- `sdi show --format json | jq '.snapshot_version'` returns `"1.0"` (no stderr contamination on stdout).
- `sdi check --format json | jq '.exit_code'` returns the integer exit code (and the process itself exits with the same value).
- `NO_COLOR=1 sdi show` produces no ANSI escape codes. `--no-color` is equivalent.
- `sdi trend --last 5` aggregates across the 5 most recent snapshots. `--last 9999` against a directory with 3 snapshots silently uses 3 (no error).
- `sdi trend` with 0 or 1 snapshot prints the friendly "not enough snapshots" message to stderr and exits `0`.
- `sdi boundaries infer|ratify|show` each prints "not implemented until M10" to stderr and exits `0`.
- Logs from `tracing` go to stderr regardless of format.

**Tests:**
- `crates/sdi-cli/tests/exit_codes.rs`: full matrix of exit codes 0/1/2/3/10. Includes first-run `sdi check` (exit 0), expired-override `sdi check` (exit 0), threshold-exceeded `sdi check` (exit 10), bad-config `sdi check` (exit 2).
- `crates/sdi-cli/tests/stdout_stderr_split.rs`: redirect each stream to a file; assert JSON validity on stdout for every `--format json` command and zero JSON contamination on stderr.
- `crates/sdi-cli/tests/check_thresholds.rs`: synthetic snapshots driving every threshold variant (pattern_entropy, convention_drift, coupling_delta, boundary_violation); verify CLI exit code matches `compute_thresholds_check` programmatically. Includes a `--no-write` assertion (snapshot dir count unchanged).
- `crates/sdi-cli/tests/show_format.rs`: `sdi show --format json` parses as `Snapshot`; `sdi show` with no id selects latest; `sdi show <id>` selects specifically.
- `crates/sdi-cli/tests/trend_format.rs`: `sdi trend --last N` clamps to available; <2 snapshots → friendly stderr message + exit 0; `--format json` parses as `TrendResult`.
- `crates/sdi-cli/tests/boundaries_stub.rs`: each of `sdi boundaries {infer, ratify, show}` exits 0 and writes to stderr only.
- `crates/sdi-cli/tests/no_color.rs`: `NO_COLOR=1` and `--no-color` both suppress color across show / check / trend.

**Watch For:**
- `sdi check` is the **only** command that may exit `10` — any other command emitting `10` is a bug.
- The text formatter must not block JSON consumers: use `tokio`-free synchronous `ratatui` rendering directly to a `Vec<u8>` buffer, then write to stdout — do **not** initialize a TUI mode (no alternate screen, no raw mode).
- `compute_thresholds_check` is the source of truth for exit logic. Any new threshold/override semantics land in `sdi-core`, not in CLI flags. CLI is presentation only.
- Threshold check consults overrides per-category and checks expiry against `cfg.today`. CLI pulls the date from `chrono::Local::now().date_naive()` and writes it into `ThresholdsInput::today` before calling `compute_thresholds_check`. The pure function never reads the clock.
- `sdi check` writes a snapshot by default; `--no-write` skips the write **and** the retention enforcement. CI gates that don't want history pollution use `--no-write`.
- `sdi show` with no id picks lexicographic-last from `.sdi/snapshots/snapshot_*.json` — relies on the M07 file-naming scheme. If the scheme ever changes, this default-selection logic must change with it.
- `sdi trend` on fewer than 2 snapshots: friendly stderr message + exit 0. `--last N` larger than the stored count is silently clamped (no error).
- The `sdi boundaries` subcommands are M09 stubs that exit 0 — do **not** wire to `sdi_core::infer_boundaries` here; that's M10. Premature wiring couples this milestone to M10's YAML-write path and breaks the parallel-development plan.

**Seeds Forward:**
- The `sdi check` exit-10 path is the public CI gate contract. Any future threshold (e.g., per-category) must continue exiting 10 and route through `compute_thresholds_check`.
- The text formatter shape is reused by `sdi boundaries show` in M10.
- JSON output shape is the contract embedders rely on; future milestones cannot break it without a snapshot-version bump. The four JSON shapes (`Snapshot`, `ThresholdCheckResult`, `TrendResult`, `DivergenceSummary`) are documented in `docs/cli-integration.md` (M11).
- The consumer app invokes `compute_thresholds_check` directly via WASM (M12); the CLI is one of three callers (CLI / Rust embedders / consumer app via WASM) — keep the function's input shape stable.
- `WriteMode::EphemeralForCheck` introduced for `--no-write` is the seam any future "dry-run snapshot" feature reuses. Don't add a second seam.

---
