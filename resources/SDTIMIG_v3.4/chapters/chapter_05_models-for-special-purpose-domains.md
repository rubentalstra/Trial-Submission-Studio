# Chapter 5: MODELS FOR SPECIAL-PURPOSE DOMAINS

> Source pages 60–91 in `SDTMIG_v3.4.pdf`.

## Page 60

5 Models for Special-purpose Domains Special-purpose Domains is an SDTM class in
its own right. Special-purpose domains provide specific, standardized structures
to represent additional important information that does not fit any of the
general observation classes. 5.1 Comments (CO) CO – Description/Overview A
special-purpose domain that contains comments that may be collected alongside
other data. CO – Specification co.xpt, Comments — Special Purpose. One record
per comment per subject, Tabulation.

1In this column, an asterisk (*) indicates that the variable may be subject to
controlled terminology. CDISC/NCI codelist values are enclosed in parentheses.

| Variable<br>Name | Variable Label                 | Type | Controlled Terms,<br>Codelist or Format1 | Role                | CDISC Notes                                                                                                     | Core |
| ---------------- | ------------------------------ | ---- | ---------------------------------------- | ------------------- | --------------------------------------------------------------------------------------------------------------- | ---- |
| STUDYID          | Study Identifier               | Char |                                          | Identifier          | Unique identifier for a study.                                                                                  | Req  |
| DOMAIN           | Domain<br>Abbreviation         | Char | CO                                       | Identifier          | Two-character abbreviation for the domain.                                                                      | Req  |
| RDOMAIN          | Related Domain<br>Abbreviation | Char | (DOMAIN)                                 | Record<br>Qualifier | Two-character abbreviation for the domain of the parent record(s). Null for comments collected on a             | Perm |
|                  |                                |      |                                          |                     | general comments or additional information CRF page.                                                            |      |
| USUBJID          | Unique Subject<br>Identifier   | Char |                                          | Identifier          | Identifier used to uniquely identify a subject across all studies for all applications or submissions involving | Req  |
|                  |                                |      |                                          |                     | the product.                                                                                                    |      |
| COSEQ            | Sequence Number                | Num  |                                          | Identifier          | Sequence Number given to ensure uniqueness of subject records within a domain. May be any valid                 | Req  |
|                  |                                |      |                                          |                     | number.                                                                                                         |      |
| IDVAR            | Identifying<br>Variable        | Char | *                                        | Record<br>Qualifier | Identifying variable in the parent dataset that identifies the record(s) to which the comment applies.          | Perm |
|                  |                                |      |                                          |                     | Examples AESEQ or CMGRPID. Used only when individual comments are related to domain records.                    |      |
|                  |                                |      |                                          |                     | Null for comments collected on separate CRFs.                                                                   |      |
| IDVARVAL         | Identifying<br>Variable Value  | Char |                                          | Record<br>Qualifier | Value of identifying variable of the parent record(s). Used only when individual comments are related to        | Perm |
|                  |                                |      |                                          |                     | domain records. Null for comments collected on separate CRFs.                                                   |      |
| COREF            | Comment<br>Reference           | Char |                                          | Record<br>Qualifier | Sponsor-defined reference associated with the comment. May be the CRF page number (e.g., 650), or a             | Perm |
|                  |                                |      |                                          |                     | module name (e.g., DEMOG), or a combination of information that identifies the reference (e.g. 650-             |      |
|                  |                                |      |                                          |                     | VITALS-VISIT 2).                                                                                                |      |
| COVAL            | Comment                        | Char |                                          | Topic               | The text of the comment. Text over 200 characters can be added to additional columns COVAL1-                    | Req  |
|                  |                                |      |                                          |                     | COVALn. See Assumption 3.                                                                                       |      |
| COEVAL           | Evaluator                      | Char | (EVAL)                                   | Record<br>Qualifier | Role of the person who provided the evaluation. Used only for results that are subjective (e.g., assigned       | Perm |
|                  |                                |      |                                          |                     | by a person or a group). Example: "INVESTIGATOR".                                                               |      |
| COEVALID         | Evaluator Identifier           | Char | (MEDEVAL)                                | Record<br>Qualifier | Used to distinguish multiple evaluators with the same role recorded in --EVAL. Examples:                        | Perm |
|                  |                                |      |                                          |                     | "RADIOLOGIST", "RADIOLOGIST 1", "RADIOLOGIST 2".                                                                |      |
| CODTC            | Date/Time of<br>Comment        | Char | ISO 8601 datetime or<br>interval         | Timing              | Date/time of comment on dedicated comment form. Should be null if this is a child record of another             | Perm |
|                  |                                |      |                                          |                     | domain or if comment date was not collected.                                                                    |      |
| CODY             | Study Day of<br>Comment        | Num  |                                          | Timing              | Study day of the comment, in integer days. The algorithm for calculations must be relative to the sponsor-      | Perm |
|                  |                                |      |                                          |                     | defined RFSTDTC variable in the Demographics (DM) domain.                                                       |      |

## Page 61

CO – Assumptions

1. The Comments special-purpose domain provides a solution for submitting
   free-text comments related to data in 1 or more SDTM domains (as described in
   Section 8.5, Relating Comments to a Parent Domain) or collected on a separate
   CRF page dedicated to comments. Comments are generally not responses to
   specific questions; instead, comments usually consist of voluntary free-text
   or unsolicited observations.
2. Although the structure for the Comments domain in the SDTM is "One record per
   comment", USUBJID is required in the comments domain for human clinical
   trials, so the structure of the Comments domain in the SDTMIG is "One record
   per comment per subject."
3. The CO dataset accommodates 3 sources of comments: a. Those unrelated to a
   specific domain or parent record(s), in which case the values of the
   variables RDOMAIN, IDVAR, and IDVARVAL are null. CODTC should be populated if
   captured. See Example 1, row 1. b. Those related to a domain but not to
   specific parent record(s), in which case the value of the variable RDOMAIN is
   set to the DOMAIN code of the parent domain and the variables IDVAR and
   IDVARVAL are null. CODTC should be populated if captured. See Example 1,
   row 2. c. Those related to a specific parent record or group of parent
   records, in which case the value of the variable RDOMAIN is set to the DOMAIN
   code of the parent record(s) and the variables IDVAR and IDVARVAL are
   populated with the key variable name and value of the parent record(s).
   Assumptions for populating IDVAR and IDVARVAL are further described in
   Section 8.5, Relating Comments to a Parent Domain. CODTC should be null
   because the timing of the parent record(s) is inherited by the comment
   record. See Example 1, rows 3-5.
4. When the comment text is longer than 200 characters, the first 200 characters
   of the comment will be in COVAL, the next 200 in COVAL1, and additional text
   stored as needed to COVALn. See Example 1, rows 3-4. Additional information
   about how to relate comments to parent SDTM records is provided in Section
   8.5, Relating Comments to a Parent Domain.
5. The variable COREF may be null unless it is used to identify the source of
   the comment. See Example 1, rows 1 and 5.
6. Identifier variables and Timing variables may be added to the CO domain, but
   the following qualifiers would generally not be used in CO: --GRPID, --
   REFID, --SPID, TAETORD, --TPT, --TPTNUM, --ELTM, --TPTREF, --RFTDTC. CO –
   Examples Example 1 Row 1: Shows a comment collected on a separate comments
   page. Since it was unrelated to any specific domain or record, RDOMAIN,
   IDVAR, and IDVARVAL are null. Row 2: Shows a comment that was collected on
   the bottom of the PE page for Visit 7, without any indication of specific
   records it applied to. Since the comment related to a specific domain,
   RDOMAIN is populated. Since it was related to a specific visit, VISIT, COREF
   is "VISIT 7". However, since it does not relate to a specific record, IDVAR
   and IDVARVAL are null. Row 3: Shows a comment related to a single AE record
   having its AESEQ=7. Row 4: Shows a comment related to multiple EX records
   with EXGRPID = "COMBO1". Row 5: Shows a comment related to multiple VS
   records with VSGRPID = "VS2".

## Page 62

Row 6: Shows one option for representing a comment collected on a visit-specific
comments page not associated with a particular domain. In this case, the comment
is linked to the Subject Visit record in SV (RDOMAIN = "SV") and IDVAR and
IDVARVAL are populated link the comment to the particular visit. Row 7: Shows a
second option for representing a comment associated only with a visit. In this
case, COREF is used to show that the comment is related to the particular visit.
Row 8: Shows a third option for representing a comment associated only with a
visit. In this case, the VISITNUM variable was populated to indicate that the
comment was associated with a particular visit. co.xpt

5.2 Demographics (DM) DM – Description/Overview A special-purpose domain that
includes a set of essential standard variables that describe each subject in a
clinical study. It is the parent domain for all other observations for human
clinical subjects. DM – Specification dm.xpt, Demographics — Special Purpose.
One record per subject, Tabulation.

| Row | STUDYID | DOMAIN | RDOMAIN | USUBJID | COSEQ | IDVAR    | IDVARVAL | COREF            | COVAL                   | COVAL1                 | COVAL2            | COEVAL                    | VISITNUM | CODTC          |
| --- | ------- | ------ | ------- | ------- | ----- | -------- | -------- | ---------------- | ----------------------- | ---------------------- | ----------------- | ------------------------- | -------- | -------------- |
| 1   | 1234    | CO     |         | AB-99   | 1     |          |          |                  | Comment text            |                        |                   | PRINCIPAL<br>INVESTIGATOR |          | 2003-11-<br>08 |
| 2   | 1234    | CO     | PE      | AB-99   | 2     |          |          | VISIT 7          | Comment text            |                        |                   | PRINCIPAL<br>INVESTIGATOR |          | 2004-01-<br>14 |
| 3   | 1234    | CO     | AE      | AB-99   | 3     | AESEQ    | 7        | PAGE 650         | First 200<br>characters | Next 200<br>characters | Remaining<br>text | PRINCIPAL<br>INVESTIGATOR |          |                |
| 4   | 1234    | CO     | EX      | AB-99   | 4     | EXGRPID  | COMBO1   | PAGE 320-<br>355 | First 200<br>characters | Remaining text         |                   | PRINCIPAL<br>INVESTIGATOR |          |                |
| 5   | 1234    | CO     | VS      | AB-99   | 5     | VSGRPID  | VS2      |                  | Comment text            |                        |                   | PRINCIPAL<br>INVESTIGATOR |          |                |
| 6   | 1234    | CO     | SV      | AB-99   | 6     | VISITNUM | 4        |                  | Comment Text            |                        |                   | PRINCIPAL<br>INVESTIGATOR |          |                |
| 7   | 1234    | CO     |         | AB-99   | 7     |          |          | VISIT 4          | Comment Text            |                        |                   | PRINCIPAL<br>INVESTIGATOR |          |                |
| 8   | 1234    | CO     |         | AB-99   | 8     |          |          |                  | Comment Text            |                        |                   | PRINCIPAL<br>INVESTIGATOR | 4        |                |

| Variable<br>Name | Variable Label                     | Type | Controlled Terms,<br>Codelist or<br>Format1 | Role       | CDISC Notes                                                                                                         | Core |
| ---------------- | ---------------------------------- | ---- | ------------------------------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------- | ---- |
| STUDYID          | Study Identifier                   | Char |                                             | Identifier | Unique identifier for a study.                                                                                      | Req  |
| DOMAIN           | Domain<br>Abbreviation             | Char | DM                                          | Identifier | Two-character abbreviation for the domain.                                                                          | Req  |
| USUBJID<br>I     | Unique Subject<br>dentifier        | Char |                                             | Identifier | Identifier used to uniquely identify a subject across all studies for all applications or submissions involving the | Req  |
|                  |                                    |      |                                             |            | product. This must be a unique value, and could be a compound identifier formed by concatenating                    |      |
|                  |                                    |      |                                             |            | STUDYID-SITEID-SUBJID.                                                                                              |      |
| SUBJID<br>f      | Subject Identifier<br>or the Study | Char |                                             | Topic      | Subject identifier, which must be unique within the study. Often the ID of the subject as recorded on a CRF.        | Req  |

## Page 63

