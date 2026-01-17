#!/bin/bash
# Diagnose macOS app bundle structure and code signing
# Usage: ./scripts/macos/diagnose-bundle.sh "Path/To/App.app"
#
# This script performs comprehensive validation and reports exactly what's wrong
# with an app bundle's structure and code signing.

set -euo pipefail

APP_PATH="${1:-Trial Submission Studio.app}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

pass() { echo -e "${GREEN}✓${NC} $1"; }
fail() { echo -e "${RED}✗${NC} $1"; FAILURES=$((FAILURES + 1)); }
warn() { echo -e "${YELLOW}!${NC} $1"; }
info() { echo -e "${BLUE}→${NC} $1"; }
header() { echo -e "\n${BLUE}=== $1 ===${NC}"; }

FAILURES=0

echo "=============================================="
echo "  macOS App Bundle Diagnostic Tool"
echo "=============================================="
echo ""
info "Diagnosing: $APP_PATH"
info "Date: $(date)"

# Validate input
if [[ ! -d "$APP_PATH" ]]; then
    fail "App bundle not found: $APP_PATH"
    exit 1
fi

header "1. BUNDLE STRUCTURE"

# Expected structure
MAIN_BINARY="$APP_PATH/Contents/MacOS/trial-submission-studio"
MAIN_PLIST="$APP_PATH/Contents/Info.plist"
RESOURCES="$APP_PATH/Contents/Resources"
PKGINFO="$APP_PATH/Contents/PkgInfo"
HELPER_BUNDLE="$APP_PATH/Contents/Helpers/tss-updater-helper.app"
HELPER_BINARY="$HELPER_BUNDLE/Contents/MacOS/tss-updater-helper"
HELPER_PLIST="$HELPER_BUNDLE/Contents/Info.plist"

# Check main app structure
info "Main app components:"
[[ -f "$MAIN_BINARY" ]] && pass "Main binary exists" || fail "Main binary MISSING: $MAIN_BINARY"
[[ -f "$MAIN_PLIST" ]] && pass "Main Info.plist exists" || fail "Main Info.plist MISSING"
[[ -d "$RESOURCES" ]] && pass "Resources directory exists" || fail "Resources directory MISSING"
[[ -f "$PKGINFO" ]] && pass "PkgInfo exists" || fail "PkgInfo MISSING"

# Check for icon
if [[ -f "$RESOURCES/AppIcon.icns" ]]; then
    pass "AppIcon.icns exists"
else
    warn "AppIcon.icns not found in Resources"
fi

# Check helper bundle structure
info "Helper app components:"
[[ -d "$HELPER_BUNDLE" ]] && pass "Helper bundle exists" || fail "Helper bundle MISSING: $HELPER_BUNDLE"
[[ -f "$HELPER_BINARY" ]] && pass "Helper binary exists" || fail "Helper binary MISSING: $HELPER_BINARY"
[[ -f "$HELPER_PLIST" ]] && pass "Helper Info.plist exists" || fail "Helper Info.plist MISSING"

# Check permissions
info "Checking executable permissions:"
if [[ -f "$MAIN_BINARY" ]]; then
    if [[ -x "$MAIN_BINARY" ]]; then
        pass "Main binary is executable"
    else
        fail "Main binary NOT executable"
        ls -la "$MAIN_BINARY"
    fi
fi
if [[ -f "$HELPER_BINARY" ]]; then
    if [[ -x "$HELPER_BINARY" ]]; then
        pass "Helper binary is executable"
    else
        fail "Helper binary NOT executable"
        ls -la "$HELPER_BINARY"
    fi
fi

# Show actual structure
info "Actual bundle contents (first 40 entries):"
find "$APP_PATH" -type f | sed "s|$APP_PATH|  .|" | sort | head -40
TOTAL_FILES=$(find "$APP_PATH" -type f | wc -l | tr -d ' ')
info "Total files in bundle: $TOTAL_FILES"

header "2. INFO.PLIST VALIDATION"

# Main Info.plist
info "Main Info.plist:"
if [[ -f "$MAIN_PLIST" ]]; then
    if plutil -lint "$MAIN_PLIST" >/dev/null 2>&1; then
        pass "Main Info.plist is valid XML"
    else
        fail "Main Info.plist is INVALID"
        plutil -lint "$MAIN_PLIST" 2>&1 || true
    fi

    # Check for unsubstituted placeholders
    if grep -q '\${' "$MAIN_PLIST" 2>/dev/null; then
        fail "Main Info.plist has UNSUBSTITUTED placeholders:"
        grep '\${' "$MAIN_PLIST" | head -5
    else
        pass "Main Info.plist placeholders substituted"
    fi

    # Show key values
    info "  Bundle identifier: $(plutil -extract CFBundleIdentifier raw "$MAIN_PLIST" 2>/dev/null || echo 'N/A')"
    info "  Bundle version: $(plutil -extract CFBundleVersion raw "$MAIN_PLIST" 2>/dev/null || echo 'N/A')"
    info "  Short version: $(plutil -extract CFBundleShortVersionString raw "$MAIN_PLIST" 2>/dev/null || echo 'N/A')"
    info "  Executable name: $(plutil -extract CFBundleExecutable raw "$MAIN_PLIST" 2>/dev/null || echo 'N/A')"
fi

