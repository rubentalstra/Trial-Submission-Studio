# SDTMIG Knowledge Access Rules (v3.4)

Note: These rules apply to the Rust rebuild as well. SDTMIG assets in
`standards/` are the authoritative offline source and must not be edited or
loaded in full by agents.

This repository includes the SDTMIG v3.4 standard as a **machine-readable
knowledge base**, consisting of:

1. A **vector index** for narrative guidance and explanations

The SDTMIG PDF itself must never be loaded directly by agents.

---

## Source of Truth

- PDF: `standards/sdtmig/v3_4/SDTMIG_v3.4.pdf`
- Vector index (textual guidance): `docs/sdtmig_index/`

⚠️ The PDF is **too large to load directly**.\
Agents must **never** attempt to read or summarize the full document.

---

## How to Access SDTMIG Knowledge

### ✅ Path A — Vector Index (RAG)

Use `docs/sdtmig_query.py` **ONLY** for:

- Domain purpose and scope
- Assumptions and general rules
- Narrative guidance and examples
- Explanatory “how / why” questions

Examples:

- “What is the role of the DM domain in SDTM?”
- “How should SUPPQUAL be used according to SDTMIG?”
- “What assumptions apply to findings domains?”

Expected behavior:

- Cite **section name**
- Cite **page number**
- Answer concisely and factually
- Do not invent rules

---

### ✅ Path B — Deterministic / Structured Data (IMPORTANT)

Questions about **normative SDTMIG facts** must NOT be answered by an LLM.

This includes:

- Required / Expected / Permissible variables
- Domain specification tables
- Variable roles, types, and cores
- Validation rules derived from tables

Examples:

- “Which variables are required in the AE domain?”
- “Which variables are core for DM?”
- “Is AEDECOD required or expected?”

Correct behavior:

- Retrieve structured domain data (when available)
- Do **not** infer or summarize from text
- Do **not** hallucinate missing variables
- Prefer deterministic outputs over prose

⚠️ If structured tables are not yet available, return source excerpts verbatim
and clearly indicate that structured extraction is pending.

---

## ❌ Forbidden

- Loading the full PDF into context
- Guessing SDTMIG rules
- Inferring required variables from narrative text
- Using outdated SDTMIG versions
- Hallucinating domain specifications

---

## Expected Answer Format

When answering SDTMIG-related questions:

1. Give a **direct, concise answer**
2. Reference the **SDTMIG section**
3. Mention **page number(s)** when available
4. Clearly distinguish between:
   - Narrative guidance
   - Deterministic rules
5. Avoid speculation

Example (narrative):

> According to SDTMIG v3.4, the AE domain captures adverse events occurring
> during a clinical study (Section 6.2.1, p. 132).

Example (deterministic):

> Required AE variables are defined in the AE domain specification table (SDTMIG
> v3.4, Section 6.2.1, p. 133).\
> Structured extraction should be used for validation.

# SDTMIG Agent Guidelines

```bash
python - <<'PY'
from docs.sdtmig_query import query_sdtmig
for q in [
    "SDTMIG expected variables when to include",
    "permissible variables may be omitted if not collected",
    "should not include variables not collected CRF SDTMIG 3.4",
    "omit expected variable if not applicable",
]:
    print('\n\n### QUERY:', q)
    query_sdtmig(q)
PY
```
