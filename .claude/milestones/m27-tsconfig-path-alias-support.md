#### Milestone 27: tsconfig Path Alias Support

<!-- milestone-meta
id: "27"
status: "pending"
-->

**Scope:** Add support for TypeScript / JavaScript path aliases declared in `tsconfig.json` (or `jsconfig.json`) so specifiers like `@/lib/foo` and `~components/Button` resolve to the correct files. The resolver added in M26 handles relative paths and external packages but treats any unrecognized prefix as external. Modern Next.js, Vite, Nx, and bare-tsc projects commonly use `compilerOptions.paths` (with optional `baseUrl`) to define short aliases — and bifl-tracker is one such project (`paths: { "@/*": ["./*"] }`). Without alias support, every `@/foo` import in such codebases is dropped, leaving a noticeable hole in the dependency graph even after M25 + M26 land.

**Why this milestone exists:** After M25 (adapters emit specifiers) and M26 (resolver navigates parents), bifl-tracker's edge count will jump from 0 to "most of the way there" — but `import { foo } from "@/lib/util"` still fails to resolve because the resolver has no understanding of `@/`. tsconfig path aliases are a structural property of the project (declared in checked-in config), not a runtime concern, so they belong in static analysis. The `tsc`, `node --experimental-...`, `webpack`, `vite`, `next`, and `tsx` resolvers all honor them; SDIVI should too. Skipping aliases means SDIVI's graph for any post-2020 TS project is incomplete by design — and "incomplete by design" is the same failure mode as the M25/M26 bugs, just less obvious because aliases are a smaller fraction of imports than relatives.

**Theoretical basis:** `tsconfig.json` is a strict subset of JSON (with comments — JSON-with-comments / JSONC). The relevant fields are:

- `compilerOptions.baseUrl` (optional, string): all module resolution starts from this directory, resolved relative to the `tsconfig.json` location.
- `compilerOptions.paths` (optional, object: pattern → array of target patterns): each key is a pattern with at most one `*`; each value is a list of substitution patterns also with at most one `*`. The first matching alias wins; within an alias, the first existing target wins.

Examples:
- `"@/*": ["./*"]` (bifl-tracker): `@/lib/foo` → `./lib/foo` relative to the tsconfig.
- `"@components/*": ["src/components/*", "vendor/components/*"]`: `@components/Button` tries `src/components/Button` first, falls back to `vendor/components/Button`.
- `"~lib": ["./src/lib/index.ts"]` (no `*`): exact-match alias.

`tsconfig` `extends` chains are out of scope for this milestone — projects that use them are rare in the bifl-tracker validation set, and full chain resolution is its own can of worms (multiple files, transitive merges, Node-resolve semantics for the extended path). If extends becomes blocking, file a follow-up milestone.

**Deliverables:**

- **Locate and parse `tsconfig.json` / `jsconfig.json`:**
  - At graph-build time (`build_dependency_graph` in `sdivi-graph/src/dependency_graph.rs`), look for `tsconfig.json` at the repo root. If absent, look for `jsconfig.json`. If both absent, no aliases are configured — the resolver behaves exactly as M26.
  - Parse with a JSONC-tolerant parser. The workspace already pulls `serde_json`; line and block comments are not legal JSON. Two options:
    - (A) Strip comments with a small regex/state-machine pre-pass before `serde_json::from_str`. Lightweight, no new dep.
    - (B) Add `jsonc-parser` or `serde_jsonc` as a dep. Slightly heavier; better corner-case handling.
  - Prefer option (A) — the comment grammar is small and a 30-line state-machine handles it. Document the chosen approach in rustdoc.
  - Define a typed view: `struct TsConfigPaths { base_url: Option<PathBuf>, mappings: BTreeMap<String, Vec<String>> }`. Populate from `compilerOptions.baseUrl` and `compilerOptions.paths`. Resolve `base_url` relative to the tsconfig file's directory.
  - On parse failure (malformed JSON even after comment-strip, or non-JSON), log a `WARN` to stderr ("tsconfig.json present but unparseable; alias resolution disabled") and proceed without aliases. Do not fail the snapshot — Rule 15 (warning, not crash) extends naturally to this case.

