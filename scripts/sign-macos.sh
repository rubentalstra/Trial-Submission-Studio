#!/bin/bash
# Sign and notarize macOS app bundle
# Usage: ./scripts/sign-macos.sh [--local]
#
# Modes:
#   Default (CI): Uses environment variables for certificates and notarization
#   --local: Uses locally installed Developer ID certificate (no notarization)
#
# Environment variables (CI mode):
#   APPLE_DEVELOPER_CERTIFICATE_P12_BASE64 - Base64-encoded P12 certificate
#   APPLE_DEVELOPER_CERTIFICATE_PASSWORD - Certificate password
#   APPLE_CODESIGN_IDENTITY - Signing identity name
#   APPLE_NOTARIZATION_APPLE_ID - Apple ID for notarization
#   APPLE_NOTARIZATION_APP_PASSWORD - App-specific password
#   APPLE_DEVELOPER_TEAM_ID - Team ID
#   CI_KEYCHAIN_PASSWORD - Keychain password (CI only)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

BUNDLE_NAME="Trial Submission Studio"
APP_PATH="${BUNDLE_NAME}.app"
LOCAL_MODE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --local)
            LOCAL_MODE=true
            shift
            ;;
        *)
            APP_PATH="$1"
            shift
            ;;
    esac
done

cd "$PROJECT_ROOT"

if [[ ! -d "$APP_PATH" ]]; then
    echo "Error: App bundle not found at: $APP_PATH"
    echo "Run ./scripts/package-macos.sh first."
    exit 1
fi

echo "=== Signing macOS App Bundle ==="
echo "App: $APP_PATH"
echo "Mode: $(if $LOCAL_MODE; then echo "Local"; else echo "CI"; fi)"

