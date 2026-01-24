---
paths:
  - "crates/tss-standards/**/*.rs"
  - "standards/**/*"
---

# Standards & Validation Rules (tss-standards)

## CRITICAL: Always Ask First

Changes to CDISC standards validation require explicit approval.

This includes:
- Variable definitions (SDTM, ADaM, SEND)
- Controlled terminology lookups
- Validation rule logic
- Standards registry behavior

## Embedded Standards

Standards are embedded CSV files in `standards/` for offline operation.
Do NOT modify these files without explicit approval.

## When Adding New Standards

1. Ask which standard version
2. Ask about backward compatibility needs
3. Document the source of truth