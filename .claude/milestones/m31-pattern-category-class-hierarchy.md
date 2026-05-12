
#### Milestone 31: Pattern Category — `class_hierarchy`

<!-- milestone-meta
id: "31"
status: "planned"
-->

**Scope:** Introduce `class_hierarchy` as the eighth canonical pattern category for `snapshot_version "1.0"`. Unlike M30 (`logging`, catalog-only), this is a **routing** milestone in the M29 shape — the node kinds involved (`class_declaration`, `class_definition`, `abstract_class_declaration`, `interface_declaration`, `impl_item`) are *not* claimed by any existing category, so `category_for_node_kind` can be extended with a real `class_hierarchy` branch and the language adapters' `PATTERN_KINDS` can be widened to collect declaration nodes. Adds the category to `sdivi_core::categories::CATALOG_ENTRIES`, wires the routing in `sdivi_patterns::queries`, updates four language adapters (TypeScript, Python, Rust, Java — Go skipped because Go has no class/interface AST shape), and updates `docs/pattern-categories.md`. Classification is purely node-kind based: every class/interface/impl declaration is classified as `class_hierarchy` regardless of whether it has a heritage clause, base classes, or trait conformance. Heritage-aware filtering is documented as an explicit deferral with a soundness argument — see Watch For.

**Why this milestone exists:** Of every category sdivi measures, `class_hierarchy` is the strongest divergence signal in object-oriented or component-based codebases. The same 5400-file TypeScript reference codebase that benchmarked the other categories showed `class_hierarchy` at **10.54 pattern entropy / 86% convention drift** — the highest of any category measured, by a wide margin. That reading is the canonical "architectural rot" signal: when component construction varies in five-plus structurally distinct ways across a codebase, it indicates conflicting paradigms layered over time (class inheritance here, composition there, mixins elsewhere, HOCs in a fourth corner). Adopters running sdivi on OOP-shaped codebases will look for this number first; without `class_hierarchy` as a category sdivi cannot produce that reading at all. The Python POC tracked it; sdivi-rust currently does not.

**Deliverables:**
- Create `crates/sdivi-patterns/src/queries/class_hierarchy.rs`:
  ```rust
  //! Node kinds classified as class-hierarchy patterns.
  //!
  //! These node kinds correspond to the `class_hierarchy` category in the
  //! [`PatternCatalog`](crate::catalog::PatternCatalog).

  /// Tree-sitter node kinds for class-hierarchy patterns.
  ///
  /// All declaration kinds are classified here regardless of whether the
  /// declaration has an `extends`/`implements`/`for Trait` clause. Heritage-
  /// aware narrowing is the embedder's responsibility (or a future native
  /// enhancement); the entropy/drift signal survives the broader collection
  /// because hierarchy-free declarations have low structural variance and
  /// therefore contribute low entropy.
  ///
  /// - `class_declaration`: TypeScript / Java / JavaScript class declarations.
  /// - `class_definition`: Python class definitions.
  /// - `abstract_class_declaration`: TypeScript abstract classes (always part
  ///   of a hierarchy by definition — they cannot be instantiated directly).
  /// - `interface_declaration`: TypeScript / Java interface declarations
  ///   (define the contract another type implements).
  /// - `impl_item`: Rust `impl Type {…}` and `impl Trait for Type {…}` blocks.
  pub const NODE_KINDS: &[&str] = &[
      "class_declaration",
      "class_definition",
      "abstract_class_declaration",
      "interface_declaration",
      "impl_item",
  ];
  ```
