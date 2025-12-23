"""Unit tests for DatasetOutputAdapter."""

from pathlib import Path
from unittest.mock import Mock

import pandas as pd
import pytest

from cdisc_transpiler.application.models import (
    DatasetOutputDirs,
    DatasetOutputRequest,
    DatasetOutputResult,
)
from cdisc_transpiler.domain.entities.mapping import ColumnMapping, MappingConfig
from cdisc_transpiler.infrastructure.io.dataset_output import DatasetOutputAdapter


@pytest.fixture
def sample_dataframe():
    """Create a sample DataFrame for testing."""
    return pd.DataFrame(
        {
            "STUDYID": ["STUDY001", "STUDY001"],
            "DOMAIN": ["DM", "DM"],
            "USUBJID": ["001", "002"],
            "AGE": [25, 30],
        }
    )


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


@pytest.fixture
def mock_xpt_writer():
    """Create a mock XPT writer."""
    return Mock()


@pytest.fixture
def mock_xml_writer():
    """Create a mock Dataset-XML writer."""
    return Mock()


@pytest.fixture
def mock_sas_writer():
    """Create a mock SAS writer."""
    return Mock()


@pytest.fixture
def dataset_output(mock_xpt_writer, mock_xml_writer, mock_sas_writer):
    """Create a DatasetOutputAdapter with mock writers."""
    return DatasetOutputAdapter(
        xpt_writer=mock_xpt_writer,
        xml_writer=mock_xml_writer,
        sas_writer=mock_sas_writer,
    )


class TestDatasetOutputDirs:
    """Test suite for DatasetOutputDirs model."""

    def test_create_output_dirs(self, tmp_path: Path):
        """Test creating DatasetOutputDirs with all directories."""
        dirs = DatasetOutputDirs(
            xpt_dir=tmp_path / "xpt",
            xml_dir=tmp_path / "xml",
            sas_dir=tmp_path / "sas",
        )

        assert dirs.xpt_dir == tmp_path / "xpt"
        assert dirs.xml_dir == tmp_path / "xml"
        assert dirs.sas_dir == tmp_path / "sas"

    def test_create_output_dirs_partial(self, tmp_path: Path):
        """Test creating DatasetOutputDirs with only some directories."""
        dirs = DatasetOutputDirs(xpt_dir=tmp_path / "xpt")

        assert dirs.xpt_dir == tmp_path / "xpt"
        assert dirs.xml_dir is None
        assert dirs.sas_dir is None


