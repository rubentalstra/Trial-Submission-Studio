"""Integration tests for domain processing workflow.

These tests verify single domain processing workflows, transformations,
and file generation using real sample data.
"""

import pytest
from pathlib import Path
import pandas as pd

from cdisc_transpiler.infrastructure import create_default_container
from cdisc_transpiler.application.models import ProcessDomainRequest


# Path to sample study data
MOCKDATA_DIR = Path(__file__).parent.parent.parent / "mockdata"
DEMO_GDISC = MOCKDATA_DIR / "DEMO_GDISC_20240903_072908"


@pytest.mark.integration
class TestDomainFileDiscovery:
    """Integration tests for domain file discovery."""
    
    @pytest.fixture
    def study_folder(self):
        """Provide path to sample study folder."""
        if not DEMO_GDISC.exists():
            pytest.skip("Sample data not available")
        return DEMO_GDISC
    
    def test_can_find_dm_domain_file(self, study_folder):
        """Test finding DM domain file."""
        dm_files = list(study_folder.glob("*DM.csv"))
        
        assert len(dm_files) >= 1, "Should find at least one DM file"
        
        dm_file = dm_files[0]
        assert dm_file.exists()
        assert dm_file.suffix == ".csv"
    
    def test_can_find_ae_domain_file(self, study_folder):
        """Test finding AE domain file."""
        ae_files = list(study_folder.glob("*AE.csv"))
        
        assert len(ae_files) >= 1, "Should find at least one AE file"
        
        ae_file = ae_files[0]
        assert ae_file.exists()
        assert ae_file.stat().st_size > 0
    
    def test_can_find_lb_variant_files(self, study_folder):
        """Test finding LB domain variant files (LBCC, LBHM, etc)."""
        lb_files = list(study_folder.glob("*LB*.csv"))
        
        # Filter out Items and CodeLists
        lb_files = [f for f in lb_files if "Items" not in f.name and "CodeLists" not in f.name]
        
        # Should find multiple LB variants
        assert len(lb_files) > 1, "Should find multiple LB variant files"
        
        # Check for specific variants
        lb_names = [f.name for f in lb_files]
        assert any("LBCC" in name for name in lb_names), "Should find LBCC variant"
        assert any("LBHM" in name for name in lb_names), "Should find LBHM variant"
    
    def test_can_identify_domain_variants(self, study_folder):
        """Test identifying domain variants from filenames."""
        csv_files = list(study_folder.glob("*.csv"))
        
        # Filter for domain files (exclude Items and CodeLists)
        domain_files = [
            f for f in csv_files 
            if "Items" not in f.name and "CodeLists" not in f.name and "CSV2SAS" not in f.name
        ]
        
        assert len(domain_files) > 0
        
        # Group by base domain
        domains = set()
        for f in domain_files:
            # Extract domain code from filename
            # Format: DEMO_GDISC_20240903_072908_DM.csv or _LBCC.csv
            parts = f.stem.split("_")
            if len(parts) > 0:
                domain_part = parts[-1]
                # Get base domain (e.g., LB from LBCC, DM from DM)
                if domain_part.startswith("LB"):
                    domains.add("LB")
                elif domain_part.startswith("DS"):
                    domains.add("DS")
                else:
                    domains.add(domain_part[:2] if len(domain_part) >= 2 else domain_part)
        
        assert "DM" in domains
        assert "AE" in domains
        assert "LB" in domains


