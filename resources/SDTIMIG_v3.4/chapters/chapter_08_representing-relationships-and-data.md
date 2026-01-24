# Chapter 8: REPRESENTING RELATIONSHIPS AND DATA

> Source pages 427–440 in `SDTMIG_v3.4.pdf`.


## Page 427

8 Representing Relationships and Data
The defined variables of the SDTM general observation classes could restrict the ability of sponsors to represent all the data they wish to submit. Collected data that may not entirely fit includes relationships between records within a domain, records in separate domains, and sponsor-defined "variables." As a result, the SDTM has methods to represent distinct types of relationships, all of which are described in more detail in subsequent sections:
• Section 8.1, Relating Groups of Records Within a Domain Using the --GRPID Variable, describes
representing a relationship between a group of records for a given subject within the same domain.
• Section 8.2, Relating Peer Records, describes representing relationships between independent records
(usually in separate domains) for a subject, such as a concomitant medication taken to treat an adverse event.
• Section 8.3, Relating Datasets, describes representing a relationship between 2 (or more) datasets where
records of 1 (or more) dataset(s) are related to record(s) in another dataset (or datasets).
• Section 8.4, Relating Non-standard Variable Values to a Parent Domain, describes the method for
representing the dependent relationship where data that cannot be represented by a standard variable within the demographics domain (DM) or a general observation-class domain record (or records) can be related back to that record (or records).
• Section 8.5, Relating Comments to a Parent Domain, describes representing a dependent relationship
between a comment in the Comments (CO) domain (see also Section 5.1, Comments) and a parent record (or records) in other domains, such as a comment recorded in association with an adverse event.
• Section 8.6, How to Determine Where Data Belong in SDTM-Compliant Data Tabulations, discusses the
concept of related datasets and whether to place additional data in a separate domain or a supplemental qualifier special-purpose dataset, and the concept of modeling findings data that refer to data in another general observation class domain.
• Section 8.7, Relating Subjects, describes representing collected relationships between persons, both of
whom are study subjects. For example: "MOTHER, BIOLOGICAL"; "CHILD, BIOLOGICAL"; "TWIN, DIZOGOTIC".
• Section 8.8, Related Specimens, is a dataset used to represent relationships between specimens.
All relationships make use of the standard domain identifiers STUDYID, DOMAIN, and USUBJID. In addition, the variables IDVAR and IDVARVAL are used for identifying the record-level merge/join keys. These keys are used to tie information together by linking records. The specific set of identifiers necessary to properly identify each type of relationship is described in detail in the following sections. The following are examples of variables that could be used in IDVAR:
• The sequence number (--SEQ) variable uniquely identifies a record for a given USUBJID within a domain.
The variable --SEQ is required in all domains except DM. For example, if a subject has 25 adverse events in the Adverse Event (AE) domain, then 25 unique AESEQ values should be established for this subject. Conventions for establishing and maintaining --SEQ values are sponsor-defined. Values may or may not be sequential depending on data processes and sources.
• The reference identifier (--REFID) variable can be used to capture a sponsor-defined or external identifier,
such as an identifier provided in an electronic data transfer. Some examples are lab-specimen identifiers and ECG identifiers. --REFID is permissible in all general observation-class domains, but is never required. Values for --REFID are sponsor-defined and can be any alphanumeric strings the sponsor chooses, consistent with their internal practices.
• The grouping identifier (--GRPID) variable, used to link related records for a subject within a domain, is
explained in Section 8.1, Relating Groups of Records Within a Domain Using the --GRPID Variable.


## Page 428

8.1 Relating Groups of Records Within a Domain Using the --
GRPID Variable The optional grouping identifier variable --GRPID is Permissible in all domains that are based on the general observation classes. It is used to identify relationships between records within a USUBJID within a single domain (e.g., intervention records for a combination therapy where treatments in the combination varies from subject to subject). In such cases, the relationship is defined by assigning the same unique character value to the --GRPID variable. The values used for --GRPID can be any values the sponsor chooses; however, if the sponsor uses values with some embedded meaning (rather than arbitrary numbers), those values should be consistent across the submission to avoid confusion. It is important to note that --GRPID has no inherent meaning across subjects or across domains. Using --GRPID in the general-observation class domains can reduce the number of records in the RELREC, SUPP-- , and CO datasets, when those datasets are submitted to describe relationships/associations for records or values to a "group" of general observation class records.
8.1.1 --GRPID Example
The following table illustrates --GRPID used in the Concomitant Medications (CM) domain to identify a combination therapy. In this example, both subjects 1234 and 5678 have reported 2 combination therapies, each consisting of 3 separate medications. The components of a combination all have the same value for CMGRPID. This example illustrates how CMGRPID groups information only within a subject within a domain. Rows 1-3: Show 3 medications taken by subject 1234. GMGRPID="COMBO THPY 1" has been used to group these medications. Rows 4-6: Show 3 different medications taken by subject 1234, with CMGRPID="COMBO THPY 2". Rows 7-9: Show 3 medications taken by subject 5678. CMGRPID="COMBO THPY 1" has been used to group these medications. Note that the medications with GMGRPID "COMBO THPY 1" are completely different for subjects 1234 and 5678. Rows 10-12: Show 3 different medications taken by subject 5678, with CMGRPID=“COMBO THPY 2”. Again, the medications with “COMBO THPY 2” are completely different for subjects 1234 and 5678. cm.xpt

