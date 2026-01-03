# Chapter 3: SUBMITTING DATA IN STANDARD FORMAT

> Source pages 17–21 in `SDTMIG_v3.4.pdf`.

## Page 17

3 Submitting Data in Standard Format 3.1 Standard Metadata for Dataset Contents
and Attributes The SDTMIG provides standard descriptions of some of the most
commonly used data domains, with metadata attributes. These include descriptive
metadata attributes that should be included in a Define-XML document. In
addition, the CDISC domain models include 2 shaded columns that are not sent to
the FDA, but which assist sponsors in preparing their datasets: • The CDISC
Notes column provides information regarding the relevant use of each variable. •
The Core column indicates how a variable is classified (see Section 4.1.5, SDTM
Core Designations). The domain models in Section 6, Domain Models Based on the
General Observation Classes, illustrate how to apply the SDTM when creating a
specific domain dataset. In particular, these models illustrate the selection of
a subset of the variables offered in 1 of the general observation classes, along
with applicable timing variables. The models also show how a standard variable
from a general observation class should be adjusted to meet the specific content
needs of a particular domain, including making the label more meaningful,
specifying controlled terminology, and creating domain-specific notes and
examples. Thus, the domain models not only demonstrate how to apply the model
for the most common domains but also give insight on how to apply general model
concepts to other domains not yet defined by CDISC. 3.2 Using the CDISC Domain
Models in Regulatory Submissions – Dataset Metadata The Define-XML document that
accompanies a submission should also describe each dataset that is included in
the submission and describe the natural key structure of each dataset. Most
studies will include Demographics (DM) and a set of safety domains based on the
3 general observation classes—typically including Exposure (EX), Concomitant and
Prior Medications (CM), Adverse Events (AE), Disposition (DS), Medical History
(MH), Laboratory Test Results (LB), and Vital Signs (VS). However, choosing
which data to submit will depend on the protocol and the needs of the regulatory
review division or agency. Dataset definition metadata should include the
dataset filenames, descriptions, locations, structures, class, purpose, and
keys, as shown in Section 3.2.1, Dataset-level Metadata. In addition, comments
can also be provided where needed. 3.2.1 Dataset-level Metadata Note that the
key variables shown in this table are examples only. A sponsor’s actual key
structure may be different. The order of classes and datasets in this table is
not intended as a normative order of datasets in a submission.

| Dataset | Description                      | Class              | Structure                                                                                                 | Purpose    | Keys                                           | Location |
| ------- | -------------------------------- | ------------------ | --------------------------------------------------------------------------------------------------------- | ---------- | ---------------------------------------------- | -------- |
| CO      | Comments                         | Special<br>Purpose | One record per comment per<br>subject                                                                     | Tabulation | STUDYID, USUBJID,<br>IDVAR, COREF, CODTC       | co.xpt   |
| DM      | Demographics                     | Special<br>Purpose | One record per subject                                                                                    | Tabulation | STUDYID, USUBJID                               | dm.xpt   |
| SE      | Subject Elements                 | Special<br>Purpose | One record per actual Element per<br>subject                                                              | Tabulation | STUDYID, USUBJID,<br>ETCD, SESTDTC             | se.xpt   |
| SM      | Subject Disease<br>Milestones    | Special<br>Purpose | One record per Disease Milestone<br>per subject                                                           | Tabulation | STUDYID, USUBJID,<br>MIDS                      | sm.xpt   |
| SV      | Subject Visits                   | Special<br>Purpose | One record per actual or planned<br>visit per subject                                                     | Tabulation | STUDYID, USUBJID,<br>SVTERM                    | sv.xpt   |
| AG      | Procedure Agents                 | Interventions      | One record per recorded<br>intervention occurrence per subject                                            | Tabulation | STUDYID, USUBJID,<br>AGTRT, AGSTDTC            | ag.xpt   |
| CM      | Concomitant/Prior<br>Medications | Interventions      | One record per recorded<br>intervention occurrence or constant-<br>dosing interval per subject            | Tabulation | STUDYID, USUBJID,<br>CMTRT, CMSTDTC            | cm.xpt   |
| EC      | Exposure as<br>Collected         | Interventions      | One record per protocol-specified<br>study treatment, collected-dosing<br>interval, per subject, per mood | Tabulation | STUDYID, USUBJID,<br>ECTRT, ECSTDTC,<br>ECMOOD | ec.xpt   |

