#!/bin/bash
# Generate Linux hicolor icons from master SVG
# Usage: ./scripts/generate-linux-icons.sh
#
# Prerequisites:
#   - ImageMagick (magick command)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

SOURCE_SVG="$PROJECT_ROOT/assets/icon.svg"
OUTPUT_DIR="$PROJECT_ROOT/assets/linux/icons/hicolor"
ICON_NAME="io.github.rubentalstra.trial-submission-studio.png"

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

echo "Generating Linux hicolor icons from: $SOURCE_SVG"

SIZES=(16 24 32 48 64 96 128 256 512)

for SIZE in "${SIZES[@]}"; do
    DIR="$OUTPUT_DIR/${SIZE}x${SIZE}/apps"
    mkdir -p "$DIR"
    magick -background transparent "$SOURCE_SVG" -resize "${SIZE}x${SIZE}" -strip PNG32:"$DIR/$ICON_NAME"
    echo "  Generated: ${SIZE}x${SIZE}"
done

echo "All Linux icons generated in: $OUTPUT_DIR"
