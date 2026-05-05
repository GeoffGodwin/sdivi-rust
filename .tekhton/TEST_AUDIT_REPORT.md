## Test Audit Report

### Audit Summary
Tests audited: 2 files, 24 test functions (12 in resolver_edge_cases.rs, 12 in resolver_no_panic.rs)
Verdict: PASS

---

### Methodology

Read both test files in full; read `crates/sdivi-graph/src/resolve.rs`,
`crates/sdivi-graph/src/resolve_lang.rs`, and
`crates/sdivi-graph/src/dependency_graph.rs` to trace every asserted value
back to implementation logic. Verified `node_for_path`, `node_path`, and
`edges_as_pairs` signatures against their implementations to confirm test
comparisons are type-safe and semantically correct.

---

### Findings

#### COVERAGE: assert_no_panic doc comment promises edge_count assertion that is never made
- File: `crates/sdivi-graph/tests/resolver_no_panic.rs:30`
- Issue: The helper's doc comment states "assert the edge_count is 0
  (specifier unresolvable) and no panic occurred." The body is
  `let _ = build_dependency_graph(&records);` — the return value is
  discarded. No edge_count assertion exists. A regression that causes the
  resolver to fabricate edges for adversarial inputs (e.g. `"../../../foo"`)
  would pass all 12 tests silently. The no-panic contract is correctly
  tested, but the promised secondary contract is absent.
- Severity: MEDIUM
- Action: Either (a) delete the "assert the edge_count is 0 (specifier
  unresolvable)" clause from the doc comment so it accurately describes what
  is tested, or (b) add `assert_eq!(dg.edge_count(), 0, "adversarial input
  must not fabricate edges")` to the helper body. Option (b) is preferred
  as it strengthens coverage of adversarial-input regressions.

#### NAMING: double_super_finds_intermediate_level_stem comment mis-states the walked-up base directory
- File: `crates/sdivi-graph/tests/resolver_edge_cases.rs:104`
- Issue: The test comment says "subtree search in the walked-up base `a/`
  returns a unique match there." `resolve_super` with `super::super::` from
  `a/b/c.rs` computes from_dir=`a/b`, then walks up twice: 1st → `a`,
  2nd → `""` (empty / repo root). The call is therefore
  `find_stem_in_subtree("models", "", ...)`, which searches the entire
  repository, not just the `a/` subtree. The assertion is correct (only one
  file has stem "models") but the comment describing the mechanism is wrong
  and would mislead future readers diagnosing a subtree-search failure.
- Severity: LOW
- Action: Change the comment to reference the actual walked-up base (`""`
  repo root), e.g. "walked-up base is repo root `\"\"` because two super::
  levels exhaust `a/b` → `a` → `\"\"`; find_stem_in_subtree searches the
  entire repo."

#### COVERAGE: triple_super_overshoot_falls_back_to_global_stem comment imprecise about overshoot point
- File: `crates/sdivi-graph/tests/resolver_edge_cases.rs:123`
- Issue: The comment says "three levels up overshoots the root (only two
  path components above)." The actual mechanism is: from_dir=`a/b`, level 1
  → `a`, level 2 → `""`, level 3 → `"".parent()` returns `None`, triggering
  the fallback. The overshoot is not about path component count; it is about
  `Path::parent()` returning `None` on an empty path at the third iteration.
  The assertion (`edge_count() == 1`) is correct.
- Severity: LOW
- Action: Replace the parenthetical with: "three super:: levels: `a/b` →
  `a` → `""` → `None` on the third `.parent()`, which triggers the
  global stem-map fallback."

---

### Positive Findings

**Assertion honesty.** Every `edge_count()` value in `resolver_edge_cases.rs`
was independently traced through `resolve.rs` / `resolve_lang.rs` and
confirmed to match implementation logic. No hard-coded values that are
disconnected from what the code actually produces.

**Implementation exercise.** All tests call the real
`build_dependency_graph` / `build_dependency_graph_with_go_module`
functions through the public crate API with no mocking.
`resolve.rs`, `resolve_lang.rs`, and `dependency_graph.rs` are all
exercised on every run.

**Edge-case coverage alignment.** The twelve edge-case scenarios map
precisely to the two M26 bugs fixed by the coder (missing `../` parent
navigation, missing per-language dispatch): Python PEP 420 namespace
packages, Python triple-dot relative imports, Rust double/triple `super::`,
Rust `super::` overshoot fallback, Java multi-module Maven root discovery,
Java wildcard imports, Go module multi-file package edges, graph
determinism, and self-loop prevention are all covered.

**Structural invariant test.** `all_resolved_edges_point_to_nodes_from_input_records`
verifies that no resolver path fabricates a node that was not in the
original input records. This test would catch any future path-fabrication
regression regardless of language or specifier type.

**Determinism tests.** Both `build_dependency_graph_is_deterministic` and
`build_dependency_graph_go_module_is_deterministic` sort both edge lists
before comparison (correctly handling unstable `raw_edges()` ordering)
and the Go variant additionally pins an absolute edge count (`== 3`) that
is derivable from the three `.go` files in the fixture.

**No test weakening.** The tester added only new files; no existing test
was modified.

**No orphaned tests.** All imports reference symbols present in the
current codebase. The deleted `.tekhton/test_dedup.fingerprint` and the
removed `bindings/sdivi-wasm/package.json` are not referenced by either
audited file.

**Fixture isolation.** All test data is constructed inline via the `rec()`
factory function. No test reads `.tekhton/` reports, pipeline logs, or
other mutable project state.
