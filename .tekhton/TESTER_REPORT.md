## Audit Rework
- [x] Fixed: HIGH COVERAGE — added `check_exits_ten_when_threshold_breached` to `exit_codes.rs`; uses `coupling_delta_rate = -1.0` to guarantee breach; asserts `.code(10)` and `exceeded` non-empty in JSON
- [x] Fixed: MEDIUM COVERAGE — added `config_error_exits_two` to `exit_codes.rs`; threshold override block without `expires` triggers `ConfigError::MissingExpiresOnOverride` → exit 2
- [x] Fixed: MEDIUM COVERAGE — added `snapshot_exits_zero_on_all_unknown_languages` to `exit_codes.rs`; asserts current behavior (exit 0); see BUG below — Rule 15 / System Rule 7 require exit 3 but `PipelineError::NoGrammarsAvailable` is never emitted and `error_exit_code` has no downcast for it
- [x] Fixed: MEDIUM COVERAGE — removed three check tests from `exit_codes.rs` that duplicated `check_thresholds.rs` (`check_first_run_exits_zero`, `check_no_write_exits_zero_and_creates_no_snapshot`, `check_below_thresholds_exits_zero`); `exit_codes.rs` now focuses solely on numeric exit code contracts
- [x] Fixed: MEDIUM NAMING — renamed `no_color_flag_suppresses_ansi_in_show` → `default_show_output_has_no_ansi_codes` in `no_color.rs`; added guidance comment for when formatters gain colour support
- [x] Fixed: LOW NAMING — replaced `.failure()` with `.code(1)` in `show_no_snapshots_exits_one` and `diff_nonexistent_file_exits_one` in `exit_codes.rs`
- [x] Fixed: LOW COVERAGE — replaced `parsed.get("coupling_slope").is_some()` with `parsed["coupling_slope"].as_f64().is_some()` in `trend_format.rs`; now catches `null` values that the former assertion missed
- [x] Deferred: LOW INTEGRITY — `minimal_snapshot_json` helper in `boundaries_comment_loss_warning.rs` uses hand-crafted JSON; the test currently passes and the fragility is forward-looking; extracting `write_fake_snapshot` from `boundaries.rs` to a shared utility is non-trivial without touching implementation code — deferred to a future refactor pass
- [x] Deferred: LOW EXERCISE — `path_partition_entry_count_matches_graph_node_count` in `path_partition.rs` conflates two independent counts; adding a fixture with non-ASCII paths requires a new fixture crate or in-test file creation; deferred because the current test is correct for the `simple-rust` fixture and the UTF-8 filtering unit test is an implementation concern

## Planned Tests
- [x] `crates/sdi-cli/tests/exit_codes.rs` — snapshot exits zero when all extensioned files are inside .sdi/ (not exit 3; .sdi/-exclusion branch)
- [x] `crates/sdi-cli/tests/version.rs` — replace hardcoded `"0.0.11"` with `env!("CARGO_PKG_VERSION")` to avoid recurring stale-string failures on version bumps
- [x] `crates/sdi-cli/tests/exit_codes.rs` — verify exit codes for all M09 commands (init/snapshot/check/trend/show/diff/boundaries)
- [x] `crates/sdi-cli/tests/check_thresholds.rs` — check command threshold and write-mode behavior
- [x] `crates/sdi-cli/tests/stdout_stderr_split.rs` — stdout/stderr separation for JSON output
- [x] `crates/sdi-cli/tests/show_format.rs` — show command JSON and text formatting
- [x] `crates/sdi-cli/tests/trend_format.rs` — trend command output formatting and --last clamping
- [x] `crates/sdi-cli/tests/boundaries_stub.rs` — boundaries subcommands exit 0 and write only to stderr
- [x] `crates/sdi-cli/tests/no_color.rs` — NO_COLOR=1 suppresses ANSI codes in show/check/trend
- [x] `crates/sdi-pipeline/tests/write_boundary_spec.rs` — write_boundary_spec: new file creation, existing file with `#` comments is overwritten, parent dir created
- [x] `crates/sdi-cli/tests/boundaries_comment_loss_warning.rs` — CLI: ratify emits stderr warning when pre-existing boundaries.yaml has YAML `#` comments (store.rs:108-128 warning path)
- [x] `crates/sdi-pipeline/tests/path_partition.rs` — indirect coverage of private compute_path_partition: pipeline on real fixture produces non-empty path_partition with valid path keys and u32 community values

## Test Run Results
Passed: 580  Failed: 0

## Bugs Found
- BUG: [crates/sdi-pipeline/src/pipeline.rs:135] `parse_repository` returns `Vec<FeatureRecord>` unconditionally; `PipelineError::NoGrammarsAvailable` is defined in `sdi_pipeline::error` but never emitted — Rule 15 / System Rule 7 (`exit 3` when all detected languages lack grammars) is unreachable via the CLI
- BUG: [crates/sdi-cli/src/main.rs:148-153] `error_exit_code` only downcasts `sdi_config::ConfigError`; a `PipelineError::NoGrammarsAvailable` wrapped in `anyhow` would be mapped to `ExitCode::RuntimeError` (1), not `ExitCode::AnalysisError` (3), even if it were ever produced

## Files Modified
- [x] `crates/sdi-cli/tests/version.rs`
- [x] `crates/sdi-cli/tests/exit_codes.rs`
- [x] `crates/sdi-cli/tests/check_thresholds.rs`
- [x] `crates/sdi-cli/tests/stdout_stderr_split.rs`
- [x] `crates/sdi-cli/tests/show_format.rs`
- [x] `crates/sdi-cli/tests/trend_format.rs`
- [x] `crates/sdi-cli/tests/boundaries_stub.rs`
- [x] `crates/sdi-cli/tests/no_color.rs`
- [x] `crates/sdi-pipeline/tests/write_boundary_spec.rs`
- [x] `crates/sdi-cli/tests/boundaries_comment_loss_warning.rs`
- [x] `crates/sdi-pipeline/tests/path_partition.rs`

### Milestone 11 Coverage Gap
- [x] `crates/sdi-cli/tests/exit_codes.rs` — added `snapshot_exits_zero_when_all_extensioned_files_are_inside_sdi_dir`

### Audit Rework — Additional Modified Files
- [x] `crates/sdi-cli/tests/exit_codes.rs` — added exit-10/2 tests; removed 3 duplicates; .code(1) for show/diff
- [x] `crates/sdi-cli/tests/no_color.rs` — renamed `no_color_flag_suppresses_ansi_in_show`
- [x] `crates/sdi-cli/tests/trend_format.rs` — strengthened coupling_slope assertion

## Timing
- Test executions: 1
- Approximate total test execution time: 83s
- Test files written: 0
