# sdivi-patterns

Pattern fingerprinting and catalog for [sdivi-rust](https://github.com/geoffgodwin/sdivi-rust).

Builds a `PatternCatalog` from tree-sitter AST queries using `blake3`-keyed fingerprints.
Computes entropy and convention drift per pattern category.

Part of the `sdivi-rust` workspace. See the [workspace README](../../README.md) for full documentation.