| Row | STUDYID | DOMAIN | USUBJID | CMSEQ | CMGRPID | CMTRT | CMDECOD | CMDOSE | CMDOSU | CMSTDTC | CMENDTC |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 1234 | CM | 1234 | 1 | COMBO<br>THPY 1 | Verbatim Med<br>A | Generic Med<br>A | 100 | mg | 2004-01-<br>17 | 2004-01-<br>19 |
| 2 | 1234 | CM | 1234 | 2 | COMBO<br>THPY 1 | Verbatim Med<br>B | Generic Med<br>B | 50 | mg | 2004-01-<br>17 | 2004-01-<br>19 |
| 3 | 1234 | CM | 1234 | 3 | COMBO<br>THPY 1 | Verbatim Med<br>C | Generic Med<br>C | 200 | mg | 2004-01-<br>17 | 2004-01-<br>19 |
| 4 | 1234 | CM | 1234 | 4 | COMBO<br>THPY 2 | Verbatim Med<br>D | Generic Med<br>D | 150 | mg | 2004-01-<br>21 | 2004-01-<br>22 |
| 5 | 1234 | CM | 1234 | 5 | COMBO<br>THPY 2 | Verbatim Med<br>E | Generic Med<br>E | 100 | mg | 2004-01-<br>21 | 2004-01-<br>22 |
| 6 | 1234 | CM | 1234 | 6 | COMBO<br>THPY 2 | Verbatim Med<br>F | Generic Med<br>F | 75 | mg | 2004-01-<br>21 | 2004-01-<br>22 |
| 7 | 1234 | CM | 5678 | 1 | COMBO<br>THPY 1 | Verbatim Med<br>G | Generic Med<br>G | 37.5 | mg | 2004-03-<br>17 | 2004-03-<br>25 |
| 8 | 1234 | CM | 5678 | 2 | COMBO<br>THPY 1 | Verbatim Med<br>H | Generic Med<br>H | 60 | mg | 2004-03-<br>17 | 2004-03-<br>25 |
| 9 | 1234 | CM | 5678 | 3 | COMBO<br>THPY 1 | Verbatim Med<br>I | Generic Med I | 20 | mg | 2004-03-<br>17 | 2004-03-<br>25 |
| 10 | 1234 | CM | 5678 | 4 | COMBO<br>THPY 2 | Verbatim Med<br>J | Generic Med<br>J | 100 | mg | 2004-03-<br>21 | 2004-03-<br>22 |
| 11 | 1234 | CM | 5678 | 5 | COMBO<br>THPY 2 | Verbatim Med<br>K | Generic Med<br>K | 50 | mg | 2004-03-<br>21 | 2004-03-<br>22 |
| 12 | 1234 | CM | 5678 | 6 | COMBO<br>THPY 2 | Verbatim Med<br>L | Generic Med<br>L | 10 | mg | 2004-03-<br>21 | 2004-03-<br>22 |


## Page 429

8.2 Relating Peer Records
The Related Records (RELREC) special-purpose dataset is used to describe relationships between records for a subject (as described in this section), and relationships between datasets (as described in Section 8.3, Relating Datasets). In both cases, relationships represented in RELREC are collected relationships, either by explicit references or checkboxes on the CRF, or by design of the CRF (e.g., vital signs captured during an exercise stress test). A relationship is defined by adding a record to RELREC for each record to be related and by assigning a unique character identifier value for the relationship. Each record in the RELREC special-purpose dataset contains keys that identify a record (or group of records) and the relationship identifier, which is stored in the RELID variable. The value of RELID is chosen by the sponsor, but must be identical for all related records within USUBJID. It is recommended that the sponsor use a standard system or naming convention for RELID (e.g., all letters, all numbers, capitalized). Records expressing a relationship are specified using the key variables STUDYID, RDOMAIN (the domain code of the record in the relationship), and USUBJID, along with IDVAR and IDVARVAL. Single records can be related by using a unique-record-identifier variable such as --SEQ in IDVAR. Groups of records can be related by using grouping variables such as --GRPID in IDVAR. IDVARVAL would contain the value of the variable described in IDVAR. Using --GRPID can be a more efficient method of representing relationships in RELREC, such as when relating an adverse event (or events) to a group of concomitant medications taken to treat the adverse event(s). The RELREC dataset should be used to represent either:
• explicit relationships, such as concomitant medications taken as a result of an adverse event; or
• information of a nature that necessitates using multiple datasets, as described in Section 8.3, Relating
Datasets.
8.2.1 Related Records (RELREC)
RELREC - Description/Overview A dataset used to describe relationships between records for a subject within or across domains, and relationships of records across datasets. RELREC – Specification relrec.xpt, Related Records — Relationship. One record per related record, group of records or dataset, Tabulation.

1In this column, an asterisk (*) indicates that the variable may be subject to controlled terminology. CDISC/NCI codelist values are enclosed in parentheses.

| Variable<br>Name | Variable<br>Label | Type | Controlled<br>Terms,<br>Codelist or<br>Format1 | Role | CDISC Notes | Core |
| --- | --- | --- | --- | --- | --- | --- |
| STUDYID | Study Identifier | Char |  | Identifier | Unique identifier for a study. | Req |
| RDOMAIN | Related<br>Domain<br>Abbreviation | Char | (DOMAIN) | Identifier | Abbreviation for the domain of the parent record(s). | Req |
| USUBJID | Unique<br>Subject<br>Identifier | Char |  | Identifier | Identifier used to uniquely identify a subject across all studies for | Exp |
|  |  |  |  |  | all applications or submissions involving the product. |  |
| IDVAR | Identifying<br>Variable | Char | * | Identifier | Name of the identifying variable in the general-observation-class | Req |
|  |  |  |  |  | dataset that identifies the related record(s). Examples: --SEQ, -- |  |
|  |  |  |  |  | GRPID. |  |
| IDVARVAL | Identifying<br>Variable Value | Char |  | Identifier | Value of identifying variable described in IDVAR. If --SEQ is the | Exp |
|  |  |  |  |  | variable being used to describe this record, then the value of -- |  |
|  |  |  |  |  | SEQ would be entered here. |  |
| RELTYPE | Relationship<br>Type | Char | (RELTYPE) | Record<br>Qualifier | Identifies the hierarchical level of the records in the relationship. | Exp |
|  |  |  |  |  | Values should be either "ONE" or "MANY". Used only when |  |
|  |  |  |  |  | identifying a relationship between datasets (as described in |  |
|  |  |  |  |  | Section 8.3, Relating Datasets). |  |
| RELID | Relationship<br>Identifier | Char |  | Record<br>Qualifier | Unique value within USUBJID that identifies the relationship. All | Req |
|  |  |  |  |  | records for the same USUBJID that have the same RELID are |  |
|  |  |  |  |  | considered related/associated. RELID can be any value the |  |
|  |  |  |  |  | sponsor chooses, and is only meaningful within the RELREC |  |
|  |  |  |  |  | dataset to identify the related/associated domain records. |  |


## Page 430

8.2.2 RELREC Dataset Examples
Example 1 This example illustrates the use of the RELREC dataset to relate records stored in separate domains for USUBJID = "123456". This example represents a situation in which a single adverse event is part of 2 collected relationships, one with 2 concomitant medications and the other with 2 laboratory findings, but there is no collected relationship between the 2 laboratory findings and the 2 concomitant medications. Rows 1-3: Show the representation of a relationship between an AE record and 2 concomitant medication records. Rows 4-6: Show the representation of a relationship between the same AE record and 2 laboratory findings records. relrec.xpt

