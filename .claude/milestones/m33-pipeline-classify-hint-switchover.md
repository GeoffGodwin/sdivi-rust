
#### Milestone 33: Native Pipeline Switchover to `classify_hint`

<!-- milestone-meta
id: "33"
status: "planned"
-->

**Scope:** Switch the native pattern-catalog construction in `crates/sdivi-patterns/src/catalog.rs:102` from `category_for_node_kind` (node-kind-only) to `classify_hint` (node-kind + callee-text, M32). Multi-category `Vec` return is honoured: when `classify_hint` returns more than one category for a single hint, the hint is added to each category's bucket; when it returns the empty `Vec`, the hint is silently dropped (same as the current `None` path). Promotes `logging` from catalog-only (M30) to natively populated; tightens `data_access` to *actual* data-access calls instead of every `call_expression`/`call`; routes Promise-chain `.then()/.catch()/.finally()` calls into `async_patterns` instead of `data_access`; routes `tracing::info!`/`log::debug!` etc. into `logging` instead of `resource_management`. Updates `docs/pattern-categories.md` to remove the catalog-only caveat from the `logging` rows. CHANGELOG carries the user-visible "snapshot numbers will shift on upgrade" note. The M30 sentinel test (`category_for_node_kind_never_returns_logging`) stays green — `category_for_node_kind` is unchanged; the new behaviour is in the pipeline's choice of classifier, not the older function.

**Why this milestone exists:** M32 added the API but left the pipeline pinned to the old classifier. That deliberate split means M32 ships pure-additive (no snapshot diff for any adopter) and M33 carries the entire behavioural-change risk in isolation. CLI users — first-class v0 surface per the working decision recorded post-M30 — have been getting empty `logging` buckets and over-broad `data_access` buckets since M29 merged. M33 closes that gap. The change is meaningfully observable: an adopter's `pattern_metrics.logging.entropy` goes from `0.0` (literally no instances) to a real positive number, `pattern_metrics.data_access.instances` shrinks (calls that no longer match the data-access regex are dropped or re-routed), and `pattern_metrics.async_patterns.instances` grows (Promise chains now count). Threshold gates configured against M29-M31 baseline numbers will trip on the next snapshot post-upgrade unless adopters retune. The milestone exists because the gap is real and the v0 audience includes people who will run `sdivi snapshot` from the CLI and read the JSON directly.

**Deliverables:**
- Modify `crates/sdivi-patterns/src/catalog.rs`. The current loop body at lines 101–119 is:
  ```rust
  for hint in &record.pattern_hints {
      let Some(category) = queries::category_for_node_kind(&hint.node_kind, &record.language) else {
          continue;
      };
      let fp = fingerprint_node_kind(&hint.node_kind);
      // …add (category, fp) to entries…
  }
  ```
  The replacement:
  ```rust
  for hint in &record.pattern_hints {
      let categories = queries::classify_hint(hint, &record.language);
      if categories.is_empty() {
          continue;
      }
      let fp = fingerprint_node_kind(&hint.node_kind);
      let location = PatternLocation { /* …same as today… */ };
      for category in categories {
          let cat_map = entries.entry(category.to_string()).or_default();
          let stats = cat_map.entry(fp).or_insert(PatternStats {
              count: 0,
              locations: vec![],
          });
          stats.count += 1;
          stats.locations.push(location.clone());
      }
  }
  ```
  Note `location.clone()` inside the inner loop — `PatternLocation` carries an owned `PathBuf` so a clone per category-per-hint is required when a hint lands in multiple categories. v0's regex tables are disjoint per language (M32 invariant: at most one of `async_patterns`/`logging`/`data_access` matches a given callee), so in practice the inner loop runs once per hint; the clone path is exercised only by future categories that legitimately co-occur. Keep the `clone()` — it's correct, cheap on the rare path, and avoids an early optimization that would need to be undone the moment a co-occurring category is added.
