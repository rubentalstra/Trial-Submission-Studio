"""File Organization Service - Output file and directory management.

This service handles the creation and organization of output directories
and files for study processing, including XPT, Dataset-XML, SAS, and
Define-XML outputs.

Extracted from cli/commands/study.py as part of Phase 2 refactoring.
"""

from pathlib import Path

from ..infrastructure.sdtm_spec.registry import get_domain


def ensure_acrf_pdf(path: Path) -> None:
    """Create a minimal, valid PDF at path if one is not already present.

    This creates a placeholder Annotated CRF PDF file required by Define-XML.

    Args:
        path: Path where PDF should be created
    """
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


class FileOrganizationService:
    """Service for organizing output files and directories.

    This service manages the directory structure for study outputs,
    ensuring proper organization by file type and format.
    """

    def setup_output_directories(
        self,
        study_folder: Path,
        output_dir: Path | None,
        output_format: str,
        generate_sas: bool,
        generate_define: bool,
        acrf_href: str,
    ) -> tuple[Path, Path | None, Path | None, Path | None]:
        """Set up output directory structure for study processing.

        Creates the necessary subdirectories based on the requested output formats.

        Args:
            study_folder: The input study folder path
            output_dir: Custom output directory (if None, uses study_folder/output)
            output_format: Output format ("xpt", "xml", or "both")
            generate_sas: Whether to generate SAS programs
            generate_define: Whether to generate Define-XML
            acrf_href: Reference to annotated CRF PDF file

        Returns:
            Tuple of (output_dir, xpt_dir, xml_dir, sas_dir) where directories
            may be None if that format is not requested

        Examples:
            >>> service = FileOrganizationService()
            >>> out, xpt, xml, sas = service.setup_output_directories(
            ...     Path("study"), None, "both", True, True, "acrf.pdf"
            ... )
            >>> out
            Path("study/output")
            >>> xpt
            Path("study/output/xpt")
        """
        # Set output directory
        if output_dir is None:
            output_dir = study_folder / "output"

        # Determine subdirectories based on output format
        xpt_dir = output_dir / "xpt" if output_format in ("xpt", "both") else None
        xml_dir = (
            output_dir / "dataset-xml" if output_format in ("xml", "both") else None
        )
        sas_dir = output_dir / "sas" if generate_sas else None

        # Create directories
        output_dir.mkdir(parents=True, exist_ok=True)
        if xpt_dir:
            xpt_dir.mkdir(parents=True, exist_ok=True)
        if xml_dir:
            xml_dir.mkdir(parents=True, exist_ok=True)
        if sas_dir:
            sas_dir.mkdir(parents=True, exist_ok=True)

        # Ensure annotated CRF exists if generating Define-XML
        if generate_define:
            self._ensure_acrf_exists(output_dir / acrf_href)

        return output_dir, xpt_dir, xml_dir, sas_dir

    def _ensure_acrf_exists(self, acrf_path: Path) -> None:
        """Ensure annotated CRF PDF exists for Define-XML reference.

        Args:
            acrf_path: Path to the annotated CRF PDF file
        """
        ensure_acrf_pdf(acrf_path)

    def get_dataset_filename(self, domain_code: str) -> str:
        """Get the standardized lowercase filename for a domain.

        Args:
            domain_code: SDTM domain code (e.g., "DM", "SUPPAE")

        Returns:
            Lowercase filename without extension (e.g., "dm", "suppae")

        Examples:
            >>> service = FileOrganizationService()
            >>> service.get_dataset_filename("DM")
            'dm'
            >>> service.get_dataset_filename("SUPPAE")
            'suppae'
        """

        domain = get_domain(domain_code)
        base_filename = domain.resolved_dataset_name()
        return base_filename.lower()

    def get_output_paths(
        self,
        domain_code: str,
        output_format: str,
        xpt_dir: Path | None,
        xml_dir: Path | None,
        sas_dir: Path | None,
        generate_sas: bool,
    ) -> dict[str, Path | None]:
        """Get output file paths for a domain.

        Args:
            domain_code: SDTM domain code
            output_format: Output format ("xpt", "xml", or "both")
            xpt_dir: Directory for XPT files
            xml_dir: Directory for Dataset-XML files
            sas_dir: Directory for SAS programs
            generate_sas: Whether to generate SAS programs

        Returns:
            Dictionary with keys "xpt_path", "xml_path", "sas_path" (may be None)

        Examples:
            >>> service = FileOrganizationService()
            >>> paths = service.get_output_paths(
            ...     "DM", "both", Path("/out/xpt"), Path("/out/xml"), None, False
            ... )
            >>> paths["xpt_path"]
            Path("/out/xpt/dm.xpt")
        """
        disk_name = self.get_dataset_filename(domain_code)

        return {
            "xpt_path": xpt_dir / f"{disk_name}.xpt"
            if xpt_dir and output_format in ("xpt", "both")
            else None,
            "xml_path": xml_dir / f"{disk_name}.xml"
            if xml_dir and output_format in ("xml", "both")
            else None,
            "sas_path": sas_dir / f"{disk_name}.sas"
            if sas_dir and generate_sas
            else None,
        }
