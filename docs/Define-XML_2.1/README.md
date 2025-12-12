# define-XML v2.1 Release package

Publication date: 2025-09-26

## Introduction

Define-XML v2.1 was published on May 15,2019.

This publication package contains the specification, schemas and other documents related to Define.XML v2.1.10. Updates to the define-enumerations schema reflect the CDISC Controlled Terminology Team 2025-09-26 publication of Package 60.

### Table of contents

- Define-XML v2.1.pdf -- Specification document
- schema folder -- The full set of schemas required to validate Define-XML v2.1 documents.
- stylesheet folder -- XSL Stylesheet following the Define-XML Stylesheet Recommnedations developed through the PhUSE Optimizing the Use of Data Standards working Group.
- examples folder -- Sample ADaM And SDTM Define-XML v2.1 documents

### Changes in Define-XML v2.1.10

The Define-XML enumerations schema file(schema/cdisc-define-2.1/define-enumerations.xsd) and it's accompanying documentation file (schema/cdisc-define-2.1/define-enumerations.html) reflect the Package 60 publication by the CDISC Terminology Team.
In particular the following change was made to define-enumerations.xsd:

ExternalCodeListDictionary (C66788):

- Enumeration for ClinicalTrials.gov (C124233) was added.
- Enumeration for CTIS (C221530) was added.
- Enumeration for EudraCT (C132782) was added.
- Enumeration for EUDRAVIGILANCE (C221532)  was added.
- Enumeration for GENC (C221529) was added.
- Enumeration for jRCT (C221531) was added.
- Enumeration for MeSH (C82845) was added.
- Enumeration for PubMed (C42881) was added.

The enumerations documentation (schema/cdisc-define-2.1/define-enumerations.html) was regenerated to reflect these changes.

### Changes in Define-XML v2.1.9

The Define-XML enumerations schema file(schema/cdisc-define-2.1/define-enumerations.xsd) and it's accompanying documentation file (schema/cdisc-define-2.1/define-enumerations.html) reflect the Package 59 publication by the CDISC Terminology Team.
In particular the following change was made to define-enumerations.xsd:

StandardName (C170452):

- Enumeration for ADaM-OCCDSIG (C214535) was added.
- Enumeration for ADaMIG-MD (C214532) was added.
- Enumeration for ADaMIG-NCA (C214533) was added.
- Enumeration for ADaMIG-popPK (C214534) was added.

The enumerations documentation (schema/cdisc-define-2.1/define-enumerations.html) was regenerated to reflect these changes.

### Changes in Define-XML v2.1.8

The Define-XML enumerations schema file(schema/cdisc-define-2.1/define-enumerations.xsd) and it's accompanying documentation file (schema/cdisc-define-2.1/define-enumerations.html) reflect the Package 58 publication by the CDISC Terminology Team.
In particular the following change was made to define-enumerations.xsd:

ExternalCodeListDictionary (C66788):

- The definition of the Enumeration simpleType was updated to: Terminology relevant to the names given to a reference source that lists words and gives their meaning.
- Enumeration for ISO 21090 (C81895) was added.
- Enumeration for ISO 3166 (C209537) was added.

SdtmigVersionResponse (C160924):

- The definition of the Enumeration simpleType was updated to: A terminology codelist relevant to the version of the CDISC Study Data Tabulation Model implementation guide that is being used in the study submission.

OriginType (C170449):

- The CDISC Definition of the Predecessor (C170550) Enumeration was updated from "A value that is copied from a variable in another dataset." to "A value that is copied from another variable.".

The enumerations documentation (schema/cdisc-define-2.1/define-enumerations.html) was regenerated to reflect these changes.

### Changes in Define-XML v2.1.7

The Define-XML enumerations schema file(schema/cdisc-define-2.1/define-enumerations.xsd) and it's accompanying documentation file (schema/cdisc-define-2.1/define-enumerations.html) reflect the Package 57 publication by the CDISC Terminology Team.
In particular the following change was made to define-enumerations.xsd:

ItemGroupClass (C103329):

- Enumeration for REFERENCE DATA STRUCTURE (C204611) was added.

The enumerations documentation (schema/cdisc-define-2.1/define-enumerations.html) was regenerated to reflect these changes.


#### Changes in Define-XML v2.1.6

The Define-XML enumerations schema file(schema/cdisc-define-2.1/define-enumerations.xsd) and it's accompanying documentation file (schema/cdisc-define-2.1/define-enumerations.html) reflect the Package 54 publication by the CDISC Terminology Team.
In particular the following change was made to define-enumerations.xsd:

StandardName (C170452):

- Enumeration for SENDIG-GENETOX (C199687) was added.

The enumerations documentation (schema/cdisc-define-2.1/define-enumerations.html) was regenerated to reflect these changes.

#### Changes in Define-XML v2.1.5

StandardName (C170452):

- Enumeration for BIMO (C191213) was added.

SdtmigVersionResponse (C160924):

- Terminology relevant to the version of the study data tabulation model implementation guide used in the study was added (C160924).
  This is an extensible enumeration.

#### Changes in Define-XML v2.1.4

ExternalCodeListDictionary (C66788):

- Enumeration for ICD-O (C37978) was added.

ItemGroupSubClass:

- Enumeration for POPULATION PHARMACOKINETIC ANALYSIS (C189348) was added.

StandardStatus (C172332):

- The simpleType was made extensible, as is the Standards Status codelist (C172332)

#### Changes in Define-XML v2.1.3

StandardName (C170452):

- Enumeration for SDTMIG-PGx (Code C170555) was removed.
- Enumeration for SENDIG-AR (C181230) was added.

#### Changes in Define-XML v2.1.2

 **StandardName** codelist  is no longer extensible
 **StandardPublishingSet** codelist is no longer extensible.
 **StandardPublishingSet** codelist terms all have updated C-Codes.
 **StandardType** codelist is no longer extensible.

#### Changes in Define-XML v2.1.1

The Define-XML enumerations schema file(schema/cdisc-define-2.1/define-enumerations.xsd) and it's accompanying documentation file (schema/cdisc-define-2.1/define-enumerations.html) were updaated to reflect the Package 45 publication by the CDISC Terminology Team.


