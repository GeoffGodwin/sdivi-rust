
#### Milestone 32: Callee-Text Classification API — `classify_hint`

<!-- milestone-meta
id: "32"
status: "planned"
-->

**Scope:** Introduce `pub fn classify_hint(hint: &PatternHint, language: &str) -> Vec<&'static str>` in `sdivi_patterns::queries`, the additive API that promotes pattern classification from node-kind-only to node-kind + callee-text. Add per-language `matches_callee` helpers in the relevant per-category modules (`data_access`, `logging`, `async_patterns`, `resource_management`). Expose `classify_hint` via `sdivi-core` re-export and `bindings/sdivi-wasm`. **The native pipeline is intentionally not switched in this milestone** — `crates/sdivi-patterns/src/catalog.rs:102` still calls `category_for_node_kind`, so `Pipeline::snapshot` output is unchanged. Foreign extractors (Meridian, custom WASM consumers) can opt into `classify_hint` immediately and drop their hand-rolled callee filters; CLI users see no behavioural change. M33 flips the pipeline switch.

**Why this milestone exists:** After M29/M30/M31 the system is code-stable but design-incomplete: every `call_expression`/`call` is bucketed into `data_access` regardless of callee (so `console.log(…)`, `Math.max(…)`, `arr.push(…)` all count as data access), every `macro_invocation` is bucketed into `resource_management` regardless of macro name (so `tracing::info!` counts as resource management), and `logging` is empty on every native CLI run because `category_for_node_kind` never returns it. The CLI is a first-class v0 surface (per the working decision recorded after M30) so this gap is real. The clean fix needs callee-text inspection — every `PatternHint` already carries `text: String` (the truncated source text of the node, up to 256 bytes), so the data is sitting there waiting to be used. Splitting the work into "API exists" (M32) and "pipeline uses the API" (M33) isolates two distinct risks: M32 has the design surface (regex tables, function shape, WASM compatibility) and M33 has the behavioural risk (snapshot numbers shift for every adopter on upgrade).

**Deliverables:**
- Add `regex` as a workspace dependency for `sdivi-patterns`. Use the workspace `Cargo.toml` to pin a single version. Verify it does not enter the `sdivi-core --target wasm32-unknown-unknown --no-default-features` dependency tree until intentionally added (it should — `regex` is needed in WASM too — but the addition must be deliberate and reflected in the `cargo tree` CI assertion).
- Modify `crates/sdivi-patterns/src/queries/data_access.rs`:
  - Add `pub fn matches_callee(text: &str, language: &str) -> bool` returning `true` when `text` looks like a data-access call. Per-language regex tables (compiled once via `LazyLock<Regex>`):
    - **TypeScript / JavaScript / Go:** `^(fetch|axios)\b|\b(query|read|write|get|post|put|delete|patch)\(|\b(db|sql)\.|\.(query|read|write|fetch)\(`
    - **Python:** `^(open\(|requests\.|httpx\.|cursor\.|session\.|conn\.)`
    - **Rust:** none for v0 (Rust data access is library-shaped: `sqlx::query!`, `tokio::fs::read`, `reqwest::get` — left to a future regex pass; for now Rust `data_access` stays node-kind-empty).
    - **Java:** none for v0.
  - Each regex is a `LazyLock<Regex>` keyed by language; one compile per process. Regex tables are pure data; no allocation in the hot path.
- Modify `crates/sdivi-patterns/src/queries/logging.rs`:
  - Add `pub fn matches_callee(text: &str, language: &str) -> bool`. Per-language tables:
    - **TypeScript / JavaScript:** `^(console|logger|log)\.`
    - **Python:** `^(logging\.|print\b)`
    - **Go:** `^fmt\.(Print|Println|Printf|Errorf|Fprint|Sprint)`
    - **Rust:** `^(tracing|log)::|^(println|eprintln|print|eprint|dbg)!`
    - **Java:** `^(System\.(out|err)\.|logger\.|Log\.|LOG\.)`
  - This is the function that promotes `logging` from catalog-only (M30) to natively classifiable. M33 wires it into the pipeline; M32 only adds the function.
- Modify `crates/sdivi-patterns/src/queries/async_patterns.rs`:
  - Add `pub fn matches_callee(text: &str, language: &str) -> bool` for the `.then(…)` / `.catch(…)` / `.finally(…)` Promise-chain shapes that are AST-level `call_expression`s but conceptually async-pattern instances:
    - **TypeScript / JavaScript:** `\.(then|catch|finally)\(`
    - **Other languages:** none (await/async are node-kind-routed already).
