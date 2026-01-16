#!/bin/bash
# Build macOS binary for specified target
# Usage: ./scripts/build-macos.sh
#
# Environment variables:
#   TARGET - Rust target triple (default: host architecture)
#   TSS_BUILD_NUMBER - Build number for version info

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Determine target architecture
TARGET="${TARGET:-$(rustc -vV | grep host | cut -d' ' -f2)}"

echo "=== Building macOS Binary ==="
echo "Target: $TARGET"
echo "Build number: ${TSS_BUILD_NUMBER:-LOCAL}"

cd "$PROJECT_ROOT"

# Ensure target is available
rustup target add "$TARGET" 2>/dev/null || true

# Build release binaries (GUI and updater helper)
cargo build --release --target "$TARGET" --package tss-gui --package tss-updater-helper

BINARY="target/$TARGET/release/trial-submission-studio"
HELPER="target/$TARGET/release/tss-updater-helper"

if [[ ! -f "$BINARY" ]]; then
    echo "Error: Binary not found at $BINARY"
    exit 1
fi

if [[ ! -f "$HELPER" ]]; then
    echo "Error: Helper binary not found at $HELPER"
    exit 1
fi

echo ""
echo "Binaries built successfully:"
echo "  App:    $BINARY ($(du -h "$BINARY" | cut -f1))"
echo "  Helper: $HELPER ($(du -h "$HELPER" | cut -f1))"