## Page 18

| Dataset | Description                               | Class         | Structure                                                                                                                             | Purpose    | Keys                                                                        | Location |
| ------- | ----------------------------------------- | ------------- | ------------------------------------------------------------------------------------------------------------------------------------- | ---------- | --------------------------------------------------------------------------- | -------- |
| EX      | Exposure                                  | Interventions | One record per protocol-specified<br>study treatment, constant-dosing<br>interval, per subject                                        | Tabulation | STUDYID, USUBJID,<br>EXTRT, EXSTDTC                                         | ex.xpt   |
| ML      | Meal Data                                 | Interventions | One record per food product<br>occurrence or constant intake<br>interval per subject                                                  | Tabulation | STUDYID, USUBJID,<br>MLTRT, MLSTDTC                                         | ml.xpt   |
| PR      | Procedures                                | Interventions | One record per recorded procedure<br>per occurrence per subject                                                                       | Tabulation | STUDYID, USUBJID,<br>PRTRT, PRSTDTC                                         | pr.xpt   |
| SU      | Substance Use                             | Interventions | One record per substance type per<br>reported occurrence per subject                                                                  | Tabulation | STUDYID, USUBJID,<br>SUTRT, SUSTDTC                                         | su.xpt   |
| AE      | Adverse Events                            | Events        | One record per adverse event per<br>subject                                                                                           | Tabulation | STUDYID, USUBJID,<br>AEDECOD, AESTDTC                                       | ae.xpt   |
| BE      | Biospecimen Events                        | Events        | One record per instance per<br>biospecimen event per biospecimen<br>identifier per subject                                            | Tabulation | STUDYID, USUBJID,<br>BEREFID, BETERM,<br>BESDTC                             | be.xpt   |
| CE      | Clinical Events                           | Events        | One record per event per subject                                                                                                      | Tabulation | STUDYID, USUBJID,<br>CETERM, CESTDTC                                        | ce.xpt   |
| DS      | Disposition                               | Events        | One record per disposition status or<br>protocol milestone per subject                                                                | Tabulation | STUDYID, USUBJID,<br>DSDECOD, DSSTDTC                                       | ds.xpt   |
| DV      | Protocol Deviations                       | Events        | One record per protocol deviation<br>per subject                                                                                      | Tabulation | STUDYID, USUBJID,<br>DVTERM, DVSTDTC                                        | dv.xpt   |
| HO      | Healthcare<br>Encounters                  | Events        | One record per healthcare<br>encounter per subject                                                                                    | Tabulation | STUDYID, USUBJID,<br>HOTERM, HOSTDTC                                        | ho.xpt   |
| MH      | Medical History                           | Events        | One record per medical history<br>event per subject                                                                                   | Tabulation | STUDYID, USUBJID,<br>MHDECOD                                                | mh.xpt   |
| BS      | Biospecimen<br>Findings                   | Findings      | One record per measurement per<br>biospecimen identifier per subject                                                                  | Tabulation | STUDYID, USUBJID,<br>BSREFID, BSTESTCD                                      | bs.xpt   |
| CP      | Cell Phenotype<br>Findings                | Findings      | One record per test per specimen<br>per timepoint per visit per subject                                                               | Tabulation | STUDYID, USUBJID,<br>CPTESTCD, CPSPEC,<br>VISITNUM, CPTPTREF,<br>CPTPTNUM   | cp.xpt   |
| CV      | Cardiovascular<br>System Findings         | Findings      | One record per finding or result per<br>time point per visit per subject                                                              | Tabulation | STUDYID, USUBJID,<br>VISITNUM, CVTESTCD,<br>CVTPTREF, CVTPTNUM              | cv.xpt   |
| DA      | Product<br>Accountability                 | Findings      | One record per product<br>accountability finding per subject                                                                          | Tabulation | STUDYID, USUBJID,<br>DATESTCD, DADTC                                        | da.xpt   |
| DD      | Death Details                             | Findings      | One record per finding per subject                                                                                                    | Tabulation | STUDYID, USUBJID,<br>DDTESTCD, DDDTC                                        | dd.xpt   |
| EG      | ECG Test Results                          | Findings      | One record per ECG observation<br>per replicate per time point or one<br>record per ECG observation per<br>beat per visit per subject | Tabulation | STUDYID, USUBJID,<br>EGTESTCD, VISITNUM,<br>EGTPTREF,<br>EGTPTNUM           | eg.xpt   |
| FT      | Functional Tests                          | Findings      | One record per Functional Test<br>finding per time point per visit per<br>subject                                                     | Tabulation | STUDYID, USUBJID,<br>TESTCD, VISITNUM,<br>FTTPTREF, FTTPTNUM                | ft.xpt   |
| GF      | Genomics Findings                         | Findings      | One record per finding per<br>observation per biospecimen per<br>subject                                                              | Tabulation | STUDYID, USUBJID,<br>GFTESTCD, GFSPEC,<br>VISITNUM, GFTPTREF,<br>GFTPTNUM   | gf.xpt   |
| IE      | Inclusion/Exclusion<br>Criteria Not Met   | Findings      | One record per inclusion/exclusion<br>criterion not met per subject                                                                   | Tabulation | STUDYID, USUBJID,<br>IETESTCD                                               | ie.xpt   |
| IS      | Immunogenicity<br>Specimen<br>Assessments | Findings      | One record per test per visit per<br>subject                                                                                          | Tabulation | STUDYID, USUBJID,<br>ISTESTCD, ISBDAGNT,<br>ISSCMBCL, ISTSTOPO,<br>VISITNUM | is.xpt   |
| LB      | Laboratory Test<br>Results                | Findings      | One record per lab test per time<br>point per visit per subject                                                                       | Tabulation | STUDYID, USUBJID,<br>LBTESTCD, LBSPEC,<br>VISITNUM, LBTPTREF,<br>LBTPTNUM   | lb.xpt   |
| MB      | Microbiology<br>Specimen                  | Findings      | One record per microbiology<br>specimen finding per time point per<br>visit per subject                                               | Tabulation | STUDYID, USUBJID,<br>MBTESTCD, VISITNUM,<br>MBTPTREF,<br>MBTPTNUM           | mb.xpt   |
| MI      | Microscopic Findings                      | Findings      | One record per finding per<br>specimen per subject                                                                                    | Tabulation | STUDYID, USUBJID,<br>MISPEC, MITESTCD                                       | mi.xpt   |
| MK      | Musculoskeletal<br>System Findings        | Findings      | One record per assessment per visit<br>per subject                                                                                    | Tabulation | STUDYID, USUBJID,<br>VISITNUM, MKTESTCD,<br>MKLOC, MKLAT                    | mk.xpt   |
| MS      | Microbiology<br>Susceptibility            | Findings      | One record per microbiology<br>susceptibility test (or other<br>organism-related finding) per<br>organism found in MB                 | Tabulation | STUDYID, USUBJID,<br>MSTESTCD, VISITNUM,<br>MSTPTREF,<br>MSTPTNUM           | ms.xpt   |

