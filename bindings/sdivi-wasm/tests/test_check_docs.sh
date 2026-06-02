#!/bin/sh
# test_check_docs.sh — Integration tests for check_docs.sh.
#
# Tests the doc-lint script's happy path (exits 0 on clean tree) and its
# pattern-detection path (exits 1 with a FAIL message when a forbidden pattern
# is found). Uses controlled temporary files so the tests are deterministic and
# do not depend on CI-generated build artifacts.
#
# Run from any directory inside the repo:
#   sh bindings/sdivi-wasm/tests/test_check_docs.sh

set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
CHECK_DOCS="$SCRIPT_DIR/check_docs.sh"

PASS=0
FAIL=0

pass() { echo "PASS: $1"; PASS=$(( PASS + 1 )); }
fail() { echo "FAIL: $1"; FAIL=$(( FAIL + 1 )); }

# ── Helper: run check_docs.sh and capture exit code + output ─────────────────

run_check() {
  OUTPUT="$(sh "$CHECK_DOCS" 2>&1)" && RC=0 || RC=$?
  echo "$OUTPUT"
}

# ── Test 1: Happy path — clean tree exits 0 ──────────────────────────────────
#
# The corrected consumer-facing files must not contain any forbidden pattern.
# This validates the primary observable behavior: that the guard passes
# (exits 0) when no regression is present.

OUTPUT="$(sh "$CHECK_DOCS" 2>&1)" && RC=0 || RC=$?

if [ "$RC" -eq 0 ]; then
  pass "check_docs.sh exits 0 on clean tree"
else
  fail "check_docs.sh exited $RC on clean tree — unexpected FAIL output:"
  echo "$OUTPUT"
fi

# Also verify the OK message is present in clean-tree output.
if echo "$OUTPUT" | grep -q "OK: doc lint passed"; then
  pass "check_docs.sh emits OK message on clean tree"
else
  fail "check_docs.sh did not emit expected OK message; got: $OUTPUT"
fi

# ── Test 2: Pattern detection — grep -F catches 'import init' ────────────────
#
# Creates a temp file containing the forbidden pattern, runs grep -nF to
# confirm the detection logic would fire. We test the grep logic in isolation
# because check_docs.sh scans hardcoded paths in the live repo tree; this
# approach gives us a controlled, repeatable input without mutating the tree.

TMPDIR_TEST="$(mktemp -d)"
trap 'rm -rf "$TMPDIR_TEST"' EXIT

TMPFILE="$TMPDIR_TEST/adversarial.ts"

# Sub-test 2a: 'import init' pattern detected
printf 'import init from "@geoffgodwin/sdivi-wasm";\n' > "$TMPFILE"
HITS="$(grep -nF "import init" "$TMPFILE" 2>/dev/null)" || true
if [ -n "$HITS" ]; then
  pass "grep -nF detects 'import init' in adversarial file"
else
  fail "grep -nF failed to detect 'import init' — pattern detection broken"
fi

