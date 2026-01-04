#!/bin/bash
# Sign app bundle locally for testing (without notarization)
# Usage: ./scripts/macos/sign-local.sh [path-to-app]
#
# Prerequisites:
#   1. Build the app: cargo build --release
#   2. Create bundle: ./scripts/macos/create-bundle.sh (or manually)
#   3. Have Developer ID Application certificate installed

set -e
APP_NAME="trial-submission-studio"
BUNDLE_NAME="Trial Submission Studio"
APP_PATH="${1:-${BUNDLE_NAME}.app}"

if [ ! -d "$APP_PATH" ]; then
    echo "ERROR: App bundle not found at: $APP_PATH"
    echo "First build and create the bundle, then run this script."
    exit 1
fi

echo "Finding code signing identity..."
IDENTITY=$(security find-identity -v -p codesigning | grep "Developer ID Application" | head -1 | awk -F'"' '{print $2}')

if [ -z "$IDENTITY" ]; then
    echo "ERROR: No Developer ID Application certificate found"
    echo "Run: security find-identity -v -p codesigning"
    exit 1
fi

echo "Using identity: $IDENTITY"

echo "Signing app bundle..."
codesign --force --options runtime \
    --entitlements packaging/macos/entitlements.plist \
    --sign "$IDENTITY" \
    --timestamp \
    "$APP_PATH"

echo "App signed. Run ./scripts/macos/verify-bundle.sh \"$APP_PATH\" to verify."
