#!/bin/bash
# Package macOS app bundle and DMG
# Usage: ./scripts/package-macos.sh
#
# Environment variables:
#   TARGET - Rust target triple (default: host architecture)
#   VERSION - Version string (default: from Cargo.toml)
#   TSS_BUILD_NUMBER - Build number (default: LOCAL.{commits})
#   SKIP_DMG - Set to 1 to skip DMG creation (useful when DMG needs the signed app bundle)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

APP_NAME="trial-submission-studio"
BUNDLE_NAME="Trial Submission Studio"

# Determine target architecture
TARGET="${TARGET:-$(rustc -vV | grep host | cut -d' ' -f2)}"

# Get version from environment or Cargo.toml
if [[ -z "${VERSION:-}" ]]; then
    VERSION="v$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | head -1 | cut -d'"' -f2)"
fi

# Strip 'v' prefix if present for internal use
VERSION_NUM="${VERSION#v}"

# Extract marketing version (strip -alpha.X, -beta.X, -rc.X suffixes)
MARKETING_VERSION=$(echo "${VERSION_NUM}" | sed -E 's/-(alpha|beta|rc)\.[0-9]+$//')

# Generate build version
if [[ -z "${TSS_BUILD_NUMBER:-}" ]]; then
    COMMIT_COUNT=$(git rev-list --count HEAD 2>/dev/null || echo "0")
    BUILD_VERSION="TSS-LOCAL.${COMMIT_COUNT}"
else
    BUILD_VERSION="$TSS_BUILD_NUMBER"
fi

echo "=== Packaging macOS App Bundle ==="
echo "Target: $TARGET"
echo "Version: $VERSION"
echo "Marketing Version: $MARKETING_VERSION"
echo "Build Version: $BUILD_VERSION"

cd "$PROJECT_ROOT"

# Find binary
BINARY="target/$TARGET/release/$APP_NAME"
if [[ ! -f "$BINARY" ]]; then
    BINARY="target/release/$APP_NAME"
fi
if [[ ! -f "$BINARY" ]]; then
    echo "Error: Binary not found. Run ./scripts/build-macos.sh first."
    exit 1
fi

# Find helper binary
HELPER="target/$TARGET/release/tss-updater-helper"
if [[ ! -f "$HELPER" ]]; then
    HELPER="target/release/tss-updater-helper"
fi
if [[ ! -f "$HELPER" ]]; then
    echo "Error: Helper binary not found. Run ./scripts/build-macos.sh first."
    exit 1
fi

# Create app bundle structure
APP_DIR="${BUNDLE_NAME}.app"
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"
mkdir -p "$APP_DIR/Contents/Helpers/tss-updater-helper.app/Contents/MacOS"

# Copy main binary
cp "$BINARY" "$APP_DIR/Contents/MacOS/"

# Create nested helper app bundle
cp "$HELPER" "$APP_DIR/Contents/Helpers/tss-updater-helper.app/Contents/MacOS/"

# Process helper Info.plist template
sed -e "s/\${BUILD_VERSION}/${BUILD_VERSION}/g" \
    -e "s/\${MARKETING_VERSION}/${MARKETING_VERSION}/g" \
    "assets/macos/tss-updater-helper.app/Contents/Info.plist" \
    > "$APP_DIR/Contents/Helpers/tss-updater-helper.app/Contents/Info.plist"

# Process Info.plist template
sed -e "s/\${BUILD_VERSION}/${BUILD_VERSION}/g" \
    -e "s/\${MARKETING_VERSION}/${MARKETING_VERSION}/g" \
    "assets/macos/TrialSubmissionStudio.app/Contents/Info.plist" > "$APP_DIR/Contents/Info.plist"

# Copy static files
cp "assets/macos/TrialSubmissionStudio.app/Contents/Resources/AppIcon.icns" "$APP_DIR/Contents/Resources/"
cp "assets/macos/TrialSubmissionStudio.app/Contents/PkgInfo" "$APP_DIR/Contents/"

# Validate plist
plutil -lint "$APP_DIR/Contents/Info.plist" || { echo "Error: Invalid Info.plist"; exit 1; }

echo ""
echo "App bundle created: $APP_DIR"

# Create DMG if requested and create-dmg is available
if [[ "${SKIP_DMG:-0}" != "1" ]]; then
    if command -v create-dmg &> /dev/null; then
        DMG_NAME="${APP_NAME}-${VERSION}-${TARGET}.dmg"

        echo ""
        echo "Creating DMG: $DMG_NAME"

        # Remove existing DMG
        rm -f "$DMG_NAME"

        create-dmg \
            --volname "${BUNDLE_NAME}" \
            --volicon "assets/macos/TrialSubmissionStudio.app/Contents/Resources/AppIcon.icns" \
            --window-pos 200 120 \
            --window-size 600 400 \
            --icon-size 100 \
            --icon "${BUNDLE_NAME}.app" 150 190 \
            --hide-extension "${BUNDLE_NAME}.app" \
            --app-drop-link 450 190 \
            "$DMG_NAME" \
            "$APP_DIR" || true

        if [[ -f "$DMG_NAME" ]]; then
            echo "DMG created: $DMG_NAME"
            echo "Size: $(du -h "$DMG_NAME" | cut -f1)"
        else
            echo "Warning: DMG creation failed (create-dmg may have returned non-zero)"
        fi
    else
        echo ""
        echo "Note: create-dmg not found, skipping DMG creation"
        echo "Install with: brew install create-dmg"
    fi
else
    echo ""
    echo "Skipping DMG creation (SKIP_DMG=${SKIP_DMG})"
fi

echo ""
echo "=== Packaging Complete ==="
echo "Next: Run ./scripts/sign-macos.sh to sign the app"
