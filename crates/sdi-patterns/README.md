# sdi-patterns

Pattern fingerprinting and catalog for [sdi-rust](https://github.com/geoffgodwin/sdi-rust).

Builds a `PatternCatalog` from tree-sitter AST queries using `blake3`-keyed fingerprints.
Computes entropy and convention drift per pattern category.

Part of the `sdi-rust` workspace. See the [workspace README](../../README.md) for full documentation.
