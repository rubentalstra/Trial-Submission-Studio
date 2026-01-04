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

# Gather git metadata for version.plist
GIT_COMMIT_SHA=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
GIT_COMMIT_SHORT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
GIT_BRANCH=$(git branch --show-current 2>/dev/null || echo "unknown")
GIT_CLEAN=$(git diff --quiet 2>/dev/null && echo "true" || echo "false")

# Gather build environment metadata
BUILD_DATE=$(date -u +%Y-%m-%dT%H:%M:%SZ)
BUILD_TIMESTAMP=$(date +%s)
RUST_VERSION=$(rustc --version 2>/dev/null | cut -d' ' -f2 || echo "unknown")

echo "Creating app bundle for target: $TARGET"
echo "Marketing Version: $MARKETING_VERSION"
echo "Build Version: $BUILD_VERSION"
echo "Git SHA: $GIT_COMMIT_SHORT"

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

# Process version.plist template (enterprise build metadata)
# Note: Using | as delimiter for GIT_BRANCH since branch names can contain /
sed -e "s/\${MARKETING_VERSION}/${MARKETING_VERSION}/g" \
    -e "s/\${FULL_VERSION}/${VERSION}/g" \
    -e "s/\${BUILD_VERSION}/${BUILD_VERSION}/g" \
    -e "s/\${GIT_COMMIT_SHA}/${GIT_COMMIT_SHA}/g" \
    -e "s/\${GIT_COMMIT_SHORT}/${GIT_COMMIT_SHORT}/g" \
    -e "s|\${GIT_BRANCH}|${GIT_BRANCH}|g" \
    -e "s/\${GIT_COMMIT_COUNT}/${COMMIT_COUNT}/g" \
    -e "s/\${GIT_CLEAN}/${GIT_CLEAN}/g" \
    -e "s/\${BUILD_DATE}/${BUILD_DATE}/g" \
    -e "s/\${BUILD_TIMESTAMP}/${BUILD_TIMESTAMP}/g" \
    -e "s|\${TARGET_ARCH}|${TARGET}|g" \
    -e "s/\${RUST_VERSION}/${RUST_VERSION}/g" \
    -e "s/\${BUILD_TYPE}/release/g" \
    -e "s/\${CI_PROVIDER}/local/g" \
    -e "s/\${CI_RUN_NUMBER}/0/g" \
    -e "s/\${CI_RUN_ID}/local/g" \
    packaging/macos/version.plist > "$APP_DIR/Contents/Resources/version.plist"

# Copy static files
cp packaging/macos/AppIcon.icns "$APP_DIR/Contents/Resources/"
cp packaging/macos/PkgInfo "$APP_DIR/Contents/"

# Validate plists
plutil -lint "$APP_DIR/Contents/Info.plist" || { echo "ERROR: Invalid Info.plist"; exit 1; }
plutil -lint "$APP_DIR/Contents/Resources/version.plist" || { echo "ERROR: Invalid version.plist"; exit 1; }

echo ""
echo "App bundle created: $APP_DIR"
echo "  CFBundleVersion: $BUILD_VERSION"
echo "  CFBundleShortVersionString: $MARKETING_VERSION"
echo "  GitCommitSHA: $GIT_COMMIT_SHORT"
echo ""
echo "Next: Run ./scripts/macos/sign-local.sh to sign the app"
