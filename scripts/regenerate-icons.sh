#!/bin/bash
# Regenerate all icons from the source logo.svg
# Usage: ./scripts/regenerate-icons.sh
#
# Prerequisites:
#   - ImageMagick (magick command)
#   - iconutil (macOS built-in, for ICNS generation)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

SOURCE_SVG="$PROJECT_ROOT/docs/src/images/logo.svg"

# Verify source exists
if [[ ! -f "$SOURCE_SVG" ]]; then
    echo "Error: Source logo not found at $SOURCE_SVG"
    exit 1
fi

# Check for ImageMagick
if ! command -v magick &> /dev/null; then
    echo "Error: ImageMagick (magick) is required but not installed."
    echo "Install with: brew install imagemagick"
    exit 1
fi

echo "Regenerating icons from: $SOURCE_SVG"

# Step 1: Copy SVG to other locations
echo "Copying SVG files..."
cp "$SOURCE_SVG" "$PROJECT_ROOT/crates/tss-gui/assets/icon.svg"
cp "$SOURCE_SVG" "$PROJECT_ROOT/docs/theme/favicon.svg"
echo "  - crates/tss-gui/assets/icon.svg"
echo "  - docs/theme/favicon.svg"

# Step 2: Generate PNG files (256x256)
echo "Generating PNG files..."
magick -background transparent "$SOURCE_SVG" -resize 256x256 "$PROJECT_ROOT/crates/tss-gui/assets/icon.png"
magick -background transparent "$SOURCE_SVG" -resize 256x256 "$PROJECT_ROOT/packaging/linux/trial-submission-studio.png"
echo "  - crates/tss-gui/assets/icon.png (256x256)"
echo "  - packaging/linux/trial-submission-studio.png (256x256)"

# Step 3: Generate Windows ICO (multi-resolution)
echo "Generating Windows ICO..."
TEMP_DIR=$(mktemp -d)
magick -background transparent "$SOURCE_SVG" -resize 16x16 "$TEMP_DIR/icon-16.png"
magick -background transparent "$SOURCE_SVG" -resize 32x32 "$TEMP_DIR/icon-32.png"
magick -background transparent "$SOURCE_SVG" -resize 48x48 "$TEMP_DIR/icon-48.png"
magick -background transparent "$SOURCE_SVG" -resize 256x256 "$TEMP_DIR/icon-256.png"
magick "$TEMP_DIR/icon-16.png" "$TEMP_DIR/icon-32.png" "$TEMP_DIR/icon-48.png" "$TEMP_DIR/icon-256.png" "$PROJECT_ROOT/packaging/windows/icon.ico"
rm -rf "$TEMP_DIR"
echo "  - packaging/windows/icon.ico (16/32/48/256px)"

# Step 4: Generate macOS ICNS (requires iconutil on macOS)
if command -v iconutil &> /dev/null; then
    echo "Generating macOS ICNS..."
    ICONSET_DIR=$(mktemp -d)/AppIcon.iconset
    mkdir -p "$ICONSET_DIR"

    magick -background transparent "$SOURCE_SVG" -resize 16x16 "$ICONSET_DIR/icon_16x16.png"
    magick -background transparent "$SOURCE_SVG" -resize 32x32 "$ICONSET_DIR/icon_16x16@2x.png"
    magick -background transparent "$SOURCE_SVG" -resize 32x32 "$ICONSET_DIR/icon_32x32.png"
    magick -background transparent "$SOURCE_SVG" -resize 64x64 "$ICONSET_DIR/icon_32x32@2x.png"
    magick -background transparent "$SOURCE_SVG" -resize 128x128 "$ICONSET_DIR/icon_128x128.png"
    magick -background transparent "$SOURCE_SVG" -resize 256x256 "$ICONSET_DIR/icon_128x128@2x.png"
    magick -background transparent "$SOURCE_SVG" -resize 256x256 "$ICONSET_DIR/icon_256x256.png"
    magick -background transparent "$SOURCE_SVG" -resize 512x512 "$ICONSET_DIR/icon_256x256@2x.png"
    magick -background transparent "$SOURCE_SVG" -resize 512x512 "$ICONSET_DIR/icon_512x512.png"
    magick -background transparent "$SOURCE_SVG" -resize 1024x1024 "$ICONSET_DIR/icon_512x512@2x.png"

    iconutil -c icns "$ICONSET_DIR" -o "$PROJECT_ROOT/packaging/macos/AppIcon.icns"
    rm -rf "$(dirname "$ICONSET_DIR")"
    echo "  - packaging/macos/AppIcon.icns (16-1024px)"
else
    echo "Warning: iconutil not found (macOS only). Skipping ICNS generation."
fi

echo ""
echo "Done! All icons regenerated successfully."
