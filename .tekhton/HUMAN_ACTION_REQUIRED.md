# Human Action Required

The pipeline identified items that need your attention. Review each item
and check it off when addressed. The pipeline will display a banner until
all items are resolved.

## Action Items

_None at present. Resolved items move to the section below._

## Resolved

- [x] [2026-04-29 | resolved by user] **`init.rs` stdout/stderr contract** — chose option (b): all `sdi init` progress lines moved to `eprintln!` and the two `init.rs` integration tests flipped from `.stdout(...)` to `.stderr(...)`. CLAUDE.md Rule 8 stays unmodified — `sdi init` produces no stdout payload. Resolved before 0.1.0 stdout/stderr contract was locked.
- [x] [2026-04-29 | resolved by user] **`serde_yaml` re-evaluation** — swapped to `serde_yml` ("0.0.12") now while the YAML surface is a single call site (`crates/sdi-config/src/boundary.rs`). Workspace dep, `sdi-config` dep, and the lone `serde_yaml::from_str` → `serde_yml::from_str` call updated. Closes the M10 deferral early; keeps `cargo audit` clean for the rest of the milestone work.
