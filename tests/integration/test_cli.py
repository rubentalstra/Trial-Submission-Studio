"""Integration tests for CLI commands.

This module contains end-to-end integration tests for all CLI commands,
testing the complete workflow from command invocation to file output.
"""

import pytest
from pathlib import Path
from click.testing import CliRunner

from cdisc_transpiler.cli import app


# Path to sample study data
MOCKDATA_DIR = Path(__file__).parent.parent.parent / "mockdata"
DEMO_GDISC = MOCKDATA_DIR / "DEMO_GDISC_20240903_072908"
DEMO_CF = MOCKDATA_DIR / "DEMO_CF1234_NL_20250120_104838"


@pytest.mark.integration
class TestStudyCommand:
    """Integration tests for the study command."""

    @pytest.fixture
    def runner(self):
        """Create a CLI runner."""
        return CliRunner()

    @pytest.fixture
    def study_folder(self):
        """Provide path to DEMO_GDISC study folder for tests that expect files."""
        if not DEMO_GDISC.exists():
            pytest.skip("DEMO_GDISC sample data not available")
        return DEMO_GDISC

    def test_study_help(self, runner):
        """Test that study help displays correctly."""
        result = runner.invoke(app, ["study", "--help"])

        assert result.exit_code == 0
        assert "Process an entire study folder" in result.output
        assert "STUDY_FOLDER" in result.output
        assert "--format" in result.output
        assert "--define-xml" in result.output
        assert "--sas" in result.output

    def test_study_with_default_options(self, runner, study_folder, tmp_path):
        """Test study command defaults (strict conformance gate enabled)."""
        output_dir = tmp_path / "output"

        result = runner.invoke(
            app,
            [
                "study",
                str(study_folder.absolute()),  # Use absolute path
                "--output-dir",
                str(output_dir),
                "--format",
                "xpt",  # Explicitly specify format
                "--no-define-xml",  # Skip Define-XML for faster test
                "--no-sas",  # Skip SAS for faster test
            ],
        )

        # With strict gating default enabled, DEMO_GDISC should pass conformance.
        assert result.exit_code == 0

        # Check output contains summary
        assert "Study Processing Summary" in result.output
        assert "conformance" in result.output.lower()

        # Ensure XPT files were generated
        xpt_dir = output_dir / "xpt"
        xpt_files = list(xpt_dir.glob("*.xpt")) if xpt_dir.exists() else []
        assert len(xpt_files) > 0, "XPT files should be generated when gating passes"

    def test_study_with_xpt_format(self, runner, study_folder, tmp_path):
        """Test study command with XPT format only."""
        output_dir = tmp_path / "output"

        result = runner.invoke(
            app,
            [
                "study",
                str(study_folder.absolute()),
                "--output-dir",
                str(output_dir),
                "--format",
                "xpt",
                "--no-fail-on-conformance-errors",
                "--no-define-xml",
                "--no-sas",
            ],
        )

        assert result.exit_code == 0

        # Check XPT directory exists
        xpt_dir = output_dir / "xpt"
        assert xpt_dir.exists()

        # Check that XML directory was not created
        xml_dir = output_dir / "dataset-xml"
        assert not xml_dir.exists(), (
            "XML directory should not be created with --format xpt"
        )

    def test_study_with_xml_format(self, runner, study_folder, tmp_path):
        """Test study command with XML format only."""
        output_dir = tmp_path / "output"

        result = runner.invoke(
            app,
            [
                "study",
                str(study_folder.absolute()),
                "--output-dir",
                str(output_dir),
                "--format",
                "xml",
                "--no-define-xml",
                "--no-sas",
            ],
        )

        assert result.exit_code == 0

        # Check XML directory exists
        xml_dir = output_dir / "dataset-xml"
        assert xml_dir.exists()

        # Check that XPT directory was not created
        xpt_dir = output_dir / "xpt"
        assert not xpt_dir.exists(), (
            "XPT directory should not be created with --format xml"
        )

    def test_study_with_both_formats(self, runner, study_folder, tmp_path):
        """Test study command with both XPT and XML formats."""
        output_dir = tmp_path / "output"

        result = runner.invoke(
            app,
            [
                "study",
                str(study_folder.absolute()),
                "--output-dir",
                str(output_dir),
                "--format",
                "both",
                "--no-fail-on-conformance-errors",
                "--no-define-xml",
                "--no-sas",
            ],
        )

        assert result.exit_code == 0

        # Check both directories exist
        xpt_dir = output_dir / "xpt"
        xml_dir = output_dir / "dataset-xml"
        assert xpt_dir.exists()
        assert xml_dir.exists()

    def test_study_with_sas_generation(self, runner, study_folder, tmp_path):
        """Test study command with SAS program generation."""
        output_dir = tmp_path / "output"

        result = runner.invoke(
            app,
            [
                "study",
                str(study_folder.absolute()),
                "--output-dir",
                str(output_dir),
                "--format",
                "xpt",
                "--sas",
                "--no-fail-on-conformance-errors",
                "--no-define-xml",
            ],
        )

        assert result.exit_code == 0

        # Check SAS directory exists
        sas_dir = output_dir / "sas"
        assert sas_dir.exists()

        # Check that SAS files were created
        sas_files = list(sas_dir.glob("*.sas"))
        assert len(sas_files) > 0, "Should generate at least one SAS file"

    def test_study_with_define_xml(self, runner, study_folder, tmp_path):
        """Test study command with Define-XML generation."""
        output_dir = tmp_path / "output"

        result = runner.invoke(
            app,
            [
                "study",
                str(study_folder.absolute()),
                "--output-dir",
                str(output_dir),
                "--format",
                "xpt",
                "--define-xml",
                "--no-sas",
                "--no-fail-on-conformance-errors",
            ],
        )

        assert result.exit_code == 0

        # Check Define-XML was created
        define_xml = output_dir / "define.xml"
        assert define_xml.exists(), "Define-XML should be created"

        # Check ACRF PDF placeholder
        acrf_pdf = output_dir / "acrf.pdf"
        assert acrf_pdf.exists(), "ACRF PDF placeholder should be created"

    def test_study_with_custom_study_id(self, runner, study_folder, tmp_path):
        """Test study command with custom study ID."""
        output_dir = tmp_path / "output"

        result = runner.invoke(
            app,
            [
                "study",
                str(study_folder.absolute()),
                "--output-dir",
                str(output_dir),
                "--study-id",
                "CUSTOM_STUDY",
                "--format",
                "xpt",
                "--no-fail-on-conformance-errors",
                "--no-define-xml",
                "--no-sas",
            ],
        )

        assert result.exit_code == 0
        # Note: Study ID doesn't appear directly in output, but files should be created
        assert (output_dir / "xpt").exists()

    def test_study_with_verbose_flag(self, runner, study_folder, tmp_path):
        """Test study command with verbose output."""
        output_dir = tmp_path / "output"

        result = runner.invoke(
            app,
            [
                "study",
                str(study_folder.absolute()),
                "--output-dir",
                str(output_dir),
                "--format",
                "xpt",
                "--no-fail-on-conformance-errors",
                "--no-define-xml",
                "--no-sas",
                "-v",
            ],
        )

        assert result.exit_code == 0
        # Verbose mode should show more details
        assert "Found" in result.output or "CSV files" in result.output

    def test_study_with_very_verbose_flag(self, runner, study_folder, tmp_path):
        """Test study command with very verbose output."""
        output_dir = tmp_path / "output"

        result = runner.invoke(
            app,
            [
                "study",
                str(study_folder.absolute()),
                "--output-dir",
                str(output_dir),
                "--format",
                "xpt",
                "--no-fail-on-conformance-errors",
                "--no-define-xml",
                "--no-sas",
                "-vv",
            ],
        )

        assert result.exit_code == 0
        # Very verbose mode should show even more details

    def test_study_missing_folder(self, runner):
        """Test study command with non-existent folder."""
        result = runner.invoke(app, ["study", "/nonexistent/folder"])

        # Click should fail with non-zero exit code
        assert result.exit_code != 0
        # Error message should indicate the path doesn't exist
        assert "does not exist" in result.output.lower() or "Error" in result.output

    def test_study_invalid_format(self, runner, study_folder, tmp_path):
        """Test study command with invalid format option."""
        output_dir = tmp_path / "output"

        result = runner.invoke(
            app,
            [
                "study",
                str(study_folder.absolute()),
                "--output-dir",
                str(output_dir),
                "--format",
                "invalid",
            ],
        )

        # Should fail with error about invalid choice
        assert result.exit_code != 0
        assert "Invalid value" in result.output or "invalid" in result.output.lower()

    def test_study_output_summary_format(self, runner, study_folder, tmp_path):
        """Test that study output has correct summary table format."""
        output_dir = tmp_path / "output"

        result = runner.invoke(
            app,
            [
                "study",
                str(study_folder.absolute()),
                "--output-dir",
                str(output_dir),
                "--format",
                "xpt",
                "--no-fail-on-conformance-errors",
                "--no-define-xml",
                "--no-sas",
            ],
        )

        assert result.exit_code == 0

        # Check for summary table elements
        assert "Study Processing Summary" in result.output
        assert "Domain" in result.output
        assert "Records" in result.output
        assert "Total" in result.output
        assert "domains processed successfully" in result.output


