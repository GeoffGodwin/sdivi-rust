
#### Milestone 47: WASM Consumer-Surface Typecheck Guard

<!-- milestone-meta
id: "47"
status: "pending"
-->

**Scope:** Add a deterministic CI guard that typechecks the `@geoffgodwin/sdivi-wasm` *consumer* TypeScript surface — the runnable examples and the documented usage patterns — against the **freshly built** `.d.ts` that the package actually ships. Today nothing does this: the `wasm.yml` workflow builds both wasm-pack targets, runs `wasm-pack test --node`, and runs the Node.js smoke tests (`tests/node_smoke/`), but the smoke tests only call `list_categories()` and never exercise the import shape, the init contract, or the `Map`-typed inputs/outputs that real consumers use. As a result, the binding's docs and examples drifted away from the shipped types and the drift shipped undetected. This milestone closes that gap by (1) adding a strict `tsc --noEmit` typecheck of `examples/binding_node.ts` and `examples/binding_bundler.ts` against the generated `pkg/` types, (2) adding a self-verifying **negative** fixture that asserts the previously-shipped broken patterns *fail* to typecheck, (3) adding a cheap forbidden-pattern lint over the consumer-facing docs to lock the exact regressions that shipped, and (4) wiring all three into `wasm.yml` and reconciling the now-stale "bundler path is not exercised" comments. No library code changes; this is pure consumer-contract test infrastructure.

**Why this milestone exists:** A pre-release review of the WASM consumer experience found that every documented onboarding example was broken in two independent ways, and both had shipped to `main`:

- **`await init()` against a package with no `init`.** The README, the root `README.md`, `docs/pattern-categories.md`, the binding module rustdoc, and `examples/binding_node.ts` all instructed consumers to `import init, { … }` and `await init()`. The shipped wasm-pack `--target bundler` and `--target nodejs` builds expose **no default export and no `init` function** — the bundler target auto-runs `wasm.__wbindgen_start()` on import and the nodejs target loads the `.wasm` synchronously at require time. A consumer copy-pasting the headline example hit `TypeError: init is not a function` (or, in strict TS, "Module has no default export") on the first line. The pattern traced back to the M12 spec, which designed for the `--target web` `await init()` idiom; the implementation sensibly shipped bundler+nodejs instead, and the docs were never reconciled.

- **`Map`-typed inputs passed as plain objects.** `tsify_next` (no object-map override) maps Rust `BTreeMap`/`HashMap` to JS `Map` — the `.d.ts` types `edge_weights`, `cluster_assignments`, `internal_edge_density`, `entropy_per_category`, and `overrides` as `Map<…>`. The weighted-Leiden example passed `edge_weights` as an object literal `{ 'a:b': 0.8 }`, which `serde-wasm-bindgen` rejects at runtime.

The docs and examples were corrected by hand in the same review (commit `docs(wasm): fix broken consumer onboarding`), but **nothing in CI prevents the same drift from shipping again.** The existing `node_smoke` test even documents the blind spot in its own header comment: "the bundler path … is not exercised end-to-end in this CI workflow." Both shipped bugs were type-level/contract drift, so a `tsc --noEmit` of the examples against the real `.d.ts` would have caught both at PR time. This milestone makes that check a permanent gate so the consumer-facing contract can never silently rot again.

**Non-Goals (explicitly out of scope):**

- **Full browser-bundler *runtime* end-to-end harness.** Spinning up a real webpack/vite/rollup build and executing the bundled output in a browser-like runtime is deliberately deferred — it is the team's existing documented stance (`wasm.yml`: the bundler runtime path "is validated by wasm-bindgen-cli's own pinned test suite and by real consumer integration … it is not exercised end-to-end in this CI workflow"). The two defects that motivated this milestone were *type-contract* defects, fully caught by `tsc`; a runtime bundler harness adds CI flakiness (experimental wasm-ESM loaders, version-sensitive bundler plugins) disproportionate to the residual risk. Recorded as a **Seed Forward** with a concrete recommended approach for when a real browser adopter justifies the maintenance cost.
- **Changing the shipped package shape, the `exports` map, or the wasm-pack targets.** The bundler+nodejs dual build is correct; this milestone validates usage *against* it, it does not modify it.
- **Publishing or version changes.** No `package.json` version bump, no npm publish path changes.

**Deliverables:**

