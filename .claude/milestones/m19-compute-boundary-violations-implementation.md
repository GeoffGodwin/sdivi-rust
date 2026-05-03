#### Milestone 19: `compute_boundary_violations` Implementation

<!-- milestone-meta
id: "19"
status: "done"
-->

**Scope:** Replace the M10 stub at `crates/sdivi-core/src/compute/boundaries.rs:237` with a real implementation that walks the `DependencyGraphInput`'s edges, classifies each endpoint's boundary via `BoundaryDefInput.modules` glob matching, and emits a `(from, to)` pair when the target's boundary is not in the source boundary's `allow_imports_from` whitelist. This unblocks Factor 4 (boundary violation velocity) for both native consumers and the WASM consumer-app integration — the WASM wrapper already calls into `sdivi-core::compute_boundary_violations` and inherits the fix automatically.

**Why this milestone exists:** The function currently returns `BoundaryViolationResult { violation_count: 0, violations: vec![] }` with the comment "Full violation detection is implemented in Milestone 10" (boundaries.rs:244). M10 shipped boundary inference and the `boundaries` CLI subcommand, but the violation-counting compute itself was never wired up. Result: `boundary_violation_delta` is always `0`, so `compute_thresholds_check`'s `boundary_violation_rate` gate can never trigger. From the consumer app's perspective, Factor 4 is dead on arrival — Meridian's CI gate would happily pass codebases with arbitrary cross-layer import sprawl.

**Deliverables:**
- Implement `compute_boundary_violations(graph, spec)` using `globset::GlobSet` (already a workspace dep) to compile each `BoundaryDefInput.modules` glob set once. Match every node's `id` against the compiled sets to produce a `BTreeMap<String, &str>` of `node_id → boundary_name`. Nodes that match zero boundaries are unscoped and produce no violation regardless of edge direction.
- Walk `graph.edges`. For each edge `(from, to)` where both endpoints are scoped to a boundary AND the boundaries differ AND `to`'s boundary is not in `from`-boundary's `allow_imports_from`, push `(from.clone(), to.clone())` onto `violations`.
- Nodes matching multiple boundary globs: pick the **most specific** match (longest matching glob string by character length, ties broken by sort order of the boundary name). Document the rule in the rustdoc with a `# Determinism` note.
- Return `BoundaryViolationResult { violation_count: violations.len() as u32, violations }`. Sort `violations` by `(from, to)` lexicographically before returning to preserve the `BTreeMap`-style determinism contract (Rule 5 / KDD-10).
- Update the doc-test at boundaries.rs:228 to also exercise a non-empty case (two boundaries, one violation).
- Update `crates/sdivi-snapshot/src/lib.rs` (or wherever `intent_divergence` is assembled) so `IntentDivergenceInfo.violation_count` reflects the real number, not the stub `0`. Verify the assembly path that already calls `compute_boundary_violations` — no new call sites needed; the function's return value just becomes meaningful.

**Migration Impact:** Snapshots produced after M19 with a non-empty `BoundarySpec` will report non-zero `intent_divergence.violation_count` where they previously reported `0`. This is a *correctness fix*, not a schema change — `snapshot_version` stays `"1.0"`. Adopters with a `.sdivi/boundaries.yaml` should expect the first post-M19 snapshot to surface their existing violations as a single large delta against the prior (always-zero) baseline. Document in CHANGELOG that adopters may want to re-baseline at the M19 boundary or use a one-time `boundary_violation_rate` override with `expires` set 1–2 weeks out to absorb the cutover.

**Files to create or modify:**
- **Modify:** `crates/sdivi-core/src/compute/boundaries.rs` — real `compute_boundary_violations` body. Add private helper `match_boundary(node_id, compiled_specs) -> Option<&str>` for the most-specific-wins lookup.
- **Modify:** `crates/sdivi-core/Cargo.toml` — add `globset` if not already a direct dep (verify via `cargo tree -p sdivi-core`). Note the WASM-build constraint: `globset` must compile for `wasm32-unknown-unknown` with default features. Verify; if a feature flag is needed, gate it.
- **Modify:** `crates/sdivi-snapshot/src/` — wherever `assemble_snapshot` receives the violation count, ensure it threads through unchanged. The compute call already exists; only the returned value changes.
- **Modify:** `CHANGELOG.md` — Fixed: "compute_boundary_violations now performs real violation detection; previously stubbed to return zero. Factor 4 (boundary violation velocity) is now active in `sdivi check`."

