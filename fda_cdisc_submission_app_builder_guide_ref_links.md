# FDA + CDISC Study Data Submission Guide (for Application Builders)

*Version:* 1.0  
*Last updated:* 2026-01-01  
*Scope:* Building a **CDISC-compliant export pipeline** that can generate **SDTM / ADaM / SEND** deliverables for **FDA
** electronic submissions.

> **Disclaimer**: This is engineering guidance, not legal advice. Always confirm the **exact** required versions and
> deliverables for your submission type and review division using FDA’s current resources.

---

## 1) Executive answers (your questions)

### Do we need to submit SAS code?

**For SDTM/SEND tabulations:** FDA does **not** generally require SAS programs *just to accompany SDTM/SEND XPTs*. The
focus is on **XPT + Define-XML + reviewer documentation**.  
**For ADaM and primary/secondary efficacy outputs:** FDA **expects source code** for the creation of **ADaM datasets**
and the **tables/figures** tied to primary and secondary efficacy analyses. The FDA Study Data Technical Conformance
Guide states sponsors **should provide the source code** and submit it as **single‑byte ASCII text**, with **no Windows
executables**.  
➡️ Your app should therefore support exporting **human-readable analysis programs** (SAS, R, Python, etc.) as **plain
text** when ADaM and key analyses are in scope.  
**Source:** FDA Study Data Technical Conformance Guide (sdTCG) v6.1, “Software Programs” section. [1]

### Is Define-XML + XPT + some PDFs “enough”?

**Typically yes for the core study-data package** — but “some PDFs” is very specific:

- **SDTM (clinical tabulations):**
    - SDTM datasets **(.xpt)**
    - **define.xml** (Define‑XML; FDA strongly prefers **v2.0+**) and **stylesheet**
    - **aCRF** (annotated CRF) as **acrf.pdf** (preferred at protocol submission time)
    - **cSDRG** as **csdrg.pdf** (highly recommended; common expectation)
- **ADaM (analysis):**
    - ADaM datasets **(.xpt)**
    - **define.xml** (analysis define) and stylesheet
    - **ADRG** as **adrg.pdf** (highly recommended; common expectation)
    - **analysis source code** (ASCII text)
- **SEND (nonclinical tabulations):**
    - SEND datasets **(.xpt)**
    - **define.xml** (SEND define) and stylesheet
    - **nSDRG** as **nsdrg.pdf** (highly recommended; common expectation)

**Source:** sdTCG v6.1 for naming/expectations for aCRF/SDRG/ADRG and Define‑XML handling. [1]
**Reviewer guide templates:** PhUSE SDRG/ADRG/nSDRG materials. [9]

---

## 2) Regulatory & standards context

### 2.1 FDA “study data standards” landscape (what drives requirements)

FDA provides a central page of **Study Data Standards Resources** linking to the sdTCG and the Data Standards Catalog.
The same page notes that CDER/CBER have conducted preliminary testing of CDISC **Dataset‑JSON** as a potential future
replacement for XPT v5.  
**Source:** FDA Study Data Standards Resources. [2]

> Practical implication for your app: **XPT v5 is still the current baseline**, but architect the export layer so you
> can add **Dataset‑JSON** output later without rewriting the mapping engine.

### 2.2 CDISC “Foundational” standards you’ll encounter

CDISC Foundational Standards cover end-to-end clinical/nonclinical data flows (protocol → collection → tabulation →
analysis → submission).  
Key foundational components for your application:

- **CDASH**: standardizes **data collection** to improve traceability into SDTM. [3]
- **SDTM**: clinical **tabulation** standard (submission tabulations). [4]
- **SEND**: SDTM-based implementation for **nonclinical** studies. [5]
- **ADaM**: standard for **analysis datasets** with traceability from SDTM to results. [6]
- **Define‑XML**: standard metadata format that describes SDTM/SEND/ADaM datasets. [7]
- **Controlled Terminology (CT)**: CDISC terminology is updated regularly (quarterly) and is hosted via NCI EVS. [8]
- **Therapeutic Area User Guides (TAUGs)**: extend foundational standards for disease areas. [10]
- **CDISC Library (API)**: machine-readable metadata for standards and versions. [11]

---

## 3) Submission artifacts: what your app should be able to generate

### 3.1 Common deliverables (clinical and nonclinical)

#### A) Transport datasets (XPT)

FDA expects datasets in **SAS XPORT transport format v5** (“XPT v5”). The sdTCG also states:

