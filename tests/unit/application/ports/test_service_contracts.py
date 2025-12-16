"""Contract tests for service port interfaces.

These tests define the expected behavior that any service implementation
must satisfy. They verify that implementations correctly adhere to the
port interfaces using runtime protocol checks.
"""

from pathlib import Path

import pandas as pd
import pytest

from cdisc_transpiler.application.ports import FileGeneratorPort, LoggerPort
from cdisc_transpiler.application.models import (
    OutputDirs,
    OutputRequest,
    OutputResult,
)
from cdisc_transpiler.domain.entities.mapping import MappingConfig


class MockLogger:
    """Mock logger for testing protocol compliance."""

    def __init__(self):
        self.messages = []

    def info(self, message: str) -> None:
        self.messages.append(("info", message))

    def success(self, message: str) -> None:
        self.messages.append(("success", message))

    def warning(self, message: str) -> None:
        self.messages.append(("warning", message))

    def error(self, message: str) -> None:
        self.messages.append(("error", message))

    def debug(self, message: str) -> None:
        self.messages.append(("debug", message))

    def verbose(self, message: str) -> None:
        self.messages.append(("verbose", message))

    def log_study_start(
        self,
        study_id: str,
        study_folder: Path,
        output_format: str,
        supported_domains: list[str],
    ) -> None:
        self.messages.append(
            (
                "log_study_start",
                {
                    "study_id": study_id,
                    "study_folder": study_folder,
                    "output_format": output_format,
                    "supported_domains": supported_domains,
                },
            )
        )

    def log_metadata_loaded(
        self,
        *,
        items_count: int | None,
        codelists_count: int | None,
    ) -> None:
        self.messages.append(
            (
                "log_metadata_loaded",
                {"items_count": items_count, "codelists_count": codelists_count},
            )
        )

    def log_processing_summary(
        self,
        *,
        study_id: str,
        domain_count: int,
        file_count: int,
        output_format: str,
        generate_define: bool,
        generate_sas: bool,
    ) -> None:
        self.messages.append(
            (
                "log_processing_summary",
                {
                    "study_id": study_id,
                    "domain_count": domain_count,
                    "file_count": file_count,
                    "output_format": output_format,
                    "generate_define": generate_define,
                    "generate_sas": generate_sas,
                },
            )
        )

    def log_final_stats(self) -> None:
        self.messages.append(("log_final_stats", None))

    def log_domain_start(
        self,
        domain_code: str,
        files_for_domain: list[tuple[Path, str]],
    ) -> None:
        self.messages.append(
            (
                "log_domain_start",
                {"domain_code": domain_code, "files_for_domain": files_for_domain},
            )
        )

    def log_synthesis_start(self, domain_code: str, reason: str) -> None:
        self.messages.append(
            ("log_synthesis_start", {"domain_code": domain_code, "reason": reason})
        )

    def log_synthesis_complete(self, domain_code: str, records: int) -> None:
        self.messages.append(
            ("log_synthesis_complete", {"domain_code": domain_code, "records": records})
        )


class MockFileGenerator:
    """Mock file generator for testing protocol compliance."""

    def generate(self, request: OutputRequest) -> OutputResult:
        """Mock implementation that returns success result."""
        result = OutputResult()

        if "xpt" in request.formats and request.output_dirs.xpt_dir:
            result.xpt_path = (
                request.output_dirs.xpt_dir / f"{request.domain_code.lower()}.xpt"
            )

        if "xml" in request.formats and request.output_dirs.xml_dir:
            result.xml_path = (
                request.output_dirs.xml_dir / f"{request.domain_code.lower()}.xml"
            )

        if "sas" in request.formats and request.output_dirs.sas_dir:
            result.sas_path = (
                request.output_dirs.sas_dir / f"{request.domain_code.lower()}.sas"
            )

        return result


class TestLoggerPortContract:
    """Contract tests for LoggerPort implementations."""

    @pytest.fixture
    def logger(self):
        """Provide a mock logger."""
        return MockLogger()

    def test_implements_protocol(self, logger):
        """Test that implementation satisfies the protocol."""
        assert isinstance(logger, LoggerPort)

    def test_has_info_method(self, logger):
        """Test that logger has info method."""
        assert hasattr(logger, "info")
        assert callable(logger.info)

    def test_has_success_method(self, logger):
        """Test that logger has success method."""
        assert hasattr(logger, "success")
        assert callable(logger.success)

    def test_has_warning_method(self, logger):
        """Test that logger has warning method."""
        assert hasattr(logger, "warning")
        assert callable(logger.warning)

    def test_has_error_method(self, logger):
        """Test that logger has error method."""
        assert hasattr(logger, "error")
        assert callable(logger.error)

    def test_has_debug_method(self, logger):
        """Test that logger has debug method."""
        assert hasattr(logger, "debug")
        assert callable(logger.debug)

    def test_info_accepts_string_message(self, logger):
        """Test that info accepts a string message."""
        logger.info("Test message")
        # Should not raise an exception

    def test_success_accepts_string_message(self, logger):
        """Test that success accepts a string message."""
        logger.success("Test success")
        # Should not raise an exception

    def test_warning_accepts_string_message(self, logger):
        """Test that warning accepts a string message."""
        logger.warning("Test warning")
        # Should not raise an exception

    def test_error_accepts_string_message(self, logger):
        """Test that error accepts a string message."""
        logger.error("Test error")
        # Should not raise an exception

    def test_debug_accepts_string_message(self, logger):
        """Test that debug accepts a string message."""
        logger.debug("Test debug")
        # Should not raise an exception

    def test_all_log_methods_work(self, logger):
        """Test that all log methods can be called successfully."""
        logger.info("Info message")
        logger.success("Success message")
        logger.warning("Warning message")
        logger.error("Error message")
        logger.debug("Debug message")

        # Verify all messages were captured (implementation-specific)
        if hasattr(logger, "messages"):
            assert len(logger.messages) == 5


