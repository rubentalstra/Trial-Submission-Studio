"""SAS file writing.

This module handles writing SAS programs to disk.
"""

from __future__ import annotations

from pathlib import Path

from .constants import SAS_FILE_ENCODING


def write_sas_file(code: str, path: str | Path) -> None:
    """Write SAS program code to a file.

    Creates parent directories if they don't exist.

    Args:
        code: SAS program code to write
        path: Path where the file should be written

    Example:
        >>> program = generate_sas_program(...)
        >>> write_sas_file(program, "output/sas/dm.sas")
    """
    file_path = Path(path)
    file_path.parent.mkdir(parents=True, exist_ok=True)

    with file_path.open("w", encoding=SAS_FILE_ENCODING) as handle:
        handle.write(code)