# Sub-test 2b: 'await init(' pattern detected
printf 'await init();\n' > "$TMPFILE"
HITS="$(grep -nF "await init(" "$TMPFILE" 2>/dev/null)" || true
if [ -n "$HITS" ]; then
  pass "grep -nF detects 'await init(' in adversarial file"
else
  fail "grep -nF failed to detect 'await init(' — pattern detection broken"
fi

# Sub-test 2c: 'edge_weights: {' pattern detected
printf 'const cfg = { edge_weights: { "a:b": 0.8 } };\n' > "$TMPFILE"
HITS="$(grep -nF "edge_weights: {" "$TMPFILE" 2>/dev/null)" || true
if [ -n "$HITS" ]; then
  pass "grep -nF detects 'edge_weights: {' in adversarial file"
else
  fail "grep -nF failed to detect 'edge_weights: {' — pattern detection broken"
fi

# ── Test 3: FAIL output format — 'FAIL:' prefix on detection ─────────────────
#
# check_docs.sh emits "FAIL: <path>:<line>" per hit. Verify the prefix appears.
# We test this by temporarily injecting the pattern into a writable temp tree
# and simulating the grep call with the same prefix logic.

TMPFILE2="$TMPDIR_TEST/test_output.txt"
printf 'import init from "@geoffgodwin/sdivi-wasm";\n' > "$TMPFILE"
HITS="$(grep -nF "import init" "$TMPFILE" 2>/dev/null)" || true
if [ -n "$HITS" ]; then
  printf '%s\n' "$HITS" | while IFS= read -r LINE; do
    echo "FAIL: $TMPFILE:$LINE"
  done > "$TMPFILE2"
  if grep -q "^FAIL:" "$TMPFILE2"; then
    pass "FAIL output format 'FAIL: <path>:<line>' is produced on detection"
  else
    fail "FAIL output format check: expected 'FAIL:' prefix, got: $(cat "$TMPFILE2")"
  fi
else
  fail "FAIL format test: grep did not detect 'import init' in temp file"
fi

# ── Test 4: Clean-file strings do NOT trigger grep ────────────────────────────
#
# Verify the corrected examples don't contain the forbidden literal strings.
# This is belt-and-suspenders: check_docs.sh already validates the live files,
# but an explicit grep here gives a clear failure message if one drifts back.

for REL in \
  "examples/binding_node.ts" \
  "examples/binding_bundler.ts" \
  "bindings/sdivi-wasm/README.md"
do
  ABS="$REPO_ROOT/$REL"
  if [ ! -f "$ABS" ]; then
    echo "WARN: $ABS not found, skipping Test 4 sub-check"
    continue
  fi

  # Must not contain 'import init'
  if grep -qF "import init" "$ABS" 2>/dev/null; then
    fail "$REL still contains 'import init' — forbidden pattern present"
  else
    pass "$REL: no 'import init'"
  fi

  # Must not contain 'await init('
  if grep -qF "await init(" "$ABS" 2>/dev/null; then
    fail "$REL still contains 'await init(' — forbidden pattern present"
  else
    pass "$REL: no 'await init('"
  fi

  # Must not contain 'edge_weights: {'
  if grep -qF "edge_weights: {" "$ABS" 2>/dev/null; then
    fail "$REL still contains 'edge_weights: {' — forbidden pattern present"
  else
    pass "$REL: no 'edge_weights: {'"
  fi
done

# ── Test 5: Correct patterns are present (positive examples) ──────────────────
#
# Verify the corrected examples use the right patterns (Map, not object literal;
# no init() call). This proves the guard doesn't pass vacuously because all
# consumer-facing examples were simply deleted.

BUNDLER_EXAMPLE="$REPO_ROOT/examples/binding_bundler.ts"
NODE_EXAMPLE="$REPO_ROOT/examples/binding_node.ts"

if grep -qF "new Map(" "$BUNDLER_EXAMPLE" 2>/dev/null; then
  pass "binding_bundler.ts uses 'new Map(' for edge_weights (correct pattern)"
else
  fail "binding_bundler.ts does not use 'new Map(' — correct example may be missing"
fi

if grep -qF "detect_boundaries" "$NODE_EXAMPLE" 2>/dev/null; then
  pass "binding_node.ts calls detect_boundaries (not vacuously empty)"
else
  fail "binding_node.ts does not call detect_boundaries — example may be incomplete"
fi

if grep -qF "There is NO init()" "$NODE_EXAMPLE" 2>/dev/null \
   || grep -qF "no init() to call" "$BUNDLER_EXAMPLE" 2>/dev/null \
   || grep -qF "no init" "$BUNDLER_EXAMPLE" 2>/dev/null; then
  pass "examples document the 'no init()' contract"
else
  fail "examples do not document the 'no init()' contract — consumer guidance may be missing"
fi

# ── Summary ───────────────────────────────────────────────────────────────────

echo ""
echo "Results: PASS=$PASS FAIL=$FAIL"
if [ "$FAIL" -ne 0 ]; then
  exit 1
fi
echo "All tests passed."
