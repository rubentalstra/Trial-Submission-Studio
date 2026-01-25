---
name: check-standards
description: Query CDISC standards definitions (SDTM, ADaM, SEND, CT)
---
# Query CDISC Standards

Look up CDISC standard definitions from the embedded registry.

## Usage
`/check-standards <domain> [variable]`

Examples:
- `/check-standards DM` - List all DM variables
- `/check-standards DM USUBJID` - Get USUBJID specification
- `/check-standards CT AGEU` - Find codelist for AGEU

## Key Files
- `crates/tss-standards/src/registry.rs` - Standards registry (~400 lines)
- `crates/tss-standards/src/ct/mod.rs` - Controlled Terminology
- `standards/` - Embedded CSV standards files

## Available Standards
- **SDTM-IG v3.4**: 60+ domains
- **ADaM-IG v1.3**: ADSL, BDS, OCCDS, TTE
- **SEND-IG v3.1.1**: 10+ domains
- **Controlled Terminology**: 2024-2025 versions
