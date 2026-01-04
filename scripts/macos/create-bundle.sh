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

echo "Creating app bundle for target: $TARGET"
echo "Version: $VERSION"

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

# Create Info.plist with version
sed "s/\${VERSION}/${VERSION}/g" packaging/macos/Info.plist > "$APP_DIR/Contents/Info.plist"

# Copy icon
cp packaging/macos/AppIcon.icns "$APP_DIR/Contents/Resources/"

# Create PkgInfo
echo -n "APPL????" > "$APP_DIR/Contents/PkgInfo"

echo "App bundle created: $APP_DIR"
echo "Next: Run ./scripts/macos/sign-local.sh to sign the app"
