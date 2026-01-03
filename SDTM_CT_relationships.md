# SDTMIG ↔ CDISC Controlled Terminology (CT) link map

Generated from the provided CSV extracts on **2025-12-28** (timezone
Europe/Amsterdam).

## Goal

This document explains how to correctly link **SDTM domains + variables** (from
`Variables.csv` / `Datasets.csv`) to **CDISC Controlled Terminology (CT)
codelists and terms** (from the official CT CSVs).

It’s written to help a coding/validation agent implement **validation** (is a
value allowed?) and **normalization** (convert synonyms/variants to canonical
CDISC submission values).

## Input files and key columns

### SDTMIG metadata

- `Datasets.csv`
    - **Key:** `Dataset Name` (domain, e.g., `DM`, `AE`)
    - Fields used: `Class`, `Dataset Label`, `Structure`
- `Variables.csv`
    - **Key:** (`Dataset Name`, `Variable Name`)
    - Fields used:
        - `CDISC CT Codelist Code(s)` → one or more **NCI codelist codes** like
          `C66731`
        - `Described Value Domain(s)` → external formats/dictionaries (e.g.,
          `MedDRA`, `LOINC`, `ISO 8601...`)
        - `Value List` → fixed constant values (e.g., `DOMAIN` must equal the 2-char
          domain code)
        - plus `Type`, `Role`, `Core`, `CDISC Notes` for additional constraints

#### `Core` column (`Req`, `Exp`, `Perm`)

In SDTMIG domain specification tables, the **Core** column indicates whether a
variable is:

- **`Req` (Required)**: The variable is required for the domain. A conforming
  dataset for that domain should **include the column**. In practice, validators
  typically treat a missing required column (or a required column that is
  inappropriately null) as an **error**, unless an SDTMIG rule for that specific
  variable allows nulls in certain situations.
- **`Exp` (Expected)**: The variable is expected to be present **when applicable
  / when the data exist** for the domain. If the concept never occurs in the
  study it may be omitted or left null, but if it is relevant, its absence is
  usually treated as a **conformance issue** (often a warning needing
  justification).
- **`Perm` (Permissible)**: The variable is optional. If present, it must
  conform to type/role/CT/value rules; if absent, that is **not** a conformance
  issue.

**Important modeling nuance:** SDTMIG explicitly states that a domain
specification table contains **all required and expected variables**, plus **a
set of permissible variables** that are “likely to be included” — **not an
exhaustive list** of every permissible/allowable variable.\
So for system-building: **do not treat “not listed in `Variables.csv` for this
domain” as automatically invalid**; it may be an allowable SDTM variable or a
sponsor extension, and should be handled via separate “allowable variables”
logic. fileciteturn0file0

Practical use in a validator/normalizer:

- Use `Core` to set **severity** (Req=error, Exp=warning, Perm=info/optional),
  while still applying CT/value checks to any variable that is present.
- When a dataset is missing an `Exp` variable, decide whether it is “not
  applicable” vs “missing” by checking study context and/or related variables
  (e.g., if other variables indicate the concept was collected).

### CT (Controlled Terminology) files

Official CT CSVs provided:

- `SDTM_CT_2024-03-29.csv`
- `SEND_CT_2024-03-29.csv`
- `ADaM_CT_2024-03-29.csv`
- `Define-XML_CT_2024-03-29.csv`
- `DDF_CT_2024-03-29.csv`
- `MRCT_CT_2024-03-29.csv`
- `Protocol_CT_2024-03-29.csv`

All CT CSVs share the same columns:

- `Code`
- `Codelist Code`
- `Codelist Extensible (Yes/No)`
- `Codelist Name`
- `CDISC Submission Value`
- `CDISC Synonym(s)`
- `CDISC Definition`
- `NCI Preferred Term`
- `Standard and Date`

## CT file structure: **codelist rows** vs **term rows**

Each CT CSV contains **two types of rows**:

1. **Codelist definition row** (the parent)

- `Code` = the codelist’s NCI code (e.g., `C66731`)
- `Codelist Code` = _(blank / null)_
- `Codelist Extensible (Yes/No)` = `Yes` or `No`
- `CDISC Submission Value` = the codelist short name (used in define.xml
  contexts), **NOT** a permissible dataset value

2. **Codelist term rows** (the children)

- `Code` = the term’s NCI concept code (e.g., `C20197`)
- `Codelist Code` = parent codelist code (e.g., `C66731`)
- `CDISC Submission Value` = the **permissible value in datasets**
- `CDISC Synonym(s)` = alternative spellings/aliases you may want to normalize

### Example: `DM.SEX`

From `Variables.csv`: `DM.SEX` has `CDISC CT Codelist Code(s) = C66731`.

In `SDTM_CT_2024-03-29.csv`:

- codelist row: `Code=C66731`, `Codelist Name=Sex`, `Extensible=No`
- term rows: `Codelist Code=C66731` with submission values `F`, `M`, `INTERSEX`,
  `U` (with synonyms like `UNK`, `Unknown`)

**Important:** for validating the _value_ of `DM.SEX`, you only use the **term
rows** where `Codelist Code = C66731`.

## Relationship model (how everything connects)

Conceptually, the join graph looks like:

```text
SDTM Standard (SDTMIG v3.4)
  └── Domain / Dataset (Datasets.csv: Dataset Name)
        └── Variable (Variables.csv: Variable Name)
              ├──(A) CDISC CT Codelist Code(s)  ──>  CT Codelist (CT: codelist row where Code == codelist code)
              │                                     └── CT Terms (CT: term rows where Codelist Code == codelist code)
              │                                           └── Allowed dataset values = CDISC Submission Value
              ├──(B) Described Value Domain(s)  ──>  External dictionary/format rule (MedDRA, LOINC, ISO 8601, ...)
              └──(C) Value List  ──>  Fixed constant value(s)
```

## Choosing the correct CT package

The same NCI **codelist code** can appear in more than one CT package (e.g.,
both SDTM and SEND). Term sets may overlap but are not guaranteed to be
identical, so **scoping matters**.

**Rule of thumb**

- If your dataset standard is **SDTM**, use **SDTM CT**.
- If your dataset standard is **SEND**, use **SEND CT**.
- If your dataset standard is **ADaM**, use **ADaM CT**.
- Define-XML / DDF / Protocol / MRCT CT are primarily for those artifacts, not
  for SDTM dataset values.

**For the mappings below:** because `Variables.csv` is **SDTMIG v3.4**, the
preferred CT package is **SDTM CT 2024-03-29** when resolving codelists.

## Validation rules for CT-linked variables

Given a variable `(DOMAIN, VARIABLE)` with codelist code(s) `[Cxxxxxx, ...]`:

1. **Allowed values** come from CT term rows:

- Find all rows in the chosen CT package where `Codelist Code == Cxxxxxx`
- The allowed dataset values are the set of `CDISC Submission Value` from those
  rows

2. **Extensible vs Non-extensible**

- `Codelist Extensible (Yes/No) = No`: treat as **closed** list → value not in
  allowed set = **error**
- `... = Yes`: treat as **open** list → value not in allowed set = **warning**
  (or configurable), because sponsors may extend

3. **Normalization (recommended)**

- Build a case-insensitive lookup from:
    - `CDISC Submission Value` (canonical)
    - plus all tokens in `CDISC Synonym(s)` split on `;`
- Normalize an incoming value to the canonical `CDISC Submission Value` when it
  matches either the canonical value or a synonym.

Example (from the `Sex` codelist): normalize `UNK` or `Unknown` → `U`.

### Multiple codelists on a single variable

Some variables list **multiple** codelist codes (separated by `;`). In these
cases:

- **Validation set = union** of allowed term submission values across all listed
  codelists.
- **Normalization:** keep track of which codelist matched (useful for
  metadata/reporting).
- Semantics can be context-dependent (e.g., DS reasons). If you have additional
  study metadata, you may choose to restrict to the most appropriate codelist
  for that study.

## Variables with _external_ value domains (not CDISC CT)

If `CDISC CT Codelist Code(s)` is blank, check these columns:

### `Described Value Domain(s)`

This explicitly indicates that the value space is defined by an **external
standard** (not CDISC CT). In the provided SDTMIG extract, the distinct values
are:

| Described Value Domain        | Variable Count |
|-------------------------------|----------------|
| ISO 8601 datetime or interval | 113            |
| ISO 8601 duration             | 45             |
| MedDRA                        | 11             |
| ISO 8601 duration or interval | 4              |
| LOINC                         | 2              |
| ISO 21090 NullFlavor          | 1              |

Recommended handling:

- `MedDRA`: validate against a MedDRA dictionary version used by the study.
- `LOINC`: validate against the LOINC database.
- `ISO 8601 ...`: validate format (datetime/interval/duration).
- `ISO 21090 NullFlavor`: validate against the NullFlavor code list.

**Important:** You can still store these as a 'controlled' domain, but the
source is not CDISC CT.

### `CDISC Notes` referencing external lists (example: `DM.COUNTRY`)

`DM.COUNTRY` has **no** `CDISC CT Codelist Code(s)` in SDTMIG, but the notes say
it is generally represented using **ISO 3166-1 Alpha-3**.

Even though **SEND CT** includes a `Country` codelist (`C66786` with submission
values like `ABW`, `AFG`, ...), SDTMIG does not link `DM.COUNTRY` to a CDISC CT
codelist. For SDTM validation you should therefore:

- Treat `DM.COUNTRY` as an **ISO 3166-1 alpha-3** validation problem (external
  list), OR
- If your organization decides to normalize across SDTM+SEND using CDISC CT, do
  so deliberately and document that rule (it is not implied by SDTMIG metadata).

### `Value List` (fixed constants)

`Value List` is used when a variable is required to have a fixed value. In this
extract it is mainly used for `DOMAIN` variables where the value must equal the
2-character domain code (e.g., in `DM`, `DOMAIN` must be `DM`).

## Cross-variable consistency using CT (important for validators)

### `--TESTCD` ↔ `--TEST` pairs

Many Findings-like domains use both:

- `--TESTCD` (short code) with a **Test Code** codelist
- `--TEST` (decode/name) with a **Test Name** codelist

In CT, these are typically _paired_ codelists containing the **same term `Code`
values** (NCI concept code) but different `CDISC Submission Value` (code vs
name).

**Implementation pattern**:

1. Look up the `--TESTCD` submission value in its codelist → get the term `Code`
   (NCI concept)
2. In the `--TEST` codelist, find the row with the same term `Code` → expected
   `--TEST` submission value
3. Validate that dataset `--TEST` equals that expected value (or normalize it)

Example from Lab:

- `LBTESTCD` uses codelist `C65047` (Laboratory Test Code)
- `LBTEST` uses codelist `C67154` (Laboratory Test Name)
- Term `ALB` (Albumin) has term code `C64431` in both codelists; `LBTEST` should
  be `Albumin` when `LBTESTCD=ALB`.

## Extracted link map from the provided SDTMIG v3.4 extracts

- Domains in `Datasets.csv`: **63**
- Variables in `Variables.csv`: **1917**
- Variables with CT links: **570**
- Distinct CT codelists referenced: **147**

### Variables that reference multiple codelists

