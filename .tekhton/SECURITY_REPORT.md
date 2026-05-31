## Summary
This change consists entirely of doc comment and description string updates in two files: the `ALL_CATEGORIES` rustdoc comment in `sdivi-patterns/src/queries/mod.rs` and the `logging` entry string literal in the `CATALOG_ENTRIES` array in `sdivi-core/src/categories.rs`. No logic, control flow, I/O, authentication, cryptography, network communication, or user input handling was modified. The description string in `CATALOG_ENTRIES` is returned verbatim via `list_categories()` as a read-only `String` field — it is never interpreted as code, a path, or a command. Security posture is unchanged from the pre-M33 baseline.

## Findings
None

## Verdict
CLEAN
