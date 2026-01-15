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

# Build release binary
cargo build --release --target "$TARGET"

BINARY="target/$TARGET/release/trial-submission-studio"

if [[ ! -f "$BINARY" ]]; then
    echo "Error: Binary not found at $BINARY"
    exit 1
fi

echo ""
echo "Binary built successfully: $BINARY"
echo "Size: $(du -h "$BINARY" | cut -f1)"
