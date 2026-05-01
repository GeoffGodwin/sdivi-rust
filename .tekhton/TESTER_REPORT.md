## Planned Tests
- [x] `crates/sdi-cli/tests/version.rs` — replace hardcoded `"0.0.11"` with `env!("CARGO_PKG_VERSION")` to avoid recurring stale-string failures on version bumps
- [x] `crates/sdi-cli/tests/exit_codes.rs` — verify exit codes for all M09 commands (init/snapshot/check/trend/show/diff/boundaries)
- [x] `crates/sdi-cli/tests/check_thresholds.rs` — check command threshold and write-mode behavior
- [x] `crates/sdi-cli/tests/stdout_stderr_split.rs` — stdout/stderr separation for JSON output
- [x] `crates/sdi-cli/tests/show_format.rs` — show command JSON and text formatting
- [x] `crates/sdi-cli/tests/trend_format.rs` — trend command output formatting and --last clamping
- [x] `crates/sdi-cli/tests/boundaries_stub.rs` — boundaries subcommands exit 0 and write only to stderr
- [x] `crates/sdi-cli/tests/no_color.rs` — NO_COLOR=1 suppresses ANSI codes in show/check/trend

## Test Run Results
Passed: 59  Failed: 0

## Bugs Found
None

## Files Modified
- [x] `crates/sdi-cli/tests/version.rs`
- [x] `crates/sdi-cli/tests/exit_codes.rs`
- [x] `crates/sdi-cli/tests/check_thresholds.rs`
- [x] `crates/sdi-cli/tests/stdout_stderr_split.rs`
- [x] `crates/sdi-cli/tests/show_format.rs`
- [x] `crates/sdi-cli/tests/trend_format.rs`
- [x] `crates/sdi-cli/tests/boundaries_stub.rs`
- [x] `crates/sdi-cli/tests/no_color.rs`
