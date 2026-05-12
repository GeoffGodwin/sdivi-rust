
#### Milestone 30: Pattern Category — `logging`

<!-- milestone-meta
id: "30"
status: "planned"
-->

**Scope:** Introduce `logging` as the seventh canonical pattern category for `snapshot_version "1.0"`, with a **catalog-only** wiring. `list_categories()` returns `"logging"`, the `docs/pattern-categories.md` canonical table and per-language tables document it, and `crates/sdivi-patterns/src/queries/logging.rs` records the conceptual node-kind set as reference documentation — but `category_for_node_kind` is **not** extended with a `logging` branch. Sdivi-rust's native pipeline therefore never auto-classifies anything as `logging`; the category exists in the contract so embedders (Meridian and other foreign extractors) can emit `PatternInstanceInput { category: "logging", … }` after applying their own callee-text filter (`console.*`, `logger.*`, `log.*`, `tracing::info!`, `logging.*`, `print`, `fmt.Print*`, etc.) and have those instances flow through `compute_pattern_metrics` and `compute_delta` as a recognised category. The catalog-only approach is forced by two prior decisions: `call_expression` / `call` are already claimed by `data_access` (M29) and `macro_invocation` is already claimed by `resource_management` (M06) — re-routing either at the node-kind layer would silently steal hits from those existing categories or duplicate-classify, both of which break the SemVer-bound contract.

**Why this milestone exists:** Logging is one of the strongest baseline-comparison signals sdivi produces. The same real-world 5400-file TypeScript codebase that showed `data_access` at 5.54 entropy / 47% drift showed `logging` at **4.83 entropy / 20% drift** — the *lowest* drift of any category measured. That asymmetry is the actionable read: when logging stays consistent while data access does not, it confirms the codebase *can* maintain conventions and the divergence elsewhere is category-specific, not a culture problem. Adopters running sdivi want this baseline. The Python POC tracked it; sdivi-rust does not. After this milestone, foreign extractors can emit `category: "logging"` and have it round-trip through `compute_pattern_metrics`, `compute_delta`, and the CHANGELOG-visible `pattern_metrics` map — closing the parity gap and giving adopters the comparison-baseline reading without forcing them to hard-code an out-of-contract string.

**Why catalog-only (the design tension, recorded for posterity):** Every reasonable native routing collides with an existing category. The collisions are:

- `call_expression` (TS/JS/Go) and `call` (Python) are the canonical AST shape for `logger.info(…)`, `console.log(…)`, `print(…)`, `fmt.Println(…)`. They are also the canonical AST shape for every other function call — and **M29 already routes them to `data_access`**. Splitting them between two categories requires callee-text filtering, which `category_for_node_kind(node_kind: &str, _language: &str)` is intentionally architected *not* to do.
- `macro_invocation` (Rust) is the AST shape for `tracing::info!`, `log::debug!`, etc. It is also the AST shape for `vec!`, `drop!`, `assert!`, `println!`, `format!` — and **M06 already routes it to `resource_management`**. Re-routing the whole node kind would empty `resource_management` in Rust and misclassify every non-logging macro as logging. Macro-name filtering needs the same callee-text mechanism we don't yet have.

Two paths out: (a) move callee-text filtering into `category_for_node_kind` now, turning `_language` into a load-bearing parameter and adding per-language regex tables; (b) leave classification node-kind-only and accept that `logging` (and the harder `data_access` vs `logging` split) is the embedder's call. **We pick (b)** because it matches sdivi-rust's documented architecture for v0 — the WASM contract is deliberately narrow, foreign extractors already implement callee filtering on their side (M23 was built for exactly this scenario), and a future milestone can promote callee filtering into the native pipeline once a real adopter demands precision parity. Recording the deferral here so future readers do not relitigate the design without context.

