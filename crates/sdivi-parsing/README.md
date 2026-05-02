# sdivi-parsing

File discovery and AST parsing stage for [sdivi-rust](https://github.com/geoffgodwin/sdivi-rust).

Provides `LanguageAdapter` trait and the file-discovery + parsing pipeline using
`walkdir`, `ignore`, `rayon`, and `tree-sitter`. Tree-sitter CSTs are dropped
before each parse call returns (memory proportional to largest single file).

Part of the `sdivi-rust` workspace. See the [workspace README](../../README.md) for full documentation.
