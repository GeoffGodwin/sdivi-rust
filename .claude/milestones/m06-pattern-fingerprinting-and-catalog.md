#### Milestone 6: Pattern Fingerprinting and Catalog
<!-- milestone-meta
id: "06"
status: "done"
-->


**Scope:** Implement `sdivi-patterns` — extract per-category subtree shapes from `FeatureRecord` pattern handles, hash with `blake3`, build a `PatternCatalog`, compute pattern entropy. This is the Stage 4 of the pipeline. `sdivi-patterns` does **NOT** depend on `sdivi-graph` or `sdivi-detection` — DESIGN dependency rule.

**Deliverables:**
- `PatternFingerprint` newtype around a `[u8; 32]` blake3 digest
- `PatternCatalog` keyed by `BTreeMap<CategoryName, BTreeMap<PatternFingerprint, PatternStats>>` with instance counts and per-fingerprint file-location lists
- Per-category tree-sitter query strings in `sdivi-patterns::queries::<category>` for the default categories (`error_handling`, `async_patterns`, `state_management`, …)
- Pattern entropy calculator (distinct-shape count adjusted for instance distribution)
- `Config::patterns.min_pattern_nodes` filter and `Config::patterns.scope_exclude` excluding files from the catalog only — files remain in graph and partition
- `sdivi catalog` command printing the catalog as JSON or text

**Files to create or modify:**
- `crates/sdivi-patterns/src/{lib.rs,catalog.rs,fingerprint.rs,entropy.rs}`
- `crates/sdivi-patterns/src/queries/{mod.rs,error_handling.rs,async_patterns.rs,...}`
- `crates/sdivi-cli/src/commands/catalog.rs`
- `tests/fixtures/high-entropy/` (deliberate variance)

**Acceptance criteria:**
- Same fixture + same config → bit-identical `PatternCatalog` JSON across 100 runs
- `scope_exclude` removes files from the catalog but does not change graph/partition output
- `min_pattern_nodes = 5` filters subtrees with fewer than 5 nodes
- `high-entropy/` fixture produces a higher entropy score than `simple-rust/`
- `sdivi catalog --format json` outputs valid JSON to stdout; logs go to stderr
- `blake3` is keyed with the fixed key constant defined exactly once

**Tests:**
- `crates/sdivi-patterns/tests/determinism.rs`: 100-run identical-output proptest
- `crates/sdivi-patterns/tests/scope_exclude.rs`: file in `scope_exclude` absent from catalog, present in `FeatureRecord` stream
- `crates/sdivi-patterns/tests/entropy_ordering.rs`: `entropy(high) > entropy(simple)`
- `crates/sdivi-cli/tests/catalog_format.rs`: JSON and text formats both succeed

**Watch For:**
- Tree-sitter queries must be parsed once per category, not per file — cache them in a `OnceCell` keyed by `(language, category)`
- The pattern instance handles in `FeatureRecord` must carry enough info to re-extract the subtree shape without re-walking the CST (the CST has been dropped per Rule 4). If they don't, this milestone has to push some work back into Milestone 3 — flag early
- `BTreeMap` ordering is critical for determinism; `IndexMap` would also work but is forbidden by KDD-10 unless profiling demands
- `categories = "auto"` resolution depends on which languages are present — implement detection from `FeatureRecord` languages

**Seeds Forward:**
- `PatternCatalog` is an input to snapshot assembly in Milestone 7
- The category-name set is publicly stable from here. Adding a category is non-breaking; renaming is breaking
- `sdivi catalog` command shape sets the precedent for `sdivi show` formatting in Milestone 8

---
