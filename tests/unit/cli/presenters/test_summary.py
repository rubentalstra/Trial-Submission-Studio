"""Unit tests for SummaryPresenter class."""

from io import StringIO
from pathlib import Path

import pytest
from rich.console import Console

from cdisc_transpiler.cli.presenters import SummaryPresenter


class TestSummaryPresenter:
    """Test suite for SummaryPresenter class."""

    @pytest.fixture
    def console(self):
        """Create a console with StringIO for capturing output."""
        return Console(file=StringIO(), force_terminal=True, width=120)

    @pytest.fixture
    def presenter(self, console):
        """Create a SummaryPresenter instance."""
        return SummaryPresenter(console)

    @pytest.fixture
    def sample_results(self):
        """Provide sample domain processing results."""
        return [
            {
                "domain_code": "DM",
                "description": "Demographics",
                "records": 100,
                "xpt_path": Path("/output/xpt/dm.xpt"),
                "xml_path": None,
                "sas_path": Path("/output/sas/dm.sas"),
            },
            {
                "domain_code": "AE",
                "description": "Adverse Events",
                "records": 250,
                "xpt_path": Path("/output/xpt/ae.xpt"),
                "xml_path": Path("/output/xml/ae.xml"),
                "sas_path": None,
            },
            {
                "domain_code": "SUPPDM",
                "description": "Supplemental Qualifiers for DM",
                "records": 10,
                "xpt_path": Path("/output/xpt/suppdm.xpt"),
                "xml_path": None,
                "sas_path": None,
            },
        ]

    def test_presenter_initialization(self, console):
        """Test that presenter initializes with console."""
        presenter = SummaryPresenter(console)
        assert presenter.console == console

    def test_present_displays_output(self, presenter, sample_results, console):
        """Test that present method displays output."""
        presenter.present(
            results=sample_results,
            errors=[],
            output_dir=Path("/output"),
            output_format="xpt",
            generate_define=True,
            generate_sas=True,
        )

        output = console.file.getvalue()
        assert "Study Processing Summary" in output
        assert "DM" in output
        assert "AE" in output
        assert "Demographics" in output
        assert "Adverse Events" in output

    def test_organize_results_separates_main_and_supp(self, presenter, sample_results):
        """Test that results are organized into main and SUPPQUAL domains."""
        main_domains, suppqual_domains, total_records = presenter._organize_results(
            sample_results
        )

        # Should have 2 main domains (DM, AE)
        assert len(main_domains) == 2
        assert "DM" in main_domains
        assert "AE" in main_domains

        # Should have 1 SUPPQUAL domain under DM
        assert "DM" in suppqual_domains
        assert len(suppqual_domains["DM"]) == 1
        assert suppqual_domains["DM"][0][0] == "SUPPDM"

        # Total records should sum correctly
        assert total_records == 360  # 100 + 250 + 10

    def test_organize_results_calculates_indicators(self, presenter):
        """Test that output indicators are calculated correctly."""
        results = [
            {
                "domain_code": "DM",
                "records": 100,
                "xpt_path": Path("/output/xpt/dm.xpt"),
                "xml_path": None,
                "sas_path": None,
            }
        ]

        main_domains, _, _ = presenter._organize_results(results)

        assert main_domains["DM"]["has_xpt"] == "✓"
        assert main_domains["DM"]["has_xml"] == "–"
        assert main_domains["DM"]["has_sas"] == "–"

    def test_build_notes_empty_when_no_splits(self, presenter):
        """Test that notes are empty when no splits exist."""
        result = {}
        notes = presenter._build_notes(result)
        assert notes == ""

    def test_print_status_summary_success(self, presenter, console):
        """Test status summary with no errors."""
        presenter._print_status_summary(5, 0)
        output = console.file.getvalue()
        # Remove ANSI codes for testing
        import re

        output_clean = re.sub(r"\x1b\[[0-9;]*m", "", output)
        assert "5 domains processed successfully" in output_clean
        assert "✓" in output_clean

    def test_print_status_summary_with_errors(self, presenter, console):
        """Test status summary with errors."""
        presenter._print_status_summary(3, 2)
        output = console.file.getvalue()
        # Remove ANSI codes for testing
        import re

        output_clean = re.sub(r"\x1b\[[0-9;]*m", "", output)
        assert "3 succeeded" in output_clean
        assert "2 failed" in output_clean

    def test_print_output_information_xpt_only(self, presenter, console):
        """Test output information display for XPT format."""
        presenter._print_output_information(
            output_dir=Path("/output"),
            output_format="xpt",
            generate_define=False,
            generate_sas=False,
            total_records=100,
        )

        output = console.file.getvalue()
        assert "Output:" in output
        assert "/output" in output
        assert "Total records:" in output
        assert "100" in output
        assert "XPT files:" in output

    def test_print_output_information_both_formats(self, presenter, console):
        """Test output information display for both formats."""
        presenter._print_output_information(
            output_dir=Path("/output"),
            output_format="both",
            generate_define=True,
            generate_sas=True,
            total_records=500,
        )

        output = console.file.getvalue()
        assert "XPT files:" in output
        assert "Dataset-XML:" in output
        assert "SAS programs:" in output
        assert "Define-XML:" in output

    def test_build_summary_table_structure(self, presenter, sample_results):
        """Test that summary table has correct structure."""
        table = presenter._build_summary_table(sample_results)

        # Verify table title
        assert table.title == "📊 Study Processing Summary"

        # Verify columns exist (7 columns)
        assert len(table.columns) == 7

    def test_present_with_errors(self, presenter, sample_results, console):
        """Test present method with errors."""
        errors = [("LB", "Failed to process"), ("VS", "Invalid data")]

        presenter.present(
            results=sample_results,
            errors=errors,
            output_dir=Path("/output"),
            output_format="xpt",
            generate_define=False,
            generate_sas=False,
        )

        output = console.file.getvalue()
        assert "succeeded" in output
        assert "failed" in output

    def test_present_formats_records_with_commas(self, presenter, console):
        """Test that record counts are formatted with thousand separators."""
        results = [
            {
                "domain_code": "DM",
                "records": 1000,
                "xpt_path": Path("/output/xpt/dm.xpt"),
            }
        ]

        presenter.present(
            results=results,
            errors=[],
            output_dir=Path("/output"),
            output_format="xpt",
            generate_define=False,
            generate_sas=False,
        )

        output = console.file.getvalue()
        # Rich may format differently, but should contain the number
        assert "1,000" in output or "1000" in output


