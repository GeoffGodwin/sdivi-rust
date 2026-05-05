#### Milestone 26: Resolver — Parent Navigation and Per-Language Module Conventions

<!-- milestone-meta
id: "26"
status: "done"
-->

**Scope:** Rewrite `crates/sdivi-graph/src/dependency_graph.rs::resolve_relative` and `resolve_import` to (a) actually navigate parent directories for `../` and `super::` prefixes, (b) handle Python-style dotted module specifiers as path lookups, (c) handle Go module-path imports against the repository's `go.mod` (when present), and (d) handle Java dotted package specifiers as path lookups. The current resolver strips `../` characters with `trim_start_matches` but never walks up a directory, joins the remainder onto the importer's own directory, and looks for a flat file there — producing zero matches for the vast majority of real-world relative imports. This milestone fixes the resolver itself; M25 fixes what the adapters feed into it; M27 layers tsconfig path aliases on top.

**Why this milestone exists:** The function `resolve_relative` (sdivi-graph/src/dependency_graph.rs:220) reads:

```rust
let from_dir = from_path.parent()?;
let rel = import.trim_start_matches("./").trim_start_matches("../");
for ext in &["rs", "py", "ts", "tsx", "js", "go", "java"] {
    let candidate = from_dir.join(format!("{rel}.{ext}"));
    ...
```

`trim_start_matches` strips the *characters* `./` or `../` from the start, greedily and without bound. For `../lib/foo` from `components/Bar.tsx`, after stripping it becomes `lib/foo`, then the resolver looks for `components/lib/foo.tsx` — wrong; should be `lib/foo.tsx`. For `../../shared/x` from `app/items/Edit.tsx` it strips both `../` and looks for `app/items/shared/x.tsx` — also wrong; should be `shared/x.tsx`. The existing test `relative_import_parent_slash_strips_prefix_resolves_in_same_dir` (sdivi-graph/tests/dependency_graph.rs:152) explicitly pins this broken behavior with the comment `"See BUG note in TESTER_REPORT.md re: parent navigation not implemented"`, confirming the bug was observed during M05/M06 development but never fixed. Compounding this, the resolver only handles two import shapes:

1. Relative paths starting with `./` or `../` (broken as above)
2. Rust `crate::`/`self::`/`super::` prefixes (where `super::` has the same parent-navigation gap as `../`)

It has no handling for Python dotted imports (`foo.bar.baz`), Go module-path imports (`github.com/foo/bar/pkg`), or Java dotted packages (`com.acme.lib.Util`). Even with M25 making the adapters emit clean specifiers, these languages would still produce zero or near-zero edges because the resolver doesn't know how to look up their conventions. This milestone fixes the resolver to handle all four conventions correctly.

**Theoretical basis:** Path resolution is a per-language convention. The fixes here implement the *language-agnostic* core of each language's module resolution algorithm — enough to find the file given the specifier, not enough to fully replicate `tsc`'s or `node`'s resolver (which need full node_modules walking, package.json `exports` maps, etc., far beyond static structural analysis). The contract is: "for any specifier that names a file in the repository, return its `NodeIndex`; for specifiers that name external packages or unresolvable paths, return `None` and let the resolver drop them silently at `DEBUG`." External imports (e.g. `import React from "react"`, `import "fmt"`) deliberately produce no edges — they're not part of the structural graph.

**Deliverables:**

