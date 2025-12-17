import pandas as pd

from cdisc_transpiler.domain.services.transformers.date import DateTransformer


class TestDateTransformer:
    def test_compute_study_day_ignores_time_same_calendar_day(self):
        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "AESTDTC": ["2023-01-15T00:00"],
                "AESTDY": [pd.NA],
            }
        )

        DateTransformer.compute_study_day(
            df,
            "AESTDTC",
            "AESTDY",
            reference_starts={"001": "2023-01-15T23:59"},
            ref="RFSTDTC",
        )

        assert df.loc[0, "AESTDY"] == 1

    def test_compute_study_day_ignores_time_day_before(self):
        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "AESTDTC": ["2023-01-14T23:59"],
                "AESTDY": [pd.NA],
            }
        )

        DateTransformer.compute_study_day(
            df,
            "AESTDTC",
            "AESTDY",
            reference_starts={"001": "2023-01-15T00:01"},
            ref="RFSTDTC",
        )

        assert df.loc[0, "AESTDY"] == -1