## Page 19

| Dataset | Description                                    | Class             | Structure                                                                                                                                                    | Purpose    | Keys                                                                                                                                              | Location |
| ------- | ---------------------------------------------- | ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| NV      | Nervous System<br>Findings                     | Findings          | One record per finding per location<br>per time point per visit per subject                                                                                  | Tabulation | STUDYID, USUBJID,<br>VISITNUM, NVTPTNUM,<br>NVLOC, NVTESTCD                                                                                       | nv.xpt   |
| OE      | Ophthalmic<br>Examinations                     | Findings          | One record per ophthalmic finding<br>per method per location, per time<br>point per visit per subject                                                        | Tabulation | STUDYID, USUBJID,<br>FOCID, OETESTCD,<br>OETSTDTL,<br>OEMETHOD, OELOC,<br>OELAT, OEDIR,<br>VISITNUM, OEDTC,<br>OETPTREF,<br>OETPTNUM,<br>OEREPNUM | oe.xpt   |
| PC      | Pharmacokinetics<br>Concentrations             | Findings          | One record per sample<br>characteristic or time-point<br>concentration per reference time<br>point or per analyte per subject                                | Tabulation | STUDYID, USUBJID,<br>PCTESTCD, VISITNUM,<br>PCTPTREF, PCTPTNUM                                                                                    | pc.xpt   |
| PE      | Physical<br>Examination                        | Findings          | One record per body system or<br>abnormality per visit per subject                                                                                           | Tabulation | STUDYID, USUBJID,<br>PETESTCD, VISITNUM                                                                                                           | pe.xpt   |
| PP      | Pharmacokinetics<br>Parameters                 | Findings          | One record per PK parameter per<br>time-concentration profile per<br>modeling method per subject                                                             | Tabulation | STUDYID, USUBJID,<br>PPTESTCD, PPCAT,<br>VISITNUM, PPRFTDTC                                                                                       | pp.xpt   |
| QS      | Questionnaires                                 | Findings          | One record per questionnaire per<br>question per time point per visit per<br>subject                                                                         | Tabulation | STUDYID, USUBJID,<br>QSCAT, QSSCAT,<br>VISITNUM, QSTESTCD                                                                                         | qs.xpt   |
| RE      | Respiratory System<br>Findings                 | Findings          | One record per finding or result per<br>time point per visit per subject                                                                                     | Tabulation | STUDYID, USUBJID,<br>VISITNUM, RETESTCD,<br>RETPTNUM,<br>REREPNUM                                                                                 | re.xpt   |
| RP      | Reproductive<br>System Findings                | Findings          | One record per finding or result per<br>time point per visit per subject                                                                                     | Tabulation | STUDYID, DOMAIN,<br>USUBJID, RPTESTCD,<br>VISITNUM                                                                                                | rp.xpt   |
| RS      | Disease Response<br>and Clin<br>Classification | Findings          | One record per response<br>assessment or clinical classification<br>assessment per time point per visit<br>per subject per assessor per<br>medical evaluator | Tabulation | STUDYID, USUBJID,<br>RSTESTCD, VISITNUM,<br>RSTPTREF,<br>RSTPTNUM, RSEVAL,<br>RSEVALID                                                            | rs.xpt   |
| SC      | Subject<br>Characteristics                     | Findings          | One record per characteristic per<br>visit per subject.                                                                                                      | Tabulation | STUDYID, USUBJID,<br>SCTESTCD, VISITNUM                                                                                                           | sc.xpt   |
| SS      | Subject Status                                 | Findings          | One record per status per visit per<br>subject                                                                                                               | Tabulation | STUDYID, USUBJID,<br>SSTESTCD, VISITNUM                                                                                                           | ss.xpt   |
| TR      | Tumor/Lesion<br>Results                        | Findings          | One record per tumor<br>measurement/assessment per visit<br>per subject per assessor                                                                         | Tabulation | STUDYID, USUBJID,<br>TRTESTCD, TREVALID,<br>VISITNUM                                                                                              | tr.xpt   |
| TU      | Tumor/Lesion<br>Identification                 | Findings          | One record per identified tumor per<br>subject per assessor                                                                                                  | Tabulation | STUDYID, USUBJID,<br>TUEVALID, TULNKID                                                                                                            | tu.xpt   |
| UR      | Urinary System<br>Findings                     | Findings          | One record per finding per location<br>per per visit per subject                                                                                             | Tabulation | STUDYID, USUBJID,<br>VISITNUM, URTESTCD,<br>URLOC, URLAT, URDIR                                                                                   | ur.xpt   |
| VS      | Vital Signs                                    | Findings          | One record per vital sign<br>measurement per time point per<br>visit per subject                                                                             | Tabulation | STUDYID, USUBJID,<br>VSTESTCD, VISITNUM,<br>VSTPTREF, VSTPTNUM                                                                                    | vs.xpt   |
| FA      | Findings About<br>Events or<br>Interventions   | Findings<br>About | One record per finding, per object,<br>per time point, per visit per subject                                                                                 | Tabulation | STUDYID, USUBJID,<br>FATESTCD, FAOBJ,<br>VISITNUM, FATPTREF,<br>FATPTNUM                                                                          | fa.xpt   |
| SR      | Skin Response                                  | Findings<br>About | One record per finding, per object,<br>per time point, per visit per subject                                                                                 | Tabulation | STUDYID, USUBJID,<br>SRTESTCD, SROBJ,<br>VISITNUM, SRTPTREF,<br>SRTPTNUM                                                                          | sr.xpt   |
| TA      | Trial Arms                                     | Trial Design      | One record per planned Element<br>per Arm                                                                                                                    | Tabulation | STUDYID, ARMCD,<br>TAETORD                                                                                                                        | ta.xpt   |
| TD      | Trial Disease<br>Assessments                   | Trial Design      | One record per planned constant<br>assessment period                                                                                                         | Tabulation | STUDYID, TDORDER                                                                                                                                  | td.xpt   |
| TE      | Trial Elements                                 | Trial Design      | One record per planned Element                                                                                                                               | Tabulation | STUDYID, ETCD                                                                                                                                     | te.xpt   |
| TI      | Trial<br>Inclusion/Exclusion<br>Criteria       | Trial Design      | One record per I/E criterion                                                                                                                                 | Tabulation | STUDYID, IETESTCD                                                                                                                                 | ti.xpt   |
| TM      | Trial Disease<br>Milestones                    | Trial Design      | One record per Disease Milestone<br>type                                                                                                                     | Tabulation | STUDYID, MIDSTYPE                                                                                                                                 | tm.xpt   |
| TS      | Trial Summary                                  | Trial Design      | One record per trial summary<br>parameter value                                                                                                              | Tabulation | STUDYID, TSPARMCD,<br>TSSEQ                                                                                                                       | ts.xpt   |

