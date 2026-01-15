#!/bin/bash
# Generate Windows ICO icon from master SVG
# Usage: ./scripts/generate-windows-ico.sh
#
# Prerequisites:
#   - ImageMagick (magick command)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

SOURCE_SVG="$PROJECT_ROOT/assets/icon.svg"
OUTPUT_ICO="$PROJECT_ROOT/assets/windows/trial-submission-studio.ico"

# Verify source exists
if [[ ! -f "$SOURCE_SVG" ]]; then
    echo "Error: Source icon not found at $SOURCE_SVG"
    exit 1
fi

# Check for ImageMagick
if ! command -v magick &> /dev/null; then
    echo "Error: ImageMagick (magick) is required but not installed."
    echo "Install with: brew install imagemagick (macOS) or apt install imagemagick (Linux)"
    exit 1
fi

echo "Generating Windows ICO from: $SOURCE_SVG"

# Create output directory
mkdir -p "$(dirname "$OUTPUT_ICO")"

# Create temporary directory for PNGs
TEMP_DIR=$(mktemp -d)

# Generate multiple sizes
SIZES=(16 32 48 64 128 256 512)
PNG_FILES=()

for SIZE in "${SIZES[@]}"; do
    PNG_FILE="$TEMP_DIR/icon-${SIZE}.png"
    magick -background transparent "$SOURCE_SVG" -resize "${SIZE}x${SIZE}" "$PNG_FILE"
    PNG_FILES+=("$PNG_FILE")
    echo "  Generated: ${SIZE}x${SIZE}"
done

# Combine into ICO
magick "${PNG_FILES[@]}" "$OUTPUT_ICO"

# Cleanup
rm -rf "$TEMP_DIR"

echo "Generated: $OUTPUT_ICO"