- Modify `crates/sdivi-patterns/src/queries/resource_management.rs`:
  - Add `pub fn excludes_callee(text: &str, language: &str) -> bool`. The semantic is **inverted** from the others: returns `true` when the callee text indicates the macro is *not* resource management and should fall through to a different category (`logging` for Rust). This keeps `resource_management` as the default for `macro_invocation` while letting `tracing::info!` etc. land in `logging` instead.
    - **Rust:** `^(tracing|log)::|^(println|eprintln|print|eprint|dbg)!` — same regex as `logging::matches_callee` for Rust; if it matches, the macro is logging.
    - Other languages: returns `false` (no macros to disambiguate).
- Modify `crates/sdivi-patterns/src/queries/mod.rs`:
  - Add `pub fn classify_hint(hint: &PatternHint, language: &str) -> Vec<&'static str>`. The dispatch:
    ```rust
    pub fn classify_hint(hint: &PatternHint, language: &str) -> Vec<&'static str> {
        match hint.node_kind.as_str() {
            "call_expression" | "call" => {
                // Priority order matters — first match wins, but matches are mutually
                // exclusive in v0's regex tables (no callee shape matches two categories).
                if async_patterns::matches_callee(&hint.text, language) {
                    return vec!["async_patterns"];
                }
                if logging::matches_callee(&hint.text, language) {
                    return vec!["logging"];
                }
                if data_access::matches_callee(&hint.text, language) {
                    return vec!["data_access"];
                }
                vec![] // unrecognised call — no category, hint is dropped
            }
            "macro_invocation" => {
                if resource_management::excludes_callee(&hint.text, language)
                    && logging::matches_callee(&hint.text, language)
                {
                    return vec!["logging"];
                }
                vec!["resource_management"]
            }
            other => category_for_node_kind(other, language)
                .map(|c| vec![c])
                .unwrap_or_default(),
        }
    }
    ```
  - Document explicitly that `Vec` return is forward-looking — v0 always returns 0 or 1 entry (the regex tables are designed to be disjoint per language). The `Vec` shape is preserved so a future category that legitimately co-occurs with another (e.g. `console.error(err)` could plausibly be both `logging` and `error_handling`) does not require an API break.
  - Add unit tests covering each per-language regex case: a positive (`console.log("x")` → `["logging"]`), a negative (`Math.max(a, b)` → `[]`), a fall-through (`Object.keys(obj)` → `[]`), and a Rust macro disambiguation (`tracing::info!("hi")` → `["logging"]` vs `vec![1, 2, 3]` → `["resource_management"]`).
- Modify `crates/sdivi-core/src/lib.rs` (or `categories.rs` if the re-export lives there):
  - Re-export `classify_hint` from `sdivi-patterns::queries`. Add a doc comment cross-referencing `category_for_node_kind` and explaining the precision difference. This must be exposed from `sdivi-core` so WASM can reach it (`sdivi-core` is the WASM-compatible facade per KDD-12).
  - Re-export `PatternHint` from `sdivi-parsing` if it isn't already exposed. The WASM consumer needs to construct one to pass into `classify_hint`. (Verify in PR — `PatternHint` may already be re-exported as part of the existing input struct family. If not, add the re-export and tsify-derive it.)
- Modify `crates/sdivi-patterns/src/queries/mod.rs` rustdoc on `category_for_node_kind`:
  - Add a "See also" pointing to `classify_hint`. Document that `category_for_node_kind` is the **node-kind-only** classifier kept as a convenience for callers that have a node kind but no source text. Foreign extractors with full `PatternHint` access should prefer `classify_hint` for precision.
- Modify `bindings/sdivi-wasm/src/exports.rs`:
  - Export `classify_hint` as a `#[wasm_bindgen]` function. Tsify-derive `PatternHint` if not already done, so the JS signature is `classify_hint(hint: PatternHint, language: string): string[]`.
  - Add a `wasm-pack test --node` integration test asserting the JS-side signature: pass a constructed `PatternHint { node_kind: "call_expression", text: "console.log('x')", … }` with `language: "typescript"` and assert the return is `["logging"]`.