| Domain  | Variable | Codelist Codes                             | Codelist Names                                                                                                                                     |
|---------|----------|--------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------|
| DS      | DSDECOD  | C66727; C114118; C150811                   | Completion/Reason for Non-Completion, Protocol Milestone, Other Disposition Event Response                                                         |
| BS      | BSSPEC   | C78734; C111114                            | Specimen Type, Genetic Sample Type                                                                                                                 |
| EG      | EGTESTCD | C71153; C120523                            | ECG Test Code, Holter ECG Test Code                                                                                                                |
| EG      | EGTEST   | C71152; C120524                            | ECG Test Name, Holter ECG Test Name                                                                                                                |
| EG      | EGSTRESC | C71150; C120522; C101834                   | ECG Result, Holter ECG Results, Normal Abnormal Response                                                                                           |
| IS      | ISBDAGNT | C85491; C181169                            | Microorganism, Binding Agent for Immunogenicity Tests                                                                                              |
| PP      | PPORRESU | C85494; C128684; C128683; C128685; C128686 | PK Units of Measure, PK Units of Measure - Weight g, PK Units of Measure - Weight kg, PK Units of Measure - Dose mg, PK Units of Measure - Dose ug |
| PP      | PPSTRESU | C85494; C128684; C128683; C128685; C128686 | PK Units of Measure, PK Units of Measure - Weight g, PK Units of Measure - Weight kg, PK Units of Measure - Dose mg, PK Units of Measure - Dose ug |
| RS      | RSCAT    | C124298; C118971                           | Category of Oncology Response Assessment, Category of Clinical Classification                                                                      |
| RELSPEC | SPEC     | C78734; C111114                            | Specimen Type, Genetic Sample Type                                                                                                                 |

### Most reused codelists (top 25 by number of variables referencing them)

