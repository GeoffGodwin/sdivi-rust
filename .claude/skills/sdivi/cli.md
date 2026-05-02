# SDIVI CLI Reference

The `sdivi` binary is produced by the `sdivi-cli` crate. It is a thin presentation
layer over `sdivi-core`; every command is reachable as a library call.

## Commands

| Command                                                      | Purpose                                                         |
|--------------------------------------------------------------|-----------------------------------------------------------------|
| `sdivi init`                                                   | Write default `.sdivi/config.toml` and detect languages           |
| `sdivi snapshot [--commit REF] [--format json\|text]`          | Capture and store a snapshot in `.sdivi/snapshots/`               |
| `sdivi diff <prev> <curr>`                                     | Compute delta between two stored snapshots                      |
| `sdivi trend [--last N]`                                       | Trend across stored snapshots                                   |
| `sdivi check`                                                  | Exit `10` if any threshold exceeded; `0` otherwise              |
| `sdivi show [<id>] [--format json\|text]`                      | Inspect a snapshot                                              |
| `sdivi boundaries {infer,ratify,show}`                         | Manage `.sdivi/boundaries.yaml`                                   |
| `sdivi catalog [--format json\|text]`                          | Pattern catalog inspection                                      |

`sdivi diff` does not accept stdin. `sdivi config` is not implemented — edit
`.sdivi/config.toml` directly.

## Exit codes (public API — breaking change to alter)

| Code | Meaning                                  | Emitted by                       |
|------|------------------------------------------|----------------------------------|
| `0`  | Success                                  | every command's success path     |
| `1`  | I/O or unexpected runtime failure        | any command                      |
| `2`  | Config or environment error              | any command (e.g. missing `expires`) |
| `3`  | Parse, graph, or detection failure       | `snapshot`, `check`              |
| `10` | Threshold exceeded                       | **`sdivi check` only**             |

Code `3` from `sdivi snapshot` fires only when **all** detected languages lack
tree-sitter grammars. A single missing grammar is a stderr warning and skips
those files — not a fatal error.

## Stdout / stderr discipline

- **stdout:** snapshot JSON, summaries, table output, anything pipeable.
- **stderr:** `tracing` logs, progress bars, warnings, deprecation notices.
- `sdivi show --format json | jq '.'` must work without contamination. This is
  enforced by `crates/sdivi-cli/tests/stdout_stderr_split.rs`.

## Flag → config key mapping

| Flag             | Effect                                         |
|------------------|------------------------------------------------|
| `--format json`  | `output.format = "json"`                       |
| `--no-color`     | `output.color = "never"` (also `NO_COLOR=1`)   |
| `--workers N`    | Effective parallelism (no config key)          |
| `--seed N`       | `core.random_seed = N`                         |

Precedence is **CLI > env > project config > global config > built-in defaults**.

## Environment variables

- `SDIVI_LOG_LEVEL` — `tracing` level (`error`, `warn`, `info`, `debug`, `trace`)
- `SDIVI_WORKERS` — parallel parsing workers
- `SDIVI_CONFIG_PATH` — override config search path (absolute path)
- `SDIVI_SNAPSHOT_DIR` — override snapshot directory
- `NO_COLOR=1` — disable ANSI color (also `--no-color`)

## CI gate recipe

```bash
sdivi snapshot
sdivi check    # exits 10 on threshold breach — fail the build
```

`sdivi check` is the only command that emits `10`. Treat it as the gate; treat
every other non-zero exit as a real error.

## Common runtime behaviors

- **Missing `.sdivi/boundaries.yaml`:** normal operation. No warning. Intent
  divergence fields are simply absent from the snapshot.
- **Snapshot writes are atomic:** tempfile in target dir + rename. A killed
  process never leaves a half-written `.json`.
- **Retention** (`snapshots.retention`, default `100`, `0` = unlimited) is
  enforced synchronously after each successful write.
- **Reading an incompatible `snapshot_version`:** stderr warning + baseline
  treatment (no delta). Never a crash.
