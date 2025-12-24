from pathlib import Path

import pandas as pd
import pyreadstat

from ...domain.entities.study_metadata import StudyMetadata
from ..io.csv_reader import CSVReader, CSVReadOptions
from ..io.exceptions import DataParseError, DataSourceNotFoundError
from .study_metadata_loader import load_study_metadata as _load_metadata


class StudyDataRepository:
    pass

    def __init__(self, csv_reader: CSVReader | None = None) -> None:
        super().__init__()
        self._csv_reader = csv_reader or CSVReader()

    def read_dataset(self, file_path: str | Path) -> pd.DataFrame:
        path = Path(file_path)
        if not path.exists():
            raise DataSourceNotFoundError(f"File not found: {path}")
        if not path.is_file():
            raise DataSourceNotFoundError(f"Not a file: {path}")
        ext = path.suffix.lower()
        if ext in (".csv", ".tsv", ".txt"):
            return self._read_csv(path)
        if ext in (".xls", ".xlsx"):
            return self._read_excel(path)
        if ext == ".sas7bdat":
            return self._read_sas(path)
        supported = ".csv, .tsv, .txt, .xls, .xlsx, .sas7bdat"
        raise DataParseError(f"Unsupported format '{ext}'. Supported: {supported}")

    def load_study_metadata(self, study_folder: Path) -> StudyMetadata:
        if not study_folder.exists():
            return StudyMetadata(source_path=study_folder)
        return _load_metadata(study_folder)

    def list_data_files(self, folder: Path, pattern: str = "*.csv") -> list[Path]:
        if not folder.exists() or not folder.is_dir():
            return []
        return sorted(folder.glob(pattern))

    def _read_csv(self, path: Path) -> pd.DataFrame:
        options = CSVReadOptions(
            normalize_headers=True, strict_na_handling=True, detect_header_row=True
        )
        return self._csv_reader.read(path, options)

    def _read_excel(self, path: Path) -> pd.DataFrame:
        try:
            return pd.read_excel(path)
        except Exception as e:
            raise DataParseError(f"Failed to read Excel file {path}: {e}") from e

    def _read_sas(self, path: Path) -> pd.DataFrame:
        if pyreadstat is None:
            raise DataParseError(
                "pyreadstat is required to read SAS files (optional dependency). "
                + "Install with: pip install pyreadstat"
            )
        try:
            frame, _meta = pyreadstat.read_sas7bdat(str(path))
            return frame
        except Exception as e:
            raise DataParseError(f"Failed to read SAS file {path}: {e}") from e
