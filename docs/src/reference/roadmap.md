# Roadmap

Development plans for Trial Submission Studio.

> [!NOTE]
> This roadmap reflects current plans and priorities. Items may change based on
> community feedback and project needs.

## Current Focus

Features actively being developed:

- [ ] Complete SDTM transformation pipeline
- [ ] Dataset-XML export
- [ ] Define-XML 2.1 generation
- [ ] Comprehensive SDTM validation rules
- [ ] Full export workflow

## Short-term

Features planned for near-term development:

- [ ] Batch processing (multiple domains)
- [ ] Export templates and presets
- [ ] Improved error messages and validation feedback
- [ ] Session save/restore
- [ ] Mapping templates (save and reuse mappings)

## Medium-term

Features planned after core functionality is stable:

- [ ] ADaM (Analysis Data Model) support
- [ ] SUPP domain handling improvements
- [ ] Custom validation rules
- [ ] Report generation
- [ ] Undo/redo functionality improvements

## Long-term

Features for future consideration:

- [ ] SEND (Standard for Exchange of Nonclinical Data) support
- [ ] Batch CLI mode for automation
- [ ] Define-XML import (reverse engineering)
- [ ] Plugin system for custom transformations
- [ ] Multi-study support

## Completed

Features that have been implemented:

- [x] Core XPT read/write (V5 + V8)
- [x] CSV ingestion with schema detection
- [x] Fuzzy column mapping engine
- [x] Controlled Terminology validation
- [x] Desktop GUI (Iced 0.14.0)
- [x] SDTM-IG v3.4 standards embedded
- [x] Controlled Terminology (2024-2025)
- [x] Cross-platform support (macOS, Windows, Linux)

## How to Contribute

We welcome contributions! See the
[Contributing Guide](../contributing/getting-started.md) for details.

### Working on Roadmap Items

If you'd like to work on a roadmap item:

1. Check if there's an existing
   [GitHub Issue](https://github.com/rubentalstra/Trial-Submission-Studio/issues)
2. Comment to express interest
3. Wait for maintainer feedback before starting work
4. Follow the [PR guidelines](../contributing/pull-requests.md)

### Suggesting New Features

Have ideas for the roadmap?

1. Check existing
   [issues](https://github.com/rubentalstra/Trial-Submission-Studio/issues) and
   [discussions](https://github.com/rubentalstra/Trial-Submission-Studio/discussions)
2. Open a new issue or discussion
3. Describe the feature and use case
4. Engage with community feedback

## Prioritization

Features are prioritized based on:

1. **Regulatory compliance** - FDA submission requirements
2. **User impact** - Benefit to most users
3. **Complexity** - Development effort required
4. **Dependencies** - Prerequisites from other features
5. **Community feedback** - Requested features

## Versioning Plan

| Version | Focus                      |
|---------|----------------------------|
| 0.1.0   | Core SDTM workflow stable  |
| 0.2.0   | Define-XML and Dataset-XML |
| 0.3.0   | ADaM support               |
| 1.0.0   | Production ready           |

## Stay Updated

- Watch the
  [GitHub repository](https://github.com/rubentalstra/Trial-Submission-Studio)
- Check
  [Releases](https://github.com/rubentalstra/Trial-Submission-Studio/releases)
- Follow
  [Discussions](https://github.com/rubentalstra/Trial-Submission-Studio/discussions)
