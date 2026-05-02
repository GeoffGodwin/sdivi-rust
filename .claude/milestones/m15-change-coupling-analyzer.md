
#### Milestone 15: Change-Coupling Analyzer
<!-- milestone-meta
id: "15"
status: "done"
-->
<!-- PM-tweaked: 2026-05-01 -->


**Scope:** Implement the change-coupling computation that turns the existing `[change_coupling]` config block into a real analyzer. Walk the last `history_depth` commits via shell-out to `git log`, compute file-pair co-change frequencies, surface them in the snapshot, and feed them into Leiden as edge weights when `boundaries.weighted_edges = true`. Two-layer split: I/O lives in `sdivi-pipeline::change_coupling`; the math lives in `sdivi-core::compute_change_coupling` (pure, WASM-callable, foreign-extractor-friendly).

**Why this milestone exists:** `ChangeCouplingConfig`, `boundaries.weighted_edges`, and the `[change_coupling]` block in `init`'s default config template are advertised features that no code currently reads. CLAUDE.md's `Config Architecture` table documents `min_frequency = 0.6` and `history_depth = 500` as if they did something. They don't. Shipping v0 with a documented-but-inert analyzer is worse than shipping v0 without it: it actively misleads embedders and adopters. This milestone makes the existing config surface load-bearing and gives consumer-app (and any other consumer that may want temporal coupling later) a stable, pure-compute entry point that does not depend on the consumer having its own libgit2 or git binary in scope.

**Deliverables:**

- **Pure-compute entry point in `sdivi-core`:**
  - `compute_change_coupling(events: &[CoChangeEventInput], cfg: &ChangeCouplingConfigInput) -> ChangeCouplingResult` — given an ordered list of commit-events (each carrying the set of paths touched by that commit), produces the set of file-pair frequencies that meet `min_frequency`. Pure, referentially transparent, no I/O, no clock.
  - Algorithm: for each commit, enumerate every unordered pair of paths in the commit's `files`; increment a per-pair counter; divide each pair's count by `commits_analyzed`; emit pairs whose frequency `>= min_frequency` **and whose `cochange_count >= 2`** (a single co-occurrence is not change coupling — it may be a coincidence of the same initial commit creating both files). [PM: Added the `cochange_count >= 2` guard to match the stated unit-test behaviour "single commit → no pairs (a pair requires two distinct commits)." Without this guard, a single commit touching `{a, b}` produces frequency `1.0`, which passes any `min_frequency` filter. The guard makes the test case and the algorithm consistent.]
  - Pairs are ordered by `(source, target)` lexicographically with `source < target`; output is a `Vec<CoChangePair>` sorted by `(source, target)` for byte-identical determinism.
  - `commits_analyzed = min(events.len(), cfg.history_depth as usize)`. The function operates on the trailing window: events are assumed oldest-first; the last `history_depth` are the analyzed window.
- **New input/output types in `sdivi-core::input`:**
  - `CoChangeEventInput { commit_sha: String, commit_date: String, files: Vec<String> }` — `commit_date` is ISO-8601 (`YYYY-MM-DDTHH:MM:SSZ`); `files` are canonical NodeIds (validated via the existing `validate_node_id`).
  - `ChangeCouplingConfigInput { min_frequency: f64, history_depth: u32 }` — mirrors `sdivi-config::ChangeCouplingConfig` with serde-and-tsify derives. `min_frequency` validated `[0.0, 1.0]` at the entry point; out-of-range returns `AnalysisError::InvalidConfig`.
  - `ChangeCouplingResult { pairs: Vec<CoChangePair>, commits_analyzed: u32, distinct_files_touched: u32 }`. [PM: `distinct_files_touched` is the count of unique path strings that appear in at least one of the analyzed commit events (i.e., `HashSet`-collect `files` across the trailing `commits_analyzed` events, then `.len()`). This was not defined in the original.]
  - `CoChangePair { source: String, target: String, frequency: f64, cochange_count: u32 }` — `source < target` lexicographically; `frequency = cochange_count / commits_analyzed`.
