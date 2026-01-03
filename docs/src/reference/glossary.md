# Glossary

Terms and definitions used in Trial Submission Studio and CDISC standards.

## A

### ADaM

**Analysis Data Model** - CDISC standard for analysis-ready datasets derived from SDTM data.

### ADSL

**ADaM Subject-Level** - ADaM dataset containing one record per subject with demographics and key variables.

## B

### BDS

**Basic Data Structure** - An ADaM structure used for parameter-based data like vital signs and lab results.

## C

### CDISC

**Clinical Data Interchange Standards Consortium** - Organization that develops global data standards for clinical
research.

### Codelist

A defined set of valid values for a variable. Also known as controlled terminology.

### Controlled Terminology (CT)

Standardized sets of terms and codes published by CDISC for use in SDTM and ADaM datasets.

## D

### Dataset-XML

A CDISC standard XML format for representing tabular clinical data.

### Define-XML

An XML standard for describing the structure and content of clinical trial datasets. Required for FDA submissions.

### Domain

A logical grouping of SDTM data organized by observation type (e.g., DM for Demographics, AE for Adverse Events).

### DM

**Demographics** - SDTM domain containing one record per subject with demographic information.

## E

### eCTD

**Electronic Common Technical Document** - Standard format for regulatory submissions.

## F

### FDA

**Food and Drug Administration** - US regulatory agency that requires CDISC standards for drug submissions.

### Findings Class

SDTM observation class for collected measurements and test results (e.g., Labs, Vital Signs).

## I

### ISO 8601

International standard for date and time formats. SDTM uses ISO 8601 format: `YYYY-MM-DD`.

### Interventions Class

SDTM observation class for treatments given to subjects (e.g., Exposure, Concomitant Medications).

## M

### MedDRA

**Medical Dictionary for Regulatory Activities** - Standard medical terminology for adverse events.

### Metadata

Data that describes other data. In Define-XML, metadata describes dataset structure and variable definitions.

## O

### ODM

**Operational Data Model** - CDISC standard for representing clinical data and metadata in XML.

## P

### PMDA

**Pharmaceuticals and Medical Devices Agency** - Japanese regulatory agency that requires CDISC standards.

## S

### SAS Transport (XPT)

File format for SAS datasets used for FDA submissions. See XPT.

### SDTM

**Study Data Tabulation Model** - CDISC standard structure for organizing clinical trial data.

### SDTM-IG

**SDTM Implementation Guide** - Detailed guidance for implementing SDTM, including variable definitions and business
rules.

### SEND

**Standard for Exchange of Nonclinical Data** - CDISC standard for nonclinical (animal) study data.

### Special Purpose Domain

SDTM domains that don't fit standard observation classes (e.g., DM, Trial Design domains).

### STUDYID

Standard SDTM variable containing the unique study identifier.

## U

### USUBJID

**Unique Subject Identifier** - Standard SDTM variable that uniquely identifies each subject across all studies.

## V

### Variable

An individual data element within a dataset. In SDTM, variables have standard names, labels, and data types.

## X

### XPT

**SAS Transport Format** - Binary file format used to transport SAS datasets. Required by FDA for data submissions.

#### XPT V5

Original SAS Transport format with 8-character variable names.

#### XPT V8

Extended SAS Transport format supporting 32-character variable names.

## Numbers

### --DTC Variables

SDTM timing variables containing dates/times in ISO 8601 format (e.g., AESTDTC, VSDTC).

### --SEQ Variables

SDTM sequence variables providing unique record identifiers within a domain (e.g., AESEQ, VSSEQ).

### --TESTCD Variables

SDTM test code variables in Findings domains (e.g., VSTESTCD, LBTESTCD).
