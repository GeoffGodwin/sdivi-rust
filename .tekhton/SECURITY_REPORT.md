## Summary

M37 adds the `null_safety` pattern category for TypeScript and JavaScript by registering two tree-sitter node kind strings (`optional_chain`, `non_null_expression`) in static `&[&str]` slices across `sdivi-patterns`, `sdivi-core`, and the two language adapter crates. All changes are purely additive constant data wired into existing classification dispatch logic. There are no new I/O paths, no user-controlled inputs, no authentication surfaces, no cryptographic operations, and no network calls. The change surface is minimal and carries no meaningful attack surface.

## Findings

None

## Verdict

CLEAN
