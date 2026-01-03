# System Requirements

Trial Submission Studio is designed to run on modern desktop systems with minimal resource requirements.

## Supported Platforms

| Platform    | Architecture              | Minimum Version            | Status    |
|-------------|---------------------------|----------------------------|-----------|
| **macOS**   | Apple Silicon (M1/M2/M3+) | macOS 11.0 (Big Sur)       | Supported |
| **macOS**   | Intel (x86_64)            | macOS 10.15 (Catalina)     | Supported |
| **Windows** | x86_64 (64-bit)           | Windows 10                 | Supported |
| **Windows** | ARM64                     | Windows 11                 | Supported |
| **Linux**   | x86_64 (64-bit)           | Ubuntu 20.04 or equivalent | Supported |

## Hardware Requirements

| Component      | Minimum  | Recommended |
|----------------|----------|-------------|
| **RAM**        | 4 GB     | 8 GB+       |
| **Disk Space** | 200 MB   | 500 MB      |
| **Display**    | 1280x720 | 1920x1080+  |

## Software Dependencies

Trial Submission Studio is a **standalone application** with no external dependencies:

- No SAS installation required
- No Java runtime required
- No internet connection required (works fully offline)
- All CDISC standards are embedded in the application

## Performance Considerations

### Large Datasets

Trial Submission Studio can handle datasets with:

- Hundreds of thousands of rows
- Hundreds of columns

For very large datasets (1M+ rows), consider:

- Ensuring adequate RAM (8GB+)
- Using SSD storage for faster I/O
- Processing data in batches if needed

### Memory Usage

Memory usage scales with dataset size. Approximate guidelines:

- Small datasets (<10,000 rows): ~100 MB RAM
- Medium datasets (10,000-100,000 rows): ~500 MB RAM
- Large datasets (100,000+ rows): 1+ GB RAM

## Troubleshooting

### macOS Gatekeeper

On first launch, macOS may block the application. To resolve:

1. Right-click the application
2. Select "Open"
3. Click "Open" in the dialog

### Linux Permissions

Ensure the executable has run permissions:

```bash
chmod +x trial-submission-studio
```

### Windows SmartScreen

If Windows SmartScreen blocks the application:

1. Click "More info"
2. Click "Run anyway"

---

## Next Steps

- [Installation](installation.md) - Download and install the application
- [Quick Start](quick-start.md) - Get started in 5 minutes