- **I/O entry point in `sdivi-pipeline`:**
  - `crates/sdivi-pipeline/src/change_coupling.rs` — `collect_cochange_events(repo_root: &Path, history_depth: u32, ending_at: Option<&str>) -> Result<Vec<CoChangeEventInput>, ChangeCouplingError>`. Shells out to `git log --no-pager -z --name-only --format=%x00COMMIT%x00%H%x00%cI%x00 -n <history_depth> [<ending_at>]`. Parses NUL-separated output into events. Translates paths to canonical NodeId form (forward slashes, repo-relative, no leading `./`).
  - `ending_at = None` defaults to `HEAD`. `M16` will pass a resolved SHA when `--commit REF` is in play.
  - Returns `Ok(vec![])` if the repo has no `.git/` (Rule 16-style: missing input is normal), with a stderr `tracing::info!` line — never an error.
  - Returns a structured `ChangeCouplingError` for malformed `git log` output or non-zero exit (other than the "no git" path).
- **Pipeline integration:**
  - `Pipeline::snapshot` calls `collect_cochange_events` after Stage 2 (graph), passes the events plus `ChangeCouplingConfigInput` to `compute_change_coupling`, stores the result in the snapshot.
  - When `config.boundaries.weighted_edges = true`, the Leiden detection stage (Stage 3) receives the change-coupling pairs as edge weights: for each `(source, target)` pair in `ChangeCouplingResult.pairs` whose endpoints both exist in the dependency graph, the existing import edge's weight is **multiplied** by `(1.0 + frequency)`. (Multiplicative — never additive — so a pair with frequency 1.0 doubles the edge weight; a pair with frequency 0.6 raises it by 60%.) Pairs whose endpoints are not connected by an import edge are **not** added as new edges (KDD-5 keeps the graph import-only; change-coupling is a weight modulation, not a new topology).
  - `compute_change_coupling` is called regardless of `weighted_edges`; the result lands in the snapshot either way.
- **Snapshot shape extension (additive, schema stays `"1.0"`):**
  - `Snapshot.change_coupling: Option<ChangeCouplingResult>` — `None` when the repo has no git history (or `history_depth = 0`); `Some(_)` otherwise. `#[serde(default)]` so M14-era snapshots deserialize as `None`.
  - No new `DivergenceSummary` field for v0 — change-coupling is a descriptive snapshot dimension, not a threshold-gated one. (The existing `coupling_delta_rate` continues to gate graph density, not change-coupling.)
- **Leiden input shape:**
  - `LeidenConfigInput` gains `edge_weights: Option<BTreeMap<(String, String), f64>>` — `None` (default) means "all weights = 1.0" (existing behavior); `Some(map)` means "use the supplied weights, defaulting to 1.0 for unlisted edges". `BTreeMap` keys are `(source, target)` with `source < target`. `#[serde(default)]`.
  - `detect_boundaries` (existing in `sdivi-core`) feeds the optional weights into the petgraph edge weights at graph-construction time. The existing Leiden algorithm already operates on weighted edges (graph density, modularity calculations); this surfaces the existing capability through the input shape.
  - Pipeline path: when `weighted_edges = true`, `Pipeline::snapshot` constructs the `edge_weights` map from `ChangeCouplingResult.pairs` and passes it through `LeidenConfigInput`.
