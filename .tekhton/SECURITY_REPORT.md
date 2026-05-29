## Summary
This change adds the `logging` pattern category as a catalog-only entry across `sdivi-patterns`, `sdivi-core`, WASM bindings tests, and documentation. All modifications are additive static string constants (`NODE_KINDS`, `ALL_CATEGORIES`, `CATALOG_ENTRIES`) with no logic changes, no I/O, no user-input handling, no cryptographic operations, no authentication paths, and no network calls. The surface area is minimal and entirely within the deterministic, pure-compute layer.

## Findings
None

## Verdict
CLEAN