**Deliverables:**
- Create `crates/sdivi-patterns/src/queries/logging.rs` as documentation-by-source:
  ```rust
  //! Node kinds that conceptually belong to the `logging` category.
  //!
  //! **This module is intentionally not wired into [`category_for_node_kind`].**
  //! The node kinds listed below overlap with [`data_access`](super::data_access)
  //! (`call_expression`, `call`) and [`resource_management`](super::resource_management)
  //! (`macro_invocation`) at the AST level — only the callee name distinguishes
  //! a logging invocation from a data-access or resource-management one. Native
  //! classification by node kind alone would either steal hits from those
  //! existing categories or duplicate-classify every call/macro.
  //!
  //! Foreign extractors (e.g. the Meridian consumer app) MUST apply callee-text
  //! filtering on their side before emitting [`PatternInstanceInput`] values
  //! with `category = "logging"`. The supported callee shapes per language are
  //! documented in `docs/pattern-categories.md`.
  //!
  //! [`category_for_node_kind`]: super::category_for_node_kind
  //! [`PatternInstanceInput`]: sdivi-core's PatternInstanceInput re-export

  /// Conceptual tree-sitter node kinds for logging patterns. Reference only —
  /// not consulted by [`category_for_node_kind`](super::category_for_node_kind).
  ///
  /// Listed for documentation parity with sibling category modules and so
  /// that embedders can grep `sdivi-patterns/src/queries/logging.rs` to see
  /// the node-kind shapes the canonical contract has in mind.
  ///
  /// - `call_expression`: TS/JS/Go logger / console / Print* calls
  /// - `call`: Python `logging.*` and `print`
  /// - `macro_invocation`: Rust `tracing::*!`, `log::*!`, `println!`/`eprintln!`
  pub const NODE_KINDS: &[&str] = &["call_expression", "call", "macro_invocation"];
  ```
- Modify `crates/sdivi-patterns/src/queries/mod.rs`:
  - Add `pub mod logging;` alphabetically.
  - Add `"logging"` to `ALL_CATEGORIES` alphabetically — slots between `"error_handling"` and `"resource_management"`. The constant documents the canonical category set; the fact that `category_for_node_kind` never returns `Some("logging")` is intentional and documented in the new module's docstring.
  - **Do not extend `category_for_node_kind`** with a `logging` branch. There is no node-kind routing to add.
  - Update the entry-count assertion: rename `all_categories_has_six_entries` to `all_categories_has_seven_entries` and bump the constant to `7`. Add a `logging_is_in_all_categories` test asserting membership.
  - Add a regression test confirming the sentinel behavior:
    ```rust
    #[test]
    fn category_for_node_kind_never_returns_logging() {
        // logging is a catalog-only category for v0 — foreign extractors emit
        // it directly. Document the contract here so a future change that adds
        // a native routing must update this test deliberately, not by accident.
        for kind in ["call_expression", "call", "macro_invocation"] {
            for lang in ["rust", "python", "typescript", "javascript", "go", "java"] {
                assert_ne!(
                    category_for_node_kind(kind, lang),
                    Some("logging"),
                    "logging is catalog-only in v0; routing for ({kind}, {lang}) \
                     would steal from data_access/resource_management"
                );
            }
        }
    }
    ```