- **Typecheck project (`bindings/sdivi-wasm/tests/typecheck/`):**
  - `tsconfig.json` configured to mirror a realistic *strict* consumer and to match the guarantees the binding README advertises ("Compatible with `--strict --noUncheckedIndexedAccess --exactOptionalPropertyTypes`"):
    - `"strict": true`, `"noUncheckedIndexedAccess": true`, `"exactOptionalPropertyTypes": true`, `"noEmit": true`, `"esModuleInterop": true`, `"skipLibCheck": true`, `"target": "ES2020"`, `"lib": ["ES2020"]`, `"moduleResolution": "bundler"`, `"module": "ESNext"`.
    - Resolve the bare package specifier to the **freshly built** types via tsconfig `paths` (deterministic, location-independent, no symlink needed):
      ```jsonc
      "baseUrl": ".",
      "paths": {
        "@geoffgodwin/sdivi-wasm":         ["../../pkg/bundler/sdivi_wasm.d.ts"],
        "@geoffgodwin/sdivi-wasm/bundler": ["../../pkg/bundler/sdivi_wasm.d.ts"],
        "@geoffgodwin/sdivi-wasm/node":    ["../../pkg/node/sdivi_wasm.d.ts"]
      }
      ```
      (Confirm the relative depth from `tests/typecheck/` to `pkg/` is exactly `../../pkg/…` — `tests/typecheck` → `tests` → `sdivi-wasm` → `pkg`.)
    - `"include"` the two repo-root examples and the negative fixture. Compute the correct relative path from `tests/typecheck/` to the repo-root `examples/` directory (`../../../../examples/binding_node.ts`, `../../../../examples/binding_bundler.ts`) and `"./negative.ts"`. The examples are the **single source of truth** — do not copy them into the typecheck dir.
  - The typecheck must pass cleanly (`tsc --noEmit -p bindings/sdivi-wasm/tests/typecheck/tsconfig.json` → exit 0) once the corrected examples are in place. This validates that the real consumer code matches the shipped types under strict settings.

- **Negative / self-verifying fixture (`bindings/sdivi-wasm/tests/typecheck/negative.ts`):**
  - A `.ts` file using `// @ts-expect-error` to assert that each previously-shipped broken pattern *fails* to typecheck. This gives the guard teeth: if a future change re-introduces a default `init` export, loosens a `Map` input to an object, or otherwise relaxes the contract, the `@ts-expect-error` becomes unused and `tsc` fails (TS error 2578 "Unused '@ts-expect-error' directive"). Cover at minimum:
    1. `import init from '@geoffgodwin/sdivi-wasm'` followed by `await init()` — assert the call is rejected (no callable default export). Place the `@ts-expect-error` on the offending line; pick the construction that produces the error under `esModuleInterop: true` (the synthesized default is the module namespace, which is not callable → "This expression is not callable").
    2. Passing `edge_weights` (and one other `Map` input, e.g. `overrides` on `WasmThresholdsInput`) as a plain object literal — assert the object→`Map` mismatch errors.
    3. Reading a `Map` output by bracket index (e.g. `boundaries.cluster_assignments['x']`) — assert it errors under the `Map` type.
  - Add a short header comment explaining the file's purpose and that an "unused @ts-expect-error" failure means the public contract changed and the docs/examples (and likely this fixture) must be revisited.

