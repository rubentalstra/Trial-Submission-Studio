# macOS Gatekeeper Issues

This guide helps resolve common issues when opening Trial Submission Studio on macOS.

## "Trial Submission Studio is damaged and can't be opened"

This error typically means the app is not properly signed or notarized by Apple.

### For Users: Quick Fix

If you downloaded from our official GitHub releases and see this error:

1. Open **System Settings** → **Privacy & Security**
2. Scroll down to the **Security** section
3. Look for a message about "Trial Submission Studio" being blocked
4. Click **Open Anyway**
5. Confirm in the dialog that appears

### For Developers: Root Causes

This error can occur when:

1. **App is not code signed** - No Developer ID certificate was used
2. **App is not notarized** - Apple's notary service didn't approve it
3. **Entitlements are too permissive** - Certain entitlements can cause rejection
4. **GitHub secrets not configured** - CI skipped signing due to missing secrets

## "Apple cannot check it for malicious software"

This warning appears for apps that are signed but not notarized.

### Workaround

1. **Right-click** (or Control+click) the app
2. Select **Open** from the context menu
3. Click **Open** in the dialog

> **Note:** On macOS Sequoia (15.0+), Control+click bypass no longer works. You must use System Settings → Privacy &
> Security → Open Anyway.

## Verifying App Signature

To check if an app is properly signed:

```bash
# Check code signature
codesign --verify --deep --strict --verbose=2 "Trial Submission Studio.app"

# Check notarization
xcrun stapler validate "Trial Submission Studio.app"

# Check Gatekeeper assessment
spctl --assess --type execute --verbose=2 "Trial Submission Studio.app"
```

Expected output for a properly signed and notarized app:

- `valid on disk` from codesign
- `The validate action worked!` from stapler
- `accepted` from spctl

## Removing Quarantine Attribute

If you're a developer testing the app, you can remove the quarantine attribute:

```bash
xattr -d com.apple.quarantine "Trial Submission Studio.app"
```

> **Warning:** Only do this for apps you trust. This bypasses macOS security.

## macOS Sequoia (15.0+) Changes

Apple significantly tightened Gatekeeper in macOS Sequoia:

- **Control+click bypass removed** - The old workaround no longer works
- **New bypass path**: System Settings → Privacy & Security → Open Anyway
- **Admin password required** - You'll need to authenticate twice
- **`spctl --master-disable` removed** - Can't globally disable Gatekeeper via terminal

This makes proper code signing and notarization more important than ever.

## Reporting Issues

If you downloaded from our official releases and still have issues:

1. Check the [GitHub Releases](https://github.com/rubentalstra/Trial-Submission-Studio/releases) page
2. Ensure you downloaded the `.dmg` file (not the `.zip`)
3. Report issues at [GitHub Issues](https://github.com/rubentalstra/Trial-Submission-Studio/issues)

Include:

- macOS version (`sw_vers`)
- Where you downloaded the app from
- The exact error message
- Output of `codesign --verify --verbose=2` (if possible)