Example 2 Example 2 is the same scenario as Example 1. In this case, however, the way the data were collected indicated that the concomitant medications and laboratory findings were all in a single relationship to each other and the adverse event. relrec.xpt

| Row | STUDYID | RDOMAIN | USUBJID | IDVAR | IDVARVAL | RELTYPE | RELID |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | EFC1234 | AE | 123456 | AESEQ | 5 |  | 1 |
| 2 | EFC1234 | CM | 123456 | CMSEQ | 11 |  | 1 |
| 3 | EFC1234 | CM | 123456 | CMSEQ | 12 |  | 1 |
| 4 | EFC1234 | AE | 123456 | AESEQ | 5 |  | 2 |
| 5 | EFC1234 | LB | 123456 | LBSEQ | 47 |  | 2 |
| 6 | EFC1234 | LB | 123456 | LBSEQ | 48 |  | 2 |

Example 3 Example 3 is the same scenario as Example 2. However, the sponsor grouped the 2 concomitant medications in the CM domain using CMGRPID = “COMBO 1”, allowing the relationship among these 5 records to be represented with 4, rather than 5, records in the RELREC dataset. relrec.xpt

| Row | STUDYID | RDOMAIN | USUBJID | IDVAR | IDVARVAL | RELTYPE | RELID |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | EFC1234 | AE | 123456 | AESEQ | 5 |  | 1 |
| 2 | EFC1234 | CM | 123456 | CMSEQ | 11 |  | 1 |
| 3 | EFC1234 | CM | 123456 | CMSEQ | 12 |  | 1 |
| 4 | EFC1234 | LB | 123456 | LBSEQ | 47 |  | 1 |
| 5 | EFC1234 | LB | 123456 | LBSEQ | 48 |  | 1 |

Additional examples may be found in the domain examples such as Section 6.2.4, Disposition, Example 4, and all of the Pharmacokinetics examples in Section 6.3.5.9.3, Relating PP Records to PC Records.
8.3 Relating Datasets
The Related Records (RELREC) special-purpose dataset can also be used to identify relationships between datasets (e.g., a one-to-many or parent-child relationship). The relationship is defined by including a single record for each related dataset that identifies the key(s) of the dataset that can be used to relate the respective records. Relationships between datasets should only be recorded in the RELREC dataset when the sponsor has found it necessary to split information between datasets that are related, and that may need to be examined together for analysis or proper interpretation. Note that it is not necessary to use the RELREC dataset to identify associations from data in the SUPP-- datasets or the Comments (CO) dataset to their parent general-observation class dataset records or special-purpose domain records, as both these datasets include the key variable identifiers of the parent record(s) that are necessary to make the association.

| Row | STUDYID | RDOMAIN | USUBJID | IDVAR | IDVARVA | L RELTYPE | RELID |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | EFC1234 | AE | 123456 | AESEQ | 5 |  | 1 |
| 2 | EFC1234 | CM | 123456 | CMGRPID | COMBO1 |  | 1 |
| 3 | EFC1234 | LB | 123456 | LBSEQ | 47 |  | 1 |
| 4 | EFC1234 | LB | 123456 | LBSEQ | 48 |  | 1 |


## Page 431

8.3.1 RELREC Dataset Relationship Example
Example 1 This example illustrates RELREC records used to represent the relationship between records in 2 datasets that have a one-to-many relationship. Note that because this is a dataset-to-dataset relationship, USUBJID and IDVARVAL are null. relrec.xpt

In the sponsor's operational database, these datasets may have existed as either separate datasets that were merged for analysis, or a single dataset that may have included observations from more than 1 general observation class (e.g., Events and Findings). The value in IDVAR must be the name of the key used to merge/join the 2 datasets. In this example, the --LNKID variable is used as the key to identify the related observations. The values for the ‑‑LNKID variable in the 2 datasets are sponsor-defined. Although other variables may also serve as a single merge key when the corresponding values for IDVAR are equal, --GRPID, --SPID, --REFID, --LNKID, or --LNKGRP are typically used for this purpose. The variable RELTYPE identifies the type of relationship between the datasets. The allowable values are ONE and MANY (controlled terminology is expected). This information defines how a merge/join would be written, and what would be the result of the merge/join. The possible combinations are:
1. ONE and ONE. This combination indicates that there is NO hierarchical relationship between the datasets
and the records in the datasets. Only 1 record from each dataset will potentially have the same value of the IDVAR within USUBJID.
2. ONE and MANY. This combination indicates that there IS a hierarchical (parent-child) relationship
between the datasets. One record within USUBJID in the dataset identified by RELTYPE = "ONE" will potentially have the same value of the IDVAR with many (1 or more) records in the dataset identified by RELTYPE = "MANY".
3. MANY and MANY. This combination is unusual and challenging to manage in a merge/join, and may
represent a relationship that was never intended to convey a usable merge/join, such as described in Section 6.3.5.9.3, Relating PP Records to PC Records. Because IDVAR identifies the keys that can be used to merge/join records between the datasets, --SEQ cannot be used. --SEQ only has meaning within a subject within a dataset, not across datasets.
8.4 Relating Non-standard Variable Values to a Parent
Domain The SDTM does not allow the addition of new variables. Therefore, the Supplemental Qualifiers special-purpose dataset model is used to capture non-standard variables (NSVs) and their association to parent records in generalobservation class datasets (Events, Findings, Interventions), Demographics (DM), and Subject Visits (SV). Supplemental qualifiers are represented as separate SUPP-- datasets for each dataset containing sponsor-defined variables (see Section 8.4.2, Submitting Supplemental Qualifiers in Separate Datasets). SUPP-- represents the metadata and data for each NSV/value combination. As the name suggests, this dataset is intended to capture additional qualifiers for an observation. Data that represent separate observations should be treated as separate observations. The Supplemental Qualifiers dataset is structured similarly to the RELREC dataset, in that it uses the same set of keys to identify parent records. Each SUPP-- record also includes the name of the qualifier variable being added (QNAM), the label for the variable (QLABEL), the actual value for each instance or record (QVAL), the origin (QORIG) of the value (see Section 4.1.8, Origin Metadata), and the evaluator (QEVAL) to specify the role of the individual who assigned the value (e.g., "ADJUDICATION COMMITTEE", "SPONSOR"). Controlled terminology for certain expected values for QNAM and QLABEL is included in Appendix C1, Supplemental Qualifiers Name Codes.

