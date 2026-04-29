//! Pattern fingerprinting and catalog for sdi-rust.
//!
//! Implemented in Milestone 6. Runs tree-sitter queries against parsed ASTs
//! and produces a `BTreeMap`-keyed `PatternCatalog` with `blake3` fingerprints.
