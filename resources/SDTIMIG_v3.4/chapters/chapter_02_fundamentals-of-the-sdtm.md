# Chapter 2: FUNDAMENTALS OF THE SDTM

> Source pages 11–16 in `SDTMIG_v3.4.pdf`.

## Page 11

2 Fundamentals of the SDTM 2.1 Observations and Variables The SDTMIG for Human
Clinical Trials is based on the SDTM’s general framework for organizing clinical
trial information that is to be submitted to regulatory authorities. The SDTM is
built around the concept of observations collected about subjects who
participated in a clinical study. Each observation can be described by a series
of variables, corresponding to a row in a dataset. Each variable can be
classified according to its role. A role determines the type of information
conveyed by the variable about each distinct observation and how it can be used.
Variables can be classified into 5 major roles: • Identifier variables, such as
those that identify the study, subject, domain, and sequence number of the
record • Topic variables, which specify the focus of the observation (e.g., the
name of a lab test) • Timing variables, which describe the timing of the
observation (e.g., start date and end date) • Qualifier variables, which include
additional illustrative text or numeric values that describe the results or
additional traits of the observation (e.g., units, descriptive adjectives) •
Rule variables, which describe the condition to start, end, branch, or loop in
the Trial Design Model The set of Qualifier variables can be further categorized
into 5 subclasses: • Grouping Qualifiers are used to group together a collection
of observations within the same domain. Examples include --CAT and --SCAT. •
Result Qualifiers describe the specific results associated with the topic
variable in a Findings dataset. They answer the question raised by the topic
variable. Result Qualifiers are --ORRES, --STRESC, and -- STRESN. • Synonym
Qualifiers specify an alternative name for a particular variable in an
observation. Examples include ‑‑MODIFY and --DECOD, which are equivalent terms
for a --TRT or --TERM topic variable, and - -TEST and ‑‑LOINC, which are
equivalent terms for a --TESTCD. • Record Qualifiers define additional
attributes of the observation record as a whole (rather than describing a
particular variable within a record). Examples include --REASND, AESLIFE, and
all other serious adverse event (SAE) flag variables in the AE domain; AGE, SEX,
and RACE in the DM domain; and --BLFL, -- POS, --LOC, --SPEC and --NAM in a
Findings domain • Variable Qualifiers are used to further modify or describe a
specific variable within an observation and are only meaningful in the context
of the variable they qualify. Examples include --ORRESU, --ORNRHI, and ‑‑ORNRLO,
all of which are Variable Qualifiers of --ORRES; and --DOSU, which is a Variable
Qualifier of ‑‑DOSE. For example, in the observation, "Subject 101 had mild
nausea starting on study day 6," the Topic variable value is the term for the
adverse event, "NAUSEA". The Identifier variable is the subject identifier,
"101". The Timing variable is the study day of the start of the event, which
captures the information, "starting on study day 6," whereas an example of a
Record Qualifier is the severity, the value for which is "MILD". Additional
Timing and Qualifier variables could be included to provide the necessary detail
to adequately describe an observation. 2.2 Datasets and Domains Observations
about study subjects are normally collected for all subjects in a series of
domains. A domain is defined as a collection of logically related observations
with a common topic. The logic of the relationship may pertain to the scientific
subject matter of the data or to its role in the trial. Each domain is
represented by a single dataset. Each domain dataset is distinguished by a
unique, 2-character code that should be used consistently throughout the
submission. This code, which is stored in the SDTM variable named DOMAIN, is
used in 4 ways: as the dataset

## Page 12

