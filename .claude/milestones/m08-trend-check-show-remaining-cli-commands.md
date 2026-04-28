#### Milestone 8: Trend, Check, Show — Remaining CLI Commands

**Scope:** The four remaining CLI commands — `trend`, `check`, `show`, plus the boundaries top-level (subcommands in Milestone 9). Wire the threshold-exceeded exit-10 contract. Polish stdout/stderr discipline and JSON output shape.

**Deliverables:**
- `sdi trend [--last N]` aggregating across stored snapshots
- `sdi check` — exit `10` if any threshold exceeded, `0` otherwise; honors per-category overrides with `expires`
- `sdi show [<id>] [--format json|text]` inspects a snapshot
- `sdi boundaries` parent command with subcommand stubs (`infer`, `ratify`, `show`)
- Text formatter using `ratatui` for tables and `owo-colors`/`anstream` for color (auto-detected via `Config::output.color` and `NO_COLOR`)
- JSON formatter producing schema-stable output

**Files to create or modify:**
- `crates/sdi-snapshot/src/trend.rs`
- `crates/sdi-cli/src/commands/{trend.rs,check.rs,show.rs,boundaries.rs}`
- `crates/sdi-cli/src/output/{json.rs,text.rs}`
- `crates/sdi-cli/src/logging.rs` (tracing-subscriber → stderr)

**Acceptance criteria:**
- `sdi check` exits `0` on a fresh snapshot below thresholds; `10` when a threshold-exceeded condition is met
- An expired threshold override is silently ignored — `sdi check` uses defaults after expiry
- `sdi show --format json | jq '.snapshot_version'` returns `"1.0"` (no stderr contamination on stdout)
- `NO_COLOR=1 sdi show` produces no ANSI escape codes
- `sdi trend --last 5` aggregates across the 5 most recent snapshots
- Logs from `tracing` go to stderr regardless of format

**Tests:**
- `crates/sdi-cli/tests/exit_codes.rs`: full matrix of exit codes 0/1/2/3/10
- `crates/sdi-cli/tests/stdout_stderr_split.rs`: redirect each to a file and assert schemas
- `crates/sdi-cli/tests/check_thresholds.rs`: synthetic snapshots driving every threshold variant
- `crates/sdi-cli/tests/show_format.rs`: `--format json` validates against a JSON schema
- `crates/sdi-cli/tests/no_color.rs`: `NO_COLOR=1` and `--no-color` both suppress color

**Watch For:**
- `sdi check` is the **only** command that may exit `10` — any other command emitting `10` is a bug
- The text formatter must not block JSON consumers: use `tokio`-free synchronous `ratatui` rendering directly to a `Vec<u8>` buffer, then write to stdout — do **not** initialize a TUI mode (no alternate screen, no raw mode)
- Threshold check must consult overrides per-category and check expiry against today's date — pull date from `chrono` or `time`, but use the same crate everywhere to avoid TZ drift
- `sdi trend` on fewer than 2 snapshots: print a friendly "not enough snapshots" message and exit 0

**Seeds Forward:**
- The `sdi check` exit-10 path is the public CI gate contract. Any future threshold (e.g., per-category) must continue exiting 10
- The text formatter shape is reused by `sdi boundaries show` in Milestone 9
- JSON output shape is the contract embedders rely on; future milestones cannot break it without a snapshot version bump

---
