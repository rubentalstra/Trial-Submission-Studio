#!/bin/bash
# Verify app bundle structure and code signature
# Usage: ./scripts/macos/verify-bundle.sh [path-to-app]

set -e
APP_PATH="${1:-Trial Submission Studio.app}"

echo "=== Checking bundle structure ==="
test -f "$APP_PATH/Contents/MacOS/trial-submission-studio" || { echo "FAIL: Binary missing"; exit 1; }
test -f "$APP_PATH/Contents/Info.plist" || { echo "FAIL: Info.plist missing"; exit 1; }
test -f "$APP_PATH/Contents/Resources/AppIcon.icns" || { echo "FAIL: Icon missing"; exit 1; }
test -f "$APP_PATH/Contents/Resources/version.plist" || { echo "FAIL: version.plist missing"; exit 1; }
test -f "$APP_PATH/Contents/PkgInfo" || { echo "FAIL: PkgInfo missing"; exit 1; }
echo "Bundle structure OK"

echo ""
echo "=== Checking Info.plist ==="
plutil -lint "$APP_PATH/Contents/Info.plist" || { echo "FAIL: Invalid Info.plist"; exit 1; }
BUNDLE_VERSION=$(plutil -extract CFBundleVersion raw "$APP_PATH/Contents/Info.plist")
SHORT_VERSION=$(plutil -extract CFBundleShortVersionString raw "$APP_PATH/Contents/Info.plist")
echo "CFBundleVersion: $BUNDLE_VERSION"
echo "CFBundleShortVersionString: $SHORT_VERSION"

echo ""
echo "=== Checking version.plist ==="
plutil -lint "$APP_PATH/Contents/Resources/version.plist" || { echo "FAIL: Invalid version.plist"; exit 1; }
GIT_SHA=$(plutil -extract GitCommitSHA raw "$APP_PATH/Contents/Resources/version.plist")
echo "GitCommitSHA: $GIT_SHA"

echo ""
echo "=== Checking code signature ==="
if codesign --verify --deep --strict --verbose=2 "$APP_PATH" 2>&1; then
    echo "Signature valid"
else
    echo "WARN: App is not signed or signature is invalid"
    echo "      Run ./scripts/macos/sign-local.sh to sign the app"
fi

echo ""
echo "=== Checking hardened runtime ==="
if codesign -d --verbose=4 "$APP_PATH" 2>&1 | grep -q "runtime"; then
    echo "Hardened runtime enabled"
else
    echo "WARN: Hardened runtime NOT enabled"
fi

echo ""
echo "=== Checking notarization staple ==="
if xcrun stapler validate "$APP_PATH" 2>/dev/null; then
    echo "Notarization ticket stapled"
else
    echo "WARN: No notarization ticket (expected for local builds)"
fi

echo ""
echo "=== Gatekeeper assessment ==="
spctl --assess --type execute --verbose=2 "$APP_PATH" 2>&1 || echo "WARN: Gatekeeper may block (not notarized)"

echo ""
echo "All checks complete!"
