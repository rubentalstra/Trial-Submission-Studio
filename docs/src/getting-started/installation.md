# Installation

Download the latest release for your platform from
our [GitHub Releases](https://github.com/rubentalstra/Trial-Submission-Studio/releases/latest) page.

## Download Options

| Platform    | Architecture              | Format           | Download                                                                            |
|-------------|---------------------------|------------------|-------------------------------------------------------------------------------------|
| **macOS**   | Apple Silicon (M1/M2/M3+) | `.dmg` or `.zip` | [Download](https://github.com/rubentalstra/Trial-Submission-Studio/releases/latest) |
| **macOS**   | Intel (x86_64)            | `.dmg` or `.zip` | [Download](https://github.com/rubentalstra/Trial-Submission-Studio/releases/latest) |
| **Windows** | x86_64 (64-bit)           | `.zip`           | [Download](https://github.com/rubentalstra/Trial-Submission-Studio/releases/latest) |
| **Windows** | ARM64                     | `.zip`           | [Download](https://github.com/rubentalstra/Trial-Submission-Studio/releases/latest) |
| **Linux**   | x86_64 (64-bit)           | `.tar.gz`        | [Download](https://github.com/rubentalstra/Trial-Submission-Studio/releases/latest) |

## Verifying Your Download

Each release includes SHA256 checksum files (`.sha256`) for security verification.

### macOS/Linux

```bash
# Download the checksum file and binary, then verify
shasum -a 256 -c trial-submission-studio-*.sha256
```

### Windows (PowerShell)

```powershell
# Compare the checksum
Get-FileHash trial-submission-studio-*.zip -Algorithm SHA256
```

## Platform-Specific Instructions

### macOS

1. Download the `.dmg` file for your architecture
2. Open the `.dmg` file
3. Drag **Trial Submission Studio** to your Applications folder
4. On first launch, you may need to right-click and select "Open" to bypass Gatekeeper

> [!TIP]
> **Which version do I need?**
>
> Click the Apple menu () > **About This Mac**:
> - **Chip: Apple M1/M2/M3** → Download the **Apple Silicon** version
> - **Processor: Intel** → Download the **Intel** version

### Windows

1. Download the `.zip` file for your architecture
2. Extract the archive to your preferred location
3. Run `trial-submission-studio.exe`

### Linux

1. Download the `.tar.gz` file
2. Extract: `tar -xzf trial-submission-studio-*.tar.gz`
3. Run: `./trial-submission-studio`

## Next Steps

- [Quick Start Guide](quick-start.md) - Get up and running in 5 minutes
- [System Requirements](system-requirements.md) - Verify your system meets the requirements
- [Building from Source](build-from-source.md) - For developers who want to compile from source
