#### Milestone 25: Adapter Module-Specifier Extraction

<!-- milestone-meta
id: "25"
status: "pending"
-->

**Scope:** Change every non-Rust language adapter (`sdivi-lang-python`, `sdivi-lang-typescript`, `sdivi-lang-javascript`, `sdivi-lang-go`, `sdivi-lang-java`) to emit just the module specifier(s) of each import statement into `FeatureRecord::imports`, not the full statement text. The current implementation pushes whole-statement text such as `"import { foo } from '../lib/x';"` into the records, which `dependency_graph::resolve_import` immediately rejects (the string starts with `import`, not `./` / `../` / `crate::` / `self::` / `super::`), producing zero cross-file edges in the dependency graph for every non-Rust language. The Rust adapter already extracts specifiers correctly via its `use ` keyword strip and is unchanged here.

**Why this milestone exists:** Validating sdivi on a real Next.js TypeScript application (bifl-tracker) produced `nodes=38 edges=0 communities=38 modularity=0.000` — every file alone, every coupling-based metric meaningless. Investigation traced the cause to `extract_imports` in each non-Rust adapter calling `node.utf8_text(source)` on the whole `import_statement` / `import_declaration` / `import_from_statement` node and pushing the result. The graph stage's `resolve_import` then drops every such string at `DEBUG` level because it doesn't match any known prefix. This silently breaks every coupling-based metric — `coupling_delta`, `community_count_delta`, `modularity`, and Factor 4 (`boundary_violation_rate` from M19) all become near-zero or trivial because they require a non-trivial graph. The same symptom has shown up on a large internal TS project at work where boundary violations stayed at `0` despite obvious layering breaks. The existing test `relative_import_parent_slash_strips_prefix_resolves_in_same_dir` in `crates/sdivi-graph/tests/dependency_graph.rs` even pins a downstream symptom of this with a comment ("See BUG note in TESTER_REPORT.md re: parent navigation not implemented") — confirming the issue was observed during initial development but never resolved. This milestone is the precondition for any non-Rust project to get a usable structural graph; M26 (parent navigation) and M27 (tsconfig aliases) build on it.

**Deliverables:**

- **Python (`sdivi-lang-python`):** Walk into `import_statement` and `import_from_statement` to extract specifiers rather than pushing the whole node text.
  - `import a, b.c, d as e` → emit `["a", "b.c", "d"]` (one entry per `dotted_name`; the `as` alias is a local name, drop it).
  - `from foo.bar import x, y as z` → emit `["foo.bar"]` (the module is the `dotted_name` after `from`; the imported names are not modules).
  - `from . import foo` → emit `["."]` (relative-package import; the resolver will treat this as "current package").
  - `from .. import foo` / `from ..pkg import foo` → emit `[".."]` / `["..pkg"]` (relative-package navigation, count of leading dots preserved).
  - `from __future__ import …` (a `future_import_statement`) → emit nothing; `__future__` is a synthetic module the graph should never resolve to a file.

- **TypeScript (`sdivi-lang-typescript`):** Walk into `import_statement` to find the `string_fragment` child of the `string` node that holds the module specifier.
  - `import { foo } from "../lib/x"` → emit `["../lib/x"]`.
  - `import * as ns from "./util"` → emit `["./util"]`.
  - `import "./side-effect"` → emit `["./side-effect"]` (side-effect import; specifier is the only child).
  - `import type { T } from "./types"` → emit `["./types"]` (type-only imports still create a structural dependency for our purposes).
  - `export { foo } from "./util"` (an `export_statement` with `from` clause; **TSX/JSX too**) — out of scope for this milestone. Note in `Watch For`; it's a real edge type but `extract_imports` only walks `import_statement` today. Track in **Seeds Forward**.
  - Tree-sitter-typescript node kinds to assert: `import_statement` → child `import_clause` (optional) + child `string` → child `string_fragment` (the unquoted specifier). Verify against `tree-sitter-typescript` v0.20+ which is what the grammar pin in `Cargo.toml` ships.

- **JavaScript (`sdivi-lang-javascript`):** Same pattern as TypeScript — extract `string_fragment` from `import_statement`'s `string` child. Additionally, capture CommonJS `require("…")` calls:
  - Walk for `call_expression` whose `function` child is an `identifier` named `require` and whose `arguments` first child is a `string` → emit the `string_fragment`. This catches the `const fs = require("fs")` and `const utils = require("./utils")` patterns ubiquitous in older Node code and in mixed-module projects.
  - Bare `import("…")` dynamic imports (a `call_expression` whose `function` is the `import` keyword node) → emit the specifier. Note: dynamic imports may take expressions, not just literals — only emit when the argument is a string literal, skip otherwise.