| Variable<br>Name | Variable Label                                 | Type | Controlled Terms,<br>Codelist or<br>Format1 | Role                  | CDISC Notes                                                                                                     | Core |
| ---------------- | ---------------------------------------------- | ---- | ------------------------------------------- | --------------------- | --------------------------------------------------------------------------------------------------------------- | ---- |
| RFSTDTC          | Subject Reference<br>Start Date/Time           | Char | ISO 8601 datetime<br>or interval            | Record<br>Qualifier   | Reference start date/time for the subject in ISO 8601 character format. Usually equivalent to date/time when    | Exp  |
|                  |                                                |      |                                             |                       | subject was first exposed to study treatment. See assumption 9 for additional detail on when RFSTDTC may        |      |
|                  |                                                |      |                                             |                       | be null.                                                                                                        |      |
| RFENDTC          | Subject Reference<br>End Date/Time             | Char | ISO 8601 datetime<br>or interval            | Record<br>Qualifier   | Reference end date/time for the subject in ISO 8601 character format. Usually equivalent to the date/time       | Exp  |
|                  |                                                |      |                                             |                       | when subject was determined to have ended the trial, and often equivalent to date/time of last exposure to      |      |
|                  |                                                |      |                                             |                       | study treatment. Required for all randomized subjects; null for screen failures or unassigned subjects.         |      |
| RFXSTDTC         | Date/Time of First<br>Study Treatment          | Char | ISO 8601 datetime<br>or interval            | Record<br>Qualifier   | First date/time of exposure to any protocol-specified treatment or therapy, equal to the earliest value of      | Exp  |
|                  |                                                |      |                                             |                       | EXSTDTC.                                                                                                        |      |
| RFXENDTC         | Date/Time of Last<br>Study Treatment           | Char | ISO 8601 datetime<br>or interval            | Record<br>Qualifier   | Last date/time of exposure to any protocol-specified treatment or therapy, equal to the latest value of         | Exp  |
|                  |                                                |      |                                             |                       | EXENDTC (or the latest value of EXSTDTC if EXENDTC was not collected or is missing).                            |      |
| RFCSTDTC         | Date/Time of First<br>Challenge Agent<br>Admin | Char | ISO 8601 datetime<br>or interval            | Record<br>Qualifier   | Used only when protocol specifies a challenge agent to induce a condition that the investigational treatment    | Perm |
|                  |                                                |      |                                             |                       | is intended to cure, mitigate, treat, or prevent. Equal to the earliest value of AGSTDTC for the challenge      |      |
|                  |                                                |      |                                             |                       | agent.                                                                                                          |      |
| RFCENDTC         | Date/Time of Last<br>Challenge Agent<br>Admin  | Char | ISO 8601 datetime<br>or interval            | Record<br>Qualifier   | Used only when protocol specifies a challenge agent to induce a condition that the investigational treatment    | Perm |
|                  |                                                |      |                                             |                       | is intended to cure, mitigate, treat, or prevent. Equal to the latest value of AGENDTC for the challenge agent  |      |
|                  |                                                |      |                                             |                       | (or the latest value of AGSTDTC if AGENDTC was not collected or is missing).                                    |      |
| RFICDTC          | Date/Time of<br>Informed Consent               | Char | ISO 8601 datetime<br>or interval            | Record<br>Qualifier   | Date/time of informed consent in ISO 8601 character format. This will be the same as the date of informed       | Exp  |
|                  |                                                |      |                                             |                       | consent in the Disposition domain, if that protocol milestone is documented. Would be null only in studies not  |      |
|                  |                                                |      |                                             |                       | collecting the date of informed consent.                                                                        |      |
| RFPENDTC         | Date/Time of End<br>of Participation           | Char | ISO 8601 datetime<br>or interval            | Record<br>Qualifier   | Date/time when subject ended participation or follow-up in a trial, as defined in the protocol, in ISO 8601     | Exp  |
|                  |                                                |      |                                             |                       | character format. Should correspond to the last known date of contact. Examples include completion date,        |      |
|                  |                                                |      |                                             |                       | withdrawal date, last follow-up, date recorded for lost to follow up, and death date.                           |      |
| DTHDTC           | Date/Time of<br>Death                          | Char | ISO 8601 datetime<br>or interval            | Record<br>Qualifier   | Date/time of death for any subject who died, in ISO 8601 format. Should represent the date/time that is         | Exp  |
|                  |                                                |      |                                             |                       | captured in the clinical-trial database.                                                                        |      |
| DTHFL            | Subject Death<br>Flag                          | Char | (NY)                                        | Record<br>Qualifier   | Indicates the subject died. Should be "Y" or null. Should be populated even when the death date is unknown.     | Exp  |
| SITEID           | Study Site<br>Identifier                       | Char | *                                           | Record<br>Qualifier   | Unique identifier for a site within a study.                                                                    | Req  |
| INVID            | Investigator<br>Identifier                     | Char |                                             | Record<br>Qualifier   | An identifier to describe the Investigator for the study. May be used in addition to SITEID. Not needed if      | Perm |
|                  |                                                |      |                                             |                       | SITEID is equivalent to INVID.                                                                                  |      |
| INVNAM           | Investigator Name                              | Char |                                             | Synonym<br>Qualifier  | Name of the investigator for a site.                                                                            | Perm |
| BRTHDTC          | Date/Time of Birth                             | Char | ISO 8601 datetime<br>or interval            | Record<br>Qualifier   | Date/time of birth of the subject.                                                                              | Perm |
| AGE              | Age                                            | Num  |                                             | Record<br>Qualifier   | Age expressed in AGEU. May be derived from RFSTDTC and BRTHDTC, but BRTHDTC may not be                          | Exp  |
|                  |                                                |      |                                             |                       | available in all cases (due to subject privacy concerns).                                                       |      |
| AGEU             | Age Units                                      | Char | (AGEU)                                      | Variable<br>Qualifier | Units associated with AGE.                                                                                      | Exp  |
| SEX              | Sex                                            | Char | (SEX)                                       | Record<br>Qualifier   | Sex of the subject.                                                                                             | Req  |
| RACE             | Race                                           | Char | (RACE)                                      | Record<br>Qualifier   | Race of the subject. Sponsors should refer to the FDA guidance2 regarding the collection of race. See           | Exp  |
|                  |                                                |      |                                             |                       | assumption below regarding RACE.                                                                                |      |
| ETHNIC           | Ethnicity                                      | Char | (ETHNIC)                                    | Record<br>Qualifier   | The ethnicity of the subject. Sponsors should refer to the FDA guidance1 regarding the collection of ethnicity. | Perm |
| ARMCD            | Planned Arm<br>Code                            | Char | *                                           | Record<br>Qualifier   | ARMCD is limited to 20 characters. It is not subject to the character restrictions that apply to TESTCD. The    | Exp  |
|                  |                                                |      |                                             |                       | maximum length of ARMCD is longer than for other "short" variables to accommodate the kind of values that       |      |
|                  |                                                |      |                                             |                       | are likely to be needed for crossover trials. For example, if ARMCD values for a 7-period crossover were        |      |
|                  |                                                |      |                                             |                       | constructed using 2-character abbreviations for each treatment and separating hyphens, the length of            |      |

## Page 64

1In this column, an asterisk (*) indicates that the variable may be subject to
controlled terminology. CDISC/NCI codelist values are enclosed in parentheses.
2Food and Drug Administration. Collection of Race and Ethnicity Data in Clinical
Trials. US Department of Health and Human Services;2016. Accessed January 8,
2020.
https://www.fda.gov/downloads/regulatoryinformation/guidances/ucm126396.pdf)

| Variable<br>Name | Variable Label                             | Type | Controlled Terms,<br>Codelist or<br>Format1 | Role                 | CDISC Notes                                                                                                     | Core |
| ---------------- | ------------------------------------------ | ---- | ------------------------------------------- | -------------------- | --------------------------------------------------------------------------------------------------------------- | ---- |
|                  |                                            |      |                                             |                      | ARMCD values would be 20. If the subject was not assigned to a trial arm, ARMCD is null and ARMNRS is           |      |
|                  |                                            |      |                                             |                      | populated.                                                                                                      |      |
|                  |                                            |      |                                             |                      | With the exception of studies which use multistage arm assignments, must be a value of ARMCD in the Trial       |      |
|                  |                                            |      |                                             |                      | Arms dataset.                                                                                                   |      |
| ARM              | Description of<br>Planned Arm              | Char | *                                           | Synonym<br>Qualifier | Name of the arm to which the subject was assigned. If the subject was not assigned to an arm, ARM is null       | Exp  |
|                  |                                            |      |                                             |                      | and ARMNRS is populated.                                                                                        |      |
|                  |                                            |      |                                             |                      | With the exception of studies which use multistage arm assignments, must be a value of ARM in the Trial         |      |
|                  |                                            |      |                                             |                      | Arms dataset.                                                                                                   |      |
| ACTARMCD         | Actual Arm Code                            | Char | *                                           | Record<br>Qualifier  | Code of actual arm. ACTARMCD is limited to 20 characters. It is not subject to the character restrictions that  | Exp  |
|                  |                                            |      |                                             |                      | apply to TESTCD. The maximum length of ACTARMCD is longer than for other short variables to                     |      |
|                  |                                            |      |                                             |                      | accommodate the kind of values that are likely to be needed for crossover trials.                               |      |
|                  |                                            |      |                                             |                      | With the exception of studies which use multistage arm assignments, must be a value of ARMCD in the Trial       |      |
|                  |                                            |      |                                             |                      | Arms dataset.                                                                                                   |      |
|                  |                                            |      |                                             |                      | If the subject was not assigned to an arm or followed a course not described by any planned arm,                |      |
|                  |                                            |      |                                             |                      | ACTARMCD is null and ARMNRS is populated.                                                                       |      |
| ACTARM           | Description of<br>Actual Arm               | Char | *                                           | Synonym<br>Qualifier | Description of actual arm.                                                                                      | Exp  |
|                  |                                            |      |                                             |                      | With the exception of studies which use multistage arm assignments, must be a value of ARM in the Trial         |      |
|                  |                                            |      |                                             |                      | Arms dataset.                                                                                                   |      |
|                  |                                            |      |                                             |                      | If the subject was not assigned to an arm or followed a course not described by any planned arm, ACTARM         |      |
|                  |                                            |      |                                             |                      | is null and ARMNRS is populated.                                                                                |      |
| ARMNRS           | Reason Arm<br>and/or Actual Arm<br>is Null | Char | (ARMNULRS)                                  | Record<br>Qualifier  | A coded reason that arm variables (ARM and ARMCD) and/or actual arm variables (ACTARM and                       | Exp  |
|                  |                                            |      |                                             |                      | ACTARMCD) are null. Examples: "SCREEN FAILURE", "NOT ASSIGNED", "ASSIGNED, NOT TREATED",                        |      |
|                  |                                            |      |                                             |                      | "UNPLANNED TREATMENT". It is assumed that if the arm and actual arm variables are null, the same                |      |
|                  |                                            |      |                                             |                      | reason applies to both arm and actual arm.                                                                      |      |
| ACTARMUD         | Description of<br>Unplanned Actual<br>Arm  | Char |                                             | Record<br>Qualifier  | A description of actual treatment for a subject who did not receive treatment described in a planned trial arm. | Exp  |
| COUNTRY          | Country                                    | Char |                                             | Record<br>Qualifier  | Country of the investigational site in which the subject participated in the trial.                             | Req  |
|                  |                                            |      |                                             |                      | Generally represented using ISO 3166-1 Alpha-3. Note that regulatory agency specific requirements (e.g.,        |      |
|                  |                                            |      |                                             |                      | US FDA) may require other terminologies; in such cases, follow regulatory requirements.                         |      |
| DMDTC            | Date/Time of<br>Collection                 | Char | ISO 8601 datetime<br>or interval            | Timing               | Date/time of demographic data collection.                                                                       | Perm |
| DMDY             | Study Day of<br>Collection                 | Num  |                                             | Timing               | Study day of collection measured as integer days.                                                               | Perm |

## Page 65

DM –Assumptions

1. Investigator and site identification: Companies use different methods to
   distinguish sites and investigators. CDISC assumes that SITEID will always be
   present, with INVID and INVNAM used as necessary. This should be done
   consistently and the meaning of the variable made clear in the Define-XML
   document.
2. Every subject in a study must have a subject identifier (SUBJID). In some
   cases a subject may participate in more than 1 study. To identify a subject
   uniquely across all studies for all applications or submissions involving the
   product, a unique identifier (USUBJID) must be included in all datasets.
   Subjects occasionally change sites during the course of a clinical trial.
   Sponsors must decide how to populate variables such as USUBJID, SUBJID and
   SITEID based on their operational and analysis needs, but only 1 DM record
   should be submitted for each subject. The Supplemental Qualifiers dataset may
   be used if appropriate to provide additional information.
3. Concerns for subject privacy suggest caution regarding the collection of
   variables like BRTHDTC. This variable is included in the Demographics model
   in the event that a sponsor intends to submit it; however, sponsors should
   follow regulatory guidelines and guidance as appropriate.
4. With the exception of trials that use multistage processes to assign subjects
   to arms described below, ARM and ACTARM must be populated with ARM values
   from the Trial Arms (TA) dataset and ARMCD and ACTARMCD must be populated
   with ARMCD values from the TA dataset or be null. The ARM and ARMCD values in
   the TA dataset have a one-to-one relationship, and that one-to-one
   relationship must be preserved in the values used to populate ARM and ARMCD
   in DM, and to populate the values of ACTARM and ACTARMCD in DM. a. Rules for
   the arm-related variables: i. If ARMCD is null, then ARM must be null and
   ARMNRS must be populated with the reason ARMCD is null. ii. If ACTARMCD is
   null, then ACTARM must be null and ARMNRS must be populated with the reason
   ACTARMCD is null. Both ARMCD and ACTARMCD will be null for subjects who were
   not assigned to treatment. The same reason will provide the reason that both
   are null. iii. ARMNRS may not be populated if both ARMCD and ACTARMCD are
   populated. ARMCD and ACTARMCD will be populated if the subject was assigned
   to an arm and received treatment consistent with 1 of the arms in the TA
   dataset. If ARMCD and ACTARMCD are not the same, that is sufficient to
   explain the situation; ARMNRS should not be populated. iv. If ARMNRS is
   populated with "UNPLANNED TREATMENT", ACTARMUD should be populated with a
   description of the unplanned treatment received. b. Multistage assignment to
   treatment: Some trials use a multistage process for assigning a subject to an
   arm (see Section 7.2.1, Trial Arms, Example Trial 3). In such a case, best
   practice is to create ARMCD values composed of codes representing the results
   of the multiple stages of the treatment assignment process. If a subject is
   partially assigned, then truncated codes representing the stages completed
   can be used in ARMCD, and similar truncated codes can be used in ACTARMCD.
   The descriptions used to populate ARM and ACTARM should be similarly
   truncated, and the one-to-one relationship between these truncated codes
   should be maintained for all affected subjects in the trial. Example 3 below
   provides an example of this situation; see also Section 5.3, Subject
   Elements, Example 2. Note that this use of values not in the TA dataset is
   allowable only for trials with multistage assignment to arms and to subjects
   in those trials who do not complete all stages of the assignment. c. Examples
   illustrating the arm-related variables i. Example 1 below shows how to handle
   a subject who was a screen failure and was never treated. ii. The Subject
   Elements (SE) dataset records the series of elements a subject passed through
   in the course of a trial, and these determine the value of ACTARMCD. The
   following examples include sample data for both datasets to illustrate this
   relationship.
5. Example 2 below shows how subjects who started the trial but were never
   assigned to an arm would be handled.

## Page 66

2. Section 5.3, Subject Elements, Example 1 illustrates a situation for a
   subject who received a treatment that was not the one to which they were
   assigned.
3. Section 5.3, Subject Elements, Example 2 illustrates a situation in which a
   subject received a set of treatments different from that for any of the
   planned arms.
4. Study population flags should not be included in SDTM data. The standard
   supplemental qualifiers included in previous versions of the SDTMIG (COMPLT,
   FULLSET, ITT, PPROT, SAFETY) should not be used. Note: The ADaM Subject-level
   Analysis Dataset (ADSL) specifies standard variable names for the most common
   populations and requires the inclusion of these flags when necessary for
   analysis; consult the ADaMIG for more information about these variables.
