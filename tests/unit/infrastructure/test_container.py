"""Tests for dependency injection container.

These tests verify the container correctly creates and wires up dependencies,
including singleton and transient patterns, configuration injection, and
testing overrides.
"""

import pytest
from rich.console import Console

from cdisc_transpiler.infrastructure import DependencyContainer, create_default_container
from cdisc_transpiler.infrastructure.io import CSVReader, FileGenerator
from cdisc_transpiler.infrastructure.logging import ConsoleLogger, NullLogger
from cdisc_transpiler.application.ports import LoggerPort, FileGeneratorPort


class MockLogger:
    """Mock logger for testing overrides."""
    
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


class TestDependencyContainer:
    """Tests for DependencyContainer class."""
    
    def test_create_container_with_defaults(self):
        """Test creating container with default configuration."""
        container = DependencyContainer()
        
        assert container.verbose == 0
        assert container.console is not None
        assert container.use_null_logger is False
    
    def test_create_container_with_verbose(self):
        """Test creating container with verbose mode."""
        container = DependencyContainer(verbose=2)
        
        assert container.verbose == 2
    
    def test_create_container_with_null_logger(self):
        """Test creating container with null logger enabled."""
        container = DependencyContainer(use_null_logger=True)
        
        assert container.use_null_logger is True
        logger = container.create_logger()
        assert isinstance(logger, NullLogger)
    
    def test_create_container_with_custom_console(self):
        """Test creating container with custom console."""
        custom_console = Console()
        container = DependencyContainer(console=custom_console)
        
        assert container.console is custom_console


class TestLoggerFactory:
    """Tests for logger factory method."""
    
    def test_create_logger_returns_console_logger(self):
        """Test that create_logger returns ConsoleLogger by default."""
        container = DependencyContainer()
        logger = container.create_logger()
        
        assert isinstance(logger, ConsoleLogger)
        assert isinstance(logger, LoggerPort)
    
    def test_create_logger_returns_null_logger_when_configured(self):
        """Test that create_logger returns NullLogger when configured."""
        container = DependencyContainer(use_null_logger=True)
        logger = container.create_logger()
        
        assert isinstance(logger, NullLogger)
        assert isinstance(logger, LoggerPort)
    
    def test_create_logger_is_singleton(self):
        """Test that create_logger returns the same instance (singleton)."""
        container = DependencyContainer()
        
        logger1 = container.create_logger()
        logger2 = container.create_logger()
        
        assert logger1 is logger2
    
    def test_create_logger_respects_verbose_level(self):
        """Test that logger is created with correct verbosity."""
        container = DependencyContainer(verbose=2)
        logger = container.create_logger()
        
        assert isinstance(logger, ConsoleLogger)
        # ConsoleLogger should have verbosity set
        assert hasattr(logger, 'verbosity')


class TestCSVReaderFactory:
    """Tests for CSV reader factory method."""
    
    def test_create_csv_reader_returns_instance(self):
        """Test that create_csv_reader returns CSVReader."""
        container = DependencyContainer()
        reader = container.create_csv_reader()
        
        assert isinstance(reader, CSVReader)
    
    def test_create_csv_reader_is_singleton(self):
        """Test that create_csv_reader returns the same instance."""
        container = DependencyContainer()
        
        reader1 = container.create_csv_reader()
        reader2 = container.create_csv_reader()
        
        assert reader1 is reader2


class TestFileGeneratorFactory:
    """Tests for file generator factory method."""
    
    def test_create_file_generator_returns_instance(self):
        """Test that create_file_generator returns FileGenerator."""
        container = DependencyContainer()
        generator = container.create_file_generator()
        
        assert isinstance(generator, FileGenerator)
        assert isinstance(generator, FileGeneratorPort)
    
    def test_create_file_generator_is_singleton(self):
        """Test that create_file_generator returns the same instance."""
        container = DependencyContainer()
        
        gen1 = container.create_file_generator()
        gen2 = container.create_file_generator()
        
        assert gen1 is gen2


