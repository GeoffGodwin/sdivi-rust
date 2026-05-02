# SDI CLI Reference

The `sdi` binary is produced by the `sdi-cli` crate. It is a thin presentation
layer over `sdi-core`; every command is reachable as a library call.

## Commands

| Command                                                      | Purpose                                                         |
|--------------------------------------------------------------|-----------------------------------------------------------------|
| `sdi init`                                                   | Write default `.sdi/config.toml` and detect languages           |
| `sdi snapshot [--commit REF] [--format json\|text]`          | Capture and store a snapshot in `.sdi/snapshots/`               |
| `sdi diff <prev> <curr>`                                     | Compute delta between two stored snapshots                      |
| `sdi trend [--last N]`                                       | Trend across stored snapshots                                   |
| `sdi check`                                                  | Exit `10` if any threshold exceeded; `0` otherwise              |
| `sdi show [<id>] [--format json\|text]`                      | Inspect a snapshot                                              |
| `sdi boundaries {infer,ratify,show}`                         | Manage `.sdi/boundaries.yaml`                                   |
| `sdi catalog [--format json\|text]`                          | Pattern catalog inspection                                      |

`sdi diff` does not accept stdin. `sdi config` is not implemented — edit
`.sdi/config.toml` directly.

## Exit codes (public API — breaking change to alter)

| Code | Meaning                                  | Emitted by                       |
|------|------------------------------------------|----------------------------------|
| `0`  | Success                                  | every command's success path     |
| `1`  | I/O or unexpected runtime failure        | any command                      |
| `2`  | Config or environment error              | any command (e.g. missing `expires`) |
| `3`  | Parse, graph, or detection failure       | `snapshot`, `check`              |
| `10` | Threshold exceeded                       | **`sdi check` only**             |

Code `3` from `sdi snapshot` fires only when **all** detected languages lack
tree-sitter grammars. A single missing grammar is a stderr warning and skips
those files — not a fatal error.

## Stdout / stderr discipline

- **stdout:** snapshot JSON, summaries, table output, anything pipeable.
- **stderr:** `tracing` logs, progress bars, warnings, deprecation notices.
- `sdi show --format json | jq '.'` must work without contamination. This is
  enforced by `crates/sdi-cli/tests/stdout_stderr_split.rs`.

## Flag → config key mapping

| Flag             | Effect                                         |
|------------------|------------------------------------------------|
| `--format json`  | `output.format = "json"`                       |
| `--no-color`     | `output.color = "never"` (also `NO_COLOR=1`)   |
| `--workers N`    | Effective parallelism (no config key)          |
| `--seed N`       | `core.random_seed = N`                         |

Precedence is **CLI > env > project config > global config > built-in defaults**.

## Environment variables

- `SDI_LOG_LEVEL` — `tracing` level (`error`, `warn`, `info`, `debug`, `trace`)
- `SDI_WORKERS` — parallel parsing workers
- `SDI_CONFIG_PATH` — override config search path (absolute path)
- `SDI_SNAPSHOT_DIR` — override snapshot directory
- `NO_COLOR=1` — disable ANSI color (also `--no-color`)

## CI gate recipe

```bash
sdi snapshot
sdi check    # exits 10 on threshold breach — fail the build
```

`sdi check` is the only command that emits `10`. Treat it as the gate; treat
every other non-zero exit as a real error.

## Common runtime behaviors

- **Missing `.sdi/boundaries.yaml`:** normal operation. No warning. Intent
  divergence fields are simply absent from the snapshot.
- **Snapshot writes are atomic:** tempfile in target dir + rename. A killed
  process never leaves a half-written `.json`.
- **Retention** (`snapshots.retention`, default `100`, `0` = unlimited) is
  enforced synchronously after each successful write.
- **Reading an incompatible `snapshot_version`:** stderr warning + baseline
  treatment (no delta). Never a crash.
