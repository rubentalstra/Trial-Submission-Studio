# Code Signing Policy

Trial Submission Studio uses code signing to ensure authenticity and integrity
of distributed binaries.

## Attribution

**Windows**: Free code signing provided by [SignPath.io](https://signpath.io),
certificate by [SignPath Foundation](https://signpath.org).

**macOS**: Signed and notarized with Apple Developer ID.

**Linux**: Unsigned (standard for AppImage distribution).

## Team Roles

Per SignPath Foundation requirements, this project has a single maintainer:

| Role         | Member                                           | Responsibility                         |
|--------------|--------------------------------------------------|----------------------------------------|
| **Author**   | [@rubentalstra](https://github.com/rubentalstra) | Source code ownership, trusted commits |
| **Reviewer** | [@rubentalstra](https://github.com/rubentalstra) | Review all external contributions      |
| **Approver** | [@rubentalstra](https://github.com/rubentalstra) | Authorize signing requests             |

All external contributions (pull requests) are reviewed before merging.
Only merged code is included in signed releases.

## Privacy & Network Communication

See [Privacy Policy](../legal/privacy.md) for full details.

**Summary:** This application only connects to GitHub when you explicitly
request an update check. No clinical data or personal information is ever
transmitted.

## Build Verification

All signed binaries are:

- Built from source code in this repository
- Compiled via GitHub Actions (auditable CI/CD)
- Tagged releases with full git history
- Verified with SLSA build provenance attestations

## Security Requirements

- MFA required for SignPath access
- MFA recommended for GitHub access (best practice)
- Private signing keys are HSM-protected (SignPath infrastructure)
- All signing requests are logged and auditable

## Verifying Signatures

### Windows

Right-click the `.exe` file → Properties → Digital Signatures tab.

Or use PowerShell:

```powershell
Get-AuthenticodeSignature "trial-submission-studio.exe"
```

The publisher should show **SignPath Foundation**.

### macOS

```bash
codesign -dv --verbose=4 /Applications/Trial\ Submission\ Studio.app
spctl --assess -vvv /Applications/Trial\ Submission\ Studio.app
```

## Reporting Issues

- Security vulnerabilities:
  [GitHub Security Advisories](https://github.com/rubentalstra/Trial-Submission-Studio/security/advisories/new)
- Code signing concerns: [support@signpath.io](mailto:support@signpath.io)