- Modify `docs/pattern-categories.md`:
  - Add a new top-level section **"Callee-text classification (`classify_hint`)"** after the existing **"Normalization rules"** section. Document each per-language regex table verbatim, with a worked example per language showing what gets matched into which category. Cross-reference the **"Embedder responsibilities"** section.
  - Update the **"Embedder responsibilities"** section to note: as of M32, embedders should call `classify_hint` instead of hand-rolling their own callee filter — the regex tables are now part of the canonical contract. The catalog-only note for `logging` (added in M30) gets a "**Updated in M32:**" addendum noting that `classify_hint` *does* return `["logging"]` for matching callees, while `category_for_node_kind` continues to never return it (the M30 sentinel still holds for the older API).
- Update `CHANGELOG.md` under the next-release `Added` section: "`sdivi_core::classify_hint(hint, language) -> Vec<String>` — callee-text-aware classification API. Returns multi-category support and inspects `PatternHint.text` for callee-name matching against per-language regex tables. Catalog-only `logging` (M30) is now natively classifiable when called via this API. The native pipeline (`Pipeline::snapshot`) continues to use `category_for_node_kind` in M32 — M33 will switch the pipeline over."

**Migration Impact:** Strictly additive. `snapshot_version` stays `"1.0"`. No native pipeline behaviour change in M32 — `Pipeline::snapshot` produces bit-identical output before and after this milestone (verify with the existing `prop_test_pipeline_deterministic` proptest plus a fixture-based snapshot diff). Embedder behaviour is opt-in: Meridian and other foreign extractors can switch from their hand-rolled callee filters to `classify_hint` at their own pace. The M30 catalog-only sentinel test (`category_for_node_kind_never_returns_logging`) is unchanged and remains green — it asserts a property of the older API, which M32 does not modify. The `regex` crate joins the WASM dependency tree (~250 KB compressed). Verify this does not push `bindings/sdivi-wasm`'s npm-published bundle past whatever size budget M24 set; if it does, evaluate `regex-lite` or a hand-rolled prefix matcher in Watch For follow-ups.

**Files to create or modify:**
- **Create:** none (all changes are additions to existing modules).
- **Modify:** `crates/sdivi-patterns/Cargo.toml` (add `regex` dep).
- **Modify:** `crates/sdivi-patterns/src/queries/data_access.rs` (`matches_callee`).
- **Modify:** `crates/sdivi-patterns/src/queries/logging.rs` (`matches_callee` — promotes the catalog-only category to natively classifiable).
- **Modify:** `crates/sdivi-patterns/src/queries/async_patterns.rs` (`matches_callee` for Promise chains).
- **Modify:** `crates/sdivi-patterns/src/queries/resource_management.rs` (`excludes_callee` for Rust macro disambiguation).
- **Modify:** `crates/sdivi-patterns/src/queries/mod.rs` (`classify_hint`, dispatch logic, tests, rustdoc cross-references on `category_for_node_kind`).
- **Modify:** `crates/sdivi-core/src/lib.rs` (re-export `classify_hint`; verify `PatternHint` re-export).
- **Modify:** `bindings/sdivi-wasm/src/exports.rs` (`#[wasm_bindgen]` wrapper for `classify_hint`; tsify-derive `PatternHint` if needed).
- **Modify:** `docs/pattern-categories.md` (new "Callee-text classification" section + embedder-responsibilities update).
- **Modify:** `CHANGELOG.md` (Added section).
- **Do not modify:** `crates/sdivi-patterns/src/catalog.rs` (the pipeline call site stays on `category_for_node_kind`; M33 switches it).

**Acceptance criteria:**
- `cargo test --workspace` passes, including the new per-language `classify_hint` tests.
- `cargo test -p sdivi-patterns` passes, including positive/negative/fall-through cases per language for each `matches_callee` function.
- `cargo test -p sdivi-core` passes — `category_for_node_kind_never_returns_logging` (M30 sentinel) is **still green** because `category_for_node_kind` is unchanged.
- `cargo build -p sdivi-core --target wasm32-unknown-unknown --no-default-features` succeeds with `regex` in the dependency tree.
- `cargo tree -p sdivi-core --target wasm32-unknown-unknown --no-default-features` reports `regex` as the *only* new entry vs the M31 baseline. No new entries for `tree-sitter*`, `walkdir`, `ignore`, `rayon`, `tempfile` (Rule 21 / KD21 invariant preserved).
- `wasm-pack test --node bindings/sdivi-wasm` passes, including a JS-side test that constructs a `PatternHint` and verifies `classify_hint` returns the expected `string[]`.
- `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` pass.
- `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS=-D warnings` passes.
- **Native snapshot output is bit-identical pre/post M32.** Snapshot the workspace fixtures (`tests/fixtures/simple-typescript`, `simple-python`, etc.) before and after the M32 commit; assert byte-equal JSON. This proves the pipeline-unchanged guarantee.
- `prop_test_pipeline_deterministic` remains green.
- Cross-platform determinism: regex-driven classification produces identical results in WASM and native for the same `PatternHint` + language. Add to `bindings/sdivi-wasm` test that runs the same `PatternHint` set through both native (via a Rust integration test) and WASM (via wasm-pack test) and asserts equal `Vec` results.