@pytest.mark.integration
class TestDomainsCommand:
    """Integration tests for the domains command."""

    @pytest.fixture
    def runner(self):
        """Create a CLI runner."""
        return CliRunner()

    def test_domains_list(self, runner):
        """Test that domains command lists all supported domains."""
        result = runner.invoke(app, ["domains"])

        assert result.exit_code == 0
        assert "Supported SDTM Domains" in result.output

        # Check for some known domains
        assert "DM" in result.output
        assert "AE" in result.output
        assert "LB" in result.output
        assert "VS" in result.output

    def test_domains_help(self, runner):
        """Test that domains help displays correctly."""
        result = runner.invoke(app, ["domains", "--help"])

        assert result.exit_code == 0
        assert "List all supported SDTM domains" in result.output


@pytest.mark.integration
class TestCLIGeneral:
    """General CLI integration tests."""

    @pytest.fixture
    def runner(self):
        """Create a CLI runner."""
        return CliRunner()

    def test_app_help(self, runner):
        """Test that main app help displays correctly."""
        result = runner.invoke(app, ["--help"])

        assert result.exit_code == 0
        assert "CDISC Transpiler CLI" in result.output
        assert "study" in result.output
        assert "domains" in result.output

    def test_app_no_command(self, runner):
        """Test running app without any command."""
        result = runner.invoke(app, [])

        # Click returns exit code 2 for missing required arguments
        # Should display help/usage when no command is given
        assert "CDISC Transpiler CLI" in result.output or "Usage:" in result.output

    def test_invalid_command(self, runner):
        """Test running app with invalid command."""
        result = runner.invoke(app, ["invalid-command"])

        assert result.exit_code != 0
        assert "Error" in result.output or "No such command" in result.output