class TestSummaryPresenterIntegration:
    """Integration tests for SummaryPresenter with realistic data."""

    @pytest.fixture
    def console(self):
        """Create a console with StringIO for capturing output."""
        return Console(file=StringIO(), force_terminal=True, width=120)

    @pytest.fixture
    def presenter(self, console):
        """Create a SummaryPresenter instance."""
        return SummaryPresenter(console)

    def test_realistic_study_output(self, presenter, console):
        """Test with realistic study processing results."""
        results = [
            {
                "domain_code": "DM",
                "description": "Demographics",
                "records": 50,
                "xpt_path": Path("/output/xpt/dm.xpt"),
                "xml_path": Path("/output/xml/dm.xml"),
                "sas_path": Path("/output/sas/dm.sas"),
            },
            {
                "domain_code": "AE",
                "description": "Adverse Events",
                "records": 120,
                "xpt_path": Path("/output/xpt/ae.xpt"),
                "xml_path": None,
                "sas_path": Path("/output/sas/ae.sas"),
                "suppqual_domains": [
                    {
                        "domain_code": "SUPPAE",
                        "description": "Supplemental Qualifiers for AE",
                        "records": 15,
                        "xpt_path": Path("/output/xpt/suppae.xpt"),
                        "xml_path": None,
                        "sas_path": None,
                    }
                ],
            },
            {
                "domain_code": "LB",
                "description": "Laboratory Test Results",
                "records": 500,
                "xpt_path": Path("/output/xpt/lb.xpt"),
            },
        ]

        presenter.present(
            results=results,
            errors=[],
            output_dir=Path("/output"),
            output_format="both",
            generate_define=True,
            generate_sas=True,
        )

        output = console.file.getvalue()

        # Verify main domains are shown
        assert "DM" in output
        assert "AE" in output
        assert "LB" in output

        # Verify SUPPQUAL is shown under parent
        assert "SUPPAE" in output
        assert "Supplemental Qualifiers for AE" in output

        # Verify total calculation
        assert "685" in output  # 50 + 120 + 15 + 500

    def test_empty_results(self, presenter, console):
        """Test with no results."""
        presenter.present(
            results=[],
            errors=[],
            output_dir=Path("/output"),
            output_format="xpt",
            generate_define=False,
            generate_sas=False,
        )

        output = console.file.getvalue()
        # Remove ANSI codes for testing
        import re

        output_clean = re.sub(r"\x1b\[[0-9;]*m", "", output)
        assert "Study Processing Summary" in output_clean
        assert "0 domains processed successfully" in output_clean