- **One dataset per transport file**
- **Dataset name must match the file name**
- **CPORT is not accepted**
- **No SAS custom formats** should be used to decode values
- **Do not compress** the transport files  
  **Source:** sdTCG v6.1 dataset format guidance. [1]

#### B) Define-XML (`define.xml` + stylesheet)

Key expectations from sdTCG v6.1:

- Define‑XML **v2.0 or later** is strongly preferred.
- `define.xml` should be **printable**; if not printable, submit an additional **define.pdf**.
- Include the **Define‑XML stylesheet** in the same directory.  
  Also:
- Use **separate define.xml files** for different dataset types (e.g., SDTM vs ADaM vs SEND).  
  **Source:** sdTCG v6.1 Define‑XML section. [1]

#### C) Reviewer guides (PDF)

Recommended practice is to provide:

- **csdrg.pdf** for clinical SDTM
- **nsdrg.pdf** for nonclinical SEND
- **adrg.pdf** for ADaM  
  These guides give reviewers a human-readable orientation to known issues, derivations, deviations, etc.  
  **Source:** sdTCG v6.1 for naming conventions; PhUSE templates for structure. [1],[9]

---

## 4) SDTM package (clinical tabulations)

### 4.1 What your app should output

Minimum “practical SDTM package”:

- `/tabulations/sdtm/*.xpt`
- `define.xml` + stylesheet (+ `define.pdf` if needed)
- `acrf.pdf`
- `csdrg.pdf`

### 4.2 aCRF expectations

An aCRF maps collected CRF fields to SDTM variables/values. FDA indicates it **should be submitted preferably when the
protocol is submitted**, and the file name should be **acrf.pdf**.  
**Source:** sdTCG v6.1 aCRF section. [1]

### 4.3 Handling large domains (splitting)

The sdTCG notes that the FDA ESG file size limit is **5 GB** and describes splitting large domains (example: splitting
LB) into smaller files in a `split` subdirectory while also submitting the non-split “standard” domain file.  
**Source:** sdTCG v6.1 dataset splitting guidance. [1]

---

## 5) ADaM package (analysis)

### 5.1 What your app should output

Minimum “practical ADaM package”:

- `/analysis/adam/*.xpt`
- `define.xml` (analysis define) + stylesheet
- `adrg.pdf`
- **analysis source code** (ASCII text)
- (Optional but valuable) traceability outputs (e.g., derivation specs)

### 5.2 Source code requirements (important for your question)

FDA sdTCG v6.1: sponsors **should provide source code** used to create all **ADaM datasets** and tables/figures
associated with **primary and secondary efficacy** analyses, and submit this code in **single-byte ASCII text**.  
**Source:** sdTCG v6.1 “Software Programs”. [1]

> For an application: treat “source code” as a required export class when ADaM is produced (and/or when the tool
> produces TLFs).

---

## 6) SEND package (nonclinical tabulations)

### 6.1 What your app should output

Minimum “practical SEND package”:

- `/tabulations/send/*.xpt`
- `define.xml` (SEND define) + stylesheet
- `nsdrg.pdf`

### 6.2 SEND-specific gotchas to support

From sdTCG v6.1:

- For nonclinical studies, `StudyName` in the Define‑XML should be the **study descriptor** (commonly the study number)
  and `ProtocolName` should be the protocol identifier.
- All records in SEND datasets should have the **same STUDYID** for that study.  
  **Source:** sdTCG v6.1 SEND/Define guidance. [1]

### 6.3 SEND is “SDTM-based”

CDISC describes SEND as an **implementation of SDTM for nonclinical studies**.  
**Source:** CDISC SEND page. [5]

---

## 7) eCTD placement (where files go)

FDA submissions are typically delivered in **eCTD format**; FDA’s eCTD guidance explains that submissions not in a
processable electronic format may not be received (unless exempt/waived).  
**Source:** FDA eCTD guidance. [12]

Within the eCTD structure, study datasets are placed in dataset folders (commonly under Module 5), with separate
subfolders for **tabulations** (SDTM/SEND) and **analysis** (ADaM). Your app should be able to export an **eCTD-friendly
directory tree**.

> Exact eCTD folder names and validation rules vary by submission type and eCTD version. Use sdTCG and eCTD technical
> conformance resources as the operational reference. [1],[2],[12]

---

## 8) CDISC-aware application requirements (engineering checklist)