| Codelist Code | Codelist Name                    | Extensible | Term Count | Referenced By (#variables) |
|---------------|----------------------------------|------------|------------|----------------------------|
| C66742        | No Yes Response                  | No         | 4          | 123                        |
| C71620        | Unit                             | Yes        | 901        | 58                         |
| C99079        | Epoch                            | Yes        | 13         | 44                         |
| C66789        | Not Done                         | No         | 1          | 36                         |
| C66728        | Relation to Reference Period     | No         | 8          | 26                         |
| C74456        | Anatomical Location              | Yes        | 1376       | 19                         |
| C78735        | Evaluator                        | Yes        | 60         | 19                         |
| C85492        | Method                           | Yes        | 504        | 19                         |
| C99073        | Laterality                       | Yes        | 7          | 17                         |
| C99074        | Directionality                   | Yes        | 55         | 13                         |
| C96777        | Medical Evaluator Identifier     | Yes        | 56         | 12                         |
| C78734        | Specimen Type                    | Yes        | 127        | 11                         |
| C78733        | Specimen Condition               | Yes        | 23         | 8                          |
| C66726        | Pharmaceutical Dosage Form       | Yes        | 189        | 7                          |
| C66729        | Route of Administration Response | Yes        | 142        | 6                          |
| C71113        | Frequency                        | Yes        | 102        | 6                          |
| C71148        | Position                         | Yes        | 17         | 6                          |
| C78736        | Reference Range Indicator        | Yes        | 4          | 5                          |
| C85494        | PK Units of Measure              | Yes        | 598        | 4                          |
| C99075        | Portion/Totality                 | Yes        | 7          | 4                          |
| C111114       | Genetic Sample Type              | Yes        | 33         | 3                          |
| C158113       | QRS Method                       | Yes        | 18         | 3                          |
| C181175       | Test Condition Response          | Yes        | 6          | 3                          |
| C66734        | SDTM Domain Abbreviation         | Yes        | 83         | 3                          |
| C128683       | PK Units of Measure - Weight kg  | Yes        | 54         | 2                          |

### All referenced codelists

| Codelist Code | Codelist Name                                          | Extensible | CT Package Used | Term Count | Referenced By (#variables) |
|---------------|--------------------------------------------------------|------------|-----------------|------------|----------------------------|
| C66742        | No Yes Response                                        | No         | SDTM            | 4          | 123                        |
| C71620        | Unit                                                   | Yes        | SDTM            | 901        | 58                         |
| C99079        | Epoch                                                  | Yes        | SDTM            | 13         | 44                         |
| C66789        | Not Done                                               | No         | SDTM            | 1          | 36                         |
| C66728        | Relation to Reference Period                           | No         | SDTM            | 8          | 26                         |
| C74456        | Anatomical Location                                    | Yes        | SDTM            | 1376       | 19                         |
| C78735        | Evaluator                                              | Yes        | SDTM            | 60         | 19                         |
| C85492        | Method                                                 | Yes        | SDTM            | 504        | 19                         |
| C99073        | Laterality                                             | Yes        | SDTM            | 7          | 17                         |
| C99074        | Directionality                                         | Yes        | SDTM            | 55         | 13                         |
| C96777        | Medical Evaluator Identifier                           | Yes        | SDTM            | 56         | 12                         |
| C78734        | Specimen Type                                          | Yes        | SDTM            | 127        | 11                         |
| C78733        | Specimen Condition                                     | Yes        | SDTM            | 23         | 8                          |
| C66726        | Pharmaceutical Dosage Form                             | Yes        | SDTM            | 189        | 7                          |
| C66729        | Route of Administration Response                       | Yes        | SDTM            | 142        | 6                          |
| C71113        | Frequency                                              | Yes        | SDTM            | 102        | 6                          |
| C71148        | Position                                               | Yes        | SDTM            | 17         | 6                          |
| C78736        | Reference Range Indicator                              | Yes        | SDTM            | 4          | 5                          |
| C85494        | PK Units of Measure                                    | Yes        | SDTM            | 598        | 4                          |
| C99075        | Portion/Totality                                       | Yes        | SDTM            | 7          | 4                          |
| C111114       | Genetic Sample Type                                    | Yes        | SDTM            | 33         | 3                          |
| C158113       | QRS Method                                             | Yes        | SDTM            | 18         | 3                          |
| C181175       | Test Condition Response                                | Yes        | SDTM            | 6          | 3                          |
| C66734        | SDTM Domain Abbreviation                               | Yes        | SDTM            | 83         | 3                          |
| C128683       | PK Units of Measure - Weight kg                        | Yes        | SDTM            | 54         | 2                          |
| C128684       | PK Units of Measure - Weight g                         | Yes        | SDTM            | 57         | 2                          |
| C128685       | PK Units of Measure - Dose mg                          | Yes        | SDTM            | 154        | 2                          |
| C128686       | PK Units of Measure - Dose ug                          | Yes        | SDTM            | 132        | 2                          |
| C177908       | Collected Summarized Value Type Response               | Yes        | SDTM            | 7          | 2                          |
| C177910       | Result Scale Response                                  | Yes        | SDTM            | 5          | 2                          |
| C179588       | Result Type Response                                   | Yes        | SDTM            | 61         | 2                          |
| C181170       | Test Operational Objective                             | No         | SDTM            | 4          | 2                          |
| C66770        | Units for Vital Signs Results                          | Yes        | SDTM            | 29         | 2                          |
| C66797        | Category of Inclusion/Exclusion                        | No         | SDTM            | 2          | 2                          |
| C100129       | Category of Questionnaire                              | Yes        | SDTM            | 299        | 1                          |
| C100130       | Relationship to Subject                                | Yes        | SDTM            | 83         | 1                          |
| C101832       | Findings About Test Code                               | Yes        | SDTM            | 221        | 1                          |
| C101833       | Findings About Test Name                               | Yes        | SDTM            | 221        | 1                          |
| C101834       | Normal Abnormal Response                               | No         | SDTM            | 5          | 1                          |
| C101846       | Cardiovascular Test Name                               | Yes        | SDTM            | 164        | 1                          |
| C101847       | Cardiovascular Test Code                               | Yes        | SDTM            | 164        | 1                          |
| C101858       | Procedure                                              | Yes        | SDTM            | 145        | 1                          |
| C102580       | Laboratory Test Standard Character Result              | Yes        | SDTM            | 14         | 1                          |
| C103330       | Subject Characteristic Test Name                       | Yes        | SDTM            | 56         | 1                          |
| C106478       | Reproductive System Findings Test Name                 | Yes        | SDTM            | 99         | 1                          |
| C106479       | Reproductive System Findings Test Code                 | Yes        | SDTM            | 99         | 1                          |
| C111106       | Respiratory Test Code                                  | Yes        | SDTM            | 135        | 1                          |
| C111107       | Respiratory Test Name                                  | Yes        | SDTM            | 135        | 1                          |
| C111110       | Device Events Action Taken with Device                 | Yes        | SDTM            | 3          | 1                          |
| C112023       | Skin Response Test Name                                | Yes        | SDTM            | 13         | 1                          |
| C112024       | Skin Response Test Code                                | Yes        | SDTM            | 13         | 1                          |
| C114118       | Protocol Milestone                                     | Yes        | SDTM            | 13         | 1                          |
| C115304       | Category of Functional Test                            | Yes        | SDTM            | 26         | 1                          |
| C116103       | Nervous System Findings Test Name                      | Yes        | SDTM            | 75         | 1                          |
| C116104       | Nervous System Findings Test Code                      | Yes        | SDTM            | 75         | 1                          |
| C116107       | SDTM Death Diagnosis and Details Test Name             | Yes        | SDTM            | 13         | 1                          |
| C116108       | SDTM Death Diagnosis and Details Test Code             | Yes        | SDTM            | 13         | 1                          |
| C117742       | Ophthalmic Exam Test Name                              | Yes        | SDTM            | 53         | 1                          |
| C117743       | Ophthalmic Exam Test Code                              | Yes        | SDTM            | 53         | 1                          |
| C118971       | Category of Clinical Classification                    | Yes        | SDTM            | 85         | 1                          |
| C119013       | Ophthalmic Focus of Study Specific Interest            | No         | SDTM            | 3          | 1                          |
| C120522       | Holter ECG Results                                     | Yes        | SDTM            | 238        | 1                          |
| C120523       | Holter ECG Test Code                                   | Yes        | SDTM            | 16         | 1                          |
| C120524       | Holter ECG Test Name                                   | Yes        | SDTM            | 16         | 1                          |
| C120525       | Immunogenicity Specimen Assessments Test Code          | Yes        | SDTM            | 83         | 1                          |
| C120526       | Immunogenicity Specimen Assessments Test Name          | Yes        | SDTM            | 83         | 1                          |
| C120527       | Microbiology Test Code                                 | Yes        | SDTM            | 490        | 1                          |
| C120528       | Microbiology Test Name                                 | Yes        | SDTM            | 490        | 1                          |
| C123650       | Tumor or Lesion Identification Test Results            | Yes        | SDTM            | 28         | 1                          |
| C124297       | Biospecimen Events Dictionary Derived Term             | Yes        | SDTM            | 27         | 1                          |
| C124298       | Category of Oncology Response Assessment               | Yes        | SDTM            | 93         | 1                          |
| C124299       | Biospecimen Characteristics Test Name                  | Yes        | SDTM            | 24         | 1                          |
| C124300       | Biospecimen Characteristics Test Code                  | Yes        | SDTM            | 24         | 1                          |
| C124301       | Medical History Event Date Type                        | Yes        | SDTM            | 16         | 1                          |
| C124304       | Subject Status Response                                | Yes        | SDTM            | 3          | 1                          |
| C124305       | Subject Status Test Code                               | Yes        | SDTM            | 3          | 1                          |
| C124306       | Subject Status Test Name                               | Yes        | SDTM            | 3          | 1                          |
| C124309       | Tumor or Lesion Properties Test Result                 | Yes        | SDTM            | 22         | 1                          |
| C125922       | Microscopic Findings Test Details                      | Yes        | SDTM            | 63         | 1                          |
| C125923       | BRIDG Activity Mood                                    | Yes        | SDTM            | 2          | 1                          |
| C127269       | Musculoskeletal System Finding Test Code               | Yes        | SDTM            | 50         | 1                          |
| C127270       | Musculoskeletal System Finding Test Name               | Yes        | SDTM            | 50         | 1                          |
| C128687       | Microbiology Susceptibility Test Name                  | Yes        | SDTM            | 18         | 1                          |
| C128688       | Microbiology Susceptibility Test Code                  | Yes        | SDTM            | 18         | 1                          |
| C129941       | Urinary System Test Name                               | Yes        | SDTM            | 9          | 1                          |
| C129942       | Urinary System Test Code                               | Yes        | SDTM            | 9          | 1                          |
| C132262       | SDTM Microscopic Findings Test Name                    | Yes        | SDTM            | 206        | 1                          |
| C132263       | SDTM Microscopic Findings Test Code                    | Yes        | SDTM            | 206        | 1                          |
| C142179       | Arm Null Reason                                        | Yes        | SDTM            | 4          | 1                          |
| C150811       | Other Disposition Event Response                       | Yes        | SDTM            | 4          | 1                          |
| C160922       | Laboratory Analytical Method Calculation Formula       | Yes        | SDTM            | 26         | 1                          |
| C165643       | Severity Response                                      | Yes        | SDTM            | 7          | 1                          |
| C170443       | Subcategory for Disposition Event                      | Yes        | SDTM            | 2          | 1                          |
| C171444       | Health Care Encounters Dictionary Derived Term         | Yes        | SDTM            | 37         | 1                          |
| C171445       | Mode of Subject Contact                                | Yes        | SDTM            | 9          | 1                          |
| C172330       | PK Analytical Method                                   | Yes        | SDTM            | 4          | 1                          |
| C174225       | Microbiology Findings Test Details                     | Yes        | SDTM            | 8          | 1                          |
| C179589       | Test Method Sensitivity                                | Yes        | SDTM            | 3          | 1                          |
| C179590       | Non-host Organism Identifier Parameters                | Yes        | SDTM            | 16         | 1                          |
| C179591       | Non-host Organism Identifier Parameters Code           | Yes        | SDTM            | 16         | 1                          |
| C181169       | Binding Agent for Immunogenicity Tests                 | Yes        | SDTM            | 293        | 1                          |
| C181171       | Category for Cell Phenotyping                          | Yes        | SDTM            | 3          | 1                          |
| C181172       | Cell State Response                                    | Yes        | SDTM            | 8          | 1                          |
| C181173       | Cell Phenotyping Test Code                             | Yes        | SDTM            | 628        | 1                          |
| C181174       | Cell Phenotyping Test Name                             | Yes        | SDTM            | 628        | 1                          |
| C181176       | Genomic Symbol Type Response                           | Yes        | SDTM            | 23         | 1                          |
| C181177       | Genomic Inheritability Type Response                   | Yes        | SDTM            | 3          | 1                          |
| C181178       | Genomic Findings Test Code                             | Yes        | SDTM            | 10         | 1                          |
| C181179       | Genomic Findings Test Name                             | Yes        | SDTM            | 10         | 1                          |
| C181180       | Genomic Findings Test Detail                           | Yes        | SDTM            | 33         | 1                          |
| C181181       | Genomic Findings Analytical Method Calculation Formula | Yes        | SDTM            | 7          | 1                          |
| C65047        | Laboratory Test Code                                   | Yes        | SDTM            | 2397       | 1                          |
| C66727        | Completion/Reason for Non-Completion                   | Yes        | SDTM            | 43         | 1                          |
| C66731        | Sex                                                    | No         | SDTM            | 4          | 1                          |
| C66738        | Trial Summary Parameter Test Code                      | Yes        | SDTM            | 126        | 1                          |
| C66741        | Vital Signs Test Code                                  | Yes        | SDTM            | 72         | 1                          |
| C66767        | Action Taken with Study Treatment                      | No         | SDTM            | 8          | 1                          |
| C66768        | Outcome of Event                                       | No         | SDTM            | 6          | 1                          |
| C66769        | Severity/Intensity Scale for Adverse Events            | No         | SDTM            | 3          | 1                          |
| C66781        | Age Unit                                               | No         | SDTM            | 5          | 1                          |
| C66788        | Dictionary Name                                        | Yes        | SDTM            | 14         | 1                          |
| C66790        | Ethnic Group                                           | No         | SDTM            | 4          | 1                          |
| C67152        | Trial Summary Parameter Test Name                      | Yes        | SDTM            | 126        | 1                          |
| C67153        | Vital Signs Test Name                                  | Yes        | SDTM            | 72         | 1                          |
| C67154        | Laboratory Test Name                                   | Yes        | SDTM            | 2397       | 1                          |
| C71150        | ECG Result                                             | Yes        | SDTM            | 235        | 1                          |
| C71151        | ECG Test Method                                        | Yes        | SDTM            | 36         | 1                          |
| C71152        | ECG Test Name                                          | Yes        | SDTM            | 109        | 1                          |
| C71153        | ECG Test Code                                          | Yes        | SDTM            | 109        | 1                          |
| C74457        | Race                                                   | No         | SDTM            | 8          | 1                          |
| C74558        | Category of Disposition Event                          | No         | SDTM            | 3          | 1                          |
| C74559        | Subject Characteristic Test Code                       | Yes        | SDTM            | 56         | 1                          |
| C78731        | Drug Accountability Test Name                          | Yes        | SDTM            | 6          | 1                          |
| C78732        | Drug Accountability Test Code                          | Yes        | SDTM            | 6          | 1                          |
| C78737        | Relationship Type                                      | No         | SDTM            | 2          | 1                          |
| C85491        | Microorganism                                          | Yes        | SDTM            | 1669       | 1                          |
| C85493        | PK Parameters                                          | Yes        | SDTM            | 388        | 1                          |
| C85495        | Microbiology Susceptibility Testing Result Category    | No         | SDTM            | 9          | 1                          |
| C85839        | PK Parameters Code                                     | Yes        | SDTM            | 388        | 1                          |
| C90013        | ECG Lead                                               | Yes        | SDTM            | 26         | 1                          |
| C96778        | Tumor or Lesion Properties Test Name                   | Yes        | SDTM            | 72         | 1                          |
| C96779        | Tumor or Lesion Properties Test Code                   | Yes        | SDTM            | 72         | 1                          |
| C96781        | Oncology Response Assessment Test Name                 | Yes        | SDTM            | 47         | 1                          |
| C96782        | Oncology Response Assessment Test Code                 | Yes        | SDTM            | 47         | 1                          |
| C96783        | Tumor or Lesion Identification Test Name               | Yes        | SDTM            | 19         | 1                          |
| C96784        | Tumor or Lesion Identification Test Code               | Yes        | SDTM            | 19         | 1                          |
| C96785        | Oncology Response Assessment Result                    | Yes        | SDTM            | 95         | 1                          |

## Domain → Variable → Codelist links (SDTMIG v3.4)

Only variables that have `CDISC CT Codelist Code(s)` are listed here. For
variables not listed, use `Described Value Domain(s)`, `Value List`, and/or
`CDISC Notes` for constraints.

### AE

_Dataset label:_ **Adverse Events**\
_Class:_ `Events`\
_Structure:_ One record per adverse event per subject

| Variable   | Codelist link(s)                                                             |
|------------|------------------------------------------------------------------------------|
| `AEACN`    | C66767 (Action Taken with Study Treatment; Extensible=No; Terms=8)           |
| `AEACNDEV` | C111110 (Device Events Action Taken with Device; Extensible=Yes; Terms=3)    |
| `AECONTRT` | C66742 (No Yes Response; Extensible=No; Terms=4)                             |
| `AEENRF`   | C66728 (Relation to Reference Period; Extensible=No; Terms=8)                |
| `AEENRTPT` | C66728 (Relation to Reference Period; Extensible=No; Terms=8)                |
| `AELOC`    | C74456 (Anatomical Location; Extensible=Yes; Terms=1376)                     |
| `AEOUT`    | C66768 (Outcome of Event; Extensible=No; Terms=6)                            |
| `AEPRESP`  | C66742 (No Yes Response; Extensible=No; Terms=4)                             |
| `AESCAN`   | C66742 (No Yes Response; Extensible=No; Terms=4)                             |
| `AESCONG`  | C66742 (No Yes Response; Extensible=No; Terms=4)                             |
| `AESDISAB` | C66742 (No Yes Response; Extensible=No; Terms=4)                             |
| `AESDTH`   | C66742 (No Yes Response; Extensible=No; Terms=4)                             |
| `AESER`    | C66742 (No Yes Response; Extensible=No; Terms=4)                             |
| `AESEV`    | C66769 (Severity/Intensity Scale for Adverse Events; Extensible=No; Terms=3) |
| `AESHOSP`  | C66742 (No Yes Response; Extensible=No; Terms=4)                             |
| `AESINTV`  | C66742 (No Yes Response; Extensible=No; Terms=4)                             |
| `AESLIFE`  | C66742 (No Yes Response; Extensible=No; Terms=4)                             |
| `AESMIE`   | C66742 (No Yes Response; Extensible=No; Terms=4)                             |
| `AESOD`    | C66742 (No Yes Response; Extensible=No; Terms=4)                             |
| `AEUNANT`  | C66742 (No Yes Response; Extensible=No; Terms=4)                             |
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                                     |

### AG

_Dataset label:_ **Procedure Agents**\
_Class:_ `Interventions`\
_Structure:_ One record per recorded intervention occurrence per subject

| Variable   | Codelist link(s)                                                     |
|------------|----------------------------------------------------------------------|
| `AGDOSFRM` | C66726 (Pharmaceutical Dosage Form; Extensible=Yes; Terms=189)       |
| `AGDOSFRQ` | C71113 (Frequency; Extensible=Yes; Terms=102)                        |
| `AGDOSU`   | C71620 (Unit; Extensible=Yes; Terms=901)                             |
| `AGENRF`   | C66728 (Relation to Reference Period; Extensible=No; Terms=8)        |
| `AGENRTPT` | C66728 (Relation to Reference Period; Extensible=No; Terms=8)        |
| `AGOCCUR`  | C66742 (No Yes Response; Extensible=No; Terms=4)                     |
| `AGPRESP`  | C66742 (No Yes Response; Extensible=No; Terms=4)                     |
| `AGROUTE`  | C66729 (Route of Administration Response; Extensible=Yes; Terms=142) |
| `AGSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                            |
| `AGSTRF`   | C66728 (Relation to Reference Period; Extensible=No; Terms=8)        |
| `AGSTRTPT` | C66728 (Relation to Reference Period; Extensible=No; Terms=8)        |
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                             |

### BE

_Dataset label:_ **Biospecimen Events**\
_Class:_ `Events`\
_Structure:_ One record per instance per biospecimen event per biospecimen
identifier per subject

| Variable  | Codelist link(s)                                                               |
|-----------|--------------------------------------------------------------------------------|
| `BEDECOD` | C124297 (Biospecimen Events Dictionary Derived Term; Extensible=Yes; Terms=27) |
| `BELOC`   | C74456 (Anatomical Location; Extensible=Yes; Terms=1376)                       |

### BS

_Dataset label:_ **Biospecimen Findings**\
_Class:_ `Findings`\
_Structure:_ One record per measurement per biospecimen identifier per subject

| Variable   | Codelist link(s)                                                                                           |
|------------|------------------------------------------------------------------------------------------------------------|
| `BSBLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)                                                           |
| `BSMETHOD` | C85492 (Method; Extensible=Yes; Terms=504)                                                                 |
| `BSORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                                                   |
| `BSSPCCND` | C78733 (Specimen Condition; Extensible=Yes; Terms=23)                                                      |
| `BSSPEC`   | C78734 (Specimen Type; Extensible=Yes; Terms=127); C111114 (Genetic Sample Type; Extensible=Yes; Terms=33) |
| `BSSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                                                                  |
| `BSSTRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                                                   |
| `BSTEST`   | C124299 (Biospecimen Characteristics Test Name; Extensible=Yes; Terms=24)                                  |
| `BSTESTCD` | C124300 (Biospecimen Characteristics Test Code; Extensible=Yes; Terms=24)                                  |

### CE

_Dataset label:_ **Clinical Events**\
_Class:_ `Events`\
_Structure:_ One record per event per subject

| Variable   | Codelist link(s)                                              |
|------------|---------------------------------------------------------------|
| `CEENRF`   | C66728 (Relation to Reference Period; Extensible=No; Terms=8) |
| `CEENRTPT` | C66728 (Relation to Reference Period; Extensible=No; Terms=8) |
| `CEOCCUR`  | C66742 (No Yes Response; Extensible=No; Terms=4)              |
| `CEPRESP`  | C66742 (No Yes Response; Extensible=No; Terms=4)              |
| `CESEV`    | C165643 (Severity Response; Extensible=Yes; Terms=7)          |
| `CESTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                     |
| `CESTRF`   | C66728 (Relation to Reference Period; Extensible=No; Terms=8) |
| `CESTRTPT` | C66728 (Relation to Reference Period; Extensible=No; Terms=8) |
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                      |

### CM

_Dataset label:_ **Concomitant/Prior Medications**\
_Class:_ `Interventions`\
_Structure:_ One record per recorded intervention occurrence or constant-dosing
interval per subject

| Variable   | Codelist link(s)                                                     |
|------------|----------------------------------------------------------------------|
| `CMDOSFRM` | C66726 (Pharmaceutical Dosage Form; Extensible=Yes; Terms=189)       |
| `CMDOSFRQ` | C71113 (Frequency; Extensible=Yes; Terms=102)                        |
| `CMDOSU`   | C71620 (Unit; Extensible=Yes; Terms=901)                             |
| `CMENRF`   | C66728 (Relation to Reference Period; Extensible=No; Terms=8)        |
| `CMENRTPT` | C66728 (Relation to Reference Period; Extensible=No; Terms=8)        |
| `CMOCCUR`  | C66742 (No Yes Response; Extensible=No; Terms=4)                     |
| `CMPRESP`  | C66742 (No Yes Response; Extensible=No; Terms=4)                     |
| `CMROUTE`  | C66729 (Route of Administration Response; Extensible=Yes; Terms=142) |
| `CMSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                            |
| `CMSTRF`   | C66728 (Relation to Reference Period; Extensible=No; Terms=8)        |
| `CMSTRTPT` | C66728 (Relation to Reference Period; Extensible=No; Terms=8)        |
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                             |

### CO

_Dataset label:_ **Comments**\
_Class:_ `Special-Purpose`\
_Structure:_ One record per comment per subject

| Variable   | Codelist link(s)                                                |
|------------|-----------------------------------------------------------------|
| `COEVAL`   | C78735 (Evaluator; Extensible=Yes; Terms=60)                    |
| `COEVALID` | C96777 (Medical Evaluator Identifier; Extensible=Yes; Terms=56) |
| `RDOMAIN`  | C66734 (SDTM Domain Abbreviation; Extensible=Yes; Terms=83)     |

### CP

_Dataset label:_ **Cell Phenotype Findings**\
_Class:_ `Findings`\
_Structure:_ One record per test per specimen per timepoint per visit per
subject

| Variable   | Codelist link(s)                                                            |
|------------|-----------------------------------------------------------------------------|
| `CPBLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)                            |
| `CPCAT`    | C181171 (Category for Cell Phenotyping; Extensible=Yes; Terms=3)            |
| `CPCELSTA` | C181172 (Cell State Response; Extensible=Yes; Terms=8)                      |
| `CPCLSIG`  | C66742 (No Yes Response; Extensible=No; Terms=4)                            |
| `CPCOLSRT` | C177908 (Collected Summarized Value Type Response; Extensible=Yes; Terms=7) |
| `CPDRVFL`  | C66742 (No Yes Response; Extensible=No; Terms=4)                            |
| `CPLOBXFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                            |
| `CPMETHOD` | C85492 (Method; Extensible=Yes; Terms=504)                                  |
| `CPNRIND`  | C78736 (Reference Range Indicator; Extensible=Yes; Terms=4)                 |
| `CPORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                    |
| `CPRESSCL` | C177910 (Result Scale Response; Extensible=Yes; Terms=5)                    |
| `CPRESTYP` | C179588 (Result Type Response; Extensible=Yes; Terms=61)                    |
| `CPSPCCND` | C78733 (Specimen Condition; Extensible=Yes; Terms=23)                       |
| `CPSPEC`   | C78734 (Specimen Type; Extensible=Yes; Terms=127)                           |
| `CPSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                                   |
| `CPSTRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                    |
| `CPTEST`   | C181174 (Cell Phenotyping Test Name; Extensible=Yes; Terms=628)             |
| `CPTESTCD` | C181173 (Cell Phenotyping Test Code; Extensible=Yes; Terms=628)             |
| `CPTSTCND` | C181175 (Test Condition Response; Extensible=Yes; Terms=6)                  |
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                                    |

