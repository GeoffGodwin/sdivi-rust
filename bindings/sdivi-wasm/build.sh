#!/usr/bin/env bash
# build.sh — local-dev WASM build for @geoffgodwin/sdivi-wasm
#
# Prerequisites:
#   cargo install wasm-pack
#   cargo install wasm-opt   (or: brew install binaryen / apt install binaryen)
#
# Usage: ./build.sh [--dev]
#
# Produces two targets under pkg/:
#   pkg/bundler/  — for webpack, vite, rollup (ESM + import.meta.url wasm loading)
#   pkg/node/     — for Node.js 18+ CLI/server consumers (CJS + synchronous fs wasm loading)
# A top-level pkg/package.json with conditional exports is copied from pkg-template/.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

MODE="${1:-}"

if [[ "$MODE" == "--dev" ]]; then
    echo "==> Building WASM bundler target (dev profile)…"
    wasm-pack build --target bundler --dev --out-dir pkg/bundler

    echo "==> Building WASM nodejs target (dev profile)…"
    wasm-pack build --target nodejs --dev --out-dir pkg/node
else
    echo "==> Building WASM bundler target (release profile)…"
    wasm-pack build --target bundler --release --out-dir pkg/bundler

    echo "==> Building WASM nodejs target (release profile)…"
    wasm-pack build --target nodejs --release --out-dir pkg/node

    WASM_BUNDLER="pkg/bundler/sdivi_wasm_bg.wasm"
    WASM_NODE="pkg/node/sdivi_wasm_bg.wasm"

    if command -v wasm-opt &>/dev/null; then
        if [[ -f "$WASM_BUNDLER" ]]; then
            echo "==> Optimising bundler .wasm with wasm-opt -Os…"
            wasm-opt -Os -o "$WASM_BUNDLER" "$WASM_BUNDLER"
        fi
        if [[ -f "$WASM_NODE" ]]; then
            echo "==> Optimising nodejs .wasm with wasm-opt -Os…"
            wasm-opt -Os -o "$WASM_NODE" "$WASM_NODE"
        fi
    else
        echo "[warn] wasm-opt not found — skipping size optimisation"
    fi

    SIZE_BUNDLER=$(wc -c < "$WASM_BUNDLER" 2>/dev/null || echo "?")
    SIZE_NODE=$(wc -c < "$WASM_NODE" 2>/dev/null || echo "?")
    echo "==> bundler .wasm size: ${SIZE_BUNDLER} bytes"
    echo "==> nodejs  .wasm size: ${SIZE_NODE} bytes"

    # 5 MB combined budget (two builds)
    COMBINED=$(( SIZE_BUNDLER + SIZE_NODE ))
    if [[ "$COMBINED" -gt 5242880 ]]; then
        echo "[WARN] Combined bundle exceeds 5 MB budget (${COMBINED} bytes)"
    fi
fi

echo "==> Assembling pkg/package.json from pkg-template/…"
cp pkg-template/package.json pkg/package.json

# wasm-pack writes a `.gitignore` containing `*` into each out-dir. `npm pack`
# honors it and silently drops bundler/ and node/ from the tarball despite the
# `files` field. Remove them so the published tarball actually contains the
# build artifacts.
rm -f pkg/bundler/.gitignore pkg/node/.gitignore pkg/.gitignore

echo "==> Build complete."
echo "    bundler target : $SCRIPT_DIR/pkg/bundler/"
echo "    nodejs target  : $SCRIPT_DIR/pkg/node/"
echo "    package.json   : $SCRIPT_DIR/pkg/package.json"
