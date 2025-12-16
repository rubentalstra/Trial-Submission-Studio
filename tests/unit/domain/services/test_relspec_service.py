import pandas as pd

from cdisc_transpiler.domain.services import RelspecService


def test_build_relspec_infers_from_refid_columns():
    service = RelspecService()

    lb = pd.DataFrame(
        {
            "USUBJID": ["01", "01", "02"],
            "LBREFID": ["S1", "S1", "S2"],
            "LBSPEC": ["BLOOD", "BLOOD", "SERUM"],
        }
    )

    df, config = service.build_relspec(
        domain_dataframes={"LB": lb}, study_id="STUDY001"
    )

    assert config.domain == "RELSPEC"
    assert config.study_id == "STUDY001"

    assert list(df.columns) == [
        "STUDYID",
        "USUBJID",
        "REFID",
        "SPEC",
        "PARENT",
        "LEVEL",
    ]
    assert len(df) == 2

    rows = {(r.USUBJID, r.REFID, r.SPEC) for r in df.itertuples(index=False)}
    assert ("01", "S1", "BLOOD") in rows
    assert ("02", "S2", "SERUM") in rows


def test_build_relspec_empty_when_no_refid_present():
    service = RelspecService()

    dm = pd.DataFrame({"USUBJID": ["01"], "AGE": [30]})
    df, _ = service.build_relspec(domain_dataframes={"DM": dm}, study_id="STUDY001")

    assert df.empty
    assert list(df.columns) == [
        "STUDYID",
        "USUBJID",
        "REFID",
        "SPEC",
        "PARENT",
        "LEVEL",
    ]