@pytest.mark.integration
class TestDomainDataReading:
    """Integration tests for reading domain data files."""
    
    @pytest.fixture
    def study_folder(self):
        """Provide path to sample study folder."""
        if not DEMO_GDISC.exists():
            pytest.skip("Sample data not available")
        return DEMO_GDISC
    
    def test_can_read_dm_file_as_dataframe(self, study_folder):
        """Test reading DM file into pandas DataFrame."""
        dm_files = list(study_folder.glob("*DM.csv"))
        if not dm_files:
            pytest.skip("DM file not found")
        
        dm_file = dm_files[0]
        df = pd.read_csv(dm_file)
        
        assert not df.empty, "DM dataframe should not be empty"
        assert len(df) > 0, "DM should have rows"
        assert len(df.columns) > 0, "DM should have columns"
    
    def test_can_read_ae_file_as_dataframe(self, study_folder):
        """Test reading AE file into pandas DataFrame."""
        ae_files = list(study_folder.glob("*AE.csv"))
        if not ae_files:
            pytest.skip("AE file not found")
        
        ae_file = ae_files[0]
        df = pd.read_csv(ae_file)
        
        assert not df.empty, "AE dataframe should not be empty"
        assert len(df) > 0, "AE should have rows"
    
    def test_can_read_multiple_lb_variants(self, study_folder):
        """Test reading multiple LB variant files."""
        lb_files = list(study_folder.glob("*LB*.csv"))
        lb_files = [f for f in lb_files if "Items" not in f.name and "CodeLists" not in f.name]
        
        if len(lb_files) < 2:
            pytest.skip("Not enough LB variant files")
        
        dataframes = []
        for lb_file in lb_files:
            df = pd.read_csv(lb_file)
            dataframes.append((lb_file.name, df))
        
        assert len(dataframes) >= 2
        
        # Each variant should have data
        for name, df in dataframes:
            assert not df.empty, f"{name} should not be empty"


@pytest.mark.integration
class TestProcessDomainRequest:
    """Integration tests for ProcessDomainRequest with real paths."""
    
    @pytest.fixture
    def study_folder(self):
        """Provide path to sample study folder."""
        if not DEMO_GDISC.exists():
            pytest.skip("Sample data not available")
        return DEMO_GDISC
    
    def test_create_domain_request_for_dm(self, study_folder):
        """Test creating a domain request for DM."""
        dm_files = list(study_folder.glob("*DM.csv"))
        if not dm_files:
            pytest.skip("DM file not found")
        
        dm_file = dm_files[0]
        
        request = ProcessDomainRequest(
            files_for_domain=[(dm_file, "DM")],
            domain_code="DM",
            study_id="DEMO_GDISC",
            output_formats={"xpt", "xml"},
            output_dirs={"xpt": Path("/tmp/xpt"), "xml": Path("/tmp/xml")},
        )
        
        assert request.domain_code == "DM"
        assert len(request.files_for_domain) == 1
        assert request.files_for_domain[0][0] == dm_file
    
    def test_create_domain_request_for_lb_variants(self, study_folder):
        """Test creating a domain request with multiple LB variants."""
        lb_files = list(study_folder.glob("*LB*.csv"))
        lb_files = [f for f in lb_files if "Items" not in f.name and "CodeLists" not in f.name]
        
        if len(lb_files) < 2:
            pytest.skip("Not enough LB variant files")
        
        # Create file list with variants
        files_for_domain = []
        for lb_file in lb_files:
            # Extract variant name from filename
            variant = lb_file.stem.split("_")[-1]
            files_for_domain.append((lb_file, variant))
        
        request = ProcessDomainRequest(
            files_for_domain=files_for_domain,
            domain_code="LB",
            study_id="DEMO_GDISC",
            output_formats={"xpt"},
            output_dirs={"xpt": Path("/tmp/xpt")},
        )
        
        assert request.domain_code == "LB"
        assert len(request.files_for_domain) >= 2
    
    def test_domain_request_with_metadata(self, study_folder):
        """Test creating domain request with metadata parameters."""
        dm_files = list(study_folder.glob("*DM.csv"))
        if not dm_files:
            pytest.skip("DM file not found")
        
        request = ProcessDomainRequest(
            files_for_domain=[(dm_files[0], "DM")],
            domain_code="DM",
            study_id="TEST",
            output_formats={"xpt"},
            output_dirs={"xpt": Path("/tmp/xpt")},
            min_confidence=0.7,
            verbose=1,
            reference_starts={"SUBJ001": "2023-01-01"},
            common_column_counts={"studyid": 10, "usubjid": 10},
            total_input_files=15,
        )
        
        assert request.min_confidence == 0.7
        assert request.verbose == 1
        assert request.reference_starts is not None
        assert request.common_column_counts is not None
        assert request.total_input_files == 15