**Tests:**
- Unit (`data_access.rs`, `logging.rs`, `async_patterns.rs`, `resource_management.rs`): positive and negative `matches_callee` cases per language. Aim for 3–5 cases per regex (one obvious match, one obvious miss, one borderline that the regex must accept or reject deliberately).
- Unit (`queries/mod.rs::classify_hint`): the dispatch logic — `call_expression` priority order (`async` beats `logging` beats `data_access`), `macro_invocation` Rust disambiguation, fall-through to `category_for_node_kind` for non-call/non-macro kinds.
- Unit: `classify_hint` returns empty `Vec` for unrecognised callees (`Math.max(a, b)` in TS, `len(x)` in Python — neither is data, logging, nor async).
- Doc tests on `classify_hint` and each `matches_callee` with `# Examples` blocks. Keep examples short and language-explicit.
- Integration (`bindings/sdivi-wasm`): JS-side `classify_hint` shape + behaviour test as described above.
- Integration: bit-identity check for snapshot output pre/post M32 (the pipeline-unchanged guarantee).
- Property test: for any `PatternHint` whose `node_kind` is not `call_expression`/`call`/`macro_invocation`, `classify_hint(hint, lang)` returns the same result as `category_for_node_kind(&hint.node_kind, lang).map(|c| vec![c]).unwrap_or_default()`. This proves the fall-through path is consistent.
- Cross-platform: identical `Vec` results native vs WASM for a representative `PatternHint` set.

**Watch For:**
- **The pipeline is intentionally untouched.** A reviewer will reasonably ask "why add a function nobody calls?" The answer is that splitting the API addition from the behavioural switch isolates two risks; M33 immediately follows. Document this in the PR description prominently. The CHANGELOG note already says the pipeline still uses `category_for_node_kind` — keep that sentence.
- **Regex compilation cost.** `Regex::new` is non-trivial (parsing + DFA construction). Use `LazyLock<Regex>` per regex per language, not `Regex::new` per call. The hot path inside `classify_hint` should be a `LazyLock` deref + `is_match` call — single-digit microseconds. Benchmark on a fixture with hundreds of thousands of `call_expression` hints (the consumer-app TS sample is a good stress test) and confirm the regex check is not a hotspot.
- **WASM bundle size.** The `regex` crate adds ~250 KB compressed to the WASM bundle. If M24 set a published-package size budget, verify M32 stays under it. If not, the `regex-lite` crate (smaller, slower, fewer features) is a fallback — but the patterns we need are simple anchored prefixes, so `regex-lite` would suffice. Don't switch unless the size becomes a problem.
- **Per-language regex tables are part of the contract surface.** Once published, an adopter relies on `console.log` being classified as `logging`. Removing or narrowing a regex is a behavioural break and should require a `MIGRATION_NOTES.md` entry under the relevant version. Adding new shapes to a regex (broadening) is additive and acceptable.
- **`Vec` over `Option`.** The Vec return is the right call even though v0 always returns 0 or 1. Future categories that legitimately co-occur (e.g. `error_handling` + `logging` for `console.error(err)`) extend without an API break. Don't backslide to `Option<&'static str>` for "simplicity" — the caller cost of handling `Vec` is one `for` loop, and we already pay it for the M33 pipeline switchover.
- **Disjoint-regex invariant in v0.** The dispatch comment claims regex tables are disjoint per language (no callee shape matches two categories). Verify this with a property test: for every example callee in the test suite, at most one of (`async_patterns`, `logging`, `data_access`)'s `matches_callee` returns `true` for the same language. If this invariant ever needs to break (legitimate co-occurrence), the dispatch order in `classify_hint` becomes load-bearing — document the change at that point.
- **The M30 sentinel test stays green.** `category_for_node_kind_never_returns_logging` is about the older API. M32 does not touch `category_for_node_kind`. If a future contributor "fixes" the sentinel by reading the docstring and concluding that logging is now native, they will incorrectly delete the test — keep an inline comment in the sentinel pointing to M32: "`classify_hint` is the precision-aware classifier; this function is the node-kind-only fallback and continues to never return `logging` by design."
- **Regex anchoring matters.** `^console\.` vs `console\.` — the first matches `console.log("x")` but not `myconsole.log("x")`. The second matches both. Use `^` anchors or `\b` word boundaries deliberately; over-broad regexes silently misclassify. Each per-language regex needs a comment explaining the anchoring choice.
- **Truncated text in `PatternHint`.** `PatternHint.text` is truncated to 256 bytes (per `PatternHint` rustdoc). Long callees like `extremelyVerboseModulePath.someFunction(…)` may have their callee shape preserved (it's at the start of the text) but their argument list cut off. The regexes match only the prefix — that's fine for classification. Do not write a regex that depends on argument-list content; it will silently fail on long calls.
- **Doc-comment placement (CLAUDE.md convention).** When inserting `pub fn matches_callee` and `pub fn excludes_callee` into existing per-category modules, ensure each new pub item has its own `///` block separated by a blank line from neighbouring items. `#![deny(missing_docs)]` does not apply to `sdivi-patterns` directly, but every `pub` re-export from `sdivi-core` does require docs and an `# Examples` block where meaningful.
- **`PatternHint` re-export from `sdivi-core`.** If `PatternHint` is currently `sdivi-parsing`-only, M32 needs to re-export it from `sdivi-core` so WASM can construct one. Confirm in PR; if the re-export does not exist, add it with a tsify-derive so the JS-side type is strict.
- **No `sdivi-cli` change in M32.** The CLI does not call `classify_hint` directly — the pipeline does, and that switchover is M33. Confirm `cargo test -p sdivi-cli` is green without edits.