- **Go (`sdivi-lang-go`):** Walk into `import_declaration` to find `import_spec` children, each with a `path` field (an `interpreted_string_literal`).
  - `import "fmt"` → emit `["fmt"]`.
  - Grouped `import ( "fmt"; "os" )` → emit `["fmt", "os"]` (one per `import_spec`).
  - Aliased `import f "fmt"` → emit `["fmt"]` (alias is a local name).
  - Dot import `import . "fmt"` → emit `["fmt"]` (the dot is a syntactic marker for namespace flattening).
  - Blank import `import _ "github.com/lib/pq"` → emit `["github.com/lib/pq"]` (side-effect import; structurally still a dependency).

- **Java (`sdivi-lang-java`):** Walk into `import_declaration` and extract the `scoped_identifier` child as a dotted string.
  - `import java.util.List;` → emit `["java.util.List"]`.
  - `import java.util.*;` → emit `["java.util.*"]` (preserve the wildcard; the resolver will need to decide how to handle wildcard imports — see M26).
  - `import static org.junit.Assert.assertEquals;` → emit `["org.junit.Assert"]` (drop the trailing static-imported member name; the module is the class).

- **Update existing per-adapter unit tests:** Each adapter has a `tests/` file (or `#[cfg(test)] mod tests`) asserting `extract_imports` on small fixtures. Update those assertions to expect specifier-only strings, not whole-statement text. Add new test cases covering the bullet variants above (relative dots in Python, `string_fragment` extraction in TS/JS, `require()` and dynamic `import()` in JS, grouped/aliased/blank in Go, wildcards and statics in Java).

- **Update `crates/sdivi-graph/tests/dependency_graph.rs`:** The existing test `relative_import_parent_slash_strips_prefix_resolves_in_same_dir` (line 152) explicitly pins broken downstream behavior with a "See BUG note" comment. Leave its assertions untouched in this milestone (M26 handles the resolver fix that flips it). But add a new test `python_from_import_yields_dotted_specifier` and `typescript_default_import_yields_string_fragment` that build a `FeatureRecord` directly and assert the resolver receives the right shape — confirming end-to-end shape regardless of language.

**Migration Impact:** Snapshots produced after M25 will have substantially **larger** edge counts on Python / TS / JS / Go / Java projects than prior baselines, because edges that were silently dropped now resolve. This is a correctness fix, not a schema change — `snapshot_version` stays `"1.0"`. Existing baselines remain readable; the first post-M25 snapshot will simply produce a large `coupling_delta` and `community_count_delta` against the prior (artificially-sparse) baseline. Document in `CHANGELOG.md` under **Fixed** and recommend either re-baselining at the M25 boundary or using one-time `coupling_delta_rate` and `boundary_violation_rate` overrides with `expires` set 1–2 weeks out to absorb the cutover. Boundary-violation deltas will increase on projects with `.sdivi/boundaries.yaml` declared — a *good* sign that M19's gate is finally meaningful.

**Files to create or modify:**

