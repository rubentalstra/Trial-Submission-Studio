# CDISC Implementation Guide MCP Server

A Rust-based MCP (Model Context Protocol) server that provides searchable access to CDISC Implementation Guide documentation:

- **SDTM-IG v3.4** (461 pages)
- **SEND-IG v3.1.1** (244 pages)
- **ADaM-IG v1.3** (88 pages)

## Purpose

This is a **development tool for Claude Code**, not part of the Trial Submission Studio desktop application. It enables Claude to search and query the official CDISC guidance documents when answering questions about standards compliance, derivation rules, and best practices.

## What This Server Provides

The CDISC Implementation Guides are **guidance documents**, not databases. They contain:

- Narrative explanations of derivation rules
- Compliance requirements in prose form
- Examples and edge cases in context
- Best practices and recommendations
- Tables embedded within larger explanatory sections

This server indexes the text content and enables semantic search across all that guidance.

## Setup

### 1. Place PDF files

Download the official CDISC Implementation Guide PDFs (requires CDISC membership) and place them in the `pdfs/` directory:

```
tools/mcp-cdisc-ig/pdfs/
├── ADaMIG_v1.3.pdf
├── SDTMIG_v3.4.pdf
└── SENDIG_v3.1.1.pdf
```

### 2. Build the server

```bash
cd tools/mcp-cdisc-ig
cargo build --release
```

### 3. Process PDFs (TODO)

Currently uses placeholder data. A PDF processing pipeline needs to be implemented to extract text chunks from the actual PDFs.

### 4. Register with Claude Code

The server is registered via `.mcp.json` in the project root:

```json
{
  "mcpServers": {
    "cdisc-ig": {
      "command": "./tools/mcp-cdisc-ig/target/release/mcp-cdisc-ig",
      "args": [],
      "env": {}
    }
  }
}
```

## Available Tools

### `search_ig`
Full-text search across CDISC Implementation Guides.

```
Search for: "USUBJID derivation"
Returns: Relevant text chunks with page references
```

### `get_domain_spec`
Get all guidance text related to a specific domain.

```
Domain: DM, IG: sdtm
Returns: All text chunks discussing the DM domain
```

### `get_variable_spec`
Get all guidance text about a specific variable.

```
Variable: RFSTDTC
Returns: All text chunks discussing RFSTDTC derivation and usage
```

## Data Model

Each chunk represents a meaningful passage from the IG:

```json
{
  "heading": "2.2 Unique Subject Identifier (USUBJID)",
  "page": 25,
  "content": "USUBJID is designed to uniquely identify a subject across all studies...",
  "domain": null,
  "variable": "USUBJID"
}
```

- **heading**: Section/chapter title from the PDF
- **page**: Page number for reference
- **content**: The actual guidance text
- **domain**: Optional domain code if chunk relates to specific domain
- **variable**: Optional variable name if chunk discusses specific variable

## Architecture

- **Standalone crate**: Independent from the main TSS workspace
- **rmcp 0.14**: Official Rust MCP SDK
- **aho-corasick**: Fast multi-pattern text search
- **Embedded data**: Pre-processed content compiled into the binary

## TODO

- [ ] PDF text extraction pipeline (using `pdf-extract` or similar)
- [ ] Section detection and chunking logic
- [ ] Page number extraction
- [ ] Domain/variable tagging heuristics
- [ ] Full content from all 793 pages

## License

MIT (same as Trial Submission Studio)
