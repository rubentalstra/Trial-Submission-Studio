"""Infrastructure adapter for output directory preparation.

This adapter performs filesystem I/O needed to create the output layout for a
study run (directories and a placeholder ACRF PDF used by Define-XML).

It implements the application port OutputPreparerPort.
"""

from __future__ import annotations

from pathlib import Path

from ...application.ports import OutputPreparerPort


def _ensure_acrf_pdf(path: Path) -> None:
    """Create a minimal, valid PDF at path if one is not already present."""
    if path.exists():
        return

    path.parent.mkdir(parents=True, exist_ok=True)

    obj_bodies: dict[int, str] = {
        1: "<< /Type /Catalog /Pages 2 0 R >>",
        2: "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        3: (
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] "
            "/Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>"
        ),
    }
    stream_text = "Annotated CRF placeholder"
    stream_content = f"BT /F1 12 Tf 72 720 Td ({stream_text}) Tj ET".encode("latin-1")
    obj_bodies[4] = (
        f"<< /Length {len(stream_content)} >>\nstream\n"
        + stream_content.decode("latin-1")
        + "\nendstream"
    )
    obj_bodies[5] = "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>"

    parts: list[str] = ["%PDF-1.4\n"]
    offsets: dict[int, int] = {}
    for obj_num in sorted(obj_bodies):
        offsets[obj_num] = sum(len(p.encode("latin-1")) for p in parts)
        parts.append(f"{obj_num} 0 obj\n{obj_bodies[obj_num]}\nendobj\n")

    xref_start = sum(len(p.encode("latin-1")) for p in parts)
    size = max(obj_bodies) + 1
    xref_lines = ["xref", f"0 {size}", "0000000000 65535 f "]
    for i in range(1, size):
        offset = offsets.get(i, 0)
        xref_lines.append(f"{offset:010d} 00000 n ")
    xref_section = "\n".join(xref_lines) + "\n"
    trailer = (
        f"trailer\n<< /Size {size} /Root 1 0 R >>\nstartxref\n{xref_start}\n%%EOF\n"
    )
    parts.append(xref_section)
    parts.append(trailer)

    pdf_bytes = "".join(parts).encode("latin-1")
    path.write_bytes(pdf_bytes)


class OutputPreparer(OutputPreparerPort):
    """Filesystem-based output preparation."""

    def prepare(
        self,
        *,
        output_dir: Path,
        output_formats: set[str],
        generate_sas: bool,
        generate_define_xml: bool,
    ) -> None:
        output_dir.mkdir(parents=True, exist_ok=True)

        if "xpt" in output_formats:
            (output_dir / "xpt").mkdir(parents=True, exist_ok=True)

        if "xml" in output_formats:
            (output_dir / "dataset-xml").mkdir(parents=True, exist_ok=True)

        if generate_sas:
            (output_dir / "sas").mkdir(parents=True, exist_ok=True)

        if generate_define_xml:
            _ensure_acrf_pdf(output_dir / "acrf.pdf")

    def ensure_dir(self, path: Path) -> None:
        path.mkdir(parents=True, exist_ok=True)
