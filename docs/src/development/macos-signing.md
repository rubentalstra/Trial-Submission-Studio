# macOS Code Signing Setup

This guide explains how to set up Apple Developer certificates for signing and notarizing Trial Submission Studio
releases.

## Prerequisites

- Active [Apple Developer Program](https://developer.apple.com/programs/) membership ($99/year)
- macOS with Xcode Command Line Tools installed
- Access to the GitHub repository settings (for adding secrets)

## Step 1: Create Developer ID Application Certificate

### 1.1 Request Certificate from Apple

1. Open **Keychain Access** (Applications → Utilities → Keychain Access)
2. Go to **Keychain Access → Certificate Assistant → Request a Certificate From a Certificate Authority**
3. Fill in:
    - **Email Address**: Your Apple ID email
    - **Common Name**: Your name
    - **Request is**: Saved to disk
4. Save the `.certSigningRequest` file

### 1.2 Create Certificate in Apple Developer Portal

1. Go to [Apple Developer Certificates](https://developer.apple.com/account/resources/certificates/list)
2. Click **+** to create a new certificate
3. Select **Developer ID Application** (NOT "Developer ID Installer")
4. Upload your `.certSigningRequest` file
5. Download the generated `.cer` file
6. Double-click the `.cer` file to install it in Keychain Access

### 1.3 Verify Certificate Installation

Run this command to verify the certificate is installed:

```bash
security find-identity -v -p codesigning
```

You should see output like:

```
1) ABCDEF1234567890... "Developer ID Application: Your Name (TEAM_ID)"
```

## Step 2: Export Certificate for GitHub Actions

### 2.1 Export as .p12

1. Open **Keychain Access**
2. Find your certificate: "Developer ID Application: Your Name"
3. Right-click → **Export**
4. Choose **.p12** format
5. Set a strong password (you'll need this later)
6. Save the file

### 2.2 Convert to Base64

```bash
base64 -i YourCertificate.p12 | pbcopy
```

This copies the base64-encoded certificate to your clipboard.

## Step 3: Create App-Specific Password

Apple requires an app-specific password for notarization (not your regular Apple ID password).

1. Go to [Apple ID Account](https://appleid.apple.com/account/manage)
2. Sign in with your Apple ID
3. Navigate to **App-Specific Passwords**
4. Click **Generate an app-specific password**
5. Label: "GitHub Actions Notarization"
6. Copy the generated password (format: `xxxx-xxxx-xxxx-xxxx`)

## Step 4: Find Your Team ID

1. Go to [Apple Developer Account](https://developer.apple.com/account)
2. Click **Membership** in the left sidebar
3. Copy your **Team ID** (10-character alphanumeric string)

## Step 5: Configure GitHub Secrets

Go to your repository's **Settings → Secrets and variables → Actions** and add these 7 secrets:

| Secret Name                              | Description                          | How to Get                                        |
|------------------------------------------|--------------------------------------|---------------------------------------------------|
| `APPLE_DEVELOPER_CERTIFICATE_P12_BASE64` | Base64-encoded .p12 certificate      | Step 2.2 output                                   |
| `APPLE_DEVELOPER_CERTIFICATE_PASSWORD`   | Password you set when exporting .p12 | Step 2.1                                          |
| `APPLE_CODESIGN_IDENTITY`                | Full certificate name                | `security find-identity -v -p codesigning` output |
| `APPLE_NOTARIZATION_APPLE_ID`            | Your Apple ID email                  | Your Apple Developer email                        |
| `APPLE_NOTARIZATION_APP_PASSWORD`        | App-specific password                | Step 3 output                                     |
| `APPLE_DEVELOPER_TEAM_ID`                | 10-character Team ID                 | Step 4                                            |
| `CI_KEYCHAIN_PASSWORD`                   | Random secure password               | Generate any secure string                        |

### Example Values

```
APPLE_CODESIGN_IDENTITY: Developer ID Application: Ruben Talstra (ABCD1234EF)
APPLE_DEVELOPER_TEAM_ID: ABCD1234EF
APPLE_NOTARIZATION_APPLE_ID: your.email@example.com
```

## Local Development

### Create App Bundle

```bash
cargo build --release
./scripts/macos/create-bundle.sh
```

### Sign Locally (for testing)

```bash
./scripts/macos/sign-local.sh
```

### Verify Bundle

```bash
./scripts/macos/verify-bundle.sh
```

### Test Gatekeeper

```bash
./scripts/macos/test-gatekeeper.sh
open "Trial Submission Studio.app"
```

## Troubleshooting

### "No Developer ID Application certificate found"

Ensure the certificate is in your login keychain and not expired:

```bash
security find-identity -v -p codesigning
```

### "The signature is invalid"

Re-sign with the `--force` flag:

```bash
codesign --force --options runtime --sign "Developer ID Application: ..." "Trial Submission Studio.app"
```

### "Notarization failed"

Check the notarization log:

```bash
xcrun notarytool log <submission-id> --apple-id "..." --password "..." --team-id "..."
```

Common issues:

- Missing hardened runtime (`--options runtime`)
- Problematic entitlements (JIT, unsigned memory)
- Unsigned nested code

## Security Notes

- Never commit certificates or passwords to the repository
- Use GitHub's encrypted secrets for all sensitive values
- The app-specific password is NOT your Apple ID password
- Rotate credentials if you suspect they've been compromised
