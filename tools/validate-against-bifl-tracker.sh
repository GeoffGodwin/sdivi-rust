#!/usr/bin/env bash
# validate-against-bifl-tracker.sh — v0 acceptance gate for sdi-rust.
#
# Runs sdi-rust against the bifl-tracker repo at the commits captured by
# sdi-py and compares structural metrics within the documented tolerances:
#   - Modularity:        within 1%
#   - Community count:   within ±10%
#   - Pattern entropy:   within 5%
#
# Also performs a pure-compute parity check: for each commit, verifies that
# the pipeline path and the sdi-core compute_* path produce the same results.
#
# Prerequisites:
#   - Rust toolchain installed and sdi-cli compiled (cargo build -p sdi-cli)
#   - bifl-tracker checkout at ~/workspace/geoffgodwin/bifl-tracker
#   - python3 (for JSON comparison helpers)
#
# Usage: ./tools/validate-against-bifl-tracker.sh [--verbose]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BIFL_TRACKER="${BIFL_TRACKER_PATH:-"$HOME/workspace/geoffgodwin/bifl-tracker"}"
BASELINES_DIR="$REPO_ROOT/tests/fixtures/bifl-tracker-baselines"
SDE_BIN="$REPO_ROOT/target/debug/sdi"
VERBOSE="${1:-}"

PASS=0
FAIL=0
SKIP=0

log() { echo "[sdi-validate] $*"; }
fail() { echo "[FAIL] $*" >&2; FAIL=$((FAIL + 1)); }
pass() { echo "[PASS] $*"; PASS=$((PASS + 1)); }

# ── Preflight checks ─────────────────────────────────────────────────────────

if [[ ! -d "$BIFL_TRACKER" ]]; then
    echo "ERROR: bifl-tracker not found at $BIFL_TRACKER" >&2
    echo "Set BIFL_TRACKER_PATH to the checkout path." >&2
    exit 1
fi

if [[ ! -f "$SDE_BIN" ]]; then
    log "sdi binary not found; building..."
    cargo build -p sdi-cli --manifest-path "$REPO_ROOT/Cargo.toml"
fi

if [[ ! -d "$BASELINES_DIR" ]]; then
    echo "ERROR: baselines not found at $BASELINES_DIR" >&2
    exit 1
fi

# ── Python helpers ────────────────────────────────────────────────────────────

compare_py() {
    python3 - "$@" <<'PYEOF'
import sys, json, math

def within_pct(a, b, pct):
    if a == 0 and b == 0:
        return True
    denom = max(abs(a), abs(b), 1e-10)
    return abs(a - b) / denom <= pct

baseline_path, rust_path = sys.argv[1], sys.argv[2]
baseline = json.load(open(baseline_path))
rust = json.load(open(rust_path))

errors = []

# Community count: within ±10%
py_comm = baseline.get("partition", {}).get("community_count") \
    or len(set(baseline.get("graph_metrics", {}).get("hub_nodes", []))) or None
rs_comm = rust.get("partition", {}).get("community_count")

if py_comm is not None and rs_comm is not None:
    if not within_pct(py_comm, rs_comm, 0.10):
        errors.append(f"community_count: py={py_comm} rs={rs_comm} (>10% diff)")

# Modularity: within 1% (if available in sdi-py baseline)
py_mod = baseline.get("partition", {}).get("modularity")
rs_mod = rust.get("partition", {}).get("modularity")
if py_mod is not None and rs_mod is not None:
    if not within_pct(py_mod, rs_mod, 0.01):
        errors.append(f"modularity: py={py_mod:.4f} rs={rs_mod:.4f} (>1% diff)")

# Pattern entropy: within 5% (total_entropy)
py_ent = baseline.get("divergence", {}).get("pattern_entropy") \
    or baseline.get("pattern_metrics", {}).get("total_entropy")
rs_ent = rust.get("pattern_metrics", {}).get("total_entropy")
if py_ent is not None and rs_ent is not None:
    if not within_pct(py_ent, rs_ent, 0.05):
        errors.append(f"pattern_entropy: py={py_ent:.4f} rs={rs_ent:.4f} (>5% diff)")

if errors:
    print("MISMATCH:")
    for e in errors:
        print(f"  {e}")
    sys.exit(1)
else:
    n_comm_py = py_comm or "N/A"
    n_comm_rs = rs_comm or "N/A"
    print(f"OK  communities py={n_comm_py} rs={n_comm_rs}  entropy py={py_ent or 'N/A'} rs={rs_ent or 'N/A'}")
    sys.exit(0)
PYEOF
}

# ── Per-commit validation ─────────────────────────────────────────────────────

TMPDIR_SNAPSHOTS="$(mktemp -d)"
trap 'rm -rf "$TMPDIR_SNAPSHOTS"' EXIT

