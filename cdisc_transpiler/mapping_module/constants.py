"""Constants and patterns for SDTM mapping inference.

This module contains the pattern dictionaries used for inferring SDTM variable
names from source column names.

SDTM Reference:
    Variable naming follows SDTM v2.0 and SDTMIG v3.4 standards.
    Variables use a domain prefix followed by a standardized suffix
    (e.g., AESTDTC = AE + STDTC for Adverse Event Start Date/Time).

    General Observation Classes:
    - Interventions: --TRT, --DOSE, --DOSU, --ROUTE, etc.
    - Events: --TERM, --DECOD, --BODSYS, --SEV, etc.
    - Findings: --TESTCD, --TEST, --ORRES, --STRESC, etc.
"""

from __future__ import annotations

# Patterns for inferring SDTM variable names from source column names
# These patterns map normalized source column names to SDTM variables
SDTM_INFERENCE_PATTERNS: dict[str, dict[str, list[str]]] = {
    # Global patterns (apply to all domains)
    # These are Identifier variables that don't follow the --VAR pattern
    "_GLOBAL": {
        # Subject/Study Identifiers (SDTM v2.0 Section 3.1)
        "STUDYID": ["STUDYID", "STUDY", "STUDYIDENTIFIER", "PROTOCOL"],
        "USUBJID": [
            "SUBJECTID",
            "SUBJECTIDENTIFIER",
            "PATIENTID",
            "SUBJECT",
            "PATIENTIDENTIFIER",
            "UNIQUESUBJECTID",
        ],
        "POOLID": ["POOLID", "POOLIDENTIFIER"],
        "SPDEVID": ["SPDEVID", "DEVICEID", "DEVICEIDENTIFIER", "SPONSORDEVICEID"],
        # Demographics variables (SDTMIG v3.4 DM domain)
        "SEX": ["SEX", "GENDER"],
        "AGE": ["AGE", "SUBJECTAGE"],
        "AGEU": ["AGEU", "AGEUNIT", "AGEUNITS"],
        "RACE": ["RACE"],
        "ETHNIC": ["ETHNIC", "ETHNICITY"],
        "ARMCD": ["ARMCD", "ARMCODE", "TREATMENTARMCODE"],
        "ARM": ["ARM", "TREATMENTARM", "PLANNEDARM"],
        "ACTARMCD": ["ACTARMCD", "ACTUALARMCODE"],
        "ACTARM": ["ACTARM", "ACTUALARM"],
        "COUNTRY": ["COUNTRY", "COUNTRYCD", "COUNTRYCODE"],
        "SITEID": ["SITEID", "SITECODE", "SITE", "SITEIDENTIFIER"],
        "INVID": ["INVID", "INVESTIGATORID", "INVESTIGATOR"],
        "INVNAM": ["INVNAM", "INVESTIGATORNAME"],
        # Reference dates (SDTM v2.0 Section 3.1.3)
        "RFSTDTC": ["ICDAT", "INFORMEDCONSENTDATE", "RFSTDTC", "REFERENCESTART"],
        "RFENDTC": ["RFENDTC", "REFERENCEEND"],
        "RFXSTDTC": ["RFXSTDTC", "FIRSTEXPOSURE", "FIRSTDOSEDATE"],
        "RFXENDTC": ["RFXENDTC", "LASTEXPOSURE", "LASTDOSEDATE"],
        "RFICDTC": ["RFICDTC", "INFORMEDCONSENTDATETIME"],
        "RFPENDTC": ["RFPENDTC", "PERMANENTDISCONTINUATION"],
        "DTHDTC": ["DTHDTC", "DEATHDATE", "DEATHDATETIME"],
        "DTHFL": ["DTHFL", "DEATHFLAG", "SUBJECTDIED"],
        "BRTHDTC": ["BRTHDTC", "BIRTHDATE", "DOB", "DATEOFBIRTH"],
        # Timing variables (SDTM v2.0 Section 3.3)
        "EPOCH": ["EPOCH", "VISITEPOCH", "STUDYEPOCH"],
        "TAETORD": ["TAETORD", "ELEMENTORDER", "PLANNEDORDER"],
        "VISITNUM": ["VISITNUM", "VISITNUMBER", "VISITSEQUENCE"],
        "VISIT": ["VISIT", "VISITNAME", "EVENTNAME", "VISITLABEL"],
        "VISITDY": ["VISITDY", "PLANNEDSTUDYDAYOFVISIT", "PLANNEDVISITDAY"],
    },
    # Domain-specific patterns (--VARIABLE becomes DOMAIN + VARIABLE)
    # Based on SDTM v2.0 General Observation Classes
    "_DOMAIN_SUFFIXES": {
        # Identifier Variables (SDTM v2.0 Section 3.1)
        "SEQ": ["SEQ", "EVENTSEQ", "SEQUENCENUMBER", "EVENTSEQUENCENUMBER", "RECORDSEQ"],
        "GRPID": ["GRPID", "GROUPID", "GROUP"],
        "REFID": ["REFID", "REFERENCEID", "REFIDENTIFIER", "SPECIMENID"],
        "SPID": ["SPID", "SPONSORID", "SPONSORIDENTIFIER"],
        "LNKID": ["LNKID", "LINKID", "LINKIDENTIFIER", "LINK"],
        "LNKGRP": ["LNKGRP", "LINKGROUP", "LINKGRP"],
        # Topic Variables - Interventions Class (SDTM v2.0 Section 3.2.1)
        "TRT": ["TRT", "TREATMENT", "MEDICATION", "DRUG", "NAMEOFTREATMENT"],
        "MODIFY": ["MODIFY", "MODIFIEDTERM", "MODIFIEDTREATMENT"],
        "DECOD": ["DECOD", "DECODE", "DICTIONARYTERM", "STANDARDTERM", "STANDARDIZEDTERM"],
        # Topic Variables - Events Class (SDTM v2.0 Section 3.2.2)
        "TERM": ["TERM", "REPORTEDTERM", "VERBATIM", "COLLECTEDNAME"],
        "LLT": ["LLT", "LOWESTLEVELTERM"],
        "LLTCD": ["LLTCD", "LOWESTLEVELTERMCODE"],
        "PTCD": ["PTCD", "PREFERREDTERMCODE"],
        "HLT": ["HLT", "HIGHLEVELTERM"],
        "HLTCD": ["HLTCD", "HIGHLEVELTERMCODE"],
        "HLGT": ["HLGT", "HIGHLEVELGROUPTERM"],
        "HLGTCD": ["HLGTCD", "HIGHLEVELGROUPTERMCODE"],
        "SOC": ["SOC", "SYSTEMORGANCLASS", "PRIMARYSOC"],
        "SOCCD": ["SOCCD", "SYSTEMORGANCLASSCODE"],
        # Topic Variables - Findings Class (SDTM v2.0 Section 3.2.3)
        "TESTCD": ["TESTCD", "TESTCODE", "SHORTNAME"],
        "TEST": ["TEST", "TESTNAME", "MEASUREMENTNAME"],
        # Qualifier Variables - Grouping (SDTM v2.0 Section 3.2.4)
        "CAT": ["CAT", "CATEGORY"],
        "SCAT": ["SCAT", "SUBCATEGORY"],
        "BODSYS": ["BODSYS", "BODYSYSTEM", "ORGANCLASS", "SYSTEMORANCLASS"],
        # Qualifier Variables - Result (SDTM v2.0 Section 3.2.5)
        "ORRES": ["ORRES", "RESULT", "ORIGINALRESULT", "VALUE", "FINDINGORIGINAL"],
        "ORRESU": ["ORRESU", "UNIT", "UNITS", "ORIGINALUNIT", "ORIGINALUNITS"],
        "ORNRLO": ["ORNRLO", "NORMALLO", "NORMALRANGELOWER"],
        "ORNRHI": ["ORNRHI", "NORMALHI", "NORMALRANGEUPPER"],
        "STRESC": ["STRESC", "STANDARDRESULT", "STANDARDIZEDRESULT"],
        "STRESN": ["STRESN", "NUMERICRESULT", "NUMERICVALUE"],
        "STRESU": ["STRESU", "STANDARDUNIT", "STANDARDUNITS"],
        "STNRLO": ["STNRLO", "STANDARDNORMALLO"],
        "STNRHI": ["STNRHI", "STANDARDNORMALHI"],
        "NRIND": ["NRIND", "NORMALRANGEINDICATOR", "REFRANGEINDICATOR"],
        "RESCAT": ["RESCAT", "RESULTCATEGORY"],
        # Qualifier Variables - Record (SDTM v2.0 Section 3.2.6)
        "STAT": ["STAT", "STATUS", "COMPLETIONSTATUS"],
        "REASND": ["REASND", "REASONNOTDONE", "REASON"],
        "PRESP": ["PRESP", "PRESPECIFIED"],
        "OCCUR": ["OCCUR", "OCCURRENCE", "OCCURRENCEINDICATOR"],
        # Qualifier Variables - Intervention-specific
        "DOSE": ["DOSE", "DOSEAMOUNT", "DOSEVALUE"],
        "DOSTXT": ["DOSTXT", "DOSEDESCRIPTION", "DOSETEXT"],
        "DOSU": ["DOSU", "DOSEUNIT", "DOSEUNITS"],
        "DOSFRM": ["DOSFRM", "DOSEFORM", "DOSAGEFORM"],
        "DOSFRQ": ["DOSFRQ", "DOSINGFREQUENCY", "FREQUENCY"],
        "DOSTOT": ["DOSTOT", "TOTALDAILYDOSE"],
        "DOSRGM": ["DOSRGM", "DOSEREGIMEN"],
        "ROUTE": ["ROUTE", "ADMINISTRATIONROUTE", "ROUTEOFADMINISTRATION"],
        "LOT": ["LOT", "LOTNUMBER", "BATCHNUMBER"],
        "LOC": ["LOC", "LOCATION", "SITE", "ADMINISTRATIONLOCATION"],
        "METHOD": ["METHOD", "COLLECTIONMETHOD", "ADMINISTRATIONMETHOD"],
        "LAT": ["LAT", "LATERALITY", "SIDE"],
        "DIR": ["DIR", "DIRECTION", "DIRECTIONALITY"],
        "INDC": ["INDC", "INDICATION"],
        # Qualifier Variables - Event-specific
        "SER": ["SER", "SERIOUS", "SERIOUSEVENT"],
        "SEV": ["SEV", "SEVERITY", "INTENSITY"],
        "REL": ["REL", "RELATIONSHIP", "CAUSALITY", "RELATEDTOSTUDY"],
        "RELNST": ["RELNST", "RELATIONSHIPTONONTREATMENT"],
        "PATT": ["PATT", "PATTERN", "EVENTPATTERN"],
        "OUT": ["OUT", "OUTCOME", "EVENTOUTCOME"],
        "ACN": ["ACN", "ACTION", "ACTIONTAKEN", "ACTIONWITHTREATMENT"],
        "ACNOTH": ["ACNOTH", "OTHERACTION", "OTHERACTIONTAKEN"],
        "CONTRT": ["CONTRT", "CONCOMITANTTREATMENT", "ADDITIONALTREATMENT"],
        "TOXGR": ["TOXGR", "TOXICITYGRADE", "GRADE"],
        "TOX": ["TOX", "TOXICITY"],
        # Qualifier Variables - Findings-specific
        "POS": ["POS", "POSITION", "SUBJECTPOSITION"],
        "SPEC": ["SPEC", "SPECIMEN", "SPECIMENTYPE"],
        "FAST": ["FAST", "FASTING", "FASTINGSTATUS"],
        "EVAL": ["EVAL", "EVALUATOR"],
        "EVALID": ["EVALID", "EVALUATORID", "EVALUATORIDENTIFIER"],
        # Timing Variables (SDTM v2.0 Section 3.3)
        "STDTC": ["STDTC", "STDAT", "STARTDATE", "STARTDATETIME"],
        "ENDTC": ["ENDTC", "ENDAT", "ENDDATE", "ENDDATETIME"],
        "DTC": ["DTC", "DAT", "DATE", "DATETIME", "COLLECTIONDATE"],
        "RFTDTC": ["RFTDTC", "REFERENCEDTC", "REFERENCETIMEPOINT"],
        "DY": ["DY", "STUDYDAY", "DAY"],
        "STDY": ["STDY", "STARTDY", "STUDYDAYSTART", "STARTSTUDYDAY"],
        "ENDY": ["ENDY", "ENDDY", "STUDYDAYEND", "ENDSTUDYDAY"],
        "DUR": ["DUR", "DURATION", "COLLECTEDDURATION"],
        "ELTM": ["ELTM", "ELAPSEDTIME", "ELAPSED"],
        "TPT": ["TPT", "TIMEPOINT", "PLANNEDTIMEPOINT", "PLANNEDTIMEPOINTNAME"],
        "TPTNUM": ["TPTNUM", "TIMEPOINTNUM", "TPTNUMBER", "TIMEPOINTNUMBER"],
        "TPTREF": ["TPTREF", "TIMEPOINTREF", "REFERENCEPOINT", "TIMEPOINTREFERENCE"],
        "STRTPT": ["STRTPT", "STARTTPT", "STARTREFERENCE", "STARTRELATIVETOTIMEPOINT"],
        "STTPT": ["STTPT", "STARTTP", "STARTTIMEPOINT", "STARTREFERENCETIMEPOINT"],
        "ENRTPT": ["ENRTPT", "ENDRTPT", "ENDREFERENCEPOINT", "ENDRELATIVETOTIMEPOINT"],
        "ENTPT": ["ENTPT", "ENDTPT", "ENDTIMEPOINT", "ENDREFERENCETIMEPOINT"],
        "ENRF": ["ENRF", "ENDREF", "ENDREFERENCE", "ENDRELATIVETOREFERENCEPERIOD"],
        "STRF": ["STRF", "STARTREF", "STARTREFERENCE", "STARTRELATIVETOREFERENCEPERIOD"],
        "EVLINT": ["EVLINT", "EVALUATIONINTERVAL"],
        "EVINTX": ["EVINTX", "EVALUATIONINTERVALTEXT"],
    },
}
