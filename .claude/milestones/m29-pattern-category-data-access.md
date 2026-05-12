
#### Milestone 29: Pattern Category — `data_access`

<!-- milestone-meta
id: "29"
status: "planned"
-->

**Scope:** Introduce `data_access` as the sixth canonical pattern category for `snapshot_version "1.0"`. Map the tree-sitter node kinds `call_expression` (TypeScript, JavaScript, Go) and `call` (Python) to the new category. Add the catalog entry in `sdivi_core::categories` so [`list_categories`] returns the new name, extend the per-category node-kind tables in `crates/sdivi-patterns/src/queries/`, add `"call"` to the Python adapter's `PATTERN_KINDS` (TS/JS/Go already collect `call_expression`), and update `docs/pattern-categories.md` so the human-readable contract matches the runtime contract. Classification is strictly node-kind-based — no callee-name filtering at the sdivi-rust layer; downstream embedders apply that narrowing themselves. The M23 `category_contract.rs` test is the durable drift gate: if `data_access` appears in `sdivi-patterns` but not in `CATALOG_ENTRIES`, that test fails.

**Why this milestone exists:** Of the architectural drift dimensions sdivi measures, *how a codebase accesses data* is one of the most actionable signals — a real-world 5400-file TypeScript codebase showed `data_access` running at **5.54 pattern entropy / 47% convention drift**, meaning service patterns vary substantially by team and there are no enforced guardrails around how the application reaches its data stores. The Python POC tracked this category; sdivi-rust currently does not, and `compute_pattern_metrics` therefore reports nothing for it. Adding `data_access` extends the measurement surface to one of the dimensions adopters most want to see, with no compute-cost surprise (every `call_expression`/`call` is *already* collected as a `PatternHint` in TS/JS/Go; only Python needs to add `"call"` to its `PATTERN_KINDS`).

**Deliverables:**
- Create `crates/sdivi-patterns/src/queries/data_access.rs` declaring the node-kind table — exact shape of the existing per-category modules (`error_handling.rs`, `async_patterns.rs`, etc.):
  ```rust
  //! Node kinds classified as data-access patterns.
  //!
  //! These node kinds correspond to the `data_access` category in the
  //! [`PatternCatalog`](crate::catalog::PatternCatalog).

  /// Tree-sitter node kinds for data-access patterns.
  ///
  /// - `call_expression`: function/method calls that access data stores or
  ///   external resources (TypeScript/JavaScript/Go: `fetch`, `query`, `read`,
  ///   `write`, `db.*`, `sql.*`, etc.). All `call_expression` nodes are
  ///   classified here; callee-name narrowing is the consumer's responsibility.
  /// - `call`: Python function calls accessing data (`cursor.*`, `session.*`,
  ///   `open`). Same broad-classification rule as above.
  pub const NODE_KINDS: &[&str] = &["call_expression", "call"];
  ```
- Modify `crates/sdivi-patterns/src/queries/mod.rs`:
  - Add `pub mod data_access;` (keep modules in alphabetical order).
  - Insert `"data_access"` in `ALL_CATEGORIES` alphabetically — slots between `"async_patterns"` and `"error_handling"`.
  - Extend `category_for_node_kind` with a new branch matching `data_access::NODE_KINDS`. Keep alphabetical branch order for readability.
  - Rename and update the existing `all_categories_has_five_entries` test to `all_categories_has_six_entries` (`assert_eq!(ALL_CATEGORIES.len(), 6)`).
  - Add a new unit test `call_expression_is_data_access`:
    ```rust
    #[test]
    fn call_expression_is_data_access() {
        assert_eq!(
            category_for_node_kind("call_expression", "typescript"),
            Some("data_access")
        );
        assert_eq!(category_for_node_kind("call", "python"), Some("data_access"));
    }
    ```
- Modify `crates/sdivi-core/src/categories.rs`:
  - Insert the new entry alphabetically into `CATALOG_ENTRIES` (between `async_patterns` and `error_handling`):
    ```rust
    (
        "data_access",
        "Code constructs that perform I/O against data stores or external resources — \
        e.g., database queries (`query`, `cursor.*`), HTTP fetches (`fetch`), \
        file reads (`open`, `read`), and ORM method calls. All `call_expression` / \
        `call` nodes are classified here; callee-name narrowing is the embedder's \
        responsibility.",
    ),
    ```
  - Extend the `CATEGORIES` `&[&str]` constant with the new index (`CATALOG_ENTRIES[1].0`) and shift the remaining indices by one.
  - Update the doc-test assertion `assert_eq!(CATEGORIES.len(), 5)` to `6`. Update the inline rustdoc example for [`list_categories`] if it references a specific length.
