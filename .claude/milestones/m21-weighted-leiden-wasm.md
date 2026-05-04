#### Milestone 21: Weighted Leiden on WASM

<!-- milestone-meta
id: "21"
status: "done"
-->

**Scope:** Expose weighted-edge Leiden community detection through the WASM bindings by adding `edge_weights: Option<HashMap<String, f64>>` (keyed `"source_node_id:target_node_id"`) to `WasmLeidenConfigInput` and routing the value into the existing native `run_leiden_with_weights` path. Removes ADL-4 from `bindings/sdivi-wasm/src/types.rs:46`. Native `LeidenConfigInput.edge_weights` already exists and works (M15); this milestone is purely the binding-layer surface.

**Why this milestone exists:** Meridian (the consumer app — see memory) carries git churn data per file pair and wants to use it as edge weights so co-frequently-changed files get nudged into the same Leiden community. The native `sdivi-pipeline` Rust path can already do this via `LeidenConfigInput.edge_weights`; the WASM binding intentionally omitted the field at M12 with the comment "WASM bindings expose unweighted Leiden only (ADL-4)." That gap is now blocking the consumer-app integration.

**Deliverables:**
- Add `pub edge_weights: Option<BTreeMap<String, f64>>` to `WasmLeidenConfigInput` in `bindings/sdivi-wasm/src/types.rs`. Keyed by `"source:target"` strings (matching the existing native serde representation when round-tripped through JSON). Use `BTreeMap` not `HashMap` to preserve determinism across iteration sites, even though only `tsify` is consuming the type at the boundary.
- Mark the field `#[tsify(optional)]` so existing TS callers (passing no weights) compile unchanged.
- In `bindings/sdivi-wasm/src/exports.rs`, the `detect_boundaries` (or equivalent) entry point that consumes `WasmLeidenConfigInput`: when `edge_weights` is `Some`, build the native `LeidenConfigInput.edge_weights` value and call `run_leiden_with_weights`; when `None`, keep the existing unweighted call path. Single branch, no behaviour change for existing callers.
- Validation: a key not parseable as `"source:target"` (no colon, or empty source/target) returns `JsError` with a message naming the offending key. A weight value of `NaN` or negative is rejected; `0.0` is accepted (treated as edge absent for weighting purposes).
- Remove the ADL-4 comment block at types.rs:46–48. Remove the corresponding entry from `.tekhton/DESIGN.md` (or wherever ADL-4 is canonically logged) — replace with an "Implemented in M21" pointer.
- Update `bindings/sdivi-wasm/README.md` example snippet to demonstrate passing `edge_weights`.

**Migration Impact:** Strictly additive. Existing TS/JS callers that omit the field see no behavioural change. Snapshots produced with `edge_weights: None` are bit-identical to pre-M21 snapshots. Snapshots produced *with* weights will produce different `LeidenPartition.cluster_assignments` than unweighted — this is the desired outcome and is the user's explicit choice. `snapshot_version` stays `"1.0"`.

**Files to create or modify:**
- **Modify:** `bindings/sdivi-wasm/src/types.rs` — add field to `WasmLeidenConfigInput`; remove ADL-4 comment.
- **Modify:** `bindings/sdivi-wasm/src/exports.rs` — branch on `edge_weights.is_some()`; route to weighted or unweighted Leiden.
- **Modify:** `bindings/sdivi-wasm/tests/` — add a JS-level test that passes weights and asserts a different cluster assignment than the unweighted run on the same graph (existence of difference, not specific assignment).
- **Modify:** `bindings/sdivi-wasm/README.md` — usage example.
- **Modify:** `.tekhton/DESIGN.md` (or wherever ADLs live) — mark ADL-4 implemented.
- **Modify:** `CHANGELOG.md` — under Added.

**Acceptance criteria:**
- `wasm-pack test --node bindings/sdivi-wasm` passes, including a new test that:
  1. Builds a 4-node graph with edges `(a,b), (a,c), (b,c), (c,d)`.
  2. Runs unweighted Leiden, records the partition.
  3. Runs weighted Leiden with weights `{"a:b": 100.0, "c:d": 100.0}`, records the partition.
  4. Asserts the two partitions differ (the weighted run should pull `a,b` into one community and `c,d` into another more strongly).
- A test passing a malformed weight key (e.g. `"a-b"` with no colon) gets a `JsError` with a clear message.
- A test passing `NaN` or negative weight is rejected with a clear `JsError`.
- TS type generation: `tsc --noEmit` against the published `.d.ts` shows `edge_weights?: Record<string, number>` (or the BTreeMap equivalent tsify produces) on `WasmLeidenConfigInput`.
- Existing WASM tests pass unchanged.
- `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` pass.

**Tests:**
- Unit (Rust): the conversion helper from `BTreeMap<String, f64>` to native `LeidenConfigInput.edge_weights` correctly parses `"src:dst"` keys and propagates errors.
- Unit (Rust): malformed key `"no_colon_here"` returns an error naming the key.
- Unit (Rust): `NaN` weight rejected; `-0.5` weight rejected; `0.0` accepted (edge effectively unweighted); positive weight passed through.
- Integration (JS): the weighted-vs-unweighted differ test described above.
- Determinism: same seed + same weights → same partition across two runs (existing seeded-determinism test extended with weights).

**Watch For:**
- **Key parsing must handle node IDs that themselves contain colons.** Path-shaped node IDs like `crates/foo:bar.rs` would break naïve `split(':')`. Use `splitn(2, ':')` (split into source / rest) and treat anything after the first colon as the target. Document this in the rustdoc on the field. Alternative: use a different separator (e.g. `"\u{0001}"`); rejected because the `"src:dst"` form is what `LeidenConfigInput.edge_weights` already serialises to natively.
- **Edge missing from the graph but present in `edge_weights`.** Silently ignore — the weight is informational; if the edge isn't in the graph it can't influence the partition. Don't error; weights produced from coupling history may name pairs that no longer exist as imports.
- **Determinism contract holds with weights.** Same `seed` + same `gamma` + same `edge_weights` map (BTreeMap iteration order!) → bit-identical partition. Verify in the determinism test.
- **`HashMap<String, f64>` in TS calls.** TS-side users pass plain JS objects (`{ "a:b": 1.0 }`); `tsify` deserialises these as `BTreeMap` on the Rust side. Iteration order in JS object literals is insertion-order; the `BTreeMap` re-sorts. So `{ "b:c": 1, "a:b": 2 }` and `{ "a:b": 2, "b:c": 1 }` produce the same partition. Document.
- **Doc-comment placement.** Per CLAUDE.md, ensure a blank line separates the new `edge_weights` doc from the existing `quality` field doc.

**Seeds Forward:**
- M22 lands the change-coupling field in `WasmAssembleSnapshotInput` — the natural pairing for weighted Leiden, since coupling data is what makes good edge weights.
- A future milestone could derive weights automatically from a passed-in coupling-events array, removing the requirement that the consumer hand-build the `"src:dst"` map. Not in scope here; the manual map is the lower-friction primitive for v0.
- If a real user reports the colon-in-node-id ambiguity hitting them, switch to a structured `Vec<{from: String, to: String, weight: f64}>` representation in a future binding bump. Tracked as a possible binding-API v2 concern; not blocking.

---