### CV

_Dataset label:_ **Cardiovascular System Findings**\
_Class:_ `Findings`\
_Structure:_ One record per finding or result per time point per visit per
subject

| Variable   | Codelist link(s)                                                |
|------------|-----------------------------------------------------------------|
| `CVBLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)                |
| `CVDIR`    | C99074 (Directionality; Extensible=Yes; Terms=55)               |
| `CVDRVFL`  | C66742 (No Yes Response; Extensible=No; Terms=4)                |
| `CVEVAL`   | C78735 (Evaluator; Extensible=Yes; Terms=60)                    |
| `CVEVALID` | C96777 (Medical Evaluator Identifier; Extensible=Yes; Terms=56) |
| `CVLAT`    | C99073 (Laterality; Extensible=Yes; Terms=7)                    |
| `CVLOBXFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                |
| `CVLOC`    | C74456 (Anatomical Location; Extensible=Yes; Terms=1376)        |
| `CVMETHOD` | C85492 (Method; Extensible=Yes; Terms=504)                      |
| `CVORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                        |
| `CVPOS`    | C71148 (Position; Extensible=Yes; Terms=17)                     |
| `CVSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                       |
| `CVSTRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                        |
| `CVTEST`   | C101846 (Cardiovascular Test Name; Extensible=Yes; Terms=164)   |
| `CVTESTCD` | C101847 (Cardiovascular Test Code; Extensible=Yes; Terms=164)   |
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                        |

### DA

_Dataset label:_ **Product Accountability**\
_Class:_ `Findings`\
_Structure:_ One record per product accountability finding per subject

| Variable   | Codelist link(s)                                                |
|------------|-----------------------------------------------------------------|
| `DAORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                        |
| `DASTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                       |
| `DASTRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                        |
| `DATEST`   | C78731 (Drug Accountability Test Name; Extensible=Yes; Terms=6) |
| `DATESTCD` | C78732 (Drug Accountability Test Code; Extensible=Yes; Terms=6) |
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                        |

### DD

_Dataset label:_ **Death Details**\
_Class:_ `Findings`\
_Structure:_ One record per finding per subject

| Variable   | Codelist link(s)                                                               |
|------------|--------------------------------------------------------------------------------|
| `DDEVAL`   | C78735 (Evaluator; Extensible=Yes; Terms=60)                                   |
| `DDTEST`   | C116107 (SDTM Death Diagnosis and Details Test Name; Extensible=Yes; Terms=13) |
| `DDTESTCD` | C116108 (SDTM Death Diagnosis and Details Test Code; Extensible=Yes; Terms=13) |

### DM

_Dataset label:_ **Demographics**\
_Class:_ `Special-Purpose`\
_Structure:_ One record per subject

| Variable | Codelist link(s)                                   |
|----------|----------------------------------------------------|
| `AGEU`   | C66781 (Age Unit; Extensible=No; Terms=5)          |
| `ARMNRS` | C142179 (Arm Null Reason; Extensible=Yes; Terms=4) |
| `DTHFL`  | C66742 (No Yes Response; Extensible=No; Terms=4)   |
| `ETHNIC` | C66790 (Ethnic Group; Extensible=No; Terms=4)      |
| `RACE`   | C74457 (Race; Extensible=No; Terms=8)              |
| `SEX`    | C66731 (Sex; Extensible=No; Terms=4)               |

### DS

_Dataset label:_ **Disposition**\
_Class:_ `Events`\
_Structure:_ One record per disposition status or protocol milestone per subject

| Variable  | Codelist link(s)                                                                                                                                                                                     |
|-----------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `DSCAT`   | C74558 (Category of Disposition Event; Extensible=No; Terms=3)                                                                                                                                       |
| `DSDECOD` | C66727 (Completion/Reason for Non-Completion; Extensible=Yes; Terms=43); C114118 (Protocol Milestone; Extensible=Yes; Terms=13); C150811 (Other Disposition Event Response; Extensible=Yes; Terms=4) |
| `DSSCAT`  | C170443 (Subcategory for Disposition Event; Extensible=Yes; Terms=2)                                                                                                                                 |
| `EPOCH`   | C99079 (Epoch; Extensible=Yes; Terms=13)                                                                                                                                                             |

### DV

_Dataset label:_ **Protocol Deviations**\
_Class:_ `Events`\
_Structure:_ One record per protocol deviation per subject

| Variable | Codelist link(s)                         |
|----------|------------------------------------------|
| `EPOCH`  | C99079 (Epoch; Extensible=Yes; Terms=13) |

### EC

_Dataset label:_ **Exposure as Collected**\
_Class:_ `Interventions`\
_Structure:_ One record per protocol-specified study treatment, collected-dosing
interval, per subject, per mood

| Variable   | Codelist link(s)                                                     |
|------------|----------------------------------------------------------------------|
| `ECDIR`    | C99074 (Directionality; Extensible=Yes; Terms=55)                    |
| `ECDOSFRM` | C66726 (Pharmaceutical Dosage Form; Extensible=Yes; Terms=189)       |
| `ECDOSFRQ` | C71113 (Frequency; Extensible=Yes; Terms=102)                        |
| `ECDOSU`   | C71620 (Unit; Extensible=Yes; Terms=901)                             |
| `ECFAST`   | C66742 (No Yes Response; Extensible=No; Terms=4)                     |
| `ECLAT`    | C99073 (Laterality; Extensible=Yes; Terms=7)                         |
| `ECLOC`    | C74456 (Anatomical Location; Extensible=Yes; Terms=1376)             |
| `ECMOOD`   | C125923 (BRIDG Activity Mood; Extensible=Yes; Terms=2)               |
| `ECOCCUR`  | C66742 (No Yes Response; Extensible=No; Terms=4)                     |
| `ECPORTOT` | C99075 (Portion/Totality; Extensible=Yes; Terms=7)                   |
| `ECPRESP`  | C66742 (No Yes Response; Extensible=No; Terms=4)                     |
| `ECPSTRGU` | C71620 (Unit; Extensible=Yes; Terms=901)                             |
| `ECROUTE`  | C66729 (Route of Administration Response; Extensible=Yes; Terms=142) |
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                             |

### EG

