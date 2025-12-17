import os

import pytest


@pytest.fixture(autouse=True)
def _default_country_env_for_tests(monkeypatch: pytest.MonkeyPatch) -> None:
    """Ensure required DM.COUNTRY can be populated during tests.

    The demo datasets shipped in mockdata don't include a source country field,
    but SDTMIG v3.4 marks DM.COUNTRY as Required.

    Production behavior remains unchanged (no default); tests provide an explicit
    value via DEFAULT_COUNTRY.
    """
    if os.getenv("DEFAULT_COUNTRY") is None:
        monkeypatch.setenv("DEFAULT_COUNTRY", "NLD")