name, as the value of the DOMAIN variable in that dataset, as a prefix for most
variable names in that dataset, and as a value in the RDOMAIN variable in
relationship tables (see Section 8, Representing Relationships and Data). All
datasets are structured as flat files with rows representing observations and
columns representing variables. Each dataset is described by metadata
definitions that provide information about the variables used in the dataset.
The metadata are described in a data definition document (i.e., a Define-XML
document) that is submitted with the data to regulatory authorities. The
Define-XML standard (available at
https://www.cdisc.org/standards/dataexchange/define-xml) specifies metadata
attributes to describe SDTM data. Data represented in SDTM datasets include data
as originally collected or received, data from the protocol, assigned data, and
derived data. The SDTM lists only the name, label, and type, with a set of brief
CDISC guidelines that provide a general description for each variable. The
domain dataset models in Section 5, Models for Special-purpose Domains, and
Section 6, Domain Models Based on the General Observation Classes, provide
additional information about controlled terms or format, notes on proper usage,
and examples. See also Section 1.4.1, How to Read a Domain Specification. 2.3
The General Observation Classes Most subject-level observations collected during
the study should be represented according to 1 of the 3 SDTM general observation
classes: Interventions, Events, or Findings. The lists of variables allowed to
be used in each of these can be found in the SDTM. • The Interventions class
captures investigational, therapeutic, and other treatments that are
administered to the subject (with some actual or expected physiological effect)
either as specified by the study protocol (e.g., exposure to study drug),
coincident with the study assessment period (e.g., concomitant medications), or
self‑administered by the subject (e.g., use of alcohol, tobacco, or caffeine). •
The Events class captures planned protocol milestones such as randomization and
study completion, and occurrences, conditions, or incidents independent of
planned study evaluations occurring during the trial (e.g., adverse events) or
prior to the trial (e.g., medical history). • The Findings class captures the
observations resulting from planned evaluations to address specific tests or
questions (e.g., laboratory tests, ECG testing, questions listed on
questionnaires). In most cases, the choice of observation class appropriate to a
specific collection of data can be easily determined according to these
descriptions. The majority of data, which typically consists of measurements or
responses to questions, usually at specific visits or time points, will fit the
Findings general observation class. Additional guidance on choosing the
appropriate general observation class is provided in Section 8.6.1, Guidelines
for Determining the General Observation Class. General assumptions for use with
all domain models and custom domains based on the general observation classes
are described in Section 4, Assumptions for Domain Models; specific assumptions
for individual domains are included with the domain models. 2.4 Datasets Other
than General Observation Class Domains The SDTM includes 4 types of datasets
other than those based on the general observation classes: • Domain datasets
with subject-level data that do not conform to 1 of the 3 general observation
classes. These include Demographics (DM), Comments (CO), Subject Elements (SE),
and Subject Visits (SV), and are described in Section 5, Models for
Special-purpose Domains. • Trial Design Model (TDM) datasets, which represent
information about the study design but do not contain subject data. These
include datasets such as Trial Arms (TA) and Trial Elements (TE) and are
described in Section 7, Trial Design Model Datasets. • Relationship datasets,
such as the RELREC and SUPP-- datasets. These are described in Section 8,
Representing Relationships and Data.

## Page 13

• Study Reference datasets include Device Identifiers (DI) and Non-host Organism
Identifiers (OI). These provide structures for representing study-specific
terminology used in subject data. These are described in Section 9, Study
References. 2.5 The SDTM Standard Domain Models A sponsor should only submit
domain datasets that were actually collected (or directly derived from the
collected data) for a given study. Decisions on what data to collect should be
based on the scientific objectives of the study, rather than the SDTM. Note that
any data collected that will be submitted in an analysis (ADaM) dataset must be
traceable to a source in a tabulation (SDTM) dataset. The collected data for a
given study may use standard domains from this and other SDTM implementation
guides as well as additional custom domains based on the 3 general observation
classes. A list of standard domains is provided in Section 3.2.1, Dataset-level
Metadata. Final domains will be published only in an SDTM implementation guide
(this guide or another implementation guide, e.g., SDTMIG for Medical Devices).
Therapeutic-area standards projects and other projects may develop proposals for
additional domains. Draft versions of these domains may be made available in the
CDISC wiki in the SDTM Draft Domains space
(https://wiki.cdisc.org/display/SDD/SDTM+Draft+Domains+Home). These general
rules apply when determining which variables to include in a domain: • The
Identifier variables, STUDYID, USUBJID, DOMAIN, and --SEQ are required in all
domains based on the general observation classes. Other Identifiers may be added
as needed. • Any Timing variables are permissible for use in any submission
dataset based on a general observation class except where restricted by specific
domain assumptions. • Any additional Qualifier variables from the same general
observation class may be added to a domain model except where restricted by
specific domain assumptions. • Sponsors may not add any variables other than
those described in the preceding 3 bullets. The SDTM allows for the inclusion of
a sponsor's non-SDTM variables using the Supplemental Qualifiers specialpurpose
dataset structure, described in Section 8.4, Relating Non-standard Variable
Values to a Parent Domain. As the SDTM continues to evolve, certain additional
standard variables may be added to the general observation classes. • Standard
variables must not be renamed or modified for novel usage. Their metadata should
not be changed. • A Permissible variable should be used in an SDTM dataset
wherever appropriate. o If a study includes a data item that would be
represented in a Permissible variable, then that variable must be included in
the SDTM dataset, even if null. Refer to the Define-XML standard (available at
https://www.cdisc.org/standards/data-exchange/define-xml) for additional details
on how to manage no data availability. o If a study did not include a data item
that would be represented in a Permissible variable, then that variable should
not be included in the SDTM dataset and should not be declared in the Define-XML
document. 2.6 Creating a New Domain This section describes the overall process
for creating a custom domain, which must be based on 1 of the 3 SDTM general
observation classes. The number of domains submitted should be based on the
specific requirements of the study. To create a custom domain,

1. Confirm that none of the existing published domains will fit the need. A
   custom domain may only be created if the data are different in nature and do
   not fit into an existing published domain. o Establish a domain of a common
   topic; that is, where the nature of the data is the same rather than by a
   specific method of collection (e.g., electrocardiogram). Group and separate
   data within the domain using --CAT, --SCAT, --METHOD, --SPEC, --LOC, and so
   on, as appropriate. Examples of different

## Page 14

topics are: microbiology, tumor measurements, pathology/histology, vital signs,
and physical exam results. o Do not create separate domains based on time;
rather, represent both prior and current observations in a domain (e.g., CM for
all non-study medications). Note that Adverse Events (AE) and Medical History
(MH) are an exception to this best practice because of regulatory reporting
needs. o How collected data are used (e.g., to support analyses and/or efficacy
endpoints) must not result in the creation of a custom domain. For example, if
blood pressure measurements are endpoints in a hypertension study, they must
still be represented in the Vital Signs (VS) domain, as opposed to a custom
“efficacy” domain. Similarly, if liver function test results are of special
interest, they must still be represented in the Laboratory Tests (LB) domain. o
Data that were collected on separate CRF modules or pages may fit into an
existing domain (e.g., as separate questionnaires into the QS domain, prior and
concomitant medications in the CM domain). o If it is necessary to represent
relationships between data that are hierarchical in nature (e.g., a parent
record must be observed before child records), then establish a domain pair
(e.g., MB/MS, PC/PP). Note: Domain pairs have been modeled for microbiology data
(MB/MS domains) and pharmacokinetics (PK) data (PC/PP domains) to enable
dataset-level relationships to be described using RELREC. The domain pair uses
DOMAIN as an identifier to group parent records (e.g., MB) from child records
(e.g., MS) and enables a dataset-level relationship to be described in RELREC.
Without using DOMAIN to facilitate description of the data relationships,
RELREC, as currently defined, could not be used without introducing a variable
that would group data like DOMAIN. 2. Check the SDTM Draft Domains area of the
CDISC wiki (https://wiki.cdisc.org/display/SDD/SDTM+Draft+Domains+Home) for
proposed domains developed since the last published version of the SDTMIG. These
proposed domains may be used as custom domains in a submission. 3. Look for an
existing, relevant domain model to serve as a prototype. If no existing model
seems appropriate, choose the general observation class (Interventions, Events,
or Findings) that best fits the data by considering the topic of the
observation. As illustrated in the following figure, the general approach for
selecting variables for a custom domain is: a. Select and include the required
identifier variables (e.g., STUDYID, DOMAIN, USUBJID, --SEQ) and any permissible
Identifier variables from the SDTM. b. Include the topic variable from the
identified general observation class (e.g., --TESTCD for Findings) in the SDTM.
c. Select and include the relevant qualifier variables from the identified
general observation class in the SDTM. Variables belonging to other general
observation classes must not be added. d. Select and include the applicable
timing variables in the SDTM. e. Determine the domain code, one that is not a
domain code in the CDISC Controlled Terminology SDTM Domain Abbreviations
codelist (see
https://datascience.cancer.gov/resources/cancervocabulary/cdisc-terminology). If
it is desired to have this domain code be part of CDISC Controlled Terminology,
submit a request at https://ncitermform.nci.nih.gov/ncitermform/?version=cdisc.
The sponsor-selected, 2‑character domain code should be used consistently
throughout the submission. AD, AX, AP, SQ, and SA may not be used as custom
domain codes. f. Apply the 2-character domain code to the appropriate variables
in the domain. Replace all variable prefixes (shown in the models as “--“) with
the domain code. g. Set the order of variables consistent with the order defined
in the SDTM for the general observation class. h. Adjust the labels of the
variables only as appropriate to properly convey the meaning in the context of
the data being submitted in the newly created domain. Use title case for all
labels (title case means to capitalize the first letter of every word except for
articles, prepositions, and conjunctions).

## Page 15

i. Ensure that appropriate standard variables are being properly applied by
comparing their use in the custom domain to their use in standard domains. j.
Describe the dataset within the Define-XML document. See Section 3.2, Using the
CDISC Domain Models in Regulatory Submissions — Dataset Metadata. k. Place any
non-standard (SDTM) variables in a Supplemental Qualifier dataset. Mechanisms
for representing additional non-standard qualifier variables not described in
the general observation classes and for defining relationships between separate
datasets or records are described in Section 8.4, Relating Non-standard Variable
Values to a Parent Domain. Figure. Creating a New Domain 2.7 SDTM Variables Not
Allowed in the SDTMIG This section identifies those SDTM variables that either
(1) should not be used in SDTM-compliant data tabulations of clinical trials
data or (2) have not yet been evaluated for use in human clinical trials. The
following SDTM variables, defined for use in nonclinical studies (SEND), must
NEVER be used in the submission of SDTM-based data for human clinical trials: •
--USCHFL (Interventions, Events, Findings) • --METHOD (Interventions) • --RSTIND
(Interventions, Findings) • --RSTMOD (Interventions, Findings) • --IMPLBL
(Findings) • --RESLOC (Findings) • --DTHREL (Findings) • --EXCLFL (Findings) •
--REASEX (Findings) • FETUSID (Identifiers) • RPHASE (Timing Variables) • RPPLDY
(Timing Variables) • RPPLSTDY (Timing Variables) • RPPLENDY (Timing Variables)

## Page 16

• --NOMDY (Timing Variables) • --NOMLBL (Timing Variables) • --RPDY (Timing
Variables) • --RPSTDY (Timing Variables) • --RPENDY (Timing Variables) •
--DETECT (Timing Variables) The following variables can be used for nonclinical
studies (SEND) but must NEVER be used in the Demographics (DM) domain for human
clinical trials, where all subjects are human. See Section 9.2, Non-host
Organism Identifiers, for information about representing taxonomic information
for non-host organisms such as bacteria and viruses. • SPECIES (Demographics) •
STRAIN (Demographics) • SBSTRAIN (Demographics) • RPATHCD (Demographics) The
following variables have not been evaluated for use in human clinical trials and
must therefore be used with extreme caution: • --ANTREG (Findings) • --CHRON
(Findings) • --DISTR (Findings) • SETCD (Demographics) The use of SETCD
additionally requires the use of the Trials Sets domain. The following
identifier variable can be used for nonclinical studies (SEND), and may be used
in human clinical trials when appropriate: • POOLID The use of POOLID
additionally requires the use of the Pool Definition dataset. Other variables
defined in the SDTM are allowed for use as defined in this SDTMIG except when
explicitly stated. Custom domains, created following the guidance in Section
2.6, Creating a New Domain, may utilize any appropriate qualifier variables from
the selected general observation class.