## Page 20

Separate Supplemental Qualifier datasets of the form supp--.xpt are required.
See Section 8.4, Relating Nonstandard Variable Values to a Parent Domain.
3.2.1.1 Primary Keys The table in Section 3.2.1, Dataset-level Metadata, shows
examples of what a sponsor might submit as variables that comprise the primary
key for SDTM datasets. Because the purpose of the Keys column is to aid
reviewers in understanding the structure of a dataset, sponsors should list all
of the natural keys for the dataset. These keys should define uniqueness for
records within a dataset, and may define a record sort order. The identified
keys for each dataset should be consistent with the description of the dataset
structure as described in the Define-XML document. For all the general
observation-class domains (and for some special-purpose domains), the --SEQ
variable was created so that a unique record could be identified consistently
across all of these domains via its use, along with STUDYID, USUBJID, and
DOMAIN. In most domains, --SEQ will be a surrogate key for a set of variables
that comprise the natural key. In certain instances, a supplemental qualifier
(SUPP--) variable might also contribute to the natural key of a record for a
particular domain. See Section 4.1.9, Assigning Natural Keys in the Metadata,
for how this should be represented, and for additional information on keys.
Definitions A natural key is a set of data (1 or more columns of an entity) that
uniquely identifies that entity and distinguishes it from any other row in the
table. The advantage of natural keys is that they exist already; one does not
need to introduce a new, “unnatural” value to the data schema. One of the
difficulties in choosing a natural key is that just about any natural key one
can think of has the potential to change. Because they have business meaning,
natural keys are effectively coupled to the business, and they may need to be
reworked when business requirements change. An example of such a change in
clinical trials data would be the addition of a position or location that
becomes a key in a new study, but which was not collected in previous studies. A
surrogate key is a single-part, artificially established identifier for a
record. Surrogate key assignment is a special case of derived data, one where a
portion of the primary key is derived. A surrogate key is immune to changes in
business needs. In addition, the key depends on only 1 field, so it is compact.
A common way of deriving surrogate key values is to assign integer values
sequentially. The --SEQ variable in the SDTM datasets is an example of a
surrogate key for most datasets; in some instances, however, --SEQ might be a
part of a natural key as a replacement for what might have been a key (e.g., a
repeat sequence number) in the sponsor's database. 3.2.1.2 CDISC Submission
Value-level Metadata In general, findings data models are closely related to
normalized, relational data models in a vertical structure of 1 record per
observation. Because general observation class data structures are fixed,
sometimes information that might appear as columns in a more horizontal
(denormalized) structure in presentations and reports will instead be
represented as rows in an SDTM Findings structure. Because many different types
of observations are all presented in the same structure, there is a need to
provide additional metadata to describe expected properties that differentiate
(e.g., hematology lab results from serum chemistry lab results in terms of data
type, standard units, and other attributes). For example, the Vital Signs (VS)
data domain could contain subject records related to diastolic and systolic
blood pressure, height, weight, and body mass index (BMI). These data are all
submitted in the normalized SDTM

