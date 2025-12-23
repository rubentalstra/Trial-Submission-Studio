"""Domain processor for Trial Elements (TE) domain."""

from typing import TYPE_CHECKING, override

if TYPE_CHECKING:
    import pandas as pd

from .base import BaseDomainProcessor


class TEProcessor(BaseDomainProcessor):
    """Trial Elements domain processor.

    Handles domain-specific processing for the TE domain.
    """

    @override
    def process(self, frame: pd.DataFrame) -> None:
        """Process TE domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        self._drop_placeholder_rows(frame)

        # Do not synthesize TE records. Only normalize existing values.
        for col in ("STUDYID", "DOMAIN", "ETCD", "ELEMENT", "TESTRL", "TEENRL"):
            if col in frame.columns:
                frame.loc[:, col] = frame[col].astype("string").fillna("").str.strip()
