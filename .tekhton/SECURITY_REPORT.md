## Summary

This change touches three files — `crates/sdivi-patterns/src/queries/mod.rs`, `crates/sdivi-patterns/src/queries/async_patterns.rs`, and `crates/sdivi-patterns/tests/test_all_categories_doc_classification.rs` — making purely documentation and test-classification corrections. The `mod.rs` doc comment is updated to move `async_patterns` from the "Node-kind only" list to the "Hybrid" list; `async_patterns.rs` gains a clarifying module-level doc block; and the test file adds an `async_patterns_is_hybrid_both_node_kind_and_callee` assertion while removing `async_patterns` from the node-kind-only test vector. None of the changed lines touch authentication, cryptography, user input validation, network communication, filesystem access, or secret handling. The regex in `async_patterns.rs` (`\.(then|catch|finally)\(`) is used exclusively for in-process classification and presents no injection surface. Security posture is unchanged by this diff.

## Findings

None

## Verdict

CLEAN
