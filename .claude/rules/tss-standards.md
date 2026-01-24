---
paths:
  - "crates/tss-standards/**/*.rs"
  - "standards/**/*"
---

# Standards & Validation Rules (tss-standards)

## MANDATORY: Deliberation First

Standards changes are especially critical. Before ANY modification:

1. State the problem clearly
2. Present 2-3 approaches with pros/cons (including architectural alternatives)
3. Wait for explicit approval

---

## CRITICAL: Real Failure Example

**What happened (WRONG):**
> "Standards not bundled on Windows" -> immediately modified packaging scripts

**What should have happened (RIGHT):**
> Consider embedding standards in the crate itself (include_str!/include_bytes!)

The architectural solution was better but wasn't considered.
**Always consider where code/data should live, not just how to make it work.**

---

## Always Ask First

Changes to CDISC standards validation require explicit approval.

This includes:

- Variable definitions (SDTM, ADaM, SEND)
- Controlled terminology lookups
- Validation rule logic
- Standards registry behavior

---

## Embedded Standards

Standards are embedded CSV files in `standards/` for offline operation.
Do NOT modify these files without explicit approval.

---

## When Adding New Standards

1. Ask which standard version
2. Ask about backward compatibility needs
3. Document the source of truth

---

## Architecture Questions to Ask

Before any change, consider:

1. **Should this be embedded in the binary?** (compile-time vs. runtime loading)
2. **Should this be in tss-standards or tss-submit?** (definition vs. usage)
3. **Is this a CDISC standard or our interpretation?** (document clearly)