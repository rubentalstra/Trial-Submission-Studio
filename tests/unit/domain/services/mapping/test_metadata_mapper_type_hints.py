import pandas as pd

from cdisc_transpiler.domain.entities.sdtm_domain import SDTMDomain, SDTMVariable
from cdisc_transpiler.domain.entities.study_metadata import SourceColumn, StudyMetadata
from cdisc_transpiler.domain.services.mapping.metadata_mapper import MetadataAwareMapper


def _domain_with_date_and_test_vars() -> SDTMDomain:
    return SDTMDomain(
        code="DA",
        description="Drug Accountability",
        class_name="Findings",
        structure="",
        label=None,
        variables=(
            SDTMVariable(
                name="DATEST",
                label="Name of Accountability Assessment",
                type="Char",
                length=200,
                core="Req",
            ),
            SDTMVariable(
                name="DATESTCD",
                label="Short Name of Accountability Assessment",
                type="Char",
                length=8,
                core="Req",
            ),
            SDTMVariable(
                name="DADTC",
                label="Date/Time of Accountability Assessment",
                type="Char",
                length=20,
                core="Perm",
            ),
        ),
    )


def test_date_typed_source_prefers_dtc_over_test() -> None:
    domain = _domain_with_date_and_test_vars()
    metadata = StudyMetadata(
        items={
            "EVENTDATE": SourceColumn(
                id="EventDate",
                label="Event date",
                data_type="date",
                mandatory=False,
                format_name=None,
                content_length=None,
            )
        }
    )

    frame = pd.DataFrame({"EventDate": ["2023-08-04"]})

    mapper = MetadataAwareMapper(domain, metadata, min_confidence=0.5)
    suggestions = mapper.suggest(frame)

    # We should map a date-like source column to a date-like SDTM variable.
    assert any(
        m.source_column == "EventDate" and m.target_variable == "DADTC"
        for m in suggestions.mappings
    )

    # And we should *not* incorrectly map it to --TEST/--TESTCD.
    assert not any(
        m.source_column == "EventDate" and m.target_variable in {"DATEST", "DATESTCD"}
        for m in suggestions.mappings
    )
