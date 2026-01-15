#!/bin/bash
# Generate macOS ICNS icon from master SVG
# Usage: ./scripts/generate-macos-icns.sh
#
# Prerequisites:
#   - ImageMagick (magick command)
#   - iconutil (macOS built-in)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

SOURCE_SVG="$PROJECT_ROOT/assets/icon.svg"
OUTPUT_ICNS="$PROJECT_ROOT/assets/macos/TrialSubmissionStudio.app/Contents/Resources/AppIcon.icns"

# Verify source exists
if [[ ! -f "$SOURCE_SVG" ]]; then
    echo "Error: Source icon not found at $SOURCE_SVG"
    exit 1
fi

# Check for ImageMagick
if ! command -v magick &> /dev/null; then
    echo "Error: ImageMagick (magick) is required but not installed."
    echo "Install with: brew install imagemagick"
    exit 1
fi

# Check for iconutil (macOS only)
if ! command -v iconutil &> /dev/null; then
    echo "Error: iconutil is required (macOS only)"
    exit 1
fi

echo "Generating macOS ICNS from: $SOURCE_SVG"

# Create output directory
mkdir -p "$(dirname "$OUTPUT_ICNS")"

# Create temporary iconset directory
ICONSET_DIR=$(mktemp -d)/AppIcon.iconset
mkdir -p "$ICONSET_DIR"

# Generate all required sizes
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

# Convert to ICNS
iconutil -c icns "$ICONSET_DIR" -o "$OUTPUT_ICNS"

# Cleanup
rm -rf "$(dirname "$ICONSET_DIR")"

echo "Generated: $OUTPUT_ICNS"
