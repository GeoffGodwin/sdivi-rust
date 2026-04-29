# Reviewer Report — Expedited Architect Remediation — 2026-04-29

## Verdict
APPROVED_WITH_NOTES

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes

- **Merged doc comment on `project_config_path` / missing doc on `load_or_default`
  (`crates/sdi-config/src/load.rs:17–58`)**
  The `load_or_default` doc comment (lines 17–37) and the `project_config_path` doc
  comment (lines 38–50) are consecutive `///` lines with no blank non-`///` separator
  between them. Rust attaches the entire block to `project_config_path` (line 51);
  `load_or_default` (line 58) receives no doc comment. The generated rustdoc for
  `project_config_path` will open with "Load configuration for the repository rooted
  at `repo_root`." — the wrong description. `load_or_default` will appear undocumented.
  Not a build failure because `sdi-config` does not carry `#![deny(missing_docs)]`,
  and both embedded doc-test examples are valid code that will pass `cargo test --doc`.
  Fix: insert a blank non-`///` line between the two doc blocks and move the
  `load_or_default` doc comment to immediately precede that function at line 58.

## Coverage Gaps
- None

## ACP Verdicts
(No Architecture Change Proposals in either coder summary.)

## Drift Observations

All nine DRIFT_LOG.md items targeted by ARCHITECT_PLAN.md appear correctly resolved:

| Plan item | File | Verification |
|---|---|---|
| `collect_hints` truncation predicate | `extract.rs:107` | `*i + c.len_utf8() <= 256` — correct |
| `extract_exports` nested recursion | `extract.rs:67` | `continue` after recording exportable item — mirrors `extract_imports` pattern |
| TOCTOU `load_toml_file` | `load.rs:114–123` | `read_to_string` direct; `NotFound → Ok(None)`; other errors → `ConfigError::Io` |
| TOCTOU `BoundarySpec::load` | `boundary.rs:60–64` | Same pattern, consistent |
| TOCTOU `init.rs::run` | `init.rs:87–106` | `create_new(true)` atomic open; `create_dir_all` moved above match (idempotent) |
| `warn_unknown_keys` ANSI injection | `load.rs:129` | `{key:?}` — correct |
| `validate_date_format` impossible days | `thresholds.rs:54–64` | `max_day` per-month; new test covers Feb 30, Apr 31, Feb 29 |
| Duplicate `SDI_CONFIG_PATH` logic | `load.rs:51–56`, `init.rs:64–65` | `project_config_path` extracted, exported from `lib.rs:25`, both call sites delegate |
| `ACTIVE_TREES` doc comment staleness | `sdi-parsing/src/lib.rs:18–21` | Comment now correctly describes invocation-frame tracking |

One systemic observation for the next audit cycle: the doc comment misplacement is a
direct consequence of inserting a new `pub fn` immediately before an existing documented
`pub fn` without verifying that the existing function's doc block remained attached to it.
Future guidance: when inserting a function before another, always confirm the gap between
the two doc blocks contains a non-`///` line.
