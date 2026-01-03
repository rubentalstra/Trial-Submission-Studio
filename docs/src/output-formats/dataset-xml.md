# Dataset-XML Format

Dataset-XML is a CDISC standard XML format for clinical data exchange.

## Overview

Dataset-XML provides:

- Human-readable data format
- Full Unicode support
- Embedded metadata
- Alternative to XPT binary format

## When to Use Dataset-XML

| Use Case               | Recommendation         |
|------------------------|------------------------|
| FDA submission         | Use XPT (required)     |
| Internal data exchange | Dataset-XML works well |
| Archive/audit trail    | Good for documentation |
| Non-SAS environments   | Easier integration     |
| Full character support | Unicode capable        |

## Format Structure

### ODM Container

Dataset-XML is based on CDISC ODM (Operational Data Model):

```xml
<?xml version="1.0" encoding="UTF-8"?>
<ODM xmlns="http://www.cdisc.org/ns/odm/v1.3"
     xmlns:data="http://www.cdisc.org/ns/Dataset-XML/v1.0"
     FileType="Snapshot">
    <ClinicalData StudyOID="..." MetaDataVersionOID="...">
        <SubjectData SubjectKey="...">
            <StudyEventData StudyEventOID="...">
                <ItemGroupData ItemGroupOID="DM">
                    <ItemData ItemOID="STUDYID">ABC123</ItemData>
                    <ItemData ItemOID="USUBJID">ABC123-001</ItemData>
                    <!-- More items -->
                </ItemGroupData>
            </StudyEventData>
        </SubjectData>
    </ClinicalData>
</ODM>
```

### Key Elements

| Element         | Description          |
|-----------------|----------------------|
| `ODM`           | Root container       |
| `ClinicalData`  | Study data container |
| `SubjectData`   | Per-subject data     |
| `ItemGroupData` | Domain records       |
| `ItemData`      | Individual values    |

## Creating Dataset-XML

### Export Steps

1. Complete data mapping
2. Run validation
3. Click **Export → Dataset-XML**
4. Configure options
5. Choose output location
6. Click **Save**

### Export Options

| Option           | Description                  |
|------------------|------------------------------|
| Include metadata | Embed variable definitions   |
| Pretty print     | Format XML for readability   |
| Compress         | Reduce file size             |
| Single file      | One file vs. file per domain |

## Dataset-XML vs XPT

| Aspect         | Dataset-XML    | XPT            |
|----------------|----------------|----------------|
| Format         | Text (XML)     | Binary         |
| Readability    | Human-readable | Requires tools |
| Size           | Larger         | Smaller        |
| Unicode        | Full support   | Limited        |
| FDA submission | Accepted       | Required       |
| Integration    | Easier         | SAS-focused    |

## Advantages

### Human Readable

- Open in any text editor
- Easily inspectable
- Good for debugging

### Full Unicode

- International characters
- Special symbols
- No character limitations

### Self-Describing

- Metadata embedded
- Schema validation
- No external dependencies

### Platform Independent

- Standard XML format
- Any programming language
- No proprietary tools needed

## Limitations

### File Size

- Larger than binary XPT
- Compression recommended for large datasets

### FDA Preference

- FDA prefers XPT for submissions
- Dataset-XML accepted but less common

### Processing Overhead

- XML parsing slower than binary
- More memory for large files

## Validation

### Schema Validation

Dataset-XML can be validated against:

- CDISC Dataset-XML schema
- ODM schema
- Custom validation rules

### Common Checks

- Well-formed XML
- Valid element structure
- Data type conformance
- Required elements present

## Working with Dataset-XML

### Reading Files

Dataset-XML can be read by:

- Any XML parser
- CDISC-compatible tools
- Statistical software with XML support

### Converting to Other Formats

From Dataset-XML, you can convert to:

- XPT (for FDA submission)
- CSV (for analysis)
- Database tables

## Technical Details

### Encoding

- UTF-8 (default and recommended)
- UTF-16 supported
- Encoding declared in XML header

### Namespaces

```xml
xmlns="http://www.cdisc.org/ns/odm/v1.3"
        xmlns:data="http://www.cdisc.org/ns/Dataset-XML/v1.0"
```

### File Extension

- `.xml` for Dataset-XML files
- Optionally: `domain.xml` (e.g., `dm.xml`)

## Next Steps

- [XPT Format](xpt-format.md) - FDA standard format
- [Define-XML](define-xml.md) - Metadata documentation
- [Exporting Data](../user-guide/exporting-data.md) - Export guide