5. Submission of multiple race responses should be represented in the
   Demographics (DM) domain and Supplemental Qualifiers (SUPPDM) dataset as
   described in Section 4.2.8.3, Multiple Values for a Nonresult Qualifier
   Variable. If multiple races are collected, then the value of RACE should be
   “MULTIPLE” and the additional information will be included in the
   Supplemental Qualifiers dataset. Controlled terminology for RACE should be
   used in both DM and SUPPDM so that consistent values are available for
   summaries regardless of whether the data are found in a column or row. If
   multiple races were collected and 1 was designated as primary, RACE in DM
   should be the primary race and additional races should be reported in SUPPDM.
   When additional free-text information is reported about subject's race using
   “Other, Specify”, sponsors should refer to Section 4.2.7.1, "Specify" Values
   for Non-Result Qualifier Variables. If race was collected via an "Other,
   Specify" field and the sponsor chooses not to map the value as described in
   the current FDA guidance (see CDISC Notes for RACE in the domain
   specification), then the value of RACE should be “OTHER”. For subjects who
   refuse to provide or do not know their race information, the value of RACE
   could be “UNKNOWN”. See DM Example 4, DM Example 5, DM Example 6, and DM
   Example 7. a. The Racec-Ethnicc Codetable (available at
   https://www.cdisc.org/standards/terminology/controlledterminology) represents
   associations between collected race values and published race Controlled
   Terminology, as well as collected ethnicity values and published ethnicity
   Controlled Terminology.
6. RFSTDTC, RFENDTC, RFXSTDTC, RFXENDTC, RFCSTDTC, RFCENDTC, RFICDTC, RFPENDTC,
   DTHDTC, and BRTHDTC represent date/time values, but they are considered to
   have a record qualifier role in DM. They are not considered to be timing
   variables because they are not intended for use in the general observation
   classes.
7. Additional permissible identifier, qualifier, and timing variables: a. Only
   the following timing variables are permissible and may be added as
   appropriate: VISITNUM, VISIT, VISITDY. The record qualifier DMXFN (External
   File Name) is the only additional qualifier variable that may be added, which
   is adopted from the Findings general observation class, may also be used to
   refer to an external file, such as a patient narrative. b. The order of these
   additional variables within the domain should follow the rules as described
   in Section 4.1.4, Order of the Variables, and the order described in Section
   4.2, General Variable Assumptions.
8. As described in Section 4.1.4, Order of the Variables, RFSTDTC is used to
   calculate study day variables. RFSTDTC is usually defined as the date/time
   when a subject was first exposed to study drug. This definition applies for
   most interventional studies, when the start of treatment is the natural and
   preferred starting point for study day variables and thus the logical value
   for RFSTDTC. In such studies, when data are submitted for subjects who are
   ineligible for treatment (e.g., screen failures with ARMNRS = "SCREEN
   FAILURE"), subjects who were enrolled but not assigned to an arm (e.g.,
   ARMNRS = "NOT ASSIGNED"), or subjects who were randomized but not treated
   (e.g., ARMNRS = "NOT TREATED"), RFSTDTC will be null. For studies with
   designs that include a substantial portion of subjects who are not expected
   to be treated, a different protocol milestone may be chosen as the starting
   point for study day variables. Some examples include non-interventional or
   observational studies, studies with a no-treatment arm, and studies where
   there is a delay between randomization and treatment.
9. The DM domain contains several pairs of reference period variables: RFSTDTC
   and RFENDTC, RFXSTDTC and RFXENDTC, RFCSTDTC and RFCENDTC, and RFICDTC and
   RFPENDTC. There are 4 sets of reference variables to accommodate distinct
   reference-period definitions and there are instances

## Page 67

when the values of the variables may be exactly the same, particularly with
RFSTDTC-RFENDTC and RFXSTDTC-RFXENDTC. a. RFSTDTC and RFENDTC: This pair of
variables is sponsor-defined, but usually represents the date/time of first and
last study exposure. However, there are certain study designs where the start of
the reference period is defined differently, such as studies that have a washout
period before randomization or have a medical procedure required during
screening (e.g., biopsy). In these cases, RFSTDTC may be the enrollment date,
which is prior to first dose. Because study day values are calculated using
RFSTDTC, in this case study days would not be based on the date of first dose.
b. RFXSTDTC and RFXENDTC: This pair of variables defines a consistent reference
period for all interventional studies and is not open to customization. RFXSTDTC
and RFXENDTC always represent the date/time of first and last study exposure.
The study reference period often duplicates the reference period defined in
RFSTDTC and RFENDTC, but not always. Therefore, this pair of variables is
important as they guarantee that a reviewer will always be able to reference the
first and last study exposure reference period. RFXSTDTC should be the same as
SESTDTC for the first treatment element described in the SE dataset. RFXENDTC
may often be the same as the SEENDTC for the last treatment element described in
the SE dataset. c. RFCSTDTC and RFCENDTC: This pair of variables is used only
when the study uses a protocolspecified challenge agent to induce a condition
that the investigational treatment is intended to cure, mitigate, treat, or
prevent. RFCSTDTC and RFCENDTC always represent the date/time of first and last
exposure to the challenge agent. d. RFICDTC and RFPENDTC: The definitions of
this pair of variables are consistent in every study in which they are used:
They represent the entire period of a subject’s involvement in a study, from
providing informed consent through the last participation event or activity.
There may be times when this period coincides with other reference periods but
that is unusual. An example of when these periods might coincide with the study
reference period, RFSTDTC to RFENDTC, might be an observational trial where no
study intervention is administered. RFICDTC should correspond to the date of the
informed consent protocol milestone in Disposition (DS), if that protocol
milestone is documented in DS. In the event that there are multiple informed
consents, this will be the date of the first. RFPENDTC will be the last date of
participation for a subject for data included in a submission. This should be
the last date of any record for the subject in the database at the time it is
locked for submission. As such, it may not be the last date of participation in
the study if the submission includes interim data.

## Page 68

DM – Examples Example 1 dm.xpt

Example 2 This example Demographics dataset does not include all the DM required
and expected variables, only those that illustrate the variables that represent
arm information. The following example illustrates values of ARMCD for subjects
in Example Trial 1, described in Section 7.2.1, Trial Arms. This study included
2 elements, screen and run-in, before subjects were randomized to treatment. For
this study, the sponsor submitted data on all subjects, including screen-failure
subjects. Row 1: Subject 001 was randomized to arm "Drug A". As shown in the SE
dataset, this subject completed the "Drug A" element, so their actual arm was
also "Drug A". Row 2: Subject 002 was randomized to arm "Drug B". As shown in
the SE dataset, their actual arm was consistent with their randomization. Row 3:
Subject 003 was a screen failure, so they were not assigned to an arm or
treated. The arm actual arm variables are null, and ARMNRS="SCREEN FAILURE". Row
4: Subject 004 withdrew during the run-in element. Like subject 003, they were
not assigned to an arm or treated. However, they were not considered a screen
failure, and ARMNRS="NOT ASSIGNED". Row 5: Subject 005 was randomized but
dropped out before being treated. Thus, the actual arm variables are not
populated and ARMNRS="ASSIGNED, NOT TREATED". dm.xpt

| Row | STUDYID | DOMAIN | USUBJID    | SUBJID | RFSTDTC        | RFENDTC R        | FXSTDTC   | RFXENDTC   | RFICDTC        | RFPENDTC   | SITEID | INVNAM         | BRTHDTC        | AGE | AGEU  | SEX | RACE                                                   | ETHNIC                       | ARMCD | ARM     | ACTARMCD | ACTARM  | ARMNRS            | ACTARMUD | COUNTRY |
| --- | ------- | ------ | ---------- | ------ | -------------- | ---------------- | --------- | ---------- | -------------- | ---------- | ------ | -------------- | -------------- | --- | ----- | --- | ------------------------------------------------------ | ---------------------------- | ----- | ------- | -------- | ------- | ----------------- | -------- | ------- |
| 1   | ABC123  | DM A   | BC12301001 | 01001  | 2006-01-<br>12 | 2006-03- 2<br>10 | 006-01-12 | 2006-03-10 | 2006-01-<br>03 | 2006-04-01 | 01     | JOHNSON,<br>M  | 1948-12-<br>13 | 57  | YEARS | M   | WHITE                                                  | HISPANIC<br>OR LATINO        | A     | Drug A  | A        | Drug A  |                   |          | USA     |
| 2   | ABC123  | DM A   | BC12301002 | 01002  | 2006-01-<br>15 | 2006-02- 2<br>28 | 006-01-15 | 2006-02-28 | 2006-01-<br>04 | 2006-03-26 | 01     | JOHNSON,<br>M  | 1955-03-<br>22 | 50  | YEARS | M   | WHITE                                                  | NOT<br>HISPANIC<br>OR LATINO | P     | Placebo | P        | Placebo |                   |          | USA     |
| 3   | ABC123  | DM A   | BC12301003 | 01003  | 2006-01-<br>16 | 2006-03- 2<br>19 | 006-01-16 | 2006-03-19 | 2006-01-<br>02 | 2006-03-19 | 01     | JOHNSON,<br>M  | 1938-01-<br>19 | 68  | YEARS | F   | BLACK OR<br>AFRICAN<br>AMERICAN                        | NOT<br>HISPANIC<br>OR LATINO | P     | Placebo | P        | Placebo |                   |          | USA     |
| 4   | ABC123  | DM A   | BC12301004 | 01004  |                |                  |           |            | 2006-01-<br>07 | 2006-01-08 | 01     | JOHNSON,<br>M  | 1941-07-<br>02 |     |       | M   | ASIAN                                                  | NOT<br>HISPANIC<br>OR LATINO |       |         |          |         | SCREEN<br>FAILURE |          | USA     |
| 5   | ABC123  | DM A   | BC12302001 | 02001  | 2006-02-<br>02 | 2006-03- 2<br>31 | 006-02-02 | 2006-03-31 | 2006-01-<br>15 | 2006-04-12 | 02     | GONZALEZ,<br>E | 1950-06-<br>23 | 55  | YEARS | F   | AMERICAN<br>INDIAN OR<br>ALASKA<br>NATIVE              | NOT<br>HISPANIC<br>OR LATINO | P     | Placebo | P        | Placebo |                   |          | USA     |
| 6   | ABC123  | DM A   | BC12302002 | 02002  | 2006-02-<br>03 | 2006-04- 2<br>05 | 006-02-03 | 2006-04-05 | 2006-01-<br>10 | 2006-04-25 | 02     | GONZALEZ,<br>E | 1956-05-<br>05 | 49  | YEARS | F   | NATIVE<br>HAWAIIAN OR<br>OTHER<br>PACIFIC<br>ISLANDERS | NOT<br>HISPANIC<br>OR LATINO | A     | Drug A  | A        | Drug A  |                   |          | USA     |

| Row | STUDYID | DOMAIN | USUBJID | ARMCD | ARM    | ACTARMC | D ACTARM | ARMNRS                | ACTARMUD |
| --- | ------- | ------ | ------- | ----- | ------ | ------- | -------- | --------------------- | -------- |
| 1   | ABC     | DM     | 001     | A     | Drug A | A       | Drug A   |                       |          |
| 2   | ABC     | DM     | 002     | B     | Drug B | B       | Drug B   |                       |          |
| 3   | ABC     | DM     | 003     |       |        |         |          | SCREEN FAILURE        |          |
| 4   | ABC     | DM     | 004     |       |        |         |          | NOT ASSIGNED          |          |
| 5   | ABC     | DM     | 005     | A     | Drug A |         |          | ASSIGNED, NOT TREATED |          |

## Page 69

Rows 1-3: Subject 001 completed all the elements for arm A. Rows 4-6: Subject
002 completed all the elements for arm B. Row 7: Subject 003 was a screen
failure, who participated only in the "Screen" element. Rows 8-9: Subject 004
withdrew during the "Run-in" element, before they could be randomized. Rows
10-11: Subject 005 withdrew after they were randomized, but did not start
treatment. se.xpt

Example 3 Row 1: Subject 001 was randomized to drug A. At the end of the
double-blind treatment epoch, they were assigned to open label A; thus, their
ARMCD is "AA". They received the treatment to which they were assigned, so
ACTRMCD is also "AA". Row 2: Subject 002 was randomized to drug A. They were
lost to follow-up during the double-blind treatment epoch, so never reached the
open label epoch, when they would have been assigned to either the open drug A
or the rescue element. Their ARMCD is "A". This case illustrates the exception
to the rule that ARMCD, ARM, ACTARMCD, and ACTARM must be populated with values
from the TA dataset. Row 3: Subject "003" was randomized to drug A, but received
drug B. At the end of the double-blind treatment epoch, they were assigned to
rescue treatment. ARMCD shows the result of their assignments, "AR"; ACTARMCD
shows their actual treatment, "BR". dm.xpt

| Row | STUDYID | DOMAIN | USUBJID | SESEQ | ETCD | ELEMENT | SESTDTC    | SEENDTC    |
| --- | ------- | ------ | ------- | ----- | ---- | ------- | ---------- | ---------- |
| 1   | ABC     | SE     | 001     | 1     | SCRN | Screen  | 2006-06-01 | 2006-06-07 |
| 2   | ABC     | SE     | 001     | 2     | RI   | Run-In  | 2006-06-07 | 2006-06-21 |
| 3   | ABC     | SE     | 001     | 3     | A    | Drug A  | 2006-06-21 | 2006-07-05 |
| 4   | ABC     | SE     | 002     | 1     | SCRN | Screen  | 2006-05-03 | 2006-05-10 |
| 5   | ABC     | SE     | 002     | 2     | RI   | Run-In  | 2006-05-10 | 2006-05-24 |
| 6   | ABC     | SE     | 002     | 3     | B    | Drug B  | 2006-05-24 | 2006-06-07 |
| 7   | ABC     | SE     | 003     | 1     | SCRN | Screen  | 2006-06-27 | 2006-06-30 |
| 8   | ABC     | SE     | 004     | 1     | SCRN | Screen  | 2006-05-14 | 2006-05-21 |
| 9   | ABC     | SE     | 004     | 2     | RI   | Run-In  | 2006-05-21 | 2006-05-26 |
| 10  | ABC     | SE     | 005     | 1     | SCRN | Screen  | 2006-05-14 | 2006-05-21 |
| 11  | ABC     | SE     | 005     | 2     | RI   | Run-In  | 2006-05-21 | 2006-05-26 |

The following example illustrates values of ARMCD for subjects in Example Trial
3, described in Section 7.2.1, Trial Arms. Rows 1-3: Show that the subject
passed through all 3 elements for the AA arm. Rows 4-5: Show the 2 elements
("Screen" and "Treatment A") the subject passed through. Rows 6-8: Show that the
subject passed through the 3 elements associated with the "B-Rescue" arm. se.xpt

| Row | STUDYID | DOMAIN U | SUBJID | ARMCD | ARM     | ACTARMCD | ACTARM | ARMNRS | ACTARMUD |
| --- | ------- | -------- | ------ | ----- | ------- | -------- | ------ | ------ | -------- |
| 1   | DEF     | DM 0     | 01 A   | A     | A-OPEN  | A AA     | A-OPEN | A      |          |
| 2   | DEF     | DM 0     | 02 A   |       | A       | A        | A      |        |          |
| 3   | DEF     | DM 0     | 03 A   | R     | A-RESCU | E BR     | B-RESC | UE     |          |

| Row | STUDYID | DOMAIN | USUBJID S | ESEQ E | TCD | ELEMENT S     | ESTDTC    | SEENDTC    |
| --- | ------- | ------ | --------- | ------ | --- | ------------- | --------- | ---------- |
| 1   | DEF     | SE     | 001 1     | S      | CRN | Screen 2      | 006-01-07 | 2006-01-12 |
| 2   | DEF     | SE     | 001 2     | D      | BA  | Treatment A 2 | 006-01-12 | 2006-04-10 |
| 3   | DEF     | SE     | 001 3     | O      | A   | Open Drug A 2 | 006-04-10 | 2006-07-05 |
| 4   | DEF     | SE     | 002 1     | S      | CRN | Screen 2      | 006-02-03 | 2006-02-10 |
| 5   | DEF     | SE     | 002 2     | D      | BA  | Treatment A 2 | 006-02-10 | 2006-03-24 |
| 6   | DEF     | SE     | 003 1     | S      | CRN | Screen 2      | 006-02-22 | 2006-03-01 |
| 7   | DEF     | SE     | 003 2     | D      | BB  | Treatment B 2 | 006-03-01 | 2006-06-27 |
| 8   | DEF     | SE     | 003 3     | R      | SC  | Rescue 2      | 006-06-27 | 2006-09-24 |

## Page 70

Example 4 The CRF in this example is annotated to show the CDASH variable name
and the target SDTMIG variable. Data that are collected using the same variable
name as defined in the SDTMIG are in RED . If the CDASHIG variable differs from
the one defined in the SDTMIG, the CDASHIG variable is in GREY . See the CDASH
Model and Implementation Guide for additional information:
https://www.cdisc.org/standards/foundational/cdash. This example shows multiple
race categories and subcategories. Only a subset of options is shown for this
instrument due to space constraints. Demographics Sample aCRF for Race with
Additional Granularity If the study participant answered: AMERICAN INDIAN OR
ALASKA NATIVE

## Page 71

If the study participant answered: ASIAN If the study participant answered:
BLACK OR AFRICAN AMERICAN If the study participant answered: WHITE

## Page 72

CRF Metadata

| CDASH<br>Variable                                               | Order | Question Text                                                                                                    | Prompt | CRF<br>Completion<br>Instructions                                                                           | Type | SDTMIG Target<br>Variable | SDTM Target<br>Mapping                                                               | Controlled<br>Terminology<br>Code List<br>Name | Permissible Values                                                                  | Pre-<br>specified<br>Value | Query<br>Display | List<br>Style | Hidden |
| --------------------------------------------------------------- | ----- | ---------------------------------------------------------------------------------------------------------------- | ------ | ----------------------------------------------------------------------------------------------------------- | ---- | ------------------------- | ------------------------------------------------------------------------------------ | ---------------------------------------------- | ----------------------------------------------------------------------------------- | -------------------------- | ---------------- | ------------- | ------ |
| RACE01                                                          | 1     | Which of the following<br>racial designations<br>best describes you?<br>(More than one<br>choice is acceptable.) | Race   | Study<br>participants<br>should self-<br>report race, with<br>race being<br>asked about<br>after ethnicity. | Text | RACE                      |                                                                                      | (RACE)                                         | AMERICAN INDIAN OR<br>ALASKA NATIVE                                                 |                            |                  | checkbox      |        |
| RACE02                                                          | 2     | Which of the following<br>racial designations<br>best describes you?<br>(More than one<br>choice is acceptable.) | Race   | Study<br>participants<br>should self-<br>report race, with<br>race being<br>asked about<br>after ethnicity. | Text | RACE                      |                                                                                      | (RACE)                                         | ASIAN                                                                               |                            |                  | checkbox      |        |
| RACE03                                                          | 3     | Which of the following<br>racial designations<br>best describes you?<br>(More than one<br>choice is acceptable.) | Race   | Study<br>participants<br>should self-<br>report race, with<br>race being<br>asked about<br>after ethnicity. | Text | RACE                      |                                                                                      | (RACE)                                         | BLACK OR AFRICAN<br>AMERICAN                                                        |                            |                  | checkbox      |        |
| RACE04                                                          | 4     | Which of the following<br>racial designations<br>best describes you?<br>(More than one<br>choice is acceptable.) | Race   | Study<br>participants<br>should self-<br>report race, with<br>race being<br>asked about<br>after ethnicity. | Text | RACE                      |                                                                                      | (RACE)                                         | NATIVE HAWAIIAN OR<br>OTHER PACIFIC ISLANDER                                        |                            |                  | checkbox      |        |
| RACE05                                                          | 5     | Which of the following<br>racial designations<br>best describes you?<br>(More than one<br>choice is acceptable.) | Race   | Study<br>participants<br>should self-<br>report race, with<br>race being<br>asked about<br>after ethnicity. | Text | RACE                      |                                                                                      | (RACE)                                         | WHITE                                                                               |                            |                  | checkbox      |        |
| RACE06                                                          | 6     | Which of the following<br>racial designations<br>best describes you?<br>(More than one<br>choice is acceptable.) | Race   | Study<br>participants<br>should self-<br>report race, with<br>race being<br>asked about<br>after ethnicity. | Text | RACE                      |                                                                                      | (RACE)                                         | NOT REPORTED                                                                        |                            |                  | checkbox      |        |
| RACE07                                                          | 7     | Which of the following<br>racial designations<br>best describes you?<br>(More than one<br>choice is acceptable.) | Race   | Study<br>participants<br>should self-<br>report race, with<br>race being<br>asked about<br>after ethnicity. | Text | RACE                      |                                                                                      | (RACE)                                         | UNKNOWN                                                                             |                            |                  | checkbox      |        |
| If study participant answered: AMERICAN INDIAN OR ALASKA NATIVE |       |                                                                                                                  |        |                                                                                                             |      |                           |                                                                                      |                                                |                                                                                     |                            |                  |               |        |
| CRACE01-<br>CRACE04                                             | 10    | Which of the following<br>racial designations<br>best describes you?                                             | Race   | Select each<br>value that<br>applies if the<br>subject<br>answered                                          | Text | SUPPDM.QVAL               | For each value that<br>applies,<br>SUPPDM.QVAL where<br>SUPPDM.QNAM<br>="CRACEn" and | (RACEC)                                        | ALASKA NATIVE; AMERICAN<br>INDIAN; CARIBBEAN INDIAN;<br>CENTRAL AMERICAN<br>INDIAN; |                            |                  | checkbox      |        |

## Page 73

The value of RACE is used to represent the high-level racial designation as a
single collected value per CDISC Controlled Terminology in dm.xpt. When more
than 1 choice is selected, the value is represented with "MULTIPLE" as shown in
this example. Note: Only those variables relevant to this example are shown. Row
1: Shows that USUBJID ABC789-010-045 designated 1 race, "WHITE", as the value
that best describes their race. Row 2: Shows that USUBJID ABC789-010-046
designated 1 race, "ASIAN", as the value that best describes their race. Row 3:
Shows that USUBJID ABC789-010-047 designated multiple races as the values that
best describe their race. "MULTIPLE" is assigned in RACE. dm.xpt

| CDASH<br>Variable                                        | Order | Question Text                                                        | Prompt | CRF<br>Completion<br>Instructions                                                                                                    | Type | SDTMIG Target<br>Variable | SDTM Target<br>Mapping                                                                                                                                           | Controlled<br>Terminology<br>Code List<br>Name | Permissible Values                                                                                            | Pre-<br>specified<br>Value | Query<br>Display | List<br>Style | Hidden              |
| -------------------------------------------------------- | ----- | -------------------------------------------------------------------- | ------ | ------------------------------------------------------------------------------------------------------------------------------------ | ---- | ------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------- | ------------------------------------------------------------------------------------------------------------- | -------------------------- | ---------------- | ------------- | ------------------- |
|                                                          |       |                                                                      |        | "AMERICAN<br>INDIAN OR<br>ALASKA<br>NATIVE". Check<br>all that apply.                                                                |      |                           | SUPPDM.QLABEL =<br>"Collected Race<br>n" where n is the<br>choice value.                                                                                         |                                                |                                                                                                               |                            |                  |               |                     |
| If study participant answered: ASIAN                     |       |                                                                      |        |                                                                                                                                      |      |                           |                                                                                                                                                                  |                                                |                                                                                                               |                            |                  |               |                     |
| CRACE05-<br>CRACE10                                      | 11    | Which of the following<br>racial designations<br>best describes you? | Race   | Select each<br>value that<br>applies if the<br>subject<br>answered<br>"ASIAN". Check<br>all that apply.                              | Text | SUPPDM.QVAL               | For each value that<br>applies,<br>SUPPDM.QVAL where<br>SUPPDM.QNAM<br>="CRACEn" and<br>SUPPDM.QLABEL =<br>"Collected Race n"<br>where n is the choice<br>value. | (RACEC)                                        | ASIAN AMERICAN; ASIAN<br>INDIAN;<br>BANGLADESHI; CHINESE;<br>JAPANESE; KOREAN;                                |                            |                  | checkbox      | CRACE05-<br>CRACE10 |
| If study participant answered: BLACK OR AFRICAN AMERICAN |       |                                                                      |        |                                                                                                                                      |      |                           |                                                                                                                                                                  |                                                |                                                                                                               |                            |                  |               |                     |
| CRACE11-<br>CRACE17                                      | 12    | Which of the following<br>racial designations<br>best describes you? | Race   | Select each<br>value that<br>applies if the<br>subject<br>answered<br>"BLACK OR<br>AFRICAN<br>AMERICAN".<br>Check all that<br>apply. | Text | SUPPDM.QVAL               | For each value that<br>applies,<br>SUPPDM.QVAL where<br>SUPPDM.QNAM<br>="CRACEn" and<br>SUPPDM.QLABEL =<br>"Collected Race n"<br>where n is the choice<br>value. | (RACEC)                                        | AFRICAN; AFRICAN<br>AMERICAN; AFRICAN<br>CARIBBEAN; BAHAMIAN;<br>BARBADIAN; BLACK; BLACK<br>CENTRAL AMERICAN; |                            |                  | checkbox      |                     |
| If study participant answered: WHITE                     |       |                                                                      |        |                                                                                                                                      |      |                           |                                                                                                                                                                  |                                                |                                                                                                               |                            |                  |               |                     |
| CRACE18-<br>CRACE21                                      | 13    | Which of the following<br>racial designations<br>best describes you? | Race   | Select each<br>value that<br>applies if the<br>subject<br>answered<br>"WHITE". Check<br>all that apply.                              | Text | SUPPDM.QVAL               | For each value that<br>applies,<br>SUPPDM.QVAL where<br>SUPPDM.QNAM<br>="CRACEn" and<br>SUPPDM.QLABEL =<br>"Collected Race n"<br>where n is the choice<br>value. | (RACEC)                                        | ARAB; EUROPEAN; MIDDLE<br>EASTERN; RUSSIAN;                                                                   |                            |                  | checkbox      |                     |

| Row | STUDYID | DOMAIN | USUBJID        | SUBJID  | RACE     |
| --- | ------- | ------ | -------------- | ------- | -------- |
| 1   | ABC789  | DM     | ABC789-010-045 | 010-045 | WHITE    |
| 2   | ABC789  | DM     | ABC789-010-046 | 010-046 | ASIAN    |
| 3   | ABC789  | DM     | ABC789-010-047 | 010-047 | MULTIPLE |

## Page 74

When a subject selects multiple race values, as USUBJID ABC789-010-047 did, the
values selected are represented in SUPPDM. Collected race, which is the specific
race subcategory (or subcategories) selected by each subject, is represented in
SUPPDM to ensure subject self-identification and/or country-specific
requirements are available for reference. CDASH recommended QNAM-QLABEL values
have been provided.suppdm.xpt Rows 1, 2: Show that USUBJID ABC789-010-047
selected 2 RACE values, "ASIAN" and "WHITE". CDASH recommended QNAM-QLABEL
values have been provided. Rows 3-5: Show that USUBJID ABC789-010-047 selected 3
collected race (CRACE) values, "CHINESE", "KOREAN", and "RUSSIAN". CDASH
recommended QNAM-QLABEL values have been provided. suppdm.xpt

Example 5 This example shows different Chinese regional ethnicity
subcategorizations (majority and minority). CRF Mock Example In this CRF
example, subcategorizations of ethnicity are made available. RACE is identified
as "ASIAN" and ETHNIC as "NOT HISPANIC OR LATINO". dm.xpt

| Row | STUDYID | RDOMAIN | USUBJID        | IDVAR | IDVARVAL | QNAM    | QLABEL            | QVAL    | QORIG | QEVAL |
| --- | ------- | ------- | -------------- | ----- | -------- | ------- | ----------------- | ------- | ----- | ----- |
| 1   | ABC789  | DM      | ABC789-010-047 |       |          | RACE2   | Race 2            | ASIAN   | CRF   |       |
| 2   | ABC789  | DM      | ABC789-010-047 |       |          | RACE5   | Race 5            | WHITE   | CRF   |       |
| 3   | ABC789  | DM      | ABC789-010-047 |       |          | CRACE8  | Collected Race 8  | CHINESE | CRF   |       |
| 4   | ABC789  | DM      | ABC789-010-047 |       |          | CRACE10 | Collected Race 10 | KOREAN  | CRF   |       |
| 5   | ABC789  | DM      | ABC789-010-047 |       |          | CRACE21 | Collected Race 21 | RUSSIAN | CRF   |       |

Row 1: Ethnicity subcategorization of subject self-identification being "HAN
CHINESE". CDASH recommended QNAM-QLABEL values have been provided. Rows 2-3:
Ethnicity subcategorization of subject self-identification being "MIAO" and
"ZHUANG". CDASH recommended QNAM-QLABEL values have been provided. suppdm.xpt

| Row | STUDYID | DOMAIN | USUBJID        | SUBJID  | AGE A | GEU  | SEX | RACE  | ETHNIC                 |
| --- | ------- | ------ | -------------- | ------- | ----- | ---- | --- | ----- | ---------------------- |
| 1   | ABC789  | DM     | ABC789-010-045 | 010-045 | 20 Y  | EARS | M   | ASIAN | NOT HISPANIC OR LATINO |
| 2   | ABC789  | DM     | ABC789-010-047 | 010-047 | 24 Y  | EARS | F   | ASIAN | NOT HISPANIC OR LATINO |

Example 6 The CRF in this example is annotated to show the CDASH variable name
and the target SDTMIG variable. Data that are collected using the same variable
name as defined in the SDTMIG are in RED . If the CDASHIG variable differs from
the one defined in the SDTMIG, the CDASHIG variable is in GREY . See the CDASH
Model and Implementation Guide for additional information:
https://www.cdisc.org/standards/foundational/cdash.

| Row | STUDYID | RDOMAIN | USUBJID I      | DVAR ID | VARVAL | QNAM    | QLABEL                | QVAL        | QORIG | QEVAL |
| --- | ------- | ------- | -------------- | ------- | ------ | ------- | --------------------- | ----------- | ----- | ----- |
| 1   | ABC789  | DM      | ABC789-010-045 |         |        | ETHNIC1 | Collected Ethnicity 1 | HAN CHINESE | CRF   |       |
| 2   | ABC789  | DM      | ABC789-010-047 |         |        | ETHNIC1 | Collected Ethnicity 1 | MIAO        | CRF   |       |
| 3   | ABC789  | DM      | ABC789-010-047 |         |        | ETHNIC2 | Collected Ethnicity 2 | ZHUANG      | CRF   |       |

## Page 75

This example shows race categories and subcategories. Only a subset of options
are shown for this instrument due to space constraints. For a complete aCRF
example see the CDASHIG v2.1, Section 7.3. Demographics Sample aCRF for Race
with Additional Granularity If the study participant answered: ASIAN If the
study participant answered: BLACK OR AFRICAN AMERICAN

## Page 76

CRF Metadata

| CDASH<br>Variable | Order | Question Text                                                                                                          | Prompt | CRF<br>Completion<br>Instructions                                                                              | Type | SDTMIG Target<br>Variable | SDTM Target<br>Mapping | Controlled<br>Terminology<br>Code List<br>Name | Permissible Values                           | Pre-<br>specified<br>Value | Query<br>Display | List<br>Style | Hidden |
| ----------------- | ----- | ---------------------------------------------------------------------------------------------------------------------- | ------ | -------------------------------------------------------------------------------------------------------------- | ---- | ------------------------- | ---------------------- | ---------------------------------------------- | -------------------------------------------- | -------------------------- | ---------------- | ------------- | ------ |
| RACE01            | 3     | Which of the<br>following racial<br>designations best<br>describes you?<br>(More than one<br>choice is<br>acceptable.) | Race   | Study<br>participants<br>should self-<br>report race,<br>with race<br>being asked<br>about after<br>ethnicity. | Text | RACE                      |                        | (RACE)                                         | AMERICAN INDIAN OR<br>ALASKA NATIVE          |                            |                  | checkbox      |        |
| RACE02            | 4     | Which of the<br>following<br>racial designations<br>best describes you?<br>(More than one<br>choice is<br>acceptable.) | Race   | Study<br>participants<br>should self-<br>report race,<br>with race<br>being asked<br>about after<br>ethnicity. | Text | RACE                      |                        | (RACE)                                         | ASIAN                                        |                            |                  | checkbox      |        |
| RACE03            | 5     | Which of the<br>following racial<br>designations best<br>describes you?<br>(More than one<br>choice is<br>acceptable.) | Race   | Study<br>participants<br>should self-<br>report race,<br>with race<br>being asked<br>about after<br>ethnicity. | Text | RACE                      |                        | (RACE)                                         | BLACK OR AFRICAN<br>AMERICAN                 |                            |                  | checkbox      |        |
| RACE04            | 6     | Which of the<br>following racial<br>designations best<br>describes you?<br>(More than one<br>choice is<br>acceptable.) | Race   | Study<br>participants<br>should self-<br>report race,<br>with race<br>being asked<br>about after<br>ethnicity. | Text | RACE                      |                        | (RACE)                                         | NATIVE HAWAIIAN OR<br>OTHER PACIFIC ISLANDER |                            |                  | checkbox      |        |
| RACE05            | 7     | Which of the<br>following racial<br>designations best<br>describes you?<br>(More than one<br>choice is<br>acceptable.) | Race   | Study<br>participants<br>should self-<br>report race,<br>with race<br>being asked<br>about after<br>ethnicity. | Text | RACE                      |                        | (RACE)                                         | WHITE                                        |                            |                  | checkbox      |        |
| RACE06            | 8     | Which of the<br>following racial<br>designations best<br>describes you?<br>(More than one<br>choice is<br>acceptable.) | Race   | Study<br>participants<br>should self-<br>report race,<br>with race<br>being asked<br>about after<br>ethnicity. | Text | RACE                      |                        | (RACE)                                         | NOT REPORTED                                 |                            |                  | checkbox      |        |
| RACE07            | 9     | Which of the<br>following racial<br>designations best                                                                  | Race   | Study<br>participants<br>should self-                                                                          | Text | RACE                      |                        | (RACE)                                         | UNKNOWN                                      |                            |                  | checkbox      |        |

## Page 77

The value of RACE is used to represent the high-level racial designation as a
single collected value per CDISC Controlled Terminology in dm.xpt. In this
example, subjects chose to select 1 high-level racial designation. Note: Only
those variables relevant to this example are shown. Row 1: Shows that USUBJID
ABC789-010-001 designated 1 race, "ASIAN", as the value that best describes
their race. Row 2: Shows that USUBJID ABC789-010-002 designated 1 race, "BLACK
OR AFRICAN AMERICAN", as the value that best describes their race. Row 3: Shows
that USUBJID ABC789-010-003 designated 1 race, "BLACK OR AFRICAN AMERICAN", as
the value that best describes their race. dm.xpt

| CDASH<br>Variable                                            | Order | Question Text                                                           | Prompt | CRF<br>Completion<br>Instructions                                                                                                    | Type | SDTMIG Target<br>Variable | SDTM Target<br>Mapping                                                                                                                                              | Controlled<br>Terminology<br>Code List<br>Name | Permissible Values                                                                                               | Pre-<br>specified<br>Value | Query<br>Display | List<br>Style | Hidden |
| ------------------------------------------------------------ | ----- | ----------------------------------------------------------------------- | ------ | ------------------------------------------------------------------------------------------------------------------------------------ | ---- | ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- | -------------------------- | ---------------- | ------------- | ------ |
|                                                              |       | describes you?<br>(More than one<br>choice is<br>acceptable.)           |        | report race,<br>with race<br>being asked<br>about after<br>ethnicity.                                                                |      |                           |                                                                                                                                                                     |                                                |                                                                                                                  |                            |                  |               |        |
| If the study participant answered: ASIAN                     |       |                                                                         |        |                                                                                                                                      |      |                           |                                                                                                                                                                     |                                                |                                                                                                                  |                            |                  |               |        |
| CRACE05-<br>CRACE10                                          | 11    | Which of the<br>following racial<br>designations best<br>describes you? | Race   | Select each<br>value that<br>applies if the<br>subject<br>answered<br>"ASIAN".<br>Check all that<br>apply.                           | Text | SUPPDM.QVAL               | For each value that<br>applies,<br>SUPPDM.QVAL<br>where<br>SUPPDM.QNAM<br>="CRACEn" and<br>SUPPDM.QLABEL<br>= "Collected Race<br>n" where n is the<br>choice value. | (RACEC)                                        | ASIAN AMERICAN; ASIAN<br>INDIAN;<br>BANGLADESHI; CHINESE;<br>JAPANESE; KOREAN;                                   |                            |                  | checkbox      |        |
| If the study participant answered: BLACK OR AFRICAN AMERICAN |       |                                                                         |        |                                                                                                                                      |      |                           |                                                                                                                                                                     |                                                |                                                                                                                  |                            |                  |               |        |
| CRACE11-<br>CRACE17                                          | 12    | Which of the<br>following racial<br>designations best<br>describes you? | Race   | Select each<br>value that<br>applies if the<br>subject<br>answered<br>"BLACK OR<br>AFRICAN<br>AMERICAN".<br>Check all that<br>apply. | Text | SUPPDM.QVAL               | For each value that<br>applies,<br>SUPPDM.QVAL<br>where<br>SUPPDM.QNAM<br>="CRACEn" and<br>SUPPDM.QLABEL<br>= "Collected Race<br>n" where n is the<br>choice value. | (RACEC)                                        | AFRICAN; AFRICAN<br>AMERICAN; AFRICAN<br>CARIBBEAN; BAHAMIAN;<br>BARBADIAN; BLACK;<br>BLACK CENTRAL<br>AMERICAN; |                            |                  | checkbox      |        |

Collected race, which is the specific race subcategory for each subject, is
represented in SUPPDM to ensure subject self-identification and/or
country-specific requirements are available for reference. In this example, each
subject selected 1 race and 1 race subcategory. CDASH recommended QNAM-QLABEL
values have been provided.

| Row | STUDYID | DOMAIN | USUBJID        | SUBJID  | RACE                      |
| --- | ------- | ------ | -------------- | ------- | ------------------------- |
| 1   | ABC789  | DM     | ABC789-010-001 | 010-001 | ASIAN                     |
| 2   | ABC789  | DM     | ABC789-010-002 | 010-002 | BLACK OR AFRICAN AMERICAN |
| 3   | ABC789  | DM     | ABC789-010-003 | 010-003 | BLACK OR AFRICAN AMERICAN |

## Page 78

Row 1: Shows USUBJID ABC789-010-001 selected "JAPANESE" as the specific ASIAN
race collected. Row 2: Shows USUBJID ABC789-010-002 selected "AFRICAN AMERICAN"
as the specific BLACK OR AFRICAN AMERICAN race collected. Row 3: Shows USUBJID
ABC789-010-003 selected "BLACK" as the specific BLACK OR AFRICAN AMERICAN race
collected. suppdm.xpt

Example 7 CRF Mock Example Rows 1-2: Subjects self-identify to 1 of the first 5
race options on the CRF form. Row 3: Subject did not self-identify to 1 of the
existing race options and selected "Other". RACE was populated with "OTHER" in
this case. Row 4: Subject could not self-identify to any of the race options
including identification of an "Other". RACE was populated with "UNKNOWN" in
this case. Note: Not all DM variables are shown. dm.xpt

| Row | STUDYID | RDOMAIN | USUBJID       | IDVAR | IDVARVAL Q | NAM Q   | LABEL           | QVAL             | QORIG | QEVAL |
| --- | ------- | ------- | ------------- | ----- | ---------- | ------- | --------------- | ---------------- | ----- | ----- |
| 1   | ABC789  | DM A    | BC789-010-001 |       | C          | RACE3 C | ollected Race 3 | JAPANESE         | CRF   |       |
| 2   | ABC789  | DM A    | BC789-010-002 |       | C          | RACE5 C | ollected Race 5 | AFRICAN AMERICAN | CRF   |       |
| 3   | ABC789  | DM A    | BC789-010-003 |       | C          | RACE8 C | ollected Race 8 | BLACK            | CRF   |       |

| Row | STUDYID | DOMAIN | USUBJID        | SUBJID  | AGE | AGEU  | SEX | RACE    | ETHNIC                 |
| --- | ------- | ------ | -------------- | ------- | --- | ----- | --- | ------- | ---------------------- |
| 1   | ABC789  | DM     | ABC789-010-045 | 010-045 | 20  | YEARS | M   | WHITE   | HISPANIC OR LATINO     |
| 2   | ABC789  | DM     | ABC789-010-046 | 010-046 | 21  | YEARS | F   | ASIAN   | NOT HISPANIC OR LATINO |
| 3   | ABC789  | DM     | ABC789-010-047 | 010-047 | 24  | YEARS | F   | OTHER   | HISPANIC OR LATINO     |
| 3   | ABC789  | DM     | ABC789-010-048 | 010-048 | 33  | YEARS | M   | UNKNOWN | HISPANIC OR LATINO     |

## Page 79

Row 1: Sponsor allowed for an "Other" option to be collected, where its specify
details are in SUPPDM. Row 2: Sponsor allowed for an "Unknown" option to be
collected, where its reason is collected in SUPPDM. Note: Recommended
QNAM-QLABEL values have been provided. suppdm.xpt

5.3 Subject Elements (SE) SE – Description/Overview A special-purpose domain
that contains the actual order of elements followed by the subject, together
with the start date/time and end date/time for each element. The Subject
Elements dataset consolidates information about the timing of each subject’s
progress through the epochs and elements of the trial. For elements that involve
study treatments, the identification of which element the subject passed through
(e.g., drug X vs. placebo) is likely to derive from data in the Exposure domain
or another Interventions domain. The dates of a subject’s transition from one
element to the next will be taken from the Interventions domain(s) and from
other relevant domains, according to the definitions (TESTRL values) in the
Trial Elements (TE) dataset (see Section 7.2.2, Trial Elements). The SE dataset
is particularly useful for studies with multiple treatment periods, such as
crossover studies. The SE dataset contains the date/times at which a subject
moved from one element to another, so when this dataset, the Trial Arms (TA; see
Section 7.2.1, Trial Arms) dataset, and the Trial Elements (TE; see Section
7.2.2, Trial Elements) dataset are included in a submission, reviewers can
relate all observations made about a subject to that subject’s progression
through the trial. • Comparison of the --DTC of a finding observation to the
element transition dates (values of SESTDTC and SEENDTC) identifies which
element the subject was in at the time of the finding. Similarly, one can
determine the element during which an event or intervention started or ended. •
“Day within Element” or “Day within Epoch” can be derived. Such variables relate
an observation to the start of an element or epoch in the same way that study
day (--DY) variables relate it to the reference start date (RFSTDTC) for the
study as a whole. See Section 4.4.4, Use of the "Study Day" Variables. • Having
knowledge of SE start and end dates can be helpful in the determination of
baseline values. SE – Specification se.xpt, Subject Elements — Special Purpose.
One record per actual Element per subject, Tabulation.

| Row | STUDYID | RDOMAIN | USUBJID        | IDVAR | IDVARVAL | QNAM     | QLABEL               | QVAL                          | QORIG | QEVAL |
| --- | ------- | ------- | -------------- | ----- | -------- | -------- | -------------------- | ----------------------------- | ----- | ----- |
| 1   | ABC789  | DM      | ABC789-010-047 |       |          | RACEOTH  | Race, Other          | BRAZILIAN                     | CRF   |       |
| 2   | ABC789  | DM      | ABC789-010-048 |       |          | RACEREAS | Race, Reason Details | REFUGEE - DO NOT KNOW MY RACE | CRF   |       |

| Variable<br>Name | Variable Label               | Type | Controlled Terms,<br>Codelist or Format1 | Role       | CDISC Notes                                                                                           | Core |
| ---------------- | ---------------------------- | ---- | ---------------------------------------- | ---------- | ----------------------------------------------------------------------------------------------------- | ---- |
| STUDYID          | Study Identifier             | Char |                                          | Identifier | Unique identifier for a study.                                                                        | Req  |
| DOMAIN           | Domain Abbreviation          | Char | SE                                       | Identifier | Two-character abbreviation for the domain.                                                            | Req  |
| USUBJID          | Unique Subject<br>Identifier | Char |                                          | Identifier | Identifier used to uniquely identify a subject across all studies for all applications or submissions | Req  |
|                  |                              |      |                                          |            | involving the product.                                                                                |      |
| SESEQ            | Sequence Number              | Num  |                                          | Identifier | Sequence number given to ensure uniqueness of subject records within a domain. Should be assigned     | Req  |
|                  |                              |      |                                          |            | to be consistent chronological order.                                                                 |      |

## Page 80

1In this column, an asterisk (*) indicates that the variable may be subject to
controlled terminology. CDISC/NCI codelist values are enclosed in parentheses.
SE – Assumptions Submission of the SE dataset is strongly recommended, as it
provides information needed by reviewers to place observations in context within
the study. As noted in the SE - Description/Overview, the TE and TA datasets
should also be submitted, as these define the design and the terms referenced by
the SE dataset. The SE domain allows the submission of data on the timing of the
trial elements a subject actually passed through in their participation in the
trial. Section 7.2.2, Trial Elements, and Section 7.2.1, Trial Arms, provide
additional information on these datasets, which define a trial's planned
elements and describe the planned sequences of elements for the arms of the
trial.

1. For any particular subject, the dates in the SE table are the dates when the
   transition events identified in the TE table occurred. Judgment may be needed
   to match actual events in a subject's experience with the definitions of
   transition events (i.e., events that mark the start of new elements) in the
   TE table; actual events may vary from the plan. For instance, in a
   single-dose pharmacokinetics (PK) study, the transition events might
   correspond to study drug doses of 5 and 10 mg. If a subject actually received
   a dose of 7 mg when they were scheduled to receive 5 mg, a decision will have
   to be made on how to represent this in the SE domain.
2. If the date/time of a transition element was not collected directly, the
   method used to infer the element start date/time should be explained in the
   Comments column of the Define-XML document.
3. Judgment will also have to be used in deciding how to represent a subject's
   experience if an element does not proceed or end as planned. For instance,
   the plan might identify a trial element that is to start with the first of a
   series of 5 daily doses and end after 1 week, when the subject transitions to
   the next treatment element. If the subject actually started the next
   treatment epoch (see Section 7.1, Introduction to Trial Design Model
   Datasets, and Section 7.1.2, Definitions of Trial Design Concepts) after 4
   weeks, the sponsor would have to decide whether to represent this as an
   abnormally long element, or as a normal element plus an unplanned
   non-treatment element.

| Variable<br>Name | Variable Label                         | Type | Controlled Terms,<br>Codelist or Format1 | Role                 | CDISC Notes                                                                                            | Core |
| ---------------- | -------------------------------------- | ---- | ---------------------------------------- | -------------------- | ------------------------------------------------------------------------------------------------------ | ---- |
| ETCD             | Element Code                           | Char | *                                        | Topic                | 1. ETCD (the companion to ELEMENT) is limited to 8 characters and does not have special                | Req  |
|                  |                                        |      |                                          |                      | character restrictions. These values should be short for ease of use in programming, but it is         |      |
|                  |                                        |      |                                          |                      | not expected that ETCD will need to serve as a variable name.                                          |      |
|                  |                                        |      |                                          |                      | 2. If an encountered element differs from the planned element to the point that it is considered a     |      |
|                  |                                        |      |                                          |                      | new element, then use "UNPLAN" as the value for ETCD to represent this element.                        |      |
| ELEMENT          | Description of<br>Element              | Char | *                                        | Synonym<br>Qualifier | The name of the element. If ETCD has a value of "UNPLAN", then ELEMENT should be null.                 | Perm |
| TAETORD          | Planned Order of<br>Element within Arm | Num  |                                          | Timing               | Number that gives the planned order of the element within the subject's assigned trial arm.            | Perm |
| EPOCH            | Epoch                                  | Char | (EPOCH)                                  | Timing               | Epoch associated with the element in the planned sequence of elements for the arm to which the subject | Perm |
|                  |                                        |      |                                          |                      | was assigned.                                                                                          |      |
| SESTDTC          | Start Date/Time of<br>Element          | Char | ISO 8601 datetime or<br>interval         | Timing               | Start date/time for an element for each subject.                                                       | Req  |
| SEENDTC          | End Date/Time of<br>Element            | Char | ISO 8601 datetime or<br>interval         | Timing               | End date/time for an element for each subject.                                                         | Exp  |
| SESTDY           | Study Day of Start of<br>Element       | Num  |                                          | Timing               | Study day of start of element relative to the sponsor-defined RFSTDTC.                                 | Perm |
| SEENDY           | Study Day of End of<br>Element         | Num  |                                          | Timing               | Study day of end of element relative to the sponsor-defined RFSTDTC.                                   | Perm |
| SEUPDES          | Description of<br>Unplanned Element    | Char |                                          | Synonym<br>Qualifier | Description of what happened to the subject during an unplanned element. Used only if ETCD has the     | Perm |
|                  |                                        |      |                                          |                      | value of "UNPLAN".                                                                                     |      |

## Page 81

4. If the sponsor decides that the subject's experience for a particular period
   of time cannot be represented with one of the planned elements, then that
   period of time should be represented as an unplanned element. The value of
   ETCD for an unplanned element is “UNPLAN” and SEUPDES should be populated
   with a description of the unplanned element.
5. The values of SESTDTC provide the chronological order of the actual subject
   elements. SESEQ should be assigned to be consistent with the chronological
   order. Note that the requirement that SESEQ be consistent with chronological
   order is more stringent than in most other domains, where - -SEQ values need
   only be unique within subject.
6. When TAETORD is included in the SE domain, it represents the planned order of
   an element in an arm. This should not be confused with the actual order of
   the elements, which will be represented by their chronological order and
   SESEQ. TAETORD will not be populated for subject elements that are not
   planned for the arm to which the subject was assigned. Thus, TAETORD will not
   be populated for any element with an ETCD value of “UNPLAN”. TAETORD also
   will not be populated if a subject passed through an element that, although
   defined in the TE dataset, was out of place for the arm to which the subject
   was assigned. For example, if a subject in a parallel study of drug A vs.
   drug B was assigned to receive drug A but received drug B instead, then
   TAETORD would be left blank for the SE record for their drug B element. If a
   subject was assigned to receive the sequence of elements A, B, C, D, and
   instead received A, D, B, C, then the sponsor would have to decide for which
   of these SE records TAETORD should be populated. The rationale for this
   decision should be documented in the Comments column of the Define-XML
   document.
7. For subjects who follow the planned sequence of elements for the arm to which
   they were assigned, the values of EPOCH in the SE domain will match those
   associated with the elements for the subject's arm in the TA dataset. The
   sponsor will have to decide what value, if any, of EPOCH to assign SE records
   for unplanned elements and in other cases where the subject's actual elements
   deviate from the plan. The sponsor's methods for such decisions should be
   documented in the Define-XML document, in the row for EPOCH in the SE dataset
   table.
8. Because there are, by definition, no gaps between elements, the value of
   SEENDTC for one element will always be the same as the value of SESTDTC for
   the next element.
9. Note that SESTDTC is required, although --STDTC is not required in any other
   subject-level dataset. The purpose of the dataset is to record the elements a
   subject actually passed through. If it is known that a subject passed through
   a particular element, then there must be some information (perhaps imprecise)
   on when it started. Thus, SESTDTC may not be null, although some records may
   not have all the components (e.g., year, month, day, hour, minute) of the
   date/time value collected.
10. The following identifier variables are permissible and may be added as
    appropriate: --GRPID, --REFID, --SPID.
11. Care should be taken in adding additional timing variables: a. The purpose
    of --DTC and --DY is to record the date and study day on which data was
    collected. Elements are generally “derived” in the sense that they are a
    secondary use of data collected elsewhere; it is not generally useful to
    know when those date/times were recorded. b. --DUR could be added only if
    the duration of an element was collected, not derived. c. It would be
    inappropriate to add the variables that support time points (--TPT,
    --TPTNUM, --ELTM, --TPTREF, and --RFTDTC), because the topic of this dataset
    is elements. SE – Examples STUDYID and DOMAIN, which are required in the SE
    and Demographics (DM) domains, have not been included in the following
    examples, to improve readability.

## Page 82

Example 1 This example shows data for 2 subjects for a crossover trial with 4
epochs. Row 1: The record for the SCREEN element for subject 789. Note that only
the date of the start of the SCREEN element was collected, whereas for the end
of the element (which corresponds to the start of IV dosing) both date and time
were collected. Row 2: The record for the IV element for subject 789. The IV
element started with the start of IV dosing and ended with the start of oral
dosing, and full date/times were collected for both. Row 3: The record for the
ORAL element for subject 789. Only the date, and not the time, of the start of
follow-up was collected. Row 4: The FOLLOWUP element for subject 789 started and
ended on the same day. Presumably, the element had a positive duration, but no
times were collected. Rows 5-8: Subject 790 was treated incorrectly. This
subject entered the IV element before the ORAL element, although the planned
order of elements for this subject was ORAL, then IV. The sponsor has assigned
EPOCH values for this subject according to the actual order of elements, rather
than the planned order. Per Assumption 6, TAETORD is missing for the elements
that were out of order. The correct order of elements is the subject's ARMCD,
shown in the DM dataset. Rows 9-10: Subject 791 was screened, randomized to the
IV-ORAL arm, and received the IV treatment, but did not return to the unit for
the treatment epoch or follow-up. se.xpt

Row 1: Subject 789 was assigned to the IV-ORAL arm and was treated accordingly.
Row 2: Subject 790 was assigned to the ORAL-IV arm, but their actual treatment
was IV, then oral. Row 3: Subject 791 was assigned to the IV-ORAL arm, received
the first of the 2 planned treatment elements, and were following the assigned
treatment when they withdrew early. The actual arm variables are populated with
the values for the arm to which subject 791 was assigned. dm.xpt

| Row | USUBJID | SESEQ | ETCD     | SESTDTC          | SEENDTC          | SEUPDES | TAETORD | EPOCH       |
| --- | ------- | ----- | -------- | ---------------- | ---------------- | ------- | ------- | ----------- |
| 1   | 789     | 1     | SCREEN   | 2006-06-01       | 2006-06-03T10:32 |         | 1       | SCREENING   |
| 2   | 789     | 2     | IV       | 2006-06-03T10:32 | 2006-06-10T09:47 |         | 2       | TREATMENT 1 |
| 3   | 789     | 3     | ORAL     | 2006-06-10T09:47 | 2006-06-17       |         | 3       | TREATMENT 2 |
| 4   | 789     | 4     | FOLLOWUP | 2006-06-17       | 2006-06-17       |         | 4       | FOLLOW-UP   |
| 5   | 790     | 1     | SCREEN   | 2006-06-01       | 2006-06-03T10:14 |         | 1       | SCREENING   |
| 6   | 790     | 2     | IV       | 2006-06-03T10:14 | 2006-06-10T10:32 |         |         | TREATMENT 1 |
| 7   | 790     | 3     | ORAL     | 2006-06-10T10:32 | 2006-06-17       |         |         | TREATMENT 2 |
| 8   | 790     | 4     | FOLLOWUP | 2006-06-17       | 2006-06-17       |         | 4       | FOLLOW-UP   |
| 9   | 791     | 1     | SCREEN   | 2006-06-01       | 2006-06-03T10:17 |         | 1       | SCREENING   |
| 10  | 791     | 2     | IV       | 2006-06-03T10:17 | 2006-06-07       |         | 2       | TREATMENT 1 |

| Row | USUBJID S | UBJID | RFSTDTC        | RFENDTC        | SITEID | INVNAM      | BIRTHDTC   | AGE A | GEU  | SEX | RACE  | ETHNIC                       | ARMCD | ARM         | ACTARMCD | ACTARM  | ARMNRS | ACTARMUD | COUNTRY |
| --- | --------- | ----- | -------------- | -------------- | ------ | ----------- | ---------- | ----- | ---- | --- | ----- | ---------------------------- | ----- | ----------- | -------- | ------- | ------ | -------- | ------- |
| 1   | 789 0     | 01    | 2006-06-<br>03 | 2006-06-<br>17 | 01     | SMITH,<br>J | 1948-12-13 | 57 Y  | EARS | M   | WHITE | HISPANIC OR<br>LATINO        | IO    | IV-<br>ORAL | IO       | IV-ORAL |        |          | USA     |
| 2   | 790 0     | 02    | 2006-06-<br>03 | 2006-06-<br>17 | 01     | SMITH,<br>J | 1955-03-22 | 51 Y  | EARS | M   | WHITE | NOT<br>HISPANIC OR<br>LATINO | OI    | ORAL-<br>IV | IO       | IV-ORAL |        |          | USA     |

## Page 83

Example 2 The following data represent 2 subjects enrolled in a trial in which
assignment to an arm occurs in 2 stages. See Section 7.2.1, Trial Arms, Example
Trial 3. In this trial, subjects were randomized at the beginning of the blinded
treatment epoch, then assigned to treatment for the open treatment epoch
according to their response to treatment in the blinded treatment epoch. See
Section 5.2, Demographics, for other examples of ARM and ARMCD values for this
trial. In this trial, start of dosing was recorded as dates without times, so
SESTDTC values include only dates. Epochs could not be assigned to observations
that occurred on epoch transition dates on the basis of the SE dataset alone, so
the sponsor's algorithms for dealing with this ambiguity were documented in the
Define-XML document. Rows 1-2: Show data for a subject who completed only 2
elements of the trial. Rows 3-6: Show data for a subject who completed the
trial, but received the wrong drug for the last 2 weeks of the double-blind
treatment period. This has been represented by treating the period when the
subject received the wrong drug as an unplanned element. Note that TAETORD,
which represents the planned order of elements within an arm, has not been
populated for this unplanned element. Even though this element was unplanned,
the sponsor assigned a value of BLINDED TREATMENT to EPOCH. se.xpt

| Row | USUBJID | SUBJID | RFSTDTC        | RFENDTC        | SITEI | D INVNAM    | BIRTHDTC   | AGE | AGEU  | SEX | RACE  | ETHNIC                       | ARMCD | ARM         | ACTARMCD | ACTARM  | ARMNRS | ACTARMUD | COUNTRY |
| --- | ------- | ------ | -------------- | -------------- | ----- | ----------- | ---------- | --- | ----- | --- | ----- | ---------------------------- | ----- | ----------- | -------- | ------- | ------ | -------- | ------- |
| 3   | 791     | 003    | 2006-06-<br>03 | 2006-06-<br>07 | 01    | SMITH,<br>J | 1956-07-17 | 49  | YEARS | M   | WHITE | NOT<br>HISPANIC OR<br>LATINO | IO    | IV-<br>ORAL | IO       | IV-ORAL |        |          | USA     |

Row 1: Shows the record for a subject who was randomized to blinded treatment A,
but withdrew from the trial before the open treatment epoch and did not have a
second treatment assignment. They were thus incompletely assigned to an arm. The
code used to represent this incomplete assignment, "A", is not in the TA table
for this trial design, but is the first part of the codes for the 2 arms to
which subject 123 could have been assigned ("AR" or "AO"). Row 2: Shows the
record for a subject who was randomized to blinded treatment A, but was
erroneously treated with drug B for part of the blinded treatment epoch. ARM and
ARMCD for this subject reflect the planned treatment and are not affected by the
fact that treatment deviated from plan. The subject's assignment to Rescue
treatment for the open treatment epoch proceeded as planned. The sponsor decided
that the subject's treatment, which consisted partly of drug A and partly of
drug B, did not match any planned arm, so ACTARMCD and ACTARM were left null.
ARMNRS was populated with "UNPLANNED TREATMENT" and the way in which this
treatment was unplanned was described in ACTARMUD. dm.xpt

| Row | USUBJID | SESEQ | ETCD   | SESTDTC      | SEENDTC   | SEUPDES                   | TAETORD | EPOCH                |
| --- | ------- | ----- | ------ | ------------ | --------- | ------------------------- | ------- | -------------------- |
| 1   | 123     | 1     | SCRN   | 2006-06-01 2 | 006-06-03 |                           | 1       | SCREENING            |
| 2   | 123     | 2     | DBA    | 2006-06-03 2 | 006-06-10 |                           | 2       | BLINDED TREATMENT    |
| 3   | 456     | 1     | SCRN   | 2006-05-01 2 | 006-05-03 |                           | 1       | SCREENING            |
| 4   | 456     | 2     | DBA    | 2006-05-03 2 | 006-05-31 |                           | 2       | BLINDED TREATMENT    |
| 5   | 456     | 3     | UNPLAN | 2006-05-31 2 | 006-06-13 | Drug B dispensed in error |         | BLINDED TREATMENT    |
| 6   | 456     | 4     | RSC    | 2006-06-13 2 | 006-07-30 |                           | 3       | OPEN LABEL TREATMENT |

| Row | USUBJID | SUBJID     | RFSTDTC          | RFENDTC      | SITEI | D INVNAM    | BIRTHDTC   | AGE | AGEU  | SEX | RACE  | ETHNIC                   | ARMCD | ARM A | CTARMCD | ACTARM | ARMNRS | ACTARMUD | COUNTRY |
| --- | ------- | ---------- | ---------------- | ------------ | ----- | ----------- | ---------- | --- | ----- | --- | ----- | ------------------------ | ----- | ----- | ------- | ------ | ------ | -------- | ------- |
| 1   | 123     | 012 2<br>0 | 006-06- 2<br>3 1 | 006-06-<br>0 | 01    | JONES,<br>D | 1943-12-08 | 62  | YEARS | M   | ASIAN | HISPANIC<br>OR<br>LATINO | A     | A A   |         | A      |        |          | USA     |

## Page 84

5.4 Subject Disease Milestones (SM) SM – Description/Overview A special-purpose
domain that is designed to record the timing, for each subject, of disease
milestones that have been defined in the Trial Disease Milestones (TM) domain.
SM – Specification sm.xpt, Subject Disease Milestones — Special Purpose. One
record per Disease Milestone per subject, Tabulation.

| Row | USUBJID | SUBJID | RFSTDTC        | RFENDTC        | SITEID | INVNAM      | BIRTHDTC   | AGE | AGEU  | SEX | RACE  | ETHNIC                          | ARMCD | ARM          | ACTARMCD | ACTARM | ARMNRS                 | ACTARMUD                                                | COUNTRY |
| --- | ------- | ------ | -------------- | -------------- | ------ | ----------- | ---------- | --- | ----- | --- | ----- | ------------------------------- | ----- | ------------ | -------- | ------ | ---------------------- | ------------------------------------------------------- | ------- |
| 2   | 456     | 103    | 2006-05-<br>03 | 2006-07-<br>30 | 01     | JONES,<br>D | 1950-05-15 | 55  | YEARS | F   | WHITE | NOT<br>HISPANIC<br>OR<br>LATINO | AR    | A-<br>Rescue |          |        | UNPLANNED<br>TREATMENT | Drug B<br>dispensed<br>for part of<br>Drug A<br>element | USA     |

1In this column, an asterisk (*) indicates that the variable may be subject to
controlled terminology. CDISC/NCI codelist values are enclosed in parentheses.
SM – Assumptions

1. Disease milestones are observations or activities whose timings are of
   interest in the study. The types of disease milestones are defined at the
   study level in the TM dataset. The purpose of the SM dataset is to provide a
   summary timeline of the milestones for a particular subject.
2. The name of the disease milestone is recorded in MIDS. a. For disease
   milestones that can occur only once (TMRPT = "N"), the value of MIDS may be
   the value in MIDSTYPE or may an abbreviated version.

| Variable<br>Name | Variable Label                     | Type | Controlled Terms,<br>Codelist or Format1 | Role                | CDISC Notes                                                                                           | Core |
| ---------------- | ---------------------------------- | ---- | ---------------------------------------- | ------------------- | ----------------------------------------------------------------------------------------------------- | ---- |
| STUDYID          | Study Identifier                   | Char |                                          | Identifier          | Unique identifier for a study.                                                                        | Req  |
| DOMAIN           | Domain Abbreviation                | Char | SM                                       | Identifier          | Two-character abbreviation for the domain.                                                            | Req  |
| USUBJID          | Unique Subject<br>Identifier       | Char |                                          | Identifier          | Identifier used to uniquely identify a subject across all studies for all applications or submissions | Req  |
|                  |                                    |      |                                          |                     | involving the product.                                                                                |      |
| SMSEQ            | Sequence Number                    | Num  |                                          | Identifier          | Sequence number to ensure uniqueness of subject records. Should be assigned to be                     | Req  |
|                  |                                    |      |                                          |                     | consistent chronological order.                                                                       |      |
| MIDS             | Disease Milestone<br>Instance Name | Char | *                                        | Topic               | Name of the specific disease milestone. For types of disease milestones that can occur multiple       | Req  |
|                  |                                    |      |                                          |                     | times, the name will end with a sequence number. Example: "HYPO1".                                    |      |
| MIDSTYPE         | Disease Milestone<br>Type          | Char | *                                        | Record<br>Qualifier | The type of disease milestone. Example: "HYPOGLYCEMIC EVENT".                                         | Req  |
| SMSTDTC          | Start Date/Time of<br>Milestone    | Char | ISO 8601 datetime or<br>interval         | Timing              | Start date/time of milestone instance (if milestone is an intervention or event) or date of           | Exp  |
|                  |                                    |      |                                          |                     | milestone (if Milestone is a finding).                                                                |      |
| SMENDTC          | End Date/Time of<br>Milestone      | Char | ISO 8601 datetime or<br>interval         | Timing              | End date/time of disease milestone instance.                                                          | Exp  |
| SMSTDY           | Study Day of Start of<br>Milestone | Num  |                                          | Timing              | Study day of start of disease milestone instance, relative to the sponsor-defined RFSTDTC.            | Exp  |
| SMENDY           | Study Day of End of<br>Milestone   | Num  |                                          | Timing              | Study day of end of disease milestone instance, relative to the sponsor-defined RFSTDTC.              | Exp  |

## Page 85

b. For types of disease milestones that can occur multiple times, MIDS will
usually be an abbreviated version of MIDSTYPE and will always end with a
sequence number. Sequence numbers should start with 1 and indicate the
chronological order of the instances of this type of disease milestone. 3. The
timing variables SMSTDTC and SMENDTC hold start and end date/times of data
collected for the disease milestone(s) for each subject. SMSTDY and SMENDY
represent the corresponding study day variables. a. The start date/time of the
disease milestone is the critical date/time, and must be populated. If the
disease milestone is an event, then the meaning of “start date” for the event
may need to be defined. b. The start study day will not be populated if the
start date/time includes only a year or only a year and month. c. The end
date/time for the disease milestone is less important than the start date/time.
It will not be populated if the disease milestone is a finding without an end
date/time or if it is an event or intervention for which an end date/time has
not yet occurred or was not collected. d. The end study day will not be
populated if the end date/time includes only a year or only a year and month. SM
– Examples Example 1 In this study, the disease milestones of interest were
initial diagnosis and hypoglycemic events, as shown in Section 7.3.3, Trial
Disease Milestones, Example 1. Row 1: Shows that subject 001's initial diagnosis
of diabetes occurred in October 2005. Because this is a partial date, SMDY is
not populated. No end date/time was recorded for this milestone. Rows 2-3: Show
that subject 001 had 2 hypoglycemic events. In this case, only start date/times
have been collected. Because these date/times include full dates, SMSTDY has
been populated in each case. Row 4: Shows that subject 002’s initial diagnosis
of diabetes occurred on May 15, 2010. Because a full date was collected, the
study day of this milestone was populated. Diagnosis was pre-study, so the study
day of the disease milestone is negative. No hypoglycemic events were recorded
for this subject. sm.xpt

Information in SM is taken from records in other domains. In this study,
diagnosis was represented in the Medical History (MH) domain, and hyypoglycemic
events were represented in the Clinical Events (CE) domain. The MH records for
diabetes (MHEVDTYP = "DIAGNOSIS") are the records which represent the disease
milestones for the defined MIDSTYPE of "DIAGNOSIS", so these records include the
MIDS variable with the value "DIAG". Because these are records for disease
milestones rather than associated records, the variables RELMIDS and MIDSDTC are
not needed. mh.xpt

| Row | STUDYID | DOMAIN | USUBJID | SMSEQ | MIDS  | MIDSTYPE           | SMSTDTC          | SMENDTC | SMSTDY | SMENDY |
| --- | ------- | ------ | ------- | ----- | ----- | ------------------ | ---------------- | ------- | ------ | ------ |
| 1   | XYZ     | SM     | 001     | 1     | DIAG  | DIAGNOSIS          | 2005-10          |         |        |        |
| 2   | XYZ     | SM     | 001     | 2     | HYPO1 | HYPOGLYCEMIC EVENT | 2013-09-01T11:00 |         | 25     |        |
| 3   | XYZ     | SM     | 001     | 3     | HYPO2 | HYPOGLYCEMIC EVENT | 2013-09-24T8:48  |         | 50     |        |
| 4   | XYZ     | SM     | 002     | 1     | DIAG  | DIAGNOSIS          | 2010-05-15       |         | -1046  |        |

| Row | STUDYID | DOMAIN | USUBJID | MHSEQ | MHTERM          | MHDECOD                  | MHEVDTYP  | MHPRESP | MHOCCUR | MHDTC      | MHSTDTC    | MHENDTC | MHDY | MIDS |
| --- | ------- | ------ | ------- | ----- | --------------- | ------------------------ | --------- | ------- | ------- | ---------- | ---------- | ------- | ---- | ---- |
| 1   | XYZ     | MH     | 001     | 1     | TYPE 2 DIABETES | Type 2 diabetes mellitus | DIAGNOSIS | Y       | Y       | 2013-08-06 | 2005-10    |         | 1    | DIAG |
| 2   | XYZ     | MH     | 002     | 1     | TYPE 2 DIABETES | Type 2 diabetes mellitus | DIAGNOSIS | Y       | Y       | 2013-08-06 | 2010-05-15 |         | 1    | DIAG |

## Page 86

In this study, information about hypoglycemic events was collected in a separate
CRF module, and CE records recorded in this module were represented with CECAT =
"HYPOGLYCEMIC EVENT". Each CE record for a hypoglycemic event is a disease
milestone, and records for a study have distinct values of MIDS. ce.xpt

5.5 Subject Visits (SV) SV – Description/Overview A special purpose domain that
contains information for each subject's actual and planned visits. The Subject
Visits domain consolidates information about the timing of subject visits that
is otherwise spread over domains that include the visit variables (VISITNUM and
possibly VISIT and/or VISITDY). Unless the beginning and end of each visit is
collected, populating the SV dataset will involve derivations. In a simple case,
where, for each subject visit, exactly 1 date appears in every such domain, the
SV dataset can be created easily by populating both SVSTDTC and SVENDTC with the
single date for a visit. When there are multiple dates and/or date/times for a
visit for a particular subject, the derivation of values for SVSTDTC and SVENDTC
may be more complex. The method for deriving these values should be consistent
with the visit definitions in the Trial Visits (TV) dataset (see Section 7.3.1,
Trial Visits). For some studies, a visit may be defined to correspond with a
clinic visit that occurs within 1 day, whereas for other studies, a visit may
reflect data collection over a multiday period. The SV dataset provides
reviewers with a summary of a subject’s visits over the course of their
participation in a study. Comparison of an individual subject’s SV dataset with
the TV dataset, which describes the planned visits for the trial, supports the
identification of planned but not expected visits due to a subject not
completing the study. Comparison of the values of SVSTDY and SVENDY to VISIT
and/or VISITDY can often highlight departures from the planned timing of visits.
SV – Specification sv.xpt, Subject Visits — Special Purpose. One record per
actual or planned visit per subject, Tabulation.

| Row | STUDYID | DOMAIN | USUBJID | CESEQ C | ETERM             | CEDECOD C       | ECAT              | CEPRESP | CEOCCUR | CESTDTC          | CEENDTC          | MIDS  |
| --- | ------- | ------ | ------- | ------- | ----------------- | --------------- | ----------------- | ------- | ------- | ---------------- | ---------------- | ----- |
| 1   | XYZ     | CE     | 001     | 1 H     | YPOGLYCEMIC EVENT | Hypoglycaemia H | YPOGLYCEMIC EVENT | Y       | Y       | 2013-09-01T11:00 | 2013-09-01T2:30  | HYPO1 |
| 2   | XYZ     | CE     | 001     | 1 H     | YPOGLYCEMIC EVENT | Hypoglycaemia H | YPOGLYCEMIC EVENT | Y       | Y       | 2013-09-24T8:48  | 2013-09-24T10:00 | HYPO2 |

| Variable<br>Name | Variable Label            | Type | Controlled Terms,<br>Codelist or Format1 | Role                  | CDISC Notes                                                                                                                                                                                     | Core |
| ---------------- | ------------------------- | ---- | ---------------------------------------- | --------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---- |
| STUDYID          | Study Identifier          | Char |                                          | Identifier            | Unique identifier for a study.                                                                                                                                                                  | Req  |
| DOMAIN           | Domain Abbreviation       | Char | SV                                       | Identifier            | Two-character abbreviation for the domain most relevant to the observation. The domain<br>abbreviation is also used as a prefix for variables to ensure uniqueness when datasets are<br>merged. | Req  |
| USUBJID          | Unique Subject Identifier | Char |                                          | Identifier            | Identifier used to uniquely identify a subject across all studies for all applications or<br>submissions involving the product.                                                                 | Req  |
| VISITNUM         | Visit Number              | Num  |                                          | Topic                 | Clinical encounter number. Numeric version of VISIT, used for sorting.                                                                                                                          | Req  |
| VISIT            | Visit Name                | Char |                                          | Synonym<br>Qualifier  | Protocol-defined description of a clinical encounter.                                                                                                                                           | Perm |
| SVPRESP          | Pre-specified             | Char | (NY)                                     | Variable<br>Qualifier | Used to indicate whether the visit was planned (i.e., visits specified in the TV domain). Value<br>is "Y" for planned visits, null for unplanned visits.                                        | Exp  |
| SVOCCUR          | Occurrence                | Char | (NY)                                     | Record<br>Qualifier   | Used to record whether a planned visit occurred. The value is null for unplanned visits.                                                                                                        | Exp  |
| SVREASOC         | Reason for Occur Value    | Char |                                          | Record<br>Qualifier   | The reason for the value in SVOCCUR. If SVOCCUR="N", SVREASOC is the reason the<br>visit did not occur.                                                                                         | Perm |

## Page 87

1In this column, an asterisk (*) indicates that the variable may be subject to
controlled terminology. CDISC/NCI codelist values are enclosed in parentheses.
SV – Assumptions

1. The Subject Visits domain allows the submission of data on the timing of the
   trial visits for a subject, including both those visits they actually passed
   through in their participation in the trial and those visits that did not
   occur. Refer to Section 7.3.1, Trial Visits (TV), as the TV dataset defines
   the planned visits for the trial.
2. Subjects can have 1 and only 1 record per VISITNUM.
3. Subjects who screen fail, withdraw, die, or otherwise discontinue study
   participation will not have records for planned visits subsequent to their
   final disposition event.
4. Planned and unplanned visits with a subject, whether or not they are physical
   visits to the investigational site, are represented in this domain. a.
   SVPRESP = "Y" identifies rows for planned visits. b. For planned visits,
   SVOCCUR indicates whether the visit occurred. c. For unplanned visits,
   SVPRESP and SVOCCUR are null. d. See Section 4.5.7, Presence or Absence of
   Prespecified Interventions and Events, for more information on the use of
   --PRESP and --OCCUR.
5. The identification of an actual visit with a planned visit sometimes calls
   for judgment. In general, data collection forms are prepared for particular
   visits, and the fact that data was collected on a form labeled with a planned
   visit is sufficient to make the association. Occasionally, the association
   will not be so clear, and the sponsor will need to make decisions about how
   to label actual visits. The sponsor's rules for making such decisions should
   be documented in the Define-XML document.
6. Records for unplanned visits should be included in the SV dataset. For
   unplanned visits, SVUPDES can be populated with a description of the reason
   for the unplanned visit. Some judgment may be required to determine what
   constitutes an unplanned visit. When data are collected outside a planned
   visit, that act of collecting data may or may not be described as a "visit."
   The encounter should generally be treated as a visit if data from the
   encounter are included in any domain for which VISITNUM is included; a record
   with a missing value for VISITNUM is generally less useful than a record with

| Variable<br>Name | Variable Label                           | Type | Controlled Terms,<br>Codelist or Format1 | Role                | CDISC Notes                                                                                 | Core |
| ---------------- | ---------------------------------------- | ---- | ---------------------------------------- | ------------------- | ------------------------------------------------------------------------------------------- | ---- |
| SVCNTMOD         | Contact Mode                             | Char | (CNTMODE)                                | Record<br>Qualifier | The way in which the visit was conducted. Examples: "IN PERSON", "TELEPHONE CALL",          | Perm |
|                  |                                          |      |                                          |                     | "IVRS".                                                                                     |      |
| SVEPCHGI         | Epi/Pandemic Related<br>Change Indicator | Char | (NY)                                     | Record<br>Qualifier | Indicates whether the visit was changed due to an epidemic or pandemic.                     | Perm |
| VISITDY          | Planned Study Day of<br>Visit            | Num  |                                          | Timing              | Planned study day of VISIT. Should be an integer.                                           | Perm |
| SVSTDTC          | Start Date/Time of<br>Observation        | Char | ISO 8601 datetime or<br>interval         | Timing              | Start date/time of an observation represented in IS0 8601 character format.                 | Exp  |
| SVENDTC          | End Date/Time of<br>Observation          | Char | ISO 8601 datetime or<br>interval         | Timing              | End date/time of the observation represented in IS0 8601 character format.                  | Exp  |
| SVSTDY           | Study Day of Start of<br>Observation     | Num  |                                          | Timing              | Actual study day of start of observation expressed in integer days relative to the sponsor- | Perm |
|                  |                                          |      |                                          |                     | defined RFSTDTC in Demographics.                                                            |      |
| SVENDY           | Study Day of End of<br>Observation       | Num  |                                          | Timing              | Actual study day of end of observation expressed in integer days relative to the sponsor-   | Perm |
|                  |                                          |      |                                          |                     | defined RFSTDTC in Demographics.                                                            |      |
| SVUPDES          | Description of Unplanned<br>Visit        | Char |                                          | Record<br>Qualifier | Description of what happened to the subject during an unplanned visit. Only populated for   | Perm |
|                  |                                          |      |                                          |                     | unplanned visits.                                                                           |      |

## Page 88

VISITNUM populated. If the occasion is considered a visit, its date/times must
be included in the SV table and a value of VISITNUM must be assigned. Refer to
Section 4.4.5, Clinical Encounters and Visits, for information on the population
of visit variables for unplanned visits. 7. The variable SVCNTMOD is used to
record the way in which the visit was conducted. For example, for visits to a
clinic, SVCNTMOD = "IN PERSON", visits conducted remotely might have values such
as "TELEPHONE", "REMOTE AUDIO VIDEO", or "IVRS". If there are multiple contact
modes, refer to Section 4.2.8.3, Multiple Values for a Non-result Qualifier
Variable. 8. The planned study day of visit variable (VISITDY) should not be
populated for unplanned visits. 9. If SVSTDY is included, it is the actual study
day corresponding to SVSTDTC. In studies for which VISITDY has been populated,
it may be desirable to populate SVSTDY, as this will facilitate the comparison
of planned (VISITDY) and actual (SVSTDY) study days for the start of a visit.
10. If SVENDY is included, it is the actual day corresponding to SVENDTC. 11.
For many studies, all visits are assumed to occur within 1 calendar day, and
only 1 date is collected for the visit. In such a case, the values for SVENDTC
duplicate values in SVSTDTC. However, if the data for a visit is actually
collected over several physical visits and/or over several days, then SVSTDTC
and SVENDTC should reflect this fact. Note that it is fairly common for
screening data to be collected over several days, but for the data to be treated
as belonging to a single planned screening visit, even in studies for which all
other visits are single-day visits. 12. Differentiating between planned and
unplanned visits may be challenging if unplanned assessments (e.g., repeat labs)
are performed during the time period of a planned visit. 13. Algorithms for
populating SVSTDTC and SVENDTC from the dates of assessments performed at a
visit may be particularly challenging for screening visits, since baseline
values collected at a screening visit are sometimes historical data from tests
performed before the subject started screening for the trial. Therefore dates
prior to informed consent are not part of the determination of SVSTDTC. 14. The
following Identifier variables are permissible and may be added as appropriate:
--SEQ, --GRPID, --REFID, and --SPID. 15. Care should be taken in adding
additional timing variables: a. If TAETORD and/or EPOCH are added, then the
values must be those at the start of the visit. b. The purpose of --DTC and --DY
in other domains with start and end dates (Event and Intervention Domains) is to
record the date on which data was collected. For a visit that occurred, it is
not necessary to submit the date on which information about the visit was
recorded. When SVPRESP = "Y" and SVOCCUR = "N", --DTC and --DY are available for
use to represent the date on which it was recorded that the visit did not take
place. c. --DUR could be added if the duration of a visit was collected. d. It
would be inappropriate to add the variables that support time points (--TPT,
--TPTNUM, --ELTM, --TPTREF, and --RFTDTC), because the topic of this dataset is
visits. e. --STRF and --ENRF could be used to say whether a visit started and
ended before, during, or after the study reference period, although this seems
unnecessary. f. --STRTPT, --STTPT, --ENRTPT, and --ENTPT could be used to say
that a visit started or ended before or after particular dates, although this
seems unnecessary. 16. SVOCCUR = "N" records are only to be created for planned
visits that were expected to occur before the end of the subject's
participation.

## Page 89

SV – Examples Example 1 This example shows the planned visit schedule for a
study, along with disposition and study events data for 3 subjects. For this
study, data on screen failures were submitted. The study was disrupted by the
COVID-19 pandemic after many subjects had completed the study. This is the
planned schedule of visits for the study in this example. Row 1: The activities
for the SCREEN visit may occur over up to 7 days. Row 2: The day 1 visit is
planned to start before the start of treatment and end after the start of
treatment. Rows 3-7: These visits are scheduled relative to the start of the
treatment epoch. Row 8: The follow-up visit is generally scheduled relative to
the start of the treatment epoch, but may occur earlier if treatment is stopped
early. tv.xpt

This table shows the disposition records for the subjects in this example. Row
1: Shows informed consent for subject 37. Row 2: Shows the subject 37 was
discontinued due to screen failure. Note that because the subject did not start
treatment, DSSTDY is not populated in their records. Row 3: Shows informed
consent for subject 85. Row 4: Shows that subject 85 completed the study. Row 5:
Shows informed consent for subject 101 Row 6: Shows that subject 101 chose to
withdraw early. ds.xpt

| Row | STUDYID | DOMAIN | VISITNUM | VISIT         | VISITDY | TVSTRL                                                                                                         | TVENRL                                                        |
| --- | ------- | ------ | -------- | ------------- | ------- | -------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| 1   | 123456  | TV     | 1        | SCREEN        |         | Start of Screening Epoch                                                                                       | Up to 7 days after start of the Screening Epoch               |
| 2   | 123456  | TV     | 2        | DAY 1         | 1       | On the day of, but before, the end of the Screen Epoch                                                         | On the day of, but after, the start of the<br>Treatment Epoch |
| 3   | 123456  | TV     | 3        | WEEK 1        | 8       | 1 week after the start of the Treatment Epoch                                                                  |                                                               |
| 4   | 123456  | TV     | 4        | WEEK 2        | 15      | 2 weeks after the start of the Treatment Epoch                                                                 |                                                               |
| 5   | 123456  | TV     | 5        | WEEK 4        | 29      | 4 weeks after the start of the Treatment Epoch                                                                 |                                                               |
| 6   | 123456  | TV     | 6        | WEEK 6        | 43      | 6 weeks after the start of the Treatment Epoch                                                                 |                                                               |
| 7   | 123456  | TV     | 7        | WEEK 8        | 57      | 8 weeks after the start of the Treatment Epoch                                                                 |                                                               |
| 8   | 123456  | TV     | 8        | FOLLOW-<br>UP |         | The earlier of 14 days after the last dose of treatment and 10 weeks after the start of the<br>Treatment Epoch | At Trial Exit                                                 |

| Row | STUDYID | DOMAIN | USUBJID D | SSEQ | DSTERM                       | DSDECOD                      | DSCAT                 | DSSCAT                 | EPOCH     | DSDTC          | DSSTDTC        | DSSTDY |
| --- | ------- | ------ | --------- | ---- | ---------------------------- | ---------------------------- | --------------------- | ---------------------- | --------- | -------------- | -------------- | ------ |
| 1   | 123456  | DS     | 37 1      |      | INFORMED CONSENT<br>OBTAINED | INFORMED CONSENT<br>OBTAINED | PROTOCOL<br>MILESTONE |                        | SCREENING | 2019-09-<br>10 | 2019-09-<br>10 |        |
| 2   | 123456  | DS     | 37 2      |      | SCREEN FAILURE               | SCREEN FAILURE               | DISPOSITION EVENT     | STUDY<br>PARTICIPATION | SCREENING | 2019-09-<br>16 | 2019-09-<br>16 |        |
| 3   | 123456  | DS     | 85 1      |      | INFORMED CONSENT<br>OBTAINED | INFORMED CONSENT<br>OBTAINED | PROTOCOL<br>MILESTONE |                        | SCREENING | 2019-12-<br>13 | 2019-12-<br>13 | -6     |
| 4   | 123456  | DS     | 85 2      |      | COMPLETED                    | COMPLETED                    | DISPOSITION EVENT     | STUDY<br>PARTICIPATION | TREATMENT | 2020-02-<br>27 | 2020-02-<br>27 | 72     |

## Page 90

Because the study in this example was disrupted by an epidemic, the permissible
variable SVEPCHGI (Epi/Pandemic Related Change Indicator) was included in the SV
dataset. As originally planned, visits were to be conducted in person, but
pandemic disruption included conducting some visits remotely. When the change to
a remote visit was a change due to the pandemic, SVEPCHGI = "Y". Row 1: Shows
that screening data for subject 37 was collected during a period of 4 days. This
subject is shown as a screen failure in ds.xpt and therefore would have a null
DM.RFSTDTC, hence the study day values in SVSTDY and SVENDY, which are based on
the sponsor-defined reference start date, are null. Rows 2-3: Show normal
completion of the first 2 visits for subject 85. Row 4: Shows that for subject
85, the visit called "WEEK 1" did not occur; the reason it did not occur is
represented in SVREASOC. Rows 5-9: Normal completion of remaining visits for
subject 85. Row 10: Data for the screening visit was gathered over the course of
six days. For this and subsequent visits, SVPRESP = "Y" indicates that a visit
was planned and SVOCCUR = "Y" indicates that the visit occurred. Row 11: The
visit called "DAY 1" started and ended as planned, on Day 1. Row 12: The visit
scheduled for Day 8 occurred one day early, on Day 7. Row 13: The visit called
"WEEK 2" did not occur due to clinic closure. SVOCCUR = "N" and SVREASOC
contains the reason the visit did not occur. Row 14: Shows an unscheduled visit.
SVUPDES provides the information that this visit dealt with evaluation of an
adverse event. Since this visit was not planned, VISITDY was not populated,
SVPRESP and SVOCCUR are both null. VISITNUM is populated as required, but the
sponsor chose not to populate VISIT. Data collected at this encounter may be in
a Findings domain such as EG, LB, or VS, in which VISITNUM is treated as an
important timing variable. This visit was over remote audio video due to having
an adverse event during a pandemic. Row 15: This subject had their last visit, a
follow-up visit on study day 26, eight days after the unscheduled visit. sv.xpt

| Row | STUDYID | DOMAIN | USUBJID | DSSEQ D  | STERM                      | DSDECOD                      | DSCAT D                  | SSCAT                | EPOCH     | DSDTC          | DSSTDTC        | DSSTDY |
| --- | ------- | ------ | ------- | -------- | -------------------------- | ---------------------------- | ------------------------ | -------------------- | --------- | -------------- | -------------- | ------ |
| 5   | 123456  | DS     | 101     | 1 I<br>O | NFORMED CONSENT<br>BTAINED | INFORMED CONSENT<br>OBTAINED | PROTOCOL<br>MILESTONE    |                      | SCREENING | 2020-02-<br>13 | 2020-02-<br>13 | -6     |
| 6   | 123456  | DS     | 101     | 2        | WITHDRAWAL BY SUBJECT      | WITHDRAWAL BY SUBJECT        | DISPOSITION EVENT S<br>P | TUDY<br>ARTICIPATION | TREATMENT | 2020-03-<br>16 | 2020-03-<br>16 | 26     |

| Row | STUDYID | DOMAIN | USUBJID | VISITNUM | VISIT  | SVPRESP | SVOCCUR | SVREASOC                         | SVCNTMOD  | SVEPCHGI | VISITDY | SVSTDTC        | SVENDTC        | SVSTDY | SVENDY | SVUPDES |
| --- | ------- | ------ | ------- | -------- | ------ | ------- | ------- | -------------------------------- | --------- | -------- | ------- | -------------- | -------------- | ------ | ------ | ------- |
| 1   | 123456  | SV     | 37      | 1        | SCREEN | Y       | Y       |                                  | IN PERSON |          |         | 2019-09-<br>10 | 2019-09-<br>16 |        |        |         |
| 2   | 123456  | SV     | 85      | 1        | SCREEN | Y       | Y       |                                  | IN PERSON |          |         | 2019-12-<br>13 | 2019-12-<br>18 | -6     | -1     |         |
| 3   | 123456  | SV     | 85      | 2        | DAY 1  | Y       | Y       |                                  | IN PERSON |          | 1       | 2019-12-<br>19 | 2019-12-<br>19 | 1      | 1      |         |
| 4   | 123456  | SV     | 85      | 3        | WEEK 1 | Y       | N       | SUBJECT LACKED<br>TRANSPORTATION |           |          | 8       |                |                |        |        |         |
| 5   | 123456  | SV     | 85      | 4        | WEEK 2 | Y       | Y       |                                  | IN PERSON |          | 15      | 2020-01-<br>02 | 2020-01-<br>02 | 15     | 15     |         |
| 6   | 123456  | SV     | 85      | 5        | WEEK 4 | Y       | Y       |                                  | IN PERSON |          | 29      | 2020-01-<br>16 | 2020-01-<br>16 | 30     | 30     |         |
| 7   | 123456  | SV     | 85      | 6        | WEEK 6 | Y       | Y       |                                  | IN PERSON |          | 43      | 2020-01-<br>30 | 2020-01-<br>30 | 43     | 43     |         |
| 8   | 123456  | SV     | 85      | 7        | WEEK 8 | Y       | Y       |                                  | IN PERSON |          | 57      | 2020-02-<br>13 | 2020-02-<br>13 | 57     | 57     |         |

## Page 91

| Row | STUDYID | DOMAIN | USUBJID | VISITNUM | VISIT         | SVPRESP | SVOCCUR | SVREASOC                            | SVCNTMOD              | SVEPCHGI | VISITDY | SVSTDTC        | SVENDTC        | SVSTDY | SVENDY | SVUPDES             |
| --- | ------- | ------ | ------- | -------- | ------------- | ------- | ------- | ----------------------------------- | --------------------- | -------- | ------- | -------------- | -------------- | ------ | ------ | ------------------- |
| 9   | 123456  | SV     | 85      | 8        | FOLLOW-<br>UP | Y       | Y       |                                     | IN PERSON             |          |         | 2020-02-<br>27 | 2020-02-<br>27 | 72     | 72     |                     |
| 10  | 123456  | SV     | 101     | 1        | SCREEN        | Y       | Y       |                                     | IN PERSON             |          |         | 2020-02-<br>13 | 2020-02-<br>18 | -6     | -1     |                     |
| 11  | 123456  | SV     | 101     | 2        | DAY 1         | Y       | Y       |                                     | IN PERSON             |          | 1       | 2020-02-<br>19 | 2020-02-<br>19 | 1      | 1      |                     |
| 12  | 123456  | SV     | 101     | 3        | WEEK 1        | Y       | Y       |                                     | IN PERSON             |          | 8       | 2020-02-<br>25 | 2020-02-<br>25 | 7      | 7      |                     |
| 13  | 123456  | SV     | 101     | 4        | WEEK 2        | Y       | N       | CLINIC CLOSED DUE<br>TO BAD WEATHER |                       |          | 15      |                |                |        |        |                     |
| 14  | 123456  | SV     | 101     | 4.1      |               |         |         |                                     | REMOTE<br>AUDIO VIDEO | Y        |         | 2020-03-<br>07 | 2020-03-<br>07 | 18     | 18     | EVALUATION<br>OF AE |
| 15  | 123456  | SV     | 101     | 8        | FOLLOW-<br>UP | Y       | Y       |                                     | TELEPHONE<br>CALL     | Y        |         | 2020-03-<br>16 | 2020-03-<br>16 | 26     | 26     |                     |