| Dataset | Description                                     | Class              | Structure                                                                       | Purpose    | Keys                                                    | Location    |
| ------- | ----------------------------------------------- | ------------------ | ------------------------------------------------------------------------------- | ---------- | ------------------------------------------------------- | ----------- |
| TV      | Trial Visits                                    | Trial Design       | One record per planned Visit per<br>Arm                                         | Tabulation | STUDYID, ARM, VISIT                                     | tv.xpt      |
| RELREC  | Related Records                                 | Relationship       | One record per related record,<br>group of records or dataset                   | Tabulation | STUDYID, RDOMAIN,<br>USUBJID, IDVAR,<br>IDVARVAL, RELID | relrec.xpt  |
| RELSPEC | Related Specimens                               | Relationship       | One record per specimen identifier<br>per subject                               | Tabulation | STUDYID, USUBJID,<br>REFID                              | relspec.xpt |
| RELSUB  | Related Subjects                                | Relationship       | One record per relationship per<br>related subject per subject                  | Tabulation | STUDYID, USUBJID,<br>RSUBJID, SREL                      | relsub.xpt  |
| SUPP--  | Supplemental<br>Qualifiers for<br>[domain name] | Relationship       | One record per supplemental<br>qualifier per related parent domain<br>record(s) | Tabulation | STUDYID, RDOMAIN,<br>USUBJID, IDVAR,<br>IDVARVAL, QNAM  | supp--.xpt  |
| OI      | Non-host Organism<br>Identifiers                | Study<br>Reference | One record per taxon per non-host<br>organism                                   | Tabulation | NHOID, OISEQ                                            | oi.xpt      |