**Seeds Forward:**
- **M33 — pipeline switchover.** Modify `crates/sdivi-patterns/src/catalog.rs:102` to use `classify_hint` instead of `category_for_node_kind`. Multi-category Vec means a single hint can populate multiple buckets; v0 always returns 0 or 1 so the loop body in `catalog.rs` looks like `for category in classify_hint(hint, &record.language) { /* existing add-to-bucket logic */ }`. This is the behavioural-change milestone — adopters' threshold-tuned configs see one-time recalibration; CHANGELOG breaking-numbers note required.
- **Regex tables become per-version.** Once snapshots are produced under the M33 regex tables, snapshot/delta consumers depend on the classification being stable. A future regex change that re-classifies a callee shape from one category to another shifts the per-category instance counts; that is observable and must be a deliberate, documented change. `docs/pattern-categories.md` should grow a "Regex change log" sub-section under "Callee-text classification" — defer until the first regex revision lands.
- **`error_handling` could grow callee filtering too.** `try { … } catch (e) { console.error(e) }` — the `console.error` is logging, but the surrounding `catch_clause` is error_handling. Currently `catch_clause` is a node-kind-routed category, so the two coexist cleanly. If a future precision pass wants to track "error-handling calls" specifically (`assert(…)`, `panic!`, `throw new Error(…)`), it can grow an `error_handling::matches_callee`. Out of scope for v0 — `error_handling` is already the most node-kind-precise category we have.
- **A `Classifier` type (Seeds Forward from Seeds Forward).** If the per-category `matches_callee` functions accumulate enough configuration knobs (per-language regex tables, custom-regex injection, deprecation flags) to feel like a struct, package them into a `pub struct Classifier { … }` with a builder. Until that pressure exists, free functions are simpler and serializable. Defer until concrete pressure.
- **Promote `regex-lite` evaluation if WASM bundle size becomes a complaint.** The patterns are all anchored prefixes; `regex-lite` covers the surface area. The migration is a Cargo.toml swap and one or two regex syntax tweaks. Trivial; do it under user pressure, not pre-emptively.
- **Snapshot of regex tables exposed via WASM.** A future `list_callee_patterns(language: string) -> CalleePatternTable` WASM export lets foreign extractors mirror sdivi's exact regex set without re-typing them. Useful for embedders that want to apply the same filter outside `classify_hint` (e.g. as a streaming filter before constructing a `PatternHint`). Defer until requested; the current contract has the regex tables as code constants documented in `docs/pattern-categories.md`.