# Helper Info.plist
echo ""
info "Helper Info.plist:"
if [[ -f "$HELPER_PLIST" ]]; then
    if plutil -lint "$HELPER_PLIST" >/dev/null 2>&1; then
        pass "Helper Info.plist is valid XML"
    else
        fail "Helper Info.plist is INVALID"
        plutil -lint "$HELPER_PLIST" 2>&1 || true
    fi

    if grep -q '\${' "$HELPER_PLIST" 2>/dev/null; then
        fail "Helper Info.plist has UNSUBSTITUTED placeholders:"
        grep '\${' "$HELPER_PLIST" | head -5
    else
        pass "Helper Info.plist placeholders substituted"
    fi

    info "  Bundle identifier: $(plutil -extract CFBundleIdentifier raw "$HELPER_PLIST" 2>/dev/null || echo 'N/A')"
    info "  Bundle version: $(plutil -extract CFBundleVersion raw "$HELPER_PLIST" 2>/dev/null || echo 'N/A')"
    info "  Executable name: $(plutil -extract CFBundleExecutable raw "$HELPER_PLIST" 2>/dev/null || echo 'N/A')"
else
    warn "Helper Info.plist not found, skipping validation"
fi

header "3. SIGNATURE STATUS (per component)"

check_signature() {
    local path="$1"
    local name="$2"

    echo ""
    info "Checking: $name"
    info "Path: $path"

    if [[ ! -e "$path" ]]; then
        fail "$name does not exist"
        return
    fi

    # Get signature info
    echo "  Signature details:"
    if codesign -dvvv "$path" 2>&1 | grep -E "^(Executable|Identifier|Format|CodeDirectory|Signature|Authority|TeamIdentifier|Sealed|flags)" | sed 's/^/    /'; then
        pass "$name has signature info"
    else
        warn "$name may not be signed"
    fi

    # Quick verify
    echo "  Verification:"
    local verify_output
    if verify_output=$(codesign --verify --strict "$path" 2>&1); then
        pass "$name signature is VALID"
    else
        fail "$name signature is INVALID"
        echo "    $verify_output"
        # Get more details
        codesign --verify --strict --verbose=4 "$path" 2>&1 | sed 's/^/    /' || true
    fi

    # Check for hardened runtime (flags field shows "(runtime)" when enabled)
    local runtime_check
    runtime_check=$(codesign -d --verbose=4 "$path" 2>&1)
    if echo "$runtime_check" | grep -q "(runtime)"; then
        pass "$name has hardened runtime"
    else
        warn "$name does NOT have hardened runtime"
    fi
}

# Check components in order (inner to outer)
[[ -f "$HELPER_BINARY" ]] && check_signature "$HELPER_BINARY" "Helper binary"
[[ -d "$HELPER_BUNDLE" ]] && check_signature "$HELPER_BUNDLE" "Helper bundle"
[[ -f "$MAIN_BINARY" ]] && check_signature "$MAIN_BINARY" "Main binary"
check_signature "$APP_PATH" "Main bundle"

header "4. DEEP VERIFICATION"

info "Running: codesign --verify --deep --strict --verbose=4"
echo ""
if codesign --verify --deep --strict --verbose=4 "$APP_PATH" 2>&1; then
    pass "Deep verification PASSED"
else
    fail "Deep verification FAILED"
    echo ""
    info "Attempting to identify the failing component..."
    # Try to identify what specifically failed
    codesign --verify --deep --strict --verbose=4 "$APP_PATH" 2>&1 | grep -i "invalid\|error\|fail" | head -10 || true
fi

header "5. ENTITLEMENTS CHECK"

info "Main app entitlements:"
if codesign -d --entitlements - "$APP_PATH" 2>/dev/null | head -30; then
    pass "Main app has entitlements"
else
    warn "No entitlements found for main app"
fi

echo ""
info "Helper app entitlements:"
if [[ -d "$HELPER_BUNDLE" ]]; then
    if codesign -d --entitlements - "$HELPER_BUNDLE" 2>/dev/null | head -30; then
        pass "Helper app has entitlements"
    else
        warn "No entitlements found for helper app"
    fi
fi

header "6. GATEKEEPER ASSESSMENT"

info "Running: spctl --assess --type execute --verbose=4"
echo ""
if spctl --assess --type execute --verbose=4 "$APP_PATH" 2>&1; then
    pass "Gatekeeper assessment PASSED"
else
    warn "Gatekeeper assessment failed (may be expected for local/unsigned builds)"
    spctl --assess --type execute --verbose=4 "$APP_PATH" 2>&1 || true
fi

header "7. NOTARIZATION STATUS"

info "Checking for stapled notarization ticket..."
if xcrun stapler validate "$APP_PATH" 2>/dev/null; then
    pass "Notarization ticket is stapled"
else
    warn "No notarization ticket stapled (expected for local builds)"
fi

header "8. QUARANTINE ATTRIBUTE"

if xattr -l "$APP_PATH" 2>/dev/null | grep -q "com.apple.quarantine"; then
    warn "App has quarantine attribute set"
    xattr -p com.apple.quarantine "$APP_PATH" 2>/dev/null || true
else
    info "No quarantine attribute (app not downloaded or already cleared)"
fi

header "9. SUMMARY"

echo ""
if [[ $FAILURES -eq 0 ]]; then
    echo -e "${GREEN}=============================================="
    echo -e "  ALL CHECKS PASSED ($FAILURES failures)"
    echo -e "==============================================${NC}"
else
    echo -e "${RED}=============================================="
    echo -e "  $FAILURES CHECK(S) FAILED"
    echo -e "==============================================${NC}"
    echo ""
    info "Common issues and solutions:"
    echo "  - Bundle structure wrong → Check packaging script"
    echo "  - Info.plist has placeholders → Check template substitution"
    echo "  - Helper unsigned → Sign helper BEFORE main app"
    echo "  - Deep verification fails → Check signing order (inside-out)"
    echo "  - Gatekeeper fails → App needs notarization"
fi

exit $FAILURES
