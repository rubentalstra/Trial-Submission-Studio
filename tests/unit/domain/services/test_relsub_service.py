import pandas as pd

from cdisc_transpiler.domain.services import RelsubService


def test_build_relsub_returns_empty_scaffold():
    service = RelsubService()
    df, config = service.build_relsub(domain_dataframes={}, study_id="STUDY001")

    assert config.domain == "RELSUB"
    assert config.study_id == "STUDY001"

    assert isinstance(df, pd.DataFrame)
    assert list(df.columns) == ["STUDYID", "USUBJID", "POOLID", "RSUBJID", "SREL"]
    assert df.empty
