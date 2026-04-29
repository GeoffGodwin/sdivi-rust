## Planned Tests

### M05 (previous milestone)
- [x] `crates/sdi-graph/tests/dependency_graph.rs` — DependencyGraph accessors, relative/stem import resolution, ambiguous stem
- [x] `crates/sdi-detection/tests/partition.rs` — LeidenPartition helpers and LeidenConfig::from_sdi_config
- [x] `crates/sdi-cli/tests/version.rs` — fix hardcoded version expectation from 0.0.1 to 0.0.3
- [x] `crates/sdi-parsing/tests/extract_behavior.rs::collect_hints_long_unicode_text_truncated_at_char_boundary` — investigated truncation producing 257 bytes instead of ≤ 256
- [x] `crates/sdi-parsing/tests/extract_behavior.rs::pub_fn_inside_pub_mod_not_in_top_level_exports` — investigated extract_exports recursing into nested pub fns
- [x] `crates/sdi-lang-python/tests/extract_behavior.rs` — 12 tests covering imports, exports, pattern hints, and 256-byte truncation
- [x] `crates/sdi-lang-typescript/tests/extract_behavior.rs` — 11 tests covering imports, exports (function/class/TSX), pattern hints, and truncation
- [x] `crates/sdi-lang-javascript/tests/extract_behavior.rs` — 10 tests covering imports, exports, pattern hints, and truncation
- [x] `crates/sdi-lang-go/tests/extract_behavior.rs` — 9 tests covering imports (grouped), exports (capitalized rule), pattern hints, and truncation
- [x] `crates/sdi-lang-java/tests/extract_behavior.rs` — 9 tests covering imports, public-modifier export rule, pattern hints, and truncation

### M06 (current milestone — Reviewer: Coverage Gaps: None)
- [x] `crates/sdi-patterns/tests/determinism.rs` — 100-case proptest: same records + same config → bit-identical PatternCatalog JSON (2 tests)
- [x] `crates/sdi-patterns/tests/scope_exclude.rs` — scope_exclude removes files from catalog; non-excluded files present; empty exclude no-ops; multiple globs (4 tests)
- [x] `crates/sdi-patterns/tests/entropy_ordering.rs` — high-entropy fixture > simple-rust total entropy; category-level entropy assertions (3 tests)
- [x] `crates/sdi-cli/tests/catalog_format.rs` — `sdi catalog --format json` valid JSON on stdout; text format exits 0 with output; high-entropy entries field present (5 tests)

## Test Run Results
Passed: 267  Failed: 0 (pre-existing doc-test failure in cpm.rs — see Bugs Found; all M06 and M05 integration tests pass)

## Bugs Found
- BUG: [crates/sdi-detection/src/leiden/cpm.rs:22] doc test references `sdi_detection::leiden::cpm::cpm_move_gain` but `cpm` is a private module — doc test fails to compile

## Files Modified
- [x] `crates/sdi-graph/tests/dependency_graph.rs`
- [x] `crates/sdi-detection/tests/partition.rs`
- [x] `crates/sdi-cli/tests/version.rs` — updated hardcoded version from 0.0.1 to 0.0.3
