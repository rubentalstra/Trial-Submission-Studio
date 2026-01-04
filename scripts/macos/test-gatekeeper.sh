#!/bin/bash
# Simulate Gatekeeper by adding quarantine attribute
# Usage: ./scripts/macos/test-gatekeeper.sh [path-to-app]

set -e
APP_PATH="${1:-Trial Submission Studio.app}"

echo "Adding quarantine attribute to simulate download..."
xattr -w com.apple.quarantine "0183;$(printf '%x' $(date +%s));Safari;$(uuidgen)" "$APP_PATH"

echo "Quarantine added. Now try to open the app:"
echo "  open \"$APP_PATH\""
echo ""
echo "If properly signed and notarized, it should open without warnings."
echo "If you see 'damaged' error, the app is not properly signed/notarized."
