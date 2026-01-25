---
name: check-standards
description: Query CDISC standards definitions (SDTM, ADaM, SEND, CT)
---
# Query CDISC Standards

Look up CDISC standard definitions using the `cdisc-ig` MCP server and embedded registry.

## Usage
`/check-standards <domain> [variable]`

Examples:
- `/check-standards DM` - Get DM domain specification
- `/check-standards DM USUBJID` - Get USUBJID requirements
- `/check-standards CT AGEU` - Find codelist for AGEU
- `/check-standards define ItemGroupDef` - Look up Define-XML element

## Lookup Strategy

### 1. Use MCP Server First (Authoritative)
Query the `cdisc-ig` MCP server for Implementation Guide content:

| Query Type | MCP Tool | Parameters |
|------------|----------|------------|
| Domain overview | `get_domain_spec` | domain, ig (sdtm/send/adam) |
| Variable derivation | `search_ig` | query, ig |
| Compliance rules | `search_ig` | query, ig |
| Define-XML elements | `search_ig` | query, ig=define |
| Browse sections | `list_sections` | ig |

### 2. Check Embedded Standards (Definitions)
For variable metadata and CT codelists:
- `crates/tss-standards/src/registry.rs` - Variable definitions
- `crates/tss-standards/src/ct/mod.rs` - Controlled Terminology
- `standards/` - Embedded CSV standards files

## Available Standards
- **SDTM-IG v3.4**: 824 chunks, 180+ sections
- **ADaM-IG v1.3**: Analysis datasets
- **SEND-IG v3.1.1**: Nonclinical data
- **Define-XML v2.1**: 132 chunks, metadata specification
- **Controlled Terminology**: 2024-2025 versions (embedded CSV)
