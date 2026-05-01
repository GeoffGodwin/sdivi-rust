# CLI Integration Guide

This guide covers CI integration for `sdi check`, the full exit-code reference,
and tips for interpreting threshold output.

## GitHub Actions Snippet

```yaml
name: Structural health gate

on:
  push:
    branches: [main]
  pull_request:

jobs:
  sdi-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0   # full history for coupling analysis

      - name: Install sdi
        run: cargo install sdi-cli

      - name: Capture snapshot
        run: sdi snapshot --commit "$GITHUB_SHA"

      - name: Run threshold gate
        run: sdi check
        # Exits 0 on success, 10 if any threshold is exceeded.
        # The check also writes a new snapshot for trend tracking.
```

### JSON output for downstream tooling

```yaml
      - name: Check (JSON output)
        id: sdi
        run: sdi check --format json | tee sdi-check.json
        continue-on-error: true

      - name: Upload check result
        uses: actions/upload-artifact@v4
        with:
          name: sdi-check
          path: sdi-check.json
```

### Check without writing a snapshot

```yaml
      - name: Dry-run check
        run: sdi check --no-write
```

## Exit Code Reference

| Code | Meaning |
|---|---|
| `0` | Success — all commands except `sdi check` always exit 0 on success |
| `1` | Runtime error — I/O failure, unreadable snapshot, or unexpected error |
| `2` | Configuration error — malformed TOML, missing required field (`expires`) |
| `3` | Analysis error — all detected languages lack tree-sitter grammars |
| `10` | Threshold exceeded — **`sdi check` only**; at least one dimension exceeds its limit |

Exit code `10` is exclusive to `sdi check`. Adding or repurposing any exit code
is a breaking change requiring a major version bump.

## Threshold Configuration

Thresholds are declared in `.sdi/config.toml`:

```toml
[thresholds]
pattern_entropy_rate    = 2.0   # bits per snapshot
convention_drift_rate   = 3.0
coupling_delta_rate     = 0.15
boundary_violation_rate = 2.0
```

### Per-category overrides (migration windows)

```toml
[thresholds.overrides.error_handling]
pattern_entropy_rate = 5.0
expires = "2026-09-30"
reason  = "Migrating to ? operator from match Err(_) chains"
```

`expires` is **mandatory**. Missing it is a configuration error (exit 2). After
expiry the override is silently ignored and defaults resume — no manual reset.

## Command Reference

| Command | Description |
|---|---|
| `sdi init` | Initialise `.sdi/` and write a default `config.toml` |
| `sdi snapshot` | Capture a structural snapshot of the current repo state |
| `sdi check` | Snapshot + threshold gate; exit 10 if any dimension is exceeded |
| `sdi show [<id>]` | Inspect a stored snapshot (latest by default) |
| `sdi diff <prev> <curr>` | Compare two snapshot files |
| `sdi trend [--last N]` | Show trend statistics across stored snapshots |
| `sdi catalog` | Display the pattern catalog for the current repo state |
| `sdi boundaries infer` | Propose community-based module boundaries |
| `sdi boundaries ratify` | Write inferred boundaries to `.sdi/boundaries.yaml` |
| `sdi boundaries show` | Display currently declared boundaries |

### Flags available on most commands

| Flag | Description |
|---|---|
| `--repo <path>` | Repository root (default: `.`) |
| `--format json\|text` | Output format (default: `text`) |
| `--no-color` | Disable ANSI color output (also `NO_COLOR=1`) |

### `sdi snapshot` flags

| Flag | Description |
|---|---|
| `--commit <sha>` | Git commit SHA to record in the snapshot |

### `sdi check` flags

| Flag | Description |
|---|---|
| `--no-write` | Compute threshold check without writing a snapshot |

### `sdi trend` flags

| Flag | Description |
|---|---|
| `--last N` | Include only the N most-recent snapshots |

## Environment Variables

| Variable | Effect |
|---|---|
| `SDI_LOG_LEVEL=debug` | Enable structured tracing output to stderr |
| `SDI_WORKERS=N` | Parallel parsing worker count |
| `SDI_SNAPSHOT_DIR=<path>` | Override snapshot directory |
| `SDI_CONFIG_PATH=<path>` | Override config file path |
| `NO_COLOR=1` | Disable ANSI output (same as `--no-color`) |

## Interpreting Threshold Output

`sdi check --format json` returns a JSON object:

```json
{
  "exit_code": 10,
  "exceeded": [
    {
      "dimension": "pattern_entropy",
      "actual": 3.2,
      "limit": 2.0
    }
  ]
}
```

`exceeded` is empty when no threshold is breached (`exit_code: 0`).

All thresholds use **rate** semantics (delta per snapshot interval). A value of
`0` means two consecutive snapshots are identical on that dimension.

## Snapshot Retention

By default sdi retains the last 100 snapshots. Override in config:

```toml
[snapshots]
retention = 50   # 0 = unlimited
```

Retention is enforced synchronously after each successful snapshot write. A
failed write never removes an existing snapshot.

## Change-coupling and weighted community detection

Set `boundaries.weighted_edges = true` in `.sdi/config.toml` to have the
Leiden community detection step use change-coupling frequencies as edge
weights. Edges between files that frequently co-change are weighted by
`1.0 + frequency`, pulling highly-coupled files into the same community.

```toml
[boundaries]
weighted_edges = true

[change_coupling]
min_frequency = 0.6
history_depth = 500
```

Note: files renamed in git register as separate delete/add events, which
inflates the change-coupling signal for rename-heavy histories. This is by
design for v0 (see `docs/migrating-from-sdi-py.md`).