- Modify `docs/pattern-categories.md`:
  - **Remove** the catalog-only caveat from the `logging` rows in the per-language tables. Replace with the actual node-kinds-and-callee-shape now in effect:
    - **Rust:** `macro_invocation` where callee matches `^(tracing|log)::|^(println|eprintln|print|eprint|dbg)!`.
    - **Python:** `call` where callee matches `^(logging\.|print\b)`.
    - **TypeScript / JavaScript:** `call_expression` where callee matches `^(console|logger|log)\.`.
    - **Go:** `call_expression` where callee matches `^fmt\.(Print|Println|Printf|Errorf|Fprint|Sprint)`.
    - **Java:** `call_expression` where callee matches `^(System\.(out|err)\.|logger\.|Log\.|LOG\.)`.
  - Update the `data_access` rows to reflect the new narrower shape (`call_expression`/`call` filtered by the data-access regex, not all calls).
  - Update the `async_patterns` rows in TS/JS to add the `.then()/.catch()/.finally()` shape alongside `await_expression`.
  - Update the `resource_management` Rust row to note that `macro_invocation` excludes the logging macros (tracing/log/println family).
  - In the **"Embedder responsibilities"** section, replace the M30 catalog-only paragraph for `logging` with: "As of M33 the native pipeline classifies `logging` via `classify_hint` against per-language callee regex tables. Embedders that pass `PatternInstanceInput { category: \"logging\" }` directly will continue to round-trip — their instances merge with the natively-classified ones in `compute_pattern_metrics` output. Embedders that previously hand-rolled their own logging filter should consider switching to `classify_hint` (M32) to stay aligned with the canonical regex set."
  - Update the **"Callee-text classification"** section header (added in M32) with a note that the regex tables are now load-bearing for native pipeline output, not just embedder convenience.
- Update `CHANGELOG.md` under the next-release `Changed` section (this is a behaviour change, not an addition):
  ```
  - The native pattern-catalog pipeline now classifies hints via `classify_hint`
    (M32) instead of `category_for_node_kind`. Per-category instance counts and
    entropy values shift on the next snapshot post-upgrade — `data_access`
    typically narrows (non-data calls are dropped), `logging` becomes non-empty
    on languages with logging regex tables, `async_patterns` grows on TS/JS
    (`.then()/.catch()` Promise chains), and Rust `macro_invocation` calls to
    `tracing::*!`/`log::*!`/`println!`-family macros land in `logging` instead
    of `resource_management`. Threshold gates configured against pre-M33
    baseline numbers may trip; use `[thresholds.overrides.<category>]` with
    an `expires` date to defer recalibration during migration. `snapshot_version`
    stays "1.0" — the contract has not broken; only the per-category instance
    distribution has shifted.
  ```
- Update `MIGRATION_NOTES.md` with the same content as the CHANGELOG entry, plus a worked example: pre-M33 vs post-M33 `pattern_metrics` JSON for a synthetic `simple-typescript` fixture, side-by-side, so an adopter can pattern-match the shape of their own recalibration.
- Update `crates/sdivi-patterns/src/queries/mod.rs` rustdoc on `category_for_node_kind`:
  - Strengthen the deferral language: "**Used internally by `classify_hint` for non-call/non-macro node kinds; the native pipeline no longer calls this function directly.** Foreign extractors with full `PatternHint` access should call `classify_hint`. This function is preserved for callers that have a node kind but no source text — and for backward compatibility with embedders that integrated against the M29 API."
- Modify `crates/sdivi-patterns/src/queries/mod.rs` tests:
  - Keep `category_for_node_kind_never_returns_logging` (M30 sentinel) — the function being unchanged is still a load-bearing invariant.
  - Add a *new* sentinel: `classify_hint_returns_logging_for_console_log` and `classify_hint_returns_logging_for_tracing_macro` — positive tests that capture the M33 promotion of `logging` to natively classified.
  - Add `classify_hint_drops_unrecognised_calls` — `Math.max(a, b)` returns `[]`, exercising the dropped-hint path.
- Update `tests/fixtures/simple-*` if any fixture's pre-M33 snapshot was wired into a regression test that asserts specific per-category instance counts. The right fix is to re-snapshot and commit the new expected output, with a comment in the test pointing to M33 as the cause.
- Update `bindings/sdivi-wasm` integration tests:
  - Any test that asserts the shape of a WASM-side `assemble_snapshot` output's `pattern_metrics` distribution needs its expected counts re-snapshotted post-M33. WASM consumers that bring their own `PatternInstanceInput` are unaffected — their inputs determine their outputs — but tests that exercise the full `Pipeline::snapshot` path through WASM (M22's change-coupling-in-WASM milestone surface) need re-baselining.