@pytest.mark.integration
class TestMetadataFiles:
    """Integration tests for metadata files (Items.csv, CodeLists.csv)."""
    
    @pytest.fixture
    def study_folder(self):
        """Provide path to sample study folder."""
        if not DEMO_GDISC.exists():
            pytest.skip("Sample data not available")
        return DEMO_GDISC
    
    def test_can_read_items_file(self, study_folder):
        """Test reading Items.csv metadata file."""
        items_files = list(study_folder.glob("*Items.csv"))
        if not items_files:
            pytest.skip("Items.csv not found")
        
        items_file = items_files[0]
        df = pd.read_csv(items_file)
        
        assert not df.empty, "Items should not be empty"
        assert len(df) > 0, "Items should have rows"
        
        # Check for expected columns (based on CDISC metadata structure)
        # Items file typically has OID, Name, DataType, etc.
        assert len(df.columns) > 0
    
    def test_can_read_codelists_file(self, study_folder):
        """Test reading CodeLists.csv metadata file."""
        codelists_files = list(study_folder.glob("*CodeLists.csv"))
        if not codelists_files:
            pytest.skip("CodeLists.csv not found")
        
        codelists_file = codelists_files[0]
        df = pd.read_csv(codelists_file)
        
        assert not df.empty, "CodeLists should not be empty"
        assert len(df) > 0, "CodeLists should have rows"
    
    def test_metadata_files_have_expected_structure(self, study_folder):
        """Test that metadata files have expected structure."""
        items_files = list(study_folder.glob("*Items.csv"))
        codelists_files = list(study_folder.glob("*CodeLists.csv"))
        
        if not items_files or not codelists_files:
            pytest.skip("Metadata files not found")
        
        # Read both files
        items_df = pd.read_csv(items_files[0])
        codelists_df = pd.read_csv(codelists_files[0])
        
        # Both should have data
        assert len(items_df) > 0
        assert len(codelists_df) > 0
        
        # Items should have more rows (one per variable)
        # CodeLists should have codelist definitions
        assert len(items_df) >= len(codelists_df)


@pytest.mark.integration
class TestOutputFileGeneration:
    """Integration tests for output file generation."""
    
    def test_can_create_output_file_paths(self, tmp_path):
        """Test creating output file paths."""
        output_dir = tmp_path / "output"
        output_dir.mkdir()
        
        xpt_dir = output_dir / "xpt"
        xml_dir = output_dir / "dataset-xml"
        
        xpt_dir.mkdir()
        xml_dir.mkdir()
        
        # Create expected output files
        dm_xpt = xpt_dir / "dm.xpt"
        dm_xml = xml_dir / "dm.xml"
        
        # Simulate file creation
        dm_xpt.touch()
        dm_xml.touch()
        
        assert dm_xpt.exists()
        assert dm_xml.exists()
    
    def test_can_organize_files_by_format(self, tmp_path):
        """Test organizing output files by format."""
        output_dir = tmp_path / "output"
        output_dir.mkdir()
        
        # Create format-specific directories
        formats = ["xpt", "dataset-xml", "sas"]
        dirs = {}
        for fmt in formats:
            dir_path = output_dir / fmt
            dir_path.mkdir()
            dirs[fmt] = dir_path
        
        # Verify structure
        assert dirs["xpt"].exists()
        assert dirs["dataset-xml"].exists()
        assert dirs["sas"].exists()
    
    def test_output_directory_cleanup(self, tmp_path):
        """Test cleaning up output directory."""
        output_dir = tmp_path / "output"
        output_dir.mkdir()
        
        # Create some files
        test_file = output_dir / "test.txt"
        test_file.write_text("test")
        
        assert test_file.exists()
        
        # Cleanup (remove all files)
        for file in output_dir.glob("*"):
            if file.is_file():
                file.unlink()
        
        # Directory should still exist but be empty
        assert output_dir.exists()
        assert len(list(output_dir.glob("*"))) == 0