- **`resolve_relative` parent navigation rewrite:**
  - Count leading `../` segments (and `./` which contributes 0 levels up). Walk `from_path.parent()` that many times. Treat overshoot (more `../` than path components) as an unresolvable import and return `None` — do not panic, do not silently bottom-out at the repo root.
  - The remainder after the dot-prefixes is the path-relative-to-base. Join it onto the walked-up base and try **language-specific** extension lists, not a global list — see "Multi-Language Regression Guarantees" below for why. Per-language extension priority:
    - `rust`: `["rs"]` then directory-index `["mod.rs"]`.
    - `python`: `["py"]` then directory-index `["__init__.py"]`.
    - `typescript`: `["ts", "tsx", "d.ts"]` then directory-index `["index.ts", "index.tsx"]`.
    - `javascript`: `["js", "jsx", "mjs", "cjs"]` then directory-index `["index.js", "index.jsx", "index.mjs", "index.cjs"]`.
    - `go`: `["go"]` (no directory-index — Go packages are directories, handled by `resolve_go_module`).
    - `java`: `["java"]` (no directory-index).
    - **Cross-language fallback** (when language doesn't match any of the above — should never happen in practice but defensive): try the union `["rs", "py", "ts", "tsx", "js", "jsx", "mjs", "cjs", "go", "java", "d.ts"]` in the order listed, then the union of directory-index names. This preserves the pre-M26 behavior for any unknown `language` value.
  - Order matters for tie-breaking when both a file and a directory exist (e.g. `./util` matching both `util.ts` and `util/index.ts`). Pick the **file** over the directory, matching node and tsc resolution behavior. Document the rule in rustdoc.

- **`resolve_super` for Rust `super::` prefix:**
  - Currently `super::` is bundled with `crate::`/`self::` and treated as a stem search, ignoring the parent-navigation semantics. Split it out: count consecutive leading `super::` segments, walk up that many directories from `from_path.parent()`, then resolve the remainder via the existing stem-map logic against the repo-relative subtree rooted at the walked-up directory.

- **`resolve_python_dotted` for Python:**
  - Specifier shape from M25: bare dotted (`foo.bar.baz`), package-relative (`.`, `..`, `..pkg`, `...sibling.module`).
  - Bare dotted: replace `.` with `/`, look up `path_to_node` for `<repo-rel>/foo/bar/baz.py` and `<repo-rel>/foo/bar/baz/__init__.py`. If neither exists, treat as external (e.g. `os`, `pytest`) and return `None`.
  - Package-relative: count leading dots (1 dot = current package, 2 dots = parent package, etc.). Walk up from `from_path.parent()` that many levels, then apply the post-dots remainder as a path. Same file-vs-`__init__.py` lookup as bare dotted.
  - Python's resolution is package-aware: `from . import foo` from `pkg/sub/mod.py` looks up `pkg/sub/foo.py` or `pkg/sub/foo/__init__.py`. Honor that.

- **`resolve_go_module` for Go:**
  - On graph construction, look for a `go.mod` at the repo root; if present, parse the `module <path>` line (regex `^\s*module\s+(\S+)$` is sufficient — full `go.mod` parsing is overkill). Cache the module path on the graph builder.
  - Specifier `github.com/foo/bar` where `<module-path> = "github.com/foo/bar"` → an internal import. Strip the module prefix, look up `path_to_node` for the remaining sub-path with `.go` extension. Multi-file packages: a Go package is a directory; an internal import `github.com/foo/bar/pkg/util` resolves to **all** `.go` files in `pkg/util/`. Emit one edge per `.go` file in the directory (matches the structural convention that each `.go` file is a node).
  - Specifier without the module prefix: external; return `None`.
  - Standard library imports (`fmt`, `os`, `net/http`): external; return `None`.
  - When `go.mod` is absent: treat all Go imports as external (no edges from Go files). Log to stderr at `INFO` once per snapshot run (deduplicated) so users discover why their Go graph is empty.

- **`resolve_java_dotted` for Java:**
  - Specifier shape: dotted (`com.acme.lib.Util`) or wildcard (`com.acme.lib.*`).
  - Dotted: replace `.` with `/` and append `.java`, look up against `path_to_node`. Java places source under `src/main/java/<package-path>/Class.java` conventionally — accept both repo-relative (`com/acme/lib/Util.java`) and `src/main/java`-prefixed locations. Walk a small list of common roots: `["", "src/main/java", "src/test/java", "src", "java"]` — try each as a base.
  - Wildcard `com.acme.lib.*`: emit one edge per `.java` file in the resolved `com/acme/lib/` directory. Document the wildcard expansion behavior; this is the only language where one specifier can produce multiple edges.

- **Update the resolver dispatch:** `resolve_import` currently dispatches by string-prefix. Extend to dispatch by the **language** of the importing record (use `FeatureRecord.language`). Pass through to the language-specific resolver:
  - `language == "rust"` → existing `crate::`/`self::`/`super::` paths plus the new `resolve_super` parent-walk.
  - `language == "python"` → `resolve_python_dotted` (handles relative dots and bare dotted).
  - `language == "go"` → `resolve_go_module`.
  - `language == "java"` → `resolve_java_dotted`.
  - `language in ("typescript", "javascript")` → `resolve_relative` (the `./`/`../` case) for relative specifiers; absolute specifiers are external until M27 adds tsconfig alias support.
  - **Multi-language fallback:** if the specifier starts with `./` or `../`, always go through `resolve_relative` regardless of language. This handles the rare cross-language relative import (e.g. a TS file importing a generated `.json` — won't resolve to a node anyway, but the dispatch shouldn't panic).

- **Pass `language` through to the resolver:** `resolve_import`'s signature gains a `language: &str` parameter. Update the caller in `build_dependency_graph` (line 161-176) to thread `record.language.as_str()` through.

- **Update the broken-by-design test:** `relative_import_parent_slash_strips_prefix_resolves_in_same_dir` (sdivi-graph/tests/dependency_graph.rs:152) is the test that pins the current bug. Rewrite it as `relative_import_parent_slash_resolves_to_parent_dir` and update both the import path and the expected file:
  ```rust
  make_record("src/sub/module.py", &["../shared"]),
  make_record("src/shared.py", &[]),  // parent dir, was: src/sub/shared.py (same dir)
  ```
  Assert `edge_count == 1` resolving to `src/shared.py`. This flip is the canary that the fix landed.

- **Add new resolver tests** (in `crates/sdivi-graph/tests/dependency_graph.rs`):
  - Multi-level parent: `../../shared/x` from `app/items/Edit.tsx` resolves to `shared/x.tsx`.
  - Overshoot: `../../../../foo` from `src/util.ts` returns `None` (more `../` than path depth).
  - File-over-directory tie-break: `./util` with both `./util.ts` and `./util/index.ts` present resolves to `./util.ts`.
  - Python bare dotted: `foo.bar` with `foo/bar.py` present resolves correctly; with `foo/bar/__init__.py` present resolves correctly; with neither present returns `None` (external).
  - Python package-relative: `from .. import x` from `a/b/c.py` resolves to `a/x.py` or `a/__init__.py`.
  - Go: with `go.mod` declaring `module example.com/myapp`, importing `example.com/myapp/internal/util` from `cmd/main.go` resolves to one or more nodes under `internal/util/`.
  - Java: dotted `com.acme.lib.Util` with `src/main/java/com/acme/lib/Util.java` present resolves correctly.
  - Java wildcard: `com.acme.lib.*` with three `.java` files in `src/main/java/com/acme/lib/` produces three edges.

- **Add an integration test** `crates/sdivi-graph/tests/integration_real_world.rs` that builds a small synthetic multi-language repo (TS app importing relative siblings + parents; Python package with `__init__.py`; Go module with internal imports) and asserts edge counts match a hand-computed expected value. This is the "does it actually work end-to-end" test.

**Migration Impact:** Edge counts on existing repos will jump again post-M26 — anywhere parent-relative imports were used (most TS/JS code, package-relative Python, internal Go imports). Combined with M25, snapshot deltas will be substantial on the first run after upgrade. This is purely additive correctness — `snapshot_version` stays `"1.0"`. Update `MIGRATION_NOTES.md` with M25-and-M26 combined re-baselining guidance. The existing `relative_import_parent_slash_strips_prefix_resolves_in_same_dir` test's comment about "BUG" goes away — the test is replaced by one that asserts the correct behavior, and the broken-as-documented behavior is no longer present anywhere in the test suite.

**Files to create or modify:**

- **Modify:** `crates/sdivi-graph/src/dependency_graph.rs` — split `resolve_import` to dispatch by language; rewrite `resolve_relative` for proper parent navigation; add `resolve_super`, `resolve_python_dotted`, `resolve_go_module`, `resolve_java_dotted`. Add a `GoModInfo` struct for the parsed module path; load it once per `build_dependency_graph` call.
- **Modify:** `crates/sdivi-graph/Cargo.toml` — no new deps expected. `regex` is already in the workspace tree for the M16 historical-commit work; verify it's available transitively. If not, prefer hand-rolling the `module ` line scan over adding a dep.
- **Modify:** `crates/sdivi-graph/tests/dependency_graph.rs` — rewrite the parent-navigation test (was pinning the bug); add the multi-level, overshoot, tie-break, and per-language tests listed in Deliverables.
- **Create:** `crates/sdivi-graph/tests/integration_real_world.rs` — multi-language synthetic repo with hand-computed expected edge counts.
- **Modify:** `crates/sdivi-snapshot/tests/...` (workspace integration tests) — update fixture expectations now that edges resolve.
- **Modify:** `tests/full_pipeline.rs` — likewise.
- **Modify:** `CHANGELOG.md` — under **Fixed**: "Dependency graph resolver now navigates parent directories for `../` and `super::` imports; supports Python dotted, Go module-path, and Java dotted resolution. Combined with M25 adapter fixes, edge counts on real codebases increase substantially."
- **Modify:** `MIGRATION_NOTES.md` — combined M25+M26 re-baseline note.

**Acceptance criteria:**

- The rewritten parent-navigation test passes with the **corrected** expected target (`src/shared.py`, not `src/sub/shared.py`).
- All new unit tests pass.
- Multi-language integration test asserts pinned non-zero edge counts per language.
- `cargo test --workspace` continues to pass.
- `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` pass.
- `cargo build --target wasm32-unknown-unknown -p sdivi-core --no-default-features` continues to succeed (this milestone is scoped to `sdivi-graph` with the `pipeline-records` feature on; the WASM build path is unaffected because `compute_*` functions take pre-resolved `*Input` structs from the caller).
- Snapshotting `bifl-tracker` (the M11 validation fixture, also the user's bifl-tracker) produces `edge_count > 50` (rough lower bound — exact count pinned in CHANGELOG).
- Snapshotting `sdivi-rust` itself produces a substantially higher edge count than the current ~20 (rough target: ≥100, reflecting actual cross-crate use statements via the improved `resolve_super` and stem fallback).
- Determinism: two runs against the same repo produce a bit-identical `Snapshot.graph` field.

**Tests:**

- All Deliverables-section tests are required.
- Property test: generate random `(from_path, import_specifier)` pairs and assert resolution either succeeds with a path that exists in `path_to_node` or returns `None`. No panics.
- Regression test on `verify-leiden` quality fixtures: confirm modularity values stay within KDD-2's 1% / ±10% tolerance (the resolver fix does not change the Leiden algorithm; it changes the input graph density. Modularity may shift but the algorithm's output quality on a given graph is unchanged).
- Snapshot-roundtrip test: serialize and deserialize the snapshot, confirm the edge list survives.

**Multi-Language Regression Guarantees:**

M26 introduces language-based dispatch in the resolver where there was previously string-prefix dispatch. This is the highest cross-language risk surface in the M25–M27 sequence; the following invariants are non-negotiable:

- **Rust resolution must not regress.** The `crate::foo::bar` and `self::foo::bar` paths continue to dispatch to the existing `resolve_stem` logic (strip prefix → first segment → stem search across the whole repo). Behavior is byte-identical to pre-M26 for these cases. The `super::` path is the *only* Rust-specific change: parent-walk + stem-search-in-subtree, with **stem-search-across-whole-repo as a final fallback** if the parent-walk path doesn't resolve. This guarantees no Rust import that resolved before M26 fails to resolve after.
- **Per-language extension lists prevent extension-collision regressions.** Pre-M26 the resolver tried Rust extensions on Python files (and vice versa). With per-language extension lists, a Python file with `./util` import only tries `util.py` and `util/__init__.py` — not `util.rs` or `util.ts`. This eliminates a class of subtle false-positive resolutions but could in principle drop edges that depended on the old cross-extension behavior. Audit: search the existing fixtures and per-crate tests for any case where a Python/Go/Java specifier resolved to a non-`.py`/`.go`/`.java` file. None should exist; if any do, that's a latent bug in the fixture, not a regression.
- **Test fixtures using `language: "rust"` for non-Rust paths continue to work.** The `make_record` helper in `crates/sdivi-graph/tests/dependency_graph.rs` defaults to `language: "rust"` even for tests with `.py`/`.ts` paths. M26's dispatch logic for relative specifiers (`./` or `../`) overrides language and always routes to `resolve_relative` regardless of `record.language`. This preserves every existing relative-import test. (The cross-language fallback extension list is the safety net that makes this work even for fixtures that mix languages.)
- **Pluralized resolver API does not break callers outside `sdivi-graph`.** The `resolve_import` → `resolve_imports` rename and `Option<NodeIndex>` → `Vec<NodeIndex>` return-type change are confined to `sdivi-graph/src/dependency_graph.rs`. The public surface of `sdivi-graph` (the `DependencyGraph` type, `build_dependency_graph` constructor, `compute_*` consumers in `sdivi-core`) is unchanged. Verify no external callers use `resolve_import` directly — `grep -r resolve_import crates/ examples/ benches/` should turn up only the file being modified.
- **Pre-M26 test suite must pass green except for the one explicitly-flipped test.** Before merging, run `cargo test -p sdivi-graph` and confirm exactly one test fails (`relative_import_parent_slash_strips_prefix_resolves_in_same_dir`, which the milestone replaces with `relative_import_parent_slash_resolves_to_parent_dir`). Any other test failure is a regression that must be diagnosed before merge.
- **Snapshot determinism must hold per-language.** Add a determinism test that snapshots each of the 6 simple-language fixtures twice and asserts byte-identical `Snapshot.graph` output. Pre-M26 this guarantee held trivially because most graphs were near-empty; post-M26 the graphs are populated and a non-deterministic resolver path (e.g. iterating over a `HashMap` instead of a `BTreeMap`) would surface as a flaky snapshot.
- **`sdivi-lang-rust` is unchanged.** This milestone touches `sdivi-graph` only. The Rust adapter, its tests, and its fixtures are not modified. Rust resolution improvements live in M26's resolver work, not the adapter.
- **CI gates the cross-language regression net.** Add `tests/per_language_baselines.rs` with one pinned edge count per language fixture. Any future PR that changes the resolver causing a per-language baseline to shift gets a visible diff in the test output, forcing a deliberate update rather than a silent regression.

**Watch For:**

- **Path normalization on Windows.** `from_path.parent()` returns `\\`-separated paths on Windows; specifiers in import statements use `/`. `Path::join` handles the mismatch but `path_to_node`'s keys are constructed during graph build — verify the lookup uses the same separator convention. The `tests/stdout_stderr_split.rs` and existing fixtures pass on Windows CI; M26's fixtures should too.
- **Symlinks inside the repo.** `from_path.parent()` walked up `n` levels could cross a symlink boundary. Don't follow symlinks during resolution — `Path::join` doesn't, which is the safe default. Document.
- **Case sensitivity.** macOS HFS+ is case-insensitive; Linux ext4 is case-sensitive. The `path_to_node` map uses exact-byte path matching. Imports of `./Util` against a file named `util.ts` resolve on macOS but fail on Linux. Don't paper over this — match the underlying filesystem behavior. Document in `docs/determinism.md` as a caveat.
- **Go module path parsing edge cases.** `go.mod` may have `module path/with/slashes // a comment`, multi-line `module (\n path \n)` (rare), or be absent. The simple regex handles the common case; on parse failure, log and treat all Go imports as external rather than crashing.
- **Python's implicit namespace packages (PEP 420).** A directory without `__init__.py` can still be a package in Python 3.3+. The resolver should treat both as resolvable: if `foo.bar.baz` matches `foo/bar/baz.py` OR `foo/bar/baz/` (a directory containing any `.py` files), produce an edge. Test this case explicitly.
- **Java conventional source roots.** The list `["", "src/main/java", "src/test/java", "src", "java"]` covers Maven, Gradle, and ad-hoc layouts. It does not cover multi-module Maven (each module having its own `src/main/java`). For multi-module repos, every module's `src/main/java` is a separate root — the resolver should try them all. Implementation: at graph-build time, scan for any directory ending in `/src/main/java` or `/src/test/java` and add each to the list of roots. Document in rustdoc.
- **The `resolve_super` interaction with Rust's module tree.** Rust modules are not 1:1 with files; `mod foo; mod bar;` declarations in a `lib.rs` create a virtual tree. `super::foo::Bar` from inside `mod bar` could refer to a sibling module at any nesting. The resolver here is a heuristic: walk up `n` directories per `super::`, then stem-search the remainder. False negatives are acceptable (the import doesn't resolve, no edge added); false positives (resolving to the wrong file) are not. Tie-break: if `resolve_stem` finds multiple candidates, return `None` (current behavior — preserve it).
- **`resolve_relative` for bare specifiers in TS/JS.** `import "./util"` (no extension) is the common case; `import "./util.js"` (explicit `.js`) is also valid in ESM and refers to a `.ts` file at compile time. Try the specifier verbatim first (with both extension stripped and as-given), then with each extension. Document.
- **Multi-edge emissions (Java wildcard, Go package-import).** These are the only resolver paths that produce >1 edge per import. Make sure the caller (`build_dependency_graph`) handles a `Vec<NodeIndex>` return rather than `Option<NodeIndex>`. Plumb the API change cleanly: rename `resolve_import` to `resolve_imports` (plural), return `Vec<NodeIndex>`, with empty Vec for unresolvable. The single-target case becomes `vec![ni]`. Update all call sites.
- **Don't add tsconfig path-alias support here.** That's M27. Specifiers that *would* match a tsconfig alias (e.g. `@/lib/foo`) should fail to resolve in this milestone (return empty Vec) and log at `DEBUG`. M27 layers the alias step in front of the existing resolver chain.
- **Don't change the cache key in `.sdivi/cache/partition.json`.** The warm-start cache is keyed by graph topology hash; the topology will change due to new edges, so the cache will invalidate naturally on the first post-M26 run. That's correct — don't try to preserve cache continuity across the bug-fix boundary.

**Seeds Forward:**

- **M27** picks up tsconfig `compilerOptions.paths` and `baseUrl` for TS path aliases.
- **`package.json` `imports` field** (Node subpath imports, `#alias/foo` style). Fewer projects use this than tsconfig aliases. Defer until requested.
- **Python `setup.py` / `pyproject.toml` package roots.** The current resolver assumes the repo root is the import root. Real Python projects often have `src/<package>/...` layouts where `src/` is the import root. A future milestone could detect this from `pyproject.toml`'s `[tool.setuptools.packages.find]` or similar. Defer.
- **Multi-edge Java wildcards introduce a precision/recall trade-off.** A wildcard import that resolves to 50 classes in the package emits 50 edges, all with equal weight. This may inflate `coupling_delta` artificially. If this becomes noisy in practice, consider weighting wildcard-derived edges at `1/n` rather than `1` per file. Defer until observed.
- **Cross-language imports** (TS importing a generated Python file's stub, or Java FFI). Out of scope; SDIVI doesn't track these structurally.

---
