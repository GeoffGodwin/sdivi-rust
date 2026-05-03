#### Milestone 23: Pattern Category Contract + WASM `list_categories()`

<!-- milestone-meta
id: "23"
status: "pending"
-->

**Scope:** Establish the canonical pattern-category schema as a versioned, documented contract — the canonical category names (`error_handling`, `data_access`, `logging`, etc.), their expected tree-sitter node-kind shapes per language, and the normalization rules that produce a `blake3` fingerprint. Ship as `docs/pattern-categories.md` versioned to `snapshot_version "1.0"`. Add a `list_categories() -> CategoryCatalog` WASM export so embedders can discover the contract at runtime instead of hard-coding category names. Doc + runtime ship together so consumers can't drift from the contract.

**Why this milestone exists:** Currently the canonical category list lives implicitly in `crates/sdivi-patterns/src/` as match-arm strings — readable to a sdivi-rust contributor, opaque to an embedder. Meridian is building its own tree-sitter extractors (it doesn't go through `sdivi-parsing`) and needs to know exactly which AST subtrees map to which category so its extracted `PatternInstanceInput.category` strings match what `compute_pattern_metrics` expects. Without a shared contract, Meridian's categories diverge silently from sdivi-rust's, and `convention_drift_rate` becomes meaningless across the boundary.

**Deliverables:**
- Audit `crates/sdivi-patterns/src/` to enumerate every canonical category name in current use. Confirm with a grep across the workspace that no category string appears outside the patterns crate without crossing this contract. Capture the exhaustive list.
- Write `docs/pattern-categories.md`. Sections:
  1. **Versioning** — bound to `snapshot_version "1.0"`; categories are reserved forever once introduced (Rule 10 spillover); changes within a snapshot version may add but never remove or rename.
  2. **Canonical category list** — table with category name, one-paragraph definition, the kinds of code constructs it covers.
  3. **Per-language node-kind mappings** — for each supported language, a table of tree-sitter node-kind strings that count as instances of each category. Where a category requires a specific child structure (e.g. `try_statement` → `error_handling` only when paired with a `catch_clause`), document the structural constraint.
  4. **Normalization rules** — the `NormalizeNode` shape, what subtrees are stripped or canonicalised before fingerprint computation, the `blake3` key constant. Cross-reference `normalize_and_hash` rustdoc.
  5. **Embedder responsibilities** — what an embedder must do to produce `PatternInstanceInput` values that round-trip with native sdivi-rust output: same category strings, same normalization, same fingerprint computation via `normalize_and_hash`.
- Add a `pub fn list_categories() -> CategoryCatalog` in `sdivi-core` that returns a `CategoryCatalog` struct containing the canonical category list plus version metadata. Source of truth lives in code (`const CATEGORIES: &[&str] = &[...]`), not docs — the docs render from the constant via a doc-test snippet to keep them in sync.
- Define `CategoryCatalog`:
  ```rust
  pub struct CategoryCatalog {
      pub schema_version: &'static str, // matches snapshot_version
      pub categories: Vec<CategoryInfo>,
  }
  pub struct CategoryInfo {
      pub name: String,
      pub description: String,
  }
  ```
  Tsify-derive for WASM use.
- Export `list_categories()` from `bindings/sdivi-wasm/src/exports.rs` as a `#[wasm_bindgen]` function returning `CategoryCatalog`.
- Add a CI gate `tests/category_contract.rs` that asserts every category string used inside `crates/sdivi-patterns/src/` (discovered via grep at test time) is present in `list_categories()` output. Catches drift between code and contract.

**Migration Impact:** Strictly additive. The category strings already in use are unchanged. Existing snapshots and existing embedders that hard-coded category names continue to work — the contract simply documents what they were already doing. New embedders should consume `list_categories()` instead of hard-coding. `snapshot_version` stays `"1.0"`.

**Files to create or modify:**
- **Create:** `docs/pattern-categories.md` — the versioned contract.
- **Create:** `crates/sdivi-core/src/categories.rs` — `CategoryCatalog`, `CategoryInfo`, `list_categories()`, `const CATEGORIES`.
- **Modify:** `crates/sdivi-core/src/lib.rs` — re-export the catalog types and the function.
- **Modify:** `crates/sdivi-patterns/src/` — replace any inline category-string `&str` literals with references to the constant in `sdivi-core::categories::CATEGORIES`. (Mechanical refactor — same strings, single source.)
- **Create:** `bindings/sdivi-wasm/src/exports.rs` — add `list_categories()` `#[wasm_bindgen]` wrapper.
- **Create:** `crates/sdivi-core/tests/category_contract.rs` — drift-detection test.
- **Modify:** `bindings/sdivi-wasm/README.md` — show `list_categories()` usage.
- **Modify:** `CHANGELOG.md` — under Added.

**Acceptance criteria:**
- `cargo test -p sdivi-core` passes, including the new `category_contract.rs` test.
- `cargo build -p sdivi-core --target wasm32-unknown-unknown --no-default-features` succeeds.
- `wasm-pack test --node bindings/sdivi-wasm` passes, including a test that calls `list_categories()` and asserts the returned `schema_version === "1.0"` and the `categories` array length matches the expected count.
- The `docs/pattern-categories.md` doc-test (which calls `list_categories()` and pretty-prints the table) renders the same set of categories as the markdown table earlier in the file. Test framework: a Rust integration test that parses the markdown table and compares to `list_categories()` output.
- `cargo doc --workspace --no-deps` passes with `RUSTDOCFLAGS=-D warnings`.
- `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` pass.

**Tests:**
- Unit: `list_categories()` returns `schema_version == "1.0"` and a non-empty `categories` vec.
- Unit: `list_categories()` is referentially transparent — two calls return equal values.
- Integration (`category_contract.rs`): every category string discovered via grep across `crates/sdivi-patterns/src/` is present in `list_categories()` output. (This is the drift gate.)
- Integration: the markdown table in `docs/pattern-categories.md` and the runtime `list_categories()` output enumerate the same set of categories.
- Integration (JS): `list_categories()` callable from WASM, returns expected shape.

**Watch For:**
- **Single source of truth in code, not docs.** The `const CATEGORIES` array drives the markdown table (via doc-test or a one-off generator script committed to `tools/`). Docs that drift from code are inevitable; tests that catch drift are the only durable solution.
- **Per-language node-kind mappings are language-specific and may grow.** A new language adapter (post-MVP) will add new node-kind strings for the same categories. The schema version `"1.0"` covers the *category set*; per-language node-kinds are an implementation detail that can grow within a snapshot version. Document this distinction.
- **Reserved-forever invariant.** Once a category name appears in `list_categories()`, it cannot be removed within a `snapshot_version`. A category that becomes obsolete must be marked deprecated in the description but still returned — Meridian or any other embedder may have stored snapshots referencing it, and `compute_delta` must keep working.
- **Tsify derivation for `CategoryCatalog` and `CategoryInfo`.** Both must be `Tsify` so the WASM consumer gets the strict-TS type. Test by running `tsc --noEmit` against the generated `.d.ts`.
- **`#![deny(missing_docs)]` on `sdivi-core`.** The new `categories` module, every pub item in it, every field of `CategoryInfo` needs a doc comment with an `# Examples` block where meaningful (`list_categories` definitely needs one).
- **Don't add a "category" enum.** Use `String` for category names. An enum couples the binding ABI to the category set and forces a binding bump every time a category is added; a string is open-ended and the contract is enforced at the higher `list_categories()` layer.
- **Doc-comment placement.** Per CLAUDE.md, when inserting `CategoryCatalog` and `CategoryInfo` adjacent to existing items in `categories.rs`, ensure blank lines separate the doc blocks.

**Seeds Forward:**
- A `categories.json` machine-readable export (sibling to `list_categories()`, generated at build time and shipped under `docs/`) becomes worthwhile if a non-WASM, non-Rust consumer (e.g. a shell-script CI integration) needs to enumerate categories. Defer until requested.
- Per-language node-kind tables in `docs/pattern-categories.md` are written by hand for the v0 supported languages. A future milestone could derive them from the tree-sitter queries themselves to eliminate the doc/code drift surface area entirely. Out of scope here — the manual table plus the contract test are sufficient for v0.
- If a category is ever truly retired (vs deprecated), that requires a `snapshot_version` bump per Rule 16 — not a small decision. Document the bump procedure in `docs/pattern-categories.md` Versioning section so the cost is visible to future contributors.

---