| Row | STUDYID | RDOMAIN | USUBJID | IDVAR | IDVARVAL | RELTYPE | RELID |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | EFC1234 | TU |  | TULNKID |  | ONE | 1 |
| 2 | EFC1234 | TR |  | TRLNKID |  | MANY | 1 |


## Page 432

SUPP-- datasets are also used to capture attributions. An attribution is typically an interpretation or subjective classification of 1 or more observations by a specific evaluator, such as a flag that indicates whether an observation was considered to be clinically significant. It is possible that different attributions may be necessary in some cases; SUPP-- provides a mechanism for incorporating as many attributions as are necessary. A SUPP-- dataset can contain both objective data (where values are collected or derived algorithmically) and subjective data (attributions where values are assigned by a person or committee). For objective data, the value in QEVAL will be null. For subjective data, the value in QEVAL should reflect the role of the person or institution assigning the value (e.g., "SPONSOR", "ADJUDICATION COMMITTEE"). The combined set of values for the first 6 columns (STUDYID…QNAM) should be unique for every record. That is, there should not be multiple records in a SUPP-- dataset for the same QNAM value, as it relates to IDVAR/IDVARVAL for a USUBJID in a domain. For example, if 2 individuals (e.g., the investigator and an independent adjudicator) provide a determination regarding whether an adverse event is treatment-emergent, then separate QNAM values should be used for each set of information (e.g., "AETRTEMI", "AETRTEMA"). This is necessary to ensure that reviewers can join/merge/transpose the information back with the records in the original domain without risk of losing information. Just as use of the optional grouping identifier variable (--GRPID) can be a more efficient method of representing relationships in RELREC, it can also be used in a SUPP-- dataset to identify individual qualifier values (SUPP-records) related to multiple general-observation class domain records that could be grouped, such as relating an attribution to a group of ECG measurements.
8.4.1 Supplemental Qualifiers (SUPP--)
supp--.xpt, Supplemental Qualifiers for [domain name] — Relationship. One record per supplemental qualifier per related parent domain record(s), Tabulation.

1In this column, an asterisk (*) indicates that the variable may be subject to controlled terminology. CDISC/NCI codelist values are enclosed in parentheses.

| Variable<br>Name | Variable<br>Label | Type | Controlled<br>Terms,<br>Codelist or<br>Format1 | Role | CDISC Notes | Core |
| --- | --- | --- | --- | --- | --- | --- |
| STUDYID | Study<br>Identifier | Char |  | Identifier | Study identifier of the parent record(s). | Req |
| RDOMAIN | Related<br>Domain<br>Abbreviation | Char | (DOMAIN) | Identifier | Two-character abbreviation for the domain of the parent record(s). | Req |
| USUBJID | Unique<br>Subject<br>Identifier | Char |  | Identifier | Identifier used to uniquely identify a subject across all studies for all | Req |
|  |  |  |  |  | applications or submissions involving the product. This is the value of |  |
|  |  |  |  |  | USUBJID in the parent record(s). |  |
| IDVAR | Identifying<br>Variable | Char | * | Identifier | Identifying variable in the dataset that identifies the related record(s). | Exp |
|  |  |  |  |  | Examples: --SEQ, --GRPID. |  |
| IDVARVAL | Identifying<br>Variable Value | Char |  | Identifier | Value of identifying variable of the parent record(s). | Exp |
| QNAM | Qualifier<br>Variable Name | Char | * | Topic | The short name of the qualifier variable, which is used as a column | Req |
|  |  |  |  |  | name in a domain view with data from the parent domain. The value in |  |
|  |  |  |  |  | QNAM cannot be longer than 8 characters, nor can it start with a |  |
|  |  |  |  |  | number (e.g., "1TEST" is not valid). QNAM cannot contain characters |  |
|  |  |  |  |  | other than letters, numbers, or underscores. This will often be the |  |
|  |  |  |  |  | column name in the sponsor's operational dataset. |  |
| QLABEL | Qualifier<br>Variable Label | Char |  | Synonym<br>Qualifier | This is the long name or label associated with QNAM. The value in | Req |
|  |  |  |  |  | QLABEL cannot be longer than 40 characters. This will often be the |  |
|  |  |  |  |  | column label in the sponsor's original dataset. |  |
| QVAL | Data Value | Char |  | Result<br>Qualifier | Result of, response to, or value associated with QNAM. A value for this | Req |
|  |  |  |  |  | column is required; no records can be in SUPP-- with a null value for |  |
|  |  |  |  |  | QVAL. |  |
| QORIG | Origin | Char |  | Record<br>Qualifier | Because QVAL can represent a mixture of collected (on a CRF), | Req |
|  |  |  |  |  | derived, or assigned items, QORIG is used to indicate the origin of this |  |
|  |  |  |  |  | data. Examples: "CRF", "Assigned", "Derived". See Section 4.1.8, Origin |  |
|  |  |  |  |  | Metadata. |  |
| QEVAL | Evaluator | Char | (EVAL) | Record<br>Qualifier | Used only for results that are subjective (e.g., assigned by a person or a | Exp |
|  |  |  |  |  | group). Should be null for records that contain objectively collected or |  |
|  |  |  |  |  | derived data. Examples: "ADJUDICATION COMMITTEE", |  |
|  |  |  |  |  | "STATISTICIAN", "DATABASE ADMINISTRATOR", "CLINICAL |  |
|  |  |  |  |  | COORDINATOR". |  |


## Page 433

