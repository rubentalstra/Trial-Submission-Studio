# Disclaimer

Important notices about Trial Submission Studio.

## Alpha Software Notice

> [!WARNING]
> **Trial Submission Studio is currently in alpha development.**

This software is provided for **evaluation and development purposes only**. It is **not yet suitable for production use
** in regulatory submissions.

### What This Means

- Features may be incomplete or change without notice
- Bugs and unexpected behavior may occur
- Data outputs should be independently validated
- No guarantee of regulatory compliance

## Not for Production Submissions

**Do not use Trial Submission Studio outputs for actual FDA, PMDA, or other regulatory submissions** until the software
reaches stable release (version 1.0.0 or later).

### Before Submission

All outputs from Trial Submission Studio should be:

1. **Validated** by qualified regulatory professionals
2. **Verified** against CDISC standards independently
3. **Reviewed** for completeness and accuracy
4. **Tested** with regulatory authority validation tools

## Limitation of Liability

Trial Submission Studio is provided "as is" without warranty of any kind,
express or implied. The authors and contributors:

- Make no guarantees about output accuracy
- Are not responsible for submission rejections
- Cannot be held liable for regulatory issues
- Do not provide regulatory consulting

See the full
[MIT License](https://github.com/rubentalstra/Trial-Submission-Studio/blob/main/LICENSE)
for complete terms.

## CDISC Standards

Trial Submission Studio implements CDISC standards based on publicly available
documentation:

- **SDTM-IG v3.4** - Study Data Tabulation Model Implementation Guide
- **Controlled Terminology** - 2024-2025 versions

CDISC standards are developed by the
[Clinical Data Interchange Standards Consortium](https://www.cdisc.org/). Trial
Submission Studio is not affiliated with or endorsed by CDISC.

## Regulatory Guidance

This software does not constitute regulatory advice. For guidance on:

- **FDA submissions**: Consult
  [FDA Data Standards](https://www.fda.gov/industry/fda-data-standards-advisory-board)
- **PMDA submissions**: Consult [PMDA guidelines](https://www.pmda.go.jp/)
- **EMA submissions**: Consult [EMA standards](https://www.ema.europa.eu/)

## Data Privacy

Trial Submission Studio:

- Processes all clinical data locally on your computer
- Does not collect usage analytics or telemetry
- Does not transmit clinical data over the network

**Network communication** is limited to user-initiated update checks via GitHub
API. No clinical data or personal information is ever transmitted.

See our full [Privacy Policy](privacy.md) for details.

You are responsible for protecting any sensitive or confidential data processed
with this software.

## Reporting Issues

If you encounter problems:

1. **Do not** rely on potentially incorrect outputs
2. **Report** issues on
   [GitHub](https://github.com/rubentalstra/Trial-Submission-Studio/issues)
3. **Validate** outputs through independent means

## Future Stability

We are actively working toward a stable release. Progress can be tracked on our
[Roadmap](../reference/roadmap.md).

| Version | Status                     |
|---------|----------------------------|
| 0.x.x   | Alpha - Not for production |
| 1.0.0+  | Stable - Production ready  |

## Questions?

- [FAQ](../reference/faq.md)
- [GitHub Discussions](https://github.com/rubentalstra/Trial-Submission-Studio/discussions)