_Dataset label:_ **ECG Test Results**\
_Class:_ `Findings`\
_Structure:_ One record per ECG observation per replicate per time point or one
record per ECG observation per beat per visit per subject

| Variable   | Codelist link(s)                                                                                                                                                    |
|------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `EGBLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)                                                                                                                    |
| `EGCLSIG`  | C66742 (No Yes Response; Extensible=No; Terms=4)                                                                                                                    |
| `EGDRVFL`  | C66742 (No Yes Response; Extensible=No; Terms=4)                                                                                                                    |
| `EGEVAL`   | C78735 (Evaluator; Extensible=Yes; Terms=60)                                                                                                                        |
| `EGEVALID` | C96777 (Medical Evaluator Identifier; Extensible=Yes; Terms=56)                                                                                                     |
| `EGLEAD`   | C90013 (ECG Lead; Extensible=Yes; Terms=26)                                                                                                                         |
| `EGLOBXFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                                                                                                                    |
| `EGMETHOD` | C71151 (ECG Test Method; Extensible=Yes; Terms=36)                                                                                                                  |
| `EGORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                                                                                                            |
| `EGPOS`    | C71148 (Position; Extensible=Yes; Terms=17)                                                                                                                         |
| `EGSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                                                                                                                           |
| `EGSTRESC` | C71150 (ECG Result; Extensible=Yes; Terms=235); C120522 (Holter ECG Results; Extensible=Yes; Terms=238); C101834 (Normal Abnormal Response; Extensible=No; Terms=5) |
| `EGSTRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                                                                                                            |
| `EGTEST`   | C71152 (ECG Test Name; Extensible=Yes; Terms=109); C120524 (Holter ECG Test Name; Extensible=Yes; Terms=16)                                                         |
| `EGTESTCD` | C71153 (ECG Test Code; Extensible=Yes; Terms=109); C120523 (Holter ECG Test Code; Extensible=Yes; Terms=16)                                                         |
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                                                                                                                            |

### EX

_Dataset label:_ **Exposure**\
_Class:_ `Interventions`\
_Structure:_ One record per protocol-specified study treatment, constant-dosing
interval, per subject

| Variable   | Codelist link(s)                                                     |
|------------|----------------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                             |
| `EXDIR`    | C99074 (Directionality; Extensible=Yes; Terms=55)                    |
| `EXDOSFRM` | C66726 (Pharmaceutical Dosage Form; Extensible=Yes; Terms=189)       |
| `EXDOSFRQ` | C71113 (Frequency; Extensible=Yes; Terms=102)                        |
| `EXDOSU`   | C71620 (Unit; Extensible=Yes; Terms=901)                             |
| `EXFAST`   | C66742 (No Yes Response; Extensible=No; Terms=4)                     |
| `EXLAT`    | C99073 (Laterality; Extensible=Yes; Terms=7)                         |
| `EXLOC`    | C74456 (Anatomical Location; Extensible=Yes; Terms=1376)             |
| `EXROUTE`  | C66729 (Route of Administration Response; Extensible=Yes; Terms=142) |

### FA

_Dataset label:_ **Findings About Events or Interventions**\
_Class:_ `Findings About`\
_Structure:_ One record per finding, per object, per time point, per visit per
subject

| Variable   | Codelist link(s)                                              |
|------------|---------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                      |
| `FABLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)              |
| `FAEVAL`   | C78735 (Evaluator; Extensible=Yes; Terms=60)                  |
| `FALAT`    | C99073 (Laterality; Extensible=Yes; Terms=7)                  |
| `FALOBXFL` | C66742 (No Yes Response; Extensible=No; Terms=4)              |
| `FALOC`    | C74456 (Anatomical Location; Extensible=Yes; Terms=1376)      |
| `FAORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                      |
| `FASTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                     |
| `FASTRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                      |
| `FATEST`   | C101833 (Findings About Test Name; Extensible=Yes; Terms=221) |
| `FATESTCD` | C101832 (Findings About Test Code; Extensible=Yes; Terms=221) |

### FT

_Dataset label:_ **Functional Tests**\
_Class:_ `Findings`\
_Structure:_ One record per Functional Test finding per time point per visit per
subject

| Variable   | Codelist link(s)                                                |
|------------|-----------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                        |
| `FTBLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)                |
| `FTCAT`    | C115304 (Category of Functional Test; Extensible=Yes; Terms=26) |
| `FTDRVFL`  | C66742 (No Yes Response; Extensible=No; Terms=4)                |
| `FTLOBXFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                |
| `FTMETHOD` | C158113 (QRS Method; Extensible=Yes; Terms=18)                  |
| `FTORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                        |
| `FTPOS`    | C71148 (Position; Extensible=Yes; Terms=17)                     |
| `FTSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                       |
| `FTSTRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                        |

### GF

_Dataset label:_ **Genomics Findings**\
_Class:_ `Findings`\
_Structure:_ One record per finding per observation per biospecimen per subject

| Variable   | Codelist link(s)                                                                          |
|------------|-------------------------------------------------------------------------------------------|
| `GFANMETH` | C181181 (Genomic Findings Analytical Method Calculation Formula; Extensible=Yes; Terms=7) |
| `GFBLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)                                          |
| `GFDRVFL`  | C66742 (No Yes Response; Extensible=No; Terms=4)                                          |
| `GFINHERT` | C181177 (Genomic Inheritability Type Response; Extensible=Yes; Terms=3)                   |
| `GFMETHOD` | C85492 (Method; Extensible=Yes; Terms=504)                                                |
| `GFORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                                  |
| `GFSPEC`   | C111114 (Genetic Sample Type; Extensible=Yes; Terms=33)                                   |
| `GFSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                                                 |
| `GFSTRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                                  |
| `GFSYMTYP` | C181176 (Genomic Symbol Type Response; Extensible=Yes; Terms=23)                          |
| `GFTEST`   | C181179 (Genomic Findings Test Name; Extensible=Yes; Terms=10)                            |
| `GFTESTCD` | C181178 (Genomic Findings Test Code; Extensible=Yes; Terms=10)                            |
| `GFTSTDTL` | C181180 (Genomic Findings Test Detail; Extensible=Yes; Terms=33)                          |

### HO

_Dataset label:_ **Healthcare Encounters**\
_Class:_ `Events`\
_Structure:_ One record per healthcare encounter per subject

| Variable   | Codelist link(s)                                                                   |
|------------|------------------------------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                                           |
| `HODECOD`  | C171444 (Health Care Encounters Dictionary Derived Term; Extensible=Yes; Terms=37) |
| `HOENRTPT` | C66728 (Relation to Reference Period; Extensible=No; Terms=8)                      |
| `HOOCCUR`  | C66742 (No Yes Response; Extensible=No; Terms=4)                                   |
| `HOPRESP`  | C66742 (No Yes Response; Extensible=No; Terms=4)                                   |
| `HOSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                                          |
| `HOSTRTPT` | C66728 (Relation to Reference Period; Extensible=No; Terms=8)                      |

### IE

_Dataset label:_ **Inclusion/Exclusion Criteria Not Met**\
_Class:_ `Findings`\
_Structure:_ One record per inclusion/exclusion criterion not met per subject

| Variable   | Codelist link(s)                                                 |
|------------|------------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                         |
| `IECAT`    | C66797 (Category of Inclusion/Exclusion; Extensible=No; Terms=2) |
| `IEORRES`  | C66742 (No Yes Response; Extensible=No; Terms=4)                 |
| `IESTRESC` | C66742 (No Yes Response; Extensible=No; Terms=4)                 |

### IS

_Dataset label:_ **Immunogenicity Specimen Assessments**\
_Class:_ `Findings`\
_Structure:_ One record per test per visit per subject

| Variable   | Codelist link(s)                                                                                                                |
|------------|---------------------------------------------------------------------------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                                                                                        |
| `ISBDAGNT` | C85491 (Microorganism; Extensible=Yes; Terms=1669); C181169 (Binding Agent for Immunogenicity Tests; Extensible=Yes; Terms=293) |
| `ISBLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)                                                                                |
| `ISDRVFL`  | C66742 (No Yes Response; Extensible=No; Terms=4)                                                                                |
| `ISLOBXFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                                                                                |
| `ISMETHOD` | C85492 (Method; Extensible=Yes; Terms=504)                                                                                      |
| `ISNRIND`  | C78736 (Reference Range Indicator; Extensible=Yes; Terms=4)                                                                     |
| `ISORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                                                                        |
| `ISSPCCND` | C78733 (Specimen Condition; Extensible=Yes; Terms=23)                                                                           |
| `ISSPCUFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                                                                                |
| `ISSPEC`   | C78734 (Specimen Type; Extensible=Yes; Terms=127)                                                                               |
| `ISSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                                                                                       |
| `ISSTRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                                                                        |
| `ISTEST`   | C120526 (Immunogenicity Specimen Assessments Test Name; Extensible=Yes; Terms=83)                                               |
| `ISTESTCD` | C120525 (Immunogenicity Specimen Assessments Test Code; Extensible=Yes; Terms=83)                                               |
| `ISTSTCND` | C181175 (Test Condition Response; Extensible=Yes; Terms=6)                                                                      |
| `ISTSTOPO` | C181170 (Test Operational Objective; Extensible=No; Terms=4)                                                                    |

### LB

_Dataset label:_ **Laboratory Test Results**\
_Class:_ `Findings`\
_Structure:_ One record per lab test per time point per visit per subject

| Variable   | Codelist link(s)                                                                     |
|------------|--------------------------------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                                             |
| `LBANMETH` | C160922 (Laboratory Analytical Method Calculation Formula; Extensible=Yes; Terms=26) |
| `LBBLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)                                     |
| `LBCLSIG`  | C66742 (No Yes Response; Extensible=No; Terms=4)                                     |
| `LBCOLSRT` | C177908 (Collected Summarized Value Type Response; Extensible=Yes; Terms=7)          |
| `LBDRVFL`  | C66742 (No Yes Response; Extensible=No; Terms=4)                                     |
| `LBFAST`   | C66742 (No Yes Response; Extensible=No; Terms=4)                                     |
| `LBLOBXFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                                     |
| `LBMETHOD` | C85492 (Method; Extensible=Yes; Terms=504)                                           |
| `LBNRIND`  | C78736 (Reference Range Indicator; Extensible=Yes; Terms=4)                          |
| `LBORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                             |
| `LBPTFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)                                     |
| `LBRESSCL` | C177910 (Result Scale Response; Extensible=Yes; Terms=5)                             |
| `LBRESTYP` | C179588 (Result Type Response; Extensible=Yes; Terms=61)                             |
| `LBSPCCND` | C78733 (Specimen Condition; Extensible=Yes; Terms=23)                                |
| `LBSPCUFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                                     |
| `LBSPEC`   | C78734 (Specimen Type; Extensible=Yes; Terms=127)                                    |
| `LBSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                                            |
| `LBSTRESC` | C102580 (Laboratory Test Standard Character Result; Extensible=Yes; Terms=14)        |
| `LBSTRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                             |
| `LBTEST`   | C67154 (Laboratory Test Name; Extensible=Yes; Terms=2397)                            |
| `LBTESTCD` | C65047 (Laboratory Test Code; Extensible=Yes; Terms=2397)                            |
| `LBTMTHSN` | C179589 (Test Method Sensitivity; Extensible=Yes; Terms=3)                           |
| `LBTSTCND` | C181175 (Test Condition Response; Extensible=Yes; Terms=6)                           |
| `LBTSTOPO` | C181170 (Test Operational Objective; Extensible=No; Terms=4)                         |