- **Modify:** `crates/sdivi-lang-python/src/extract.rs` — rewrite `extract_imports` to walk into `import_statement` / `import_from_statement` children rather than emitting the whole node text. Add a private helper `dotted_name_text(node, source) -> Option<String>` for the dotted-identifier extraction.
- **Modify:** `crates/sdivi-lang-python/tests/extract.rs` (if it exists; otherwise the inline `#[cfg(test)]` block) — update fixtures.
- **Modify:** `crates/sdivi-lang-typescript/src/extract.rs` — rewrite `extract_imports` to extract `string_fragment`. Update the docstring on the function.
- **Modify:** `crates/sdivi-lang-typescript/tests/...` — update.
- **Modify:** `crates/sdivi-lang-javascript/src/extract.rs` — rewrite `extract_imports` to extract `string_fragment`; add `require()` and dynamic `import()` walk.
- **Modify:** `crates/sdivi-lang-javascript/tests/...` — update; new tests for `require` and dynamic `import`.
- **Modify:** `crates/sdivi-lang-go/src/extract.rs` — walk `import_spec` children rather than the whole `import_declaration`.
- **Modify:** `crates/sdivi-lang-go/tests/...` — update; new tests for grouped, aliased, blank imports.
- **Modify:** `crates/sdivi-lang-java/src/extract.rs` — walk `scoped_identifier` rather than the whole `import_declaration`; strip the trailing member name on static imports.
- **Modify:** `crates/sdivi-lang-java/tests/...` — update; new tests for wildcard and static imports.
- **Modify:** `crates/sdivi-graph/tests/dependency_graph.rs` — add the two new shape-assertion tests; do not touch the broken-by-design parent-navigation test (M26 owns that flip).
- **Modify:** `tests/full_pipeline.rs` (or whichever workspace-level integration test exists) — assert non-zero edges on the existing per-language fixtures (`tests/fixtures/simple-{python,typescript,javascript,go,java}/`). This becomes the regression net that catches future adapter regressions across all 5 languages at once.
- **Modify:** `CHANGELOG.md` — under **Fixed**: "Non-Rust language adapters now emit module specifiers from imports rather than whole-statement text; cross-file edge counts on Python/TS/JS/Go/Java projects increase substantially. May produce a one-time large `coupling_delta` against pre-M25 baselines."
- **Modify:** `MIGRATION_NOTES.md` — add an entry under "0.x → 0.y" describing the re-baseline guidance.

**Acceptance criteria:**