- **Add `resolve_tsconfig_alias` to `sdivi-graph/src/dependency_graph.rs`:**
  - Signature: `fn resolve_tsconfig_alias(specifier: &str, paths: &TsConfigPaths, path_to_node: &BTreeMap<PathBuf, NodeIndex>) -> Vec<NodeIndex>`.
  - Iterate `paths.mappings` in **insertion order from the config** (not BTree order — tsc honors longest-prefix-match, but the dominant convention is "first match wins" and the spec's most-specific rules can come later if needed). Update the `TsConfigPaths` struct to use an ordered map (`Vec<(String, Vec<String>)>`) to preserve insertion order from the JSON parse.
  - For each pattern key, attempt match: split key on `*` (at most one `*` allowed by spec). If exact (no `*`) and `specifier == key`, match with empty capture. If pattern `prefix*` and `specifier.starts_with(prefix)`, capture is `&specifier[prefix.len()..]`. If pattern `prefix*suffix`, capture is the middle. (One `*` total — TypeScript's spec.)
  - On match, iterate the target list. For each target, substitute the capture for the target's `*` (if any) and join with `base_url` (if present) or with the tsconfig directory (if `base_url` absent). Pass the resulting path through the same file-vs-directory-index lookup logic as `resolve_relative`. First target that resolves to a node wins.
  - Return `Vec<NodeIndex>` (matching M26's pluralized resolver API). Empty Vec means no alias matched or no alias target resolved.

- **Wire into the resolver dispatch:** In `resolve_imports` (M26's renamed function), for `language in ("typescript", "javascript")` and a specifier that does **not** start with `./`, `../`, or `/` (an absolute path), try `resolve_tsconfig_alias` *before* declaring the import external. Order:
  1. Relative (`./`, `../`) → `resolve_relative`.
  2. tsconfig alias match (specifier matches a `paths` key pattern) → `resolve_tsconfig_alias`.
  3. Otherwise → external (return empty `Vec`).

- **Tests:**
  - Unit test in `sdivi-graph/tests/tsconfig_alias.rs` (new): build a synthetic `path_to_node` and `TsConfigPaths`, exercise:
    - Exact alias (`"~lib": ["./src/lib/index.ts"]`).
    - Wildcard alias (`"@/*": ["./*"]`) with several specifiers.
    - Multi-target fallback (`"@x/*": ["a/*", "b/*"]`) where only `b/foo.ts` exists.
    - Pattern with prefix and suffix (`"#int/*.types": ["src/types/*.types.ts"]`).
    - No match (specifier doesn't match any pattern) → empty Vec.
    - Match but target doesn't exist → empty Vec, falls through to external.
  - Integration test: drop a fixture under `tests/fixtures/tsconfig-alias/` with `tsconfig.json`, a few `.ts` files using `@/`, and assert pinned edge counts via `Pipeline::snapshot`.
  - Bifl-tracker validation: run `Pipeline::snapshot` on the user's bifl-tracker (or a copy) and confirm `@/...` imports now resolve. This is a manual verification step in the M27 PR description, not an automated test.

**Migration Impact:** Edge counts increase on TS/JS projects that use path aliases — typically by 10–40% on top of the M25+M26 baseline, depending on how alias-heavy the codebase is. `snapshot_version` stays `"1.0"`. For projects without `tsconfig.json` / `jsconfig.json`, behavior is unchanged. Update `CHANGELOG.md` and combine the M25-M26-M27 re-baseline guidance into a single `MIGRATION_NOTES.md` entry — telling adopters who upgrade across all three milestones to expect a ~5-50× edge count increase on TS/JS projects, and to use a 1-2 week threshold override during the cutover.

**Files to create or modify:**

- **Modify:** `crates/sdivi-graph/src/dependency_graph.rs` — add tsconfig discovery, JSONC-tolerant parse, `TsConfigPaths` struct, `resolve_tsconfig_alias`, dispatch wiring. Cache the parsed config on the graph builder so it's read once per snapshot, not once per import.
- **Modify:** `crates/sdivi-graph/Cargo.toml` — no new deps if option (A) is chosen; otherwise add `jsonc-parser` or equivalent. Verify any new dep compiles for `wasm32-unknown-unknown` (this code is feature-gated behind `pipeline-records` and not in `sdivi-core`'s WASM build, so a WASM-incompatible JSONC dep is acceptable — but flag it explicitly in the PR description).
- **Create:** `crates/sdivi-graph/tests/tsconfig_alias.rs` — unit tests for alias resolution.
- **Create:** `tests/fixtures/tsconfig-alias/tsconfig.json` plus a few `.ts` files — integration fixture.
- **Modify:** `tests/full_pipeline.rs` (or wherever the workspace integration tests live) — add a case using the new fixture.
- **Modify:** `CHANGELOG.md` — under **Added**: "tsconfig.json / jsconfig.json `compilerOptions.paths` alias resolution."
- **Modify:** `docs/cli-integration.md` and/or `docs/library-embedding.md` — note alias support and the (deferred) `extends` limitation.
- **Modify:** `MIGRATION_NOTES.md` — combined M25+M26+M27 re-baseline note.

**Acceptance criteria:**

- New unit and integration tests pass.
- `cargo test --workspace` continues green.
- `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` pass.
- `cargo build --target wasm32-unknown-unknown -p sdivi-core --no-default-features` continues to succeed (this milestone is in `sdivi-graph` behind `pipeline-records`, not in `sdivi-core`'s WASM build).
- Snapshotting the bifl-tracker fixture (or a copy of the user's bifl-tracker) shows `@/`-aliased edges resolved — verifiable by inspecting the snapshot JSON for edges crossing alias-resolved boundaries.
- Determinism preserved: two runs against the same repo produce a bit-identical snapshot, including the parsed alias map's contribution to edges.
- A repo with malformed `tsconfig.json` produces a stderr `WARN` and a successful snapshot (alias resolution disabled, other resolution unaffected).
- A repo with no `tsconfig.json` produces no warnings and a successful snapshot (M27 silently does nothing).

**Tests:**

- All unit tests in `tsconfig_alias.rs`.
- Integration via `tests/fixtures/tsconfig-alias/`.
- Property test: random alias patterns + random specifiers; resolver either resolves to a path in `path_to_node` or returns empty Vec — no panics, no non-deterministic outcomes.
- Negative test: `tsconfig.json` declared with a pattern that has two `*`s (illegal per spec) — log `WARN`, skip that pattern, continue.
- Determinism test: same alias map, same specifiers, two runs produce identical edge lists in identical order.

**Multi-Language Regression Guarantees:**

M27 is the most surgically scoped of the M25–M27 trio:

- **Only the `language in ("typescript", "javascript")` dispatch arm is modified.** Rust, Python, Go, Java resolution code paths are not touched. A Rust file with a `crate::foo::bar` import goes through the same M26 resolver logic before and after M27.
- **No effect when tsconfig is absent.** The `TsConfigPaths` struct is only built if a `tsconfig.json` or `jsconfig.json` exists at the repo root. When absent, `resolve_tsconfig_alias` is never invoked, and the resolver behaves identically to M26 for all 6 languages — including TS/JS.
- **Layered before "external" determination, not before relative resolution.** The dispatch order for TS/JS in M26 is: relative (`./`/`../`) → external. M27 inserts alias resolution between the two: relative → alias → external. Specifiers that match `./` or `../` never reach the alias step; specifiers that don't match any alias still fall through to "external" as in M26. Pure-additive on the resolution path.
- **JSONC parser is feature-gated to `pipeline-records`.** Tsconfig parsing happens only inside `build_dependency_graph` which is `#[cfg(feature = "pipeline-records")]`. The `sdivi-core` WASM build path doesn't see it — Rule 21 is preserved by construction.
- **Existing TS/JS fixtures (`tests/fixtures/simple-typescript`, `simple-javascript`) must produce identical edge counts pre- and post-M27** (assuming they don't have a `tsconfig.json`). Add this as a test sentinel.
- **Per-language baselines pinned in M26 are extended for M27.** The `per_language_baselines.rs` file gains a 7th entry (`tsconfig-alias` fixture). The other 6 baselines are byte-identical between M26 and M27 as a regression guard.

**Watch For:**

- **JSONC corner cases.** `// line comments`, `/* block comments */`, trailing commas in objects/arrays. The state-machine pre-pass must handle: comments inside strings (don't strip), strings inside comments (don't terminate the comment early), escape sequences (`\"` inside strings). A small property test that strips comments and then re-parses the unstripped JSON's string literals is a good guard.
- **`baseUrl` interaction with path aliases.** When `baseUrl` is set and `paths` is also set, alias resolution starts from `baseUrl`. When only `baseUrl` is set, *every* non-relative import is resolved against `baseUrl` first — this is technically separate from `paths` but commonly used together. Decision for M27: only resolve specifiers that match a `paths` key. Pure `baseUrl`-only resolution (without an alias) is rare enough to defer; document the limitation in `Watch For` and `Seeds Forward`.
- **`extends` chains.** `tsconfig.json` can extend another tsconfig (e.g. `"extends": "@tsconfig/next/tsconfig.json"` from a node_modules package). Resolving the chain requires walking package.json's, which is a different scope. M27 reads only the literal `compilerOptions.paths` from the root `tsconfig.json`; if the project's actual paths are inherited from a base, those won't be picked up. Document. If a real adopter hits this, file a follow-up milestone.
- **Multiple tsconfig.json files (monorepos).** Per-package `tsconfig.json` under `packages/*/tsconfig.json` is common in pnpm/yarn workspaces. M27 reads only the root `tsconfig.json`. For monorepo support, the resolver would need to find the nearest tsconfig walking up from each importing file. Defer to a follow-up. Document the limitation explicitly: "tsconfig path aliases are read from the repo root tsconfig.json only; per-package tsconfigs in monorepos are not currently supported."
- **The `*` substitution exact semantics.** When `"@/*": ["./*"]` and the specifier is `@/foo/bar.css`, the capture is `foo/bar.css`. Substitution gives `./foo/bar.css`. The `.css` is not in the resolver's extension list. Decision: alias resolution honors any extension if the file exists at exactly that path (no extension append). If the substituted path doesn't exist as-is, then try the standard extension/index list. This matches `tsc`'s behavior closely enough for structural analysis.
- **Determinism (Rule 5 / KDD-10).** The alias map is a `BTreeMap` for serde output ordering — but the resolver matching order is the **insertion order from the JSON file**, not BTreeMap's lexicographic order. Use `Vec<(String, Vec<String>)>` for the matching loop while preserving `BTreeMap` for any serialized representation.
- **Don't add a CLI flag to disable aliases.** If a user wants to skip alias resolution, they can move the tsconfig out of the way. Keeping the surface area small avoids a config-key proliferation.
- **Don't try to honor `.npmrc` / pnpm workspace files / yarn workspaces.** These are package-resolution concerns, not module-resolution. Out of scope.

**Seeds Forward:**

- **`extends` chain support.** When a project's `compilerOptions.paths` is defined in a base tsconfig (common in Next.js / Vite starter kits), M27 won't see it. A follow-up milestone could chase the chain.
- **`baseUrl`-only resolution** (no `paths` block) — the current decision is to skip. If observed in real adopter projects, add as a follow-up.
- **Per-package monorepo tsconfig discovery.** Walk up from each importing file to find the nearest tsconfig. Adds complexity (per-file alias map) but unlocks pnpm/yarn workspace projects. Defer until requested.
- **`package.json` `imports` field** (Node subpath imports, `#alias/foo` style). Parallel to tsconfig aliases but in a different config file. Defer.
- **`vite.config.ts` / `webpack.config.js` resolver aliases.** These are JavaScript files, not JSON; reading them statically is a non-starter. Document as out of scope.

---