### MB

_Dataset label:_ **Microbiology Specimen**\
_Class:_ `Findings`\
_Structure:_ One record per microbiology specimen finding per time point per
visit per subject

| Variable   | Codelist link(s)                                                      |
|------------|-----------------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                              |
| `MBBLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)                      |
| `MBDIR`    | C99074 (Directionality; Extensible=Yes; Terms=55)                     |
| `MBDRVFL`  | C66742 (No Yes Response; Extensible=No; Terms=4)                      |
| `MBFAST`   | C66742 (No Yes Response; Extensible=No; Terms=4)                      |
| `MBLAT`    | C99073 (Laterality; Extensible=Yes; Terms=7)                          |
| `MBLOBXFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                      |
| `MBLOC`    | C74456 (Anatomical Location; Extensible=Yes; Terms=1376)              |
| `MBMETHOD` | C85492 (Method; Extensible=Yes; Terms=504)                            |
| `MBORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                              |
| `MBSPCCND` | C78733 (Specimen Condition; Extensible=Yes; Terms=23)                 |
| `MBSPEC`   | C78734 (Specimen Type; Extensible=Yes; Terms=127)                     |
| `MBSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                             |
| `MBSTRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                              |
| `MBTEST`   | C120528 (Microbiology Test Name; Extensible=Yes; Terms=490)           |
| `MBTESTCD` | C120527 (Microbiology Test Code; Extensible=Yes; Terms=490)           |
| `MBTSTDTL` | C174225 (Microbiology Findings Test Details; Extensible=Yes; Terms=8) |

### MH

_Dataset label:_ **Medical History**\
_Class:_ `Events`\
_Structure:_ One record per medical history event per subject

| Variable   | Codelist link(s)                                                    |
|------------|---------------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                            |
| `MHENRF`   | C66728 (Relation to Reference Period; Extensible=No; Terms=8)       |
| `MHENRTPT` | C66728 (Relation to Reference Period; Extensible=No; Terms=8)       |
| `MHEVDTYP` | C124301 (Medical History Event Date Type; Extensible=Yes; Terms=16) |
| `MHOCCUR`  | C66742 (No Yes Response; Extensible=No; Terms=4)                    |
| `MHPRESP`  | C66742 (No Yes Response; Extensible=No; Terms=4)                    |
| `MHSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                           |

### MI

_Dataset label:_ **Microscopic Findings**\
_Class:_ `Findings`\
_Structure:_ One record per finding per specimen per subject

| Variable   | Codelist link(s)                                                         |
|------------|--------------------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                                 |
| `MIBLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)                         |
| `MIDIR`    | C99074 (Directionality; Extensible=Yes; Terms=55)                        |
| `MIEVAL`   | C78735 (Evaluator; Extensible=Yes; Terms=60)                             |
| `MILAT`    | C99073 (Laterality; Extensible=Yes; Terms=7)                             |
| `MILOBXFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                         |
| `MILOC`    | C74456 (Anatomical Location; Extensible=Yes; Terms=1376)                 |
| `MIMETHOD` | C85492 (Method; Extensible=Yes; Terms=504)                               |
| `MIORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                 |
| `MISPCCND` | C78733 (Specimen Condition; Extensible=Yes; Terms=23)                    |
| `MISPEC`   | C78734 (Specimen Type; Extensible=Yes; Terms=127)                        |
| `MISTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                                |
| `MISTRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                 |
| `MITEST`   | C132262 (SDTM Microscopic Findings Test Name; Extensible=Yes; Terms=206) |
| `MITESTCD` | C132263 (SDTM Microscopic Findings Test Code; Extensible=Yes; Terms=206) |
| `MITSTDTL` | C125922 (Microscopic Findings Test Details; Extensible=Yes; Terms=63)    |

### MK

_Dataset label:_ **Musculoskeletal System Findings**\
_Class:_ `Findings`\
_Structure:_ One record per assessment per visit per subject

| Variable   | Codelist link(s)                                                             |
|------------|------------------------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                                     |
| `MKBLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)                             |
| `MKDIR`    | C99074 (Directionality; Extensible=Yes; Terms=55)                            |
| `MKDRVFL`  | C66742 (No Yes Response; Extensible=No; Terms=4)                             |
| `MKEVAL`   | C78735 (Evaluator; Extensible=Yes; Terms=60)                                 |
| `MKEVALID` | C96777 (Medical Evaluator Identifier; Extensible=Yes; Terms=56)              |
| `MKLAT`    | C99073 (Laterality; Extensible=Yes; Terms=7)                                 |
| `MKLOBXFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                             |
| `MKLOC`    | C74456 (Anatomical Location; Extensible=Yes; Terms=1376)                     |
| `MKMETHOD` | C85492 (Method; Extensible=Yes; Terms=504)                                   |
| `MKORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                     |
| `MKPOS`    | C71148 (Position; Extensible=Yes; Terms=17)                                  |
| `MKSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                                    |
| `MKSTRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                     |
| `MKTEST`   | C127270 (Musculoskeletal System Finding Test Name; Extensible=Yes; Terms=50) |
| `MKTESTCD` | C127269 (Musculoskeletal System Finding Test Code; Extensible=Yes; Terms=50) |

### ML

_Dataset label:_ **Meal Data**\
_Class:_ `Interventions`\
_Structure:_ One record per food product occurrence or constant intake interval
per subject

| Variable   | Codelist link(s)                                               |
|------------|----------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                       |
| `MLDOSFRM` | C66726 (Pharmaceutical Dosage Form; Extensible=Yes; Terms=189) |
| `MLDOSU`   | C71620 (Unit; Extensible=Yes; Terms=901)                       |
| `MLOCCUR`  | C66742 (No Yes Response; Extensible=No; Terms=4)               |
| `MLPRESP`  | C66742 (No Yes Response; Extensible=No; Terms=4)               |
| `MLSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                      |

### MS

_Dataset label:_ **Microbiology Susceptibility**\
_Class:_ `Findings`\
_Structure:_ One record per microbiology susceptibility test (or other
organism-related finding) per organism found in MB

| Variable   | Codelist link(s)                                                                     |
|------------|--------------------------------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                                             |
| `MSACPTFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                                     |
| `MSBLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)                                     |
| `MSCONCU`  | C71620 (Unit; Extensible=Yes; Terms=901)                                             |
| `MSDIR`    | C99074 (Directionality; Extensible=Yes; Terms=55)                                    |
| `MSDRVFL`  | C66742 (No Yes Response; Extensible=No; Terms=4)                                     |
| `MSEVAL`   | C78735 (Evaluator; Extensible=Yes; Terms=60)                                         |
| `MSEVALID` | C96777 (Medical Evaluator Identifier; Extensible=Yes; Terms=56)                      |
| `MSFAST`   | C66742 (No Yes Response; Extensible=No; Terms=4)                                     |
| `MSLAT`    | C99073 (Laterality; Extensible=Yes; Terms=7)                                         |
| `MSLOBXFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                                     |
| `MSLOC`    | C74456 (Anatomical Location; Extensible=Yes; Terms=1376)                             |
| `MSMETHOD` | C85492 (Method; Extensible=Yes; Terms=504)                                           |
| `MSNRIND`  | C78736 (Reference Range Indicator; Extensible=Yes; Terms=4)                          |
| `MSORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                             |
| `MSRESCAT` | C85495 (Microbiology Susceptibility Testing Result Category; Extensible=No; Terms=9) |
| `MSSPCCND` | C78733 (Specimen Condition; Extensible=Yes; Terms=23)                                |
| `MSSPEC`   | C78734 (Specimen Type; Extensible=Yes; Terms=127)                                    |
| `MSSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                                            |
| `MSSTRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                             |
| `MSTEST`   | C128687 (Microbiology Susceptibility Test Name; Extensible=Yes; Terms=18)            |
| `MSTESTCD` | C128688 (Microbiology Susceptibility Test Code; Extensible=Yes; Terms=18)            |

### NV

_Dataset label:_ **Nervous System Findings**\
_Class:_ `Findings`\
_Structure:_ One record per finding per location per time point per visit per
subject

| Variable   | Codelist link(s)                                                      |
|------------|-----------------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                              |
| `NVBLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)                      |
| `NVDIR`    | C99074 (Directionality; Extensible=Yes; Terms=55)                     |
| `NVDRVFL`  | C66742 (No Yes Response; Extensible=No; Terms=4)                      |
| `NVEVAL`   | C78735 (Evaluator; Extensible=Yes; Terms=60)                          |
| `NVEVALID` | C96777 (Medical Evaluator Identifier; Extensible=Yes; Terms=56)       |
| `NVLAT`    | C99073 (Laterality; Extensible=Yes; Terms=7)                          |
| `NVLOBXFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                      |
| `NVLOC`    | C74456 (Anatomical Location; Extensible=Yes; Terms=1376)              |
| `NVMETHOD` | C85492 (Method; Extensible=Yes; Terms=504)                            |
| `NVORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                              |
| `NVSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                             |
| `NVSTRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                              |
| `NVTEST`   | C116103 (Nervous System Findings Test Name; Extensible=Yes; Terms=75) |
| `NVTESTCD` | C116104 (Nervous System Findings Test Code; Extensible=Yes; Terms=75) |

### OE

_Dataset label:_ **Ophthalmic Examinations**\
_Class:_ `Findings`\
_Structure:_ One record per ophthalmic finding per method per location, per time
point per visit per subject

| Variable   | Codelist link(s)                                                              |
|------------|-------------------------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                                      |
| `FOCID`    | C119013 (Ophthalmic Focus of Study Specific Interest; Extensible=No; Terms=3) |
| `OEACPTFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                              |
| `OEBLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)                              |
| `OEDIR`    | C99074 (Directionality; Extensible=Yes; Terms=55)                             |
| `OEDRVFL`  | C66742 (No Yes Response; Extensible=No; Terms=4)                              |
| `OEEVAL`   | C78735 (Evaluator; Extensible=Yes; Terms=60)                                  |
| `OEEVALID` | C96777 (Medical Evaluator Identifier; Extensible=Yes; Terms=56)               |
| `OELAT`    | C99073 (Laterality; Extensible=Yes; Terms=7)                                  |
| `OELOBXFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                              |
| `OELOC`    | C74456 (Anatomical Location; Extensible=Yes; Terms=1376)                      |
| `OEMETHOD` | C85492 (Method; Extensible=Yes; Terms=504)                                    |
| `OENRIND`  | C78736 (Reference Range Indicator; Extensible=Yes; Terms=4)                   |
| `OEORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                      |
| `OEPORTOT` | C99075 (Portion/Totality; Extensible=Yes; Terms=7)                            |
| `OESTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                                     |
| `OESTRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                      |
| `OETEST`   | C117742 (Ophthalmic Exam Test Name; Extensible=Yes; Terms=53)                 |
| `OETESTCD` | C117743 (Ophthalmic Exam Test Code; Extensible=Yes; Terms=53)                 |

### OI

_Dataset label:_ **Non-host Organism Identifiers**\
_Class:_ `Study Reference`\
_Structure:_ One record per taxon per non-host organism

| Variable   | Codelist link(s)                                                                 |
|------------|----------------------------------------------------------------------------------|
| `OIPARM`   | C179590 (Non-host Organism Identifier Parameters; Extensible=Yes; Terms=16)      |
| `OIPARMCD` | C179591 (Non-host Organism Identifier Parameters Code; Extensible=Yes; Terms=16) |

### PC

