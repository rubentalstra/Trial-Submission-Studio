# Security Policy

Trial Submission Studio is a desktop application for transforming clinical trial data into FDA-compliant CDISC formats.
We take security seriously, particularly given the sensitive nature of clinical data.

## Supported Versions

| Version       | Supported          |
|---------------|--------------------|
| 0.0.1-alpha.x | :white_check_mark: |

> [!NOTE]
> This project is currently in alpha. Only the latest release receives security updates. We recommend always using the
> most recent version.

## Clinical Data Handling

Trial Submission Studio is designed with privacy in mind:

- **Local Processing Only**: All data transformation occurs locally on your machine. No data is transmitted to external
  servers.
- **No Telemetry**: The application does not collect or send usage data, analytics, or clinical information.
- **Standard Export Formats**: Exported files (XPT, Dataset-XML, Define-XML) use industry-standard formats without
  additional encryption, as required by regulatory submission specifications.

### User Responsibility

This tool is a **data transformation utility**, not a validated regulatory system. Users are responsible for:

- Compliance with applicable regulations (HIPAA, GxP, 21 CFR Part 11)
- Secure storage and handling of source data and exported files
- Access control on their systems
- Validation activities required for production regulatory submissions

> [!CAUTION]
> Trial Submission Studio is alpha software under active development. It is not intended for production regulatory
> submissions at this time.

## Reporting a Vulnerability

We appreciate responsible disclosure of security vulnerabilities.

### How to Report

Please report security vulnerabilities
through [GitHub Security Advisories](https://github.com/rubentalstra/Trial-Submission-Studio/security/advisories/new).
This allows for private discussion and coordinated disclosure.

### What to Include

- Description of the vulnerability
- Steps to reproduce the issue
- Potential impact assessment
- Any suggested fixes (optional)

### Response Timeline

- **Acknowledgment**: Within 7 days of receiving your report
- **Initial Assessment**: We will provide an initial severity assessment and next steps
- **Resolution**: We will work with you on an appropriate fix timeline based on severity

We follow coordinated disclosure practices and will credit reporters in release notes (unless you prefer to remain
anonymous).

## Security Practices

### Code Security

- **Rust Memory Safety**: Built with Rust, providing memory safety guarantees without garbage collection
- **Unsafe Code Policy**: Workspace-level lint warns on `unsafe` code usage
- **Static Analysis**: All commits checked with Clippy (Rust linter) and rustfmt

### Dependency Management

[![dependency status](https://deps.rs/repo/github/rubentalstra/trial-submission-studio/status.svg)](https://deps.rs/repo/github/rubentalstra/trial-submission-studio)

- Workspace-level dependency pinning for reproducible builds
- Minimal external dependencies

### CI/CD Security

- Automated testing on every pull request
- Code signing and notarization for macOS releases
- Releases distributed through GitHub Releases with checksums

### Auto-Update Security

- Uses `rustls` for TLS (not OpenSSL)
- Updates verified through GitHub API
- Platform detection prevents cross-platform version mismatches

## Out of Scope

The following are explicitly **not** provided by Trial Submission Studio:

- **Encryption at rest**: Export formats must comply with regulatory specifications
- **User authentication**: This is a single-user desktop application
- **Audit trails**: Required audit capabilities should be implemented at the system level
- **Network security**: The application is designed for offline use

For production regulatory environments, implement appropriate controls at the infrastructure level.
