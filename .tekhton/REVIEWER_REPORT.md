# Reviewer Report — M32: Callee-Text Classification API (`classify_hint`)
Review cycle: 1 of 4
Reviewed by: reviewer agent

## Verdict
APPROVED_WITH_NOTES

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- `CHANGELOG.md` states the Rust function signature as `-> Vec<String>` for `sdivi_core::classify_hint`, but the actual Rust signature is `-> Vec<&'static str>` (the WASM wrapper maps to `Vec<String>`). Minor type-signature inaccuracy in user-facing changelog entry.
- `sdivi-core/src/lib.rs`: the re-export doc comments for `classify_hint` (line 128–139) and `PatternHintInput` (line 121–126) replace the original definitions' docs, which included `# Examples` blocks. CLAUDE.md requires `# Examples` on public items in `sdivi-core` where meaningful. The original doc tests still run via `cargo test --doc -p sdivi-patterns`, but `cargo test --doc -p sdivi-core` will not exercise them. Inlining one short example per re-export doc would satisfy the requirement and keep the `sdivi-core` doc surface self-contained.

## Coverage Gaps
- Missing proptest for fall-through consistency: the milestone acceptance criteria explicitly requires "for any `PatternHint` whose `node_kind` is not `call_expression`/`call`/`macro_invocation`, `classify_hint(hint, lang)` returns the same result as `category_for_node_kind(&hint.node_kind, lang).map(|c| vec![c]).unwrap_or_default()`." The current test `classify_hint_falls_through_for_non_call_kinds` covers only two hardcoded cases; a `proptest` over the full node-kind/language space would fulfil this criteria.
- Disjoint-regex invariant test (`disjoint_regex_invariant_for_typescript_samples`) covers only TypeScript. The milestone Watch For says to verify the invariant as a property test across all languages. Missing: Rust (e.g. `tracing::info!("x")` should match only `logging`, not also `data_access` or `async_patterns`), Python (`logging.info("x")`), and Go (`fmt.Println("x")`).
- No bit-identity snapshot integration test asserting byte-equal output pre/post-M32. The pipeline is verifiably untouched and `prop_test_pipeline_deterministic` covers the property; but the milestone acceptance criteria also specifies a fixture-level byte-comparison (snapshot the workspace fixtures before and after the M32 commit and assert byte-equal JSON), which is absent from the test suite.

## ACP Verdicts
- ACP: PatternHintInput in sdivi-patterns (not re-exporting PatternHint from sdivi-parsing) — ACCEPT — Correct architectural reasoning: `sdivi-parsing` is forbidden from the WASM dep tree (Rule 21 / KDD-12). Defining `PatternHintInput` in `sdivi-patterns::hint_input` follows the established `*Input` struct family pattern (KDD-12) and keeps the WASM dep tree clean. Backward-compatible (classify_hint is a new API). No architecture doc update required beyond what CLAUDE.md already describes.

## Drift Observations
- `crates/sdivi-patterns/Cargo.toml`: `regex` is an unconditional dependency (not gated by `pipeline-records`). This is intentional (regex is needed in both the pipeline and WASM paths), but the only documentation of this decision is the `# ── pattern classification ──` comment in the workspace `Cargo.toml`. A note near the `pipeline-records` feature definition in `sdivi-patterns/Cargo.toml` explaining why `regex` is unconditional would prevent a future contributor from mistakenly gating it.
- `crates/sdivi-patterns/src/queries/resource_management.rs:20–22` and `crates/sdivi-patterns/src/queries/logging.rs:57–60` — `RUST_LOGGING_RE` and `logging::RUST_RE` contain identical regex literals. Both are `LazyLock<Regex>` and compile independently. In v0 the duplication is harmless, but if one is updated and the other is not, Rust macros will be silently mis-classified. A cross-reference comment at the `resource_management::RUST_LOGGING_RE` definition pointing to `logging::matches_callee` (which owns the canonical pattern) would surface the invariant.
- `crates/sdivi-core/src/categories.rs` — `CATEGORIES` const remains in the hand-indexed `CATALOG_ENTRIES[N].0` form noted in prior reviews (M29, M30, M31). The indices are correct at 8 entries but the index-shift hazard accumulates as new categories are added. Seeds Forward cleanup remains open.
