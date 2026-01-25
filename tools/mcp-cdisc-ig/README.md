# CDISC Implementation Guide MCP Server

A Rust-based MCP (Model Context Protocol) server providing searchable access to CDISC standards documentation:

| Standard       | Version | Content                            |
|----------------|---------|------------------------------------|
| **SDTM-IG**    | v3.4    | 824 chunks across 180+ sections    |
| **SEND-IG**    | v3.1.1  | Nonclinical study data             |
| **ADaM-IG**    | v1.3    | Analysis datasets                  |
| **Define-XML** | v2.1    | 132 chunks, metadata specification |

## Purpose

This is a **development tool for Claude Code**, not part of the Trial Submission Studio desktop application. It enables
Claude to search and query official CDISC guidance documents when answering questions about:

- Standards compliance requirements
- Variable derivation rules
- Domain specifications and examples
- Define-XML element definitions
- Best practices and recommendations

## Available Tools

### `search_ig`

Full-text search across CDISC Implementation Guides with relevance scoring.

```
Query: "USUBJID derivation requirements"
IG: sdtm | send | adam | define | all (default)
Limit: 1-50 results (default: 10)
```

Returns matching chunks with heading, content preview, page number, domain tag, and relevance score.

### `list_sections`

List all section headings like a table of contents.

```
IG: sdtm | send | adam | define
```

Returns section headings with chunk counts - useful for discovering what's documented before searching.

### `get_domain_spec`

Get all guidance text related to a specific CDISC domain.

```
Domain: DM, AE, LB, EX, VS, CM, etc.
IG: sdtm | send | adam | define
```

Returns complete domain specification including variables, assumptions, and examples.

### `get_chunk_by_index`

Retrieve a specific chunk by its index number.

```
IG: sdtm | send | adam | define
Index: chunk number from search results
```

Use this to fetch a parent chunk when search results include a `parent_index`.

### `get_related_chunks`

Get a complete section with parent and all sibling chunks.

```
IG: sdtm | send | adam | define
Index: any chunk index (parent or child)
```

Use when search returns a continuation chunk (has `parent_index`) and you need full context.

## Data Model

Each chunk represents a meaningful passage from the Implementation Guide:

```json
{
  "index": 119,
  "heading": "5.2 Demographics (DM)",
  "content": "DM – Description/Overview A special-purpose domain...",
  "domain": "DM",
  "page": 62,
  "parent_index": null,
  "score": 0.85
}
```

| Field          | Description                                                          |
|----------------|----------------------------------------------------------------------|
| `index`        | Unique chunk identifier within the IG                                |
| `heading`      | Section/chapter title from the PDF                                   |
| `content`      | The actual guidance text (truncated in search results)               |
| `domain`       | Domain code if chunk relates to specific domain (e.g., "DM", "AE")   |
| `page`         | PDF page number for reference                                        |
| `parent_index` | If set, this is a continuation chunk - fetch parent for full context |
| `score`        | Relevance score (0.0-1.0) in search results                          |

## Setup

### 1. Build the server

```bash
cd tools/mcp-cdisc-ig
cargo build --release
```

### 2. Process PDFs (if updating content)

Place official CDISC Implementation Guide PDFs in the `pdfs/` directory:

```
tools/mcp-cdisc-ig/pdfs/
├── ADaMIG_v1.3.pdf
├── SDTMIG_v3.4.pdf
├── SENDIG_v3.1.1.pdf
└── DefineXML_2-1.pdf
```

Run the PDF processor (includes TUI with progress indicators):

```bash
cargo run --bin process_pdfs
```

This extracts text, detects sections, chunks content, and outputs JSON to `data/`.

### 3. Register with Claude Code

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

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    MCP Server (stdio)                       │
├─────────────────────────────────────────────────────────────┤
│  Tools: search_ig, list_sections, get_domain_spec,          │
│         get_chunk_by_index, get_related_chunks              │
├─────────────────────────────────────────────────────────────┤
│  Index: IgIndex (in-memory, loaded from JSON)               │
├─────────────────────────────────────────────────────────────┤
│  Search: aho-corasick (fast multi-pattern text search)      │
└─────────────────────────────────────────────────────────────┘
         ▲
         │ JSON data files
         │
┌─────────────────────────────────────────────────────────────┐
│  data/                                                      │
│  ├── sdtm-ig-v3.4.json      (1.8 MB, 824 chunks)            │
│  ├── adam-ig-v1.3.json      (385 KB)                        │
│  ├── define-xml-v2.1.json   (247 KB, 132 chunks)            │
│  └── send-ig-v3.1.1.json                                    │
└─────────────────────────────────────────────────────────────┘
         ▲
         │ PDF processing (offline)
         │
┌─────────────────────────────────────────────────────────────┐
│  process_pdfs binary                                        │
│  - Two-pass extraction (sections → chunks)                  │
│  - Domain detection heuristics                              │
│  - TUI with progress indicators (ratatui)                   │
│  - lopdf for PDF text extraction                            │
└─────────────────────────────────────────────────────────────┘
```

### Key Dependencies

| Crate                  | Purpose                        |
|------------------------|--------------------------------|
| `rmcp 0.14`            | Official Rust MCP SDK          |
| `aho-corasick`         | Fast multi-pattern text search |
| `lopdf`                | PDF text extraction            |
| `ratatui`              | TUI for PDF processing         |
| `tokio`                | Async runtime                  |
| `serde` / `serde_json` | Serialization                  |

### Design Principles

- **Standalone crate**: Independent from the main TSS workspace
- **Embedded data**: Pre-processed JSON compiled/loaded at startup
- **Offline-first**: No network calls, all content local
- **Fast startup**: Index loads in milliseconds

## Usage Examples

### Finding USUBJID requirements

```
Tool: search_ig
Query: "USUBJID derivation"
IG: sdtm
```

### Getting complete DM domain specification

```
Tool: get_domain_spec
Domain: DM
IG: sdtm
```

### Exploring Define-XML structure

```
Tool: list_sections
IG: define
```

Then search for specific elements:

```
Tool: search_ig
Query: "ItemGroupDef dataset"
IG: define
```

### Following continuation chunks

When search returns a chunk with `parent_index: 119`:

```
Tool: get_related_chunks
IG: sdtm
Index: 119
```

This returns the parent chunk and all its children for complete context.

## License

MIT (same as Trial Submission Studio)
