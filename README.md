# Trial Submission Studio

<<<<<<< HEAD
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.92+-orange.svg)](https://www.rust-lang.org/)
[![Build Status](https://img.shields.io/github/actions/workflow/status/rubentalstra/trial-submission-studio/ci.yml?branch=main)](https://github.com/rubentalstra/trial-submission-studio/actions)
[![Latest Release](https://img.shields.io/github/v/release/rubentalstra/trial-submission-studio)](https://github.com/rubentalstra/trial-submission-studio/releases)

> Transform clinical trial data into FDA-compliant CDISC SDTM formats with
> confidence.

---

> **ALPHA SOFTWARE - ACTIVE DEVELOPMENT**
>
> Trial Submission Studio is currently in **early development (alpha)**.
> Features are incomplete, APIs may change, and bugs are expected. **Do not use
> for production regulatory submissions.**
>
> **Disclaimer:** This software is provided "as is" without warranty of any
> kind. It does not constitute legal, regulatory, or compliance advice. The
> developers are not responsible for any regulatory submissions made using this
> tool. Always consult with qualified regulatory professionals and validate all
> outputs before submission to regulatory authorities.

---

## What is Trial Submission Studio?

Trial Submission Studio is a desktop application for transforming clinical trial
source data (CSV) into CDISC-compliant submission formats.

**Current focus:** SDTM (Study Data Tabulation Model)

**Target users:** Clinical data programmers, biostatisticians, and data managers

## Features

- Multi-format output (XPT V5/V8, Dataset-XML, Define-XML)
- Intelligent column mapping with fuzzy matching
- Comprehensive CDISC validation
- Native cross-platform GUI (macOS, Windows, Linux)
- Embedded CDISC standards (SDTM 3.4, ADaM 1.3, SEND 3.1.1)
- Works offline (no internet required)

## Installation

Download the latest release for your platform:

| Platform | Download                                                                                                    |
| -------- | ----------------------------------------------------------------------------------------------------------- |
| macOS    | [Trial-Submission-Studio.dmg](https://github.com/rubentalstra/trial-submission-studio/releases/latest)      |
| Windows  | [Trial-Submission-Studio.exe](https://github.com/rubentalstra/trial-submission-studio/releases/latest)      |
| Linux    | [Trial-Submission-Studio.AppImage](https://github.com/rubentalstra/trial-submission-studio/releases/latest) |

> **Note:** Releases coming soon! See
> [Releases](https://github.com/rubentalstra/trial-submission-studio/releases)
> page.

### Build from Source (Developers)

<details>
<summary>Click to expand build instructions</summary>

Requires Rust 1.92+

````bash
git clone https://github.com/rubentalstra/trial-submission-studio.git
cd trial-submission-studio
cargo build --release
=======
A Rust-first GUI application for transforming clinical trial source data into CDISC SDTM
outputs (XPT, Dataset-XML, Define-XML) with strict, offline validation.

```bash
>>>>>>> 6978fab (refactored the whole codebase to the new name.  (#41))
cargo run --package tss-gui
````

</details>

## Supported Standards & Formats

### Input

- CSV files (auto-schema detection)

### Output Formats

| Format      | Version      | Description                |
| ----------- | ------------ | -------------------------- |
| XPT         | V5 (default) | FDA-standard SAS Transport |
| XPT         | V8           | Extended names/labels      |
| Dataset-XML | 1.0          | CDISC data exchange        |
| Define-XML  | 2.1          | Metadata documentation     |

### CDISC Standards

**Currently supported:**

- SDTM-IG v3.4
- Controlled Terminology (2024-2025 versions)

**Planned support:**

- ADaM-IG v1.3
- SEND-IG v3.1.1

## FDA Compliance (Planned)

Our goal is full FDA compliance for regulatory submissions:

- Deterministic, auditable output generation
- All 28 SAS missing value codes supported
- IEEE to IBM mainframe float conversion
- XPT V5 format (FDA standard)
- Define-XML 2.1 generation

<<<<<<< HEAD

> **Note:** Currently in alpha. Validate all outputs with qualified
> professionals before submission.

## Why Trial Submission Studio?

| Feature           | Trial Submission Studio       | SAS                  | Pinnacle 21 Community | Pinnacle 21 Enterprise       |
| ----------------- | ----------------------------- | -------------------- | --------------------- | ---------------------------- |
| **Cost**          | Free & Open Source            | Licensed             | Free                  | Licensed                     |
| **License**       | MIT (open source)             | Proprietary          | Proprietary           | Proprietary                  |
| **Platforms**     | macOS, Windows, Linux         | Windows, Unix, Linux | Windows, macOS        | Cloud/Hosted                 |
| **Primary Use**   | Source to SDTM transformation | Full data processing | Validation & QC       | Team validation & governance |
| **CT Validation** | Built-in                      | Via custom code      | Built-in              | Built-in                     |
| **Dependencies**  | Standalone                    | SAS installation     | Minimal               | Browser-based                |

**Our focus:** Trial Submission Studio is a free, open-source tool for
transforming source data into SDTM-compliant formats. Currently focused on SDTM,
with ADaM and SEND planned for future releases. Best suited for individual users
and small teams who want an accessible alternative without license costs.

**Note:** Each tool has different strengths. SAS excels in programmable data
processing. Pinnacle 21 is the industry standard for validation and QC. Trial
Submission Studio focuses on accessible CDISC data transformation.

## System Requirements

| Platform | Minimum Version            | RAM  | Disk Space |
| -------- | -------------------------- | ---- | ---------- |
| macOS    | 10.15+ (Catalina)          | 4 GB | 200 MB     |
| Windows  | Windows 10+                | 4 GB | 200 MB     |
| Linux    | Ubuntu 20.04+ / equivalent | 4 GB | 200 MB     |

## Project Status

**Current Stage: Alpha**

### Working

- Core XPT read/write (V5 + V8)
- CSV ingestion with schema detection
- Fuzzy column mapping engine
- Controlled Terminology validation
- Desktop GUI (egui/eframe)

### In Development

- Dataset-XML export
- Define-XML 2.1 export
- Comprehensive SDTM validation rules
- Export workflow

### Roadmap

See [ROADMAP.md](ROADMAP.md) for planned features including:

- ADaM (Analysis Data Model) support
- SEND (Standard for Exchange of Nonclinical Data) support

## Architecture

10-crate workspace:

- `tss-gui` - Desktop application
- `tss-xpt` - XPT format I/O
- `tss-validate` - Conformance validation
- `tss-map` - Column mapping engine
- `tss-transform` - Data transformations
- `tss-ingest` - CSV loading
- `tss-output` - Multi-format export
- `tss-standards` - CDISC standards loader
- `tss-model` - Core types
- `tss-common` - Shared utilities

## Documentation

- [Contributing Guide](CONTRIBUTING.md)
- [Project Roadmap](ROADMAP.md)
- [Technical Docs](docs/)

## FAQ

<details>
<summary><strong>Is my data sent anywhere?</strong></summary>

No. Your clinical trial data stays on your computer. Trial Submission Studio
works offline and all CDISC standards are embedded in the application.

</details>

<details>
<summary><strong>Which CDISC standards are supported?</strong></summary>

**Currently:**

- SDTM-IG v3.4
- Controlled Terminology (2024-2025 versions)

**Planned:**

- ADaM-IG v1.3
- SEND-IG v3.1.1

</details>

<details>
<summary><strong>Can I use this for FDA submissions?</strong></summary>

**Not yet.** Our goal is to generate FDA-compliant XPT files (SAS Transport V5
format) and Define-XML 2.1 as required for regulatory submissions. The software
is currently in alpha development. Once stable, outputs should still be
validated by qualified professionals before regulatory submission.

</details>

<details>
<summary><strong>Do I need SAS installed?</strong></summary>

No. Trial Submission Studio is completely standalone and does not require SAS or
any other software.

</details>

## Acknowledgments

Trial Submission Studio is built on the shoulders of giants:

- [CDISC](https://www.cdisc.org/) - For the SDTM, ADaM, and SEND standards
- [Polars](https://pola.rs/) - High-performance DataFrame library
- [egui](https://github.com/emilk/egui) - Immediate mode GUI framework
- [RapidFuzz](https://github.com/rapidfuzz/rapidfuzz-rs) - Fuzzy string matching

## License

MIT License - see [LICENSE](LICENSE)

## Contributing

# See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

## References

<<<<<<< HEAD

=======

>>>>>>> b0b8aa2 (Add roundtrip tests for XPT V5 and V8 formats)
>>>>>>> [record-layout-of-a-sas-version-5-or-6-data-set-in-sas-transport-xport-format.pdf](crates/tss-xpt/record-layout-of-a-sas-version-5-or-6-data-set-in-sas-transport-xport-format.pdf)
>>>>>>> [record-layout-of-a-sas-version-8-or-9-data-set-in-sas-transport-format.pdf](crates/tss-xpt/record-layout-of-a-sas-version-8-or-9-data-set-in-sas-transport-format.pdf)

>>>>>>> 6978fab (refactored the whole codebase to the new name. (#41))
