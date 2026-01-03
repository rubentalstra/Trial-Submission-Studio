# ADaM (Preview)

The Analysis Data Model (ADaM) defines standards for analysis-ready datasets.

> [!NOTE]
> ADaM support is planned for a future release of Trial Submission Studio.

## What is ADaM?

ADaM (Analysis Data Model) provides:

- Standards for analysis datasets
- Derived from SDTM data
- Ready for statistical analysis
- Required for FDA submissions

## ADaM vs SDTM

| Aspect        | SDTM                | ADaM                 |
|---------------|---------------------|----------------------|
| **Purpose**   | Data tabulation     | Data analysis        |
| **Timing**    | Raw data collection | Derived for analysis |
| **Structure** | Observation-based   | Analysis-ready       |
| **Audience**  | Data managers       | Statisticians        |

## ADaM Dataset Types

### ADSL - Subject-Level Analysis Dataset

One record per subject containing:

- Demographics
- Treatment information
- Key baseline characteristics
- Analysis flags

### BDS - Basic Data Structure

Vertical structure for:

- Laboratory data (ADLB)
- Vital signs (ADVS)
- Efficacy parameters

### OCCDS - Occurrence Data Structure

For event data:

- Adverse events (ADAE)
- Concomitant medications (ADCM)

### Other Structures

- Time-to-Event (ADTTE)
- Medical History (ADMH)

## Planned Features

When ADaM support is added, Trial Submission Studio will provide:

### ADaM Generation

- Derive ADSL from DM and other SDTM domains
- Create BDS datasets from SDTM findings
- Generate OCCDS from events domains

### ADaM Validation

- Check ADaM IG compliance
- Validate traceability to SDTM
- Verify required variables

### ADaM Export

- Export to XPT format
- Generate Define-XML for ADaM
- Include in submission package

## Current Workarounds

Until ADaM support is available:

1. **Export SDTM first**
    - Use Trial Submission Studio for SDTM
    - Generate XPT files

2. **Derive ADaM externally**
    - Use SAS or R
    - Apply ADaM derivation rules
    - Generate analysis datasets

3. **Validate separately**
    - Use external validation tools
    - Check ADaM compliance

## Timeline

ADaM support is on our [roadmap](../reference/roadmap.md). Priority features:

- ADSL generation
- BDS for VS and LB
- OCCDS for AE

## Resources

### CDISC ADaM Resources

- [ADaM Implementation Guide](https://www.cdisc.org/standards/foundational/adam)
- [ADaM Structure for Occurrence Data](https://www.cdisc.org/standards/foundational/adam/adam-structure-occurrence-data-occds-v1-0)
- [CDISC Library - ADaM](https://library.cdisc.org/)

## Stay Updated

- Check the [Roadmap](../reference/roadmap.md) for updates
- Watch the
  [GitHub repository](https://github.com/rubentalstra/Trial-Submission-Studio)
  for releases
