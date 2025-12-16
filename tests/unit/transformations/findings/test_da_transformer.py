import pandas as pd

from cdisc_transpiler.transformations.base import TransformationContext
from cdisc_transpiler.transformations.findings.da_transformer import DATransformer


def test_da_transformer_reshapes_retamt_and_disamt_groups() -> None:
    df = pd.DataFrame(
        {
            "SubjectId": ["KIEM-01"],
            "EventName": ["Visit 2"],
            "EventSeq": [2],
            "EventDate": ["2023-08-04"],
            "RETAMT_DAORRES": [50],
            "RETAMT_DADAT": ["2023-08-04"],
            "DISAMT_DAORRES": [100],
            "DISAMT_DADAT": ["2023-08-01"],
        }
    )

    def _label_getter(domain: str, testcd: str) -> str:
        assert domain == "DA"
        return {"RETAMT": "Returned Amount", "DISPAMT": "Dispensed Amount"}.get(
            testcd, testcd
        )

    def _normalizer(domain: str, source: str) -> str | None:
        assert domain == "DA"
        return {"RETAMT": "RETAMT", "DISAMT": "DISPAMT"}.get(source)

    transformer = DATransformer(
        test_code_normalizer=_normalizer, test_label_getter=_label_getter
    )
    result = transformer.transform(
        df, TransformationContext(domain="DA", study_id="DEMO")
    )

    assert result.success
    assert result.applied

    out = result.data
    assert len(out) == 2
    assert set(out["DATESTCD"].tolist()) == {"RETAMT", "DISPAMT"}

    # DATEST is populated (required variable)
    assert out.loc[out["DATESTCD"] == "RETAMT", "DATEST"].iloc[0] == "Returned Amount"
    assert out.loc[out["DATESTCD"] == "DISPAMT", "DATEST"].iloc[0] == "Dispensed Amount"

    # DADTC comes from the per-group date columns
    assert out.loc[out["DATESTCD"] == "RETAMT", "DADTC"].iloc[0] == "2023-08-04"
    assert out.loc[out["DATESTCD"] == "DISPAMT", "DADTC"].iloc[0] == "2023-08-01"

    # DAORRES comes from the per-group result columns
    assert out.loc[out["DATESTCD"] == "RETAMT", "DAORRES"].iloc[0] == 50
    assert out.loc[out["DATESTCD"] == "DISPAMT", "DAORRES"].iloc[0] == 100
