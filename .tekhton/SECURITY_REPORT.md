## Summary
M34 is a pure internal refactor: the three-if-block `call_expression`/`call` arm inside `classify_hint` was replaced with a `const CALL_DISPATCH` function-pointer table and a single loop. No authentication, cryptography, network communication, user-facing input validation, or I/O paths were touched. The four changed files are `crates/sdivi-patterns/src/queries/mod.rs`, a new test file `crates/sdivi-patterns/tests/dispatch_disjointness.rs`, `docs/pattern-categories.md`, and `CHANGELOG.md`. The change carries no exploitable surface and introduces no new trust boundaries.

## Findings
None

## Verdict
CLEAN
