
#### Milestone 48: Post-v0.2.47 CI Greening — WASM Size Optimisation, M47 Typecheck Determinism, Rustdoc Link

<!-- milestone-meta
id: "48"
status: "done"
-->

**Scope:** Restore a fully green CI surface after the M34–M47 batch (released as v0.2.47) shipped with three pre-existing, mutually-masking CI failures, and re-publish the result as v0.2.48. This milestone is documented retroactively (status `done`) so the work is captured in the manifest and future milestone numbering does not collide. Three independent defects are fixed: (1) the WASM `wasm.yml` bundle-size gate, which the new pattern-category regex tables pushed from ~1.75 MB to ~2.06 MB, over the per-target budget; (2) the M47 consumer-surface `tsc --noEmit` typecheck, which only passed where `@types/node` happened to be ambiently resolvable and additionally tripped on a self-referential `@ts-expect-error` token inside a prose comment; and (3) the CI `Docs` job, where `classify_hint`'s rustdoc linked to the private `CALL_DISPATCH` const and failed under `RUSTDOCFLAGS="-D warnings"`. No `snapshot_version`, schema, config, or public-API change. `snapshot_version` stays `"1.0"`.

**Why this milestone exists:** v0.2.47 was tagged and published to crates.io and npm before these three gates were observed to be red on `main`. The failures were latent for two reasons: the size step runs before the typecheck step in `wasm.yml` and aborts the job (masking the typecheck defect), and the initial release sanity check ran `cargo build/test/clippy/fmt` plus the WASM forbidden-dep tree but did **not** replicate `cargo doc` or the WASM workflow's size/typecheck gates. The published artifacts are functionally correct — the failing gates are a size budget, a strict-TS consumer typecheck, and a rustdoc lint, none of which affect snapshot correctness or determinism — but `main` was red and the npm package shipped an unnecessarily large `.wasm`. This milestone makes the gates pass deterministically on every runner and ships the smaller artifact.

**Deliverables:**

- WASM build drives the size-optimised `release-wasm` settings (`opt-level="z"`, `lto="fat"`, `codegen-units=1`) by overriding the `release` profile via job-scoped `CARGO_PROFILE_RELEASE_*` env in both `wasm.yml` (the CI gate) and `release.yml`'s `publish-npm` job (the shipped artifact), sidestepping the wasm-pack 0.13.x `--release`/`--profile` conflict. Measured: bundler and node targets each drop 2,062,372 → 1,531,484 bytes (≈25%, ~300 KB headroom under the 1.75 MB budget). Scoped per-job so the native release binaries keep their profile.
- `tests/typecheck/tsconfig.json` adds `"DOM"` to `lib` so `console` (and other host globals the examples use) resolves deterministically instead of depending on ambient `@types/node`.
- `tests/typecheck/negative.ts` reworded the guard comment so it no longer begins with the literal `@ts-expect-error` token (which TypeScript parsed as a real directive → spurious `TS2578`).
- `crates/sdivi-patterns/src/queries/mod.rs` demotes the intra-doc link `[`CALL_DISPATCH`]` to a plain code span referencing the private registry by name.

**Migration Impact:** None to schema, config, or public API. `Snapshot` stays `"1.0"`. The npm `@geoffgodwin/sdivi-wasm` `.wasm` is ~25% smaller from v0.2.48 onward; behaviour is byte-for-byte equivalent (same wasm-bindgen surface, only the codegen profile differs). No new config keys, no new `pub` items.

**Files to create or modify:**

- `.github/workflows/wasm.yml` — job-level `CARGO_PROFILE_RELEASE_*` env; budget comment refresh.
- `.github/workflows/release.yml` — `publish-npm` job-level `CARGO_PROFILE_RELEASE_*` env.
- `bindings/sdivi-wasm/tests/typecheck/tsconfig.json` — `lib: ["ES2020", "DOM"]`.
- `bindings/sdivi-wasm/tests/typecheck/negative.ts` — comment reword.
- `crates/sdivi-patterns/src/queries/mod.rs` — rustdoc link → code span.
- Version bump to `0.2.48`, `CHANGELOG.md` `[0.2.48]`, `tools/release_notes/v0.2.48.md`.

**Acceptance criteria:**

- `wasm.yml` `Check bundle sizes` passes on ubuntu-latest and macos-latest; the `Cross-platform hash determinism check` runs (no longer skipped) and passes.
- `npx tsc --noEmit -p bindings/sdivi-wasm/tests/typecheck/tsconfig.json` exits 0 with pinned TypeScript 5.5.4, with every real `@ts-expect-error` directive still load-bearing.
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --exclude sdivi-wasm --exclude sdivi-rust --no-deps` exits 0.
- `cargo test --workspace` green (incl. `wasm_package_json_version_matches_workspace` against `0.2.48`).

**Tests:** No new test files. The fixtures themselves (`negative.ts`, `check_docs.sh`) and the existing `workspace_version.rs` are the regression guards. Verification is the green CI run on the merge commit plus the local reproductions captured during the fix.

**Watch For:** The size-opt env must stay **job-scoped** — applying `CARGO_PROFILE_RELEASE_OPT_LEVEL=z` at workflow level in `release.yml` would shrink and slow the native CLI binaries. Do not write the literal `@ts-expect-error` / `@ts-ignore` token at the start of a line comment in any fixture. If categories keep growing, revisit the 1.75 MB budget (or add `wasm-opt -Oz` post-processing) rather than silently raising it.

**Seeds Forward:** A future milestone could tighten the per-target budget toward ~1.6 MB once category growth stabilises, and add an explicit `cargo doc` gate to the release sanity checklist so doc regressions are caught before tagging rather than after.
