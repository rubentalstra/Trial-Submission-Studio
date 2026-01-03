# Controlled Terminology

CDISC Controlled Terminology (CT) provides standardized values for SDTM variables.

## Overview

Controlled Terminology ensures:

- **Consistency** across studies and organizations
- **Interoperability** between systems
- **Regulatory compliance** with FDA requirements

## Embedded CT Packages

Trial Submission Studio includes the following CT versions:

| Version    | Release Date   | Status    |
|------------|----------------|-----------|
| 2024-12-20 | December 2024  | Current   |
| 2024-09-27 | September 2024 | Supported |
| 2024-06-28 | June 2024      | Supported |

## Common Codelists

### SEX (C66731)

| Code             | Decoded Value    |
|------------------|------------------|
| M                | MALE             |
| F                | FEMALE           |
| U                | UNKNOWN          |
| UNDIFFERENTIATED | UNDIFFERENTIATED |

### RACE (C74457)

| Decoded Value                             |
|-------------------------------------------|
| AMERICAN INDIAN OR ALASKA NATIVE          |
| ASIAN                                     |
| BLACK OR AFRICAN AMERICAN                 |
| NATIVE HAWAIIAN OR OTHER PACIFIC ISLANDER |
| WHITE                                     |
| MULTIPLE                                  |
| NOT REPORTED                              |
| UNKNOWN                                   |

### ETHNIC (C66790)

| Decoded Value          |
|------------------------|
| HISPANIC OR LATINO     |
| NOT HISPANIC OR LATINO |
| NOT REPORTED           |
| UNKNOWN                |

### COUNTRY (C66729)

ISO 3166-1 alpha-3 country codes:

- USA, CAN, GBR, DEU, FRA, JPN, etc.

### AESEV (C66769) - Severity

| Decoded Value |
|---------------|
| MILD          |
| MODERATE      |
| SEVERE        |

### AESER (C66742) - Serious

| Code | Decoded Value |
|------|---------------|
| Y    | Y             |
| N    | N             |

### NY (C66742) - No Yes Response

| Code | Decoded Value |
|------|---------------|
| Y    | Y             |
| N    | N             |

### VSTESTCD (C66741) - Vital Signs Test Codes

| Code   | Decoded Value            |
|--------|--------------------------|
| BMI    | Body Mass Index          |
| DIABP  | Diastolic Blood Pressure |
| HEIGHT | Height                   |
| HR     | Heart Rate               |
| PULSE  | Pulse Rate               |
| RESP   | Respiratory Rate         |
| SYSBP  | Systolic Blood Pressure  |
| TEMP   | Temperature              |
| WEIGHT | Weight                   |

### LBTESTCD - Lab Test Codes

Common examples:
| Code | Description |
|------|-------------|
| ALB | Albumin |
| ALT | Alanine Aminotransferase |
| AST | Aspartate Aminotransferase |
| BILI | Bilirubin |
| BUN | Blood Urea Nitrogen |
| CREAT | Creatinine |
| GLUC | Glucose |
| HGB | Hemoglobin |
| PLAT | Platelet Count |
| WBC | White Blood Cell Count |

## Extensible vs Non-Extensible

### Non-Extensible Codelists

Values must exactly match the codelist:

- SEX
- COUNTRY
- Unit codelists

### Extensible Codelists

Additional values allowed with sponsor definition:

- RACE (can add study-specific values)
- Some test codes

## Using CT in Trial Submission Studio

### Automatic Validation

When you map variables with controlled terminology:

1. Values are checked against the codelist
2. Non-matching values are flagged
3. Suggestions are provided

### Value Mapping

For source values not in CT format:

1. Create value-level mappings
2. Map "Male" → "M", "Female" → "F"
3. Apply consistently

### CT Version Selection

1. Go to **Settings → Controlled Terminology**
2. Select the appropriate CT version
3. Validation uses selected version

## Handling CT Errors

### Value Not in Codelist

**Error**: "Value 'XYZ' not found in codelist"

**Solutions**:

1. Check spelling/case
2. Find the correct CT value
3. Map source value to CT value
4. For extensible codelists, document new value

### Common Mappings

| Source Value     | CT Value                  |
|------------------|---------------------------|
| Male             | M                         |
| Female           | F                         |
| Yes              | Y                         |
| No               | N                         |
| Caucasian        | WHITE                     |
| African American | BLACK OR AFRICAN AMERICAN |

## Updating CT

New CT versions are released quarterly by CDISC. To use newer versions:

1. Check for Trial Submission Studio updates
2. New CT is included in app updates
3. Select version in settings

## Resources

### Official References

- [CDISC CT Browser](https://www.cdisc.org/standards/terminology)
- [NCI Term Browser](https://ncit.nci.nih.gov/ncitbrowser/)

## Next Steps

- [SDTM Variables](sdtm/variables.md) - Variables requiring CT
- [Validation](../user-guide/validation.md) - CT validation in practice
