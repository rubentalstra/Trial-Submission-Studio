# Chapter 1: INTRODUCTION

> Source pages 7–10 in `SDTMIG_v3.4.pdf`.

## Page 7

1 Introduction 1.1 Purpose The Study Data Tabulation Model Implementation Guide
for Human Clinical Trials (SDTMIG) Version 3.4 has been prepared by the
Submissions Data Standards (SDS) team of the Clinical Data Interchange Standards
Consortium (CDISC). Like its predecessors, v3.4 is intended to guide the
organization, structure, and format of standard clinical trial tabulation
datasets submitted to a regulatory authority. Version 3.4 supersedes all prior
versions of the SDTMIG. The SDTMIG should be used in close concert with Version
2.0 of the CDISC Study Data Tabulation Model (SDTM, available at
https://www.cdisc.org/standards/foundational/sdtm), which describes the general
conceptual model for representing clinical study data that is submitted to
regulatory authorities and should be read prior to reading the SDTMIG. SDTMIG
Version 3.4 provides specific domain models, assumptions, business rules, and
examples for preparing standard tabulation datasets that are based on the SDTM.
This document is intended for companies and individuals involved in the
collection, preparation, and analysis of clinical data that will be submitted to
regulatory authorities. 1.2 Organization of this Document This document is
organized into the following sections: • Section 1, Introduction, provides an
overall introduction to the v3.4 models and describes changes from prior
versions. • Section 2, Fundamentals of the SDTM, recaps the basic concepts of
the SDTM, and describes how this implementation guide should be used in concert
with the SDTM. • Section 3, Submitting Data in Standard Format, explains how to
describe metadata for regulatory submissions, and how to assess conformance with
the standards. • Section 4, Assumptions for Domain Models, describes basic
concepts, business rules, and assumptions that should be taken into
consideration before applying the domain models. • Section 5, Models for
Special-purpose Domains, describes special-purpose domains, including
Demographics, Comments, Subject Visits, and Subject Elements. • Section 6,
Domain Models Based on the General Observation Classes, provides specific
metadata models based on the 3 general observation classes, along with
assumptions and example data. • Section 7, Trial Design Model Datasets,
describes domains for trial-level data, with assumptions and examples. • Section
8, Representing Relationships and Data, describes how to represent relationships
between separate domains, datasets, and/or records, and provides information to
help sponsors determine where data belong in the SDTM. • Section 9, Study
References, provides structures for representing study-specific terminology used
in subject data. • Appendices provide additional background material and
describe other supplemental material relevant to implementation. 1.3
Relationship to Prior CDISC Documents This document, together with the SDTM,
represents the most recent version of the CDISC submission data domain models.
All updates are intended to be backward-compatible. The most significant changes
since SDTMIG v3.3 include:

## Page 8

• Expanded the scope of the DA domain to include study products in addition to
study drugs. See Section 6.3.1, Product Accountability. • Grouped specimen-based
lab domains (e.g., CP, GF, LB) in Sections 6.3.5.1-6.3.5.9 and added a generic
specification for these domains. See Section 6.3.5, Specimen-based Findings
Domains. • Expanded the scope of the IS domain for assessments of
antigen-induced humoral or cell-mediated immune response. Added 3 new variables
(i.e., Binding Agent, Molecule Secreted by Cells, Test Operational Objective).
See Section 6.3.5.5, Immunogenicity Specimen Assessments. • Updated the LB
domain specification to include the following 10 new variables: Test Condition,
Binding Agent, Test Operational Objective, Result Scale, Result Type, Collected
Summary Result Type, Lower Limit of Detection, Method Sensitivity, Point in Time
Flag, and Planned Duration. See Section 6.3.5.6, Laboratory Test Results. •
Decommissioned the Morphology (MO) domain. • Added Cell Phenotyping Findings
(CP) and Genomics Findings (GF) domains. See Section 6.3.5.3, Cell Phenotype
Findings (CP), and Section 6.3.5.4, Genomics Findings (GF). • Copied in
Biospecimen Events (BE), Biospecimen Findings (BS), and Related Specimens
(RELSPEC) from the provisional SDTMIG-PGx v1.0 in preparation for its eventual
retirement. See Section 6.2.2, Biospecimen Events (BE); Section 6.3.5.2,
Biospecimen Findings (BS); and Section 8.8, Related Specimens (RELSPEC). •
Updated QRS specifications and assumptions. Also introduced subsections to
separate assumptions and examples describing the RS Disease Response use case
and the RS Clinical Classifications use case. See Section 6.3.9, Questionnaires,
Ratings, and Scales (QRS) Domains (FT, QS, RS). • Updated the Tumor/Lesion (TU
and TR) domain assumptions to describe use of indicator questions, disease
recurrence conventions, and modeling of location of interest. See Section
6.3.12, Tumor/Lesion Domains. • Expanded the scope of the SC domain to support
collection over time. See Section 6.3.10, Subject Characteristics. • Updated
guidance and examples for the FA domain. See Section 6.4, Findings About Events
or Interventions. • Corrected Core values for the following variables: DSDY,
DSSTDY, LBSTREFC, MILOBXFL, and MIBLFL. • Updated Controlled Terminology for
applicable variables across all domains, if available. • Removed Appendix C1,
Trial Summary Codes. A detailed list of changes between versions is provided in
Appendix E, Revision History. Version 3.1 was the first fully
implementation-ready version of the CDISC submission data standards that was
directly referenced by the US FDA for use in human clinical studies involving
drug products. However, future improvements and enhancements will continue to be
made as sponsors gain more experience submitting data in this format. Therefore,
CDISC will be preparing regular updates to the implementation guide to provide
corrections, clarifications, additional domain models, examples, business rules,
and conventions for using the standard domain models. Because CDISC will produce
further documentation for Controlled Terminology as separate publications,
sponsors are encouraged to check the CDISC website
(https://www.cdisc.org/standards/terminology/controlledterminology) frequently
for additional information. See Section 4.3, Coding and Controlled Terminology
Assumptions, for the most up-to-date information on applying Controlled
Terminology. 1.4 How to Read this Implementation Guide The SDTMIG is best read
online, so the reader can benefit from the many hyperlinks to internal and
external references. The following guidelines may be helpful in reading this
document:

1. First, read the SDTM to gain a general understanding of SDTM concepts.

## Page 9

2. Next, read Sections 1-3 of this document to review the key concepts for
   preparing domains and submitting data to regulatory authorities. Refer to
   Appendix B, Glossary and Abbreviations, as necessary.
3. Read Section 4, Assumptions for Domain Models.
4. Review Section 5, Models for Special-purpose Domains, and Section 6, Domain
   Models Based on the General Observation Classes, in detail, referring back to
   Section 4, Assumptions for Domain Models, as directed. See the implementation
   examples for each domain to gain an understanding of how to apply the domain
   models for specific types of data.
5. Read Section 7, Trial Design Model Datasets, to understand the fundamentals
   of the Trial Design Model and consider how to apply the concepts for typical
   protocols.
6. Review Section 8, Representing Relationships and Data, to learn advanced
   concepts of how to express relationships between datasets, records, and
   additional variables not specifically defined in the models.
7. Review Section 9, Study References, to learn about occasions when it is
   necessary to establish studyspecific references that will be used in
   accordance with subject data.
8. Finally, review the appendices as appropriate. Appendix C, Controlled
   Terminology, in particular, describes how CDISC Terminology is centrally
   managed by the CDISC Controlled Terminology Team. Efforts are made at
   publication time to ensure all SDTMIG domain/dataset specification tables
   and/or examples reflect the latest CDISC Terminology; users, however, should
   refer to https://www.cancer.gov/research/resources/terminology/cdisc as the
   authoritative source of controlled terminology, as CDISC Controlled
   Terminology is updated on a quarterly basis. This implementation guide covers
   most data collected in human clinical trials, but separate implementation
   guides provide information about certain data, and should be consulted when
   needed. The following guides are available at
   https://www.cdisc.org/standards/foundational/sdtmig: • The SDTM
   Implementation Guide: Associated Persons (SDTMIG-AP) provides structures for
   representing data collected about persons who are not study subjects. • The
   SDTM Implementation Guide for Medical Devices (SDTMIG-MD) provides structures
   for data about devices. • Historically, the SDTM Implementation Guide for
   Pharmacogenomics/Genetics (SDTMIG-PGx) has provided structures for
   pharmacogenetic/genomic data and for data about biospecimens. Much of the
   content of the SDTMIG-PGx has been incorporated into and/or superseded by the
   SDTMIG v3.4. 1.4.1 How to Read a Domain Specification A domain specification
   table includes rows for all required and expected variables for a domain and
   for a set of permissible variables. The permissible variables do not include
   all the variables that are allowed for the domain; they are a set of
   variables that the SDS Team considered likely to be included. The columns of
   the table are: • Variable Name o For variables that do not include a domain
   prefix, this name is taken directly from the SDTM. o For variables with a
   "--" placeholder in the SDTM, the "--" is replaced by the 2-character domain
   code. • Variable Label: A longer name for the variable o This may be the same
   as the label in the SDTM, or it may be customized for the domain. o Sponsors
   should create an appropriate label if they include in a dataset an allowable
   variable not in the domain specification. • Type: One of the 2 SAS datatypes,
   "Num" or "Char". These values are taken directly from the SDTM. • Controlled
   Terms, Codelist, or Format o Controlled Terms ▪ As noted in the table note,
   an asterisk (*) indicates that the variable may be subject to controlled
   terminology.

## Page 10

• The controlled terminology might be of a type that would inherently be
sponsor-defined. • The controlled terminology might be of a type that could be
standardized, but for which a codelist not yet been developed. • The controlled
terminology might be terminology specified in value-level metadata. ▪ The name
of an external code system (e.g., MedDRA) will be listed in plain text. o
Codelist ▪ A hyperlinked codelist name in parentheses indicates that the
variable is subject to the CDISC Controlled Terminology in the named codelist. ▪
Multiple hyperlinked codelist names indicate that the variable is subject to 1
or more of the named codelists from CDISC Controlled Terminology. If multiple
codelists are in use for a single domain, value-level metadata would indicate
where each codelist is applicable. ▪ A hyperlinked codelist name and an asterisk
(*) indicate that the variable is subject to either the named codelist from the
CDISC Controlled Terminology or to an external dictionary. The specific
dictionary is identified in the metadata. o Format: "ISO 8601 datetime or
interval" or "ISO 8601 duration" in plain text indicates that the variable
values should be formatted in conformance with that standard. • Role: This is
taken directly from the SDTM. Note that if a variable is either a Variable
Qualifier or a Synonym Qualifier, the SDTM includes the qualified variable, but
SDTMIG domain specifications do not. • CDISC Notes may include any of the
following: o A description of what the variable means o Information about how
this variable relates to another variable o Rules for when or how the variable
should be populated, or how the contents should be formatted o Examples of
values that might appear in the variable. Such examples are only examples, and
although they may be CDISC Controlled Terminology values, their presence in a
CDISC Note should not be construed as definitive. For authoritative information
on CDISC Controlled Terminology, consult
https://www.cancer.gov/research/resources/terminology/cdisc. • Core: Contains 1
of the 3 values—"Req", "Exp", or "Perm"—explained further in Section 4.1.5, SDTM
Core Designations. 1.5 Known Issues Derived Records and the use of --DRVFL
Although it is implicit in the general concept of a derived record that there is
no collected result (--ORRES should be null), this is not an explicit
requirement currently stated in published CDISC material. This is being
evaluated for clarification in a future release of the SDTMIG and/or Model
document. Use of --LNKID and --LNKGRP The definition of --LNKID says that it is
"used to identify a record," and the definition of --LNKGRP says that it is
"used to identify a group of records." This implies that when setting up a
relationship in RELREC, a row where RELTYPE = ONE will have an IDVAR of --LNKID
(and not --LNKGRP); a row where RELTYPE = MANY will have an IDVAR of --LNKID
(and not --LNKGRP). The examples in this version of the SDTMIG have not been
systematically reviewed to implement this distinction between --LNKID and
--LNKGRP. This distinction between - -LNKID and --LNKGRP, and the
appropriateness of using other identification variables for linking, will be
clarified in a future release of the SDTMIG and/or Model document, and at that
time examples in the SDTMIG will be systematically reviewed and updated to
reflect that clarification.
