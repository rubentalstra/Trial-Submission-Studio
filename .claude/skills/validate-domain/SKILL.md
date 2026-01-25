---
name: validate-domain
description: Run CDISC validation checks on a domain DataFrame
---
# Validate CDISC Domain

Run comprehensive CDISC validation on the specified domain.

## Usage
`/validate-domain <domain-code>` (e.g., `/validate-domain DM`)

## Workflow
1. Identify the domain in the codebase (check tss-submit/src/validate/)
2. Understand the validation checks for this domain type
3. Run the test suite: `cargo test --package tss-submit validate`
4. Analyze any validation failures in CDISC context
5. Report findings with severity levels (Error/Warning/Informational)

## Key Files
- `crates/tss-submit/src/validate/mod.rs` - Validation entry points
- `crates/tss-submit/src/validate/ct.rs` - Controlled Terminology validation
- `crates/tss-submit/src/validate/cross_domain.rs` - Cross-domain checks
- `crates/tss-standards/src/ct/` - CT registry

## Validation Categories
- **CT**: Controlled Terminology compliance
- **Required/Expected**: SDTM variable requirements
- **Data Types**: Numeric/string compliance
- **ISO 8601**: Date/datetime formats
- **Sequence**: Uniqueness per USUBJID
- **Text Length**: Character limits per spec
- **Identifiers**: Primary key null checks
- **Cross-Domain**: USUBJID consistency, RDOMAIN references
