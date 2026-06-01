#!/bin/sh
# check_docs.sh — Forbidden-pattern lint for @geoffgodwin/sdivi-wasm consumer docs.
#
# ─── Configuration (update here when adding new regressions) ─────────────────
#
# Locked regressions from M47 (both shipped broken before M47):
#
#   "import init" / "await init("
#       The M12-era await-init idiom (designed for --target web) does not work
#       with the bundler+nodejs build — there is no init() export. Caught by
#       tsc for code, but grep catches prose snippets in doc fences that tsc
#       cannot reach as standalone files.
#
#   "edge_weights: {"
#       Passing edge_weights as a plain object literal; it must be a JS Map.
#       serde-wasm-bindgen rejects a plain object at runtime.
#
# Files scanned (relative to repo root):
#   bindings/sdivi-wasm/README.md
#   README.md
#   docs/pattern-categories.md
#   bindings/sdivi-wasm/src/lib.rs   (rustdoc code fences)
#   examples/binding_node.ts
#   examples/binding_bundler.ts
#
# ─────────────────────────────────────────────────────────────────────────────
#
# Run locally from any directory inside the repo:
#   sh bindings/sdivi-wasm/tests/check_docs.sh

set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

FAIL=0

# scan_pattern LITERAL_STRING
#   Greps all consumer-facing files for LITERAL_STRING (grep -F, no regex).
#   Sets FAIL=1 and prints "FAIL: <path>:<lineno>:<match>" for each hit.
scan_pattern() {
  PAT="$1"
  for REL in \
    "bindings/sdivi-wasm/README.md" \
    "README.md" \
    "docs/pattern-categories.md" \
    "bindings/sdivi-wasm/src/lib.rs" \
    "examples/binding_node.ts" \
    "examples/binding_bundler.ts"
  do
    ABS="$REPO_ROOT/$REL"
    if [ ! -f "$ABS" ]; then
      echo "WARN: $ABS not found, skipping"
      continue
    fi
    HITS="$(grep -nF "$PAT" "$ABS" 2>/dev/null)" || true
    if [ -n "$HITS" ]; then
      printf '%s\n' "$HITS" | while IFS= read -r LINE; do
        echo "FAIL: $ABS:$LINE"
      done
      FAIL=1
    fi
  done
}

# ─── Forbidden patterns (one call per pattern) ───────────────────────────────
scan_pattern "import init"
scan_pattern "await init("
scan_pattern "edge_weights: {"
# ─────────────────────────────────────────────────────────────────────────────

if [ "$FAIL" -ne 0 ]; then
  echo ""
  echo "FAIL: forbidden pattern(s) found in consumer-facing docs/examples."
  echo "  'import init' / 'await init(' — bundler+nodejs build has no init() export."
  echo "  'edge_weights: {' — edge_weights must be a JS Map, not a plain object."
  exit 1
fi

echo "OK: doc lint passed — no forbidden patterns in consumer-facing docs."