- **WASM exports:**
  - `bindings/sdivi-wasm/src/lib.rs` — export `compute_change_coupling`, `CoChangeEventInput`, `ChangeCouplingConfigInput`, `ChangeCouplingResult`, `CoChangePair`. tsify regenerates `.d.ts`.
  - This is the consumer-app-facing surface: a consumer with its own commit history walker (e.g., reading from the IDE's git index) supplies `Vec<CoChangeEventInput>` directly, no shell-out needed.
- **CLI surface:**
  - `sdivi show` text output gains a `change coupling: <N> pairs (top 5: …)` line when `change_coupling` is `Some` and non-empty.
  - `sdivi show --format json` already serializes the full snapshot; the new field appears automatically.
  - No new CLI flags. `boundaries.weighted_edges` is the existing knob.
- **Documentation:**
  - `docs/library-embedding.md` — new section "Computing change-coupling from a foreign extractor" showing a consumer-app-style consumer that supplies its own `CoChangeEventInput` slice (e.g., from VSCode's git extension) and calls `compute_change_coupling` directly via WASM.
  - `docs/cli-integration.md` — short note on `boundaries.weighted_edges` and how it interacts with Leiden communities.
  - `docs/determinism.md` — note that `git log` output ordering is deterministic for a fixed `HEAD`, and the canonical NodeId translation ensures cross-platform path equivalence (forward slashes, no `./`).
- **CHANGELOG.md** entry: "Change-coupling analyzer wired up. New snapshot field `change_coupling`. New `boundaries.weighted_edges = true` mode multiplies import-edge weights by `(1.0 + frequency)`. New pure-compute entry point `sdivi_core::compute_change_coupling` exported through WASM. Schema stays `1.0`."

**Migration Impact:** [PM: Added this section. The snapshot schema stays `"1.0"` and the new field is `#[serde(default)]`, so M14-era snapshots deserialize with `change_coupling = None` without any error. No config migration is required. The `boundaries.weighted_edges` key was already present in the schema (previously inert); after this milestone it becomes load-bearing, which changes its observable behaviour but not its type or default value (`false`). Users who had `weighted_edges = true` in their config before M15 will now see Leiden receive modified edge weights on their next snapshot run; they should be aware their community partition may shift.]

**Files to create or modify:**

- **New:** `crates/sdivi-pipeline/src/change_coupling.rs` (`collect_cochange_events`, `ChangeCouplingError`).
- **New:** `crates/sdivi-core/src/compute/change_coupling.rs` (`compute_change_coupling`).
- **New (or extend `crates/sdivi-core/src/input/types.rs`):** add `CoChangeEventInput`, `ChangeCouplingConfigInput`, `ChangeCouplingResult`, `CoChangePair`.
- **Modify:** `crates/sdivi-core/src/input/types.rs` — extend `LeidenConfigInput` with `edge_weights: Option<BTreeMap<(String, String), f64>>`.
- **Modify:** `crates/sdivi-core/src/compute/boundaries.rs` (or wherever `detect_boundaries` lives) — feed `edge_weights` into the petgraph edge weights.
- **Modify:** `crates/sdivi-snapshot/src/snapshot.rs` — add `change_coupling: Option<ChangeCouplingResult>` to `Snapshot`. Update `assemble_snapshot` to take the new field.
- **Modify:** `crates/sdivi-pipeline/src/pipeline.rs` — call `collect_cochange_events`, call `compute_change_coupling`, populate `Snapshot.change_coupling`, wire edge weights into Leiden when `weighted_edges = true`.
- **Modify:** `crates/sdivi-pipeline/src/lib.rs` — re-export `change_coupling` module.
- **Modify:** `crates/sdivi-cli/src/output/text.rs`, `crates/sdivi-cli/src/output/json.rs` — surface change-coupling in `sdivi show`.
- **Modify:** `bindings/sdivi-wasm/src/lib.rs` — export the new compute function and types.
- **Modify:** `bindings/sdivi-wasm/Cargo.toml` — no new deps; tsify handles the `.d.ts`.
- **New tests:**
  - `crates/sdivi-core/tests/compute_change_coupling.rs` — pure unit tests, including determinism, sort order, and `min_frequency` filtering.
  - `crates/sdivi-pipeline/tests/change_coupling_git.rs` — integration test against a tempdir git repo with a known commit history (set up in-test via `std::process::Command::new("git")` calls).
  - `tests/change_coupling_lifecycle.rs` (workspace-level) — end-to-end: snapshot a fixture, assert `Snapshot.change_coupling` is `Some` with expected pairs.
  - `crates/sdivi-detection/tests/leiden_weighted_edges.rs` — assert that `weighted_edges = true` with a non-trivial weight map produces a different community partition than `false` on the same graph (or the same partition with higher modularity — either is acceptance).
- **Test fixture:** `tests/fixtures/change-coupling-history/` — a small git fixture (built by a setup script under `target/test-fixtures` like the existing `evolving/` fixture) with 10 commits exercising known co-change pairs. The setup script lives in `tools/build-test-fixtures.sh`; the integration test invokes it before running. The fixture is **not** checked in as a git repo (no nested `.git/`); the script reconstructs it from a manifest.
- **Modify:** `tools/validate-against-bifl-tracker.sh` (M11 harness) — extend to assert `Snapshot.change_coupling` is `Some` for the bifl-tracker baselines (it has plenty of history). Tolerance: existing per-dimension tolerances unchanged; the new field is asserted to be non-empty but its content is not gated against sdi-py baselines (sdi-py's change-coupling implementation may not have shipped, and KDD-1 says clean break on snapshots anyway).
- **Modify:** `CHANGELOG.md`, `docs/library-embedding.md`, `docs/cli-integration.md`, `docs/determinism.md`.

**Acceptance criteria:**

- `Pipeline::snapshot` against the test fixture with 10 commits produces a snapshot whose `change_coupling.pairs` contains exactly the expected pairs (computed by hand in the test) at exactly the expected frequencies.
- `compute_change_coupling` is pure: same `events` + same `cfg` produces a byte-identical `ChangeCouplingResult` (proptest covers this).
- A repo with no `.git/` directory produces a snapshot with `change_coupling = None` and no warning to stderr beyond a `tracing::info!` line.
- A repo with `history_depth = 0` produces `change_coupling = Some(ChangeCouplingResult { pairs: vec![], commits_analyzed: 0, distinct_files_touched: 0 })` (zero-history is *not* the same as "no git" — the user explicitly disabled the analyzer).
- `cargo build -p sdivi-core --target wasm32-unknown-unknown --no-default-features` succeeds; `cargo tree -p sdivi-core --target wasm32-unknown-unknown --no-default-features` shows zero entries for shell-out crates, `git2`, `gix`, `walkdir`, `ignore`, `rayon`, `tempfile`.
- `bindings/sdivi-wasm` test imports `compute_change_coupling` from the bundled `.wasm` and produces the same result as the native call for the same `CoChangeEventInput` slice.
- `boundaries.weighted_edges = true` against the change-coupling fixture produces a Leiden partition whose modularity is **not less than** the unweighted-baseline modularity (equality is acceptable; degradation is a regression).
- The existing M11 bifl-tracker harness still passes within the documented tolerances on per-dimension metrics.
- `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` pass.
- `cargo test --workspace` passes including doctests.
- `snapshot_version` remains `"1.0"`.
- [PM: `sdivi show` (text format) against a snapshot with a non-empty `change_coupling` field prints a line matching the pattern `change coupling: <N> pairs (top 5: …)` to stdout and nothing to stderr for that field. Verified by `assert_cmd` in the CLI exit-code test suite.]

**Tests:**

- Unit (pure-compute): empty events → empty pairs; single commit touching `{a, b}` → no pairs (frequency = 1.0 but `cochange_count = 1 < 2`); [PM: Restated to match the clarified algorithm: a pair requires `cochange_count >= 2`, not just `frequency >= min_frequency`.] two commits each touching `{a, b}` → one pair `(a, b)` at frequency 1.0 with `cochange_count = 2`; out-of-window commits ignored when `events.len() > history_depth`.
- Unit (filter): three pairs with frequencies 0.8, 0.6, 0.4 against `min_frequency = 0.6` → exactly two pairs survive, sorted lexicographically (both also have `cochange_count >= 2` in the fixture).
- Property: `prop_test_change_coupling_deterministic` — randomly generated `Vec<CoChangeEventInput>` × random `cfg` produces stable output.
- Property: `prop_test_change_coupling_pair_ordering` — emitted pairs are always sorted with `source < target`.
- Integration (git): `crates/sdivi-pipeline/tests/change_coupling_git.rs` — tempdir repo, 5 commits with known file touches, `collect_cochange_events` returns the expected `Vec<CoChangeEventInput>`.
- Integration (path canonicalization): a fixture with a path containing a space and a Unicode character round-trips correctly through the `git log -z` parser and through `validate_node_id`.
- Integration (Leiden weighted): synthetic graph with known partition under unweighted Leiden, weighted Leiden produces a different (or higher-modularity) partition with a known weight map.
- Integration (no-git): a tempdir without `.git/` produces `change_coupling = None`.
- Integration (history-depth-0): a repo with `history_depth = 0` produces an empty `ChangeCouplingResult`.
- [PM: Unit (`distinct_files_touched`): two commits, first touching `{a, b}`, second touching `{b, c}` → `distinct_files_touched = 3` (unique paths: a, b, c). Validates the "unique path strings across analyzed events" definition.]
- WASM smoke: `compute_change_coupling` callable from a JS test (mirrors M12 patterns).
- Doctests on `compute_change_coupling`, `CoChangeEventInput`, `ChangeCouplingResult`, `CoChangePair` with concrete `# Examples` blocks.

**Watch For:**

- **Path canonicalization.** `git log -z --name-only` emits paths in repo-relative form with forward slashes on every platform (git's internal representation). Translation to canonical NodeId is essentially a no-op except for stripping any leading `./` and asserting via `validate_node_id`. Test fixture must include a path that would *not* round-trip cleanly under naive `Path::to_str()` (e.g., a path with a space) so the parser is exercised.
- **`-z` framing.** `git log -z --name-only --format=%x00COMMIT%x00%H%x00%cI%x00 -n N` produces NUL-separated output where commits are delimited by a sentinel `COMMIT` token (with NULs around it). A more careful framing using `--format` with multiple unique sentinels may be needed; the milestone author should validate against a fixture with merge commits, empty commits, and commits whose subject contains arbitrary bytes. Do not parse with regex; parse byte-by-byte.
- **Merge commits.** A merge commit's `--name-only` output lists files that conflict-resolved during the merge — typically zero files for a non-conflict merge. This is fine for v0; document the behavior. Do not pass `--first-parent` (it would skip topic-branch commits' co-change signal).
- **Renames.** `git log --follow` is single-file only; for a multi-file co-change view, follow-renames is impractical at the v0 scope. Renames register as one delete + one add in two separate commits, which spuriously inflates the file count. Document in `docs/migrating-from-sdi-py.md` and `docs/cli-integration.md`. Do not silently filter rename pairs — that hides real signal.
- **Submodules.** `git log` does not recurse. Submodule changes do not contribute to co-change. Document.
- **Determinism across machines.** `git log` output ordering is deterministic given a fixed HEAD/ref. The pure-compute side preserves order; the I/O side does not reorder. A cross-platform regression test (run on Linux, macOS, Windows in CI) must produce byte-identical `ChangeCouplingResult` for the same fixture.
- **Windows shell-out.** `std::process::Command::new("git")` works on Windows when `git` is on `PATH` (Git for Windows installs it there by default). The milestone must include a Windows CI matrix entry that verifies the integration test runs on `windows-latest`. If `git` is not installed in the CI runner image, `actions/setup-git` or equivalent is required.
- **Empty-window edge case.** `history_depth` larger than the actual commit count: `commits_analyzed = events.len()`, not `history_depth`. The frequency denominator is the actual analyzed window, never the configured ceiling. Test explicitly.
- **`min_frequency = 0.0`.** Yields every pair that co-changed in at least 2 commits (`cochange_count >= 2`). Allowed; document the implication. [PM: Consistent with the `cochange_count >= 2` guard: `min_frequency = 0.0` does not mean "any single co-occurrence counts."]
- **`min_frequency = 1.0`.** Yields only pairs that co-changed in every analyzed commit. Allowed; document.
- **Weighted Leiden + KDD-2 verification.** The `verify-leiden` harness (M11) compares against `leidenalg`'s unweighted output by default. Weighted-edge verification against `leidenalg`'s weighted mode is *out of scope* for v0 — KDD-2's tolerance is partition quality, not bit-identity, and weighted-mode parity verification is best left for v0.x. Document this in `docs/determinism.md`.
- **CLAUDE.md update.** The "Config Architecture" table currently lists `min_frequency = 0.6` and `history_depth = 500` without indicating the analyzer is wired. The user should update CLAUDE.md after this milestone closes to reflect that `[change_coupling]` is no longer a forward-reference. Do not modify CLAUDE.md from this milestone.
- **Doc-comment placement.** When inserting `change_coupling: Option<ChangeCouplingResult>` into `Snapshot`, ensure a blank line separates the new field's `///` block from the next field's `///` block.

**Seeds Forward:**

- `compute_change_coupling` taking `CoChangeEventInput` (string SHAs, string dates, string paths) is the contract consumer-app relies on. Adding fields to `CoChangeEventInput` is additive. Removing fields is breaking and post-1.0.
- `boundaries.weighted_edges = true` is the lever for "use both structural and historical coupling for community detection." A future v0.x can introduce per-pair weight overrides for users who want to suppress noisy co-change signals.
- The `LeidenConfigInput.edge_weights` shape is the precedent for future graph-edge-weighting features (test coupling, runtime trace coupling, etc.) — they all reduce to producing a `BTreeMap<(String, String), f64>` and feeding it through the same input shape.
- M16 (`--commit REF`) reuses the same `git` shell-out pattern (`std::process::Command`, NUL-framing, repo-relative paths) and `change_coupling`'s `ending_at` parameter for "history ending at REF."
- Post-v0 metric: a `change_coupling_delta_rate` threshold that gates "unusual increase in co-change pair density" can be added with no schema bump — the per-snapshot pair count is already a number.
