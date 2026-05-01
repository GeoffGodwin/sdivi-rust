# Coder Summary
## Status: COMPLETE

## What Was Implemented
M08 (`sdi-core` Pure-Compute Reshape) was already implemented in a prior session (milestone status: "done"). The workspace had one stale failing test from that implementation:

- Updated `crates/sdi-cli/tests/version.rs`: the version assertion `contains("0.0.10")` was stale after M08 bumped the workspace to `0.0.11`. Updated to `contains("0.0.11")`.

All other M08 deliverables (sdi-pipeline crate, sdi-core compute facade, input structs, feature gating, I/O extraction, etc.) were already in place and passing.

## Root Cause (bugs only)
The `version_flag_prints_crate_version` test hardcoded `0.0.10` but M08's version bump changed the workspace to `0.0.11`. The test was not updated atomically with the version bump.

## Files Modified
- `crates/sdi-cli/tests/version.rs` — updated hardcoded version assertion from `0.0.10` to `0.0.11`

## Human Notes Status
No human notes listed in this task.

## Docs Updated
None — no public-surface changes in this task.

## Observed Issues (out of scope)
- The version test hardcodes a version string. A more robust pattern would use `env!("CARGO_PKG_VERSION")` from the sdi-cli crate to avoid this recurring issue on future version bumps.
- `rust-toolchain.toml` specifies `1.85.0` but the running toolchain is `1.75.0` (flagged in pre-flight report). `cargo clippy` and `cargo fmt` are unavailable as a result. This is an environment issue, not a code issue.
