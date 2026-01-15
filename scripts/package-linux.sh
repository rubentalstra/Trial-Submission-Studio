#!/bin/bash
# Package Linux tarball and AppImage
# Usage: ./scripts/package-linux.sh
#
# Environment variables:
#   TARGET - Rust target triple (default: x86_64-unknown-linux-gnu)
#   VERSION - Version string (default: from Cargo.toml)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

APP_NAME="trial-submission-studio"
APP_ID="io.github.rubentalstra.trial-submission-studio"

# Determine target architecture
TARGET="${TARGET:-x86_64-unknown-linux-gnu}"

# Get version from environment or Cargo.toml
if [[ -z "${VERSION:-}" ]]; then
    VERSION="v$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | head -1 | cut -d'"' -f2)"
fi

echo "=== Packaging Linux Release ==="
echo "Target: $TARGET"
echo "Version: $VERSION"

cd "$PROJECT_ROOT"

# Find binary
BINARY="target/$TARGET/release/$APP_NAME"
if [[ ! -f "$BINARY" ]]; then
    echo "Error: Binary not found at $BINARY"
    echo "Run ./scripts/build-linux.sh first."
    exit 1
fi

# Create tarball
TAR_NAME="${APP_NAME}-${VERSION}-${TARGET}.tar.gz"
echo ""
echo "Creating tarball: $TAR_NAME"

mkdir -p release-tar
cp "$BINARY" release-tar/
chmod +x "release-tar/$APP_NAME"

tar -czvf "$TAR_NAME" -C release-tar .
rm -rf release-tar

echo "Tarball created: $TAR_NAME"
echo "Size: $(du -h "$TAR_NAME" | cut -f1)"

# Create AppImage if appimagetool is available or can be downloaded
APPIMAGE_NAME="${APP_NAME}-${VERSION}-${TARGET}.AppImage"
echo ""
echo "Creating AppImage: $APPIMAGE_NAME"

# Download appimagetool if needed
if [[ ! -f appimagetool ]]; then
    ARCH=$(uname -m)
    echo "Downloading appimagetool for $ARCH..."
    wget -q "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-${ARCH}.AppImage" -O appimagetool || {
        echo "Warning: Could not download appimagetool, skipping AppImage creation"
        echo ""
        echo "=== Packaging Complete ==="
        exit 0
    }
    chmod +x appimagetool
fi

# Extract appimagetool to avoid FUSE issues
./appimagetool --appimage-extract > /dev/null 2>&1
mv squashfs-root appimagetool-extracted

# Create AppDir structure
mkdir -p AppDir/usr/bin
mkdir -p AppDir/usr/share/applications
mkdir -p AppDir/usr/share/metainfo

# Copy binary
cp "$BINARY" AppDir/usr/bin/
chmod +x "AppDir/usr/bin/$APP_NAME"

# Copy desktop file and metainfo
cp "assets/linux/${APP_ID}.desktop" AppDir/usr/share/applications/
cp "assets/linux/${APP_ID}.metainfo.xml" AppDir/usr/share/metainfo/

# Copy all icon sizes
for SIZE in 16 24 32 48 64 96 128 256 512; do
    mkdir -p "AppDir/usr/share/icons/hicolor/${SIZE}x${SIZE}/apps"
    if [[ -f "assets/linux/icons/hicolor/${SIZE}x${SIZE}/apps/${APP_ID}.png" ]]; then
        cp "assets/linux/icons/hicolor/${SIZE}x${SIZE}/apps/${APP_ID}.png" \
           "AppDir/usr/share/icons/hicolor/${SIZE}x${SIZE}/apps/"
    fi
done

# Create symlinks in AppDir root (required by AppImage)
ln -sf "usr/bin/$APP_NAME" AppDir/AppRun
ln -sf "usr/share/applications/${APP_ID}.desktop" "AppDir/${APP_ID}.desktop"
ln -sf "usr/share/icons/hicolor/256x256/apps/${APP_ID}.png" "AppDir/${APP_ID}.png"

# Build AppImage
ARCH=$(uname -m) ./appimagetool-extracted/AppRun AppDir "$APPIMAGE_NAME"

# Cleanup
rm -rf AppDir appimagetool appimagetool-extracted

if [[ -f "$APPIMAGE_NAME" ]]; then
    echo "AppImage created: $APPIMAGE_NAME"
    echo "Size: $(du -h "$APPIMAGE_NAME" | cut -f1)"
else
    echo "Warning: AppImage creation may have failed"
fi

echo ""
echo "=== Packaging Complete ==="