if $LOCAL_MODE; then
    # Local mode: Find Developer ID certificate automatically
    echo ""
    echo "Finding code signing identity..."
    IDENTITY=$(security find-identity -v -p codesigning | grep "Developer ID Application" | head -1 | awk -F'"' '{print $2}')

    if [[ -z "$IDENTITY" ]]; then
        echo "Error: No Developer ID Application certificate found"
        echo "Run: security find-identity -v -p codesigning"
        exit 1
    fi

    echo "Using identity: $IDENTITY"

    # Sign with local certificate
    echo ""
    echo "Signing app bundle..."

    MACOS_DIR="$APP_PATH/Contents/MacOS"
    ENTITLEMENTS="assets/macos/TrialSubmissionStudio.app/Contents/entitlements.plist"

    # Sign helper binaries first (everything except the main binary)
    for binary in "$MACOS_DIR"/*; do
        if [[ "$(basename "$binary")" != "trial-submission-studio" ]]; then
            echo "Signing helper: $binary"
            codesign --force --options runtime \
                --entitlements "$ENTITLEMENTS" \
                --sign "$IDENTITY" \
                --timestamp \
                "$binary"
        fi
    done

    # Sign the main binary
    echo "Signing main binary: $MACOS_DIR/trial-submission-studio"
    codesign --force --options runtime \
        --entitlements "$ENTITLEMENTS" \
        --sign "$IDENTITY" \
        --timestamp \
        "$MACOS_DIR/trial-submission-studio"

    # Sign the bundle itself
    echo "Signing bundle: $APP_PATH"
    codesign --force --options runtime \
        --entitlements "$ENTITLEMENTS" \
        --sign "$IDENTITY" \
        --timestamp \
        "$APP_PATH"

    echo ""
    echo "=== Local Signing Complete ==="
    echo "Note: App is signed but NOT notarized (local mode)"
    echo "Run ./scripts/macos/verify-bundle.sh \"$APP_PATH\" to verify"
else
    # CI mode: Import certificate and perform full signing + notarization

    # Validate required environment variables
    : "${APPLE_DEVELOPER_CERTIFICATE_P12_BASE64:?Missing APPLE_DEVELOPER_CERTIFICATE_P12_BASE64}"
    : "${APPLE_DEVELOPER_CERTIFICATE_PASSWORD:?Missing APPLE_DEVELOPER_CERTIFICATE_PASSWORD}"
    : "${APPLE_CODESIGN_IDENTITY:?Missing APPLE_CODESIGN_IDENTITY}"
    : "${APPLE_NOTARIZATION_APPLE_ID:?Missing APPLE_NOTARIZATION_APPLE_ID}"
    : "${APPLE_NOTARIZATION_APP_PASSWORD:?Missing APPLE_NOTARIZATION_APP_PASSWORD}"
    : "${APPLE_DEVELOPER_TEAM_ID:?Missing APPLE_DEVELOPER_TEAM_ID}"
    : "${CI_KEYCHAIN_PASSWORD:?Missing CI_KEYCHAIN_PASSWORD}"

    # Create and configure keychain
    echo ""
    echo "Setting up CI keychain..."
    security create-keychain -p "$CI_KEYCHAIN_PASSWORD" build.keychain
    security list-keychains -d user -s build.keychain login.keychain
    security default-keychain -s build.keychain
    security unlock-keychain -p "$CI_KEYCHAIN_PASSWORD" build.keychain
    security set-keychain-settings -t 3600 -u build.keychain

    # Import certificate
    echo "Importing certificate..."
    echo "$APPLE_DEVELOPER_CERTIFICATE_P12_BASE64" | base64 --decode > certificate.p12
    security import certificate.p12 -k build.keychain \
        -P "$APPLE_DEVELOPER_CERTIFICATE_PASSWORD" \
        -T /usr/bin/codesign -T /usr/bin/security
    security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$CI_KEYCHAIN_PASSWORD" build.keychain
    rm certificate.p12

    # Sign app bundle
    echo ""
    echo "Signing app bundle..."

    MACOS_DIR="$APP_PATH/Contents/MacOS"
    ENTITLEMENTS="assets/macos/TrialSubmissionStudio.app/Contents/entitlements.plist"

    # Sign helper binaries first (everything except the main binary)
    for binary in "$MACOS_DIR"/*; do
        if [[ "$(basename "$binary")" != "trial-submission-studio" ]]; then
            echo "Signing helper: $binary"
            codesign --force --options runtime \
                --entitlements "$ENTITLEMENTS" \
                --sign "$APPLE_CODESIGN_IDENTITY" \
                --timestamp \
                "$binary"
        fi
    done

    # Sign the main binary
    echo "Signing main binary: $MACOS_DIR/trial-submission-studio"
    codesign --force --options runtime \
        --entitlements "$ENTITLEMENTS" \
        --sign "$APPLE_CODESIGN_IDENTITY" \
        --timestamp \
        "$MACOS_DIR/trial-submission-studio"

    # Sign the bundle itself
    echo "Signing bundle: $APP_PATH"
    codesign --force --options runtime \
        --entitlements "$ENTITLEMENTS" \
        --sign "$APPLE_CODESIGN_IDENTITY" \
        --timestamp \
        "$APP_PATH"

    # Verify signature
    echo ""
    echo "Verifying signature..."
    codesign --verify --deep --strict --verbose=2 "$APP_PATH"

    # Notarize
    echo ""
    echo "Submitting for notarization..."
    ditto -c -k --keepParent "$APP_PATH" notarize.zip
    xcrun notarytool submit notarize.zip \
        --apple-id "$APPLE_NOTARIZATION_APPLE_ID" \
        --password "$APPLE_NOTARIZATION_APP_PASSWORD" \
        --team-id "$APPLE_DEVELOPER_TEAM_ID" \
        --wait --timeout 30m
    rm notarize.zip

    # Staple ticket
    echo ""
    echo "Stapling notarization ticket..."
    xcrun stapler staple "$APP_PATH"

    # Verify notarization
    echo ""
    echo "Verifying notarization..."
    xcrun stapler validate "$APP_PATH"
    spctl --assess --type execute --verbose=2 "$APP_PATH"

    # Cleanup keychain
    echo ""
    echo "Cleaning up keychain..."
    security delete-keychain build.keychain 2>/dev/null || true

    echo ""
    echo "=== CI Signing & Notarization Complete ==="
fi
