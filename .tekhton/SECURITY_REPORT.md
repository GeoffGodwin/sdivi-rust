## Summary
This change set comprises two files: a minor refactor removing an unused `current` parameter from the `pub(crate)` function `compute_historical_stability` in `crates/sdi-core/src/compute/boundaries.rs`, and a documentation-only update to `.tekhton/NON_BLOCKING_LOG.md` marking nine prior notes resolved. The modified Rust file is a pure-compute function with no I/O, no network calls, no cryptography, and no user-controlled shell expansion. All node ID inputs continue to go through the existing `validate_node_id` guard. No security-sensitive surface area was introduced or altered.

## Findings
None

## Verdict
CLEAN
