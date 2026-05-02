#### Milestone 13: Release Pipeline and Distribution
<!-- milestone-meta
id: "13"
status: "done"
-->


**Scope:** Ship the v0 release. Tag-driven workflow publishes affected crates to crates.io and the WASM bundle to npm, both behind a manual approval gate. Matrix-built binaries attached to GitHub Releases. `cargo audit` weekly. Cut `0.1.0`.

**Deliverables:**
- `.github/workflows/release.yml` with manual approval gate before crates.io and npm pushes
- `cargo dist` or hand-rolled matrix build (Linux x86_64+aarch64, macOS x86_64+aarch64, Windows x86_64) attaching stripped binaries with LTO to GitHub Release
- `CHANGELOG.md` with `0.1.0` entry covering all milestones
- `cargo audit` weekly cron in `.github/workflows/audit.yml`
- Binary size and `.wasm` bundle size tracked in `CHANGELOG.md` per release
- crates.io publish (in dependency order):
  1. `sdivi-config`
  2. six `sdivi-lang-*`
  3. `sdivi-parsing`
  4. `sdivi-graph`
  5. `sdivi-detection`
  6. `sdivi-patterns`
  7. `sdivi-snapshot`
  8. `sdivi-core` (depends on 4–7 with `default-features = false`)
  9. `sdivi-pipeline` (new in M08; depends on 3–8)
  10. `sdivi-cli` (depends on 9 + `sdivi-core` for shared types)
  11. `sdivi-rust` (install-discovery meta-crate)
- npm publish: `@geoffgodwin/sdivi-wasm@0.1.0` on the same tag, behind the same manual approval
- PyO3/napi-rs bindings remain post-MVP / v1 era (see deferred `m12-bindings-pyo3-and-napi-rs-post-mvp.md`)

**Files to create or modify:**
- `.github/workflows/release.yml` (full)
- `CHANGELOG.md` (0.1.0 entry)
- `Cargo.toml` (workspace version bump to `0.1.0`)
- Each crate's `Cargo.toml` populated with `description`, `repository`, `license = "Apache-2.0"`, `readme`, `keywords`, `categories`
- `bindings/sdivi-wasm/package.json` version pinned to `0.1.0`

**Acceptance criteria:**
- Tagging `v0.1.0` triggers the release workflow; crates.io and npm pushes wait on manual approval
- After approval, `cargo install sdivi-rust` from crates.io succeeds and produces a working `sdivi` binary on PATH (binary name comes from `[[bin]] name = "sdivi"` in `sdivi-cli`)
- After approval, `npm install @geoffgodwin/sdivi-wasm` works and the consumer app can `import init, { ... }` successfully against a non-local registry
- GitHub Release page has binaries for all five Tier-1 + Tier-2 platforms
- Binary size and `.wasm` bundle size noted in CHANGELOG
- `cargo audit` cron green
- bifl-tracker validation harness from M11 passes against the tagged commit

**Tests:**
- Dry-run the release workflow on a `v0.1.0-rc.N` pre-tag
- Smoke test: `cargo install --version 0.1.0 sdivi-rust` on each platform; `sdivi --version` reports `0.1.0`
- Smoke test: `npm install @geoffgodwin/sdivi-wasm@0.1.0-rc.N` from a clean Node project; `await init()` and call one export
- `cargo audit` clean

**Watch For:**
- crates.io is append-only — no yanking-as-rollback; once `0.1.0` is published it stays. Validate carefully via the dry run.
- npm is also effectively append-only at the version level (unpublishing is restricted after 72h). Same care applies.
- Manual approval gate must be enforced for both registries — auto-publish on tag is explicitly rejected by DESIGN
- LICENSE in each crate's metadata must say `Apache-2.0`, matching the repo LICENSE; npm `license` field also `Apache-2.0`
- Strip + LTO bloat fix: `[profile.release] lto = "thin"`, `strip = true`, `panic = "abort"` (the last only if no test code unwinds)
- WASM profile is separate: `[profile.release-wasm] inherits = "release", lto = "fat", opt-level = "s"`
- npm scope `@geoffgodwin/` must be claimed and the publish token configured in GitHub Actions secrets before this milestone runs
- Publish ordering: `sdivi-core` before `sdivi-pipeline` (the new dep order); `sdivi-wasm` last (depends only on `sdivi-core` but published to npm, not crates.io, so it doesn't block other crates)

**Seeds Forward:**
- `0.1.0` is the SemVer commitment baseline. Adding `pub` items to `sdivi-core` is now deliberate; removing them requires a major bump to `1.0.0`.
- The release workflow is reused for every subsequent tag, including npm-only patches if WASM ever needs a fast-track fix
- Distribution channels (crates.io + GitHub Releases + npm) are the public commitments. Adding PyPI in a v1 era must not regress these.
- The consumer app becomes a real-world post-release validation source — track its issue intake against `sdivi-wasm` as the first signal of API churn pressure.

---
