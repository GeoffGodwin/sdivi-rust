## Summary
M36.2 extends the `decorators` pattern category to Python by adding the `"decorated_definition"` node kind to a compile-time constant string slice in `decorators.rs`, extracts an inline test block into a separate `tests.rs` file (no logic change), and adds new test cases in the lang-python and sdivi-core test suites. All changes are purely additive data and test code. There is no new I/O, no user-controlled input processing, no cryptographic code, no authentication surface, and no network interaction introduced by this change.

## Findings
None

## Verdict
CLEAN
