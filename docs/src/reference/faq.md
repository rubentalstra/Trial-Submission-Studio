# Frequently Asked Questions

Common questions about Trial Submission Studio.

## General

### What is Trial Submission Studio?

Trial Submission Studio is a free, open-source desktop application for transforming clinical trial source data (CSV)
into CDISC-compliant formats like XPT for FDA submissions.

### Is my data sent anywhere?

**No.** Your clinical trial data stays on your computer. Trial Submission Studio works completely offline - all CDISC
standards are embedded in the application, and no data is transmitted over the network.

### Is Trial Submission Studio free?

Yes! Trial Submission Studio is free and open source, licensed under the MIT License. You can use it commercially
without any fees.

### Which platforms are supported?

- macOS (Apple Silicon and Intel)
- Windows (x86_64 and ARM64)
- Linux (x86_64)

## CDISC Standards

### Which CDISC standards are supported?

**Currently Supported:**

- SDTM-IG v3.4
- Controlled Terminology (2024-2025 versions)

**Planned:**

- ADaM-IG v1.3
- SEND-IG v3.1.1

### Can I use this for FDA submissions?

**Not yet.** Trial Submission Studio is currently in alpha development. Our goal is to generate FDA-compliant outputs,
but until the software reaches stable release, all outputs should be validated by qualified regulatory professionals
before submission.

### How often is controlled terminology updated?

Controlled terminology updates are included in application releases. We aim to incorporate new CDISC CT versions within
a reasonable time after their official release.

## Technical

### Do I need SAS installed?

**No.** Trial Submission Studio is completely standalone and does not require SAS or any other software. It generates
XPT files natively.

### What input formats are supported?

Currently, Trial Submission Studio supports **CSV files** as input. The CSV should have:

- Headers in the first row
- UTF-8 encoding (recommended)
- Comma-separated values

### What output formats are available?

- **XPT V5** - FDA standard SAS Transport format
- **XPT V8** - Extended SAS Transport (longer names)
- **Dataset-XML** - CDISC XML format
- **Define-XML 2.1** - Metadata documentation

### How large datasets can it handle?

Trial Submission Studio can handle datasets with hundreds of thousands of rows. For very large datasets (1M+ rows),
ensure adequate RAM (8GB+) and consider processing in batches.

## Usage

### How does column mapping work?

Trial Submission Studio uses fuzzy matching to suggest mappings between your source column names and SDTM variables. It
analyzes name similarity and provides confidence scores. You can accept suggestions or map manually.

### What happens if validation fails?

Validation errors must be resolved before export. The validation panel shows:

- **Errors** (red) - Must fix
- **Warnings** (yellow) - Should review
- **Info** (blue) - Informational

Each message includes the affected rows and suggestions for fixing.

### Can I save my mapping configuration?

Yes, you can save mapping templates and reuse them for similar datasets. This is useful when processing multiple studies
with consistent source data structures.

## Troubleshooting

### The application won't start on macOS

On first launch, macOS may block the application. Right-click and select "Open", then click "Open" in the dialog to
bypass Gatekeeper.

### Import shows garbled characters

Your file may not be UTF-8 encoded. Open it in a text editor and save with UTF-8 encoding, then re-import.

### Validation shows many errors

Common causes:

1. Incorrect domain selection
2. Wrong column mappings
3. Data quality issues in source
4. Controlled terminology mismatches

Review errors one by one, starting with mapping issues.

### Export creates empty file

Ensure:

1. Data is imported successfully
2. Mappings are configured
3. No blocking validation errors exist

## Development

### How can I contribute?

See our [Contributing Guide](../contributing/getting-started.md) for details. We welcome:

- Bug reports
- Feature requests
- Code contributions
- Documentation improvements

### Where do I report bugs?

Open an issue on [GitHub Issues](https://github.com/rubentalstra/Trial-Submission-Studio/issues).

### Is there a roadmap?

Yes! See our [Roadmap](roadmap.md) for planned features and development priorities.

## More Questions?

- **GitHub Discussions**: [Ask questions](https://github.com/rubentalstra/Trial-Submission-Studio/discussions)
- **Issues**: [Report problems](https://github.com/rubentalstra/Trial-Submission-Studio/issues)
- **Documentation**: You're reading it!
