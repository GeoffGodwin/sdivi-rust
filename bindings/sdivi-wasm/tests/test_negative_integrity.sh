#!/bin/sh
# test_negative_integrity.sh — Structural integrity tests for the M47 CI guard.
#
# Verifies that:
#  1. negative.ts has the expected number of @ts-expect-error directives (4).
#  2. Each @ts-expect-error in negative.ts is immediately followed by a
#     non-blank, non-comment line (no intermediate comment between directive
#     and guarded code — a previously-reported bug in cycle 1).
#  3. wasm.yml contains the required pinned TypeScript step and ordered guards.
#  4. tsconfig.json has the correct strict compiler flags and paths entries.
#  5. The node_smoke/index.mjs stale-comment has been updated to reflect M47.
#
# Run from any directory inside the repo:
#   sh bindings/sdivi-wasm/tests/test_negative_integrity.sh

set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

NEGATIVE_TS="$SCRIPT_DIR/typecheck/negative.ts"
TSCONFIG="$SCRIPT_DIR/typecheck/tsconfig.json"
WASM_YML="$REPO_ROOT/.github/workflows/wasm.yml"
INDEX_MJS="$SCRIPT_DIR/node_smoke/index.mjs"

PASS=0
FAIL=0

pass() { echo "PASS: $1"; PASS=$(( PASS + 1 )); }
fail() { echo "FAIL: $1"; FAIL=$(( FAIL + 1 )); }

# ── Test 1: negative.ts has exactly 4 @ts-expect-error directives ─────────────
#
# The milestone spec mandates four cases (await init, edge_weights, overrides,
# bracket-index Map). If the count changes, either a case was dropped (coverage
# regression) or a new case was added without updating this test.

DIRECTIVE_COUNT="$(grep -c "^// @ts-expect-error$" "$NEGATIVE_TS" 2>/dev/null)" || DIRECTIVE_COUNT=0

if [ "$DIRECTIVE_COUNT" -eq 4 ]; then
  pass "negative.ts has exactly 4 @ts-expect-error directives"
else
  fail "negative.ts has $DIRECTIVE_COUNT @ts-expect-error directives; expected 4"
fi

# ── Test 2: No intermediate comment line between @ts-expect-error and code ────
#
# TypeScript only suppresses diagnostics on the *immediately following* line.
# A comment line between the directive and the guarded code causes TS2578
# "Unused '@ts-expect-error' directive" because the directive fires on the
# blank/comment line, not the erroring statement. This was the cycle-1 bug.
#
# Algorithm: for each line containing "@ts-expect-error", verify the next
# non-empty line is neither a comment (starts with //) nor blank.

LINE_NUM=0
PREV_WAS_DIRECTIVE=0
FOUND_INTERMEDIATE_COMMENT=0

while IFS= read -r LINE; do
  LINE_NUM=$(( LINE_NUM + 1 ))
  TRIMMED="$(echo "$LINE" | sed 's/^[[:space:]]*//')"

  if [ "$PREV_WAS_DIRECTIVE" -eq 1 ]; then
    # This line immediately follows a @ts-expect-error directive.
    # It must not be blank or a comment.
    if [ -z "$TRIMMED" ]; then
      echo "FAIL: blank line after @ts-expect-error at line $((LINE_NUM - 1)) in negative.ts"
      FOUND_INTERMEDIATE_COMMENT=1
    elif echo "$TRIMMED" | grep -q "^//"; then
      echo "FAIL: comment line after @ts-expect-error at line $((LINE_NUM - 1)) in negative.ts"
      FOUND_INTERMEDIATE_COMMENT=1
    fi
    PREV_WAS_DIRECTIVE=0
  fi

  if echo "$TRIMMED" | grep -q "^// @ts-expect-error$"; then
    PREV_WAS_DIRECTIVE=1
  fi
done < "$NEGATIVE_TS"

if [ "$FOUND_INTERMEDIATE_COMMENT" -eq 0 ]; then
  pass "negative.ts: every @ts-expect-error immediately precedes its guarded statement"
else
  FAIL=$(( FAIL + 1 ))
fi

# ── Test 3: negative.ts guards against vacuous pass ───────────────────────────
#
# The milestone spec requires void calls on the bad-assignment variables to
# prevent tsc from tree-shaking the test cases away.

if grep -qF "void _badEdgeWeights" "$NEGATIVE_TS" && \
   grep -qF "void _badOverrides"   "$NEGATIVE_TS" && \
   grep -qF "void _badBracket"     "$NEGATIVE_TS" && \
   grep -qF "void detect_boundaries" "$NEGATIVE_TS"; then
  pass "negative.ts has all four anti-vacuous-pass void statements"
else
  fail "negative.ts is missing one or more void anti-vacuous-pass statements"
fi

# ── Test 4: tsconfig has all required strict flags ────────────────────────────

for FLAG in \
  '"strict": true' \
  '"noUncheckedIndexedAccess": true' \
  '"exactOptionalPropertyTypes": true' \
  '"noEmit": true' \
  '"incremental": false' \
  '"esModuleInterop": true' \
  '"skipLibCheck": true'
do
  if grep -qF "$FLAG" "$TSCONFIG" 2>/dev/null; then
    pass "tsconfig.json contains: $FLAG"
  else
    fail "tsconfig.json missing: $FLAG"
  fi
done

# ── Test 5: tsconfig has all three paths entries ──────────────────────────────
#
# Covers the coverage gap flagged by the reviewer: paths for /bundler and /node
# subpaths are declared. If one disappears, the subpath_imports.ts fixture
# will fail to compile, but this structural check gives an earlier, clearer signal.

