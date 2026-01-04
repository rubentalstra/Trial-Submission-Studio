# Windows Code Signing Setup

This guide explains how to set up Windows code signing using SignPath Foundation for Trial Submission Studio releases.

## Overview

Windows code signing uses Authenticode certificates to sign executables. This eliminates SmartScreen warnings ("Windows
protected your PC") and builds user trust.

**We use [SignPath Foundation](https://signpath.org/)** which provides **free code signing certificates** for open
source projects. The certificate is issued to SignPath Foundation, and they vouch for your project by verifying binaries
are built from your open source repository.

## Prerequisites

- Open source project with an [OSI-approved license](https://opensource.org/licenses)
- GitHub repository with automated builds
- MFA enabled on both GitHub and SignPath accounts
- At least one prior release of your application

## Step 1: Apply to SignPath Foundation

### 1.1 Check Eligibility

Your project must meet [these criteria](https://signpath.org/terms):

1. **OSI-approved license** - Must use an approved open source license (no dual-licensing)
2. **No malware** - No malware or potentially unwanted programs
3. **Actively maintained** - Project must be actively maintained
4. **Already released** - Must have prior releases in the form to be signed
5. **Documented** - Functionality described on download page
6. **All team members use MFA** - For both SignPath and GitHub
7. **Automated builds** - Build process must be fully automated

### 1.2 Submit Application

1. Go to [signpath.org/apply](https://signpath.org/apply)
2. Fill out the application form with your project details
3. Link your GitHub repository
4. Wait for approval (typically a few days)

### 1.3 After Approval

Once approved, you'll receive:

- Organization ID
- Project slug
- Access to the SignPath dashboard

## Step 2: Install SignPath GitHub App

1. Go to [github.com/apps/signpath](https://github.com/apps/signpath)
2. Click **Install**
3. Select your repository
4. Grant necessary permissions

## Step 3: Configure SignPath Dashboard

### 3.1 Add GitHub as Trusted Build System

1. Log in to [app.signpath.io](https://app.signpath.io)
2. Navigate to your project
3. Go to **Trusted Build Systems**
4. Add **GitHub.com** as a trusted build system
5. Link to your repository

### 3.2 Configure Artifact Format

1. Go to **Artifact Configurations**
2. Create a new configuration or use the default
3. Set the root element to `<zip-file>` (GitHub packages artifacts as ZIP)
4. Configure the PE file signing within the ZIP

Example artifact configuration:

```xml

<artifact-configuration xmlns="http://signpath.io/artifact-configuration/v1">
    <zip-file>
        <pe-file path="*.exe">
            <authenticode-sign/>
        </pe-file>
    </zip-file>
</artifact-configuration>
```

### 3.3 Create API Token

1. Go to **My Profile** → **API Tokens**
2. Click **Create API Token**
3. Name: "GitHub Actions"
4. Permissions: **Submitter** role for your project
5. Copy the token (you won't see it again!)

## Step 4: Configure GitHub Secrets

Go to your repository's **Settings → Secrets and variables → Actions** and add these 4 secrets:

| Secret Name                    | Description                          | Where to Find                                  |
|--------------------------------|--------------------------------------|------------------------------------------------|
| `SIGNPATH_API_TOKEN`           | API token with submitter permissions | Step 3.3                                       |
| `SIGNPATH_ORGANIZATION_ID`     | Your organization ID                 | SignPath dashboard URL or settings             |
| `SIGNPATH_PROJECT_SLUG`        | Project identifier                   | SignPath project settings                      |
| `SIGNPATH_SIGNING_POLICY_SLUG` | Signing policy name                  | SignPath project (typically "release-signing") |

### Finding Your IDs

**Organization ID**: Look at your SignPath dashboard URL:

```
https://app.signpath.io/Web/YOUR_ORG_ID/...
```

**Project Slug**: Found in your project's URL or settings page.

**Signing Policy Slug**: Usually `release-signing` for open source projects.

## How It Works

When you push a tag to release:

1. **Build**: GitHub Actions builds the unsigned `.exe`
2. **Upload**: The unsigned artifact is uploaded to GitHub
3. **Submit**: The SignPath action submits the artifact for signing
4. **Sign**: SignPath signs the executable with their certificate
5. **Download**: The signed artifact is downloaded back to the workflow
6. **Verify**: The workflow verifies the signature is valid
7. **Release**: The signed executable is included in the GitHub release

## Verification

After signing, users can verify the signature:

### Windows

Right-click the `.exe` → **Properties** → **Digital Signatures** tab

### PowerShell

```powershell
Get-AuthenticodeSignature "trial-submission-studio.exe"
```

The publisher will show as **SignPath Foundation**.

## Troubleshooting

### "Signing request rejected"

Check the SignPath dashboard for the rejection reason. Common issues:

- Artifact format doesn't match configuration
- Missing permissions on API token
- Project not linked to GitHub as trusted build system

### "API token invalid"

- Ensure the token has Submitter permissions
- Check token hasn't expired
- Verify the token is for the correct organization

### "Artifact not found"

- Ensure the artifact is uploaded before the signing step
- Check the artifact ID is correctly passed between steps
- Verify artifact name matches what SignPath expects

### SmartScreen still warns

After signing, SmartScreen warnings should disappear. If they persist:

- The signature may need time to build reputation
- Check the certificate is valid in Properties → Digital Signatures
- Ensure users download from official GitHub releases

## Security Notes

- Never commit API tokens to the repository
- Use GitHub's encrypted secrets for all sensitive values
- SignPath stores keys in HSM (Hardware Security Module)
- The signing certificate is managed by SignPath Foundation
- All signing requests are logged and auditable

## Cost

SignPath Foundation is **free for open source projects** that meet the eligibility criteria. There are no hidden fees or
limits for qualifying projects.

## Resources

- [SignPath Foundation](https://signpath.org/) - Official website
- [SignPath Documentation](https://docs.signpath.io/) - Full documentation
- [SignPath GitHub Action](https://github.com/SignPath/github-action-submit-signing-request) - GitHub Action
- [SignPath Terms](https://signpath.org/terms) - Eligibility requirements
