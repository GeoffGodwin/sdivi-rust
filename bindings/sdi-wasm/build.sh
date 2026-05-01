#!/usr/bin/env bash
# build.sh — local-dev WASM build for @geoffgodwin/sdi-wasm
#
# Prerequisites:
#   cargo install wasm-pack
#   cargo install wasm-opt   (or: brew install binaryen / apt install binaryen)
#
# Usage: ./build.sh [--dev]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

MODE="${1:-}"

if [[ "$MODE" == "--dev" ]]; then
    echo "==> Building WASM (dev profile)…"
    wasm-pack build --target bundler --dev
else
    echo "==> Building WASM (release-wasm profile)…"
    wasm-pack build --target bundler --profile release-wasm

    WASM_FILE="pkg/sdi_wasm_bg.wasm"
    if command -v wasm-opt &>/dev/null && [[ -f "$WASM_FILE" ]]; then
        echo "==> Optimising with wasm-opt -Os…"
        wasm-opt -Os -o "$WASM_FILE" "$WASM_FILE"
    else
        echo "[warn] wasm-opt not found — skipping size optimisation"
    fi

    SIZE=$(wc -c < "$WASM_FILE" 2>/dev/null || echo "?")
    echo "==> .wasm size: ${SIZE} bytes"
    if [[ "$SIZE" -gt 1258291 ]]; then
        echo "[WARN] Bundle exceeds 1.2 MB budget (${SIZE} bytes)"
    fi
fi

echo "==> Build complete. Output in: $SCRIPT_DIR/pkg/"
