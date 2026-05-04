---
title: CLI integration
---

# CLI Integration Guide

This guide covers CI integration for `sdivi check`, the full exit-code reference,
and tips for interpreting threshold output.

## GitHub Actions Snippet

```yaml
name: Structural health gate

on:
  push:
    branches: [main]
  pull_request:

jobs:
  sdivi-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0   # full history for coupling analysis

      - name: Install sdivi
        run: cargo install sdivi-cli

      - name: Capture snapshot
        run: sdivi snapshot --commit "$GITHUB_SHA"

      - name: Run threshold gate
        run: sdivi check
        # Exits 0 on success, 10 if any threshold is exceeded.
        # The check also writes a new snapshot for trend tracking.
```

### JSON output for downstream tooling

```yaml
      - name: Check (JSON output)
        id: sdivi
        run: sdivi check --format json | tee sdivi-check.json
        continue-on-error: true

      - name: Upload check result
        uses: actions/upload-artifact@v4
        with:
          name: sdivi-check
          path: sdivi-check.json
```

### Check without writing a snapshot

```yaml
      - name: Dry-run check
        run: sdivi check --no-write
```

## Exit Code Reference

| Code | Meaning |
|---|---|
| `0` | Success — all commands except `sdivi check` always exit 0 on success |
| `1` | Runtime error — I/O failure, unreadable snapshot, or unexpected error |
| `2` | Configuration error — malformed TOML, missing required field (`expires`) |
| `3` | Analysis error — all detected languages lack tree-sitter grammars |
| `10` | Threshold exceeded — **`sdivi check` only**; at least one dimension exceeds its limit |

Exit code `10` is exclusive to `sdivi check`. Adding or repurposing any exit code
is a breaking change requiring a major version bump.

## Threshold Configuration

Thresholds are declared in `.sdivi/config.toml`:

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
| `sdivi init` | Initialise `.sdivi/` and write a default `config.toml` |
| `sdivi snapshot` | Capture a structural snapshot of the current repo state |
| `sdivi check` | Snapshot + threshold gate; exit 10 if any dimension is exceeded |
| `sdivi show [<id>]` | Inspect a stored snapshot (latest by default) |
| `sdivi diff <prev> <curr>` | Compare two snapshot files |
| `sdivi trend [--last N]` | Show trend statistics across stored snapshots |
| `sdivi catalog` | Display the pattern catalog for the current repo state |
| `sdivi boundaries infer` | Propose community-based module boundaries |
| `sdivi boundaries ratify` | Write inferred boundaries to `.sdivi/boundaries.yaml` |
| `sdivi boundaries show` | Display currently declared boundaries |

### Flags available on most commands

| Flag | Description |
|---|---|
| `--repo <path>` | Repository root (default: `.`) |
| `--format json\|text` | Output format (default: `text`) |
| `--no-color` | Disable ANSI color output (also `NO_COLOR=1`) |

### `sdivi snapshot` flags

| Flag | Description |
|---|---|
| `--commit <ref>` | Analyze the tree at `<ref>` (branch, tag, or SHA); labels the snapshot with the resolved SHA and the commit's commit-date |

### `sdivi check` flags

| Flag | Description |
|---|---|
| `--no-write` | Compute threshold check without writing a snapshot |

### `sdivi trend` flags

| Flag | Description |
|---|---|
| `--last N` | Include only the N most-recent snapshots |

## Environment Variables

| Variable | Effect |
|---|---|
| `SDIVI_LOG_LEVEL=debug` | Enable structured tracing output to stderr |
| `SDIVI_WORKERS=N` | Parallel parsing worker count |
| `SDIVI_SNAPSHOT_DIR=<path>` | Override snapshot directory |
| `SDIVI_CONFIG_PATH=<path>` | Override config file path |
| `NO_COLOR=1` | Disable ANSI output (same as `--no-color`) |

## Interpreting Threshold Output

`sdivi check --format json` returns a JSON object:

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

By default sdivi retains the last 100 snapshots. Override in config:

```toml
[snapshots]
retention = 50   # 0 = unlimited
```

Retention is enforced synchronously after each successful snapshot write. A
failed write never removes an existing snapshot.

## Analyzing a historical commit

`sdivi snapshot --commit REF` analyzes the **actual source tree** at `REF`, not
the working directory.

```bash
sdivi snapshot --commit v1.2.0          # analyze a release tag
sdivi snapshot --commit HEAD~10         # analyze 10 commits ago
sdivi snapshot --commit abc123def456    # analyze a specific SHA
```

What happens internally:

1. `REF` is resolved to a full 40-char SHA via `git rev-parse --verify`.
2. The tree at that SHA is extracted to a temporary directory via
   `git archive --format=tar <sha> | tar -xC <tmpdir>`.
3. All five pipeline stages run against the extracted tree.
4. The snapshot's `commit` field is set to the **resolved SHA** (not the ref
   name you supplied).
5. The snapshot's `timestamp` is the **commit's commit-date** (normalised to
   UTC), not the wall-clock time of the invocation. This ensures that
   lexicographic file ordering matches chronological order across mixed
   historical and current snapshots.
6. Change-coupling history is collected ending at the resolved SHA, so the
   coupling section reflects commits up to that point in history.
7. The Leiden partition cache (`.sdivi/cache/partition.json`) and the snapshot
   output directory (`.sdivi/snapshots/`) are located relative to the **original
   `repo_root`**, not the temporary extraction directory.
8. The temporary directory is removed before `sdivi snapshot` returns.

### Caveats

- **`.gitattributes export-ignore`**: `git archive` honours this directive.
  Files marked `export-ignore` are excluded from the extracted tree and will
  not appear in the snapshot graph. This differs from `git checkout` semantics.
- **Shallow clones**: if `REF` is below the shallow boundary, `git rev-parse`
  succeeds but `git archive` may fail. Use `fetch-depth: 0` in CI checkouts
  that require historical analysis.
- **Submodules**: `git archive` does not recurse into submodules. Submodule
  contents are not included in the snapshot.
- **`tar` on PATH**: `tar` must be available. On Linux and macOS this is
  always the case. Windows ships `tar.exe` in `System32` since Windows 10 1803.

### Historical backfill

To produce snapshots for a range of commits, script the CLI:

```bash
for sha in $(git rev-list v1.0.0..v2.0.0 --reverse); do
  sdivi snapshot --commit "$sha"
done
sdivi trend
```

Batch backfill (`sdivi backfill`) is not provided in v0.

## Change-coupling and weighted community detection

Set `boundaries.weighted_edges = true` in `.sdivi/config.toml` to have the
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

## Absorbing the M19 boundary violation cutover

Before M19, `sdivi check` never triggered on `boundary_violation_rate` because
`compute_boundary_violations` always returned zero. After upgrading, the first
snapshot taken against a repo with a `.sdivi/boundaries.yaml` will surface all
existing violations at once. The delta from zero to N violations may exceed
`boundary_violation_rate` and fail the CI gate unexpectedly.

To absorb the cutover without a surprise gate failure, add a short-lived
per-category override before taking the first M19 snapshot:

```toml
[thresholds.overrides.boundary_violations]
boundary_violation_rate = 9999.0
expires = "2026-06-30"
reason = "M19 cutover — re-baselining boundary violation count"
```

Remove the override (or let it expire) once the team has reviewed and accepted
the surfaced violations as the new baseline.