_Dataset label:_ **Pharmacokinetics Concentrations**\
_Class:_ `Findings`\
_Structure:_ One record per sample characteristic or time-point concentration
per reference time point or per analyte per subject

| Variable   | Codelist link(s)                                        |
|------------|---------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                |
| `PCDRVFL`  | C66742 (No Yes Response; Extensible=No; Terms=4)        |
| `PCFAST`   | C66742 (No Yes Response; Extensible=No; Terms=4)        |
| `PCMETHOD` | C85492 (Method; Extensible=Yes; Terms=504)              |
| `PCORRESU` | C85494 (PK Units of Measure; Extensible=Yes; Terms=598) |
| `PCSPCCND` | C78733 (Specimen Condition; Extensible=Yes; Terms=23)   |
| `PCSPEC`   | C78734 (Specimen Type; Extensible=Yes; Terms=127)       |
| `PCSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)               |
| `PCSTRESU` | C85494 (PK Units of Measure; Extensible=Yes; Terms=598) |

### PE

_Dataset label:_ **Physical Examination**\
_Class:_ `Findings`\
_Structure:_ One record per body system or abnormality per visit per subject

| Variable   | Codelist link(s)                                         |
|------------|----------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                 |
| `PEBLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)         |
| `PEEVAL`   | C78735 (Evaluator; Extensible=Yes; Terms=60)             |
| `PELAT`    | C99073 (Laterality; Extensible=Yes; Terms=7)             |
| `PELOBXFL` | C66742 (No Yes Response; Extensible=No; Terms=4)         |
| `PELOC`    | C74456 (Anatomical Location; Extensible=Yes; Terms=1376) |
| `PEMETHOD` | C85492 (Method; Extensible=Yes; Terms=504)               |
| `PEORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                 |
| `PESTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                |

### PP

_Dataset label:_ **Pharmacokinetics Parameters**\
_Class:_ `Findings`\
_Structure:_ One record per PK parameter per time-concentration profile per
modeling method per subject

| Variable   | Codelist link(s)                                                                                                                                                                                                                                                                                                                         |
|------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                                                                                                                                                                                                                                                                                                 |
| `PPANMETH` | C172330 (PK Analytical Method; Extensible=Yes; Terms=4)                                                                                                                                                                                                                                                                                  |
| `PPORRESU` | C85494 (PK Units of Measure; Extensible=Yes; Terms=598); C128684 (PK Units of Measure - Weight g; Extensible=Yes; Terms=57); C128683 (PK Units of Measure - Weight kg; Extensible=Yes; Terms=54); C128685 (PK Units of Measure - Dose mg; Extensible=Yes; Terms=154); C128686 (PK Units of Measure - Dose ug; Extensible=Yes; Terms=132) |
| `PPSPEC`   | C78734 (Specimen Type; Extensible=Yes; Terms=127)                                                                                                                                                                                                                                                                                        |
| `PPSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                                                                                                                                                                                                                                                                                                |
| `PPSTRESU` | C85494 (PK Units of Measure; Extensible=Yes; Terms=598); C128684 (PK Units of Measure - Weight g; Extensible=Yes; Terms=57); C128683 (PK Units of Measure - Weight kg; Extensible=Yes; Terms=54); C128685 (PK Units of Measure - Dose mg; Extensible=Yes; Terms=154); C128686 (PK Units of Measure - Dose ug; Extensible=Yes; Terms=132) |
| `PPTEST`   | C85493 (PK Parameters; Extensible=Yes; Terms=388)                                                                                                                                                                                                                                                                                        |
| `PPTESTCD` | C85839 (PK Parameters Code; Extensible=Yes; Terms=388)                                                                                                                                                                                                                                                                                   |

### PR

_Dataset label:_ **Procedures**\
_Class:_ `Interventions`\
_Structure:_ One record per recorded procedure per occurrence per subject

| Variable   | Codelist link(s)                                                     |
|------------|----------------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                             |
| `PRDECOD`  | C101858 (Procedure; Extensible=Yes; Terms=145)                       |
| `PRDIR`    | C99074 (Directionality; Extensible=Yes; Terms=55)                    |
| `PRDOSFRM` | C66726 (Pharmaceutical Dosage Form; Extensible=Yes; Terms=189)       |
| `PRDOSFRQ` | C71113 (Frequency; Extensible=Yes; Terms=102)                        |
| `PRDOSU`   | C71620 (Unit; Extensible=Yes; Terms=901)                             |
| `PRENRTPT` | C66728 (Relation to Reference Period; Extensible=No; Terms=8)        |
| `PRLAT`    | C99073 (Laterality; Extensible=Yes; Terms=7)                         |
| `PRLOC`    | C74456 (Anatomical Location; Extensible=Yes; Terms=1376)             |
| `PROCCUR`  | C66742 (No Yes Response; Extensible=No; Terms=4)                     |
| `PRPORTOT` | C99075 (Portion/Totality; Extensible=Yes; Terms=7)                   |
| `PRPRESP`  | C66742 (No Yes Response; Extensible=No; Terms=4)                     |
| `PRROUTE`  | C66729 (Route of Administration Response; Extensible=Yes; Terms=142) |
| `PRSTRTPT` | C66728 (Relation to Reference Period; Extensible=No; Terms=8)        |

### QS

_Dataset label:_ **Questionnaires**\
_Class:_ `Findings`\
_Structure:_ One record per questionnaire per question per time point per visit
per subject

| Variable   | Codelist link(s)                                               |
|------------|----------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                       |
| `QSBLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)               |
| `QSCAT`    | C100129 (Category of Questionnaire; Extensible=Yes; Terms=299) |
| `QSDRVFL`  | C66742 (No Yes Response; Extensible=No; Terms=4)               |
| `QSLOBXFL` | C66742 (No Yes Response; Extensible=No; Terms=4)               |
| `QSMETHOD` | C158113 (QRS Method; Extensible=Yes; Terms=18)                 |
| `QSORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                       |
| `QSSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                      |
| `QSSTRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                       |

### RE

_Dataset label:_ **Respiratory System Findings**\
_Class:_ `Findings`\
_Structure:_ One record per finding or result per time point per visit per
subject

| Variable   | Codelist link(s)                                                |
|------------|-----------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                        |
| `REBLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)                |
| `REDIR`    | C99074 (Directionality; Extensible=Yes; Terms=55)               |
| `REDRVFL`  | C66742 (No Yes Response; Extensible=No; Terms=4)                |
| `REEVAL`   | C78735 (Evaluator; Extensible=Yes; Terms=60)                    |
| `REEVALID` | C96777 (Medical Evaluator Identifier; Extensible=Yes; Terms=56) |
| `RELAT`    | C99073 (Laterality; Extensible=Yes; Terms=7)                    |
| `RELOBXFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                |
| `RELOC`    | C74456 (Anatomical Location; Extensible=Yes; Terms=1376)        |
| `REMETHOD` | C85492 (Method; Extensible=Yes; Terms=504)                      |
| `REORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                        |
| `REPOS`    | C71148 (Position; Extensible=Yes; Terms=17)                     |
| `RESTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                       |
| `RESTRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                        |
| `RETEST`   | C111107 (Respiratory Test Name; Extensible=Yes; Terms=135)      |
| `RETESTCD` | C111106 (Respiratory Test Code; Extensible=Yes; Terms=135)      |

### RELREC

_Dataset label:_ **Related Records**\
_Class:_ `Relationship`\
_Structure:_ One record per related record, group of records or dataset

| Variable  | Codelist link(s)                                            |
|-----------|-------------------------------------------------------------|
| `RDOMAIN` | C66734 (SDTM Domain Abbreviation; Extensible=Yes; Terms=83) |
| `RELTYPE` | C78737 (Relationship Type; Extensible=No; Terms=2)          |

### RELSPEC

_Dataset label:_ **Related Specimens**\
_Class:_ `Relationship`\
_Structure:_ One record per specimen identifier per subject

| Variable | Codelist link(s)                                                                                           |
|----------|------------------------------------------------------------------------------------------------------------|
| `SPEC`   | C78734 (Specimen Type; Extensible=Yes; Terms=127); C111114 (Genetic Sample Type; Extensible=Yes; Terms=33) |

### RELSUB

_Dataset label:_ **Related Subjects**\
_Class:_ `Relationship`\
_Structure:_ One record per relationship per related subject per subject

| Variable | Codelist link(s)                                            |
|----------|-------------------------------------------------------------|
| `SREL`   | C100130 (Relationship to Subject; Extensible=Yes; Terms=83) |

### RP

_Dataset label:_ **Reproductive System Findings**\
_Class:_ `Findings`\
_Structure:_ One record per finding or result per time point per visit per
subject

| Variable   | Codelist link(s)                                                           |
|------------|----------------------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                                   |
| `RPBLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)                           |
| `RPDRVFL`  | C66742 (No Yes Response; Extensible=No; Terms=4)                           |
| `RPLOBXFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                           |
| `RPORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                   |
| `RPSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                                  |
| `RPSTRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                   |
| `RPTEST`   | C106478 (Reproductive System Findings Test Name; Extensible=Yes; Terms=99) |
| `RPTESTCD` | C106479 (Reproductive System Findings Test Code; Extensible=Yes; Terms=99) |

### RS

_Dataset label:_ **Disease Response and Clin Classification**\
_Class:_ `Findings`\
_Structure:_ One record per response assessment or clinical classification
assessment per time point per visit per subject per assessor per medical
evaluator

| Variable   | Codelist link(s)                                                                                                                                      |
|------------|-------------------------------------------------------------------------------------------------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                                                                                                              |
| `RSACPTFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                                                                                                      |
| `RSBLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)                                                                                                      |
| `RSCAT`    | C124298 (Category of Oncology Response Assessment; Extensible=Yes; Terms=93); C118971 (Category of Clinical Classification; Extensible=Yes; Terms=85) |
| `RSDRVFL`  | C66742 (No Yes Response; Extensible=No; Terms=4)                                                                                                      |
| `RSENRTPT` | C66728 (Relation to Reference Period; Extensible=No; Terms=8)                                                                                         |
| `RSEVAL`   | C78735 (Evaluator; Extensible=Yes; Terms=60)                                                                                                          |
| `RSEVALID` | C96777 (Medical Evaluator Identifier; Extensible=Yes; Terms=56)                                                                                       |
| `RSLOBXFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                                                                                                      |
| `RSMETHOD` | C158113 (QRS Method; Extensible=Yes; Terms=18)                                                                                                        |
| `RSORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                                                                                              |
| `RSSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                                                                                                             |
| `RSSTRESC` | C96785 (Oncology Response Assessment Result; Extensible=Yes; Terms=95)                                                                                |
| `RSSTRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                                                                                              |
| `RSSTRTPT` | C66728 (Relation to Reference Period; Extensible=No; Terms=8)                                                                                         |
| `RSTEST`   | C96781 (Oncology Response Assessment Test Name; Extensible=Yes; Terms=47)                                                                             |
| `RSTESTCD` | C96782 (Oncology Response Assessment Test Code; Extensible=Yes; Terms=47)                                                                             |

### SC

_Dataset label:_ **Subject Characteristics**\
_Class:_ `Findings`\
_Structure:_ One record per characteristic per visit per subject.

| Variable   | Codelist link(s)                                                     |
|------------|----------------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                             |
| `SCORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                             |
| `SCSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                            |
| `SCSTRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                             |
| `SCTEST`   | C103330 (Subject Characteristic Test Name; Extensible=Yes; Terms=56) |
| `SCTESTCD` | C74559 (Subject Characteristic Test Code; Extensible=Yes; Terms=56)  |