**Acceptance criteria:**
- `cargo test -p sdivi-core` passes, including a new test with two boundaries (`api` modules `crates/api/**`, `db` modules `crates/db/**`, `api` allows imports from `db` only) and an edge from `crates/db/foo.rs` to `crates/api/bar.rs` produces exactly one violation.
- The boundary-lifecycle integration test under `tests/` (already present) is extended with a violation-emitting fixture and asserts `violation_count > 0` in the resulting snapshot.
- `cargo build -p sdivi-core --target wasm32-unknown-unknown --no-default-features` still succeeds (no new forbidden deps).
- `cargo tree -p sdivi-core --target wasm32-unknown-unknown --no-default-features` shows zero entries for `tree-sitter*`, `walkdir`, `ignore`, `rayon`, `tempfile` — preserved per Rule 21 / KDD-12.
- `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` pass.
- WASM smoke tests in `bindings/sdivi-wasm/tests/` continue to pass with no signature change.

**Tests:**
- Unit: `match_boundary` selects the most-specific matching glob. Hand-built case: glob `crates/**` and glob `crates/api/**` both match `crates/api/foo.rs`; longer-match (`crates/api/**`) wins.
- Unit: an edge between two unscoped nodes produces no violation; an edge from a scoped node to an unscoped node produces no violation; an edge between two nodes in the same boundary produces no violation.
- Unit: `allow_imports_from` whitelist semantics — a → b allowed when `a.allow_imports_from` contains `b.name`. Reverse direction (b → a) is independent and depends on `b.allow_imports_from`.
- Property (proptest): for any graph + boundary spec, `violation_count == violations.len()`. Trivial but catches future drift.
- Property (proptest): `violations` is sorted lexicographically by `(from, to)` (determinism gate).
- Integration: extend `tests/boundary_lifecycle.rs` with a fixture that has known violations; assert the snapshot's `intent_divergence.violation_count` matches the hand-counted value.

**Watch For:**
- **Glob compilation cost.** Compile each boundary's `GlobSet` once at the top of `compute_boundary_violations`, not per-node. For a 10k-node graph with 8 boundaries, naive recompilation per call is the difference between sub-millisecond and seconds.
- **Most-specific tie-break must be deterministic.** Two globs of equal length matching the same node: break by sorted boundary name (BTreeMap iteration order). Document in rustdoc.
- **Self-loops.** An edge `(n, n)` where both endpoints resolve to the same boundary is not a violation. Skip same-boundary edges before checking the whitelist.
- **`allow_imports_from` not a transitive closure.** If `a` allows `b` and `b` allows `c`, `a → c` is still a violation unless `a.allow_imports_from` explicitly contains `c`. Do not synthesise transitive permissions.
- **Empty `boundaries` spec is normal operation (Rule 16).** Return `BoundaryViolationResult { violation_count: 0, violations: vec![] }` immediately without iterating edges.
- **Doc-comment placement.** Per CLAUDE.md, ensure a blank line separates the new `match_boundary` doc block from the existing `compute_boundary_violations` doc block — `#![deny(missing_docs)]` will catch a re-attached comment as a missing-docs error.
- **WASM dep tree.** `globset` pulls `regex`-family crates. Verify all transitively compile to `wasm32-unknown-unknown` before merging; if not, gate the implementation on a default-on Cargo feature and provide a plain-string-match fallback for the WASM build.

**Seeds Forward:**
- Extends Factor 4 to per-boundary breakdown if/when a future milestone adds `boundary_violation_per_boundary` to `DivergenceSummary`. The current implementation returns the flat `(from, to)` list; a downstream aggregator can group by boundary without changing this signature.
- Adopters in flight with a `.sdivi/boundaries.yaml` will benefit from a one-time `boundary_violation_rate` override; document the pattern in `docs/cli-integration.md` as part of M19.

---
