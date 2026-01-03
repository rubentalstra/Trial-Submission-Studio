# Define-XML 2.1

Define-XML provides metadata documentation for CDISC datasets.

## Overview

Define-XML is:

- Required for FDA electronic submissions
- Describes dataset structure and content
- Documents variable definitions
- Provides value-level metadata

## What Define-XML Contains

### Dataset Metadata

- Dataset names and descriptions
- Domain structure
- Keys and sort order
- Dataset locations

### Variable Metadata

- Variable names and labels
- Data types and lengths
- Origin information
- Controlled terminology references

### Value-Level Metadata

- Specific value definitions
- Conditional logic
- Derivation methods

### Computational Methods

- Derivation algorithms
- Imputation rules
- Analysis methods

## Define-XML 2.1 Structure

### Root Element

```xml
<?xml version="1.0" encoding="UTF-8"?>
<ODM xmlns="http://www.cdisc.org/ns/odm/v1.3"
     xmlns:def="http://www.cdisc.org/ns/def/v2.1"
     ODMVersion="1.3.2"
     FileType="Snapshot"
     FileOID="DEFINE-XML-EXAMPLE">
```

### Key Components

| Component         | Description             |
|-------------------|-------------------------|
| `Study`           | Study-level information |
| `MetaDataVersion` | Metadata container      |
| `ItemGroupDef`    | Dataset definitions     |
| `ItemDef`         | Variable definitions    |
| `CodeList`        | Controlled terminology  |
| `MethodDef`       | Computational methods   |
| `CommentDef`      | Comments and notes      |

## Creating Define-XML

### Automatic Generation

Trial Submission Studio generates Define-XML from:

1. Mapped datasets
2. Variable definitions
3. Controlled terminology
4. Validation rules

### Export Steps

1. Complete all domain mappings
2. Run validation
3. Click **Export → Define-XML**
4. Review generated metadata
5. Add comments/methods if needed
6. Click **Save**

### Generated Content

The exported Define-XML includes:

| Element              | Source                      |
|----------------------|-----------------------------|
| Dataset definitions  | From mapped domains         |
| Variable definitions | From SDTM standards         |
| Origins              | From mapping configuration  |
| Codelists            | From controlled terminology |

## Define-XML Elements

### ItemGroupDef (Datasets)

```xml

<ItemGroupDef OID="IG.DM"
              Name="DM"
              Repeating="No"
              Domain="DM"
              def:Structure="One record per subject"
              def:Class="SPECIAL PURPOSE">
    <Description>
        <TranslatedText xml:lang="en">Demographics</TranslatedText>
    </Description>
    <ItemRef ItemOID="IT.DM.STUDYID" OrderNumber="1" Mandatory="Yes"/>
    <!-- More ItemRefs -->
</ItemGroupDef>
```

### ItemDef (Variables)

```xml

<ItemDef OID="IT.DM.USUBJID"
         Name="USUBJID"
         DataType="text"
         Length="50"
         def:Origin="CRF">
    <Description>
        <TranslatedText xml:lang="en">Unique Subject Identifier</TranslatedText>
    </Description>
</ItemDef>
```

### CodeList (Controlled Terminology)

```xml

<CodeList OID="CL.SEX"
          Name="Sex"
          DataType="text">
    <CodeListItem CodedValue="M">
        <Decode>
            <TranslatedText xml:lang="en">Male</TranslatedText>
        </Decode>
    </CodeListItem>
    <CodeListItem CodedValue="F">
        <Decode>
            <TranslatedText xml:lang="en">Female</TranslatedText>
        </Decode>
    </CodeListItem>
</CodeList>
```

## Variable Origins

Define-XML documents where data comes from:

| Origin   | Description                |
|----------|----------------------------|
| CRF      | Case Report Form           |
| Derived  | Calculated from other data |
| Assigned | Assigned by sponsor        |
| Protocol | From study protocol        |
| eDT      | Electronic data transfer   |

## Customizing Define-XML

### Adding Comments

Add explanatory comments for:

- Complex derivations
- Data collection notes
- Exception documentation

### Computational Methods

Document derivation algorithms:

- Formulas
- Conditions
- Source variables

### Value-Level Metadata

For variables with parameter-dependent definitions:

- Different units by test
- Conditional codelists
- Test-specific origins

## Validation

### Schema Validation

Define-XML is validated against:

- CDISC Define-XML 2.1 schema
- Stylesheet rendering rules

### Common Issues

| Issue                     | Solution              |
|---------------------------|-----------------------|
| Missing required elements | Add required metadata |
| Invalid references        | Check OID references  |
| Codelist mismatches       | Verify CT alignment   |

## FDA Requirements

### Submission Package

- `define.xml` - Metadata file
- `define.pdf` - Rendered stylesheet (optional)
- Referenced XPT datasets

### Naming Convention

- File: `define.xml` (lowercase)
- Location: Study root folder

### Stylesheet

Include the CDISC stylesheet for rendering:

```xml
<?xml-stylesheet type="text/xsl" href="define2-1.xsl"?>
```

## Best Practices

1. **Generate early** - Create Define-XML as you build datasets
2. **Review carefully** - Verify all metadata is accurate
3. **Document derivations** - Explain complex logic
4. **Test rendering** - View with stylesheet before submission
5. **Validate** - Use Define-XML validators

## Next Steps

- [XPT Format](xpt-format.md) - Data file format
- [Dataset-XML](dataset-xml.md) - Alternative data format
- [Exporting Data](../user-guide/exporting-data.md) - Export guide
