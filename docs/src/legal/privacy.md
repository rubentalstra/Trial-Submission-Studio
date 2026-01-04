# Privacy Policy

Trial Submission Studio is designed with privacy as a core principle.

## Data Collection

**We do not collect any data.** Trial Submission Studio:

- Does not collect usage analytics or telemetry
- Does not track user behavior
- Does not collect personal information
- Does not access or transmit clinical trial data

## Local Processing

All clinical data processing occurs **entirely on your local computer**:

- Source files (CSV, XPT) are read locally
- Transformations execute in local memory
- Output files are written to local storage
- No data is uploaded to any server

## Network Communication

Trial Submission Studio connects to the internet **only when you explicitly
request it**:

| Action            | Destination      | Purpose                   |
|-------------------|------------------|---------------------------|
| Check for Updates | `api.github.com` | Fetch latest release info |
| Download Update   | `github.com`     | Download new version      |

**Important:**

- Update checks are **user-initiated only** (not automatic)
- No clinical data is ever transmitted
- No personal information is sent
- All connections use TLS encryption

This complies with SignPath Foundation's requirement:

> "This program will not transfer any information to other networked systems
> unless specifically requested by the user."

## Third-Party Services

The only third-party service used is **GitHub** for:

- Hosting releases and source code
- Providing update information via GitHub Releases API

For GitHub's data practices, see:
[GitHub Privacy Statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement)

## Data Storage

Trial Submission Studio may store the following locally:

| Data              | Location            | Purpose           |
|-------------------|---------------------|-------------------|
| User preferences  | OS config directory | Remember settings |
| Recent files list | OS config directory | Quick access      |
| Window state      | OS config directory | Restore layout    |

**Storage locations by platform:**

- **Windows:** `%APPDATA%\trial-submission-studio\`
- **macOS:** `~/Library/Application Support/trial-submission-studio/`
- **Linux:** `~/.config/trial-submission-studio/`

No clinical data is ever stored by the application itself.

## Your Responsibilities

You are responsible for:

- Protecting clinical data on your system
- Compliance with HIPAA, GxP, 21 CFR Part 11 as applicable
- Secure storage of source and output files
- Access control on your computer

## Changes to This Policy

Changes will be documented in release notes and this file.

## Contact

Questions about privacy:
[GitHub Discussions](https://github.com/rubentalstra/Trial-Submission-Studio/discussions)