## Page 21

Findings structure of 1 row per vital signs measurement. This means that there
could be 5 records per subject (1 for each test or measurement) for a single
visit or time point, with the parameter names stored in the Test Code/Name
variables, and the parameter values stored in result variables. Because the
unique test code/names could have different attributes (e.g., different origins,
roles, definitions) there would be a need to provide value-level metadata for
this information. The value-level metadata should be provided as a separate
section of the Define-XML document. For details on the CDISC Define-XML
standard, see https://www.cdisc.org/standards/data-exchange/define-xml. 3.2.2
Conformance Conformance with the SDTMIG domain models is minimally indicated by:
• Following the complete metadata structure for data domains • Following SDTMIG
domain models wherever applicable • Using SDTM-specified standard domain names
and prefixes where applicable • Using SDTM-specified standard variable names •
Using SDTM-specified data types for all variables • Following SDTM-specified
controlled terminology and format guidelines for variables, when provided •
Including all collected and relevant derived data in one of the standard
domains, special-purpose datasets, or general observation class structures •
Including all Required and Expected variables as columns in standard domains,
and ensuring that all Required variables are populated • Ensuring that each
record in a dataset includes the appropriate Identifier and Timing variables, as
well as a Topic variable • Conforming to all business rules described in the
CDISC Notes column and general and domain-specific assumptions
