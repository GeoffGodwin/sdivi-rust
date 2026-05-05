# sdivi-rust Contributor Invariants

Read this before editing crates inside the sdivi-rust workspace. These rules are
non-negotiable: they encode the architectural contract and most are
CI-enforced. Breaking one is a blocker, not a discussion.

## The 20 non-negotiable rules (verbatim from `CLAUDE.md`)

1. **`unsafe` is forbidden** in `sdivi-core` and language adapter crates.
   Bindings crates (`sdivi-py`, `sdivi-node`) may use `unsafe` only as required by
   the binding macro. Any other `unsafe` lives in a dedicated crate behind a
   feature flag with a per-block `// SAFETY:` justification.
2. **No network calls anywhere in the analysis pipeline.** No telemetry, no
   update checks, no remote lookups. Snapshots must be producible airgapped.
3. **No ML/LLM calls in the pipeline.** Determinism is the contract.
4. **Tree-sitter CSTs are dropped before `parse_file` returns.** No type
   containing a `tree_sitter::Tree` may escape that function.
5. **`BTreeMap` is the default ordered map.** `HashMap` only when iteration
   order does not influence output.
6. **All RNG is `StdRng` seeded explicitly from `Config::random_seed`.** Default
   `42`. No `thread_rng`, no `SystemTime`-based seeding, no implicit RNG.
7. **Pattern fingerprints use `blake3` with a fixed key constant** defined once
   in `sdivi-patterns::fingerprint`. Never changes within a `snapshot_version`.
8. **stdout vs stderr is strict.** Snapshot JSON, summaries, table output ⇒
   stdout. Logs, progress, warnings ⇒ stderr.
9. **Exit codes `0/1/2/3/10` are public API.** `10` is exclusively `sdivi check`.
   Adding or repurposing an exit code is a breaking change.
10. **`.sdivi/config.toml` and `.sdivi/boundaries.yaml` are read-compatible with
    the Python POC.** New keys are additive. Existing semantics may not change.
    Removed keys are reserved forever.
11. **`snapshot_version` is `"1.0"` for all sdivi-rust output.** sdivi-rust does
    not read the Python POC's snapshots. Incompatible versions ⇒ stderr warning +
    baseline treatment, never a crash.
12. **Per-category threshold overrides require `expires`.** Missing `expires` ⇒
    config error, exit `2`. After expiry the override is silently ignored.
13. **Snapshot writes are atomic** (tempfile in target dir + rename). A killed
    process must never leave a half-written `.json`.
14. **First-snapshot deltas are `null`, not `0`.** `null` = no comparison
    possible. `0` = compared and unchanged.
15. **Missing tree-sitter grammars are warnings** unless **all** detected
    languages lack grammars (then exit `3`). A single missing grammar logs to
    stderr and skips files.
16. **Missing `BoundarySpec` is normal operation.** No warning. Intent
    divergence is simply absent from the snapshot.
17. **`sdivi-cli` cannot add code paths unreachable through `sdivi-core`.** Every
    CLI feature is a library feature first (KDD-3 / KD12).
18. **Public API stability begins at `0.1.0`.** Adding `pub` is deliberate;
    removing or renaming `pub` is a breaking change.
19. **`#![deny(missing_docs)]` is enabled on `sdivi-core`.** Every public item
    has at least one rustdoc comment with `# Examples` where meaningful.
20. **`cargo clippy -- -D warnings` and `cargo fmt --check` are CI gates.** No
    `#[allow(...)]` on public items without an inline justification.

## Crate dependency rules

```
sdivi-cli  ──► every library crate (composition root; anyhow allowed here)
sdivi-core ──► re-exports public API; never imports sdivi-cli
sdivi-snapshot ──► sdivi-graph, sdivi-detection, sdivi-patterns, sdivi-config
sdivi-detection ──► sdivi-graph
sdivi-graph ──► sdivi-parsing
sdivi-patterns ──► sdivi-parsing      (NOT sdivi-graph, NOT sdivi-detection)
sdivi-parsing ──► tree-sitter, sdivi-config
sdivi-config ──► leaf
sdivi-lang-* ──► sdivi-parsing + tree-sitter grammars only
```

**No cycles.** CI inspects `cargo metadata` and fails on a cycle. The
`sdivi-patterns` rule is the one most likely to be violated by accident — patterns
must derive from `FeatureRecord` alone, never from graph or partition data.

## File and naming conventions

- Crate names: `kebab-case`, `sdivi-` prefix.
- Modules: `snake_case`. Types/traits: `PascalCase`. Fns/fields/locals:
  `snake_case`. Constants: `SCREAMING_SNAKE_CASE`.
- File ceiling: 500 lines guideline. Above that, split by sub-concern.
- `pub(crate)` for internal items; `pub` for the SemVer surface only.
- `#[non_exhaustive]` on enums that may grow (`BoundaryViolation`,
  `PatternInstance`). **`ExitCode` is closed** — its contract is fixed.

## Error handling

- Library crates: `thiserror` with named variants carrying structured fields
  (`MissingExpires { category: String }`, `InvalidValue { key, message }`).
- `sdivi-cli/src/main.rs`: `anyhow::Result` at the binary boundary **only**.
- `panic!` is reserved for "this should be impossible." Recoverable failures
  return `Result<T, E>`.
- All variants carry context (file path, key name, line number) so callers can
  surface them meaningfully.

## Testing requirements (per crate)

- Unit tests via `#[cfg(test)] mod tests` blocks; coverage targets 80%+ on
  library crates, 60%+ on `sdivi-cli`.
- Doc tests on every public function with an `# Examples` block; broken
  examples fail CI.
- Determinism property tests via `proptest`
  (`prop_test_pipeline_deterministic`, `prop_test_delta_pure`,
  `prop_test_leiden_seeded`). `proptest-regressions/` is committed.
- KD11 Leiden verification: feature-gated `verify-leiden`. Pass criteria:
  modularity within 1%, community count within ±10%. **Not bit-identity.**

## Critical CI-enforced contracts

- `tests/stdout_stderr_split.rs` (in `sdivi-cli`)
- `tests/exit_codes.rs` (in `sdivi-cli`)
- `tests/atomic_writes.rs` (in `sdivi-snapshot`)
- `tests/memory_invariant.rs` (in `sdivi-parsing`) — asserts the CST-drop rule
- `cargo metadata` cycle check
- `cargo doc --workspace --no-deps` zero warnings

## Things to never do

- Add a `pub` item to `sdivi-core` without a doc comment + example.
- Change `snapshot_version` without bumping major + writing
  `MIGRATION_NOTES.md`.
- Import the Python POC's snapshot JSON (clean break, KDD-1).
- FFI to the C++ Leiden (native port only, KDD-2 / KD11).
- Auto-classify drift as "drift" vs "evolution" (KD1 — humans declare intent).
- Add a watch mode, daemon, or interactive TUI (CLI invocation only).
- Pass a `tree_sitter::Tree` across files or hold two simultaneously.
- Use `thread_rng` or `SystemTime`-derived seeds.
- Change the `blake3` fingerprint key constant.

## When the spec and the skill disagree

`CLAUDE.md` and `.tekhton/DESIGN.md` are the source of truth. This skill is a
distillation. If you find a contradiction, trust the repo docs and update the
skill in the same PR.
