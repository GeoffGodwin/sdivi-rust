## Test Audit Report

### Audit Summary
Tests audited: 9 files, 92 test functions
Verdict: CONCERNS

---

### Findings

#### SCOPE: version.rs asserts stale version string — test will fail on current codebase
- File: `crates/sdi-cli/tests/version.rs:20`
- Issue: `version_flag_prints_crate_version` asserts `stdout(contains("0.0.4"))`. The workspace `Cargo.toml` sets `version = "0.0.5"` and `sdi-cli` inherits via `version.workspace = true`. When `sdi --version` runs it will print `sdi 0.0.5`, causing this assertion to fail. The TESTER_REPORT documents the change as "0.0.1 → 0.0.3", which is doubly inaccurate: the test now says `0.0.4` and the workspace is `0.0.5`. Neither the tester's reported source nor target version is correct.
- Severity: HIGH
- Action: Update the assertion to `contains("0.0.5")` to match the current workspace version. Also correct the TESTER_REPORT entry to reflect the actual transition and version.

#### NAMING: `run_leiden_single_clique_positive_modularity_and_one_community` does not assert positive modularity
- File: `crates/sdi-detection/tests/partition.rs:181`
- Issue: The test name promises two things: positive modularity and a single community. Neither is fully asserted. The test only checks `assignments.len() == 4`, `community_count() >= 1` (accepts any count), and `seed == 42`. No assertion verifies `partition.modularity > 0.0`. The ">= 1 community" assertion is also vacuously satisfied by any non-empty partition.
- Severity: MEDIUM
- Action: Add `assert!(partition.modularity > 0.0, "dense clique must yield positive modularity, got {}", partition.modularity)`. Optionally tighten the community count to `== 1` since a single dense clique should collapse to one community under default Modularity.

#### NAMING: Stale inline comment in `pub_fn_inside_pub_mod_not_in_top_level_exports` describes a fixed bug as current
- File: `crates/sdi-parsing/tests/extract_behavior.rs:126`
- Issue: Comment reads "This test documents a known latent issue: the traversal currently recurses into mod_item children, so `inner` is also captured as an export." The previous TEST_AUDIT_REPORT noted this was fixed via a `continue` guard in `sdi-lang-rust/src/extract.rs:67`. The word "currently" is factually wrong — the test now verifies correct, post-fix behavior. A reader of this test is misled into thinking the assertion is expected to fail.
- Severity: MEDIUM
- Action: Replace the comment with one describing the proven behavior: the `continue` guard stops traversal into nested `pub mod` children so `inner` does not surface as a top-level export.

#### SCOPE: Comment in `relative_import_parent_slash_strips_prefix_resolves_in_same_dir` references a bug note that does not exist in TESTER_REPORT
- File: `crates/sdi-graph/tests/dependency_graph.rs:142`
- Issue: The comment says "See BUG note in TESTER_REPORT.md re: parent navigation not implemented." The TESTER_REPORT contains exactly one bug (the CPM doc-test compile failure); there is no bug note about `../` parent navigation. The reference is a dangling pointer. The test logic itself is correct — it accurately documents the implementation's behavior of treating `../` identically to `./` — but the external cross-reference is wrong.
- Severity: LOW
- Action: Remove the "See BUG note" cross-reference. Replace with a self-contained explanation such as "The implementation strips both `./` and `../` prefixes and resolves in the importer's own directory; true parent navigation is not implemented."

---

### Clean Findings (no issues)

- **Assertion Honesty (all files):** All assertions derive their expected values from the real implementation — default field values (`seed: 42`, `max_iterations: 100`, `gamma: 1.0`) match the `LeidenConfig::Default` impl; community counts and sizes are computed from fixture data, not pulled from thin air; no always-true or tautological assertions detected.
- **Test Isolation (all files):** Every test constructs source input inline or builds `FeatureRecord` fixtures in-memory. No test reads `.tekhton/`, build artifacts, pipeline logs, or any mutable project-state file.
- **Implementation Exercise (all files):** No mocking of internal functions. All tests invoke real entry points (`build_dependency_graph`, `run_leiden`, `parse_file`) and observe their actual outputs.
- **Edge Case Coverage (dependency_graph.rs):** Empty graph, out-of-range index, unresolvable relative import, ambiguous stem, and external crate imports are all exercised. Ratio of error-path to happy-path tests is healthy.
- **Edge Case Coverage (partition.rs):** Empty partition and single-node partition cases are tested for `largest_community_size` and `communities`. Malformed JSON rejection is verified for `from_json`.
- **Scope Alignment (all language adapter files):** All imported adapter types (`PythonAdapter`, `TypeScriptAdapter`, `JavaScriptAdapter`, `GoAdapter`, `JavaAdapter`, `RustAdapter`) exist and are correctly imported. No orphaned or stale symbols.
- **Test Weakening:** No existing assertions were broadened or removed in the files under audit. The `version.rs` change added a version-string check (strictly more specific than the prior `success()`-only assertion), though the string itself is now stale (see HIGH finding above).
- **Test Naming (language adapter files):** Names follow `<subject>_<scenario>_<expected_outcome>` and encode both input condition and asserted result unambiguously.
- **full_pipeline.rs (freshness sample):** This file is a documented layout placeholder, not a compiled test. No action required.
- **test_dedup.fingerprint (freshness sample):** File was intentionally deleted per coder. No orphaned tests reference it.