- Add `"call"` to `PATTERN_KINDS` in `crates/sdivi-lang-python/src/extract.rs`. **Do not change TypeScript or Go** — `call_expression` is already in their `PATTERN_KINDS` (TS line 12, Go line 13).
- Update `docs/pattern-categories.md`:
  - Add a row `| data_access | Code constructs that perform I/O against data stores… |` to the **Canonical category list** table (alphabetical).
  - Add a `data_access` row to each per-language **node-kind mapping** sub-section: `call_expression` for Rust, TypeScript/JavaScript, and Go; `call` for Python. Rust has no native data-access node kind in v0 — list `(none in v0)` to match the existing pattern for the categories Python doesn't cover.
  - Update the **Embedder responsibilities** section to document that `data_access` covers *all* call nodes at the sdivi-rust layer; embedders that want callee-name precision must filter `PatternInstanceInput` themselves before calling `compute_pattern_metrics`.
- Update `CHANGELOG.md` under the next-release `Added` section: "New pattern category `data_access` covering call expressions across all supported languages. Adds `"call"` to the Python adapter's collected node kinds."

**Migration Impact:** Strictly additive. `snapshot_version` stays `"1.0"`. Two observable behavioural shifts on the next post-upgrade snapshot:

1. **Existing TS/JS/Go snapshots gain a new category bucket.** `call_expression` `PatternHint`s were already collected; they were just classified as `None` and therefore absent from the catalog. After this milestone they get bucketed under `data_access` and appear in `compute_pattern_metrics` output. `compute_delta` between a pre-upgrade snapshot (no `data_access` key) and a post-upgrade snapshot (new `data_access` key) is a one-time recalibration event — document it explicitly in CHANGELOG so an adopter doesn't read it as a sudden drift spike.
2. **Python-heavy codebases see a substantial increase in `PatternHint` volume.** Adding `"call"` to the Python adapter's `PATTERN_KINDS` means every Python function call now emits a hint. On a multi-thousand-file Python repo this is a non-trivial uplift in catalog size and per-snapshot blake3 fingerprint work. The pipeline is still bounded (CST is still dropped per file per Rule 4), but expect a measurable wall-clock increase on Python-heavy fixtures. Run `cargo bench --features bench` on `tests/fixtures/simple-python` and a multi-language fixture; record the delta in the milestone PR description.

`.sdivi/config.toml` is unchanged — `[thresholds.overrides.data_access]` becomes a legal override block, governed by the existing `expires`-required rule (Rule 12 / KD12).

**Files to create or modify:**
- **Create:** `crates/sdivi-patterns/src/queries/data_access.rs`
- **Modify:** `crates/sdivi-patterns/src/queries/mod.rs` (module decl, `ALL_CATEGORIES`, `category_for_node_kind`, tests)
- **Modify:** `crates/sdivi-core/src/categories.rs` (`CATALOG_ENTRIES`, `CATEGORIES`, doc-test length, `list_categories` example if needed)
- **Modify:** `crates/sdivi-lang-python/src/extract.rs` (add `"call"` to `PATTERN_KINDS`)
- **Modify:** `docs/pattern-categories.md` (canonical table + per-language tables + embedder note)
- **Modify:** `CHANGELOG.md` (Added section)

**Acceptance criteria:**
- `cargo test --workspace` passes.
- `cargo test -p sdivi-core` passes — specifically `category_contract.rs` (the M23 grep-based drift gate). This is the load-bearing test: if `data_access` is introduced in `sdivi-patterns` but missed in `CATALOG_ENTRIES`, this test fails.
- `cargo test -p sdivi-patterns` passes — `call_expression_is_data_access` is new, `all_categories_has_six_entries` replaces the old five-entry assertion.
- `cargo build -p sdivi-core --target wasm32-unknown-unknown --no-default-features` succeeds.
- `cargo tree -p sdivi-core --target wasm32-unknown-unknown --no-default-features` still reports zero entries for `tree-sitter*`, `walkdir`, `ignore`, `rayon`, `tempfile` (Rule 21 / KD21).
- `wasm-pack test --node bindings/sdivi-wasm` passes, including an assertion that `list_categories()` from JS returns six entries with `data_access` present and `schema_version === "1.0"`.
- `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` pass.
- `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS=-D warnings` passes — every new pub item carries a doc comment per Rule 19 / `#![deny(missing_docs)]` on `sdivi-core`.
- A snapshot run against `tests/fixtures/simple-typescript` produces a non-empty `data_access` bucket in `pattern_metrics`; a snapshot against `tests/fixtures/simple-python` likewise. Determinism property tests (`prop_test_pipeline_deterministic`) remain green.