- Modify `crates/sdivi-patterns/src/queries/mod.rs`:
  - Add `pub mod class_hierarchy;` alphabetically (between `async_patterns` and `data_access` module decls).
  - Insert `"class_hierarchy"` in `ALL_CATEGORIES` alphabetically — slots between `"async_patterns"` and `"data_access"`.
  - Extend `category_for_node_kind` with a `class_hierarchy::NODE_KINDS` branch. Keep alphabetical branch order — slot the branch after `async_patterns` and before `data_access`.
  - Rename `all_categories_has_seven_entries` to `all_categories_has_eight_entries` and bump the constant to `8`.
  - Add three new unit tests (one per representative language):
    ```rust
    #[test]
    fn class_declaration_is_class_hierarchy() {
        assert_eq!(
            category_for_node_kind("class_declaration", "typescript"),
            Some("class_hierarchy")
        );
    }

    #[test]
    fn class_definition_is_class_hierarchy() {
        assert_eq!(
            category_for_node_kind("class_definition", "python"),
            Some("class_hierarchy")
        );
    }

    #[test]
    fn impl_item_is_class_hierarchy() {
        assert_eq!(category_for_node_kind("impl_item", "rust"), Some("class_hierarchy"));
    }

    #[test]
    fn interface_declaration_is_class_hierarchy() {
        assert_eq!(
            category_for_node_kind("interface_declaration", "java"),
            Some("class_hierarchy")
        );
    }

    #[test]
    fn abstract_class_declaration_is_class_hierarchy() {
        assert_eq!(
            category_for_node_kind("abstract_class_declaration", "typescript"),
            Some("class_hierarchy")
        );
    }
    ```
- Modify `crates/sdivi-core/src/categories.rs`:
  - Insert the new entry alphabetically into `CATALOG_ENTRIES` (between `async_patterns` and `data_access`):
    ```rust
    (
        "class_hierarchy",
        "Code constructs that establish inheritance, interface implementation, or trait \
        conformance relationships — e.g. classes with `extends`/`implements` clauses, \
        Python classes with base classes, and Rust `impl Trait for Type` blocks. All \
        declaration kinds are classified here regardless of whether they carry a \
        heritage clause; heritage-aware narrowing is the embedder's responsibility.",
    ),
    ```
  - Extend the `CATEGORIES` `&[&str]` constant with the new index. With M29 (`data_access`) and M30 (`logging`) merged, the post-M31 alphabetical layout of `CATALOG_ENTRIES` is `async_patterns, class_hierarchy, data_access, error_handling, logging, resource_management, state_management, type_assertions` — indices `[0..=7]`. Re-verify every `CATALOG_ENTRIES[N].0` in the `CATEGORIES` const after the edit (the hand-written indices are the recurring foot-gun across this family of milestones).
  - Bump the doc-test assertion `assert_eq!(CATEGORIES.len(), 7)` to `8`.