### 8.1 Standards/version management (must-have)

Your app should let a sponsor lock a project to:

- SDTMIG version (and any relevant **TAUG** extensions)
- ADaMIG version
- SENDIG version (if nonclinical)
- Define-XML version
- Controlled Terminology package date/version  
  **Why:** reviewers expect you to identify what you implemented; Define‑XML v2.1 explicitly supports identifying
  standards/CT versions referenced. [13]

**Best practice:** integrate the **CDISC Library API** to retrieve standards metadata programmatically. [11]

### 8.2 Controlled terminology (CT) pipeline

- Provide CT browsing/search and **mapping suggestions** (e.g., for `--TESTCD`, `--CAT`, `--SCAT`, `--ORRESU`, etc.)
- Track CT “package date” used per study
- Support periodic CT updates without silently changing past submissions  
  **Source:** CDISC Controlled Terminology page (regular updates) and NCI CDISC terminology hosting. [8]

### 8.3 Mapping engine core capabilities

Your SDTM/SEND mapper should support:

- Domain assignment (standard + custom domains)
- Variable-level mapping (name, label, type, length)
- Controlled terminology validation
- Value-level metadata (VLMD) and **Define-XML value-level items**
- SUPPQUAL patterns and RELREC where appropriate
- Derivations with traceability (carry source identifiers)
- Date/time handling and ISO formats
- Unit harmonization (and documenting conversions)

### 8.4 Define-XML generator capabilities

A robust Define-XML generator should:

- Generate dataset-level metadata (labels, structure)
- Generate variable-level metadata (type, length, label, origin)
- Produce codelists and external dictionaries references
- Support computational methods/derivations
- Include “leaf” references to PDFs (aCRF, SDRG, ADRG) where applicable
- Output a standards-compliant stylesheet bundle  
  **Source:** CDISC Define‑XML overview + Define‑XML v2.1 capabilities. [7],[13]

### 8.5 Reviewer guide (SDRG/ADRG/nSDRG) generator capabilities

If your tool can auto-draft reviewer guides, it should prefill:

- Data standards versions used
- Key study design summary
- Known data issues and rationale
- Handling of missing data / imputation (especially for ADaM)
- Non-standard approaches and sponsor decisions
- Split-domain explanations (if any)  
  **Source:** PhUSE reviewer guide resources. [9]

### 8.6 Validation integration (your current approach)

You already run **Pinnacle 21** validation. Keep that, and add:

- A “ruleset selection” aligned to project versions
- A “waiver rationale” tracker (what you suppress and why)
- A submission-ready validation report bundle (PDF/CSV)  
  (Your reviewers typically care about *why* a rule is violated and whether it is justified.)

---

## 9) Using the protocol as metadata input (what you can extract)

Your dummy protocol contains enough structure for a metadata bootstrap. Examples you can reliably extract or request
from users:

- **Trial design**: randomized, double-blind, placebo-controlled, 2 parallel arms; treatment period 8 weeks. (Dummy
  protocol synopsis)
- **Arms**: active vs placebo, 1:1 ratio.
- **Population**: inclusion/exclusion criteria and target disease.
- **Endpoints**: primary safety endpoints (TEAE/SAE) and primary efficacy endpoint (e.g., change from baseline in
  scoring parameter).
- **Schedule of assessments**: visit schedule with procedures (ECG, vitals, labs, PGA score, etc.)

This protocol-derived information can feed:

- SDTM Trial Design domains (`TA`, `TE`, `TV`, `TS`, etc.)
- Planned SDTM domains (AE, VS, LB, EG, CM, MH, etc.)
- ADaM analysis set definitions and key derived variables (flags, baseline, endpoints)

(Your app can store these as a structured “study configuration” and keep it traceable to the protocol version.)

---

## 10) “Fully compatible app” compliance checklist

### 10.1 Output package checklist

- [ ] SDTM/SEND datasets: XPT v5, one dataset per file, correct naming
- [ ] No compression, no CPORT, no custom SAS formats
- [ ] Define‑XML v2.0+ with stylesheet, printable (or define.pdf)
- [ ] Separate define.xml per dataset type (SDTM vs ADaM vs SEND)
- [ ] Reviewer guides: csdrg.pdf / nsdrg.pdf / adrg.pdf
- [ ] aCRF: acrf.pdf (clinical SDTM)
- [ ] ADaM: analysis source code (ASCII)
- [ ] Large file strategy (5 GB limit; split folder strategy)
- [ ] Version locking (IG versions + CT package date)