### SE

_Dataset label:_ **Subject Elements**\
_Class:_ `Special-Purpose`\
_Structure:_ One record per actual Element per subject

| Variable | Codelist link(s)                         |
|----------|------------------------------------------|
| `EPOCH`  | C99079 (Epoch; Extensible=Yes; Terms=13) |

### SR

_Dataset label:_ **Skin Response**\
_Class:_ `Findings About`\
_Structure:_ One record per finding, per object, per time point, per visit per
subject

| Variable   | Codelist link(s)                                            |
|------------|-------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                    |
| `SRBLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)            |
| `SREVAL`   | C78735 (Evaluator; Extensible=Yes; Terms=60)                |
| `SRLAT`    | C99073 (Laterality; Extensible=Yes; Terms=7)                |
| `SRLOBXFL` | C66742 (No Yes Response; Extensible=No; Terms=4)            |
| `SRLOC`    | C74456 (Anatomical Location; Extensible=Yes; Terms=1376)    |
| `SRMETHOD` | C85492 (Method; Extensible=Yes; Terms=504)                  |
| `SRORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                    |
| `SRSPEC`   | C78734 (Specimen Type; Extensible=Yes; Terms=127)           |
| `SRSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                   |
| `SRSTRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                    |
| `SRTEST`   | C112023 (Skin Response Test Name; Extensible=Yes; Terms=13) |
| `SRTESTCD` | C112024 (Skin Response Test Code; Extensible=Yes; Terms=13) |

### SS

_Dataset label:_ **Subject Status**\
_Class:_ `Findings`\
_Structure:_ One record per status per visit per subject

| Variable   | Codelist link(s)                                            |
|------------|-------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                    |
| `SSEVAL`   | C78735 (Evaluator; Extensible=Yes; Terms=60)                |
| `SSSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                   |
| `SSSTRESC` | C124304 (Subject Status Response; Extensible=Yes; Terms=3)  |
| `SSTEST`   | C124306 (Subject Status Test Name; Extensible=Yes; Terms=3) |
| `SSTESTCD` | C124305 (Subject Status Test Code; Extensible=Yes; Terms=3) |

### SU

_Dataset label:_ **Substance Use**\
_Class:_ `Interventions`\
_Structure:_ One record per substance type per reported occurrence per subject

| Variable   | Codelist link(s)                                                     |
|------------|----------------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                             |
| `SUDOSFRM` | C66726 (Pharmaceutical Dosage Form; Extensible=Yes; Terms=189)       |
| `SUDOSFRQ` | C71113 (Frequency; Extensible=Yes; Terms=102)                        |
| `SUDOSU`   | C71620 (Unit; Extensible=Yes; Terms=901)                             |
| `SUENRF`   | C66728 (Relation to Reference Period; Extensible=No; Terms=8)        |
| `SUENRTPT` | C66728 (Relation to Reference Period; Extensible=No; Terms=8)        |
| `SUOCCUR`  | C66742 (No Yes Response; Extensible=No; Terms=4)                     |
| `SUPRESP`  | C66742 (No Yes Response; Extensible=No; Terms=4)                     |
| `SUROUTE`  | C66729 (Route of Administration Response; Extensible=Yes; Terms=142) |
| `SUSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                            |
| `SUSTRF`   | C66728 (Relation to Reference Period; Extensible=No; Terms=8)        |
| `SUSTRTPT` | C66728 (Relation to Reference Period; Extensible=No; Terms=8)        |

### SUPPQUAL

_Dataset label:_ **Supplemental Qualifiers for [domain name]**\
_Class:_ `Relationship`\
_Structure:_ One record per supplemental qualifier per related parent domain
record(s)

| Variable  | Codelist link(s)                                            |
|-----------|-------------------------------------------------------------|
| `QEVAL`   | C78735 (Evaluator; Extensible=Yes; Terms=60)                |
| `RDOMAIN` | C66734 (SDTM Domain Abbreviation; Extensible=Yes; Terms=83) |

### SV

_Dataset label:_ **Subject Visits**\
_Class:_ `Special-Purpose`\
_Structure:_ One record per actual or planned visit per subject

| Variable   | Codelist link(s)                                           |
|------------|------------------------------------------------------------|
| `SVCNTMOD` | C171445 (Mode of Subject Contact; Extensible=Yes; Terms=9) |
| `SVEPCHGI` | C66742 (No Yes Response; Extensible=No; Terms=4)           |
| `SVOCCUR`  | C66742 (No Yes Response; Extensible=No; Terms=4)           |
| `SVPRESP`  | C66742 (No Yes Response; Extensible=No; Terms=4)           |

### TA

_Dataset label:_ **Trial Arms**\
_Class:_ `Trial Design`\
_Structure:_ One record per planned Element per Arm

| Variable | Codelist link(s)                         |
|----------|------------------------------------------|
| `EPOCH`  | C99079 (Epoch; Extensible=Yes; Terms=13) |

### TI

_Dataset label:_ **Trial Inclusion/Exclusion Criteria**\
_Class:_ `Trial Design`\
_Structure:_ One record per I/E criterion

| Variable | Codelist link(s)                                                 |
|----------|------------------------------------------------------------------|
| `IECAT`  | C66797 (Category of Inclusion/Exclusion; Extensible=No; Terms=2) |

### TM

_Dataset label:_ **Trial Disease Milestones**\
_Class:_ `Trial Design`\
_Structure:_ One record per Disease Milestone type

| Variable | Codelist link(s)                                 |
|----------|--------------------------------------------------|
| `TMRPT`  | C66742 (No Yes Response; Extensible=No; Terms=4) |

### TR

_Dataset label:_ **Tumor/Lesion Results**\
_Class:_ `Findings`\
_Structure:_ One record per tumor measurement/assessment per visit per subject
per assessor

| Variable   | Codelist link(s)                                                           |
|------------|----------------------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                                   |
| `TRACPTFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                           |
| `TRBLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)                           |
| `TREVAL`   | C78735 (Evaluator; Extensible=Yes; Terms=60)                               |
| `TREVALID` | C96777 (Medical Evaluator Identifier; Extensible=Yes; Terms=56)            |
| `TRLOBXFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                           |
| `TRMETHOD` | C85492 (Method; Extensible=Yes; Terms=504)                                 |
| `TRORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                   |
| `TRSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                                  |
| `TRSTRESC` | C124309 (Tumor or Lesion Properties Test Result; Extensible=Yes; Terms=22) |
| `TRSTRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                                   |
| `TRTEST`   | C96778 (Tumor or Lesion Properties Test Name; Extensible=Yes; Terms=72)    |
| `TRTESTCD` | C96779 (Tumor or Lesion Properties Test Code; Extensible=Yes; Terms=72)    |

### TS

_Dataset label:_ **Trial Summary**\
_Class:_ `Trial Design`\
_Structure:_ One record per trial summary parameter value

| Variable   | Codelist link(s)                                                      |
|------------|-----------------------------------------------------------------------|
| `TSPARM`   | C67152 (Trial Summary Parameter Test Name; Extensible=Yes; Terms=126) |
| `TSPARMCD` | C66738 (Trial Summary Parameter Test Code; Extensible=Yes; Terms=126) |
| `TSVCDREF` | C66788 (Dictionary Name; Extensible=Yes; Terms=14)                    |

### TU

_Dataset label:_ **Tumor/Lesion Identification**\
_Class:_ `Findings`\
_Structure:_ One record per identified tumor per subject per assessor

| Variable   | Codelist link(s)                                                                |
|------------|---------------------------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                                        |
| `TUACPTFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                                |
| `TUBLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)                                |
| `TUDIR`    | C99074 (Directionality; Extensible=Yes; Terms=55)                               |
| `TUEVAL`   | C78735 (Evaluator; Extensible=Yes; Terms=60)                                    |
| `TUEVALID` | C96777 (Medical Evaluator Identifier; Extensible=Yes; Terms=56)                 |
| `TULAT`    | C99073 (Laterality; Extensible=Yes; Terms=7)                                    |
| `TULOBXFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                                |
| `TULOC`    | C74456 (Anatomical Location; Extensible=Yes; Terms=1376)                        |
| `TUMETHOD` | C85492 (Method; Extensible=Yes; Terms=504)                                      |
| `TUPORTOT` | C99075 (Portion/Totality; Extensible=Yes; Terms=7)                              |
| `TUSTRESC` | C123650 (Tumor or Lesion Identification Test Results; Extensible=Yes; Terms=28) |
| `TUTEST`   | C96783 (Tumor or Lesion Identification Test Name; Extensible=Yes; Terms=19)     |
| `TUTESTCD` | C96784 (Tumor or Lesion Identification Test Code; Extensible=Yes; Terms=19)     |

### UR

_Dataset label:_ **Urinary System Findings**\
_Class:_ `Findings`\
_Structure:_ One record per finding per location per per visit per subject

| Variable   | Codelist link(s)                                                |
|------------|-----------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                        |
| `URBLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)                |
| `URDIR`    | C99074 (Directionality; Extensible=Yes; Terms=55)               |
| `URDRVFL`  | C66742 (No Yes Response; Extensible=No; Terms=4)                |
| `UREVAL`   | C78735 (Evaluator; Extensible=Yes; Terms=60)                    |
| `UREVALID` | C96777 (Medical Evaluator Identifier; Extensible=Yes; Terms=56) |
| `URLAT`    | C99073 (Laterality; Extensible=Yes; Terms=7)                    |
| `URLOBXFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                |
| `URLOC`    | C74456 (Anatomical Location; Extensible=Yes; Terms=1376)        |
| `URMETHOD` | C85492 (Method; Extensible=Yes; Terms=504)                      |
| `URORRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                        |
| `URSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                       |
| `URSTRESU` | C71620 (Unit; Extensible=Yes; Terms=901)                        |
| `URTEST`   | C129941 (Urinary System Test Name; Extensible=Yes; Terms=9)     |
| `URTESTCD` | C129942 (Urinary System Test Code; Extensible=Yes; Terms=9)     |

### VS

_Dataset label:_ **Vital Signs**\
_Class:_ `Findings`\
_Structure:_ One record per vital sign measurement per time point per visit per
subject

| Variable   | Codelist link(s)                                                 |
|------------|------------------------------------------------------------------|
| `EPOCH`    | C99079 (Epoch; Extensible=Yes; Terms=13)                         |
| `VSBLFL`   | C66742 (No Yes Response; Extensible=No; Terms=4)                 |
| `VSCLSIG`  | C66742 (No Yes Response; Extensible=No; Terms=4)                 |
| `VSDRVFL`  | C66742 (No Yes Response; Extensible=No; Terms=4)                 |
| `VSLAT`    | C99073 (Laterality; Extensible=Yes; Terms=7)                     |
| `VSLOBXFL` | C66742 (No Yes Response; Extensible=No; Terms=4)                 |
| `VSLOC`    | C74456 (Anatomical Location; Extensible=Yes; Terms=1376)         |
| `VSORRESU` | C66770 (Units for Vital Signs Results; Extensible=Yes; Terms=29) |
| `VSPOS`    | C71148 (Position; Extensible=Yes; Terms=17)                      |
| `VSSTAT`   | C66789 (Not Done; Extensible=No; Terms=1)                        |
| `VSSTRESU` | C66770 (Units for Vital Signs Results; Extensible=Yes; Terms=29) |
| `VSTEST`   | C67153 (Vital Signs Test Name; Extensible=Yes; Terms=72)         |
| `VSTESTCD` | C66741 (Vital Signs Test Code; Extensible=Yes; Terms=72)         |
