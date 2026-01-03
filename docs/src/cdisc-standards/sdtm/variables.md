# SDTM Variables

Variables are the individual data elements within SDTM domains.

## Variable Categories

### Identifier Variables

Identify the study, subject, and domain.

| Variable | Label                 | Description               |
|----------|-----------------------|---------------------------|
| STUDYID  | Study Identifier      | Unique study ID           |
| DOMAIN   | Domain Abbreviation   | Two-letter domain code    |
| USUBJID  | Unique Subject ID     | Unique across all studies |
| SUBJID   | Subject ID            | Subject ID within study   |
| SITEID   | Study Site Identifier | Site number               |

### Topic Variables

Describe what was observed.

| Domain | Variable | Description        |
|--------|----------|--------------------|
| AE     | AETERM   | Adverse event term |
| CM     | CMTRT    | Medication name    |
| LB     | LBTEST   | Lab test name      |
| VS     | VSTEST   | Vital sign test    |

### Timing Variables

Capture when observations occurred.

| Variable | Label           | Description              |
|----------|-----------------|--------------------------|
| --DTC    | Date/Time       | ISO 8601 date/time       |
| --STDTC  | Start Date/Time | Start of observation     |
| --ENDTC  | End Date/Time   | End of observation       |
| --DY     | Study Day       | Study day number         |
| VISITNUM | Visit Number    | Numeric visit identifier |
| VISIT    | Visit Name      | Visit label              |

### Qualifier Variables

Provide additional context.

| Type         | Examples          | Description              |
|--------------|-------------------|--------------------------|
| **Grouping** | --CAT, --SCAT     | Category, subcategory    |
| **Result**   | --ORRES, --STRESC | Original/standard result |
| **Record**   | --SEQ, --GRPID    | Sequence, grouping       |
| **Synonym**  | --DECOD, --MODIFY | Coded/modified terms     |

## Variable Naming Conventions

### Prefix Pattern

Most variables use a domain-specific prefix:

- `AE` + `TERM` = `AETERM`
- `VS` + `TESTCD` = `VSTESTCD`
- `LB` + `ORRES` = `LBORRES`

### Common Suffixes

| Suffix     | Meaning                    | Example            |
|------------|----------------------------|--------------------|
| `--TESTCD` | Test Code                  | VSTESTCD, LBTESTCD |
| `--TEST`   | Test Name                  | VSTEST, LBTEST     |
| `--ORRES`  | Original Result            | VSORRES, LBORRES   |
| `--ORRESU` | Original Units             | VSORRESU, LBORRESU |
| `--STRESC` | Standardized Result (Char) | VSSTRESC           |
| `--STRESN` | Standardized Result (Num)  | VSSTRESN           |
| `--STRESU` | Standardized Units         | VSSTRESU           |
| `--STAT`   | Status                     | VSSTAT (NOT DONE)  |
| `--REASND` | Reason Not Done            | VSREASND           |
| `--LOC`    | Location                   | VSLOC              |
| `--DTC`    | Date/Time                  | VSDTC, AESTDTC     |

## Data Types

### Character Variables

- Text values
- Max length: 200 characters (XPT V5)
- Example: AETERM, VSTEST

### Numeric Variables

- Integer or floating-point
- Example: AGE, VSSTRESN, LBSTRESN

### Date/Time Variables

ISO 8601 format:

- Full: `2024-01-15T09:30:00`
- Date only: `2024-01-15`
- Partial: `2024-01`, `2024`

## Variable Requirements

### Required Variables

Must be present and populated for every record.

| Domain | Required Variables                   |
|--------|--------------------------------------|
| All    | STUDYID, DOMAIN, USUBJID             |
| DM     | RFSTDTC, RFENDTC, SITEID, ARM, ARMCD |
| AE     | AETERM, AEDECOD, AESTDTC             |
| VS     | VSTESTCD, VSTEST, VSORRES, VSDTC     |

### Expected Variables

Should be present when applicable.

| Domain | Expected Variables           |
|--------|------------------------------|
| AE     | AEENDTC, AESEV, AESER, AEREL |
| VS     | VSSTRESN, VSSTRESU, VISITNUM |

### Permissible Variables

Can be included if relevant data exists.

## Controlled Terminology

Variables requiring controlled terminology:

| Variable | Codelist              |
|----------|-----------------------|
| SEX      | Sex                   |
| RACE     | Race                  |
| ETHNIC   | Ethnicity             |
| COUNTRY  | Country               |
| AESEV    | Severity              |
| AESER    | No Yes Response       |
| VSTESTCD | Vital Signs Test Code |
| LBTESTCD | Lab Test Code         |

## Variable Metadata

### Label

40 characters max (XPT V5):

- Descriptive text
- Example: "Adverse Event Reported Term"

### Length

Define appropriate length for each variable:

- Consider actual data values
- XPT V5 max: 200 characters

### Order

Maintain consistent variable ordering:

1. Identifier variables
2. Topic variables
3. Qualifier variables
4. Timing variables

## Next Steps

- [Validation Rules](validation-rules.md) - Variable validation
- [Controlled Terminology](../controlled-terminology.md) - CT values