class TestUseCaseFactories:
    """Tests for use case factory methods."""
    
    @pytest.mark.skip(reason="Circular import issue: services -> cli -> services")
    def test_create_study_processing_use_case(self):
        """Test creating study processing use case."""
        container = DependencyContainer(use_null_logger=True)
        
        # Import here to avoid circular import
        from cdisc_transpiler.application.study_processing_use_case import StudyProcessingUseCase
        
        use_case = container.create_study_processing_use_case()
        
        assert use_case is not None
        assert isinstance(use_case, StudyProcessingUseCase)
        assert hasattr(use_case, 'execute')
        assert hasattr(use_case, 'logger')
    
    @pytest.mark.skip(reason="Circular import issue: services -> cli -> services")
    def test_create_study_processing_use_case_is_transient(self):
        """Test that study use case factory returns new instances (transient)."""
        container = DependencyContainer(use_null_logger=True)
        
        use_case1 = container.create_study_processing_use_case()
        use_case2 = container.create_study_processing_use_case()
        
        # Should be different instances
        assert use_case1 is not use_case2
    
    def test_create_domain_processing_use_case(self):
        """Test creating domain processing use case."""
        container = DependencyContainer(use_null_logger=True)
        
        # Import here to avoid circular import
        from cdisc_transpiler.application.domain_processing_use_case import DomainProcessingUseCase
        
        use_case = container.create_domain_processing_use_case()
        
        assert use_case is not None
        assert isinstance(use_case, DomainProcessingUseCase)
        assert hasattr(use_case, 'execute')
        assert hasattr(use_case, 'logger')
    
    def test_create_domain_processing_use_case_is_transient(self):
        """Test that domain use case factory returns new instances (transient)."""
        container = DependencyContainer(use_null_logger=True)
        
        use_case1 = container.create_domain_processing_use_case()
        use_case2 = container.create_domain_processing_use_case()
        
        # Should be different instances
        assert use_case1 is not use_case2
    
    @pytest.mark.skip(reason="Circular import issue: services -> cli -> services")
    def test_use_cases_share_logger_singleton(self):
        """Test that use cases share the same logger instance."""
        container = DependencyContainer(use_null_logger=True)
        
        use_case1 = container.create_study_processing_use_case()
        use_case2 = container.create_domain_processing_use_case()
        
        # Should share the same logger instance
        assert use_case1.logger is use_case2.logger


class TestSingletonReset:
    """Tests for singleton reset functionality."""
    
    def test_reset_singletons(self):
        """Test that reset_singletons clears all cached instances."""
        container = DependencyContainer()
        
        # Create instances
        logger1 = container.create_logger()
        gen1 = container.create_file_generator()
        reader1 = container.create_csv_reader()
        
        # Reset
        container.reset_singletons()
        
        # Create new instances
        logger2 = container.create_logger()
        gen2 = container.create_file_generator()
        reader2 = container.create_csv_reader()
        
        # Should be different instances
        assert logger1 is not logger2
        assert gen1 is not gen2
        assert reader1 is not reader2


class TestOverrideMethods:
    """Tests for override methods for testing."""
    
    def test_override_logger(self):
        """Test overriding logger with custom implementation."""
        container = DependencyContainer()
        mock_logger = MockLogger()
        
        container.override_logger(mock_logger)
        logger = container.create_logger()
        
        assert logger is mock_logger
        
        # Test it works
        logger.info("Test message")
        assert len(mock_logger.messages) == 1
        assert mock_logger.messages[0] == ("info", "Test message")
    
    def test_override_file_generator(self):
        """Test overriding file generator with custom implementation."""
        container = DependencyContainer()
        
        class MockFileGenerator:
            def generate(self, request):
                return {"mock": True}
        
        mock_gen = MockFileGenerator()
        container.override_file_generator(mock_gen)
        generator = container.create_file_generator()
        
        assert generator is mock_gen
    
    @pytest.mark.skip(reason="Circular import issue: services -> cli -> services")
    def test_overridden_logger_used_by_use_cases(self):
        """Test that overridden logger is used by use cases."""
        container = DependencyContainer()
        mock_logger = MockLogger()
        container.override_logger(mock_logger)
        
        use_case = container.create_study_processing_use_case()
        
        assert use_case.logger is mock_logger


class TestConvenienceFunction:
    """Tests for convenience function."""
    
    def test_create_default_container(self):
        """Test creating container with convenience function."""
        container = create_default_container()
        
        assert isinstance(container, DependencyContainer)
        assert container.verbose == 0
    
    def test_create_default_container_with_verbose(self):
        """Test creating container with verbose mode."""
        container = create_default_container(verbose=1)
        
        assert container.verbose == 1
    
    def test_default_container_creates_functional_components(self):
        """Test that default container creates working components."""
        container = create_default_container()
        
        logger = container.create_logger()
        generator = container.create_file_generator()
        reader = container.create_csv_reader()
        
        assert logger is not None
        assert generator is not None
        assert reader is not None