class TestDatasetOutputRequest:
    """Test suite for DatasetOutputRequest model."""

    def test_create_request(self, sample_dataframe, sample_config, tmp_path: Path):
        """Test creating an output request."""
        dirs = DatasetOutputDirs(xpt_dir=tmp_path / "xpt")
        request = DatasetOutputRequest(
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


class TestDatasetOutputResult:
    """Test suite for DatasetOutputResult model."""

    def test_result_success(self, tmp_path: Path):
        """Test result with no errors is successful."""
        result = DatasetOutputResult(
            xpt_path=tmp_path / "dm.xpt",
            errors=[],
        )

        assert result.success is True
        assert result.xpt_path == tmp_path / "dm.xpt"

    def test_result_failure(self, tmp_path: Path):
        """Test result with errors is not successful."""
        result = DatasetOutputResult(
            xpt_path=tmp_path / "dm.xpt",
            errors=["XPT generation failed"],
        )

        assert result.success is False
        assert len(result.errors) == 1


class TestDatasetOutputAdapter:
    """Test suite for DatasetOutputAdapter class."""

    def test_generate_xpt_only(
        self,
        dataset_output,
        mock_xpt_writer,
        sample_dataframe,
        sample_config,
        tmp_path: Path,
    ):
        """Test generating only XPT file."""
        # Setup
        xpt_dir = tmp_path / "xpt"
        xpt_dir.mkdir()

        dirs = DatasetOutputDirs(xpt_dir=xpt_dir)
        request = DatasetOutputRequest(
            dataframe=sample_dataframe,
            domain_code="DM",
            config=sample_config,
            output_dirs=dirs,
            formats={"xpt"},
            base_filename="dm",  # Provide base_filename to avoid domain lookup
        )

        # Execute
        result = dataset_output.generate(request)

        # Verify
        assert result.success
        assert result.xpt_path == xpt_dir / "dm.xpt"
        assert result.xml_path is None
        assert result.sas_path is None
        mock_xpt_writer.write.assert_called_once_with(
            sample_dataframe, "DM", xpt_dir / "dm.xpt"
        )

    def test_generate_all_formats(
        self,
        dataset_output,
        mock_xpt_writer,
        mock_xml_writer,
        mock_sas_writer,
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

        dirs = DatasetOutputDirs(xpt_dir=xpt_dir, xml_dir=xml_dir, sas_dir=sas_dir)
        request = DatasetOutputRequest(
            dataframe=sample_dataframe,
            domain_code="DM",
            config=sample_config,
            output_dirs=dirs,
            formats={"xpt", "xml", "sas"},
            base_filename="dm",  # Provide base_filename to avoid domain lookup
        )

        # Execute
        result = dataset_output.generate(request)

        # Verify
        assert result.success
        assert result.xpt_path == xpt_dir / "dm.xpt"
        assert result.xml_path == xml_dir / "dm.xml"
        assert result.sas_path == sas_dir / "dm.sas"
        mock_xpt_writer.write.assert_called_once_with(
            sample_dataframe, "DM", xpt_dir / "dm.xpt"
        )
        mock_xml_writer.write.assert_called_once_with(
            sample_dataframe, "DM", sample_config, xml_dir / "dm.xml"
        )
        mock_sas_writer.write.assert_called_once_with(
            "DM",
            sample_config,
            sas_dir / "dm.sas",
            input_dataset="work.dm",
            output_dataset="sdtm.dm",
        )

    def test_generate_with_error(
        self,
        dataset_output,
        mock_xpt_writer,
        sample_dataframe,
        sample_config,
        tmp_path: Path,
    ):
        """Test error handling during generation."""
        # Setup
        xpt_dir = tmp_path / "xpt"
        xpt_dir.mkdir()

        mock_xpt_writer.write.side_effect = Exception("Write failed")

        dirs = DatasetOutputDirs(xpt_dir=xpt_dir)
        request = DatasetOutputRequest(
            dataframe=sample_dataframe,
            domain_code="DM",
            config=sample_config,
            output_dirs=dirs,
            formats={"xpt"},
            base_filename="dm",
        )

        # Execute
        result = dataset_output.generate(request)

        # Verify
        assert not result.success
        assert len(result.errors) == 1
        assert "XPT generation failed" in result.errors[0]

    def test_generate_with_custom_base_filename(
        self,
        dataset_output,
        mock_xpt_writer,
        sample_dataframe,
        sample_config,
        tmp_path: Path,
    ):
        """Test generation with custom base filename."""
        # Setup
        xpt_dir = tmp_path / "xpt"
        xpt_dir.mkdir()

        dirs = DatasetOutputDirs(xpt_dir=xpt_dir)
        request = DatasetOutputRequest(
            dataframe=sample_dataframe,
            domain_code="DM",
            config=sample_config,
            output_dirs=dirs,
            formats={"xpt"},
            base_filename="CUSTOM",
        )

        # Execute
        result = dataset_output.generate(request)

        # Verify
        assert result.success
        assert result.xpt_path == xpt_dir / "custom.xpt"  # lowercase
        mock_xpt_writer.write.assert_called_once_with(
            sample_dataframe, "DM", xpt_dir / "custom.xpt"
        )

    def test_generate_sas_with_custom_datasets(
        self,
        dataset_output,
        mock_sas_writer,
        sample_dataframe,
        sample_config,
        tmp_path: Path,
    ):
        """Test SAS generation with custom input/output dataset names."""
        # Setup
        sas_dir = tmp_path / "sas"
        sas_dir.mkdir()

        dirs = DatasetOutputDirs(sas_dir=sas_dir)
        request = DatasetOutputRequest(
            dataframe=sample_dataframe,
            domain_code="DM",
            config=sample_config,
            output_dirs=dirs,
            formats={"sas"},
            base_filename="dm",
            input_dataset="raw.demographics",
            output_dataset="final.dm",
        )

        # Execute
        result = dataset_output.generate(request)

        # Verify
        assert result.success
        assert result.sas_path == sas_dir / "dm.sas"

        # Check that custom dataset names were passed
        mock_sas_writer.write.assert_called_once_with(
            "DM",
            sample_config,
            sas_dir / "dm.sas",
            input_dataset="raw.demographics",
            output_dataset="final.dm",
        )

    def test_generate_no_formats(
        self,
        dataset_output,
        sample_dataframe,
        sample_config,
        tmp_path: Path,
    ):
        """Test generation with no formats requested."""
        # Setup
        dirs = DatasetOutputDirs()
        request = DatasetOutputRequest(
            dataframe=sample_dataframe,
            domain_code="DM",
            config=sample_config,
            output_dirs=dirs,
            formats=set(),  # Empty set
        )

        # Execute
        result = dataset_output.generate(request)

        # Verify
        assert result.success
        assert result.xpt_path is None
        assert result.xml_path is None
        assert result.sas_path is None
