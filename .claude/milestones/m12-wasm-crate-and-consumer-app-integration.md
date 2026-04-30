#### Milestone 12: WASM Crate, npm Package, Consumer App Integration

**Scope:** Build the `sdi-wasm` crate that wraps `sdi-core` with `wasm-bindgen` + `tsify`-derived `.d.ts`. Produce npm package `@geoffgodwin/sdi-wasm` matching the shape the consumer app needs (sql.js-style async `init()`, then synchronous calls). Validate end-to-end against the consumer app as the first concrete consumer. This milestone closes the loop on KDD-13 (WASM in v0).

**Deliverables:**
- `bindings/sdi-wasm/{Cargo.toml, src/lib.rs, src/exports.rs}` — `crate-type = ["cdylib"]`. Depends on `sdi-core` only (and `wasm-bindgen`, `tsify`, `serde-wasm-bindgen`, `js-sys` as needed). No `sdi-pipeline`, no `sdi-parsing`, no FS dep.
- `wasm-bindgen` exports for each pure-compute function delivered in M08:
  - `compute_coupling_topology(graph: DependencyGraphInput) -> CouplingTopologyResult`
  - `detect_boundaries(graph: DependencyGraphInput, cfg: LeidenConfigInput, prior: Vec<PriorPartition>) -> BoundaryDetectionResult`
  - `compute_boundary_violations(graph: DependencyGraphInput, spec: BoundarySpecInput) -> BoundaryViolationResult`
  - `compute_pattern_metrics(patterns: Vec<PatternInstanceInput>) -> PatternMetricsResult`
  - `compute_thresholds_check(snapshot: Snapshot, summary: DivergenceSummary, cfg: ThresholdsInput) -> ThresholdCheckResult`
  - `compute_delta(prev: Snapshot, curr: Snapshot) -> DivergenceSummary`
  - `assemble_snapshot(...) -> Snapshot`
  - `compute_trend(snapshots: Vec<Snapshot>, last_n: Option<u32>) -> TrendResult`
  - `infer_boundaries(prior_partitions: Vec<PriorPartition>, stability_threshold: u32) -> BoundaryInferenceResult`
  - `normalize_and_hash(node_kind: String, children: Vec<NormalizeNode>) -> String`
- All input/output types derive `tsify::Tsify` so `.d.ts` is generated automatically — the consumer app gets accurate types without manual sync. Strict-TS-compatible: every optional field is explicitly `T | undefined`, no implicit `any`.
- `package.json` shape (consumer-app compatible):
  ```json
  {
    "name": "@geoffgodwin/sdi-wasm",
    "version": "0.1.0",
    "main": "sdi-wasm.js",
    "types": "sdi_wasm.d.ts",
    "files": ["sdi_wasm_bg.wasm", "sdi_wasm.js", "sdi_wasm.d.ts"],
    "license": "Apache-2.0",
    "repository": "..."
  }
  ```
- Build pipeline: `wasm-pack build --target bundler --release` (the consumer app uses webpack/vite-style bundlers; switch to `--target web` only if the consumer explicitly needs raw `.wasm` URL loading).
- `wasm-opt -Os` post-processing to keep bundle size down. Target: under 1 MB compressed `.wasm`.
- Async `init()` pattern matching the sql.js / wasm-bindgen idiom: caller `await init()`, then synchronous calls thereafter.
- `examples/binding_node.ts` — consumer-app-shaped usage:
  ```ts
  import init, { detect_boundaries, normalize_and_hash } from '@geoffgodwin/sdi-wasm';
  await init();
  const hash = normalize_and_hash('try_expression', [...]);
  const result = detect_boundaries(graph, cfg, []);
  ```
- `bindings/sdi-wasm/README.md` covering install, init pattern, every export, and the strict-TS guarantees.
- `bindings/sdi-wasm/build.sh` — single script for the local-dev WASM build.
- `.github/workflows/wasm.yml` — builds the WASM bundle on push to any branch, dry-runs `npm publish` on tagged releases. Asserts bundle size budget (fails CI above 1.2 MB).