A record in a SUPP-- dataset relates back to its parent record(s) via the key identified by the STUDYID, RDOMAIN, USUBJID, and IDVAR/IDVARVAL variables. An exception is SUPP-- dataset records that are related to Demographics (DM) records, where both IDVAR and IDVARVAL will be null because the key variables STUDYID, RDOMAIN, and USUBJID are sufficient to identify the unique parent record in DM (DM has 1 record per USUBJID). All records in the SUPP-- datasets must have a value for QVAL. Transposing source variables with missing/null values may generate SUPP-- records with null values for QVAL, causing the SUPP-- datasets to be extremely large. When this happens, the sponsor must delete the records where QVAL is null prior to submission. See Section 4.5.3, Text Strings that Exceed the Maximum Length for General Observation-class Domain Variables, for information on representing data values greater than 200 characters in length. See Appendix C1, Supplemental Qualifiers Name Codes, for controlled terminology for QNAM and QLABEL for some of the most common supplemental qualifiers. Additional QNAM values may be created as needed, following the guidelines provided in the CDISC Notes for QVAL.
8.4.2 Submitting Supplemental Qualifiers in Separate Datasets
There is a one-to-one correspondence between a domain dataset and its Supplemental Qualifier dataset. The single SUPPQUAL dataset option that was introduced in SDTMIG v3.1 was deprecated. The set of supplemental qualifiers for each domain is included in a separate dataset with the name SUPP-- (where "--" denotes the source domain which the supplemental qualifiers relate back to). For example, Demographics (DM) qualifiers would be submitted in suppdm.xpt. When data have been split into multiple datasets (see Section 4.1.7, Splitting Domains), longer names such as SUPPFAMH may be needed. In cases where data about associated persons have been collected, supplemental qualifiers for Findings About events or interventions for an associated person may need to be represented (see the SDTMIG for Associated Persons, https://www.cdisc.org/standards/foundational/sdtmig/). A dataset name with the SUPP fragment (e.g., SUPPAPFAMH) would be too long. In this case only, the “SUPP” portion should be shortened to “SQ” (e.g., resulting in the dataset name SQAPFAMH.
8.4.3 SUPP-- Examples
These examples llustrate how a set of SUPP-- datasets could be used to relate non-standard information to a parent domain. Example 1 The 2 rows of suppae.xpt add qualifying information to adverse event data (RDOMAIN = "AE"). IDVAR defines the key variable used to link this information to the AE data (AESEQ). IDVARVAL specifies the value of the key variable within the parent AE record to which the SUPPAE record applies. The remaining columns specify the supplemental variables’ names (AESOSP and AETRTEM), labels, values, origin, and who made the evaluation. suppae.xpt

Example This example illustrates how the language used for a questionnaire might be represented. The parent domain (RDOMAIN) is QS, and IDVAR is QSCAT. QNAM holds the name of the supplemental qualifier variable being defined (QSLANG). The language recorded in QVAL applies to all of the subject’s records, where IDVAR (QSCAT) equals the value specified in IDVARVAL. In this case, IDVARVAL has values for 2 questionnaires— Brief Pain Inventory (BPI) and Alzheimer's Disease Assessment Scale–Cognitive Subscale (ADAS-Cog)—for 2 separate subjects. QVAL identifies the questionnaire language version (French or German) for each subject.

| Row | STUDYID | RDOMAIN | USUBJID | IDVAR | IDVARVAL | QNAM | QLABEL | QVAL | QORIG | QEVAL |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 1996001 | AE | 99-401 | AESEQ | 1 | AESOSP | Other Medically<br>Important SAE | Spontaneous<br>Abortion | CRF |  |
| 2 | 1996001 | AE | 99-401 | AESEQ | 1 | AETRTEM | Treatment Emergent<br>Flag | N | Derived | SPONSOR |


## Page 434

suppqs.xpt

Additional examples may be found in the domain examples, such as in Section 5.2 Demographics, Examples 3 and 4, in Section 6.3.3, ECG Test Results, Example 1, and in Section 6.3.5.6, Laboratory Test Results, Example 1.
8.4.4 When Not to Use Supplemental Qualifiers
The following are examples of data that should not be submitted as supplemental qualifiers:
• Subject-level objective data that fit in Subject Characteristics (SC; e.g., national origin, twin type)
• Findings interpretations that should be added as an additional test code and result. An example of this
would be a record for electrocardiogram interpretation where EGTESTCD = “INTP”, and the same EGGRPID or EGREFID value would be assigned for all records associated with that ECG (see Section 4.5.5, Clinical Significance for Findings Observation Class Data).
• Comments related to a record or records contained within a parent dataset. Although they may have been
collected in the same record by the sponsor, comments should instead be captured in the CO specialpurpose domain.
• Data not directly related to records in a parent domain. Such records should instead be captured in either a
separate general observation class domain or special-purpose domain.
8.5 Relating Comments to a Parent Domain
The Comments (CO) special-purpose domain (see Section 5.1, Comments) is used to capture unstructured free-text comments. It allows for the submission of comments related to a particular domain (e.g., Adverse Events) or those collected on separate general-comment log-style pages not associated with a domain. Comments may be related to a subject, a domain for a subject, or to specific parent records in any domain. The CO special-purpose domain is structured similarly to the Supplemental Qualifiers (SUPP--) dataset, in that it uses the same set of keys (STUDYID, RDOMAIN, USUBJID, IDVAR, and IDVARVAL) to identify related records. All comments except those collected on log-style pages not associated with a domain are considered child records of subject data captured in domains. STUDYID, USUBJID, and DOMAIN (with the value CO) must always be populated. RDOMAIN, IDVAR, and IDVARVAL should be populated as follows:
1. Comments related only to a subject in general (likely collected on a log-style CRF page/screen) would have
RDOMAIN, IDVAR, IDVARVAL null, as the only key needed to identify the relationship/association to that subject is USUBJID.
2. Comments related only to a specific domain (and not to any specific records) for a subject would populate
RDOMAIN with the domain code for the domain with which they are associated. IDVAR and IDVARVAL would be null.
3. Comments related to specific domain record(s) for a subject would populate the RDOMAIN, IDVAR, and
IDVARVAL variables with values that identify the specific parent record(s). For additional information collected further describing the comment relationship to a parent record(s) that cannot be represented using the relationship variables RDOMAIN, IDVAR and IDVARVAL:
1. Values (e.g., CRF page number or name) may be placed in COREF.
2. Timing variables (e.g., VISITNUM, VISIT) may be added to the CO special-purpose domain. See Section
5.1, Comments, assumption 5 for a complete list of identifier and timing variables that can be added to the CO special-purpose domain. As with Supplemental Qualifiers (SUPP--) and Related Records (RELREC), --GRPID and other grouping variables can be used as the value in IDVAR to identify comments with relationships to multiple domain records, for example a comment that applies to a group of concomitant medications, perhaps taken as a combination therapy. The

| Row | STUDYID | RDOMAIN | USUBJID | IDVAR | IDVARVAL | QNAM | QLABEL | QVAL | QORIG | QEVAL |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 1996001 | QS | 99-401 | QSCAT | BPI | QSLANG | Questionnaire Language | FRENCH | CRF |  |
| 2 | 1996001 | QS | 99-401 | QSCAT | ADAS-COG | QSLANG | Questionnaire Language | FRENCH | CRF |  |
| 3 | 1996001 | QS | 99-802 | QSCAT | BPI | QSLANG | Questionnaire Language | GERMAN | CRF |  |
| 4 | 1996001 | QS | 99-802 | QSCAT | ADAS-COG | QSLANG | Questionnaire Language | GERMAN | CRF |  |


## Page 435

limitation of this is that a single comment may only be related to a group of records in 1 domain (RDOMAIN can have only 1 value). If a single comment relates to records in multiple domains, the comment may need to be repeated in the CO special-purpose domain to facilitate the understanding of the relationships. See examples for CO data in Section 5.1, Comments.
8.6 How to Determine Where Data Belong in SDTM-
Compliant Data Tabulations
8.6.1 Guidelines for Determining the General Observation Class
Section 2.6, Creating a New Domain, discusses when to place data in an existing domain and how to create a new domain. A key part of the process of creating a new domain is determining whether an observation represents an event, an intervention, or a finding. Begin by considering the content of the information in the light of the definitions of the 3 general observation classes (see Section 2.3, The General Observation Classes), rather than by trying to deduce the class from the information's physical structure; physical structure can sometimes be misleading. For example, from a structural standpoint, one might expect events observations to include a start and stop date. However, medical history data (data about previous conditions or events) is events data regardless of whether dates were collected. An intervention is something that is done to a subject (possibly by the subject) that is expected to have a physiological effect. This concept of intended effect makes interventions relatively easy to recognize, although there are gray areas around some testing procedures. For example, exercise stress tests are designed to produce and then measure certain physiological effects. The measurements from such a testing procedure are findings, although some aspects of the procedure might be modeled as interventions. An event is something that happens to a subject spontaneously. Most, although not all, events data captured in clinical trials is about medical events. Because many medical events must, by regulation, be treated as adverse events, a new Events domain will be created only for events that are clearly not adverse events; the existing Medical History (MH) and Clinical Events (CE) domains are the appropriate places to store most medical events that are not adverse events. Many aspects of medical events—including tests performed to evaluate them, interventions that may have caused them, and interventions given to treat them—may be collected in clinical trials. Where to place data on assessments of events can be particularly challenging, and is discussed further in Section 8.6.3, Guidelines for Differentiating Between Interventions, Events, Findings, and Findings About Events or Interventions. Findings general observation class data include measurements, tests, assessments, or examinations performed on a subject in the clinical trial. These may be performed on the subject as a whole (e.g., height, heart rate), or on a specimen taken from a subject (e.g., blood sample, ECG tracing, tissue sample). Sometimes the relationship between a subject and a finding is less direct; a finding may be about an event that happened to the subject or an intervention received. Findings about events and interventions are discussed further in Section 8.6.3, Guidelines for Differentiating Between Interventions, Events, Findings, and Findings About Events or Interventions.
8.6.2 Guidelines for Forming New Domains
It may not always be clear whether a set of data represents a single topic or more, and thus whether it should be combined into 1 domain or split into 2 or more domains. This implementation guide shows examples of both. In some cases, a single data structure works well for a variety of types of data. For example, all questionnaire data are placed in the QS domain, with particular questionnaires identified by QSCAT (see Section 6.3.9, Questionnaires, Ratings, and Scales (QRS) Domains (FT, QS, RS)). Although some operational databases may store urinalysis data in a separate dataset, the SDTM places all lab data in the LB domain (see Section 6.3.5.6, Laboratory Test Results (LB)) with urinalysis tests identified using LBSPEC. In other cases, a particular topic may be very broad and/or require more than 1 data structure (and therefore require more than 1 dataset). This implementation guide includes examples in microbiology and pharmacokinetics (see Section 6.3.5.7, Microbiology Domains, and Section 6.3.5.9, Pharmacokinetics Domains). These examples use 2 domain datasets because within these scientific areas there is more than 1 topic, and each topic results in a different data structure. In the Pharmacokinetics domain example, the topic for PC is plasma (or other specimen) drug


## Page 436

concentration as a function of time, and the structure is 1 record per analyte per time point per reference time point (e.g., dosing event) per subject. PP contains characteristics of the time-concentration curve (e.g., AUC, Cmax, Tmax, half-life) and elimination rate constant; the structure is 1 record per parameter per analyte per reference time point per subject.
8.6.3 Guidelines for Differentiating Between Interventions, Events,
Findings, and Findings About Events or Interventions This section discusses events, findings, and findings about events. The relationship between interventions, findings, and findings about interventions would be handled similarly. The Findings About (FA) domain was initially created to represent findings about events, but can also be used for findings about interventions. This section discusses events and findings generally, but it is particularly useful for understanding the distinction between the Clinical Events (CE) and FA domains. There may be several sources of confusion about whether a particular piece of data belongs in an Event record or a Findings record. Although an "event" is generally perceived as something that happens spontaneously, and has a beginning and end, one should consider the following:
• Events of interest in a particular trial may be prespecified, rather than collected as free text.
• Some events may be so long-lasting that they are perceived as "conditions" rather than events, and their
beginning and end dates are not of interest.
• Some variables or data items one generally expects to see in an Events record may not be present. For
example, a post-marketing study might collect the occurrence of certain adverse events, but no dates.
• Properties of an event may be measured or assessed, and these are then treated as findings about events,
rather than as events.
• Some assessments of events (e.g., severity, relationship to study treatment) have been built into the SDTM
Events model as qualifiers, rather than being treated as findings about events.
• Sponsors may choose how they define an event. See Section 6.2.1, Adverse Events, assumption 7e.
The structure of the data being considered, although not definitive, will often help determine whether the data represent as an event or a finding. The following table presents questions that may assist sponsors in deciding where data should be placed in SDTM.

| Question | Interpretation of Answers |
| --- | --- |
| Is this a measurement, with<br>units, etc.? | • A “Yes” answer indicates a finding. If the measurement is of some aspect of an event, it may be<br>represented as a finding about the event.<br>• A “No” answer is inconclusive. |
| Are the data collected in a<br>CRF for each visit, or an<br>overall CRF log form? | • Log forms (forms that are completed over the course of the study rather than at a single visit)<br>suggest Events or Interventions general observation class data. Dates collected are usually<br>start and end dates of an event or intervention. Dates when the information was collected are<br>generally not of interest and are usually not recorded by the investigator.<br>• Observations made at individual visits are usually planned findings.<br>o Data collected at an initial visit may include information about past events and interventions,<br>as well as findings. |
| What date/times are<br>collected? | • If the dates collected are start and/or end dates, then data are probably about an event or<br>intervention.<br>• If the dates collected are dates of assessments, then data probably represent a finding.<br>• If dates of collection are different from other dates collected, it suggests that data are historical<br>or are about an event or intervention that happened independently of the study schedule for<br>data collection. |
| Is verbatim text collected and<br>then coded? | • A “Yes” answer suggests that this is Events or Interventions general observation class data.<br>However, Findings general observation class data from an examination (e.g., physical<br>examination) that identifies abnormalities may also be coded). Note that for Events and<br>Interventions general observation class data, the topic variable is coded, whereas for Findings<br>general observation class data, the result is coded. |