- Modify `crates/sdivi-lang-typescript/src/extract.rs`: add `"class_declaration"`, `"abstract_class_declaration"`, and `"interface_declaration"` to `PATTERN_KINDS`. **Note for the implementer:** these three kinds already appear in the adapter's separate `DECLARATION_KINDS` constant (used for the export-extraction code path) — that constant is unrelated to `PATTERN_KINDS` and must not be changed. The two constants exist independently; touching one does not implicate the other.
- Modify `crates/sdivi-lang-python/src/extract.rs`: add `"class_definition"` to `PATTERN_KINDS`. (Python's `PATTERN_KINDS` already gained `"call"` in M29; this milestone adds `"class_definition"` for the same broad-collection reason.)
- Modify `crates/sdivi-lang-rust/src/extract.rs`: add `"impl_item"` to `PATTERN_KINDS`. Note that `EXPORTABLE_KINDS` already lists `"trait_item"` etc. for export tracking; `impl_item` is *not* an exportable kind and is only being added to `PATTERN_KINDS`.
- Modify `crates/sdivi-lang-java/src/extract.rs`: add `"class_declaration"` and `"interface_declaration"` to `PATTERN_KINDS`. Same disjoint-constant caution — both kinds appear in `EXPORTABLE_KINDS` for a different purpose; leave `EXPORTABLE_KINDS` alone.
- **Do not modify** `crates/sdivi-lang-go/src/extract.rs`. Go has no class/interface AST shape (Go interfaces are duck-typed and do not produce hierarchy-shaped declarations); the category exists in the catalog but produces zero hits on Go fixtures. Document this in `docs/pattern-categories.md` (see below).
- **Do not modify** `crates/sdivi-lang-javascript/src/extract.rs` unless it has diverged from the TypeScript adapter (verify in the PR). JavaScript has `class_declaration` but lacks `interface_declaration` and `abstract_class_declaration` — only `class_declaration` is meaningful there.
- Update `docs/pattern-categories.md`:
  - Add a row `| class_hierarchy | Code constructs that establish inheritance, interface implementation, or trait conformance relationships… |` to the **Canonical category list** table, alphabetically (between `async_patterns` and `data_access`).
  - Add a `class_hierarchy` row to every per-language **node-kind mapping** sub-section:
    - **Rust:** `impl_item` — "All `impl` blocks, including inherent `impl Type {…}` (no trait) and trait conformance `impl Trait for Type {…}`. Inherent-only narrowing is the embedder's responsibility."
    - **Python:** `class_definition` — "All `class` definitions, including those with no base classes (which are effectively `class Foo(object)` and contribute low entropy)."
    - **TypeScript / JavaScript:** `class_declaration`, `abstract_class_declaration`, `interface_declaration` — "Abstract classes and interfaces always count. Concrete classes count regardless of `extends` / `implements`; entropy survives the broader collection because heritage-free classes have similar structure and contribute low entropy." (JavaScript: only `class_declaration` is emitted; `interface_declaration` and `abstract_class_declaration` are TS-only AST shapes.)
    - **Go / Java:** Java row `class_declaration`, `interface_declaration` — same broader-collection caveat. **Go:** "(none in v0 — Go has no class/interface AST shape; the duck-typed interface model does not surface as a `class_hierarchy` declaration. The category exists in the catalog so cross-language reporting is uniform, but it produces zero Go hits.)"
  - Update the **Embedder responsibilities** section to add a note: "The `class_hierarchy` category in `snapshot_version "1.0"` is wired natively but classified broadly — every declaration of the listed node kinds is included regardless of heritage. Embedders that want heritage-only precision (e.g. only classes with an `extends` clause, only `impl Trait for …` blocks) should filter `PatternInstanceInput` on their side before passing to `compute_pattern_metrics`. Entropy-based divergence signals remain meaningful under the broader collection because hierarchy-free declarations contribute low structural variance — see M31 for the soundness argument."
- Update `CHANGELOG.md` under the next-release `Added` section: "New pattern category `class_hierarchy` covering class, interface, and impl declarations across TypeScript, JavaScript, Python, Rust, and Java. Go is skipped (no class/interface AST shape). Adds `class_declaration` / `class_definition` / `abstract_class_declaration` / `interface_declaration` / `impl_item` to the relevant language adapters' `PATTERN_KINDS`."

**Migration Impact:** Strictly additive. `snapshot_version` stays `"1.0"`. Observable effects:

1. **`list_categories()` returns 8 entries.**
2. **Substantial expansion of `PatternHint` volume on OOP-shaped codebases.** Unlike M29 (where TS/Go already collected `call_expression` and only Python adapter actually expanded), this milestone expands collection in **four** adapters simultaneously — TS gains 3 new kinds, Python 1, Rust 1, Java 2. On a Java-heavy or Rust-heavy repo this is a measurable uplift in hint volume, blake3 fingerprint work, and snapshot file size. Benchmark before/after on a multi-thousand-file Java fixture (or the bifl-tracker reference codebase) and record the wall-clock and snapshot-size delta in the PR description.
3. **Existing snapshots see a new `class_hierarchy` bucket appear.** The pre-upgrade snapshot has no entries; post-upgrade it does. `compute_delta` between snapshots straddling this milestone shows `class_hierarchy` as a new top-level key. Document in CHANGELOG as a one-time recalibration event, same shape as the M29 note.
4. **Heritage-free declarations contribute "background" entropy.** A Python repo with 200 simple `class Foo: …` declarations and 5 hierarchical `class Bar(Foo): …` declarations sees most of its `class_hierarchy` instances as low-variance background. The 5 hierarchical declarations still produce the divergence signal because they introduce structural variation the background lacks. Document this so adopters don't read a moderate-entropy `class_hierarchy` reading on a class-heavy codebase as a false positive.
5. **`[thresholds.overrides.class_hierarchy]` becomes a legal override block.** Same `expires`-required rule as every other override.

**Files to create or modify:**
- **Create:** `crates/sdivi-patterns/src/queries/class_hierarchy.rs`
- **Modify:** `crates/sdivi-patterns/src/queries/mod.rs` (module decl, `ALL_CATEGORIES`, `category_for_node_kind` new branch, tests)
- **Modify:** `crates/sdivi-core/src/categories.rs` (`CATALOG_ENTRIES`, `CATEGORIES`, doc-test length)
- **Modify:** `crates/sdivi-lang-typescript/src/extract.rs` (add three kinds to `PATTERN_KINDS`)
- **Modify:** `crates/sdivi-lang-python/src/extract.rs` (add `"class_definition"` to `PATTERN_KINDS`)
- **Modify:** `crates/sdivi-lang-rust/src/extract.rs` (add `"impl_item"` to `PATTERN_KINDS`)
- **Modify:** `crates/sdivi-lang-java/src/extract.rs` (add `"class_declaration"`, `"interface_declaration"` to `PATTERN_KINDS`)
- **Modify:** `docs/pattern-categories.md` (canonical table + per-language tables, including the Go zero-hits note + embedder-responsibility note about broad collection)
- **Modify:** `CHANGELOG.md` (Added section)
- **Do not modify:** `crates/sdivi-lang-go/src/extract.rs`, `crates/sdivi-lang-javascript/src/extract.rs` (unless JavaScript adapter has diverged from TypeScript — verify in PR).

**Acceptance criteria:**
- `cargo test --workspace` passes, including the five new node-kind classification tests.
- `cargo test -p sdivi-core` passes — in particular:
  - `categories_constant_matches_list_categories` (length 8 on both sides).
  - `no_category_string_in_patterns_src_missing_from_list_categories` — green because `Some("class_hierarchy")` in `category_for_node_kind` is matched by the M23 grep gate, and `CATALOG_ENTRIES` includes `class_hierarchy`. **This is the load-bearing safety net for this milestone** — adding the routing branch without the catalog entry trips this test.
  - `markdown_table_matches_list_categories_output` — green only if `docs/pattern-categories.md` adds `class_hierarchy` to the canonical-category-list table.
- `cargo build -p sdivi-core --target wasm32-unknown-unknown --no-default-features` succeeds.
- `cargo tree -p sdivi-core --target wasm32-unknown-unknown --no-default-features` still reports zero entries for `tree-sitter*`, `walkdir`, `ignore`, `rayon`, `tempfile`.
- `wasm-pack test --node bindings/sdivi-wasm` passes, with the JS-side `list_categories()` assertion bumped to 8 entries and `class_hierarchy` membership checked.
- `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` pass.
- `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS=-D warnings` passes.
- A snapshot run against `tests/fixtures/simple-typescript` produces a non-empty `class_hierarchy` bucket in `pattern_metrics` (assuming the fixture defines at least one class — confirm in PR; if not, extend the fixture). Same for `simple-python`, `simple-java`, and `simple-rust` (one impl block is enough for Rust).
- A snapshot run against `tests/fixtures/simple-go` produces **zero** `class_hierarchy` entries (Go has no class AST shape). This is the negative-result sentinel — proves the per-language skip works.

**Tests:**
- Unit (`queries/mod.rs`): the five new classification tests above, plus the renamed/bumped `all_categories_has_eight_entries` and a `class_hierarchy_is_in_all_categories` membership test.
- Doc test (`categories.rs`): `CATEGORIES.len() == 8`; the `list_categories` example continues to assert `schema_version == "1.0"` and non-empty `categories` rather than a specific length (encourages future-proofing established in M30).
- Integration (`category_contract.rs`, exists): all four M23 tests continue to pass without modification — the routing addition produces a `Some("class_hierarchy")` literal that the grep gate matches against `CATALOG_ENTRIES`. Verify CI green.
- Integration (WASM, `bindings/sdivi-wasm`): the M29/M30-updated `list_categories()` JS test gets its expected count bumped from 7 to 8 and a `class_hierarchy` membership assertion added.
- Adapter-level fixture tests: extend `tests/fixtures/simple-typescript` to include a class with `extends`, a class without, an abstract class, and an interface; assert that all four are collected as `class_hierarchy` instances. Same shape for Java (`class extends`, plain class, interface). For Python, ensure the fixture has at least one `class Foo(Bar):` and one bare `class Baz:`. For Rust, ensure at least one `impl Trait for Type` block.
- Negative-result test for Go: assert `simple-go` fixture snapshot produces zero `class_hierarchy` instances.
- Determinism property test (`prop_test_pipeline_deterministic`, exists): must remain green. The new node-kind collection paths through tree-sitter are still deterministic.

**Watch For:**
- **Don't conflate `PATTERN_KINDS` and `EXPORTABLE_KINDS` in the adapter edits.** In TypeScript, `DECLARATION_KINDS`; in Java, `EXPORTABLE_KINDS`; in Rust, `EXPORTABLE_KINDS` — all three already list class / interface / trait declaration kinds for the export-extraction code path. **That is a separate concern from pattern hint collection.** Add only to `PATTERN_KINDS` in this milestone; leave the export-extraction constants untouched. A reviewer who skims the diff and sees "you already had `class_declaration` in this file" is reading the wrong constant.
- **Broad classification is deliberate.** Every class/interface/impl declaration becomes a `class_hierarchy` hint regardless of whether it has an `extends`/`implements`/`for Trait` clause. The soundness argument is in the spec deliberately and bears repeating in the PR description: heritage-free declarations have low structural variance and therefore contribute *low* entropy; hierarchical declarations introduce variation that produces the divergence signal. The signal is the difference, not the absolute count. A reviewer who says "this categorises non-hierarchical classes as `class_hierarchy`" is correct — but the resulting metric still measures what we want it to measure. If a future adopter insists on heritage-only narrowing, that is the same callee-text / structural-filtering work flagged for `data_access` and `logging` in M29/M30 Seeds Forward — a separate milestone.
- **Rust `impl_item` covers both inherent and trait impls.** `impl Foo {…}` (no trait) and `impl Trait for Foo {…}` produce the same node kind. The category therefore counts inherent impls as `class_hierarchy` even though they are not strictly hierarchical. Same broad-collection argument as above — inherent impls have low variance and contribute low entropy.
- **Go is genuinely skipped, not "skipped in v0."** Go interfaces are structurally typed; there is no AST declaration that establishes a `Type1 implements Interface1` relationship. The category exists in the contract so cross-language reporting is uniform, but the Go adapter has nothing to contribute. Document this in `docs/pattern-categories.md` Go sub-section and in the milestone PR description so a future contributor doesn't try to "fill in the gap."
- **JavaScript adapter may be a shared module with TypeScript.** Verify in the PR whether `crates/sdivi-lang-javascript/src/extract.rs` is a real file or a re-export of the TS extractor. If real, only `class_declaration` is meaningful (JavaScript has no `interface_declaration` or `abstract_class_declaration` AST kinds). If shared, the TS edit covers both.
- **`CATEGORIES` const index plumbing — third and last time.** With this milestone closing the M29/M30/M31 trio, the const goes from 7 to 8 entries with the new entry inserted at index 1. Every subsequent index shifts by one. If the recurring fragility of this const has not been addressed by Seeds Forward in M29/M30, this milestone is the final reminder before the next contributor inherits the same hazard. Consider closing the Seeds Forward debt as a small cleanup before or after this milestone.
- **Doc-comment placement (CLAUDE.md convention).** When inserting the new `CATALOG_ENTRIES` tuple between `async_patterns` and `data_access`, ensure a blank line separates the new entry from the preceding `///` block, or the existing doc block silently re-attaches to the new item and `#![deny(missing_docs)]` fails the build on the original constant.
- **Hint volume uplift can shift `pattern_entropy_rate` numbers.** Adopters with thresholds set against a pre-M31 baseline will see a one-time recalibration on the next snapshot post-upgrade. This is the canonical "additive category" event — flag it in CHANGELOG and in `docs/migrating-from-the-python-poc.md` if that doc references threshold-tuning advice. The `[thresholds.overrides.class_hierarchy]` block with an `expires` date is the documented escape hatch for adopters who need to defer recalibration until they have time to retune.
- **`#![deny(missing_docs)]` on `sdivi-core`.** The new `CATALOG_ENTRIES` tuple and the bumped `CATEGORIES` const length need doc coverage; the new module-level docstring on `class_hierarchy.rs` should explain the broad-collection design choice in one paragraph.
- **No `sdivi-cli` change needed.** Same as M29 / M30 — the CLI is category-agnostic. Confirm `cargo test -p sdivi-cli` is green without edits.

**Seeds Forward:**
- **Heritage-aware filtering.** If an adopter needs precision parity with their consumer's filter, the work is roughly: (1) inspect each `PatternHint`'s child structure to detect `extends_clause`, `implements_clause`, `class_heritage`, `bases`, `for_clause`, etc.; (2) emit two sub-categories (`class_hierarchy_with_heritage`, `class_hierarchy_inherent`) or a single category gated by a structural predicate. Same shape of design tension as the `data_access`-vs-`logging` callee-filtering deferral from M30 Seeds Forward — and the same answer: defer until a real adopter asks. The broad-collection metric is meaningful for the divergence-signal use case.
- **Per-language `extends`/`implements` count as a separate metric.** A future milestone could expose "average inheritance depth" or "interface-implementation count distribution" as supplementary `pattern_metrics` fields alongside the existing entropy/drift numbers. This is closer to a "code complexity" measurement than to a divergence measurement and arguably outside sdivi's stated scope ("we measure drift, not quality") — but it is the kind of supplementary number adopters often ask for when the `class_hierarchy` entropy reading is high. Record the conversation; defer the decision.
- **`CATEGORIES` const refactor.** This is the third time the index-plumbing fragility has appeared as a Watch For. A small follow-up milestone (`crates/sdivi-core/src/categories.rs` rewrite to `LazyLock<Vec<&'static str>>` or `const fn` over `CATALOG_ENTRIES`) closes it permanently. Mechanical, low-risk, no contract change — should not wait for a fourth category-adding milestone.
- **Cross-category entropy comparison view.** Now that the catalog covers eight categories with real-world divergence ranges (low ~4.83, mid ~5.54, high ~10.54 from the reference TypeScript codebase), `sdivi show` could grow a `--compare-categories` flag that highlights the category with the highest entropy relative to the rest. Useful UX; defer to a later UX milestone.
- **Per-language fixture coverage matrix.** The `tests/fixtures/simple-*` family will, after this milestone, exercise: error_handling, async_patterns, state_management, type_assertions, resource_management, data_access (M29), class_hierarchy (this), and — only via embedder input — logging. A small `tests/fixtures/all-categories/<lang>/` per-language fixture suite that intentionally exercises every native-routed category in every language would catch regressions in the routing tables. Useful; defer until M28-style performance work or a similar regression-heavy milestone makes the upkeep cost obvious.
