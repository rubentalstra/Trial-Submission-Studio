#!/bin/bash
# Regenerate all platform icons from the master SVG
# Usage: ./scripts/regenerate-all-icons.sh
#
# Prerequisites:
#   - ImageMagick (magick command)
#   - iconutil (macOS only, for ICNS generation)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

SOURCE_SVG="$PROJECT_ROOT/assets/icon.svg"

# Verify source exists
if [[ ! -f "$SOURCE_SVG" ]]; then
    echo "Error: Master icon not found at $SOURCE_SVG"
    exit 1
fi

# Check for ImageMagick
if ! command -v magick &> /dev/null; then
    echo "Error: ImageMagick (magick) is required but not installed."
    echo "Install with: brew install imagemagick (macOS) or apt install imagemagick (Linux)"
    exit 1
fi

echo "=== Regenerating All Icons ==="
echo "Source: $SOURCE_SVG"
echo ""

# Step 1: Generate base PNG for GUI usage
echo "Generating base PNG..."
magick -background transparent "$SOURCE_SVG" -resize 256x256 "$PROJECT_ROOT/assets/icon.png"
echo "  - assets/icon.png (256x256)"

# Step 2: Copy to crates/tss-gui/assets for runtime use
echo ""
echo "Copying to GUI assets..."
mkdir -p "$PROJECT_ROOT/crates/tss-gui/assets"
cp "$SOURCE_SVG" "$PROJECT_ROOT/crates/tss-gui/assets/icon.svg"
cp "$PROJECT_ROOT/assets/icon.png" "$PROJECT_ROOT/crates/tss-gui/assets/icon.png"
echo "  - crates/tss-gui/assets/icon.svg"
echo "  - crates/tss-gui/assets/icon.png"

# Step 3: Copy to docs theme
echo ""
echo "Copying to docs theme..."
if [[ -d "$PROJECT_ROOT/docs/theme" ]]; then
    cp "$SOURCE_SVG" "$PROJECT_ROOT/docs/theme/favicon.svg"
    echo "  - docs/theme/favicon.svg"
else
    echo "  (skipped - docs/theme not found)"
fi

# Step 4: Generate Windows ICO
echo ""
echo "=== Windows ICO ==="
"$SCRIPT_DIR/generate-windows-ico.sh"

# Step 5: Generate Linux hicolor icons
echo ""
echo "=== Linux Icons ==="
"$SCRIPT_DIR/generate-linux-icons.sh"

# Step 6: Generate macOS ICNS (macOS only)
echo ""
echo "=== macOS ICNS ==="
if command -v iconutil &> /dev/null; then
    "$SCRIPT_DIR/generate-macos-icns.sh"
else
    echo "Skipping macOS ICNS generation (iconutil not available - macOS only)"
fi

echo ""
echo "=== All icons regenerated successfully ==="
