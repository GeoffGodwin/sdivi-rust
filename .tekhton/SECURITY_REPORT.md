## Summary
This change adds a new `data_access` pattern category by registering static string constants (`"call_expression"`, `"call"`) in the classification tables across `sdivi-patterns`, `sdivi-core`, and `sdivi-lang-python`. All modified code is pure static data, linear string matching against tree-sitter node kinds, and AST text collection with existing truncation. No new I/O, no new dependencies, no network paths, no authentication surface, and no cryptographic operations are introduced. The security posture of the change is sound.

## Findings
None

## Verdict
CLEAN
