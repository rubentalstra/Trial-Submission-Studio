#!/bin/bash
# Package Windows ZIP and standalone executable
# Usage: ./scripts/package-windows.sh
#
# Environment variables:
#   TARGET - Rust target triple (default: x86_64-pc-windows-msvc)
#   VERSION - Version string (default: from Cargo.toml)
#
# Note: This script is designed to run in Git Bash on Windows
# or in CI environments

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

APP_NAME="trial-submission-studio"

# Determine target architecture
TARGET="${TARGET:-x86_64-pc-windows-msvc}"

# Get version from environment or Cargo.toml
if [[ -z "${VERSION:-}" ]]; then
    VERSION="v$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | head -1 | cut -d'"' -f2)"
fi

echo "=== Packaging Windows Release ==="
echo "Target: $TARGET"
echo "Version: $VERSION"

cd "$PROJECT_ROOT"

# Find binary
BINARY="target/$TARGET/release/${APP_NAME}.exe"
if [[ ! -f "$BINARY" ]]; then
    echo "Error: Binary not found at $BINARY"
    echo "Run ./scripts/build-windows.sh first."
    exit 1
fi

# Create ZIP archive
ZIP_NAME="${APP_NAME}-${VERSION}-${TARGET}.zip"
echo ""
echo "Creating ZIP archive: $ZIP_NAME"

# Use PowerShell on Windows, zip on Unix
if command -v powershell &> /dev/null; then
    powershell -Command "Compress-Archive -Path '$BINARY' -DestinationPath '$ZIP_NAME' -Force"
elif command -v zip &> /dev/null; then
    zip -j "$ZIP_NAME" "$BINARY"
else
    echo "Error: No zip utility found"
    exit 1
fi

echo "ZIP created: $ZIP_NAME"

# Copy standalone executable with versioned name
EXE_NAME="${APP_NAME}-${VERSION}-${TARGET}.exe"
echo ""
echo "Creating standalone executable: $EXE_NAME"
cp "$BINARY" "$EXE_NAME"

echo ""
echo "=== Packaging Complete ==="
echo "Created:"
echo "  - $ZIP_NAME"
echo "  - $EXE_NAME"
