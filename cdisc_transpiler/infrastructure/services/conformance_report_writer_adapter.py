"""Infrastructure adapter for writing conformance reports.

This adapter implements the application-level ConformanceReportWriterPort and
performs filesystem I/O (JSON persistence).
"""

from collections.abc import Iterable
from datetime import UTC, datetime
import json
from pathlib import Path
from typing import override

from ...application.ports.services import ConformanceReportWriterPort
from ...domain.services.sdtm_conformance_checker import ConformanceReport


class ConformanceReportWriterAdapter(ConformanceReportWriterPort):
    @override
    def write_json(
        self,
        *,
        output_dir: Path,
        study_id: str,
        reports: Iterable[ConformanceReport],
        filename: str = "conformance_report.json",
    ) -> Path:
        output_dir.mkdir(parents=True, exist_ok=True)
        output_path = output_dir / filename

        now = datetime.now(UTC).isoformat()
        payload = {
            "schema": "cdisc-transpiler.conformance-report",
            "schema_version": 1,
            "generated_at": now,
            "study_id": study_id,
            "reports": [report.to_dict() for report in reports],
        }

        output_path.write_text(
            json.dumps(payload, indent=2, sort_keys=True, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )
        return output_path