**Migration Impact:** Behaviour change with public-API stability preserved. `snapshot_version` stays `"1.0"`. The category set, the field shapes in `pattern_metrics`, and the JSON schema are all unchanged. What changes is the per-category instance distribution:

1. **`data_access` shrinks.** Pre-M33 every `call_expression`/`call` was a `data_access` instance. Post-M33 only callees matching the data-access regex are. On a typical TS/JS codebase this drops `data_access` instance count by an order of magnitude. The corresponding entropy reading also shifts — usually downward, because the dropped calls were structurally homogeneous (most function calls have similar AST shape) and were inflating the denominator without contributing variance.
2. **`logging` becomes non-zero.** Catalog-only since M30; now natively populated. Adopters who set `[thresholds.overrides.logging]` blocks with `expires` dates expecting the category to stay empty will see those overrides come into play. The override-resolves-on-expiry behaviour from Rule 12 / KD12 still applies.
3. **`async_patterns` grows on TS/JS.** Promise chains (`.then()/.catch()/.finally()`) are now classified into `async_patterns` instead of being dropped (in M32) or being `data_access` (in M29). Adopters tracking async-pattern entropy will see the metric become more meaningful — heterogeneous Promise/`async-await` mixing was previously invisible.
4. **`resource_management` shrinks on Rust.** Logging macros (`tracing::info!`, `log::debug!`, `println!`-family) leave the bucket; only "real" resource macros (`vec!`, `drop!`, `Box::new`-via-macro, etc.) remain. Entropy may shift either direction depending on whether the logging macros were homogeneous or varied.
5. **Threshold gates may trip.** Adopters who tuned `[thresholds.entropy_rate]` or per-category overrides against M29-M31 baseline numbers will see their `sdivi check` exit-10 logic change behaviour on the next snapshot. The escape hatch is documented (per-category overrides with `expires`); the migration story is the CHANGELOG/MIGRATION_NOTES paragraph.
6. **Foreign extractors are unaffected.** Meridian and other consumers that bypass the pipeline by emitting `PatternInstanceInput` directly continue to work — their inputs determine their outputs. If they have already migrated to `classify_hint` in M32, they are now aligned with the native pipeline; if they haven't, they are simply running the same hand-rolled filter they were before.

**Files to create or modify:**
- **Modify:** `crates/sdivi-patterns/src/catalog.rs` (the load-bearing change — single call site swap, multi-category loop body).
- **Modify:** `docs/pattern-categories.md` (per-language tables, embedder-responsibilities note, callee-text-classification header note).
- **Modify:** `crates/sdivi-patterns/src/queries/mod.rs` (rustdoc strengthening, two new positive sentinel tests, drop-unrecognised test).
- **Modify:** `CHANGELOG.md` (Changed section).
- **Modify:** `MIGRATION_NOTES.md` (worked-example side-by-side).
- **Modify:** `tests/fixtures/simple-*` snapshot expectations (re-baseline only the tests that assert specific instance counts; tests that assert structural properties are unaffected).
- **Modify:** `bindings/sdivi-wasm` integration tests if any assert pattern_metrics distribution shape.
- **Do not modify:** `crates/sdivi-patterns/src/queries/{data_access,logging,async_patterns,resource_management}.rs` — the regex tables and `matches_callee`/`excludes_callee` functions land in M32 and are not changed in M33.
- **Do not modify:** `crates/sdivi-core/src/categories.rs`, `CATALOG_ENTRIES`, `CATEGORIES` — the category set is unchanged.

**Acceptance criteria:**
- `cargo test --workspace` passes, including the two new positive sentinel tests for `classify_hint`.
- `cargo test -p sdivi-core` passes, including:
  - `category_for_node_kind_never_returns_logging` (M30 sentinel, still load-bearing — `category_for_node_kind` is unchanged).
  - `categories_constant_matches_list_categories` (length 8 still — no contract change).
  - `markdown_table_matches_list_categories_output` (the docs table updates are in per-language sections, not the canonical-category-list table).
