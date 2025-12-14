"""Unit tests for FileGenerator."""

from __future__ import annotations

from pathlib import Path
from unittest.mock import MagicMock, patch

import pandas as pd
import pytest

from cdisc_transpiler.infrastructure.io import (
    FileGenerator,
    OutputDirs,
    OutputRequest,
    OutputResult,
)
from cdisc_transpiler.mapping_module import ColumnMapping, MappingConfig


@pytest.fixture
def sample_dataframe():
    """Create a sample DataFrame for testing."""
    return pd.DataFrame({
        "STUDYID": ["STUDY001", "STUDY001"],
        "DOMAIN": ["DM", "DM"],
        "USUBJID": ["001", "002"],
        "AGE": [25, 30],
    })


@pytest.fixture
def sample_config():
    """Create a sample MappingConfig for testing."""
    mappings = [
        ColumnMapping(
            source_column="STUDYID",
            target_variable="STUDYID",
            transformation=None,
            confidence_score=1.0,
        ),
        ColumnMapping(
            source_column="DOMAIN",
            target_variable="DOMAIN",
            transformation=None,
            confidence_score=1.0,
        ),
        ColumnMapping(
            source_column="USUBJID",
            target_variable="USUBJID",
            transformation=None,
            confidence_score=1.0,
        ),
        ColumnMapping(
            source_column="AGE",
            target_variable="AGE",
            transformation=None,
            confidence_score=1.0,
        ),
    ]
    config = MappingConfig(
        domain="DM",
        mappings=mappings,
        study_id="STUDY001",
        unmapped_columns=[],
    )
    return config


class TestOutputDirs:
    """Test suite for OutputDirs model."""
    
    def test_create_output_dirs(self, tmp_path: Path):
        """Test creating OutputDirs with all directories."""
        dirs = OutputDirs(
            xpt_dir=tmp_path / "xpt",
            xml_dir=tmp_path / "xml",
            sas_dir=tmp_path / "sas",
        )
        
        assert dirs.xpt_dir == tmp_path / "xpt"
        assert dirs.xml_dir == tmp_path / "xml"
        assert dirs.sas_dir == tmp_path / "sas"
    
    def test_create_output_dirs_partial(self, tmp_path: Path):
        """Test creating OutputDirs with only some directories."""
        dirs = OutputDirs(xpt_dir=tmp_path / "xpt")
        
        assert dirs.xpt_dir == tmp_path / "xpt"
        assert dirs.xml_dir is None
        assert dirs.sas_dir is None


class TestOutputRequest:
    """Test suite for OutputRequest model."""
    
    def test_create_request(self, sample_dataframe, sample_config, tmp_path: Path):
        """Test creating an output request."""
        dirs = OutputDirs(xpt_dir=tmp_path / "xpt")
        request = OutputRequest(
            dataframe=sample_dataframe,
            domain_code="DM",
            config=sample_config,
            output_dirs=dirs,
            formats={"xpt"},
        )
        
        assert len(request.dataframe) == 2
        assert request.domain_code == "DM"
        assert request.config.domain == "DM"
        assert "xpt" in request.formats


class TestOutputResult:
    """Test suite for OutputResult model."""
    
    def test_result_success(self, tmp_path: Path):
        """Test result with no errors is successful."""
        result = OutputResult(
            xpt_path=tmp_path / "dm.xpt",
            errors=[],
        )
        
        assert result.success is True
        assert result.xpt_path == tmp_path / "dm.xpt"
    
    def test_result_failure(self, tmp_path: Path):
        """Test result with errors is not successful."""
        result = OutputResult(
            xpt_path=tmp_path / "dm.xpt",
            errors=["XPT generation failed"],
        )
        
        assert result.success is False
        assert len(result.errors) == 1