## Page 437

Question Interpretation of Answers
• A “No” answer is inconclusive. This does not rule out Events or Interventions general
observation class data, because events or interventions are not always collected as verbatim text, but may be prespecified or chosen from a list; it also does not rule out Findings general observation class data. If this is data about an event, • A “Yes” answer suggests this is traditional Events general observation class data, and it should does it apply to the event as have a record in an Events domain. a whole? • A “No” answer suggests that there are multiple time-based findings about an event, and that these data should be treated as Findings About data. Does this data meet the • The criteria for QRS are available at: https://www.cdisc.org/foundational/qrs. criteria for representation in a • There are many standard questionnaires and rating scales that collect information about Questionnaires, Ratings, and symptoms. The fact that these data are findings about symptoms does not mean that the data Scales (QRS) domain? should be represented in FA rather than a QRS domain (QS, FT, or RS).
• Data about symptoms may be collected in a format that is usually associated with
questionnaires and rating scales (e.g., visual analog scale). This format alone does not mean that the data should be represented in a QRS domain. A record in the Events general observation class is intended to represent the event as a whole. (Note that sponsors may choose how they define the "event as a whole." See Section 6.2.1, Adverse Events, assumption 7e.) Such a record typically includes what the condition was (captured in --TERM, the topic variable) and when it happened (captured in its start and/or end dates). Other qualifier values (e.g., severity, seriousness) apply to the totality of the event. Data that do not describe the event as a whole should not be stored in the record for that event or in a --SUPP record tied to that event. If there are multiple assessments of an event, then each should be stored in a separate FA record. When data related to an event do not fit into one of the existing Event general observation class qualifiers, the first question to consider is whether the data represent information about the event itself, or about something (a finding or intervention) that is associated with the event.
• If the data consist of a finding or intervention that is associated with the event, it is likely that it can be
stored in a relevant Findings or Intervention general observation class dataset, with the connection to the Event record being captured using RELREC. For example, if a subject had a fever of 102°F that was treated with aspirin, the fever would be stored in an AE record, the temperature could be stored in a Vital Signs record, and the aspirin could be stored in a Concomitant Medication record; RELREC might be used to link those records.
• If the data item contains information about the event, then the choices for representing it are a
supplemental qualifier or in an FA record. The data should be represented as a supplemental qualifier unless circumstances rule out the use of a supplemental qualifier. If a supplemental qualifier is not appropriate, the data may be stored in FA. See Section 6.4.1, When to Use Findings About Events or Interventions.
8.7 Related Subjects (RELSUB)
RELSUB – Description/Overview A dataset used to represent relationships between study subjects. Some studies include subjects who are related to each other, and in some cases it is important to record those relationships. A study in which pregnant women are treated and both the mother and her child(ren) are study subjects is the most common case in which relationships between subjects are collected. There are also studies of genetically based diseases where subjects who are related to each other are enrolled, and the relationships between subjects are recorded.