### 10.2 Product/engineering checklist

- [ ] Standards metadata via CDISC Library API
- [ ] Controlled terminology update workflow (no silent changes)
- [ ] Mapping traceability store (source → SDTM/SEND → ADaM → outputs)
- [ ] Automated generation of Define-XML and reviewer guide skeletons
- [ ] Pinnacle 21 integration + rule waiver tracking
- [ ] Export to eCTD-friendly folder layout

---

## 11) References (authoritative sources)

1. [FDA Study Data Technical Conformance Guide (sdTCG) v6.1 (Dec 2025)][1]
2. [FDA Study Data Standards Resources][2]
3. [CDISC CDASH (Foundational)][3]
4. [CDISC SDTM (Foundational)][4]
5. [CDISC SEND (Foundational)][5]
6. [CDISC ADaM (Foundational)][6]
7. [CDISC Define‑XML (Data Exchange)][7]
8. [CDISC Controlled Terminology (a)][8a] • [CDISC Controlled Terminology (b)][8b]
9. [PhUSE Reviewer’s Guide materials (a)][9a] • [PhUSE Reviewer’s Guide materials (b)][9b]• [PhUSE Reviewer’s Guide materials (c)][9c]
10. [CDISC Therapeutic Area User Guides (TAUGs)][10]
11. [CDISC Library + API documentation (a)][11a] • [CDISC Library + API documentation (b)][11b]
12. [FDA eCTD guidance (electronic format using eCTD specifications)][12]
13. [Define‑XML v2.1][13]

---

## 12) Notes on what may change (future-proofing)

- FDA has publicly noted preliminary testing of **CDISC Dataset-JSON** as a potential replacement for XPT v5, so design
  your export layer to support multiple output formats. [2]

- Exact “required standards and versions” can vary by submission type and can change over time; treat FDA’s
  catalog/resources as a live dependency. [2],[12]

---

## Link references

[1]: <https://www.fda.gov/media/153632/download> "FDA Study Data Technical Conformance Guide (sdTCG) v6.1 (Dec 2025)"

[2]: <https://www.fda.gov/industry/fda-data-standards-advisory-board/study-data-standards-resources> "FDA Study Data Standards Resources"

[3]: <https://www.cdisc.org/standards/foundational/cdash> "CDISC CDASH (Foundational)"

[4]: <https://www.cdisc.org/standards/foundational/sdtm> "CDISC SDTM (Foundational)"

[5]: <https://www.cdisc.org/standards/foundational/send> "CDISC SEND (Foundational)"

[6]: <https://www.cdisc.org/standards/foundational/adam> "CDISC ADaM (Foundational)"

[7]: <https://www.cdisc.org/standards/data-exchange/define-xml> "CDISC Define‑XML (Data Exchange)"

[8a]: <https://www.cdisc.org/standards/terminology/controlled-terminology> "CDISC Controlled Terminology (a)"

[8b]: <https://www.cancer.gov/about-nci/organization/cbiit/vocabulary/cdisc> "CDISC Controlled Terminology (b)"

[9a]: <https://advance.hub.phuse.global/wiki/spaces/WEL/pages/26804405/Clinical%2BIntegrated%2BStudy%2BData%2BAnalysis%2BData%2BReviewer%2Bs%2BGuide> "PhUSE Reviewer’s Guide materials (a)"

[9b]: <https://advance.hub.phuse.global/wiki/spaces/WEL/pages/26804660/Analysis%2BData%2BReviewer%2Bs%2BGuide%2BADRG%2BPackage> "PhUSE Reviewer’s Guide materials (b)"

[9c]: <https://advance.hub.phuse.global/wiki/x/nQCZAQ> "PhUSE Reviewer’s Guide materials (c)"

[10]: <https://www.cdisc.org/standards/therapeutic-areas/published-user-guides> "CDISC Therapeutic Area User Guides (TAUGs)"

[11a]: <https://www.cdisc.org/cdisc-library> "CDISC Library + API documentation (a)"

[11b]: <https://www.cdisc.org/cdisc-library/api-documentation> "CDISC Library + API documentation (b)"

[12]: <https://www.fda.gov/media/135373/download> "FDA eCTD guidance (electronic format using eCTD specifications)"

[13]: <https://www.cdisc.org/standards/data-exchange/define-xml/define-xml-v2-1> "Define‑XML v2.1"