class TestFileGeneratorPortContract:
    """Contract tests for FileGeneratorPort implementations."""

    @pytest.fixture
    def file_generator(self):
        """Provide a mock file generator."""
        return MockFileGenerator()

    @pytest.fixture
    def sample_dataframe(self):
        """Provide a sample DataFrame for testing."""
        return pd.DataFrame(
            {
                "STUDYID": ["TEST001"],
                "DOMAIN": ["DM"],
                "USUBJID": ["TEST001-001"],
            }
        )

    @pytest.fixture
    def sample_config(self):
        """Provide a sample MappingConfig for testing."""
        return MappingConfig(
            domain="DM",
            study_id="TEST001",
            mappings=[],
        )

    @pytest.fixture
    def sample_request(self, sample_dataframe, sample_config, tmp_path):
        """Provide a sample OutputRequest for testing."""
        return OutputRequest(
            dataframe=sample_dataframe,
            domain_code="DM",
            config=sample_config,
            output_dirs=OutputDirs(
                xpt_dir=tmp_path / "xpt",
                xml_dir=tmp_path / "xml",
                sas_dir=tmp_path / "sas",
            ),
            formats={"xpt", "xml", "sas"},
        )

    def test_implements_protocol(self, file_generator):
        """Test that implementation satisfies the protocol."""
        assert isinstance(file_generator, FileGeneratorPort)

    def test_has_generate_method(self, file_generator):
        """Test that file generator has generate method."""
        assert hasattr(file_generator, "generate")
        assert callable(file_generator.generate)

    def test_generate_returns_output_result(self, file_generator, sample_request):
        """Test that generate returns OutputResult."""
        result = file_generator.generate(sample_request)
        assert isinstance(result, OutputResult)

    def test_generate_result_has_expected_attributes(
        self, file_generator, sample_request
    ):
        """Test that OutputResult has expected attributes."""
        result = file_generator.generate(sample_request)
        assert hasattr(result, "xpt_path")
        assert hasattr(result, "xml_path")
        assert hasattr(result, "sas_path")
        assert hasattr(result, "errors")
        assert hasattr(result, "success")

    def test_generate_result_paths_are_path_or_none(
        self, file_generator, sample_request
    ):
        """Test that result paths are Path objects or None."""
        result = file_generator.generate(sample_request)
        assert result.xpt_path is None or isinstance(result.xpt_path, Path)
        assert result.xml_path is None or isinstance(result.xml_path, Path)
        assert result.sas_path is None or isinstance(result.sas_path, Path)

    def test_generate_result_errors_is_list(self, file_generator, sample_request):
        """Test that result errors is a list."""
        result = file_generator.generate(sample_request)
        assert isinstance(result.errors, list)

    def test_generate_result_success_is_bool(self, file_generator, sample_request):
        """Test that result success is a boolean."""
        result = file_generator.generate(sample_request)
        assert isinstance(result.success, bool)

    def test_generate_with_xpt_format(self, file_generator, sample_request):
        """Test generate with XPT format only."""
        sample_request.formats = {"xpt"}
        result = file_generator.generate(sample_request)
        assert isinstance(result, OutputResult)
        # XPT path should be set or None (depending on implementation)

    def test_generate_with_xml_format(self, file_generator, sample_request):
        """Test generate with XML format only."""
        sample_request.formats = {"xml"}
        result = file_generator.generate(sample_request)
        assert isinstance(result, OutputResult)
        # XML path should be set or None (depending on implementation)

    def test_generate_with_sas_format(self, file_generator, sample_request):
        """Test generate with SAS format only."""
        sample_request.formats = {"sas"}
        result = file_generator.generate(sample_request)
        assert isinstance(result, OutputResult)
        # SAS path should be set or None (depending on implementation)

    def test_generate_with_multiple_formats(self, file_generator, sample_request):
        """Test generate with multiple formats."""
        sample_request.formats = {"xpt", "xml", "sas"}
        result = file_generator.generate(sample_request)
        assert isinstance(result, OutputResult)
        # Multiple paths may be set (depending on implementation)

    def test_generate_with_empty_formats(self, file_generator, sample_request):
        """Test generate with no formats."""
        sample_request.formats = set()
        result = file_generator.generate(sample_request)
        assert isinstance(result, OutputResult)
        # No paths should be generated

    def test_generate_accepts_output_request(self, file_generator, sample_request):
        """Test that generate accepts OutputRequest parameter."""
        # Should not raise TypeError
        result = file_generator.generate(sample_request)
        assert result is not None