class TestFileGenerator:
    """Test suite for FileGenerator class."""
    
    @patch("cdisc_transpiler.infrastructure.io.file_generator.write_xpt_file")
    @patch("cdisc_transpiler.infrastructure.io.file_generator.get_domain")
    def test_generate_xpt_only(
        self,
        mock_get_domain,
        mock_write_xpt,
        sample_dataframe,
        sample_config,
        tmp_path: Path,
    ):
        """Test generating only XPT file."""
        # Setup
        xpt_dir = tmp_path / "xpt"
        xpt_dir.mkdir()
        
        mock_domain = MagicMock()
        mock_domain.resolved_dataset_name.return_value = "dm"
        mock_get_domain.return_value = mock_domain
        
        dirs = OutputDirs(xpt_dir=xpt_dir)
        request = OutputRequest(
            dataframe=sample_dataframe,
            domain_code="DM",
            config=sample_config,
            output_dirs=dirs,
            formats={"xpt"},
        )
        
        # Execute
        generator = FileGenerator()
        result = generator.generate(request)
        
        # Verify
        assert result.success
        assert result.xpt_path == xpt_dir / "dm.xpt"
        assert result.xml_path is None
        assert result.sas_path is None
        mock_write_xpt.assert_called_once()
    
    @patch("cdisc_transpiler.infrastructure.io.file_generator.write_sas_file")
    @patch("cdisc_transpiler.infrastructure.io.file_generator.generate_sas_program")
    @patch("cdisc_transpiler.infrastructure.io.file_generator.write_dataset_xml")
    @patch("cdisc_transpiler.infrastructure.io.file_generator.write_xpt_file")
    @patch("cdisc_transpiler.infrastructure.io.file_generator.get_domain")
    def test_generate_all_formats(
        self,
        mock_get_domain,
        mock_write_xpt,
        mock_write_xml,
        mock_gen_sas,
        mock_write_sas,
        sample_dataframe,
        sample_config,
        tmp_path: Path,
    ):
        """Test generating all formats (XPT, XML, SAS)."""
        # Setup
        xpt_dir = tmp_path / "xpt"
        xml_dir = tmp_path / "xml"
        sas_dir = tmp_path / "sas"
        xpt_dir.mkdir()
        xml_dir.mkdir()
        sas_dir.mkdir()
        
        mock_domain = MagicMock()
        mock_domain.resolved_dataset_name.return_value = "dm"
        mock_get_domain.return_value = mock_domain
        mock_gen_sas.return_value = "/* SAS code */"
        
        dirs = OutputDirs(xpt_dir=xpt_dir, xml_dir=xml_dir, sas_dir=sas_dir)
        request = OutputRequest(
            dataframe=sample_dataframe,
            domain_code="DM",
            config=sample_config,
            output_dirs=dirs,
            formats={"xpt", "xml", "sas"},
        )
        
        # Execute
        generator = FileGenerator()
        result = generator.generate(request)
        
        # Verify
        assert result.success
        assert result.xpt_path == xpt_dir / "dm.xpt"
        assert result.xml_path == xml_dir / "dm.xml"
        assert result.sas_path == sas_dir / "dm.sas"
        mock_write_xpt.assert_called_once()
        mock_write_xml.assert_called_once()
        mock_gen_sas.assert_called_once()
        mock_write_sas.assert_called_once()
    
    @patch("cdisc_transpiler.infrastructure.io.file_generator.write_xpt_file")
    @patch("cdisc_transpiler.infrastructure.io.file_generator.get_domain")
    def test_generate_with_error(
        self,
        mock_get_domain,
        mock_write_xpt,
        sample_dataframe,
        sample_config,
        tmp_path: Path,
    ):
        """Test error handling during generation."""
        # Setup
        xpt_dir = tmp_path / "xpt"
        xpt_dir.mkdir()
        
        mock_domain = MagicMock()
        mock_domain.resolved_dataset_name.return_value = "dm"
        mock_get_domain.return_value = mock_domain
        mock_write_xpt.side_effect = Exception("Write failed")
        
        dirs = OutputDirs(xpt_dir=xpt_dir)
        request = OutputRequest(
            dataframe=sample_dataframe,
            domain_code="DM",
            config=sample_config,
            output_dirs=dirs,
            formats={"xpt"},
        )
        
        # Execute
        generator = FileGenerator()
        result = generator.generate(request)
        
        # Verify
        assert not result.success
        assert len(result.errors) == 1
        assert "XPT generation failed" in result.errors[0]
    
    @patch("cdisc_transpiler.infrastructure.io.file_generator.write_xpt_file")
    @patch("cdisc_transpiler.infrastructure.io.file_generator.get_domain")
    def test_generate_with_custom_base_filename(
        self,
        mock_get_domain,
        mock_write_xpt,
        sample_dataframe,
        sample_config,
        tmp_path: Path,
    ):
        """Test generation with custom base filename."""
        # Setup
        xpt_dir = tmp_path / "xpt"
        xpt_dir.mkdir()
        
        dirs = OutputDirs(xpt_dir=xpt_dir)
        request = OutputRequest(
            dataframe=sample_dataframe,
            domain_code="DM",
            config=sample_config,
            output_dirs=dirs,
            formats={"xpt"},
            base_filename="CUSTOM",
        )
        
        # Execute
        generator = FileGenerator()
        result = generator.generate(request)
        
        # Verify
        assert result.success
        assert result.xpt_path == xpt_dir / "custom.xpt"  # lowercase
        mock_write_xpt.assert_called_once()
        # Should NOT call get_domain since base_filename is provided
        mock_get_domain.assert_not_called()
    
    @patch("cdisc_transpiler.infrastructure.io.file_generator.write_sas_file")
    @patch("cdisc_transpiler.infrastructure.io.file_generator.generate_sas_program")
    @patch("cdisc_transpiler.infrastructure.io.file_generator.get_domain")
    def test_generate_sas_with_custom_datasets(
        self,
        mock_get_domain,
        mock_gen_sas,
        mock_write_sas,
        sample_dataframe,
        sample_config,
        tmp_path: Path,
    ):
        """Test SAS generation with custom input/output dataset names."""
        # Setup
        sas_dir = tmp_path / "sas"
        sas_dir.mkdir()
        
        mock_domain = MagicMock()
        mock_domain.resolved_dataset_name.return_value = "dm"
        mock_get_domain.return_value = mock_domain
        mock_gen_sas.return_value = "/* SAS code */"
        
        dirs = OutputDirs(sas_dir=sas_dir)
        request = OutputRequest(
            dataframe=sample_dataframe,
            domain_code="DM",
            config=sample_config,
            output_dirs=dirs,
            formats={"sas"},
            input_dataset="raw.demographics",
            output_dataset="final.dm",
        )
        
        # Execute
        generator = FileGenerator()
        result = generator.generate(request)
        
        # Verify
        assert result.success
        assert result.sas_path == sas_dir / "dm.sas"
        
        # Check that custom dataset names were passed
        call_args = mock_gen_sas.call_args
        assert call_args[1]["input_dataset"] == "raw.demographics"
        assert call_args[1]["output_dataset"] == "final.dm"
    
    def test_generate_no_formats(
        self,
        sample_dataframe,
        sample_config,
        tmp_path: Path,
    ):
        """Test generation with no formats requested."""
        # Setup
        dirs = OutputDirs()
        request = OutputRequest(
            dataframe=sample_dataframe,
            domain_code="DM",
            config=sample_config,
            output_dirs=dirs,
            formats=set(),  # Empty set
        )
        
        # Execute
        generator = FileGenerator()
        result = generator.generate(request)
        
        # Verify
        assert result.success
        assert result.xpt_path is None
        assert result.xml_path is None
        assert result.sas_path is None
