---
paths:
  - "crates/tss-submit/**/*.rs"
---

# Submission Pipeline Rules (tss-submit)

## MANDATORY: Deliberation First

Before ANY change to the submission pipeline:

1. State the problem
2. Present 2-3 approaches with pros/cons
3. Wait for explicit approval

**Example:** If validation is failing, don't just fix the rule.
Ask: "Is the validation correct? Should it be in tss-standards instead? Is there a better approach?"

---

## Core Responsibilities

- Mapping source data to CDISC domains
- Normalization and validation
- Export to XPT/XML formats

---

## Before Modifying

ASK before:

- Changing validation rules or error types
- Modifying the mapping pipeline
- Changing export format behavior

---

## Dependencies

Uses:

- `tss-standards` for CDISC definitions
- `tss-ingest` for source data
- `xportrs` for XPT generation
- `quick-xml` for XML generation

---

## Architecture Considerations

When fixing issues in this crate, always ask:

1. **Is this the right layer?** Should this logic be in tss-standards instead?
2. **Is the validation correct?** Does it match CDISC specifications?
3. **Are we validating or transforming?** Keep these concerns separate.