**Tests:**
- Unit (`queries/mod.rs`): `call_expression_is_data_access` (TS), `call_is_data_access` (Python). Optionally one Go test for completeness — same node kind as TS so the test is mostly insurance.
- Unit (`queries/mod.rs`): `all_categories_has_six_entries` replaces `all_categories_has_five_entries`. Keep both assertions: `ALL_CATEGORIES.len() == 6` and `ALL_CATEGORIES.contains(&"data_access")`.
- Doc test (`categories.rs`): updated `CATEGORIES.len()` assertion to `6`; doc example for `list_categories` references `data_access` (or, more durably, just asserts the catalog is non-empty and `schema_version == "1.0"` to avoid edits each time a category is added).
- Integration (`category_contract.rs`, already exists from M23): no change needed — it greps `sdivi-patterns/src/` at test time and asserts every discovered category string is in `list_categories()` output. **This is the safety net.** Verify it goes green after the change.
- Integration (WASM, `bindings/sdivi-wasm`): the `list_categories()` JS test from M23 needs its expected-count assertion bumped from 5 to 6 and an explicit `data_access` membership check added.
- Fixture-level: snapshot the `simple-typescript` and `simple-python` fixtures, assert the resulting `pattern_metrics` includes a `data_access` key with at least one instance.

**Watch For:**
- **The M23 `category_contract.rs` test is load-bearing.** Adding `data_access` to `sdivi-patterns/src/queries/data_access.rs` without also updating `CATALOG_ENTRIES` in `sdivi-core` trips this test. That's by design — the test exists precisely to catch this milestone's main risk. Don't paper over a failure by editing the test; fix the data.
- **No callee-name filtering at the sdivi-rust layer.** The current `category_for_node_kind(node_kind: &str, _language: &str)` ignores the `_language` parameter, and that is the contract for v0 — every `call_expression` in TS/JS/Go and every `call` in Python becomes a `data_access` `PatternHint`. Resist the urge to add per-language regex on callee names (`fetch|query|read|write|db\.|sql\.`, `cursor\.|session\.|open`, etc.) here — that belongs in the consumer layer (Meridian and similar embedders apply the narrowing on `PatternInstanceInput.text` before calling `compute_pattern_metrics`). Adding callee filtering at this layer would couple the WASM contract to language-specific naming conventions and turn the `_language` parameter into a load-bearing API. **Seeds Forward** records this as a deliberate deferral.
- **Python adapter `PATTERN_KINDS` expansion is the real cost driver.** Adding `"call"` floods the Python adapter with hints — every function call. The blake3 fingerprint, the catalog bucket, the delta compute all scale up linearly. Benchmark before/after on a multi-thousand-file Python fixture (or the bifl-tracker reference codebase from M11) and record the wall-clock and memory delta in the PR description. If it's catastrophic, the fallback is to gate the Python `"call"` inclusion behind a future config knob — but that should not be the default; broad classification is the contract.
- **Alphabetical order discipline.** `ALL_CATEGORIES` in `sdivi-patterns::queries::mod.rs`, `CATALOG_ENTRIES` in `sdivi-core::categories`, the module declarations, the `category_for_node_kind` branches, the documentation table in `docs/pattern-categories.md` — **all** must remain alphabetically ordered. `data_access` slots between `async_patterns` and `error_handling`. Mismatched order across these files creates noise in future milestone diffs and makes the contract harder to read.
- **Doc-comment placement (the CLAUDE.md convention).** When inserting the new `CATALOG_ENTRIES` tuple between `async_patterns` and `error_handling`, ensure a blank line separates the new entry from the existing `///` doc block on the constant — otherwise the existing doc block silently re-attaches to the new item. `#![deny(missing_docs)]` on `sdivi-core` will catch the resulting missing-doc on the original constant, but the failure mode is confusing. Same caution applies inside `data_access.rs` itself — the `///` block on `NODE_KINDS` must be the *last* thing before the `pub const`, with no other items in between.
- **`CATEGORIES` const has hand-written indices.** It's literally:
  ```rust
  pub const CATEGORIES: &[&str] = &[
      CATALOG_ENTRIES[0].0, CATALOG_ENTRIES[1].0, ...
  ];
  ```
  Inserting a new entry at index 1 shifts every subsequent index by one. Double-check after the edit — a typo here is a silent mismatch between `CATEGORIES` and `CATALOG_ENTRIES` that the existing doc tests will catch but only if their length assertions are also updated. If the index plumbing keeps causing edit-time errors, **Seeds Forward** suggests rewriting `CATEGORIES` as a build-time-generated `OnceLock<Vec<&'static str>>` or a `const fn` over `CATALOG_ENTRIES` — out of scope here.
