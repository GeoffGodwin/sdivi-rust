#### Milestone 24: Node.js WASM Distribution Target

<!-- milestone-meta
id: "24"
status: "pending"
-->

**Scope:** Ship `@geoffgodwin/sdivi-wasm` as a single npm package with conditional exports — `./bundler` for webpack/vite consumers (current default), `./node` for Node 18+ CLI consumers (Meridian's case). Updates `bindings/sdivi-wasm/build.sh` to produce both `wasm-pack` outputs, the npm `package.json` `exports` map to route by environment, and adds a Node-import smoke test to CI. Does not bump the package's major version because the existing `bundler` import path remains the default.

**Why this milestone exists:** The current `bindings/sdivi-wasm/build.sh` invokes `wasm-pack --target bundler`, which produces output that webpack and vite consume natively but requires a manual `WebAssembly.instantiate` shim to run under Node.js. Meridian runs as a Node CLI without a bundler in the loop and currently has to hand-write that shim — fragile and friction. The cleanest fix is to publish both targets under one package and let Node.js callers `require('@geoffgodwin/sdivi-wasm/node')` while bundler callers continue with `import from '@geoffgodwin/sdivi-wasm'`.

**Deliverables:**
- Update `bindings/sdivi-wasm/build.sh` to produce both targets:
  - `wasm-pack build --release --target bundler --out-dir pkg/bundler` (existing path, moved to subdir).
  - `wasm-pack build --release --target nodejs --out-dir pkg/node`.
  - Both outputs share the same Rust source; only the JS shim differs.
- Restructure `pkg/` (the npm publish root) to host both subdirs and a top-level `package.json` with conditional exports:
  ```json
  {
    "name": "@geoffgodwin/sdivi-wasm",
    "version": "...",
    "main": "./bundler/sdivi_wasm.js",
    "module": "./bundler/sdivi_wasm.js",
    "types": "./bundler/sdivi_wasm.d.ts",
    "exports": {
      ".":      { "import": "./bundler/sdivi_wasm.js", "require": "./node/sdivi_wasm.js", "types": "./bundler/sdivi_wasm.d.ts" },
      "./node": { "require": "./node/sdivi_wasm.js", "types": "./node/sdivi_wasm.d.ts" },
      "./bundler": { "import": "./bundler/sdivi_wasm.js", "types": "./bundler/sdivi_wasm.d.ts" }
    },
    "files": ["bundler/", "node/", "README.md", "LICENSE"]
  }
  ```
  The `.` conditional exports route Node `require` to the nodejs build automatically without the consumer specifying `/node` — the explicit `/node` and `/bundler` subpaths remain for callers who want to be explicit.
- Update the npm publish step in the existing release workflow (`release.yml`, the M13 publish job) to publish the restructured `pkg/` rather than the single-target output.
- Add a CI smoke test under `bindings/sdivi-wasm/tests/node_smoke/`: a small Node 18+ project (`package.json` + `index.cjs` + `index.mjs`) that does `require('@geoffgodwin/sdivi-wasm')` (CommonJS path) and `import from '@geoffgodwin/sdivi-wasm'` (ESM path) and calls `list_categories()`. Run in CI on `ubuntu-latest` after a successful build.
- Update `bindings/sdivi-wasm/README.md` with two usage sections: "Bundler consumers (webpack, vite, rollup)" and "Node.js consumers (CLI, server)". Each shows the import pattern plus a runnable snippet.
- Verify the existing `wasm.yml` CI workflow continues to build cleanly. If it currently builds only the bundler target, extend it to build both.

**Migration Impact:** Existing bundler consumers see no change — the default `import from '@geoffgodwin/sdivi-wasm'` continues to resolve to the bundler build (now under `pkg/bundler/`). Node consumers who were hand-writing a `WebAssembly.instantiate` shim can delete it and switch to the standard `require`. The package's major version does not bump because the public interface is strictly additive (a new `/node` subpath and a new `require` resolution); the existing default path remains the bundler build. `snapshot_version` is unaffected.

**Files to create or modify:**
- **Modify:** `bindings/sdivi-wasm/build.sh` — produce both targets.
- **Create:** `bindings/sdivi-wasm/pkg-template/package.json` — the conditional-exports template (rendered or copied into `pkg/` after build).
- **Modify:** `.github/workflows/release.yml` — publish the restructured pkg/.
- **Modify:** `.github/workflows/wasm.yml` — build both targets in CI; run the Node smoke test.
- **Create:** `bindings/sdivi-wasm/tests/node_smoke/package.json`, `index.cjs`, `index.mjs` — minimal smoke-test project.
- **Modify:** `bindings/sdivi-wasm/README.md` — dual-target usage docs.
- **Modify:** `CHANGELOG.md` — under Added.

**Acceptance criteria:**
- Running `bindings/sdivi-wasm/build.sh` produces `pkg/bundler/sdivi_wasm.js` AND `pkg/node/sdivi_wasm.js` (and the corresponding `.d.ts` and `.wasm` files for each).
- The `pkg/package.json` `exports` map resolves correctly: in a CI smoke test, `node -e "require('@geoffgodwin/sdivi-wasm')"` succeeds (uses node target via `require` conditional); `node --input-type=module -e "import('@geoffgodwin/sdivi-wasm').then(m => m.list_categories())"` succeeds (uses bundler target via `import` conditional, which works on Node 18+ with synchronous WASM).
- The existing bundler consumer test (whatever asserts that vite/webpack can consume the package) continues to pass.
- The `wasm.yml` CI workflow runs both builds and the Node smoke test on `ubuntu-latest`. Total wall-clock under 5 minutes; if it grows past that, investigate.
- `npm pack --dry-run` from the `pkg/` directory lists both `bundler/` and `node/` subdirectories in the tarball contents. No extraneous files.
- `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` continue to pass (no Rust source changes; this is a packaging milestone).
- The `.tekhton/DESIGN.md` (or wherever distribution-target decisions live) is updated to reflect the dual-target shape.

**Tests:**
- CI smoke (`tests/node_smoke/`): CommonJS `require` succeeds, returns a working module with `list_categories` callable.
- CI smoke (`tests/node_smoke/`): ESM `import` succeeds, returns a working module with `list_categories` callable.
- CI smoke: assert `list_categories()` returns the same array under CJS and ESM (sanity — same underlying wasm).
- Bundler test: existing webpack/vite/rollup smoke (whichever the M12/M13 era set up) continues to pass against the new `pkg/bundler/` path.
- Manual (release dry run): `npm pack` produces a tarball; extract and verify the `exports` map matches the template.

**Watch For:**
- **Conditional `exports` precedence on Node 18+.** Node resolves `require` and `import` conditions independently of the `main` / `module` legacy fields. Verify on Node 18, 20, 22 (LTS line). Older Node versions (<18) ignore the `exports` map and fall back to `main`, getting the bundler build — they need to use the explicit `/node` subpath. Document the Node-18 minimum.
- **`wasm-pack --target nodejs` produces a CJS module.** It uses `require('fs')` to load the `.wasm` file synchronously. Webpack/vite consumers must NOT use this build (they expect ESM with import-based wasm loading). The conditional exports map prevents this — verify nothing leaks.
- **`wasm-pack --target bundler` does NOT work in Node.js by default.** The bundler target uses `import.meta.url` style loading that Node 18+ supports for ESM but with caveats. The `.` `import` conditional points to the bundler build for Node ESM users; the `.` `require` conditional points to the node build for Node CJS users. Test both.
- **The `.d.ts` files differ slightly between targets.** `wasm-pack` generates target-specific TypeScript declarations. The `types` field in the conditional export should point to the bundler `.d.ts` for the default `.` import (TS resolves `types` from the `import` condition's directory) and the node `.d.ts` for the `/node` subpath. Verify with `tsc --noEmit` against both consumption patterns.
- **Package size budget.** Two builds doubles the published package size. Verify the published tarball stays under a sensible cap (e.g. 5 MB for the combined wasm + JS shims). If it bloats, investigate `wasm-opt` settings.
- **Don't publish a separate npm package.** A single package with conditional exports is the modern (post-Node 14) idiomatic way; avoid `@geoffgodwin/sdivi-wasm-node` as a sibling. Single-package keeps versioning aligned and reduces the matrix consumers need to track.
- **The `release.yml` manual approval gate.** This milestone changes what gets published — re-verify the gate prompt clearly indicates "both bundler and node targets" so the approver isn't surprised by a 2× size jump.

**Seeds Forward:**
- A `--target deno` build is a plausible v1 follow-up if a Deno consumer appears. Same conditional-exports machinery extends naturally with a `"deno"` condition.
- Cloudflare Workers / edge-runtime consumers may need yet another target (`--target web` with specific glue). Defer until requested.
- If the smoke-test surface grows, factor it into its own minimal npm package under `bindings/sdivi-wasm/tests/` rather than inlining; for v0 the inline `index.cjs` + `index.mjs` is sufficient.

---
