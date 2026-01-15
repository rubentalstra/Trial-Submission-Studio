#!/bin/bash
# Build Flatpak for local testing
# Usage: ./scripts/flatpak.sh [--generate-sources]
#
# Options:
#   --generate-sources  Only generate cargo-sources.json from Cargo.lock
#
# Prerequisites:
#   - flatpak
#   - flatpak-builder
#   - flatpak-cargo-generator (pip install flatpak-cargo-generator)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

APP_ID="io.github.rubentalstra.trial-submission-studio"
MANIFEST="$PROJECT_ROOT/assets/flatpak/${APP_ID}.json"
CARGO_SOURCES="$PROJECT_ROOT/assets/flatpak/cargo-sources.json"

GENERATE_ONLY=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --generate-sources)
            GENERATE_ONLY=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

cd "$PROJECT_ROOT"

echo "=== Flatpak Build ==="

# Check for flatpak-cargo-generator
if ! command -v flatpak-cargo-generator &> /dev/null; then
    echo "Installing flatpak-cargo-generator..."
    pip install --user flatpak-cargo-generator || {
        echo "Error: Could not install flatpak-cargo-generator"
        echo "Install manually: pip install flatpak-cargo-generator"
        exit 1
    }
fi

# Generate cargo sources from Cargo.lock
echo ""
echo "Generating cargo-sources.json from Cargo.lock..."
flatpak-cargo-generator Cargo.lock -o "$CARGO_SOURCES"
echo "Generated: $CARGO_SOURCES"

if $GENERATE_ONLY; then
    echo ""
    echo "=== Source Generation Complete ==="
    exit 0
fi

# Check for flatpak-builder
if ! command -v flatpak-builder &> /dev/null; then
    echo "Error: flatpak-builder is required but not installed"
    echo "Install with: sudo apt install flatpak-builder (Debian/Ubuntu)"
    exit 1
fi

# Add Flathub remote if not present
echo ""
echo "Ensuring Flathub remote is configured..."
flatpak remote-add --if-not-exists --user flathub https://flathub.org/repo/flathub.flatpakrepo || true

# Install required runtime and SDK
echo ""
echo "Installing runtime and SDK (if needed)..."
flatpak install --user -y flathub org.freedesktop.Platform//24.08 org.freedesktop.Sdk//24.08 || true
flatpak install --user -y flathub org.freedesktop.Sdk.Extension.rust-stable//24.08 || true

# Build Flatpak
echo ""
echo "Building Flatpak..."
flatpak-builder --user --install --force-clean \
    build-dir \
    "$MANIFEST"

echo ""
echo "=== Flatpak Build Complete ==="
echo "Run with: flatpak run $APP_ID"