- `cargo build -p sdivi-core --target wasm32-unknown-unknown --no-default-features` succeeds.
- `cargo tree -p sdivi-core --target wasm32-unknown-unknown --no-default-features` reports the same dependency set as M32 — M33 adds no new dependencies, only changes a single function call.
- `wasm-pack test --node bindings/sdivi-wasm` passes with re-baselined fixture expectations.
- `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` pass.
- `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS=-D warnings` passes.
- **Native snapshot output is intentionally NOT bit-identical pre/post M33.** This is the milestone's whole point. The fixture-snapshot tests need re-baselining; do not chase bit-equality with the M32 output. Document each re-baselined fixture in the PR description with a one-line "X moved from Y to Z" summary.
- `prop_test_pipeline_deterministic` remains green — same `Config` + same repo state still produces bit-identical output, just a different bit-identical output than M32 produced.
- `prop_test_delta_pure` remains green — `compute_delta` is referentially transparent regardless of how categories were assigned.
- A snapshot run against `tests/fixtures/simple-typescript` produces a non-empty `logging` bucket (was zero in M32). A snapshot run against `tests/fixtures/simple-rust` has `tracing::info!` instances landing in `logging` rather than `resource_management` (assuming the fixture imports `tracing` — extend the fixture if it doesn't).
- A snapshot run against `tests/fixtures/simple-go` has `data_access` containing only `db.*`/`sql.*`-shape calls and `logging` containing only `fmt.Print*`-shape calls; non-matching calls are dropped from both.

**Tests:**
- Unit (`queries/mod.rs`): two positive sentinels and one negative sentinel as above. The positive sentinels are the load-bearing M33 invariants — they encode the design decision in code so a future contributor cannot revert the pipeline to `category_for_node_kind` without tripping them.
- Integration: re-baselined fixture snapshot tests. For each `simple-*` fixture, capture the pre-M33 `pattern_metrics` map, capture the post-M33 map, commit the diff into the test as the new expected value with a comment `// re-baselined in M33: switched to classify_hint`. The diff is the milestone's evidence.
- Integration: a "no regression in determinism" test — run `Pipeline::snapshot` twice against the same fixture and assert byte-equal JSON. Same as `prop_test_pipeline_deterministic` but with an explicit fixture rather than a property-generated one.
- Integration: a `compute_delta` round-trip — capture two snapshots of the same fixture (commit 1, commit 2 with a deliberate logging-callee change), assert the resulting `DivergenceSummary` shows the expected per-category shift.
- WASM integration: re-baseline the M22-era change-coupling pipeline test if it asserts pattern_metrics shape.

**Watch For:**
- **The single call site (`catalog.rs:102`) is the entire behavioural surface.** A reviewer could miss this and look for many code changes; there are not. The work is concentrated in re-baselining tests and updating docs. Make sure the PR description leads with "one function call swapped" so reviewers know where to look.
- **`PatternLocation` clone inside the inner loop.** v0 regex tables are disjoint so the inner loop runs at most once per hint — the clone is the cold path. Do not pre-optimize by changing `PatternLocation` to `Arc<PatternLocation>` or by deduplicating the categories array; both add complexity for a path that runs zero times today. Revisit only when a co-occurring category lands.
- **Re-baselining is risky if done sloppily.** The right shape: capture `cargo run -p sdivi-cli -- snapshot tests/fixtures/simple-typescript` output, compare to the existing committed expectation, manually verify the diff matches the M33 expected reshape (logging gains, data_access shrinks, etc.), then commit. Do not auto-replace the expected files — that hides regressions. A reviewer should see the diff in the PR and be able to mentally validate it against the migration story.
- **`prop_test_pipeline_deterministic` is not the regression check for M33.** It only proves "same input → same output." It will pass even if the M33 output is wrong (incorrectly classified, dropping hints that should be kept, etc.). The real regression check is the re-baselined fixture tests — verify *every* re-baseline matches a story you can articulate in one sentence.
- **Adopters with `[thresholds.overrides]` already have the escape hatch.** The CHANGELOG and MIGRATION_NOTES paragraphs should explicitly point at this — adopters who can't accept a one-time recalibration set per-category overrides with an `expires` date matching their migration window. The override loader is unchanged; only the underlying instance counts are.
- **`MIGRATION_NOTES.md` worked example must be exact.** Generate the pre/post `pattern_metrics` JSON from a real fixture run, not from a description. Adopters will pattern-match against this example to recognise the shape of their own recalibration; an inaccurate example is worse than no example.
- **The M30 sentinel test stays.** A future contributor reading the M33 PR may want to delete `category_for_node_kind_never_returns_logging` because "logging is now native." That is wrong — the sentinel is about a property of the *older* function (`category_for_node_kind`, which M33 does not touch). The new positive sentinels (`classify_hint_returns_logging_for_console_log` etc.) capture the M33 invariant; the M30 sentinel captures the older-API invariant. Both stay. Inline-comment the relationship in `queries/mod.rs` so the next contributor sees it.
- **`docs/pattern-categories.md` per-language table updates are easy to under-do.** Every per-language sub-section needs the `data_access` row narrowed, the `logging` row promoted from "consumer responsibility" to actual-regex-shape, the `async_patterns` row in TS/JS extended for Promise chains, and the Rust `resource_management` row noting the logging-macro exclusion. Five tables, one row each, easy to miss one. The `markdown_table_matches_list_categories_output` test does NOT cover per-language tables — it only checks the canonical category-list table. Per-language accuracy is human-review-only. Audit deliberately in the PR.
- **No `sdivi-cli` change.** The CLI continues to call `Pipeline::snapshot`; the pipeline now calls `classify_hint`. No flag, no help text, no exit-code change. Confirm `cargo test -p sdivi-cli` is green without edits.
- **WASM consumers that bring their own classifier are unaffected.** Meridian's pre-M32 hand-rolled filter continues to work; Meridian's potential post-M32 migration to `classify_hint` is its own decision. M33 changes only what happens inside `Pipeline::snapshot` — it does not change `compute_pattern_metrics` semantics, `compute_delta` semantics, or any WASM-export shape. Confirm with the consumer-app team that the upgrade path is understood before tagging the M33 release.
- **Threshold-comparison epsilon (M20) interacts with this.** M20 added an epsilon to per-category threshold comparisons for cross-architecture stability. M33's instance-count shifts are far larger than the M20 epsilon, so adopters' threshold gates will trip even with the epsilon in effect. The epsilon protects against floating-point platform drift, not deliberate metric reshapes. Note this in MIGRATION_NOTES so adopters do not assume the epsilon will absorb the M33 change.

**Seeds Forward:**
- **Per-fixture regression baselines.** Now that fixture snapshots have observable per-category instance counts that adopters care about, the `tests/fixtures/simple-*` family becomes a de-facto regression baseline. A small `tests/fixtures/regression/` sub-tree containing committed snapshot expectations for each fixture (one expected JSON per fixture, regenerated per behavioural-change milestone) would catch unintended drift between milestones. Defer until a regression actually lands; M33 produces the first interesting baseline.
- **`pattern_metrics.coverage_ratio` field.** With `classify_hint` returning empty `Vec` for unrecognised callees, a non-trivial fraction of hints are now silently dropped (a TS codebase has many `Math.max`-shape calls that match no category). Surfacing "what fraction of hints were classified" as a `pattern_metrics` field would help adopters understand the shape of their classifications. Out of scope here; useful UX for a later milestone.
- **Custom regex injection via `Config`.** An adopter on a codebase with a non-standard logger (e.g. `myorg.observability.emit(…)` instead of `console.log(…)`) cannot currently classify their logging into `logging`. A `[patterns.callee_overrides.logging]` config block adding extra regexes per language would close this gap. Adds runtime config to a path that is currently compile-time constants — meaningful design surface, defer until a real adopter asks.
- **Per-category coverage badges for `docs/pattern-categories.md`.** Now that some categories have rich callee filters and others (Rust `data_access`, Java `data_access`) have none, the per-language tables benefit from a visible "coverage status" column: `node-kind only`, `node-kind + callee regex`, `not applicable`. Trivial documentation enhancement; defer to a doc-polish pass.
- **Move the regex tables to a versioned data file.** Long-term, the regex tables are a maintenance surface that a non-Rust contributor (a TypeScript engineer fixing a logger pattern) might want to update. A `crates/sdivi-patterns/src/queries/regex_tables.toml` loaded at compile time via `include_str!` and validated against a schema would let non-Rust contributors propose changes. Adds build complexity; defer until contributor pressure.
- **`compute_pattern_metrics` becomes the M33 stability boundary.** Once adopters depend on M33's per-category distribution, future regex changes are user-visible drift. The "Regex change log" subsection in `docs/pattern-categories.md` (mentioned in M32 Seeds Forward) becomes load-bearing in M33's wake — every regex change post-M33 is a behavioural shift adopters will notice.
