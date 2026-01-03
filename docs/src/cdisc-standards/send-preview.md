# SEND (Preview)

The Standard for Exchange of Nonclinical Data (SEND) extends SDTM for animal studies.

> [!NOTE]
> SEND support is planned for a future release of Trial Submission Studio.

## What is SEND?

SEND (Standard for Exchange of Nonclinical Data) provides:

- Standardized format for nonclinical (animal) study data
- Based on SDTM structure
- Required for FDA nonclinical submissions
- Supports toxicology and pharmacology studies

## SEND vs SDTM

| Aspect           | SDTM             | SEND                   |
|------------------|------------------|------------------------|
| **Subjects**     | Human            | Animal                 |
| **Studies**      | Clinical trials  | Nonclinical studies    |
| **Domains**      | Clinical domains | Nonclinical domains    |
| **Requirements** | NDA, BLA         | IND, NDA (nonclinical) |

## SEND Domains

### Special Purpose

| Domain | Name           |
|--------|----------------|
| DM     | Demographics   |
| DS     | Disposition    |
| TA     | Trial Arms     |
| TE     | Trial Elements |
| TS     | Trial Summary  |
| TX     | Trial Sets     |

### Findings

| Domain | Name                           |
|--------|--------------------------------|
| BW     | Body Weight                    |
| BG     | Body Weight Gain               |
| CL     | Clinical Observations          |
| DD     | Death Diagnosis                |
| FW     | Food/Water Consumption         |
| LB     | Laboratory Results             |
| MA     | Macroscopic Findings           |
| MI     | Microscopic Findings           |
| OM     | Organ Measurements             |
| PC     | Pharmacokinetic Concentrations |
| PP     | Pharmacokinetic Parameters     |
| TF     | Tumor Findings                 |
| VS     | Vital Signs                    |

### Interventions

| Domain | Name     |
|--------|----------|
| EX     | Exposure |

## Key Differences from SDTM

### Subject Identification

- USUBJID format differs for animals
- Species and strain information required
- Group/cage identification

### Domain-Specific Variables

SEND includes nonclinical-specific variables:

- Species, strain, sex
- Dose group information
- Study day calculations
- Sacrifice/necropsy data

### Controlled Terminology

SEND uses specific CT:

- Animal species
- Strain/substrain
- Route of administration (nonclinical)
- Specimen types

## Planned Features

When SEND support is added, Trial Submission Studio will provide:

### SEND Import/Mapping

- Support nonclinical data formats
- Map to SEND domains
- Handle group-level data

### SEND Validation

- SEND-IG compliance checking
- Nonclinical-specific rules
- Controlled terminology for SEND

### SEND Export

- XPT V5 format
- Define-XML for SEND
- Submission-ready packages

## Current Workarounds

Until SEND support is available:

1. **Manual Mapping**
    - Use current SDTM workflow
    - Manually adjust for SEND differences
    - Export to XPT

2. **External Tools**
    - Use specialized nonclinical tools
    - Validate with SEND validators

## SEND Versions

| Version    | Description          |
|------------|----------------------|
| SEND 3.1.1 | Current FDA standard |
| SEND 3.1   | Previous version     |
| SEND 3.0   | Initial release      |

## Resources

### CDISC SEND Resources

- [SEND Implementation Guide](https://www.cdisc.org/standards/foundational/send)
- [CDISC Library - SEND](https://library.cdisc.org/)

### FDA Resources

- [FDA SEND Requirements](https://www.fda.gov/industry/study-data-standards-resources)

## Stay Updated

- Check the [Roadmap](../reference/roadmap.md) for SEND progress
- Watch for announcements on [GitHub](https://github.com/rubentalstra/Trial-Submission-Studio)