**Files to create or modify:**
- New: `bindings/sdi-wasm/{Cargo.toml, src/lib.rs, src/exports.rs, package.json, README.md, build.sh}`
- New: `bindings/sdi-wasm/tests/wasm_smoke.rs` (using `wasm-bindgen-test` against headless Node)
- New: `examples/binding_node.ts`
- New: `.github/workflows/wasm.yml`
- Modify: workspace `Cargo.toml` — add `bindings/sdi-wasm` to `members`. Pin `wasm-bindgen`, `tsify`, `serde-wasm-bindgen` versions in `[workspace.dependencies]`.

**Acceptance criteria:**
- `wasm-pack build --target bundler --release` produces a working bundle. Output `.wasm` < 1.2 MB.
- `npm pack` produces a tarball the consumer app can `npm install` from a file path; `import init, { ... } from '@geoffgodwin/sdi-wasm'` resolves all named exports.
- All `compute_*` functions callable from TS with correct types. The generated `.d.ts` passes `tsc --strict --noUncheckedIndexedAccess --exactOptionalPropertyTypes` against a sample consumer.
- Smoke test: feed a known fixture's nodes/edges/patterns through both `sdi-wasm` (in Node) and native `sdi-core` (in Rust); assert per-dimension equality within the FMA tolerance documented in `docs/determinism.md` (Open Q #10). `normalize_and_hash` outputs must be **bit-identical** across the two — they're hash-only, no float math.
- `normalize_and_hash` produces identical `blake3` digests in WASM vs native sdi-core for the same input. The consumer app's TS extractors can call this and trust the hash matches what Rust would produce.
- `@geoffgodwin/sdi-wasm@0.1.0-rc.0` dry-run publish via `npm publish --dry-run` succeeds.
- A real consumer-app integration smoke: invoke from a consumer-app dev branch, run a full divergence-summary cycle on a sample repo, confirm reasonable output. (This is a manual gate — checked off by the M12 author after coordinating with the consumer-app repo.)

**Tests:**
- `bindings/sdi-wasm/tests/wasm_smoke.rs` — `wasm-bindgen-test` runs each export and asserts non-trivial output
- Cross-platform hash determinism: a Linux CI job builds + runs the WASM smoke; a macOS CI job does the same; both must produce the same `normalize_and_hash` output for a fixture input.
- Bundle-size regression: CI job compares `.wasm` size against a checked-in budget file; fails if over.

**Watch For:**
- `wasm-bindgen` doesn't auto-serialize `BTreeMap<String, T>` — must use `tsify`'s serde adapter or convert to `Vec<(String, T)>` at the boundary. Pick one approach and document.
- `tsify` is still pre-1.0; pin a specific version in workspace deps and add a watch entry to `DRIFT_LOG.md` if a breaking version bump appears.
- The consumer app uses strict TS with `noUncheckedIndexedAccess` and `exactOptionalPropertyTypes`. Verify the generated `.d.ts` passes these flags before claiming compatibility.
- WASM has no FS — by construction since `sdi-core` has none after M08. But `cargo tree -p sdi-wasm --target wasm32-unknown-unknown` is a CI assertion: zero entries for `tree-sitter*`, `walkdir`, `ignore`, `rayon`, `tempfile`, `std::time::SystemTime`-touching crates.
- The `blake3` fingerprint must produce identical bytes between native sdi-core and sdi-wasm. If `blake3`'s SIMD intrinsics are platform-specific, force the portable backend or assert FMA-tolerance equality only.
- npm scope `@geoffgodwin/` requires an org or user-scope on npmjs.com. Verify the scope is registered and `geoff.godwin@gmail.com` has publish rights before M13.
- Bundle size: `wasm-opt -Os` is mandatory. `lto = "fat"` for the WASM build profile. Strip debug symbols.
- WASM error handling: any `panic!` in sdi-core becomes an unhelpful `unreachable executed` in JS. Use `console_error_panic_hook` so panics produce stack traces in dev builds.

**Seeds Forward:**
- M13 publishes `@geoffgodwin/sdi-wasm` on the same tag-driven workflow as crates.io, behind the same manual approval gate.
- The consumer-app integration is the canonical embedder usage pattern; future bindings (PyO3, napi-rs — currently post-MVP / v1 era) follow the same wrap-`sdi-core` shape.
- If WASM integration surfaces missing pure-compute capabilities (e.g., the consumer app needs a function not exposed in M08), they're added to `sdi-core` with the same input-struct + `compute_*` pattern. The pattern, not the specific function list, is the load-bearing API decision.

---