- **Forbidden-pattern doc lint (`bindings/sdivi-wasm/tests/check_docs.sh`, or an inline `wasm.yml` step):**
  - A small, dependency-free guard (POSIX `sh` + `grep`) that fails if any consumer-facing doc reintroduces the exact regressions:
    - greps `bindings/sdivi-wasm/README.md`, root `README.md`, `docs/pattern-categories.md`, `bindings/sdivi-wasm/src/lib.rs`, and `examples/*.ts` for `import init` / `await init(` and fails if found (the corrected files only ever say "no `init()`…").
    - greps the same set for `edge_weights: {` (object-literal form) and fails if found.
  - Keep the matched file list and forbidden patterns in one place at the top of the script with a comment, so it is obvious what is being locked and why. Emit a clear `FAIL: <file>:<line>` message on a hit. This is belt-and-suspenders: `tsc` catches the example code, the grep catches prose snippets that `tsc` cannot (the README's partial fences reference undeclared identifiers and cannot be typechecked standalone).

- **`wasm.yml` CI wiring:**
  - Add steps (ubuntu-latest only, consistent with the existing Node-smoke gating) that run after the `pkg/` build + `pkg/package.json`/`.d.ts` are present:
    - Set up Node.js 20 (a setup-node step already exists for the smoke tests — reuse/extend it; do not duplicate).
    - Install a **pinned** TypeScript via `npm install --no-save typescript@<pinned>` (pin an exact version in an `env:` key, e.g. `TYPESCRIPT_VERSION: "5.5.4"`, mirroring how `WASM_PACK_VERSION` / `WASM_BINDGEN_VERSION` are pinned). Run `npx tsc --noEmit -p bindings/sdivi-wasm/tests/typecheck/tsconfig.json`.
    - Run the forbidden-pattern doc lint.
  - Both steps must run on the bundler+node `pkg/` that the same job just built — they depend on the generated `.d.ts`, so they must be ordered after the build + "Assemble pkg/package.json" steps and before/after the smoke tests (either ordering is fine as long as `pkg/bundler/sdivi_wasm.d.ts` and `pkg/node/sdivi_wasm.d.ts` exist).
  - These steps are **required** (no `continue-on-error`). A typecheck failure or a forbidden-pattern hit fails the WASM workflow.

- **Reconcile stale "not exercised" comments:**
  - Update the header comment in `bindings/sdivi-wasm/tests/node_smoke/index.mjs` and any matching note in `wasm.yml` that claims the bundler path is unexercised, to state accurately that the bundler **type contract** is now validated by the `tsc` guard (examples + negative fixture against the generated `.d.ts`), while the bundler **runtime** path remains covered by upstream wasm-bindgen tests and real consumer integration (the Non-Goal above). Do not overstate: the new guard is type-level, not a runtime bundler e2e.

- **CHANGELOG.md** entry (under the Unreleased / next-release section, "Added"):
  > **CI:** Added a TypeScript consumer-contract guard for `@geoffgodwin/sdivi-wasm`. The WASM workflow now `tsc --noEmit`-typechecks the published examples against the freshly generated `.d.ts` under strict settings, asserts via a negative fixture that the previously-broken `await init()` and object-as-`Map` patterns fail to typecheck, and lints the consumer docs for those regressions. Prevents the binding's documented usage from drifting away from the shipped types.

- **DRIFT_LOG.md** entry:
  > M47 records that the M12-era `await init()` docs idiom (designed for `--target web`) survived the switch to the bundler+nodejs build and shipped broken, alongside an object-literal-where-`Map`-expected example, because no CI step typechecked the consumer surface against the generated `.d.ts`. Closed by a strict `tsc` guard over the examples + a self-verifying negative fixture + a forbidden-pattern doc lint. Bundler *runtime* e2e remains intentionally deferred to upstream wasm-bindgen tests + real consumer integration (see Seeds Forward).

**Migration Impact:** None. No library code, no public Rust/TS API, no `Snapshot`/`BoundarySpec` schema, no config keys, no package shape. `snapshot_version` stays `"1.0"`. The only additions are test-infrastructure files and CI steps. No `MIGRATION_NOTES.md` entry required (no breaking change).

**Files to create or modify:**

- **New** `bindings/sdivi-wasm/tests/typecheck/tsconfig.json` — strict consumer tsconfig with `paths` → built `.d.ts` and `include` → repo-root examples + negative fixture.
- **New** `bindings/sdivi-wasm/tests/typecheck/negative.ts` — `@ts-expect-error` assertions locking the broken patterns.
- **New** `bindings/sdivi-wasm/tests/check_docs.sh` — forbidden-pattern doc lint (or inline the equivalent as a `wasm.yml` run step; prefer the script so it is locally runnable).
- **Modify** `.github/workflows/wasm.yml` — pin `TYPESCRIPT_VERSION`; add the typecheck + doc-lint steps (ubuntu-latest), ordered after `pkg/` assembly; reconcile any stale comment.
- **Modify** `bindings/sdivi-wasm/tests/node_smoke/index.mjs` — correct the stale "bundler path not exercised" header comment.
- **No change expected** to `examples/binding_node.ts`, `examples/binding_bundler.ts`, or the corrected docs — they are the inputs the guard validates. If `tsc` surfaces a genuine error in them, fix the example (it is consumer-facing) and note it; do not relax the tsconfig to make a real error disappear.

**Acceptance criteria:**

- A local `wasm-pack build --target bundler --release --out-dir pkg/bundler` + `--target nodejs … --out-dir pkg/node` followed by `npx tsc --noEmit -p bindings/sdivi-wasm/tests/typecheck/tsconfig.json` exits `0` with the corrected examples in place.
- The negative fixture compiles cleanly **only because** every `@ts-expect-error` is consumed: temporarily deleting any one `@ts-expect-error` line makes `tsc` fail with TS2578 (proves each guarded pattern genuinely errors). Verify at least one such deletion manually during implementation; do not commit the deletion.
- `bindings/sdivi-wasm/tests/check_docs.sh` exits `0` on the current (corrected) tree, and exits non-zero with a clear `FAIL:` message if a `import init` / `await init(` / `edge_weights: {` pattern is reintroduced (verify by a temporary local edit, reverted before commit).
- The `wasm.yml` workflow passes end-to-end on the milestone PR within the existing 30-minute job cap, including the new typecheck and doc-lint steps, on ubuntu-latest. macOS job behavior is unchanged (the new steps are ubuntu-gated, matching the existing Node-smoke steps).
- TypeScript is pinned to an exact version via an `env:` key; the typecheck does not run an unpinned `typescript@latest`.
- No change to `cargo test`, `cargo clippy`, `cargo fmt`, or any Rust gate (this milestone touches no Rust). The existing workspace test suite and the existing `node_smoke` / `wasm-pack test --node` steps continue to pass unchanged.
- `git status` is clean of any stray build output: the typecheck consumes the CI-built `pkg/` (which is gitignored per the M47-adjacent cleanup); no `pkg/` artifacts, `node_modules/`, or `tsconfig.tsbuildinfo` are committed. Add `tsconfig.tsbuildinfo` to `.gitignore` if `tsc` emits one despite `noEmit`.

**Tests:**

- **Positive typecheck (`tests/typecheck/` via `tsc --noEmit`):** the two corrected examples typecheck under `--strict --noUncheckedIndexedAccess --exactOptionalPropertyTypes`. This is the primary regression gate — it would have caught both shipped defects (the `await init()` call is not callable against the namespace default; the `edge_weights` object literal is not assignable to `Map<string, number>`).
- **Negative typecheck (`tests/typecheck/negative.ts`):** `@ts-expect-error`-guarded assertions that `await init()`, object-as-`Map` inputs, and bracket-indexing a `Map` output all fail. Self-verifying: the file only compiles if every guarded line genuinely errors.
- **Doc lint (`check_docs.sh`):** asserts the corrected consumer docs/examples contain no `import init` / `await init(` / `edge_weights: {` occurrences, across the five enumerated paths. Run in CI and locally.
- **CI integration:** the new `wasm.yml` steps run on the freshly built dual-target `pkg/` and gate the workflow. Confirm ordering: the typecheck step must come after `pkg/bundler/sdivi_wasm.d.ts` and `pkg/node/sdivi_wasm.d.ts` exist.
- **No new Rust tests** — this milestone adds no Rust surface. Do not add `#[cfg(test)]` blocks or `crates/**/tests/*.rs` files.

**Watch For:**

- **`exactOptionalPropertyTypes` vs the optional `commit`/`boundary_count`/`leiden_seed` fields.** `examples/binding_node.ts` passes `commit: undefined` and `boundary_count: undefined` explicitly to `assemble_snapshot`. Under `exactOptionalPropertyTypes: true`, an optional field typed `T` (i.e. `T | undefined` only via `?`) may reject an explicit `undefined` depending on how `tsify` emits the field (`prop?: T` accepts explicit `undefined`; `prop: T | undefined` requires it). Check the generated `.d.ts` for these fields (they were emitted as `commit?: string`, `boundary_count?: number`, `leiden_seed?: number`, i.e. `?`-optional, which *does* accept explicit `undefined`). If `tsc` complains, the correct fix is to **omit** those keys in the example rather than passing `undefined`, and update the example accordingly (it stays a valid, more idiomatic consumer example). Do not disable `exactOptionalPropertyTypes` — the README explicitly advertises compatibility with it.
- **`paths` resolution does not validate the `exports` map.** The tsconfig `paths` approach maps the bare specifier straight to the `.d.ts`, bypassing the package.json `exports`/`types` conditions. That is acceptable here — the `exports` map's *runtime* resolution is already validated by the `node_smoke` CJS/ESM tests, and this guard's job is to validate consumer *usage* against the *types*. Do not try to also validate the exports map through `tsc` in this milestone; if that is wanted later it belongs in a runtime test, not the typecheck. Note this trade-off in a comment in the tsconfig.
- **Relative-path depth from `tests/typecheck/` to `examples/` and `pkg/`.** Easy to miscount. `tests/typecheck/` → repo root is four levels up (`typecheck`→`tests`→`sdivi-wasm`→`bindings`→root), so examples are `../../../../examples/…`; `pkg/` is inside `bindings/sdivi-wasm/`, two levels up (`../../pkg/…`). Verify by running `tsc` locally — a wrong path yields "Cannot find module" / "File not found in include", not a silent skip.
- **`tsc` must run against a built `pkg/`, which is gitignored.** `pkg/` is regenerated by `wasm-pack` in the same CI job and is not committed (per the repo's `.gitignore`). The typecheck cannot run before the build steps. Locally, contributors must run `./build.sh` (or `--dev`) first; document this in a one-line comment at the top of the tsconfig or in the binding README's "Building locally" section.
- **Pin TypeScript exactly.** An unpinned `typescript@latest` would make CI non-deterministic — a new TS release could change inference and flip the typecheck (or the negative fixture's expected errors) without any repo change. Pin via `env:` exactly as `wasm-pack`/`wasm-bindgen` are pinned. When bumping TS later, re-verify the negative fixture's `@ts-expect-error` lines still fire (newer TS may change error grouping).
- **Negative fixture brittleness.** `@ts-expect-error` suppresses *whatever* error occurs on the next line, so a fixture line could pass for the wrong reason (e.g. an unrelated typo also errors). Where practical prefer `// @ts-expect-error` immediately above a line whose *only* plausible error is the contract violation, and add a trailing comment naming the expected error. Keep each negative case to a single, minimal statement.
- **Do not let the guard pass vacuously.** If a path typo makes `tsc` include zero files, it exits `0` and the guard is dead. The positive examples reference enough of the real API that an empty include would instead error on the negative fixture's `@ts-expect-error` (unused). As a deliberate safeguard, ensure the negative fixture is always included so a vacuous run fails loudly.
- **Stale-comment honesty.** When reconciling the `node_smoke`/`wasm.yml` comments, state the new coverage precisely: *type-level* contract validation of the bundler + node surfaces. Do not claim runtime bundler e2e — that remains a Non-Goal, and an overstated comment would mislead the next reviewer the same way the original docs did.
- **`.gitignore` for tsc byproducts.** Even with `noEmit`, `tsc` can write `tsconfig.tsbuildinfo` when `incremental`/composite is implied. Set `"incremental": false` or add `tsconfig.tsbuildinfo` (and any `node_modules/` created by the `npm install --no-save`) to `.gitignore` so nothing stray is committed.

**Seeds Forward:**

- **Bundler runtime end-to-end harness.** When a real browser/bundler adopter justifies the maintenance cost, add a runtime smoke that bundles `examples/binding_bundler.ts` with a *pinned* bundler and executes it, asserting expected output. Recommended approach: a pinned `vite build` (library mode) with `vite-plugin-wasm` + `vite-plugin-top-level-await`, or `esbuild` with a pinned wasm plugin, run under Node and asserting a known `list_categories()` count / a `normalize_and_hash` digest. Gate it as its own job so its flakiness cannot block the deterministic Rust/typecheck gates, and pin every tool version. Defer until the residual risk (bundler-specific runtime wiring, distinct from the type contract this milestone locks) is demonstrated by an actual consumer.
- **Typecheck the README code fences directly.** The forbidden-pattern grep is a blunt lock on known regressions. A future milestone could add a markdown-fence extractor that pulls ```ts blocks from the binding README and typechecks the *self-contained* ones (those that declare their own `graph`/`summary`), turning the docs themselves into compiled artifacts. Non-trivial because most fences are intentionally partial; scope it only if doc-fence drift recurs.
- **Matrix the consumer tsconfig across `moduleResolution` modes.** Real consumers use `"node16"`, `"nodenext"`, and `"bundler"` resolution, which differ in how they honor `exports`/`types`. A follow-up could run the typecheck under each mode to catch resolution-specific breakage (e.g. a consumer on `node16` who must use the explicit `/node` subpath). Defer until a consumer reports a resolution-mode-specific failure.
- **Promote the examples to a runnable, asserted Node test.** `examples/binding_node.ts` is currently illustrative. A follow-up could execute it under `tsx` in CI (resolving via the `node` condition against the built `pkg/`) and assert its console output, turning the canonical example into an executable contract alongside the type-level guard. Lower priority than the typecheck because the node *runtime* surface is already smoke-tested.

---