## Page 438

RELSUB – Specification relsub.xpt, Related Subjects — Relationship. One record per relationship per related subject per subject, Tabulation.

1In this column, an asterisk (*) indicates that the variable may be subject to controlled terminology. CDISC/NCI codelist values are enclosed in parentheses. RELSUB – Assumptions
1. RELSUB is used to represent relationships between persons, both of whom are study subjects. A
relationship between a study subject and a person who is not a study subject may not be represented in RELSUB; this may only be reported in APRELSUB. The existence of the RELSUB dataset should not affect whether relationships are collected; that should remain a decision based on the needs of the particular study.
2. The variable POOLID was developed for nonclinical studies, where assessments may be made for groups
of animals, and identifiers are needed for those groups (pools). It is included here because POOLID can be used for human clinical trials, if necessary. If POOLID is submitted, the POOLDEF dataset must be submitted.
3. If POOLID is submitted, then in any record, 1 and only 1 of USUBJID and POOLID must be populated.
4. If a study does not include the use of POOLID, then USUBJID must be populated in every record.
5. RSUBJID must be a USUBJID value present in the Demographics (DM) domain. RSUBJID must be
populated in every record.
6. Values of SREL should be taken from the CDISC Controlled Terminology codelist RELSUB wherever
possible. However, if an appropriate term does not exist in the codelist, another term may be used. The SREL term should not be less specific than the verbatim term collected. For instance, it would be inappropriate to record a relationship using the term "RELATIVE, FIRST DEGREE" when the collected relationship was "brother".
7. Every relationship between 2 study subjects is represented in RELSUB as 2 directional relationships: (1)
with the first subject's identifier in USUBJID and the second subject's identifier in RSUBJID, and (2) with the second subject's identifier in USUBJID and the first subject's identifier in RSUBJID. The SREL values in the 2 records will describe the same relationship, but from the viewpoint of each subject (e.g., "MOTHER, BIOLOGICAL"; "CHILD, BIOLOGICAL")
8. All collected relationships between subjects should be recorded in RELSUB. In some cases, 2 subjects may
have more than 1 relationship. For instance, a woman might be both maternal aunt and wet nurse to an infant. When there are multiple relationships between 2 subjects, each relationship will be represented by 2 records in RELSUB.

