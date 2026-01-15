#!/bin/bash
# Build Windows binary for specified target
# Usage: ./scripts/build-windows.sh
#
# Environment variables:
#   TARGET - Rust target triple (default: x86_64-pc-windows-msvc)
#   TSS_BUILD_NUMBER - Build number for version info
#
# Note: This script is designed to run in Git Bash on Windows
# or via cross-compilation on other platforms

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Determine target architecture
TARGET="${TARGET:-x86_64-pc-windows-msvc}"

echo "=== Building Windows Binary ==="
echo "Target: $TARGET"
echo "Build number: ${TSS_BUILD_NUMBER:-LOCAL}"

cd "$PROJECT_ROOT"

# Ensure target is available
rustup target add "$TARGET" 2>/dev/null || true

# Build release binary
cargo build --release --target "$TARGET"

BINARY="target/$TARGET/release/trial-submission-studio.exe"

if [[ ! -f "$BINARY" ]]; then
    echo "Error: Binary not found at $BINARY"
    exit 1
fi

echo ""
echo "Binary built successfully: $BINARY"

# Show size (works on both Windows and Unix)
if command -v du &> /dev/null; then
    echo "Size: $(du -h "$BINARY" | cut -f1)"
fi