@pytest.mark.integration
@pytest.mark.slow
class TestStudyCommandWithGDISC:
    """Integration tests for study command with larger DEMO_GDISC dataset.

    These tests are marked as slow since they process more data.
    """

    @pytest.fixture
    def runner(self):
        """Create a CLI runner."""
        return CliRunner()

    @pytest.fixture
    def study_folder(self):
        """Provide path to DEMO_GDISC study folder."""
        if not DEMO_GDISC.exists():
            pytest.skip("DEMO_GDISC sample data not available")
        return DEMO_GDISC

    def test_study_full_processing(self, runner, study_folder, tmp_path):
        """Test full study processing with DEMO_GDISC dataset."""
        output_dir = tmp_path / "output"

        result = runner.invoke(
            app,
            [
                "study",
                str(study_folder.absolute()),
                "--output-dir",
                str(output_dir),
                "--format",
                "xpt",
                "--no-fail-on-conformance-errors",
                "--no-define-xml",
                "--no-sas",
            ],
        )

        assert result.exit_code == 0

        # DEMO_GDISC should have multiple domains
        xpt_dir = output_dir / "xpt"
        xpt_files = list(xpt_dir.glob("*.xpt"))

        # Should have at least 10 domains
        assert len(xpt_files) >= 10, (
            f"Expected at least 10 XPT files, got {len(xpt_files)}"
        )

        # Check for key domains
        for domain_file in ["dm.xpt", "ae.xpt", "vs.xpt", "lb.xpt"]:
            assert (xpt_dir / domain_file).exists(), f"{domain_file} should exist"