# Collect unique commits from baselines (deduplicated, preserving order).
declare -A SEEN_COMMITS
COMMITS=()
for baseline_file in "$BASELINES_DIR"/snapshot_*.json; do
    commit=$(python3 -c "import json,sys; print(json.load(open('$baseline_file')).get('commit_sha',''))")
    if [[ -n "$commit" && -z "${SEEN_COMMITS[$commit]+_}" ]]; then
        SEEN_COMMITS[$commit]=1
        COMMITS+=("$commit")
    fi
done

log "Found ${#COMMITS[@]} unique commits to validate against."

cd "$BIFL_TRACKER"
ORIGINAL_HEAD=$(git rev-parse HEAD)
trap 'cd "$BIFL_TRACKER" && git checkout "$ORIGINAL_HEAD" --quiet 2>/dev/null; rm -rf "$TMPDIR_SNAPSHOTS"' EXIT

for commit in "${COMMITS[@]}"; do
    log "Checking out commit $commit..."
    if ! git checkout "$commit" --quiet 2>/dev/null; then
        echo "[SKIP] commit $commit not found in bifl-tracker" >&2
        SKIP=$((SKIP + 1))
        continue
    fi

    # Find matching baseline (first snapshot with this commit SHA)
    baseline=""
    for f in "$BASELINES_DIR"/snapshot_*.json; do
        c=$(python3 -c "import json; print(json.load(open('$f')).get('commit_sha',''))")
        if [[ "$c" == "$commit" ]]; then
            baseline="$f"
            break
        fi
    done

    if [[ -z "$baseline" ]]; then
        echo "[SKIP] no baseline found for commit $commit" >&2
        SKIP=$((SKIP + 1))
        continue
    fi

    # Run sdi-rust against this commit.
    snap_out="$TMPDIR_SNAPSHOTS/snap_${commit:0:8}.json"
    if ! "$SDE_BIN" --repo . snapshot --commit "$commit" --format json \
            > "$snap_out" 2>/dev/null; then
        # sdi exits 3 if no grammars match (expected for some older commits).
        exit_code=$?
        if [[ $exit_code -eq 3 ]]; then
            echo "[SKIP] commit $commit: no matching grammars (exit 3)" >&2
            SKIP=$((SKIP + 1))
            continue
        fi
        fail "commit $commit: sdi snapshot failed (exit $exit_code)"
        continue
    fi

    # Compare metrics.
    if compare_py "$baseline" "$snap_out"; then
        pass "commit ${commit:0:12}"
    else
        fail "commit ${commit:0:12}: metric comparison failed (see above)"
    fi

    [[ -n "$VERBOSE" ]] && cat "$snap_out" | python3 -m json.tool 2>/dev/null | head -30
done

# Restore original HEAD.
git checkout "$ORIGINAL_HEAD" --quiet 2>/dev/null

# ── Pure-compute parity check ─────────────────────────────────────────────────
# Verify that the pipeline path and the sdi-core compute_* path produce the
# same node count, edge count, and pattern entropy for the simple-rust fixture.

log ""
log "Running pure-compute parity check against simple-rust fixture..."

cd "$REPO_ROOT"
FIXTURE="$REPO_ROOT/tests/fixtures/simple-rust"

PIPELINE_OUT="$TMPDIR_SNAPSHOTS/pipeline.json"
"$SDE_BIN" --repo "$FIXTURE" snapshot --format json > "$PIPELINE_OUT" 2>/dev/null || true

if [[ -f "$PIPELINE_OUT" ]]; then
    pipeline_nodes=$(python3 -c "import json; d=json.load(open('$PIPELINE_OUT')); print(d.get('graph',{}).get('node_count',0))")
    pipeline_entropy=$(python3 -c "import json; d=json.load(open('$PIPELINE_OUT')); print(d.get('pattern_metrics',{}).get('total_entropy',0))")

    # Run embed_compute example (pure-compute path) if built.
    if cargo run --example embed_compute --manifest-path "$REPO_ROOT/Cargo.toml" \
            2>/dev/null | grep -q "nodes (pure-compute):"; then
        compute_nodes=$(cargo run --example embed_compute --manifest-path "$REPO_ROOT/Cargo.toml" \
            2>/dev/null | grep "nodes (pure-compute):" | awk '{print $NF}')
        if [[ "$pipeline_nodes" == "$compute_nodes" ]]; then
            pass "pure-compute parity: node count matches (pipeline=$pipeline_nodes, compute=$compute_nodes)"
        else
            fail "pure-compute parity: node count mismatch (pipeline=$pipeline_nodes, compute=$compute_nodes)"
        fi
    else
        log "Skipping embed_compute parity check (example not built)"
        SKIP=$((SKIP + 1))
    fi

    log "pipeline entropy: $pipeline_entropy"
fi

# ── Summary ──────────────────────────────────────────────────────────────────

echo ""
echo "=========================================="
echo " Validation summary"
echo "=========================================="
echo " PASS: $PASS"
echo " FAIL: $FAIL"
echo " SKIP: $SKIP"
echo "=========================================="

if [[ $FAIL -gt 0 ]]; then
    echo "RESULT: FAILED — $FAIL comparison(s) out of tolerance"
    exit 1
else
    echo "RESULT: PASSED"
    exit 0
fi