- Modify `crates/sdivi-core/src/categories.rs`:
  - Insert the new entry alphabetically into `CATALOG_ENTRIES` (between `error_handling` and `resource_management`):
    ```rust
    (
        "logging",
        "Code constructs that produce diagnostic or observability output — \
        e.g., `console.*` calls, structured logger invocations (`logger.info`), \
        `print` statements, and logging macros (`tracing::info!`, `log::debug!`). \
        Classification at the sdivi-rust layer is catalog-only: native code does \
        not auto-classify by node kind alone (the relevant kinds — `call_expression`, \
        `call`, `macro_invocation` — are already claimed by `data_access` and \
        `resource_management`). Foreign extractors apply callee-name filtering \
        and emit `PatternInstanceInput { category: \"logging\", … }` directly.",
    ),
    ```
  - Extend the `CATEGORIES` `&[&str]` constant with the new index (`CATALOG_ENTRIES[3].0` assuming alphabetical placement post-M29: `async_patterns`, `data_access`, `error_handling`, `logging`, `resource_management`, `state_management`, `type_assertions`). Re-verify each index against the actual position post-edit.
  - Bump the doc-test assertion `assert_eq!(CATEGORIES.len(), 6)` to `7`. Update any inline rustdoc example on `list_categories` that quotes a specific length.
- Modify `docs/pattern-categories.md`:
  - Add a row `| logging | Code constructs that produce diagnostic or observability output… |` to the **Canonical category list** table, alphabetically.
  - Add a `logging` row to every per-language **node-kind mapping** sub-section:
    - **Rust:** `(consumer extractor responsibility — `macro_invocation` overlaps with `resource_management` at the AST level)`
    - **Python:** `(consumer extractor responsibility — `call` overlaps with `data_access` at the AST level)`
    - **TypeScript / JavaScript:** `(consumer extractor responsibility — `call_expression` overlaps with `data_access` at the AST level)`
    - **Go / Java:** same note as TS/JS.
  - Add a paragraph under the **Embedder responsibilities** section documenting the catalog-only contract for `logging` explicitly: "The `logging` category in `snapshot_version "1.0"` is **catalog-only**: `sdivi_patterns::queries::category_for_node_kind` never returns `Some("logging")`. Embedders that wish to emit logging instances MUST apply callee-text filtering on their side (typical patterns: `console.*`, `logger.*`, `log.*`, `tracing::*!`, `log::*!`, `logging.*`, `print`, `fmt.Print*`) and pass `PatternInstanceInput { category: "logging", … }` directly into `compute_pattern_metrics`. The category exists in `list_categories()` so embedder output round-trips through `compute_delta` without being treated as an unknown category. This is a deliberate v0 deferral — see M30 for the design discussion."
- Update `CHANGELOG.md` under the next-release `Added` section: "New pattern category `logging` (catalog-only — foreign extractors populate it via callee filtering; native pipeline does not auto-classify). `list_categories()` now returns 7 entries."

**Migration Impact:** Strictly additive. `snapshot_version` stays `"1.0"`. Concrete observable effects:

1. **`list_categories()` returns 7 entries.** TS/Rust consumers that hard-coded the count (despite M23 warning against it) will fail their assertions. Consumers that call `list_categories()` properly pick up the new entry transparently.
2. **`[thresholds.overrides.logging]` becomes a legal override block.** The override loader is category-agnostic; no code path changes. Configs that previously set this block expecting "unknown category warning" semantics will now have their override resolved against the category. The `expires` requirement (Rule 12 / KD12) applies.
3. **No change in native snapshot output.** Because the category is catalog-only, snapshots produced by `Pipeline::snapshot` show no `logging` entries unless the consumer pipeline that wraps sdivi-rust populates them. A pure-`sdivi`-CLI run sees no change in `pattern_metrics`.
4. **WASM consumer behavior change.** Meridian and similar embedders that pass `PatternInstanceInput { category: "logging", … }` will start seeing those instances flow through unchanged (previously the M23 category-contract surface would have flagged `"logging"` as out-of-catalog if the embedder had performed a strict check). This is the actual user-visible win.

**Files to create or modify:**
- **Create:** `crates/sdivi-patterns/src/queries/logging.rs` — documentation-by-source; `NODE_KINDS` is reference-only, not wired.
- **Modify:** `crates/sdivi-patterns/src/queries/mod.rs` (module decl, `ALL_CATEGORIES`, length test rename, sentinel-behavior regression test). **Do not modify** the `category_for_node_kind` body.
- **Modify:** `crates/sdivi-core/src/categories.rs` (`CATALOG_ENTRIES`, `CATEGORIES`, doc-test length).
- **Modify:** `docs/pattern-categories.md` (canonical table, per-language tables with the consumer-responsibility note, embedder-responsibilities paragraph).
- **Modify:** `CHANGELOG.md` (Added section).
- **Do not modify:** `crates/sdivi-lang-*/src/extract.rs`. No language adapter changes — `PATTERN_KINDS` already collects the relevant node kinds (M29 added Python `"call"`), and they continue routing to their existing categories (`data_access`, `resource_management`). Foreign extractors handle logging on their side.

**Acceptance criteria:**
- `cargo test --workspace` passes, including:
  - `all_categories_has_seven_entries` (renamed from six).
  - `logging_is_in_all_categories`.
  - `category_for_node_kind_never_returns_logging` (sentinel regression — explicitly enforces the catalog-only contract).
- `cargo test -p sdivi-core` passes — specifically:
  - `categories_constant_matches_list_categories` (length 7 on both sides).
  - `no_category_string_in_patterns_src_missing_from_list_categories` (M23 grep gate — green because nothing routes `Some("logging")`; the only `"logging"` string in sdivi-patterns is inside `ALL_CATEGORIES` which the grep doesn't match).
  - `markdown_table_matches_list_categories_output` (M23 bidirectional doc parity — green because `docs/pattern-categories.md` adds `logging` to the canonical table).
- `cargo build -p sdivi-core --target wasm32-unknown-unknown --no-default-features` succeeds.
- `cargo tree -p sdivi-core --target wasm32-unknown-unknown --no-default-features` still reports zero entries for `tree-sitter*`, `walkdir`, `ignore`, `rayon`, `tempfile`.
- `wasm-pack test --node bindings/sdivi-wasm` passes, including an updated assertion that `list_categories()` returns 7 entries with `logging` present.
- `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` pass.
- `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS=-D warnings` passes — `logging.rs` module-level docstring documents the catalog-only design; `NODE_KINDS` has its own doc comment.
- A snapshot run against `tests/fixtures/simple-typescript` produces **no** `logging` entries in `pattern_metrics` (sentinel — the native pipeline does not classify by node kind). A test that submits a synthetic `PatternInstanceInput { category: "logging" }` to `compute_pattern_metrics` produces a non-empty `logging` bucket in the output.

**Tests:**
- Unit (`queries/mod.rs`): `all_categories_has_seven_entries`, `logging_is_in_all_categories`, `category_for_node_kind_never_returns_logging` (the sentinel regression).
- Doc test (`categories.rs`): `CATEGORIES.len() == 7`; the example for `list_categories` continues to assert `schema_version == "1.0"` and non-empty `categories` rather than a specific length, so future categories don't keep rewriting examples.
- Integration (`category_contract.rs`, exists): the four existing tests in M23 cover length parity, runtime/docs parity, and grep-drift. All four should pass without modification — the catalog-only design is what keeps them green.
- Compute-level test in `sdivi-core` (new or extended): synthesise a `Vec<PatternInstanceInput>` containing one instance with `category: "logging"` and call `compute_pattern_metrics`. Assert the resulting per-category metric map contains `"logging"` with the expected entropy / instance count. This is the "consumer round-trip" gate — proves that catalog-only doesn't mean "broken," just "embedder-supplied."
- Integration (WASM, `bindings/sdivi-wasm`): `list_categories()` from JS returns 7 entries; `data_access` and `logging` both present; `schema_version === "1.0"`.

**Watch For:**
- **`ALL_CATEGORIES` advertises `logging` but `category_for_node_kind` never returns it.** This is intentional. The module docstring on `logging.rs` and the new `category_for_node_kind_never_returns_logging` regression test together document the design. A reader who tries to "fix the inconsistency" by adding a routing branch will trip the regression test — that's by design. The PR description for this milestone should call this out so reviewers don't request the "fix."
- **The M23 grep gate (`no_category_string_in_patterns_src_missing_from_list_categories`) is not the safety net here.** The grep only catches `Some("…")` literals. `"logging"` appears in `ALL_CATEGORIES` (an array literal) and in `logging.rs` module text — neither pattern triggers the grep. The load-bearing test for this milestone is `categories_constant_matches_list_categories` (length parity) and `markdown_table_matches_list_categories_output` (doc parity). Both will fail if `CATALOG_ENTRIES` and `docs/pattern-categories.md` aren't updated together. Confirm both gates go green in CI before merge.
- **`CATALOG_ENTRIES` index plumbing in `CATEGORIES` strikes again.** Alphabetical insertion of `logging` between `error_handling` and `resource_management` shifts indices 3+ by one. Re-verify each `CATALOG_ENTRIES[N].0` in the `CATEGORIES` const after the edit; M29's Watch For already flagged this as an ergonomic hazard. Until the const is rewritten (Seeds Forward), each pattern-category milestone repeats this risk.
- **Doc-comment placement (CLAUDE.md convention).** When inserting the new tuple in `CATALOG_ENTRIES` and a new module in `queries/mod.rs`, ensure a blank line separates the new entry from the preceding `///` block, or the existing doc block silently re-attaches to the new item.
- **Per-language tables in `docs/pattern-categories.md` need *every* language sub-section updated.** The bidirectional doc-parity test (`markdown_table_matches_list_categories_output`) only checks the canonical-category-list table, not the per-language tables. The per-language tables are checked by human review on every category-adding milestone — a missed sub-section is a documentation drift surface, not a CI failure. Audit them deliberately in the PR.
- **Don't create a separate `logging::matches_callee(text, language) -> bool` helper.** That is what the next-level callee filter would look like, and it belongs to the future milestone that promotes callee filtering into the native pipeline, not here. Including it here would tempt a future refactor to "just wire this up" without the contract bump that change actually requires.
- **The sentinel test (`category_for_node_kind_never_returns_logging`) is load-bearing.** It encodes the design decision in code. Removing it under the assumption that "this is obvious" is the same class of mistake as removing a regression test for a fixed bug — the design intent gets forgotten and the next contributor re-introduces the routing. Keep the test, keep its comment.
- **`#![deny(missing_docs)]` on `sdivi-core`.** The new `CATALOG_ENTRIES` tuple, the bumped `CATEGORIES` const length, and any new field on `CategoryInfo` (there should be none) all need doc coverage. `logging.rs` in `sdivi-patterns` is not under `deny(missing_docs)` — but document it anyway, per CLAUDE.md doc-discipline section.
- **No `sdivi-cli` change needed.** The exit-code surface, the help text, and the JSON/text formatter are all category-agnostic. Confirm `cargo test -p sdivi-cli` is green without edits.

