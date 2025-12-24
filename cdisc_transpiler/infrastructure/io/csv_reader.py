from dataclasses import dataclass
from typing import TYPE_CHECKING, Any

import pandas as pd

from .exceptions import DataParseError, DataSourceNotFoundError

if TYPE_CHECKING:
    from pathlib import Path
HEADER_SAMPLE_ROWS = 2
HEADER_SPACE_THRESHOLD = 0.5
HEADER_CODE_THRESHOLD = 0.5


@dataclass(slots=True)
class CSVReadOptions:
    normalize_headers: bool = True
    strict_na_handling: bool = True
    dtype: Any = str
    encoding: str = "utf-8"
    detect_header_row: bool = True


class CSVReader:
    pass

    def read(self, path: Path, options: CSVReadOptions | None = None) -> pd.DataFrame:
        if options is None:
            options = CSVReadOptions()
        if not path.exists():
            raise DataSourceNotFoundError(f"File not found: {path}")
        if not path.is_file():
            raise DataSourceNotFoundError(f"Not a file: {path}")
        try:
            header_row = 0
            if options.detect_header_row:
                header_row = self._detect_header_row(path)
            df = pd.read_csv(
                path,
                header=header_row,
                dtype=options.dtype,
                keep_default_na=not options.strict_na_handling,
                na_values=[""] if options.strict_na_handling else None,
                encoding=options.encoding,
            )
        except FileNotFoundError as e:
            raise DataSourceNotFoundError(f"File not found: {path}") from e
        except pd.errors.ParserError as e:
            raise DataParseError(f"Failed to parse CSV {path}: {e}") from e
        except pd.errors.EmptyDataError as e:
            raise DataParseError(f"CSV file is empty: {path}") from e
        except UnicodeDecodeError as e:
            raise DataParseError(
                f"Encoding error reading {path}. Try a different encoding: {e}"
            ) from e
        except Exception as e:
            raise DataParseError(f"Unexpected error reading {path}: {e}") from e
        if df.shape[1] == 0:
            raise DataParseError(f"CSV file has no columns: {path}")
        if options.normalize_headers:
            df = self._normalize_headers(df)
        return df

    def _detect_header_row(self, path: Path) -> int:
        try:
            sample = pd.read_csv(path, nrows=HEADER_SAMPLE_ROWS, header=None)
            if sample.empty or len(sample) < HEADER_SAMPLE_ROWS:
                return 0
            first_row = sample.iloc[0].astype(str)
            second_row = sample.iloc[1].astype(str)
            first_has_spaces = (
                first_row.str.contains("\\s").mean() > HEADER_SPACE_THRESHOLD
            )
            second_is_codes = (
                second_row.str.match("^[A-Z][A-Za-z0-9_]*$").mean()
                > HEADER_CODE_THRESHOLD
            )
            if first_has_spaces and second_is_codes:
                return 1
        except Exception:
            return 0
        return 0

    def _normalize_headers(self, df: pd.DataFrame) -> pd.DataFrame:
        df.columns = [str(col).strip() for col in df.columns]
        return df