- **`docs/pattern-categories.md` is part of the contract surface.** The category-set additive rule (Versioning section of the contract doc) is what authorises this change without a `snapshot_version` bump. Update the doc in the same PR; do not let the docs lag.
- **`[thresholds.overrides.data_access]` is now legal.** No code change is required — the override loader is category-agnostic — but the new category name will pass through `Config::load_or_default` validation as soon as `CATALOG_ENTRIES` lists it. If any existing test fixture sets a `[thresholds.overrides.<unknown_category>]` block expecting it to be silently ignored, that test stays green; this milestone does not change override resolution.
- **`compute_delta` between snapshots straddling this milestone produces a new top-level key.** Embedders (Meridian) that hard-code a closed set of category keys when consuming `DivergenceSummary` will need to widen their schema. The whole point of M23's `list_categories()` was to make that discovery dynamic — confirm Meridian is calling it rather than hard-coding before announcing the release.
- **JavaScript shares TypeScript's adapter.** No separate edit is needed for `sdivi-lang-javascript` — verify that's still the case (it was at M04) and reflect it in the `docs/pattern-categories.md` per-language tables.

**Seeds Forward:**
- **M30 — `logging` pattern category.** Same milestone shape, different node kinds (likely also `call_expression` / `call`, distinguished only by callee name — which means at the sdivi-rust layer `logging` will overlap with `data_access` unless either (a) callee filtering moves into sdivi-rust, or (b) we accept that `logging` cannot be separated from `data_access` without consumer-side narrowing). Resolve the design tension in M30's spec, not here.
- **M31 — `class_hierarchy` pattern category.** Different shape — class declarations, interface extensions, inheritance constructs (`class_declaration`, `class_definition`, `interface_declaration`, possibly `heritage_clause` / `bases` children). Node-kind based classification works cleanly because these are distinct AST shapes from calls.
- **Callee-name filtering at the sdivi-rust layer.** If a real adopter needs precision parity with their consumer's filter, the work is: (1) extend `category_for_node_kind` to accept `text: &str` from `PatternHint`, (2) add per-language regex tables, (3) gate behind the now-load-bearing `language` parameter, (4) add `compute_pattern_metrics` doc explaining the per-language matching. This is a meaningful surface change and probably its own milestone — explicitly deferred here.
- **A native `PatternHint` field for callee text.** `PatternHint` currently carries `text` for fingerprint normalisation; making "callee" a first-class field (vs requiring the consumer to re-parse `text`) would simplify both the consumer-layer filter and any future move of the filter into sdivi-rust. Out of scope here; revisit when M30 forces the design.
- **Index plumbing in `CATEGORIES`.** The hand-written `CATALOG_ENTRIES[N].0` array is fragile to insertions. A future cleanup pass should replace it with either a `const fn` walk or a `LazyLock<Vec<&'static str>>` derived from `CATALOG_ENTRIES`. Mechanical, low-risk, no contract change — defer until it bites.
- **Per-language node-kind tables in `docs/pattern-categories.md` are hand-maintained.** M23 already flagged this. A future milestone could derive the tables from the tree-sitter queries themselves to eliminate the doc/code drift surface entirely. Out of scope for v0; the category-contract test catches the high-value drift (category set), and the per-language tables are checked by human review on every category-adding milestone.
