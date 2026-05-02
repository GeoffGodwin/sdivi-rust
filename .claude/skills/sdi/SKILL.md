---
name: sdi
description: |
  Use SDI (Structural Divergence Indexer) — the `sdi` CLI, the `sdi-core` Rust
  library, or the sdi-rust workspace itself. SDI is a deterministic measurement
  instrument that snapshots a codebase's structural state and reports drift over
  time.

  TRIGGER when: user runs or asks about `sdi {init,snapshot,diff,trend,check,show,boundaries,catalog}`;
  edits `.sdi/config.toml` or `.sdi/boundaries.yaml`; imports `sdi_core` or
  calls `Pipeline::{new,snapshot,delta}`; works in the sdi-rust repo on
  `crates/sdi-*`; sees a non-zero `sdi` exit code; asks "what does this
  divergence number mean."

  SKIP: unrelated Rust work; questions about sdi-py (the Python predecessor) —
  this skill only covers sdi-rust (`snapshot_version: "1.0"`).
---

# SDI — Structural Divergence Indexer

SDI measures structural drift in a codebase. It does **not** judge code quality;
it reports divergence from declared intent. It is deterministic by construction:
same repo state + same `Config` ⇒ bit-identical `Snapshot` JSON.

## Always-true facts (memorize these)

1. **`snapshot_version` is the literal string `"1.0"`** for all sdi-rust output.
   sdi-rust does not read sdi-py snapshots. An incompatible version is a stderr
   warning + baseline treatment, never a crash.
2. **Exit codes are public API: `0`, `1`, `2`, `3`, `10`.** `10` is exclusively
   `sdi check` (threshold breach). Every other command's success exits `0`.
3. **First-snapshot delta is `null`, not `0`.** `null` = no prior snapshot to
   compare. `0` = compared and unchanged. These are different and observable.
4. **stdout vs stderr is strict.** Snapshot JSON, summaries, table output ⇒
   stdout. Logs, progress bars, warnings ⇒ stderr. `sdi show --format json | jq`
   must always work.
5. **Determinism contract:** `BTreeMap` is the default ordered map. RNG is
   `StdRng` seeded from `Config::random_seed` (default `42`). Pattern
   fingerprints are `blake3` with a fixed key. No `thread_rng`, no clock-seeded
   RNG, no `HashMap` where output ordering matters.
6. **Pipeline is five sequential stages:** parsing → graph → detection →
   patterns → snapshot/delta. No backward reach; downstream consumes upstream
   output as data.
7. **No network, no LLM, no daemon.** A snapshot is producible on an airgapped
   machine. CI tests must not require network.

## Routing — load the sub-file that matches the task

| If the user is...                                            | Read                                  |
|--------------------------------------------------------------|---------------------------------------|
| running `sdi <cmd>`, asking about a flag or exit code        | `cli.md`                              |
| editing `.sdi/config.toml`, adding a threshold override, or working with `boundaries.yaml` | `config.md` |
| importing `sdi_core`, embedding the pipeline, or asking about `Pipeline`/`Snapshot`/`Config` | `embedding.md` |
| editing crates inside the sdi-rust workspace (contributor work)             | `invariants.md`        |

Load only what the current task needs. Do not preload all four.

## Quick orientation — the data flow

```
config.toml + boundaries.yaml + repo path
       ▼
Config::load_or_default
       ▼
Pipeline::new(&Config)
       ▼
Stage 1: parsing       (tree-sitter; CST dropped per file)
Stage 2: graph         (petgraph dependency graph)
Stage 3: detection     (native Leiden, seeded)
Stage 4: patterns      (tree-sitter queries + blake3 fingerprints)
Stage 5: snapshot      (assemble + atomic write to .sdi/snapshots/)
       ▼
Pipeline::delta(prev, curr) ⇒ DivergenceSummary
```

## When you don't know

- Authoritative spec: `CLAUDE.md` and `.tekhton/DESIGN.md` at the repo root.
- The skill is a distillation; the repo docs are the source of truth.
- If sub-file content disagrees with `CLAUDE.md`, trust `CLAUDE.md` and flag
  the drift — the skill needs an update.
