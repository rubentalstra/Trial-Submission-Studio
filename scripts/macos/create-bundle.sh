#!/bin/bash
# Create macOS app bundle from compiled binary
# Usage: ./scripts/macos/create-bundle.sh [target]
# Example: ./scripts/macos/create-bundle.sh aarch64-apple-darwin

set -e
APP_NAME="trial-submission-studio"
BUNDLE_NAME="Trial Submission Studio"
TARGET="${1:-$(rustc -vV | grep host | cut -d' ' -f2)}"

# Get version from Cargo.toml
VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)

# Extract marketing version (strip -alpha.X, -beta.X, -rc.X suffixes)
MARKETING_VERSION=$(echo "${VERSION}" | sed -E 's/-(alpha|beta|rc)\.[0-9]+$//')

# Generate JetBrains-style build version: TSS-LOCAL.{commit_count}
COMMIT_COUNT=$(git rev-list --count HEAD 2>/dev/null || echo "0")
BUILD_VERSION="TSS-LOCAL.${COMMIT_COUNT}"

echo "Creating app bundle for target: $TARGET"
echo "Marketing Version: $MARKETING_VERSION"
echo "Build Version: $BUILD_VERSION"

APP_DIR="${BUNDLE_NAME}.app"
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

# Copy binary
BINARY="target/${TARGET}/release/${APP_NAME}"
if [ ! -f "$BINARY" ]; then
    BINARY="target/release/${APP_NAME}"
fi
if [ ! -f "$BINARY" ]; then
    echo "ERROR: Binary not found. Run 'cargo build --release' first."
    exit 1
fi
cp "$BINARY" "$APP_DIR/Contents/MacOS/"

# Process Info.plist template
sed -e "s/\${BUILD_VERSION}/${BUILD_VERSION}/g" \
    -e "s/\${MARKETING_VERSION}/${MARKETING_VERSION}/g" \
    packaging/macos/Info.plist > "$APP_DIR/Contents/Info.plist"

# Copy static files
cp packaging/macos/AppIcon.icns "$APP_DIR/Contents/Resources/"
cp packaging/macos/PkgInfo "$APP_DIR/Contents/"

# Validate plists
plutil -lint "$APP_DIR/Contents/Info.plist" || { echo "ERROR: Invalid Info.plist"; exit 1; }

echo ""
echo "App bundle created: $APP_DIR"
echo "  CFBundleVersion: $BUILD_VERSION"
echo "  CFBundleShortVersionString: $MARKETING_VERSION"
echo ""
echo "Next: Run ./scripts/macos/sign-local.sh to sign the app"