**Seeds Forward:**
- **M31 — `class_hierarchy`.** Different shape entirely: class/interface declaration and inheritance constructs (`class_declaration`, `class_definition`, `interface_declaration`, plus the heritage / `bases` children). These are structurally distinct from `call_expression` / `call` / `macro_invocation`, so M31 can be a *routing* milestone (extend `category_for_node_kind`, add to `ALL_CATEGORIES`, add to `CATALOG_ENTRIES`) like M29, not a catalog-only one like M30.
- **Promote callee-text filtering into the native pipeline.** When (not if) an adopter needs native precision for the `data_access` vs `logging` split:
  1. Add `callee: Option<&str>` (or equivalent) to the `category_for_node_kind` signature, sourced from `PatternHint.text`.
  2. Add per-language regex tables: TS/JS `console\.|logger\.|log\.`, Python `^(logging\.|print$)`, Rust `^(tracing|log)::`, Go `^fmt\.Print`.
  3. Promote `logging::NODE_KINDS` from documentation-only to a real routing branch — but the routing must be **callee-conditional**, i.e., `call_expression` AND callee matches the logging regex → `logging`; `call_expression` AND callee does not match → `data_access`.
  4. Update the M30 sentinel test from "never returns logging" to "returns logging only for matching callees, returns data_access otherwise."
  5. This is a SemVer-relevant change to `compute_pattern_metrics` output distribution. It does not require a `snapshot_version` bump — categories are still the same set — but it does shift the per-category instance counts on every snapshot. CHANGELOG note required.
  6. Estimate: one milestone, ~400-line surface change, two-day implementation, one week of verification on real-world fixtures.
- **`logging::NODE_KINDS` rotting.** Because the const is not consulted by `category_for_node_kind`, no test fails if a new logging-relevant node kind appears (e.g., a future tree-sitter grammar adds a dedicated `log_call` node). The list is reference documentation and will drift. Acceptable for v0; revisit when M31 lands and the conversation about callee filtering is open anyway.
- **A `category_origin` field on `CategoryInfo`.** If more catalog-only categories accumulate, embedders may want to introspect which categories are populated natively vs by consumer-only. A `CategoryInfo { name, description, origin: NativeRouted | EmbedderOnly }` field would surface this. Out of scope for v0; `list_categories()` is already the canonical discovery surface and the docstring on each entry conveys this in prose.
- **Bidirectional sync of `ALL_CATEGORIES` and `CATEGORIES`.** Currently nothing tests that `sdivi_patterns::queries::ALL_CATEGORIES` and `sdivi_core::CATEGORIES` are equal. After this milestone they intentionally are equal (both 7 entries), but a future catalog-only addition could let them drift. A small CI test asserting set equality between the two would close this gap — defer until a category does drift; trivial to add when needed.