| Variable<br>Name | Variable Label | Type | Controlled<br>Terms, Codelist<br>or Format1 | Role | CDISC Notes | Core |
| --- | --- | --- | --- | --- | --- | --- |
| STUDYID | Study Identifier | Char |  | Identifier | Unique identifier for a study. | Req |
| USUBJID | Unique Subject<br>Identifier | Char |  | Identifier | Identifier used to uniquely identify a subject across all | Exp |
|  |  |  |  |  | studies for all applications or submissions involving the |  |
|  |  |  |  |  | product. Either USUBJID or POOLID must be populated. |  |
| POOLID | Pool Identifier | Char |  | Identifier | Identifier used to identify a pool of subjects. If POOLID is | Perm |
|  |  |  |  |  | entered, POOLDEF records must exist for each subject in |  |
|  |  |  |  |  | the pool and USUBJID must be null. Either USUBJID or |  |
|  |  |  |  |  | POOLID must be populated. |  |
| RSUBJID | Related Subject<br>or Pool Identifier | Char |  | Identifier | Identifier used to identify a related subject or pool of | Req |
|  |  |  |  |  | subjects. RSUBJID will be populated with either the |  |
|  |  |  |  |  | USUBJID of the related subject or the POOLID of the |  |
|  |  |  |  |  | related pool. |  |
| SREL | Subject<br>Relationship | Char | (RELSUB) | Record<br>Qualifier | Describes the relationship of the subject identified in | Req |
|  |  |  |  |  | USUBJID or the pool identified in POOLID to the subject or |  |
|  |  |  |  |  | pool identified in RSUBJID. |  |


## Page 439

RELSUB – Examples Example 1 The following data are from a hemophilia study (HEM021) in which the study subjects are a pair of fraternal (dizygotic) twins and their mother. Some expected and required variables not needed to illustrate the example are not shown. Row 1: Subject is the mother. Rows 2-3: Subjects are the children. dm.xpt

The RELSUB table is for the 3 subjects whose demography data is shown in the preceding table. Rows 1-2: The relationship of the mother to the 2 children. Rows 3, 5: The relationships of the children to the mother. Rows 4, 6: The relationships of the children to each other. relsub.xpt

| Row | STUDYID | DOMAIN | USUBJID | BRTHDTC | AGE | AGEU | SEX |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | HEM021 | DM | HEM021-001 | 1941-05-16 | 60 | YEARS | F |
| 2 | HEM021 | DM | HEM021-002 | 1965-04-12 | 35 | YEARS | M |
| 3 | HEM021 | DM | HEM021-003 | 1965-04-12 | 35 | YEARS | M |

8.8 Related Specimens (RELSPEC)
BE, BS, and RELSPEC domain specifications, assumptions, and examples were copied and minimally updated from the provisional SDTMIG-PGx, published 2015-05-26. This was done in preparation for the retirement of the SDTMIG-PGx upon publication of SDTMIG v3.4. These domains are currently under extensive revision for inclusion in a future SDTMIG. RELSPEC – Description/Overview A dataset used to represent relationships between specimens. RELSPEC – Specification relspec.xpt, Related Specimens — Relationship. One record per specimen identifier per subject, Tabulation.

| Row | STUDYID | USUBJID | RSUBJID | SREL |
| --- | --- | --- | --- | --- |
| 1 | HEM021 | HEM021-001 | HEM021-002 | MOTHER, BIOLOGICAL |
| 2 | HEM021 | HEM021-001 | HEM021-003 | MOTHER, BIOLOGICAL |
| 3 | HEM021 | HEM021-002 | HEM021-001 | CHILD, BIOLOGICAL |
| 4 | HEM021 | HEM021-002 | HEM021-003 | TWIN, DIZYGOTIC |
| 5 | HEM021 | HEM021-003 | HEM021-001 | CHILD, BIOLOGICAL |
| 6 | HEM021 | HEM021-003 | HEM021-002 | TWIN, DIZYGOTIC |

1In this column, an asterisk (*) indicates that the variable may be subject to controlled terminology. CDISC/NCI codelist values are enclosed in parentheses.

| Variable<br>Name | Variable<br>Label | Type | Controlled Terms, Codelist<br>or Format1 | Role | CDISC Notes | Core |
| --- | --- | --- | --- | --- | --- | --- |
| STUDYID | Study<br>Identifier | Char |  | Identifier | Unique identifier for a study. | Req |
| USUBJID | Unique<br>Subject<br>Identifier | Char |  | Identifier | Identifier used to uniquely identify a subject | Req |
|  |  |  |  |  | across all studies for all applications or |  |
|  |  |  |  |  | submissions involving the product. |  |
| REFID | Specimen ID | Char |  | Identifier | Specimen identifier, unique within USUBJID. | Req |
| SPEC | Specimen<br>Type | Char | (SPECTYPE)(GENSMP) | Variable<br>Qualifier | Defines the type of specimen used for a | Perm |
|  |  |  |  |  | measurement. Examples: "SERUM", "PLASMA", |  |
|  |  |  |  |  | "URINE", "SOFT TISSUE". |  |
| PARENT | Specimen<br>Parent | Char |  | Identifier | Identifies the REFID of the parent of a specimen | Exp |
|  |  |  |  |  | to support tracking its genealogy. |  |
| LEVEL | Specimen<br>Level | Num |  | Variable<br>Qualifier | Identifies the generation number of the sample | Req |
|  |  |  |  |  | where the collected sample is considered the |  |
|  |  |  |  |  | first generation. |  |


## Page 440

RELSPEC – Assumptions
1. The RELSPEC dataset is not used to manage relationships between any other datasets or domains.
2. The RELSPEC dataset is only used to maintain relationships between specimens, therefore is does not
require any additional variables such as those used in RELREC.
3. There are three CDISC controlled terminology codelists that may be applicable to SPEC: SPEC (C77529),
SPECTYPE (C78734), and GENSMP (C111114). Sponsors are responsible for determining the most appropriate codelist(s) for their submission. RELSPEC – Examples Example 1 This example uses the sample specimen lineage illustrated below. Figure. Sample Specimen Relationship A specimen with a LEVEL value of "1" and a blank value for PARENT indicates a collected sample. All other values represent a derived sample. SPEC reflects the specimen type for the sample regardless of whether it is collected or derived. relspec.xpt

| Row | STUDYID | USUBJID | REFID | SPEC | PARENT | LEVEL |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | ABC-123 | 001-01 | SPC-001 | TISSUE |  | 1 |
| 2 | ABC-123 | 001-01 | SPC-001-A | TISSUE | SPC-001 | 2 |
| 3 | ABC-123 | 001-01 | SPC-001-B | TISSUE | SPC-001 | 2 |
| 4 | ABC-123 | 001-01 | SPC-001-B-1 | DNA | SPC-001-B | 3 |
| 5 | ABC-123 | 001-01 | SPC-003 | TISSUE |  | 1 |
| 6 | ABC-123 | 001-01 | SPC-003-A | RNA | SPC-003 | 2 |

