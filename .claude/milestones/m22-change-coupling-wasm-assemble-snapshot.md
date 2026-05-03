#### Milestone 22: Change Coupling in WASM `assemble_snapshot`

<!-- milestone-meta
id: "22"
status: "pending"
-->

**Scope:** Add an optional `change_coupling: Option<WasmChangeCouplingInput>` field to `WasmAssembleSnapshotInput` and thread it through the WASM `assemble_snapshot` call to the 5th positional argument of native `sdivi_core::assemble_snapshot` (currently passed as `None` at `bindings/sdivi-wasm/src/exports.rs:170`). Removes ADL-7. WASM consumers can now produce snapshots that carry their own change-coupling analysis output, parallel to what `sdivi-pipeline` does for native callers.

**Why this milestone exists:** M15 (Change-Coupling Analyzer) added the native `compute_change_coupling` function and threaded the result through `assemble_snapshot` for the pipeline path. The WASM binding stayed at the M12 surface — `compute_change_coupling` is exposed as a standalone WASM function, but its output cannot be included in a WASM-built snapshot because `WasmAssembleSnapshotInput` has no field for it. Meridian needs the full snapshot to include change-coupling metrics so its CI gate can read them from a single artifact instead of stitching two outputs together.

**Deliverables:**
- Define `WasmChangeCouplingInput` in `bindings/sdivi-wasm/src/types.rs` mirroring the shape of native `sdivi_core::ChangeCouplingResult` (or whatever type the 5th arg of `assemble_snapshot` accepts — verify before writing). Likely shape: `coupling_score: f64`, `top_pairs: Vec<{from: String, to: String, frequency: f64}>` and any additional summary fields the native struct carries. Tsify-derive both the wrapper and any nested types.
- Add `pub change_coupling: Option<WasmChangeCouplingInput>` to `WasmAssembleSnapshotInput`. Mark `#[tsify(optional)]`.
- In `exports.rs::assemble_snapshot`, replace the `None` at the 5th positional argument (line 170) with `input.change_coupling.map(to_core).transpose()?`. Convert via the standard `to_core` pattern already used for the other input types in the file.
- Remove the TODO comment block at exports.rs:162–164. Remove ADL-7 from `.tekhton/DESIGN.md` (or wherever it's logged) — replace with an "Implemented in M22" pointer.
- Update `bindings/sdivi-wasm/README.md` example snippet to demonstrate the round-trip: call `compute_change_coupling`, pass the result into `assemble_snapshot`.

**Migration Impact:** Strictly additive. Existing TS/JS callers that omit `change_coupling` see no behavioural change — the field defaults to `None`, the snapshot's `change_coupling` field is absent (or `null`, depending on the native serde representation), exactly matching pre-M22 output. Callers that supply `change_coupling` get a snapshot with the field populated, identical to what the native pipeline produces. `snapshot_version` stays `"1.0"` — the field already exists in the schema; only the WASM path was previously unable to populate it.

**Files to create or modify:**
- **Modify:** `bindings/sdivi-wasm/src/types.rs` — add `WasmChangeCouplingInput` (and any nested types). Add field to `WasmAssembleSnapshotInput`.
- **Modify:** `bindings/sdivi-wasm/src/exports.rs` — replace the hardcoded `None` at line 170 with the threaded value. Remove TODO/ADL-7 comment block.
- **Modify:** `bindings/sdivi-wasm/tests/` — add a test asserting that a snapshot built with `change_coupling: Some(...)` has the expected field populated in the output JSON; a snapshot built with `None` does not.
- **Modify:** `bindings/sdivi-wasm/README.md` — round-trip example.
- **Modify:** `.tekhton/DESIGN.md` — mark ADL-7 implemented.
- **Modify:** `CHANGELOG.md` — under Added.

**Acceptance criteria:**
- `wasm-pack test --node bindings/sdivi-wasm` passes, including a new test that:
  1. Calls `compute_change_coupling` with a fixture coupling-events array.
  2. Passes the result into `assemble_snapshot` via the new field.
  3. Asserts the returned snapshot's `change_coupling` field matches the input value (round-trip).
- A test omitting the field produces a snapshot whose `change_coupling` field is absent / null — bit-identical to the pre-M22 behaviour.
- TS type generation: `tsc --noEmit` against the published `.d.ts` shows `change_coupling?: WasmChangeCouplingInput` on `WasmAssembleSnapshotInput`.
- Existing WASM tests pass unchanged.
- `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` pass.

**Tests:**
- Unit (Rust): the `to_core` conversion from `WasmChangeCouplingInput` to the native type round-trips correctly for representative shapes (empty top_pairs, populated top_pairs, edge-case scores).
- Integration (JS): the round-trip test above.
- Integration (JS): omit-the-field test asserting absent / null in output.
- Determinism: same `change_coupling` input produces bit-identical snapshot bytes across two `assemble_snapshot` calls (determinism gate).

**Watch For:**
- **Verify the exact native type at the 5th argument before defining the wrapper.** The signature is `assemble_snapshot(graph, partition, catalog, pm, change_coupling: Option<???>, timestamp, commit, ...)`. Read `crates/sdivi-snapshot/src/lib.rs` (or wherever `assemble_snapshot` lives) to confirm the type name and field shape; do not guess. The wrapper must be field-equivalent so `to_core` is mechanical.
- **Deeply-nested types must all be tsify-derived.** If `ChangeCouplingResult` contains `Vec<CouplingPair>` and `CouplingPair` is its own struct, the WASM mirror needs both. Missing one fails compilation at the macro layer with a less-than-helpful error.
- **Field-naming convention.** `tsify` produces TS field names matching the Rust serde representation. If the native type uses `#[serde(rename = "...")]` anywhere, mirror that exactly so the JSON output is bit-identical to the native pipeline's output for the same input.
- **Snapshot output bytes must match the native pipeline.** Compare a WASM-assembled snapshot byte-for-byte against a `sdivi-pipeline` snapshot for the same graph + partition + catalog + change-coupling inputs; any divergence is a serde-config bug. This is the determinism contract per Rule 23.
- **Doc-comment placement.** Per CLAUDE.md, ensure a blank line separates the new `change_coupling` doc from the next field's doc.

**Seeds Forward:**
- With M21 (weighted Leiden) and M22 (change coupling) both shipped, the WASM surface fully matches the native pipeline's `assemble_snapshot` capabilities. Document this state in `bindings/sdivi-wasm/README.md` as "WASM API parity reached for snapshot assembly" so future contributors don't reintroduce gaps.
- M23 (pattern category contract) is the last remaining WASM-surface gap — once that lands, the consumer-app integration story is complete from the binding-API side and only the distribution work (M24) remains.

---