- For each of the 5 fixture repos under `tests/fixtures/simple-{python,typescript,javascript,go,java}/`, the integration test asserts `dg.edge_count() > 0` after `Pipeline::snapshot`. Exact counts pinned per fixture so regressions are caught precisely.
- `cargo test --workspace` passes. The verify-leiden, change-coupling, and snapshot-roundtrip suites continue to pass (they use synthetic graphs and are unaffected by adapter changes).
- `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` pass.
- `cargo build --target wasm32-unknown-unknown -p sdivi-core --no-default-features` continues to succeed (this milestone touches only adapter crates, which are not in `sdivi-core`'s WASM dep tree — Rule 21 is preserved).
- Snapshot determinism: running `Pipeline::snapshot` twice on the same fixture yields a bit-identical `Snapshot` JSON (Rule 1 + Critical System Rule 1).
- Per-language baseline of edge counts (regression sentinels): pinned in the integration test as `assert_eq!`, not `>=`. If grammar updates change the count, it's a deliberate decision that requires updating the pinned value.
- The `relative_import_parent_slash_strips_prefix_resolves_in_same_dir` test continues to pass — M25 doesn't fix it; M26 does. Its presence confirms M25 didn't accidentally also flip parent-navigation behavior.

**Tests:**

- **Per-adapter unit tests** in each `sdivi-lang-*/src/extract.rs` (or `tests/extract.rs`):
  - **Python:** plain `import`, `import a as b`, `from foo import x`, `from foo.bar import x`, `from . import x`, `from .. import x`, `from ..pkg import x`, `from __future__ import annotations` (yields nothing), multi-name `import a, b, c`.
  - **TypeScript:** named `import { a }`, default `import a`, namespace `import * as a`, side-effect `import "./x"`, type-only `import type { T }`, mixed `import a, { b } from "./x"`.
  - **JavaScript:** all TS shapes, plus `require("./x")`, `require(\`./tpl\`)` (template literal — skip), `require(name)` (variable arg — skip), dynamic `import("./x")`.
  - **Go:** single, grouped, aliased, dot, blank, multi-line grouped with comments interleaved.
  - **Java:** single, wildcard, static-method, static-wildcard, package-relative.
- **Workspace integration test** `tests/import_extraction.rs` (new): for each fixture under `tests/fixtures/simple-*`, run `Pipeline::snapshot` and assert `dg.edge_count() == EXPECTED` where `EXPECTED` is a pinned number per fixture.
- **Property test (proptest)** in `crates/sdivi-graph/tests/proptest.rs`: generate `FeatureRecord` lists with `imports = [random specifier]` and assert that resolution either succeeds or fails deterministically — no panics, no non-deterministic outcomes.

**Multi-Language Regression Guarantees:**

This milestone touches 5 of 6 adapter crates but leaves the Rust adapter (`sdivi-lang-rust`) **completely untouched** — its existing `extract_imports` already returns proper specifiers via the `use ` keyword strip. Concretely:

- **Rust:** zero source-file changes in `sdivi-lang-rust`. The crate's existing tests must pass byte-identically.
- **Cross-crate isolation:** each non-Rust adapter is its own Cargo crate. A change in `sdivi-lang-python::extract` cannot affect `sdivi-lang-typescript` (no shared mutable state, no shared types beyond `FeatureRecord` and `PatternHint` which are already-stable).
- **`FeatureRecord` shape unchanged:** `imports: Vec<String>` is unchanged in field name and type. Only the *content* of those strings changes for non-Rust languages. `sdivi-graph::resolve_import`'s current behavior on Rust-style strings (`crate::foo::bar`) is preserved verbatim — M25 changes inputs the resolver sees, not the resolver itself.
- **Per-language regression sentinel test required:** `tests/import_extraction.rs` must include all 6 fixtures (including `simple-rust/`) with pinned edge counts. The Rust fixture's edge count is **identical pre- and post-M25**; this catches accidental cross-language refactor leakage during review.

**Watch For:**

- **Tree-sitter grammar version drift.** The exact node kinds (e.g. `string_fragment` vs `string`, `dotted_name` vs `identifier`) depend on the pinned grammar version. Each adapter's `Cargo.toml` pins a specific `tree-sitter-*` crate version; check the actual node names against the pinned version's grammar.json before assuming the structure. If the grammar exposes different node kinds, the test fixtures catch it but the extractor logic must match.
- **Python relative-package dot count.** `from ... import foo` (three dots) is "two parents up." The graph resolver in M26 will need the count of leading dots — preserve it in the specifier string verbatim (`"..."` literally) rather than collapsing. Document this contract in the rustdoc on `extract_imports` so M26 knows what to consume.
- **TypeScript export-from (`export { foo } from "./x"`)** is not handled by this milestone — `extract_imports` only walks `import_statement`. Track the gap in the `Watch For` of M26 (which is the natural place to expand resolver behavior). Skipping it now keeps M25 scoped to a literal one-statement-class change.
- **JavaScript dynamic `import()` with non-string args.** `import(somePathVar)` cannot be resolved statically. Skip silently — emitting an unresolvable specifier costs nothing but adds noise to `DEBUG` logs. The `if argument is a string literal` gate is essential.
- **Go grouped imports with build-tag-conditional blocks.** `import (\n//go:build foo\n"foo"\n)` — the build tag is a comment node and should be ignored. The walk is structural (`import_spec` children only); build tags don't appear inside `import_spec`. Verify nonetheless.
- **Java `import static foo.Bar.*`** — wildcard static import. Strip the trailing `*` as well as the member; specifier is `foo.Bar`. Document.
- **CST-drop discipline (Rule 4).** `extract_imports` must not return any `Node` reference; it returns `Vec<String>`. The current pattern (allocate `String` from `utf8_text`) is already compliant — preserve it.
- **Determinism of multi-import order.** When a single statement yields multiple specifiers (`import a, b, c` in Python; grouped Go imports), preserve syntactic order. `BTreeMap` ordering does *not* apply here — these are inputs to graph construction, and the graph builder takes them in `Vec` order. Stability across runs requires syntactic-order preservation, which tree-sitter's child-iteration provides naturally. Don't sort.

**Seeds Forward:**

- M26 picks up the resolver work: parent-path navigation for `../`, language-specific module resolution (Python dots → paths, Go module paths via `go.mod`, Java dotted packages → paths).
- M27 picks up tsconfig `compilerOptions.paths` and `baseUrl` for TS path aliases (`@/lib/...` style) — a separate concern from the adapter extraction handled here.
- TS `export { … } from "…"` re-export edges. Real structural dependency, not currently captured. Probably add to `extract_imports` (or rename to `extract_imports_and_reexports`) once a consumer needs it. Defer until requested.
- Dynamic and computed imports (template literals, `require(varName)`). Static analysis can never resolve these; consumers who care will need a runtime-tracing extractor — outside SDIVI's scope.
- Per-language `extract_imports` could share a common return shape (`Vec<ModuleSpecifier>` with kind: `Relative { dots: u8, path: String } | Absolute { name: String } | PackageRelative { dots: u8, path: String }`) instead of bare `Vec<String>`. That would let the resolver branch on kind rather than re-parse the string. Worth doing eventually; deferred to keep this milestone scoped.

---