for PATH_ENTRY in \
  '"@geoffgodwin/sdivi-wasm"' \
  '"@geoffgodwin/sdivi-wasm/bundler"' \
  '"@geoffgodwin/sdivi-wasm/node"'
do
  if grep -qF "$PATH_ENTRY" "$TSCONFIG" 2>/dev/null; then
    pass "tsconfig.json paths contains: $PATH_ENTRY"
  else
    fail "tsconfig.json paths missing: $PATH_ENTRY"
  fi
done

# ── Test 6: tsconfig include covers negative.ts and both examples ─────────────

for INC in \
  '"./negative.ts"' \
  'binding_node.ts' \
  'binding_bundler.ts' \
  'subpath_imports.ts'
do
  if grep -qF "$INC" "$TSCONFIG" 2>/dev/null; then
    pass "tsconfig.json include covers: $INC"
  else
    fail "tsconfig.json include missing: $INC"
  fi
done

# ── Test 7: wasm.yml has pinned TYPESCRIPT_VERSION env key ───────────────────

if grep -qF "TYPESCRIPT_VERSION:" "$WASM_YML" 2>/dev/null; then
  pass "wasm.yml defines TYPESCRIPT_VERSION env key"
else
  fail "wasm.yml missing TYPESCRIPT_VERSION env key"
fi

# Verify it is not 'latest' (unpinned)
if grep "TYPESCRIPT_VERSION:" "$WASM_YML" 2>/dev/null | grep -q "latest"; then
  fail "wasm.yml TYPESCRIPT_VERSION is unpinned ('latest') — must be an exact version"
else
  pass "wasm.yml TYPESCRIPT_VERSION is not 'latest'"
fi

# ── Test 8: wasm.yml typecheck step is ubuntu-gated ──────────────────────────
#
# The typecheck and doc-lint steps must be ubuntu-only (consistent with the
# existing node smoke tests). An ubuntu-gated step has 'if: matrix.os ==
# '"'"'ubuntu-latest'"'"'' on or near the step.

if grep -A2 "Typecheck WASM consumer surface" "$WASM_YML" 2>/dev/null | grep -q "ubuntu-latest"; then
  pass "wasm.yml typecheck step is ubuntu-gated"
else
  fail "wasm.yml typecheck step does not appear to be ubuntu-gated"
fi

if grep -A2 "Lint consumer docs" "$WASM_YML" 2>/dev/null | grep -q "ubuntu-latest"; then
  pass "wasm.yml doc-lint step is ubuntu-gated"
else
  fail "wasm.yml doc-lint step does not appear to be ubuntu-gated"
fi

# ── Test 9: wasm.yml typecheck step is not continue-on-error ─────────────────
#
# The steps must be required (no 'continue-on-error: true'). We check that
# the typecheck and doc-lint steps do not have that attribute.

if grep -A5 "Typecheck WASM consumer surface" "$WASM_YML" 2>/dev/null | grep -q "continue-on-error"; then
  fail "wasm.yml typecheck step has continue-on-error (must be required)"
else
  pass "wasm.yml typecheck step has no continue-on-error"
fi

if grep -A5 "Lint consumer docs" "$WASM_YML" 2>/dev/null | grep -q "continue-on-error"; then
  fail "wasm.yml doc-lint step has continue-on-error (must be required)"
else
  pass "wasm.yml doc-lint step has no continue-on-error"
fi

# ── Test 10: wasm.yml typecheck step references the tsconfig path ─────────────

if grep -qF "bindings/sdivi-wasm/tests/typecheck/tsconfig.json" "$WASM_YML" 2>/dev/null; then
  pass "wasm.yml typecheck step references the correct tsconfig path"
else
  fail "wasm.yml typecheck step does not reference bindings/sdivi-wasm/tests/typecheck/tsconfig.json"
fi

# ── Test 11: wasm.yml doc-lint step references check_docs.sh ─────────────────

if grep -qF "check_docs.sh" "$WASM_YML" 2>/dev/null; then
  pass "wasm.yml doc-lint step references check_docs.sh"
else
  fail "wasm.yml doc-lint step does not reference check_docs.sh"
fi

# ── Test 12: node_smoke/index.mjs stale comment has been updated ─────────────
#
# The old "bundler path is not exercised" comment was reconciled in M47. The
# updated comment must reference M47's tsc guard and be accurate about what
# is now validated (type contract) vs what remains deferred (runtime e2e).

if grep -qF "M47" "$INDEX_MJS" 2>/dev/null; then
  pass "node_smoke/index.mjs references M47 in its updated comment"
else
  fail "node_smoke/index.mjs does not reference M47 — stale comment may not have been updated"
fi

if grep -qF "type" "$INDEX_MJS" 2>/dev/null && \
   grep -qF "tsc" "$INDEX_MJS" 2>/dev/null; then
  pass "node_smoke/index.mjs updated comment mentions type-level validation and tsc"
else
  fail "node_smoke/index.mjs updated comment does not mention type-level validation or tsc"
fi

# ── Test 13: .gitignore covers tsc build artifacts ───────────────────────────

GITIGNORE="$REPO_ROOT/.gitignore"

if grep -qF "tsconfig.tsbuildinfo" "$GITIGNORE" 2>/dev/null; then
  pass ".gitignore covers tsconfig.tsbuildinfo"
else
  fail ".gitignore missing tsconfig.tsbuildinfo entry"
fi

# ── Summary ───────────────────────────────────────────────────────────────────

echo ""
echo "Results: PASS=$PASS FAIL=$FAIL"
if [ "$FAIL" -ne 0 ]; then
  exit 1
fi
echo "All tests passed."
