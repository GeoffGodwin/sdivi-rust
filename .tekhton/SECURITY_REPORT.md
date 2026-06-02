## Summary

This change modifies a single test file (`crates/sdivi-patterns/tests/go_concurrency_node_kind.rs`) to replace a manually maintained list of `assert_ne!` calls with a loop over the internal `ALL_CATEGORIES` constant. The change is entirely within test code; no production code, no public API surface, no I/O, no user input, no authentication or cryptographic logic is touched. The security posture of this change is clean.

## Findings

None

## Verdict

CLEAN
