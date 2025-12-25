# Chapter 9: STUDY REFERENCES

> Source pages 441–443 in `SDTMIG_v3.4.pdf`.


## Page 441

9 Study References
There are occasions when it is necessary to establish study-specific terminology that will be used in tabulated data. The following situations have been identified thus far:
• Identifiers for devices
• Identifiers for non-host organisms
9.1 Device Identifiers
The Device Identifiers (DI) dataset establishes identifiers for devices, which are used to populate the variable SPDEVID. The DI dataset was introduced as part of the SDTMIG for Medical Devices (SDTMIG-MD). It was originally classified as a special-purpose domain, but since SDTM v1.7 it has been classified as a study reference dataset. The SDTMIG-MD (available at https://www.cdisc.org/standards/foundational/medical-devices-sdtmig/) includes the DI domain specification and assumptions and provides examples of its use.
9.2 Non-host Organism Identifiers (OI)
OI – Description/Overview A special-purpose domain containing information that identifies levels of taxonomic nomenclature of microbes or parasites that have been either experimentally determined in the course of a study or are previously known, as in the case of lab strains used as reference in the study. The biological classification of a non-host organism typically stops at the taxonomic rank of "species." Scientific taxonomic nomenclature below the rank of species is not clearly defined, lacks a globally accepted standard terminology, and is frequently organism-dependent. Therefore, the OI domain addresses organism taxonomy with a series of parameters that name the taxa appropriate to the organism and the granularity with which the organism has been identified in the particular study. OI – Specification oi.xpt, Non-host Organism Identifiers — Study Reference. One record per taxon per non-host organism, Tabulation.

1In this column, an asterisk (*) indicates that the variable may be subject to controlled terminology. CDISC/NCI codelist values are enclosed in parentheses.

| Variable<br>Name | Variable Label | Type | Controlled<br>Terms,<br>Codelist or<br>Format1 | Role | CDISC Notes | Core |
| --- | --- | --- | --- | --- | --- | --- |
| STUDYID | Study Identifier | Char |  | Identifier | Unique identifier for a study. | Req |
| DOMAIN | Domain<br>Abbreviation | Char | OI | Identifier | Two-character abbreviation for the domain. | Req |
| NHOID | Non-host<br>Organism<br>Identifier | Char |  | Identifier | Sponsor-defined identifier for a non-host organism. NHOID | Req |
|  |  |  |  |  | should be populated with an intuitive name based on the |  |
|  |  |  |  |  | identity of the organism as reported by the lab. It must be |  |
|  |  |  |  |  | unique for each unique organism as defined by the specific |  |
|  |  |  |  |  | values of the organism's entire known taxonomy described by |  |
|  |  |  |  |  | pairs of OIPARMCD and OIVAL . |  |
| OISEQ | Sequence<br>Number | Num |  | Identifier | Sequence number to given to ensure uniqueness within a | Req |
|  |  |  |  |  | parameter within an organism (NHOID) within dataset. |  |
| OIPARMCD | Non-host<br>Organism ID<br>Element Short<br>Name | Char | (OIPRMCD) | Topic | Short name of the taxon being described. Examples: | Req |
|  |  |  |  |  | "GROUP", "GENTYP", "SUBTYP". |  |
| OIPARM | Non-host<br>Organism ID<br>Element Name | Char | (OIPRM) | Synonym<br>Qualifier | Name of the taxon being described. Examples: "Group", | Req |
|  |  |  |  |  | "Genotype", "Subtype". |  |
| OIVAL | Non-host<br>Organism ID<br>Element Value | Char | * | Result<br>Qualifier | Value for the taxon in OIPARMCD/OIPARM for the organism | Req |
|  |  |  |  |  | identified by NHOID. |  |


## Page 442

OI – Assumptions
1. Non-host organisms include viruses and organisms such as pathogens or parasites, but also non-pathogenic
organisms such as normal intestinal flora. Non-host organism identifiers are not to be used for host species identification (e.g., for animals used in preclinical studies), nor should they be used to represent other, nontaxonomy characteristics of non-host species (e.g., drug susceptibility, growth rates).
2. NHOID is sponsor-defined, with the following constraints:
a. A unique NHOID must represent a unique identity as represented in its combination of OIPARMCD/OIVAL pairs. If 2 organisms share the same first 2 levels of taxonomy with regard to OIPARMCD/OIVAL, but 1 is identified to a third level and the other is not, they should be assigned 2 unique NHOIDs. b. Study sponsors should populate NHOID with intuitive name values based on either i. the name of the organism as reported by a lab or specified by the investigator, or ii. published references/databases where applicable and appropriate (e.g., when reference strain H77 is used in a HCV study, NHOID for this strain should be populated with “H77” or “HCV1a- H77”).
3. NHOID can be used in any domain where observations about these organisms are being represented,
allowing end users to determine what is known about the organism’s identity by merging on NHOID, or by otherwise referring to the OI domain.
4. OIPARMCD and OIPARM must represent parameters for the identification of non-host organisms with
regard to nomenclature only. a. Mostly, this will represent taxonomic ranks (i.e., species) as well as commonly used grouping terms (taxa that are not officially ranked, e.g., subtype, group, strain). b. They may also include other nomenclature terms that are less widely known but are used frequently for organism identification in a specific field of study (e.g., spoligotype in tuberculosis). c. They should be listed in the OI dataset in hierarchical order of least to most specific with increasing OISEQ values.
5. Variables not listed in the OI domain specification table should not be used in OI data sets.
OI – Examples Example 1 This example shows taxonomic identifiers for human immunodeficiency virus (HIV) and hepatitis C virus (HCV). NHOID is a unique non-host organism ID used to link findings on that organism in other datasets with details about its identification in OI. OIPARM shows the name of the individual taxa identified and OIVAL shows the experimentally determined values of those taxa. Rows 1-4: Show the taxonomy for the HIV organism given the NHOID of HIV1MC. This virus has been identified as HIV-1, Group M, Subtype C. Rows 5-8: Show the taxonomy for the HIV organism given the NHOID of HIV1MB, which was used as a reference. This virus has been identified as HIV-1, Group M, Subtype B. Rows 9-11: Show the taxonomy for the HCV organism given the NHOID of HCV2C. This virus has been identified as HCV 2c. Rows 12-14: Show the taxonomy for the HCV organism given the NHOID of H77. This virus is a known reference strain of HCV 1a. oi.xpt

| Row | STUDYID | DOMAIN | NHOID | OISEQ | OIPARMCD | OIPARM | OIVAL |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | STUDY123 | OI | HIV1MC | 1 | SPCIES | Species | HIV |
| 2 | STUDY123 | OI | HIV1MC | 2 | TYPE | Type | 1 |
| 3 | STUDY123 | OI | HIV1MC | 3 | GROUP | Group | M |
| 4 | STUDY123 | OI | HIV1MC | 4 | SUBTYP | Subtype | C |
| 5 | STUDY123 | OI | HIV1MB | 1 | SPCIES | Species | HIV |


## Page 443

| Row | STUDYID | DOMAIN | NHOID | OISEQ | OIPARMCD | OIPARM | OIVAL |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 6 | STUDY123 | OI | HIV1MB | 2 | TYPE | Type | 1 |
| 7 | STUDY123 | OI | HIV1MB | 3 | GROUP | Group | M |
| 8 | STUDY123 | OI | HIV1MB | 4 | SUBTYP | Subtype | B |
| 9 | STUDY123 | OI | HCV2C | 1 | SPCIES | Species | HCV |
| 10 | STUDY123 | OI | HCV2C | 2 | GENTYP | Genotype | 2 |
| 11 | STUDY123 | OI | HCV2C | 3 | SUBTYP | Subtype | C |
| 12 | STUDY123 | OI | H77 | 1 | SPCIES | Species | HCV |
| 13 | STUDY123 | OI | H77 | 2 | GENTYP | Genotype | 1 |
| 14 | STUDY123 | OI | H77 | 3 | SUBTYP | Subtype | A |

